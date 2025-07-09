.PHONY: quick full check watch multiprocess

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

# 清理构建缓存
clean:
	@echo "🧹 清理构建缓存..."
	@cargo clean