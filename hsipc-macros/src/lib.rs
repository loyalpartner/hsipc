//! Procedural macros for hsipc

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemImpl, ItemTrait};

mod event;
mod service;
mod subscribe;

/// Derive macro for creating events
#[proc_macro_derive(Event, attributes(event))]
pub fn event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    event::derive_event_impl(input)
}

/// Attribute macro for defining services with direct method calls
#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    service::service_impl_new(args, input)
}

/// Generate service wrapper for a struct with service methods
#[proc_macro_derive(Service, attributes(service))]
pub fn derive_service(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    service::derive_service_impl(input)
}

/// Attribute macro for service trait definitions
#[proc_macro_attribute]
pub fn service_trait(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemTrait);
    service::service_impl(args, input)
}

/// Attribute macro for service implementations (legacy)
#[proc_macro_attribute]
pub fn service_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    service::service_impl_impl(args, input)
}

/// Attribute macro for subscription methods
#[proc_macro_attribute]
pub fn subscribe(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemFn);
    subscribe::subscribe_impl(args, input)
}
