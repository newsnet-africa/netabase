# Netabase Store

[![Rust](https://img.shields.io/badge/rust-2024+-orange.svg)](https://www.rust-lang.org)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

**Netabase Store** is a type-safe, macro-driven database abstraction layer providing unified interfaces for multiple storage backends with first-class support for both native and WebAssembly environments.

## Features

### Core Capabilities
- **Type-Safe Models**: Automatic code generation using proc macros with compile-time guarantees
- **Primary & Secondary Keys**: Efficient indexing and querying with automatic key management
- **Multiple Storage Backends**:
  - **Sled** (native): High-performance embedded database
  - **IndexedDB** (WASM): Browser-native persistent storage
  - **Memory** (both): In-memory storage for testing and caching
- **Cross-Platform**: Single API works seamlessly across native and WASM targets
- **LibP2P Integration** (optional): Direct integration with Kademlia DHT for P2P applications

### Architecture
-  **Definition-Based**: Models are organized into type-safe enum-based schemas
- **Zero-Cost Abstractions**: Compile-time code generation eliminates runtime overhead
- **Trait-Driven**: Common traits enable generic programming across storage backends
- **Automatic Conversions**: Seamless conversion between models, keys, and storage formats

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# For native applications
netabase_store = { version = "*", features = ["native"] }
netabase_macros = "*"

# For WASM applications
netabase_store = { version = "*", features = ["wasm"], default-features = false }
netabase_macros = "*"

# For LibP2P integration
netabase_store = { version = "*", features = ["native", "libp2p"] }
```

### Feature Flags

- **`native`**: Enables Sled backend for native platforms (includes `sled` dependency)
- **`wasm`**: Enables IndexedDB backend for WebAssembly (includes `web-sys`, `js-sys`, etc.)
- **`libp2p`**: Enables integration with libp2p Kademlia DHT (for P2P applications)
- **`record-store`**: Additional record storage features for DHT operations

## Quick Start

### 1. Define Your Schema

Use the `netabase_definition_module` macro to automatically generate all necessary types and traits:

```rust
use netabase_macros::{NetabaseModel, netabase_definition_module};
use netabase_deps::{bincode, serde};

#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod blog {
    use super::*;

    /// User model with primary and secondary keys
    #[derive(NetabaseModel, Clone, Debug, PartialEq, Eq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
    }

    /// Post model with relational secondary key
    #[derive(NetabaseModel, Clone, Debug, PartialEq, Eq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
        #[secondary_key]
        pub published: bool,
    }
}

use blog::*;
```

**What the macro generates:**
- `BlogDefinition` enum: `BlogDefinition::User(User)` and `BlogDefinition::Post(Post)`
- `BlogKeys` enum: `BlogKeys::User(UserKey)` and `BlogKeys::Post(PostKey)`
- `UserKey` and `PostKey` enums with `Primary` and `Secondary` variants
- Primary key newtypes: `UserPrimaryKey(u64)`, `PostPrimaryKey(u64)`
- Secondary key enums: `UserSecondaryKeys`, `PostSecondaryKeys`
- All required trait implementations for database operations

### 2. Use with Native (Sled) Backend

```rust
use netabase_store::databases::sled_store::SledStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database
    let temp_dir = tempfile::tempdir()?;
    let store = SledStore::<BlogDefinition>::new(temp_dir.path())?;

    // Open typed tree for User models
    let user_tree = store.open_tree::<User>();

    // Create and insert a user
    let alice = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    user_tree.put(alice.clone())?;

    // Retrieve by primary key
    let retrieved = user_tree.get(UserPrimaryKey(1))?.unwrap();
    assert_eq!(retrieved, alice);

    // Query by secondary key (email)
    let users = user_tree.get_by_secondary_key(
        EmailSecondaryKey("alice@example.com".to_string())
    )?;
    assert_eq!(users.len(), 1);

    Ok(())
}
```

### 3. Use with WASM (IndexedDB) Backend

```rust
#[cfg(target_arch = "wasm32")]
use netabase_store::databases::indexeddb_store::IndexedDBStore;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn run_wasm_example() -> Result<JsValue, JsValue> {
    // Create IndexedDB store
    let store = IndexedDBStore::<BlogDefinition>::new("my_blog_db")
        .await
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

    // Same API as Sled!
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    user_tree.put(alice.clone())
        .await
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

    Ok(JsValue::from_str("Success!"))
}
```

## Architecture Overview

### Type Hierarchy

```
BlogDefinition (enum)
├── User(User)
└── Post(Post)

BlogKeys (enum)
├── User(UserKey)
└── Post(PostKey)

UserKey (enum)
├── Primary(UserPrimaryKey)
└── Secondary(UserSecondaryKeys)

UserSecondaryKeys (enum)
└── Email(EmailSecondaryKey)
```

### Storage Backends

| Backend | Platform | Use Case | Performance |
|---------|----------|----------|-------------|
| **Sled** | Native | Production embedded DB | Excellent |
| **IndexedDB** | WASM | Browser applications | Good |
| **Memory** | Both | Testing/caching | Excellent |

### Trait System

All storage backends implement these core traits:

- **`Store<D>`**: Store-level operations (open trees, discriminants)
- **`StoreTree<M>`**: Tree-level CRUD operations
- **`NetabaseModel`**: Model trait with key extraction
- **`NetabaseDefinition`**: Definition enum trait for schema organization
- **`NetabaseDefinitionKey`**: Keys enum trait for type-safe operations

### LibP2P Integration

When the `libp2p` feature is enabled, additional traits are available:

- **`KademliaRecord`**: Convert definitions to/from libp2p `Record`
- **`KademliaRecordKey`**: Convert keys to/from libp2p `RecordKey`
- **`RecordStore`**: Full libp2p RecordStore trait implementation

```rust
#[cfg(feature = "libp2p")]
use libp2p::kad::store::RecordStore;
use netabase_store::traits::dht::KademliaRecord;

// Convert model to DHT record
let user_def = BlogDefinition::User(alice);
let record = user_def.try_to_record()?;

// Store in Kademlia
RecordStore::put(&mut store, record)?;

// Retrieve from Kademlia
let retrieved = RecordStore::get(&store, &record.key);
```

## Examples

See the `examples/` directory for comprehensive demonstrations:

- **`basic_store.rs`**: LibP2P Record conversion and DHT integration

Run native examples:
```bash
cargo run --example basic_store --features "native,libp2p"
```

Build WASM examples:
```bash
wasm-pack build --features wasm --no-default-features
```

## Testing

### Native Tests
```bash
# Run all tests
cargo test --features native

# Run specific test suite
cargo test --features native --test sled_store_tests
```

### WASM Tests
```bash
# Install wasm-pack
cargo install wasm-pack

# Run WASM tests in headless browser
wasm-pack test --headless --firefox --features wasm

# Or with Chrome
wasm-pack test --headless --chrome --features wasm
```

## API Documentation

### Key Traits

#### `NetabaseModel`
```rust
pub trait NetabaseModel {
    type PrimaryKey: NetabaseModelKey;
    type SecondaryKeys: NetabaseModelKey;
    type Keys: NetabaseModelKey;

    fn primary_key(&self) -> Self::PrimaryKey;
    fn secondary_keys(&self) -> Vec<Self::SecondaryKeys>;
    fn discriminant_name() -> &'static str;
}
```

#### `Store<D>`
```rust
pub trait Store<D: NetabaseDefinition> {
    fn open_tree<M: NetabaseModel>(&self) -> impl StoreTree<M>;
    fn active_discriminants(&self) -> Vec<D::Discriminants>;
}
```

#### `StoreTree<M>`
```rust
pub trait StoreTree<M: NetabaseModel> {
    fn put(&self, model: M) -> Result<()>;
    fn get(&self, key: M::PrimaryKey) -> Result<Option<M>>;
    fn remove(&self, key: M::PrimaryKey) -> Result<bool>;
    fn get_by_secondary_key(&self, key: M::SecondaryKeys) -> Result<Vec<M>>;
    fn iter(&self) -> impl Iterator<Item = Result<(M::PrimaryKey, M)>>;
    // ... more methods
}
```

## Performance

### Benchmarks

See `benches/` for comprehensive performance tests:

```bash
cargo bench --features native
```

Expected performance (native/Sled):
- **Put operations**: ~50-100μs per insert
- **Get operations**: ~10-20μs per lookup
- **Secondary key queries**: O(n) where n = matching records
- **Iteration**: ~5-10μs per record

## Advanced Features

### Custom Serialization

While `bincode` is the default, you can implement custom serialization:

```rust
// Custom conversion trait
impl ToIVec for MyType {
    fn to_ivec(&self) -> Result<sled::IVec> {
        // Custom serialization logic
    }
}
```

### Batch Operations

```rust
// Batch inserts for better performance
let users = vec![user1, user2, user3];
for user in users {
    user_tree.put(user)?;
}
user_tree.flush()?; // Ensure persistence
```

### Atomic Operations

```rust
// Compare-and-swap for atomic updates
user_tree.compare_and_swap(
    key,
    Some(old_value),
    Some(new_value)
)?;
```

## Migration Guide

### From Old Macro System

If you're migrating from the older `netabase_schema_module` macro:

**Old:**
```rust
#[netabase_schema_module(BlogSchema, BlogKeys)]
mod blog { ... }
```

**New:**
```rust
#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod blog { ... }
```

Key changes:
- Schema enums are now called "Definitions"
- Automatic generation of all key types and variants
- Simplified trait requirements

## Troubleshooting

### Common Issues

**"Type does not implement NetabaseModel"**
- Ensure you've derived `NetabaseModel` on your struct
- Check that `bincode::Encode` and `bincode::Decode` are derived
- Verify `serde` traits are derived if using `libp2p` feature

**"Failed to open database"**
- Check file permissions on the database directory
- Ensure the path exists and is writable
- For WASM, verify IndexedDB is supported in the browser

**"Secondary key query returns empty"**
- Verify the secondary key field is marked with `#[secondary_key]`
- Ensure data was inserted after the index was created
- Check that the query key matches the field type exactly

## Contributing

Contributions are welcome! Please:

1. Add tests for new features
2. Update documentation
3. Follow Rust naming conventions
4. Run `cargo fmt` and `cargo clippy`

## License

This project is licensed under the GNU GPL v3 License - see the [LICENSE](../LICENSE) file for details.

## Acknowledgments

- [Sled](https://github.com/spacejam/sled) - Embedded database engine
- [web-sys](https://rustwasm.github.io/wasm-bindgen/web-sys/index.html) - Web API bindings for Rust
- [libp2p](https://libp2p.io/) - Modular peer-to-peer networking stack
- [bincode](https://github.com/bincode-org/bincode) - Binary serialization
