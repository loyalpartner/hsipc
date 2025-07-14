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

impl IpmbTransport {
    /// Create a builder for IpmbTransport
    pub fn builder(process_name: impl Into<String>) -> IpmbTransportBuilder {
        IpmbTransportBuilder::new(process_name)
    }

    /// Create a new IPMB transport with default settings (for backward compatibility)
    pub async fn new(process_name: &str) -> Result<Self> {
        Self::builder(process_name).build().await
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
            guard.recv(None)  // No timeout, true blocking wait
        })
        .await
        .map_err(|e| Error::runtime_msg(format!("Blocking task failed: {e}")))?;

        match recv_result {
            Ok(ipmb_msg) => {
                let msg = ipmb_msg.payload.inner;
                
                // Check if this is a shutdown message
                if matches!(msg.msg_type, crate::message::MessageType::Shutdown) {
                    tracing::info!("🛑 Received shutdown message");
                    return Err(Error::connection_msg("Transport closed"));
                }
                
                Ok(msg)
            }
            Err(e) => {
                // Real transport error
                Err(Error::transport_msg(format!("IPMB recv failed: {e}")))
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
            target: Some(self.process_name.clone()), // Send to self
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
