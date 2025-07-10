// 最小化测试案例，验证消息路由问题
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::test]
async fn test_broadcast_channel() {
    // 创建一个 broadcast channel
    let (tx, _rx) = broadcast::channel::<String>(1024);
    
    // 创建两个接收者
    let mut rx1 = tx.subscribe();
    let mut rx2 = tx.subscribe();
    
    // 发送两个消息
    println!("发送第一个消息");
    tx.send("Message 1".to_string()).unwrap();
    
    println!("发送第二个消息");
    tx.send("Message 2".to_string()).unwrap();
    
    // 接收者1接收消息
    println!("接收者1开始接收");
    let msg1_1 = rx1.recv().await.unwrap();
    println!("接收者1收到: {}", msg1_1);
    
    let msg1_2 = rx1.recv().await.unwrap();
    println!("接收者1收到: {}", msg1_2);
    
    // 接收者2接收消息
    println!("接收者2开始接收");
    let msg2_1 = rx2.recv().await.unwrap();
    println!("接收者2收到: {}", msg2_1);
    
    let msg2_2 = rx2.recv().await.unwrap();
    println!("接收者2收到: {}", msg2_2);
}

#[tokio::main]
async fn main() {
    test_broadcast_channel().await;
}