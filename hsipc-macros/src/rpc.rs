//! Simplified RPC macro implementation with subscription support
//!
//! This is a clean rewrite focusing on the essential functionality.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, FnArg, ItemTrait, ReturnType, TraitItem, Type};

/// Parse RPC macro arguments
fn parse_rpc_args(args: &str) -> RpcConfig {
    let server = args.contains("server");
    let client = args.contains("client");
    let mut config = RpcConfig {
        server,
        client,
        ..Default::default()
    };

    // Parse namespace
    if let Some(start) = args.find("namespace = \"") {
        let start = start + 13;
        if let Some(end) = args[start..].find('"') {
            config.namespace = args[start..start + end].to_string();
        }
    }

    config
}

/// Extract the inner type from Result<T>
fn extract_result_inner_type(return_type: Option<&Type>) -> proc_macro2::TokenStream {
    match return_type {
        Some(Type::Path(type_path)) => {
            // Check if it's Result<T>
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Result" {
                    // Extract T from Result<T>
                    match &segment.arguments {
                        syn::PathArguments::AngleBracketed(args) => {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                            {
                                quote! { #inner_type }
                            } else {
                                quote! { () }
                            }
                        }
                        _ => quote! { () },
                    }
                } else {
                    quote! { #return_type }
                }
            } else {
                quote! { () }
            }
        }
        _ => quote! { () },
    }
}

#[derive(Default)]
#[allow(dead_code)]
struct RpcConfig {
    server: bool,
    client: bool,
    namespace: String,
}

/// Method type enumeration
#[derive(Debug, PartialEq)]
enum MethodType {
    Method,       // Regular RPC method
    Subscription, // Subscription method
}

/// Parse method attributes to extract method type and RPC name
fn parse_method_attributes(attrs: &[Attribute], default_name: &str) -> (MethodType, String) {
    for attr in attrs {
        if attr.path().is_ident("method") {
            // Parse #[method(name = "...")]
            if let Ok(meta) = attr.meta.require_list() {
                let tokens = meta.tokens.to_string();
                if let Some(start) = tokens.find("name = \"") {
                    let start = start + 8; // length of "name = \""
                    if let Some(end) = tokens[start..].find('"') {
                        let method_name = tokens[start..start + end].to_string();
                        return (MethodType::Method, method_name);
                    }
                }
            }
            return (MethodType::Method, default_name.to_string());
        } else if attr.path().is_ident("subscription") {
            // Parse #[subscription(name = "...")]
            if let Ok(meta) = attr.meta.require_list() {
                let tokens = meta.tokens.to_string();
                if let Some(start) = tokens.find("name = \"") {
                    let start = start + 8; // length of "name = \""
                    if let Some(end) = tokens[start..].find('"') {
                        let subscription_name = tokens[start..start + end].to_string();
                        return (MethodType::Subscription, subscription_name);
                    }
                }
            }
            return (MethodType::Subscription, default_name.to_string());
        }
    }
    // Default to method if no attribute found
    (MethodType::Method, default_name.to_string())
}

/// Generate handler for subscription methods
fn generate_subscription_handler(
    _method_name: &syn::Ident,
    rpc_method_name: &str,
) -> proc_macro2::TokenStream {
    quote! {
        #rpc_method_name => {
            // Subscription methods are handled through the subscription protocol
            // not through regular RPC calls
            Err(hsipc::Error::method_not_found(self.name(), method))
        }
    }
}

/// Generate handler for subscription method in handle_subscription
fn generate_subscription_method_handler(
    method_name: &syn::Ident,
    rpc_method_name: &str,
    params: &[&Type],
) -> proc_macro2::TokenStream {
    if params.len() == 1 {
        let param_type = params[0];
        quote! {
            #rpc_method_name => {
                let params_value: #param_type = bincode::deserialize(&params)?;
                self.inner.#method_name(pending, params_value).await?;
                Ok(())
            }
        }
    } else if params.is_empty() {
        quote! {
            #rpc_method_name => {
                self.inner.#method_name(pending).await?;
                Ok(())
            }
        }
    } else {
        // Multiple parameters - serialize as tuple
        let param_tuple = quote! { (#(#params),*) };
        quote! {
            #rpc_method_name => {
                let params_tuple: #param_tuple = bincode::deserialize(&params)?;
                self.inner.#method_name(pending, params_tuple.0, params_tuple.1).await?;
                Ok(())
            }
        }
    }
}

/// Generate handler for regular RPC methods
fn generate_method_handler(
    method_name: &syn::Ident,
    rpc_method_name: &str,
    params: &[&Type],
    is_async: bool,
) -> proc_macro2::TokenStream {
    if params.len() == 1 {
        let param_type = params[0];
        if is_async {
            quote! {
                #rpc_method_name => {
                    let request: #param_type = bincode::deserialize(&payload)?;
                    let response = self.inner.#method_name(request).await?;
                    Ok(bincode::serialize(&response)?)
                }
            }
        } else {
            quote! {
                #rpc_method_name => {
                    let request: #param_type = bincode::deserialize(&payload)?;
                    let response = self.inner.#method_name(request)?;
                    Ok(bincode::serialize(&response)?)
                }
            }
        }
    } else if params.is_empty() {
        if is_async {
            quote! {
                #rpc_method_name => {
                    let response = self.inner.#method_name().await?;
                    Ok(bincode::serialize(&response)?)
                }
            }
        } else {
            quote! {
                #rpc_method_name => {
                    let response = self.inner.#method_name()?;
                    Ok(bincode::serialize(&response)?)
                }
            }
        }
    } else {
        // Multiple parameters - serialize as tuple
        let param_tuple = quote! { (#(#params),*) };
        if is_async {
            quote! {
                #rpc_method_name => {
                    let params: #param_tuple = bincode::deserialize(&payload)?;
                    let response = self.inner.#method_name(params.0, params.1).await?;
                    Ok(bincode::serialize(&response)?)
                }
            }
        } else {
            quote! {
                #rpc_method_name => {
                    let params: #param_tuple = bincode::deserialize(&payload)?;
                    let response = self.inner.#method_name(params.0, params.1)?;
                    Ok(bincode::serialize(&response)?)
                }
            }
        }
    }
}

/// Generate client method for regular RPC calls
fn generate_rpc_client_method(
    method_name: &syn::Ident,
    rpc_method_name: &str,
    params: &[&Type],
    client_return_type: &proc_macro2::TokenStream,
    namespace: &str,
    is_async: bool,
) -> proc_macro2::TokenStream {
    if params.len() == 1 {
        let param_type = params[0];
        if is_async {
            quote! {
                pub async fn #method_name(&self, request: #param_type) -> hsipc::Result<#client_return_type> {
                    let result: #client_return_type = self.hub.call(&format!("{}.{}", #namespace, #rpc_method_name), request).await?;
                    Ok(result)
                }
            }
        } else {
            quote! {
                pub fn #method_name(&self, request: #param_type) -> hsipc::Result<#client_return_type> {
                    let result: #client_return_type = futures::executor::block_on(
                        self.hub.call(&format!("{}.{}", #namespace, #rpc_method_name), request)
                    )?;
                    Ok(result)
                }
            }
        }
    } else if params.is_empty() {
        if is_async {
            quote! {
                pub async fn #method_name(&self) -> hsipc::Result<#client_return_type> {
                    let result: #client_return_type = self.hub.call(&format!("{}.{}", #namespace, #rpc_method_name), ()).await?;
                    Ok(result)
                }
            }
        } else {
            quote! {
                pub fn #method_name(&self) -> hsipc::Result<#client_return_type> {
                    let result: #client_return_type = futures::executor::block_on(
                        self.hub.call(&format!("{}.{}", #namespace, #rpc_method_name), ())
                    )?;
                    Ok(result)
                }
            }
        }
    } else {
        // Multiple parameters
        let param_names: Vec<syn::Ident> = (0..params.len())
            .map(|i| syn::Ident::new(&format!("p{i}"), method_name.span()))
            .collect();

        if is_async {
            quote! {
                pub async fn #method_name(&self, #(#param_names: #params),*) -> hsipc::Result<#client_return_type> {
                    let params = (#(#param_names),*);
                    let result: #client_return_type = self.hub.call(&format!("{}.{}", #namespace, #rpc_method_name), params).await?;
                    Ok(result)
                }
            }
        } else {
            quote! {
                pub fn #method_name(&self, #(#param_names: #params),*) -> hsipc::Result<#client_return_type> {
                    let params = (#(#param_names),*);
                    let result: #client_return_type = futures::executor::block_on(
                        self.hub.call(&format!("{}.{}", #namespace, #rpc_method_name), params)
                    )?;
                    Ok(result)
                }
            }
        }
    }
}

/// Transform trait to add PendingSubscriptionSink parameters to subscription methods
fn transform_trait_for_subscription(input: &ItemTrait) -> proc_macro2::TokenStream {
    let trait_ident = &input.ident;
    let trait_generics = &input.generics;
    let trait_bounds = &input.supertraits;

    let mut transformed_items = Vec::new();

    for item in &input.items {
        if let TraitItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();

            // Check if this is a subscription method
            let (method_type, _) = parse_method_attributes(&method.attrs, &method_name_str);

            if method_type == MethodType::Subscription {
                // Transform subscription method to include PendingSubscriptionSink
                let mut transformed_method = method.clone();

                // Insert PendingSubscriptionSink parameter after &self
                let mut new_inputs = syn::punctuated::Punctuated::new();

                // Add &self parameter
                if let Some(first_input) = transformed_method.sig.inputs.first() {
                    new_inputs.push(first_input.clone());
                }

                // Add PendingSubscriptionSink parameter
                let pending_param: syn::FnArg =
                    syn::parse_str("pending: hsipc::PendingSubscriptionSink").unwrap();
                new_inputs.push(pending_param);

                // Add remaining parameters
                for input in transformed_method.sig.inputs.iter().skip(1) {
                    new_inputs.push(input.clone());
                }

                transformed_method.sig.inputs = new_inputs;
                transformed_items.push(TraitItem::Fn(transformed_method));
            } else {
                // Keep non-subscription methods unchanged
                transformed_items.push(item.clone());
            }
        } else {
            // Keep non-function items unchanged
            transformed_items.push(item.clone());
        }
    }

    quote! {
        pub trait #trait_ident #trait_generics: #trait_bounds {
            #(#transformed_items)*
        }
    }
}

/// Generate client method for subscription calls
fn generate_subscription_client_method(
    method_name: &syn::Ident,
    rpc_method_name: &str,
    params: &[&Type],
    _namespace: &str,
    _return_type: Option<&Type>,
) -> proc_macro2::TokenStream {
    // Generate subscription client method that sends subscription request
    if params.len() == 1 {
        let param_type = params[0];
        quote! {
            pub async fn #method_name(&self, params: #param_type) -> hsipc::Result<hsipc::RpcSubscription<hsipc::serde_json::Value>> {
                // Serialize parameters
                let serialized_params = bincode::serialize(&params)?;

                // Send subscription request
                let request_msg = hsipc::Message::subscription_request(
                    self.hub.name().to_string(),
                    None, // Broadcast to all processes
                    format!("subscription.{}", #rpc_method_name),
                    serialized_params,
                );

                // Get the subscription ID from the message
                let subscription_id = request_msg.correlation_id.unwrap();
                
                println!("📡 Sending subscription request: method={}, id={}", #rpc_method_name, subscription_id);

                // Create channel for receiving subscription data
                let (tx, rx) = hsipc::tokio::sync::mpsc::unbounded_channel();

                // Register the subscription with the hub for data forwarding
                self.hub.register_subscription(subscription_id, tx).await;

                // Create the RPC subscription with hub for cancellation support
                let subscription = hsipc::RpcSubscription::new_with_hub(subscription_id, rx, self.hub.clone());

                // Actually send the subscription request
                println!("🚀 About to send subscription request to hub...");
                let send_result = self.hub.send_message(request_msg).await;
                match &send_result {
                    Ok(()) => println!("✅ Subscription request sent successfully"),
                    Err(e) => println!("❌ Failed to send subscription request: {}", e),
                }
                send_result?;

                Ok(subscription)
            }
        }
    } else if params.is_empty() {
        quote! {
            pub async fn #method_name(&self) -> hsipc::Result<hsipc::RpcSubscription<hsipc::serde_json::Value>> {
                // Send subscription request with no parameters
                let request_msg = hsipc::Message::subscription_request(
                    self.hub.name().to_string(),
                    None, // Broadcast to all processes
                    format!("subscription.{}", #rpc_method_name),
                    vec![], // No parameters
                );

                // Get the subscription ID from the message
                let subscription_id = request_msg.correlation_id.unwrap();

                // Create channel for receiving subscription data
                let (tx, rx) = hsipc::tokio::sync::mpsc::unbounded_channel();

                // Register the subscription with the hub for data forwarding
                self.hub.register_subscription(subscription_id, tx).await;

                // Create the RPC subscription with hub for cancellation support
                let subscription = hsipc::RpcSubscription::new_with_hub(subscription_id, rx, self.hub.clone());

                // Actually send the subscription request
                println!("🚀 About to send subscription request to hub...");
                let send_result = self.hub.send_message(request_msg).await;
                match &send_result {
                    Ok(()) => println!("✅ Subscription request sent successfully"),
                    Err(e) => println!("❌ Failed to send subscription request: {}", e),
                }
                send_result?;

                Ok(subscription)
            }
        }
    } else {
        // Multiple parameters
        let param_names: Vec<syn::Ident> = (0..params.len())
            .map(|i| syn::Ident::new(&format!("p{i}"), method_name.span()))
            .collect();

        quote! {
            pub async fn #method_name(&self, #(#param_names: #params),*) -> hsipc::Result<hsipc::RpcSubscription<hsipc::serde_json::Value>> {
                // Serialize parameters as tuple
                let params_tuple = (#(#param_names),*);
                let serialized_params = bincode::serialize(&params_tuple)?;

                // Send subscription request
                let request_msg = hsipc::Message::subscription_request(
                    self.hub.name().to_string(),
                    None, // Broadcast to all processes
                    format!("subscription.{}", #rpc_method_name),
                    serialized_params,
                );

                // Get the subscription ID from the message
                let subscription_id = request_msg.correlation_id.unwrap();
                
                println!("📡 Sending subscription request: method={}, id={}", #rpc_method_name, subscription_id);

                // Create channel for receiving subscription data
                let (tx, rx) = hsipc::tokio::sync::mpsc::unbounded_channel();

                // Register the subscription with the hub for data forwarding
                self.hub.register_subscription(subscription_id, tx).await;

                // Create the RPC subscription with hub for cancellation support
                let subscription = hsipc::RpcSubscription::new_with_hub(subscription_id, rx, self.hub.clone());

                // Actually send the subscription request
                println!("🚀 About to send subscription request to hub...");
                let send_result = self.hub.send_message(request_msg).await;
                match &send_result {
                    Ok(()) => println!("✅ Subscription request sent successfully"),
                    Err(e) => println!("❌ Failed to send subscription request: {}", e),
                }
                send_result?;

                Ok(subscription)
            }
        }
    }
}

/// RPC macro implementation
pub fn rpc_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemTrait);
    let args_str = args.to_string();
    let config = parse_rpc_args(&args_str);

    let trait_name = &input.ident;
    let service_name = syn::Ident::new(&format!("{trait_name}Service"), trait_name.span());
    let client_name = syn::Ident::new(&format!("{trait_name}Client"), trait_name.span());

    let namespace = &config.namespace;

    // Extract methods from trait
    let mut method_names = Vec::new();
    let mut service_handlers = Vec::new();
    let mut client_methods = Vec::new();
    let mut subscription_handlers = Vec::new();

    for item in &input.items {
        if let TraitItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();

            // Parse method attributes to determine type and RPC name
            let (method_type, rpc_method_name) =
                parse_method_attributes(&method.attrs, &method_name_str);
            method_names.push(rpc_method_name.clone());

            // Extract parameters (skip &self)
            let params: Vec<&Type> = method
                .sig
                .inputs
                .iter()
                .filter_map(|arg| match arg {
                    FnArg::Typed(pat_type) => Some(&*pat_type.ty),
                    _ => None,
                })
                .collect();

            // Extract return type
            let return_type = match &method.sig.output {
                ReturnType::Type(_, ty) => Some(&**ty),
                ReturnType::Default => None,
            };

            // Check if method is async
            let is_async = method.sig.asyncness.is_some();

            // Generate service handler based on method type
            let handler = match method_type {
                MethodType::Subscription => {
                    // For subscription methods, we need special handling
                    // These are handled through the subscription protocol, not regular RPC
                    let sub_handler = generate_subscription_method_handler(method_name, &rpc_method_name, &params);
                    subscription_handlers.push(sub_handler);
                    
                    // Still generate the regular handler to reject RPC calls to subscription methods
                    generate_subscription_handler(method_name, &rpc_method_name)
                }
                MethodType::Method => {
                    // Regular method handling
                    generate_method_handler(method_name, &rpc_method_name, &params, is_async)
                }
            };
            service_handlers.push(handler);

            // Generate client method
            let client_method = match method_type {
                MethodType::Subscription => {
                    // Generate subscription client method
                    generate_subscription_client_method(
                        method_name,
                        &rpc_method_name,
                        &params,
                        namespace,
                        return_type,
                    )
                }
                MethodType::Method => {
                    // Generate regular RPC client method
                    let client_return_type = extract_result_inner_type(return_type);
                    generate_rpc_client_method(
                        method_name,
                        &rpc_method_name,
                        &params,
                        &client_return_type,
                        namespace,
                        is_async,
                    )
                }
            };
            client_methods.push(client_method);
        }
    }

    // Transform the trait to add PendingSubscriptionSink parameters to subscription methods
    let transformed_trait = transform_trait_for_subscription(&input);

    let expanded = quote! {
        // Generate transformed trait for implementation with PendingSubscriptionSink parameters
        #[hsipc::async_trait]
        #transformed_trait

        // Generate service struct
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

        // Implement Service trait
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

            async fn handle(&self, method: &str, payload: Vec<u8>) -> hsipc::Result<Vec<u8>> {
                match method {
                    #(#service_handlers)*
                    _ => Err(hsipc::Error::method_not_found(self.name(), method))
                }
            }
            
            async fn handle_subscription(
                &self,
                method: &str,
                params: Vec<u8>,
                pending: hsipc::PendingSubscriptionSink,
            ) -> hsipc::Result<()> {
                match method {
                    #(#subscription_handlers)*
                    _ => {
                        let _ = pending.reject(format!("Subscription method '{}' not found", method)).await;
                        Ok(())
                    }
                }
            }
        }

        // Generate client struct
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
