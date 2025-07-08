//! Event trait and subscription system for publish/subscribe pattern

use crate::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Trait for events that can be published
pub trait Event: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {
    /// Get the topic for this event
    fn topic(&self) -> String;
}

/// Subscriber trait for handling events
#[async_trait]
pub trait Subscriber: Send + Sync + 'static {
    /// Topic pattern to subscribe to
    fn topic_pattern(&self) -> &str;

    /// Handle an event
    async fn handle(&mut self, topic: &str, payload: Vec<u8>) -> Result<()>;
}

/// Sync subscriber trait
pub trait SyncSubscriber: Send + Sync + 'static {
    /// Topic pattern to subscribe to
    fn topic_pattern(&self) -> &str;

    /// Handle an event synchronously
    fn handle_sync(&mut self, topic: &str, payload: Vec<u8>) -> Result<()>;
}

/// Subscription handle
pub struct Subscription {
    pub id: Uuid,
    pub topic_pattern: String,
    registry: Arc<SubscriptionRegistry>,
}

impl Subscription {
    /// Unsubscribe
    pub async fn unsubscribe(self) -> Result<()> {
        self.registry.unsubscribe(&self.id).await
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        // Queue unsubscribe for later processing
        let id = self.id;
        let registry = self.registry.clone();
        tokio::spawn(async move {
            let _ = registry.unsubscribe(&id).await;
        });
    }
}

/// Subscription registry
pub struct SubscriptionRegistry {
    /// Map of subscription ID to subscriber
    subscribers: Arc<DashMap<Uuid, Box<dyn Subscriber>>>,

    /// Map of topic pattern to subscription IDs
    topic_subscriptions: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
}

impl SubscriptionRegistry {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(DashMap::new()),
            topic_subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to a topic pattern
    pub async fn subscribe<S: Subscriber>(&self, subscriber: S) -> Result<Subscription> {
        let id = Uuid::new_v4();
        let topic_pattern = subscriber.topic_pattern().to_string();

        // Store subscriber
        self.subscribers.insert(id, Box::new(subscriber));

        // Update topic mapping
        let mut topics = self.topic_subscriptions.write().await;
        topics
            .entry(topic_pattern.clone())
            .or_insert_with(Vec::new)
            .push(id);

        Ok(Subscription {
            id,
            topic_pattern,
            registry: Arc::new(self.clone()),
        })
    }

    /// Unsubscribe
    pub async fn unsubscribe(&self, id: &Uuid) -> Result<()> {
        // Remove subscriber
        if let Some((_, subscriber)) = self.subscribers.remove(id) {
            let topic_pattern = subscriber.topic_pattern();

            // Remove from topic mapping
            let mut topics = self.topic_subscriptions.write().await;
            if let Some(subs) = topics.get_mut(topic_pattern) {
                subs.retain(|sub_id| sub_id != id);
                if subs.is_empty() {
                    topics.remove(topic_pattern);
                }
            }
        }

        Ok(())
    }

    /// Publish an event to matching subscribers
    pub async fn publish(&self, topic: &str, payload: Vec<u8>) -> Result<()> {
        let topics = self.topic_subscriptions.read().await;
        let mut matching_ids = Vec::new();

        // Find matching subscriptions
        for (pattern, ids) in topics.iter() {
            if topic_matches(topic, pattern) {
                matching_ids.extend(ids.iter().copied());
            }
        }
        drop(topics);

        // Deliver to subscribers
        for id in matching_ids {
            if let Some(mut subscriber) = self.subscribers.get_mut(&id) {
                // Clone payload for each subscriber
                let _ = subscriber.handle(topic, payload.clone()).await;
            }
        }

        Ok(())
    }
}

impl Clone for SubscriptionRegistry {
    fn clone(&self) -> Self {
        Self {
            subscribers: self.subscribers.clone(),
            topic_subscriptions: self.topic_subscriptions.clone(),
        }
    }
}

impl Default for SubscriptionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a topic matches a pattern
/// Supports wildcards: + (single level), # (multi level)
fn topic_matches(topic: &str, pattern: &str) -> bool {
    // Exact match
    if topic == pattern {
        return true;
    }

    let topic_parts: Vec<&str> = topic.split('/').collect();
    let pattern_parts: Vec<&str> = pattern.split('/').collect();

    let mut t_idx = 0;
    let mut p_idx = 0;

    while p_idx < pattern_parts.len() && t_idx < topic_parts.len() {
        match pattern_parts[p_idx] {
            "#" => return true, // Multi-level wildcard matches everything
            "+" => {
                // Single-level wildcard matches one part
                t_idx += 1;
                p_idx += 1;
            }
            part => {
                if part != topic_parts[t_idx] {
                    return false;
                }
                t_idx += 1;
                p_idx += 1;
            }
        }
    }

    // Both should be exhausted for a match
    t_idx == topic_parts.len() && p_idx == pattern_parts.len()
}

/// Adapter for sync subscribers
pub struct SyncSubscriberAdapter<S: SyncSubscriber> {
    inner: S,
}

impl<S: SyncSubscriber> SyncSubscriberAdapter<S> {
    pub fn new(subscriber: S) -> Self {
        Self { inner: subscriber }
    }
}

#[async_trait]
impl<S: SyncSubscriber> Subscriber for SyncSubscriberAdapter<S> {
    fn topic_pattern(&self) -> &str {
        self.inner.topic_pattern()
    }

    async fn handle(&mut self, _topic: &str, _payload: Vec<u8>) -> Result<()> {
        // Note: This is a simplified implementation
        // In practice, we'd need better integration between sync and async
        Ok(())
    }
}
