use hsipc::{ProcessHub, Result, Subscriber, async_trait};
use serde::{Serialize, Deserialize};
use tokio::time::{timeout, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebugEvent {
    pub message: String,
    pub id: u64,
}

pub struct DebugSubscriber {
    pub received_events: Arc<Mutex<Vec<DebugEvent>>>,
}

#[async_trait]
impl Subscriber for DebugSubscriber {
    fn topic_pattern(&self) -> &str {
        "debug/*"
    }

    async fn handle(&mut self, topic: &str, payload: Vec<u8>) -> Result<()> {
        println\!("🔔 DebugSubscriber received message on topic: {}", topic);
        
        match bincode::deserialize::<DebugEvent>(&payload) {
            Ok(event) => {
                println\!("📦 Decoded event: {:?}", event);
                self.received_events.lock().await.push(event);
                println\!("✅ Event stored successfully");
            }
            Err(e) => {
                println\!("❌ Failed to decode event: {}", e);
                // 尝试作为字符串解码
                match bincode::deserialize::<String>(&payload) {
                    Ok(s) => println\!("📝 Raw string: {}", s),
                    Err(e2) => println\!("❌ Also failed as string: {}", e2),
                }
            }
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println\!("🧪 开始传输层和事件传递调试测试...");
    
    // 创建hub
    let hub = ProcessHub::new("debug_test").await?;
    println\!("✅ ProcessHub 创建成功");
    
    // 创建订阅者
    let received_events = Arc::new(Mutex::new(Vec::new()));
    let subscriber = DebugSubscriber {
        received_events: received_events.clone(),
    };
    
    // 订阅事件
    println\!("🔔 创建订阅者...");
    let subscription = hub.subscribe(subscriber).await?;
    println\!("✅ 订阅创建成功");
    
    // 等待一下确保订阅建立
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 发布事件
    println\!("📤 发布调试事件...");
    let debug_event = DebugEvent {
        message: "Debug test message".to_string(),
        id: 12345,
    };
    
    hub.publish("debug/test", debug_event.clone()).await?;
    println\!("✅ 事件发布成功");
    
    // 等待事件传递
    println\!("⏳ 等待事件传递...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // 检查接收到的事件
    let events = received_events.lock().await;
    println\!("📊 接收到 {} 个事件", events.len());
    
    if events.is_empty() {
        println\!("❌ 未接收到任何事件 - 事件传递链路可能有问题");
    } else {
        println\!("✅ 成功接收到事件:");
        for (i, event) in events.iter().enumerate() {
            println\!("  {}: {:?}", i + 1, event);
        }
    }
    
    drop(subscription);
    println\!("🏁 调试测试完成");
    
    Ok(())
}
EOF < /dev/null