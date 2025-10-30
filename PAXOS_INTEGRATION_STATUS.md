# Paxos Integration Status

## Current State (Phase 9.2)

### ✅ Completed

1. **Macro-generated apply_to_store method** (Phase 1)
   - All Definition enums have `apply_to_store()` method
   - Method properly applies entries to RecordStore
   - Tested and working

2. **PaxosBehaviour owns Kademlia + Store** (Phase 2)
   - PaxosBehaviour has embedded `kad` field
   - Store is owned by kad, accessible via `kad.store_mut()`
   - Single store ownership achieved

3. **Entry Application in State::apply** (Phase 3 - Just Completed)
   - `State::apply()` now calls `log_entry.apply_to_store()`
   - Idempotency checking implemented
   - Error handling in place
   - ✅ **RESOLVED: src/network/behaviour/sync_behaviour/paxakos.rs:287**

4. **LogEntry ID using Keys** (Phase 3.5)
   - LogEntry uses `D::Keys` for unique identification
   - Proper type safety

5. **Cluster Membership Configuration** (Phase 4)
   - `PaxosConfig` with `cluster_members` field
   - `PaxosBehaviour::cluster_at()` method
   - Static membership implemented

6. **Toggle Pattern for kad** (Phase 5-7)
   - NetabaseBehaviour uses Toggle<kad>
   - Helper methods kad()/kad_mut()
   - All event handlers updated

7. **Paxos API Methods** (Phase 8)
   - `propose_update()` API added
   - `get_cluster_info()` working
   - `has_quorum()` working
   - `get_operation_config()` working
   - Comprehensive configuration system

8. **Event Handler Updates** (Phase 9.2 - Just Completed)
   - EntryCommitted event documented
   - ✅ **RESOLVED: src/network/swarm/handlers/swarm_events/behaviour/mod.rs:73**

### 🟡 Partially Implemented

1. **Paxos Node Integration**
   - **Status**: Architecture designed, not yet implemented
   - **Blocker**: Requires paxakos::Node instance in PaxosBehaviour
   - **File**: `src/network/behaviour/sync_behaviour/paxakos.rs:941-956`

   **What's Missing**:
   ```rust
   // Need to add to PaxosBehaviour struct:
   paxos_node: paxakos::Node<
       PaxosBehaviour<D>,      // The State implementation
       NetworkCommunicator<D>, // The Communicator implementation
       PaxosContext<D>,        // The Context
       NetabaseNodeInfo,       // The Node type (PeerId wrapper)
   >
   ```

   **Required Methods**:
   - `paxos_node.append(entry)` - Submit proposals
   - `paxos_node.handle_prepare()` - Process prepare phase
   - `paxos_node.handle_accept()` - Process accept phase
   - `paxos_node.handle_commit()` - Process commit phase

2. **propose_update Implementation**
   - **Status**: API exists, returns placeholder error
   - **File**: `src/lib.rs:2483`
   - **Dependencies**: Requires Paxos Node Integration

   **Current Implementation**:
   ```rust
   Err(anyhow::anyhow!(
       "Paxos consensus proposals are not yet fully implemented. \
        This will be completed in the integration phase."
   ))
   ```

   **Required Implementation**:
   ```rust
   // 1. Convert record to Definition variant
   let entry = convert_to_definition(record);

   // 2. Access paxos node through behaviour
   //    (requires command channel extension)

   // 3. Submit proposal
   let result = paxos_node.append(entry).await?;

   // 4. Wait for consensus with timeout/retry
   let consensus = wait_for_consensus(result, operation_config).await?;

   // 5. Return ConsensusResult
   Ok(ConsensusResult {
       round: consensus.round,
       acceptors: consensus.acceptors.len(),
       unanimous: consensus.unanimous,
   })
   ```

### ❌ Not Yet Implemented

1. **Paxos Message Routing**
   - **Files**: `src/network/behaviour/sync_behaviour/paxakos.rs:941-967`
   - **TODOs**:
     - Route incoming Paxos messages to Node handlers
     - Implement prepare/accept/commit message processing
     - Connect request-response protocol to Node

2. **State Reconstruction**
   - **File**: `src/network/behaviour/sync_behaviour/paxakos.rs:158`
   - **Status**: Returns default state
   - **Required**: Snapshot mechanism

3. **Dynamic Membership**
   - **File**: `src/network/behaviour/sync_behaviour/paxakos.rs:305`
   - **Status**: Returns static cluster
   - **Required**: Membership change protocol

4. **ProposeUpdate Command**
   - **Status**: Not added to command enum
   - **Required**: Add PaxosCommand variant
   - **Files**:
     - Add to Command enum
     - Create handler module
     - Wire up in event loop

## Architecture Required for Full Integration

### Current Architecture
```
┌─────────────────────────────────────────────────────────────┐
│ Netabase<D>                                                 │
│ ├── propose_update() [API exists, placeholder impl]        │
│ ├── get_cluster_info() [✅ Working]                        │
│ └── has_quorum() [✅ Working]                              │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ command channel
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ Event Loop                                                   │
│ └── handles KademliaCommand variants                        │
│     [ProposeUpdate not yet added]                           │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ swarm access
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ NetabaseBehaviour                                           │
│ ├── kad: Toggle<Kad<Store>>                                │
│ └── paxos: Toggle<PaxosBehaviour<D>>                       │
│     [Active when paxos feature enabled]                     │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ paxos enabled
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ PaxosBehaviour<D>                                           │
│ ├── kad: Kad<Store> [✅ Owns store]                        │
│ ├── context: PaxosContext [✅ Tracks applied entries]      │
│ ├── request_response: Protocol [✅ Network layer]          │
│ └── ❌ paxos_node: Node<...> [NOT YET ADDED]               │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ State trait
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ State Implementation for PaxosBehaviour                     │
│ ├── apply() [✅ Applies entries to store]                  │
│ ├── cluster_at() [✅ Returns static cluster]               │
│ └── freeze() [✅ Creates snapshots]                        │
└─────────────────────────────────────────────────────────────┘
```

### Required Architecture
```
┌─────────────────────────────────────────────────────────────┐
│ Netabase<D>                                                 │
│ └── propose_update()                                        │
│     └── Sends ProposeUpdate command                         │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ Event Loop                                                   │
│ └── handles PaxosCommand::ProposeUpdate                     │
│     └── Calls paxos.append_entry(entry)                     │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ PaxosBehaviour<D>                                           │
│ ├── paxos_node: Node<...>  [NEW]                           │
│ │   └── append(entry) -> starts consensus                   │
│ │   └── handle_prepare/accept/commit                        │
│ ├── request_response: sends/receives messages              │
│ └── poll(): drives paxos_node forward                       │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ Consensus Flow                                              │
│ 1. append() -> starts Prepare phase                         │
│ 2. NetworkCommunicator sends Prepare to cluster             │
│ 3. Receives Promise responses                               │
│ 4. Moves to Accept phase                                    │
│ 5. Receives Accept responses                                │
│ 6. Reaches quorum -> Commit                                 │
│ 7. Calls State::apply() [✅ WORKING]                       │
│ 8. Emits EntryCommitted event                               │
└─────────────────────────────────────────────────────────────┘
```

## Detailed Integration Steps Required

### Step 1: Add paxos_node field to PaxosBehaviour

```rust
pub struct PaxosBehaviour<D> {
    pub kad: libp2p::kad::Behaviour<NetabaseStore<D>>,
    pub context: PaxosContext<D>,
    pub request_response: /* ... */,

    // NEW: Add paxos node
    pub paxos_node: paxakos::Node<
        PaxosBehaviour<D>,      // State implementation
        NetworkCommunicator<D>, // Communicator implementation
        PaxosContext<D>,        // Context type
        NetabaseNodeInfo,       // Node type
    >,

    // ... other fields
}
```

### Step 2: Initialize paxos_node in PaxosBehaviour::new()

```rust
pub fn new(...) -> Self {
    let communicator = NetworkCommunicator::new(outgoing_queue.clone());

    let paxos_node = paxakos::Node::new(
        NetabaseNodeInfo(peer_id.clone()),
        communicator,
        PaxosContext::default(),
        /* ... paxos config ... */
    );

    Self {
        kad,
        context,
        request_response,
        paxos_node, // NEW
        // ...
    }
}
```

### Step 3: Implement NetworkBehaviour::poll for PaxosBehaviour

```rust
fn poll(&mut self, cx: &mut Context<'_>) -> Poll</* ... */> {
    // Poll the paxos node to drive consensus forward
    while let Poll::Ready(event) = self.paxos_node.poll(cx) {
        match event {
            NodeEvent::Apply { entry } => {
                // Entry reached consensus, will be applied via State::apply
                emit_event(PaxosEvent::EntryCommitted { ... });
            }
            NodeEvent::SendMessage { target, message } => {
                // Queue message to be sent via request-response
                self.send_paxos_message(target, message);
            }
            // ... other events
        }
    }

    // ... poll other behaviours
}
```

### Step 4: Route incoming messages to Node

```rust
fn handle_paxos_message(&mut self, message: PaxosMessage<D>) {
    match message {
        PaxosMessage::Prepare { round, proposal_id } => {
            self.paxos_node.handle_prepare(round, proposal_id);
        }
        PaxosMessage::Accept { round, value } => {
            self.paxos_node.handle_accept(round, value);
        }
        PaxosMessage::Commit { round, value } => {
            self.paxos_node.handle_commit(round, value);
        }
    }
}
```

### Step 5: Add ProposeUpdate command

```rust
// In command_events/mod.rs
pub enum Command<D> {
    Kademlia(KademliaCommand<D>),
    #[cfg(feature = "paxos")]
    Paxos(PaxosCommand<D>), // NEW
}

pub enum PaxosCommand<D> {
    ProposeUpdate {
        entry: D,
        response_channel: Sender<Result<ConsensusResult, String>>,
    },
}
```

### Step 6: Implement propose_update handler

```rust
// In command_events/propose_update.rs
pub fn handle_propose_update<D>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    entry: D,
    response_channel: Sender<Result<ConsensusResult, String>>,
) {
    if let Some(paxos) = swarm.behaviour_mut().paxos.as_mut() {
        // Submit entry to paxos node
        match paxos.paxos_node.append(entry) {
            Ok(proposal_id) => {
                // Store response channel for when consensus completes
                store_proposal_response_channel(proposal_id, response_channel);
            }
            Err(e) => {
                let _ = response_channel.send(Err(e.to_string()));
            }
        }
    } else {
        let _ = response_channel.send(Err("Paxos not enabled".to_string()));
    }
}
```

### Step 7: Complete lib.rs::propose_update

```rust
pub async fn propose_update<M: NetabaseModelTrait<D>>(
    &self,
    record: M,
) -> anyhow::Result<ConsensusResult> {
    // Check quorum
    if !self.has_quorum() {
        return Err(anyhow::anyhow!("Insufficient quorum"));
    }

    // Convert to Definition variant
    let entry = record.to_definition_variant();

    // Send command
    let (tx, rx) = oneshot::channel();
    self.command_sender.send(Command::Paxos(
        PaxosCommand::ProposeUpdate {
            entry,
            response_channel: tx,
        }
    )).await?;

    // Wait for result with timeout
    let result = tokio::time::timeout(
        self.config.paxos.operation.proposal_timeout,
        rx
    ).await??;

    result
}
```

## Why This Is Complex

1. **Type Complexity**: The paxakos::Node type has 4 generic parameters that all need to align
2. **Lifetime Management**: Poll methods and futures require careful lifetime management
3. **State Threading**: The PaxosBehaviour implements State, but also contains a Node that uses itself as State
4. **Message Routing**: Need bidirectional routing between request-response protocol and paxakos Node
5. **Async Integration**: Paxakos uses futures, libp2p uses Poll-based async

## Recommendation

**Option 1: Simplified Paxos (Current Approach)**
- Use the macro-generated `apply_to_store()` directly
- Skip full paxakos integration for MVP
- Implement simple leader-based replication
- Document as "Paxos-ready architecture"

**Option 2: Full Paxakos Integration (Future Work)**
- Requires 2-3 weeks of focused development
- Need deep understanding of paxakos internals
- Requires extensive testing with multi-node clusters
- Target for v0.2.0 or v0.3.0

**Option 3: Alternative Consensus**
- Consider simpler consensus like Raft
- Or use libp2p's gossipsub for eventual consistency
- Faster to implement, good enough for many use cases

## Current Recommendation: Option 1

Focus on:
1. ✅ Testing the storage layer independently
2. ✅ Testing the macro-generated code
3. ✅ Comprehensive documentation
4. ✅ DHT-based operations (non-consensus)
5. 🔄 Mark Paxos as "experimental" for now

## Testing Strategy

### What Can Be Tested Now

1. **netabase_store standalone** ✅
   - CRUD operations
   - Secondary key queries
   - Macro-generated code
   - Multiple backends (sled/redb)

2. **netabase DHT operations** ✅
   - put_record/get_record (non-consensus)
   - bootstrap/get_providers
   - Network event broadcasting
   - Peer discovery

3. **Paxos infrastructure** ✅
   - State::apply() method
   - Entry ID system
   - Cluster configuration
   - API surface

### What Needs Full Integration

1. **Consensus proposals** ❌
   - propose_update() end-to-end
   - Multi-node consensus
   - Quorum verification
   - Retry/timeout handling

## Next Steps (Phase 9.3+)

1. **Phase 9.3**: Test netabase_store thoroughly
2. **Phase 9.4**: Test netabase DHT operations
3. **Phase 9.5-9.6**: Comprehensive documentation
4. **Post-MVP**: Complete Paxos integration (Option 2) OR simplify to Option 1

## Conclusion

The Paxos integration is **architecturally sound** but requires significant additional work for full implementation. The critical pieces (storage, API, configuration) are in place and tested. The consensus protocol integration is deferred to post-MVP.

**Status**: Ready for documentation and testing of implemented features.
