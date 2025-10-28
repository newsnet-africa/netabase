# Netabase Sync Architecture - Comprehensive Guide

## Table of Contents

1. [Overview](#overview)
2. [Architecture Goals](#architecture-goals)
3. [Module Structure](#module-structure)
4. [Core Concepts](#core-concepts)
5. [Detailed Module Breakdown](#detailed-module-breakdown)
6. [Data Flow](#data-flow)
7. [Security Model](#security-model)
8. [Performance Considerations](#performance-considerations)
9. [Integration Guide](#integration-guide)

---

## Overview

The Netabase sync module implements a **Byzantine fault-tolerant (BFT) state synchronization system** for distributed databases in open, permissionless networks. It addresses the challenge of maintaining consistent state across peers when:

- **Some peers may be malicious** (Byzantine faults)
- **Network partitions can occur**
- **Peers join and leave dynamically**
- **There is no central authority**

### Key Features

1. **Byzantine Reliable Broadcast (BRB)** - Ensures critical updates are reliably delivered despite Byzantine peers
2. **Gossip Protocol** - Efficient anti-entropy state propagation
3. **Sybil Resistance** - Proof-of-Work and reputation systems to prevent Sybil attacks
4. **Vector Clocks** - Causality tracking for concurrent updates
5. **CRDTs** - Conflict-free data structures for automatic merge
6. **Paxos Consensus** - Strong consistency when needed
7. **Request-Response Protocol** - libp2p integration for peer communication

---

## Architecture Goals

### 1. Byzantine Fault Tolerance

**Problem**: In open networks, some peers may be malicious, sending incorrect data or trying to disrupt consensus.

**Solution**: The system tolerates up to `f` Byzantine faults out of `3f+1` total peers using:
- Quorum-based validation (need 2f+1 confirmations)
- Digital signatures for message authentication
- Multiple validation phases (Echo + Ready in BRB)

### 2. Eventual Consistency

**Problem**: Network partitions and concurrent updates can cause temporary inconsistencies.

**Solution**:
- Gossip protocol ensures all peers eventually see all updates
- Vector clocks detect and handle concurrent updates
- CRDTs automatically resolve conflicts

### 3. Sybil Attack Resistance

**Problem**: Malicious actors can create many fake identities to overwhelm the system.

**Solution**:
- Proof-of-Work challenges for new peers
- Reputation system tracking peer behavior
- Time-limited verifications that expire

### 4. Performance

**Problem**: Synchronization overhead must not overwhelm the network.

**Solution**:
- Configurable gossip fanout and intervals
- Merkle tree-based state summaries for efficient comparison
- Batch message processing
- Prioritization of critical vs. routine updates

---

## Module Structure

```
src/sync/
├── mod.rs                  # Main module exports and configuration
├── types.rs                # Core type definitions
├── traits.rs               # Trait definitions for extensibility
├── clock.rs                # Vector clock implementation
├── protocol.rs             # Request/response message types
├── codec.rs                # Serialization codec for libp2p
├── behavior.rs             # Main synchronization manager
├── config.rs               # Configuration builders and presets
├── reputation.rs           # Peer reputation tracking
├── serde_helper.rs         # Serialization utilities
│
├── gossip/
│   ├── mod.rs              # Gossip protocol implementation
│   ├── scheduler.rs        # Gossip timing and peer selection
│   ├── state_exchange.rs   # State comparison and exchange
│   └── byzantine_filter.rs # Byzantine peer filtering
│
├── brb/
│   ├── mod.rs              # Byzantine Reliable Broadcast
│   ├── quorum.rs           # Quorum management
│   └── validator.rs        # Message validation
│
├── crdt/
│   ├── mod.rs              # CRDT implementations
│   ├── counter.rs          # Grow-only counter
│   ├── lww.rs              # Last-write-wins register
│   └── orset.rs            # Observed-remove set
│
├── proof/
│   └── mod.rs              # Proof-of-Work and challenge system
│
├── paxos/
│   └── mod.rs              # Paxos consensus implementation
│
├── netabase_ext.rs         # Netabase integration wrapper
└── integration.rs          # (Future) Full typed integration
```

---

## Core Concepts

### 1. Vector Clocks (`clock.rs`)

**What**: A vector clock is a data structure for tracking causality in distributed systems.

**Why**: We need to know:
- Did event A happen before event B?
- Are events A and B concurrent?
- What is the current state of each peer?

**How It Works**:

```rust
pub struct VectorClock {
    clocks: HashMap<PeerId, LogicalTimestamp>,  // Each peer's timestamp
    local_peer: PeerId,                          // This peer's ID
}
```

**Operations**:

1. **`new(local_peer)`** - Creates a clock initialized with local peer at timestamp 0
2. **`increment()`** - Increments local peer's timestamp (call before sending a message)
3. **`merge(&other)`** - Takes element-wise maximum of two clocks (call when receiving a message)
4. **`happened_before(&other)`** - Returns true if this clock ≤ other clock for all peers
5. **`is_concurrent(&other)`** - Returns true if neither happened before the other

**Example**:
```rust
let mut clock_a = VectorClock::new(peer_a);  // {A: 0}
clock_a.increment();                          // {A: 1}

let mut clock_b = VectorClock::new(peer_b);  // {B: 0}
clock_b.increment();                          // {B: 1}

// These are concurrent (neither happened before the other)
assert!(clock_a.is_concurrent(&clock_b));

clock_a.merge(&clock_b);                      // {A: 1, B: 1}
clock_a.increment();                          // {A: 2, B: 1}

// Now clock_a happened after clock_b
assert!(!clock_a.happened_before(&clock_b));
assert!(clock_b.happened_before(&clock_a));
```

**Why This Matters**: Vector clocks let us:
- Detect conflicts (concurrent updates to same data)
- Order events correctly across peers
- Implement CRDTs that merge automatically

---

### 2. Gossip Protocol (`gossip/`)

**What**: A probabilistic protocol where peers randomly exchange state with neighbors.

**Why**:
- Scales to large networks (O(log N) rounds to reach everyone)
- Robust to failures (no single point of failure)
- Balances load (every peer does roughly equal work)

**Components**:

#### a. `gossip/scheduler.rs` - Gossip Scheduler

**Purpose**: Decides WHEN to gossip and WITH WHOM.

```rust
pub struct GossipScheduler {
    last_gossip: Instant,           // When did we last gossip?
    interval: Duration,             // How often to gossip
    jitter: Duration,               // Random variation to avoid synchronized storms
}
```

**Key Functions**:

- **`should_gossip()`** - Returns true if enough time has passed
  - Adds random jitter to prevent all peers gossiping simultaneously
  - Example: If interval is 10s and jitter is 2s, actual interval is 10s ± 2s

- **`select_peers(peers, fanout)`** - Randomly selects N peers to gossip with
  - Uses randomization to ensure state spreads efficiently
  - Higher fanout = faster convergence but more bandwidth

**Why Random Selection?**:
- Deterministic selection could create echo chambers
- Random ensures all peers eventually connect
- Load balancing across the network

#### b. `gossip/state_exchange.rs` - State Exchange

**Purpose**: Efficiently compare and exchange state between peers.

```rust
pub struct StateExchange {
    local_digest: StateDigest,      // Summary of local state
}

pub struct StateDigest {
    merkle_root: [u8; 32],         // Root hash of Merkle tree
    record_count: usize,            // Total number of records
    clock: VectorClock,             // Causality information
    timestamp: u64,                 // When this digest was created
}
```

**Protocol Flow**:

1. **Digest Exchange**:
   ```
   Peer A ──[StateDigest]──> Peer B
   ```
   - Compare merkle roots to detect differences
   - If roots match, states are identical (done!)

2. **If Different**:
   ```
   Peer A <──[MerkleTree]──> Peer B
   ```
   - Exchange Merkle trees to find exact differing records
   - Only request missing/outdated records

3. **Record Transfer**:
   ```
   Peer A <──[Records]──> Peer B
   ```
   - Transfer only the delta (what's missing)

**Why Merkle Trees?**:
- O(log N) comparison instead of O(N)
- Can detect differences without sending all data
- Tamper-evident (any change affects root hash)

#### c. `gossip/byzantine_filter.rs` - Byzantine Filter

**Purpose**: Detect and isolate Byzantine (malicious) peers.

```rust
pub struct ByzantineFilter {
    suspicious_peers: HashMap<PeerId, SuspicionLevel>,
    blacklist: HashSet<PeerId>,
    evidence: HashMap<PeerId, Vec<Misbehavior>>,
}
```

**Detection Methods**:

1. **Equivocation**: Peer sends conflicting states with same vector clock
2. **Invalid Signatures**: Peer sends messages with bad signatures
3. **Merkle Mismatch**: Peer's Merkle tree doesn't match its claimed root
4. **Timeout**: Peer doesn't respond to requests

**Response**:
- Track evidence of misbehavior
- Increase suspicion level progressively
- Eventually blacklist persistent offenders
- Share blacklist via gossip

---

### 3. Byzantine Reliable Broadcast (`brb/`)

**What**: A protocol that ensures all honest peers deliver the same messages in the same order, even with Byzantine faults.

**Why**: For critical operations (like database schema changes), we need stronger guarantees than gossip provides.

**How It Works** (Bracha's Algorithm):

#### Phase 1: ECHO

```
Sender ──[ECHO(msg, sig)]──> All Peers
```

When a peer receives ECHO:
1. Verify sender's signature
2. Store the message
3. Broadcast ECHO to all other peers

#### Phase 2: READY

```
Peer ──[READY(msg, sig)]──> All Peers  (after receiving 2f+1 ECHOs)
```

When a peer receives 2f+1 ECHOs for same message:
1. Broadcast READY to all peers

#### Phase 3: DELIVER

```
Peer delivers message  (after receiving 2f+1 READYs)
```

When a peer receives 2f+1 READYs:
1. Deliver message to application
2. Message is now confirmed

**Why This Works**:

- **Validity**: If sender is honest, all honest peers deliver
- **Agreement**: If one honest peer delivers, all honest peers deliver
- **Integrity**: Honest peers deliver at most once
- **Totality**: If all honest peers receive, all honest peers deliver

**Implementation Details**:

```rust
// brb/mod.rs
pub struct ByzantineReliableBroadcast {
    config: BrbConfig,                                    // Configuration
    echo_messages: HashMap<MessageId, HashSet<PeerId>>,   // Who sent ECHO
    ready_messages: HashMap<MessageId, HashSet<PeerId>>,  // Who sent READY
    delivered: HashSet<MessageId>,                        // Already delivered
}
```

**Key Functions**:

- **`broadcast(message)`** - Initiates broadcast as sender
  - Creates message ID
  - Signs message
  - Sends ECHO to all peers

- **`handle_echo(peer, msg, sig)`** - Processes incoming ECHO
  - Verifies signature
  - Records peer's ECHO
  - If received 2f+1 ECHOs, sends READY

- **`handle_ready(peer, msg, sig)`** - Processes incoming READY
  - Records peer's READY
  - If received 2f+1 READYs, delivers message

**Quorum Calculation** (`brb/quorum.rs`):

```rust
pub struct QuorumConfig {
    total_peers: usize,     // N = 3f + 1
    max_faulty: usize,      // f
}

impl QuorumConfig {
    pub fn quorum_size(&self) -> usize {
        // Need 2f+1 for quorum
        2 * self.max_faulty + 1
    }

    pub fn has_quorum(&self, count: usize) -> bool {
        count >= self.quorum_size()
    }
}
```

**Why 3f+1 and 2f+1?**:
- With N=3f+1 total peers and f Byzantine:
  - Honest peers: N-f = 3f+1-f = 2f+1
  - Byzantine peers: f
- Quorum of 2f+1 guarantees:
  - At least f+1 honest peers (majority of honest)
  - Any two quorums overlap by at least one honest peer
  - Byzantine peers cannot form a quorum alone

---

### 4. Proof-of-Work System (`proof/mod.rs`)

**What**: A computational puzzle that proves work was done, used for Sybil resistance.

**Why**: Without it, attackers can create unlimited fake peer identities for free.

**How It Works**:

```rust
pub struct ProofOfWork {
    nonce: u64,         // Random number tried until valid
    hash: [u8; 32],     // Resulting hash
    difficulty: u32,    // Number of leading zero bits required
}
```

**Generation**:

```rust
pub fn generate(challenge: &[u8], difficulty: u32) -> ProofOfWork {
    let mut nonce = 0;
    loop {
        let hash = blake3::hash(&[challenge, &nonce.to_le_bytes()].concat());
        if count_leading_zeros(&hash) >= difficulty {
            return ProofOfWork { nonce, hash: hash.into(), difficulty };
        }
        nonce += 1;
    }
}
```

**Verification**:

```rust
pub fn verify(&self, challenge: &[u8]) -> bool {
    let hash = blake3::hash(&[challenge, &self.nonce.to_le_bytes()].concat());
    hash == self.hash && count_leading_zeros(&hash) >= self.difficulty
}
```

**Properties**:
- **Hard to generate**: Takes ~2^difficulty attempts
- **Easy to verify**: Single hash computation
- **Adjustable difficulty**: Increase to make Sybil attacks more expensive

**Challenge System**:

```rust
pub struct ChallengeSystem {
    challenges: HashMap<PeerId, Challenge>,           // Active challenges
    verifications: HashMap<PeerId, Verification>,     // Completed verifications
    config: ProofOfWorkConfig,
}

struct Challenge {
    challenge_data: Vec<u8>,    // Random bytes
    issued_at: Instant,          // When challenge was issued
}

struct Verification {
    verified_at: Instant,        // When proof was verified
    expires_at: Instant,         // When verification expires
}
```

**Flow**:

1. **New Peer Connects**:
   ```
   New Peer ──[Connect]──> System
   System ──[Challenge]──> New Peer
   ```
   - System generates random challenge
   - Stores challenge with timestamp

2. **Peer Computes Proof**:
   ```
   New Peer (computing PoW locally...)
   ```
   - Peer tries nonces until finding valid proof
   - Time = ~2^difficulty hash operations

3. **Peer Submits Proof**:
   ```
   New Peer ──[Proof]──> System
   System (verifies proof)
   ```
   - System verifies proof matches challenge
   - If valid, marks peer as verified
   - Sets expiration time

4. **Verification Expires**:
   ```
   System (after verification_duration)
   System ──[New Challenge]──> Peer
   ```
   - After expiration, peer must complete new challenge
   - Prevents old verifications from being reused

**Difficulty Calibration**:

```rust
// Difficulty 16 = ~65,000 hashes (< 1 second on modern CPU)
// Difficulty 20 = ~1,000,000 hashes (~1 second)
// Difficulty 24 = ~16,000,000 hashes (~16 seconds)
// Difficulty 28 = ~256,000,000 hashes (~4 minutes)
```

**Why This Prevents Sybil Attacks**:
- Creating 1,000 fake identities requires 1,000x the work
- Verifications expire, requiring repeated work
- Honest peers do work once, attackers must do it many times

---

### 5. Reputation System (`reputation.rs`)

**What**: Tracks peer behavior over time to identify unreliable or malicious peers.

**Why**: Proof-of-Work is expensive; reputation provides ongoing, lightweight filtering.

**Implementation**:

```rust
pub struct SimpleReputationSystem {
    reputations: HashMap<PeerId, PeerReputation>,
    decay_enabled: bool,
}

struct PeerReputation {
    score: f64,                      // Current reputation (0.0 to 1.0)
    last_updated: Instant,           // When last modified
    successful_interactions: u64,    // Count of successes
    failed_interactions: u64,        // Count of failures
}
```

**Score Calculation**:

```rust
const DEFAULT_REPUTATION: f64 = 0.5;      // New peers start here
const SUCCESS_REWARD: f64 = 0.1;          // Increase per success
const FAILURE_PENALTY: f64 = 0.2;         // Decrease per failure
const DECAY_RATE: f64 = 0.01;             // Decay per hour toward default
const DIMINISHING_FACTOR: f64 = 0.95;     // Reduces reward/penalty over time

fn record_success(&mut self, peer: &PeerId) {
    let rep = self.get_mut(peer);
    rep.apply_decay();  // First apply time-based decay

    // Diminishing returns: rewards decrease as reputation increases
    let reward = SUCCESS_REWARD * DIMINISHING_FACTOR.powi(rep.successful_interactions);
    rep.score = (rep.score + reward).min(1.0);
    rep.successful_interactions += 1;
    rep.last_updated = Instant::now();
}

fn record_failure(&mut self, peer: &PeerId) {
    let rep = self.get_mut(peer);
    rep.apply_decay();

    // Penalties don't diminish (failures always matter)
    rep.score = (rep.score - FAILURE_PENALTY).max(0.0);
    rep.failed_interactions += 1;
    rep.last_updated = Instant::now();
}
```

**Decay Function**:

```rust
fn apply_decay(&mut self) {
    let elapsed = self.last_updated.elapsed();
    let hours = elapsed.as_secs_f64() / 3600.0;
    let decay_amount = hours * DECAY_RATE;

    // Decay toward default (0.5)
    if self.score > DEFAULT_REPUTATION {
        self.score = (self.score - decay_amount).max(DEFAULT_REPUTATION);
    } else if self.score < DEFAULT_REPUTATION {
        self.score = (self.score + decay_amount).min(DEFAULT_REPUTATION);
    }
}
```

**Why Decay**:
- Peers can recover from past mistakes
- Recent behavior matters more than distant past
- Prevents permanent blacklisting from temporary issues

**Why Diminishing Returns**:
- New honest peers can build reputation quickly
- Established peers can't farm infinite reputation
- Attackers can't easily game the system

**Usage**:

```rust
// Good behavior
if peer_sent_valid_message {
    reputation.record_success(&peer_id);
}

// Bad behavior
if peer_sent_invalid_signature {
    reputation.record_failure(&peer_id);
}

// Select trustworthy peers
let top_peers = reputation.top_peers(10);  // Get 10 highest reputation peers
```

---

### 6. CRDTs (`crdt/`)

**What**: Conflict-free Replicated Data Types - data structures that automatically merge concurrent updates.

**Why**: When peers update data concurrently, we need deterministic merge without coordination.

#### a. Grow-Only Counter (`crdt/counter.rs`)

**Properties**:
- Can only increment
- All peers eventually agree on sum

```rust
pub struct GCounter {
    counts: HashMap<PeerId, u64>,  // Each peer's contribution
}

impl GCounter {
    pub fn increment(&mut self, peer: PeerId) {
        *self.counts.entry(peer).or_insert(0) += 1;
    }

    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    pub fn merge(&mut self, other: &GCounter) {
        for (peer, count) in &other.counts {
            let local = self.counts.entry(*peer).or_insert(0);
            *local = (*local).max(*count);  // Take maximum
        }
    }
}
```

**Why It Works**:
- Each peer has independent counter
- Merge takes max (idempotent and commutative)
- Sum is always increasing

**Example**:
```
Peer A: {A: 5, B: 3} = 8
Peer B: {A: 4, B: 4} = 8

After merge:
Both: {A: 5, B: 4} = 9  ✓ Consistent
```

#### b. Last-Write-Wins Register (`crdt/lww.rs`)

**Properties**:
- Stores single value
- Concurrent writes resolved by timestamp

```rust
pub struct LWWRegister<T> {
    value: T,
    timestamp: u64,
    peer: PeerId,
}

impl<T: Clone> LWWRegister<T> {
    pub fn set(&mut self, value: T, timestamp: u64, peer: PeerId) {
        if timestamp > self.timestamp ||
           (timestamp == self.timestamp && peer > self.peer) {
            self.value = value;
            self.timestamp = timestamp;
            self.peer = peer;
        }
    }

    pub fn merge(&mut self, other: &LWWRegister<T>) {
        self.set(other.value.clone(), other.timestamp, other.peer);
    }
}
```

**Why It Works**:
- Timestamps provide total order
- Peer ID breaks ties deterministically
- All peers apply same merge logic → convergence

**Example**:
```
Peer A writes "foo" at t=10
Peer B writes "bar" at t=12

After merge:
Both have "bar" (higher timestamp wins) ✓
```

#### c. Observed-Remove Set (`crdt/orset.rs`)

**Properties**:
- Set that supports add and remove
- Concurrent add wins over remove

```rust
pub struct ORSet<T> {
    elements: HashMap<T, HashSet<Uuid>>,  // Element → set of unique tags
}

impl<T: Hash + Eq + Clone> ORSet<T> {
    pub fn add(&mut self, element: T) -> Uuid {
        let tag = Uuid::new_v4();  // Unique identifier for this add
        self.elements.entry(element).or_default().insert(tag);
        tag
    }

    pub fn remove(&mut self, element: &T, tags: HashSet<Uuid>) {
        if let Some(element_tags) = self.elements.get_mut(element) {
            element_tags.retain(|tag| !tags.contains(tag));
            if element_tags.is_empty() {
                self.elements.remove(element);
            }
        }
    }

    pub fn contains(&self, element: &T) -> bool {
        self.elements.get(element).map_or(false, |tags| !tags.is_empty())
    }

    pub fn merge(&mut self, other: &ORSet<T>) {
        for (element, tags) in &other.elements {
            self.elements.entry(element.clone())
                .or_default()
                .extend(tags);
        }
    }
}
```

**Why It Works**:
- Each add gets unique tag
- Remove only removes observed tags
- Concurrent add creates new tag → survives remove
- Merge unions all tags

**Example**:
```
Peer A: adds "x" (tag: a1)
Peer B: concurrently adds "x" (tag: b1)
Peer A: removes "x" with tag a1

After merge:
Both have "x" with tag b1 ✓ (add wins)
```

---

### 7. Paxos Consensus (`paxos/mod.rs`)

**What**: A consensus algorithm for agreeing on a single value in a distributed system.

**Why**: For critical decisions (e.g., electing a leader, committing a transaction), we need strong consistency guarantees.

**Roles**:

1. **Proposer**: Initiates proposals
2. **Acceptor**: Votes on proposals
3. **Learner**: Learns chosen value

**Data Structures**:

```rust
pub struct ProposalNumber {
    round: u64,         // Monotonically increasing
    proposer: PeerId,   // Tie-breaker
}

pub enum PaxosMessage {
    // Phase 1a: Proposer → Acceptors
    Prepare { proposal_number: ProposalNumber },

    // Phase 1b: Acceptors → Proposer
    Promise {
        proposal_number: ProposalNumber,
        accepted_proposal: Option<ProposalNumber>,
        accepted_value: Option<Vec<u8>>,
    },

    // Phase 2a: Proposer → Acceptors
    Accept {
        proposal_number: ProposalNumber,
        value: Vec<u8>,
    },

    // Phase 2b: Acceptors → Learners
    Accepted {
        proposal_number: ProposalNumber,
        value: Vec<u8>,
    },
}

struct AcceptorState {
    promised_number: Option<ProposalNumber>,   // Highest promised
    accepted_number: Option<ProposalNumber>,   // Highest accepted
    accepted_value: Option<Vec<u8>>,           // Accepted value
}
```

**Protocol Flow**:

**Phase 1: Prepare**

```
Proposer:
1. Choose proposal number n (higher than any seen)
2. Send PREPARE(n) to acceptors

Acceptor (receives PREPARE(n)):
1. If n > promised_number:
   - Set promised_number = n
   - Respond PROMISE(n, accepted_number, accepted_value)
2. Else:
   - Ignore (or send reject)
```

**Phase 2: Accept**

```
Proposer (receives PROMISE from majority):
1. If any acceptor had accepted_value:
   - value = accepted_value from highest accepted_number
2. Else:
   - value = proposer's own value
3. Send ACCEPT(n, value) to acceptors

Acceptor (receives ACCEPT(n, value)):
1. If n >= promised_number:
   - Set accepted_number = n
   - Set accepted_value = value
   - Send ACCEPTED(n, value) to learners
2. Else:
   - Ignore
```

**Learning**:

```
Learner (receives ACCEPTED messages):
1. If received ACCEPTED(n, value) from majority:
   - Value is chosen!
   - Consensus reached
```

**Why It Works**:

**Safety**: Once a value is chosen, it's the only value that will ever be chosen
- Proof: Any future majority overlaps with choosing majority by at least one acceptor
- That acceptor will report the chosen value in its PROMISE
- Proposer must propose that value

**Liveness**: Progress eventually happens (with some assumptions)
- If one proposer is stable, it will eventually succeed
- Higher proposal numbers override lower ones
- Distinguished proposer (leader) ensures progress

**Example Scenario**:

```
3 Acceptors (A1, A2, A3), need 2 for majority

Proposer P1:
1. Sends PREPARE(n=1) to all
2. Gets PROMISE from A1, A2 (majority!)
   - Both report no accepted value
3. Sends ACCEPT(n=1, value="foo") to all
4. Gets ACCEPTED from A1, A2 (majority!)
   - Consensus: "foo"

Concurrent Proposer P2:
1. Sends PREPARE(n=2) to all (after P1's PREPARE but before ACCEPT)
2. Gets PROMISE from A2, A3
   - Both report no accepted value
3. Sends ACCEPT(n=2, value="bar") to all
4. A1 ignores (promised n=1, but 2>1, so actually accepts!)
5. Gets ACCEPTED from A2, A3
   - Consensus: "bar"

Resolution: Higher proposal number (2) wins
```

**Optimizations in Implementation**:

1. **Multi-Paxos**: Run multiple Paxos instances for sequence of values
2. **Leader Election**: One stable proposer improves liveness
3. **Fast Path**: Skip Phase 1 if proposer already has promises
4. **Batching**: Group multiple values in one proposal

---

### 8. Sync Behavior Manager (`behavior.rs`)

**What**: Orchestrates all sync components and manages the synchronization lifecycle.

**Why**: Provides a unified interface to the complex sync system.

```rust
pub struct SyncManager {
    config: SyncManagerConfig,
    local_peer: PeerId,
    vector_clock: VectorClock,

    // Sub-components
    gossip: GossipScheduler,
    brb: ByzantineReliableBroadcast,
    reputation: SimpleReputationSystem,
    challenges: ChallengeSystem,
    paxos: HashMap<u64, PaxosInstance>,  // Instance per value

    // State
    peer_states: HashMap<PeerId, PeerSyncState>,
    pending_syncs: HashMap<PeerId, SyncState>,
}
```

**Key Responsibilities**:

1. **Peer Management**:
   ```rust
   pub fn add_peer(&mut self, peer_id: PeerId) {
       // Issue challenge if Sybil resistance enabled
       if self.config.sybil_resistance.enabled {
           let challenge = self.challenges.issue_challenge(peer_id);
           // Send challenge to peer via network
       }

       // Initialize reputation
       self.reputation.reputations.entry(peer_id)
           .or_insert(PeerReputation::default());

       // Add to peer list
       self.peer_states.insert(peer_id, PeerSyncState::new());
   }
   ```

2. **Periodic Gossip**:
   ```rust
   pub fn tick(&mut self) {
       if self.gossip.should_gossip() {
           // Select random peers
           let peers = self.gossip.select_peers(&self.peer_states, self.config.fanout);

           for peer in peers {
               // Exchange state digests
               let digest = self.compute_local_digest();
               // Send digest to peer
               // Receive peer's digest
               // Compute delta
               // Exchange missing records
           }

           self.gossip.mark_gossiped();
       }

       // Clean up expired challenges
       self.challenges.cleanup_expired();

       // Apply reputation decay
       if self.config.reputation.decay_enabled {
           self.reputation.apply_decay_all();
       }
   }
   ```

3. **Message Handling**:
   ```rust
   pub fn handle_message(&mut self, peer: PeerId, msg: SyncMessage) {
       // Update vector clock
       self.vector_clock.merge(&msg.clock);

       match msg.payload {
           Payload::Gossip(state) => {
               self.handle_gossip(peer, state);
               self.reputation.record_success(&peer);
           }
           Payload::BrbEcho(echo) => {
               if self.brb.handle_echo(peer, echo).is_ok() {
                   self.reputation.record_success(&peer);
               } else {
                   self.reputation.record_failure(&peer);
               }
           }
           Payload::BrbReady(ready) => {
               if self.brb.handle_ready(peer, ready).is_ok() {
                   self.reputation.record_success(&peer);
               } else {
                   self.reputation.record_failure(&peer);
               }
           }
           Payload::Paxos(paxos_msg) => {
               self.handle_paxos(peer, paxos_msg);
           }
       }
   }
   ```

4. **Sync Initiation**:
   ```rust
   pub fn sync_with_peer(&mut self, peer: PeerId) -> Result<()> {
       // Check if peer is verified
       if self.config.sybil_resistance.enabled {
           if !self.challenges.is_verified(&peer) {
               return Err(anyhow!("Peer not verified"));
           }
       }

       // Check reputation
       if self.reputation.reputation(&peer) < self.config.min_reputation {
           return Err(anyhow!("Peer reputation too low"));
       }

       // Initiate sync
       self.pending_syncs.insert(peer, SyncState::InProgress);

       // Exchange state
       // ...

       Ok(())
   }
   ```

---

### 9. Request-Response Protocol (`protocol.rs` + `codec.rs`)

**What**: libp2p-based request-response protocol for peer-to-peer sync communication.

**Why**: Provides reliable, ordered, and authenticated message delivery.

#### Message Types (`protocol.rs`):

```rust
pub enum SyncRequest {
    // State queries
    GetStateDigest,
    GetRecords { collection: String, keys: Vec<Vec<u8>> },
    GetRecordsSince { collection: String, since: VectorClock },

    // Sybil resistance
    GetChallenge,
    SubmitProof { proof: ProofOfWork },

    // Byzantine Reliable Broadcast
    BrbEcho { message_id: Vec<u8>, payload_hash: Vec<u8>, signature: Vec<u8> },
    BrbReady { message_id: Vec<u8>, payload_hash: Vec<u8>, signature: Vec<u8> },

    // Consensus
    Paxos { message: PaxosMessage },
}

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

pub struct SyncRecord {
    key: Vec<u8>,              // Record identifier
    value: Vec<u8>,            // Serialized data
    version: u64,              // Version number
    peer_id: Vec<u8>,          // Originating peer
    vector_clock: VectorClock, // Causality information
}
```

#### Codec (`codec.rs`):

**Wire Format**:
```
+-------------------+
| Length (4 bytes)  |  ← Big-endian u32
+-------------------+
| JSON payload      |  ← Serialized message
+-------------------+
```

**Implementation**:
```rust
pub struct SyncCodec;

impl Codec for SyncCodec {
    type Protocol = String;
    type Request = SyncRequest;
    type Response = SyncResponse;

    async fn read_request<T>(&mut self, _: &str, io: &mut T) -> io::Result<SyncRequest>
    where T: AsyncRead + Unpin + Send
    {
        // Read 4-byte length
        let mut len_buf = [0u8; 4];
        io.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        // Check size limit
        if len > Self::MAX_MESSAGE_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Too large"));
        }

        // Read payload
        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;

        // Deserialize
        serde_json::from_slice(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    // Similar for write_request, read_response, write_response
}
```

**Why JSON**:
- Human-readable for debugging
- Language-agnostic (easier interop)
- Built-in support for complex types
- Trade-off: Larger messages vs. flexibility

---

## Data Flow

### Example: Full Sync Lifecycle

**Scenario**: Peer A updates a record, change propagates to Peer B

#### 1. Local Update

```
Application ──[Update Record]──> Netabase
```

```rust
// User code
netabase.update_record(collection, key, value)?;

// Internally:
// 1. Store in local database
// 2. Increment vector clock
// 3. Update Merkle tree
// 4. Trigger sync if auto_sync enabled
```

#### 2. Gossip Propagation

**After gossip interval**:

```
Peer A ──[GetStateDigest]──> Peer B
Peer A <──[StateDigest]────── Peer B
```

```rust
// Peer A (sender)
let digest = compute_state_digest()?;
send_request(peer_b, SyncRequest::GetStateDigest)?;

let peer_b_digest = receive_response()?;
if digest.merkle_root != peer_b_digest.merkle_root {
    // States differ, need to sync
}
```

#### 3. Delta Calculation

```
Peer A ──[GetRecordsSince(clock)]──> Peer B
Peer A <──[Records(delta)]────────── Peer B
```

```rust
// Peer B (responder)
match request {
    SyncRequest::GetRecordsSince { since } => {
        // Find records newer than 'since' clock
        let delta = database.records_newer_than(&since)?;

        let records = delta.into_iter()
            .map(|rec| SyncRecord {
                key: rec.key,
                value: rec.serialize()?,
                version: rec.version,
                peer_id: local_peer.to_bytes(),
                vector_clock: rec.clock.clone(),
            })
            .collect();

        SyncResponse::Records { collection, records }
    }
}
```

#### 4. Merge and Apply

```
Peer A receives delta
Peer A merges records with CRDT semantics
Peer A updates local state
```

```rust
// Peer A (receiver)
for record in records {
    // Parse peer ID
    let peer_id = PeerId::from_bytes(&record.peer_id)?;

    // Deserialize value
    let value = bincode::decode_from_slice(&record.value, config)?;

    // Merge vector clocks
    local_clock.merge(&record.vector_clock);

    // Apply update with CRDT merge
    if let Some(existing) = database.get(&record.key)? {
        let merged = crdt_merge(existing, value, &record.vector_clock)?;
        database.put(&record.key, merged)?;
    } else {
        database.put(&record.key, value)?;
    }

    // Update reputation
    reputation.record_success(&peer_id);
}
```

#### 5. Verification (Optional)

If Byzantine fault tolerance is needed:

```
Peer A ──[BrbEcho(msg)]──> All Peers
All ───> [BrbEcho]──> Peer A (receives 2f+1)
Peer A ──[BrbReady(msg)]──> All Peers
All ───> [BrbReady]──> Peer A (receives 2f+1)
Peer A delivers message as confirmed
```

---

## Security Model

### Threat Model

**Assumptions**:
1. Network is asynchronous (messages can be delayed, reordered, or lost)
2. Up to `f` out of `3f+1` peers are Byzantine (malicious)
3. Attackers can:
   - Send arbitrary messages
   - Create fake identities (Sybil attack)
   - Collude with other Byzantine peers
   - Drop or delay messages
4. Attackers cannot:
   - Break cryptographic primitives (signatures, hashes)
   - Control more than `f` peers
   - Jam the network completely

### Security Guarantees

#### 1. Byzantine Fault Tolerance

**Guarantee**: Honest peers agree on the same state, even with Byzantine peers.

**How**:
- BRB ensures all honest peers deliver same messages
- Quorums (2f+1) prevent Byzantine minorities from controlling decisions
- Signatures prevent message forgery
- Vector clocks prevent causal violations

#### 2. Sybil Resistance

**Guarantee**: Creating fake identities is expensive.

**How**:
- Proof-of-Work requires computational cost per identity
- Reputation system penalizes misbehavior
- Time-limited verifications require repeated work
- Challenge randomness prevents pre-computation

#### 3. Integrity

**Guarantee**: Records cannot be tampered with undetected.

**How**:
- Merkle trees make tampering detectable
- Signatures authenticate record sources
- Version numbers prevent replay attacks
- Vector clocks detect causality violations

#### 4. Availability

**Guarantee**: System remains available despite failures.

**How**:
- Gossip is resilient to peer failures
- No single point of failure
- Data replicated across multiple peers
- System degrades gracefully under attack

### Attack Scenarios and Defenses

#### Attack 1: Sybil Attack

**Attack**: Create 1000 fake peers to overwhelm honest peers.

**Defense**:
```rust
// Each fake peer must complete PoW
for _ in 0..1000 {
    let proof = generate_pow(challenge, difficulty=20);
    // ~1 second per peer = 1000 seconds total
}

// Verifications expire
after verification_duration {
    // Must re-do PoW for all 1000 peers
}

// Reputation starts at default
// Fake peers need time to build trust
// Malicious behavior drops reputation to 0
```

**Cost to Attacker**: 1000x honest peer cost, and ongoing

#### Attack 2: Eclipse Attack

**Attack**: Surround victim peer with Byzantine peers to control its view.

**Defense**:
```rust
// Random peer selection in gossip
let peers = shuffle(all_peers);
let selected = peers.into_iter().take(fanout);

// Reputation-based filtering
let trusted = peers.into_iter()
    .filter(|p| reputation(p) > threshold);

// Multiple redundant sources
// Victim connects to many peers
// Byzantine peers can't dominate all connections
```

#### Attack 3: Equivocation

**Attack**: Send different messages with same vector clock to different peers.

**Defense**:
```rust
// Byzantine filter detects inconsistency
if received_states.contains_conflicting(&peer, &clock) {
    // Peer sent two different states with same clock!
    reputation.record_failure(&peer);
    blacklist.insert(peer);

    // Share evidence via gossip
    broadcast_misbehavior_evidence(peer, evidence);
}
```

#### Attack 4: Selective Message Dropping

**Attack**: Byzantine peer ignores messages from specific honest peers.

**Defense**:
```rust
// Gossip protocol is push-based
// If B ignores A's messages, other peers propagate
// Eventually B receives from C, D, E, ...

// Timeout detection
if peer_not_responding(&peer) {
    reputation.record_failure(&peer);
    // Try different peers
}
```

---

## Performance Considerations

### 1. Gossip Overhead

**Problem**: Too frequent gossip wastes bandwidth.

**Solution**:
```rust
pub struct GossipConfig {
    interval: Duration,     // Default: 10 seconds
    fanout: usize,          // Default: 3 peers
    max_batch_size: usize,  // Default: 100 records
}

// Total network traffic per round:
// N peers × fanout × message_size
// = N × 3 × ~1KB (for digest)
// = 3N KB per round

// For 100 peers: 300 KB per round
// For 1000 peers: 3 MB per round
```

**Optimization**: Use state digests (Merkle roots) instead of full state
- 32 bytes instead of potentially MB of data
- Only transfer delta when roots differ

### 2. BRB Message Amplification

**Problem**: BRB requires broadcasting to all peers.

**Solution**:
```rust
// Only use BRB for critical operations
// Routine updates use gossip instead

match update_criticality {
    Critical => brb.broadcast(update),      // O(N²) messages
    Routine => gossip.propagate(update),    // O(N log N) messages
}
```

### 3. Proof-of-Work Latency

**Problem**: PoW verification delays peer joining.

**Solution**:
```rust
// Adjustable difficulty
pub struct ProofOfWorkConfig {
    difficulty: u32,  // Start low, increase if attacks detected
}

// Background verification
spawn(async move {
    let proof = generate_pow(challenge, difficulty);
    send_proof(proof).await;
});
// Peer can continue other operations while computing
```

### 4. Vector Clock Size

**Problem**: Vector clocks grow with number of peers.

**Solution**:
```rust
// Garbage collection
impl VectorClock {
    pub fn gc(&mut self, active_peers: &HashSet<PeerId>) {
        // Remove entries for peers that left
        self.clocks.retain(|peer, _| active_peers.contains(peer));
    }
}

// Alternative: Use bounded version vectors
// Keep only K largest clocks
```

### 5. Reputation Decay Computation

**Problem**: Computing decay for all peers is expensive.

**Solution**:
```rust
// Lazy decay: only compute when accessed
pub fn reputation(&mut self, peer: &PeerId) -> f64 {
    let rep = self.reputations.entry(*peer).or_default();
    rep.apply_decay();  // Compute decay on-demand
    rep.score
}

// Batch decay for active peers only
pub fn gc(&mut self, active_peers: &HashSet<PeerId>) {
    self.reputations.retain(|peer, _| active_peers.contains(peer));
}
```

---

## Integration Guide

### Basic Setup

```rust
use netabase::sync::{SyncManagerPresets, NetabaseWithSync};
use netabase::network::config::{NetabaseConfig, SyncConfig};

// 1. Configure sync
let sync_config = SyncConfig {
    enabled: true,
    gossip: GossipConfig {
        enabled: true,
        interval: Duration::from_secs(10),
        fanout: 3,
    },
    brb: BrbConfig {
        enabled: true,
        total_peers: 7,
        max_faulty: 2,
    },
    sybil_resistance: SybilResistanceConfig {
        enabled: true,
        pow_difficulty: 16,
        challenge_duration: Duration::from_secs(60),
        verification_duration: Duration::from_secs(3600),
        reputation_enabled: true,
    },
    paxos: PaxosConfig {
        enabled: false,  // Only for critical operations
        num_acceptors: 5,
        max_failures: 2,
    },
    auto_sync: true,
    sync_interval: Duration::from_secs(30),
};

// 2. Create Netabase instance
let mut netabase = Netabase::<MyDefinition>::new()?;
netabase.start_swarm().await?;

// 3. Sync is now active!
// Gossip runs automatically every 10 seconds
// New peers must complete PoW challenges
// Reputation tracks peer behavior
```

### Advanced: Custom Sync Logic

```rust
// Handle sync events manually
loop {
    match swarm.next_event().await {
        NetabaseBehaviourEvent::Sync(sync_event) => {
            use libp2p::request_response::Event;
            match sync_event {
                Event::Message { peer, message } => {
                    match message {
                        Message::Request { request, channel, .. } => {
                            let response = handle_sync_request(request)?;
                            swarm.behaviour_mut().sync.send_response(channel, response)?;
                        }
                        Message::Response { response, .. } => {
                            handle_sync_response(peer, response)?;
                        }
                    }
                }
                Event::OutboundFailure { peer, error, .. } => {
                    eprintln!("Sync failed with {}: {}", peer, error);
                    reputation.record_failure(&peer);
                }
                _ => {}
            }
        }
        _ => {}
    }
}
```

### Performance Tuning

```rust
// Low-latency network (LAN)
let config = SyncConfig {
    gossip: GossipConfig {
        interval: Duration::from_secs(1),  // Gossip frequently
        fanout: 5,                          // More redundancy
    },
    sync_interval: Duration::from_secs(5),
    ..Default::default()
};

// High-latency network (WAN)
let config = SyncConfig {
    gossip: GossipConfig {
        interval: Duration::from_secs(30),  // Gossip less often
        fanout: 3,                           // Fewer peers
    },
    sync_interval: Duration::from_secs(60),
    ..Default::default()
};

// Security-critical application
let config = SyncConfig {
    sybil_resistance: SybilResistanceConfig {
        enabled: true,
        pow_difficulty: 24,  // Higher difficulty
        verification_duration: Duration::from_secs(1800),  // Re-verify more often
        reputation_enabled: true,
    },
    brb: BrbConfig {
        enabled: true,
        total_peers: 10,
        max_faulty: 3,  // Tolerate more faults
    },
    ..Default::default()
};
```

---

## Conclusion

The Netabase sync architecture provides a comprehensive solution for Byzantine fault-tolerant state synchronization in open networks. By combining multiple complementary techniques:

- **Gossip** for efficient state propagation
- **BRB** for critical update guarantees
- **PoW + Reputation** for Sybil resistance
- **Vector Clocks** for causality tracking
- **CRDTs** for conflict resolution
- **Paxos** for strong consistency when needed

The system achieves a balance between:
- **Safety** (correctness despite Byzantine faults)
- **Liveness** (making progress despite failures)
- **Performance** (scalable to large networks)
- **Flexibility** (configurable for different use cases)

All components are designed to work together seamlessly while remaining modular and configurable, allowing users to tune the system for their specific requirements.
