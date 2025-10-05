# netabase_store

A robust and efficient object-oriented key-value store built on top of sled db, providing advanced querying capabilities through primary and secondary key indexing.

## Features

- Object-oriented wrapper around sled db with strong typing
- Support for primary and secondary key indexing
- Advanced querying capabilities
- Batch operations with automatic indexing
- Macro-based model definitions
- Type-safe database operations

## Getting Started

Add netabase_store to your `Cargo.toml`:

```toml
[dependencies]
netabase_store = { path = "https://github.com/newsnet-africa/netabase_store.git" }
```

## Usage

### Defining Models

Use the provided macros to define your models:

```rust
use netabase_macros::{NetabaseModel, netabase_schema_module};

#[netabase_schema_module(MySchema, MySchemaKey)]
pub mod schema {
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug)]
    #[key_name(UserKey)]
    pub struct User {
        #[key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
        pub created_at: u64,
    }
}
```

### Key Features

1. **Primary Keys**: Mark fields with `#[key]` attribute
2. **Secondary Keys**: Add `#[secondary_key]` to enable indexing on additional fields
3. **Custom Key Names**: Use `#[key_name(KeyName)]` to specify custom key type names

### Basic Operations

```rust
// Initialize database
let db = NetabaseSledDatabase::new_with_path("path/to/db")?;

// Get tree for specific model
let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

// CRUD operations
user_tree.insert(user.key(), user)?;
let user = user_tree.get(key)?;
user_tree.remove(key)?;
```

### Secondary Key Queries

```rust
// Query by secondary key
let users = user_tree.query_by_secondary_key(
    UserSecondaryKeys::EmailKey("user@example.com".to_string())
)?;

// Query with custom filter
let senior_users = user_tree.query_with_filter(|user| user.age >= 30)?;
```

### Batch Operations

```rust
// Batch insert with automatic indexing
let items = vec![(key1, value1), (key2, value2)];
tree.batch_insert_with_indexing(items)?;
```

## Available Macros

1. `#[netabase_schema_module(Schema, SchemaKey)]`: Defines a schema module
2. `#[derive(NetabaseModel)]`: Implements necessary traits for database models
3. `#[key_name(KeyName)]`: Specifies the key type name for a model
4. `#[key]`: Marks the primary key field
5. `#[secondary_key]`: Marks fields for secondary indexing

## Advanced Features

- Range queries with prefix matching
- Count operations with predicates
- Secondary key value extraction
- Database-level indexing operations
- Relationship support between models

## TODO Items and Unimplemented Features

### High Priority TODOs

#### Dead Code and Unused Imports
- **Unused HashMap Import**: Remove unused import in traits.rs
  - Location: `src/traits.rs:1279`
  - Status: Warning - unused import `std::collections::HashMap`
  - Action: Remove unused import or implement usage

- **Unused Cow Import**: Remove unused import in sled.rs
  - Location: `src/database/sled.rs:11`
  - Status: Warning - unused import `std::borrow::Cow`
  - Action: Remove unused import

- **Unused NetabaseSchemaQuery**: Remove unused import
  - Location: `src/database/sled.rs:32`
  - Status: Warning - unused import `NetabaseSchemaQuery`
  - Action: Remove unused import or implement query functionality

#### Documentation TODOs
- **Bincode Explicit Requirement**: Fix old refactor issue
  - Location: `src/traits.rs:285-289`
  - Issue: Comment indicates "some old refactor did something weird that requires bincode to be explicit here"
  - Status: Technical debt from previous refactor
  - Action: Investigate and fix bincode requirement or document why it's needed

- **Example Code in Documentation**: Incomplete examples in lib.rs
  - Location: `src/lib.rs:207-208`
  - Issue: Documentation example uses `todo!()` placeholder
  - Status: Incomplete documentation
  - Action: Complete the example with working code

#### Test Implementation Gaps
- **Conversion Function Tests**: Test functions are commented out
  - Location: `tests/test_simple_record_store.rs:74-80`
  - Issue: `_test_conversions` function has commented out test code using `unimplemented!()`
  - Status: Tests not fully implemented
  - Action: Implement proper test cases or remove placeholder

- **Discriminant Method Tests**: Test methods are commented out
  - Location: `tests/test_simple_record_store.rs:139-146`
  - Issue: `_test_discriminant_methods` function has commented out test code using `unimplemented!()`
  - Status: Tests not fully implemented
  - Action: Implement proper test methods or document why they're disabled

### Medium Priority TODOs

#### Feature Implementation
- **WASM Compatibility**: Add WebAssembly support
  - Current: Only native support implemented
  - Action: Add conditional compilation for WASM builds
  - Considerations: Browser storage APIs, limited file system access

- **Async Operations**: Consider async database operations
  - Current: Synchronous sled operations
  - Action: Evaluate async sled features or wrapper implementations

- **Distributed Features**: Network-aware database operations
  - Current: Local database only
  - Action: Add networking capabilities for distributed scenarios

#### Performance Optimizations
- **Batch Operation Efficiency**: Optimize large batch operations
- **Index Performance**: Improve secondary key indexing performance
- **Memory Usage**: Optimize memory usage for large datasets

### Low Priority TODOs

#### Code Quality
- **Error Handling**: Standardize error types across the crate
- **Logging**: Add structured logging for database operations
- **Metrics**: Add performance metrics and monitoring

#### Testing
- **Property Tests**: Add property-based tests for database operations
- **Benchmark Tests**: Add performance benchmarks
- **Integration Tests**: Add full integration test suite

#### Documentation
- **Migration Guide**: Create migration guide for schema changes
- **Performance Guide**: Document performance characteristics
- **Best Practices**: Expand best practices section

## Platform Compatibility

### Native Features
- Full sled database functionality
- File system access for database storage
- Complete indexing and querying capabilities
- Batch operations with atomic transactions

### WASM Limitations
- **File System**: No direct file system access in WASM
- **Threading**: Limited threading capabilities
- **Storage**: Must use browser storage APIs
- **Performance**: May have different performance characteristics

## Known Issues

1. **Import Warnings**: Several unused imports need cleanup
2. **Documentation Examples**: Some examples use placeholder code
3. **Test Coverage**: Some test functions are incomplete
4. **WASM Support**: No WebAssembly compatibility implemented
5. **Async Support**: All operations are currently synchronous

## Best Practices

1. Always define primary keys for your models
2. Use secondary keys for frequently queried fields
3. Consider using batch operations for bulk updates
4. Properly handle database errors in your application
5. Use appropriate types for your keys and fields
6. Clean up unused imports to avoid warnings
7. Implement proper error handling for all database operations

