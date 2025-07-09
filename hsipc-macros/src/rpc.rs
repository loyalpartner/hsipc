//! RPC macro implementation for jsonrpsee-style RPC system
//!
//! This module implements the #[rpc] macro that generates server and client code
//! with support for async/sync modes and PendingSubscriptionSink pattern.

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, FnArg, ItemTrait, ReturnType, TraitItem, Type, Attribute};

/// Configuration for a method attribute
#[derive(Debug, Default)]
struct MethodConfig {
    pub name: Option<String>,
    pub timeout: Option<u64>,
    pub is_sync: bool,
}

/// Configuration for a subscription attribute
#[derive(Debug, Default)]
struct SubscriptionConfig {
    pub name: Option<String>,
    pub item_type: Option<String>,
    pub is_sync: bool,
}

/// Method type determined from attributes
#[derive(Debug)]
enum MethodType {
    Regular(MethodConfig),
    Subscription(SubscriptionConfig),
}

/// Extract parameter types from a function signature
fn extract_param_types(inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>) -> Vec<Type> {
    let mut param_types = Vec::new();
    for input in inputs {
        if let FnArg::Typed(pat_type) = input {
            param_types.push((*pat_type.ty).clone());
        }
        // Skip self parameters
    }
    param_types
}

/// Extract return type from function signature
fn extract_return_type(output: &ReturnType) -> Option<Type> {
    match output {
        ReturnType::Type(_, ty) => Some((**ty).clone()),
        ReturnType::Default => None,
    }
}

/// Extract the inner type and error type from Result<T> or Result<T, E>
fn extract_result_types(ty: &Type) -> Option<(Type, Option<Type>)> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Result" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        let error_type = if args.args.len() > 1 {
                            if let Some(syn::GenericArgument::Type(error_type)) = args.args.iter().nth(1) {
                                Some(error_type.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        return Some((inner_type.clone(), error_type));
                    }
                }
            }
        }
    }
    None
}

/// Extract the inner type from Result<T> (backward compatibility)
fn extract_result_inner_type(ty: &Type) -> Option<Type> {
    extract_result_types(ty).map(|(inner, _)| inner)
}

/// Check if type is already a Result type (including SubscriptionResult)
fn is_result_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Result" || segment.ident == "SubscriptionResult";
        }
    }
    false
}

/// RPC macro arguments configuration
#[derive(Debug, Default)]
struct RpcArgs {
    pub server: bool,
    pub client: bool,
    pub namespace: String,
    pub sync: bool,
}

/// Parse RPC macro arguments
fn parse_rpc_args(args: &str) -> RpcArgs {
    let mut config = RpcArgs::default();
    
    // Parse server flag
    config.server = args.contains("server");
    
    // Parse client flag
    config.client = args.contains("client");
    
    // Parse sync flag
    config.sync = args.contains("sync");
    
    // Parse namespace = "name"
    if let Some(start) = args.find("namespace = \"") {
        let start = start + 13; // length of "namespace = \""
        if let Some(end) = args[start..].find('"') {
            config.namespace = args[start..start + end].to_string();
        }
    } else {
        config.namespace = "default".to_string();
    }
    
    config
}

/// Parse method attribute arguments
fn parse_method_attribute(args_str: &str) -> MethodConfig {
    let mut config = MethodConfig::default();
    
    // Parse name = "method_name"
    if let Some(start) = args_str.find("name = \"") {
        let start = start + 8; // length of "name = \""
        if let Some(end) = args_str[start..].find('"') {
            config.name = Some(args_str[start..start + end].to_string());
        }
    }
    
    // Parse timeout = 1000
    if let Some(start) = args_str.find("timeout = ") {
        let start = start + 10; // length of "timeout = "
        let end = args_str[start..]
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(args_str[start..].len());
        if let Ok(timeout) = args_str[start..start + end].parse::<u64>() {
            config.timeout = Some(timeout);
        }
    }
    
    // Parse sync flag
    config.is_sync = args_str.contains("sync");
    
    config
}

/// Parse subscription attribute arguments
fn parse_subscription_attribute(args_str: &str) -> SubscriptionConfig {
    let mut config = SubscriptionConfig::default();
    
    // Parse name = "subscription_name"
    if let Some(start) = args_str.find("name = \"") {
        let start = start + 8; // length of "name = \""
        if let Some(end) = args_str[start..].find('"') {
            config.name = Some(args_str[start..start + end].to_string());
        }
    }
    
    // Parse item = Type
    if let Some(start) = args_str.find("item = ") {
        let start = start + 7; // length of "item = "
        let end = args_str[start..]
            .find(|c: char| c == ',' || c == ')')
            .unwrap_or(args_str[start..].len());
        config.item_type = Some(args_str[start..start + end].trim().to_string());
    }
    
    // Parse sync flag
    config.is_sync = args_str.contains("sync");
    
    config
}

/// Parse method attributes to determine method type and configuration
fn parse_method_attributes(attrs: &[Attribute]) -> Option<MethodType> {
    for attr in attrs {
        if attr.path().is_ident("method") {
            let args_str = attr.meta.to_token_stream().to_string();
            let config = parse_method_attribute(&args_str);
            return Some(MethodType::Regular(config));
        } else if attr.path().is_ident("subscription") {
            let args_str = attr.meta.to_token_stream().to_string();
            let config = parse_subscription_attribute(&args_str);
            return Some(MethodType::Subscription(config));
        }
    }
    None
}

/// Implementation of the #[rpc] macro
pub fn rpc_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemTrait);

    // Parse the macro arguments
    let args_str = args.to_string();
    let rpc_args = parse_rpc_args(&args_str);

    let trait_name = &input.ident;
    let service_name = syn::Ident::new(&format!("{trait_name}Service"), trait_name.span());
    let client_name = syn::Ident::new(&format!("{trait_name}Client"), trait_name.span());

    // Extract namespace for use in generated code
    let namespace = &rpc_args.namespace;
    
    // Check if we need async_trait - temporarily disabled to debug
    let needs_async_trait = false; // TODO: Fix async_trait support
    // let needs_async_trait = !rpc_args.sync && input.items.iter().any(|item| {
    //     if let TraitItem::Fn(method) = item {
    //         method.sig.asyncness.is_some()
    //     } else {
    //         false
    //     }
    // });
    
    // Extract method information from trait
    let mut method_names = Vec::new();
    let mut service_handlers = Vec::new();
    let mut client_methods = Vec::new();

    for item in &input.items {
        if let TraitItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();
            
            // Parse method attributes to determine configuration
            let method_type = parse_method_attributes(&method.attrs);
            
            // Determine the actual method name to use (from attribute or function name)
            let actual_method_name = match &method_type {
                Some(MethodType::Regular(config)) => {
                    config.name.as_ref().unwrap_or(&method_name_str).clone()
                }
                Some(MethodType::Subscription(config)) => {
                    config.name.as_ref().unwrap_or(&method_name_str).clone()
                }
                None => method_name_str.clone(),
            };
            
            method_names.push(actual_method_name.clone());

            // Extract parameter and return types
            let param_types = extract_param_types(&method.sig.inputs);
            let return_type = extract_return_type(&method.sig.output);
            
            // Check if the return type has a custom error
            let has_custom_error = if let Some(ref ret_type) = return_type {
                if let Some((_, Some(_))) = extract_result_types(ret_type) {
                    true
                } else {
                    false
                }
            } else {
                false
            };
            
            // Determine if this is a sync method from attributes
            let is_sync_method = match &method_type {
                Some(MethodType::Regular(config)) => config.is_sync,
                Some(MethodType::Subscription(config)) => config.is_sync,
                None => false,
            };
            
            // Use attribute sync flag or function asyncness to determine method type
            let is_async = method.sig.asyncness.is_some() && !is_sync_method;

            // Generate service handler for this method
            let handler = if is_async {
                // Async method - handle async_trait transformed methods
                if param_types.len() == 1 {
                    // Single parameter
                    let param_type = &param_types[0];
                    if has_custom_error {
                        quote! {
                            #actual_method_name => {
                                let request: #param_type = bincode::deserialize(&payload)?;
                                match self.inner.#method_name(request).await {
                                    Ok(result) => Ok(bincode::serialize(&result)?),
                                    Err(err) => {
                                        let serialized_err = bincode::serialize(&err).unwrap_or_default();
                                        Err(hsipc::Error::runtime_msg(format!("Custom error: {:?}", serialized_err)))
                                    },
                                }
                            }
                        }
                    } else {
                        quote! {
                            #actual_method_name => {
                                let request: #param_type = bincode::deserialize(&payload)?;
                                let response = self.inner.#method_name(request).await?;
                                Ok(bincode::serialize(&response)?)
                            }
                        }
                    }
                } else if param_types.is_empty() {
                    // No parameters - just call the method
                    if has_custom_error {
                        quote! {
                            #actual_method_name => {
                                let response = self.inner.#method_name().await;
                                match response {
                                    Ok(result) => Ok(bincode::serialize(&result)?),
                                    Err(err) => {
                                        let serialized_err = bincode::serialize(&err).unwrap_or_default();
                                        Err(hsipc::Error::runtime_msg(format!("Custom error: {:?}", serialized_err)))
                                    },
                                }
                            }
                        }
                    } else {
                        quote! {
                            #actual_method_name => {
                                let response = self.inner.#method_name().await?;
                                Ok(bincode::serialize(&response)?)
                            }
                        }
                    }
                } else {
                    // Multiple parameters - serialize as tuple
                    let param_count = param_types.len();
                    let param_names: Vec<syn::Ident> = (0..param_count)
                        .map(|i| syn::Ident::new(&format!("param_{}", i), proc_macro2::Span::call_site()))
                        .collect();
                    
                    if has_custom_error {
                        quote! {
                            #actual_method_name => {
                                let params: (#(#param_types),*) = bincode::deserialize(&payload)?;
                                let (#(#param_names),*) = params;
                                let response = self.inner.#method_name(#(#param_names),*).await;
                                match response {
                                    Ok(result) => Ok(bincode::serialize(&result)?),
                                    Err(err) => {
                                        let serialized_err = bincode::serialize(&err).unwrap_or_default();
                                        Err(hsipc::Error::runtime_msg(format!("Custom error: {:?}", serialized_err)))
                                    },
                                }
                            }
                        }
                    } else {
                        quote! {
                            #actual_method_name => {
                                let params: (#(#param_types),*) = bincode::deserialize(&payload)?;
                                let (#(#param_names),*) = params;
                                let response = self.inner.#method_name(#(#param_names),*).await?;
                                Ok(bincode::serialize(&response)?)
                            }
                        }
                    }
                }
            } else {
                // Sync method
                if param_types.len() == 1 {
                    let param_type = &param_types[0];
                    if has_custom_error {
                        quote! {
                            #actual_method_name => {
                                let request: #param_type = bincode::deserialize(&payload)?;
                                let response = self.inner.#method_name(request);
                                match response {
                                    Ok(result) => Ok(bincode::serialize(&result)?),
                                    Err(err) => {
                                        let serialized_err = bincode::serialize(&err).unwrap_or_default();
                                        Err(hsipc::Error::runtime_msg(format!("Custom error: {:?}", serialized_err)))
                                    },
                                }
                            }
                        }
                    } else {
                        quote! {
                            #actual_method_name => {
                                let request: #param_type = bincode::deserialize(&payload)?;
                                let response = self.inner.#method_name(request)?;
                                Ok(bincode::serialize(&response)?)
                            }
                        }
                    }
                } else if param_types.is_empty() {
                    // No parameters - just call the method
                    quote! {
                        #actual_method_name => {
                            let response = self.inner.#method_name()?;
                            Ok(bincode::serialize(&response)?)
                        }
                    }
                } else {
                    // Multiple parameters - serialize as tuple
                    let param_count = param_types.len();
                    let param_names: Vec<syn::Ident> = (0..param_count)
                        .map(|i| syn::Ident::new(&format!("param_{}", i), proc_macro2::Span::call_site()))
                        .collect();
                    
                    quote! {
                        #actual_method_name => {
                            let params: (#(#param_types),*) = bincode::deserialize(&payload)?;
                            let (#(#param_names),*) = params;
                            let response = self.inner.#method_name(#(#param_names),*)?;
                            Ok(bincode::serialize(&response)?)
                        }
                    }
                }
            };
            service_handlers.push(handler);

            // Generate client method
            let client_method = if method.sig.asyncness.is_some() {
                if param_types.len() == 1 {
                    let param_type = &param_types[0];
                    if let Some(return_type) = &return_type {
                        // Check if return type is already a Result type
                        if is_result_type(return_type) {
                            // If it's SubscriptionResult, keep it as is since it's already Result<()>
                            if return_type
                                .to_token_stream()
                                .to_string()
                                .contains("SubscriptionResult")
                            {
                                quote! {
                                    pub async fn #method_name(&self, request: #param_type) -> #return_type {
                                        let _result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), request).await?;
                                        Ok(())
                                    }
                                }
                            } else {
                                // For Result<T>, extract inner type and wrap in std::result::Result<T, hsipc::Error>
                                if let Some(inner_type) = extract_result_inner_type(return_type) {
                                    quote! {
                                        pub async fn #method_name(&self, request: #param_type) -> std::result::Result<#inner_type, hsipc::Error> {
                                            let result: #inner_type = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), request).await?;
                                            Ok(result)
                                        }
                                    }
                                } else {
                                    quote! {
                                        pub async fn #method_name(&self, request: #param_type) -> #return_type {
                                            let result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), request).await?;
                                            Ok(result)
                                        }
                                    }
                                }
                            }
                        } else {
                            quote! {
                                pub async fn #method_name(&self, request: #param_type) -> std::result::Result<#return_type, hsipc::Error> {
                                    let result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), request).await?;
                                    Ok(result)
                                }
                            }
                        }
                    } else {
                        quote! {
                            pub async fn #method_name(&self, request: #param_type) -> std::result::Result<(), hsipc::Error> {
                                let _result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), request).await?;
                                Ok(())
                            }
                        }
                    }
                } else if param_types.is_empty() {
                    // No parameters
                    if let Some(return_type) = &return_type {
                        // Check if return type is already a Result type
                        if is_result_type(return_type) {
                            // If it's SubscriptionResult, keep it as is since it's already Result<()>
                            if return_type
                                .to_token_stream()
                                .to_string()
                                .contains("SubscriptionResult")
                            {
                                quote! {
                                    pub async fn #method_name(&self) -> #return_type {
                                        let _result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), ()).await?;
                                        Ok(())
                                    }
                                }
                            } else {
                                // For Result<T>, extract inner type and wrap in std::result::Result<T, hsipc::Error>
                                if let Some(inner_type) = extract_result_inner_type(return_type) {
                                    quote! {
                                        pub async fn #method_name(&self) -> std::result::Result<#inner_type, hsipc::Error> {
                                            let result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), ()).await?;
                                            Ok(result)
                                        }
                                    }
                                } else {
                                    quote! {
                                        pub async fn #method_name(&self) -> #return_type {
                                            let result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), ()).await?;
                                            Ok(result)
                                        }
                                    }
                                }
                            }
                        } else {
                            quote! {
                                pub async fn #method_name(&self) -> std::result::Result<#return_type, hsipc::Error> {
                                    let result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), ()).await?;
                                    Ok(result)
                                }
                            }
                        }
                    } else {
                        quote! {
                            pub async fn #method_name(&self) -> std::result::Result<(), hsipc::Error> {
                                let _result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), ()).await?;
                                Ok(())
                            }
                        }
                    }
                } else {
                    // Multiple parameters - create tuple and call
                    let param_names: Vec<syn::Ident> = (0..param_types.len())
                        .map(|i| syn::Ident::new(&format!("param_{}", i), proc_macro2::Span::call_site()))
                        .collect();
                    
                    if let Some(return_type) = &return_type {
                        if is_result_type(return_type) {
                            if let Some(inner_type) = extract_result_inner_type(return_type) {
                                quote! {
                                    pub async fn #method_name(&self, #(#param_names: #param_types),*) -> std::result::Result<#inner_type, hsipc::Error> {
                                        let params = (#(#param_names),*);
                                        let result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), params).await?;
                                        Ok(result)
                                    }
                                }
                            } else {
                                quote! {
                                    pub async fn #method_name(&self, #(#param_names: #param_types),*) -> #return_type {
                                        let params = (#(#param_names),*);
                                        let _result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), params).await?;
                                        Ok(())
                                    }
                                }
                            }
                        } else {
                            quote! {
                                pub async fn #method_name(&self, #(#param_names: #param_types),*) -> std::result::Result<#return_type, hsipc::Error> {
                                    let params = (#(#param_names),*);
                                    let result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), params).await?;
                                    Ok(result)
                                }
                            }
                        }
                    } else {
                        quote! {
                            pub async fn #method_name(&self, #(#param_names: #param_types),*) -> std::result::Result<(), hsipc::Error> {
                                let params = (#(#param_names),*);
                                let _result = self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), params).await?;
                                Ok(())
                            }
                        }
                    }
                }
            } else {
                // Sync method - use runtime to block on async call
                if param_types.len() == 1 {
                    let param_type = &param_types[0];
                    if let Some(return_type) = &return_type {
                        // Check if return type is already a Result type
                        if is_result_type(return_type) {
                            // If it's SubscriptionResult, keep it as is since it's already Result<()>
                            if return_type
                                .to_token_stream()
                                .to_string()
                                .contains("SubscriptionResult")
                            {
                                quote! {
                                    pub fn #method_name(&self, request: #param_type) -> #return_type {
                                        // Use Handle::current() to avoid creating a new runtime in async context
                                        let _result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), request))?;
                                        Ok(())
                                    }
                                }
                            } else {
                                // For Result<T>, extract inner type and wrap in std::result::Result<T, hsipc::Error>
                                if let Some(inner_type) = extract_result_inner_type(return_type) {
                                    quote! {
                                        pub fn #method_name(&self, request: #param_type) -> std::result::Result<#inner_type, hsipc::Error> {
                                            // For sync methods, use futures::executor::block_on to avoid runtime issues
                                            let result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), request))?;
                                            Ok(result)
                                        }
                                    }
                                } else {
                                    quote! {
                                        pub fn #method_name(&self, request: #param_type) -> #return_type {
                                            // Use Handle::current() to avoid creating a new runtime in async context
                                            let result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), request))?;
                                            Ok(result)
                                        }
                                    }
                                }
                            }
                        } else {
                            quote! {
                                pub fn #method_name(&self, request: #param_type) -> std::result::Result<#return_type, hsipc::Error> {
                                    // Use Handle::current() to avoid creating a new runtime in async context
                                    let result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), request))?;
                                    Ok(result)
                                }
                            }
                        }
                    } else {
                        quote! {
                            pub fn #method_name(&self, request: #param_type) -> std::result::Result<(), hsipc::Error> {
                                // Use Handle::current() to avoid creating a new runtime in async context
                                let _result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), request))?;
                                Ok(())
                            }
                        }
                    }
                } else if param_types.is_empty() {
                    // No parameters
                    if let Some(return_type) = &return_type {
                        // Check if return type is already a Result type
                        if is_result_type(return_type) {
                            // If it's SubscriptionResult, keep it as is since it's already Result<()>
                            if return_type
                                .to_token_stream()
                                .to_string()
                                .contains("SubscriptionResult")
                            {
                                quote! {
                                    pub fn #method_name(&self) -> #return_type {
                                        // Use Handle::current() to avoid creating a new runtime in async context
                                        let _result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), ()))?;
                                        Ok(())
                                    }
                                }
                            } else {
                                // For Result<T>, extract inner type and wrap in std::result::Result<T, hsipc::Error>
                                if let Some(inner_type) = extract_result_inner_type(return_type) {
                                    quote! {
                                        pub fn #method_name(&self) -> std::result::Result<#inner_type, hsipc::Error> {
                                            // Use Handle::current() to avoid creating a new runtime in async context
                                            let result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), ()))?;
                                            Ok(result)
                                        }
                                    }
                                } else {
                                    quote! {
                                        pub fn #method_name(&self) -> #return_type {
                                            // Use Handle::current() to avoid creating a new runtime in async context
                                            let result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), ()))?;
                                            Ok(result)
                                        }
                                    }
                                }
                            }
                        } else {
                            quote! {
                                pub fn #method_name(&self) -> std::result::Result<#return_type, hsipc::Error> {
                                    // Use Handle::current() to avoid creating a new runtime in async context
                                    let result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), ()))?;
                                    Ok(result)
                                }
                            }
                        }
                    } else {
                        quote! {
                            pub fn #method_name(&self) -> std::result::Result<(), hsipc::Error> {
                                // Use Handle::current() to avoid creating a new runtime in async context
                                let _result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), ()))?;
                                Ok(())
                            }
                        }
                    }
                } else {
                    // Multiple parameters - create tuple and call
                    let param_names: Vec<syn::Ident> = (0..param_types.len())
                        .map(|i| syn::Ident::new(&format!("param_{}", i), proc_macro2::Span::call_site()))
                        .collect();
                    
                    if let Some(return_type) = &return_type {
                        if is_result_type(return_type) {
                            if let Some(inner_type) = extract_result_inner_type(return_type) {
                                quote! {
                                    pub fn #method_name(&self, #(#param_names: #param_types),*) -> std::result::Result<#inner_type, hsipc::Error> {
                                        let params = (#(#param_names),*);
                                        let result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), params))?;
                                        Ok(result)
                                    }
                                }
                            } else {
                                quote! {
                                    pub fn #method_name(&self, #(#param_names: #param_types),*) -> #return_type {
                                        let params = (#(#param_names),*);
                                        let _result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), params))?;
                                        Ok(())
                                    }
                                }
                            }
                        } else {
                            quote! {
                                pub fn #method_name(&self, #(#param_names: #param_types),*) -> std::result::Result<#return_type, hsipc::Error> {
                                    let params = (#(#param_names),*);
                                    let result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), params))?;
                                    Ok(result)
                                }
                            }
                        }
                    } else {
                        quote! {
                            pub fn #method_name(&self, #(#param_names: #param_types),*) -> std::result::Result<(), hsipc::Error> {
                                let params = (#(#param_names),*);
                                let _result = futures::executor::block_on(self.hub.call(&format!("{}.{}", #namespace, #actual_method_name), params))?;
                                Ok(())
                            }
                        }
                    }
                }
            };
            client_methods.push(client_method);
        }
    }

    // Generate the expanded code
    let trait_with_async = if needs_async_trait {
        quote! {
            #[hsipc::async_trait]
            #input
        }
    } else {
        quote! {
            #input
        }
    };
    
    let expanded = quote! {
        // Original trait with optional async_trait
        #trait_with_async

        // Generated service struct
        pub struct #service_name<T> {
            inner: T,
        }

        impl<T> #service_name<T>
        where
            T: #trait_name + Send + Sync,
        {
            pub fn new(inner: T) -> Self {
                Self { inner }
            }
        }

        // Implement the hsipc::Service trait
        #[hsipc::async_trait]
        impl<T> hsipc::Service for #service_name<T>
        where
            T: #trait_name + Send + Sync + 'static,
        {
            fn name(&self) -> &'static str {
                #namespace
            }

            fn methods(&self) -> Vec<&'static str> {
                vec![#(#method_names),*]
            }

            async fn handle(&self, method: &str, payload: Vec<u8>) -> std::result::Result<Vec<u8>, hsipc::Error> {
                match method {
                    #(#service_handlers)*
                    _ => Err(hsipc::Error::method_not_found(self.name(), method))
                }
            }
        }

        // Generated client struct
        #[derive(Clone)]
        pub struct #client_name {
            hub: hsipc::ProcessHub,
        }

        impl #client_name {
            pub fn new(hub: hsipc::ProcessHub) -> Self {
                Self { hub }
            }

            #(#client_methods)*
        }
    };

    TokenStream::from(expanded)
}
