//! Synchronization module for distributed data replication
//!
//! This module provides multiple sync protocols that can be combined:
//! - **Gossip**: Epidemic-style eventual consistency
//! - **BRB**: Byzantine Reliable Broadcast
//! - **PoW**: Proof-of-Work based Sybil resistance
//! - **Paxos**: Strong consistency via consensus (in paxos module)

pub mod gossip;
pub mod brb;
pub mod pow;

pub use gossip::{GossipProtocol, GossipMessage};
pub use brb::{ByzantineReliableBroadcast, BrbMessage, BrbState};
pub use pow::{ProofOfWork, PowChallenge, PowSolution};

use crate::network::config::{SyncConfig, GossipConfig, BrbConfig, SybilResistanceConfig};
use std::collections::HashMap;
use libp2p::PeerId;

/// Sync manager that coordinates all sync protocols
pub struct SyncManager {
    config: SyncConfig,
    gossip: Option<GossipProtocol>,
    brb: Option<ByzantineReliableBroadcast>,
    pow: Option<ProofOfWork>,
    verified_peers: HashMap<PeerId, std::time::Instant>,
}

impl SyncManager {
    /// Create a new sync manager with the given configuration
    pub fn new(config: SyncConfig) -> Self {
        let gossip = if config.gossip.enabled {
            Some(GossipProtocol::new(config.gossip.clone()))
        } else {
            None
        };

        let brb = if config.brb.enabled {
            Some(ByzantineReliableBroadcast::new(config.brb.clone()))
        } else {
            None
        };

        let pow = if config.sybil_resistance.enabled {
            Some(ProofOfWork::new(config.sybil_resistance.clone()))
        } else {
            None
        };

        Self {
            config,
            gossip,
            brb,
            pow,
            verified_peers: HashMap::new(),
        }
    }

    /// Check if sync is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Check if auto-sync is enabled
    pub fn is_auto_sync(&self) -> bool {
        self.config.auto_sync
    }

    /// Get the sync interval
    pub fn sync_interval(&self) -> std::time::Duration {
        self.config.sync_interval
    }

    /// Get gossip protocol instance
    pub fn gossip(&self) -> Option<&GossipProtocol> {
        self.gossip.as_ref()
    }

    /// Get mutable gossip protocol instance
    pub fn gossip_mut(&mut self) -> Option<&mut GossipProtocol> {
        self.gossip.as_mut()
    }

    /// Get BRB instance
    pub fn brb(&self) -> Option<&ByzantineReliableBroadcast> {
        self.brb.as_ref()
    }

    /// Get mutable BRB instance
    pub fn brb_mut(&mut self) -> Option<&mut ByzantineReliableBroadcast> {
        self.brb.as_mut()
    }

    /// Get PoW instance
    pub fn pow(&self) -> Option<&ProofOfWork> {
        self.pow.as_ref()
    }

    /// Verify a peer with PoW
    pub fn verify_peer(&mut self, peer: PeerId, solution: &PowSolution) -> bool {
        if let Some(pow) = &self.pow {
            if pow.verify_solution(solution) {
                self.verified_peers.insert(peer, std::time::Instant::now());
                return true;
            }
        }
        false
    }

    /// Check if a peer is verified
    pub fn is_peer_verified(&self, peer: &PeerId) -> bool {
        if let Some(pow) = &self.pow {
            if let Some(verified_at) = self.verified_peers.get(peer) {
                let elapsed = verified_at.elapsed();
                return elapsed < pow.config().verification_validity;
            }
            false
        } else {
            // If PoW is disabled, all peers are considered verified
            true
        }
    }

    /// Clean up expired peer verifications
    pub fn cleanup_expired_verifications(&mut self) {
        if let Some(pow) = &self.pow {
            let validity = pow.config().verification_validity;
            self.verified_peers.retain(|_, verified_at| {
                verified_at.elapsed() < validity
            });
        }
    }
}
