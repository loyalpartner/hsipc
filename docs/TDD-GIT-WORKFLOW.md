# TDD + Git Flow 统一开发工作流

## 🎯 核心理念

结合测试驱动开发(TDD)和Git工作流，创建高效、安全、可追溯的开发流程。

**核心原则:**
- **Red-Green-Refactor + Git**: TDD循环与Git提交完美结合
- **每个测试一个提交**: 保证提交历史清晰且可回滚
- **失败测试不提交**: 只有绿色(通过)状态才能提交
- **分支保护**: 确保master分支始终可用

## 🏗️ 统一工作流架构

### 开发循环模式

```
 ┌─ 需求分析 ──┐
 │             │
 ▼             │
Red Phase    ─────┐
 │ 编写失败测试   │  │
 │               │  │
 ▼               │  │  TDD
Green Phase   ────┘  │ 循环
 │ 实现最小代码      │
 │                  │
 ▼                  │
Refactor Phase ──────┘
 │ 重构和优化
 │
 ▼
Git Commit ──────┐
 │ 提交绿色状态   │
 │               │  Git
 ▼               │ 工作流
Branch/PR ────────┘
 │ 代码评审
 │
 ▼
Master Merge
```

### 分层测试策略

| 阶段 | TDD Phase | Git Action | 时间 | Claude命令 | Make命令 |
|------|-----------|------------|------|----------|----------|
| 智能循环 | 🤖 自动检测 | Working Dir | 5秒 | `/tdd` | `make tdd-full` |
| 语法检查 | - | Working Dir | 2秒 | `/check` | `cargo check` |
| Red Phase | 🔴 编写失败测试 | Working Dir | 5秒 | `/tdd-red` | `make tdd-red` |
| Green Phase | 🟢 最小实现 | Working Dir | 10秒 | `/tdd-green` | `make tdd-green` |
| Refactor | ♻️ 重构优化 | Working Dir | 30秒 | `/tdd-refactor` | `make tdd-refactor` |
| Commit | ✅ 提交绿色状态 | Staging/Commit | 5秒 | `/commit` | `make tdd-commit` |
| 状态检查 | 📊 分析状态 | Working Dir | 3秒 | `/tdd-status` | `make status-check` |

## 🚀 实际工作流程

### 0. Claude斜杠命令工作流（推荐）

#### 智能TDD开发流程

```bash
# 开始开发新功能
1. 检查当前状态
   /tdd-status

2. 智能TDD循环（自动检测状态并执行）
   /tdd
   
   # Claude会分析当前状态：
   # - 🔴 Red: 引导编写失败测试
   # - 🟢 Green: 引导实现最小代码
   # - ♻️ Refactor: 自动运行代码质量检查
   # - 📝 Commit: 建议提交绿色状态

3. 重复TDD循环直到功能完成
   /tdd -> /tdd -> /tdd ...

4. 代码质量检查
   /check

5. 创建提交
   /commit

6. 继续下一个功能或创建PR
```

#### Claude斜杠命令优势

- **智能判断**: 自动检测当前TDD阶段，无需人工判断
- **错误恢复**: 自动处理常见开发问题
- **进度跟踪**: 自动更新TodoWrite任务状态
- **质量保证**: 自动运行代码质量检查
- **一致性**: 确保遵循TDD最佳实践

### 1. 功能开发的完整流程

#### 步骤1: 任务初始化
```bash
# 1. 创建功能分支（如需要）
git checkout -b feature/subscription-data-flow

# 2. 创建任务清单
# 使用 TodoWrite 工具创建任务
```

#### 步骤2: TDD 红绿重构循环

**推荐方式 - Claude斜杠命令：**
```bash
# 智能TDD循环（推荐）
/tdd
# Claude会自动检测当前状态并执行合适的TDD阶段

# 或者使用专用命令：
/tdd-red      # 强制进入红色阶段：编写失败测试
/tdd-green    # 强制进入绿色阶段：最小实现
/tdd-refactor # 强制进入重构阶段：优化代码
/tdd-status   # 检查当前TDD状态和进度
```

**传统方式 - Make命令：**
```bash
# Red Phase: 编写失败测试
make tdd-red
# 或手动：
# - 编写测试用例
# - 确保测试失败
# - cargo test subscription_data_flow -- --nocapture

# Green Phase: 最小实现
make tdd-green  
# 或手动：
# - 编写最小代码使测试通过
# - cargo test subscription_data_flow
# - 确保测试通过

# Refactor Phase: 重构优化
make tdd-refactor
# 或手动：
# - 改进代码质量
# - 保持测试通过
# - cargo test && cargo clippy && cargo fmt
```

#### 步骤3: Git 提交绿色状态

**推荐方式 - Claude斜杠命令：**
```bash
# 智能代码质量检查
/check

# 创建Git提交（自动生成提交信息）
/commit
```

**传统方式 - Make命令：**
```bash
# 提交当前绿色状态
make tdd-commit
# 或手动：
git add .
git commit -m "feat: implement subscription data flow basic structure

- Add failing test for subscription message routing  
- Implement minimal ProcessHub subscription handling
- Ensure all tests pass in green state

🔴➡️🟢 TDD Cycle Complete

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

#### 步骤4: 继续迭代或集成
```bash
# 继续下一个TDD循环，或者进行集成测试
make integration

# 推送分支（如适用）
git push -u origin feature/subscription-data-flow
```

### 2. 问题修复的TDD流程

```bash
# 1. 创建重现测试（Red Phase）
make tdd-red
# - 编写重现bug的测试
# - 确保测试失败，证明bug存在

# 2. 修复问题（Green Phase）  
make tdd-green
# - 修改代码使测试通过
# - 最小化修改范围

# 3. 重构优化（Refactor Phase）
make tdd-refactor
# - 改进修复的代码质量
# - 确保没有引入新问题

# 4. 提交修复
make tdd-commit
# 提交信息示例：
# "fix: resolve subscription memory leak
# 
# 🔴 Add test reproducing memory leak in subscription cleanup
# 🟢 Implement proper subscription disposal  
# ♻️ Refactor subscription lifecycle management
# 
# 🤖 Generated with [Claude Code](https://claude.ai/code)"
```

## 📋 Makefile 工具支持

### TDD循环支持

```makefile
# TDD红绿重构循环
.PHONY: tdd-red tdd-green tdd-refactor tdd-commit tdd-full

# 红色阶段：编写失败测试
tdd-red:
	@echo "🔴 TDD Red Phase: Writing failing tests..."
	@cargo check || (echo "❌ 语法错误，修复后继续"; exit 1)
	@echo "✅ 准备好编写失败测试"

# 绿色阶段：最小实现通过测试  
tdd-green:
	@echo "🟢 TDD Green Phase: Making tests pass..."
	@cargo test || (echo "❌ 测试仍然失败"; exit 1)
	@echo "✅ 测试通过，进入绿色状态"

# 重构阶段：改进代码质量
tdd-refactor:
	@echo "♻️ TDD Refactor Phase: Improving code quality..."
	@cargo test || (echo "❌ 重构破坏了测试"; exit 1)
	@cargo clippy --all-targets || (echo "❌ 代码质量检查失败"; exit 1)
	@cargo fmt || (echo "❌ 代码格式化失败"; exit 1)
	@echo "✅ 重构完成，代码质量提升"

# 提交绿色状态
tdd-commit:
	@echo "📝 TDD Commit: Committing green state..."
	@git add .
	@echo "✅ 准备提交绿色状态到Git"

# 完整TDD循环
tdd-full: tdd-red tdd-green tdd-refactor tdd-commit
	@echo "🔄 完整TDD循环完成"

# 智能TDD：根据当前状态自动选择
tdd:
	@echo "🧠 智能TDD循环..."
	@if ! cargo test >/dev/null 2>&1; then \
		echo "🔴 检测到失败测试，进入绿色阶段..."; \
		make tdd-green; \
	else \
		echo "🟢 测试通过，进入重构阶段..."; \
		make tdd-refactor; \
	fi

# TDD监控模式
tdd-watch:
	@echo "👀 TDD监控模式启动..."
	@cargo watch -x 'test' -x 'clippy --all-targets' -x 'fmt'
```

### Git工作流支持

```makefile
# Git工作流集成
.PHONY: feature-start feature-finish hotfix-start hotfix-finish

# 开始新功能
feature-start:
	@read -p "功能名称: " name; \
	git checkout -b feature/$$name; \
	echo "✅ 创建功能分支: feature/$$name"

# 完成功能开发
feature-finish:
	@echo "🔗 完成功能开发..."
	@make integration || (echo "❌ 集成测试失败"; exit 1)
	@git checkout master
	@git merge --no-ff $(shell git branch --show-current)
	@git branch -d $(shell git branch --show-current)
	@echo "✅ 功能分支已合并到master"

# 开始热修复
hotfix-start:
	@read -p "热修复名称: " name; \
	git checkout -b hotfix/$$name master; \
	echo "✅ 创建热修复分支: hotfix/$$name"

# 完成热修复
hotfix-finish:
	@echo "🚨 完成热修复..."
	@make tdd-full || (echo "❌ TDD循环失败"; exit 1)
	@git checkout master
	@git merge --no-ff $(shell git branch --show-current)
	@echo "✅ 热修复已合并到master"
```

## 🎯 智能工作流

### 根据文件修改自动选择测试

```bash
# 智能测试选择
smart-test:
	@echo "🧠 智能测试选择..."
	@if git diff --name-only | grep -q "src/.*\.rs"; then \
		echo "🔍 检测到源码修改，运行核心测试..."; \
		make tdd; \
	elif git diff --name-only | grep -q "tests/.*\.rs"; then \
		echo "🧪 检测到测试修改，运行全部测试..."; \
		cargo test; \
	elif git diff --name-only | grep -q "examples/.*\.rs"; then \
		echo "📋 检测到示例修改，运行示例验证..."; \
		make examples-test; \
	else \
		echo "📄 检测到其他修改，运行快速验证..."; \
		make quick; \
	fi
```

### 状态检查和自动恢复

```bash
# 工作状态检查
status-check:
	@echo "🔍 检查当前工作状态..."
	@if ! git diff --quiet; then \
		echo "⚠️ 有未提交的修改"; \
		git status -s; \
	fi
	@if ! cargo test >/dev/null 2>&1; then \
		echo "🔴 当前处于红色状态（测试失败）"; \
		echo "💡 建议: make tdd-green"; \
	else \
		echo "🟢 当前处于绿色状态（测试通过）"; \
		echo "💡 建议: make tdd-refactor 或 make tdd-commit"; \
	fi
```

## 📊 分支策略与TDD集成

### 功能分支 + TDD

```
master ─┬─ feature/subscription-flow
        │   │
        │   ├─ 🔴 Add failing subscription test
        │   ├─ 🟢 Implement basic subscription routing  
        │   ├─ ♻️ Refactor subscription lifecycle
        │   ├─ 🔴 Add subscription cleanup test
        │   ├─ 🟢 Implement subscription cleanup
        │   └─ 📋 Integration tests pass
        │
        └─ 🔗 Merge feature (绿色状态保证)
```

### 热修复 + 快速TDD

```
master ─┬─ hotfix/memory-leak
        │   │  
        │   ├─ 🔴 Add test reproducing memory leak
        │   ├─ 🟢 Fix memory leak with minimal change
        │   └─ ♻️ Quick refactor and cleanup
        │
        └─ 🚨 Emergency merge (关键修复)
```

## 🛡️ 质量保证

### 提交前检查清单

```bash
# 自动化提交前检查
pre-commit-check:
	@echo "🛡️ 提交前质量检查..."
	@cargo test || (echo "❌ 测试失败，禁止提交"; exit 1)
	@cargo clippy --all-targets -- -D warnings || (echo "❌ 代码质量不达标"; exit 1)  
	@cargo fmt --check || (echo "❌ 代码格式不规范"; exit 1)
	@echo "✅ 质量检查通过，可以安全提交"
```

### 分支保护规则

- **master分支**: 只接受绿色状态的合并
- **feature分支**: 允许红色状态，但禁止推送到远程
- **hotfix分支**: 快速通道，但必须通过基础TDD循环

## 📈 工作流指标

### TDD效率指标

- **红绿循环时间**: 目标 < 5分钟
- **重构安全性**: 重构后测试通过率 100%
- **提交频率**: 每个TDD循环产生1个有意义的提交
- **分支生命周期**: feature分支 < 1天，hotfix分支 < 1小时

### Git工作流指标

- **master稳定性**: master分支测试通过率 100%
- **回滚能力**: 任何提交都可以安全回滚
- **追溯性**: 每个变更都有对应的测试用例
- **协作效率**: 分支冲突率 < 5%

## 🚨 应急处理

### 红色状态应急处理

```bash
# 如果陷入长时间红色状态
emergency-reset:
	@echo "🚨 应急重置到上次绿色状态..."
	@git status
	@read -p "确认重置到上次提交? (y/N): " confirm; \
	if [ "$$confirm" = "y" ]; then \
		git reset --hard HEAD; \
		echo "✅ 已重置到上次绿色状态"; \
	fi
```

### 紧急热修复流程

```bash
# 紧急修复流程（跳过部分TDD步骤）
emergency-fix:
	@echo "🚨 紧急修复模式..."
	@git checkout -b hotfix/emergency-fix
	@echo "1. 🔴 编写重现问题的测试"
	@echo "2. 🟢 实现最小修复"  
	@echo "3. ⚡ 跳过重构，直接提交"
	@echo "4. 🚀 立即合并到master"
```

## 📚 最佳实践总结

### 开发者日常习惯

1. **开始工作**: `make status-check` - 了解当前状态
2. **日常开发**: `make tdd` - 智能TDD循环
3. **快速验证**: `make smart-test` - 根据修改选择测试
4. **提交代码**: `make tdd-commit` - 确保绿色状态提交
5. **结束工作**: `make pre-commit-check` - 确保工作目录干净

### 团队协作规范

1. **永远不推送红色状态** - 远程分支必须是绿色的
2. **小步快跑** - 每个TDD循环产生一个提交
3. **有意义的提交信息** - 描述TDD阶段和实现内容
4. **定期集成** - 避免长时间的feature分支

### AI助手特殊约定

1. **TodoWrite集成** - 每个TDD循环对应一个或多个todo任务
2. **自动化执行** - AI助手自动运行TDD循环中的验证步骤  
3. **用户确认提交** - 只有用户明确要求才执行git操作
4. **清晰的状态报告** - 每个阶段都报告TDD状态和建议下一步

---

**工作流口诀**: Red → Green → Refactor → Commit → Repeat

这个统一工作流确保了代码质量、开发效率和团队协作的完美结合。