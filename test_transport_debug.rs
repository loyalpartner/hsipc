use hsipc::{ProcessHub, Result};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Enable debug logging
    tracing_subscriber::fmt()
        .with_env_filter("hsipc=debug")
        .init();

    println!("🧪 Starting transport layer debug test...");
    
    // Create a hub
    let hub = ProcessHub::new("debug_hub").await?;
    println!("✅ Hub created");
    
    // Send two messages quickly
    println!("📤 Sending first message...");
    hub.publish("test/first", "First message").await?;
    println!("✅ First message sent");
    
    // Small delay
    sleep(Duration::from_millis(10)).await;
    
    println!("📤 Sending second message...");
    hub.publish("test/second", "Second message").await?;
    println!("✅ Second message sent");
    
    // Wait a bit to see if messages are processed
    sleep(Duration::from_millis(500)).await;
    
    println!("🏁 Test complete");
    
    Ok(())
}