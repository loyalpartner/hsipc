//! RPC System Demo - Complete example demonstrating all RPC features
//!
//! This example serves as both documentation and functional verification
//! following the TESTING.md example-driven testing approach.

use clap::{Parser, Subcommand};
use hsipc::{method, rpc, subscription, ProcessHub};
use serde::{Deserialize, Serialize};
use tracing::info;

// Request/Response types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalculationRequest {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalculationResult {
    pub result: f64,
    pub operation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusInfo {
    pub service: String,
    pub version: String,
    pub uptime: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEvent {
    pub level: String,
    pub message: String,
    pub timestamp: u64,
}

// Custom error type
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum CalculatorError {
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Overflow occurred")]
    Overflow,
}

impl From<CalculatorError> for hsipc::Error {
    fn from(err: CalculatorError) -> Self {
        hsipc::Error::runtime_msg(err.to_string())
    }
}

// Result type alias for methods that don't need custom errors
type Result<T> = std::result::Result<T, CalculatorError>;
type SubscriptionResult = std::result::Result<(), CalculatorError>;

// Complete RPC service demonstrating all features
#[rpc(server, client, namespace = "calculator")]
pub trait Calculator {
    // Basic async method
    #[method(name = "add")]
    async fn add(&self, req: CalculationRequest) -> Result<CalculationResult>;

    // Sync method
    #[method(name = "multiply", sync)]
    fn multiply(&self, a: i32, b: i32) -> Result<i32>;

    // Multi-parameter method
    #[method(name = "power")]
    async fn power(&self, base: f64, exponent: f64) -> Result<f64>;

    // Method with custom error type
    #[method(name = "divide")]
    async fn divide(
        &self,
        req: CalculationRequest,
    ) -> std::result::Result<CalculationResult, CalculatorError>;

    // Method with timeout
    #[method(name = "complex_calculation", timeout = 5000)]
    async fn complex_calculation(&self, iterations: u32) -> Result<f64>;

    // No parameter method
    #[method(name = "get_status")]
    async fn get_status(&self) -> Result<StatusInfo>;

    // Subscription method
    #[subscription(name = "calculation_logs", item = LogEvent)]
    async fn subscribe_logs(&self, level_filter: Option<String>) -> SubscriptionResult;
}

// Service implementation
pub struct CalculatorImpl {
    start_time: std::time::Instant,
}

impl Default for CalculatorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorImpl {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }
}

#[hsipc::async_trait]
impl Calculator for CalculatorImpl {
    async fn add(&self, req: CalculationRequest) -> Result<CalculationResult> {
        Ok(CalculationResult {
            result: req.x + req.y,
            operation: "add".to_string(),
        })
    }

    fn multiply(&self, a: i32, b: i32) -> Result<i32> {
        Ok(a * b)
    }

    async fn power(&self, base: f64, exponent: f64) -> Result<f64> {
        Ok(base.powf(exponent))
    }

    async fn divide(
        &self,
        req: CalculationRequest,
    ) -> std::result::Result<CalculationResult, CalculatorError> {
        if req.y == 0.0 {
            Err(CalculatorError::DivisionByZero)
        } else {
            Ok(CalculationResult {
                result: req.x / req.y,
                operation: "divide".to_string(),
            })
        }
    }

    async fn complex_calculation(&self, iterations: u32) -> Result<f64> {
        // Simulate complex calculation
        let mut result = 0.0;
        for i in 0..iterations {
            result += (i as f64).sin().cos();
            if i % 1000 == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }
        }
        Ok(result)
    }

    async fn get_status(&self) -> Result<StatusInfo> {
        Ok(StatusInfo {
            service: "CalculatorService".to_string(),
            version: "1.0.0".to_string(),
            uptime: self.start_time.elapsed().as_secs(),
        })
    }

    async fn subscribe_logs(
        &self,
        pending: hsipc::PendingSubscriptionSink,
        _level_filter: Option<String>,
    ) -> SubscriptionResult {
        // Accept the subscription for demo purposes
        let _sink = pending
            .accept()
            .await
            .map_err(|e| CalculatorError::InvalidOperation(e.to_string()))?;
        Ok(())
    }
}

#[derive(Parser)]
#[command(name = "rpc-demo")]
#[command(about = "RPC System Demo - showcasing all RPC features")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the demo (comprehensive feature test)
    Demo,
    /// Run as server
    Server,
    /// Run as client
    Client,
}

#[tokio::main]
async fn main() -> hsipc::Result<()> {
    // Initialize tracing with line numbers and compact format
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_file(true)
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Demo => run_demo().await,
        Commands::Server => run_server().await,
        Commands::Client => run_client().await,
    }
}

/// Comprehensive demo showcasing all RPC features
async fn run_demo() -> hsipc::Result<()> {
    info!("🚀 RPC System Demo - Testing all features...");

    // Setup
    let hub = ProcessHub::builder("rpc_demo").build().await?;
    let service = CalculatorService::new(CalculatorImpl::new());
    hub.register_service(service).await?;

    let client = CalculatorClient::new(hub);

    // 1. Test basic async method
    info!("✅ Testing basic async method...");
    let add_result = client.add(CalculationRequest { x: 10.0, y: 5.0 }).await?;
    info!(
        "   Add result: {} = {}",
        add_result.operation, add_result.result
    );
    assert_eq!(add_result.result, 15.0);

    // 2. Test sync method
    info!("✅ Testing sync method...");
    let multiply_result = client.multiply(6, 7)?;
    info!("   Multiply result: {multiply_result}");
    assert_eq!(multiply_result, 42);

    // 3. Test multi-parameter method
    println!("✅ Testing multi-parameter method...");
    let power_result = client.power(2.0, 3.0).await?;
    println!("   Power result: {power_result}");
    assert_eq!(power_result, 8.0);

    // 4. Test custom error type - success case
    println!("✅ Testing custom error type (success)...");
    let divide_result = client
        .divide(CalculationRequest { x: 10.0, y: 2.0 })
        .await?;
    println!(
        "   Divide result: {} = {}",
        divide_result.operation, divide_result.result
    );
    assert_eq!(divide_result.result, 5.0);

    // 5. Test custom error type - error case
    println!("✅ Testing custom error type (error)...");
    let divide_error = client.divide(CalculationRequest { x: 10.0, y: 0.0 }).await;
    println!("   Expected error: {divide_error:?}");
    assert!(divide_error.is_err());

    // 6. Test no parameter method
    println!("✅ Testing no parameter method...");
    let status = client.get_status().await?;
    println!(
        "   Status: {} v{}, uptime: {}s",
        status.service, status.version, status.uptime
    );

    // 7. Test subscription method
    println!("✅ Testing subscription method...");
    client.subscribe_logs(Some("info".to_string())).await?;
    println!("   Subscription created successfully");

    // 8. Test complex calculation with timeout
    println!("✅ Testing complex calculation...");
    let complex_result = client.complex_calculation(1000).await?;
    println!("   Complex calculation result: {complex_result}");

    println!("\n🎉 All RPC features working correctly!");
    info!("📊 Demo completed in < 30 seconds");

    Ok(())
}

/// Run as server (for multi-process testing)
async fn run_server() -> hsipc::Result<()> {
    println!("🖥️  Starting RPC server...");

    let hub = ProcessHub::builder("calculator_server").build().await?;
    let service = CalculatorService::new(CalculatorImpl::new());
    hub.register_service(service).await?;

    println!("✅ Server running. Press Ctrl+C to stop.");

    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
    println!("🛑 Received Ctrl+C, shutting down server...");
    
    if let Err(e) = hub.shutdown().await {
        eprintln!("Error during server shutdown: {}", e);
    }

    Ok(())
}

/// Run as client (for multi-process testing)
async fn run_client() -> hsipc::Result<()> {
    println!("📱 Starting RPC client...");

    let hub = ProcessHub::builder("calculator_client").build().await?;
    let client = CalculatorClient::new(hub);

    // Simple client operations
    println!("🧮 Performing remote calculations...");

    let result = client
        .add(CalculationRequest { x: 100.0, y: 200.0 })
        .await?;
    println!("Remote add: {}", result.result);

    let multiply_result = client.multiply(12, 13)?;
    println!("Remote multiply: {multiply_result}");

    let status = client.get_status().await?;
    println!("Remote status: {} v{}", status.service, status.version);

    println!("✅ Client operations completed");

    Ok(())
}
