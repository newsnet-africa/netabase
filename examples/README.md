# Netabase Mock Network Examples

This directory contains mock network implementations that demonstrate the Netabase networking protocol using actual protocol state machines with in-memory channels for transport.

## Examples

### ⭐ 1. `mock_network_protocol.rs` - **RECOMMENDED** - Complete Protocol with Real State Machines

**Status**: Complete and working  
**Purpose**: Demonstrates the full Netabase protocol using actual protocol state machines from the core crate

**What it demonstrates**:
- Handshake protocol with version negotiation using `HandshakeStateMachine`
- Capability-based authorization with `Capability` types
- Secure query execution using `QueryHandler`
- Data synchronization with `SyncHandler` and fingerprints
- Session management with `SessionManager`
- Clean disconnect handling

**How it works**:
- Creates two mock nodes with Tokio channels as transport
- Uses actual protocol state machines from `netabase::protocol`
- Walks through all protocol phases
- Demonstrates proper error handling

**Run it:**
```bash
cargo run --example mock_network_protocol
```

**Key takeaways**:
- Shows how protocol state machines are used
- Demonstrates capability creation and validation
- Illustrates query authorization flow
- Reference implementation for libp2p integration

### 2. `mock_network_simple.rs` - Basic Communication

A minimal example showing:
- Two nodes communicating via channels
- Basic message passing
- Data query and replication
- Store integration

**Run it:**
```bash
cargo run --example mock_network_simple
```

### 3. `mock_network_full.rs` - Legacy Complete Protocol

**Note**: This example reimplements protocol logic rather than using the core state machines. Use `mock_network_protocol.rs` instead for the canonical implementation.

A comprehensive example demonstrating all 5 phases of the Netabase protocol:

#### Phase 1: Handshake
- Connection establishment
- Protocol version negotiation
- Schema compatibility verification
- Nonce-based replay protection

#### Phase 2: Capability Exchange
- Request/Grant capability flow
- Permission verification (Read, Write)
- Expiry support
- Multi-operation capabilities

#### Phase 3: Query Protocol
- Secure query execution
- Capability-based authorization
- Model filtering
- Result serialization/deserialization

#### Phase 4: Write Protocol
- Distributed writes
- Write capability verification
- Data propagation
- Acknowledgement system

#### Phase 5: Sync Protocol
- Range-based synchronization
- Timestamp filtering
- Efficient bulk transfer
- Incremental updates

**Run it:**
```bash
cargo run --example mock_network_full
```

## Protocol Flow (from `mock_network_protocol.rs`)

### Phase 1: Handshake
```
Node 1                          Node 2
  |                               |
  |--HandshakeRequest------------>|
  |                               |- Check version
  |                               |- Check schema
  |<--HandshakeResponse-----------|
  |- Verify accepted              |
```

### Phase 2: Capability Exchange
```
Node 1                          Node 2
  |                               |
  |- Grant capability to Node 2   |
  |   (Read access to /users/*)   |
```

### Phase 3: Data Operations
```
Node 1                          Node 2
  |- Insert data                  |
  |                               |
  |<--SecureQuery-----------------|
  |- Validate capability          |
  |- Validate nonce               |
  |- Validate timestamp           |
  |--QueryResponse--------------->|
```

### Phase 4: Synchronization
```
Node 1                          Node 2
  |                               |
  |<--SyncRequest-----------------|
  |- Calculate fingerprint        |
  |- Compare with remote          |
  |--SyncResponse---------------->|
  |  (Full sync or incremental)   |
```

### Phase 5: Disconnect
```
Node 1                          Node 2
  |                               |
  |--DisconnectMessage----------->|
  |- Clean up session             |- Clean up session
```

## Architecture

### Mock Network Components

```
┌─────────────┐         ┌─────────────┐
│  MockNode A │         │  MockNode B │
├─────────────┤         ├─────────────┤
│ NodeId      │         │ NodeId      │
│ Transport   │         │ Transport   │
│ LamportClock│         │ LamportClock│
│ Sessions    │         │ Sessions    │
│ Data Store  │         │ Data Store  │
└──────┬──────┘         └──────┬──────┘
       │                       │
       │    ┌──────────────┐   │
       └────┤   Channels   ├───┘
            │  (Mock Net)  │
            └──────────────┘
```

### Protocol State Machines (Core Crate)

All protocol logic lives in `netabase::protocol`:
- `HandshakeStateMachine` - Manages handshake state
- `QueryHandler` - Validates and executes queries
- `SyncHandler` - Manages synchronization
- `SessionManager` - Tracks peer sessions

These are **transport-agnostic** - they don't know about libp2p, TCP, or channels.

### Message Types

All messages are variants of `ProtocolMessage<PK, SK, T>`:
- `HandshakeRequest/Response`
- `Query/QueryResponse`
- `Write/WriteResponse`
- `SyncRequest/SyncResponse`
- `GrantCapability`
- `Disconnect`

## Key Concepts Demonstrated

### 1. Transport-Agnostic Protocol

The core protocol state machines have no dependencies on the transport layer:
```rust
// Protocol state machine - works with any transport
let mut handshake = HandshakeStateMachine::new(node_id, version, features, schema);
let request = handshake.initiate(&mut clock);

// Transport sends the message (could be channels, libp2p, TCP, etc.)
transport.send(peer_id, ProtocolMessage::HandshakeRequest(request));
```

### 2. Capability System
- Fine-grained permissions (Read, Write, Admin)
- Per-peer capability tracking
- Authorization checks before operations
- Capability delegation chains
- Range-restricted capabilities
- Time-limited expiry

Example:
```rust
let cap = Capability::new_root(
    grantor,
    grantee,
    Operation::Read,
    NDimensionalRange::new(...),
    expiry_timestamp,
);
```

### 3. Secure Queries

All queries include:
```rust
SecureQuery {
    range: NDimensionalRange,      // What data
    capability: Capability,         // Authorization
    nonce: u64,                    // Replay protection
    timestamp: u64,                // Freshness
    signature: CapabilitySignature, // Binding
}
```

Validation checks:
1. Nonce not reused (replay attack)
2. Timestamp within acceptable skew
3. Capability valid and not expired
4. Query range within capability scope
5. Signature verifies

### 4. Causality Tracking
- Lamport clocks for ordering events
- Clock updates on send/receive
- Tie-breaking with node IDs
- Merge semantics for distributed events

### 5. Session Management
- Connection state tracking
- Per-peer metadata
- Timeout detection
- Capability storage
- Lamport clock synchronization

## Differences from Real Network

These mock implementations **do not** include:
- Actual network transport (libp2p)
- Real cryptographic signatures (placeholders only)
- Certificate verification
- Connection pooling
- Network error handling
- Backpressure/flow control
- Peer discovery
- NAT traversal

## Building a Real Network

The mock examples use channels to simulate network transport. To build a real network:

1. **Replace MockTransport** with libp2p `NetworkBehaviour`
2. **Keep protocol state machines** - they're transport-agnostic
3. **Map network events** to state machine methods
4. **Use existing message types** for serialization

Example structure:
```rust
struct NetabaseBehaviour<PK, SK, T> {
    sessions: SessionManager<PK, SK>,
    handshakes: HashMap<PeerId, HandshakeStateMachine>,
    query_handler: QueryHandler<Store, PK, SK>,
    sync_handler: SyncHandler<PK, SK>,
}

impl NetworkBehaviour for NetabaseBehaviour {
    // Map libp2p events to protocol state machines
}
```

## Next Steps for Libp2p Integration

1. **Create libp2p NetworkBehaviour**
   - Implement `NetworkBehaviour` trait
   - Wire up protocol state machines
   - Handle connection lifecycle

2. **Request-Response Protocol**
   - Define codec for `ProtocolMessage`
   - Implement request-response handler
   - Map to existing query/sync handlers

3. **GossipSub Integration**
   - Subscription rooms
   - Area-of-interest announcements
   - Capability-based filtering

4. **Kademlia DHT**
   - Peer discovery
   - Capability advertisement
   - Schema compatibility tracking

5. **Connection Management**
   - Integrate with `SessionManager`
   - Timeout handling
   - Reconnection logic

## Testing

These examples serve as:
- **Integration tests** for protocol state machines
- **Protocol documentation** through executable code
- **Performance baselines** (no network overhead)
- **Reference implementation** for libp2p integration

Run all examples:
```bash
cargo run --example mock_network_protocol  # Recommended
cargo run --example mock_network_simple
cargo run --example mock_network_full      # Legacy

# Run tests
cargo test --lib

# Run with debug logging
RUST_LOG=debug cargo run --example mock_network_protocol
```

## Related Files

- `MIGRATION_COMPLETE.md` - Summary of completed refactoring
- `PLANNING.md` - Full protocol specification
- `HANDSHAKE_PROTOCOL.md` - Detailed handshake design
- `netabase/src/protocol/` - **Protocol state machines (NEW)**
- `netabase/src/network/protocol.rs` - Protocol message types
- `netabase/src/capabilities/` - Capability system
- `netabase/src/primitives/` - Core types (NodeId, LamportClock, etc.)

## Troubleshooting

### Example doesn't compile
- Ensure you're in the `netabase` directory
- Run `cargo clean && cargo build`

### Protocol seems to hang
- Check that channels are properly cross-wired
- Verify async tasks are spawned
- Enable debug logging: `RUST_LOG=debug`

### Tests failing
- Run `cargo test --lib` for library tests only
- Ensure all 43 library tests pass before running examples

