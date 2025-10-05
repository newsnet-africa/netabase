# Netabase UX Test Suite

This crate provides comprehensive testing of the user experience for Netabase macro hygiene and dependency auto-export functionality. It validates that users can use Netabase macros without manually importing external dependencies like `serde`, `bincode`, `strum`, etc.

## Overview

The UX Test Suite is designed to ensure that Netabase provides an excellent developer experience through:

- **Macro Hygiene**: Users don't need to manually import dependencies for generated code
- **Dependency Auto-Export**: Convenient re-exports are available when needed
- **Clean APIs**: Simple, intuitive interfaces that "just work"
- **Real-World Validation**: Tests simulate actual usage patterns

## Test Categories

### 1. Hygiene Tests (`tests/hygiene_tests.rs`)

These tests validate that macros work without any manual imports of dependencies:

```rust
use netabase_macros::NetabaseModel;

#[derive(NetabaseModel, Clone, Debug)]
#[key_name(UserKey)]
struct User {
    #[key]
    id: u64,
    name: String,
}
```

Key validations:
- ✅ No manual imports of `serde`, `bincode`, `strum` required
- ✅ Generated code doesn't conflict with user's imports
- ✅ Multiple models in same scope work correctly
- ✅ Nested modules and complex scenarios compile

### 2. Convenience Tests (`tests/convenience_tests.rs`)

These tests validate the convenience re-exports provided by `netabase_deps`:

```rust
use netabase_deps::{bincode, serde, strum};
use netabase_macros::NetabaseModel;

#[derive(
    NetabaseModel,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
#[key_name(UserKey)]
struct User {
    #[key]
    pub id: u64,
    pub name: String,
}
```

Key validations:
- ✅ Re-exports work correctly with macros
- ✅ Version compatibility between macro-generated and user code
- ✅ No conflicts with user's own dependency versions
- ✅ All re-exported features are accessible

### 3. Integration Tests (`tests/integration_tests.rs`)

These tests validate complete workflows work end-to-end:

```rust
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::database::NetabaseDatabase;

#[netabase_schema_module(AppSchema, AppKeys)]
mod app_schema {
    // Complete schema with multiple models
}

// Full CRUD operations, networking, etc.
```

Key validations:
- ✅ Local database operations work completely
- ✅ Distributed networking functions correctly
- ✅ Schema modules integrate properly
- ✅ Performance under load is acceptable
- ✅ Concurrent access patterns work

### 4. Compilation Tests (`tests/compilation_tests.rs`)

These tests ensure generated code compiles efficiently and correctly:

```rust
// Tests various field types, attribute orders, complex generics, etc.
```

Key validations:
- ✅ All field types compile correctly
- ✅ Large models compile in reasonable time
- ✅ Generated code produces no warnings
- ✅ Conditional compilation works
- ✅ Attribute order flexibility

### 5. Real-World Scenarios (`tests/real_world_scenarios.rs`)

These tests simulate actual application use cases:

- **Blog System**: Users, posts, comments with full relationships
- **E-commerce**: Customers, products, orders with complex queries
- **Chat System**: Distributed messaging with real-time updates
- **Migration Scenarios**: Schema evolution over time

## Running Tests

### Run All Tests
```bash
cd ux_test_suite
cargo test
```

### Run Specific Test Categories
```bash
# Test only macro hygiene
cargo test hygiene_tests

# Test convenience re-exports
cargo test convenience_tests

# Test full integration
cargo test integration_tests

# Test compilation behavior
cargo test compilation_tests

# Test real-world scenarios
cargo test real_world_scenarios
```

### Run with Logging
```bash
RUST_LOG=debug cargo test -- --nocapture
```

## Test Framework

The suite includes a custom test framework (`src/test_helpers.rs`) with utilities for:

- **TestConfig**: Configure test behavior (hygiene-only, networking, etc.)
- **TestRunner**: Execute tests with consistent setup/cleanup
- **TestDatabase**: Temporary database management
- **TestModelFactory**: Generate consistent test data
- **Performance Helpers**: Measure execution time and resource usage

Example usage:

```rust
#[test]
fn my_test() -> TestResult {
    let config = TestConfig::new("my_test")
        .with_description("Test description")
        .hygiene_only();
    
    let runner = TestRunner::new(config);
    
    runner.run(|_config| {
        // Test implementation
        Ok(())
    })
}
```

## Key Principles Tested

### 1. Zero Manual Imports for Macro Usage

Users should never need to write:
```rust
use serde::{Serialize, Deserialize};
use bincode::{Encode, Decode};
```

Just to use `#[derive(NetabaseModel)]`.

### 2. Convenience When Needed

When users do need the dependencies for their own code:
```rust
use netabase_deps::{serde, bincode}; // Same versions as macros use
```

### 3. No Namespace Pollution

User code like this should never conflict:
```rust
let serde = "my_variable";
let bincode = "another_variable";

#[derive(NetabaseModel)]
struct MyModel { /* ... */ }
```

### 4. Consistent Behavior

The same patterns should work across:
- Different model complexities
- Various field types and attributes
- Local and distributed operations
- Different deployment scenarios

## Contributing

When adding new tests:

1. **Choose the right category** - Put tests in the appropriate file
2. **Use the test framework** - Leverage `TestConfig` and `TestRunner` for consistency
3. **Test both positive and negative cases** - Ensure things work AND fail appropriately
4. **Include real-world context** - Tests should reflect actual usage patterns
5. **Document test intent** - Clear comments explaining what's being validated

## Expected Outcomes

All tests in this suite should pass, validating that:

- ✅ Netabase macros are fully hygienic
- ✅ Dependencies are automatically handled
- ✅ User experience is smooth and intuitive
- ✅ Real-world scenarios work correctly
- ✅ Performance is acceptable
- ✅ API is consistent and predictable

This test suite serves as both validation and documentation of Netabase's commitment to developer experience excellence.