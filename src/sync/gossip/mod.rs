//! Anti-entropy gossip protocol for state synchronization
//!
//! This module implements a gossip protocol with Byzantine filtering for
//! efficient state synchronization across peers.

pub mod scheduler;
pub mod state_exchange;
pub mod byzantine_filter;

use crate::sync::clock::VectorClock;
use crate::sync::traits::SyncableStore;
use crate::sync::types::{PeerSyncState, StateDigest, SyncMessage, SyncRecord};
use anyhow::Result;
use libp2p::PeerId;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

pub use byzantine_filter::{ByzantineFilter, ByzantineFilterConfig, FilterResult};
pub use scheduler::GossipScheduler;
pub use state_exchange::{DiffResult, StateComparison, StateExchange};

/// Gossip protocol configuration
#[derive(Clone, Debug)]
pub struct GossipConfig {
    /// Number of peers to gossip with per round
    pub fanout: usize,

    /// Gossip interval
    pub interval: Duration,

    /// Enable Byzantine filtering
    pub enable_filtering: bool,

    /// Byzantine filter configuration
    pub filter_config: ByzantineFilterConfig,

    /// Maximum batch size for record transfer
    pub max_batch_size: usize,
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            fanout: 3,
            interval: Duration::from_secs(10),
            enable_filtering: true,
            filter_config: ByzantineFilterConfig::default(),
            max_batch_size: 100,
        }
    }
}

/// Gossip protocol manager
/// Now works with a reference to a SyncableStore instead of owning in-memory state
pub struct GossipManager {
    /// Configuration
    config: GossipConfig,

    /// Scheduler for gossip rounds
    scheduler: GossipScheduler,

    /// State exchange manager (stateless)
    state_exchange: StateExchange,

    /// Byzantine filter
    filter: ByzantineFilter,

    /// Active peers
    active_peers: HashSet<PeerId>,

    /// Peer sync states
    peer_states: HashMap<PeerId, PeerSyncState>,

    /// Local peer ID
    local_peer_id: PeerId,

    /// Local vector clock
    vector_clock: VectorClock,

    /// Statistics
    stats: GossipStats,
}

impl GossipManager {
    /// Create a new gossip manager
    pub fn new(config: GossipConfig, local_peer_id: PeerId) -> Self {
        let scheduler = GossipScheduler::new(config.interval, config.fanout);
        let filter = ByzantineFilter::new(config.filter_config.clone());
        let vector_clock = VectorClock::new(local_peer_id.clone());

        Self {
            config,
            scheduler,
            state_exchange: StateExchange::new(),
            filter,
            active_peers: HashSet::new(),
            peer_states: HashMap::new(),
            local_peer_id,
            vector_clock,
            stats: GossipStats::default(),
        }
    }

    /// Add a peer to the active peer set
    pub fn add_peer(&mut self, peer_id: PeerId) {
        if peer_id != self.local_peer_id {
            self.active_peers.insert(peer_id.clone());
            self.peer_states
                .entry(peer_id.clone())
                .or_insert_with(|| PeerSyncState::new(peer_id));
        }
    }

    /// Remove a peer from the active peer set
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.active_peers.remove(peer_id);
        self.peer_states.remove(peer_id);
    }

    /// Check if it's time to gossip
    pub fn should_gossip(&self) -> bool {
        self.scheduler.should_gossip() && !self.active_peers.is_empty()
    }

    /// Initiate a gossip round
    /// Now takes a store reference to create state digests
    pub async fn initiate_gossip_round<S: SyncableStore>(
        &mut self,
        store: &S,
    ) -> Result<Vec<(PeerId, SyncMessage)>> {
        if !self.should_gossip() {
            return Ok(vec![]);
        }

        let peers: Vec<_> = self.active_peers.iter().cloned().collect();
        let selected_peers = self.scheduler.select_peers(&peers);

        self.stats.rounds += 1;
        self.stats.peers_contacted += selected_peers.len();

        // Create sync requests for selected peers
        let digest = self
            .state_exchange
            .create_digest(store, self.vector_clock.clone())
            .await?;

        let messages = selected_peers
            .into_iter()
            .map(|peer| {
                let message = SyncMessage::SyncRequest {
                    peer_id: self.local_peer_id.clone(),
                    state_digest: digest.clone(),
                };

                (peer, message)
            })
            .collect();

        Ok(messages)
    }

    /// Handle incoming gossip message
    /// Now takes a store reference for reading/writing records
    pub async fn handle_message<S: SyncableStore>(
        &mut self,
        store: &S,
        from: &PeerId,
        message: SyncMessage,
    ) -> Result<Option<SyncMessage>> {
        // Apply Byzantine filter
        if self.config.enable_filtering {
            let filter_result = self.filter.filter_message(from, &message);
            if !filter_result.is_accepted() {
                self.stats.messages_filtered += 1;
                return Ok(None);
            }
        }

        self.stats.messages_received += 1;

        match message {
            SyncMessage::SyncRequest {
                peer_id,
                state_digest,
            } => self.handle_sync_request(store, peer_id, state_digest).await,
            SyncMessage::SyncResponse {
                peer_id,
                missing_keys,
                extra_keys,
            } => {
                self.handle_sync_response(store, peer_id, missing_keys, extra_keys)
                    .await
            }
            SyncMessage::DataTransfer { peer_id, records } => {
                self.handle_data_transfer(store, peer_id, records).await
            }
            SyncMessage::GossipUpdate {
                peer_id,
                version,
                merkle_root,
            } => self.handle_gossip_update(peer_id, version, merkle_root).await,
            _ => Ok(None),
        }
    }

    /// Handle sync request
    async fn handle_sync_request<S: SyncableStore>(
        &mut self,
        store: &S,
        _peer_id: PeerId,
        remote_digest: StateDigest,
    ) -> Result<Option<SyncMessage>> {
        // Compare states
        let comparison = self
            .state_exchange
            .compare_states(store, &remote_digest)
            .await?;

        match comparison {
            StateComparison::InSync => {
                // States are in sync, no action needed
                Ok(None)
            }
            StateComparison::Diverged { .. } => {
                // States diverged, send response with key diff
                let local_keys: HashSet<_> = store.keys().await?.into_iter().collect();

                // For simplicity, send all local keys
                // In production, use merkle tree for efficient diff
                let response = SyncMessage::SyncResponse {
                    peer_id: self.local_peer_id.clone(),
                    missing_keys: vec![], // They should tell us what they need
                    extra_keys: local_keys.into_iter().collect(),
                };

                Ok(Some(response))
            }
        }
    }

    /// Handle sync response
    async fn handle_sync_response<S: SyncableStore>(
        &mut self,
        store: &S,
        _peer_id: PeerId,
        _missing_keys: Vec<Vec<u8>>,
        extra_keys: Vec<Vec<u8>>,
    ) -> Result<Option<SyncMessage>> {
        // Identify what we need
        let local_keys: HashSet<_> = store.keys().await?.into_iter().collect();

        let keys_to_fetch: Vec<_> = extra_keys
            .into_iter()
            .filter(|k| !local_keys.contains(k))
            .collect();

        if keys_to_fetch.is_empty() {
            return Ok(None);
        }

        // Track how many records we need
        self.stats.records_needed += keys_to_fetch.len();

        // For now, we don't have a way to request specific keys
        // This would be handled by the network layer
        Ok(None)
    }

    /// Handle data transfer
    async fn handle_data_transfer<S: SyncableStore>(
        &mut self,
        store: &S,
        peer_id: PeerId,
        records: Vec<SyncRecord>,
    ) -> Result<Option<SyncMessage>> {
        // Filter records through Byzantine filter
        let filtered_records = if self.config.enable_filtering {
            self.filter.filter_records(&peer_id, &records)
        } else {
            records
        };

        self.stats.records_received += filtered_records.len();

        // Add records to store (persistent storage!)
        self.state_exchange
            .add_records(store, filtered_records)
            .await?;

        // Update peer sync state
        if let Some(peer_state) = self.peer_states.get_mut(&peer_id) {
            peer_state.sync_success();
        }

        Ok(None)
    }

    /// Handle gossip update
    async fn handle_gossip_update(
        &mut self,
        peer_id: PeerId,
        version: crate::sync::types::Version,
        merkle_root: [u8; 32],
    ) -> Result<Option<SyncMessage>> {
        // Update peer state
        if let Some(peer_state) = self.peer_states.get_mut(&peer_id) {
            let digest = StateDigest::new(merkle_root, 0, version.clock.clone());
            peer_state.update_digest(digest);
        }

        Ok(None)
    }

    /// Get local state digest
    pub async fn local_digest<S: SyncableStore>(&self, store: &S) -> Result<StateDigest> {
        self.state_exchange
            .create_digest(store, self.vector_clock.clone())
            .await
    }

    /// Get active peer count
    pub fn peer_count(&self) -> usize {
        self.active_peers.len()
    }

    /// Get statistics
    pub fn stats(&self) -> &GossipStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = GossipStats::default();
    }

    /// Get peer reputation
    pub fn peer_reputation(&self, peer_id: &PeerId) -> f64 {
        self.filter.get_reputation(peer_id)
    }

    /// Increment local vector clock
    pub fn tick(&mut self) {
        self.vector_clock.increment();
    }

    /// Get configuration
    pub fn config(&self) -> &GossipConfig {
        &self.config
    }

    /// Get vector clock
    pub fn vector_clock(&self) -> &VectorClock {
        &self.vector_clock
    }
}

/// Gossip statistics
#[derive(Clone, Debug, Default)]
pub struct GossipStats {
    /// Number of gossip rounds initiated
    pub rounds: usize,

    /// Number of peers contacted
    pub peers_contacted: usize,

    /// Number of messages received
    pub messages_received: usize,

    /// Number of messages filtered (rejected)
    pub messages_filtered: usize,

    /// Number of records received
    pub records_received: usize,

    /// Number of records needed (identified as missing)
    pub records_needed: usize,
}

impl GossipStats {
    /// Get average peers per round
    pub fn avg_peers_per_round(&self) -> f64 {
        if self.rounds == 0 {
            return 0.0;
        }
        self.peers_contacted as f64 / self.rounds as f64
    }

    /// Get message acceptance rate
    pub fn message_acceptance_rate(&self) -> f64 {
        let total = self.messages_received + self.messages_filtered;
        if total == 0 {
            return 1.0;
        }
        self.messages_received as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::clock::VectorClock;
    use crate::sync::types::Version;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Mock store for testing
    struct MockStore {
        records: Arc<RwLock<HashMap<Vec<u8>, SyncRecord>>>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                records: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl SyncableStore for MockStore {
        async fn get(&self, key: &[u8]) -> Result<Option<SyncRecord>> {
            Ok(self.records.read().await.get(key).cloned())
        }

        async fn put(&self, record: SyncRecord) -> Result<()> {
            self.records
                .write()
                .await
                .insert(record.key.clone(), record);
            Ok(())
        }

        async fn remove(&self, key: &[u8]) -> Result<()> {
            self.records.write().await.remove(key);
            Ok(())
        }

        async fn keys(&self) -> Result<Vec<Vec<u8>>> {
            Ok(self.records.read().await.keys().cloned().collect())
        }

        async fn records(&self) -> Result<Vec<SyncRecord>> {
            Ok(self.records.read().await.values().cloned().collect())
        }

        async fn range(&self, _start: &[u8], _end: &[u8]) -> Result<Vec<SyncRecord>> {
            Ok(vec![])
        }

        async fn count(&self) -> Result<usize> {
            Ok(self.records.read().await.len())
        }

        async fn merkle_root(&self) -> Result<[u8; 32]> {
            Ok([0u8; 32])
        }

        async fn vector_clock(&self) -> Result<VectorClock> {
            Ok(VectorClock::new(PeerId::random()))
        }

        async fn update_vector_clock(&self, _clock: VectorClock) -> Result<()> {
            Ok(())
        }

        async fn merge(
            &self,
            _remote: SyncRecord,
            _local: Option<SyncRecord>,
        ) -> Result<SyncRecord> {
            unimplemented!()
        }

        async fn clear(&self) -> Result<()> {
            self.records.write().await.clear();
            Ok(())
        }
    }

    fn create_test_record(key: Vec<u8>, value: Vec<u8>) -> SyncRecord {
        let peer_id = PeerId::random();
        let clock = VectorClock::new(peer_id);
        let hash = blake3::hash(&value);
        let version = Version::new(clock, hash.into());
        SyncRecord::new(key, value, version)
    }

    #[tokio::test]
    async fn test_gossip_manager_creation() {
        let config = GossipConfig::default();
        let peer_id = PeerId::random();
        let manager = GossipManager::new(config, peer_id.clone());

        assert_eq!(manager.peer_count(), 0);
        assert!(!manager.should_gossip());
    }

    #[tokio::test]
    async fn test_add_remove_peer() {
        let config = GossipConfig::default();
        let local_peer = PeerId::random();
        let mut manager = GossipManager::new(config, local_peer);

        let peer1 = PeerId::random();
        manager.add_peer(peer1.clone());
        assert_eq!(manager.peer_count(), 1);

        manager.remove_peer(&peer1);
        assert_eq!(manager.peer_count(), 0);
    }

    #[tokio::test]
    async fn test_initiate_gossip_round() {
        let config = GossipConfig::default();
        let local_peer = PeerId::random();
        let mut manager = GossipManager::new(config, local_peer);
        let store = MockStore::new();

        // Add peers
        manager.add_peer(PeerId::random());
        manager.add_peer(PeerId::random());
        manager.add_peer(PeerId::random());

        // Initiate gossip
        let messages = manager.initiate_gossip_round(&store).await.unwrap();
        assert!(!messages.is_empty());
        assert!(messages.len() <= 3); // fanout is 3
    }

    #[tokio::test]
    async fn test_handle_data_transfer() {
        let config = GossipConfig::default();
        let peer_id = PeerId::random();
        let mut manager = GossipManager::new(config, peer_id.clone());
        let store = MockStore::new();

        let sender = PeerId::random();
        let records = vec![
            create_test_record(vec![1], vec![10]),
            create_test_record(vec![2], vec![20]),
        ];

        let message = SyncMessage::DataTransfer {
            peer_id: sender.clone(),
            records,
        };

        manager.handle_message(&store, &sender, message).await.unwrap();

        // Records should be in store now
        assert_eq!(store.count().await.unwrap(), 2);
        assert_eq!(manager.stats().records_received, 2);
    }

    #[test]
    fn test_stats() {
        let config = GossipConfig::default();
        let peer_id = PeerId::random();
        let manager = GossipManager::new(config, peer_id);

        let stats = manager.stats();
        assert_eq!(stats.rounds, 0);
        assert_eq!(stats.message_acceptance_rate(), 1.0);
    }
}
