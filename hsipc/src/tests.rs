//! Integration tests for hsipc core functionality
//! Tests both pub/sub pattern and req/resp pattern

use crate::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

// Test event for pub/sub pattern
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestEvent {
    pub message: String,
    pub value: i32,
}

impl Event for TestEvent {
    fn topic(&self) -> String {
        "test/event".to_string()
    }
}

// Test subscriber for pub/sub pattern
#[derive(Debug)]
pub struct TestSubscriber {
    pub received: Arc<Mutex<Vec<TestEvent>>>,
}

impl TestSubscriber {
    pub fn new() -> Self {
        Self {
            received: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl Subscriber for TestSubscriber {
    fn topic_pattern(&self) -> &str {
        "test/#"
    }

    async fn handle(&mut self, _topic: &str, payload: Vec<u8>) -> Result<()> {
        if let Ok(event) = bincode::deserialize::<TestEvent>(&payload) {
            self.received.lock().await.push(event);
        }
        Ok(())
    }
}

// Test service for req/resp pattern using manual Service implementation
pub struct Calculator;

#[async_trait::async_trait]
impl Service for Calculator {
    fn name(&self) -> &'static str {
        "CalculatorService"
    }

    fn methods(&self) -> Vec<&'static str> {
        vec!["add", "multiply"]
    }

    async fn handle(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
        match method {
            "add" => {
                let (a, b): (i32, i32) = bincode::deserialize(&payload)?;
                let result = a + b;
                Ok(bincode::serialize(&result)?)
            }
            "multiply" => {
                let (a, b): (i32, i32) = bincode::deserialize(&payload)?;
                let result = a * b;
                Ok(bincode::serialize(&result)?)
            }
            _ => Err(Error::Other(anyhow::anyhow!("Unknown method: {}", method))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_pubsub_pattern_with_events() {
        println!("Starting pub/sub test");

        // Create one hub for testing (since we're using shared message bus)
        let hub = ProcessHub::new("test_hub").await.unwrap();

        // Create and register subscriber
        let subscriber = TestSubscriber::new();
        let received_events = subscriber.received.clone();

        let _subscription = hub.subscribe(subscriber).await.unwrap();
        println!("Subscriber registered");

        // Wait a bit for subscription to be ready
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Publish some events using direct topic publishing (simpler)
        hub.publish(
            "test/event",
            TestEvent {
                message: "Hello World".to_string(),
                value: 42,
            },
        )
        .await
        .unwrap();

        hub.publish(
            "test/event",
            TestEvent {
                message: "Test Event".to_string(),
                value: 100,
            },
        )
        .await
        .unwrap();

        println!("Events published");

        // Wait for events to be processed
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Verify events were received
        let received = received_events.lock().await;
        println!("Received {} events", received.len());

        if !received.is_empty() {
            println!("First event: {:?}", received[0]);
        }

        assert!(!received.is_empty(), "Should receive at least 1 event");

        // Cleanup
        hub.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_request_response_pattern_with_service() {
        println!("Starting req/resp test");

        // Create hub for testing
        let hub = ProcessHub::new("test_hub").await.unwrap();

        // Register the calculator service
        let calculator = Calculator;
        hub.register_service(calculator).await.unwrap();
        println!("Calculator service registered");

        // Wait for service to be ready
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Test add operation
        println!("Testing add operation");
        let add_result: i32 = timeout(
            Duration::from_secs(5),
            hub.call("CalculatorService.add", (10, 5)),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(add_result, 15);
        println!("Add test passed: 10 + 5 = {add_result}");

        // Test multiply operation
        println!("Testing multiply operation");
        let multiply_result: i32 = timeout(
            Duration::from_secs(5),
            hub.call("CalculatorService.multiply", (6, 7)),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(multiply_result, 42);
        println!("Multiply test passed: 6 * 7 = {multiply_result}");

        // Cleanup
        hub.shutdown().await.unwrap();
    }
}
