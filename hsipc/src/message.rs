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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Subscribe,
    Unsubscribe,
    Heartbeat,
    Error,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Default)]
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
