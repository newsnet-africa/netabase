# Netabase Architecture

This document describes the internal architecture of the Netabase peer-to-peer database system, including its advanced Byzantine fault-tolerant synchronization protocols.

## Overview

Netabase is a distributed database that combines local storage with peer-to-peer networking and Byzantine fault-tolerant synchronization. It provides a type-safe API for storing and querying data both locally and across a network of peers using libp2p and Kademlia DHT, with optional consensus protocols for critical operations.

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
│  │  - Coordinates sync protocols (if enabled)      │   │
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
│  │  ┌──────────────────────────────────────┐     │   │
│  │  │    Sync Behavior (if enabled)        │     │   │
│  │  │  - Paxos Consensus                   │     │   │
│  │  │  - Byzantine Reliable Broadcast      │     │   │
│  │  │  - Gossip/Anti-Entropy               │     │   │
│  │  │  - Sybil Resistance (PoW)            │     │   │
│  │  │  - Reputation System                 │     │   │
│  │  └──────────────────────────────────────┘     │   │
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
- Sync configuration and initialization

**Key Fields**:
- `command_sender`: Sends commands to the background swarm task
- `broadcast_receiver`: Template for creating new event subscribers
- `swarm_thread`: Handle to the background task
- `config`: Configuration options (including sync settings)
- `database_path`: Optional custom database location

**Entry Points** (Public API Methods):

#### Lifecycle Management
- **`new()`** (`src/lib.rs:449-464`): Create Netabase instance
  - Initializes database
  - Does NOT start network (call `start_swarm()` to activate)

- **`new_with_path_and_config()`** (`src/lib.rs:472-489`): Create with custom path and config
  - **When to use**: When you need sync enabled or custom storage location
  - Accepts `NetabaseConfig` with sync settings

- **`start_swarm()`** (`src/lib.rs:497-520`): Start P2P network
  - **Called**: After creating Netabase instance, before network operations
  - Spawns background task
  - Creates libp2p swarm with configured behaviors
  - Initializes sync protocols if `config.sync.enabled = true`
  - **Returns**: Immediately, network runs in background

- **`stop_swarm()`** (`src/lib.rs:526-542`): Stop network gracefully
  - Sends shutdown command
  - Waits for background task to finish
  - Cleans up resources

#### Data Operations (DHT)
- **`put_record(model)`** (`src/lib.rs:549-571`): Store data
  - **Called by**: Application when saving data
  - Converts model to bytes
  - Sends `PutRecord` command to background task
  - Stores locally AND publishes to DHT
  - **With sync**: May trigger Paxos consensus or gossip broadcast
  - **Returns**: `Result<()>` when operation completes

- **`get_record(key)`** (`src/lib.rs:579-603`): Retrieve data
  - **Called by**: Application when fetching data
  - Queries DHT (checks local first, then network)
  - **Returns**: `Result<Option<Model>>`

- **`delete_record(key)`** (`src/lib.rs:611-635`): Delete data
  - Removes from local store only (DHT is immutable)

- **`query_local_records()`** (`src/lib.rs:663-684`): Query local database
  - **Called by**: Application for local-only queries
  - No network I/O
  - Fast, synchronous operation

#### Network Operations
- **`start_providing(key)`** (`src/lib.rs:692-711`): Advertise as provider
  - **Called by**: Application to announce data availability

- **`get_providers(key)`** (`src/lib.rs:719-741`): Find data providers
  - **Called by**: Application to discover peers with data

- **`bootstrap()`** (`src/lib.rs:749-765`): Join DHT network
  - **Called by**: Application after `start_swarm()` to discover peers
  - Required for DHT participation

#### Event Subscription
- **`subscribe_to_broadcasts()`** (`src/lib.rs:773-778`): Subscribe to network events
  - **Called by**: Application to monitor network activity
  - **Returns**: `Receiver<NetabaseSwarmEvent>` for async event stream
  - Multiple subscribers supported

**Thread Model**: Lives in the main application thread, but operations are non-blocking through channels.

### 2. Background Swarm Task (`src/network/swarm/handlers/mod.rs`)

**Purpose**: Runs the libp2p swarm event loop in a separate async task.

**Responsibilities**:
- Processing commands from the application
- Handling libp2p swarm events
- Broadcasting events to subscribers
- Managing network I/O
- Coordinating sync protocols

**Event Loop** (`src/network/swarm/handlers/mod.rs:40-74`):
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

            // Sync protocols react to events here
            // - Paxos handles consensus messages
            // - BRB processes INIT/ECHO/READY/DELIVER
            // - Gossip triggers anti-entropy rounds
        }
    }
}
```

**When Called**: Continuously runs after `start_swarm()` is called, until `stop_swarm()` shuts it down.

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
- Behavior events (Kad, mDNS, Identify, Sync)
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
- **Sync Behavior** (optional): Byzantine fault-tolerant sync protocols

**Store Integration**: Kademlia uses `NetabaseStore` as its RecordStore implementation, bridging the DHT with local storage.

**Behavior Composition** (`src/network/behaviour/mod.rs:46-76`):
```rust
#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour {
    pub kademlia: Kademlia<NetabaseStore>,
    #[cfg(feature = "native")]
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub connection_limits: ConnectionLimits,
    #[cfg(feature = "native")]
    pub sync: request_response::cbor::Behaviour<SyncRequest, SyncResponse>,
}
```

**When Initialized**: When `start_swarm()` is called.

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
  - `sync`: SyncConfig (sync protocol settings)
  - `dht_discovery`: DHTDiscoveryConfig
  - `mdns_discovery`: MDNSDiscoveryConfig
  - `storage_backend`: StorageBackend

- **`SyncConfig`** (`src/network/config/mod.rs:153-168`): Sync protocol configuration
  - `enabled: bool` - Master sync toggle
  - `gossip: GossipConfig` - Anti-entropy settings
  - `brb: BrbConfig` - Byzantine Reliable Broadcast
  - `sybil_resistance: SybilResistanceConfig` - PoW challenges
  - `paxos: PaxosConfig` - Consensus protocol
  - `auto_sync: bool` - Automatic sync on updates
  - `sync_interval: Duration` - How often to sync

**Design**: Simple struct-based configuration with sensible defaults.

## Sync Protocol Integration

### Architecture Overview

The sync system provides Byzantine fault-tolerant data synchronization through multiple protocols:

```
┌──────────────────────────────────────────────────────────┐
│                  Sync Protocol Stack                      │
├──────────────────────────────────────────────────────────┤
│                                                           │
│  Layer 4: Application Integration (NetabaseWithSync)     │
│  ┌─────────────────────────────────────────────────┐    │
│  │ - Paxos helper methods                          │    │
│  │ - Reputation tracking                           │    │
│  │ - Challenge/response system                     │    │
│  └─────────────────────────────────────────────────┘    │
│                         │                                │
│                         ▼                                │
│  Layer 3: Protocol Managers (SyncBehaviorManager)        │
│  ┌─────────────────────────────────────────────────┐    │
│  │ - PaxosInstance (consensus)                     │    │
│  │ - BrbManager (reliable broadcast)               │    │
│  │ - GossipManager (anti-entropy)                  │    │
│  │ - ChallengeSystem (sybil resistance)            │    │
│  └─────────────────────────────────────────────────┘    │
│                         │                                │
│                         ▼                                │
│  Layer 2: Core Protocols                                 │
│  ┌─────────────────────────────────────────────────┐    │
│  │ - Vector Clocks (causality)                     │    │
│  │ - Merkle Trees (state comparison)               │    │
│  │ - Proof-of-Work (challenge verification)        │    │
│  │ - Reputation System (peer scoring)              │    │
│  └─────────────────────────────────────────────────┘    │
│                         │                                │
│                         ▼                                │
│  Layer 1: Network Transport (libp2p)                     │
│  ┌─────────────────────────────────────────────────┐    │
│  │ - Request/Response protocol                     │    │
│  │ - Message serialization (CBOR)                  │    │
│  │ - Peer-to-peer communication                    │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
└──────────────────────────────────────────────────────────┘
```

### Sync Manager (`src/sync/mod.rs`)

**Purpose**: Central coordinator for all sync protocols.

**Structure** (`src/sync/mod.rs:146-161`):
```rust
pub struct SyncManager {
    config: SyncConfig,
    local_peer_id: PeerId,
    status: SyncStatus,
    vector_clock: VectorClock,
    peer_states: HashMap<PeerId, PeerSyncState>,
    pending_syncs: VecDeque<SyncOperation>,
}
```

**Lifecycle Methods**:
- **`new(config, peer_id)`** (`src/sync/mod.rs:165-175`): Create manager
  - **Called by**: `NetabaseWithSync::new()`
  - Initializes vector clock and peer tracking

- **`start()`** (`src/sync/mod.rs:176-178`): Activate sync
  - Sets status to `Active`
  - Begins processing sync operations

- **`stop()`** (`src/sync/mod.rs:181-183`): Deactivate sync
  - Sets status to `Idle`
  - Stops new sync operations

**State Management**:
- **`tick()`** (`src/sync/mod.rs:196-198`): Advance vector clock
  - **Called by**: Application on each local update

- **`update_peer_state()`** (`src/sync/mod.rs:206-208`): Track peer state
  - **Called by**: When receiving peer updates

- **`remove_peer()`** (`src/sync/mod.rs:211-213`): Clean up peer
  - **Called by**: On peer disconnect

### Paxos Consensus (`src/sync/paxos/mod.rs`)

**Purpose**: Byzantine fault-tolerant consensus for critical operations.

**When Used**: When `config.paxos.enabled = true` and strong consistency is required.

**Key Structures**:

**`PaxosInstance`** (`src/sync/paxos/mod.rs:157-175`):
```rust
pub struct PaxosInstance {
    local_peer_id: PeerId,
    config: PaxosConfig,
    acceptor: AcceptorState,
    proposals: HashMap<ProposalNumber, ProposerState>,
    learned_values: Vec<Vec<u8>>,  // Consensus history
    current_round: u64,
}
```

**Entry Points**:

1. **`propose(value)`** (`src/sync/paxos/mod.rs:191-208`): Initiate consensus
   - **Called by**: Application for critical operations
   - **Returns**: `ProposalNumber` for tracking
   - **Flow**:
     ```
     1. Increment round number
     2. Create ProposalNumber(round, peer_id)
     3. Store proposal state
     4. Send PREPARE to acceptors
     5. Wait for PROMISE quorum (f+1 responses)
     ```

2. **`handle_prepare(proposal)`** (`src/sync/paxos/mod.rs:209-231`): Accept or reject proposal
   - **Called by**: On receiving PREPARE message
   - **When**: Acting as acceptor
   - **Returns**: `PROMISE` or `NACK`
   - **Logic**:
     ```rust
     if proposal > self.promised_proposal {
         self.promised_proposal = proposal;
         Ok(PaxosMessage::Promise {
             proposal_number,
             accepted_proposal: self.accepted_proposal,
             accepted_value: self.accepted_value,
         })
     } else {
         Err(anyhow!("Already promised higher proposal"))
     }
     ```

3. **`handle_promise()`** (`src/sync/paxos/mod.rs:232-273`): Collect quorum
   - **Called by**: Proposer on receiving PROMISE
   - **When**: Building consensus
   - **Returns**: `Some(ACCEPT)` when quorum reached, `None` otherwise
   - **Flow**:
     ```
     1. Store promise
     2. Check if quorum reached (f+1)
     3. Select value (highest accepted or proposed)
     4. Return ACCEPT message
     ```

4. **`handle_accept()`** (`src/sync/paxos/mod.rs:275-297`): Accept value
   - **Called by**: On receiving ACCEPT message
   - **When**: Acting as acceptor after PROMISE
   - **Returns**: `ACCEPTED` message

5. **`handle_accepted()`** (`src/sync/paxos/mod.rs:299-323`): Learn value
   - **Called by**: On receiving ACCEPTED message
   - **When**: Value reaches consensus
   - **Side effects**: Adds to `learned_values` history

**Consensus Flow Example**:
```
Proposer (Node A):
  1. propose(value="message_1")
     → ProposalNumber(1, NodeA)
     → Send PREPARE to all acceptors

Acceptors (Nodes B, C, D):
  2. handle_prepare(1, NodeA)
     → Check if higher than promised
     → Return PROMISE + any accepted value

Proposer (Node A):
  3. handle_promise(NodeB, ...)
  4. handle_promise(NodeC, ...)  ← Quorum reached (f+1)
     → Select value
     → Send ACCEPT to all acceptors

Acceptors (Nodes B, C, D):
  5. handle_accept(value="message_1")
     → Store accepted value
     → Return ACCEPTED

All Nodes:
  6. handle_accepted(value="message_1")
     → Add to learned_values[]
     → Consensus achieved!
```

**Integration with Netabase**:
```rust
// In NetabaseWithSync (src/sync/netabase_ext.rs)

/// Propose value through Paxos
pub fn paxos_propose(&mut self, value: Vec<u8>) -> Option<ProposalNumber> {
    if let Some(paxos) = &self.paxos {
        let mut paxos = paxos.write().unwrap();
        Some(paxos.propose(value))
    } else {
        None
    }
}

/// Get consensus history
pub fn paxos_learned_values(&self) -> Vec<Vec<u8>> {
    if let Some(paxos) = &self.paxos {
        let paxos = paxos.read().unwrap();
        paxos.learned_values().to_vec()
    } else {
        Vec::new()
    }
}
```

### Byzantine Reliable Broadcast (`src/sync/brb/mod.rs`)

**Purpose**: Reliable message dissemination with Byzantine fault tolerance.

**When Used**: For critical broadcasts that must reach all honest peers despite Byzantine failures.

**Key Structure** (`src/sync/brb/mod.rs:60-71`):
```rust
pub struct BrbManager {
    local_peer_id: PeerId,
    config: BrbConfig,
    messages: HashMap<[u8; 32], BrbMessageState>,
    validator: BrbValidator,
}
```

**BRB Phases**:
```
INIT → ECHO → READY → DELIVER

Phase 1 (INIT): Sender broadcasts message
  ↓
Phase 2 (ECHO): Receivers echo to all peers
  ↓ (threshold: (n+f)/2 + 1)
Phase 3 (READY): Send READY when echo threshold reached
  ↓ (threshold: 2f+1)
Phase 4 (DELIVER): Deliver when ready threshold reached
```

**Entry Points**:

1. **`initiate_broadcast(payload, version)`** (`src/sync/brb/mod.rs:177-207`): Start broadcast
   - **Called by**: Node initiating broadcast
   - **Returns**: `(message_hash, peers_to_notify)`
   - **Flow**:
     ```
     1. Hash payload
     2. Create BrbMessageState
     3. Set phase to INIT
     4. Return peers to send INIT message
     ```

2. **`handle_init(from, hash, payload)`** (`src/sync/brb/mod.rs:209-239`): Receive initial message
   - **Called by**: On receiving INIT
   - **Returns**: `BrbAction::Echo` (peers to echo to)
   - **Flow**:
     ```
     1. Verify message hash
     2. Store message state
     3. Add INIT sender to echo quorum
     4. Return peers for ECHO phase
     ```

3. **`handle_echo(from, hash)`** (`src/sync/brb/mod.rs:241-275`): Collect echoes
   - **Called by**: On receiving ECHO
   - **Returns**: `BrbAction::Ready` when threshold reached
   - **Logic**:
     ```rust
     if state.echo_quorum.add_responder(*from) {
         if state.echo_quorum.has_reached_threshold() {
             state.phase = BrbPhase::Ready;
             return Ok(BrbAction::Ready(/* peers */));
         }
     }
     ```

4. **`handle_ready(from, hash, original_sender)`** (`src/sync/brb/mod.rs:277-318`): Collect ready messages
   - **Called by**: On receiving READY
   - **Returns**: `BrbAction::Deliver` when threshold reached
   - **Logic**:
     ```rust
     if state.ready_quorum.add_responder(*from) {
         if state.ready_quorum.has_reached_threshold() {
             state.phase = BrbPhase::Delivered;
             return Ok(BrbAction::Deliver(payload));
         }
     }
     ```

**Byzantine Fault Tolerance**:
- **n = 3f + 1**: Total nodes vs Byzantine faults
- **Echo threshold**: `(n+f)/2 + 1` ensures at least one honest node in any echo quorum
- **Ready threshold**: `2f+1` ensures two quorums intersect at honest node
- **Delivery guarantee**: If one honest node delivers, all honest nodes eventually deliver

**Example** (n=7, f=2):
```
Node A initiates broadcast:
  1. BrbManager::initiate_broadcast("message")
     → hash = blake3("message")
     → Send INIT to 6 peers

Nodes B,C,D,E,F,G receive:
  2. BrbManager::handle_init(A, hash, "message")
     → Verify hash matches
     → Send ECHO to all 7 nodes

All nodes collect echoes:
  3. BrbManager::handle_echo(B, hash)
     3a. BrbManager::handle_echo(C, hash)
     3b. BrbManager::handle_echo(D, hash)
     3c. BrbManager::handle_echo(E, hash)  ← Threshold (5) reached!
     → Send READY to all nodes

All nodes collect ready:
  4. BrbManager::handle_ready(B, hash)
     4a. BrbManager::handle_ready(C, hash)
     4b. BrbManager::handle_ready(D, hash)
     4c. BrbManager::handle_ready(E, hash)
     4d. BrbManager::handle_ready(F, hash)  ← Threshold (5) reached!
     → DELIVER message to application
```

### Gossip Protocol (`src/sync/gossip/mod.rs`)

**Purpose**: Anti-entropy state reconciliation between peers.

**When Used**: Periodically (every `config.gossip.interval`) to ensure eventual consistency.

**Key Structure** (`src/sync/gossip/mod.rs:68-73`):
```rust
pub struct GossipManager {
    config: GossipConfig,
    local_peer_id: PeerId,
    state_digests: HashMap<PeerId, StateDigest>,
}
```

**Entry Points**:

1. **`initiate_gossip_round(peers)`** (`src/sync/gossip/mod.rs:92-115`): Start anti-entropy
   - **Called by**: Timer/scheduler every `gossip.interval`
   - **Returns**: Peers selected for gossip (random fanout)
   - **Flow**:
     ```
     1. Select random peers (up to fanout count)
     2. Get local state digest
     3. Return peers to exchange digests with
     ```

2. **`compare_digests(local, remote)`** (`src/sync/gossip/mod.rs:117-139`): Find differences
   - **Called by**: After receiving peer's digest
   - **Returns**: Keys to request from peer
   - **Logic**:
     ```rust
     let mut missing_keys = Vec::new();
     for (key, remote_hash) in remote.merkle_roots {
         match local.merkle_roots.get(&key) {
             None => missing_keys.push(key),  // We don't have it
             Some(local_hash) if local_hash != remote_hash => {
                 missing_keys.push(key);  // We have different version
             }
             _ => {}  // Same version, skip
         }
     }
     ```

3. **`handle_sync_request(keys)`** (`src/sync/gossip/mod.rs:141-155`): Respond with records
   - **Called by**: On receiving sync request
   - **Returns**: Records for requested keys

4. **`handle_sync_response(records)`** (`src/sync/gossip/mod.rs:157-171`): Apply updates
   - **Called by**: On receiving records from peer
   - **Side effects**: Merges records into local store

**Gossip Round Example**:
```
Every 10 seconds (gossip.interval):

Node A:
  1. initiate_gossip_round([B, C, D, E, F])
     → Select random 3 peers: [B, D, F]
     → Get local digest: {key1: hash1, key2: hash2}
     → Send digest to B, D, F

Node B receives digest:
  2. compare_digests(local, remote)
     → Find: key3 missing in remote, key2 different
     → Send [key3, key2] to Node A

Node A receives request:
  3. handle_sync_request([key3, key2])
     → Load records from local store
     → Send records back to B

Node B receives response:
  4. handle_sync_response([record_key3, record_key2])
     → Merge into local store
     → Update vector clocks
     → Gossip round complete!
```

**Merkle Tree Integration**:
```rust
// State digest includes Merkle root for efficient comparison
pub struct StateDigest {
    pub peer_id: Vec<u8>,
    pub merkle_roots: HashMap<Vec<u8>, [u8; 32]>,  // key → merkle_root
    pub vector_clock: VectorClock,
    pub timestamp: u64,
}
```

### Sybil Resistance (`src/sync/pow.rs`, `src/sync/challenges.rs`)

**Purpose**: Prevent Sybil attacks through proof-of-work challenges.

**When Used**: When new peer connects and `config.sybil_resistance.enabled = true`.

**Entry Points**:

1. **`issue_challenge(peer)`** (`src/sync/challenges.rs:96-112`): Challenge new peer
   - **Called by**: On peer discovery
   - **Returns**: Challenge bytes (random nonce)
   - **Flow**:
     ```
     1. Generate random challenge data
     2. Store with timestamp
     3. Return challenge to peer
     4. Wait for proof-of-work response
     ```

2. **`generate(challenge)`** (`src/sync/pow.rs:67-90`): Solve challenge
   - **Called by**: Peer solving challenge
   - **Returns**: `ProofOfWork` with nonce
   - **Algorithm**:
     ```rust
     loop {
         let data = [challenge, nonce.to_le_bytes()].concat();
         let hash = blake3::hash(&data);

         if count_leading_zeros(&hash) >= difficulty {
             return ProofOfWork { nonce, timestamp };
         }

         nonce += 1;
     }
     ```

3. **`verify_response(peer, proof)`** (`src/sync/challenges.rs:114-136`): Verify proof
   - **Called by**: On receiving proof from peer
   - **Returns**: `Result<()>` if valid
   - **Verification**:
     ```rust
     let challenge = self.pending.get(peer)?;
     let data = [challenge, proof.nonce.to_le_bytes()].concat();
     let hash = blake3::hash(&data);

     if count_leading_zeros(&hash) >= config.difficulty {
         self.verified.insert(peer, timestamp);
         Ok(())
     } else {
         Err(anyhow!("Invalid proof"))
     }
     ```

**Challenge/Response Flow**:
```
New peer connects:
  1. ChallengeSystem::issue_challenge(peer_id)
     → Generate random 32 bytes
     → Store in pending_challenges
     → Send to peer

Peer receives challenge:
  2. PoWSystem::generate(challenge_data)
     → Try nonces until hash has required leading zeros
     → Return ProofOfWork { nonce, timestamp }
     → Send back to challenger

Challenger verifies:
  3. ChallengeSystem::verify_response(peer_id, proof)
     → Recompute hash with nonce
     → Check leading zeros >= difficulty
     → Add to verified_peers if valid
     → Peer can now participate in sync
```

### Reputation System (`src/sync/reputation.rs`)

**Purpose**: Track peer reliability for selective synchronization.

**When Used**: Continuously during peer interactions.

**Entry Points**:

1. **`record_success(peer)`** (`src/sync/reputation.rs:48-58`): Increase reputation
   - **Called by**: After successful sync operation
   - **Logic**: `score = min(1.0, score + 0.05 × (1.0 - score))`

2. **`record_failure(peer)`** (`src/sync/reputation.rs:60-69`): Decrease reputation
   - **Called by**: After failed sync operation
   - **Logic**: `score = max(0.0, score - 0.1)`

3. **`reputation(peer)`** (`src/sync/reputation.rs:71-79`): Get current score
   - **Returns**: Score in range [0.0, 1.0]
   - Default for new peers: 0.5

4. **`top_peers(n)`** (`src/sync/reputation.rs:81-91`): Get best peers
   - **Called by**: When selecting peers for sync
   - **Returns**: Top n peers by reputation score

**Usage in Sync**:
```rust
// Select peers for gossip based on reputation
let all_peers = netabase.get_connected_peers();
let top_peers = reputation.top_peers(gossip_config.fanout);
gossip_manager.initiate_gossip_round(top_peers);

// After sync completes
if sync_successful {
    reputation.record_success(&peer_id);
} else {
    reputation.record_failure(&peer_id);
}
```

### Vector Clocks (`src/sync/clock.rs`)

**Purpose**: Track causality and detect concurrent events.

**When Used**: Every local update increments clock; used in all sync operations.

**Entry Points**:

1. **`increment()`** (`src/sync/clock.rs:35-38`): Advance local clock
   - **Called by**: On every local data update
   ```rust
   self.clock.entry(self.local_peer_id)
       .and_modify(|c| *c += 1)
       .or_insert(1);
   ```

2. **`merge(other)`** (`src/sync/clock.rs:40-46`): Merge peer's clock
   - **Called by**: On receiving update from peer
   - **Logic**: Take maximum for each peer
   ```rust
   for (peer_id, &timestamp) in &other.clock {
       self.clock.entry(*peer_id)
           .and_modify(|c| *c = (*c).max(timestamp))
           .or_insert(timestamp);
   }
   ```

3. **`happened_before(other)`** (`src/sync/clock.rs:48-56`): Check causality
   - **Returns**: `true` if this event happened before other
   ```rust
   let all_less_or_equal = self.clock.iter().all(|(peer, &ts)| {
       other.clock.get(peer).map_or(false, |&other_ts| ts <= other_ts)
   });
   let some_less = self.clock.iter().any(|(peer, &ts)| {
       other.clock.get(peer).map_or(false, |&other_ts| ts < other_ts)
   });
   all_less_or_equal && some_less
   ```

4. **`concurrent(other)`** (`src/sync/clock.rs:58-64`): Detect conflicts
   - **Returns**: `true` if events are concurrent (conflict)
   ```rust
   !self.happened_before(other) && !other.happened_before(self)
   ```

**Usage in Sync**:
```rust
// On local update
vector_clock.increment();
let record = SyncRecord {
    key,
    value,
    version: timestamp,
    vector_clock: vector_clock.clone(),
};

// On receiving remote update
if remote_clock.happened_before(&local_clock) {
    // Remote is older, ignore
} else if local_clock.happened_before(&remote_clock) {
    // Remote is newer, apply update
    vector_clock.merge(&remote_clock);
} else if remote_clock.concurrent(&local_clock) {
    // Conflict! Need conflict resolution
    resolve_conflict(local, remote);
}
```

### Integration Flow: Put Record with Sync

**Complete data flow when sync is enabled**:

```
Application Thread:
  1. netabase.put_record(message) [src/lib.rs:549]
     ↓
  2. Convert to ChatDefinition::Message enum
     ↓
  3. Serialize with bincode
     ↓
  4. Create oneshot channel for response
     ↓
  5. command_sender.send(PutRecord { key, value, responder })
     ↓

Background Task Thread:
  6. command_receiver.recv() → PutRecord [src/network/swarm/handlers/mod.rs:47]
     ↓
  7. handle_command_events(&mut swarm, command) [mod.rs:52]
     ↓
  8. put_record::handle(...) [command_events/put_record.rs:15]
     ↓
  9. swarm.behaviour_mut().kademlia.put_record(...)
     ↓
 10. NetabaseStore::put(record) [src/network/store.rs:85]
     ↓
 11. SledStore/RedbStore::insert(key, value) [actual disk write]
     ↓

IF sync.enabled:
 12. Increment vector clock [src/sync/clock.rs:35]
     ↓
 13. Create SyncRecord with vector clock
     ↓

 IF sync.paxos.enabled:
 14. PaxosInstance::propose(value) [src/sync/paxos/mod.rs:191]
     ↓
 15. Send PREPARE to acceptors via libp2p
     ↓
 16. Await PROMISE quorum (f+1 responses)
     ↓
 17. Send ACCEPT to acceptors
     ↓
 18. Await ACCEPTED quorum
     ↓
 19. Add to learned_values[] (consensus achieved)
     ↓

 IF sync.brb.enabled:
 20. BrbManager::initiate_broadcast(payload, version) [src/sync/brb/mod.rs:177]
     ↓
 21. Hash payload with blake3
     ↓
 22. Send INIT to all peers
     ↓
 23. Peers respond with ECHO (threshold: (n+f)/2 + 1)
     ↓
 24. Send READY when echo threshold reached
     ↓
 25. Peers respond with READY (threshold: 2f+1)
     ↓
 26. DELIVER when ready threshold reached
     ↓

 IF sync.gossip.enabled:
 27. [On next gossip interval]
     GossipManager::initiate_gossip_round() [src/sync/gossip/mod.rs:92]
     ↓
 28. Select random peers (fanout)
     ↓
 29. Exchange state digests (Merkle roots)
     ↓
 30. Identify missing/different records
     ↓
 31. Transfer records in batches
     ↓
 32. Merge vector clocks
     ↓

DHT Propagation:
 33. Kademlia::put_record() publishes to DHT
     ↓
 34. Find α closest peers to key
     ↓
 35. Send STORE_RECORD to each peer
     ↓
 36. Peers store in their local NetabaseStore
     ↓
 37. QueryResult returned to application
     ↓

Application Thread:
 38. responder.send(Ok(())) [command handler]
     ↓
 39. oneshot_receiver.await → Ok(())
     ↓
 40. Application continues
```

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
9. [IF SYNC ENABLED] Vector Clock: Increment local timestamp
        ↓
10. [IF SYNC ENABLED] Create SyncRecord with clock
        ↓
11. [IF PAXOS ENABLED] Initiate consensus (PREPARE → PROMISE → ACCEPT)
        ↓
12. [IF BRB ENABLED] Initiate broadcast (INIT → ECHO → READY → DELIVER)
        ↓
13. Kademlia: Publish to DHT network
        ↓
14. Command Handler: Send response via oneshot channel
        ↓
15. Netabase: Return result to application
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
9. [IF SYNC ENABLED] Check vector clock for causality
        ↓
10. [IF SYNC ENABLED] Merge clocks if remote is newer
        ↓
11. Kademlia: Return QueryResult
        ↓
12. Command Handler: Send response via oneshot channel
        ↓
13. Netabase: Return result to application
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

### Gossip Anti-Entropy Round

```
Timer triggers (every sync.gossip.interval):
  1. GossipManager::initiate_gossip_round()
        ↓
  2. Select random peers (gossip.fanout)
        ↓
  3. Compute local state digest (Merkle roots)
        ↓
  4. Send digest to selected peers via libp2p
        ↓

Remote peer receives digest:
  5. GossipManager::compare_digests(local, remote)
        ↓
  6. Find keys with different/missing hashes
        ↓
  7. Send sync request for those keys
        ↓

Local peer receives request:
  8. GossipManager::handle_sync_request(keys)
        ↓
  9. Load records from NetabaseStore
        ↓
  10. Send records back to requester
        ↓

Remote peer receives records:
  11. GossipManager::handle_sync_response(records)
        ↓
  12. For each record:
         a. Check vector clock causality
         b. Merge if newer or concurrent
         c. Update local store
        ↓
  13. Gossip round complete (state synchronized)
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
- Sync managers use `Arc<RwLock<T>>` for thread-safe shared state

## Error Handling

### Error Types

- `anyhow::Result<T>`: Most public API methods
- `NetabaseError`: Storage-layer errors
- `libp2p` errors: Wrapped in query results
- Sync errors: Consensus failures, Byzantine faults

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
[IF SYNC] Protocol error handling (retry, fallback)
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
6. **Lazy Consensus**: Paxos only when explicitly enabled
7. **Batched Gossip**: Transfer multiple records per round
8. **Merkle Trees**: Efficient state comparison (O(log n))

### Bottlenecks

1. **Channel Capacity**: Commands can block if buffer fills
2. **Event Broadcasting**: Slow subscribers may miss events
3. **Serialization**: Large records increase overhead
4. **Network Latency**: DHT operations depend on network conditions
5. **Consensus Overhead**: Paxos requires multiple network rounds
6. **PoW Computation**: Challenge solving is CPU-intensive

## Testing Strategy

### Unit Tests

- Individual command handlers
- Event handlers
- Serialization/deserialization
- Key generation
- Protocol correctness (Paxos, BRB, Gossip)

### Integration Tests

- Multi-node scenarios
- DHT operations
- Event subscription
- Error cases
- **Sync protocol tests** (`tests/sync_comprehensive.rs` - 31 tests)
- **Netabase entrypoint tests** (`tests/netabase_sync_integration.rs` - 16 tests)

### Multi-Process Tests

- **Network tests** (`tests/sync_orchestrator.rs` - 4 scenarios)
- **Nushell orchestration** (`run_sync_tests.nu`)
- Byzantine fault tolerance validation
- Consensus verification

### Example Tests

- `simple_mdns_chat.rs`: Full application example with sync
- Manual testing for peer discovery

## Configuration Examples

### Development (Fast, Low Security)
```rust
use netabase::network::config::{NetabaseConfig, SyncConfig};

let config = NetabaseConfig {
    sync: SyncConfig {
        enabled: true,
        gossip: GossipConfig {
            enabled: true,
            interval: Duration::from_secs(5),
            fanout: 2,
        },
        brb: BrbConfig { enabled: false, .. },
        paxos: PaxosConfig { enabled: false, .. },
        sybil_resistance: SybilResistanceConfig {
            enabled: true,
            pow_difficulty: 12,  // ~4ms per challenge
            ..Default::default()
        },
        ..Default::default()
    },
    ..Default::default()
};
```

### Production (Balanced)
```rust
let config = NetabaseConfig {
    sync: SyncConfig {
        enabled: true,
        gossip: GossipConfig {
            interval: Duration::from_secs(10),
            fanout: 3,
            ..Default::default()
        },
        brb: BrbConfig {
            enabled: true,
            total_peers: 7,
            max_faulty: 2,  // Tolerates 2 Byzantine faults
        },
        paxos: PaxosConfig { enabled: false, .. },
        sybil_resistance: SybilResistanceConfig {
            pow_difficulty: 20,  // ~1s per challenge
            ..Default::default()
        },
        ..Default::default()
    },
    ..Default::default()
};
```

### High Security (Paxos + BRB)
```rust
let config = NetabaseConfig {
    sync: SyncConfig {
        enabled: true,
        gossip: GossipConfig {
            interval: Duration::from_secs(15),
            fanout: 4,
            ..Default::default()
        },
        brb: BrbConfig {
            enabled: true,
            total_peers: 10,
            max_faulty: 3,  // Tolerates 3 Byzantine faults
        },
        paxos: PaxosConfig {
            enabled: true,
            num_acceptors: 7,
            max_failures: 3,
        },
        sybil_resistance: SybilResistanceConfig {
            pow_difficulty: 24,  // ~16s per challenge
            ..Default::default()
        },
        ..Default::default()
    },
    ..Default::default()
};
```

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

### Monitor Sync Protocols

```rust
// Check Paxos history
if sync_enabled && paxos_enabled {
    let learned = netabase.sync.paxos_learned_values();
    println!("Paxos consensus history: {} values", learned.len());
}

// Check peer reputation
for peer in connected_peers {
    let reputation = netabase.sync.peer_reputation(&peer);
    println!("Peer {} reputation: {:.2}", peer, reputation);
}

// Monitor gossip activity
println!("Last gossip round: {:?}", last_gossip_time);
println!("Peers synced: {}", synced_peer_count);
```

## Future Improvements

1. **Metrics**: Built-in performance monitoring
2. **Tracing**: Distributed tracing support
3. **Connection Pooling**: Reuse connections efficiently
4. **Adaptive Replication**: Smart data distribution
5. **WASM Support**: Complete browser integration
6. **Query Optimization**: Caching and batching
7. **Conflict Resolution**: Automatic CRDTs for concurrent updates
8. **Sharding**: Horizontal scaling for large datasets
9. **Compression**: Reduce network bandwidth usage

## Related Documentation

- [README.md](./README.md): User-facing documentation
- [SYNC_TESTS.md](./SYNC_TESTS.md): Sync protocol tests
- [NETABASE_SYNC_TESTS.md](./NETABASE_SYNC_TESTS.md): Integration tests
- [TESTING_SUMMARY.md](./TESTING_SUMMARY.md): Complete test overview
- [netabase_store/ARCHITECTURE.md](../netabase_store/ARCHITECTURE.md): Storage layer architecture
- [examples/](./examples/): Working examples
