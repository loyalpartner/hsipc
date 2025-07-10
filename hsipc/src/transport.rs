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

/// Shared message buses for inter-process communication
/// In a real implementation, this would be replaced with actual ipmb
/// For testing, we create isolated buses per test process to avoid conflicts
static MESSAGE_BUSES: once_cell::sync::Lazy<Arc<RwLock<HashMap<String, Arc<MessageBus>>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Get or create a message bus for the current test process
async fn get_message_bus() -> Arc<MessageBus> {
    let bus_key = if cfg!(test) {
        // For tests, create isolated buses per process ID to avoid conflicts
        format!("test_bus_{}", std::process::id())
    } else {
        // For production, use a shared bus
        "production_bus".to_string()
    };

    let mut buses = MESSAGE_BUSES.write().await;
    if let Some(bus) = buses.get(&bus_key) {
        bus.clone()
    } else {
        let new_bus = Arc::new(MessageBus::new());
        buses.insert(bus_key.clone(), new_bus.clone());
        tracing::debug!("🚌 Created new message bus: {}", bus_key);
        new_bus
    }
}

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
        self.sender
            .send(msg)
            .map_err(|_| Error::connection_msg("broadcast channel send failed"))?;
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
        // Get isolated message bus for this test process
        let message_bus = get_message_bus().await;
        // Register with the isolated message bus
        let mut bus_receiver = message_bus.register_process(process_name).await;

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
            receiver: Arc::new(RwLock::new(message_bus.sender.subscribe())),
            local_receiver: Arc::new(RwLock::new(local_rx)),
            _receiver_task: Arc::new(receiver_task),
        })
    }
}

#[async_trait]
impl Transport for IpmbTransport {
    async fn send(&self, msg: Message) -> Result<()> {
        let message_bus = get_message_bus().await;
        message_bus.send_message(msg).await
    }

    async fn recv(&self) -> Result<Message> {
        let mut receiver = self.local_receiver.write().await;
        receiver
            .recv()
            .await
            .ok_or(Error::connection_msg("message channel closed"))
    }

    async fn close(&self) -> Result<()> {
        let message_bus = get_message_bus().await;
        message_bus.unregister_process(&self.process_name).await;
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
