// Quick test file for new RPC macro design

#[path = "hsipc/src/lib.rs"]
mod hsipc;

use hsipc::{rpc, method, subscription, Result, SubscriptionResult, PendingSubscriptionSink};

// Test async mode (default)
#[rpc(server, client, namespace = "calculator")]
pub trait Calculator {
    #[method(name = "add")]
    async fn add(&self, a: i32, b: i32) -> Result<i32>;
}

// Test sync mode
#[rpc(server, client, namespace = "sync_calc", sync)]
pub trait SyncCalculator {
    #[method(name = "multiply")]
    fn multiply(&self, a: i32, b: i32) -> Result<i32>;
}

// Test basic compilation and naming
fn test_naming() {
    // Should be CalculatorService, not CalculatorServiceService
    // Should be CalculatorClient
    let _test: Option<CalculatorService<()>> = None;
    let _test: Option<CalculatorClient> = None;
    
    // Sync versions
    let _test: Option<SyncCalculatorService<()>> = None;
    let _test: Option<SyncCalculatorClient> = None;
}

fn main() {
    println!("New RPC macro compiled successfully!");
    test_naming();
}