//! Service trait and registry for request/response pattern

use crate::{Error, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Base trait for services
#[async_trait]
pub trait Service: Send + Sync + 'static {
    /// Service name
    fn name(&self) -> &'static str;

    /// List of methods this service provides
    fn methods(&self) -> Vec<&'static str>;

    /// Handle a request
    async fn handle(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>>;
}

/// Service method handler
pub type MethodHandler =
    Arc<dyn Fn(Vec<u8>) -> futures::future::BoxFuture<'static, Result<Vec<u8>>> + Send + Sync>;

/// Type alias for service map to reduce complexity
type ServiceMap = Arc<RwLock<HashMap<String, Arc<dyn Service>>>>;

/// Type alias for method map to reduce complexity  
type MethodMap = Arc<RwLock<HashMap<String, MethodHandler>>>;

/// Service registry for managing services
pub struct ServiceRegistry {
    services: ServiceMap,
    methods: MethodMap,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            methods: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service
    pub async fn register<S: Service>(&self, service: S) -> Result<()> {
        let service = Arc::new(service);
        let service_name = service.name();

        // Register service
        self.services
            .write()
            .await
            .insert(service_name.to_string(), service.clone());

        // Register methods
        let mut methods = self.methods.write().await;
        for method_name in service.methods() {
            let full_name = format!("{service_name}.{method_name}");
            let service_clone = service.clone();
            let method_name_clone = method_name.to_string();

            let handler: MethodHandler = Arc::new(move |payload| {
                let service = service_clone.clone();
                let method = method_name_clone.clone();
                Box::pin(async move { service.handle(&method, payload).await })
            });

            methods.insert(full_name, handler);
        }

        Ok(())
    }

    /// Call a service method
    pub async fn call(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
        let methods = self.methods.read().await;
        let handler = methods
            .get(method)
            .ok_or_else(|| Error::MethodNotFound(method.to_string()))?;

        handler(payload).await
    }

    /// Get a service by name
    pub async fn get_service(&self, name: &str) -> Option<Arc<dyn Service>> {
        self.services.read().await.get(name).cloned()
    }

    /// List all registered services
    pub async fn list_services(&self) -> Vec<String> {
        self.services.read().await.keys().cloned().collect()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Sync service trait for synchronous handlers
pub trait SyncService: Send + Sync + 'static {
    /// Service name
    fn name(&self) -> &'static str;

    /// List of methods this service provides
    fn methods(&self) -> Vec<&'static str>;

    /// Handle a request synchronously
    fn handle_sync(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>>;
}

/// Adapter to use SyncService as async Service
pub struct SyncServiceAdapter<S: SyncService> {
    inner: S,
}

impl<S: SyncService> SyncServiceAdapter<S> {
    pub fn new(service: S) -> Self {
        Self { inner: service }
    }
}

#[async_trait]
impl<S: SyncService> Service for SyncServiceAdapter<S> {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn methods(&self) -> Vec<&'static str> {
        self.inner.methods()
    }

    async fn handle(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
        // Run sync handler in blocking task
        let method = method.to_string();
        let inner = Arc::new(self.inner.handle_sync(&method, payload)?);

        tokio::task::spawn_blocking(move || Ok(inner.as_ref().clone()))
            .await
            .map_err(|e| Error::Runtime(e.to_string()))?
    }
}
