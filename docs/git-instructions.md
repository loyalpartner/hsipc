# Git 操作指南 - Claude AI 助手工作流

本文档为 hsipc 项目的 Git 操作指南，专门针对 Claude AI 助手的工作流程设计。

## AI 助手工作模式

作为 AI 助手，我遵循以下工作原则：
- **只有明确请求才提交**：除非用户明确要求提交，否则不会自动提交代码
- **TodoWrite 工具管理任务**：使用 TodoWrite 工具跟踪和管理所有任务进度
- **TDD 开发模式**：遵循测试驱动开发，所有开发工作都应测试优先
- **代码质量保证**：每次修改后自动运行格式化和质量检查

## AI 助手的 Git 工作流程

### 1. 任务管理流程

我的每个开发任务都遵循以下流程：

```bash
# 1. 创建任务清单
# 使用 TodoWrite 工具创建任务列表

# 2. 标记任务为进行中
# 将当前任务状态设为 "in_progress"

# 3. 执行开发任务
# 编写/修改代码、运行测试等

# 4. 代码质量检查（自动执行）
cargo fmt
cargo clippy --fix
cargo test

# 5. 标记任务完成
# 将任务状态设为 "completed"

# 6. 提交代码（仅当用户明确要求时）
git add .
git commit -m "feat: 添加新功能描述

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### 2. AI 助手的分支管理策略

#### 分支使用原则
- **通常直接在 master 分支工作**：对于小型修改和功能添加
- **大型功能使用分支**：仅在用户明确要求或复杂功能时创建分支
- **用户决定分支策略**：根据用户需求决定是否使用分支

#### 分支命名规范（当需要时）
```bash
# AI 助手创建的分支通常包含 "claude" 前缀
feature/claude-service-v2-migration
fix/claude-multi-process-communication
refactor/claude-error-handling-thiserror
docs/claude-update-readme-examples
```

#### 分支操作（当用户要求时）
```bash
# 1. 检查当前分支状态
git status
git branch

# 2. 创建新分支（如果需要）
git checkout -b feature/claude-new-feature

# 3. 在分支上工作
# 执行代码修改、测试等

# 4. 准备合并（用户确认后）
git checkout master
git merge feature/claude-new-feature
git branch -d feature/claude-new-feature
```

### 3. AI 助手的提交信息规范

#### 提交类型
- `feat`: 新功能
- `fix`: 修复问题
- `refactor`: 重构代码
- `docs`: 文档更新
- `test`: 测试相关
- `chore`: 构建/工具相关

#### AI 助手的提交信息格式
```
<类型>: <简短描述>

<详细描述>（可选）

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>
```

#### AI 助手提交示例
```bash
feat: implement trait-based service architecture

- 添加 #[service_trait] 和 #[service_impl] 宏
- 自动生成类型化客户端
- 支持多态性和组合模式
- 完善单进程模式的功能测试

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>

fix: resolve multi-process communication issues with IPMB selector

- 修复服务发现时序问题
- 优化 IPMB 传输层稳定性
- 更新相关测试用例

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>

docs: update README with trait-based service examples

- 添加 trait-based 服务使用示例
- 更新特性对比表格
- 完善快速开始指南

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>
```

## AI 助手的开发流程集成

### 1. TodoWrite 工具使用

AI 助手使用 TodoWrite 工具管理任务：

```json
// 创建任务示例
{
  "id": "1",
  "content": "实现 trait-based 服务架构",
  "status": "in_progress",
  "priority": "high"
}

// 任务状态转换
"pending" -> "in_progress" -> "completed"
```

### 2. 状态管理集成
项目支持开发状态的保存和恢复：

```bash
# 保存当前状态（当用户说 "保存当前的状态" 时）
# AI 助手会自动生成 state.md 文件

# 恢复上一次状态（当用户说 "恢复上一次的状态" 时）
# AI 助手会从 state.md 文件恢复状态
```

### 3. 自动化代码质量检查
AI 助手在每次代码修改后自动执行：

```bash
# 格式化代码
cargo fmt

# 代码质量检查和自动修复
cargo clippy --fix

# 运行测试套件
cargo test

# 运行特定示例测试
cargo run --example trait_based_service
```

### 4. 开发规范遵循
- 使用 `thiserror` 库进行错误处理
- 不允许使用 `println!` 宏
- 日志使用 `log` 库，级别设置为 `info` 或 `trace`
- 代码注释使用英文，文档使用中文

## 常用 Git 命令

### 查看状态和历史
```bash
# 查看当前状态
git status

# 查看提交历史
git log --oneline -10

# 查看具体文件的更改
git diff filename.rs

# 查看已暂存的更改
git diff --cached
```

### 暂存和提交
```bash
# 暂存所有更改
git add .

# 暂存特定文件
git add src/lib.rs

# 提交更改
git commit -m "feat: 添加新功能"

# 修改最后一次提交
git commit --amend
```

### 分支操作
```bash
# 查看所有分支
git branch -a

# 创建并切换到新分支
git checkout -b feature/new-feature

# 合并分支
git merge feature/completed-feature

# 删除分支
git branch -d feature/completed-feature
```

### 远程操作
```bash
# 添加远程仓库
git remote add origin <repository-url>

# 推送到远程
git push origin master

# 拉取远程更改
git pull origin master

# 查看远程信息
git remote -v
```

## AI 助手的具体工作流程

### 1. 新功能开发流程
```bash
# 1. 创建任务清单
# 使用 TodoWrite 工具记录任务

# 2. 分析需求
# 搜索相关代码，理解现有架构

# 3. 编写测试（TDD 原则）
# 编写失败的测试用例

# 4. 实现功能
# 编写最小化实现使测试通过

# 5. 重构和优化
# 改进代码质量，保持测试通过

# 6. 自动化检查
cargo fmt
cargo clippy --fix
cargo test

# 7. 更新任务状态
# 将 TodoWrite 任务标记为完成

# 8. 提交更改（仅当用户要求时）
git add .
git commit -m "feat: 实现新的服务功能

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### 2. 问题修复流程
```bash
# 1. 问题分析
# 理解问题描述，搜索相关代码

# 2. 创建重现测试
# 编写测试用例重现问题

# 3. 修复实现
# 修改代码使测试通过

# 4. 验证修复
cargo test
cargo run --example relevant_example

# 5. 提交修复（用户确认后）
git add .
git commit -m "fix: 修复多进程通信时序问题

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### 3. 文档更新流程
```bash
# 更新 README.md（使用中文）
git add README.md
git commit -m "docs: 更新 README 示例代码

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"

# 更新代码注释（使用英文）
git add src/lib.rs
git commit -m "docs: improve code comments for service trait

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"
```

## AI 助手的最佳实践

### 1. 任务管理原则
- **单一职责**：每个 TodoWrite 任务专注于一个具体目标
- **状态同步**：实时更新任务状态，确保用户了解进度
- **优先级管理**：根据任务重要性设置优先级
- **完成确认**：只有在完全完成后才标记为 "completed"

### 2. 代码质量保证
- **自动化检查**：每次修改后自动运行 cargo fmt 和 cargo clippy
- **测试优先**：遵循 TDD 原则，先写测试再实现
- **持续验证**：确保所有测试通过后再继续
- **代码审查**：在提交前进行自我审查

### 3. 提交策略
- **用户授权**：只有用户明确要求时才提交代码
- **原子提交**：每次提交包含一个完整的功能或修复
- **清晰信息**：提交信息准确描述更改内容
- **AI 标识**：所有提交都包含 AI 生成标识

### 4. 分支管理
- **谨慎使用**：仅在必要时创建分支
- **用户决定**：分支策略由用户需求决定
- **及时清理**：合并后及时删除功能分支

## 紧急情况处理

### 1. 回滚更改
```bash
# 回滚到上一次提交
git reset --hard HEAD~1

# 回滚到特定提交
git reset --hard <commit-hash>

# 撤销特定文件的更改
git checkout -- filename.rs
```

### 2. 恢复误删文件
```bash
# 查看删除的文件
git log --diff-filter=D --summary

# 恢复误删的文件
git checkout <commit-hash>~1 -- <file-path>
```

### 3. 解决冲突
```bash
# 查看冲突文件
git status

# 编辑冲突文件，解决冲突后
git add <resolved-file>
git commit -m "resolve: 解决合并冲突"
```

## 工具推荐

### 1. Git GUI 工具
- GitKraken
- SourceTree
- VS Code Git 扩展

### 2. 命令行增强
- `git log --oneline --graph --all` - 图形化日志
- `git status -s` - 简洁状态显示
- `git diff --word-diff` - 词级别差异显示

### 3. 别名配置
```bash
# 添加有用的 Git 别名
git config --global alias.st status
git config --global alias.co checkout
git config --global alias.br branch
git config --global alias.ci commit
git config --global alias.lg "log --oneline --graph --all"
```

## AI 助手工作流程总结

### 核心原则
1. **用户主导**：所有重要操作都需要用户明确授权
2. **任务驱动**：使用 TodoWrite 工具管理所有任务
3. **质量保证**：自动化代码格式化和质量检查
4. **透明度**：所有操作都有清晰的标识和记录

### 工作流程要点
```
用户请求 → 创建任务 → 分析需求 → 编写测试 → 实现功能 → 
质量检查 → 完成任务 → 等待用户确认 → 提交代码
```

### 关键约束
- **不主动提交**：只有明确请求才提交代码
- **任务跟踪**：所有工作都通过 TodoWrite 管理
- **代码标准**：严格遵循项目的开发规范
- **AI 标识**：所有提交都包含 AI 生成标识

### 成功指标
- 代码质量和一致性
- 清晰的任务进度跟踪
- 明确的提交历史记录
- 高效的人机协作

记住：作为 AI 助手，我始终以用户需求为导向，确保每个操作都是有价值和可追溯的。