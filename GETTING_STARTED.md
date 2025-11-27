# Getting Started with Netabase

## Required Dependencies

Add these to your `Cargo.toml`:

```toml
[dependencies]
# Main libraries with features - IMPORTANT!
netabase = { version = "0.0.6", features = ["native"] }
netabase_store = { version = "0.0.6", features = ["native"] }

# Required for macros to work
bincode = { version = "2.0", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
strum = { version = "0.27.2", features = ["derive"] }
derive_more = { version = "2.0.1", features = ["from", "try_into", "into"] }

# Runtime dependencies
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
chrono = "0.4"  # For timestamps
libp2p = "0.56"  # For advanced P2P features
```

### Feature Flags (CRITICAL!)

**You MUST enable the `native` feature for desktop/server applications:**

```toml
# For desktop/server (recommended):
netabase = { version = "0.0.6", features = ["native"] }
netabase_store = { version = "0.0.6", features = ["native"] }
```

Available features:
- `native` - Enables TCP, QUIC, mDNS, and Kademlia DHT (desktop/server)
- `wasm` - Enables WebRTC, WebSocket for browser applications (coming soon)

**Why so many dependencies?** The macros from `netabase_store` generate code that uses these crates. Due to Rust's macro hygiene rules, they must be available in your `Cargo.toml`.

## Define Your Data Model

Inside your schema definition module, import the required items:

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

## Complete Working Example

```rust
use netabase::Netabase;
use netabase_store::{netabase_definition_module, NetabaseModel};
use anyhow::Result;

#[netabase_definition_module(ChatDefinition, ChatKeys)]
mod chat {
    use super::*;
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
    }
}

use chat::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create netabase instance
    let mut netabase = Netabase::<ChatDefinition>::new_with_path("./chat_db")?;
    
    // Start the P2P network
    netabase.start_swarm().await?;
    println!("Network started!");
    
    // Create and publish a message
    let message = Message {
        id: "msg_1".to_string(),
        author: "Alice".to_string(),
        content: "Hello, World!".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };
    
    netabase.put_record(message).await?;
    println!("Message published!");
    
    // Query local records
    let messages = netabase.query_local_records(None).await?;
    println!("Found {} messages", messages.len());
    
    // Subscribe to network events
    let mut events = netabase.subscribe_to_broadcasts();
    
    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            if let libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } = &event.0 {
                println!("Connected to peer: {}", peer_id);
            }
        }
    });
    
    // Keep running...
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    
    Ok(())
}
```

## Common Mistakes

### 1. Forgetting to import `netabase` attribute

**Error:**
```
error: cannot find attribute `netabase` in this scope
```

**Fix:** Add `use netabase_store::netabase;` inside your schema module.

### 2. Not starting the swarm

Your netabase instance won't connect to peers unless you call:
```rust
netabase.start_swarm().await?;
```

### 3. Missing tokio runtime

Netabase requires a tokio async runtime:
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // your code
}
```

### 4. Incorrect event type imports

Use the re-exported types:
```rust
use netabase::NetabaseSwarmEvent;  // ✓ Correct
// NOT: use netabase::network::behaviour::clone_impl::NetabaseSwarmEvent;
```

## Network Configuration

### Custom Storage Backend

```rust
use netabase::network::config::{NetabaseConfig, StorageBackend};

let config = NetabaseConfig::with_backend(StorageBackend::Redb);
let netabase = Netabase::<ChatDefinition>::new_with_config(config)?;
```

### Peer Discovery

**Local Network (mDNS)**:
- Automatic on native platforms
- No configuration needed
- Works within the same LAN

**Global Network (DHT)**:
```rust
// Bootstrap to join the DHT network
netabase.bootstrap().await?;

// Add known peers manually
use libp2p::{Multiaddr, PeerId};
let peer_id: PeerId = "12D3KooW...".parse()?;
let addr: Multiaddr = "/ip4/192.168.1.100/tcp/4001".parse()?;
netabase.add_address(peer_id, addr).await?;
```

### DHT Modes

```rust
// Server mode (full participation, default)
netabase.set_mode(Some(libp2p::kad::Mode::Server)).await?;

// Client mode (lower resource usage, read-only DHT)
netabase.set_mode(Some(libp2p::kad::Mode::Client)).await?;
```

## Event Handling

### Subscribe to Network Events

```rust
let mut events = netabase.subscribe_to_broadcasts();

tokio::spawn(async move {
    while let Ok(event) = events.recv().await {
        match &event.0 {
            libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected: {}", peer_id);
            }
            libp2p::swarm::SwarmEvent::Behaviour(behaviour) => {
                // Handle Kad, mDNS, Identify events
                use netabase::NetabaseBehaviourEvent;
                match behaviour {
                    NetabaseBehaviourEvent::Mdns(mdns) => {
                        // mDNS peer discovery
                    }
                    NetabaseBehaviourEvent::Kad(kad) => {
                        // Kademlia DHT events
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
});
```

### Multiple Subscribers

You can create multiple event subscribers:
```rust
let events1 = netabase.subscribe_to_broadcasts();
let events2 = netabase.subscribe_to_broadcasts();
let events3 = netabase.subscribe_to_broadcasts();
// Each receives events independently
```

## Next Steps

- **[README.md](./README.md)** - Complete API reference with advanced features
- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Deep dive into netabase internals
- **[Macro Guide](../netabase_store/MACRO_GUIDE.md)** - Learn what the macros generate behind the scenes
- **[examples/simple_mdns_chat.rs](./examples/simple_mdns_chat.rs)** - Complete chat app
- Run the example: `cargo run --example simple_mdns_chat --features native`
