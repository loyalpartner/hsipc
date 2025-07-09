//! Implementation of the #[method] attribute macro
//!
//! This module handles parsing and processing of method attributes in RPC traits

use proc_macro::TokenStream;

/// Configuration for a method attribute
#[derive(Debug, Default)]
pub struct MethodConfig {
    pub name: Option<String>,
    pub timeout: Option<u64>,
    pub is_sync: bool,
}

/// Parse method attribute arguments
fn parse_method_args(args: TokenStream) -> MethodConfig {
    let mut config = MethodConfig::default();
    
    if args.is_empty() {
        return config;
    }
    
    let args_str = args.to_string();
    
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

/// Implementation of the #[method] attribute macro
pub fn method_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let _config = parse_method_args(args);
    
    // For now, just return the original input
    // The method configuration will be processed by the #[rpc] macro
    input
}