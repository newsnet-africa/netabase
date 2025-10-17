# Netabase - Quick Reference

## Overview

Netabase is a distributed peer-to-peer database system built on libp2p with type-safe, macro-driven data modeling. It combines the power of embedded databases (via netabase_store) with P2P networking capabilities.

## Architecture

```
┌─────────────────────────────────────┐
│     Application Code                │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────┐
│  Netabase (P2P Layer)               │
│  - Swarm Management                 │
│  - Kademlia DHT                     │
│  - mDNS Discovery                   │
│  - Event Handling                   │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────┐
│  Netabase Store (Storage Layer)    │
│  - Type-Safe Models                 │
│  - Multi-Backend Support            │
│  - LibP2P Integration               │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────┐
│  Storage Backend                    │
│  Sled | IndexedDB | Memory          │
└─────────────────────────────────────┘
```

## Quick Start

```rust
// 1. Define your schema
#[netabase_definition_module(ChatSchema, ChatKeys)]
mod chat {
    #[derive(NetabaseModel, Clone, Debug, ...)]
    pub struct Message {
        #[primary_key]
        pub id: String,
        pub content: String,
        #[secondary_key]
        pub sender: String,
        pub timestamp: u64,
    }
}

// 2. Create P2P network
let config = NetabaseConfig::default();
let (netabase, mut events) = Netabase::<ChatSchema>::new(config).await?;

// 3. Store and share data
let msg = Message { ... };
let def = ChatSchema::Message(msg);
netabase.put_record(def).await?;

// 4. Retrieve from network
let key = MessagePrimaryKey(id);
let record = netabase.get_record(key).await?;
```

## Key Components

### Network Layer (libp2p)
- **Kademlia DHT**: Distributed hash table for record storage
- **mDNS**: Automatic peer discovery on local networks
- **Identify**: Peer identification and capability exchange
- **Connection Limits**: Configurable connection management

### Storage Layer (netabase_store)
- **Type-Safe Models**: Compile-time checked data structures
- **Multiple Backends**: Sled (native), IndexedDB (WASM), Memory
- **Automatic Indexing**: Primary and secondary key support
- **Cross-Platform**: Single API for native and WASM

### Event System
- **Swarm Events**: Connection lifecycle, peer discovery
- **DHT Events**: Record operations, provider announcements
- **mDNS Events**: Local peer discovery
- **Custom Commands**: User-defined operations

## Features

| Feature | Purpose |
|---------|---------|
| `default` | Native features (includes `libp2p`) |
| `libp2p` | P2P networking capabilities |
| `native` | Native platform support (Sled, TCP, mDNS) |
| `wasm` | WebAssembly support (IndexedDB, WebRTC) |

## Common Operations

### Network Operations
```rust
// Put record to DHT
netabase.put_record(definition).await?;

// Get record from DHT
let record = netabase.get_record(key).await?;

// Announce as provider
netabase.start_providing(key).await?;

// Find providers
let providers = netabase.get_providers(key).await?;
```

### Local Storage
```rust
// Access local database
let store = netabase.store();
let tree = store.open_tree::<Message>();

// Local operations
tree.put(message)?;
let msg = tree.get(key)?;
```

### Peer Management
```rust
// Add peer
netabase.add_address(peer_id, multiaddr).await?;

// Bootstrap DHT
netabase.bootstrap().await?;

// Set mode (client/server)
netabase.set_mode(mode).await?;
```

## Event Handling

```rust
while let Some(event) = events.recv().await {
    match event {
        NetabaseEvent::RecordStored(key) => {
            println!("Stored: {:?}", key);
        },
        NetabaseEvent::PeerDiscovered(peer_id) => {
            println!("Discovered: {}", peer_id);
        },
        // ... handle other events
    }
}
```

## Configuration

```rust
let config = NetabaseConfig {
    db_path: PathBuf::from("./data"),
    keypair: Some(keypair),
    listen_addrs: vec!["/ip4/0.0.0.0/tcp/0".parse()?],
    bootstrap_peers: vec![],
    enable_mdns: true,
    connection_limits: Some(limits),
};
```

## Performance

### Local Operations
- **Put**: 50-100μs (Sled)
- **Get**: 10-20μs (Sled)
- **Secondary Key Query**: O(n) matching records

### Network Operations
- **DHT Put/Get**: 3-5 network hops (average)
- **Provider Discovery**: < 1 second (local network)
- **Record Replication**: Automatic with K=20

## Examples

- **`simple_chat.rs`**: Basic P2P chat application
- **`ratatui_chat.rs`**: TUI chat with real-time updates
- **`simple_mdns_chat.rs`**: Local network chat with mDNS
- **`network_test.rs`**: Network connectivity testing

Run examples:
```bash
cargo run --example simple_mdns_chat -- alice
```

## Testing

```bash
# Unit tests
cargo test --features native --lib

# Integration tests
cargo test --features native --test integration_tests

# Kademlia tests
cargo test --features native --test kademlia_sled_test -- --test-threads=1
```

## Project Structure

```
netabase/
├── src/                    # Core P2P networking
│   ├── network/            # libp2p integration
│   │   ├── behaviour/      # Custom network behaviours
│   │   └── swarm/          # Swarm and event handlers
│   └── errors/             # Error types
├── netabase_store/         # Storage abstraction layer
│   ├── src/                # Store implementations
│   ├── netabase_macros/    # Proc macros
│   └── examples/           # Storage examples
├── examples/               # P2P application examples
└── tests/                  # Integration tests
```

## Migration from v0.x

Key changes in the new architecture:
1. **Schema → Definition**: `netabase_schema_module` → `netabase_definition_module`
2. **Automatic Key Generation**: No more manual key struct definitions
3. **Unified API**: Same interface for native and WASM
4. **Event-Driven**: New event system for network operations

## Troubleshooting

### No Peers Found
- Enable mDNS: `config.enable_mdns = true`
- Add bootstrap peers to configuration
- Check firewall settings

### Database Locked
- Use single-threaded tests: `--test-threads=1`
- Ensure proper cleanup between tests
- Check for orphaned processes

### WASM Compilation
- Use correct features: `--features wasm --no-default-features`
- Install wasm-pack: `cargo install wasm-pack`
- Check browser compatibility

## Links

- [Netabase Store README](./netabase_store/README.md)
- [Examples](./examples/README.md)
- [Tests](./tests/README.md)
- [Full README](./README.md)

## License

GNU GPL v3 - See [LICENSE](./LICENSE)
