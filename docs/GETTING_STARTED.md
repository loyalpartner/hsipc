# hsipc 快速入门

本指南将帮助您快速上手 hsipc 进程间通信框架。

## 安装

将 hsipc 添加到您的 `Cargo.toml`:

```toml
[dependencies]
hsipc = { path = "path/to/hsipc" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"  # 用于错误处理
```

## 第一个服务

让我们创建一个简单的问候服务：

### 1. 定义数据类型

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GreetRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GreetResponse {
    pub message: String,
}
```

### 2. 实现服务

```rust
use hsipc::{Service, async_trait, ProcessHub, Result};

pub struct GreetingService;

#[async_trait]
impl Service for GreetingService {
    fn name(&self) -> &'static str {
        "GreetingService"
    }
    
    fn methods(&self) -> Vec<&'static str> {
        vec!["greet"]
    }
    
    async fn handle(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
        match method {
            "greet" => {
                let req: GreetRequest = hsipc::bincode::deserialize(&payload)?;
                let response = GreetResponse {
                    message: format!("Hello, {}!", req.name),
                };
                Ok(hsipc::bincode::serialize(&response)?)
            }
            _ => Err(hsipc::Error::MethodNotFound(method.to_string())),
        }
    }
}
```

### 3. 服务端代码

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 创建 ProcessHub
    let hub = ProcessHub::new("greeting_server").await?;
    
    // 注册服务
    hub.register_service(GreetingService).await?;
    
    println!("Greeting server started!");
    
    // 保持服务运行
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
```

### 4. 客户端代码

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 创建客户端 hub
    let client = ProcessHub::new("greeting_client").await?;
    
    // 调用服务
    let request = GreetRequest {
        name: "World".to_string(),
    };
    
    let response: GreetResponse = client
        .call("GreetingService.greet", request)
        .await?;
    
    println!("Server response: {}", response.message);
    
    Ok(())
}
```

### 5. 运行

```bash
# 终端 1 - 启动服务器
cargo run --bin greeting_server

# 终端 2 - 运行客户端
cargo run --bin greeting_client
```

## 第一个事件系统

现在让我们创建一个简单的事件发布/订阅系统：

### 1. 定义事件

```rust
use hsipc::Event;
use serde::{Serialize, Deserialize};

#[derive(Event, Serialize, Deserialize, Debug, Clone)]
#[event(topic = "notifications/user_activity")]
pub struct UserActivityEvent {
    pub user_id: String,
    pub activity: String,
    pub timestamp: u64,
}
```

### 2. 实现订阅者

```rust
use hsipc::{Subscriber, async_trait, Result};

pub struct ActivityLogger;

#[async_trait]
impl Subscriber for ActivityLogger {
    fn topic_pattern(&self) -> &str {
        "notifications/*"  // 订阅所有通知事件
    }
    
    async fn handle(&mut self, topic: &str, payload: Vec<u8>) -> Result<()> {
        match topic {
            "notifications/user_activity" => {
                let event: UserActivityEvent = hsipc::bincode::deserialize(&payload)?;
                println!("[LOG] User {} performed: {} at {}", 
                        event.user_id, event.activity, event.timestamp);
            }
            _ => {
                println!("[LOG] Unknown notification: {}", topic);
            }
        }
        Ok(())
    }
}
```

### 3. 发布者代码

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let hub = ProcessHub::new("activity_publisher").await?;
    
    let event = UserActivityEvent {
        user_id: "user123".to_string(),
        activity: "logged_in".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    // 发布事件
    hub.publish_event(event).await?;
    
    println!("Event published!");
    Ok(())
}
```

### 4. 订阅者代码

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let hub = ProcessHub::new("activity_subscriber").await?;
    
    // 注册订阅者
    let _subscription = hub.subscribe(ActivityLogger).await?;
    
    println!("Waiting for events...");
    
    // 保持订阅活跃
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
```

## 常见模式

### 请求-响应服务

适用于需要即时响应的操作：

```rust
// 用户管理服务
pub struct UserService {
    db: Database,
}

#[async_trait]
impl Service for UserService {
    fn name(&self) -> &'static str { "UserService" }
    
    fn methods(&self) -> Vec<&'static str> {
        vec!["create_user", "get_user", "update_user", "delete_user"]
    }
    
    async fn handle(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
        match method {
            "create_user" => {
                let req: CreateUserRequest = hsipc::bincode::deserialize(&payload)?;
                let user = self.db.create_user(req).await?;
                Ok(hsipc::bincode::serialize(&user)?)
            }
            "get_user" => {
                let req: GetUserRequest = hsipc::bincode::deserialize(&payload)?;
                let user = self.db.get_user(req.id).await?;
                Ok(hsipc::bincode::serialize(&user)?)
            }
            // ... 其他方法
            _ => Err(hsipc::Error::MethodNotFound(method.to_string())),
        }
    }
}
```

### 事件驱动架构

适用于解耦的异步处理：

```rust
// 订单事件
#[derive(Event, Serialize, Deserialize, Debug)]
#[event(topic = "orders/created")]
pub struct OrderCreated {
    pub order_id: String,
    pub user_id: String,
    pub total: f64,
}

// 多个处理器可以订阅同一事件
pub struct EmailNotificationHandler;
pub struct InventoryUpdateHandler;
pub struct AnalyticsHandler;

// 每个处理器独立处理订单创建事件
#[async_trait]
impl Subscriber for EmailNotificationHandler {
    fn topic_pattern(&self) -> &str { "orders/*" }
    
    async fn handle(&mut self, topic: &str, payload: Vec<u8>) -> Result<()> {
        if topic == "orders/created" {
            let order: OrderCreated = hsipc::bincode::deserialize(&payload)?;
            self.send_confirmation_email(&order).await?;
        }
        Ok(())
    }
}
```

### 混合模式

同时使用服务和事件：

```rust
pub struct OrderService {
    hub: ProcessHub,
}

#[async_trait]
impl Service for OrderService {
    fn name(&self) -> &'static str { "OrderService" }
    
    fn methods(&self) -> Vec<&'static str> {
        vec!["create_order"]
    }
    
    async fn handle(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
        match method {
            "create_order" => {
                let req: CreateOrderRequest = hsipc::bincode::deserialize(&payload)?;
                
                // 1. 创建订单（同步操作）
                let order = self.create_order_in_db(req).await?;
                
                // 2. 发布事件（异步通知）
                let event = OrderCreated {
                    order_id: order.id.clone(),
                    user_id: order.user_id.clone(),
                    total: order.total,
                };
                self.hub.publish_event(event).await?;
                
                // 3. 返回响应
                Ok(hsipc::bincode::serialize(&order)?)
            }
            _ => Err(hsipc::Error::MethodNotFound(method.to_string())),
        }
    }
}
```

## 调试技巧

### 启用日志

```rust
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 启用详细日志
    tracing_subscriber::fmt::init();
    
    // 你的代码
    Ok(())
}
```

### 错误处理

```rust
match hub.call("service.method", request).await {
    Ok(response) => {
        // 处理成功
    }
    Err(hsipc::Error::Timeout) => {
        eprintln!("Request timed out - service might be down");
    }
    Err(hsipc::Error::ServiceNotFound(service)) => {
        eprintln!("Service '{}' not found - check if it's running", service);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

### 测试连接

```rust
// 简单的健康检查服务
pub struct HealthService;

#[async_trait]
impl Service for HealthService {
    fn name(&self) -> &'static str { "HealthService" }
    fn methods(&self) -> Vec<&'static str> { vec!["ping"] }
    
    async fn handle(&self, method: &str, _payload: Vec<u8>) -> Result<Vec<u8>> {
        match method {
            "ping" => Ok(hsipc::bincode::serialize(&"pong")?),
            _ => Err(hsipc::Error::MethodNotFound(method.to_string())),
        }
    }
}

// 测试连接
let response: String = client.call("HealthService.ping", ()).await?;
assert_eq!(response, "pong");
```

## 下一步

- 查看 [API 文档](API.md) 了解详细的 API 参考
- 查看 [示例目录](../examples/) 了解更多实际用例
- 阅读 [架构文档](ARCHITECTURE.md) 了解内部实现

## 常见问题

### Q: 为什么我的客户端连接超时？
A: 确保服务端已经启动并且注册了相应的服务。检查服务名和方法名是否正确。

### Q: 如何处理大量事件？
A: 考虑使用批量处理、事件聚合或者分片订阅者来提高性能。

### Q: 可以在同一个进程中运行多个服务吗？
A: 可以，一个 ProcessHub 可以注册多个服务实现。

### Q: 如何实现负载均衡？
A: 当前版本使用简单的广播机制。在生产环境中，可以扩展路由逻辑来实现负载均衡。