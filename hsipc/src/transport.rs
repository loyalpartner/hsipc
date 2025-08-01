//! Transport layer - abstract trait only

use crate::{Message, Result};
use async_trait::async_trait;

/// Transport trait for abstracting communication
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    /// Send a message
    async fn send(&self, msg: Message) -> Result<()>;

    /// Receive a message
    async fn recv(&self) -> Result<Message>;

    /// Close the transport
    async fn close(&self) -> Result<()>;
}
