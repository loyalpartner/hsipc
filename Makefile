.PHONY: quick full check watch multiprocess demo integration benchmark bench-quick bench-core tdd tdd-watch tdd-core smart-test

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

# Integration test (focused testing)
integration:
	@echo "🔧 Running integration tests..."
	@cargo test --test integration

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