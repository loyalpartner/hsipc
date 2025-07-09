//! Procedural macros for hsipc - RPC system

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod event;
mod rpc;
mod subscribe;

/// Derive macro for creating events
#[proc_macro_derive(Event, attributes(event))]
pub fn event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    event::derive_event_impl(input)
}

/// Attribute macro for subscription methods
#[proc_macro_attribute]
pub fn subscribe(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemFn);
    subscribe::subscribe_impl(args, input)
}

// RPC System Macros

/// Main RPC macro - generates server and client code
#[proc_macro_attribute]
pub fn rpc(args: TokenStream, input: TokenStream) -> TokenStream {
    rpc::rpc_impl(args, input)
}

/// Method attribute macro for RPC methods (placeholder)
#[proc_macro_attribute]
pub fn method(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Placeholder - actual processing happens in #[rpc] macro
    input
}

/// Subscription attribute macro for RPC subscriptions (placeholder)
#[proc_macro_attribute]
pub fn subscription(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Placeholder - actual processing happens in #[rpc] macro
    input
}
