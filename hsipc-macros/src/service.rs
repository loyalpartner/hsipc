//! Service macro implementation

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, FnArg, ImplItem, ItemImpl, ItemTrait, PatType, ReturnType, TraitItem, Type,
};

/// Implementation of the #[service_trait] attribute macro for traits
pub fn service_impl(_args: TokenStream, input: ItemTrait) -> TokenStream {
    let trait_name = &input.ident;
    let trait_methods = &input.items;

    // Extract method information with proper typing
    let mut method_info = Vec::new();
    for item in trait_methods {
        if let TraitItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();

            // Skip non-async methods
            if method.sig.asyncness.is_none() {
                continue;
            }

            // Extract parameter and return types from trait method
            let (param_type, return_type) = extract_trait_method_signature(method);
            method_info.push((
                method_name.clone(),
                method_name_str,
                param_type,
                return_type,
            ));
        }
    }

    // Generate client struct
    let client_name = quote::format_ident!("{}Client", trait_name);
    let service_name = trait_name.to_string();

    // Generate typed client methods
    let client_methods = method_info.iter().map(|(method_name, method_name_str, param_type, return_type)| {
        let full_method_name = format!("{service_name}.{method_name_str}");

        quote! {
            pub async fn #method_name(&self, request: #param_type) -> ::hsipc::Result<#return_type> {
                self.hub.call(#full_method_name, request).await
            }
        }
    });

    // Generate the complete output
    let expanded = quote! {
        #input

        /// Auto-generated client for the trait-based service
        #[derive(Clone)]
        pub struct #client_name {
            hub: ::hsipc::ProcessHub,
        }

        impl #client_name {
            /// Create a new client
            pub async fn new(client_name: &str) -> ::hsipc::Result<Self> {
                let hub = ::hsipc::ProcessHub::new(client_name).await?;
                Ok(Self { hub })
            }

            /// Connect to a service (alias for new)
            pub async fn connect(server_name: &str) -> ::hsipc::Result<Self> {
                Self::new(server_name).await
            }

            #(#client_methods)*
        }
    };

    TokenStream::from(expanded)
}

/// Implementation of the #[service_impl] attribute macro for implementations
pub fn service_impl_impl(_args: TokenStream, input: ItemImpl) -> TokenStream {
    let impl_block = &input;
    let impl_type = &input.self_ty;

    // Extract trait name if this is a trait impl
    let trait_name = if let Some((_, trait_path, _)) = &input.trait_ {
        if let Some(segment) = trait_path.segments.last() {
            segment.ident.to_string()
        } else {
            return syn::Error::new_spanned(trait_path, "Unable to extract trait name")
                .to_compile_error()
                .into();
        }
    } else {
        return syn::Error::new_spanned(impl_type, "Service implementation must be for a trait")
            .to_compile_error()
            .into();
    };

    // Extract method information with full signatures
    let mut method_info = Vec::new();
    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();

            // Skip non-async methods
            if method.sig.asyncness.is_none() {
                continue;
            }

            // Extract parameter and return types
            let (param_type, return_type) = extract_method_signature(method);
            method_info.push((method_name_str, param_type, return_type));
        }
    }

    // Generate service wrapper with unique name based on implementation type
    let impl_type_name = match impl_type.as_ref() {
        Type::Path(path) => {
            if let Some(segment) = path.path.segments.last() {
                segment.ident.to_string()
            } else {
                "Unknown".to_string()
            }
        }
        _ => "Unknown".to_string(),
    };
    let service_wrapper_name = quote::format_ident!("{}Service", impl_type_name);
    let service_wrapper =
        generate_trait_service_wrapper(&trait_name, impl_type, &service_wrapper_name, &method_info);

    let expanded = quote! {
        #impl_block

        #service_wrapper
    };

    TokenStream::from(expanded)
}

/// Generate the Service trait implementation
#[allow(dead_code)]
fn generate_service_impl(
    trait_name: &str,
    impl_type: &Type,
    methods: &[String],
) -> proc_macro2::TokenStream {
    let _method_names: Vec<_> = methods
        .iter()
        .map(|name| format!("{trait_name}.{name}"))
        .collect();
    let method_handlers: Vec<_> = methods.iter().map(|name| {
        let method_ident = quote::format_ident!("{}", name);
        quote! {
            #name => {
                // For now, assume all methods take a tuple of parameters
                let params: _ = ::hsipc::bincode::deserialize(&payload)
                    .map_err(|e| ::hsipc::Error::Serialization(e))?;
                let result = self.#method_ident(params).await?;
                ::hsipc::bincode::serialize(&result).map_err(|e| ::hsipc::Error::Serialization(e))
            }
        }
    }).collect();

    quote! {
        #[::hsipc::async_trait]
        impl ::hsipc::Service for #impl_type {
            fn name(&self) -> &'static str {
                #trait_name
            }

            fn methods(&self) -> Vec<&'static str> {
                vec![#(#methods),*]
            }

            async fn handle(&self, method: &str, payload: Vec<u8>) -> ::hsipc::Result<Vec<u8>> {
                match method {
                    #(#method_handlers)*
                    _ => Err(::hsipc::Error::MethodNotFound(method.to_string())),
                }
            }
        }
    }
}

/// New implementation for #[service] on impl blocks
pub fn service_impl_new(_args: TokenStream, input: ItemImpl) -> TokenStream {
    let impl_block = &input;
    let impl_type = &input.self_ty;

    // Extract the service name from the impl type
    let service_name = match impl_type.as_ref() {
        Type::Path(path) => {
            if let Some(segment) = path.path.segments.last() {
                segment.ident.to_string()
            } else {
                return syn::Error::new_spanned(impl_type, "Cannot extract service name")
                    .to_compile_error()
                    .into();
            }
        }
        _ => {
            return syn::Error::new_spanned(
                impl_type,
                "Service must be implemented for a named struct",
            )
            .to_compile_error()
            .into();
        }
    };

    // Extract methods from the impl block
    let mut methods = Vec::new();
    let mut method_handlers = Vec::new();

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();

            // Skip non-async methods for now
            if method.sig.asyncness.is_none() {
                continue;
            }

            // Extract request and response types from method signature
            let (request_type, _response_type) = extract_method_types(method);

            methods.push(method_name_str.clone());

            // Generate handler for this method
            let handler = quote! {
                #method_name_str => {
                    let req: #request_type = ::hsipc::bincode::deserialize(&payload)
                        .map_err(|e| ::hsipc::Error::Serialization(e))?;
                    let resp = self.inner.#method_name(req).await?;
                    ::hsipc::bincode::serialize(&resp)
                        .map_err(|e| ::hsipc::Error::Serialization(e))
                }
            };

            method_handlers.push(handler);
        }
    }

    // Generate the service wrapper
    let service_wrapper_name = quote::format_ident!("{}Service", service_name);
    let client_name = quote::format_ident!("{}Client", service_name);

    // Generate client methods
    let client_methods = input.items.iter().filter_map(|item| {
        if let ImplItem::Fn(method) = item {
            if method.sig.asyncness.is_some() {
                let method_name = &method.sig.ident;
                let method_name_str = method_name.to_string();
                let full_method_name = format!("{service_name}.{method_name_str}");
                let (request_type, response_type) = extract_method_types(method);

                // Generate client method that calls the full service.method name
                Some(quote! {
                    pub async fn #method_name(&self, req: #request_type) -> ::hsipc::Result<#response_type> {
                        self.hub.call(#full_method_name, req).await
                    }
                })
            } else {
                None
            }
        } else {
            None
        }
    }).collect::<Vec<_>>();

    let expanded = quote! {
        #impl_block

        /// Auto-generated service wrapper
        pub struct #service_wrapper_name {
            inner: #impl_type,
        }

        impl #service_wrapper_name {
            pub fn new(inner: #impl_type) -> Self {
                Self { inner }
            }
        }

        #[::hsipc::async_trait]
        impl ::hsipc::Service for #service_wrapper_name {
            fn name(&self) -> &'static str {
                #service_name
            }

            fn methods(&self) -> Vec<&'static str> {
                vec![#(#methods),*]
            }

            async fn handle(&self, method: &str, payload: Vec<u8>) -> ::hsipc::Result<Vec<u8>> {
                match method {
                    #(#method_handlers)*
                    _ => Err(::hsipc::Error::MethodNotFound(method.to_string())),
                }
            }
        }

        /// Auto-generated client
        pub struct #client_name {
            hub: ::hsipc::ProcessHub,
        }

        impl #client_name {
            pub async fn new(client_name: &str) -> ::hsipc::Result<Self> {
                let hub = ::hsipc::ProcessHub::new(client_name).await?;
                Ok(Self { hub })
            }

            #(#client_methods)*
        }
    };

    TokenStream::from(expanded)
}

/// Extract request and response types from a method signature
fn extract_method_types(
    method: &syn::ImplItemFn,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    // For now, assume the first parameter (after &self) is the request type
    // and the return type is Result<ResponseType>

    let request_type = method
        .sig
        .inputs
        .iter()
        .nth(1)
        .and_then(|arg| {
            if let FnArg::Typed(PatType { ty, .. }) = arg {
                Some(ty.clone())
            } else {
                None
            }
        })
        .map(|ty| quote! { #ty })
        .unwrap_or_else(|| quote! { () });

    let response_type = match &method.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => {
            // Try to extract T from Result<T>
            if let Type::Path(type_path) = ty.as_ref() {
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == "Result" {
                        // Extract the type from Result<T>
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                            {
                                return (request_type, quote! { #inner_type });
                            }
                        }
                    }
                }
            }
            // Fallback to the full type
            quote! { #ty }
        }
    };

    (request_type, response_type)
}

/// Extract method signature for trait implementations
fn extract_method_signature(
    method: &syn::ImplItemFn,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    // Extract parameter type (skip &self)
    let param_type = method
        .sig
        .inputs
        .iter()
        .nth(1)
        .and_then(|arg| {
            if let FnArg::Typed(PatType { ty, .. }) = arg {
                Some(ty.clone())
            } else {
                None
            }
        })
        .map(|ty| quote! { #ty })
        .unwrap_or_else(|| quote! { () });

    // Extract return type from Result<T>
    let return_type = match &method.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => {
            // Try to extract T from Result<T>
            if let Type::Path(type_path) = ty.as_ref() {
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == "Result" {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                            {
                                return (param_type, quote! { #inner_type });
                            }
                        }
                    }
                }
            }
            quote! { #ty }
        }
    };

    (param_type, return_type)
}

/// Extract method signature for trait method definitions
fn extract_trait_method_signature(
    method: &syn::TraitItemFn,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    // Extract parameter type (skip &self)
    let param_type = method
        .sig
        .inputs
        .iter()
        .nth(1)
        .and_then(|arg| {
            if let FnArg::Typed(PatType { ty, .. }) = arg {
                Some(ty.clone())
            } else {
                None
            }
        })
        .map(|ty| quote! { #ty })
        .unwrap_or_else(|| quote! { () });

    // Extract return type from Result<T>
    let return_type = match &method.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => {
            // Try to extract T from Result<T>
            if let Type::Path(type_path) = ty.as_ref() {
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == "Result" {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                            {
                                return (param_type, quote! { #inner_type });
                            }
                        }
                    }
                }
            }
            quote! { #ty }
        }
    };

    (param_type, return_type)
}

/// Generate Service trait implementation for trait-based services
#[allow(dead_code)]
fn generate_trait_service_impl(
    trait_name: &str,
    impl_type: &Type,
    method_info: &[(String, proc_macro2::TokenStream, proc_macro2::TokenStream)],
) -> proc_macro2::TokenStream {
    let method_handlers: Vec<_> = method_info
        .iter()
        .map(|(method_name, param_type, _return_type)| {
            let method_ident = quote::format_ident!("{}", method_name);
            quote! {
                #method_name => {
                    let params: #param_type = ::hsipc::bincode::deserialize(&payload)
                        .map_err(|e| ::hsipc::Error::Serialization(e))?;
                    let result = self.#method_ident(params).await?;
                    ::hsipc::bincode::serialize(&result)
                        .map_err(|e| ::hsipc::Error::Serialization(e))
                }
            }
        })
        .collect();

    let method_names: Vec<_> = method_info.iter().map(|(name, _, _)| name).collect();

    quote! {
        #[::hsipc::async_trait]
        impl ::hsipc::Service for #impl_type {
            fn name(&self) -> &'static str {
                #trait_name
            }

            fn methods(&self) -> Vec<&'static str> {
                vec![#(#method_names),*]
            }

            async fn handle(&self, method: &str, payload: Vec<u8>) -> ::hsipc::Result<Vec<u8>> {
                match method {
                    #(#method_handlers)*
                    _ => Err(::hsipc::Error::MethodNotFound(method.to_string())),
                }
            }
        }
    }
}

/// Generate service wrapper for trait-based services
fn generate_trait_service_wrapper(
    trait_name: &str,
    impl_type: &Type,
    wrapper_name: &syn::Ident,
    method_info: &[(String, proc_macro2::TokenStream, proc_macro2::TokenStream)],
) -> proc_macro2::TokenStream {
    let method_handlers: Vec<_> = method_info
        .iter()
        .map(|(method_name, param_type, _return_type)| {
            let method_ident = quote::format_ident!("{}", method_name);
            quote! {
                #method_name => {
                    let params: #param_type = ::hsipc::bincode::deserialize(&payload)
                        .map_err(|e| ::hsipc::Error::Serialization(e))?;
                    let result = self.inner.#method_ident(params).await?;
                    ::hsipc::bincode::serialize(&result)
                        .map_err(|e| ::hsipc::Error::Serialization(e))
                }
            }
        })
        .collect();

    let method_names: Vec<_> = method_info.iter().map(|(name, _, _)| name).collect();

    quote! {
        /// Auto-generated service wrapper for trait-based service
        pub struct #wrapper_name {
            inner: #impl_type,
        }

        impl #wrapper_name {
            pub fn new(inner: #impl_type) -> Self {
                Self { inner }
            }
        }

        #[::hsipc::async_trait]
        impl ::hsipc::Service for #wrapper_name {
            fn name(&self) -> &'static str {
                #trait_name
            }

            fn methods(&self) -> Vec<&'static str> {
                vec![#(#method_names),*]
            }

            async fn handle(&self, method: &str, payload: Vec<u8>) -> ::hsipc::Result<Vec<u8>> {
                match method {
                    #(#method_handlers)*
                    _ => Err(::hsipc::Error::MethodNotFound(method.to_string())),
                }
            }
        }
    }
}

/// Derive macro implementation for Service
pub fn derive_service_impl(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let _service_name = name.to_string();

    let expanded = quote! {
        impl #name {
            pub fn into_service(self) -> impl ::hsipc::Service {
                #name::Service::new(self)
            }
        }
    };

    TokenStream::from(expanded)
}
