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
netabase_store = { path = "path/to/netabase_store" }
netabase_macros = { path = "path/to/netabase_macros" }
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

## Best Practices

1. Always define primary keys for your models
2. Use secondary keys for frequently queried fields
3. Consider using batch operations for bulk updates
4. Properly handle database errors in your application
5. Use appropriate types for your keys and fields

