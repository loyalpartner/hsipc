//! Real IPMB-based transport for cross-process communication

use crate::transport::Transport;
use crate::{Error, Message, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

/// IPMB message wrapper for our Message type
#[derive(Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "7b07473e-9659-4d47-a502-8245d71c0078"]
struct IpmbMessage {
    inner: Message,
}

/// IPMB-based transport implementation
pub struct IpmbTransport {
    sender: ipmb::EndpointSender<IpmbMessage>,
    process_name: String,
}

/// Separate receiver for message processing
pub struct IpmbReceiver {
    receiver: ipmb::EndpointReceiver<IpmbMessage>,
}

impl IpmbReceiver {
    /// Receive a message
    pub async fn recv(&mut self) -> Result<Message> {
        let timeout = std::time::Duration::from_secs(30);
        match self.receiver.recv(Some(timeout)) {
            Ok(ipmb_msg) => Ok(ipmb_msg.payload.inner),
            Err(e) => Err(Error::transport_msg(format!("IPMB recv failed: {e}"))),
        }
    }
}

impl IpmbTransport {
    /// Create a new IPMB transport
    pub async fn new(process_name: &str) -> Result<Self> {
        let (transport, _receiver) = Self::new_with_receiver(process_name).await?;
        Ok(transport)
    }

    /// Create a new IPMB transport with separate receiver
    pub async fn new_with_receiver(process_name: &str) -> Result<(Self, IpmbReceiver)> {
        // Create a process-specific bus name for testing to avoid conflicts
        let bus_name = {
            #[cfg(test)]
            {
                // For tests, use a unique bus name per test process
                // but allow multiple instances within the same process
                let pid = std::process::id();
                format!("com.hsipc.test.{pid}")
            }
            #[cfg(not(test))]
            {
                // For production, use a shared bus
                "com.hsipc.bus".to_string()
            }
        };

        // Create IPMB options
        let options = ipmb::Options::new(&bus_name, ipmb::label!(process_name), "");

        // Join the IPMB bus
        tracing::info!(
            "🔌 Attempting to join IPMB bus: {} with process: {}",
            bus_name,
            process_name
        );
        #[cfg(test)]
        {
            tracing::info!("🧪 Test process {} using bus: {}", process_name, bus_name);
        }
        let (sender, receiver) =
            ipmb::join::<IpmbMessage, IpmbMessage>(options, None).map_err(|e| {
                tracing::error!("❌ IPMB join failed for bus {}: {}", bus_name, e);
                Error::transport_msg(format!("IPMB join failed: {e}"))
            })?;

        #[cfg(test)]
        {
            tracing::info!(
                "🧪 Joined test IPMB bus {} as process: {}",
                bus_name,
                process_name
            );
        }
        #[cfg(not(test))]
        {
            tracing::info!("🚌 Joined IPMB bus as process: {}", process_name);
        }

        let transport = Self {
            sender,
            process_name: process_name.to_string(),
        };

        let ipmb_receiver = IpmbReceiver { receiver };

        Ok((transport, ipmb_receiver))
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

        match self.sender.send(ipmb_message) {
            Ok(()) => {
                tracing::debug!(
                    "✅ Successfully sent IPMB message type: {:?} id: {} to {:?}",
                    msg.msg_type,
                    msg.id,
                    msg.target
                );
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("IPMB send failed: {e}");

                // Log detailed error information
                tracing::error!("❌ IPMB send error: {}", error_msg);
                tracing::error!(
                    "📧 Failed message details: type={:?} id={} target={:?} process={}",
                    msg.msg_type,
                    msg.id,
                    msg.target,
                    self.process_name
                );

                // Be more conservative about ignoring errors
                if error_msg.contains("Invalid argument") {
                    tracing::error!(
                        "🚨 IPMB send error (CRITICAL): {} - This may cause message loss!",
                        error_msg
                    );
                    tracing::error!(
                        "🚨 System information: process={} bus_name likely contains process name",
                        self.process_name
                    );

                    // For subscription requests, this is critical - don't ignore
                    if msg.msg_type == crate::message::MessageType::SubscriptionRequest {
                        return Err(Error::transport_msg(format!(
                            "Critical IPMB error for subscription: {error_msg}"
                        )));
                    }

                    // Still treat other messages as warnings for now
                    Ok(())
                } else {
                    tracing::error!("❌ IPMB send failed: {}", error_msg);
                    Err(Error::transport_msg(error_msg))
                }
            }
        }
    }

    async fn recv(&self) -> Result<Message> {
        // This method is not used anymore, as receiver is separated
        Err(Error::transport_msg(
            "recv not supported on IpmbTransport, use IpmbReceiver instead",
        ))
    }

    async fn close(&self) -> Result<()> {
        // IPMB handles cleanup automatically
        tracing::info!("🚌 Closing IPMB transport for {}", self.process_name);
        Ok(())
    }
}
