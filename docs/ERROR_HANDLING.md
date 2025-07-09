# hsipc 错误处理指南

## 概述

hsipc 提供了灵活的错误处理机制，允许用户以多种方式处理错误，同时保持类型安全。

## 错误处理方式

### 1. 使用字符串（最简单）

对于简单的错误情况，可以直接使用字符串：

```rust
async fn my_method(&self, input: i32) -> Result<i32> {
    if input < 0 {
        return Err("Input must be non-negative".into());
    }
    Ok(input * 2)
}
```

### 2. 使用自定义错误类型

定义自己的错误类型，并实现 `From` trait：

```rust
#[derive(Debug)]
enum MyError {
    InvalidInput(String),
    ProcessingError,
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            MyError::ProcessingError => write!(f, "Processing error"),
        }
    }
}

impl std::error::Error for MyError {}

// 可选：为更好的集成实现 From trait
impl From<MyError> for hsipc::Error {
    fn from(err: MyError) -> Self {
        match err {
            MyError::InvalidInput(_) => 
                hsipc::Error::invalid_request(err.to_string(), None),
            MyError::ProcessingError => 
                hsipc::Error::runtime_msg(err.to_string()),
        }
    }
}

// 使用
async fn my_method(&self, input: i32) -> Result<i32> {
    if input < 0 {
        return Err(MyError::InvalidInput("negative value".to_string()).into());
    }
    Ok(input * 2)
}
```

### 3. 使用标准库错误

标准库的错误类型（如 `std::io::Error`）会自动转换：

```rust
async fn read_config(&self) -> Result<String> {
    // ? 操作符会自动将 io::Error 转换为 hsipc::Error
    let content = std::fs::read_to_string("config.toml")?;
    Ok(content)
}
```

### 4. 使用 from_std 方法

对于没有实现 `From` trait 的错误类型：

```rust
use some_crate::SomeError;

async fn process(&self) -> Result<()> {
    match some_operation() {
        Ok(()) => Ok(()),
        Err(e) => Err(hsipc::Error::from_std(e)),
    }
}
```

## 错误分类

虽然用户通常使用简单的字符串错误，但了解 hsipc 的错误分类有助于调试：

- **可重试错误**：网络、连接、超时错误
- **不可重试错误**：无效请求、配置错误、序列化错误

## 最佳实践

1. **简单场景使用字符串**：对于原型和简单应用，直接使用字符串错误
2. **复杂场景使用自定义类型**：对于生产应用，定义自己的错误类型
3. **提供有意义的错误信息**：错误消息应该帮助用户理解和解决问题
4. **使用 ? 操作符**：充分利用 Rust 的错误传播机制

## 示例对比

### 之前（紧耦合）
```rust
return Err(hsipc::Error::invalid_request(
    "Negative factorial",
    Some("factorial is only defined for non-negative integers".to_string()),
));
```

### 现在（松耦合）
```rust
// 方式 1：简单字符串
return Err("Negative factorial: only defined for non-negative integers".into());

// 方式 2：自定义错误
return Err(MathError::NegativeFactorial.into());
```

## 内部实现说明

hsipc 内部使用 `thiserror` 来减少样板代码，但这是实现细节：
- 用户**不需要**依赖 `thiserror`
- 用户**不需要**了解 `hsipc::Error` 的内部结构
- 公共 API 只依赖标准库的 `std::error::Error` trait