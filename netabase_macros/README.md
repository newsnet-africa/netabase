# Netabase Macros

Procedural macros for the Netabase distributed database system. This crate provides derive macros and attribute macros that automatically implement the required traits for distributed data storage and retrieval.

## Overview

The netabase_macros crate is a crucial component of the Netabase ecosystem, providing compile-time code generation for:
- Schema definition and validation
- Key generation for distributed hash table (DHT) storage
- Serialization and deserialization of data structures
- Type-safe database operations

## Features

- 🏗️ **Derive Macros**: Automatic trait implementation for data structures
- 🔑 **Key Generation**: Automatic content-addressed key generation
- 📝 **Schema Validation**: Compile-time schema validation and type checking
- 🎯 **Attribute Macros**: Fine-grained control over field behavior
- 🔧 **Customization**: Flexible configuration options for different use cases
- ⚡ **Zero Runtime Cost**: All code generation happens at compile time

## Installation

This crate is automatically included when you use the main `netabase` crate:

```toml
[dependencies]
netabase = "0.1.0"
```

For standalone use:

```toml
[dependencies]
netabase_macros = "0.1.0"
```

## Quick Start

### Basic Schema Definition

```rust
use netabase_macros::NetabaseSchema;
use serde::{Serialize, Deserialize};
use bincode::{Encode, Decode};

#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, Encode, Decode)]
struct User {
    #[key]
    id: u64,
    name: String,
    email: String,
}
```

### Advanced Schema with Configuration

```rust
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, Encode, Decode)]
#[schema(prefix = "user", version = "v1")]
struct User {
    #[key]
    id: u64,
    
    #[schema(index)]
    email: String,
    
    #[schema(optional)]
    profile_picture: Option<String>,
    
    name: String,
    created_at: u64,
}
```

### Composite Keys

```rust
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, Encode, Decode)]
#[schema(prefix = "session")]
struct UserSession {
    #[key]
    user_id: u64,
    
    #[key]
    session_id: String,
    
    expires_at: u64,
    data: serde_json::Value,
}
```

## Available Macros

### `#[derive(NetabaseSchema)]`

The main derive macro that implements the `NetabaseSchema` trait for your data structures.

**Generated Traits:**
- `NetabaseSchema` - Core schema trait with key generation
- `NetabaseSchemaKey` - For types that can be used as lookup keys

**Example:**
```rust
#[derive(NetabaseSchema)]
struct Document {
    #[key]
    id: String,
    title: String,
    content: String,
}

// Generated methods:
// - document.key() -> String
// - Document::from_key(key: String) -> Self
// - document.into() -> libp2p::kad::Record
```

### `#[schema(...)]` Attribute

Container-level attributes that configure schema behavior.

**Available Options:**

#### `prefix = "string"`
Sets a prefix for generated keys to avoid collisions.

```rust
#[derive(NetabaseSchema)]
#[schema(prefix = "doc")]
struct Document {
    #[key]
    id: String,
    // Key will be "doc::{id}"
}
```

#### `version = "string"`
Adds version information to the schema for migration support.

```rust
#[derive(NetabaseSchema)]
#[schema(version = "v2")]
struct User {
    #[key]
    id: u64,
    // Enables future schema migration from v1 to v2
}
```

#### `separator = "string"`
Customizes the separator used in composite keys (default: "::").

```rust
#[derive(NetabaseSchema)]
#[schema(separator = "|")]
struct CompoundKey {
    #[key]
    part1: String,
    #[key]
    part2: String,
    // Key will be "part1|part2"
}
```

### `#[key]` Field Attribute

Marks fields as part of the primary key for the record.

**Features:**
- Single key fields create simple string keys
- Multiple key fields create composite keys
- Supports any type that implements `Display`

```rust
#[derive(NetabaseSchema)]
struct Example {
    #[key]
    primary_id: u64,
    
    #[key]
    secondary_id: String,
    
    data: String,
    // Key will be "primary_id::secondary_id"
}
```

### `#[schema(...)]` Field Attributes

Field-level attributes for additional functionality.

#### `index`
Marks a field for indexing (future feature).

```rust
#[derive(NetabaseSchema)]
struct User {
    #[key]
    id: u64,
    
    #[schema(index)]
    email: String, // Will be indexed for fast lookups
}
```

#### `optional`
Explicitly marks optional fields for documentation.

```rust
#[derive(NetabaseSchema)]
struct Profile {
    #[key]
    user_id: u64,
    
    #[schema(optional)]
    bio: Option<String>,
}
```

#### `transform = "function"`
Applies a transformation function to the field before key generation.

```rust
#[derive(NetabaseSchema)]
struct CaseInsensitiveUser {
    #[key]
    #[schema(transform = "str::to_lowercase")]
    username: String, // Key will use lowercase version
}
```

## Generated Code Examples

### Simple Schema

Input:
```rust
#[derive(NetabaseSchema)]
struct User {
    #[key]
    id: u64,
    name: String,
}
```

Generated (conceptual):
```rust
impl NetabaseSchema for User {
    fn key(&self) -> String {
        self.id.to_string()
    }
    
    fn schema_name() -> &'static str {
        "User"
    }
}

impl NetabaseSchemaKey for u64 {
    fn into_key(self) -> String {
        self.to_string()
    }
}

impl From<User> for libp2p::kad::Record {
    fn from(value: User) -> Self {
        let key = value.key();
        let data = bincode::serialize(&value).expect("Serialization failed");
        libp2p::kad::Record::new(key.into_bytes(), data)
    }
}
```

### Complex Schema with Prefix

Input:
```rust
#[derive(NetabaseSchema)]
#[schema(prefix = "msg", version = "v1")]
struct Message {
    #[key]
    chat_id: String,
    #[key]
    message_id: u64,
    content: String,
}
```

Generated (conceptual):
```rust
impl NetabaseSchema for Message {
    fn key(&self) -> String {
        format!("msg::{}::{}", self.chat_id, self.message_id)
    }
    
    fn schema_name() -> &'static str {
        "Message"
    }
    
    fn schema_version() -> &'static str {
        "v1"
    }
}
```

## Error Handling

The macros provide comprehensive compile-time error checking:

### Missing Key Field
```rust
#[derive(NetabaseSchema)]
struct BadSchema {
    // Error: At least one field must be marked with #[key]
    id: u64,
    name: String,
}
```

### Invalid Key Type
```rust
#[derive(NetabaseSchema)]
struct BadKeyType {
    #[key]
    // Error: Key fields must implement Display
    complex_key: Vec<String>,
}
```

### Conflicting Attributes
```rust
#[derive(NetabaseSchema)]
#[schema(separator = "::", separator = "||")]
// Error: Duplicate attribute 'separator'
struct ConflictingAttrs {
    #[key]
    id: u64,
}
```

## Advanced Usage

### Custom Key Generation

For complex key generation logic, implement the trait manually:

```rust
use netabase::{NetabaseSchema, NetabaseSchemaKey};

struct CustomKeyStruct {
    timestamp: u64,
    user_id: u64,
    action: String,
}

impl NetabaseSchema for CustomKeyStruct {
    fn key(&self) -> String {
        // Custom logic: bucket by hour, then user, then action
        let hour_bucket = self.timestamp / 3600;
        format!("action::{}::{}::{}", hour_bucket, self.user_id, self.action)
    }
    
    fn schema_name() -> &'static str {
        "CustomKeyStruct"
    }
}
```

### Schema Versioning

```rust
#[derive(NetabaseSchema)]
#[schema(version = "v1")]
struct UserV1 {
    #[key]
    id: u64,
    name: String,
}

#[derive(NetabaseSchema)]
#[schema(version = "v2")]
struct UserV2 {
    #[key]
    id: u64,
    first_name: String,
    last_name: String,
    
    #[schema(optional)]
    email: Option<String>,
}

// Migration logic (manual implementation)
impl From<UserV1> for UserV2 {
    fn from(v1: UserV1) -> Self {
        let parts: Vec<&str> = v1.name.split_whitespace().collect();
        UserV2 {
            id: v1.id,
            first_name: parts.get(0).unwrap_or(&"").to_string(),
            last_name: parts.get(1).unwrap_or(&"").to_string(),
            email: None,
        }
    }
}
```

## Testing

### Unit Tests for Generated Code

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let user = User {
            id: 42,
            name: "Alice".to_string(),
        };
        
        assert_eq!(user.key(), "42");
    }
    
    #[test]
    fn test_composite_key() {
        let session = UserSession {
            user_id: 123,
            session_id: "abc123".to_string(),
            expires_at: 1234567890,
            data: serde_json::Value::Null,
        };
        
        assert_eq!(session.key(), "123::abc123");
    }
    
    #[test]
    fn test_serialization_roundtrip() {
        let original = User {
            id: 42,
            name: "Bob".to_string(),
        };
        
        let record: libp2p::kad::Record = original.clone().into();
        let key_bytes = record.key.to_vec();
        let data = record.value;
        
        assert_eq!(String::from_utf8(key_bytes).unwrap(), original.key());
        
        let deserialized: User = bincode::deserialize(&data).unwrap();
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.name, deserialized.name);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use netabase::{Netabase, NetabaseConfig};
    
    #[tokio::test]
    async fn test_netabase_integration() {
        let config = NetabaseConfig::default();
        let keypair = libp2p::identity::ed25519::Keypair::generate();
        let mut netabase = Netabase::try_new(config, &keypair, "test").unwrap();
        netabase.start_swarm().unwrap();
        
        let user = User {
            id: 123,
            name: "Test User".to_string(),
        };
        
        // Put operation
        netabase.put(user.clone(), None, libp2p::kad::Quorum::One).await.unwrap();
        
        // Get operation
        let result = netabase.get(user.id).await.unwrap();
        // Verify result contains our data
    }
}
```

## Debugging

### Expanding Macros

Use `cargo expand` to see the generated code:

```bash
cargo install cargo-expand
cargo expand --bin your_binary
```

### Debug Attributes

Add debug printing to see what the macro generates:

```rust
#[derive(NetabaseSchema)]
#[schema(debug)]  // Prints generated code during compilation
struct DebugSchema {
    #[key]
    id: u64,
}
```

## TODO

### High Priority
- [ ] Add support for automatic indexing of fields marked with `#[schema(index)]`
- [ ] Implement schema migration helpers for version transitions
- [ ] Add validation macros for field constraints
- [ ] Create proc macro for automatic relationship definitions
- [ ] Add support for custom serialization formats

### Medium Priority
- [ ] Implement query builder macros for complex lookups
- [ ] Add support for computed fields in key generation
- [ ] Create macro for automatic timestamp fields
- [ ] Add support for field encryption markers
- [ ] Implement automatic schema documentation generation

### Low Priority
- [ ] Add support for cross-schema relationships
- [ ] Create macros for data transformation pipelines
- [ ] Implement automatic caching hints
- [ ] Add support for custom key hashing algorithms
- [ ] Create performance optimization hints

### Documentation
- [ ] Add comprehensive examples for all macro features
- [ ] Create migration guide from manual implementations
- [ ] Add best practices guide for schema design
- [ ] Create troubleshooting guide for common errors
- [ ] Add performance optimization documentation

### Testing
- [ ] Add comprehensive property-based tests for key generation
- [ ] Create stress tests for complex schema hierarchies
- [ ] Add regression tests for schema compatibility
- [ ] Implement fuzzing tests for macro parsing
- [ ] Add benchmarks for generated code performance

## Implementation Details

### Dependencies

- `proc-macro2` - Token manipulation
- `quote` - Code generation
- `syn` - Rust syntax parsing
- `darling` - Attribute parsing helper

### Architecture

```
src/
├── lib.rs              # Proc macro entry points
├── derive/             # Derive macro implementations
│   ├── mod.rs
│   ├── netabase_schema.rs  # NetabaseSchema derive macro
│   └── validation.rs   # Input validation
├── attrs/              # Attribute parsing
│   ├── mod.rs
│   ├── container.rs    # Container-level attributes
│   └── field.rs        # Field-level attributes
├── codegen/            # Code generation
│   ├── mod.rs
│   ├── key_generation.rs   # Key generation logic
│   ├── trait_impl.rs   # Trait implementations
│   └── record_conversion.rs # Record conversion code
└── utils/              # Utility functions
    ├── mod.rs
    ├── errors.rs       # Error handling
    └── helpers.rs      # Code generation helpers
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/new-macro`)
3. Add tests for new functionality
4. Ensure all tests pass (`cargo test`)
5. Add documentation for new features
6. Submit a pull request

### Development Guidelines

- Follow Rust macro best practices
- Provide clear error messages for invalid input
- Add comprehensive tests for new functionality
- Document all public APIs and examples
- Ensure backward compatibility when possible

## License

This crate is part of the Netabase project and is licensed under the MIT License.

## References

- [The Rust Programming Language - Macros](https://doc.rust-lang.org/book/ch19-06-macros.html)
- [Procedural Macros Workshop](https://github.com/dtolnay/proc-macro-workshop)
- [syn Documentation](https://docs.rs/syn/)
- [quote Documentation](https://docs.rs/quote/)