//! Request/Response pattern example using new RPC macros

use hsipc::{method, rpc, ProcessHub, Result, Service};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

// Calculator service using new RPC macro
#[rpc(server, client, namespace = "calculator")]
pub trait Calculator {
    #[method(name = "add")]
    async fn add(&self, params: (i32, i32)) -> Result<i32>;

    #[method(name = "subtract")]
    async fn subtract(&self, params: (i32, i32)) -> Result<i32>;

    #[method(name = "multiply")]
    async fn multiply(&self, params: (i32, i32)) -> Result<i32>;

    #[method(name = "divide")]
    async fn divide(&self, params: (i32, i32)) -> Result<f64>;
}

#[derive(Debug)]
pub struct CalculatorImpl;

#[hsipc::async_trait]
impl Calculator for CalculatorImpl {
    async fn add(&self, params: (i32, i32)) -> Result<i32> {
        let (a, b) = params;
        let result = a + b;
        println!("🧮 Computing: {a} + {b} = {result}");
        Ok(result)
    }

    async fn subtract(&self, params: (i32, i32)) -> Result<i32> {
        let (a, b) = params;
        let result = a - b;
        println!("🧮 Computing: {a} - {b} = {result}");
        Ok(result)
    }

    async fn multiply(&self, params: (i32, i32)) -> Result<i32> {
        let (a, b) = params;
        let result = a * b;
        println!("🧮 Computing: {a} × {b} = {result}");
        Ok(result)
    }

    async fn divide(&self, params: (i32, i32)) -> Result<f64> {
        let (a, b) = params;
        if b == 0 {
            return Err("Division by zero: denominator cannot be zero".into());
        }
        let result = a as f64 / b as f64;
        println!("🧮 Computing: {a} ÷ {b} = {result:.2}");
        Ok(result)
    }
}

// User management service trait and types
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

// User service using new RPC macro
#[rpc(server, client, namespace = "user")]
pub trait UserService {
    #[method(name = "create_user")]
    async fn create_user(&self, req: CreateUserRequest) -> Result<User>;

    #[method(name = "get_user")]
    async fn get_user(&self, id: u32) -> Result<Option<User>>;

    #[method(name = "delete_user")]
    async fn delete_user(&self, id: u32) -> Result<bool>;
}

#[derive(Debug)]
pub struct UserServiceImpl;

#[hsipc::async_trait]
impl UserService for UserServiceImpl {
    async fn create_user(&self, req: CreateUserRequest) -> Result<User> {
        let user = User {
            id: rand::random(),
            name: req.name,
            email: req.email,
        };
        println!("👤 Created user: {user:?}");
        Ok(user)
    }

    async fn get_user(&self, id: u32) -> Result<Option<User>> {
        println!("🔍 Looking up user with ID: {id}");
        // Simulate some users
        let user = match id % 3 {
            0 => Some(User {
                id,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            }),
            1 => Some(User {
                id,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            }),
            _ => None,
        };
        Ok(user)
    }

    async fn delete_user(&self, id: u32) -> Result<bool> {
        println!("🗑️  Deleting user with ID: {id}");
        let success = id % 2 == 0; // Simulate some deletions fail
        Ok(success)
    }
}

async fn run_services(hub: ProcessHub) -> Result<()> {
    println!("🚀 Starting services...");

    // Register services using macro-generated wrappers
    let calculator_service = CalculatorService::new(CalculatorImpl);
    let user_service_wrapper = UserServiceService::new(UserServiceImpl);

    println!(
        "📝 Calculator service methods: {:?}",
        calculator_service.methods()
    );
    println!(
        "📝 UserService methods: {:?}",
        user_service_wrapper.methods()
    );

    hub.register_service(calculator_service).await?;
    hub.register_service(user_service_wrapper).await?;

    println!("✅ Services registered and ready!");

    // Additional wait to ensure services are fully registered
    sleep(Duration::from_secs(1)).await;

    // Keep services running
    loop {
        sleep(Duration::from_secs(1)).await;
    }
}

async fn run_client(hub: ProcessHub) -> Result<()> {
    println!("📞 Starting client...");

    // Wait for services to be ready and service discovery
    sleep(Duration::from_secs(2)).await;

    println!("🎯 Testing Calculator service with direct hub calls...");

    // Test calculator operations using generated clients
    let calc_client = CalculatorClient::new(hub.clone());

    println!("🔍 Testing Calculator.add...");
    match calc_client.add((10, 5)).await {
        Ok(result) => println!("✅ Add result: {result}"),
        Err(e) => println!("❌ Add failed: {e}"),
    }

    println!("🔍 Testing Calculator.multiply...");
    match calc_client.multiply((6, 7)).await {
        Ok(result) => println!("✅ Multiply result: {result}"),
        Err(e) => println!("❌ Multiply failed: {e}"),
    }

    println!("\n👥 Testing User service with generated client...");

    // Test user operations using generated client
    let user_client = UserServiceClient::new(hub.clone());

    let create_req = CreateUserRequest {
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
    };
    println!("🔍 Testing UserService.create_user...");
    match user_client.create_user(create_req).await {
        Ok(user) => println!("✅ Created user: {user:?}"),
        Err(e) => println!("❌ Create user failed: {e}"),
    }

    println!("🔍 Testing UserService.get_user...");
    match user_client.get_user(42u32).await {
        Ok(user) => println!("✅ Found user: {user:?}"),
        Err(e) => println!("❌ Get user failed: {e}"),
    }

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
        Some("services") => {
            let hub = ProcessHub::new("service_provider").await?;
            println!("🔄 Services running... Press Ctrl+C to stop");
            run_services(hub).await
        }
        Some("client") => {
            let hub = ProcessHub::new("client").await?;
            run_client(hub).await?;
            println!("✅ Client completed!");
            Ok(())
        }
        _ => {
            println!("Usage: {} [services|client]", args[0]);

            // For demo, run both services and client with shared hub
            println!("🎬 Running demo with both services and client...");

            let hub = ProcessHub::new("shared_hub").await?;
            let services_hub = hub.clone();
            let client_hub = hub.clone();

            let services_handle = tokio::spawn(async move {
                if let Err(e) = run_services(services_hub).await {
                    eprintln!("Services error: {e}");
                }
            });

            // Give services time to start
            sleep(Duration::from_secs(2)).await;

            let client_handle = tokio::spawn(async move {
                if let Err(e) = run_client(client_hub).await {
                    eprintln!("Client error: {e}");
                }
            });

            // Wait for client to finish
            let _ = client_handle.await;

            println!("\n🎯 Demo completed! Shutting down...");
            services_handle.abort();

            Ok(())
        }
    }
}

// Simple random number generation for demo
mod rand {
    use std::sync::atomic::{AtomicU32, Ordering};

    static SEED: AtomicU32 = AtomicU32::new(1);

    pub fn random() -> u32 {
        let seed = SEED.load(Ordering::Relaxed);
        let next = seed.wrapping_mul(1103515245).wrapping_add(12345);
        SEED.store(next, Ordering::Relaxed);
        next
    }
}
