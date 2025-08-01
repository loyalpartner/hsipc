# hsipc 综合测试文档

## 🎯 测试架构概览

hsipc 项目采用多层次、全方位的测试策略，确保代码质量、性能和可靠性。测试系统基于 TDD 原则设计，支持快速迭代开发和持续集成。

### 测试金字塔

```
┌─────────────────────────────────────────┐
│            E2E 测试 (5%)                 │  ← 多进程集成测试
├─────────────────────────────────────────┤
│         集成测试 (15%)                   │  ← 示例验证、组件协作
├─────────────────────────────────────────┤
│         TDD 测试 (30%)                   │  ← 核心功能、RPC 系统
├─────────────────────────────────────────┤
│         单元测试 (50%)                   │  ← 模块内部逻辑
└─────────────────────────────────────────┘
```

## 📁 测试组件结构

### 核心测试文件

```
tests/
├── example_integration_tests.rs      # 示例集成测试
├── multiprocess_integration_tests.rs # 多进程通信测试
├── load_and_performance_tests.rs     # 负载和性能测试
└── test_utilities.rs                 # 测试工具和辅助函数

hsipc-macros/tests/
└── rpc_tdd_test.rs                   # TDD 核心测试 (24个测试)

scripts/
├── multiprocess_test.sh              # 多进程测试脚本
├── smart_test.sh                     # 智能测试选择
├── quick_test.sh                     # 快速测试脚本
└── benchmark.sh                      # 性能基准测试

.github/workflows/
└── ci.yml                           # CI/CD 管道配置
```

### 示例驱动测试

```
examples/rpc_system_demo/            # 合并后的综合示例
├── demo                            # 完整功能演示
├── server/client                   # RPC 通信测试
├── publisher/subscriber            # 事件系统测试
└── events                         # 事件系统综合演示
```

## 🚀 快速开始

### 基本测试命令

```bash
# 最快语法检查 (2秒)
make check

# TDD 核心测试 (10秒)
make tdd

# 快速验证 (30秒)
make quick

# 完整测试套件 (5分钟)
make full

# 智能测试选择
make smart-test
```

### 开发工作流测试

```bash
# TDD 开发循环
make tdd-red       # 编写失败测试
make tdd-green     # 实现功能
make tdd-refactor  # 重构代码
make tdd-commit    # 提交更改

# 状态检查
make status-check  # 检查当前TDD状态
```

## 🧪 测试分类详解

### 1. TDD 核心测试 (hsipc-macros/tests/rpc_tdd_test.rs)

**目的**: 验证 RPC 系统核心功能和宏生成
**测试数量**: 24个测试
**运行时间**: < 10秒
**覆盖范围**:
- RPC 宏代码生成
- 服务注册和发现
- 方法调用（同步/异步）
- 订阅协议
- 错误处理
- 并发访问

```bash
# 运行 TDD 核心测试
make tdd-core

# 实时监控
make tdd-watch
```

**关键测试场景**:
- 基本 RPC 调用
- 同步方法调用
- 订阅数据流
- 并发客户端
- 进程间通信
- 错误恢复

### 2. 示例集成测试 (tests/example_integration_tests.rs)

**目的**: 验证合并后的 rpc_system_demo 示例功能
**运行时间**: 2-15分钟
**覆盖范围**:
- 所有 CLI 命令验证
- 示例构建测试
- 功能完整性验证

```bash
# 运行示例集成测试
cargo test --test example_integration_tests

# 单独测试特定命令
cd examples/rpc_system_demo
cargo run demo           # 综合演示
cargo run server         # 服务器模式
cargo run client         # 客户端模式
cargo run events         # 事件演示
cargo run publisher      # 事件发布
cargo run subscriber     # 事件订阅
```

**测试内容**:
- ✅ 示例构建成功
- ✅ demo 命令完整性
- ✅ server 启动验证
- ✅ client-server 交互
- ✅ events 系统演示
- ✅ publisher-subscriber 通信
- ✅ CLI 命令识别

### 3. 多进程集成测试 (tests/multiprocess_integration_tests.rs)

**目的**: 验证真实的跨进程通信场景
**运行时间**: 10-20分钟
**覆盖范围**:
- 进程间 RPC 通信
- 进程间事件通信
- 并发多进程场景
- 进程生命周期管理

```bash
# 运行多进程集成测试
cargo test --test multiprocess_integration_tests

# 运行自动化多进程脚本
make multiprocess
```

**测试场景**:
- **RPC 通信**: server + client 进程交互
- **事件通信**: publisher + subscriber 进程交互  
- **并发场景**: server + client + publisher + subscriber
- **进程恢复**: 多次连接的韧性测试

### 4. 负载和性能测试 (tests/load_and_performance_tests.rs)

**目的**: 验证系统在负载下的性能和稳定性
**运行时间**: 15-25分钟
**覆盖范围**:
- RPC 调用负载测试
- 事件系统负载测试
- 高并发场景测试
- 内存使用验证

```bash
# 运行负载测试
cargo test --test load_and_performance_tests --release

# 运行基准测试
make bench-quick
```

**性能指标**:
- **RPC 吞吐量**: > 10 operations/sec
- **成功率**: > 95% (基本负载)
- **成功率**: > 90% (高并发)
- **成功率**: > 88% (事件系统)

**测试场景**:
- 基本负载测试 (5 clients × 50 ops)
- 高并发测试 (20 clients × 25 ops)
- 持续负载测试 (8 clients × 100 ops × 45s)
- 事件系统负载 (15 publishers × 30 events)
- 内存泄漏检测 (多轮重复测试)

### 5. 测试工具和辅助函数 (tests/test_utilities.rs)

**目的**: 提供统一的测试基础设施
**功能**:
- 测试环境管理
- Mock 服务和订阅者
- 进程生命周期管理
- 性能指标收集
- 测试断言助手

```rust
// 使用测试工具示例
use tests::test_utilities::*;

#[tokio::test]
async fn my_test() {
    let env = TestEnvironment::new("my_test").await?;
    let mock_service = MockCalculatorService::new();
    env.hub.register_service(mock_service.clone()).await?;
    
    let client = TestClient::new(env.hub.clone());
    let result = client.add(1.0, 2.0, 123).await?;
    
    assertions::assert_service_metrics(&mock_service, 1, 1.0).await;
}
```

## 🔄 CI/CD 测试管道

### GitHub Actions 工作流

```yaml
# .github/workflows/ci.yml 
# 分阶段测试管道，确保快速反馈和全面覆盖

快速检查 → TDD测试 → 单元测试 → 示例测试 → 多进程测试 → 性能测试
   (2分钟)   (5分钟)   (10分钟)   (5分钟)    (20分钟)    (25分钟)
```

**测试作业 (Jobs)**:

1. **quick-check**: 语法、格式、Clippy 检查
2. **tdd-tests**: TDD 核心测试 (最快反馈)
3. **unit-tests**: 单元和集成测试 (stable + beta)
4. **example-tests**: 示例验证测试
5. **multiprocess-tests**: 多进程通信测试
6. **performance-tests**: 性能测试 (仅 master 分支)
7. **cross-platform**: 跨平台测试 (Linux/macOS/Windows)
8. **security-audit**: 安全审计
9. **documentation**: 文档和覆盖率 (仅 master 分支)
10. **integration-check**: 最终集成验证

### CI 触发条件

```yaml
# 完整测试触发条件
on:
  push:
    branches: [ master, feature/* ]  # 所有功能分支
  pull_request:
    branches: [ master ]             # PR 到 master

# 性能测试仅在 master 分支触发
if: github.ref == 'refs/heads/master'
```

## 🎯 智能测试选择

### 根据修改内容自动选择测试

智能测试系统分析修改的文件，自动选择最相关的测试策略：

```bash
# 智能测试选择
make smart-test

# 或直接运行脚本
./scripts/smart_test.sh
```

**选择逻辑**:

| 修改内容 | 执行策略 | 时间 |
|---------|---------|------|
| `hsipc-macros/` | TDD 核心测试 | 10秒 |
| `rpc` 关键词 | TDD + 示例演示 | 45秒 |
| `service`, `hub` | 服务示例 + 集成测试 | 2分钟 |
| `event`, `subscription` | 事件示例 + 订阅测试 | 2分钟 |
| `transport` | 传输层单元测试 | 1分钟 |
| `examples/rpc_system_demo` | 完整演示 + 集成测试 | 3分钟 |
| `test` 文件 | 相关测试套件 | 可变 |
| 文档文件 | 语法检查 | 5秒 |
| 构建文件 | 构建验证 + 快速测试 | 1分钟 |

## 📊 性能基准和指标

### 当前验证的性能指标

基于 README.md 中的性能声明，测试套件验证以下指标：

- **消息吞吐量**: 596-739 MiB/s
- **事件发布延迟**: ~21.4µs 平均延迟  
- **Hub 创建时间**: ~1.23ms
- **小消息延迟**: 101-5300 ns (亚毫秒级)
- **并发支持**: 多任务并发事件发布

### 基准测试命令

```bash
# 快速基准测试
make bench-quick

# 核心性能测试  
make bench-core

# 完整基准测试套件
make benchmark
```

## 🔧 测试配置和定制

### 环境变量

```bash
# 测试行为控制
export RUST_BACKTRACE=1              # 详细错误信息
export CARGO_TERM_COLOR=always       # 彩色输出
export RUST_LOG=trace                # 详细日志输出

# 测试超时控制 (CI 环境)
export TEST_TIMEOUT=30                # 测试超时时间
export MULTIPROCESS_TIMEOUT=20       # 多进程测试超时
```

### Makefile 自定义

可以通过修改 Makefile 来调整测试行为：

```makefile
# 自定义快速验证流程
quick-custom:
	@echo "📋 自定义快速验证..."
	@cargo check --quiet
	@cd examples/rpc_system_demo && cargo run demo --quiet
	@cargo test --test rpc_tdd_test --quiet
```

## 🐛 故障排除和调试

### 常见问题解决

**1. 示例测试超时**
```bash
# 增加超时时间
cd examples/rpc_system_demo
timeout 120 cargo run demo  # 增加到2分钟
```

**2. 多进程测试失败**
```bash
# 检查进程状态
ps aux | grep rpc_system_demo

# 手动清理
pkill -f rpc_system_demo
```

**3. TDD 测试间歇性失败**
```bash
# 单独运行有问题的测试
cargo test --test rpc_tdd_test test_name -- --nocapture

# 检查竞态条件
cargo test --test rpc_tdd_test -- --test-threads=1
```

### 调试模式

```bash
# 详细输出模式
RUST_LOG=debug cargo test --test example_integration_tests -- --nocapture

# 单线程模式 (避免竞态条件)
cargo test -- --test-threads=1

# 特定测试调试
cargo test --test multiprocess_integration_tests test_multiprocess_rpc_communication -- --nocapture
```

## 📈 测试覆盖率

### 生成覆盖率报告

```bash
# 安装覆盖率工具
cargo install cargo-llvm-cov

# 生成覆盖率报告
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# 查看 HTML 报告
cargo llvm-cov --all-features --workspace --html
```

### 覆盖率目标

- **单元测试覆盖率**: > 80%
- **集成测试覆盖率**: > 70% 
- **总体覆盖率**: > 75%

## 🚦 测试状态和报告

### 实时状态检查

```bash
# 检查当前 TDD 状态
make status-check

# 检查 Git 状态和测试状态
make tdd-full  # 智能分析下一步操作
```

### 测试报告

每个测试类别都提供详细的执行报告：

```bash
# TDD 测试报告
✅ TDD cycle passed!
📊 Tests: 24 passed, 0 failed

# 性能测试报告  
📊 Performance Test Results:
   Duration: 15.23s
   Total Operations: 500
   Successful: 487 (97.4%)
   Operations/sec: 31.98
   Latency - Min: 12.45ms, Max: 156.78ms

# 多进程测试报告
🎉 所有多进程通信测试完成！
✅ RPC 服务器-客户端通信：通过
✅ 事件发布-订阅通信：通过  
✅ 综合功能演示：通过
```

## 📋 最佳实践

### 开发者日常测试流程

```bash
# 1. 开始开发
make status-check

# 2. TDD 开发循环
make tdd          # 智能 TDD 循环

# 3. 功能验证
make smart-test   # 根据修改选择测试

# 4. 提交前检查
make pre-commit-check

# 5. 推送前验证
make full
```

### 测试编写指南

1. **使用测试工具**: 优先使用 `test_utilities.rs` 中的辅助函数
2. **独立性**: 每个测试应该独立运行，不依赖其他测试
3. **清理**: 确保测试后正确清理资源（进程、文件等）
4. **超时**: 为长时间运行的测试设置合理的超时
5. **断言**: 使用具体的断言消息，便于故障排除

### CI/CD 集成建议

1. **分阶段执行**: 先运行快速测试，再运行耗时测试
2. **并行执行**: 利用 GitHub Actions 的并行能力
3. **缓存依赖**: 缓存 Cargo 依赖，加速构建
4. **超时保护**: 为每个作业设置合理的超时时间
5. **失败处理**: 提供清晰的失败信息和调试提示

## 🎯 总结

hsipc 的综合测试系统提供了：

- **快速反馈**: TDD 循环 < 10秒
- **全面覆盖**: 从单元测试到 E2E 测试
- **智能选择**: 根据修改内容自动选择测试
- **性能验证**: 负载测试和基准测试
- **CI/CD 集成**: 完整的自动化测试管道
- **跨平台支持**: Linux、macOS、Windows 测试
- **开发友好**: 丰富的测试工具和辅助函数

通过这个测试系统，开发者可以：
- 快速验证代码更改
- 确保功能的正确性和性能
- 安全地重构和添加新功能
- 维护高质量的代码库

测试系统遵循"测试金字塔"原则，平衡了测试的全面性和执行效率，为 hsipc 项目的持续发展提供了坚实的质量保障。