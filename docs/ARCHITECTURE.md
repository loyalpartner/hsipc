# hsipc 架构文档

## 总体架构

hsipc 采用分层架构设计，提供类型安全的进程间通信功能。

```
┌─────────────────────────────────────┐
│           应用层 (Application)       │
│   Services, Events, Subscribers     │
├─────────────────────────────────────┤
│           宏层 (Macro Layer)         │
│    #[service], #[event], etc.       │
├─────────────────────────────────────┤
│          API 层 (API Layer)          │
│        ProcessHub, Traits          │
├─────────────────────────────────────┤
│         协议层 (Protocol Layer)      │
│     Message, Serialization         │
├─────────────────────────────────────┤
│         传输层 (Transport Layer)     │
│       IpmbTransport, Routing       │
├─────────────────────────────────────┤
│         底层 (Foundation)            │
│          ipmb, tokio               │
└─────────────────────────────────────┘
```

## 核心组件

### 1. ProcessHub

ProcessHub 是框架的核心组件，负责：

- **服务管理**: 注册和调用服务
- **事件管理**: 发布和订阅事件
- **消息路由**: 处理消息的发送和接收
- **生命周期管理**: 管理连接和资源

```rust
pub struct ProcessHub {
    name: String,
    transport: Arc<dyn Transport>,
    service_registry: Arc<ServiceRegistry>,
    subscription_registry: Arc<SubscriptionRegistry>,
    pending_requests: Arc<RwLock<HashMap<Uuid, oneshot::Sender<Message>>>>,
}
```

#### 关键特性

- **异步优先**: 所有操作都是异步的，避免阻塞
- **类型安全**: 使用泛型确保编译时类型检查
- **错误处理**: 统一的错误类型和处理机制
- **超时机制**: 内置请求超时保护

### 2. 传输层 (Transport Layer)

传输层抽象了底层通信机制：

```rust
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    async fn send(&self, msg: Message) -> Result<()>;
    async fn recv(&self) -> Result<Message>;
    async fn close(&self) -> Result<()>;
}
```

#### 当前实现

- **IpmbTransport**: 基于共享消息总线的实现（演示版本）
- **MockTransport**: 用于测试的模拟实现

#### 设计原则

- **可插拔**: 可以轻松替换不同的传输实现
- **异步**: 非阻塞的 I/O 操作
- **容错**: 处理连接失败和恢复

### 3. 消息协议

统一的消息格式用于所有通信：

```rust
pub struct Message {
    pub id: Uuid,                    // 消息唯一标识
    pub msg_type: MessageType,       // 消息类型
    pub source: String,              // 发送者
    pub target: Option<String>,      // 接收者（可选）
    pub topic: Option<String>,       // 主题（用于路由）
    pub payload: Vec<u8>,            // 序列化的负载
    pub correlation_id: Option<Uuid>, // 关联ID（用于请求-响应）
    pub metadata: MessageMetadata,   // 元数据
}
```

#### 消息类型

- **Request**: 服务请求
- **Response**: 服务响应
- **Event**: 事件通知
- **Subscribe/Unsubscribe**: 订阅管理
- **Heartbeat**: 心跳检测
- **Error**: 错误响应

### 4. 服务注册表 (ServiceRegistry)

管理已注册的服务：

```rust
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, Arc<dyn Service>>>>,
    methods: Arc<RwLock<HashMap<String, MethodHandler>>>,
}
```

#### 功能

- **服务注册**: 注册服务实现
- **方法路由**: 将方法调用路由到正确的处理器
- **并发安全**: 支持多线程访问

### 5. 订阅注册表 (SubscriptionRegistry)

管理事件订阅：

```rust
pub struct SubscriptionRegistry {
    subscribers: Arc<DashMap<Uuid, Box<dyn Subscriber>>>,
    topic_subscriptions: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
}
```

#### 主题匹配算法

```rust
fn topic_matches(topic: &str, pattern: &str) -> bool {
    // 支持 MQTT 风格的通配符:
    // + : 单级通配符
    // # : 多级通配符
}
```

## 消息流程

### 请求-响应流程

```
客户端                    传输层                    服务端
  |                        |                        |
  |--- call() ------------>|                        |
  |                        |--- Message::Request -->|
  |                        |                        |--- handle()
  |                        |<-- Message::Response --|
  |<-- Result<T> ----------|                        |
  |                        |                        |
```

1. 客户端调用 `call()` 方法
2. 创建 Request 消息并分配 correlation_id
3. 通过传输层发送消息
4. 服务端接收消息并路由到对应服务
5. 服务处理请求并返回响应
6. 响应消息通过传输层发送回客户端
7. 客户端根据 correlation_id 匹配响应

### 发布-订阅流程

```
发布者                    传输层                    订阅者
  |                        |                        |
  |--- publish() --------->|                        |
  |                        |--- Message::Event ---->|
  |                        |                        |--- handle()
  |                        |                        |
```

1. 发布者调用 `publish()` 方法
2. 创建 Event 消息
3. 通过传输层广播消息
4. 所有匹配的订阅者接收消息
5. 订阅者处理事件

## 并发模型

### 异步处理

- **非阻塞**: 所有 I/O 操作都是异步的
- **多任务**: 使用 tokio 的任务调度
- **背压**: 通过通道缓冲区管理负载

### 线程安全

- **Arc + RwLock**: 共享状态的安全访问
- **DashMap**: 高性能的并发哈希映射
- **原子操作**: 减少锁竞争

### 错误隔离

- **独立任务**: 每个订阅者在独立任务中处理
- **错误恢复**: 单个处理器失败不影响其他组件
- **优雅降级**: 部分功能失效时保持核心功能

## 序列化策略

### 当前实现

使用 `bincode` 进行高效的二进制序列化：

- **性能**: 快速的序列化/反序列化
- **紧凑**: 小的消息大小
- **类型安全**: 编译时类型检查

### 扩展性

设计支持多种序列化格式：

```rust
pub trait Serializer {
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>>;
    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T>;
}
```

## 路由机制

### 服务路由

```
"ServiceName.method" -> 广播到所有进程 -> 匹配的服务处理
```

### 事件路由

```
"topic/pattern" -> 主题匹配 -> 所有匹配的订阅者
```

### 点对点路由

```
target: Some("process_name") -> 直接发送到指定进程
```

## 性能特性

### 消息处理

- **零拷贝**: 尽可能避免不必要的数据复制
- **批量处理**: 支持消息批处理（未来）
- **背压控制**: 通过通道缓冲区管理

### 内存管理

- **引用计数**: 使用 Arc 共享数据
- **弱引用**: 避免循环引用
- **及时清理**: 自动清理过期资源

### 网络优化

- **连接复用**: 复用传输连接
- **压缩**: 可选的消息压缩（未来）
- **连接池**: 连接池管理（未来）

## 扩展点

### 1. 传输层扩展

```rust
pub struct CustomTransport;

#[async_trait]
impl Transport for CustomTransport {
    // 实现自定义传输逻辑
}
```

### 2. 序列化扩展

```rust
pub struct JsonSerializer;

impl Serializer for JsonSerializer {
    // 实现 JSON 序列化
}
```

### 3. 中间件支持

```rust
pub trait Middleware {
    async fn before_send(&self, msg: &mut Message) -> Result<()>;
    async fn after_receive(&self, msg: &Message) -> Result<()>;
}
```

## 错误处理架构

### 错误分类

```rust
pub enum Error {
    Io(std::io::Error),              // I/O 错误
    Serialization(bincode::Error),   // 序列化错误
    Ipc(String),                     // IPC 错误
    ServiceNotFound(String),         // 服务未找到
    MethodNotFound(String),          // 方法未找到
    Timeout,                         // 超时
    ConnectionLost,                  // 连接丢失
    // ...
}
```

### 错误传播

- **Result 类型**: 统一的错误返回
- **错误转换**: 自动的错误类型转换
- **上下文保留**: 保留错误的上下文信息

## 监控和调试

### 日志集成

- **tracing**: 结构化日志
- **指标**: 性能指标收集（未来）
- **追踪**: 分布式追踪支持（未来）

### 调试工具

- **消息追踪**: 跟踪消息流转
- **性能分析**: 识别性能瓶颈
- **健康检查**: 系统健康状态监控

## 未来架构演进

### 短期目标

1. **完整的 ipmb 集成**: 替换当前的演示实现
2. **宏功能完善**: 完整的声明式 API
3. **错误处理增强**: 更好的错误恢复机制

### 长期目标

1. **分布式支持**: 跨网络的进程通信
2. **负载均衡**: 智能的请求分发
3. **服务发现**: 自动的服务注册和发现
4. **持久化**: 可靠的消息传递保证