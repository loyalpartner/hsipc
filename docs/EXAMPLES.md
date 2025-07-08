# hsipc 示例说明

本文档详细说明了 hsipc 框架提供的示例程序。

## 示例概览

| 示例 | 功能 | 难度 | 描述 |
|------|------|------|------|
| [simple_service](#simple_service) | 请求/响应 | 初级 | 基础的计算器服务 |
| [pubsub_events](#pubsub_events) | 发布/订阅 | 初级 | 传感器数据的事件系统 |

## simple_service

一个完整的请求/响应服务示例，实现了计算器功能。

### 功能特性

- ✅ 多种数学运算（加法、乘法、除法）
- ✅ 类型安全的请求/响应
- ✅ 错误处理（除零错误）
- ✅ 客户端-服务端分离
- ✅ 超时处理

### 文件结构

```
examples/simple_service/
├── Cargo.toml
└── src/
    └── main.rs
```

### 核心组件

#### 1. 数据类型定义

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct AddRequest {
    pub a: i32,
    pub b: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MultiplyRequest {
    pub a: i32,
    pub b: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DivideRequest {
    pub a: f64,
    pub b: f64,
}
```

#### 2. 服务实现

```rust
pub struct CalculatorService;

#[async_trait]
impl Service for CalculatorService {
    fn name(&self) -> &'static str {
        "CalculatorService"
    }
    
    fn methods(&self) -> Vec<&'static str> {
        vec!["add", "multiply", "divide"]
    }
    
    async fn handle(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
        match method {
            "add" => { /* 加法实现 */ }
            "multiply" => { /* 乘法实现 */ }
            "divide" => { /* 除法实现，包含除零检查 */ }
            _ => Err(hsipc::Error::MethodNotFound(method.to_string())),
        }
    }
}
```

### 运行示例

#### 方式一：自动演示
```bash
cd examples/simple_service
cargo run
```

输出：
```
Starting calculator server...
Calculator server ready, waiting for requests...
Starting calculator client...
Client: Testing calculator service...
Server: Computing 10 + 20
Client: 10 + 20 = 30
Server: Computing 6 * 7
Client: 6 * 7 = 42
Server: Computing 15 / 3
Client: 15.0 / 3.0 = 5
Server: Computing 10 / 0
Client: Expected error for division by zero: Other error: Remote error: Service error: Other error: Division by zero
Client: All tests completed!
Demo completed!
```

#### 方式二：分离运行

```bash
# 终端 1 - 启动服务器
cargo run server

# 终端 2 - 运行客户端
cargo run client
```

### 学习要点

1. **服务定义**: 如何实现 `Service` trait
2. **方法路由**: 根据方法名分发请求
3. **错误处理**: 优雅地处理业务错误
4. **类型安全**: 使用强类型的请求/响应
5. **异步处理**: 非阻塞的服务处理

### 扩展建议

- 添加更多数学运算（开方、指数等）
- 实现计算历史记录
- 添加输入验证
- 实现批量计算功能

## pubsub_events

一个发布/订阅事件系统示例，模拟传感器数据的收集和处理。

### 功能特性

- ✅ 多种事件类型（温度、湿度）
- ✅ 事件发布
- ✅ 模式匹配订阅
- ✅ 异步事件处理
- ✅ 随机数据生成

### 文件结构

```
examples/pubsub_events/
├── Cargo.toml
└── src/
    └── main.rs
```

### 核心组件

#### 1. 事件定义

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemperatureEvent {
    pub sensor_id: String,
    pub value: f64,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HumidityEvent {
    pub sensor_id: String,
    pub value: f64,
    pub timestamp: u64,
}
```

#### 2. 发布者

发布者定期生成模拟的传感器数据：

```rust
async fn run_publisher() -> Result<()> {
    let hub = ProcessHub::new("sensor_publisher").await?;
    
    let mut temp_interval = interval(Duration::from_secs(2));
    let mut humidity_interval = interval(Duration::from_secs(3));
    
    loop {
        tokio::select! {
            _ = temp_interval.tick() => {
                // 发布温度数据
                let temp = 20.0 + (rand::random::<f64>() - 0.5) * 30.0;
                let event = TemperatureEvent { /* ... */ };
                hub.publish("sensor/temperature", event).await?;
            }
            
            _ = humidity_interval.tick() => {
                // 发布湿度数据
                let humidity = 30.0 + rand::random::<f64>() * 40.0;
                let event = HumidityEvent { /* ... */ };
                hub.publish("sensor/humidity", event).await?;
            }
        }
    }
}
```

#### 3. 订阅者

目前示例包含基础的订阅者框架，可以扩展为完整的事件处理器。

### 运行示例

#### 自动演示
```bash
cd examples/pubsub_events
cargo run
```

#### 分离运行
```bash
# 终端 1 - 启动订阅者
cargo run subscriber

# 终端 2 - 启动发布者
cargo run publisher
```

### 学习要点

1. **事件设计**: 如何定义事件结构
2. **发布机制**: 使用 `publish()` 发布事件
3. **订阅模式**: 主题模式匹配
4. **异步流**: 使用 `tokio::select!` 处理多个事件源
5. **时间控制**: 使用 `interval` 定时发布

### 扩展建议

- 实现完整的订阅者逻辑
- 添加事件过滤功能
- 实现事件持久化
- 添加事件聚合功能
- 实现告警机制

## 创建自己的示例

### 1. 新建示例项目

```bash
# 在 examples 目录下创建新项目
mkdir examples/my_example
cd examples/my_example

# 创建 Cargo.toml
cat > Cargo.toml << EOF
[package]
name = "my_example"
version = "0.1.0"
edition = "2021"

[dependencies]
hsipc = { path = "../../hsipc" }
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
EOF

# 创建源代码目录
mkdir src
```

### 2. 基础模板

```rust
// src/main.rs
use hsipc::{ProcessHub, Result};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct MyRequest {
    // 定义请求结构
}

#[derive(Serialize, Deserialize, Debug)]
struct MyResponse {
    // 定义响应结构
}

async fn run_server() -> Result<()> {
    let hub = ProcessHub::new("my_server").await?;
    
    // 注册服务
    // hub.register_service(MyService).await?;
    
    // 保持运行
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

async fn run_client() -> Result<()> {
    let client = ProcessHub::new("my_client").await?;
    
    // 调用服务
    // let response: MyResponse = client.call("MyService.method", request).await?;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    match args.get(1).map(|s| s.as_str()) {
        Some("server") => run_server().await,
        Some("client") => run_client().await,
        _ => {
            println!("Usage: {} [server|client]", args[0]);
            Ok(())
        }
    }
}
```

### 3. 添加到工作空间

在根目录的 `Cargo.toml` 中添加：

```toml
[workspace]
members = [
    "hsipc",
    "hsipc-macros",
    "examples/*",  # 这行会自动包含新示例
]
```

## 常见模式

### 错误处理模式

```rust
async fn handle_request(&self, req: MyRequest) -> Result<MyResponse> {
    // 验证输入
    if req.value < 0 {
        return Err(hsipc::Error::Other(anyhow::anyhow!("Invalid input")));
    }
    
    // 处理逻辑
    let result = self.process(req).await?;
    
    Ok(MyResponse { result })
}
```

### 事件处理模式

```rust
#[async_trait]
impl Subscriber for MySubscriber {
    fn topic_pattern(&self) -> &str {
        "events/*"
    }
    
    async fn handle(&mut self, topic: &str, payload: Vec<u8>) -> Result<()> {
        match topic {
            "events/user_action" => {
                let event: UserAction = hsipc::bincode::deserialize(&payload)?;
                self.handle_user_action(event).await
            }
            "events/system_alert" => {
                let event: SystemAlert = hsipc::bincode::deserialize(&payload)?;
                self.handle_system_alert(event).await
            }
            _ => {
                tracing::warn!("Unknown event topic: {}", topic);
                Ok(())
            }
        }
    }
}
```

### 配置模式

```rust
#[derive(Serialize, Deserialize)]
struct Config {
    server_name: String,
    timeout_ms: u64,
    max_retries: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_name: "default_server".to_string(),
            timeout_ms: 5000,
            max_retries: 3,
        }
    }
}
```

## 调试技巧

### 启用详细日志

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 启用所有日志
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();
    
    // 你的代码
}
```

### 添加调试输出

```rust
async fn handle(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
    tracing::info!("Handling method: {}", method);
    tracing::debug!("Payload size: {} bytes", payload.len());
    
    let result = match method {
        "test" => {
            tracing::debug!("Processing test method");
            // 处理逻辑
        }
        _ => return Err(hsipc::Error::MethodNotFound(method.to_string())),
    };
    
    tracing::info!("Method {} completed successfully", method);
    Ok(result)
}
```

### 性能监控

```rust
use std::time::Instant;

async fn handle(&self, method: &str, payload: Vec<u8>) -> Result<Vec<u8>> {
    let start = Instant::now();
    
    let result = self.process_method(method, payload).await?;
    
    let duration = start.elapsed();
    tracing::info!("Method {} took {:?}", method, duration);
    
    Ok(result)
}
```

## 贡献新示例

欢迎贡献新的示例！请遵循以下准则：

1. **完整性**: 示例应该是完整可运行的
2. **文档**: 提供清晰的注释和说明
3. **简洁性**: 专注于演示特定功能
4. **错误处理**: 包含适当的错误处理
5. **测试**: 确保示例在不同环境下都能运行

提交 PR 时请包含：
- 示例代码
- README 说明
- 运行截图或输出示例