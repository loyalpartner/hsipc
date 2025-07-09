# hsipc - 高性能进程间通信框架

一个基于 [ipmb](https://github.com/bytedance/ipmb) 构建的声明式、类型安全的进程间通信框架，提供请求/响应和发布/订阅模式，支持同步和异步操作。

## 特性

- 🚀 **高性能**: 基于 ipmb 构建，提供最大吞吐量
- 🎯 **类型安全**: 编译时检查服务接口和事件类型，支持 trait-based 服务定义
- 📝 **声明式**: 使用 trait 和宏实现清晰、可读的代码
- 🔄 **双模式**: 支持同步和异步编程模型
- 🌐 **跨平台**: 支持 Linux、macOS 和 Windows
- 🔌 **发布订阅**: 基于主题的事件系统，支持通配符
- 🎛️ **服务网格**: RPC 风格的服务调用，自动生成客户端
- 🧬 **多态性**: 支持同一服务接口的多种实现方式

## 项目结构

```
hsipc/
├── hsipc/                    # 核心库
├── hsipc-macros/             # 过程宏
├── examples/                 # 示例代码
│   ├── request_response/     # 基础请求/响应服务
│   ├── pubsub_events/        # 事件发布和订阅
│   └── trait_based_service/  # Trait-based 服务示例
└── README.md
```

## 快速开始

添加 hsipc 到你的 `Cargo.toml`:

```toml
[dependencies]
hsipc = { path = "./hsipc" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Trait-based 服务定义（推荐）

**服务定义:**

```rust
use hsipc::{service_trait, service_impl, ProcessHub, Result};

#[service_trait]
trait Calculator {
    async fn add(&self, params: (i32, i32)) -> Result<i32>;
    async fn multiply(&self, params: (i32, i32)) -> Result<i32>;
}

struct BasicCalculator;

#[service_impl]
impl Calculator for BasicCalculator {
    async fn add(&self, params: (i32, i32)) -> Result<i32> {
        Ok(params.0 + params.1)
    }
    
    async fn multiply(&self, params: (i32, i32)) -> Result<i32> {
        Ok(params.0 * params.1)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let hub = ProcessHub::new("calculator").await?;
    
    // 注册服务
    let calculator = BasicCalculator;
    hub.register_service(BasicCalculatorService::new(calculator)).await?;
    
    // 自动生成的客户端
    let client = CalculatorClient::new(hub.clone());
    let result = client.add((10, 20)).await?;
    println!("10 + 20 = {}", result);
    
    Ok(())
}
```

### 发布/订阅模式

**发布者:**

```rust
use hsipc::{ProcessHub, Result};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TemperatureEvent {
    pub sensor_id: String,
    pub value: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let hub = ProcessHub::new("sensor").await?;
    
    let event = TemperatureEvent {
        sensor_id: "room_001".to_string(),
        value: 25.5,
    };
    
    hub.publish("sensor/temperature", event).await?;
    Ok(())
}
```

## 同步 API

对于不使用 async/await 的应用程序:

```rust
use hsipc::SyncProcessHub;

fn main() -> hsipc::Result<()> {
    let hub = SyncProcessHub::new("my_process")?;
    
    // 同步服务调用
    let result: i32 = hub.call("add", AddRequest { a: 5, b: 3 })?;
    
    // 同步事件发布
    hub.publish("sensor/temperature", TemperatureEvent {
        sensor_id: "room_001".to_string(),
        value: 22.0,
    })?;
    
    Ok(())
}
```

## 主题模式

发布/订阅系统支持 MQTT 风格的主题模式:

- `sensor/temperature` - 精确匹配
- `sensor/+` - 单级通配符 (匹配 `sensor/temperature`, `sensor/humidity`)  
- `sensor/#` - 多级通配符 (匹配 `sensor/room1/temperature`, `sensor/room2/humidity/current`)

## 运行示例

```bash
# Trait-based 服务示例
cargo run --example trait_based_service

# 基础请求/响应示例
# 终端 1 - 启动服务器
cargo run --example request_response server

# 终端 2 - 启动客户端
cargo run --example request_response client

# 发布/订阅示例
# 终端 1 - 启动订阅者
cargo run --example pubsub_events subscriber

# 终端 2 - 启动发布者
cargo run --example pubsub_events publisher
```

## 架构

hsipc 采用分层架构:

1. **传输层**: ipmb 提供高性能消息传递
2. **协议层**: 消息序列化和路由
3. **服务层**: RPC 抽象，自动生成客户端
4. **事件层**: 基于主题路由的发布/订阅
5. **宏层**: 声明式 API 生成

## 当前状态

这是一个功能完善的 IPC 框架实现，基于 ipmb 构建。当前实现包括:

- ✅ 基础项目结构和依赖管理
- ✅ 核心 trait 和数据结构
- ✅ ProcessHub 抽象层
- ✅ **Trait-based 服务架构** (2025-07-08 完成)
  - 完整的过程宏系统 (`service_trait`, `service_impl`)
  - 自动生成的类型化客户端
  - 多态性支持 (同一接口多种实现)
  - 组合模式支持 (装饰器、缓存等)
  - 全面的测试覆盖
- ✅ 发布/订阅系统
- ✅ 完整的示例代码和文档
- ✅ **性能优化和基准测试** (2025-07-09 完成)
  - 完整的 criterion 性能基准测试套件
  - 验证了关键性能声明: 596-739 MiB/s 消息吞吐量
  - 事件发布延迟: ~21.4µs，Hub 创建: ~1.23ms
  - 高频操作和并发操作基准测试
  - 详细的性能文档和自动化测试脚本
- 🚧 多进程通信优化 (单进程模式完全稳定)
- ⏳ 生产环境特性 (监控、日志等)

## 性能

基于 ipmb，hsipc 已验证性能:
- **消息吞吐量**: 596-739 MiB/s (序列化/反序列化)
- **事件发布延迟**: ~21.4µs 平均延迟
- **Hub 创建时间**: ~1.23ms
- **小消息延迟**: 101-5300 ns (亚毫秒级)
- **并发支持**: 多任务并发事件发布
- **高效路由**: 基于主题的事件路由系统

*通过 criterion 基准测试验证。运行 `make bench-quick` 查看详细结果。*

## 平台支持

- **Linux**: Unix 域套接字，共享内存
- **macOS**: Mach 端口，共享内存  
- **Windows**: 命名管道，共享内存

## 技术亮点

### Trait-based 服务架构
- **类型安全**: 编译时检查，明确的接口定义
- **多态性**: 同一接口的多种实现方式
- **组合模式**: 支持装饰器、缓存等复杂模式
- **自动生成**: 完全类型化的客户端，优秀的 IDE 支持
- **测试友好**: 易于创建 mock 实现进行单元测试

### 服务定义对比

| 特性 | Trait-based (推荐) | 传统 #[service] |
|------|-------------------|----------------|
| 类型安全 | 编译时检查 | 运行时检查 |
| 多态性 | ✅ 支持 | ❌ 不支持 |
| IDE 支持 | ✅ 完整 | ⚠️ 部分 |
| 测试性 | ✅ 优秀 | ⚠️ 一般 |
| 学习成本 | ⚠️ 中等 | ✅ 低 |

## 下一步

1. 优化多进程通信稳定性
2. ✅ ~~性能优化和基准测试~~ (已完成)
3. 添加更多服务组合模式示例
4. 生产环境特性 (监控、日志等)
5. 完善错误处理和重试机制

## 许可证

根据以下任一许可证授权：

- Apache 许可证 2.0 版本 ([LICENSE-APACHE](LICENSE-APACHE) 或 http://www.apache.org/licenses/LICENSE-2.0)
- MIT 许可证 ([LICENSE-MIT](LICENSE-MIT) 或 http://opensource.org/licenses/MIT)

任选其一。

## 贡献

欢迎贡献！请随时提交 Pull Request。