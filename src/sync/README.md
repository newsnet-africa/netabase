# Netabase Synchronization Module

Byzantine fault-tolerant synchronization for state replication across peers in an open/permissionless network.

## Overview

The sync module provides:

- **Anti-entropy gossip protocol** for state synchronization
- **Byzantine Reliable Broadcast (BRB)** for critical updates with Byzantine fault tolerance
- **CRDT integration** for conflict-free merges
- **Sybil resistance** mechanisms (PoW, Reputation, Stake)
- **Vector clocks** for causality tracking
- **Paxos consensus** for distributed agreement

## Quick Start

```rust
use netabase::sync::{
    SyncBehaviorManager, SyncManagerConfig,
    SyncEvent, ProofOfWorkConfig,
};
use libp2p::PeerId;
use std::time::Duration;

// Create a sync manager with default configuration
let peer_id = PeerId::random();
let config = SyncManagerConfig::default();
let mut sync_manager = SyncBehaviorManager::new(peer_id, config)?;

// Add peers (discovered via Kademlia DHT)
let other_peer = PeerId::random();
sync_manager.add_peer(other_peer);

// Issue challenge for Sybil resistance
let challenge = sync_manager.issue_challenge(other_peer);
```

## Architecture

### Core Components

#### 1. SyncManager (behavior.rs)
Main synchronization coordinator that integrates:
- Gossip manager for state propagation
- BRB manager for critical updates
- Challenge system for Sybil resistance

```rust
use netabase::sync::{SyncBehaviorManager, SyncManagerConfig};

let config = SyncManagerConfig {
    gossip: GossipConfig::default(),
    brb: BrbConfig::new(7, 2), // n=7 nodes, f=2 Byzantine faults
    pow: ProofOfWorkConfig { difficulty: 4, enabled: true },
    challenge_duration: Duration::from_secs(60),
    verification_duration: Duration::from_secs(3600),
};

let mut manager = SyncBehaviorManager::new(peer_id, config)?;
```

#### 2. Gossip Protocol (gossip/)
Anti-entropy gossip for eventual consistency:
- Periodic state exchange with random peers
- Merkle tree-based state comparison
- Delta synchronization for efficiency

#### 3. Byzantine Reliable Broadcast (brb/)
Three-phase atomic broadcast:
1. **Init**: Proposer broadcasts message
2. **Echo**: Acceptors echo after validation
3. **Ready**: Acceptors send ready after quorum
4. **Deliver**: Message delivered after ready quorum

```rust
use netabase::sync::brb::{BrbManager, BrbConfig};

let config = BrbConfig::new(7, 2); // 7 nodes, tolerate 2 Byzantine
let mut brb = BrbManager::new(config, local_peer_id)?;
```

#### 4. Sybil Resistance (proof/)
Protection against Sybil attacks:

**Proof-of-Work Challenge:**
```rust
use netabase::sync::{ChallengeSystem, ProofOfWorkConfig};

let config = ProofOfWorkConfig { difficulty: 4, enabled: true };
let mut challenges = ChallengeSystem::new(
    config,
    Duration::from_secs(60),  // Challenge duration
    Duration::from_secs(3600) // Verification validity
);

// Issue challenge to new peer
let challenge = challenges.issue_challenge(peer_id);

// Peer solves challenge and submits proof
let proof = solve_challenge(&challenge);

// Verify proof
challenges.verify_response(&peer_id, &proof)?;
```

#### 5. Reputation System (reputation.rs)
Track peer behavior with decay:

```rust
use netabase::sync::SimpleReputationSystem;

let mut reputation = SimpleReputationSystem::new(); // Decay enabled by default

// Record interactions
reputation.record_success(&peer_id);
reputation.record_failure(&peer_id);

// Query reputation
let score = reputation.reputation(&peer_id); // 0.0 to 1.0

// Get statistics
let (successes, failures) = reputation.get_stats(&peer_id).unwrap();

// Get top peers
let top_peers = reputation.top_peers(10);
```

Features:
- **Time-based decay** toward default reputation (0.5)
- **Diminishing returns** for repeated good behavior
- **Larger penalties** for failures
- **Interaction statistics** tracking

#### 6. Paxos Consensus (paxos/)
Distributed agreement for critical decisions:

```rust
use netabase::sync::{PaxosInstance, PaxosConfig, PaxosMessage};

let config = PaxosConfig::new(5, 2); // 5 acceptors, tolerate 2 failures
let mut paxos = PaxosInstance::new(local_peer_id, config);

// Propose a value
let value = b"proposed value".to_vec();
let proposal = paxos.propose(value);

// Handle messages from other nodes
match incoming_message {
    PaxosMessage::Prepare { proposal_number } => {
        let promise = paxos.handle_prepare(proposal_number)?;
        // Send promise to proposer
    }
    PaxosMessage::Promise { .. } => {
        if let Some(accept_msg) = paxos.handle_promise(..)? {
            // Send accept to all acceptors
        }
    }
    PaxosMessage::Accept { proposal_number, value } => {
        let accepted = paxos.handle_accept(proposal_number, value)?;
        // Send accepted to all learners
    }
    PaxosMessage::Accepted { .. } => {
        let consensus = paxos.handle_accepted(..)?;
        if consensus {
            // Consensus reached!
        }
    }
}
```

## Configuration

### Sync Configuration
```rust
use netabase::sync::{SyncConfig, ByzantineTolerance, SybilResistanceMode};

let config = SyncConfig {
    enabled: true,
    gossip_interval: Duration::from_secs(10),
    gossip_fanout: 3, // Gossip with 3 random peers per round
    byzantine_tolerance: ByzantineTolerance {
        max_faulty_nodes: 1,
        enable_brb: true,
        verify_signatures: true,
        require_quorum: true,
    },
    sybil_resistance: SybilResistanceMode::Reputation,
    max_sync_batch_size: 100,
    signature_required: true,
    max_concurrent_syncs: 5,
    sync_timeout: Duration::from_secs(30),
};
```

### Byzantine Tolerance Modes

#### 1. No Tolerance
```rust
SybilResistanceMode::None
```
Trust all peers (suitable for private/trusted networks).

#### 2. Proof-of-Work
```rust
SybilResistanceMode::ProofOfWork { difficulty: 4 }
```
Require computational proof for state updates.

#### 3. Reputation-Based
```rust
SybilResistanceMode::Reputation
```
Track peer behavior and filter low-reputation peers.

#### 4. Stake-Based
```rust
SybilResistanceMode::Stake { minimum_stake: 1000 }
```
Require staked tokens (placeholder - requires token integration).

## Integration with Netabase

The sync module integrates with Netabase's NetworkBehaviour:

```rust
use netabase::sync::SyncBehaviorManager;

// In your NetworkBehaviour implementation
pub struct NetabaseBehaviour {
    kademlia: Kademlia<MemoryStore>,
    identify: identify::Behaviour,
    mdns: mdns::tokio::Behaviour,
    sync: Option<SyncBehaviorManager>, // Add sync manager
}

impl NetabaseBehaviour {
    pub fn with_sync(mut self, sync_config: SyncManagerConfig) -> Result<Self> {
        let peer_id = self.kademlia.local_peer_id();
        self.sync = Some(SyncBehaviorManager::new(*peer_id, sync_config)?);
        Ok(self)
    }

    pub async fn handle_sync_message(
        &mut self,
        from: &PeerId,
        message: SyncMessage,
    ) -> Result<()> {
        if let Some(sync) = &mut self.sync {
            sync.handle_sync_message(&self.store, from, message).await?;
        }
        Ok(())
    }
}
```

## Testing

Run tests for each component:

```bash
# All sync tests
cargo test --lib sync

# Specific modules
cargo test --lib sync::gossip
cargo test --lib sync::brb
cargo test --lib sync::proof
cargo test --lib sync::reputation
cargo test --lib sync::paxos
```

## Performance Considerations

### Gossip Configuration
- **gossip_interval**: Lower = faster convergence, higher network load
- **gossip_fanout**: Higher = faster convergence, more network traffic
- Recommended: 3-5 fanout, 5-15s interval

### Byzantine Tolerance
- **max_faulty_nodes**: System requires 3f+1 nodes to tolerate f faults
- BRB has O(n²) message complexity
- Recommended: f=1 for small networks, f=2 for larger

### PoW Difficulty
- **difficulty**: Number of leading zero bits in hash
- difficulty=4: ~milliseconds (testing)
- difficulty=16: ~seconds (light protection)
- difficulty=24: ~minutes (strong protection)

### Reputation Decay
- Default: 0.1 per hour toward 0.5
- Prevents permanent reputation damage
- Gradually forgives past bad behavior

## Security Considerations

1. **Signature Verification**: Always enable in production
2. **Sybil Resistance**: Choose appropriate mechanism for your threat model
3. **Byzantine Quorums**: Ensure sufficient honest nodes (2f+1 for BRB)
4. **Paxos Safety**: Requires majority quorum (f+1 out of 2f+1)

## Future Enhancements

- [ ] Complete libp2p NetworkBehaviour integration
- [ ] Implement actual signature verification (ed25519)
- [ ] Stake-based Sybil resistance with token integration
- [ ] Optimized Merkle tree synchronization
- [ ] Persistent reputation storage
- [ ] Multi-Paxos for sequence agreement
- [ ] Network partition detection and healing

## References

- [Byzantine Reliable Broadcast](https://en.wikipedia.org/wiki/Byzantine_fault#Byzantine_Generals'_Problem)
- [Paxos Made Simple](https://lamport.azurewebsites.net/pubs/paxos-simple.pdf)
- [Gossip Protocols](https://en.wikipedia.org/wiki/Gossip_protocol)
- [CRDTs](https://crdt.tech/)
