#!/bin/bash

echo "🧪 测试多进程通信改进..."

# Check if we're in the correct directory
if [ ! -d "examples/trait_based_service" ]; then
    echo "❌ 错误：请在项目根目录运行此脚本"
    exit 1
fi

# Function to cleanup processes
cleanup() {
    echo "🧹 清理进程..."
    if [ ! -z "$SERVER_PID" ] && kill -0 $SERVER_PID 2>/dev/null; then
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi
    if [ ! -z "$CLIENT_PID" ] && kill -0 $CLIENT_PID 2>/dev/null; then
        kill $CLIENT_PID 2>/dev/null || true
        wait $CLIENT_PID 2>/dev/null || true
    fi
}

# Set up cleanup trap
trap cleanup EXIT

# Build the project first
echo "🔨 构建项目..."
cd examples/trait_based_service
if ! cargo build --quiet; then
    echo "❌ 构建失败！"
    exit 1
fi

# Start server in background
echo "🔧 启动服务器..."
cargo run server > /tmp/server.log 2>&1 &
SERVER_PID=$!

# Wait for server to start
echo "⏱️ 等待服务器启动..."
sleep 3

# Check if server is still running
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "❌ 服务器启动失败！"
    cat /tmp/server.log
    exit 1
fi

# Run client and capture output
echo "📞 启动客户端..."
if cargo run client > /tmp/client.log 2>&1; then
    # Check if client completed successfully by looking for success indicators
    if grep -q "Client test completed!" /tmp/client.log && grep -q "✅.*=" /tmp/client.log; then
        echo "✅ 多进程通信测试通过！"
        echo "📋 客户端输出："
        grep -E "✅.*=" /tmp/client.log
        exit 0
    else
        echo "❌ 客户端未完成所有测试！"
        echo "📋 客户端日志："
        tail -n 10 /tmp/client.log
        exit 1
    fi
else
    echo "❌ 多进程通信测试失败！"
    echo "📋 服务器日志："
    tail -n 10 /tmp/server.log
    echo "📋 客户端日志："
    tail -n 10 /tmp/client.log
    exit 1
fi