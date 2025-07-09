#!/bin/bash
set -e

echo "🔍 语法检查..."
cargo check --all-targets

echo "📦 示例验证..."
echo "  - 测试 trait-based 服务..."
cd examples/trait_based_service && timeout 30 cargo run demo

echo "🧪 核心测试..."
cargo test --lib --no-fail-fast

echo "✅ 快速验证通过！"