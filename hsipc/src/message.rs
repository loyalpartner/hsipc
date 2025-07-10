//! Message types and serialization

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Core message type for IPC communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    pub id: Uuid,

    /// Message type (request, response, event, etc.)
    pub msg_type: MessageType,

    /// Source process name
    pub source: String,

    /// Target process name (optional for broadcasts)
    pub target: Option<String>,

    /// Topic for pub/sub messages
    pub topic: Option<String>,

    /// Serialized payload
    pub payload: Vec<u8>,

    /// Optional correlation ID for request/response
    pub correlation_id: Option<Uuid>,

    /// Message metadata
    pub metadata: MessageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Subscribe,
    Unsubscribe,
    Heartbeat,
    Error,
    // Service discovery messages
    ServiceRegister,
    ServiceUnregister,
    ServiceQuery,
    ServiceDirectory,
    // Subscription protocol messages
    SubscriptionRequest,
    SubscriptionAccept,
    SubscriptionReject,
    SubscriptionData,
    SubscriptionCancel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Timestamp when message was created
    pub timestamp: u64,

    /// Message priority
    pub priority: Priority,

    /// Time-to-live in milliseconds
    pub ttl: Option<u64>,

    /// Whether to retain this message
    pub retain: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
}

impl Message {
    /// Create a new request message
    pub fn request(source: String, target: String, method: String, payload: Vec<u8>) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            msg_type: MessageType::Request,
            source,
            target: Some(target),
            topic: Some(method),
            payload,
            correlation_id: Some(id),
            metadata: MessageMetadata::default(),
        }
    }

    /// Create a new response message
    pub fn response(request: &Message, payload: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            msg_type: MessageType::Response,
            source: "service".to_string(), // Response from service
            target: Some(request.source.clone()),
            topic: request.topic.clone(),
            payload,
            correlation_id: request.correlation_id,
            metadata: MessageMetadata::default(),
        }
    }

    /// Create a new event message
    pub fn event(source: String, topic: String, payload: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            msg_type: MessageType::Event,
            source,
            target: None,
            topic: Some(topic),
            payload,
            correlation_id: None,
            metadata: MessageMetadata::default(),
        }
    }
}

impl Default for MessageMetadata {
    fn default() -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            priority: Priority::Normal,
            ttl: None,
            retain: false,
        }
    }
}

/// Request wrapper for type safety
#[derive(Debug, Serialize, Deserialize)]
pub struct Request<T> {
    pub method: String,
    pub params: T,
}

/// Response wrapper for type safety
#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub result: Result<T, String>,
}

/// Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub methods: Vec<String>,
    pub process_name: String,
    pub registered_at: u64,
}

/// Service directory containing all available services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDirectory {
    pub services: Vec<ServiceInfo>,
}

impl Message {
    /// Create a service registration message
    pub fn service_register(source: String, service_info: ServiceInfo) -> Self {
        let payload = bincode::serialize(&service_info).unwrap_or_default();
        Self {
            id: Uuid::new_v4(),
            msg_type: MessageType::ServiceRegister,
            source,
            target: None, // Broadcast to all processes
            topic: Some("service.register".to_string()),
            payload,
            correlation_id: None,
            metadata: MessageMetadata::default(),
        }
    }

    /// Create a service query message
    pub fn service_query(source: String, service_name: Option<String>) -> Self {
        let payload = bincode::serialize(&service_name).unwrap_or_default();
        Self {
            id: Uuid::new_v4(),
            msg_type: MessageType::ServiceQuery,
            source,
            target: None, // Broadcast to all processes
            topic: Some("service.query".to_string()),
            payload,
            correlation_id: Some(Uuid::new_v4()),
            metadata: MessageMetadata::default(),
        }
    }

    /// Create a service directory response message
    pub fn service_directory(
        source: String,
        target: String,
        directory: ServiceDirectory,
        correlation_id: Option<Uuid>,
    ) -> Self {
        let payload = bincode::serialize(&directory).unwrap_or_default();
        Self {
            id: Uuid::new_v4(),
            msg_type: MessageType::ServiceDirectory,
            source,
            target: Some(target),
            topic: Some("service.directory".to_string()),
            payload,
            correlation_id,
            metadata: MessageMetadata::default(),
        }
    }

    /// Create a subscription request message
    pub fn subscription_request(
        source: String,
        target: Option<String>,
        method: String,
        params: Vec<u8>,
    ) -> Self {
        let subscription_msg = crate::subscription::SubscriptionMessage::Request {
            id: Uuid::new_v4(),
            method: method.clone(),
            params: params.clone(),
        };
        let payload = bincode::serialize(&subscription_msg).unwrap_or_default();

        Self {
            id: Uuid::new_v4(),
            msg_type: MessageType::SubscriptionRequest,
            source,
            target,
            topic: Some(format!("subscription.{method}")),
            payload,
            correlation_id: Some(subscription_msg.id()),
            metadata: MessageMetadata::default(),
        }
    }

    /// Create a subscription accept message
    pub fn subscription_accept(source: String, target: String, subscription_id: Uuid) -> Self {
        let subscription_msg = crate::subscription::SubscriptionMessage::Accept {
            id: subscription_id,
        };
        let payload = bincode::serialize(&subscription_msg).unwrap_or_default();

        Self {
            id: Uuid::new_v4(),
            msg_type: MessageType::SubscriptionAccept,
            source,
            target: Some(target),
            topic: Some("subscription.accept".to_string()),
            payload,
            correlation_id: Some(subscription_id),
            metadata: MessageMetadata::default(),
        }
    }

    /// Create a subscription reject message
    pub fn subscription_reject(
        source: String,
        target: String,
        subscription_id: Uuid,
        reason: String,
    ) -> Self {
        let subscription_msg = crate::subscription::SubscriptionMessage::Reject {
            id: subscription_id,
            reason,
        };
        let payload = bincode::serialize(&subscription_msg).unwrap_or_default();

        Self {
            id: Uuid::new_v4(),
            msg_type: MessageType::SubscriptionReject,
            source,
            target: Some(target),
            topic: Some("subscription.reject".to_string()),
            payload,
            correlation_id: Some(subscription_id),
            metadata: MessageMetadata::default(),
        }
    }

    /// Create a subscription data message
    pub fn subscription_data(
        source: String,
        target: String,
        subscription_id: Uuid,
        data: serde_json::Value,
    ) -> Self {
        let data_bytes = serde_json::to_vec(&data).unwrap_or_default();
        let subscription_msg = crate::subscription::SubscriptionMessage::Data {
            id: subscription_id,
            data: data_bytes,
        };
        let payload = bincode::serialize(&subscription_msg).unwrap_or_default();

        Self {
            id: Uuid::new_v4(),
            msg_type: MessageType::SubscriptionData,
            source,
            target: Some(target),
            topic: Some("subscription.data".to_string()),
            payload,
            correlation_id: Some(subscription_id),
            metadata: MessageMetadata::default(),
        }
    }

    /// Create a subscription cancel message
    pub fn subscription_cancel(source: String, target: String, subscription_id: Uuid) -> Self {
        let subscription_msg = crate::subscription::SubscriptionMessage::Cancel {
            id: subscription_id,
        };
        let payload = bincode::serialize(&subscription_msg).unwrap_or_default();

        Self {
            id: Uuid::new_v4(),
            msg_type: MessageType::SubscriptionCancel,
            source,
            target: Some(target),
            topic: Some("subscription.cancel".to_string()),
            payload,
            correlation_id: Some(subscription_id),
            metadata: MessageMetadata::default(),
        }
    }
}
