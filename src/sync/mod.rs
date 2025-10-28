//! Synchronization mechanism for Netabase
//!
//! This module provides Byzantine fault-tolerant synchronization for state replication
//! across peers in an open/permissionless network. It implements:
//!
//! - Anti-entropy gossip protocol for state synchronization
//! - Byzantine Reliable Broadcast (BRB) for critical updates
//! - CRDT integration for conflict-free merges
//! - Sybil resistance mechanisms
//! - Vector clocks for causality tracking

pub mod types;
pub mod traits;
pub mod clock;
pub mod gossip;
pub mod brb;
pub mod crdt;
pub mod proof;
pub mod reputation;
pub mod behavior;
pub mod paxos;
pub mod config;
pub mod protocol;
pub mod codec;
#[cfg(feature = "native")]
pub mod netabase_ext;
mod serde_helper;

// Re-export commonly used types
pub use types::{
    SyncMessage, SyncState, SyncStatus, Version, SignedMessage, MessageSignature,
    PeerSyncState, MerkleNode, StateDigest,
};
pub use traits::{Syncable, SyncableStore, ByzantineValidator, ReputationSystem, SybilResistance as SybilResistanceTrait};
pub use clock::{VectorClock, LogicalTimestamp};
pub use behavior::{SyncManager as SyncBehaviorManager, SyncManagerConfig, SyncEvent};
pub use reputation::SimpleReputationSystem;
pub use proof::{ProofOfWork, ProofOfWorkConfig, PoWSystem, ChallengeSystem};
pub use paxos::{PaxosInstance, PaxosConfig, PaxosMessage, ProposalNumber};
pub use config::{SyncConfigBuilder, SyncManagerConfigBuilder, SyncPresets, SyncManagerPresets};
pub use protocol::{SyncRequest, SyncResponse, SyncRecord};
pub use codec::{SyncCodec, SYNC_PROTOCOL};
#[cfg(feature = "native")]
pub use netabase_ext::NetabaseWithSync;

use libp2p::PeerId;
use std::collections::HashMap;

/// Configuration for the synchronization mechanism
#[derive(Clone, Debug)]
pub struct SyncConfig {
    /// Enable/disable synchronization
    pub enabled: bool,

    /// Interval for gossip rounds
    pub gossip_interval: std::time::Duration,

    /// Number of peers to gossip with per round
    pub gossip_fanout: usize,

    /// Byzantine fault tolerance configuration
    pub byzantine_tolerance: ByzantineTolerance,

    /// Sybil resistance mechanism
    pub sybil_resistance: SybilResistanceMode,

    /// Maximum size of a sync batch
    pub max_sync_batch_size: usize,

    /// Require signatures on all messages
    pub signature_required: bool,

    /// Maximum number of concurrent sync operations
    pub max_concurrent_syncs: usize,

    /// Timeout for sync operations
    pub sync_timeout: std::time::Duration,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            gossip_interval: std::time::Duration::from_secs(10),
            gossip_fanout: 3,
            byzantine_tolerance: ByzantineTolerance::default(),
            sybil_resistance: SybilResistanceMode::Reputation,
            max_sync_batch_size: 100,
            signature_required: true,
            max_concurrent_syncs: 5,
            sync_timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Byzantine fault tolerance configuration
#[derive(Clone, Debug)]
pub struct ByzantineTolerance {
    /// Assume at most f Byzantine nodes out of 3f+1 total
    pub max_faulty_nodes: usize,

    /// Enable Byzantine Reliable Broadcast for critical updates
    pub enable_brb: bool,

    /// Verify signatures on all incoming messages
    pub verify_signatures: bool,

    /// Require quorum for state acceptance (2f+1 out of 3f+1)
    pub require_quorum: bool,
}

impl Default for ByzantineTolerance {
    fn default() -> Self {
        Self {
            max_faulty_nodes: 1, // Tolerate 1 Byzantine node out of 4
            enable_brb: true,
            verify_signatures: true,
            require_quorum: true,
        }
    }
}

/// Sybil resistance mechanism configuration
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SybilResistanceMode {
    /// No Sybil resistance (trust all peers)
    None,

    /// Proof-of-Work based (require PoW for state updates)
    ProofOfWork {
        difficulty: u32,
    },

    /// Reputation-based system (track peer behavior)
    Reputation,

    /// Stake-based (require staked tokens)
    Stake {
        minimum_stake: u64,
    },
}

/// Main synchronization manager
pub struct SyncManager {
    /// Configuration
    config: SyncConfig,

    /// Local peer ID
    local_peer_id: PeerId,

    /// Vector clock for this peer
    vector_clock: VectorClock,

    /// Known peer states
    peer_states: HashMap<PeerId, PeerSyncState>,

    /// Current sync status
    status: SyncStatus,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new(config: SyncConfig, local_peer_id: PeerId) -> Self {
        Self {
            config,
            local_peer_id: local_peer_id.clone(),
            vector_clock: VectorClock::new(local_peer_id),
            peer_states: HashMap::new(),
            status: SyncStatus::Idle,
        }
    }

    /// Start synchronization
    pub fn start(&mut self) {
        self.status = SyncStatus::Active;
    }

    /// Stop synchronization
    pub fn stop(&mut self) {
        self.status = SyncStatus::Idle;
    }

    /// Get current sync status
    pub fn status(&self) -> &SyncStatus {
        &self.status
    }

    /// Get local vector clock
    pub fn vector_clock(&self) -> &VectorClock {
        &self.vector_clock
    }

    /// Update local vector clock
    pub fn tick(&mut self) {
        self.vector_clock.increment();
    }

    /// Get peer state
    pub fn peer_state(&self, peer_id: &PeerId) -> Option<&PeerSyncState> {
        self.peer_states.get(peer_id)
    }

    /// Update peer state
    pub fn update_peer_state(&mut self, peer_id: PeerId, state: PeerSyncState) {
        self.peer_states.insert(peer_id, state);
    }

    /// Remove peer state
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peer_states.remove(peer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert!(config.enabled);
        assert_eq!(config.gossip_fanout, 3);
        assert!(config.signature_required);
    }

    #[test]
    fn test_sync_manager_creation() {
        let config = SyncConfig::default();
        let peer_id = PeerId::random();
        let manager = SyncManager::new(config, peer_id);

        assert_eq!(manager.status(), &SyncStatus::Idle);
    }
}
