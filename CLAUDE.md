## Project Ideas

- Implement a multi-process communication framework inspired by https://github.com/bytedance/ipmb
  - Support inter-process communication
  - Enable topic-based message subscription and publishing

- Readme.md 使用中文
- 注释使用英文
- 不允许使用 println 宏，库的日志请使用 log 日志库，日志级别请使用 info 级别
- 每个阶段结束后执行 cargo fmt 格式化代码， cargo clippy 检查代码质量并修复

## Future Enhancements / 待评估功能

### Trait-based Service 实现完善 (高优先级) ✅ 已完成
- **目标**: 完善并推广 `#[service_trait]` + `#[service_impl]` 的 trait-based 服务定义方式
- **实现成果** (2025-07-08 完成):
  - ✅ 修复了 `service_impl_impl` 中的类型推断问题
  - ✅ 完善了 `service_trait` 的客户端生成逻辑，生成完全类型化的客户端
  - ✅ 导出了 `service_trait` 宏到 lib.rs
  - ✅ 创建了完整的 trait-based 示例：`examples/trait_based_service/`
  - ✅ 编写了全面的测试套件，验证了单进程模式下的完整功能
  - ✅ 更新了详细文档，包含对比表格和最佳实践建议

- **优势** (已验证):
  - **更好的类型安全**: 明确的接口定义和编译时检查 ✓
  - **支持多态性**: 成功实现了 BasicCalculator, SlowCalculator, CachedCalculator ✓
  - **更好的可测试性**: 易于创建 mock 实现进行单元测试 ✓
  - **组合模式支持**: 实现了缓存装饰器模式 ✓
  - **接口分离**: 清晰的服务契约定义 ✓

- **实现细节**:
  ```rust
  // 已实现的 API
  #[service_trait]
  trait Calculator {
      async fn add(&self, params: (i32, i32)) -> Result<i32>;
      async fn multiply(&self, params: (i32, i32)) -> Result<i32>;
  }

  struct BasicCalculator;

  #[service_impl]
  impl Calculator for BasicCalculator {
      async fn add(&self, params: (i32, i32)) -> Result<i32> {
          Ok(params.0 + params.1)
      }

      async fn multiply(&self, params: (i32, i32)) -> Result<i32> {
          Ok(params.0 * params.1)
      }
  }

  // 自动生成: CalculatorClient (完全类型化), BasicCalculatorService
  ```

- **关键改进**:
  - 服务包装器名称基于实现类型生成（如 `BasicCalculatorService`），避免命名冲突
  - 完整的类型推断，支持任意参数和返回类型
  - 客户端方法完全类型化，IDE 支持优秀

- **已知限制**:
  - 多进程通信存在时序问题（单进程模式工作正常）
  - 需要两个宏配合使用（`#[service_trait]` + `#[service_impl]`）

- **推荐使用场景**:
  - ✅ 新项目 - 强烈推荐使用 trait-based 方式
  - ✅ 需要多态性的复杂服务
  - ✅ 测试驱动开发项目
  - ✅ 团队协作项目（更好的接口契约）
  - ⚠️ 简单原型 - 可以继续使用 `#[service]` 单宏方式

### Service System v2 迁移
- **目标**: 将当前的 `service.rs` 替换为更现代化的 `service_v2.rs` 设计
- **优势**:
  - 更强的类型安全性 (`ServiceMethod<Request, Response>` trait)
  - 构建器模式提供更好的开发体验
  - 支持直接使用函数/闭包作为服务方法
  - 更简洁的服务定义语法
- **示例对比**:
  ```rust
  // 当前 service.rs 方式
  #[service]
  impl Calculator {
      async fn add(&self, params: (i32, i32)) -> Result<i32> {
          Ok(params.0 + params.1)
      }
  }

  // service_v2.rs 方式 (更简洁)
  let calculator = ServiceBuilder::new("Calculator")
      .method("add", |req: (i32, i32)| async move {
          Ok(req.0 + req.1)
      }).await?
      .build();
  ```
- **迁移需求**:
  - 更新宏系统以支持新的服务定义方式
  - 修改 ProcessHub 集成新的 MethodRegistry
  - 更新示例和测试以使用新的 API
  - 确保向后兼容性或提供迁移路径

## 实现优先级

1. **✅ 已完成**: Trait-based Service 实现完善
   - 这是一个架构层面的改进，能显著提升库的设计质量
   - 用户反馈表明 trait 方式更符合 Rust 最佳实践
   - **状态**: 完全实现，单进程模式功能齐全

2. **中优先级**: Service System v2 迁移
   - 这是一个全新的 API 设计，需要更多的设计和实现工作
   - 可以在 trait-based 方式稳定后再考虑

3. **新发现的优先级**: 多进程通信优化
   - trait-based 服务在多进程模式下存在时序问题
   - 需要进一步调试和优化 IPMB 传输层的服务发现机制

## 项目进展总结 (2025-07-08)

### 🎉 重大成就
- **Trait-based Service 架构完全实现**: 成为 hsipc 的新推荐服务定义方式
- **类型安全大幅提升**: 从运行时方法解析改进到编译时类型检查
- **开发体验显著改善**: 完整的 IDE 支持和类型提示
- **多态性支持**: 首次实现同一服务接口的多种实现方式

### 📂 新增文件和功能
- `examples/trait_based_service/` - 完整的示例项目
- 增强的宏实现支持完整类型推断
- 详细的对比文档和最佳实践指南

### 🔬 技术亮点
- 自动生成完全类型化的客户端 (如 `CalculatorClient`)
- 基于实现类型的唯一服务包装器命名
- 支持复杂的组合模式 (装饰器, 缓存等)
- 全面的测试覆盖 (单元测试, 集成测试, 并发测试)

### 🚀 未来方向
1. 多进程通信稳定性优化
2. 性能基准测试和优化
3. 更多复杂的服务组合模式示例
