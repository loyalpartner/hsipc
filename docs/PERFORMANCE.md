# hsipc 性能基准测试

## 概述

本文档描述了 hsipc 的性能基准测试套件，用于验证 README.md 中的性能声明：

- **750k+ 消息/秒的吞吐量**
- **小消息的亚毫秒延迟**
- **大负载的零拷贝传输**
- **高效的基于主题的路由**

## 基准测试架构

### 测试分层

```
┌─────────────────────────────────────┐
│        Integration Benchmarks       │ ← 端到端性能测试
├─────────────────────────────────────┤
│     Service Layer Benchmarks       │ ← RPC 调用性能
├─────────────────────────────────────┤
│     Event System Benchmarks        │ ← 发布/订阅性能
├─────────────────────────────────────┤
│    Transport Layer Benchmarks      │ ← 传输层基础性能
└─────────────────────────────────────┘
```

### 基准测试套件

#### 1. 传输层基准测试 (`transport_benchmarks`)

**目标**: 验证底层传输性能

- **消息吞吐量**: 不同负载大小下的传输速率
- **延迟测试**: 往返时间测量
- **高频消息**: 验证 750k+ 消息/秒的声明
- **并发处理**: 多发送者并发性能

**关键指标**:
- 消息吞吐量 (messages/second)
- 往返延迟 (microseconds)
- 并发扩展性

#### 2. 服务层基准测试 (`service_benchmarks`)

**目标**: 测量 RPC 调用开销

- **服务调用延迟**: echo 调用的基准延迟
- **服务吞吐量**: 不同负载大小下的 RPC 性能
- **并发服务调用**: 多客户端并发调用
- **重型计算**: 计算密集型服务性能

**关键指标**:
- RPC 调用延迟 (microseconds)
- 服务吞吐量 (calls/second)
- 并发处理能力

#### 3. 事件系统基准测试 (`event_benchmarks`)

**目标**: 验证发布/订阅系统性能

- **事件发布吞吐量**: 发布事件的速率
- **事件延迟**: 发布到接收的端到端延迟
- **高频事件**: 快速连续事件发布
- **多订阅者**: 多个订阅者接收事件的性能
- **主题匹配**: 不同主题模式的匹配性能

**关键指标**:
- 事件发布率 (events/second)
- 端到端延迟 (microseconds)
- 主题匹配效率

#### 4. 集成基准测试 (`integration_benchmarks`)

**目标**: 端到端系统性能验证

- **完整工作流**: 服务调用 + 事件发布的组合性能
- **系统负载**: 并发工作负载下的系统性能
- **批处理**: 批量数据处理性能

**关键指标**:
- 端到端工作流延迟
- 系统扩展性
- 批处理吞吐量

## 运行基准测试

### 快速开始

```bash
# 运行所有基准测试
./scripts/benchmark.sh

# 运行特定基准测试
cd hsipc
cargo bench --bench transport_benchmarks
cargo bench --bench service_benchmarks
cargo bench --bench event_benchmarks
cargo bench --bench integration_benchmarks
```

### 详细基准测试

```bash
# 运行传输层基准测试并生成详细报告
cd hsipc
cargo bench --bench transport_benchmarks -- --output-format html

# 运行服务层基准测试
cargo bench --bench service_benchmarks

# 运行事件系统基准测试
cargo bench --bench event_benchmarks

# 运行集成基准测试
cargo bench --bench integration_benchmarks
```

## 性能目标验证

### 750k+ 消息/秒吞吐量

**测试**: `transport_benchmarks::rapid_fire_messages`

此测试通过快速连续发送 1000 个小消息来验证高吞吐量能力。

**预期结果**: 
- 每秒处理 750,000+ 消息
- 64 字节消息的处理时间 < 1.33 微秒

### 亚毫秒延迟

**测试**: `service_benchmarks::echo_call`

此测试测量简单 echo 服务调用的往返时间。

**预期结果**:
- 64 字节消息的往返时间 < 1000 微秒 (1 毫秒)
- 平均延迟 < 500 微秒

### 高效主题路由

**测试**: `event_benchmarks::topic_matching`

此测试验证不同主题模式的匹配性能。

**预期结果**:
- 精确匹配: < 10 微秒
- 单级通配符: < 20 微秒
- 多级通配符: < 50 微秒

## 基准测试环境

### 系统要求

- **操作系统**: Linux, macOS, Windows
- **内存**: 至少 4GB 可用内存
- **CPU**: 多核处理器推荐
- **存储**: 至少 1GB 可用空间用于结果

### 依赖项

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

## 结果分析

### 输出格式

基准测试结果以以下格式输出：

```
test_name               time:   [lower_bound mean upper_bound]
                        change: [change_lower% change_mean% change_upper%]
```

### 关键指标

1. **吞吐量** (Throughput): 每秒处理的操作数
2. **延迟** (Latency): 单个操作的时间
3. **扩展性** (Scalability): 性能随负载变化的情况

### HTML 报告

运行基准测试后，详细的 HTML 报告将生成在：
- `target/criterion/reports/index.html`
- `benchmark_results/` 目录 (通过 `benchmark.sh` 脚本运行时)

## 性能优化建议

### 传输层优化

1. **批量处理**: 将多个小消息合并为批次
2. **连接池**: 重用传输连接
3. **压缩**: 对大负载使用压缩

### 服务层优化

1. **异步处理**: 使用异步服务方法
2. **连接复用**: 重用客户端连接
3. **负载均衡**: 在多个服务实例之间分配负载

### 事件系统优化

1. **批量订阅**: 减少订阅开销
2. **主题索引**: 优化主题匹配算法
3. **异步处理**: 异步事件处理

## 持续性能监控

### 回归测试

```bash
# 运行性能回归测试
cargo bench --bench integration_benchmarks -- --save-baseline main

# 与之前的基准比较
cargo bench --bench integration_benchmarks -- --baseline main
```

### 性能分析

```bash
# 使用 perf 进行详细分析
perf record --call-graph=dwarf cargo bench --bench transport_benchmarks
perf report
```

## 故障排除

### 常见问题

1. **低吞吐量**: 检查系统资源限制
2. **高延迟**: 检查网络配置和系统负载
3. **不稳定结果**: 增加测量时间或样本数量

### 调试技巧

```bash
# 详细输出
RUST_LOG=trace cargo bench --bench transport_benchmarks

# 使用单线程运行
cargo bench --bench transport_benchmarks -- --test-threads=1
```

## 贡献指南

### 添加新基准测试

1. 在适当的基准测试文件中添加新的测试函数
2. 更新文档描述新的测试场景
3. 运行完整的基准测试套件验证

### 性能改进

1. 识别性能瓶颈
2. 实现优化方案
3. 运行基准测试验证改进
4. 更新性能文档

## 参考资料

- [Criterion.rs 文档](https://docs.rs/criterion/)
- [Rust 性能优化指南](https://nnethercote.github.io/perf-book/)
- [IPMB 性能特性](https://github.com/bytedance/ipmb)