# Netabase Architecture

This document provides a comprehensive technical overview of the Netabase architecture, explaining how data flows from user code through the P2P network layer, how storage backends are implemented, and how the type-safe API integrates with libp2p.

## Table of Contents

1. [High-Level Architecture](#high-level-architecture)
2. [Data Flow: User to Network](#data-flow-user-to-network)
3. [Public API](#public-api)
4. [Storage Backend Integration](#storage-backend-integration)
5. [Command and Event System](#command-and-event-system)
6. [Swarm Event Loop](#swarm-event-loop)
7. [Network Protocol Stack](#network-protocol-stack)
8. [Type System and Traits](#type-system-and-traits)

---

## High-Level Architecture

Netabase is a peer-to-peer networking layer built on libp2p with integrated type-safe storage. The architecture consists of several key layers:

```
┌─────────────────────────────────────────────────────────────┐
│                      User Application                        │
│  (Uses Netabase<MyDefinition> API)                          │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Netabase Public API                       │
│  • put_record<M>(model)    • query_local_records()          │
│  • get_record<K>(key)      • put_record_locally()           │
│  • start_swarm()           • subscribe_to_broadcasts()      │
└────────────────────────┬────────────────────────────────────┘
                         │ Commands sent via mpsc::channel
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   Swarm Event Loop                          │
│  (Spawned as async task in start_swarm())                  │
│  • Processes Commands from channel                          │
│  • Handles SwarmEvents from libp2p                          │
│  • Broadcasts events to subscribers                         │
└──────────┬──────────────────────┬───────────────────────────┘
           │                      │
           ▼                      ▼
┌──────────────────────┐  ┌──────────────────────────────┐
│  Command Handlers    │  │   Event Handlers             │
│  • PutRecord         │  │   • NewListenAddr            │
│  • GetRecord         │  │   • IncomingConnection       │
│  • Bootstrap         │  │   • Behaviour Events         │
│  • LocalStore        │  │     - Kademlia              │
└──────────┬───────────┘  │     - mDNS                  │
           │              │     - Identify              │
           ▼              └──────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                  libp2p Swarm & Behaviours                  │
│  • Kademlia DHT  • mDNS Discovery  • Identify              │
│  • Transport Layer (TCP, QUIC)                              │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│             Storage Backend (SledStore<D>)                  │
│  • Implements libp2p::kad::store::RecordStore              │
│  • Provides NetabaseTreeSync<D, M> via open_tree()        │
│  • Backed by netabase_store (Sled/Redb)                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Flow: User to Network

### 1. User Calls `put_record()`

When a user calls `netabase.put_record(my_model).await`, the following sequence occurs:

```rust
// User code
let user = User {
    id: 1,
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
};
netabase.put_record(user).await?;
```

**Step-by-Step Flow:**

#### Step 1: Public API (lib.rs:968)

```rust
pub async fn put_record<M: NetabaseModelTrait<D>>(
    &self,
    model: M,
) -> anyhow::Result<QueryResult>
where
    D: From<M>,  // Model converts to Definition enum
{
    // Convert model to Definition enum
    let definition = D::from(model);

    // Create oneshot channel for receiving response from swarm
    let (response_tx, response_rx) = oneshot::channel();

    // Construct command
    let command = Command::Kademlia(
        KademliaCommand::PutRecord {
            record: definition,
            response_channel: response_tx,
        }
    );

    // Send to swarm event loop via mpsc channel
    self.command_sender.send(command).await?;

    // Wait for response from swarm
    match response_rx.await? {
        Ok(result) => Ok(result),
        Err(e) => Err(anyhow::anyhow!("Put record failed: {:?}", e)),
    }
}
```

**Key Points:**
- Takes a model `M`, not the Definition enum `D`
- Automatically converts `M` to `D` via `D::from(model)`
- Returns `QueryResult` from libp2p Kademlia
- Async operation that awaits network completion

#### Step 2: Command Sent to Swarm Event Loop

The command travels through an `mpsc::channel<Command<D>>` to the swarm event loop running in a spawned tokio task (started in `start_swarm()`).

#### Step 3: Swarm Event Loop Receives Command (handlers/mod.rs)

```rust
pub(crate) async fn start_swarm_loop<D>(
    config: NetworkConfig,
    mut swarm: Swarm<NetabaseBehaviour<D>>,
    broadcast_sender: BroadcastSender,
    mut command_receiver: mpsc::Receiver<Command<D>>,
)
where
    D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static,
{
    loop {
        tokio::select! {
            // Handle incoming commands from user
            Some(command) = command_receiver.recv() => {
                handle_command_events(&mut swarm, command);
            }

            // Handle events from libp2p swarm
            event = swarm.next() => {
                if let Some(event) = event {
                    handle_swarm_events(&mut swarm, event, &broadcast_sender);
                }
            }
        }
    }
}
```

#### Step 4: Command Dispatch (handlers/command_events/mod.rs)

```rust
pub(crate) fn handle_command_events<D>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    command: Command<D>
)
where
    D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static,
{
    match command {
        Command::Kademlia(kad_command) => {
            handle_kademlia_command(swarm, kad_command);
        }
        Command::AddAddress { peer, address } => {
            swarm.behaviour_mut().kad.add_address(&peer, address);
        }
    }
}
```

#### Step 5: Kademlia Command Handler

```rust
fn handle_kademlia_command<D>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    command: KademliaCommand<D>
)
where
    D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static,
{
    match command {
        KademliaCommand::PutRecord { record, response_channel } => {
            handle_put_record(swarm, record, response_channel);
        }
        KademliaCommand::GetRecord { key, response_channel } => {
            handle_get_record(swarm, key, response_channel);
        }
        // ... other commands
    }
}

fn handle_put_record<D>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    record: D,
    response_channel: Sender<Result<QueryResult, Error>>,
)
where
    D: NetabaseDefinitionTrait + RecordStoreExt,
{
    // Convert Definition to libp2p::kad::Record using RecordStoreExt
    match record.to_record() {
        Ok(kad_record) => {
            // Call libp2p Kademlia API
            match swarm.behaviour_mut().kad.put_record(kad_record, Quorum::One) {
                Ok(_query_id) => {
                    // Query initiated successfully
                    // Response will be sent when query completes
                }
                Err(store_error) => {
                    // Send error back immediately
                    let _ = response_channel.send(Err(store_error));
                }
            }
        }
        Err(conversion_error) => {
            warn!("Failed to convert record: {:?}", conversion_error);
            let _ = response_channel.send(Err(kad::store::Error::MaxRecords));
        }
    }
}
```

#### Step 6: Record Conversion (via RecordStoreExt trait)

The `to_record()` method is provided by the `RecordStoreExt` trait:

```rust
impl RecordStoreExt for MyDefinition {
    fn to_record(&self) -> Result<libp2p::kad::Record, NetabaseError> {
        // Get the key from the definition
        let key = self.to_key()?;  // Returns MyKeys enum
        let key_bytes = key.to_ivec()?.to_vec();

        // Serialize the value
        let value_bytes = self.to_ivec()?.to_vec();

        Ok(libp2p::kad::Record {
            key: libp2p::kad::RecordKey::new(&key_bytes),
            value: value_bytes,
            publisher: None,
            expires: None,
        })
    }

    fn from_record(record: &libp2p::kad::Record) -> Result<Self, NetabaseError> {
        let ivec: IVec = record.value.clone().into();
        Self::from_ivec(&ivec)
    }
}
```

#### Step 7: Kademlia DHT Processes Put

libp2p's Kademlia implementation:
1. Stores the record in the local store (SledStore via `RecordStore` trait)
2. Initiates a DHT query to find closest peers to the key
3. Replicates the record to those peers
4. Returns a `QueryId` for tracking

#### Step 8: Query Completion Event

When the DHT query completes, libp2p emits a `KademliaEvent::OutboundQueryProgressed`:

```rust
fn handle_kad_event<D>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    event: KademliaEvent,
    broadcast_sender: &BroadcastSender,
)
where
    D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static,
{
    match event {
        KademliaEvent::OutboundQueryProgressed { id, result, .. } => {
            // Broadcast to subscribers
            let _ = broadcast_sender.send((
                SwarmEvent::Behaviour(NetabaseBehaviourEvent::Kademlia(event)),
                BehaviourWrapper::None,
            ));

            // Handle based on query result type
            match result {
                QueryResult::PutRecord(Ok(_)) => {
                    info!("Record stored successfully");
                }
                QueryResult::PutRecord(Err(e)) => {
                    warn!("Put record failed: {:?}", e);
                }
                // ... other result types
            }
        }
        // ... other events
    }
}
```

#### Step 9: Response Returns to User

The oneshot channel `response_rx` in step 1 receives the result, and `put_record()` returns to the user code.

---

## Public API

The `Netabase<D>` struct provides the main API for distributed database operations.

### Core Methods

#### `new()` / `new_with_path()`

```rust
/// Create a new Netabase instance with a temporary database
pub fn new() -> anyhow::Result<Self>

/// Create a new Netabase instance with a persistent database at the specified path
pub fn new_with_path(path: &str) -> anyhow::Result<Self>
```

#### `start_swarm()`

```rust
/// Start the P2P swarm (must be called before network operations)
pub async fn start_swarm(&mut self) -> anyhow::Result<()>
```

Initializes the libp2p swarm and begins listening for connections.

#### `put_record<M>()`

```rust
/// Store a model in the distributed hash table
pub async fn put_record<M: NetabaseModelTrait<D>>(
    &self,
    model: M,
) -> anyhow::Result<QueryResult>
where
    D: From<M>,
```

**Example:**
```rust
let user = User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() };
netabase.put_record(user).await?;
```

#### `get_record<K>()`

```rust
/// Retrieve a record from the distributed hash table by key
pub async fn get_record<K: NetabaseModelTraitKey<D>>(
    &self,
    key: K,
) -> anyhow::Result<QueryResult>
where
    D::Keys: From<K>,
```

**Example:**
```rust
let user_key = UserKey::Primary(UserPrimaryKey(1));
let result = netabase.get_record(user_key).await?;
```

#### `query_local_records()`

```rust
/// Query all records stored locally (not from network)
pub async fn query_local_records(&self, limit: Option<usize>) -> anyhow::Result<Vec<D>>
```

**Example:**
```rust
let local_records = netabase.query_local_records(None).await?;
for record in local_records {
    match record {
        MyDefinition::User(user) => println!("User: {}", user.name),
        MyDefinition::Post(post) => println!("Post: {}", post.title),
    }
}
```

#### `put_record_locally()`

```rust
/// Store a record only in local database (not distributed to network)
pub async fn put_record_locally(&self, record: D) -> anyhow::Result<()>
```

**Note:** This method takes `D` (the Definition enum), not a model.

#### `subscribe_to_broadcasts()`

```rust
/// Subscribe to network events
pub fn subscribe_to_broadcasts(&self) -> broadcast::Receiver<(SwarmEvent, BehaviourWrapper<D>)>
```

**Example:**
```rust
let mut events = netabase.subscribe_to_broadcasts();

while let Ok(event) = events.recv().await {
    match &event.0 {
        SwarmEvent::Behaviour(behaviour_event) => {
            // Handle behaviour events (Kademlia, mDNS, Identify)
        }
        SwarmEvent::NewListenAddr { address, .. } => {
            println!("Listening on {}", address);
        }
        // ... other events
    }
}
```

#### `stop_swarm()`

```rust
/// Stop the P2P swarm gracefully
pub async fn stop_swarm(&mut self) -> anyhow::Result<()>
```

### Local Database Access

For local-only operations without going through the network layer:

```rust
/// Access the underlying SledStore
pub fn store(&self) -> &SledStore<D>

/// Open a type-safe tree for a specific model
pub fn open_tree<M: NetabaseModelTrait<D>>(&self) -> SledTree<D, M>
```

**Example:**
```rust
use netabase_store::traits::tree::NetabaseTreeSync;

// Direct local access
let user_tree = netabase.store().open_tree::<User>();

// CRUD operations
user_tree.put(user.clone())?;
let retrieved = user_tree.get(UserPrimaryKey(1))?;

// Secondary key queries
let users_by_email = user_tree.get_by_secondary_key(
    UserSecondaryKeys::Email(UserEmailSecondaryKey("alice@example.com".to_string()))
)?;
```

---

## Storage Backend Integration

### SledStore as RecordStore

Netabase uses `SledStore<D>` as the storage backend, which implements both:

1. **`NetabaseTreeSync<D, M>`** - For type-safe local access via `open_tree()`
2. **`libp2p::kad::store::RecordStore`** - For Kademlia DHT integration

**File:** `netabase_store/src/databases/record_store/sled_impl.rs`

```rust
impl<D> libp2p::kad::store::RecordStore for SledStore<D>
where
    D: NetabaseDefinitionTrait + RecordStoreExt,
{
    type RecordsIter<'a> = SledRecordsIterator<'a>;
    type ProvidedIter<'a> = std::iter::Empty<Cow<'a, ProviderRecord>>;

    fn get(&self, key: &RecordKey) -> Option<Cow<'_, Record>> {
        // Raw bytes access for Kademlia
        let key_bytes = key.as_ref();
        self.db.get(key_bytes).ok()?.map(|value| {
            Cow::Owned(Record {
                key: key.clone(),
                value: value.to_vec(),
                publisher: None,
                expires: None,
            })
        })
    }

    fn put(&mut self, record: Record) -> libp2p::kad::store::Result<()> {
        // Store raw bytes from network
        let key_bytes = record.key.as_ref();
        let value_bytes = &record.value;
        self.db.insert(key_bytes, value_bytes)
            .map_err(|_| libp2p::kad::store::Error::MaxRecords)?;
        Ok(())
    }

    fn remove(&mut self, key: &RecordKey) {
        let _ = self.db.remove(key.as_ref());
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        SledRecordsIterator { inner: self.db.iter() }
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        std::iter::empty()
    }
}
```

### Dual Access Pattern

Users can access data in two ways:

**1. Network Operations (via Netabase API):**
```rust
// Distributes to DHT
netabase.put_record(user).await?;

// Queries DHT
let result = netabase.get_record(user_key).await?;
```

**2. Local Operations (via Tree API):**
```rust
// Local only
let tree = netabase.store().open_tree::<User>();
tree.put(user)?;
let user = tree.get(UserPrimaryKey(1))?;
```

Both methods work with the same underlying storage, but network operations additionally propagate changes to the DHT.

---

## Command and Event System

### Command Flow

Commands flow from user code → mpsc channel → swarm event loop → handler → libp2p

```rust
pub(crate) enum Command<D> {
    Kademlia(KademliaCommand<D>),
    AddAddress { peer: PeerId, address: Multiaddr },
}

pub(crate) enum KademliaCommand<D> {
    PutRecord {
        record: D,
        response_channel: Sender<Result<QueryResult, Error>>,
    },
    GetRecord {
        key: D::Keys,
        response_channel: Sender<Result<QueryResult, String>>,
    },
    LocalStore(LocalStoreCommand<D>),
    Bootstrap {
        response_channel: Sender<Result<BootstrapResult, String>>,
    },
}

pub(crate) enum LocalStoreCommand<D> {
    QueryRecords {
        limit: Option<usize>,
        response_channel: Sender<Result<Vec<D>, String>>,
    },
    PutRecordLocally {
        record: D,
        response_channel: Sender<Result<(), String>>,
    },
}
```

### Event Broadcasting

Network events are broadcast to subscribers:

```rust
// In start_swarm()
let (broadcast_sender, broadcast_receiver) = broadcast::channel(1000);

// In swarm event loop
handle_swarm_events(&mut swarm, event, &broadcast_sender);

// Broadcasting an event
fn handle_new_listen_addr(
    address: &Multiaddr,
    broadcast: &BroadcastSender,
) {
    info!("🎧 Listening on {}", address);
    let _ = broadcast.send((
        SwarmEvent::NewListenAddr { ... },
        BehaviourWrapper::None,
    ));
}

// User subscribes
let mut events = netabase.subscribe_to_broadcasts();
while let Ok(event) = events.recv().await {
    // Process event
}
```

---

## Swarm Event Loop

The swarm event loop is the heart of Netabase, managing all async operations:

**File:** `src/network/swarm/handlers/mod.rs`

```rust
pub(crate) async fn start_swarm_loop<D>(
    config: NetworkConfig,
    mut swarm: Swarm<NetabaseBehaviour<D>>,
    broadcast_sender: BroadcastSender,
    mut command_receiver: mpsc::Receiver<Command<D>>,
)
where
    D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static,
{
    info!("🚀 Netabase swarm loop started");

    loop {
        tokio::select! {
            // Priority 1: Handle user commands
            Some(command) = command_receiver.recv() => {
                handle_command_events(&mut swarm, command);
            }

            // Priority 2: Handle libp2p swarm events
            event = swarm.next() => {
                if let Some(event) = event {
                    handle_swarm_events(&mut swarm, event, &broadcast_sender);
                }
            }
        }
    }
}
```

### Event Types Handled

1. **SwarmEvents:**
   - `NewListenAddr`: New listening address established
   - `IncomingConnection`: Peer attempting to connect
   - `ConnectionEstablished`: Connection successfully established
   - `ConnectionClosed`: Connection terminated
   - `Behaviour`: Events from individual network behaviours

2. **Behaviour Events:**
   - **Kademlia:**
     - `OutboundQueryProgressed`: DHT query progress/completion
     - `RoutingUpdated`: Routing table changes
     - `InboundRequest`: Incoming DHT requests
   - **mDNS (native only):**
     - `Discovered`: Local peers discovered
     - `Expired`: Local peer expired
   - **Identify:**
     - `Received`: Peer identification received
     - `Sent`: Identification sent to peer

---

## Network Protocol Stack

### Transport Layer

```
Application (Netabase API)
         │
         ▼
Command/Event Layer
         │
         ▼
libp2p Behaviours (Kademlia, mDNS, Identify)
         │
         ▼
libp2p Transport Stack
         │
    ┌────┴────┐
    │         │
    ▼         ▼
  TCP       QUIC
    │         │
    └────┬────┘
         │
         ▼
Security/Multiplexing (Noise + Yamux)
         │
         ▼
Network (IP)
```

### Behaviour Composition

**File:** `src/network/behaviour/mod.rs`

```rust
#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct NetabaseBehaviour<D>
where
    D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static,
{
    /// Kademlia DHT for distributed record storage
    pub kad: libp2p::kad::Behaviour<SledStore<D>>,

    /// Identify protocol for peer information exchange
    pub identify: libp2p::identify::Behaviour,

    /// mDNS for local peer discovery (native only)
    #[cfg(feature = "native")]
    pub mdns: libp2p::mdns::tokio::Behaviour,

    /// Connection limits to prevent resource exhaustion
    pub connection_limit: libp2p::connection_limits::Behaviour,
}
```

---

## Type System and Traits

### Core Traits Hierarchy

```
User Models (struct User, struct Post)
    │ implements
    ▼
NetabaseModelTrait<D>
    ├── const DISCRIMINANT
    ├── type PrimaryKey
    ├── type SecondaryKeys
    ├── type Keys
    ├── fn primary_key() -> PrimaryKey
    ├── fn secondary_keys() -> Vec<SecondaryKeys>
    └── fn discriminant_name() -> &'static str

Definition Enum (BlogDefinition)
    │ implements
    ▼
NetabaseDefinitionTrait
    ├── type Keys = BlogKeys
    ├── fn to_key() -> Result<Keys>
    └── fn discriminant_name() -> &'static str

    │ implements
    ▼
RecordStoreExt
    ├── fn to_record() -> Result<libp2p::kad::Record>
    └── fn from_record(&Record) -> Result<Self>

    │ implements
    ▼
ToIVec + FromIVec
    ├── fn to_ivec() -> Result<IVec>
    └── fn from_ivec(&IVec) -> Result<Self>
```

### Type Conversions

The API uses several type conversions:

```rust
// User provides a model
let user = User { id: 1, ... };

// API converts to Definition
netabase.put_record(user).await?;
// Internally: D::from(user) -> BlogDefinition::User(user)

// Definition converts to libp2p Record
impl RecordStoreExt for BlogDefinition {
    fn to_record(&self) -> Result<libp2p::kad::Record> {
        // BlogDefinition -> bytes -> libp2p::kad::Record
    }
}

// Record converts back to Definition
let record = get_from_network();
let definition = BlogDefinition::from_record(&record)?;

// Pattern match to extract model
match definition {
    BlogDefinition::User(user) => { /* use user */ }
    BlogDefinition::Post(post) => { /* use post */ }
}
```

---

## Summary

Netabase architecture provides:

1. **Type-safe P2P networking:** Compile-time guarantees for distributed data through model types
2. **Dual access pattern:** Network operations via `put_record()/get_record()`, local via `open_tree()`
3. **Async command/event model:** Clean separation between user API and network layer
4. **libp2p integration:** Full DHT, mDNS, and peer discovery capabilities via Kademlia
5. **Event subscription:** Subscribe to network events for reactive applications
6. **Flexible storage:** Same `SledStore` serves both local access and DHT operations

The architecture enables developers to write distributed applications with the safety and ergonomics of local database code, while Netabase handles all networking complexity transparently.

### Key API Patterns

**Network Operations (Async):**
```rust
// Takes model, distributes to network
netabase.put_record(user).await?;

// Takes key, queries network
netabase.get_record(user_key).await?;

// Queries local store only
netabase.query_local_records(None).await?;
```

**Local Operations (Sync):**
```rust
// Type-safe tree access
let tree = netabase.store().open_tree::<User>();

// Synchronous CRUD
tree.put(user)?;
tree.get(UserPrimaryKey(1))?;
tree.get_by_secondary_key(UserSecondaryKeys::Email(...))?;
```

Both patterns work with the same underlying data, providing flexibility for different use cases.
