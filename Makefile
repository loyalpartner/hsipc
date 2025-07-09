.PHONY: quick full check watch multiprocess demo integration benchmark bench-quick bench-core

# Quick verification (30 seconds) - Primary development command
quick:
	@echo "🚀 Quick verification..."
	@cargo check --all-targets || (echo "❌ Syntax check failed"; exit 1)
	@cargo run --example rpc_system_demo demo || (echo "❌ Core functionality failed"; exit 1)
	@echo "✅ Quick verification passed!"

# Full testing (5 minutes) - Pre-commit verification
full:
	@echo "🧪 Full testing..."
	@cargo test --all || (echo "❌ Tests failed"; exit 1)
	@cargo clippy --all-targets || (echo "❌ Code quality check failed"; exit 1)
	@cargo fmt --check || (echo "❌ Code format check failed"; exit 1)
	@echo "✅ Full testing passed!"

# Syntax check (2 seconds) - Fastest feedback
check:
	@echo "🔍 Syntax check..."
	@cargo check --all-targets

# Core RPC demo (30 seconds) - Example-driven testing
demo:
	@echo "🎬 Running RPC system demo..."
	@cargo run --example rpc_system_demo demo

# Integration test (focused testing)
integration:
	@echo "🔧 Running integration tests..."
	@cargo test --test integration

# Real-time monitoring
watch:
	@echo "👀 Starting real-time monitoring..."
	@cargo watch -x 'run --example rpc_system_demo demo'

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