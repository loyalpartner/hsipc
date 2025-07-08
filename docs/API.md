# hsipc API 文档

## 核心组件

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