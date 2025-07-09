//! RPC subscription system implementation
//!
//! This module provides the core types for the RPC subscription system,
//! following the jsonrpsee pattern with PendingSubscriptionSink.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Pending subscription sink that can be accepted or rejected
///
/// This follows the jsonrpsee pattern where subscription requests
/// can be conditionally accepted or rejected with a reason.
pub struct PendingSubscriptionSink {
    id: Uuid,
    method: String,
    sender: Option<mpsc::UnboundedSender<serde_json::Value>>,
}

impl PendingSubscriptionSink {
    /// Create a new pending subscription sink
    pub fn new(
        id: Uuid,
        method: String,
        sender: mpsc::UnboundedSender<serde_json::Value>,
    ) -> Self {
        Self {
            id,
            method,
            sender: Some(sender),
        }
    }

    /// Accept the subscription and return a SubscriptionSink
    ///
    /// This consumes the PendingSubscriptionSink and returns a SubscriptionSink
    /// that can be used to send data to the subscriber.
    pub async fn accept(mut self) -> Result<SubscriptionSink> {
        let sender = self
            .sender
            .take()
            .ok_or_else(|| Error::runtime_msg("Subscription already accepted or rejected"))?;

        // TODO: Send accept message to client through ProcessHub
        log::trace!("Subscription {} accepted for method {}", self.id, self.method);

        Ok(SubscriptionSink::new(self.id, sender, self.method))
    }

    /// Reject the subscription with a reason
    ///
    /// This will send a rejection message to the client and consume
    /// the PendingSubscriptionSink.
    pub async fn reject(self, reason: String) -> Result<()> {
        // TODO: Send reject message to client through ProcessHub
        log::trace!("Subscription {} rejected for method {}: {}", self.id, self.method, reason);
        
        // Drop the sender to close the channel
        drop(self.sender);
        
        Err(Error::runtime_msg(format!("Subscription rejected: {}", reason)))
    }

    /// Get the subscription ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the method name
    pub fn method(&self) -> &str {
        &self.method
    }
}

/// Active subscription sink for sending data to subscribers
///
/// This is returned by PendingSubscriptionSink::accept() and provides
/// methods to send JSON values to the subscribing client.
pub struct SubscriptionSink {
    id: Uuid,
    sender: mpsc::UnboundedSender<serde_json::Value>,
    method: String,
}

impl SubscriptionSink {
    /// Create a new subscription sink
    pub(crate) fn new(
        id: Uuid,
        sender: mpsc::UnboundedSender<serde_json::Value>,
        method: String,
    ) -> Self {
        Self { id, sender, method }
    }

    /// Send a JSON value to the subscriber
    ///
    /// This is the low-level method for sending pre-serialized JSON.
    pub async fn send(&self, value: serde_json::Value) -> Result<()> {
        self.sender
            .send(value)
            .map_err(|_| Error::runtime_msg("Subscription channel closed"))?;
        Ok(())
    }

    /// Send a serializable value to the subscriber
    ///
    /// This is a convenience method that serializes the value to JSON
    /// before sending it.
    pub async fn send_value<T: Serialize>(&self, value: T) -> Result<()> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| Error::runtime_msg(format!("Failed to serialize value: {}", e)))?;
        self.send(json_value).await
    }

    /// Check if the subscription is still active
    ///
    /// Returns true if the subscriber is still connected and able to receive data.
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    /// Get the subscription ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the method name
    pub fn method(&self) -> &str {
        &self.method
    }
}

/// Client-side subscription handle for receiving data
///
/// This is the client-side counterpart to SubscriptionSink.
/// It provides methods to receive deserialized data from the server.
pub struct RpcSubscription<T> {
    id: Uuid,
    receiver: mpsc::UnboundedReceiver<serde_json::Value>,
    _phantom: PhantomData<T>,
}

impl<T> RpcSubscription<T>
where
    T: for<'de> Deserialize<'de>,
{
    /// Create a new RPC subscription
    pub fn new(id: Uuid, receiver: mpsc::UnboundedReceiver<serde_json::Value>) -> Self {
        Self {
            id,
            receiver,
            _phantom: PhantomData,
        }
    }

    /// Receive the next value from the subscription
    ///
    /// Returns None if the subscription has been closed or canceled.
    /// Returns Some(Err(_)) if there was an error deserializing the data.
    pub async fn next(&mut self) -> Option<Result<T>> {
        match self.receiver.recv().await {
            Some(json_value) => {
                match serde_json::from_value(json_value) {
                    Ok(value) => Some(Ok(value)),
                    Err(e) => Some(Err(Error::runtime_msg(format!(
                        "Failed to deserialize subscription data: {}", e
                    )))),
                }
            }
            None => None,
        }
    }

    /// Cancel the subscription
    ///
    /// This will close the receiver and notify the server that the
    /// subscription should be canceled.
    pub async fn cancel(self) -> Result<()> {
        // TODO: Send cancel message to server through ProcessHub
        log::trace!("Subscription {} canceled", self.id);
        
        // Closing the receiver will signal the server that we're done
        drop(self.receiver);
        Ok(())
    }

    /// Get the subscription ID
    pub fn id(&self) -> Uuid {
        self.id
    }
}

/// Subscription message types for internal communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionMessage {
    /// Request to create a new subscription
    Request {
        id: Uuid,
        method: String,
        params: serde_json::Value,
    },
    /// Subscription was accepted by the server
    Accept {
        id: Uuid,
    },
    /// Subscription was rejected by the server
    Reject {
        id: Uuid,
        reason: String,
    },
    /// Data sent from server to client
    Data {
        id: Uuid,
        data: serde_json::Value,
    },
    /// Subscription was canceled
    Cancel {
        id: Uuid,
    },
}

impl SubscriptionMessage {
    /// Get the subscription ID for this message
    pub fn id(&self) -> Uuid {
        match self {
            SubscriptionMessage::Request { id, .. } => *id,
            SubscriptionMessage::Accept { id } => *id,
            SubscriptionMessage::Reject { id, .. } => *id,
            SubscriptionMessage::Data { id, .. } => *id,
            SubscriptionMessage::Cancel { id } => *id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_pending_subscription_accept() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let pending = PendingSubscriptionSink::new(
            Uuid::new_v4(),
            "test_method".to_string(),
            tx,
        );

        let sink = pending.accept().await.unwrap();
        assert_eq!(sink.method(), "test_method");

        // Test sending data
        sink.send_value("test_data").await.unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received, json!("test_data"));
    }

    #[tokio::test]
    async fn test_pending_subscription_reject() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let pending = PendingSubscriptionSink::new(
            Uuid::new_v4(),
            "test_method".to_string(),
            tx,
        );

        let result = pending.reject("Invalid parameters".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rpc_subscription() {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut subscription: RpcSubscription<String> = RpcSubscription::new(
            Uuid::new_v4(),
            rx,
        );

        // Send data through the channel
        tx.send(json!("test_message")).unwrap();
        
        // Receive and verify
        let received = subscription.next().await.unwrap().unwrap();
        assert_eq!(received, "test_message");
    }

    #[tokio::test]
    async fn test_subscription_closed() {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut subscription: RpcSubscription<String> = RpcSubscription::new(
            Uuid::new_v4(),
            rx,
        );

        // Drop sender to close the channel
        drop(tx);

        // Should return None when closed
        let result = subscription.next().await;
        assert!(result.is_none());
    }
}