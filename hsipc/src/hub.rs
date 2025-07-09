//! ProcessHub - Main hub for inter-process communication

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    event::{Event, Subscriber, Subscription, SubscriptionRegistry},
    message::{MessageType, ServiceDirectory, ServiceInfo},
    transport::Transport,
    Error, Message, Result,
};

#[cfg(test)]
use crate::transport::IpmbTransport;

// Simple Service trait for RPC system
#[async_trait::async_trait]
pub trait Service: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn methods(&self) -> Vec<&'static str>;
    async fn handle(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>>;
}

// Service registry for managing RPC services
pub struct ServiceRegistry {
    services: Arc<RwLock<std::collections::HashMap<String, Arc<dyn Service>>>>,
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn register(&self, service: Arc<dyn Service>) -> Result<()> {
        let mut services = self.services.write().await;
        services.insert(service.name().to_string(), service);
        Ok(())
    }

    pub async fn call(&self, service_method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
        // Parse service.method format
        let parts: Vec<&str> = service_method.split('.').collect();
        if parts.len() != 2 {
            return Err(Error::invalid_request(
                "Invalid service.method format",
                None,
            ));
        }

        let (service_name, method) = (parts[0], parts[1]);
        let services = self.services.read().await;
        if let Some(service) = services.get(service_name) {
            service.handle(method, payload).await
        } else {
            Err(Error::service_not_found(service_name))
        }
    }

    pub async fn list_services(&self) -> Vec<String> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }

    pub async fn get_service(&self, service_name: &str) -> Option<std::sync::Arc<dyn Service>> {
        let services = self.services.read().await;
        services.get(service_name).cloned()
    }
}

#[cfg(not(test))]
use crate::transport_ipmb::IpmbTransport;

/// Main process hub for IPC communication
#[derive(Clone)]
pub struct ProcessHub {
    name: String,
    transport: Arc<dyn Transport>,
    service_registry: Arc<ServiceRegistry>,
    subscription_registry: Arc<SubscriptionRegistry>,
    pending_requests:
        Arc<RwLock<std::collections::HashMap<Uuid, tokio::sync::oneshot::Sender<Message>>>>,
    /// Remote service directory for cross-process service discovery
    remote_services: Arc<RwLock<std::collections::HashMap<String, ServiceInfo>>>,
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
            remote_services: Arc::new(RwLock::new(std::collections::HashMap::new())),
        };

        // Start message processing
        hub.start_message_loop().await;

        // Proactively query for existing services after startup
        let hub_clone = hub.clone();
        tokio::spawn(async move {
            // Wait a bit for the message loop to start
            tokio::time::sleep(Duration::from_millis(100)).await;
            let _ = hub_clone.query_services().await;
        });

        Ok(hub)
    }

    /// Start the message processing loop
    async fn start_message_loop(&self) {
        let transport = self.transport.clone();
        let service_registry = self.service_registry.clone();
        let subscription_registry = self.subscription_registry.clone();
        let pending_requests = self.pending_requests.clone();
        let remote_services = self.remote_services.clone();
        let hub_name = self.name.clone();

        tokio::spawn(async move {
            while let Ok(msg) = transport.recv().await {
                let _ = Self::process_message(
                    msg,
                    &service_registry,
                    &subscription_registry,
                    &pending_requests,
                    &remote_services,
                    &transport,
                    &hub_name,
                )
                .await;
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
        remote_services: &RwLock<std::collections::HashMap<String, ServiceInfo>>,
        transport: &Arc<dyn Transport>,
        hub_name: &str,
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
            MessageType::ServiceRegister => {
                // Handle remote service registration
                if let Ok(service_info) = bincode::deserialize::<ServiceInfo>(&msg.payload) {
                    tracing::info!(
                        "📋 Received service registration: {} from {}",
                        service_info.name,
                        service_info.process_name
                    );
                    let mut remote_services = remote_services.write().await;
                    for method in &service_info.methods {
                        let full_method = format!("{}.{}", service_info.name, method);
                        remote_services.insert(full_method.clone(), service_info.clone());
                        tracing::info!("📝 Registered remote method: {}", full_method);
                    }
                }
            }
            MessageType::ServiceQuery => {
                // Handle service query - respond with our local services
                tracing::info!("🔍 Received service query from {}", msg.source);
                if let Some(correlation_id) = msg.correlation_id {
                    let services = service_registry.list_services().await;
                    tracing::info!("📋 Local services available: {:?}", services);
                    let mut service_infos = Vec::new();

                    for service_name in services {
                        if let Some(service) = service_registry.get_service(&service_name).await {
                            let service_info = ServiceInfo {
                                name: service_name.clone(),
                                methods: service.methods().iter().map(|&s| s.to_string()).collect(),
                                process_name: hub_name.to_string(),
                                registered_at: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis()
                                    as u64,
                            };
                            service_infos.push(service_info);
                            tracing::info!(
                                "📤 Responding with service: {} methods={:?}",
                                service_name,
                                service.methods()
                            );
                        }
                    }

                    let directory = ServiceDirectory {
                        services: service_infos,
                    };

                    let response = Message::service_directory(
                        hub_name.to_string(),
                        msg.source.clone(),
                        directory,
                        Some(correlation_id),
                    );
                    let _ = transport.send(response).await;
                    tracing::info!("📬 Sent service directory to {}", msg.source);
                }
            }
            MessageType::ServiceDirectory => {
                // Handle service directory response
                tracing::info!("📬 Received service directory from {}", msg.source);
                if let Ok(directory) = bincode::deserialize::<ServiceDirectory>(&msg.payload) {
                    let mut remote_services = remote_services.write().await;
                    for service_info in directory.services {
                        for method in &service_info.methods {
                            let full_method = format!("{}.{}", service_info.name, method);
                            remote_services.insert(full_method.clone(), service_info.clone());
                            tracing::info!(
                                "📝 Learned remote method: {} from {}",
                                full_method,
                                service_info.process_name
                            );
                        }
                    }
                }
            }
            MessageType::SubscriptionRequest => {
                // Handle subscription request
                if let Some(ref topic) = msg.topic {
                    // Extract method name from topic (format: "subscription.{method}")
                    if let Some(method) = topic.strip_prefix("subscription.") {
                        // TODO: Create PendingSubscriptionSink and call subscription method
                        tracing::info!("📞 Received subscription request for method: {}", method);

                        // For now, just send a reject response
                        if let Some(correlation_id) = msg.correlation_id {
                            let reject_msg = Message::subscription_reject(
                                hub_name.to_string(),
                                msg.source.clone(),
                                correlation_id,
                                "Subscription not implemented yet".to_string(),
                            );
                            let _ = transport.send(reject_msg).await;
                        }
                    }
                }
            }
            MessageType::SubscriptionAccept => {
                // Handle subscription accept from server
                tracing::info!("✅ Subscription accepted from {}", msg.source);
                // TODO: Set up subscription receiver
            }
            MessageType::SubscriptionReject => {
                // Handle subscription reject from server
                tracing::info!("❌ Subscription rejected from {}", msg.source);
                // TODO: Handle subscription rejection
            }
            MessageType::SubscriptionData => {
                // Handle subscription data from server
                tracing::info!("📊 Received subscription data from {}", msg.source);
                // TODO: Forward data to subscription receiver
            }
            MessageType::SubscriptionCancel => {
                // Handle subscription cancel from client
                tracing::info!("🚫 Subscription cancelled from {}", msg.source);
                // TODO: Clean up subscription
            }
            _ => {}
        }

        Ok(())
    }

    /// Register a service
    pub async fn register_service<S: Service>(&self, service: S) -> Result<()> {
        let service_name = service.name().to_string();
        let methods: Vec<String> = service.methods().iter().map(|&s| s.to_string()).collect();

        // Register locally first
        self.service_registry
            .register(std::sync::Arc::new(service))
            .await?;

        // Broadcast service registration to other processes
        let service_info = ServiceInfo {
            name: service_name,
            methods,
            process_name: self.name.clone(),
            registered_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        let registration_msg = Message::service_register(self.name.clone(), service_info.clone());
        let _ = self.transport.send(registration_msg).await;
        // Give a small delay to ensure the broadcast message is sent
        tokio::time::sleep(Duration::from_millis(50)).await;
        tracing::info!(
            "📤 Broadcasted service registration: {} methods={:?}",
            service_info.name,
            service_info.methods
        );

        Ok(())
    }

    /// Call a service method
    pub async fn call<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        service_method: &str,
        request: T,
    ) -> Result<R> {
        // First try local service
        if let Ok(result) = self
            .service_registry
            .call(service_method, bincode::serialize(&request)?)
            .await
        {
            return Ok(bincode::deserialize(&result)?);
        }

        // If not found locally, check remote services
        let target_process = {
            let remote_services = self.remote_services.read().await;
            remote_services
                .get(service_method)
                .map(|info| info.process_name.clone())
        };

        // If we don't know about the service, query all processes
        if target_process.is_none() {
            tracing::info!(
                "🔍 Service {} not found locally, querying remote processes",
                service_method
            );
            let _ = self.query_services().await;
            // Wait longer for responses to address multi-process timing issues
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // Try to find the service again after query
        let target_process = {
            let remote_services = self.remote_services.read().await;
            remote_services
                .get(service_method)
                .map(|info| info.process_name.clone())
        };

        let payload = bincode::serialize(&request)?;
        let request_id = uuid::Uuid::new_v4();
        let msg = Message {
            id: request_id,
            msg_type: MessageType::Request,
            source: self.name.clone(),
            target: target_process, // Direct to specific process if known
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
            .map_err(|_| Error::timeout("service call", 30000))?
            .map_err(|_| Error::connection_msg("response channel closed"))?;

        match response.msg_type {
            MessageType::Response => {
                let result: R = bincode::deserialize(&response.payload)?;
                Ok(result)
            }
            MessageType::Error => {
                let error_msg = String::from_utf8_lossy(&response.payload);
                Err(Error::runtime_msg(format!("Remote error: {error_msg}")))
            }
            _ => Err(Error::protocol(
                "Unexpected response type",
                Some("Response or Error".to_string()),
                Some(format!("{:?}", response.msg_type)),
            )),
        }
    }

    /// Query remote services
    async fn query_services(&self) -> Result<()> {
        let query_msg = Message::service_query(self.name.clone(), None);
        self.transport.send(query_msg).await
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
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| Error::runtime_msg(format!("Failed to create runtime: {e}")))?;

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
            .map_err(|e| Error::runtime_msg(format!("Failed to create runtime: {e}")))?;

        Ok(Self { rt })
    }

    pub fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        self.rt.block_on(future)
    }
}
