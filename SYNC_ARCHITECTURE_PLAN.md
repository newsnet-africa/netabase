# Netabase Sync Architecture Plan

## Executive Summary

This document outlines the comprehensive architecture for integrating Paxos consensus (via Paxakos) with the existing Netabase system, including macro-based code generation for bridging NetabaseModelTrait with libp2p's RecordStore, and flexible configuration for different deployment modes.

## Problem Statement

### Current Limitations

1. **RecordStore Opacity**: libp2p's `RecordStore` trait uses opaque byte records (`Vec<u8>`) that don't directly map to Netabase's type-safe `NetabaseModelTrait<D>` system
2. **Serialization Gap**: No automatic way to serialize NetabaseModelTrait instances for storage in RecordStore
3. **Incomplete Paxos Integration**: Framework exists but core State trait methods are unimplemented (`apply`, `cluster_at`, `freeze`)
4. **Configuration Complexity**: Need flexible modes for different deployment scenarios (DHT + Paxos hybrid vs standalone Paxos)
5. **Type Safety vs Generic Storage**: Need to maintain compile-time type safety while supporting generic log entries

## Proposed Architecture

### 1. Macro-Based RecordStore Generation

#### 1.1 NetabaseStore Proc Macro

**Purpose**: Automatically generate `RecordStore` implementations for each model variant in a NetabaseDefinition enum.

**Location**: New crate `netabase_macros/` (or add to existing `netabase_store` macros)

**Usage**:
```rust
#[derive(NetabaseStore)]
#[netabase_definition(BlogDefinition)]
pub enum BlogDefinition {
    User(User),
    Post(Post),
    Comment(Comment),
}
```

**Generated Code**:
```rust
// For each variant, generate a specialized RecordStore implementation
pub struct UserRecordStore {
    inner: Arc<netabase_store::SledStore<BlogDefinition>>,
}

impl RecordStore for UserRecordStore {
    fn get(&mut self, key: &RecordKey) -> Option<Record> {
        // 1. Decode RecordKey to UserPrimaryKey
        // 2. Query store for User model
        // 3. Serialize User to bytes
        // 4. Wrap in libp2p Record
    }

    fn put(&mut self, record: Record) -> Result<()> {
        // 1. Deserialize record.value to User
        // 2. Convert User to BlogDefinition::User
        // 3. Store using netabase_store API
    }

    // ... other RecordStore methods
}
```

#### 1.2 ModelRecordStore Wrapper Trait

**Purpose**: Bridge between `NetabaseModelTrait<D>` and `RecordStore`, providing a unified interface.

**Definition**:
```rust
/// Wrapper trait that adapts NetabaseModelTrait to RecordStore
pub trait ModelRecordStore<D: NetabaseDefinitionTrait> {
    type Model: NetabaseModelTrait<D>;

    /// Encode model instance to bytes for RecordStore
    fn encode_record(model: &Self::Model) -> Result<Vec<u8>>;

    /// Decode bytes from RecordStore to model instance
    fn decode_record(bytes: &[u8]) -> Result<Self::Model>;

    /// Convert model's primary key to RecordKey
    fn to_record_key(key: &<Self::Model as NetabaseModelTrait<D>>::PrimaryKey) -> RecordKey;

    /// Convert RecordKey back to model's primary key
    fn from_record_key(key: &RecordKey) -> Result<<Self::Model as NetabaseModelTrait<D>>::PrimaryKey>;
}

// Macro generates impl for each model
impl ModelRecordStore<BlogDefinition> for User {
    type Model = User;

    fn encode_record(model: &User) -> Result<Vec<u8>> {
        bincode::encode_to_vec(model, bincode::config::standard())
            .map_err(|e| anyhow!("Encoding failed: {}", e))
    }

    fn decode_record(bytes: &[u8]) -> Result<User> {
        let (user, _) = bincode::decode_from_slice(bytes, bincode::config::standard())
            .map_err(|e| anyhow!("Decoding failed: {}", e))?;
        Ok(user)
    }

    fn to_record_key(key: &UserPrimaryKey) -> RecordKey {
        RecordKey::new(&bincode::encode_to_vec(key, bincode::config::standard()).unwrap())
    }

    fn from_record_key(key: &RecordKey) -> Result<UserPrimaryKey> {
        let (pk, _) = bincode::decode_from_slice(key.as_ref(), bincode::config::standard())?;
        Ok(pk)
    }
}
```

#### 1.3 NetabaseDefinition Router Function

**Purpose**: Type-safe routing of NetabaseDefinition variants to the appropriate tree/model store.

**Implementation**:
```rust
impl<D: NetabaseDefinitionTrait> Netabase<D> {
    /// Unwraps a NetabaseDefinition variant and routes to the correct tree
    pub fn route_definition(&self, definition: D) -> Result<()>
    where
        D: strum::IntoDiscriminant,
    {
        // Macro generates match for all variants
        match definition {
            D::User(user) => {
                self.put_record_internal(user)?;
            }
            D::Post(post) => {
                self.put_record_internal(post)?;
            }
            D::Comment(comment) => {
                self.put_record_internal(comment)?;
            }
        }
        Ok(())
    }

    /// Internal method that stores using the model's specific tree
    fn put_record_internal<M>(&self, model: M) -> Result<()>
    where
        M: NetabaseModelTrait<D>,
        D: From<M>,
    {
        // Use existing put_record logic
        // Store in model-specific tree in backend
        todo!()
    }
}
```

**Generated Macro Code**:
```rust
// This would be generated by #[derive(NetabaseStore)]
impl BlogDefinition {
    pub fn route_to_store<S>(&self, store: &mut S) -> Result<()>
    where
        S: ModelStore<BlogDefinition>,
    {
        match self {
            BlogDefinition::User(user) => store.put_user(user),
            BlogDefinition::Post(post) => store.put_post(post),
            BlogDefinition::Comment(comment) => store.put_comment(comment),
        }
    }
}
```

### 2. Paxakos Integration

#### 2.1 Complete State Trait Implementation

**File**: `src/network/behaviour/sync_behaviour/paxakos.rs`

**Implementation Strategy**:

```rust
impl<D> paxakos::State for Netabase<D>
where
    D: NetabaseDefinitionTrait + paxakos::LogEntry,
{
    type LogEntry = D;
    type Outcome = Result<QueryResult>;
    type Effect = (); // No side effects beyond state change
    type Error = NetabaseError;
    type Context = PaxosContext;

    fn apply(
        &mut self,
        log_entry: &Self::LogEntry,
        context: &mut Self::Context,
    ) -> Result<(Self::Outcome, Self::Effect), Self::Error> {
        // 1. Check if entry already applied (idempotency)
        if context.applied_entries.contains(&log_entry.id()) {
            return Ok((Ok(QueryResult::AlreadyApplied), ()));
        }

        // 2. Route the definition to the correct tree
        self.route_definition(log_entry.clone())?;

        // 3. Mark as applied
        context.applied_entries.insert(log_entry.id());

        // 4. Return success
        Ok((Ok(QueryResult::Success), ()))
    }

    fn cluster_at(&self, round: RoundNum) -> Cluster<NodeInfo> {
        // Return cluster membership for the given round
        // This enables dynamic membership changes

        // For initial implementation, use static cluster from config
        Cluster::new(
            self.config.paxos_config.cluster_members.clone(),
            self.config.paxos_config.quorum_size,
        )
    }

    fn freeze(&self, context: &mut Self::Context) -> Self::Frozen {
        // Create snapshot of current state
        FrozenState {
            applied_entries: context.applied_entries.clone(),
            last_applied_round: context.last_applied_round,
            // Snapshot the underlying store state
            store_snapshot: self.store.snapshot(),
        }
    }

    fn concurrency(&self) -> u32 {
        // Number of rounds that can be settled concurrently
        // Start with 1 for safety, increase later for performance
        1
    }
}

// Context for tracking applied entries (idempotency)
pub struct PaxosContext {
    pub applied_entries: HashSet<D::Id>,
    pub last_applied_round: RoundNum,
}

pub struct FrozenState<D> {
    pub applied_entries: HashSet<D::Id>,
    pub last_applied_round: RoundNum,
    pub store_snapshot: Vec<u8>, // Serialized store state
}
```

#### 2.2 LogEntry Implementation

**Strategy**: NetabaseDefinition enum becomes the LogEntry type.

```rust
// Add to macro generation
impl paxakos::LogEntry for BlogDefinition {
    type Id = blake3::Hash; // Use content-addressed ID

    fn id(&self) -> Self::Id {
        // Hash the serialized definition for unique ID
        let bytes = bincode::encode_to_vec(self, bincode::config::standard())
            .expect("Serialization should not fail");
        blake3::hash(&bytes)
    }
}
```

#### 2.3 Communicator Implementation

**File**: `src/network/behaviour/sync_behaviour/communicator.rs` (new)

**Strategy**: Use libp2p's request-response protocol with CBOR encoding.

```rust
use paxakos::Communicator;
use libp2p::request_response;

pub struct PaxosCommunicator {
    client: request_response::cbor::Behaviour<PaxosRequest, PaxosResponse>,
}

#[async_trait]
impl Communicator for PaxosCommunicator {
    type Node = NodeInfo;
    type Yea = Acceptance;
    type Nay = Rejection;
    type Abstain = Abstention;

    async fn send_prepare(
        &mut self,
        node: &Self::Node,
        round: RoundNum,
        coord: CoordNum,
    ) -> Result<PrepareResponse<Self::Yea, Self::Nay, Self::Abstain>> {
        let request = PaxosRequest::Prepare { round, coord };
        let response = self.send_request(node.peer_id(), request).await?;

        match response {
            PaxosResponse::Promise(promise) => Ok(PrepareResponse::Promise(promise)),
            PaxosResponse::Conflict(conflict) => Ok(PrepareResponse::Conflict(conflict)),
            PaxosResponse::Abstain => Ok(PrepareResponse::Abstain(Abstention)),
        }
    }

    async fn send_proposal(
        &mut self,
        node: &Self::Node,
        round: RoundNum,
        coord: CoordNum,
        entry: &D,
    ) -> Result<ProposalResponse<Self::Yea, Self::Nay>> {
        let request = PaxosRequest::Proposal {
            round,
            coord,
            entry: entry.clone(),
        };
        let response = self.send_request(node.peer_id(), request).await?;

        match response {
            PaxosResponse::Accept => Ok(ProposalResponse::Accept(Acceptance)),
            PaxosResponse::Reject(reason) => Ok(ProposalResponse::Reject(Rejection { reason })),
        }
    }

    async fn send_commit(
        &mut self,
        node: &Self::Node,
        round: RoundNum,
        coord: CoordNum,
    ) -> Result<()> {
        let request = PaxosRequest::Commit { round, coord };
        self.send_request(node.peer_id(), request).await?;
        Ok(())
    }
}
```

#### 2.4 Swarm Integration

**File**: `src/network/swarm/handlers/swarm_events/behaviour/paxos.rs` (new)

**Event Handlers**:
```rust
pub(crate) fn handle_paxos_event(
    event: request_response::Event<PaxosRequest, PaxosResponse>,
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    context: &mut PaxosContext,
) -> Result<()> {
    match event {
        request_response::Event::Message { peer, message } => {
            match message {
                request_response::Message::Request { request, channel, .. } => {
                    handle_paxos_request(request, channel, swarm, context)?;
                }
                request_response::Message::Response { response, .. } => {
                    // Handle response (update pending promises, etc.)
                }
            }
        }
        request_response::Event::OutboundFailure { peer, error, .. } => {
            log::warn!("Paxos outbound failure to {}: {:?}", peer, error);
        }
        request_response::Event::InboundFailure { peer, error, .. } => {
            log::warn!("Paxos inbound failure from {}: {:?}", peer, error);
        }
        _ => {}
    }
    Ok(())
}

fn handle_paxos_request(
    request: PaxosRequest,
    channel: ResponseChannel<PaxosResponse>,
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    context: &mut PaxosContext,
) -> Result<()> {
    let response = match request {
        PaxosRequest::Prepare { round, coord } => {
            // Check if we can promise
            if context.can_promise(round, coord) {
                context.record_promise(round, coord);
                PaxosResponse::Promise(Promise { round, coord })
            } else {
                PaxosResponse::Conflict(context.conflicting_coord(round))
            }
        }
        PaxosRequest::Proposal { round, coord, entry } => {
            // Check if we accepted the prepare
            if context.has_promised(round, coord) {
                context.accept_entry(round, entry);
                PaxosResponse::Accept
            } else {
                PaxosResponse::Reject("No promise for this coordinator".to_string())
            }
        }
        PaxosRequest::Commit { round, coord } => {
            // Commit the accepted entry
            if let Some(entry) = context.get_accepted(round, coord) {
                context.commit_entry(round, entry)?;
                PaxosResponse::Committed
            } else {
                PaxosResponse::Reject("No accepted entry".to_string())
            }
        }
    };

    swarm.behaviour_mut()
        .paxos
        .send_response(channel, response)
        .map_err(|_| NetabaseError::SendResponseFailed)?;

    Ok(())
}
```

### 3. Configuration Architecture

#### 3.1 Behavior Mode Enumeration

**File**: `src/network/config/mod.rs`

```rust
/// Defines how the network layer operates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviourMode {
    /// Kademlia DHT only (default, no consensus)
    KademliaOnly,

    /// Paxos consensus only (no DHT, direct peer-to-peer)
    PaxosOnly {
        cluster: Vec<PeerId>,
        quorum_size: usize,
    },

    /// Hybrid: Paxos on top of Kademlia
    /// DHT provides peer discovery, Paxos provides consensus
    Hybrid {
        /// Subset of nodes that participate in Paxos
        paxos_nodes: Vec<PeerId>,
        quorum_size: usize,
        /// Whether this node is a Paxos participant
        is_paxos_node: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub mode: BehaviourMode,
    pub enable_autofill: bool,      // Paxakos autofill decoration
    pub enable_heartbeat: bool,     // Paxakos heartbeat decoration
    pub enable_catch_up: bool,      // Paxakos catch-up decoration
    pub heartbeat_interval: Duration,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            mode: BehaviourMode::KademliaOnly,
            enable_autofill: true,
            enable_heartbeat: true,
            enable_catch_up: true,
            heartbeat_interval: Duration::from_secs(5),
        }
    }
}
```

#### 3.2 Updated NetabaseConfig

```rust
pub struct NetabaseConfig {
    pub node: NodeConfig,
    pub dht_discovery: DHTDiscoveryConfig,
    pub storage_backend: StorageBackend,
    pub sync: SyncConfig,  // NEW
}
```

#### 3.3 Conditional Behavior Construction

**File**: `src/network/behaviour/mod.rs`

```rust
#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour<D: NetabaseDefinitionTrait> {
    pub identify: identify::Behaviour,

    #[cfg(feature = "native")]
    pub mdns: mdns::tokio::Behaviour,

    pub connection_limits: connection_limits::Behaviour,

    // Make these optional based on mode
    #[behaviour(ignore)]
    pub kad: Option<libp2p::kad::Behaviour<NetabaseStore<D>>>,

    #[behaviour(ignore)]
    pub paxos: Option<request_response::cbor::Behaviour<PaxosRequest, PaxosResponse>>,
}

impl<D: NetabaseDefinitionTrait> NetabaseBehaviour<D> {
    pub fn new(
        peer_id: PeerId,
        store: NetabaseStore<D>,
        config: &NetabaseConfig,
    ) -> Self {
        let (kad, paxos) = match &config.sync.mode {
            BehaviourMode::KademliaOnly => {
                let kad = libp2p::kad::Behaviour::new(peer_id.clone(), store);
                (Some(kad), None)
            }
            BehaviourMode::PaxosOnly { .. } => {
                let paxos = Self::create_paxos_behaviour();
                (None, Some(paxos))
            }
            BehaviourMode::Hybrid { is_paxos_node, .. } => {
                let kad = libp2p::kad::Behaviour::new(peer_id.clone(), store);
                let paxos = if *is_paxos_node {
                    Some(Self::create_paxos_behaviour())
                } else {
                    None
                };
                (Some(kad), paxos)
            }
        };

        Self {
            identify: identify::Behaviour::new(identify::Config::new(
                "/netabase/1.0.0".to_string(),
                peer_id.clone(),
            )),
            #[cfg(feature = "native")]
            mdns: mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                peer_id.clone(),
            ).unwrap(),
            connection_limits: connection_limits::Behaviour::new(
                connection_limits::ConnectionLimits::default(),
            ),
            kad,
            paxos,
        }
    }

    fn create_paxos_behaviour() -> request_response::cbor::Behaviour<PaxosRequest, PaxosResponse> {
        request_response::cbor::Behaviour::new(
            [(StreamProtocol::new("/netabase/paxos/1.0.0"), ProtocolSupport::Full)],
            request_response::Config::default(),
        )
    }
}
```

### 4. Deployment Modes

#### 4.1 Kademlia Only (Default)

**Use Case**: Decentralized P2P applications without strict consistency requirements.

**Configuration**:
```rust
let config = NetabaseConfig {
    sync: SyncConfig {
        mode: BehaviourMode::KademliaOnly,
        ..Default::default()
    },
    ..Default::default()
};
```

**Behavior**:
- Standard DHT operations
- No consensus overhead
- Eventually consistent
- High availability, partition tolerant

#### 4.2 Paxos Only

**Use Case**: Trusted cluster with strong consistency (e.g., internal services, server cluster).

**Configuration**:
```rust
let config = NetabaseConfig {
    sync: SyncConfig {
        mode: BehaviourMode::PaxosOnly {
            cluster: vec![peer1, peer2, peer3, peer4, peer5],
            quorum_size: 3,
        },
        ..Default::default()
    },
    ..Default::default()
};
```

**Behavior**:
- No DHT overhead
- Direct peer connections
- Linearizable consistency
- Requires quorum to operate

#### 4.3 Hybrid Mode (Recommended for Production)

**Use Case**: Public DHT with trusted consensus layer (e.g., main servers provide consistency, edge nodes use DHT).

**Configuration**:
```rust
// For main server nodes
let config_server = NetabaseConfig {
    sync: SyncConfig {
        mode: BehaviourMode::Hybrid {
            paxos_nodes: vec![server1, server2, server3],
            quorum_size: 2,
            is_paxos_node: true,  // This is a main server
        },
        ..Default::default()
    },
    ..Default::default()
};

// For edge/client nodes
let config_client = NetabaseConfig {
    sync: SyncConfig {
        mode: BehaviourMode::Hybrid {
            paxos_nodes: vec![server1, server2, server3],
            quorum_size: 2,
            is_paxos_node: false,  // This is NOT a Paxos participant
        },
        ..Default::default()
    },
    ..Default::default()
};
```

**Behavior**:
- DHT provides discovery and caching
- Paxos nodes form consensus cluster
- Edge nodes use DHT for reads
- Writes go through Paxos cluster
- Best of both worlds

### 5. Implementation Phases

#### Phase 1: Macro Infrastructure (Week 1-2)
1. Create `netabase_macros` crate or extend `netabase_store` macros
2. Implement `#[derive(NetabaseStore)]` proc macro
3. Generate `ModelRecordStore` implementations
4. Generate routing functions for NetabaseDefinition
5. Write macro unit tests

#### Phase 2: Paxakos Core (Week 2-3)
1. Implement `State` trait methods
2. Implement `LogEntry` for NetabaseDefinition
3. Create `PaxosContext` for tracking applied entries
4. Implement `freeze` and snapshot mechanism
5. Write Paxos core unit tests

#### Phase 3: Communication Layer (Week 3-4)
1. Implement `Communicator` trait
2. Create message types (Prepare, Proposal, Commit)
3. Add swarm event handlers for Paxos
4. Integrate with existing event loop
5. Test inter-node communication

#### Phase 4: Configuration & Modes (Week 4-5)
1. Add `SyncConfig` to configuration system
2. Implement conditional behavior construction
3. Support all three modes (Kademlia, Paxos, Hybrid)
4. Add mode switching tests
5. Create example applications for each mode

#### Phase 5: Integration & Testing (Week 5-6)
1. End-to-end tests with multiple nodes
2. Byzantine fault tolerance testing
3. Performance benchmarking
4. Documentation and examples
5. Migration guide for existing users

### 6. API Examples

#### 6.1 Using Macro-Generated Code

```rust
#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod blog {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode)]
    #[netabase(BlogDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
    }

    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode)]
    #[netabase(BlogDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub author_id: u64,
        pub content: String,
    }
}

// Macro generates:
// - BlogDefinition enum with User(User) and Post(Post) variants
// - ModelRecordStore<BlogDefinition> for User
// - ModelRecordStore<BlogDefinition> for Post
// - route_definition() method on BlogDefinition
// - RecordStore implementation for each model type
```

#### 6.2 Paxos-Backed Writes

```rust
// Initialize with Paxos
let config = NetabaseConfig {
    sync: SyncConfig {
        mode: BehaviourMode::PaxosOnly {
            cluster: vec![peer1, peer2, peer3],
            quorum_size: 2,
        },
        ..Default::default()
    },
    ..Default::default()
};

let netabase = Netabase::<BlogDefinition>::new_with_config(config)?;
netabase.start_swarm().await?;

// Put record - goes through Paxos consensus
let user = User { id: 1, name: "Alice".to_string() };
let result = netabase.put_record(user).await?;
// This internally:
// 1. Converts User to BlogDefinition::User
// 2. Proposes to Paxos cluster
// 3. Achieves consensus across quorum
// 4. Applies to all nodes' stores
// 5. Returns when committed

// Get record - reads from local store
let key = UserKey::Primary(UserPrimaryKey(1));
let user = netabase.get_record::<User>(key).await?;
```

#### 6.3 Hybrid Mode Usage

```rust
// Main server configuration
let config = NetabaseConfig {
    sync: SyncConfig {
        mode: BehaviourMode::Hybrid {
            paxos_nodes: vec![server1, server2, server3],
            quorum_size: 2,
            is_paxos_node: true,
        },
        ..Default::default()
    },
    ..Default::default()
};

let netabase = Netabase::<BlogDefinition>::new_with_config(config)?;

// Writes go through Paxos
netabase.put_record(user).await?;  // Consensus

// Reads use local store
netabase.get_record(key).await?;  // Fast local read

// DHT still available for discovery
netabase.get_providers(key).await?;  // Find who has this data
```

### 7. Testing Strategy

#### 7.1 Unit Tests
- Macro-generated code correctness
- Serialization/deserialization round trips
- Individual trait implementations

#### 7.2 Integration Tests
- Multi-node Paxos consensus
- Hybrid mode operation
- Mode switching
- Failure recovery

#### 7.3 Property-Based Tests (QuickCheck)
- Linearizability of Paxos writes
- Eventually consistency of DHT
- Idempotency of apply operations

#### 7.4 Performance Tests
- Throughput comparison: Kademlia vs Paxos vs Hybrid
- Latency under load
- Memory usage with large datasets

### 8. Documentation Requirements

#### 8.1 API Documentation
- Macro attributes and generated code
- Configuration options for each mode
- Migration guide from Kademlia-only

#### 8.2 Examples
- Simple Paxos cluster
- Hybrid deployment architecture
- Custom LogEntry types

#### 8.3 Architecture Diagrams
- Component interaction
- Message flow for consensus
- Deployment topologies

### 9. Open Questions & Future Work

1. **Snapshot Storage**: Where to persist frozen state? Separate file or integrated with store?
2. **Membership Changes**: How to handle dynamic cluster membership safely?
3. **Cross-Shard Transactions**: Can Paxos support multi-model transactions?
4. **Byzantine Fault Tolerance**: Should we integrate BRB from the sync module?
5. **Performance Tuning**: What's the optimal concurrency level for `State::concurrency()`?

### 10. Success Criteria

- [ ] Macro generates valid RecordStore implementations
- [ ] Paxos cluster achieves consensus on writes
- [ ] All three modes (Kademlia, Paxos, Hybrid) functional
- [ ] Zero runtime overhead when Paxos disabled
- [ ] Maintains existing API compatibility
- [ ] Documentation complete with examples
- [ ] Test coverage > 80%
- [ ] No performance regression in Kademlia-only mode

## Conclusion

This architecture provides a flexible, type-safe foundation for adding Paxos consensus to Netabase while preserving the existing DHT functionality. The macro-based approach ensures minimal boilerplate, and the configuration system allows users to choose the right consistency/availability tradeoff for their use case.
