//! Publish/Subscribe events example using macros

use hsipc::{async_trait, bincode, ProcessHub, Result, Subscriber};
use hsipc_macros::Event;
use serde::{Deserialize, Serialize};
use tokio::time::{interval, sleep, Duration};

// Define events using Event derive macro
#[derive(Event, Serialize, Deserialize, Debug, Clone)]
#[event(topic = "sensor/temperature")]
pub struct TemperatureEvent {
    pub sensor_id: String,
    pub value: f64,
    pub timestamp: u64,
}

#[derive(Event, Serialize, Deserialize, Debug, Clone)]
#[event(topic = "sensor/humidity")]
pub struct HumidityEvent {
    pub sensor_id: String,
    pub value: f64,
    pub timestamp: u64,
}

// Define subscribers
pub struct TemperatureSubscriber;

#[async_trait]
impl Subscriber for TemperatureSubscriber {
    fn topic_pattern(&self) -> &str {
        "sensor/temperature"
    }

    async fn handle(&mut self, topic: &str, payload: Vec<u8>) -> Result<()> {
        if let Ok(event) = bincode::deserialize::<TemperatureEvent>(&payload) {
            println!(
                "🌡️  [{}] Temperature: {:.1}°C from sensor {}",
                topic, event.value, event.sensor_id
            );
        }
        Ok(())
    }
}

pub struct HumiditySubscriber;

#[async_trait]
impl Subscriber for HumiditySubscriber {
    fn topic_pattern(&self) -> &str {
        "sensor/humidity"
    }

    async fn handle(&mut self, topic: &str, payload: Vec<u8>) -> Result<()> {
        if let Ok(event) = bincode::deserialize::<HumidityEvent>(&payload) {
            println!(
                "💧 [{}] Humidity: {:.1}% from sensor {}",
                topic, event.value, event.sensor_id
            );
        }
        Ok(())
    }
}

pub struct AllSensorSubscriber;

#[async_trait]
impl Subscriber for AllSensorSubscriber {
    fn topic_pattern(&self) -> &str {
        "sensor/#" // Subscribe to all sensor events
    }

    async fn handle(&mut self, topic: &str, _payload: Vec<u8>) -> Result<()> {
        println!("📊 [Monitor] Received event on topic: {}", topic);
        Ok(())
    }
}

async fn run_publisher() -> Result<()> {
    println!("🚀 Starting sensor data publisher...");

    let hub = ProcessHub::builder("sensor_publisher").build().await?;

    let mut event_count = 0;
    let max_events = 6; // Publish 6 events total (3 temp + 3 humidity)

    let mut temp_interval = interval(Duration::from_secs(2));
    let mut humidity_interval = interval(Duration::from_secs(3));

    loop {
        if event_count >= max_events {
            println!("✅ Published {} events. Publisher exiting...", event_count);
            break;
        }

        tokio::select! {
            _ = temp_interval.tick() => {
                // Publish temperature data using Event trait
                let temp = 20.0 + (rand::random::<f64>() - 0.5) * 30.0; // Random temp 5-35°C
                let event = TemperatureEvent {
                    sensor_id: "room_001".to_string(),
                    value: temp,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                println!("📤 Publishing temperature: {:.1}°C", event.value);
                hub.publish_event(event).await?;
                event_count += 1;
            }

            _ = humidity_interval.tick() => {
                // Publish humidity data using Event trait
                let humidity = 30.0 + rand::random::<f64>() * 40.0; // Random humidity 30-70%
                let event = HumidityEvent {
                    sensor_id: "room_001".to_string(),
                    value: humidity,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                println!("📤 Publishing humidity: {:.1}%", event.value);
                hub.publish_event(event).await?;
                event_count += 1;
            }
        }
    }

    // Explicitly shutdown the hub
    println!("🛑 Shutting down publisher...");
    hub.shutdown().await?;
    
    Ok(())
}

async fn run_subscriber() -> Result<()> {
    println!("👂 Starting event subscribers...");

    let hub = ProcessHub::builder("event_subscriber").build().await?;

    // Register subscribers
    let temp_subscriber = TemperatureSubscriber;
    let humidity_subscriber = HumiditySubscriber;
    let monitor_subscriber = AllSensorSubscriber;

    let _temp_sub = hub.subscribe(temp_subscriber).await?;
    let _humidity_sub = hub.subscribe(humidity_subscriber).await?;
    let _monitor_sub = hub.subscribe(monitor_subscriber).await?;

    println!("✅ Subscribers registered, waiting for events...");

    // Keep subscribers running for a limited time
    println!("⏱️  Subscribing for 20 seconds...");
    sleep(Duration::from_secs(20)).await;
    
    println!("✅ Subscriber session completed. Exiting...");
    
    // Explicitly shutdown the hub
    println!("🛑 Shutting down subscriber...");
    hub.shutdown().await?;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with line numbers and compact format
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_file(true)
        .with_target(false)
        .compact()
        .init();

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("publisher") => run_publisher().await,
        Some("subscriber") => run_subscriber().await,
        _ => {
            println!("Usage: {} [publisher|subscriber]", args[0]);

            // For demo, run both publisher and subscriber
            println!("Running demo with both publisher and subscriber...");

            let subscriber_handle = tokio::spawn(async {
                if let Err(e) = run_subscriber().await {
                    eprintln!("Subscriber error: {e}");
                }
            });

            // Give subscriber time to start
            sleep(Duration::from_secs(1)).await;

            let publisher_handle = tokio::spawn(async {
                if let Err(e) = run_publisher().await {
                    eprintln!("Publisher error: {e}");
                }
            });

            // Run for a while to let both complete
            sleep(Duration::from_secs(25)).await;

            println!("🎯 Demo completed!");
            publisher_handle.abort();
            subscriber_handle.abort();

            Ok(())
        }
    }
}

// Simple random number generation for demo
mod rand {
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEED: AtomicU64 = AtomicU64::new(1);

    pub fn random<T>() -> T
    where
        T: From<f64>,
    {
        let seed = SEED.load(Ordering::Relaxed);
        let next = seed.wrapping_mul(1103515245).wrapping_add(12345);
        SEED.store(next, Ordering::Relaxed);

        let float_val = (next % 32768) as f64 / 32768.0;
        T::from(float_val)
    }
}
