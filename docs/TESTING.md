# hsipc 快速迭代测试流程

## 🎯 设计原则

本测试流程专为 AI 辅助的快速迭代开发设计，遵循以下原则：

- **TDD 优先** - 测试驱动开发，确保每个功能都有对应测试
- **测试集中** - 核心测试集中在少数关键场景，不要分散到多个文件
- **快速反馈** - 30秒内完成核心验证
- **示例驱动** - 运行示例即测试功能
- **智能选择** - 根据修改内容选择测试
- **渐进验证** - 从语法检查到完整测试的分层验证

## 🏗️ 测试架构

### 极简测试金字塔

```
     /\
    /E2E\     <- 1-2个关键场景（5分钟）
   /____\
  /      \
 /Examples\   <- 示例即测试（30秒）
/________\
  /      \
 /Syntax  \   <- 语法检查（2秒）
/__Check__\
```

### 测试分层策略

| 层级 | 目的 | 执行时间 | 命令 |
|------|------|----------|------|
| 语法检查 | 快速发现编译错误 | 2秒 | `cargo check` |
| TDD循环 | 核心功能快速验证 | 10秒 | `make tdd` |
| 示例验证 | 验证核心功能正常 | 30秒 | `cd examples/X && cargo run demo` |
| 核心测试 | 验证关键场景 | 2分钟 | `cargo test integration` |
| 完整测试 | 全面质量保证 | 5分钟 | `cargo test --all` |

## 🚀 快速开发流程

### 微迭代循环（推荐）

```
编写代码 → cargo check → make tdd → 运行相关示例 → 继续开发
     ↑                                                    ↓
   快速修复 ←―――――――――――――― 如果有问题 ←――――――――――――――――――
```

### 日常开发命令

```bash
# 1. 最快语法检查（2秒）
cargo check --all-targets

# 2. TDD开发循环（10秒）
make tdd           # 语法检查 + 核心测试
make tdd-core      # 只运行核心测试
make tdd-watch     # 实时监控TDD测试

# 3. 功能验证（30秒）
cd examples/trait_based_service && cargo run demo    # 验证 trait-based 服务
cd examples/request_response && cargo run client     # 验证 RPC 功能
cd examples/pubsub_events && cargo run publisher     # 验证事件系统

# 4. 快速验证（1分钟）
make quick

# 5. 完整验证（5分钟）
make full

# 6. 智能测试选择（推荐）
make smart-test    # 根据修改内容自动选择测试

# 7. 多进程通信测试
make multiprocess
```

## 📋 快速验证工具

### Makefile 配置

```makefile
.PHONY: quick full check watch multiprocess tdd tdd-core tdd-watch

# 快速验证（30秒）
quick:
	@echo "🚀 快速验证..."
	@cargo check --all-targets || (echo "❌ 语法检查失败"; exit 1)
	@cd examples/trait_based_service && cargo run demo || (echo "❌ 核心功能失败"; exit 1)
	@echo "✅ 快速验证通过！"

# 完整测试（5分钟）
full:
	@echo "🧪 完整测试..."
	@cargo test --all || (echo "❌ 测试失败"; exit 1)
	@cargo clippy --all-targets || (echo "❌ 代码质量检查失败"; exit 1)
	@cargo fmt --check || (echo "❌ 代码格式检查失败"; exit 1)
	@echo "✅ 完整测试通过！"

# 语法检查（2秒）
check:
	@echo "🔍 语法检查..."
	@cargo check --all-targets

# 实时监控
watch:
	@echo "👀 开始实时监控..."
	@cd examples/trait_based_service && cargo watch -x 'run demo'

# 多进程通信测试
multiprocess:
	@echo "🚀 多进程通信测试..."
	@./scripts/multiprocess_test.sh

# TDD开发循环（10秒）
tdd:
	@echo "🧪 TDD cycle..."
	@cargo check --all-targets || (echo "❌ Syntax check failed"; exit 1)
	@cargo test --test rpc_tdd_test --quiet || (echo "❌ Core tests failed"; exit 1)
	@echo "✅ TDD cycle passed!"

# TDD核心测试
tdd-core:
	@echo "🎯 TDD core tests..."
	@cargo test --test rpc_tdd_test --quiet

# TDD实时监控
tdd-watch:
	@echo "👀 Starting TDD monitoring..."
	@cargo watch -x 'test --test rpc_tdd_test --quiet'

# 智能测试选择
smart-test:
	@echo "🤖 Running smart test selection..."
	@./scripts/smart_test.sh
```

## 🎯 智能测试选择

### 自动测试选择策略

智能测试选择系统根据修改的文件自动选择最合适的测试策略，提高开发效率：

| 修改类型 | 检测条件 | 执行策略 |
|---------|---------|---------|
| 宏代码 | `hsipc-macros/` | `make tdd-core` |
| RPC系统 | `rpc` 关键词 | `make tdd` + trait示例 |
| 服务模块 | `service`, `hub` | 服务示例 + 集成测试 |
| 事件系统 | `event`, `subscription` | 事件示例 + 订阅测试 |
| 传输层 | `transport` | 传输层单元测试 |
| 示例代码 | `examples/` | 对应示例验证 |
| 测试文件 | `test` | 相关测试套件 |
| 文档 | `docs/`, `README` | 语法检查 |
| 构建文件 | `Cargo.toml`, `Makefile` | 构建验证 |

### 使用方法

```bash
# 推荐：智能测试选择（根据修改自动选择）
make smart-test

# 或者直接运行脚本
./scripts/smart_test.sh
```

### 测试脚本组织

```
scripts/
├── quick_test.sh        # 快速测试脚本
├── smart_test.sh        # 智能测试选择（已优化）
└── multiprocess_test.sh # 多进程通信测试
```

### 智能选择示例

```bash
# 修改宏代码时
📂 修改的文件: hsipc-macros/src/rpc.rs
🔧 检测到宏代码修改，运行TDD核心测试...

# 修改RPC相关代码时  
📂 修改的文件: hsipc/src/hub.rs
🎯 检测到RPC相关修改，运行RPC测试...

# 修改示例代码时
📂 修改的文件: examples/trait_based_service/src/main.rs
📚 检测到示例修改，运行示例验证...
```

## 🔧 开发环境配置

### VS Code 任务配置

创建 `.vscode/tasks.json`：

```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "quick-test",
            "type": "shell",
            "command": "make",
            "args": ["quick"],
            "group": {"kind": "test", "isDefault": true}
        },
        {
            "label": "full-test", 
            "type": "shell",
            "command": "make",
            "args": ["full"],
            "group": "test"
        }
    ]
}
```

## 📊 实时监控

### 使用 cargo-watch

```bash
# 安装 cargo-watch
cargo install cargo-watch

# 实时语法检查
cargo watch -x check

# 实时示例验证
cd examples/trait_based_service && cargo watch -x 'run demo'
```

## 🎪 核心测试场景

### 保留的关键测试

```rust
// src/tests/integration.rs
#[tokio::test]
async fn smoke_test_trait_based_service() {
    // 验证 trait-based 服务基本功能
    let env = TestEnvironment::new("smoke_test").await.unwrap();
    
    let calculator = BasicCalculator;
    env.hub.register_service(BasicCalculatorService::new(calculator)).await.unwrap();
    
    let client = CalculatorClient::new(env.hub.clone());
    let result = env.with_timeout(client.add((10, 5))).await.unwrap();
    
    assert_eq!(result, 15);
}
```

## 🎯 使用指南

### 日常开发流程

1. **开始开发**
   ```bash
   # 打开实时监控
   make watch
   ```

2. **编写代码**
   ```bash
   # 在另一个终端快速检查
   cargo check
   ```

3. **功能验证**
   ```bash
   # 验证当前功能
   cd examples/trait_based_service && cargo run demo
   ```

4. **阶段性验证**
   ```bash
   # 完整快速验证
   make quick
   ```

5. **提交前检查**
   ```bash
   # 可选：完整测试
   make full
   ```

### 性能指标

| 操作 | 目标时间 | 命令 |
|------|----------|------|
| 语法检查 | 2秒 | `cargo check` |
| TDD循环 | 10秒 | `make tdd` |
| 示例验证 | 30秒 | `cd examples/X && cargo run demo` |
| 快速验证 | 1分钟 | `make quick` |
| 完整测试 | 5分钟 | `make full` |

## 📝 总结

这个快速迭代测试流程设计为：

- **快速反馈** - 30秒内完成核心验证
- **智能选择** - 根据修改内容选择测试
- **渐进验证** - 从简单到复杂的分层测试
- **开发友好** - 最小化测试负担，最大化开发效率

通过这个流程，可以在保证代码质量的同时，实现快速迭代开发的目标。
