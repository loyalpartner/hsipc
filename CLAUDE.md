## 开发规范

- 文档放在 docs 目录
- 查看 @README.md 了解项目概述
- git 工作流程 @docs/git-instructions.md, 按照这个流程开发
- 测试流程 @docs/TESTING.md
- 架构文档 @docs/ARCHITECTURE.md
- API 设计 @docs/API.md
- 性能测试 @docs/PERFORMANCE.md

当我们开始开发一个新功能时，请遵循以下规范：


- 先更新相关的文档，我确认过后再开始执行
- 我们采用的是 tdd 的开发模式
- Readme.md 使用中文
- 注释使用英文
- 不允许使用 println 宏，库的日志请使用 log 日志库，日志级别请使用 info 级别, 用 trace 级别
- 错误需要用 thiserror 重构下


### TODO

目前我们提供的是 service_trait, service 宏, 我想要提供一个更好的 trait 版本的 service, 需要实现下面类似的风格, 这种宏的方式很棒
参数和返回值按照我们现在的来， 能够支持同步和异步的方式

重构不需要兼容以前的代码， 我们的代码还没有发布

```
#[rpc(server, client, namespace = "state")]
pub trait Rpc<Hash, StorageKey>
where
	Hash: std::fmt::Debug,
{
	/// Async method call example.
	#[method(name = "getKeys")]
	async fn storage_keys(
		&self,
		storage_key: StorageKey,
		hash: Option<Hash>,
	) -> Result<Vec<StorageKey>, ErrorObjectOwned>;

	/// Subscription that takes a `StorageKey` as input and produces a `Vec<Hash>`.
	#[subscription(name = "subscribeStorage" => "override", item = Vec<Hash>)]
	async fn subscribe_storage(&self, keys: Option<Vec<StorageKey>>) -> SubscriptionResult;

	#[subscription(name = "subscribeSync" => "sync", item = Vec<Hash>)]
	fn s(&self, keys: Option<Vec<StorageKey>>);
}

#[async_trait]
impl RpcServer<ExampleHash, ExampleStorageKey> for RpcServerImpl {
	async fn storage_keys(
		&self,
		storage_key: ExampleStorageKey,
		_hash: Option<ExampleHash>,
	) -> Result<Vec<ExampleStorageKey>, ErrorObjectOwned> {
		Ok(vec![storage_key])
	}

	async fn subscribe_storage(
		&self,
		pending: PendingSubscriptionSink,
		_keys: Option<Vec<ExampleStorageKey>>,
	) -> SubscriptionResult {

		Ok(())
	}

	fn s(&self, pending: PendingSubscriptionSink, _keys: Option<Vec<ExampleStorageKey>>) {
		...
	}
}
```
