#!/bin/bash

MODIFIED_FILES=$(git diff --name-only HEAD~1)

if [[ $MODIFIED_FILES == *"service"* ]]; then
    echo "🔧 检测到服务模块修改，运行服务测试..."
    cd examples/trait_based_service && cargo run demo
    cargo test --lib service
elif [[ $MODIFIED_FILES == *"event"* ]]; then
    echo "📡 检测到事件模块修改，运行事件测试..."
    cargo run --example pubsub_events publisher
    cargo test --lib event
elif [[ $MODIFIED_FILES == *"transport"* ]]; then
    echo "🚚 检测到传输层修改，运行传输测试..."
    cargo test --lib transport
else
    echo "🔍 运行通用快速验证..."
    make quick
fi