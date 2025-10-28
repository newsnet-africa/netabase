//! Extension trait for integrating sync with Netabase
//!
//! This module provides a clean API for adding sync functionality to Netabase instances
//! without modifying the core struct.

use crate::sync::{
    SyncBehaviorManager, SimpleReputationSystem, ReputationSystem,
    ProofOfWork, ChallengeSystem, ProofOfWorkConfig,
    PaxosInstance, PaxosConfig as SyncPaxosConfig,
};
use crate::network::config::{
    SyncConfig, GossipConfig, BrbConfig, SybilResistanceConfig, PaxosConfig
};
use anyhow::Result;
use libp2p::PeerId;
use std::sync::{Arc, RwLock};
use netabase_store::traits::{
    definition::NetabaseDefinitionTrait,
    model::NetabaseModelTrait,
};

/// Wrapper that adds sync capabilities to Netabase
pub struct NetabaseWithSync<D>
where
    D: NetabaseDefinitionTrait + Send + Sync,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
{
    /// Sync manager
    sync_manager: SyncBehaviorManager,

    /// Reputation system
    reputation: Arc<RwLock<SimpleReputationSystem>>,

    /// Challenge system
    challenges: Arc<RwLock<ChallengeSystem>>,

    /// Paxos instance (if enabled)
    paxos: Option<Arc<RwLock<PaxosInstance>>>,

    /// Configuration
    config: SyncConfig,

    /// Phantom data
    _phantom: std::marker::PhantomData<D>,
}

impl<D> NetabaseWithSync<D>
where
    D: NetabaseDefinitionTrait + Send + Sync,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
{
    /// Create a new sync wrapper with configuration
    pub fn new(peer_id: PeerId, config: SyncConfig) -> Result<Self> {
        // Convert config to SyncManagerConfig
        let manager_config = config.to_manager_config();
        let sync_manager = SyncBehaviorManager::new(peer_id, manager_config)?;

        let reputation = Arc::new(RwLock::new(SimpleReputationSystem::new()));

        let pow_config = ProofOfWorkConfig {
            difficulty: config.sybil_resistance.pow_difficulty,
            enabled: config.sybil_resistance.enabled,
        };

        let challenges = Arc::new(RwLock::new(ChallengeSystem::new(
            pow_config,
            config.sybil_resistance.challenge_duration,
            config.sybil_resistance.verification_duration,
        )));

        // Initialize Paxos if enabled
        let paxos = if config.paxos.enabled {
            let paxos_config = SyncPaxosConfig::new(
                config.paxos.num_acceptors,
                config.paxos.max_failures
            );
            let paxos_instance = PaxosInstance::new(peer_id, paxos_config);
            Some(Arc::new(RwLock::new(paxos_instance)))
        } else {
            None
        };

        Ok(Self {
            sync_manager,
            reputation,
            challenges,
            paxos,
            config,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Add a peer to sync
    pub fn add_peer(&mut self, peer_id: PeerId) {
        self.sync_manager.add_peer(peer_id);
    }

    /// Remove a peer from sync
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.sync_manager.remove_peer(peer_id);
    }

    /// Issue a challenge to a peer
    pub fn issue_challenge(&mut self, peer_id: PeerId) -> Vec<u8> {
        let mut challenges = self.challenges.write().unwrap();
        challenges.issue_challenge(peer_id)
    }

    /// Verify a challenge response
    pub fn verify_challenge(&mut self, peer_id: &PeerId, proof: &ProofOfWork) -> Result<()> {
        let mut challenges = self.challenges.write().unwrap();
        challenges.verify_response(peer_id, proof)
    }

    /// Check if peer is verified
    pub fn is_peer_verified(&self, peer_id: &PeerId) -> bool {
        let challenges = self.challenges.read().unwrap();
        challenges.is_verified(peer_id)
    }

    /// Record successful interaction with peer
    pub fn record_success(&mut self, peer_id: &PeerId) {
        let mut reputation = self.reputation.write().unwrap();
        reputation.record_success(peer_id);
    }

    /// Record failed interaction with peer
    pub fn record_failure(&mut self, peer_id: &PeerId) {
        let mut reputation = self.reputation.write().unwrap();
        reputation.record_failure(peer_id);
    }

    /// Get peer reputation score
    pub fn peer_reputation(&self, peer_id: &PeerId) -> f64 {
        let mut reputation = self.reputation.write().unwrap();
        reputation.reputation(peer_id)
    }

    /// Get top-reputation peers
    pub fn top_peers(&self, n: usize) -> Vec<PeerId> {
        let reputation = self.reputation.read().unwrap();
        reputation.top_peers(n)
    }

    /// Get number of synced peers
    pub fn peer_count(&self) -> usize {
        self.sync_manager.peer_count()
    }

    /// Clean up expired challenges and verifications
    pub fn cleanup(&mut self) {
        let mut challenges = self.challenges.write().unwrap();
        challenges.cleanup_expired();
    }

    /// Get sync configuration
    pub fn config(&self) -> &SyncConfig {
        &self.config
    }

    // ===== Paxos Consensus Methods =====

    /// Check if Paxos is enabled
    pub fn is_paxos_enabled(&self) -> bool {
        self.paxos.is_some()
    }

    /// Propose a value through Paxos consensus
    /// Returns the proposal number if Paxos is enabled
    pub fn paxos_propose(&mut self, value: Vec<u8>) -> Option<crate::sync::ProposalNumber> {
        if let Some(paxos) = &self.paxos {
            let mut paxos = paxos.write().unwrap();
            Some(paxos.propose(value))
        } else {
            None
        }
    }

    /// Get all learned values from Paxos consensus
    /// Returns empty vector if Paxos is not enabled
    pub fn paxos_learned_values(&self) -> Vec<Vec<u8>> {
        if let Some(paxos) = &self.paxos {
            let paxos = paxos.read().unwrap();
            paxos.learned_values().to_vec()
        } else {
            Vec::new()
        }
    }

    /// Get the number of values learned through Paxos consensus
    pub fn paxos_history_len(&self) -> usize {
        if let Some(paxos) = &self.paxos {
            let paxos = paxos.read().unwrap();
            paxos.learned_values().len()
        } else {
            0
        }
    }
}

/// Helper trait to convert SyncConfig to SyncManagerConfig
impl SyncConfig {
    pub(crate) fn to_manager_config(&self) -> crate::sync::SyncManagerConfig {
        use crate::sync::{SyncManagerConfigBuilder, gossip::GossipConfig as SyncGossipConfig, brb::BrbConfig as SyncBrbConfig};

        let gossip = SyncGossipConfig {
            fanout: self.gossip.fanout,
            interval: self.gossip.interval,
            enable_filtering: true,
            filter_config: Default::default(),
            max_batch_size: 100,
        };

        let brb = SyncBrbConfig {
            total_peers: self.brb.total_peers,
            max_faulty: self.brb.max_faulty,
            require_signatures: false,
        };

        let pow = ProofOfWorkConfig {
            difficulty: self.sybil_resistance.pow_difficulty,
            enabled: self.sybil_resistance.enabled,
        };

        SyncManagerConfigBuilder::new()
            .gossip_interval(self.gossip.interval)
            .gossip_fanout(self.gossip.fanout)
            .brb_config(self.brb.total_peers, self.brb.max_faulty)
            .pow_difficulty(self.sybil_resistance.pow_difficulty)
            .pow_enabled(self.sybil_resistance.enabled)
            .challenge_duration(self.sybil_resistance.challenge_duration)
            .verification_duration(self.sybil_resistance.verification_duration)
            .build()
    }
}

// Typed record for sync operations - moved to protocol module
// The protocol::SyncRecord uses byte arrays for flexibility
// Users can serialize/deserialize their models manually as needed

#[cfg(test)]
mod tests {
    use super::*;
    use netabase_store::netabase_definition_module;

    #[netabase_definition_module(TestDefinition, TestKeys)]
    mod test_def {
        use netabase_store::{NetabaseModel, netabase};

        #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
        #[netabase(TestDefinition)]
        pub struct TestModel {
            #[primary_key]
            pub id: u64,
            pub data: String,
        }
    }

    use test_def::*;

    #[test]
    fn test_netabase_with_sync_creation() {
        let peer_id = PeerId::random();
        let config = SyncConfig::default();

        let sync = NetabaseWithSync::<TestDefinition>::new(peer_id, config);
        assert!(sync.is_ok());
    }

    #[test]
    fn test_peer_management() {
        let peer_id = PeerId::random();
        let config = SyncConfig::default();

        let mut sync = NetabaseWithSync::<TestDefinition>::new(peer_id, config).unwrap();

        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        sync.add_peer(peer1);
        sync.add_peer(peer2);
        assert_eq!(sync.peer_count(), 2);

        sync.remove_peer(&peer1);
        assert_eq!(sync.peer_count(), 1);
    }

    #[test]
    fn test_reputation_tracking() {
        let peer_id = PeerId::random();
        let config = SyncConfig::default();

        let mut sync = NetabaseWithSync::<TestDefinition>::new(peer_id, config).unwrap();

        let peer = PeerId::random();

        // Record successes
        sync.record_success(&peer);
        sync.record_success(&peer);

        let rep = sync.peer_reputation(&peer);
        assert!(rep > 0.5); // Should be higher than default

        // Record failure
        sync.record_failure(&peer);
        let rep_after = sync.peer_reputation(&peer);
        assert!(rep_after < rep); // Should decrease
    }

    // Test moved to protocol module - using protocol::SyncRecord instead
    // which uses byte arrays for flexibility
}
