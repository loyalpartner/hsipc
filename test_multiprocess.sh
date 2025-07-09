#!/bin/bash

echo "🧪 测试多进程通信改进..."

# 启动服务器
echo "🔧 启动服务器..."
cd examples/trait_based_service
cargo run server &
SERVER_PID=$!

# 等待服务器启动
sleep 2

# 启动客户端
echo "📞 启动客户端..."
timeout 30 cargo run client
CLIENT_STATUS=$?

# 清理
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

if [ $CLIENT_STATUS -eq 0 ]; then
    echo "✅ 多进程通信测试通过！"
    exit 0
else
    echo "❌ 多进程通信测试失败！"
    exit 1
fi