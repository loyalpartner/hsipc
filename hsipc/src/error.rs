//! Error types for the hsipc library

use thiserror::Error;

/// Main error type for hsipc operations
#[derive(Error, Debug)]
pub enum Error {
    /// Transport layer errors
    #[error("Transport layer error: {message}")]
    Transport {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Service registration and discovery errors
    #[error("Service '{name}' not found")]
    ServiceNotFound { name: String },

    /// Method invocation errors
    #[error("Method '{method}' not found on service '{service}'")]
    MethodNotFound { service: String, method: String },

    /// Serialization and deserialization errors
    #[error("Serialization error: {message}")]
    Serialization {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Connection management errors
    #[error("Connection error: {message}")]
    Connection {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Timeout errors
    #[error("Operation timed out after {duration_ms}ms: {operation}")]
    Timeout { operation: String, duration_ms: u64 },

    /// Runtime errors
    #[error("Runtime error: {message}")]
    Runtime {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        field: Option<String>,
    },

    /// IO errors
    #[error("IO error: {message}")]
    Io {
        message: String,
        #[source]
        source: std::io::Error,
    },

    /// Service lifecycle errors
    #[error("Service lifecycle error: {message}")]
    ServiceLifecycle {
        message: String,
        service: String,
        state: String,
    },

    /// Protocol errors
    #[error("Protocol error: {message}")]
    Protocol {
        message: String,
        expected: Option<String>,
        received: Option<String>,
    },

    /// Topic pattern validation errors
    #[error("Invalid topic pattern: {pattern}")]
    InvalidTopicPattern { pattern: String },

    /// Subscription management errors
    #[error("Subscription error: {message}")]
    SubscriptionError {
        message: String,
        topic: Option<String>,
    },

    /// Request validation errors
    #[error("Invalid request: {message}")]
    InvalidRequest {
        message: String,
        context: Option<String>,
    },
}

impl Error {
    /// Create a transport error with source
    pub fn transport<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Transport {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a transport error without source
    pub fn transport_msg(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
            source: None,
        }
    }

    /// Create a service not found error
    pub fn service_not_found(name: impl Into<String>) -> Self {
        Self::ServiceNotFound { name: name.into() }
    }

    /// Create a method not found error
    pub fn method_not_found(service: impl Into<String>, method: impl Into<String>) -> Self {
        Self::MethodNotFound {
            service: service.into(),
            method: method.into(),
        }
    }

    /// Create a serialization error with source
    pub fn serialization<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Serialization {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a serialization error without source
    pub fn serialization_msg(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
            source: None,
        }
    }

    /// Create a connection error with source
    pub fn connection<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Connection {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a connection error without source
    pub fn connection_msg(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
            source: None,
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, duration_ms: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration_ms,
        }
    }

    /// Create a runtime error with source
    pub fn runtime<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Runtime {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a runtime error without source
    pub fn runtime_msg(message: impl Into<String>) -> Self {
        Self::Runtime {
            message: message.into(),
            source: None,
        }
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>, field: Option<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            field,
        }
    }

    /// Create a service lifecycle error
    pub fn service_lifecycle(
        message: impl Into<String>,
        service: impl Into<String>,
        state: impl Into<String>,
    ) -> Self {
        Self::ServiceLifecycle {
            message: message.into(),
            service: service.into(),
            state: state.into(),
        }
    }

    /// Create a protocol error
    pub fn protocol(
        message: impl Into<String>,
        expected: Option<String>,
        received: Option<String>,
    ) -> Self {
        Self::Protocol {
            message: message.into(),
            expected,
            received,
        }
    }

    /// Create an invalid topic pattern error
    pub fn invalid_topic_pattern(pattern: impl Into<String>) -> Self {
        Self::InvalidTopicPattern {
            pattern: pattern.into(),
        }
    }

    /// Create a subscription error
    pub fn subscription_error(message: impl Into<String>, topic: Option<String>) -> Self {
        Self::SubscriptionError {
            message: message.into(),
            topic,
        }
    }

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>, context: Option<String>) -> Self {
        Self::InvalidRequest {
            message: message.into(),
            context,
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Transport { .. } => true,
            Error::Connection { .. } => true,
            Error::Timeout { .. } => true,
            Error::Runtime { .. } => true,
            Error::Io { .. } => true,
            Error::ServiceNotFound { .. } => false,
            Error::MethodNotFound { .. } => false,
            Error::Serialization { .. } => false,
            Error::Configuration { .. } => false,
            Error::ServiceLifecycle { .. } => false,
            Error::Protocol { .. } => false,
            Error::InvalidTopicPattern { .. } => false,
            Error::SubscriptionError { .. } => false,
            Error::InvalidRequest { .. } => false,
        }
    }

    /// Get error category for debugging
    pub fn category(&self) -> &'static str {
        match self {
            Error::Transport { .. } => "transport",
            Error::ServiceNotFound { .. } => "service_discovery",
            Error::MethodNotFound { .. } => "method_resolution",
            Error::Serialization { .. } => "serialization",
            Error::Connection { .. } => "connection",
            Error::Timeout { .. } => "timeout",
            Error::Runtime { .. } => "runtime",
            Error::Configuration { .. } => "configuration",
            Error::Io { .. } => "io",
            Error::ServiceLifecycle { .. } => "service_lifecycle",
            Error::Protocol { .. } => "protocol",
            Error::InvalidTopicPattern { .. } => "topic_validation",
            Error::SubscriptionError { .. } => "subscription",
            Error::InvalidRequest { .. } => "request_validation",
        }
    }
}

// Implement From traits for common error types
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io {
            message: err.to_string(),
            source: err,
        }
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Self {
        Error::serialization("Bincode serialization failed", err)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Error::runtime("Task join failed", err)
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::runtime_msg(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::runtime_msg(msg.to_string())
    }
}

impl Error {
    /// Create an error from any type that implements std::error::Error
    pub fn from_std<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::runtime(err.to_string(), err)
    }
}

/// Result type for hsipc operations
pub type Result<T> = std::result::Result<T, Error>;
