# Netabase Architecture

This document describes the internal architecture of the Netabase peer-to-peer database system.

## Overview

Netabase is a distributed database that combines local storage with peer-to-peer networking. It provides a type-safe API for storing and querying data both locally and across a network of peers using libp2p and Kademlia DHT.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
│  (User code using Netabase<D> public API)              │
└────────────────────┬────────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
        ▼                         ▼
┌───────────────┐         ┌──────────────────┐
│ Command       │         │ Broadcast        │
│ Channel (TX)  │         │ Channel (RX)     │
└───────┬───────┘         └──────┬───────────┘
        │                         │
        │                         │
        ▼                         ▼
┌─────────────────────────────────────────────────────────┐
│               Background Swarm Task                      │
│  ┌─────────────────────────────────────────────────┐   │
│  │           Swarm Event Loop                      │   │
│  │  - Processes commands from channel              │   │
│  │  - Handles swarm events                         │   │
│  │  - Broadcasts events to subscribers             │   │
│  └────────┬────────────────────────┬────────────────┘   │
│           │                        │                    │
│           ▼                        ▼                    │
│  ┌────────────────┐      ┌──────────────────┐         │
│  │ Command        │      │ Swarm Event      │         │
│  │ Handlers       │      │ Handlers         │         │
│  └────────┬───────┘      └────────┬─────────┘         │
│           │                        │                    │
└───────────┼────────────────────────┼────────────────────┘
            │                        │
            ▼                        ▼
┌─────────────────────────────────────────────────────────┐
│                  libp2p Swarm                           │
│  ┌─────────────────────────────────────────────────┐   │
│  │           NetabaseBehaviour                     │   │
│  │  ┌──────────────┐  ┌──────────────┐           │   │
│  │  │  Kademlia    │  │   Identify   │           │   │
│  │  │     DHT      │  │              │           │   │
│  │  └──────────────┘  └──────────────┘           │   │
│  │  ┌──────────────┐  ┌──────────────┐           │   │
│  │  │    mDNS      │  │  Connection  │           │   │
│  │  │  Discovery   │  │    Limits    │           │   │
│  │  └──────────────┘  └──────────────┘           │   │
│  └─────────────────────────────────────────────────┘   │
│                         │                              │
│                         ▼                              │
│              ┌──────────────────────┐                  │
│              │  NetabaseStore       │                  │
│              │  (Sled/Redb)         │                  │
│              └──────────────────────┘                  │
└─────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│           Disk Storage (Database Files)                 │
└─────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Netabase Struct (`src/lib.rs`)

**Purpose**: Main public API and entry point for all operations.

**Responsibilities**:
- Lifecycle management (start/stop swarm)
- Channel management (command sender, broadcast receiver)
- Public API methods for DHT operations
- Event subscription management

**Key Fields**:
- `command_sender`: Sends commands to the background swarm task
- `broadcast_receiver`: Template for creating new event subscribers
- `swarm_thread`: Handle to the background task
- `config`: Configuration options
- `database_path`: Optional custom database location

**Thread Model**: Lives in the main application thread, but operations are non-blocking through channels.

### 2. Background Swarm Task (`src/network/swarm/handlers/mod.rs`)

**Purpose**: Runs the libp2p swarm event loop in a separate async task.

**Responsibilities**:
- Processing commands from the application
- Handling libp2p swarm events
- Broadcasting events to subscribers
- Managing network I/O

**Event Loop**:
```rust
loop {
    tokio::select! {
        Some(command) = command_receiver.recv() => {
            // Process user commands
            handle_command_events(&mut swarm, command);
        },
        Some(event) = swarm.next() => {
            // Handle swarm events
            let event = NetabaseSwarmEvent(event);
            broadcast_sender.send(event.clone());
            handle_swarm_events(config, &mut swarm, event);
        }
    }
}
```

### 3. Command System (`src/network/swarm/handlers/command_events/`)

**Purpose**: Request-response pattern for application to swarm communication.

**Command Types** (enum `KademliaCommand`):
- `PutRecord`: Store a record in the DHT
- `GetRecord`: Retrieve a record from the DHT
- `RemoveRecord`: Remove from local storage
- `StartProviding`: Advertise as a provider
- `StopProviding`: Stop advertising
- `GetProviders`: Find providers for a key
- `Bootstrap`: Join the DHT network
- `AddAddress`/`RemoveAddress`/`RemovePeer`: Peer management
- `Mode`/`SetMode`: DHT mode management
- `ProtocolNames`: Query protocol information
- `LocalStore`: Query local records

**Flow**:
1. Application calls public method (e.g., `put_record()`)
2. Creates oneshot channel for response
3. Sends command via `command_sender`
4. Awaits response from oneshot channel
5. Background task processes command
6. Sends response back via oneshot channel

**Handler Structure**: Each command type has its own handler module:
- `add_address.rs`, `bootstrap.rs`, `put_record.rs`, etc.
- Clean separation of concerns
- Easy to add new commands

### 4. Event System (`src/network/swarm/handlers/swarm_events/`)

**Purpose**: Broadcast network events to multiple subscribers.

**Event Types**:
- Connection events (established, closed, incoming, etc.)
- Behavior events (Kad, mDNS, Identify)
- Listener events (new address, expired, error)

**Broadcasting**:
- Uses `tokio::sync::broadcast` channel
- Multiple subscribers can independently receive events
- Events are cloned for each subscriber
- No backpressure - slow consumers may miss events

**Handler Structure**:
- `behaviour/kad.rs`: Kademlia DHT events
- `behaviour/mdns.rs`: mDNS discovery events
- `behaviour/identify.rs`: Peer identification events
- `connection_established.rs`, etc.: Connection lifecycle events

### 5. Network Behavior (`src/network/behaviour/mod.rs`)

**Purpose**: Composite libp2p NetworkBehaviour implementation.

**Components**:
- **Kademlia DHT**: Distributed record storage and peer discovery
- **mDNS**: Local network peer discovery (native only)
- **Identify**: Peer information exchange
- **Connection Limits**: Connection management

**Store Integration**: Kademlia uses `NetabaseStore` as its RecordStore implementation, bridging the DHT with local storage.

### 6. Storage Layer (`src/network/store.rs`)

**Purpose**: Unified storage backend for Kademlia DHT.

**Implementation**:
- Enum wrapping either `SledStore` or `RedbStore`
- Implements libp2p's `RecordStore` trait
- Delegates operations to the wrapped store
- Provides type-safe access to the underlying database

**RecordStore Operations**:
- `get(key)`: Retrieve a record
- `put(record)`: Store a record
- `remove(key)`: Delete a record
- `records()`: Iterate all records
- `add_provider()`: Add a provider record
- `providers(key)`: Get providers for a key
- `provided()`: Get all provided records
- `remove_provider()`: Remove a provider

### 7. Configuration (`src/network/config/mod.rs`)

**Purpose**: Configuration types for Netabase.

**Structures**:
- `NetabaseConfig`: Top-level configuration
- `DHTDiscoveryConfig`: DHT settings
- `MDNSDiscoveryConfig`: mDNS settings
- `StorageBackend`: Backend selection (Sled/Redb)

**Design**: Simple struct-based configuration with sensible defaults.

## Data Flow

### Put Record Operation

```
1. Application: netabase.put_record(model)
        ↓
2. Netabase: Convert model to definition enum
        ↓
3. Netabase: Create oneshot channel for response
        ↓
4. Netabase: Send PutRecord command via command_sender
        ↓
5. Background Task: Receive command
        ↓
6. Command Handler: Process PutRecord
        ↓
7. NetabaseStore: Store locally via RecordStore::put()
        ↓
8. SledStore/RedbStore: Persist to disk
        ↓
9. Kademlia: Publish to DHT network
        ↓
10. Command Handler: Send response via oneshot channel
        ↓
11. Netabase: Return result to application
```

### Get Record Operation

```
1. Application: netabase.get_record(key)
        ↓
2. Netabase: Convert key to definition key
        ↓
3. Netabase: Create oneshot channel
        ↓
4. Netabase: Send GetRecord command
        ↓
5. Background Task: Receive command
        ↓
6. Command Handler: Process GetRecord
        ↓
7. Kademlia: Query DHT (checks local store first)
        ↓
8. [If not local] Query remote peers
        ↓
9. Kademlia: Return QueryResult
        ↓
10. Command Handler: Send response via oneshot channel
        ↓
11. Netabase: Return result to application
```

### Event Subscription

```
1. Application: netabase.subscribe_to_broadcasts()
        ↓
2. Netabase: Call broadcast_receiver.resubscribe()
        ↓
3. Netabase: Return new Receiver<NetabaseSwarmEvent>
        ↓
4. Application: Spawn task to handle events
        ↓
5. [In background] Swarm generates events
        ↓
6. Background Task: Wrap in NetabaseSwarmEvent
        ↓
7. Background Task: broadcast_sender.send(event)
        ↓
8. Broadcast Channel: Clone event for each subscriber
        ↓
9. Application Task: receiver.recv().await
        ↓
10. Application Task: Process event
```

## Concurrency Model

### Threading

- **Main Thread**: Runs application code and Netabase API
- **Background Task**: Runs swarm event loop (tokio task)
- **Additional Tasks**: User can spawn multiple event handlers

### Synchronization

- **Command Channel**: `mpsc::channel` (single producer, single consumer)
  - Application → Background task
  - Bounded buffer (100 items)

- **Broadcast Channel**: `broadcast::channel` (single producer, multiple consumers)
  - Background task → Multiple subscribers
  - Bounded buffer (1000 events)

- **Oneshot Channels**: `oneshot::channel` (for responses)
  - One per command
  - Automatically dropped after use

### Thread Safety

- All public types are `Send` + `Sync` where appropriate
- No shared mutable state between threads
- Message passing for all inter-thread communication

## Error Handling

### Error Types

- `anyhow::Result<T>`: Most public API methods
- `NetabaseError`: Storage-layer errors
- `libp2p` errors: Wrapped in query results

### Error Propagation

```
Database Error
    ↓
NetabaseStore
    ↓
Kademlia (as RecordStore::Error)
    ↓
Command Handler
    ↓
Oneshot Channel
    ↓
Public API
    ↓
Application
```

## Performance Considerations

### Optimizations

1. **Zero-Copy**: Events use `Cow` types where possible
2. **Efficient Serialization**: Bincode for compact encoding
3. **Bounded Channels**: Prevents unbounded memory growth
4. **Background Processing**: Network I/O doesn't block application
5. **Local-First**: All operations check local store first

### Bottlenecks

1. **Channel Capacity**: Commands can block if buffer fills
2. **Event Broadcasting**: Slow subscribers may miss events
3. **Serialization**: Large records increase overhead
4. **Network Latency**: DHT operations depend on network conditions

## Testing Strategy

### Unit Tests

- Individual command handlers
- Event handlers
- Serialization/deserialization
- Key generation

### Integration Tests

- Multi-node scenarios
- DHT operations
- Event subscription
- Error cases

### Example Tests

- `simple_mdns_chat.rs`: Full application example
- Manual testing for peer discovery

## Future Improvements

1. **Metrics**: Built-in performance monitoring
2. **Tracing**: Distributed tracing support
3. **Connection Pooling**: Reuse connections efficiently
4. **Adaptive Replication**: Smart data distribution
5. **WASM Support**: Complete browser integration
6. **Query Optimization**: Caching and batching

## Debugging Tips

### Enable Logging

```bash
RUST_LOG=netabase=debug cargo run
```

### Monitor Events

```rust
let mut events = netabase.subscribe_to_broadcasts();
tokio::spawn(async move {
    while let Ok(event) = events.recv().await {
        eprintln!("EVENT: {:?}", event);
    }
});
```

### Check DHT Mode

```rust
let mode = netabase.get_mode().await?;
println!("DHT Mode: {:?}", mode);
```

### Inspect Local Store

```rust
let records = netabase.query_local_records(None).await?;
println!("Local records: {}", records.len());
```

## Related Documentation

- [README.md](./README.md): User-facing documentation
- [netabase_store/ARCHITECTURE.md](../netabase_store/ARCHITECTURE.md): Storage layer architecture
- [examples/](./examples/): Working examples
