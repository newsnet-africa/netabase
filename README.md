# Netabase

A distributed peer-to-peer database system built on libp2p, enabling decentralized storage and retrieval of structured data with automatic schema derivation and network replication.

## Overview

Netabase provides a simple yet powerful interface for creating distributed applications with built-in data persistence, peer discovery, and automatic replication. It combines the power of libp2p networking with Rust's type system to create a type-safe, schema-aware distributed database.

## Features

- 🌐 **Peer-to-Peer Networking**: Built on libp2p for robust, decentralized communication
- 🔄 **Automatic Replication**: Data is automatically replicated across network peers using Kademlia DHT
- 🛡️ **Type Safety**: Compile-time schema validation with derive macros
- 🔍 **Content Addressing**: Uses cryptographic hashes for data integrity
- ⚡ **Async/Await**: Built with Tokio for high-performance async operations
- 🔐 **Secure by Default**: Built-in encryption and peer authentication
- 📊 **Flexible Storage**: Supports any serializable Rust data structure
- 🎯 **Simple Configuration**: Automatic peer discovery with minimal setup

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
netabase = "0.1.0"
netabase_macros = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
bincode = "2.0.0-rc.3"
libp2p = { version = "0.53", features = ["kad"] }
anyhow = "1.0"
```

## Quick Start

### Basic Example

```rust
use netabase::{Netabase, NetabaseConfig, NetabaseSchema};
use serde::{Deserialize, Serialize};
use bincode::{Encode, Decode};
use libp2p::identity::ed25519::Keypair;

// Define your data structure
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct User {
    #[key]
    id: u64,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    netabase::init_logging();

    // Create configuration
    let config = NetabaseConfig::default();

    // Generate or load keypair
    let keypair = Keypair::generate();

    // Create netabase instance
    let mut netabase = Netabase::try_new(config, &keypair, "my-app")?;

    // Start the network
    netabase.start_swarm()?;

    // Wait a moment for network initialization
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Create and store data
    let user = User {
        id: 42,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Put data into the network
    let put_result = netabase.put(
        user.clone(),
        None,
        libp2p::kad::Quorum::One
    ).await?;
    println!("Data stored: {:?}", put_result);

    // Retrieve data by key
    let get_result = netabase.get(user.key()).await?;
    println!("Retrieved: {:?}", get_result);

    Ok(())
}
```

### Working with Different Key Types

Netabase currently supports single-field keys with the following types:

```rust
use netabase::NetabaseSchema;
use serde::{Deserialize, Serialize};
use bincode::{Encode, Decode};

// String keys
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, Encode, Decode)]
struct Document {
    #[key]
    doc_id: String,
    title: String,
    content: String,
}

// Integer keys (u8, u16, u32, u64, i8, i16, i32, i64)
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, Encode, Decode)]
struct Product {
    #[key]
    sku: u32,
    name: String,
    price: f64,
}

// Boolean keys
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, Encode, Decode)]
struct Setting {
    #[key]
    enabled: bool,
    config_value: String,
}
```

## Schema Definition with Macros

### NetabaseSchema Derive Macro

The `NetabaseSchema` derive macro automatically implements the required traits for your data structures:

```rust
use netabase::NetabaseSchema;
use serde::{Deserialize, Serialize};
use bincode::{Encode, Decode};

#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, Encode, Decode)]
struct Message {
    #[key]           // Marks this field as the primary key
    id: String,
    sender: String,
    recipient: String,
    content: String,
    timestamp: u64,
}
```

**Required Derives:**
- `Serialize, Deserialize` - For data serialization
- `NetabaseSchema` - Implements netabase traits
- `Clone` - Required by netabase traits
- `Encode, Decode` - For bincode serialization (version 2.x)

**Key Field Requirements:**
- Must be marked with `#[key]` attribute
- Currently supports only **single field keys**
- Supported key types: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `String`, `bool`

### Schema Module Macro

The `schema` attribute macro can be applied to modules to register and validate multiple schemas:

```rust
use netabase::{NetabaseSchema, schema};
use serde::{Deserialize, Serialize};
use bincode::{Encode, Decode};

#[schema]
mod my_schemas {
    use super::*;

    #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, Encode, Decode)]
    struct User {
        #[key]
        id: u64,
        name: String,
    }

    #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, Encode, Decode)]
    struct Post {
        #[key]
        post_id: String,
        author_id: u64,
        content: String,
    }
}
```

### What the Macros Generate

The `NetabaseSchema` derive macro generates:

1. **Key Type**: A strongly-typed key struct (e.g., `UserKey` for `User`)
2. **Trait Implementations**:
   - `NetabaseSchema` - Provides `key()` method and conversion traits
   - `NetabaseSchemaKey` - For the generated key type
3. **Conversion Traits**:
   - `From<libp2p::kad::Record>` and `Into<libp2p::kad::Record>`
   - `From<libp2p::kad::RecordKey>` and `Into<libp2p::kad::RecordKey>` for keys
4. **Serialization Support**:
   - `bincode::Encode`, `bincode::Decode`, and `bincode::BorrowDecode`
   - `std::fmt::Display` for keys

Example of what gets generated for a `User` struct:

```rust
// Original struct
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, Encode, Decode)]
struct User {
    #[key]
    id: u64,
    name: String,
}

// Generated key type
pub struct UserKey(u64);

// Generated trait implementations
impl NetabaseSchema for User {
    type Key = UserKey;
    fn key(&self) -> Self::Key {
        UserKey(self.id)
    }
}

impl NetabaseSchemaKey for UserKey {}

// Plus conversion traits, serialization, etc.
```

## Configuration

### NetabaseConfig

```rust
use netabase::config::NetabaseConfig;

let config = NetabaseConfig {
    storage_path: "./my_netabase_data".into(),
    ..Default::default()
};
```

### Environment Variables

Netabase supports configuration via environment variables:

```bash
export NETABASE_STORAGE_PATH="./data"
export RUST_LOG="netabase=debug,libp2p=info"
```

## API Reference

### Core Types

#### `Netabase`
The main database instance.

**Methods:**
```rust
// Create a new instance
pub fn try_new(
    config: NetabaseConfig,
    keypair: &Keypair,
    protocol_name: impl ToString
) -> anyhow::Result<Self>

// Start the network swarm
pub fn start_swarm(&mut self) -> anyhow::Result<()>

// Store data in the network
pub async fn put<T: NetabaseSchema>(
    &mut self,
    value: T,
    put_to: Option<impl ExactSizeIterator<Item = PeerId>>,
    quorum: libp2p::kad::Quorum,
) -> anyhow::Result<libp2p::kad::PutRecordOk>

// Retrieve data by key
pub async fn get<K: NetabaseSchemaKey>(
    &mut self,
    key: K
) -> anyhow::Result<libp2p::kad::GetRecordOk>
```

#### `NetabaseConfig`
Configuration for a Netabase instance.

```rust
pub struct NetabaseConfig {
    pub storage_path: PathBuf,
}

impl Default for NetabaseConfig {
    fn default() -> Self {
        Self {
            storage_path: "./netabase_data".into(),
        }
    }
}
```

### Traits

#### `NetabaseSchema`
Automatically implemented via derive macro. Provides:
- `type Key: NetabaseSchemaKey` - Associated key type
- `fn key(&self) -> Self::Key` - Extract the key from the data
- Conversion to/from `libp2p::kad::Record`

#### `NetabaseSchemaKey`
For types that can be used as lookup keys. Provides:
- Conversion to/from `libp2p::kad::RecordKey`
- Serialization support

## Network Protocol

### DHT-Based Storage
Netabase uses Kademlia DHT for distributed storage:
- Content-addressed keys derived from data
- Automatic peer discovery via mDNS
- Replication based on DHT topology
- Eventual consistency model

### Transport Layers
Supports multiple transport protocols:
- TCP with TLS encryption
- QUIC for low-latency connections
- Noise protocol for additional security

### Security
- Ed25519 keypairs for peer identity
- TLS 1.3 for transport encryption
- Content integrity via DHT verification

## Examples and Testing

### Running Examples

```bash
# Basic schema validation
cargo run --example validate_docs_basic

# Test supported key types
cargo run --example validate_supported_keys

# Simple schema test
cargo run --example simple_schema_test
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test distributed_tests

# Macro tests
cd netabase_macros && cargo test
```

### Distributed Testing

Test with multiple nodes:

```bash
# Terminal 1 (Node A)
NETABASE_NODE=A NETABASE_PORT=0 cargo test distributed_two_nodes_local -- --nocapture

# Terminal 2 (Node B)
NETABASE_NODE=B NETABASE_PORT=0 NETABASE_BOOTSTRAP=/ip4/127.0.0.1/tcp/<PORT>/p2p/<PEER_ID> cargo test distributed_two_nodes_local -- --nocapture
```

## Current Limitations

**⚠️ Important**: Netabase is in active development. Current limitations include:

### Schema Limitations
- ❌ **Composite keys not supported** - Only single field keys work
- ❌ **Custom key functions not implemented** - The `#[key_fn]` attribute is planned but not working
- ❌ **Schema prefixes not functional** - The `#[schema(prefix = "...")]` attribute exists but doesn't affect behavior
- ❌ **No enum support** - Only structs can be schemas currently

### Supported Key Types
✅ **Currently Working:**
- `u8`, `u16`, `u32`, `u64`
- `i8`, `i16`, `i32`, `i64`
- `String`
- `bool`

❌ **Not Yet Supported:**
- Composite keys (multiple `#[key]` fields)
- Custom types as keys
- `Vec<T>`, `Option<T>`, or other generic types as keys
- Nested structs as keys

### Network Limitations
- Basic peer discovery (mDNS only)
- No data versioning or conflict resolution
- No transaction support
- No data expiration/TTL

## Troubleshooting

### Common Issues

**Compilation Errors with Macros:**
```rust
// ❌ This won't work (composite keys)
#[derive(NetabaseSchema, ...)]
struct BadExample {
    #[key]
    user_id: u64,
    #[key]          // Second key field - not supported
    session_id: String,
}

// ✅ This works (single key)
#[derive(NetabaseSchema, ...)]
struct GoodExample {
    #[key]
    id: String,
    user_id: u64,    // Non-key field
    session_id: String,  // Non-key field
}
```

**Serialization Issues:**
- Ensure you're using `bincode = "2.0.0-rc.3"` (not 1.x)
- All schema fields must implement `Serialize` and `Deserialize`
- Include `Encode` and `Decode` derives for bincode 2.x

**Network Issues:**
- Call `start_swarm()` before any `put`/`get` operations
- Wait briefly after `start_swarm()` for network initialization
- Check that `RUST_LOG=netabase=debug` for debugging

## Development Roadmap

### Next Milestones

**v0.2.0 - Enhanced Schema Support**
- [ ] Composite key support (multiple `#[key]` fields)
- [ ] Custom key generation functions (`#[key_fn]`)
- [ ] Schema prefixes and versioning
- [ ] Enum schema support
- [ ] Functional database functionality

**v0.3.0 - Advanced Features**
- [ ] Data versioning and conflict resolution
- [ ] Range queries and indexing
- [ ] Pub/sub messaging layer
- [ ] Transaction support

**v0.4.0 - Production Readiness**
- [ ] Data expiration and TTL
- [ ] Backup and restore
- [ ] Performance optimizations
- [ ] Security audit

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/new-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Run `cargo fmt` and `cargo clippy`
7. Submit a pull request

### Development Setup

```bash
git clone https://github.com/your-org/netabase.git
cd netabase
cargo build
cargo test

# Test macros specifically
cd netabase_macros
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [libp2p](https://libp2p.io/) for peer-to-peer networking primitives
- [Tokio](https://tokio.rs/) for async runtime
- [Serde](https://serde.rs/) for serialization
- [Kademlia](https://en.wikipedia.org/wiki/Kademlia) DHT algorithm

---

**⚠️ Development Status**: Netabase is currently in alpha (v0.1.0). The API is stabilizing but may still change. Not recommended for production use yet.

For the most up-to-date examples and documentation, see the `examples/` directory and tests in the repository.
