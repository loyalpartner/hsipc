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

async fn run_single_process_test() -> Result<()> {
    info!("=== Single Process Test ===");

    // Create hub
    let hub = ProcessHub::new("test_hub").await?;
    info!("Hub created");

    // Register basic service
    let service = BasicCalculatorService::new(BasicCalculator);
    info!(
        "Service created: name={}, methods={:?}",
        service.name(),
        service.methods()
    );
    hub.register_service(service).await?;
    info!("Service registered");

    // Wait a bit for registration to settle
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Test direct hub call
    info!("Testing direct hub call:");
    match hub.call::<_, i32>("Calculator.add", (10, 5)).await {
        Ok(result) => info!("✅ Direct call result: {result}"),
        Err(e) => info!("❌ Direct call failed: {e}"),
    }

    // Test with typed client using same hub
    info!("Testing typed client:");
    let client = CalculatorClient::new("test_hub_client").await?;
    match client.add((10, 5)).await {
        Ok(result) => info!("✅ Client call result: {result}"),
        Err(e) => info!("❌ Client call failed: {e}"),
    }

    // Test multiplication
    match client.multiply((7, 8)).await {
        Ok(result) => info!("✅ Multiplication result: {result}"),
        Err(e) => info!("❌ Multiplication failed: {e}"),
    }

    // Test factorial
    match client.factorial(5).await {
        Ok(result) => info!("✅ Factorial result: {result}"),
        Err(e) => info!("❌ Factorial failed: {e}"),
    }

    info!("Single process test completed! ✅");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        info!("Usage:");
        info!("  {} test    - Run service test", args[0]);
        info!("  {} demo    - Run trait interface demonstration", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "test" => run_single_process_test().await,
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
