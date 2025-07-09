//! # hsipc - High-performance inter-process communication framework
//!
//! A declarative IPC framework built on top of ipmb, providing type-safe
//! request/response and publish/subscribe patterns.
//!
//! ## Quick Start
//!
//! ### Request/Response Pattern
//!
//! Define services using the `#[service]` macro:
//!
//! ```rust,ignore
//! use hsipc::{service, ProcessHub, Result};
//!
//! #[derive(Debug)]
//! pub struct Calculator;
//!
//! #[service]
//! impl Calculator {
//!     async fn add(&self, params: (i32, i32)) -> Result<i32> {
//!         Ok(params.0 + params.1)
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let hub = ProcessHub::new("demo").await?;
//!     
//!     // Register service
//!     let service = CalculatorService::new(Calculator);
//!     hub.register_service(service).await?;
//!     
//!     // Use direct calls
//!     let result: i32 = hub.call("Calculator.add", (10, 5)).await?;
//!     assert_eq!(result, 15);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Publish/Subscribe Pattern
//!
//! Define events using the `#[derive(Event)]` macro:
//!
//! ```rust,ignore
//! use hsipc::{Event, ProcessHub, Result};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Event, Serialize, Deserialize, Debug, Clone)]
//! #[event(topic = "sensor/temperature")]
//! pub struct TemperatureEvent {
//!     pub value: f64,
//!     pub unit: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let hub = ProcessHub::new("demo").await?;
//!     
//!     // Publish event
//!     let event = TemperatureEvent {
//!         value: 25.3,
//!         unit: "Celsius".to_string(),
//!     };
//!     hub.publish_event(event).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Available Macros
//!
//! - `#[service]` - **CURRENT**: Generate service wrappers and clients from impl blocks
//! - `#[derive(Event)]` - **PRIMARY**: Implement Event trait for pub/sub
//! - `#[subscribe]` - Mark subscriber functions (experimental)
//! - `#[service_impl]` - **TRAIT-BASED**: Service implementation from trait (alternative approach)
//!
//! ### Design Trade-offs
//!
//! **`#[service]` approach**: Simpler to use, single macro, but less type-safe interface definition
//!
//! **`#[service_trait]` + `#[service_impl]` approach**: More type-safe, better interface separation,
//! supports polymorphism and testing, but requires two macros
//!
//! ### Internal/Experimental Macros (not exported)
//!
//! - `#[service_trait]` - Generate clients from trait definitions (internal)
//! - `#[derive(Service)]` - Service wrapper generation (experimental)
//!
//! For detailed macro usage, see the [`macros`] module.

pub mod error;
pub mod event;
pub mod hub;
pub mod message;
pub mod service;
pub mod transport;
pub mod transport_ipmb;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod error_tests;

// Re-exports
pub use error::{Error, Result};
pub use event::{Event, Subscriber};
pub use hub::{ProcessHub, SyncProcessHub};
pub use message::{Message, Request, Response};
pub use service::{Service, ServiceRegistry};

// Re-export macros when feature is enabled
#[cfg(feature = "macros")]
pub use hsipc_macros::{service, service_impl, service_trait, subscribe, Event};

// Macro usage documentation
#[cfg(feature = "macros")]
pub mod macros {
    //! Macro usage examples and documentation
    //!
    //! This module provides comprehensive examples for all available macros.

    /// # Service Macro (`#[service]`)
    ///
    /// Generates service wrappers and clients for request/response pattern.
    ///
    /// ## Usage
    /// ```rust,ignore
    /// use hsipc::{service, Result};
    ///
    /// #[derive(Debug)]
    /// pub struct Calculator;
    ///
    /// #[service]
    /// impl Calculator {
    ///     async fn add(&self, params: (i32, i32)) -> Result<i32> {
    ///         Ok(params.0 + params.1)
    ///     }
    ///     
    ///     async fn multiply(&self, params: (i32, i32)) -> Result<i32> {
    ///         Ok(params.0 * params.1)
    ///     }
    /// }
    ///
    /// // Auto-generated:
    /// // - CalculatorService wrapper struct
    /// // - CalculatorClient for remote calls
    ///
    /// // Usage:
    /// async fn example() -> Result<()> {
    ///     let hub = ProcessHub::new("demo").await?;
    ///     
    ///     // Register service
    ///     let service = CalculatorService::new(Calculator);
    ///     hub.register_service(service).await?;
    ///     
    ///     // Use client
    ///     let client = CalculatorClient::new("client").await?;
    ///     let result = client.add((10, 5)).await?;
    ///     assert_eq!(result, 15);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub use crate::service;

    /// # Event Derive Macro (`#[derive(Event)]`)
    ///
    /// Automatically implements the Event trait for publish/subscribe pattern.
    ///
    /// ## Usage
    /// ```rust,ignore
    /// use hsipc::{Event, ProcessHub, Result};
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Event, Serialize, Deserialize, Debug, Clone)]
    /// #[event(topic = "sensor/temperature")]
    /// pub struct TemperatureEvent {
    ///     pub value: f64,
    ///     pub unit: String,
    ///     pub timestamp: u64,
    /// }
    ///
    /// #[derive(Event, Serialize, Deserialize, Debug, Clone)]
    /// #[event(topic = "sensor/humidity")]  
    /// pub struct HumidityEvent {
    ///     pub percentage: f64,
    ///     pub timestamp: u64,
    /// }
    ///
    /// // Usage:
    /// async fn example() -> Result<()> {
    ///     let hub = ProcessHub::new("publisher").await?;
    ///     
    ///     let temp_event = TemperatureEvent {
    ///         value: 25.3,
    ///         unit: "Celsius".to_string(),
    ///         timestamp: 1234567890,
    ///     };
    ///     
    ///     hub.publish_event(temp_event).await?;
    ///     Ok(())
    /// }
    /// ```
    pub use crate::Event;

    /// # Subscribe Macro (`#[subscribe]`)
    ///
    /// Marks functions as event subscribers (experimental).
    ///
    /// ## Usage
    /// ```rust,ignore
    /// use hsipc::subscribe;
    ///
    /// struct EventHandler;
    ///
    /// impl EventHandler {
    ///     #[subscribe("sensor/temperature")]
    ///     async fn handle_temperature(&self, event: TemperatureEvent) {
    ///         println!("Temperature: {}°{}", event.value, event.unit);
    ///     }
    /// }
    /// ```
    ///
    /// **Note**: This macro is experimental and may change in future versions.
    pub use crate::subscribe;

    /// # Service Implementation Macro (`#[service_impl]`)
    ///
    /// **Trait-based approach** for service implementations - more type-safe alternative.
    ///
    /// ## Usage
    /// ```rust,ignore
    /// use hsipc::{service_impl, Service, Result};
    ///
    /// // First define the interface
    /// trait Calculator {
    ///     async fn add(&self, params: (i32, i32)) -> Result<i32>;
    ///     async fn multiply(&self, params: (i32, i32)) -> Result<i32>;
    /// }
    ///
    /// // Then implement it
    /// struct MyCalculator;
    ///
    /// #[service_impl]
    /// impl Calculator for MyCalculator {
    ///     async fn add(&self, params: (i32, i32)) -> Result<i32> {
    ///         Ok(params.0 + params.1)
    ///     }
    ///     
    ///     async fn multiply(&self, params: (i32, i32)) -> Result<i32> {
    ///         Ok(params.0 * params.1)
    ///     }
    /// }
    /// ```
    ///
    /// ## Advantages of Trait-based Approach
    ///
    /// - **Better type safety**: Clear interface definition
    /// - **Polymorphism**: Multiple implementations of same interface
    /// - **Testability**: Easy to create mock implementations
    /// - **Composition**: Support for decorator patterns
    ///
    /// ## Trade-offs
    ///
    /// - **Complexity**: Requires separate trait definition
    /// - **Two-step process**: Need both trait and impl
    /// - **Current limitation**: May have some implementation gaps
    ///
    /// **Note**: This approach is theoretically superior but currently less polished than `#[service]`.
    pub use crate::service_impl;

    /// # Service Trait Definition Macro (`#[service_trait]`)
    ///
    /// **NEW**: Generate typed clients from trait definitions - part of the enhanced trait-based approach.
    ///
    /// ## Usage
    /// ```rust,ignore
    /// use hsipc::{service_trait, Result};
    ///
    /// #[service_trait]
    /// trait Calculator {
    ///     async fn add(&self, params: (i32, i32)) -> Result<i32>;
    ///     async fn multiply(&self, params: (i32, i32)) -> Result<i32>;
    /// }
    ///
    /// // Auto-generates: CalculatorClient with fully typed methods
    ///
    /// // Usage:
    /// async fn example() -> Result<()> {
    ///     let client = CalculatorClient::new("calculator_client").await?;
    ///     let result: i32 = client.add((10, 5)).await?; // Fully typed!
    ///     assert_eq!(result, 15);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Combined with `#[service_impl]`
    ///
    /// Use together for the complete trait-based workflow:
    ///
    /// ```rust,ignore
    /// // Step 1: Define interface with client generation
    /// #[service_trait]
    /// trait Calculator {
    ///     async fn add(&self, params: (i32, i32)) -> Result<i32>;
    /// }
    ///
    /// // Step 2: Implement with service wrapper generation  
    /// struct LocalCalculator;
    ///
    /// #[service_impl]
    /// impl Calculator for LocalCalculator {
    ///     async fn add(&self, params: (i32, i32)) -> Result<i32> {
    ///         Ok(params.0 + params.1)
    ///     }
    /// }
    ///
    /// // Auto-generated: CalculatorClient + CalculatorService
    /// ```
    ///
    /// ## Advantages
    ///
    /// - **Full type safety**: Client methods have exact parameter and return types
    /// - **Clear separation**: Interface definition separate from implementation
    /// - **Excellent IDE support**: Full autocomplete and type checking
    ///
    /// **Status**: Newly enhanced - this is the recommended trait-based approach.
    pub use crate::service_trait;
}

// Re-export commonly used dependencies
pub use async_trait::async_trait;
pub use bincode;
pub use serde::{Deserialize, Serialize};

// Runtime support is provided by SyncProcessHub in hub module
