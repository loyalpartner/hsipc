## 开发规范

- 文档放在 docs 目录
- 查看 @README.md 了解项目概述
- **统一工作流程** @docs/TDD-GIT-WORKFLOW.md - TDD + Git Flow 集成开发流程
- 测试流程 @docs/TESTING.md
- 架构文档 @docs/ARCHITECTURE.md
- API 设计 @docs/API.md
- 性能测试 @docs/PERFORMANCE.md

## TDD + Git 工作流程

当我们开始开发一个新功能时，请遵循 **TDD + Git 统一工作流程**：

### 开发循环
1. **Red Phase** (`make tdd-red`): 编写失败测试
2. **Green Phase** (`make tdd-green`): 最小实现通过测试
3. **Refactor Phase** (`make tdd-refactor`): 重构优化代码
4. **Git Commit** (`make tdd-commit`): 提交绿色状态

### 快速命令
- `make status-check` - 检查当前TDD状态
- `make tdd-full` - 智能TDD循环
- `make pre-commit-check` - 提交前质量检查

### 开发规范
- 先更新相关的文档，我确认过后再开始执行
- 严格遵循 TDD 红绿重构循环
- 只有绿色状态才能提交到Git
- Readme.md 使用中文，注释使用英文
- 错误需要用 thiserror ✅ (已完成)
- 统一使用 tracing 日志库，不使用 log 库（更适合分布式系统,  我们的库是 lib 库， 日志输出用 tracing
- 不要使用 new_with_*, 使用 Builder 模式

## 调试规范

- 使用 LLDB MCP 的调试解决问题

## 测试规范

- 注意测试请求相应和发布订阅模式的时候应该用不同的进程

