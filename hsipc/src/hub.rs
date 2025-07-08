//! ProcessHub - Main hub for inter-process communication

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    event::{Event, Subscriber, Subscription, SubscriptionRegistry},
    message::MessageType,
    service::{Service, ServiceRegistry},
    transport::{IpmbTransport, Transport},
    Error, Message, Result,
};

/// Main process hub for IPC communication
#[derive(Clone)]
pub struct ProcessHub {
    name: String,
    transport: Arc<dyn Transport>,
    service_registry: Arc<ServiceRegistry>,
    subscription_registry: Arc<SubscriptionRegistry>,
    pending_requests:
        Arc<RwLock<std::collections::HashMap<Uuid, tokio::sync::oneshot::Sender<Message>>>>,
}

impl ProcessHub {
    /// Create a new ProcessHub
    pub async fn new(name: &str) -> Result<Self> {
        let transport = IpmbTransport::new(name).await?;

        let hub = Self {
            name: name.to_string(),
            transport: Arc::new(transport),
            service_registry: Arc::new(ServiceRegistry::new()),
            subscription_registry: Arc::new(SubscriptionRegistry::new()),
            pending_requests: Arc::new(RwLock::new(std::collections::HashMap::new())),
        };

        // Start message processing
        hub.start_message_loop().await;

        Ok(hub)
    }

    /// Start the message processing loop
    async fn start_message_loop(&self) {
        let transport = self.transport.clone();
        let service_registry = self.service_registry.clone();
        let subscription_registry = self.subscription_registry.clone();
        let pending_requests = self.pending_requests.clone();

        tokio::spawn(async move {
            loop {
                match transport.recv().await {
                    Ok(msg) => {
                        let _ = Self::process_message(
                            msg,
                            &service_registry,
                            &subscription_registry,
                            &pending_requests,
                            &transport,
                        )
                        .await;
                    }
                    Err(_) => break,
                }
            }
        });
    }

    /// Process incoming messages
    async fn process_message(
        msg: Message,
        service_registry: &ServiceRegistry,
        subscription_registry: &SubscriptionRegistry,
        pending_requests: &RwLock<
            std::collections::HashMap<Uuid, tokio::sync::oneshot::Sender<Message>>,
        >,
        transport: &Arc<dyn Transport>,
    ) -> Result<()> {
        match msg.msg_type {
            MessageType::Request => {
                // Handle service request
                if let Some(ref topic) = msg.topic {
                    match service_registry.call(topic, msg.payload.clone()).await {
                        Ok(result) => {
                            let response = Message::response(&msg, result);
                            let _ = transport.send(response).await;
                        }
                        Err(e) => {
                            let error_msg = format!("Service error: {e}");
                            let mut error_response =
                                Message::response(&msg, error_msg.into_bytes());
                            error_response.msg_type = MessageType::Error;
                            let _ = transport.send(error_response).await;
                        }
                    }
                }
            }
            MessageType::Response | MessageType::Error => {
                // Handle response to our request
                if let Some(correlation_id) = msg.correlation_id {
                    let mut requests = pending_requests.write().await;
                    if let Some(sender) = requests.remove(&correlation_id) {
                        let _ = sender.send(msg);
                    }
                }
            }
            MessageType::Event => {
                // Handle event for subscribers
                if let Some(ref topic) = msg.topic {
                    let _ = subscription_registry.publish(topic, msg.payload).await;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Register a service
    pub async fn register_service<S: Service>(&self, service: S) -> Result<()> {
        self.service_registry.register(service).await
    }

    /// Call a service method
    pub async fn call<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        service_method: &str,
        request: T,
    ) -> Result<R> {
        let payload = bincode::serialize(&request)?;
        let request_id = uuid::Uuid::new_v4();
        let msg = Message {
            id: request_id,
            msg_type: MessageType::Request,
            source: self.name.clone(),
            target: None, // Service call - broadcast to all services
            topic: Some(service_method.to_string()),
            payload,
            correlation_id: Some(request_id),
            metadata: crate::message::MessageMetadata::default(),
        };

        // Set up response receiver
        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(request_id, tx);
        }

        // Send request
        self.transport.send(msg).await?;

        // Wait for response with timeout
        let response = tokio::time::timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| Error::Timeout)?
            .map_err(|_| Error::ConnectionLost)?;

        match response.msg_type {
            MessageType::Response => {
                let result: R = bincode::deserialize(&response.payload)?;
                Ok(result)
            }
            MessageType::Error => {
                let error_msg = String::from_utf8_lossy(&response.payload);
                Err(Error::Other(anyhow::anyhow!("Remote error: {}", error_msg)))
            }
            _ => Err(Error::Other(anyhow::anyhow!("Unexpected response type"))),
        }
    }

    /// Subscribe to events
    pub async fn subscribe<S: Subscriber>(&self, subscriber: S) -> Result<Subscription> {
        self.subscription_registry.subscribe(subscriber).await
    }

    /// Publish an event
    pub async fn publish_event<E: Event>(&self, event: E) -> Result<()> {
        let topic = event.topic();
        let payload = bincode::serialize(&event)?;
        let msg = Message::event(self.name.clone(), topic, payload);

        self.transport.send(msg).await
    }

    /// Publish to a specific topic
    pub async fn publish<T: Serialize>(&self, topic: &str, payload: T) -> Result<()> {
        let serialized = bincode::serialize(&payload)?;
        let msg = Message::event(self.name.clone(), topic.to_string(), serialized);

        self.transport.send(msg).await
    }

    /// Get the process name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Shutdown the hub
    pub async fn shutdown(&self) -> Result<()> {
        self.transport.close().await
    }
}

/// Synchronous wrapper for ProcessHub
pub struct SyncProcessHub {
    runtime: tokio::runtime::Runtime,
    hub: ProcessHub,
}

impl SyncProcessHub {
    /// Create a new synchronous ProcessHub
    pub fn new(name: &str) -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new().map_err(|e| Error::Runtime(e.to_string()))?;

        let hub = runtime.block_on(ProcessHub::new(name))?;

        Ok(Self { runtime, hub })
    }

    /// Register a service synchronously
    pub fn register_service<S: Service>(&self, service: S) -> Result<()> {
        self.runtime.block_on(self.hub.register_service(service))
    }

    /// Call a service method synchronously
    pub fn call<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        service_method: &str,
        request: T,
    ) -> Result<R> {
        self.runtime
            .block_on(self.hub.call(service_method, request))
    }

    /// Subscribe to events synchronously
    pub fn subscribe<S: Subscriber>(&self, subscriber: S) -> Result<Subscription> {
        self.runtime.block_on(self.hub.subscribe(subscriber))
    }

    /// Publish an event synchronously
    pub fn publish_event<E: Event>(&self, event: E) -> Result<()> {
        self.runtime.block_on(self.hub.publish_event(event))
    }

    /// Publish to a specific topic synchronously
    pub fn publish<T: Serialize>(&self, topic: &str, payload: T) -> Result<()> {
        self.runtime.block_on(self.hub.publish(topic, payload))
    }

    /// Get the process name
    pub fn name(&self) -> &str {
        self.hub.name()
    }

    /// Shutdown the hub
    pub fn shutdown(self) -> Result<()> {
        self.runtime.block_on(self.hub.shutdown())
    }
}

/// Optional runtime for sync mode
#[cfg(feature = "runtime")]
pub struct Runtime {
    rt: tokio::runtime::Runtime,
}

#[cfg(feature = "runtime")]
impl Runtime {
    pub fn new() -> Result<Self> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::Runtime(e.to_string()))?;

        Ok(Self { rt })
    }

    pub fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        self.rt.block_on(future)
    }
}
