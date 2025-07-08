//! Trait-based service example demonstrating the enhanced approach
//!
//! This example shows how to use #[service_trait] and #[service_impl] together
//! for a more type-safe and polymorphic service design.

use hsipc::{service_impl, service_trait, ProcessHub, Result, Service};
use log::info;

// Define the service interface with typed client generation
#[service_trait]
trait Calculator {
    async fn add(&self, params: (i32, i32)) -> Result<i32>;
    async fn multiply(&self, params: (i32, i32)) -> Result<i32>;
    async fn factorial(&self, n: i32) -> Result<i64>;
}

// Basic implementation
struct BasicCalculator;

#[service_impl]
impl Calculator for BasicCalculator {
    async fn add(&self, params: (i32, i32)) -> Result<i32> {
        info!("BasicCalculator: Computing {} + {}", params.0, params.1);
        Ok(params.0 + params.1)
    }

    async fn multiply(&self, params: (i32, i32)) -> Result<i32> {
        info!("BasicCalculator: Computing {} * {}", params.0, params.1);
        Ok(params.0 * params.1)
    }

    async fn factorial(&self, n: i32) -> Result<i64> {
        info!("BasicCalculator: Computing factorial of {n}");
        if n < 0 {
            return Err(hsipc::Error::InvalidRequest(
                "Negative factorial".to_string(),
            ));
        }
        let mut result = 1i64;
        for i in 1..=n {
            result *= i as i64;
        }
        Ok(result)
    }
}

async fn demonstrate_trait_interface() -> Result<()> {
    info!("=== Demonstrating Trait Interface ===");

    // This function shows how we can work with the Calculator trait
    async fn test_calculator_impl<T: Calculator + Send + Sync>(calc: T) -> Result<()> {
        let add_result = calc.add((3, 4)).await?;
        info!("3 + 4 = {add_result}");

        let mul_result = calc.multiply((3, 4)).await?;
        info!("3 * 4 = {mul_result}");

        let fact_result = calc.factorial(4).await?;
        info!("4! = {fact_result}");

        Ok(())
    }

    // Test with BasicCalculator
    test_calculator_impl(BasicCalculator).await?;

    Ok(())
}

async fn run_server() -> Result<()> {
    info!("=== Starting Calculator Server ===");

    // Create server hub
    let hub = ProcessHub::new("calculator_server").await?;
    info!("Server hub created");

    // Register calculator service
    let service = BasicCalculatorService::new(BasicCalculator);
    info!(
        "Service created: name={}, methods={:?}",
        service.name(),
        service.methods()
    );
    hub.register_service(service).await?;
    info!("✅ Calculator service registered and ready!");

    // Wait for service registration to propagate
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    info!("🔄 Server running... Press Ctrl+C to stop");

    // Keep server running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

async fn run_client() -> Result<()> {
    info!("=== Starting Calculator Client ===");

    // Wait a bit for server to be ready
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Create client hub
    let client = CalculatorClient::new("calculator_client").await?;
    info!("Client connected");

    // Wait for service discovery
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Test addition
    info!("📱 Testing addition:");
    match client.add((15, 25)).await {
        Ok(result) => info!("✅ 15 + 25 = {result}"),
        Err(e) => info!("❌ Addition failed: {e}"),
    }

    // Test multiplication
    info!("📱 Testing multiplication:");
    match client.multiply((8, 9)).await {
        Ok(result) => info!("✅ 8 × 9 = {result}"),
        Err(e) => info!("❌ Multiplication failed: {e}"),
    }

    // Test factorial
    info!("📱 Testing factorial:");
    match client.factorial(6).await {
        Ok(result) => info!("✅ 6! = {result}"),
        Err(e) => info!("❌ Factorial failed: {e}"),
    }

    info!("✅ Client test completed!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        info!("Usage:");
        info!("  {} server  - Run calculator server", args[0]);
        info!("  {} client  - Run calculator client", args[0]);
        info!("  {} demo    - Run trait interface demonstration", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "server" => run_server().await,
        "client" => run_client().await,
        "demo" => {
            demonstrate_trait_interface().await?;
            info!("Trait interface demonstration completed! ✅");
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}
