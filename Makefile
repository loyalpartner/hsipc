.PHONY: quick full check watch multiprocess demo benchmark bench-quick bench-core tdd tdd-watch tdd-core smart-test tdd-red tdd-green tdd-refactor tdd-commit tdd-full status-check pre-commit-check

# Quick verification (30 seconds) - Primary development command
quick:
	@echo "🚀 Quick verification..."
	@echo "  → Checking syntax..."
	@cargo check --all-targets --quiet || (echo "❌ Syntax check failed"; exit 1)
	@echo "  → Running core functionality..."
	@cd examples/trait_based_service && cargo run demo > /dev/null 2>&1 || (echo "❌ Core functionality failed"; exit 1)
	@echo "✅ Quick verification passed!"

# Full testing (5 minutes) - Pre-commit verification
full:
	@echo "🧪 Full testing..."
	@echo "  → Running all tests..."
	@cargo test --all --quiet || (echo "❌ Tests failed"; exit 1)
	@echo "  → Code quality check..."
	@cargo clippy --all-targets --quiet || (echo "❌ Code quality check failed"; exit 1)
	@echo "  → Format check..."
	@cargo fmt --check || (echo "❌ Code format check failed"; exit 1)
	@echo "✅ Full testing passed!"

# Syntax check (2 seconds) - Fastest feedback
check:
	@echo "🔍 Syntax check..."
	@cargo check --all-targets --quiet && echo "✅ Syntax check passed!"

# Core RPC demo (30 seconds) - Example-driven testing
demo:
	@echo "🎬 Running RPC system demo..."
	@cd examples/trait_based_service && cargo run demo


# TDD development cycle (<10 seconds) - Core functionality only
tdd:
	@echo "🧪 TDD cycle..."
	@echo "  → Checking syntax..."
	@cargo check --all-targets --quiet || (echo "❌ Syntax check failed"; exit 1)
	@echo "  → Running core tests..."
	@cargo test --test rpc_tdd_test --quiet || (echo "❌ Core tests failed"; exit 1)
	@echo "✅ TDD cycle passed!"

# TDD core tests only (fastest feedback)
tdd-core:
	@echo "🎯 TDD core tests..."
	@cargo test --test rpc_tdd_test --quiet

# TDD with real-time monitoring
tdd-watch:
	@echo "👀 Starting TDD monitoring..."
	@cargo watch -x 'test --test rpc_tdd_test --quiet'

# Smart test selection based on changed files
smart-test:
	@echo "🤖 Running smart test selection..."
	@./scripts/smart_test.sh

# Real-time monitoring
watch:
	@echo "👀 Starting real-time monitoring..."
	@cd examples/trait_based_service && cargo watch -x 'run demo'

# 格式化代码
fmt:
	@echo "📝 格式化代码..."
	@cargo fmt

# 代码质量检查
clippy:
	@echo "🔧 代码质量检查..."
	@cargo clippy --all-targets --fix

# 多进程通信测试
multiprocess:
	@echo "🚀 多进程通信测试..."
	@./scripts/multiprocess_test.sh

# 性能基准测试
benchmark:
	@echo "🚀 运行性能基准测试..."
	@./scripts/benchmark.sh

# 快速性能测试
bench-quick:
	@echo "🧪 快速性能测试..."
	@cd hsipc && cargo bench --bench simple_benchmarks

# 核心性能测试
bench-core:
	@echo "🔧 核心性能测试..."
	@cd hsipc && cargo bench --bench simple_benchmarks

# 清理构建缓存
clean:
	@echo "🧹 清理构建缓存..."
	@cargo clean

# TDD红绿重构循环支持
# ===========================

# 红色阶段：编写失败测试
tdd-red:
	@echo "🔴 TDD Red Phase: Ready to write failing tests..."
	@cargo check --all-targets --quiet || (echo "❌ 语法错误，修复后继续"; exit 1)
	@echo "✅ 语法检查通过，可以编写失败测试了"
	@echo "💡 提示: 编写测试后运行 'make tdd-green'"

# 绿色阶段：最小实现通过测试
tdd-green:
	@echo "🟢 TDD Green Phase: Making tests pass..."
	@cargo test --quiet || (echo "❌ 测试仍然失败，继续实现代码"; exit 1)
	@echo "✅ 测试通过，进入绿色状态！"
	@echo "💡 提示: 现在可以运行 'make tdd-refactor'"

# 重构阶段：改进代码质量
tdd-refactor:
	@echo "♻️ TDD Refactor Phase: Improving code quality..."
	@cargo test --quiet || (echo "❌ 重构破坏了测试"; exit 1)
	@cargo clippy --all-targets --quiet || (echo "❌ 代码质量检查失败"; exit 1)
	@cargo fmt || true
	@echo "✅ 重构完成，代码质量提升"
	@echo "💡 提示: 运行 'make tdd-commit' 提交绿色状态"

# 准备提交绿色状态
tdd-commit:
	@echo "📝 TDD Commit: Preparing green state for commit..."
	@cargo test --quiet || (echo "❌ 测试失败，无法提交"; exit 1)
	@git add .
	@echo "✅ 绿色状态已暂存，准备提交"
	@echo "💡 提示: 使用 'git commit -m \"your message\"' 提交"

# 完整TDD循环（检查状态并建议下一步）
tdd-full:
	@echo "🔄 智能TDD循环..."
	@if ! cargo test --quiet >/dev/null 2>&1; then \
		echo "🔴 检测到失败测试，建议进入绿色阶段"; \
		echo "💡 运行: make tdd-green"; \
	else \
		echo "🟢 测试通过，建议进入重构阶段"; \
		echo "💡 运行: make tdd-refactor"; \
	fi

# 工作状态检查
status-check:
	@echo "🔍 检查当前工作状态..."
	@echo "📁 Git状态:"
	@if ! git diff --quiet; then \
		echo "  ⚠️ 有未提交的修改"; \
		git status -s; \
	else \
		echo "  ✅ 工作目录干净"; \
	fi
	@echo "🧪 测试状态:"
	@if ! cargo test --quiet >/dev/null 2>&1; then \
		echo "  🔴 当前处于红色状态（测试失败）"; \
		echo "  💡 建议: make tdd-green"; \
	else \
		echo "  🟢 当前处于绿色状态（测试通过）"; \
		echo "  💡 建议: make tdd-refactor 或 make tdd-commit"; \
	fi

# 提交前检查
pre-commit-check:
	@echo "🛡️ 提交前质量检查..."
	@cargo test --quiet || (echo "❌ 测试失败，禁止提交"; exit 1)
	@cargo clippy --all-targets --quiet -- -D warnings || (echo "❌ 代码质量不达标"; exit 1)
	@cargo fmt --check || (echo "❌ 代码格式不规范"; exit 1)
	@echo "✅ 质量检查通过，可以安全提交"
