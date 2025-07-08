//! Subscribe macro implementation

use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, PatType, Type};

/// Implementation of the #[subscribe] attribute macro
pub fn subscribe_impl(args: TokenStream, input: ItemFn) -> TokenStream {
    let method = &input;
    let method_name = &method.sig.ident;

    // Parse the topic from args or infer from method signature
    let topic = parse_subscribe_topic(args).unwrap_or_else(|| {
        // Try to extract topic from method parameters
        extract_topic_from_method(method).unwrap_or_else(|| {
            // Default topic based on method name
            format!("events/{}", method_name.to_string().replace("on_", ""))
        })
    });

    // Extract the event type from the method signature
    let event_type = extract_event_type(method);

    // Generate subscriber struct name
    let subscriber_name = quote::format_ident!(
        "{}Subscriber",
        method_name
            .to_string()
            .split('_')
            .map(|s| {
                let mut c = s.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<String>()
    );

    let expanded = quote! {
        #method

        /// Auto-generated subscriber for this method
        pub struct #subscriber_name<T> {
            handler: T,
        }

        impl<T> #subscriber_name<T> {
            pub fn new(handler: T) -> Self {
                Self { handler }
            }
        }

        #[::hsipc::async_trait]
        impl<T> ::hsipc::Subscriber for #subscriber_name<T>
        where
            T: Send + Sync + 'static,
        {
            fn topic_pattern(&self) -> &str {
                #topic
            }

            async fn handle(&mut self, _topic: &str, payload: Vec<u8>) -> ::hsipc::Result<()> {
                let event: #event_type = ::bincode::deserialize(&payload)?;
                // Call the actual handler method
                // Note: This is a simplified version - in practice we'd need to
                // capture the self context and call the method properly
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Parse topic from macro arguments
fn parse_subscribe_topic(args: TokenStream) -> Option<String> {
    if args.is_empty() {
        return None;
    }

    let args_str = args.to_string();
    if args_str.starts_with('"') && args_str.ends_with('"') {
        Some(args_str[1..args_str.len() - 1].to_string())
    } else {
        None
    }
}

/// Extract topic from method signature (for methods like on_temperature)
fn extract_topic_from_method(method: &ItemFn) -> Option<String> {
    let method_name = method.sig.ident.to_string();

    method_name
        .strip_prefix("on_")
        .map(|event_name| format!("events/{event_name}"))
}

/// Extract event type from method signature
fn extract_event_type(method: &ItemFn) -> syn::Type {
    // Look for the event parameter (usually the last parameter)
    for input in &method.sig.inputs {
        if let FnArg::Typed(PatType { ty, .. }) = input {
            // Skip &self and &mut self
            if let Type::Reference(type_ref) = ty.as_ref() {
                if let Type::Path(type_path) = type_ref.elem.as_ref() {
                    if let Some(segment) = type_path.path.segments.last() {
                        if segment.ident == "Self" {
                            continue;
                        }
                    }
                }
            }

            // This is likely the event type
            return *ty.clone();
        }
    }

    // Fallback to a generic type
    syn::parse_str("()").unwrap()
}
