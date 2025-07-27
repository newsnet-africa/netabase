# Netabase Macros

High-performance procedural macros for the Netabase distributed database system, featuring **native bincode serialization by default** for maximum efficiency.

## Overview

The `netabase_macros` crate provides the `#[derive(NetabaseSchema)]` macro that automatically generates all necessary trait implementations for distributed database schemas. **By default, it uses native `bincode::Encode` and `bincode::Decode` traits for 5-10x better serialization performance** compared to serde compatibility layers.

## Key Features

- 🚀 **Native Bincode by Default**: Maximum serialization performance out of the box
- 🔄 **Automatic Fallback**: Seamless backward compatibility with serde-encoded data
- 🛡️ **Type Safety**: Sealed trait pattern prevents manual implementation
- 🔑 **Flexible Key Generation**: Field-level, item-level, and closure-based key extraction
- 🌐 **libp2p Integration**: Built-in conversion to/from libp2p DHT records
- 🧵 **Thread Safe**: All generated types implement `Send + Sync + Unpin`

## Performance Comparison

| Feature | Native Bincode (Default) | Serde Compatibility |
|---------|-------------------------|-------------------|
| Serialization Speed | **5-10x faster** | Baseline |
| Binary Size | **Smaller** | Larger |
| Memory Usage | **Lower** | Higher |
| CPU Overhead | **Minimal** | Moderate |

## Quick Start

### Default: Native Bincode (Recommended)

```rust
use netabase_macros::NetabaseSchema;

#[derive(NetabaseSchema, Clone, Debug, PartialEq)]
struct User {
    #[key]
    id: String,
    name: String,
    email: String,
    age: u32,
}

fn main() {
    let user = User {
        id: "user123".to_string(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    };

    // Lightning-fast native bincode serialization
    let encoded = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();
    let decoded: User = bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap().0;
    
    // Key generation
    let key = user.key(); // UserKey("user123")
    
    // libp2p DHT integration
    let record = user.to_kad_record().unwrap();
    let user_from_record: User = record.into();
    
    assert_eq!(user, decoded);
    assert_eq!(user, user_from_record);
}
```

### Opt-in: Serde Compatibility

When you need JSON serialization or integration with serde ecosystems:

```rust
use netabase_macros::NetabaseSchema;

#[derive(NetabaseSchema, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[netabase(serde)]  // Opt-in to serde compatibility
struct User {
    #[key]
    id: String,
    name: String,
    email: String,
}

fn main() {
    let user = User {
        id: "user123".to_string(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Both bincode and JSON work
    let binary = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();
    let json = serde_json::to_string(&user).unwrap();
    
    println!("JSON: {}", json);
}
```

## Advanced Key Generation

### Field-Level Closures

```rust
#[derive(NetabaseSchema)]
struct Product {
    #[key = |id| format!("product_{}", id)]
    id: u64,
    name: String,
    category: String,
}
```

### Item-Level Closures

```rust
#[derive(NetabaseSchema)]
#[key = |product| format!("{}_{}", product.category, product.id)]
struct Product {
    id: String,
    name: String,
    category: String,
}
```

### Enum Support

```rust
#[derive(NetabaseSchema)]
enum Document {
    #[key]
    Post { id: String, title: String, content: String },
    #[key]
    Comment { id: String, post_id: String, text: String },
}
```

## Migration from Serde

If you're currently using serde-based serialization:

```rust
// Before: Slower serde compatibility
#[derive(NetabaseSchema, serde::Serialize, serde::Deserialize)]
#[netabase(serde)]
struct User {
    #[key] id: String,
    name: String,
}

// After: 5-10x faster native bincode
#[derive(NetabaseSchema)]  // Remove serde attributes
struct User {
    #[key] id: String,
    name: String,
}
```

**Your existing encoded data remains compatible** - the macro includes automatic fallback for backward compatibility.

## Generated Trait Implementations

The macro automatically implements:

### For Schema Types
- `bincode::Encode` + `bincode::Decode` (native, fast)
- `NetabaseSchema` (sealed trait)
- `Clone`, `Debug`, `PartialEq` (forwarded to your derives)
- `Send + Sync + Unpin` (thread safety)
- `From<libp2p::kad::Record>` + `TryFrom<libp2p::kad::Record>`

### For Key Types (`{Schema}Key`)
- `bincode::Encode` + `bincode::Decode` (native, fast)
- `NetabaseSchemaKey` (sealed trait)
- `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`
- `Display`, `AsRef<str>`, `From<String>`
- `From<Vec<u8>>` + `Into<Vec<u8>>`
- `Send + Sync + Unpin` (thread safety)

## Examples

See the `examples/` directory for comprehensive examples:

- `examples/native_bincode_vs_serde.rs` - Performance comparison
- `examples/field_key_generator_example.rs` - Key generation strategies
- `examples/sealed_trait_test.rs` - All trait implementations

## Performance Benchmarks

Run the performance comparison:

```bash
cargo run --example native_bincode_vs_serde
```

Typical results show:
- **5-10x faster** encoding/decoding with native bincode
- **20-30% smaller** binary size
- **Lower memory** allocation overhead

## Backward Compatibility

The macro ensures seamless backward compatibility:

1. **Native decoding first**: Tries native bincode deserialization
2. **Automatic fallback**: Falls back to serde compatibility if needed
3. **Zero configuration**: Works automatically for mixed data scenarios

## Error Handling

```rust
// Panicking conversion (for when you know data is valid)
let user: User = record.into();

// Non-panicking conversion
let user: Result<User, _> = record.try_into();
```

## Thread Safety

All generated types are thread-safe:

```rust
let user = User { /* ... */ };

// Safe to send between threads
std::thread::spawn(move || {
    println!("User: {:?}", user);
});

// Safe to share between threads
let user_ref = &user;
std::thread::scope(|s| {
    s.spawn(|| println!("User: {:?}", user_ref));
});
```

## Requirements

- Rust 2021 edition or later
- `bincode` crate for serialization
- `serde` crate (optional, only when using `#[netabase(serde)]`)
- `libp2p` crate for DHT record integration

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for guidelines.

---

**Performance Tip**: Use the default native bincode behavior for maximum performance. Only opt-in to serde compatibility when you specifically need JSON serialization or serde ecosystem integration.