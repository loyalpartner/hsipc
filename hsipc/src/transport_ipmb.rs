//! Real IPMB-based transport for cross-process communication

use crate::transport::Transport;
use crate::{Error, Message, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use type_uuid::TypeUuid;

/// IPMB message wrapper for our Message type
#[derive(Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "7b07473e-9659-4d47-a502-8245d71c0078"]
struct IpmbMessage {
    inner: Message,
}

/// IPMB-based transport implementation with integrated receiver
pub struct IpmbTransport {
    sender: ipmb::EndpointSender<IpmbMessage>,
    receiver: std::sync::Arc<std::sync::Mutex<ipmb::EndpointReceiver<IpmbMessage>>>,
    process_name: String,
}

/// Builder for IpmbTransport
pub struct IpmbTransportBuilder {
    process_name: String,
    bus_name: Option<String>,
}

impl IpmbTransportBuilder {
    /// Create a new builder with process name
    pub fn new(process_name: impl Into<String>) -> Self {
        Self {
            process_name: process_name.into(),
            bus_name: None,
        }
    }

    /// Set custom bus name (optional)
    pub fn with_bus_name(mut self, bus_name: impl Into<String>) -> Self {
        self.bus_name = Some(bus_name.into());
        self
    }

    /// Build the IpmbTransport
    pub async fn build(self) -> Result<IpmbTransport> {
        let bus_name = self.bus_name.unwrap_or_else(|| "com.hsipc.bus".to_string());
        let process_name = &self.process_name;
        // Create IPMB options
        let options = ipmb::Options::new(&bus_name, ipmb::label!(process_name), "");

        // Join the IPMB bus
        tracing::info!(
            "🔌 Attempting to join IPMB bus: {} with process: {}",
            bus_name,
            process_name
        );
        let (sender, receiver) =
            ipmb::join::<IpmbMessage, IpmbMessage>(options, None).map_err(|e| {
                tracing::error!("❌ IPMB join failed for bus {}: {}", bus_name, e);
                Error::transport_msg(format!("IPMB join failed: {e}"))
            })?;

        tracing::info!(
            "🚌 Joined IPMB bus {} as process: {}",
            bus_name,
            process_name
        );

        Ok(IpmbTransport {
            sender,
            receiver: std::sync::Arc::new(std::sync::Mutex::new(receiver)),
            process_name: self.process_name,
        })
    }
}

/// Classification of IPMB errors for type-safe handling
#[derive(Debug, Clone, PartialEq)]
enum IpmbErrorType {
    Timeout,
    ConnectionLost,
    BusError,
    SerializationError,
    Other,
}

impl IpmbTransport {
    /// Create a builder for IpmbTransport
    pub fn builder(process_name: impl Into<String>) -> IpmbTransportBuilder {
        IpmbTransportBuilder::new(process_name)
    }

    /// Create a new IPMB transport with default settings (for backward compatibility)
    pub async fn new(process_name: &str) -> Result<Self> {
        Self::builder(process_name).build().await
    }

    /// Classify IPMB errors for type-safe error handling
    fn classify_ipmb_error(error: &ipmb::RecvError) -> IpmbErrorType {
        use std::error::Error as StdError;
        
        // First check the error type directly if available
        let error_str = error.to_string().to_lowercase();
        
        // Check for timeout patterns
        if error_str.contains("timeout") || 
           error_str.contains("timed out") ||
           error.source().map(|s| {
               let source_str = s.to_string().to_lowercase();
               source_str.contains("timeout") || source_str.contains("timed out")
           }).unwrap_or(false) {
            return IpmbErrorType::Timeout;
        }
        
        // Check for connection/bus errors
        if error_str.contains("connection") ||
           error_str.contains("disconnected") ||
           error_str.contains("bus") ||
           error_str.contains("endpoint") {
            return IpmbErrorType::ConnectionLost;
        }
        
        // Check for serialization errors
        if error_str.contains("serialize") ||
           error_str.contains("deserialize") ||
           error_str.contains("encoding") ||
           error_str.contains("decoding") {
            return IpmbErrorType::SerializationError;
        }
        
        // Check for bus-specific errors
        if error_str.contains("bus error") ||
           error_str.contains("message queue") ||
           error_str.contains("channel") {
            return IpmbErrorType::BusError;
        }
        
        IpmbErrorType::Other
    }
}

#[async_trait]
impl Transport for IpmbTransport {
    async fn send(&self, msg: Message) -> Result<()> {
        let ipmb_msg = IpmbMessage { inner: msg.clone() };

        let selector = if let Some(ref target) = msg.target {
            // Send to specific process using label match
            ipmb::Selector::unicast(ipmb::LabelOp::from(target.as_str()))
        } else {
            // Broadcast to all processes - use multicast with True to match all endpoints
            ipmb::Selector::multicast(ipmb::LabelOp::True)
        };

        let ipmb_message = ipmb::Message::new(selector, ipmb_msg);

        // Try to send with retry on certain errors
        tracing::debug!(
            "📤 Attempting to send IPMB message: type={:?} id={} target={:?} process={}",
            msg.msg_type,
            msg.id,
            msg.target,
            self.process_name
        );

        self.sender.send(ipmb_message).map_err(|e| {
            tracing::error!("Failed to send IPMB message: {e}");
            Error::transport_msg(format!("Failed to send IPMB message: {e}"))
        })
    }

    async fn recv(&self) -> Result<Message> {
        // ipmb's recv is synchronous and blocking, so we need to run it in a blocking thread
        // to avoid blocking the async runtime

        let receiver = Arc::clone(&self.receiver);

        // Use spawn_blocking to run the synchronous recv in a separate thread
        let recv_result = tokio::task::spawn_blocking(move || {
            let mut guard = receiver.lock().unwrap();
            // Set a short timeout for recv to allow graceful shutdown while maintaining responsiveness
            let timeout = Some(std::time::Duration::from_millis(100)); // 100ms timeout
            guard.recv(timeout)
        })
        .await
        .map_err(|e| Error::runtime_msg(format!("Blocking task failed: {e}")))?;

        match recv_result {
            Ok(ipmb_msg) => {
                let msg = ipmb_msg.payload.inner;

                // Check if this is a shutdown message for us
                if matches!(msg.msg_type, crate::message::MessageType::Shutdown) {
                    if msg.source == self.process_name {
                        tracing::info!("🛑 Received shutdown message for {}", self.process_name);
                        return Err(Error::connection_msg("Transport closed"));
                    }
                }

                Ok(msg)
            }
            Err(e) => {
                // Type-safe error detection based on IPMB error types  
                let error_classification = Self::classify_ipmb_error(&e);
                
                match error_classification {
                    IpmbErrorType::Timeout => {
                        // Timeout is expected, return a special timeout error to let message loop continue
                        Err(Error::timeout_msg("IPMB recv timeout - continue loop"))
                    }
                    IpmbErrorType::ConnectionLost => {
                        Err(Error::connection_msg("IPMB connection lost"))
                    }
                    IpmbErrorType::BusError => {
                        Err(Error::transport("IPMB bus error", e))
                    }
                    IpmbErrorType::SerializationError => {
                        Err(Error::serialization("IPMB message serialization failed", e))
                    }
                    IpmbErrorType::Other => {
                        Err(Error::transport("IPMB recv failed", e))
                    }
                }
            }
        }
    }

    async fn close(&self) -> Result<()> {
        tracing::info!("🚌 Closing IPMB transport for {}", self.process_name);

        // Send a shutdown message to wake up any blocking recv operations
        let shutdown_msg = crate::Message {
            id: uuid::Uuid::new_v4(),
            msg_type: crate::message::MessageType::Shutdown,
            source: self.process_name.clone(),
            target: Some("service_provider".into()), // Send to self
            topic: None,
            payload: vec![],
            correlation_id: None,
            metadata: Default::default(),
        };

        // Send shutdown message, but don't fail if it doesn't work
        if let Err(e) = self.send(shutdown_msg).await {
            tracing::warn!("Failed to send shutdown message: {}", e);
            // Continue with shutdown anyway
        }

        Ok(())
    }
}
