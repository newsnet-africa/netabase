//! State comparison and exchange logic

use crate::sync::traits::SyncableStore;
use crate::sync::types::{MerkleNode, StateDigest, SyncRecord};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// State exchange manager for comparing and synchronizing states
/// This is now STATELESS - it operates on a SyncableStore reference
pub struct StateExchange;

impl StateExchange {
    /// Create a new state exchange manager
    pub fn new() -> Self {
        Self
    }

    /// Build merkle tree from records in the store
    pub async fn build_merkle_tree<S: SyncableStore>(&self, store: &S) -> Result<[u8; 32]> {
        let records = store.records().await?;

        if records.is_empty() {
            return Ok([0u8; 32]);
        }

        let keys: Vec<_> = records.iter().map(|r| r.key.clone()).collect();
        let records_map: HashMap<_, _> = records
            .into_iter()
            .map(|r| (r.key.clone(), r))
            .collect();

        let root = Self::build_tree(&keys, &records_map);
        Ok(root.hash)
    }

    /// Build merkle tree recursively
    fn build_tree(keys: &[Vec<u8>], records: &HashMap<Vec<u8>, SyncRecord>) -> MerkleNode {
        if keys.len() == 1 {
            // Leaf node
            let key = &keys[0];
            let record = &records[key];
            let hash = blake3::hash(&record.value);
            MerkleNode::leaf(hash.into(), key.clone())
        } else {
            // Internal node
            let mid = keys.len() / 2;
            let left = Self::build_tree(&keys[..mid], records);
            let right = Self::build_tree(&keys[mid..], records);
            MerkleNode::internal(left, right)
        }
    }

    /// Get the merkle root hash from store
    pub async fn get_merkle_root<S: SyncableStore>(&self, store: &S) -> Result<[u8; 32]> {
        self.build_merkle_tree(store).await
    }

    /// Compare local state with remote state digest
    pub async fn compare_states<S: SyncableStore>(
        &self,
        store: &S,
        remote_digest: &StateDigest,
    ) -> Result<StateComparison> {
        let local_hash = self.get_merkle_root(store).await?;

        if local_hash == remote_digest.merkle_root {
            Ok(StateComparison::InSync)
        } else {
            Ok(StateComparison::Diverged {
                local_hash,
                remote_hash: remote_digest.merkle_root,
            })
        }
    }

    /// Identify missing keys by comparing with remote keys
    pub async fn identify_missing_keys<S: SyncableStore>(
        &self,
        store: &S,
        remote_keys: &HashSet<Vec<u8>>,
    ) -> Result<DiffResult> {
        let local_keys: HashSet<_> = store.keys().await?.into_iter().collect();

        let missing_locally: Vec<_> = remote_keys.difference(&local_keys).cloned().collect();
        let missing_remotely: Vec<_> = local_keys.difference(remote_keys).cloned().collect();

        Ok(DiffResult {
            missing_locally,
            missing_remotely,
        })
    }

    /// Get records for specific keys from store
    pub async fn get_records<S: SyncableStore>(
        &self,
        store: &S,
        keys: &[Vec<u8>],
    ) -> Result<Vec<SyncRecord>> {
        let mut records = Vec::new();
        for key in keys {
            if let Some(record) = store.get(key).await? {
                records.push(record);
            }
        }
        Ok(records)
    }

    /// Add or update records in store
    pub async fn add_records<S: SyncableStore>(
        &self,
        store: &S,
        records: Vec<SyncRecord>,
    ) -> Result<()> {
        for record in records {
            store.put(record).await?;
        }
        Ok(())
    }

    /// Get all records from store
    pub async fn all_records<S: SyncableStore>(&self, store: &S) -> Result<Vec<SyncRecord>> {
        store.records().await
    }

    /// Get record count from store
    pub async fn record_count<S: SyncableStore>(&self, store: &S) -> Result<usize> {
        Ok(store.keys().await?.len())
    }

    /// Create state digest from current store state
    pub async fn create_digest<S: SyncableStore>(
        &self,
        store: &S,
        clock: crate::sync::clock::VectorClock,
    ) -> Result<StateDigest> {
        let merkle_root = self.get_merkle_root(store).await?;
        let record_count = self.record_count(store).await?;
        Ok(StateDigest::new(merkle_root, record_count, clock))
    }
}

impl Default for StateExchange {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of state comparison
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateComparison {
    /// States are in sync
    InSync,

    /// States have diverged
    Diverged {
        local_hash: [u8; 32],
        remote_hash: [u8; 32],
    },
}

/// Result of diff operation
#[derive(Clone, Debug)]
pub struct DiffResult {
    /// Keys missing locally (we need to fetch)
    pub missing_locally: Vec<Vec<u8>>,

    /// Keys missing remotely (we should send)
    pub missing_remotely: Vec<Vec<u8>>,
}

impl DiffResult {
    /// Check if there are any differences
    pub fn has_differences(&self) -> bool {
        !self.missing_locally.is_empty() || !self.missing_remotely.is_empty()
    }

    /// Get total number of differences
    pub fn total_differences(&self) -> usize {
        self.missing_locally.len() + self.missing_remotely.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::clock::VectorClock;
    use crate::sync::types::Version;
    use libp2p::PeerId;
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

        async fn insert(&self, record: SyncRecord) {
            self.records.write().await.insert(record.key.clone(), record);
        }
    }

    #[async_trait::async_trait]
    impl SyncableStore for MockStore {
        async fn get(&self, key: &[u8]) -> Result<Option<SyncRecord>> {
            Ok(self.records.read().await.get(key).cloned())
        }

        async fn put(&self, record: SyncRecord) -> Result<()> {
            self.records.write().await.insert(record.key.clone(), record);
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

        async fn merge(&self, _remote: SyncRecord, _local: Option<SyncRecord>) -> Result<SyncRecord> {
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
    async fn test_state_exchange_creation() {
        let exchange = StateExchange::new();
        let store = MockStore::new();

        let count = exchange.record_count(&store).await.unwrap();
        assert_eq!(count, 0);

        let root = exchange.get_merkle_root(&store).await.unwrap();
        assert_eq!(root, [0u8; 32]);
    }

    #[tokio::test]
    async fn test_build_merkle_tree() {
        let exchange = StateExchange::new();
        let store = MockStore::new();

        store.insert(create_test_record(vec![1], vec![10, 20])).await;
        store.insert(create_test_record(vec![2], vec![30, 40])).await;

        let root = exchange.build_merkle_tree(&store).await.unwrap();
        assert_ne!(root, [0u8; 32]);

        let count = exchange.record_count(&store).await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_state_comparison() {
        let exchange = StateExchange::new();
        let store = MockStore::new();

        store.insert(create_test_record(vec![1], vec![10, 20])).await;

        let peer_id = PeerId::random();
        let clock = VectorClock::new(peer_id);

        // Same root - in sync
        let merkle_root = exchange.get_merkle_root(&store).await.unwrap();
        let same_digest = StateDigest::new(merkle_root, 1, clock.clone());

        let comparison = exchange.compare_states(&store, &same_digest).await.unwrap();
        assert_eq!(comparison, StateComparison::InSync);

        // Different root - diverged
        let diff_digest = StateDigest::new([1u8; 32], 1, clock);
        match exchange.compare_states(&store, &diff_digest).await.unwrap() {
            StateComparison::Diverged { .. } => {},
            _ => panic!("Expected Diverged"),
        }
    }

    #[tokio::test]
    async fn test_identify_missing_keys() {
        let exchange = StateExchange::new();
        let store = MockStore::new();

        store.insert(create_test_record(vec![1], vec![10])).await;
        store.insert(create_test_record(vec![2], vec![20])).await;

        let mut remote_keys = HashSet::new();
        remote_keys.insert(vec![2]); // Have key 2
        remote_keys.insert(vec![3]); // Have key 3 (we don't)

        let diff = exchange.identify_missing_keys(&store, &remote_keys).await.unwrap();

        assert_eq!(diff.missing_locally.len(), 1); // Key 3
        assert!(diff.missing_locally.contains(&vec![3]));

        assert_eq!(diff.missing_remotely.len(), 1); // Key 1
        assert!(diff.missing_remotely.contains(&vec![1]));
    }

    #[tokio::test]
    async fn test_get_records() {
        let exchange = StateExchange::new();
        let store = MockStore::new();

        store.insert(create_test_record(vec![1], vec![10])).await;
        store.insert(create_test_record(vec![2], vec![20])).await;

        let keys = vec![vec![1], vec![2]];
        let fetched = exchange.get_records(&store, &keys).await.unwrap();

        assert_eq!(fetched.len(), 2);
    }

    #[tokio::test]
    async fn test_add_records() {
        let exchange = StateExchange::new();
        let store = MockStore::new();

        let record1 = create_test_record(vec![1], vec![10]);
        let record2 = create_test_record(vec![2], vec![20]);

        exchange.add_records(&store, vec![record1, record2]).await.unwrap();

        let count = exchange.record_count(&store).await.unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_diff_result() {
        let diff = DiffResult {
            missing_locally: vec![vec![1], vec![2]],
            missing_remotely: vec![vec![3]],
        };

        assert!(diff.has_differences());
        assert_eq!(diff.total_differences(), 3);
    }
}
