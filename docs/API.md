# hsipc API 文档

## 🚀 RPC 系统设计

hsipc 基于 jsonrpsee 风格设计，提供类型安全、高性能的进程间通信框架。

### 设计原则

- **类型安全**: 编译时检查所有 RPC 调用
- **异步优先**: 默认异步模式，支持同步模式
- **订阅模式**: 基于 PendingSubscriptionSink 的灵活订阅机制
- **自动代码生成**: 自动生成服务器和客户端代码
- **命名空间隔离**: 支持服务命名空间，避免冲突
- **泛型支持**: 完整支持泛型 trait

### 核心宏系统

#### `#[rpc]` 主宏

##### 异步模式（默认）

```rust
#[rpc(server, client, namespace = "calculator")]
pub trait Calculator<T, E> 
where
    T: Send + Sync + Clone,
    E: Send + Sync + std::error::Error,
{
    /// 异步方法
    #[method(name = "add")]
    async fn add(&self, a: T, b: T) -> Result<T, E>;
    
    /// 带超时的方法
    #[method(name = "divide", timeout = 5000)]
    async fn divide(&self, a: T, b: T) -> Result<T, E>;
    
    /// 订阅方法 - trait 定义中不包含 PendingSubscriptionSink
    #[subscription(name = "results", item = CalculationResult<T>)]
    async fn subscribe_results(&self, filter: Option<String>) -> SubscriptionResult;
}
```

##### 同步模式

```rust
#[rpc(server, client, namespace = "calculator", sync)]
pub trait SyncCalculator<T, E>
where
    T: Send + Sync + Clone,
    E: Send + Sync + std::error::Error,
{
    /// 同步方法
    #[method(name = "add")]
    fn add(&self, a: T, b: T) -> Result<T, E>;
    
    /// 同步订阅 - 无返回值
    #[subscription(name = "events", item = EventType<T>)]
    fn subscribe_events(&self, filter: Option<String>);
}
```

**参数说明:**
- `server`: 生成服务器端代码
- `client`: 生成客户端代码  
- `namespace`: 服务命名空间（必需）
- `sync`: 可选，标记为同步模式

#### `#[method]` 属性宏

```rust
#[method(name = "method_name")]                    // 普通方法
#[method(name = "method_name", timeout = 5000)]    // 带超时的方法
```

**参数说明:**
- `name`: RPC 方法名称（必需）
- `timeout`: 方法超时时间（毫秒，可选）

#### `#[subscription]` 属性宏

```rust
// 基本订阅
#[subscription(name = "events", item = EventType)]
async fn subscribe_events(&self, filter: String) -> SubscriptionResult;

// 覆盖模式订阅（如果已存在则覆盖）
#[subscription(name = "events" => "override", item = EventType)]  
async fn subscribe_events(&self, filter: String) -> SubscriptionResult;

// 单次订阅
#[subscription(name = "events" => "once", item = EventType)]
async fn subscribe_events(&self) -> SubscriptionResult;
```

**参数说明:**
- `name`: 订阅名称（必需）
- `item`: 订阅事件类型（必需）
- `=> "override"`: 覆盖现有订阅
- `=> "once"`: 单次订阅模式

## 完整示例

### 异步模式示例

```rust
use hsipc::{rpc, method, subscription, ProcessHub, Result, SubscriptionResult, PendingSubscriptionSink};
use serde::{Deserialize, Serialize};

// 定义自定义错误
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum CalculatorError {
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Overflow occurred")]
    Overflow,
}

// 定义计算结果事件
#[derive(Debug, Serialize, Deserialize)]
pub struct CalculationResult {
    pub operation: String,
    pub result: f64,
    pub timestamp: u64,
}

// 异步服务接口定义
#[rpc(server, client, namespace = "calculator")]
pub trait Calculator {
    /// 异步加法操作
    #[method(name = "add")]
    async fn add(&self, a: i32, b: i32) -> Result<i32>;
    
    /// 带错误处理的除法操作
    #[method(name = "divide", timeout = 5000)]
    async fn divide(&self, a: i32, b: i32) -> Result<f64, CalculatorError>;
    
    /// 订阅计算结果 - 注意实现时会自动插入 PendingSubscriptionSink
    #[subscription(name = "results", item = CalculationResult)]
    async fn subscribe_results(&self, filter: Option<String>) -> SubscriptionResult;
}

// 服务实现 - 宏自动添加 #[async_trait]
pub struct CalculatorImpl;

impl Calculator for CalculatorImpl {
    async fn add(&self, a: i32, b: i32) -> Result<i32> {
        Ok(a + b)
    }
    
    async fn divide(&self, a: i32, b: i32) -> Result<f64, CalculatorError> {
        if b == 0 {
            Err(CalculatorError::DivisionByZero)
        } else {
            Ok(a as f64 / b as f64)
        }
    }
    
    // 实现时自动插入 PendingSubscriptionSink 参数
    async fn subscribe_results(
        &self, 
        pending: PendingSubscriptionSink, 
        filter: Option<String>
    ) -> SubscriptionResult {
        // 可以选择接受或拒绝订阅
        if filter.is_none() {
            return pending.reject("Filter is required").await;
        }
        
        // 接受订阅
        let sink = pending.accept().await?;
        
        // 启动后台任务发送事件
        tokio::spawn(async move {
            loop {
                let result = CalculationResult {
                    operation: filter.as_deref().unwrap_or("default").to_string(),
                    result: 42.0,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                
                if sink.send_value(result).await.is_err() {
                    break; // 客户端断开连接
                }
                
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
        
        Ok(())
    }
}

// 使用示例
#[tokio::main]
async fn main() -> Result<()> {
    let hub = ProcessHub::new("calculator_app").await?;
    
    // 注册服务 - 生成的是 CalculatorService，不是 CalculatorServiceService
    let service = CalculatorService::new(CalculatorImpl);
    hub.register_service(service).await?;
    
    // 使用客户端 - 生成的是 CalculatorClient
    let client = CalculatorClient::new(hub.clone());
    
    // 异步方法调用
    let result = client.add(10, 20).await?;
    println!("10 + 20 = {}", result);
    
    // 错误处理示例
    match client.divide(10, 0).await {
        Ok(result) => println!("10 / 0 = {}", result),
        Err(e) => println!("Expected error: {}", e),
    }
    
    // 订阅示例
    let mut subscription = client.subscribe_results(Some("add".to_string())).await?;
    
    // 接收事件
    while let Some(event_result) = subscription.next().await {
        match event_result {
            Ok(event) => println!("Received: {:?}", event),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    Ok(())
}
```

### 同步模式示例

```rust
use hsipc::{rpc, method, subscription, ProcessHub, Result, PendingSubscriptionSink};

// 同步服务接口定义
#[rpc(server, client, namespace = "calculator", sync)]
pub trait SyncCalculator {
    /// 同步加法操作
    #[method(name = "add")]
    fn add(&self, a: i32, b: i32) -> Result<i32>;
    
    /// 同步订阅 - 无返回值
    #[subscription(name = "events", item = i32)]
    fn subscribe_events(&self, filter: Option<String>);
}

// 同步服务实现 - 不需要 #[async_trait]
pub struct SyncCalculatorImpl;

impl SyncCalculator for SyncCalculatorImpl {
    fn add(&self, a: i32, b: i32) -> Result<i32> {
        Ok(a + b)
    }
    
    // 同步订阅实现 - 自动插入 PendingSubscriptionSink，无返回值
    fn subscribe_events(&self, pending: PendingSubscriptionSink, filter: Option<String>) {
        // 启动异步任务处理订阅
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let sink = pending.accept().await.unwrap();
                
                // 发送同步事件
                for i in 0..10 {
                    if sink.send_value(i).await.is_err() {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            });
        });
    }
}

// 同步使用示例
fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    
    rt.block_on(async {
        let hub = ProcessHub::new("sync_calculator").await?;
        
        // 注册同步服务 - 生成的是 SyncCalculatorService
        let service = SyncCalculatorService::new(SyncCalculatorImpl);
        hub.register_service(service).await?;
        
        // 使用同步客户端 - 生成的是 SyncCalculatorClient
        let client = SyncCalculatorClient::new(hub.clone());
        
        // 同步方法调用 - 不需要 .await
        let result = client.add(5, 3)?;
        println!("5 + 3 = {}", result);
        
        // 同步订阅
        let subscription = client.subscribe_events(Some("test".to_string()))?;
        
        Ok(())
    })
}

// 使用示例
#[tokio::main]
async fn main() -> Result<()> {
    // 创建 ProcessHub
    let hub = ProcessHub::new("calculator_app").await?;
    
    // 注册服务（自动生成 CalculatorService）
    let calculator = CalculatorImpl;
    let service = CalculatorService::new(calculator);
    hub.register_service(service).await?;
    
    // 使用客户端（自动生成 CalculatorClient）
    let client = CalculatorClient::new(hub.clone());
    
    // 异步方法调用
    let add_result = client.add(15, 25).await?;
    println!("15 + 25 = {}", add_result);
    
    // 同步方法调用
    let multiply_result = client.multiply(6, 7)?;
    println!("6 * 7 = {}", multiply_result);
    
    // 错误处理示例
    match client.divide(10, 0).await {
        Ok(result) => println!("10 / 0 = {}", result),
        Err(e) => println!("Expected error: {}", e),
    }
    
    // 复杂参数示例
    let params = ComplexParams {
        operation: "average".to_string(),
        values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        options: std::collections::HashMap::new(),
    };
    let complex_result = client.complex_calculation(params).await?;
    println!("Complex result: {}", complex_result);
    
    // 订阅示例
    let _results_subscription = client.subscribe_results(Some("add".to_string())).await?;
    let _errors_subscription = client.subscribe_errors()?;
    
    Ok(())
}
```

### 多服务支持

```rust
// 用户服务
#[rpc(server, client, namespace = "user")]
pub trait UserService {
    #[method(name = "get_profile")]
    async fn get_profile(&self, user_id: u64) -> Result<UserProfile>;
    
    #[method(name = "update_profile")]
    async fn update_profile(&self, user_id: u64, profile: UserProfile) -> Result<()>;
}

// 订单服务
#[rpc(server, client, namespace = "order")]
pub trait OrderService {
    #[method(name = "create_order")]
    async fn create_order(&self, order: OrderRequest) -> Result<Order>;
    
    #[subscription(name = "status_changes", item = OrderStatusChange)]
    async fn subscribe_status_changes(&self, order_id: u64) -> SubscriptionResult;
}

// 在同一个应用中使用多个服务
#[tokio::main]
async fn main() -> Result<()> {
    let hub = ProcessHub::new("multi_service_app").await?;
    
    // 注册多个服务
    hub.register_service(UserService::new(UserServiceImpl)).await?;
    hub.register_service(OrderService::new(OrderServiceImpl)).await?;
    
    // 使用多个客户端
    let user_client = UserClient::new(hub.clone());
    let order_client = OrderClient::new(hub.clone());
    
    // 跨服务调用
    let profile = user_client.get_profile(1).await?;
    let order = order_client.create_order(OrderRequest {
        user_id: profile.id,
        items: vec![/* ... */],
    }).await?;
    
    Ok(())
}
```

### 跨进程通信

```rust
// 服务器进程
#[tokio::main]
async fn server_main() -> Result<()> {
    let hub = ProcessHub::new("calculator_server").await?;
    
    // 注册服务
    let service = CalculatorService::new(CalculatorImpl);
    hub.register_service(service).await?;
    
    // 保持运行
    tokio::signal::ctrl_c().await?;
    Ok(())
}

// 客户端进程
#[tokio::main]
async fn client_main() -> Result<()> {
    let hub = ProcessHub::new("calculator_client").await?;
    
    // 连接到远程服务
    let client = CalculatorClient::new(hub);
    
    // 调用远程方法
    let result = client.add(100, 200).await?;
    println!("Remote calculation: 100 + 200 = {}", result);
    
    Ok(())
}
```

### 性能优化

#### 批量操作

```rust
// 并发调用
let batch_calls = vec![
    client.add(1, 2),
    client.add(3, 4),
    client.add(5, 6),
];

let results = futures::future::join_all(batch_calls).await;
```

#### 零拷贝参数（规划中）

```rust
#[method(name = "process_data")]
async fn process_data(&self, data: &[u8]) -> Result<Vec<u8>>;
```

### 错误处理

#### 统一错误类型

所有 RPC 方法都返回 `Result<T, E>`，其中 `E` 可以是：
- `hsipc::Error` - 框架内置错误
- 自定义错误类型（需实现 `Serialize + Deserialize`）

#### 错误传播

```rust
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ServiceError {
    #[error("Business logic error: {0}")]
    Business(String),
    #[error("Validation error: {field}")]
    Validation { field: String },
}

// 自动转换为框架错误
impl From<ServiceError> for hsipc::Error {
    fn from(err: ServiceError) -> Self {
        hsipc::Error::from_std(err)
    }
}
```

### 类型安全保证

- **编译时检查**: 所有参数和返回值类型在编译时验证
- **序列化安全**: 确保所有类型都实现 `Serialize + Deserialize`
- **方法存在性**: 防止调用不存在的方法
- **类型匹配**: 确保客户端和服务端类型一致

### 测试支持

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_calculator_service() {
        let service = CalculatorImpl;
        
        // 直接测试服务实现
        let result = service.add(2, 3).await.unwrap();
        assert_eq!(result, 5);
        
        // 测试错误处理
        let result = service.divide(10, 0).await;
        assert!(matches!(result, Err(CalculatorError::DivisionByZero)));
    }
    
    #[tokio::test]
    async fn test_rpc_integration() {
        let hub = ProcessHub::new("test_hub").await.unwrap();
        
        // 注册服务
        let service = CalculatorService::new(CalculatorImpl);
        hub.register_service(service).await.unwrap();
        
        // 测试客户端
        let client = CalculatorClient::new(hub);
        let result = client.add(1, 2).await.unwrap();
        assert_eq!(result, 3);
    }
}
```

### 实现状态

此 RPC 系统目前处于设计阶段，主要功能包括：

- ✅ **架构设计完成**
- ✅ **API 设计完成**
- ⏳ **宏实现** - 待开发
- ⏳ **代码生成** - 待开发
- ⏳ **集成测试** - 待开发

详细的实现计划请参考 [RPC_DESIGN.md](RPC_DESIGN.md) 和 [RPC_IMPLEMENTATION.md](RPC_IMPLEMENTATION.md)。

### 核心组件

### ProcessHub

`ProcessHub` 是 hsipc 的核心组件，负责管理进程间通信。

#### 创建 ProcessHub

```rust
use hsipc::{ProcessHub, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 异步方式
    let hub = ProcessHub::new("my_process").await?;
    
    // 同步方式
    let sync_hub = hsipc::SyncProcessHub::new("my_process")?;
    
    Ok(())
}
```

#### 方法列表

##### 服务相关

- `register_service<S: Service>(&self, service: S) -> Result<()>`
  - 注册一个服务实现
  - 服务实现必须实现 `Service` trait

- `call<T, R>(&self, method: &str, request: T) -> Result<R>`
  - 调用远程服务方法
  - `T`: 请求类型，必须实现 `Serialize`
  - `R`: 响应类型，必须实现 `Deserialize`

##### 事件相关

- `subscribe<S: Subscriber>(&self, subscriber: S) -> Result<Subscription>`
  - 订阅事件
  - 返回 `Subscription` 句柄用于管理订阅生命周期

- `publish<T: Serialize>(&self, topic: &str, payload: T) -> Result<()>`
  - 发布事件到指定主题

- `publish_event<E: Event>(&self, event: E) -> Result<()>`
  - 发布实现了 `Event` trait 的事件

### Service Trait

用于定义和实现服务。

```rust
use hsipc::{Service, async_trait, Result};

pub struct MyService;

#[async_trait]
impl Service for MyService {
    fn name(&self) -> &'static str {
        "MyService"
    }
    
    fn methods(&self) -> Vec<&'static str> {
        vec!["method1", "method2"]
    }
    
    async fn handle(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
        match method {
            "method1" => {
                let req: MyRequest = hsipc::bincode::deserialize(&payload)?;
                let response = self.handle_method1(req).await?;
                Ok(hsipc::bincode::serialize(&response)?)
            }
            _ => Err(hsipc::Error::MethodNotFound(method.to_string())),
        }
    }
}
```

### Event Trait

用于定义可发布的事件。

```rust
use hsipc::Event;
use serde::{Serialize, Deserialize};

#[derive(Event, Serialize, Deserialize)]
#[event(topic = "user/registered")]
pub struct UserRegistered {
    pub user_id: String,
    pub email: String,
    pub timestamp: u64,
}

// 自动实现 Event trait，topic() 方法返回 "user/registered"
```

### Subscriber Trait

用于订阅和处理事件。

```rust
use hsipc::{Subscriber, async_trait, Result};

pub struct UserEventHandler;

#[async_trait]
impl Subscriber for UserEventHandler {
    fn topic_pattern(&self) -> &str {
        "user/*"  // 订阅所有 user 相关事件
    }
    
    async fn handle(&mut self, topic: &str, payload: Vec<u8>) -> Result<()> {
        match topic {
            "user/registered" => {
                let event: UserRegistered = hsipc::bincode::deserialize(&payload)?;
                self.handle_user_registered(event).await
            }
            _ => Ok(()),
        }
    }
}
```

## 主题模式

hsipc 支持灵活的主题模式匹配：

### 精确匹配
```
"user/registered" 只匹配 "user/registered"
```

### 单级通配符 `+`
```
"user/+" 匹配:
- "user/registered"
- "user/deleted"
但不匹配:
- "user/profile/updated"
```

### 多级通配符 `#`
```
"user/#" 匹配:
- "user/registered"
- "user/profile/updated"
- "user/settings/password/changed"
```

### 动态主题
```rust
#[derive(Event, Serialize, Deserialize)]
#[event(topic = "device/{device_id}/status")]
pub struct DeviceStatus {
    pub device_id: String,
    pub online: bool,
}

// 发布时会自动生成主题: "device/sensor_001/status"
let event = DeviceStatus {
    device_id: "sensor_001".to_string(),
    online: true,
};
```

## 错误处理

hsipc 提供统一的错误类型：

```rust
use hsipc::{Error, Result};

match hub.call("service.method", request).await {
    Ok(response) => {
        // 处理成功响应
    }
    Err(Error::Timeout) => {
        // 请求超时
    }
    Err(Error::ServiceNotFound(service)) => {
        // 服务未找到
    }
    Err(Error::MethodNotFound(method)) => {
        // 方法未找到
    }
    Err(Error::ConnectionLost) => {
        // 连接丢失
    }
    Err(e) => {
        // 其他错误
        eprintln!("Error: {}", e);
    }
}
```

## 同步 API

对于不使用 async/await 的应用：

```rust
use hsipc::SyncProcessHub;

fn main() -> hsipc::Result<()> {
    let hub = SyncProcessHub::new("my_process")?;
    
    // 同步服务调用
    let result: Response = hub.call("service.method", request)?;
    
    // 同步事件发布
    hub.publish("events/something", data)?;
    
    Ok(())
}
```

## 高级功能

### 订阅管理

```rust
// 创建订阅
let subscription = hub.subscribe(MySubscriber).await?;

// 手动取消订阅
subscription.unsubscribe().await?;

// 或使用 RAII（当 subscription 离开作用域时自动取消）
{
    let _subscription = hub.subscribe(MySubscriber).await?;
    // 在这里处理事件
} // 自动取消订阅
```

### 批量操作

```rust
// 批量发布事件
let events = vec![
    ("topic1", data1),
    ("topic2", data2),
    ("topic3", data3),
];

for (topic, data) in events {
    hub.publish(topic, data).await?;
}
```

### 超时控制

```rust
use tokio::time::{timeout, Duration};

// 设置自定义超时
match timeout(Duration::from_secs(5), hub.call("slow.service", request)).await {
    Ok(Ok(response)) => {
        // 成功响应
    }
    Ok(Err(e)) => {
        // 服务错误
    }
    Err(_) => {
        // 超时
    }
}
```

## 最佳实践

### 1. 服务设计

- 保持服务方法简单和专一
- 使用明确的请求/响应类型
- 实现适当的错误处理
- 考虑向后兼容性

### 2. 事件设计

- 使用描述性的主题名称
- 包含足够的上下文信息
- 保持事件数据的稳定性
- 考虑事件的顺序性

### 3. 错误处理

- 总是处理可能的错误情况
- 提供有意义的错误信息
- 实现重试机制（如果需要）
- 记录重要的错误

### 4. 性能优化

- 重用 ProcessHub 实例
- 合理设置超时时间
- 考虑消息大小的影响
- 使用批量操作（如果适用）