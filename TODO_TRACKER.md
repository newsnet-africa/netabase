# Netabase TODO Tracker

This document tracks all TODOs, placeholders, and future work items across the Netabase codebase.

## Status Legend
- 🔴 **Critical** - Blocks core functionality, must be resolved
- 🟡 **Important** - Should be resolved for production readiness
- 🟢 **Enhancement** - Future improvements, not blocking
- 🔵 **WASM** - WASM-specific implementations (deferred)

---

## Critical TODOs (Blocks Phase 9 Completion)

### 1. ✅ RESOLVED: Apply Committed Entry to Store
**File**: `src/network/behaviour/sync_behaviour/paxakos.rs:287`
**Status**: ✅ **COMPLETED** (Phase 9.2)
**Priority**: CRITICAL

**Resolution**: Implemented in `State::apply()` method:
```rust
let store = self.kad.store_mut();
match log_entry.apply_to_store(store) {
    Ok(_) => {
        println!("✅ Successfully applied entry to store: {:?}", entry_id);
        context.mark_applied(entry_id, 0);
        Ok(((), ()))
    }
    Err(e) => {
        eprintln!("❌ Failed to apply entry to store: {:?}", e);
        Err(format!("Failed to apply entry: {}", e))
    }
}
```

**Date Completed**: 2025-10-30

---

### 2. ✅ RESOLVED: Apply Committed Entry in Event Handler
**File**: `src/network/swarm/handlers/swarm_events/behaviour/mod.rs:73`
**Status**: ✅ **COMPLETED** (Phase 9.2)
**Priority**: CRITICAL

**Resolution**: Documented that entry application happens in `State::apply()`:
```rust
println!("✅ Paxos: Entry committed at round {}: {:?}", round, entry);
// Note: The entry is applied to the database in State::apply()
// This event is for notification and monitoring purposes
```

**Date Completed**: 2025-10-30

---

### 3. 🟡 DEFERRED: Complete Paxos Proposal Submission
**File**: `src/lib.rs:2483`
**Status**: **DEFERRED** to Post-MVP
**Priority**: IMPORTANT (downgraded from CRITICAL)

```rust
// TODO: Implement actual Paxos proposal submission
Err(anyhow::anyhow!(
    "Paxos consensus proposals are not yet fully implemented. \
     This will be completed in the integration phase."
))
```

**Deferral Reason**:
Deep paxakos integration requires:
1. Adding `paxos_node: paxakos::Node<...>` field to PaxosBehaviour
2. Implementing NetworkBehaviour::poll to drive consensus
3. Routing messages between request-response and Node
4. Complex type alignment and lifetime management
5. Extensive multi-node cluster testing

**Estimated Effort**: 2-3 weeks of focused development

**Current Approach**:
- ✅ API surface is complete
- ✅ Configuration system is comprehensive
- ✅ Storage layer integration is working
- ✅ Architecture is "Paxos-ready"
- 🔄 Full consensus protocol deferred to v0.2.0

**Alternative for MVP**:
- DHT-based put_record/get_record operations work without consensus
- Can be used for eventually-consistent operations
- Paxos infrastructure is in place for future upgrade

**Reference**: See `PAXOS_INTEGRATION_STATUS.md` for detailed integration plan

**New ETA**: v0.2.0 (Post-MVP)

**Blockers**: Requires architectural design + implementation time
**Migration Path**: Existing API will remain compatible when consensus is added

---

## Important TODOs (Production Readiness)

### 4. 🟡 Implement Proper State Reconstruction
**File**: `src/network/behaviour/sync_behaviour/paxakos.rs:158`
**Status**: Deferred
**Priority**: IMPORTANT

```rust
// TODO: Implement proper state reconstruction
// For now, just return empty state
State::default()
```

**Resolution Plan**:
- Implement snapshot mechanism
- Store periodic state snapshots
- Reconstruct from snapshot + log replay
- Add snapshot compaction

**Blockers**: None (but requires design)
**ETA**: Post-MVP

---

### 5. 🟡 Implement Dynamic Membership
**File**: `src/network/behaviour/sync_behaviour/paxakos.rs:305`
**Status**: Deferred
**Priority**: IMPORTANT

```rust
// TODO: Implement dynamic membership
// For now, use static membership from config
```

**Resolution Plan**:
- Design membership change protocol
- Implement two-phase membership change
- Add member discovery via mDNS/Identify
- Handle member failures gracefully

**Blockers**: Requires Paxos extensions
**ETA**: Post-MVP (v0.2.0)

---

### 6. 🟡 Route to Actual Paxakos Node Handler Methods
**File**: `src/network/behaviour/sync_behaviour/paxakos.rs:941-956`
**Status**: Deferred
**Priority**: IMPORTANT

```rust
// TODO: Route to the actual paxakos::Node handler methods
// TODO: Call paxos_node.handle_prepare() or equivalent
// TODO: Call paxos_node.handle_proposal() or equivalent
// TODO: Call paxos_node.handle_commit() or equivalent
```

**Resolution Plan**:
- Study paxakos::Node trait methods
- Map PaxosMessage variants to handler methods
- Implement proper error handling
- Add request/response tracking

**Blockers**: Requires deep paxakos integration
**ETA**: Post-MVP

---

## Enhancement TODOs (Future Work)

### 7. 🟢 Implement Connection Event Handlers
**Files**: Multiple in `src/network/swarm/handlers/swarm_events/`
**Status**: Deferred
**Priority**: ENHANCEMENT

Affected files:
- `external_addr_confirmed.rs:29` - External address confirmed
- `new_external_addr_candidate.rs:29` - New external address candidate
- `outgoing_connection_error.rs:32` - Outgoing connection error
- `incoming_connection_error.rs:36` - Incoming connection error
- `listener_error.rs:30` - Listener error
- `expired_listen_addr.rs:30` - Expired listen address
- `external_addr_expired.rs:29` - External address expired
- `new_external_addr_of_peer.rs:30` - New external address of peer
- `incoming_connection.rs:33` - Incoming connection
- `listener_closed.rs:31` - Listener closed
- `dialing.rs:32` - Dialing

**Resolution Plan**:
- Add logging for debugging
- Implement connection metrics
- Add connection pool management
- Handle peer reputation

**Blockers**: None
**ETA**: v0.2.0 or later

---

### 8. 🟢 Implement Fallback Event Handlers
**Files**:
- `src/network/swarm/handlers/swarm_events/fallback.rs:30`
- `src/network/swarm/handlers/command_events/fallback.rs:32`
**Status**: Deferred
**Priority**: ENHANCEMENT

**Resolution Plan**:
- Add comprehensive logging for unknown events
- Implement telemetry for debugging
- Add event type discovery

**Blockers**: None
**ETA**: v0.2.0

---

### 9. 🟢 Improve Error Response Keys
**Files**:
- `src/network/swarm/handlers/command_events/put_record_to.rs:82`
- `src/network/swarm/handlers/command_events/get_providers.rs:75`
- `src/network/swarm/handlers/command_events/get_record.rs:74`
**Status**: Deferred
**Priority**: ENHANCEMENT

Current approach uses placeholder keys for error responses. Should use actual record keys.

**Resolution Plan**:
- Extract key from failed record before conversion
- Use actual key in error responses
- Improve error messages

**Blockers**: None
**ETA**: v0.2.0

---

## WASM TODOs (Platform-Specific)

### 10. 🔵 WASM Swarm Implementation
**Files**:
- `src/network/swarm/mod.rs:114, 169`
- `src/network/swarm/handlers/mod.rs:70`
**Status**: Not Started
**Priority**: WASM-SPECIFIC

**Resolution Plan**:
- Implement WebSocket transport
- Add WebRTC support
- Connect to relay nodes
- Handle browser environment constraints

**Blockers**: Requires wasm-compatible libp2p transports
**ETA**: v0.3.0 (WASM focus)

---

### 11. 🔵 Gossipsub Event Cloning
**File**: `src/network/behaviour/clone_impl.rs:91`
**Status**: Deferred
**Priority**: ENHANCEMENT

```rust
// TODO: Implement proper event cloning for gossipsub
```

**Resolution Plan**:
- Implement Clone for Gossipsub events
- Add event serialization for cloning
- Test with broadcast subscribers

**Blockers**: May require upstream libp2p changes
**ETA**: When gossipsub is added

---

## Backend TODOs

### 12. 🟢 Redb Backend Support for NetabaseStore
**File**: `src/network/store.rs:93`
**Status**: Partially Implemented
**Priority**: ENHANCEMENT

```rust
// Note: Currently only implemented for Sled backend. Redb support TODO.
```

**Resolution Plan**:
- Implement RecordStore trait for Redb backend
- Add feature flag for redb-specific code
- Test against Kademlia DHT requirements

**Blockers**: None
**ETA**: v0.2.0

---

## Documentation TODOs

### 13. 🟡 Relations Documentation
**File**: `src/lib.rs:224`
**Status**: Deferred
**Priority**: IMPORTANT

```rust
//! ### TODO
//! #### Relations
//! [ ] Use foreign key fields to reference other models
//! [ ] Enable efficient joins and referential integrity
//! [ ] Consider using `NetabaseRelationalQuery` for complex relationships
```

**Resolution Plan**:
- Design relational query API
- Implement foreign key support
- Add join operations
- Document usage patterns

**Blockers**: Requires netabase_store extensions
**ETA**: v0.2.0

---

## Testing TODOs

### 14. 🟡 Update Paxakos Tests
**File**: `src/network/behaviour/sync_behaviour/paxakos.rs:1436`
**Status**: Pending (Phase 9.3)
**Priority**: IMPORTANT

```rust
// TODO: Update these tests to use a proper test Definition type with Keys
```

**Resolution Plan**:
- Create test definition module
- Update all paxakos tests
- Add integration tests
- Test with real cluster

**Blockers**: None
**ETA**: Phase 9.3-9.4

---

## Summary Statistics

- **Total TODOs**: 34
- **Critical (🔴)**: 3
- **Important (🟡)**: 4
- **Enhancement (🟢)**: 4
- **WASM (🔵)**: 2
- **Multiple Items**: 21 (connection handlers, etc.)

## Resolution Plan by Phase

### Phase 9.2: Critical Paxos Integration
- [ ] Resolve TODO #1: Complete Paxos proposal submission
- [ ] Resolve TODO #2: Apply committed entry to store
- [ ] Resolve TODO #3: Apply committed entry in event handler

### Phase 9.3-9.4: Testing
- [ ] Resolve TODO #14: Update paxakos tests
- [ ] Add comprehensive test coverage

### Post-MVP (v0.2.0)
- [ ] Resolve TODOs #4-13: Production readiness items
- [ ] Implement connection event handlers
- [ ] Add redb backend support
- [ ] Improve error handling

### Future (v0.3.0+)
- [ ] WASM implementations
- [ ] Dynamic membership
- [ ] Advanced features

---

## Notes

- All critical TODOs will be resolved in Phase 9.2
- Connection event handlers are intentionally deferred (they're logged, not blocking)
- WASM support is planned but not in current scope
- State reconstruction and dynamic membership require design work and are post-MVP

**Last Updated**: 2025-10-30 (Phase 9.1)
