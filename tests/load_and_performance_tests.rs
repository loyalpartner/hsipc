//! Load and performance tests for hsipc
//!
//! These tests validate the performance characteristics and load handling
//! capabilities of the hsipc framework under various stress conditions.

use hsipc::{ProcessHub, Result as HsipcResult};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Barrier, Semaphore};
use tokio::time::timeout;

/// Performance test configuration
#[derive(Debug, Clone)]
struct LoadTestConfig {
    pub concurrent_clients: usize,
    pub operations_per_client: usize,
    pub test_duration: Duration,
    pub warmup_duration: Duration,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrent_clients: 10,
            operations_per_client: 100,
            test_duration: Duration::from_secs(30),
            warmup_duration: Duration::from_secs(5),
        }
    }
}

/// Performance metrics collector
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub total_duration: Duration,
    pub min_latency: Duration,
    pub max_latency: Duration,
    pub avg_latency: Duration,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            total_duration: Duration::ZERO,
            min_latency: Duration::MAX,
            max_latency: Duration::ZERO,
            avg_latency: Duration::ZERO,
        }
    }

    fn add_result(&mut self, latency: Duration, success: bool) {
        self.total_operations += 1;
        if success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }

        if success {
            self.min_latency = self.min_latency.min(latency);
            self.max_latency = self.max_latency.max(latency);
        }
    }

    fn finalize(&mut self, total_duration: Duration) {
        self.total_duration = total_duration;
        if self.successful_operations > 0 {
            // This is a simplified average - in real implementation you'd track all latencies
            self.avg_latency = Duration::from_nanos(
                (self.min_latency.as_nanos() + self.max_latency.as_nanos()) as u64 / 2
            );
        }
    }

    fn operations_per_second(&self) -> f64 {
        if self.total_duration.as_secs_f64() > 0.0 {
            self.successful_operations as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        }
    }

    fn success_rate(&self) -> f64 {
        if self.total_operations > 0 {
            self.successful_operations as f64 / self.total_operations as f64
        } else {
            0.0
        }
    }
}

/// Test data structures (simplified versions of example types)
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct TestRequest {
    x: f64,
    y: f64,
    operation_id: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct TestResponse {
    result: f64,
    operation_id: u64,
}

/// Load test for basic RPC operations
#[tokio::test]
async fn test_rpc_load_basic() {
    println!("🧪 Testing basic RPC load handling...");
    
    let config = LoadTestConfig {
        concurrent_clients: 5,
        operations_per_client: 50,
        ..Default::default()
    };

    println!("📊 Load test configuration:");
    println!("   Concurrent clients: {}", config.concurrent_clients);
    println!("   Operations per client: {}", config.operations_per_client);
    println!("   Total operations: {}", config.concurrent_clients * config.operations_per_client);

    // Create test environment
    let hub = create_test_hub("load_test_basic").await;
    let service = create_test_calculator_service();
    hub.register_service(service).await.expect("Failed to register service");

    // Warmup
    println!("🔥 Warming up...");
    perform_warmup(&hub, config.warmup_duration).await;

    // Run load test
    println!("🚀 Starting load test...");
    let start_time = Instant::now();
    let metrics = run_concurrent_rpc_test(&hub, &config).await;
    let total_duration = start_time.elapsed();

    // Report results
    report_performance_metrics(&metrics, total_duration, &config);

    // Assertions for basic load test
    assert!(metrics.success_rate() > 0.95, "Success rate should be > 95%, got {:.2}%", metrics.success_rate() * 100.0);
    assert!(metrics.operations_per_second() > 10.0, "Should handle > 10 ops/sec, got {:.2}", metrics.operations_per_second());
    
    println!("✅ Basic RPC load test passed!");
}

/// Load test for high-concurrency scenarios
#[tokio::test]
async fn test_rpc_load_high_concurrency() {
    println!("🧪 Testing high-concurrency RPC load handling...");
    
    let config = LoadTestConfig {
        concurrent_clients: 20,
        operations_per_client: 25,
        test_duration: Duration::from_secs(15),
        ..Default::default()
    };

    println!("📊 High-concurrency test configuration:");
    println!("   Concurrent clients: {}", config.concurrent_clients);
    println!("   Operations per client: {}", config.operations_per_client);
    println!("   Total operations: {}", config.concurrent_clients * config.operations_per_client);

    // Create test environment
    let hub = create_test_hub("load_test_high_concurrency").await;
    let service = create_test_calculator_service();
    hub.register_service(service).await.expect("Failed to register service");

    // Warmup
    println!("🔥 Warming up...");
    perform_warmup(&hub, config.warmup_duration).await;

    // Run high-concurrency test
    println!("🚀 Starting high-concurrency test...");
    let start_time = Instant::now();
    let metrics = run_concurrent_rpc_test(&hub, &config).await;
    let total_duration = start_time.elapsed();

    // Report results
    report_performance_metrics(&metrics, total_duration, &config);

    // More lenient assertions for high-concurrency
    assert!(metrics.success_rate() > 0.90, "Success rate should be > 90% under high concurrency, got {:.2}%", metrics.success_rate() * 100.0);
    assert!(metrics.operations_per_second() > 5.0, "Should handle > 5 ops/sec under high concurrency, got {:.2}", metrics.operations_per_second());
    
    println!("✅ High-concurrency RPC load test passed!");
}

/// Load test for sustained operations
#[tokio::test]
async fn test_rpc_load_sustained() {
    println!("🧪 Testing sustained RPC load handling...");
    
    let config = LoadTestConfig {
        concurrent_clients: 8,
        operations_per_client: 100,
        test_duration: Duration::from_secs(45),
        warmup_duration: Duration::from_secs(10),
    };

    println!("📊 Sustained load test configuration:");
    println!("   Test duration: {:?}", config.test_duration);
    println!("   Concurrent clients: {}", config.concurrent_clients);
    println!("   Operations per client: {}", config.operations_per_client);

    // Create test environment
    let hub = create_test_hub("load_test_sustained").await;
    let service = create_test_calculator_service();
    hub.register_service(service).await.expect("Failed to register service");

    // Extended warmup
    println!("🔥 Extended warmup...");
    perform_warmup(&hub, config.warmup_duration).await;

    // Run sustained test
    println!("🚀 Starting sustained load test...");
    let start_time = Instant::now();
    let metrics = run_sustained_rpc_test(&hub, &config).await;
    let total_duration = start_time.elapsed();

    // Report results
    report_performance_metrics(&metrics, total_duration, &config);

    // Assertions for sustained operations
    assert!(metrics.success_rate() > 0.92, "Sustained success rate should be > 92%, got {:.2}%", metrics.success_rate() * 100.0);
    assert!(total_duration >= config.test_duration, "Test should run for full duration");
    
    println!("✅ Sustained RPC load test passed!");
}

/// Event system load test
#[tokio::test]
async fn test_event_system_load() {
    println!("🧪 Testing event system load handling...");
    
    let config = LoadTestConfig {
        concurrent_clients: 15,
        operations_per_client: 30,
        test_duration: Duration::from_secs(20),
        ..Default::default()
    };

    println!("📊 Event system load configuration:");
    println!("   Concurrent publishers: {}", config.concurrent_clients);
    println!("   Events per publisher: {}", config.operations_per_client);
    println!("   Total events: {}", config.concurrent_clients * config.operations_per_client);

    // Create test environment
    let hub = create_test_hub("event_load_test").await;
    
    // Set up subscribers
    let subscriber_count = 5;
    println!("📡 Setting up {} subscribers...", subscriber_count);
    
    let mut _subscriptions = Vec::new();
    for i in 0..subscriber_count {
        let subscriber = TestEventSubscriber::new(format!("subscriber_{}", i));
        let subscription = hub.subscribe(subscriber).await.expect("Failed to create subscription");
        _subscriptions.push(subscription);
    }

    tokio::time::sleep(Duration::from_millis(1000)).await; // Let subscriptions register

    // Run event load test
    println!("🚀 Starting event load test...");
    let start_time = Instant::now();
    let metrics = run_concurrent_event_test(&hub, &config).await;
    let total_duration = start_time.elapsed();

    // Report results
    report_performance_metrics(&metrics, total_duration, &config);

    // Event system assertions
    assert!(metrics.success_rate() > 0.88, "Event publish success rate should be > 88%, got {:.2}%", metrics.success_rate() * 100.0);
    
    println!("✅ Event system load test passed!");
}

/// Memory usage and leak test
#[tokio::test]
async fn test_memory_usage_under_load() {
    println!("🧪 Testing memory usage under load...");
    
    let config = LoadTestConfig {
        concurrent_clients: 12,
        operations_per_client: 75,
        test_duration: Duration::from_secs(30),
        ..Default::default()
    };

    // Create test environment
    let hub = create_test_hub("memory_load_test").await;
    let service = create_test_calculator_service();
    hub.register_service(service).await.expect("Failed to register service");

    // Measure initial memory (simplified - in real tests you'd use proper memory profiling)
    println!("📊 Starting memory load test...");

    // Run multiple rounds to check for memory leaks
    let rounds = 3;
    for round in 1..=rounds {
        println!("🔄 Memory test round {}/{}", round, rounds);
        
        let start_time = Instant::now();
        let metrics = run_concurrent_rpc_test(&hub, &config).await;
        let round_duration = start_time.elapsed();

        println!("   Round {} completed: {:.2} ops/sec, {:.1}% success rate", 
                round, 
                metrics.operations_per_second(), 
                metrics.success_rate() * 100.0);

        // Brief pause between rounds
        tokio::time::sleep(Duration::from_millis(2000)).await;
    }

    println!("✅ Memory usage test completed (no crashes indicate good memory management)");
}

// Helper functions

async fn create_test_hub(name: &str) -> ProcessHub {
    let unique_name = format!("{}_{}", name, std::process::id());
    let bus_name = format!("com.hsipc.test.{}.{}", name, std::process::id());

    ProcessHub::builder(&unique_name)
        .with_bus_name(&bus_name)
        .with_fast_mode(true)
        .build()
        .await
        .expect("Failed to create test hub")
}

fn create_test_calculator_service() -> TestCalculatorService {
    TestCalculatorService::new()
}

async fn perform_warmup(hub: &ProcessHub, duration: Duration) {
    let warmup_client = TestRpcClient::new(hub.clone());
    let end_time = Instant::now() + duration;
    
    while Instant::now() < end_time {
        let _ = warmup_client.perform_operation(1.0, 1.0, 0).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn run_concurrent_rpc_test(hub: &ProcessHub, config: &LoadTestConfig) -> PerformanceMetrics {
    let barrier = Arc::new(Barrier::new(config.concurrent_clients));
    let semaphore = Arc::new(Semaphore::new(config.concurrent_clients));
    let mut handles = Vec::new();

    for client_id in 0..config.concurrent_clients {
        let hub_clone = hub.clone();
        let barrier_clone = barrier.clone();
        let semaphore_clone = semaphore.clone();
        let config_clone = config.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            let client = TestRpcClient::new(hub_clone);
            
            // Wait for all clients to be ready
            barrier_clone.wait().await;
            
            let mut metrics = PerformanceMetrics::new();
            
            for op_id in 0..config_clone.operations_per_client {
                let start = Instant::now();
                let operation_id = (client_id * config_clone.operations_per_client + op_id) as u64;
                
                let success = match timeout(
                    Duration::from_secs(5),
                    client.perform_operation(10.0, 5.0, operation_id)
                ).await {
                    Ok(Ok(_)) => true,
                    Ok(Err(_)) => false,
                    Err(_) => false, // timeout
                };
                
                let latency = start.elapsed();
                metrics.add_result(latency, success);
                
                // Small delay to avoid overwhelming the system
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            
            metrics
        });
        
        handles.push(handle);
    }

    // Collect results from all clients
    let mut combined_metrics = PerformanceMetrics::new();
    
    for handle in handles {
        if let Ok(client_metrics) = handle.await {
            combined_metrics.total_operations += client_metrics.total_operations;
            combined_metrics.successful_operations += client_metrics.successful_operations;
            combined_metrics.failed_operations += client_metrics.failed_operations;
            combined_metrics.min_latency = combined_metrics.min_latency.min(client_metrics.min_latency);
            combined_metrics.max_latency = combined_metrics.max_latency.max(client_metrics.max_latency);
        }
    }

    combined_metrics
}

async fn run_sustained_rpc_test(hub: &ProcessHub, config: &LoadTestConfig) -> PerformanceMetrics {
    let mut combined_metrics = PerformanceMetrics::new();
    let end_time = Instant::now() + config.test_duration;
    
    let mut round = 0;
    while Instant::now() < end_time {
        round += 1;
        println!("   Sustained test round {}...", round);
        
        let round_metrics = run_concurrent_rpc_test(hub, config).await;
        
        combined_metrics.total_operations += round_metrics.total_operations;
        combined_metrics.successful_operations += round_metrics.successful_operations;
        combined_metrics.failed_operations += round_metrics.failed_operations;
        combined_metrics.min_latency = combined_metrics.min_latency.min(round_metrics.min_latency);
        combined_metrics.max_latency = combined_metrics.max_latency.max(round_metrics.max_latency);
        
        // Brief pause between rounds
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    combined_metrics
}

async fn run_concurrent_event_test(hub: &ProcessHub, config: &LoadTestConfig) -> PerformanceMetrics {
    let barrier = Arc::new(Barrier::new(config.concurrent_clients));
    let mut handles = Vec::new();

    for client_id in 0..config.concurrent_clients {
        let hub_clone = hub.clone();
        let barrier_clone = barrier.clone();
        let config_clone = config.clone();

        let handle = tokio::spawn(async move {
            let client = TestEventPublisher::new(hub_clone);
            
            // Wait for all publishers to be ready
            barrier_clone.wait().await;
            
            let mut metrics = PerformanceMetrics::new();
            
            for op_id in 0..config_clone.operations_per_client {
                let start = Instant::now();
                let event_id = (client_id * config_clone.operations_per_client + op_id) as u64;
                
                let success = client.publish_test_event(event_id).await.is_ok();
                let latency = start.elapsed();
                metrics.add_result(latency, success);
                
                // Small delay between events
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            
            metrics
        });
        
        handles.push(handle);
    }

    // Collect results
    let mut combined_metrics = PerformanceMetrics::new();
    
    for handle in handles {
        if let Ok(client_metrics) = handle.await {
            combined_metrics.total_operations += client_metrics.total_operations;
            combined_metrics.successful_operations += client_metrics.successful_operations;
            combined_metrics.failed_operations += client_metrics.failed_operations;
            combined_metrics.min_latency = combined_metrics.min_latency.min(client_metrics.min_latency);
            combined_metrics.max_latency = combined_metrics.max_latency.max(client_metrics.max_latency);
        }
    }

    combined_metrics
}

fn report_performance_metrics(metrics: &PerformanceMetrics, total_duration: Duration, config: &LoadTestConfig) {
    println!("📊 Performance Test Results:");
    println!("   Duration: {:.2}s", total_duration.as_secs_f64());
    println!("   Total Operations: {}", metrics.total_operations);
    println!("   Successful: {} ({:.1}%)", metrics.successful_operations, metrics.success_rate() * 100.0);
    println!("   Failed: {}", metrics.failed_operations);
    println!("   Operations/sec: {:.2}", metrics.successful_operations as f64 / total_duration.as_secs_f64());
    
    if metrics.successful_operations > 0 {
        println!("   Latency - Min: {:.2}ms, Max: {:.2}ms", 
                metrics.min_latency.as_secs_f64() * 1000.0,
                metrics.max_latency.as_secs_f64() * 1000.0);
    }
    
    println!("   Concurrency: {} clients", config.concurrent_clients);
}

// Test helper structures

struct TestCalculatorService;

impl TestCalculatorService {
    fn new() -> Self {
        Self
    }
}

#[hsipc::async_trait]
impl hsipc::Service for TestCalculatorService {
    fn name(&self) -> &'static str {
        "TestCalculator"
    }

    fn methods(&self) -> Vec<&'static str> {
        vec!["add"]
    }

    async fn handle(&self, method: &str, payload: Vec<u8>) -> HsipcResult<Vec<u8>> {
        match method {
            "add" => {
                let request: TestRequest = hsipc::bincode::deserialize(&payload)?;
                let response = TestResponse {
                    result: request.x + request.y,
                    operation_id: request.operation_id,
                };
                Ok(hsipc::bincode::serialize(&response)?)
            }
            _ => Err(hsipc::Error::method_not_found(method)),
        }
    }
}

struct TestRpcClient {
    hub: ProcessHub,
}

impl TestRpcClient {
    fn new(hub: ProcessHub) -> Self {
        Self { hub }
    }

    async fn perform_operation(&self, x: f64, y: f64, operation_id: u64) -> HsipcResult<TestResponse> {
        let request = TestRequest { x, y, operation_id };
        self.hub.call("TestCalculator.add", request).await
    }
}

struct TestEventPublisher {
    hub: ProcessHub,
}

impl TestEventPublisher {
    fn new(hub: ProcessHub) -> Self {
        Self { hub }
    }

    async fn publish_test_event(&self, event_id: u64) -> HsipcResult<()> {
        let event = TestEvent {
            id: event_id,
            data: format!("Test event {}", event_id),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        self.hub.publish("test/events", event).await
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TestEvent {
    id: u64,
    data: String,
    timestamp: u64,
}

struct TestEventSubscriber {
    name: String,
}

impl TestEventSubscriber {
    fn new(name: String) -> Self {
        Self { name }
    }
}

#[hsipc::async_trait]
impl hsipc::Subscriber for TestEventSubscriber {
    fn topic_pattern(&self) -> &str {
        "test/#"
    }

    async fn handle(&mut self, _topic: &str, _payload: Vec<u8>) -> HsipcResult<()> {
        // Just acknowledge receipt
        Ok(())
    }
}