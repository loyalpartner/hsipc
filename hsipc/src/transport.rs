//! Transport layer - abstract trait and mock implementation for testing

use crate::{Error, Message, Result};
use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;

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

/// Mock transport for testing
#[cfg(test)]
pub struct MockTransport {
    tx: mpsc::Sender<Message>,
    rx: Arc<RwLock<mpsc::Receiver<Message>>>,
}

#[cfg(test)]
impl MockTransport {
    pub fn new() -> (Self, mpsc::Receiver<Message>) {
        let (tx1, rx1) = mpsc::channel(100);
        let (_tx2, rx2) = mpsc::channel(100);

        (
            Self {
                tx: tx1,
                rx: Arc::new(RwLock::new(rx2)),
            },
            rx1,
        )
    }
}

#[cfg(test)]
#[async_trait]
impl Transport for MockTransport {
    async fn send(&self, msg: Message) -> Result<()> {
        self.tx
            .send(msg)
            .await
            .map_err(|_| Error::connection_msg("mock transport send failed"))
    }

    async fn recv(&self) -> Result<Message> {
        let mut rx = self.rx.write().await;
        rx.recv()
            .await
            .ok_or(Error::connection_msg("mock transport recv failed"))
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}
