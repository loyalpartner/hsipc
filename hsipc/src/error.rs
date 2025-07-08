//! Error types for hsipc

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Timeout error")]
    Timeout,

    #[error("Connection lost")]
    ConnectionLost,

    #[error("Invalid topic pattern: {0}")]
    InvalidTopicPattern(String),

    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
