# Netabase Sync Integration Guide

This guide shows how to integrate the sync module with your Netabase application for Byzantine fault-tolerant state synchronization.

## Overview

The sync module provides:
- **State Synchronization**: Gossip-based anti-entropy for eventual consistency
- **Byzantine Tolerance**: Reliable broadcast for critical updates
- **Sybil Resistance**: PoW challenges and reputation-based filtering
- **Paxos Consensus**: Distributed agreement for critical decisions

## Basic Integration

### 1. Enable Sync in Your Application

```rust
use netabase::{Netabase, sync::{SyncBehaviorManager, SyncManagerPresets}};
use netabase_store::netabase_definition_module;

#[netabase_definition_module(MyDefinition, MyKeys)]
mod my_def {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
    #[netabase(MyDefinition)]
    pub struct MyData {
        #[primary_key]
        pub id: u64,
        pub content: String,
    }
}

use my_def::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Netabase instance
    let mut netabase = Netabase::<MyDefinition>::new()?;

    // Start the swarm
    netabase.start_swarm().await?;

    // Get local peer ID for sync
    let peer_id = netabase.local_peer_id().await?;

    // Create sync manager with production config
    let sync_config = SyncManagerPresets::production();
    let mut sync_manager = SyncBehaviorManager::new(peer_id, sync_config)?;

    // Use sync manager alongside Netabase
    // (See advanced integration for event handling)

    Ok(())
}
```

### 2. Configuration Presets

Choose a preset based on your use case:

```rust
use netabase::sync::{SyncPresets, SyncManagerPresets};

// Development: Fast, minimal security
let dev_config = SyncManagerPresets::development();

// Production: Balanced security and performance
let prod_config = SyncManagerPresets::production();

// High Security: Maximum Byzantine tolerance
let secure_config = SyncManagerPresets::high_security();
```

### 3. Custom Configuration

```rust
use netabase::sync::{SyncManagerConfigBuilder, ProofOfWorkConfig};
use std::time::Duration;

let config = SyncManagerConfigBuilder::new()
    .gossip_interval(Duration::from_secs(10))
    .gossip_fanout(3)
    .brb_config(7, 2)  // 7 nodes, tolerate 2 Byzantine faults
    .pow_difficulty(16)
    .pow_enabled(true)
    .challenge_duration(Duration::from_secs(60))
    .verification_duration(Duration::from_secs(3600))
    .build();
```

## Advanced Integration

### Peer Discovery via Kademlia

The sync module works alongside Netabase's Kademlia DHT for peer discovery:

```rust
use netabase::NetabaseSwarmEvent;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut netabase = Netabase::<MyDefinition>::new()?;
    netabase.start_swarm().await?;

    let peer_id = netabase.local_peer_id().await?;
    let config = SyncManagerPresets::production();
    let mut sync_manager = SyncBehaviorManager::new(peer_id, config)?;

    // Periodic sync ticker
    let mut sync_ticker = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            // Handle Netabase events
            event = netabase.next_event() => {
                match event {
                    NetabaseSwarmEvent::PeerDiscovered(peer_id) => {
                        // Add discovered peer to sync
                        sync_manager.add_peer(peer_id);
                        println!("Added peer to sync: {}", peer_id);
                    }
                    NetabaseSwarmEvent::PeerDisconnected(peer_id) => {
                        // Remove disconnected peer
                        sync_manager.remove_peer(&peer_id);
                    }
                    _ => {}
                }
            }

            // Periodic sync maintenance
            _ = sync_ticker.tick() => {
                // Issue challenges to unverified peers
                // Clean up expired challenges
                sync_manager.cleanup();
            }
        }
    }
}
```

### Sybil Resistance

Protect against Sybil attacks using PoW challenges:

```rust
// When a new peer connects
let new_peer = discovered_peer_id;

// Issue challenge
let challenge = sync_manager.issue_challenge(new_peer);

// Send challenge to peer (via custom protocol)
send_challenge_to_peer(&new_peer, &challenge).await?;

// When peer responds with proof
let proof = receive_proof_from_peer(&new_peer).await?;

// Verify proof
match sync_manager.verify_challenge(&new_peer, &proof) {
    Ok(_) => {
        println!("Peer {} verified!", new_peer);
        // Now trust this peer for sync
    }
    Err(e) => {
        println!("Peer {} failed verification: {}", new_peer, e);
        sync_manager.remove_peer(&new_peer);
    }
}
```

### Reputation-Based Filtering

Track peer behavior and filter low-reputation peers:

```rust
use netabase::sync::{SimpleReputationSystem, ReputationSystem};

let mut reputation = SimpleReputationSystem::new();

// Record successful sync
reputation.record_success(&peer_id);

// Record failed sync
reputation.record_failure(&peer_id);

// Get top peers for preferential sync
let top_peers = reputation.top_peers(10);
for peer in top_peers {
    // Sync with high-reputation peers
}

// Check reputation before accepting data
if reputation.reputation(&peer_id) < 0.3 {
    println!("Low reputation peer, rejecting data");
    return;
}
```

### Paxos Consensus for Critical Decisions

Use Paxos when you need distributed agreement:

```rust
use netabase::sync::{PaxosInstance, PaxosConfig, PaxosMessage};

// Setup (e.g., 5 nodes, tolerate 2 failures)
let config = PaxosConfig::new(5, 2);
let mut paxos = PaxosInstance::new(peer_id, config);

// Propose a value
let value = bincode::encode_to_vec(&my_critical_decision, bincode::config::standard())?;
let proposal = paxos.propose(value);

// Handle Paxos messages from network
// (This requires implementing message passing between peers)
match paxos_message {
    PaxosMessage::Prepare { proposal_number } => {
        let promise = paxos.handle_prepare(proposal_number)?;
        // Send promise to proposer
    }
    PaxosMessage::Promise { proposal_number, accepted_proposal, accepted_value } => {
        if let Some(accept_msg) = paxos.handle_promise(
            from_peer,
            proposal_number,
            accepted_proposal,
            accepted_value,
        )? {
            // Send accept to all acceptors
        }
    }
    // Handle other Paxos phases...
    _ => {}
}
```

## Sync with Typed Records

While the base sync uses byte arrays, you can create typed wrappers:

```rust
use bincode::{Encode, Decode};

#[derive(Encode, Decode, Clone, Debug)]
struct TypedSyncPayload {
    discriminant: String,  // Model type
    key: Vec<u8>,
    value: Vec<u8>,
    timestamp: u64,
}

// Convert Netabase record to sync payload
fn record_to_payload<M: NetabaseModelTrait>(record: &M) -> TypedSyncPayload {
    TypedSyncPayload {
        discriminant: std::any::type_name::<M>().to_string(),
        key: bincode::encode_to_vec(&record.primary_key(), bincode::config::standard()).unwrap(),
        value: bincode::encode_to_vec(record, bincode::config::standard()).unwrap(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}
```

## Performance Tuning

### Gossip Configuration

```rust
// High throughput: More frequent, wider fanout
.gossip_interval(Duration::from_secs(5))
.gossip_fanout(5)

// Low bandwidth: Less frequent, narrow fanout
.gossip_interval(Duration::from_secs(30))
.gossip_fanout(2)
```

### Byzantine Tolerance

```rust
// Small network: f=1 (needs 4 nodes)
.brb_config(4, 1)

// Medium network: f=2 (needs 7 nodes)
.brb_config(7, 2)

// Large network: f=3 (needs 10 nodes)
.brb_config(10, 3)
```

### PoW Difficulty

```rust
// Testing: Very fast
.pow_difficulty(4)

// Light protection: ~1 second
.pow_difficulty(16)

// Strong protection: ~10 seconds
.pow_difficulty(20)
```

## Architecture Patterns

### Pattern 1: Parallel Operation

Run sync independently alongside Netabase:

```rust
// Netabase handles DHT and record storage
// Sync handles state propagation and Byzantine tolerance
tokio::spawn(async move {
    sync_manager.run_periodic_sync().await
});
```

### Pattern 2: Integrated Events

Handle sync events in main event loop:

```rust
loop {
    tokio::select! {
        netabase_event = netabase.next_event() => {
            // Handle Netabase events
        }
        sync_event = sync_event_receiver.recv() => {
            // Handle sync events
        }
    }
}
```

### Pattern 3: Layered Sync

Different sync modes for different data:

```rust
// Critical data: Use BRB with signatures
let critical_config = SyncManagerConfigBuilder::new()
    .brb_config(7, 2)
    .pow_enabled(true)
    .build();

// Regular data: Use gossip only
let regular_config = SyncManagerConfigBuilder::new()
    .gossip_interval(Duration::from_secs(10))
    .build();
```

## Security Considerations

1. **Always enable signature verification in production**:
   ```rust
   .pow_enabled(true)
   ```

2. **Use appropriate Byzantine tolerance**:
   - f=1 for trusted environments
   - f=2+ for public/adversarial networks

3. **Monitor reputation scores**:
   ```rust
   if reputation.reputation(&peer) < MINIMUM_REPUTATION {
       disconnect_peer(peer);
   }
   ```

4. **Validate all incoming data**:
   ```rust
   if !validate_record_integrity(&record) {
       return Err("Invalid record");
   }
   ```

## Troubleshooting

### Sync Not Working
- Ensure peers are discovered via Kademlia first
- Check that sync manager has peers added
- Verify network connectivity

### High Latency
- Reduce gossip interval
- Increase fanout
- Consider disabling BRB for non-critical data

### Sybil Attacks
- Increase PoW difficulty
- Enable reputation system
- Set minimum reputation threshold

## Example: Complete Integration

See `examples/sync_demo.rs` for a complete working example that demonstrates:
- Netabase setup with sync
- Peer discovery and challenge
- State synchronization
- Reputation tracking
- Paxos consensus

## Next Steps

1. Start with `SyncManagerPresets::development()` for testing
2. Move to `SyncManagerPresets::production()` for deployment
3. Monitor and tune based on your network characteristics
4. Consider implementing custom sync protocols for specific use cases

## API Reference

See `src/sync/README.md` for detailed API documentation.
