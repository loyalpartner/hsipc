#!/bin/bash

# 智能测试选择脚本
# 根据修改的文件类型选择合适的测试策略

set -e

# 检查是否有未提交的修改
if git diff --quiet && git diff --staged --quiet; then
    echo "📋 没有检测到修改，运行基本验证..."
    MODIFIED_FILES=$(git diff --name-only HEAD~1)
else
    echo "📋 检测到本地修改，分析修改内容..."
    MODIFIED_FILES=$(git diff --name-only HEAD)
fi

echo "📂 修改的文件: $MODIFIED_FILES"

# 检查是否修改了宏代码
if [[ $MODIFIED_FILES == *"hsipc-macros"* ]]; then
    echo "🔧 检测到宏代码修改，运行TDD核心测试..."
    make tdd-core
    echo "✅ 宏测试完成"
    
# 检查是否修改了RPC相关代码
elif [[ $MODIFIED_FILES == *"rpc"* ]]; then
    echo "🎯 检测到RPC相关修改，运行RPC测试..."
    make tdd
    cd examples/trait_based_service && cargo run demo
    echo "✅ RPC测试完成"
    
# 检查是否修改了服务相关代码
elif [[ $MODIFIED_FILES == *"service"* ]] || [[ $MODIFIED_FILES == *"hub"* ]]; then
    echo "🔧 检测到服务模块修改，运行服务测试..."
    cd examples/trait_based_service && cargo run demo
    cargo test --test integration
    echo "✅ 服务测试完成"
    
# 检查是否修改了事件相关代码
elif [[ $MODIFIED_FILES == *"event"* ]] || [[ $MODIFIED_FILES == *"subscription"* ]]; then
    echo "📡 检测到事件模块修改，运行事件测试..."
    cd examples/pubsub_events && cargo run publisher
    cargo test subscription
    echo "✅ 事件测试完成"
    
# 检查是否修改了传输层代码
elif [[ $MODIFIED_FILES == *"transport"* ]]; then
    echo "🚚 检测到传输层修改，运行传输测试..."
    cargo test transport
    echo "✅ 传输层测试完成"
    
# 检查是否修改了示例代码
elif [[ $MODIFIED_FILES == *"examples"* ]]; then
    echo "📚 检测到示例修改，运行示例验证..."
    if [[ $MODIFIED_FILES == *"rpc_system_demo"* ]]; then
        echo "🎬 检测到合并示例修改，运行完整演示验证..."
        cd examples/rpc_system_demo && cargo run demo
        echo "🧪 运行示例集成测试..."
        cd ../.. && cargo test --test example_integration_tests
    else
        # Fallback for any other example changes
        echo "🎬 运行默认演示..."
        make demo
    fi
    echo "✅ 示例验证完成"
    
# 检查是否修改了测试文件
elif [[ $MODIFIED_FILES == *"test"* ]]; then
    echo "🧪 检测到测试修改，运行相关测试..."
    if [[ $MODIFIED_FILES == *"rpc_tdd_test"* ]]; then
        make tdd-core
    else
        cargo test --test integration
    fi
    echo "✅ 测试完成"
    
# 检查是否修改了文档
elif [[ $MODIFIED_FILES == *"docs"* ]] || [[ $MODIFIED_FILES == *"README"* ]]; then
    echo "📝 检测到文档修改，运行快速验证..."
    cargo check --all-targets
    echo "✅ 文档验证完成"
    
# 检查是否修改了构建文件
elif [[ $MODIFIED_FILES == *"Cargo.toml"* ]] || [[ $MODIFIED_FILES == *"Makefile"* ]]; then
    echo "🔨 检测到构建文件修改，运行构建验证..."
    cargo check --all-targets
    make quick
    echo "✅ 构建验证完成"
    
# 默认情况
else
    echo "🔍 运行通用快速验证..."
    make quick
    echo "✅ 通用验证完成"
fi

echo "🎉 智能测试选择完成！"