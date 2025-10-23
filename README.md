# Netabase

A peer-to-peer networking layer built on libp2p with integrated storage, enabling distributed applications with automatic data synchronization across native and WASM environments.

## Features

### Current Features

- **P2P Networking**:
  - Built on libp2p for robust peer-to-peer communication
  - mDNS for local peer discovery
  - Kademlia DHT for global peer discovery
  - Identify protocol for peer information exchange

- **Cross-Platform Support**:
  - Native (TCP, QUIC, WebSocket, mDNS)
  - WASM (WebRTC, WebTransport, WebSocket)
  - Unified API across platforms

- **Integrated Storage**:
  - Built on netabase_store for type-safe data management
  - Automatic data persistence
  - Support for Sled, Redb, and IndexedDB backends

- **Record Distribution**:
  - Publish records to the DHT
  - Query records from remote peers
  - Automatic record replication

- **Type-Safe Operations**:
  - Compile-time verification of network operations
  - Schema-based data models
  - Type-safe record keys and queries

- **Event System**:
  - Swarm events (connections, peer discovery)
  - Behavior events (Kad, mDNS, Identify)
  - Custom command events for application logic

### TODO for 1.0.0

- [ ] **Connection Profiles/Modes**:
  - Local mode: mDNS only, optimized for LAN
  - Global mode: Full DHT participation
  - Hybrid mode: Balanced local + global discovery
  - Relay mode: NAT traversal support

- [ ] **Data Synchronization**:
  - Automatic sync of local changes to peers
  - Conflict resolution strategies
  - Sync state tracking and recovery

- [ ] **Advanced Queries**:
  - Multi-hop queries across the network
  - Query result aggregation
  - Cached query results

- [ ] **Security**:
  - Peer authentication
  - Encrypted connections (already supported by libp2p)
  - Access control for records

- [ ] **Performance Optimizations**:
  - Connection pooling
  - Bandwidth management
  - Adaptive replication strategies

- [ ] **Monitoring & Observability**:
  - Built-in metrics collection
  - Network health monitoring
  - Peer statistics and analytics

- [ ] **Migration Tools**:
  - Network protocol version management
  - Backward compatibility helpers

## Installation

Add to your `Cargo.toml`:

```toml
# For native platforms
[dependencies]
netabase = { version = "0.1", features = ["native"] }

# For WASM platforms
[dependencies]
netabase = { version = "0.1", features = ["wasm"] }
```

## Quick Start

### Define Your Data Model

```rust
use netabase::*;
use netabase_store::*;

#[netabase_definition_module(ChatDefinition, ChatKeys)]
mod chat_schema {
    use netabase_deps::{bincode, serde};
    use netabase_macros::NetabaseModel;
    use netabase_store::netabase;

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
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

use chat_schema::*;
```

### Initialize Netabase

```rust
use netabase::Netabase;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a netabase instance with persistent storage
    let mut netabase = Netabase::<ChatDefinition>::new_with_path("chat_db")?;

    // Start the networking swarm
    netabase.start_swarm().await?;

    println!("Netabase started! Peer ID: {}", netabase.local_peer_id());

    Ok(())
}
```

### Store and Publish Records

```rust
// Create a message
let message = Message {
    id: "msg123".to_string(),
    author: "Alice".to_string(),
    content: "Hello, World!".to_string(),
    timestamp: 1234567890,
    room_id: "general".to_string(),
};

// Store locally and publish to the network
netabase.put_record(message).await?;

println!("Message published to the network!");
```

### Query Records

```rust
// Query local records
let local_messages = netabase.query_local_records(None).await;
println!("Found {} local messages", local_messages.len());

// Query by secondary key
let general_messages = netabase
    .query_local_by_secondary_key(MessageSecondaryKeys::RoomIdKey("general".to_string()))
    .await;
println!("Messages in #general: {}", general_messages.len());

// Query remote peers (coming in 1.0.0 with improved API)
// let remote_messages = netabase.query_remote_records(...).await?;
```

### Listen for Network Events

```rust
use netabase::events::*;

// Event loop (simplified)
loop {
    if let Some(event) = netabase.poll_event().await {
        match event {
            SwarmEvent::PeerDiscovered(peer_id) => {
                println!("Discovered peer: {}", peer_id);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to: {}", peer_id);
            }
            _ => {}
        }
    }
}
```

## Advanced Usage

### Multi-Model Networks

Netabase supports multiple data models in a single network:

```rust
#[netabase_definition_module(AppDefinition, AppKeys)]
mod app_schema {
    #[derive(NetabaseModel, /* ... */)]
    #[netabase(AppDefinition)]
    pub struct User { /* ... */ }

    #[derive(NetabaseModel, /* ... */)]
    #[netabase(AppDefinition)]
    pub struct Post { /* ... */ }

    #[derive(NetabaseModel, /* ... */)]
    #[netabase(AppDefinition)]
    pub struct Comment { /* ... */ }
}

let mut app = Netabase::<AppDefinition>::new_with_path("app_db")?;
app.start_swarm().await?;

// Each model type is independently managed
app.put_record(user).await?;
app.put_record(post).await?;
app.put_record(comment).await?;
```

### Custom Transport Configuration (Native)

```rust
#[cfg(feature = "native")]
{
    use netabase::config::TransportConfig;

    let config = TransportConfig {
        enable_tcp: true,
        enable_quic: true,
        enable_mdns: true,
        enable_kad: true,
        // ... more configuration options (coming in 1.0.0)
    };

    let netabase = Netabase::<ChatDefinition>::new_with_config("chat_db", config)?;
}
```

### WASM-Specific Usage

```rust
#[cfg(target_arch = "wasm32")]
{
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub async fn init_netabase() -> Result<(), JsValue> {
        // WASM uses IndexedDB for storage
        let mut netabase = Netabase::<ChatDefinition>::new_with_path("chat_db")
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        netabase.start_swarm().await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(())
    }
}
```

## Architecture

### Components

1. **Swarm**: libp2p network manager
   - Handles peer connections
   - Manages network transports
   - Coordinates network behaviors

2. **Store**: Persistent data storage
   - Type-safe key-value store
   - Backend abstraction (Sled/Redb/IndexedDB)
   - Secondary key indexing

3. **Behavior**: Network protocols
   - Kademlia DHT for peer and record discovery
   - mDNS for local network discovery
   - Identify for peer information exchange

4. **Events**: Asynchronous event system
   - Swarm events (connections, discovery)
   - Behavior events (protocol-specific)
   - Application events (custom logic)

### Data Flow

```
Application
     ↓ put_record()
  Netabase
     ↓
  ├─→ Store (local persistence)
  └─→ Swarm (network distribution)
          ↓
      libp2p DHT
          ↓
      Remote Peers
```

## Performance Considerations

- **Local-first**: All reads are local by default (fast)
- **Async writes**: Network distribution is asynchronous (non-blocking)
- **Efficient encoding**: Uses bincode for compact serialization
- **Minimal overhead**: Thin wrapper around libp2p and storage

## Platform Support

| Feature | Native | WASM |
|---------|--------|------|
| TCP | ✅ | ❌ |
| QUIC | ✅ | ❌ |
| WebSocket | ✅ | ✅ |
| WebRTC | ❌ | ✅ |
| WebTransport | ❌ | ✅ |
| mDNS | ✅ | ❌ |
| Kad DHT | ✅ | ✅ |
| Sled | ✅ | ❌ |
| Redb | ✅ | ❌ |
| IndexedDB | ❌ | ✅ |

## Examples

See the `examples/` directory:
- `simple_mdns_chat.rs`: Local chat using mDNS discovery
- More examples coming in 1.0.0

## Testing

```bash
# Run all tests (native)
cargo test --features native

# Run WASM tests (requires wasm-pack)
wasm-pack test --node --features wasm
```

## Troubleshooting

### NAT Traversal

Currently, Netabase requires direct connectivity or manual port forwarding. Relay support is planned for 1.0.0.

### Browser Compatibility (WASM)

Requires modern browsers with WebRTC support:
- Chrome/Edge: ✅
- Firefox: ✅
- Safari: ⚠️ (limited WebRTC support)

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Links

- [Netabase Store (storage layer)](../netabase_store)
- [GDELT Fetcher (data source)](../gdelt_fetcher)
- [Example Usage](../test_netabase)

## Contributing

Contributions are welcome! Please open an issue or PR on GitHub (coming in 1.0.0).
