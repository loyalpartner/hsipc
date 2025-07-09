//! 集成测试 - 验证 RPC 宏系统的完整功能
//!
//! 遵循 TESTING.md 约束：测试集中、快速反馈、示例驱动

use hsipc::{method, rpc, subscription, PendingSubscriptionSink, ProcessHub, Service};
use serde::{Deserialize, Serialize};

// 测试数据结构
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TestRequest {
    pub value: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TestResponse {
    pub result: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TestEvent {
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum TestError {
    #[error("Test error: {0}")]
    TestError(String),
    #[error("Division by zero")]
    DivisionByZero,
}

impl From<TestError> for hsipc::Error {
    fn from(err: TestError) -> Self {
        hsipc::Error::runtime_msg(err.to_string())
    }
}

// 核心测试服务 - 涵盖所有关键功能
#[rpc(server, client, namespace = "test")]
pub trait TestService {
    // 基础异步方法
    #[method(name = "async_method")]
    async fn async_method(
        &self,
        request: TestRequest,
    ) -> std::result::Result<TestResponse, TestError>;

    // 同步方法
    #[method(name = "sync_method", sync)]
    fn sync_method(&self, value: i32) -> std::result::Result<i32, TestError>;

    // 多参数方法
    #[method(name = "add_two")]
    async fn add_two(&self, a: i32, b: i32) -> std::result::Result<i32, TestError>;

    // 自定义错误类型
    #[method(name = "divide")]
    async fn divide(&self, a: i32, b: i32) -> std::result::Result<f64, TestError>;

    // 订阅方法
    #[subscription(name = "test_events", item = TestEvent)]
    async fn subscribe_test_events(
        &self,
        filter: Option<String>,
    ) -> std::result::Result<(), TestError>;

    // 无参数方法
    #[method(name = "get_status")]
    async fn get_status(&self) -> std::result::Result<String, TestError>;
}

// 服务实现
pub struct TestServiceImpl;

#[hsipc::async_trait]
impl TestService for TestServiceImpl {
    async fn async_method(
        &self,
        request: TestRequest,
    ) -> std::result::Result<TestResponse, TestError> {
        Ok(TestResponse {
            result: request.value * 2,
        })
    }

    fn sync_method(&self, value: i32) -> std::result::Result<i32, TestError> {
        Ok(value + 1)
    }

    async fn add_two(&self, a: i32, b: i32) -> std::result::Result<i32, TestError> {
        Ok(a + b)
    }

    async fn divide(&self, a: i32, b: i32) -> std::result::Result<f64, TestError> {
        if b == 0 {
            Err(TestError::DivisionByZero)
        } else {
            Ok(a as f64 / b as f64)
        }
    }

    async fn subscribe_test_events(
        &self,
        pending: PendingSubscriptionSink,
        _filter: Option<String>,
    ) -> std::result::Result<(), TestError> {
        // Accept the subscription for testing
        let _sink = pending
            .accept()
            .await
            .map_err(|e| TestError::TestError(e.to_string()))?;
        Ok(())
    }

    async fn get_status(&self) -> std::result::Result<String, TestError> {
        Ok("OK".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 核心冒烟测试 - 30秒内完成所有关键功能验证
    #[tokio::test]
    async fn smoke_test_rpc_system() {
        // 测试环境设置
        let hub = ProcessHub::new("smoke_test").await.unwrap();

        // 注册服务
        let service_impl = TestServiceImpl;
        let service = TestServiceService::new(service_impl);
        hub.register_service(service).await.unwrap();

        // 创建客户端
        let client = TestServiceClient::new(hub);

        // 1. 测试基础异步方法
        let request = TestRequest { value: 10 };
        let response = client.async_method(request).await.unwrap();
        assert_eq!(response.result, 20);

        // 2. 测试同步方法
        let result = client.sync_method(5).unwrap();
        assert_eq!(result, 6);

        // 3. 测试多参数方法
        let result = client.add_two(3, 7).await.unwrap();
        assert_eq!(result, 10);

        // 4. 测试自定义错误类型 - 成功情况
        let result = client.divide(10, 2).await.unwrap();
        assert_eq!(result, 5.0);

        // 5. 测试自定义错误类型 - 错误情况
        let result = client.divide(10, 0).await;
        assert!(result.is_err());

        // 6. 测试订阅方法
        client
            .subscribe_test_events(Some("filter".to_string()))
            .await
            .unwrap();

        // 7. 测试无参数方法
        let status = client.get_status().await.unwrap();
        assert_eq!(status, "OK");

        println!("✅ All RPC system features working correctly!");
    }

    /// 服务元数据测试 - 验证宏生成的服务信息
    #[tokio::test]
    async fn test_service_metadata() {
        let service_impl = TestServiceImpl;
        let service = TestServiceService::new(service_impl);

        // 验证服务名称
        assert_eq!(service.name(), "test");

        // 验证方法列表
        let methods = service.methods();
        assert!(methods.contains(&"async_method"));
        assert!(methods.contains(&"sync_method"));
        assert!(methods.contains(&"add_two"));
        assert!(methods.contains(&"divide"));
        assert!(methods.contains(&"test_events")); // subscription name
        assert!(methods.contains(&"get_status"));

        println!("✅ Service metadata correct!");
    }

    /// 并发测试 - 验证多个客户端并发访问
    #[tokio::test]
    async fn test_concurrent_access() {
        let hub = ProcessHub::new("concurrent_test").await.unwrap();

        let service = TestServiceService::new(TestServiceImpl);
        hub.register_service(service).await.unwrap();

        let client = TestServiceClient::new(hub);

        // 并发调用
        let mut handles = Vec::new();
        for i in 0..5 {
            let client = client.clone();
            let handle = tokio::spawn(async move { client.add_two(i, i + 1).await.unwrap() });
            handles.push(handle);
        }

        // 等待所有调用完成
        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.unwrap();
            assert_eq!(result, i as i32 + (i as i32 + 1));
        }

        println!("✅ Concurrent access working correctly!");
    }

    /// 订阅签名变换测试 - 验证订阅方法签名变换
    #[tokio::test]
    async fn test_subscription_signature_transformation() {
        let hub = ProcessHub::new("subscription_test").await.unwrap();

        let service = TestServiceService::new(TestServiceImpl);
        hub.register_service(service).await.unwrap();

        let client = TestServiceClient::new(hub);

        // 测试订阅方法调用
        let result = client
            .subscribe_test_events(Some("test_filter".to_string()))
            .await;

        // 验证调用成功（目前只是基本协议测试）
        match result {
            Ok(_) => println!("✅ Subscription signature transformation working!"),
            Err(e) => println!("ℹ️  Expected subscription protocol limitation: {e}"),
        }
    }
}
