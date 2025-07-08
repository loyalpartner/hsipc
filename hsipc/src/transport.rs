//! Transport layer - simplified implementation with shared message bus

use crate::{Error, Message, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

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

/// Shared message bus for inter-process communication
/// In a real implementation, this would be replaced with actual ipmb
static MESSAGE_BUS: once_cell::sync::Lazy<Arc<MessageBus>> =
    once_cell::sync::Lazy::new(|| Arc::new(MessageBus::new()));

struct MessageBus {
    /// Global broadcast channel for all messages
    sender: broadcast::Sender<Message>,
    /// Keep track of registered processes
    processes: Arc<RwLock<HashMap<String, ProcessInfo>>>,
}

struct ProcessInfo {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    registered_at: std::time::Instant,
}

impl MessageBus {
    fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self {
            sender,
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn register_process(&self, name: &str) -> broadcast::Receiver<Message> {
        let mut processes = self.processes.write().await;
        processes.insert(
            name.to_string(),
            ProcessInfo {
                name: name.to_string(),
                registered_at: std::time::Instant::now(),
            },
        );

        // Create a receiver for this process
        self.sender.subscribe()
    }

    async fn send_message(&self, msg: Message) -> Result<()> {
        self.sender.send(msg).map_err(|_| Error::ConnectionLost)?;
        Ok(())
    }

    #[allow(dead_code)]
    async fn unregister_process(&self, name: &str) {
        let mut processes = self.processes.write().await;
        processes.remove(name);
    }
}

/// Simple transport that uses a shared message bus
pub struct IpmbTransport {
    process_name: String,
    #[allow(dead_code)]
    receiver: Arc<RwLock<broadcast::Receiver<Message>>>,
    local_receiver: Arc<RwLock<mpsc::Receiver<Message>>>,
    _receiver_task: Arc<tokio::task::JoinHandle<()>>,
}

impl IpmbTransport {
    /// Create a new transport
    pub async fn new(process_name: &str) -> Result<Self> {
        // Register with the shared message bus
        let mut bus_receiver = MESSAGE_BUS.register_process(process_name).await;

        // Create a local channel for filtered messages
        let (local_tx, local_rx) = mpsc::channel(1024);

        // Spawn a task to filter messages for this process
        let process_name_clone = process_name.to_string();
        let receiver_task = tokio::spawn(async move {
            while let Ok(msg) = bus_receiver.recv().await {
                // Filter messages for this process
                let should_receive = match &msg.target {
                    Some(target) => target == &process_name_clone,
                    None => {
                        // This is a broadcast/event message, or a service call
                        // Both service calls and events should be received
                        true
                    }
                };

                if should_receive && local_tx.send(msg).await.is_err() {
                    break; // Receiver dropped
                }
            }
        });

        Ok(Self {
            process_name: process_name.to_string(),
            receiver: Arc::new(RwLock::new(MESSAGE_BUS.sender.subscribe())),
            local_receiver: Arc::new(RwLock::new(local_rx)),
            _receiver_task: Arc::new(receiver_task),
        })
    }
}

#[async_trait]
impl Transport for IpmbTransport {
    async fn send(&self, msg: Message) -> Result<()> {
        MESSAGE_BUS.send_message(msg).await
    }

    async fn recv(&self) -> Result<Message> {
        let mut receiver = self.local_receiver.write().await;
        receiver.recv().await.ok_or(Error::ConnectionLost)
    }

    async fn close(&self) -> Result<()> {
        MESSAGE_BUS.unregister_process(&self.process_name).await;
        Ok(())
    }
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
        self.tx.send(msg).await.map_err(|_| Error::ConnectionLost)
    }

    async fn recv(&self) -> Result<Message> {
        let mut rx = self.rx.write().await;
        rx.recv().await.ok_or(Error::ConnectionLost)
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}
