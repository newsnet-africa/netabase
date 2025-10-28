# Sync Request-Response Integration

## Overview

The sync protocol has been successfully integrated into Netabase using libp2p's `request_response` behavior. This provides a clean, efficient way to implement Byzantine fault-tolerant synchronization across peers.

## Architecture

### Components

1. **Protocol Messages** (`src/sync/protocol.rs`)
   - `SyncRequest`: Request types for sync operations
   - `SyncResponse`: Response types for sync operations
   - `SyncRecord`: Synchronized record with metadata

2. **Codec** (`src/sync/codec.rs`)
   - `SyncCodec`: Implements libp2p's `Codec` trait
   - Uses JSON serialization for compatibility
   - 10 MB message size limit
   - Length-prefixed framing (4-byte big-endian)

3. **Network Behavior** (`src/network/behaviour/mod.rs`)
   - Added `sync: libp2p::request_response::Behaviour<SyncCodec>` field
   - Integrated into `NetabaseBehaviour` via NetworkBehaviour derive
   - Enabled only with `native` feature flag

## Protocol Messages

### Requests

```rust
pub enum SyncRequest {
    // State synchronization
    GetStateDigest,
    GetRecords { collection: String, keys: Vec<Vec<u8>> },
    GetRecordsSince { collection: String, since: VectorClock },

    // Sybil resistance
    GetChallenge,
    SubmitProof { proof: ProofOfWork },

    // Byzantine Reliable Broadcast
    BrbEcho { message_id: Vec<u8>, payload_hash: Vec<u8>, signature: Vec<u8> },
    BrbReady { message_id: Vec<u8>, payload_hash: Vec<u8>, signature: Vec<u8> },

    // Paxos consensus
    Paxos { message: PaxosMessage },
}
```

### Responses

```rust
pub enum SyncResponse {
    StateDigest { digest: StateDigest, vector_clock: VectorClock },
    Records { collection: String, records: Vec<SyncRecord> },
    Challenge { challenge: Vec<u8>, challenge_id: Vec<u8> },
    ProofVerified { valid: bool, duration_secs: u64 },
    BrbEchoAck { message_id: Vec<u8> },
    BrbReadyAck { message_id: Vec<u8> },
    Paxos { message: PaxosMessage },
    Error { message: String },
}
```

## Configuration

### Enabling Sync

The sync behavior is enabled by default with the `native` feature:

```toml
[features]
native = [
    # ... other features ...
    "libp2p/request-response",
]
```

### Protocol

- **Protocol ID**: `/netabase/sync/1.0.0`
- **Support**: Full (both request and response)
- **Max Message Size**: 10 MB

## Usage Example

```rust
use netabase::sync::{SyncRequest, SyncResponse, SyncCodec, SYNC_PROTOCOL};

// The sync behavior is automatically initialized when creating NetabaseBehaviour
let mut netabase = Netabase::<MyDefinition>::new()?;
netabase.start_swarm().await?;

// Sync requests/responses are handled through the swarm event loop
// See src/network/swarm/handlers/swarm_events/behaviour/mod.rs
```

## Event Handling

Sync events are handled in the swarm event loop:

```rust
// src/network/swarm/handlers/swarm_events/behaviour/mod.rs
match behaviour_event {
    NetabaseBehaviourEvent::Sync(sync_event) => {
        // Handle sync request/response events
        // TODO: Implement specific handlers for each request type
    }
    // ... other events ...
}
```

## Implementation Details

### Serialization

- Uses `serde_json` for serialization (not `bincode`)
- All message types have `Serialize` and `Deserialize` derives
- PeerId serialization handled via custom serde helpers

### Message Format

```
+-------------------+
| Length (4 bytes)  |  <- Big-endian u32
+-------------------+
| JSON payload      |  <- Serialized message
+-------------------+
```

### Limitations

1. **Event Cloning**: Sync events do not support cloning due to libp2p limitations
2. **Native Only**: Sync is only available with the `native` feature (not WASM)
3. **JSON Overhead**: Uses JSON instead of binary for better compatibility

## Future Work

1. **Request Handlers**: Implement specific handlers for each request type
2. **Sync Manager Integration**: Wire sync manager into Netabase struct
3. **Auto-sync**: Implement automatic background synchronization
4. **Metrics**: Add sync performance metrics
5. **Compression**: Consider adding compression for large messages
6. **Binary Protocol**: Optionally use a more efficient binary protocol

## Testing

To test the sync protocol:

```bash
cargo test --features native sync
```

## Configuration Options

Sync can be configured via `NetabaseConfig`:

```rust
use netabase::network::config::{NetabaseConfig, SyncConfig};

let config = NetabaseConfig {
    sync: SyncConfig {
        enabled: true,
        gossip: GossipConfig { /* ... */ },
        brb: BrbConfig { /* ... */ },
        sybil_resistance: SybilResistanceConfig { /* ... */ },
        paxos: PaxosConfig { /* ... */ },
        auto_sync: true,
        sync_interval: Duration::from_secs(30),
    },
    // ... other config ...
};
```

## Related Files

- `src/sync/protocol.rs` - Protocol message definitions
- `src/sync/codec.rs` - Request/response codec
- `src/sync/mod.rs` - Sync module exports
- `src/network/behaviour/mod.rs` - Behavior integration
- `src/network/behaviour/clone_impl.rs` - Event cloning support
- `src/network/swarm/handlers/swarm_events/behaviour/mod.rs` - Event handling

## See Also

- [SYNC_INTEGRATION_GUIDE.md](./SYNC_INTEGRATION_GUIDE.md) - General sync integration guide
- [SYNC_IMPLEMENTATION.md](./SYNC_IMPLEMENTATION.md) - Implementation details
- [SYNC_INTEGRATION_SUMMARY.md](./SYNC_INTEGRATION_SUMMARY.md) - Integration summary
