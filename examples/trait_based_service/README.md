# Trait-based Service Example

This example demonstrates the **enhanced trait-based approach** to service definition in hsipc, showcasing the significant advantages of using `#[service_trait]` and `#[service_impl]` together over the traditional `#[service]` approach.

## ✨ Why Trait-Based Services?

The trait-based approach addresses key limitations of the direct `#[service]` implementation:

### 🎯 **Type Safety & Interface Definition**
- **Clear contracts**: Traits define explicit service interfaces
- **Compile-time verification**: Interface consistency is guaranteed  
- **IDE support**: Full autocomplete, type hints, and error checking

### 🔄 **Polymorphism & Flexibility**
- **Multiple implementations**: Same interface, different behaviors
- **Easy testing**: Mock implementations for unit tests
- **Decorator patterns**: Compose services with logging, caching, etc.

### 🧪 **Better Testability**
- **Interface separation**: Test against traits, not concrete types
- **Mock creation**: Easy to create test doubles
- **Isolation**: Test business logic independently

## Features Demonstrated

### 1. Type-Safe Interface Definition
- Clear separation between interface (`trait Calculator`) and implementation
- Fully typed client methods with compile-time verification
- Better IDE support with autocomplete and type checking

### 2. Polymorphism Support
- Multiple implementations of the same trait interface:
  - `BasicCalculator` - Simple implementation
  - `SlowCalculator` - Simulates expensive operations
  - `CachedCalculator` - Demonstrates composition with caching

### 3. Enhanced Client Generation
- Auto-generated `CalculatorClient` with typed methods
- Direct method calls: `client.add((10, 5))` returns `i32`
- No manual type annotations needed on client side

## Usage

### 1. Build the example
```bash
cd examples/trait_based_service
cargo build
```

### 2. Run polymorphism demonstration (Recommended)
```bash
cargo run demo
```
This shows how different implementations can be used polymorphically within a single process.

### 3. Run comprehensive tests
```bash
cargo run --bin tests
```
Demonstrates single-process functionality with comprehensive test coverage.

### 4. Run single process integration tests
```bash
# Simple single-process test
cargo run --bin single_process_test

# Shared hub test (server + client)
cargo run --bin shared_hub_test
```

### 5. Run as separate client-server processes (Experimental)
**Note**: Multi-process communication has some timing issues. Single-process mode is recommended.

In one terminal, start a server:
```bash
# Basic implementation
cargo run server basic

# Or slow implementation  
cargo run server slow

# Or cached implementation  
cargo run server cached
```

In another terminal, run the client:
```bash
cargo run client
```

## Code Structure

### Service Interface Definition
```rust
#[service_trait]
trait Calculator {
    async fn add(&self, params: (i32, i32)) -> Result<i32>;
    async fn multiply(&self, params: (i32, i32)) -> Result<i32>;
    async fn factorial(&self, n: i32) -> Result<i64>;
}
```

This generates:
- `CalculatorClient` with fully typed methods

### Service Implementation
```rust
struct BasicCalculator;

#[service_impl]
impl Calculator for BasicCalculator {
    async fn add(&self, params: (i32, i32)) -> Result<i32> {
        Ok(params.0 + params.1)
    }
    // ... other methods
}
```

This generates:
- `CalculatorService` wrapper for registration
- `Service` trait implementation for message handling

## 🆚 Comparison: Trait vs Direct `#[service]` Approach

| Feature | `#[service]` Approach | **Trait-Based Approach** |
|---------|----------------------|---------------------------|
| **Interface Definition** | ❌ Implicit (implementation only) | ✅ Explicit trait contract |
| **Type Safety** | ⚠️ Runtime method resolution | ✅ Compile-time verification |
| **Client Generation** | ✅ Auto-generated | ✅ **Fully typed** auto-generated |
| **Polymorphism** | ❌ Single implementation per service | ✅ Multiple implementations |
| **Testability** | ⚠️ Hard to mock | ✅ Easy mock creation |
| **IDE Support** | ⚠️ Limited autocomplete | ✅ **Full type hints & autocomplete** |
| **Composition** | ❌ Difficult | ✅ Decorator patterns supported |
| **Learning Curve** | ✅ Simple (one macro) | ⚠️ Requires understanding traits |

### Code Example Comparison

#### Traditional `#[service]` Approach:
```rust
#[derive(Debug)]
pub struct Calculator;

#[service]  // ❌ No explicit interface
impl Calculator {
    async fn add(&self, params: (i32, i32)) -> Result<i32> {
        Ok(params.0 + params.1)
    }
}

// ❌ Hard to test, no polymorphism
let service = CalculatorService::new(Calculator);
```

#### **Enhanced Trait-Based Approach:**
```rust
#[service_trait]  // ✅ Explicit interface definition
trait Calculator {
    async fn add(&self, params: (i32, i32)) -> Result<i32>;
}

struct FastCalculator;
struct CachedCalculator { /* ... */ }

#[service_impl]  // ✅ Multiple implementations
impl Calculator for FastCalculator { /* ... */ }

#[service_impl]  // ✅ Easy to test and compose
impl Calculator for CachedCalculator { /* ... */ }
```

## Advantages Over `#[service]` Approach

### 1. **🎯 Superior Type Safety**
- Interface is explicitly defined in the trait
- Client methods have exact parameter and return types  
- Compile-time verification of implementations
- **Zero runtime surprises**

### 2. **🔄 True Polymorphism**
- Multiple implementations of the same interface
- Easy to swap implementations at runtime
- Support for decorator patterns (like `CachedCalculator`)
- **Same interface, different behaviors**

### 3. **🧪 Exceptional Testability**
- Easy to create mock implementations for testing
- Interface separation enables better unit testing
- **Test against contracts, not implementations**

### 4. **🔧 Better Composition**
- Can compose services (e.g., cached wrapper around basic implementation)
- Clear separation of concerns
- **Build complex behaviors from simple parts**

### 5. **💡 Enhanced IDE Support**
- Full autocomplete for client methods
- Type hints and error checking
- Better refactoring support
- **Developer experience like working with native Rust**

## Performance Comparison

Run with different implementations to see:
- `basic`: Fast, simple operations
- `slow`: Simulated expensive operations with delays
- `cached`: Shows caching behavior on repeated calls

## Generated Code

The macros generate:

1. **From `#[service_trait]`**:
   - `CalculatorClient` struct with typed methods

2. **From `#[service_impl]`**:
   - `Service` trait implementation for the struct
   - `CalculatorService` wrapper for registration

This provides a complete, type-safe service framework with minimal boilerplate.

## 🎯 Recommendation

**The trait-based approach (`#[service_trait]` + `#[service_impl]`) is strongly recommended for new projects** due to its superior type safety, polymorphism support, and testability. While it requires understanding Rust traits, the benefits far outweigh the learning curve:

### When to Use Trait-Based Services:
- ✅ **New projects** - Start with the better foundation
- ✅ **Complex services** - Need polymorphism or composition  
- ✅ **Team development** - Better interfaces and contracts
- ✅ **Testing-heavy projects** - Easy mock creation
- ✅ **Long-term maintenance** - Better refactoring support

### When to Use `#[service]`:
- ⚠️ **Quick prototypes** - Simpler one-macro approach
- ⚠️ **Legacy projects** - Already using the old approach
- ⚠️ **Simple use cases** - Single implementation, no testing needs

## 🚀 Future Direction

The trait-based approach represents the future direction of hsipc service development. It addresses fundamental limitations of the direct implementation approach and provides a more robust foundation for building scalable IPC applications.