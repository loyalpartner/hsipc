//! Event derive macro implementation

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Meta};

/// Implementation of the Event derive macro
pub fn derive_event_impl(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse the #[event] attribute to get the topic
    let topic = parse_event_topic(&input.attrs);

    let topic_impl = match topic {
        Some(topic) => {
            if topic.contains('{') {
                // Dynamic topic with field interpolation
                generate_dynamic_topic(name, &input.data, &topic)
            } else {
                // Static topic
                quote! {
                    fn topic(&self) -> String {
                        #topic.to_string()
                    }
                }
            }
        }
        None => {
            // Default topic based on type name
            let default_topic = format!("events/{}", name.to_string().to_lowercase());
            quote! {
                fn topic(&self) -> String {
                    #default_topic.to_string()
                }
            }
        }
    };

    let expanded = quote! {
        impl #impl_generics ::hsipc::Event for #name #ty_generics #where_clause {
            #topic_impl
        }
    };

    TokenStream::from(expanded)
}

/// Parse the topic from #[event(topic = "...")] attribute
fn parse_event_topic(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("event") {
            // Try to parse meta
            let meta = &attr.meta;
            if let Meta::List(list) = meta {
                // Simple string parsing from tokens
                let tokens = list.tokens.to_string();
                if let Some(start) = tokens.find("topic") {
                    if let Some(quote_start) = tokens[start..].find('"') {
                        if let Some(quote_end) = tokens[start + quote_start + 1..].find('"') {
                            let topic = &tokens
                                [start + quote_start + 1..start + quote_start + 1 + quote_end];
                            return Some(topic.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Generate dynamic topic implementation for patterns like "device/{device_id}/status"
fn generate_dynamic_topic(
    _name: &syn::Ident,
    data: &Data,
    topic_pattern: &str,
) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let topic_code = topic_pattern.to_string();
                    let mut replacements = Vec::new();

                    // Find all {field_name} patterns in the topic
                    for field in &fields.named {
                        if let Some(field_name) = &field.ident {
                            let field_name_str = field_name.to_string();
                            let pattern = format!("{{{field_name_str}}}");

                            if topic_code.contains(&pattern) {
                                replacements.push((pattern, field_name));
                            }
                        }
                    }

                    // Generate replacement code
                    let mut format_string = topic_code.clone();
                    let mut format_args = Vec::new();

                    for (pattern, field_name) in replacements {
                        format_string = format_string.replace(&pattern, "{}");
                        format_args.push(quote! { &self.#field_name });
                    }

                    quote! {
                        fn topic(&self) -> String {
                            format!(#format_string, #(#format_args),*)
                        }
                    }
                }
                _ => {
                    // For other field types, just return the pattern as-is
                    quote! {
                        fn topic(&self) -> String {
                            #topic_pattern.to_string()
                        }
                    }
                }
            }
        }
        _ => {
            // For enums and other types, just return the pattern as-is
            quote! {
                fn topic(&self) -> String {
                    #topic_pattern.to_string()
                }
            }
        }
    }
}
