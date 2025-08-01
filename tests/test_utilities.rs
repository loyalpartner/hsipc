//! Test utilities and helpers for hsipc testing
//!
//! This module provides common utilities, fixtures, and helpers to make
//! test development easier and more consistent across the test suite.

use hsipc::{ProcessHub, Result as HsipcResult, Service, Subscriber, async_trait};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

/// Test environment builder for consistent test setup
pub struct TestEnvironment {
    pub hub: ProcessHub,
    pub process_name: String,
    pub bus_name: String,
}

impl TestEnvironment {
    /// Create a new test environment with unique naming
    pub async fn new(test_name: &str) -> HsipcResult<Self> {
        let process_id = std::process::id();
        let process_name = format!("test_{}_{}", test_name, process_id);
        let bus_name = format!("com.hsipc.test.{}.{}", test_name, process_id);

        let hub = ProcessHub::builder(&process_name)
            .with_bus_name(&bus_name)
            .with_fast_mode(true)
            .build()
            .await?;

        Ok(Self {
            hub,
            process_name,
            bus_name,
        })
    }

    /// Create a test environment with custom configuration
    pub async fn with_config(test_name: &str, fast_mode: bool, custom_bus: Option<&str>) -> HsipcResult<Self> {
        let process_id = std::process::id();
        let process_name = format!("test_{}_{}", test_name, process_id);
        let bus_name = custom_bus
            .map(|b| format!("{}.{}", b, process_id))
            .unwrap_or_else(|| format!("com.hsipc.test.{}.{}", test_name, process_id));

        let hub = ProcessHub::builder(&process_name)
            .with_bus_name(&bus_name)
            .with_fast_mode(fast_mode)
            .build()
            .await?;

        Ok(Self {
            hub,
            process_name,
            bus_name,
        })
    }

    /// Execute an operation with timeout
    pub async fn with_timeout<F, T>(&self, operation: F) -> HsipcResult<T>
    where
        F: std::future::Future<Output = HsipcResult<T>>,
    {
        tokio::time::timeout(Duration::from_secs(10), operation)
            .await
            .map_err(|_| hsipc::Error::timeout_msg("Test operation timed out"))?
    }

    /// Wait for message loop to be ready (helper for timing-sensitive tests)
    pub async fn wait_for_ready(&self) -> HsipcResult<()> {
        for _i in 0..500 {
            if self.hub.is_message_loop_ready() {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        Err(hsipc::Error::timeout_msg("Hub message loop not ready"))
    }

    /// Clean shutdown
    pub async fn shutdown(self) -> HsipcResult<()> {
        self.hub.shutdown().await
    }
}

/// Mock service for testing RPC functionality
pub struct MockCalculatorService {
    pub operation_count: Arc<Mutex<u64>>,
    pub last_operations: Arc<RwLock<Vec<MockOperation>>>,
    pub should_fail: Arc<Mutex<bool>>,
    pub delay_duration: Arc<Mutex<Option<Duration>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockOperation {
    pub method: String,
    pub timestamp: u64,
    pub success: bool,
}

impl MockCalculatorService {
    pub fn new() -> Self {
        Self {
            operation_count: Arc::new(Mutex::new(0)),
            last_operations: Arc::new(RwLock::new(Vec::new())),
            should_fail: Arc::new(Mutex::new(false)),
            delay_duration: Arc::new(Mutex::new(None)),
        }
    }

    /// Configure the service to fail operations
    pub async fn set_failure_mode(&self, should_fail: bool) {
        *self.should_fail.lock().await = should_fail;
    }

    /// Configure the service to add delay to operations
    pub async fn set_delay(&self, delay: Option<Duration>) {
        *self.delay_duration.lock().await = delay;
    }

    /// Get the number of operations performed
    pub async fn get_operation_count(&self) -> u64 {
        *self.operation_count.lock().await
    }

    /// Get the last operations performed
    pub async fn get_last_operations(&self) -> Vec<MockOperation> {
        self.last_operations.read().await.clone()
    }

    /// Reset the service state
    pub async fn reset(&self) {
        *self.operation_count.lock().await = 0;
        self.last_operations.write().await.clear();
        *self.should_fail.lock().await = false;
        *self.delay_duration.lock().await = None;
    }
}

impl Default for MockCalculatorService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Service for MockCalculatorService {
    fn name(&self) -> &'static str {
        "MockCalculator"
    }

    fn methods(&self) -> Vec<&'static str> {
        vec!["add", "multiply", "divide", "status"]
    }

    async fn handle(&self, method: &str, payload: Vec<u8>) -> HsipcResult<Vec<u8>> {
        // Apply delay if configured
        if let Some(delay) = *self.delay_duration.lock().await {
            tokio::time::sleep(delay).await;
        }

        // Check if we should fail
        let should_fail = *self.should_fail.lock().await;
        
        // Increment operation count
        *self.operation_count.lock().await += 1;

        // Record operation
        let operation = MockOperation {
            method: method.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            success: !should_fail,
        };
        self.last_operations.write().await.push(operation);

        if should_fail {
            return Err(hsipc::Error::runtime_msg(format!("Mock service configured to fail for method: {}", method)));
        }

        match method {
            "add" => {
                let request: TestAddRequest = hsipc::bincode::deserialize(&payload)?;
                let response = TestAddResponse {
                    result: request.a + request.b,
                    operation_id: request.operation_id,
                };
                Ok(hsipc::bincode::serialize(&response)?)
            }
            "multiply" => {
                let request: TestMultiplyRequest = hsipc::bincode::deserialize(&payload)?;
                let response = TestMultiplyResponse {
                    result: request.a * request.b,
                    operation_id: request.operation_id,
                };
                Ok(hsipc::bincode::serialize(&response)?)
            }
            "divide" => {
                let request: TestDivideRequest = hsipc::bincode::deserialize(&payload)?;
                if request.b == 0.0 {
                    return Err(hsipc::Error::runtime_msg("Division by zero"));
                }
                let response = TestDivideResponse {
                    result: request.a / request.b,
                    operation_id: request.operation_id,
                };
                Ok(hsipc::bincode::serialize(&response)?)
            }
            "status" => {
                let response = TestStatusResponse {
                    service_name: "MockCalculator".to_string(),
                    operation_count: *self.operation_count.lock().await,
                    uptime_seconds: 100, // Mock value
                };
                Ok(hsipc::bincode::serialize(&response)?)
            }
            _ => Err(hsipc::Error::method_not_found(method)),
        }
    }
}

/// Mock event subscriber for testing event system
pub struct MockEventSubscriber {
    pub topic_pattern: String,
    pub received_events: Arc<Mutex<Vec<ReceivedEvent>>>,
    pub should_fail: Arc<Mutex<bool>>,
    pub subscriber_id: String,
}

#[derive(Debug, Clone)]
pub struct ReceivedEvent {
    pub topic: String,
    pub payload_size: usize,
    pub timestamp: u64,
    pub success: bool,
}

impl MockEventSubscriber {
    pub fn new(topic_pattern: &str, subscriber_id: &str) -> Self {
        Self {
            topic_pattern: topic_pattern.to_string(),
            received_events: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(Mutex::new(false)),
            subscriber_id: subscriber_id.to_string(),
        }
    }

    /// Configure the subscriber to fail event handling
    pub async fn set_failure_mode(&self, should_fail: bool) {
        *self.should_fail.lock().await = should_fail;
    }

    /// Get received events
    pub async fn get_received_events(&self) -> Vec<ReceivedEvent> {
        self.received_events.lock().await.clone()
    }

    /// Get the count of received events
    pub async fn get_event_count(&self) -> usize {
        self.received_events.lock().await.len()
    }

    /// Reset subscriber state
    pub async fn reset(&self) {
        self.received_events.lock().await.clear();
        *self.should_fail.lock().await = false;
    }
}

#[async_trait]
impl Subscriber for MockEventSubscriber {
    fn topic_pattern(&self) -> &str {
        &self.topic_pattern
    }

    async fn handle(&mut self, topic: &str, payload: Vec<u8>) -> HsipcResult<()> {
        let should_fail = *self.should_fail.lock().await;
        
        let event = ReceivedEvent {
            topic: topic.to_string(),
            payload_size: payload.len(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            success: !should_fail,
        };

        self.received_events.lock().await.push(event);

        if should_fail {
            return Err(hsipc::Error::runtime_msg(format!("Mock subscriber {} configured to fail", self.subscriber_id)));
        }

        Ok(())
    }
}

/// Test data structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestAddRequest {
    pub a: f64,
    pub b: f64,
    pub operation_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestAddResponse {
    pub result: f64,
    pub operation_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestMultiplyRequest {
    pub a: i32,
    pub b: i32,
    pub operation_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestMultiplyResponse {
    pub result: i32,
    pub operation_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestDivideRequest {
    pub a: f64,
    pub b: f64,
    pub operation_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestDivideResponse {
    pub result: f64,
    pub operation_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestStatusResponse {
    pub service_name: String,
    pub operation_count: u64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestEvent {
    pub event_id: u64,
    pub message: String,
    pub timestamp: u64,
    pub source: String,
}

/// Test client wrapper for convenient testing
pub struct TestClient {
    hub: ProcessHub,
}

impl TestClient {
    pub fn new(hub: ProcessHub) -> Self {
        Self { hub }
    }

    pub async fn add(&self, a: f64, b: f64, operation_id: u64) -> HsipcResult<TestAddResponse> {
        let request = TestAddRequest { a, b, operation_id };
        self.hub.call("MockCalculator.add", request).await
    }

    pub async fn multiply(&self, a: i32, b: i32, operation_id: u64) -> HsipcResult<TestMultiplyResponse> {
        let request = TestMultiplyRequest { a, b, operation_id };
        self.hub.call("MockCalculator.multiply", request).await
    }

    pub async fn divide(&self, a: f64, b: f64, operation_id: u64) -> HsipcResult<TestDivideResponse> {
        let request = TestDivideRequest { a, b, operation_id };
        self.hub.call("MockCalculator.divide", request).await
    }

    pub async fn get_status(&self) -> HsipcResult<TestStatusResponse> {
        let request = (); // Empty request
        self.hub.call("MockCalculator.status", request).await
    }

    pub async fn publish_event(&self, event: TestEvent) -> HsipcResult<()> {
        self.hub.publish("test/events", event).await
    }

    pub async fn publish_sensor_event(&self, sensor_id: &str, value: f64) -> HsipcResult<()> {
        let event = TestEvent {
            event_id: rand::random(),
            message: format!("Sensor {} reading: {}", sensor_id, value),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source: sensor_id.to_string(),
        };
        self.hub.publish("sensor/temperature", event).await
    }
}

/// Process lifecycle manager for multi-process tests
pub struct ProcessLifecycleManager {
    processes: Vec<tokio::process::Child>,
    pub process_names: Vec<String>,
}

impl ProcessLifecycleManager {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            process_names: Vec::new(),
        }
    }

    pub async fn start_example_process(&mut self, command: &str, process_name: &str) -> HsipcResult<()> {
        println!("🚀 Starting {} process...", process_name);
        
        let process = tokio::process::Command::new("cargo")
            .arg("run")
            .arg(command)
            .current_dir("examples/rpc_system_demo")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| hsipc::Error::io_msg(format!("Failed to start {}: {}", process_name, e)))?;

        // Give process time to start up
        tokio::time::sleep(Duration::from_millis(2000)).await;

        self.processes.push(process);
        self.process_names.push(process_name.to_string());

        println!("✅ {} process started", process_name);
        Ok(())
    }

    pub async fn stop_all_processes(&mut self) {
        println!("🛑 Stopping all processes...");
        
        for (i, process) in self.processes.iter_mut().enumerate() {
            let process_name = self.process_names.get(i).unwrap_or(&"unknown".to_string());
            println!("   Stopping {}...", process_name);
            
            let _ = process.kill().await;
            let _ = process.wait().await;
        }
        
        self.processes.clear();
        self.process_names.clear();
        println!("✅ All processes stopped");
    }

    pub fn process_count(&self) -> usize {
        self.processes.len()
    }

    pub async fn check_process_health(&mut self) -> Vec<(String, bool)> {
        let mut health_status = Vec::new();
        
        for (i, process) in self.processes.iter_mut().enumerate() {
            let process_name = self.process_names.get(i).unwrap_or(&"unknown".to_string()).clone();
            
            match process.try_wait() {
                Ok(Some(_status)) => {
                    // Process has exited
                    health_status.push((process_name, false));
                }
                Ok(None) => {
                    // Process is still running
                    health_status.push((process_name, true));
                }
                Err(_) => {
                    // Error checking process
                    health_status.push((process_name, false));
                }
            }
        }
        
        health_status
    }
}

impl Drop for ProcessLifecycleManager {
    fn drop(&mut self) {
        // Synchronous cleanup
        for process in &mut self.processes {
            let _ = process.start_kill();
        }
    }
}

/// Test assertion helpers
pub mod assertions {
    use super::*;

    /// Assert that a test environment is properly set up
    pub async fn assert_test_environment_ready(env: &TestEnvironment) {
        env.wait_for_ready().await.expect("Test environment should be ready");
        assert!(!env.process_name.is_empty(), "Process name should not be empty");
        assert!(!env.bus_name.is_empty(), "Bus name should not be empty");
    }

    /// Assert service operation metrics
    pub async fn assert_service_metrics(
        service: &MockCalculatorService,
        expected_min_operations: u64,
        expected_success_rate: f64,
    ) {
        let operation_count = service.get_operation_count().await;
        let operations = service.get_last_operations().await;
        
        assert!(
            operation_count >= expected_min_operations,
            "Service should have performed at least {} operations, got {}",
            expected_min_operations,
            operation_count
        );

        if !operations.is_empty() {
            let successful_ops = operations.iter().filter(|op| op.success).count();
            let success_rate = successful_ops as f64 / operations.len() as f64;
            
            assert!(
                success_rate >= expected_success_rate,
                "Service success rate should be at least {:.2}%, got {:.2}%",
                expected_success_rate * 100.0,
                success_rate * 100.0
            );
        }
    }

    /// Assert subscriber event metrics
    pub async fn assert_subscriber_metrics(
        subscriber: &MockEventSubscriber,
        expected_min_events: usize,
    ) {
        let event_count = subscriber.get_event_count().await;
        
        assert!(
            event_count >= expected_min_events,
            "Subscriber should have received at least {} events, got {}",
            expected_min_events,
            event_count
        );
    }

    /// Assert process health
    pub async fn assert_process_health(
        manager: &mut ProcessLifecycleManager,
        expected_healthy_processes: usize,
    ) {
        let health_status = manager.check_process_health().await;
        let healthy_count = health_status.iter().filter(|(_, healthy)| *healthy).count();
        
        assert_eq!(
            healthy_count,
            expected_healthy_processes,
            "Expected {} healthy processes, got {}. Status: {:?}",
            expected_healthy_processes,
            healthy_count,
            health_status
        );
    }
}

/// Test data generators
pub mod generators {
    use super::*;

    /// Generate test add requests
    pub fn generate_add_requests(count: usize) -> Vec<TestAddRequest> {
        (0..count)
            .map(|i| TestAddRequest {
                a: i as f64,
                b: (i + 1) as f64,
                operation_id: i as u64,
            })
            .collect()
    }

    /// Generate test events
    pub fn generate_test_events(count: usize, source: &str) -> Vec<TestEvent> {
        (0..count)
            .map(|i| TestEvent {
                event_id: i as u64,
                message: format!("Test event {} from {}", i, source),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() + i as u64,
                source: source.to_string(),
            })
            .collect()
    }

    /// Generate random sensor data
    pub fn generate_sensor_events(count: usize, sensor_id: &str) -> Vec<TestEvent> {
        (0..count)
            .map(|i| TestEvent {
                event_id: i as u64,
                message: format!("Sensor {} reading: {:.2}", sensor_id, 20.0 + (i as f64 * 0.1)),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() + i as u64,
                source: sensor_id.to_string(),
            })
            .collect()
    }
}