# NetabaseSchema Generic Types Support

This crate demonstrates comprehensive support for generic types in NetabaseSchema, providing examples and patterns for using generics effectively in distributed data storage scenarios.

## Overview

NetabaseSchema supports generic types with full trait bound propagation, allowing you to create flexible, type-safe data structures that can be stored and retrieved from the distributed network. This enables you to build reusable components while maintaining compile-time type safety.

## Features

- **Single Type Parameter Generics**: Simple generic structs and enums
- **Multiple Type Parameters**: Complex data structures with multiple generic types
- **Generic Enums**: Variant-specific generic data handling
- **Nested Generics**: Generic types containing other generic types
- **Complex Trait Bounds**: Advanced constraint handling including `Send`, `Sync`, `Hash`, etc.
- **Default Type Parameters**: Flexible APIs with sensible defaults
- **Real-world Examples**: Practical patterns for common use cases

## Quick Start

### Basic Generic Structure

```rust
use netabase_macros::{NetabaseSchema, schema_module};
use bincode::{Encode, Decode};

#[schema_module(MyRegistry)]
pub mod schemas {
    use bincode::{Decode, Encode};
    use netabase::netabase_trait::NetabaseSchema as NetabaseSchemaTrait;
    use netabase_macros::{NetabaseSchema, NetabaseSchemaKey};

    #[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
    pub struct Container<T: Encode + Decode<()> + Clone + Debug> {
        #[key]
        pub id: String,
        pub content: T,
        pub timestamp: u64,
    }
}

// Usage
let string_container = Container {
    id: "example-1".to_string(),
    content: "Hello, World!".to_string(),
    timestamp: 1234567890,
};

let number_container = Container {
    id: "example-2".to_string(),
    content: 42u64,
    timestamp: 1234567891,
};
```

## Examples

### 1. Basic Generic Types (`examples/basic_generics.rs`)

Simple single-parameter generic types:

- `Container<T>`: Generic wrapper for any serializable data
- `KeyValue<V>`: Key-value pairs with typed values
- `GenericList<T>`: Lists of generic items

### 2. Advanced Generic Types (`examples/advanced_generics.rs`)

Complex multi-parameter generic types:

- `Document<TContent, TMeta>`: Documents with generic content and metadata
- `Event<TPayload, TContext>`: Event system with typed payloads
- `Database<K, V>`: Generic database with configurable key/value types
- `StateMachine<TState, TEvent, TContext>`: Type-safe state machines

## Core Generic Patterns

### 1. Single Type Parameter

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct GenericData<T: Encode + Decode<()> + Clone + Debug> {
    #[key]
    pub id: String,
    pub data: T,
    pub timestamp: u64,
}
```

### 2. Multiple Type Parameters

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct MultiGeneric<K, V>
where
    K: Encode + Decode<()> + Clone + Debug,
    V: Encode + Decode<()> + Clone + Debug,
{
    #[key]
    pub primary_key: String,
    pub key_value_pairs: HashMap<K, V>,
    pub metadata: Option<String>,
}
```

### 3. Generic Enums

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub enum GenericMessage<T: Encode + Decode<()> + Clone + Debug> {
    Text(#[key] String),
    Data {
        #[key]
        message_id: u64,
        payload: T,
    },
    Binary(#[key] String, Vec<u8>),
}
```

### 4. Complex Constraints

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct Database<K, V>
where
    K: Encode + Decode<()> + Clone + Debug + Hash + Eq,
    V: Encode + Decode<()> + Clone + Debug,
{
    #[key]
    pub database_name: String,
    pub data: HashMap<K, V>,
    pub indexes: BTreeMap<String, Vec<K>>,
}
```

## Required Trait Bounds

All generic type parameters used with NetabaseSchema must implement:

### Minimum Required Traits

- `bincode::Encode`: For serialization
- `bincode::Decode<()>`: For deserialization
- `Clone`: For data handling
- `std::fmt::Debug`: For debugging

### Additional Common Traits

- `Send + Sync`: For thread safety (recommended for distributed systems)
- `Hash + Eq`: For use as HashMap keys
- `Default`: For default value generation
- `PartialOrd + Ord`: For ordered collections

### Example with Full Constraints

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct FullyConstrained<T>
where
    T: Encode + Decode<()> + Clone + Debug + Send + Sync + Hash + Eq + Default,
{
    #[key]
    pub id: String,
    pub data: T,
    pub fallback: T,
}
```

## Real-World Use Cases

### 1. Configuration Management

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct ConfigStore<T: Encode + Decode<()> + Clone + Debug + Default> {
    #[key]
    pub config_key: String,
    pub value: T,
    pub default_value: T,
    pub description: String,
}

// Usage
let string_config = ConfigStore {
    config_key: "app.name".to_string(),
    value: "MyApp".to_string(),
    default_value: "DefaultApp".to_string(),
    description: "Application name".to_string(),
};
```

### 2. Caching System

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct CacheEntry<T: Encode + Decode<()> + Clone + Debug> {
    #[key]
    pub cache_key: String,
    pub value: T,
    pub ttl_seconds: u64,
    pub created_at: u64,
}
```

### 3. Event Processing

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub enum Event<TPayload, TContext>
where
    TPayload: Encode + Decode<()> + Clone + Debug,
    TContext: Encode + Decode<()> + Clone + Debug,
{
    UserAction {
        #[key]
        event_id: String,
        user_id: String,
        payload: TPayload,
        context: TContext,
    },
    SystemEvent {
        #[key]
        event_id: String,
        source: String,
        payload: TPayload,
    },
}
```

## Best Practices

### 1. Always Specify Required Bounds

```rust
// ✅ Good: Explicit bounds
pub struct MyStruct<T: Encode + Decode<()> + Clone + Debug> {
    #[key]
    pub id: String,
    pub data: T,
}

// ❌ Bad: Missing bounds will cause compilation errors
pub struct MyStruct<T> {
    #[key] 
    pub id: String,
    pub data: T,
}
```

### 2. Use Where Clauses for Complex Bounds

```rust
// ✅ Good: Clear where clause
pub struct ComplexStruct<K, V>
where
    K: Encode + Decode<()> + Clone + Debug + Hash + Eq,
    V: Encode + Decode<()> + Clone + Debug + Send + Sync,
{
    #[key]
    pub key: K,
    pub value: V,
}
```

### 3. Provide Helper Functions

```rust
pub mod helpers {
    use super::schemas::*;
    
    pub fn create_string_container(id: &str, content: &str) -> Container<String> {
        Container {
            id: id.to_string(),
            content: content.to_string(),
            timestamp: current_timestamp(),
        }
    }
    
    pub fn create_number_container<T>(id: &str, number: T) -> Container<T>
    where
        T: Encode + Decode<()> + Clone + Debug,
    {
        Container {
            id: id.to_string(),
            content: number,
            timestamp: current_timestamp(),
        }
    }
}
```

### 4. Document Type Requirements

```rust
/// A generic document store that can hold any serializable content.
/// 
/// # Type Parameters
/// 
/// * `TContent` - The document content type. Must be serializable.
/// * `TMeta` - The metadata type. Often `HashMap<String, String>`.
/// 
/// # Examples
/// 
/// ```rust
/// let doc = Document {
///     document_id: "doc-1".to_string(),
///     title: "My Document".to_string(),
///     content: "Document content".to_string(),
///     metadata: HashMap::new(),
///     version: 1,
/// };
/// ```
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct Document<TContent, TMeta>
where
    TContent: Encode + Decode<()> + Clone + Debug,
    TMeta: Encode + Decode<()> + Clone + Debug,
{
    #[key]
    pub document_id: String,
    pub title: String,
    pub content: TContent,
    pub metadata: TMeta,
    pub version: u32,
}
```

## Testing Your Generic Types

The crate includes comprehensive tests for all generic patterns:

```bash
cargo test --lib
```

Tests cover:

- Basic generic functionality
- Multi-parameter generics
- Generic enums
- Nested generic structures
- Complex trait bounds
- Real-world usage patterns

## Common Issues and Solutions

### 1. Missing Trait Bounds

**Error**: `the trait bound 'T: Encode' is not satisfied`

**Solution**: Add required trait bounds to your generic parameters:

```rust
// Add missing bounds
pub struct MyStruct<T: Encode + Decode<()> + Clone + Debug> { ... }
```

### 2. Complex Bounds

**Error**: Long trait bound lists in function signatures

**Solution**: Use type aliases:

```rust
type SerializableType<T> = T where T: Encode + Decode<()> + Clone + Debug;

pub struct MyStruct<T: SerializableType<T>> { ... }
```

### 3. Default Type Parameters

**Error**: Need to specify types every time

**Solution**: Use default type parameters:

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct Container<T = String>
where
    T: Encode + Decode<()> + Clone + Debug + Default,
{
    #[key]
    pub id: String,
    pub content: T,
}

// Now you can use Container::default() for Container<String>
```

## Contributing

When adding new generic type examples:

1. Ensure all required trait bounds are present
2. Add comprehensive tests
3. Include usage examples in documentation
4. Follow the established patterns for helper functions
5. Update this README with new patterns

## License

This crate is part of the NetabaseSchema project and follows the same licensing terms.