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
        // Increase broadcast channel capacity to handle message bursts
        // This helps prevent receiver lag in high-throughput scenarios
        let (sender, _) = broadcast::channel(8192);
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
        tracing::info!("🚌 MessageBus broadcasting message type: {:?} id: {}", msg.msg_type, msg.id);
        
        // Get the number of active receivers before sending
        let receiver_count = self.sender.receiver_count();
        tracing::info!("📊 Active receivers: {}", receiver_count);
        
        match self.sender.send(msg.clone()) {
            Ok(sent_count) => {
                tracing::info!("✅ MessageBus broadcast successful for message type: {:?}, sent to {} receivers", msg.msg_type, sent_count);
                Ok(())
            }
            Err(_) => {
                tracing::error!("❌ MessageBus broadcast failed - no active receivers!");
                Err(Error::connection_msg("broadcast channel send failed"))
            }
        }
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

        // Create a local channel for filtered messages with larger buffer
        // to handle bursts of messages without dropping
        let (local_tx, local_rx) = mpsc::channel(4096);

        // Spawn a task to filter messages for this process
        let process_name_clone = process_name.to_string();
        let receiver_task = tokio::spawn(async move {
            tracing::debug!("🚌 Starting message filter task for process: {}", process_name_clone);
            loop {
                match bus_receiver.recv().await {
                    Ok(msg) => {
                        // Filter messages for this process
                        let should_receive = match &msg.target {
                            Some(target) => target == &process_name_clone,
                            None => true, // Broadcast messages
                        };

                        if should_receive {
                            tracing::info!("✅ Forwarding message type: {:?} id: {} to process: {}", msg.msg_type, msg.id, process_name_clone);
                            if local_tx.send(msg).await.is_err() {
                                tracing::error!("📪 Local receiver channel closed for process: {}", process_name_clone);
                                break; // Receiver dropped
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        // Some messages were skipped due to slow processing
                        tracing::error!("⚠️  CRITICAL: Broadcast receiver lagged, skipped {} messages for process: {}. This may cause message loss!", skipped, process_name_clone);
                        // Continue processing - don't exit the loop
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        // Broadcast channel closed, exit gracefully
                        tracing::info!("📪 Broadcast channel closed for process: {}", process_name_clone);
                        break;
                    }
                }
            }
            tracing::debug!("🔚 Message filter task ended for process: {}", process_name_clone);
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
        tracing::debug!("🚌 Transport sending message type: {:?} from {}", msg.msg_type, msg.source);
        let message_bus = get_message_bus().await;
        message_bus.send_message(msg).await
    }

    async fn recv(&self) -> Result<Message> {
        tracing::debug!("🔍 Transport recv() called for process: {}", self.process_name);
        
        // Use try_write to avoid blocking if another recv is in progress
        // This helps prevent deadlocks in high-concurrency scenarios
        match self.local_receiver.try_write() {
            Ok(mut receiver) => {
                tracing::debug!("✅ Got receiver lock immediately for process: {}", self.process_name);
                match receiver.recv().await {
                    Some(msg) => {
                        tracing::info!("📨 Received message type: {:?} id: {} for process: {}", msg.msg_type, msg.id, self.process_name);
                        Ok(msg)
                    }
                    None => {
                        tracing::error!("❌ Local channel closed for process: {}", self.process_name);
                        Err(Error::connection_msg("message channel closed"))
                    }
                }
            }
            Err(_) => {
                tracing::debug!("⏳ Waiting for receiver lock for process: {}", self.process_name);
                // If we can't get the lock immediately, wait and retry
                let mut receiver = self.local_receiver.write().await;
                tracing::debug!("✅ Got receiver lock after waiting for process: {}", self.process_name);
                match receiver.recv().await {
                    Some(msg) => {
                        tracing::info!("📨 Received message type: {:?} id: {} for process: {}", msg.msg_type, msg.id, self.process_name);
                        Ok(msg)
                    }
                    None => {
                        tracing::error!("❌ Local channel closed for process: {}", self.process_name);
                        Err(Error::connection_msg("message channel closed"))
                    }
                }
            }
        }
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
