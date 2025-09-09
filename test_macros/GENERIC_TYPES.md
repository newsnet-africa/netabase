# NetabaseSchema Generic Types Guide

## Overview

NetabaseSchema provides comprehensive support for generic types, allowing you to create flexible, type-safe data structures that can be stored and retrieved from the distributed network. This guide demonstrates how to effectively use generics with NetabaseSchema.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Basic Generic Types](#basic-generic-types)
3. [Advanced Generic Patterns](#advanced-generic-patterns)
4. [Real-World Examples](#real-world-examples)
5. [Best Practices](#best-practices)
6. [Common Patterns](#common-patterns)
7. [Troubleshooting](#troubleshooting)
8. [API Reference](#api-reference)

## Quick Start

### Minimum Requirements

All generic type parameters must implement these traits:
- `bincode::Encode` - For serialization
- `bincode::Decode<()>` - For deserialization  
- `Clone` - For data handling
- `std::fmt::Debug` - For debugging

### Basic Example

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

## Basic Generic Types

### Single Type Parameter

The simplest form of generic NetabaseSchema:

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct GenericData<T: Encode + Decode<()> + Clone + Debug> {
    #[key]
    pub id: String,
    pub data: T,
    pub created_at: u64,
}
```

### Multiple Type Parameters

For more complex data relationships:

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct KeyValueStore<K, V>
where
    K: Encode + Decode<()> + Clone + Debug + Hash + Eq,
    V: Encode + Decode<()> + Clone + Debug,
{
    #[key]
    pub store_name: String,
    pub data: HashMap<K, V>,
    pub metadata: Option<String>,
}
```

### Generic Enums

Enums can also be generic with different key strategies per variant:

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub enum Message<T: Encode + Decode<()> + Clone + Debug> {
    Text(#[key] String),
    Data {
        #[key]
        message_id: u64,
        payload: T,
    },
    Binary(#[key] String, Vec<u8>),
}
```

### Default Type Parameters

Make your APIs more ergonomic with defaults:

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct ConfigValue<T = String>
where
    T: Encode + Decode<()> + Clone + Debug + Default,
{
    #[key]
    pub key: String,
    pub value: T,
    pub default: T,
}
```

## Advanced Generic Patterns

### Nested Generics

Generic types can contain other generic types:

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct CompositeData<T, U>
where
    T: Encode + Decode<()> + Clone + Debug,
    U: Encode + Decode<()> + Clone + Debug,
{
    #[key]
    pub composite_id: String,
    pub primary: GenericData<T>,
    pub secondary: GenericData<U>,
    pub relation: String,
}
```

### Complex Trait Bounds

For advanced use cases, you can specify additional trait bounds:

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct ThreadSafeCache<T>
where
    T: Encode + Decode<()> + Clone + Debug + Send + Sync + 'static,
{
    #[key]
    pub cache_key: String,
    pub value: T,
    pub ttl: Duration,
    pub created_at: SystemTime,
}
```

### Generic with Associated Types

You can use generics alongside associated types:

```rust
pub trait Processor {
    type Input: Encode + Decode<()> + Clone + Debug;
    type Output: Encode + Decode<()> + Clone + Debug;
}

#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct ProcessingTask<P: Processor> {
    #[key]
    pub task_id: String,
    pub input: P::Input,
    pub output: Option<P::Output>,
    pub status: TaskStatus,
}
```

## Real-World Examples

### 1. Distributed Caching System

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct CacheEntry<T: Encode + Decode<()> + Clone + Debug> {
    #[key]
    pub cache_key: String,
    pub value: T,
    pub ttl_seconds: u64,
    pub created_at: u64,
    pub access_count: u32,
    pub replicas: Vec<String>,
}

// Usage
let user_cache = CacheEntry {
    cache_key: "user:123".to_string(),
    value: UserProfile {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    },
    ttl_seconds: 3600,
    created_at: current_timestamp(),
    access_count: 0,
    replicas: vec!["node-1".to_string(), "node-2".to_string()],
};
```

### 2. Event Processing System

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
        timestamp: u64,
    },
    SystemEvent {
        #[key]
        event_id: String,
        source: String,
        payload: TPayload,
        severity: u8,
    },
}

// Usage
let login_event = Event::UserAction {
    event_id: "evt-001".to_string(),
    user_id: "user-123".to_string(),
    payload: LoginPayload {
        ip_address: "192.168.1.1".to_string(),
        user_agent: "Mozilla/5.0".to_string(),
    },
    context: RequestContext {
        session_id: "sess-456".to_string(),
        request_id: "req-789".to_string(),
    },
    timestamp: current_timestamp(),
};
```

### 3. Document Management

```rust
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
    pub owner_id: String,
    pub collaborators: Vec<String>,
    pub version: u32,
    pub last_modified: u64,
}

// Text document
let text_doc = Document {
    document_id: "doc-001".to_string(),
    title: "Meeting Notes".to_string(),
    content: "## Meeting Notes\n\n- Topic 1\n- Topic 2".to_string(),
    metadata: HashMap::from([
        ("format".to_string(), "markdown".to_string()),
        ("category".to_string(), "meeting".to_string()),
    ]),
    owner_id: "user-alice".to_string(),
    collaborators: vec!["user-bob".to_string()],
    version: 1,
    last_modified: current_timestamp(),
};

// Structured document
let data_doc = Document {
    document_id: "doc-002".to_string(),
    title: "Sensor Readings".to_string(),
    content: vec![
        SensorReading { temp: 23.5, humidity: 65.2 },
        SensorReading { temp: 24.1, humidity: 63.8 },
    ],
    metadata: SensorMetadata {
        location: "Room 101".to_string(),
        device_id: "sensor-001".to_string(),
    },
    owner_id: "system".to_string(),
    collaborators: vec![],
    version: 15,
    last_modified: current_timestamp(),
};
```

### 4. Configuration Management

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct Config<T: Encode + Decode<()> + Clone + Debug + Default> {
    #[key]
    pub config_key: String,
    pub value: T,
    pub default_value: T,
    pub description: String,
    pub is_secret: bool,
    pub scope: ConfigScope,
    pub last_updated: u64,
}

// String configuration
let app_name = Config {
    config_key: "app.name".to_string(),
    value: "MyApplication".to_string(),
    default_value: "DefaultApp".to_string(),
    description: "Application display name".to_string(),
    is_secret: false,
    scope: ConfigScope::Global,
    last_updated: current_timestamp(),
};

// Numeric configuration
let max_connections = Config {
    config_key: "server.max_connections".to_string(),
    value: 1000u32,
    default_value: 100u32,
    description: "Maximum concurrent connections".to_string(),
    is_secret: false,
    scope: ConfigScope::NodeLocal,
    last_updated: current_timestamp(),
};
```

## Best Practices

### 1. Always Specify Required Bounds

```rust
// ✅ Good: Explicit bounds
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct MyData<T: Encode + Decode<()> + Clone + Debug> {
    #[key]
    pub id: String,
    pub data: T,
}

// ❌ Bad: Missing bounds will cause compilation errors
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct MyData<T> {
    #[key] 
    pub id: String,
    pub data: T,  // This will fail
}
```

### 2. Use Where Clauses for Complex Bounds

```rust
// ✅ Good: Clear where clause
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct ComplexData<K, V>
where
    K: Encode + Decode<()> + Clone + Debug + Hash + Eq,
    V: Encode + Decode<()> + Clone + Debug + Send + Sync,
{
    #[key]
    pub key: K,
    pub value: V,
}
```

### 3. Provide Helper Constructors

```rust
impl<T> GenericData<T> 
where 
    T: Encode + Decode<()> + Clone + Debug,
{
    pub fn new(id: String, data: T) -> Self {
        Self {
            id,
            data,
            created_at: current_timestamp(),
        }
    }
}

// Or as module functions
pub mod helpers {
    pub fn create_string_data(id: &str, content: &str) -> GenericData<String> {
        GenericData::new(id.to_string(), content.to_string())
    }
    
    pub fn create_numeric_data<T>(id: &str, number: T) -> GenericData<T>
    where
        T: Encode + Decode<()> + Clone + Debug,
    {
        GenericData::new(id.to_string(), number)
    }
}
```

### 4. Document Type Requirements

```rust
/// A generic container that can hold any serializable data.
/// 
/// # Type Parameters
/// 
/// * `T` - The data type to store. Must implement:
///   - `bincode::Encode` for serialization
///   - `bincode::Decode<()>` for deserialization
///   - `Clone` for data handling
///   - `Debug` for debugging
/// 
/// # Examples
/// 
/// ```rust
/// let string_container = Container {
///     id: "example".to_string(),
///     content: "Hello, World!".to_string(),
///     timestamp: 1234567890,
/// };
/// ```
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct Container<T: Encode + Decode<()> + Clone + Debug> {
    #[key]
    pub id: String,
    pub content: T,
    pub timestamp: u64,
}
```

### 5. Test Different Type Parameters

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_different_types() {
        // Test with String
        let string_data = GenericData::new("test-1".to_string(), "hello".to_string());
        assert_eq!(string_data.key().0, "test-1");
        
        // Test with numbers
        let number_data = GenericData::new("test-2".to_string(), 42i32);
        assert_eq!(number_data.data, 42);
        
        // Test with complex types
        let vec_data = GenericData::new("test-3".to_string(), vec![1, 2, 3]);
        assert_eq!(vec_data.data.len(), 3);
        
        // Test with custom types
        #[derive(Encode, Decode, Clone, Debug, PartialEq)]
        struct CustomData {
            name: String,
            value: u32,
        }
        
        let custom_data = GenericData::new("test-4".to_string(), CustomData {
            name: "test".to_string(),
            value: 100,
        });
        assert_eq!(custom_data.data.name, "test");
    }
}
```

## Common Patterns

### 1. Generic Wrapper Pattern

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct Wrapper<T: Encode + Decode<()> + Clone + Debug> {
    #[key]
    pub id: String,
    pub inner: T,
    pub metadata: HashMap<String, String>,
}
```

### 2. Generic Key-Value Pattern

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct KeyValue<K, V>
where
    K: Encode + Decode<()> + Clone + Debug + Hash + Eq,
    V: Encode + Decode<()> + Clone + Debug,
{
    #[key]
    pub namespace: String,
    pub entries: HashMap<K, V>,
}
```

### 3. Generic Event Pattern

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub enum GenericEvent<T: Encode + Decode<()> + Clone + Debug> {
    Created(#[key] String, T),
    Updated(#[key] String, T),
    Deleted(#[key] String),
}
```

### 4. Generic Cache Pattern

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct CacheItem<T: Encode + Decode<()> + Clone + Debug> {
    #[key]
    pub key: String,
    pub value: T,
    pub expires_at: Option<u64>,
    pub tags: Vec<String>,
}
```

### 5. Generic State Pattern

```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct StateMachine<S, E, C>
where
    S: Encode + Decode<()> + Clone + Debug,
    E: Encode + Decode<()> + Clone + Debug,
    C: Encode + Decode<()> + Clone + Debug,
{
    #[key]
    pub machine_id: String,
    pub current_state: S,
    pub context: C,
    pub history: Vec<Transition<S, E>>,
}
```

## Troubleshooting

### Common Errors and Solutions

#### 1. Missing Trait Bounds

**Error**: `the trait bound 'T: bincode::Encode' is not satisfied`

**Solution**: Add the required trait bounds:
```rust
// Add missing bounds
pub struct MyStruct<T: Encode + Decode<()> + Clone + Debug> { ... }
```

#### 2. Circular Generic Dependencies

**Error**: Compilation hangs or complex error messages

**Solution**: Simplify generic relationships or use type aliases:
```rust
type SerializableData<T> = T where T: Encode + Decode<()> + Clone + Debug;
```

#### 3. Key Generation Issues

**Error**: `cannot determine key type for generic struct`

**Solution**: Ensure the `#[key]` field has a concrete, non-generic type:
```rust
#[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
pub struct MyStruct<T: Encode + Decode<()> + Clone + Debug> {
    #[key]
    pub id: String,  // ✅ Concrete type for key
    pub data: T,     // ✅ Generic data is fine
}
```

#### 4. Complex Lifetime Issues

**Error**: Complex lifetime error messages

**Solution**: Use owned types instead of borrowed data:
```rust
// ✅ Good: Use owned String
pub struct MyStruct<T: Encode + Decode<()> + Clone + Debug> {
    #[key]
    pub id: String,
    pub data: T,
}

// ❌ Problematic: Borrowed data
pub struct MyStruct<'a, T: Encode + Decode<()> + Clone + Debug> {
    #[key]
    pub id: &'a str,  // Lifetime complications
    pub data: T,
}
```

## API Reference

### Required Traits

All generic type parameters must implement:

```rust
trait NetabaseGeneric: bincode::Encode + bincode::Decode<()> + Clone + std::fmt::Debug {}
```

### Optional Traits (for additional functionality)

- `Send + Sync` - For thread safety
- `Hash + Eq` - For use as HashMap keys  
- `Default` - For default values
- `PartialOrd + Ord` - For ordering
- `Serialize + Deserialize` - For JSON compatibility (with serde)

### Macro Attributes

- `#[key]` - Marks the field used as the unique key
- `#[schema_module(RegistryName)]` - Creates a schema registry

### Generated Types

For each generic schema `MySchema<T>`, the following are generated:
- `MySchemaKey` - The key type for lookups
- Registry enum variants for network operations

### Network Operations

Generic types work with all standard Netabase operations:

```rust
// Put data
let result = netabase.put(key, generic_data).await?;

// Get data  
let retrieved: Option<MyGenericType<String>> = netabase.get(key).await?;

// Subscribe to changes
let mut stream = netabase.subscribe(key_pattern).await?;
```

## Examples in This Crate

This crate provides comprehensive examples:

1. **`examples/basic_generics.rs`** - Simple single-parameter generics
2. **`examples/advanced_generics.rs`** - Complex multi-parameter patterns  
3. **`examples/user_guide.rs`** - Step-by-step implementation guide
4. **`examples/network_integration.rs`** - Real network usage scenarios
5. **`src/lib.rs`** - Core examples with tests

### Running Examples

```bash
# Run basic examples
cargo run --example basic_generics

# Run advanced patterns  
cargo run --example advanced_generics

# Run network integration
cargo run --example network_integration

# Run all tests
cargo test
```

## Conclusion

NetabaseSchema's generic type support enables you to:

- Create flexible, reusable data structures
- Maintain type safety across the distributed network
- Handle complex nested and multi-parameter generics
- Build real-world applications with clean, maintainable code

The examples and patterns in this crate provide a solid foundation for implementing your own generic NetabaseSchema types. Start with the basic patterns and gradually adopt more advanced techniques as your needs evolve.

For more examples and detailed implementations, explore the example files in this crate. Each example is thoroughly tested and documented to help you understand the concepts and apply them to your own projects.