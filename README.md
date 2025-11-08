![crates.io](https://img.shields.io/crates/v/netabase.svg)
![crates.io downloads](https://img.shields.io/crates/d/netabase.svg) ![docs.rs](https://docs.rs/netabase/badge.svg)

# Netabase

A peer-to-peer networking layer built on libp2p with integrated type-safe storage, enabling distributed applications with automatic data synchronization across native and WASM environments.

# This crate is still in early development and will change frequently as it stabalises. It is not advised to use this in a production environment until it stabalises.

## Roadmap

### Version 1.0

- [ ] Complete WASM support (WebRTC, IndexedDB)
- [ ] Connection profiles (local/global/hybrid modes)
- [ ] Data synchronization with conflict resolution
- [ ] Relay support for NAT traversal
- [ ] Advanced query API
- [ ] Metrics and monitoring
- [ ] Migration tools

### Paxos Consensus Integration

Netabase has begun integrating paxakos for distributed consensus. Implementation roadmap:

#### Phase 1: Core Trait Implementation
- [ ] **Implement `LogEntry` trait** for netabase_store definitions
  - Add `id()` method returning unique identifier for each entry
  - Ensure serialization compatibility with bincode
  - Update `NetabaseDefinitionTrait` to require `LogEntry` bound

- [ ] **Create `State` implementation** for distributed state management
  - `apply(&mut self, entry: &LogEntry)` - process entries and update state
  - `freeze(&self)` - create immutable snapshot of current state
  - `cluster_at(&self, round: RoundNum)` - return cluster membership at round
  - `concurrency(&self)` - return parallelism level for round processing

- [ ] **Implement `NodeInfo` trait** for peer identification
  - Define node identity compatible with libp2p `PeerId`
  - Add serialization for network transmission
  - Integrate with existing peer discovery mechanisms

#### Phase 2: libp2p Communicator
- [ ] **Create `PaxosComm unificator` for libp2p integration**
  - Implement `Communicator` trait with 12 associated types:
    - `Node`, `RoundNum`, `CoordNum`, `LogEntry`, `Error`
    - Future types: `SendPrepare`, `SendProposal`, `SendCommit`, `SendCommitById`
    - Vote metadata: `Abstain`, `Yea`, `Nay`
  - Implement 4 required message-sending methods:
    - `send_prepare(coord, round, receivers)` - broadcast prepare messages
    - `send_proposal(coord, round, entry, receivers)` - propose log entry
    - `send_commit(coord, round, entry, receivers)` - commit with full entry
    - `send_commit_by_id(coord, round, entry_id, receivers)` - commit by ID only

- [ ] **Create custom libp2p protocol handler** (`/paxos/1.0.0`)
  - Define request/response message types using serde
  - Integrate with libp2p's `request_response` behavior
  - Handle protocol message routing through swarm
  - Implement timeout and retry logic

- [ ] **Add `PaxosBehaviour` to libp2p swarm**
  - Create behavior struct wrapping paxakos `Node`
  - Implement `NetworkBehaviour` trait
  - Handle protocol events and route to paxakos
  - Integrate with existing Kademlia and Identify behaviors

#### Phase 3: Node Lifecycle Integration
- [ ] **Integrate paxakos node with netabase lifecycle**
  - Initialize paxakos `Node` in `start_swarm()`
  - Use `NodeBuilder` with custom `Communicator` and `State`
  - Handle graceful shutdown in `stop_swarm()`
  - Expose node handle through netabase API

- [ ] **Implement consensus-backed operations**
  - `put_record_consensus(&mut self, record: D)` - append via paxakos
  - Handle `Commit<S, R, P>` futures and apply outcomes
  - Propagate consensus results through event system
  - Provide fallback to DHT for non-consensus operations

#### Phase 4: Optional Features & Optimizations
- [ ] **Add paxakos decorations**
  - `heartbeats` - node liveness monitoring
  - `autofill` - automatic log gap filling
  - `catch-up` - synchronize lagging nodes
  - `master-leases` - optimize read-only operations

- [ ] **Implement cluster management**
  - Dynamic cluster membership changes
  - Node addition/removal protocols
  - Quorum reconfiguration

- [ ] **Performance optimization**
  - Batch log entry applications
  - Concurrent round processing
  - Message compression and deduplication

#### Phase 5: Testing & Documentation
- [ ] **Comprehensive testing**
  - Multi-node consensus tests with nextest
  - Network partition tolerance tests
  - Leader election and failover tests
  - Performance benchmarks with criterion

- [ ] **Documentation & examples**
  - Paxos API documentation
  - Consensus vs DHT operation guide
  - Example: distributed counter with strong consistency
  - Example: replicated state machine

#### Implementation Notes

**Key Design Decisions:**
- Paxos consensus optional via `paxos` feature flag (native-only)
- Coexists with existing DHT-based operations (eventual consistency)
- Users choose consistency model per operation
- WebRTC/WASM compatibility requires alternative consensus (Paxos uses threading)

**Dependencies:**
- `paxakos = "0.13.0"` (already added to netabase_store)
- libp2p request-response protocol for message transport
- Separate feature flag to avoid WASM compilation issues

## Features

### Current Features

- **P2P Networking**:
  - Built on libp2p for robust peer-to-peer communication
  - mDNS for automatic local peer discovery
  - Kademlia DHT for distributed record storage and discovery
  - Identify protocol for peer information exchange
  - Connection limits and management

- **Cross-Platform Support**:
  - Native (TCP, QUIC, mDNS)
  - WASM (WebRTC, WebSocket) - *coming soon*
  - Unified API across platforms

- **Integrated Storage**:
  - Built on `netabase_store` for type-safe data management
  - Multiple backend support (Sled, Redb)
  - Automatic data persistence with secondary key indexing
  - libp2p RecordStore integration

- **Record Distribution**:
  - Publish records to the DHT network
  - Query records from remote peers
  - Automatic record replication
  - Provider advertisement and discovery

- **Type-Safe Operations**:
  - Compile-time verification of network operations
  - Schema-based data models with macros
  - Type-safe record keys and queries

- **Event System**:
  - Broadcast channels for network events
  - Multiple concurrent subscribers
  - Real-time peer discovery notifications
  - Connection and behavior events

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
netabase = "0.0.3"
netabase_store = "0.0.3"
netabase_deps = "0.0.3"

# Required for macros to work
bincode = { version = "2.0", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
strum = { version = "0.27.2", features = ["derive"] }
derive_more = { version = "2.0.1", features = ["from", "try_into", "into"] }

# Runtime dependencies
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

## Quick Start

### 1. Define Your Data Model

```rust
use netabase_store::netabase_definition_module;

#[netabase_definition_module(ChatDefinition, ChatKeys)]
pub mod chat {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(NetabaseModel, bincode::Encode, bincode::Decode, Clone, Debug)]
    #[netabase(ChatDefinition)]
    pub struct Message {
        #[primary_key]
        pub id: String,
        pub author: String,
        pub content: String,
        pub timestamp: i64,
        #[secondary_key]
        pub room_id: String,
    }
}

use chat::*;
```

### 2. Initialize Netabase

```rust
use netabase::Netabase;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a netabase instance with persistent storage
    let mut netabase = Netabase::<ChatDefinition>::new_with_path("./chat_db")?;

    // Start the networking swarm
    netabase.start_swarm().await?;

    println!("Netabase started and listening for peers!");

    Ok(())
}
```

### 3. Store and Publish Records

```rust
// Create a message
let message = Message {
    id: "msg_123".to_string(),
    author: "Alice".to_string(),
    content: "Hello, World!".to_string(),
    timestamp: chrono::Utc::now().timestamp(),
    room_id: "general".to_string(),
};

// Store locally and publish to the DHT
let result = netabase.put_record(message).await?;
println!("Message published! Result: {:?}", result);
```

### 4. Query Records

```rust
// Query a specific record by key
let key = MessageKey::Primary(MessagePrimaryKey("msg_123".to_string()));
let result = netabase.get_record(key).await?;

// Query local records
let local_messages = netabase.query_local_records(Some(10)).await?;
println!("Found {} local messages", local_messages.len());
```

### 5. Provider Management

```rust
// Advertise as a provider for a key
let key = MessageKey::Primary(MessagePrimaryKey("msg_123".to_string()));
netabase.start_providing(key.clone()).await?;
println!("Now providing this message");

// Find providers for a key
let providers_result = netabase.get_providers(key).await?;
match providers_result {
    libp2p::kad::QueryResult::GetProviders(Ok(get_providers_ok)) => {
        use libp2p::kad::GetProvidersOk;
        match get_providers_ok {
            GetProvidersOk::FoundProviders { providers, .. } => {
                println!("Found {} providers", providers.len());
            }
            GetProvidersOk::FinishedWithNoAdditionalRecord { .. } => {
                println!("No providers found");
            }
        }
    }
    _ => {}
}
```

### 6. Listen for Network Events

```rust
use netabase::NetabaseSwarmEvent;

// Subscribe to network events
let mut event_receiver = netabase.subscribe_to_broadcasts();

// Spawn a background task to handle events
tokio::spawn(async move {
    while let Ok(event) = event_receiver.recv().await {
        match &event.0 {
            libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("✓ Connected to peer: {}", peer_id);
            }
            libp2p::swarm::SwarmEvent::Behaviour(behaviour_event) => {
                // Handle mDNS, Kad, Identify events
                println!("Behaviour event: {:?}", behaviour_event);
            }
            _ => {}
        }
    }
});
```

## Advanced Usage

### Multi-Model Networks

Netabase supports multiple data models in a single network:

```rust
#[netabase_definition_module(AppDefinition, AppKeys)]
mod app {
    use super::*;

    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
    #[netabase(AppDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub email: String,
    }

    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
    #[netabase(AppDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub author_id: u64,
    }
}

let mut app = Netabase::<AppDefinition>::new_with_path("./app_db")?;
app.start_swarm().await?;

// Each model type is independently managed
app.put_record(user).await?;
app.put_record(post).await?;
```

### Custom Storage Backend

```rust
use netabase::network::config::{NetabaseConfig, StorageBackend};

// Use Redb instead of default Sled
let config = NetabaseConfig::with_backend(StorageBackend::Redb);
let netabase = Netabase::<ChatDefinition>::new_with_config(config)?;

// Or specify both path and backend
let netabase = Netabase::<ChatDefinition>::new_with_path_and_backend(
    "./my_db",
    StorageBackend::Redb
)?;
```

### DHT Mode Management

```rust
// Get current DHT mode
let mode = netabase.get_mode().await?;
println!("Current mode: {:?}", mode);

// Switch to client mode (read-only, lower resource usage)
netabase.set_mode(Some(libp2p::kad::Mode::Client)).await?;

// Switch to server mode (full participation)
netabase.set_mode(Some(libp2p::kad::Mode::Server)).await?;
```

### Bootstrap and Peer Management

```rust
use libp2p::{Multiaddr, PeerId};

// Add a known peer
let peer_id: PeerId = "12D3KooW...".parse()?;
let address: Multiaddr = "/ip4/192.168.1.100/tcp/4001".parse()?;
netabase.add_address(peer_id, address).await?;

// Bootstrap to join the DHT network
let result = netabase.bootstrap().await?;
println!("Bootstrap result: {:?}", result);

// Remove a peer
netabase.remove_peer(peer_id).await?;
```

## Architecture

### Components

1. **Netabase Struct**: Main API entry point
   - Manages lifecycle (start/stop swarm)
   - Provides typed record operations
   - Handles event subscriptions

2. **Network Layer** (internal):
   - `NetabaseBehaviour`: libp2p network behaviour
   - `NetabaseStore`: Unified storage backend for DHT
   - Swarm handlers for command and event processing

3. **Storage Layer** (`netabase_store`):
   - Type-safe key-value stores
   - Backend abstraction (Sled/Redb/IndexedDB)
   - Secondary key indexing

4. **Event System**:
   - Broadcast channels for network events
   - Multiple concurrent subscribers
   - Zero-cost resubscribe

### Data Flow

```
Application
     ↓ put_record()
  Netabase
     ├─→ Command Channel → Swarm Handler
     │                        ↓
     │                   NetabaseStore (local)
     │                        ↓
     │                   Kademlia DHT
     │                        ↓
     │                   Remote Peers
     │
     └─→ Broadcast Channel ← Swarm Events
              ↓
         Event Subscribers
```

## Performance Considerations

- **Local-first**: All operations start with local storage (fast)
- **Async operations**: Network operations don't block
- **Efficient encoding**: Uses bincode for compact serialization
- **Channel-based**: Non-blocking communication between layers
- **Secondary key indexing**: O(m) queries where m is matching records

### Abstraction Overhead

Netabase builds on `netabase_store` for its storage layer, which provides excellent type safety and multi-backend support. However, this abstraction does come with some performance overhead (typically 5-10%). For applications where maximum performance is critical and you don't need the networking features, consider using `netabase_store` directly.

The main overhead sources are:
- Type conversions for DHT record storage
- libp2p's RecordStore trait implementation
- Channel-based async communication between layers

We're actively working to reduce this overhead while maintaining type safety and the clean API.

### Future Plans

**UniFFI Integration**: We're planning to add UniFFI support to enable using netabase from other languages (Python, Kotlin/Swift, etc.):
- Export generated model code to UniFFI
- Create FFI-safe API wrappers for all major operations
- Enable cross-language distributed applications
- Support for callbacks and async operations across language boundaries

This will make it possible to build distributed applications in Python, Swift, or Kotlin that can seamlessly communicate with Rust-based netabase nodes.

**P2P Network Profiles**: Planned features for easier distributed application development:
- Configurable connection profiles (local-only, DHT-backed, full mesh, etc.)
- Protocol abstraction for easier integration with different transport layers
- Automatic conflict resolution strategies (CRDT-based, last-write-wins, custom)
- Built-in data synchronization patterns

## Platform Support

| Feature | Native | WASM |
|---------|--------|------|
| TCP | ✅ | ❌ |
| QUIC | ✅ | ❌ |
| mDNS | ✅ | ❌ |
| Kad DHT | ✅ | 🚧 |
| Sled Backend | ✅ | ❌ |
| Redb Backend | ✅ | ❌ |
| IndexedDB | ❌ | 🚧 |

*🚧 = Planned for future release*

## Examples

See the `examples/` directory:
- **simple_mdns_chat.rs**: Complete chat application using mDNS discovery
  ```bash
  cargo run --example simple_mdns_chat --features native alice
  # In another terminal
  cargo run --example simple_mdns_chat --features native bob
  ```

## Testing

```bash
# Run all tests (native)
cargo test --features native

# Run a specific test
cargo test --features native test_name

# Build with release optimizations
cargo build --release --features native
```

## API Reference

### Main Methods

- `new()` - Create with defaults
- `new_with_path(path)` - Custom database path
- `new_with_config(config)` - Custom configuration
- `start_swarm()` - Start networking
- `stop_swarm()` - Shutdown gracefully
- `subscribe_to_broadcasts()` - Get event receiver

### Record Operations

- `put_record(model)` - Store and publish
- `get_record(key)` - Query network
- `remove_record(key)` - Remove locally
- `query_local_records(limit)` - Query local store

### Provider Operations

- `start_providing(key)` - Advertise as provider
- `stop_providing(key)` - Stop advertising
- `get_providers(key)` - Find providers

### Network Management

- `bootstrap()` - Join DHT network
- `add_address(peer_id, addr)` - Add peer
- `remove_address(peer_id, addr)` - Remove address
- `remove_peer(peer_id)` - Remove peer
- `get_mode()` - Query DHT mode
- `set_mode(mode)` - Change DHT mode
- `get_protocol_names()` - Get protocol info

## Testing

Netabase includes a comprehensive test suite to ensure reliability and correctness.

### Test Suite Overview

1. **Unit Tests**: Core functionality tests
   ```bash
   cargo test --lib
   ```

2. **Integration Tests**: Multi-node P2P tests using `std::process::Command`
   ```bash
   # Basic P2P tests
   cargo test --test p2p_integration_tests -- --ignored --test-threads=1

   # Advanced DHT tests
   cargo test --test dht_advanced_tests -- --ignored --test-threads=1

   # Chat application tests
   cargo test --test chat_integration_tests -- --ignored --test-threads=1
   ```

3. **Build Verification**: Ensures examples, doctests, and benchmarks compile
   ```bash
   cargo test --test build_verification
   ```

4. **WASM Compilation Tests**: Verifies WASM target compilation
   ```bash
   cargo test --test wasm_compilation
   ```

5. **Benchmarks**: Performance benchmarking
   ```bash
   cargo bench
   ```

### Comprehensive Test Runner

Run all tests systematically using the provided Nushell script:

```bash
# Make script executable
chmod +x run_comprehensive_tests.nu

# Run all tests
./run_comprehensive_tests.nu
```

### Test Coverage

The test suite covers:

- ✅ **mDNS Peer Discovery**: Automatic local network peer discovery
- ✅ **DHT Record Operations**: Put/get records across nodes
- ✅ **Provider Records**: Advertising and querying content providers
- ✅ **Bootstrap**: Joining the DHT network
- ✅ **Cross-Node Communication**: Message passing between nodes
- ✅ **Concurrent Operations**: Multiple simultaneous operations
- ✅ **Network Scalability**: Tests with 2-15 nodes
- ✅ **Record Replication**: Data distribution across the network
- ✅ **Local Storage**: Query and persistence operations
- ✅ **Event Subscription**: Network event broadcasting
- ✅ **Build Verification**: Examples and doctests compilation
- ⚠️ **WASM Compilation**: Currently failing (see WASM Support section)

### CI/CD Integration

For continuous integration, use:

```bash
# Quick test suite (no integration tests)
cargo test --all-features

# Full test suite including integration tests
cargo test --all-features -- --ignored --test-threads=1
```

**Note**: Integration tests use `--test-threads=1` to avoid port conflicts when spawning multiple test nodes.

## Troubleshooting

### Peers Not Discovered

- mDNS only works on local networks
- Check firewall settings
- Ensure both peers are in server mode
- Try adding peers manually with `add_address()`

### Records Not Found

- Bootstrap to join the DHT network first
- Check if you're in client or server mode
- Verify the record was published successfully
- Allow time for DHT propagation

## WASM Support

### Current Status

WASM support is under active development. The `wasm` feature exists but requires additional work to fully function.

### Known WASM Compilation Issues

The following issues prevent successful WASM compilation and need to be resolved:

#### 1. Storage Backend Abstraction Issue

**Problem**: The `IndexedDBStore` and `MemoryStore` implementations use sled-specific methods (`to_ivec()` and `from_ivec()`) that don't exist in the WASM context.

**Location**:
- `netabase_store/src/databases/indexeddb_store.rs:204`
- `netabase_store/src/databases/memory_store.rs:334, 376, 607`

**Error**:
```
error[E0599]: no method named `to_ivec` found for type parameter `D`
error[E0599]: no function or associated item named `from_ivec` found for type parameter `D`
```

**Resolution Needed**:
- Create a platform-agnostic serialization trait that provides `to_vec()` and `from_vec()` for all backends
- Update WASM backends to use the generic serialization methods
- Properly feature-gate sled-specific `IVec` usage

#### 2. Feature Gating

**Issue**: Some features are referenced but not properly defined:
- `feature = "memory"` - Referenced but not in Cargo.toml
- `feature = "indexeddb"` - Referenced as "indexeddb" but defined as "indexed_db_futures"

**Resolution Needed**:
- Add missing feature flags to Cargo.toml
- Ensure consistent feature naming across the codebase

### WASM TODO List

To complete WASM support, the following tasks must be completed:

- [ ] Fix storage backend serialization abstraction (see issue #1 above)
- [ ] Properly implement IndexedDB storage backend for WASM
- [ ] Test WebRTC transport in WASM environment
- [ ] Test WebSocket-WebSys transport
- [ ] Test WebTransport-WebSys functionality
- [ ] Create WASM-specific examples
- [ ] Add browser-based integration tests
- [ ] Document browser storage limitations
- [ ] Add WASM-specific configuration guide
- [ ] Benchmark WASM performance vs native

### Testing WASM Compilation

To test WASM compilation:

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Attempt to build for WASM
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm

# Run WASM-specific tests
cargo test --test wasm_compilation
```

### Workaround for Development

Until WASM support is fully implemented, you can:

1. Use the `native` feature for desktop/server applications
2. Implement your own browser-specific storage using `IndexedDB` directly
3. Use a native backend as a bridge for WASM applications
4. Consider using the library in client mode only (no local storage)

## Documentation

- **[Getting Started](./GETTING_STARTED.md)**: Step-by-step tutorial for building your first distributed application
- **[Architecture](./ARCHITECTURE.md)**: Deep dive into netabase's design and internal architecture
- **[Macro Guide](../netabase_store/MACRO_GUIDE.md)**: Learn what happens behind the scenes with the `netabase_definition_module` macro

## License

This project is licensed under the GPL 3 License.

## Related Projects

- [netabase_store](https://github.com/newsnet-africa/netabase_store) - Type-safe storage layer
- [gdelt_fetcher](https://github.com/newsnet-africa/gdelt_fetcher) - GDELT data source integration

## Contributing

Contributions welcome! Please ensure:
- Code passes all tests
- New features include tests and documentation
- Follow existing code style
