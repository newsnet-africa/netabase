//! Grow-only Counter (G-Counter) CRDT
//!
//! TODO: Phase 4 - Implement G-Counter

use crate::sync::traits::{Syncable, CRDT};
use crate::sync::types::Version;
use anyhow::Result;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Grow-only Counter
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GCounter {
    /// Per-peer counters
    #[serde(with = "crate::sync::serde_helper::peer_id_map")]
    counters: HashMap<PeerId, u64>,

    /// Version
    version: Version,
}

impl GCounter {
    /// Create a new G-Counter
    pub fn new(version: Version) -> Self {
        Self {
            counters: HashMap::new(),
            version,
        }
    }

    /// Increment the counter for a peer
    pub fn increment(&mut self, peer: &PeerId) {
        *self.counters.entry(peer.clone()).or_insert(0) += 1;
    }

    /// Get the total count
    pub fn count(&self) -> u64 {
        self.counters.values().sum()
    }

    /// Get count for a specific peer
    pub fn peer_count(&self, peer: &PeerId) -> u64 {
        *self.counters.get(peer).unwrap_or(&0)
    }
}

impl Syncable for GCounter {
    fn key(&self) -> Vec<u8> {
        // TODO: Implement proper key generation
        vec![]
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn from_bytes(data: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(data)?)
    }

    fn version(&self) -> &Version {
        &self.version
    }
}

impl CRDT for GCounter {
    fn merge(&mut self, other: &Self) -> Result<()> {
        // Merge by taking max of each peer's counter
        for (peer, &count) in &other.counters {
            let local_count = self.counters.entry(peer.clone()).or_insert(0);
            *local_count = (*local_count).max(count);
        }
        Ok(())
    }

    fn can_merge(&self, _other: &Self) -> bool {
        true // G-Counter can always merge
    }

    fn value(&self) -> Vec<u8> {
        self.count().to_le_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::clock::VectorClock;

    #[test]
    fn test_gcounter_creation() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let counter = GCounter::new(version);
        assert_eq!(counter.count(), 0);
    }

    #[test]
    fn test_gcounter_increment() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let mut counter = GCounter::new(version);
        counter.increment(&peer);
        counter.increment(&peer);

        assert_eq!(counter.count(), 2);
        assert_eq!(counter.peer_count(&peer), 2);
    }

    #[test]
    fn test_gcounter_multiple_peers() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        let clock = VectorClock::new(peer1);
        let version = Version::new(clock, [0u8; 32]);

        let mut counter = GCounter::new(version);
        counter.increment(&peer1);
        counter.increment(&peer1);
        counter.increment(&peer2);
        counter.increment(&peer3);
        counter.increment(&peer3);
        counter.increment(&peer3);

        assert_eq!(counter.count(), 6);
        assert_eq!(counter.peer_count(&peer1), 2);
        assert_eq!(counter.peer_count(&peer2), 1);
        assert_eq!(counter.peer_count(&peer3), 3);
    }

    #[test]
    fn test_gcounter_peer_count_nonexistent() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let clock = VectorClock::new(peer1);
        let version = Version::new(clock, [0u8; 32]);

        let mut counter = GCounter::new(version);
        counter.increment(&peer1);

        // Non-existent peer should return 0
        assert_eq!(counter.peer_count(&peer2), 0);
    }

    #[test]
    fn test_gcounter_merge_basic() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let clock = VectorClock::new(peer1);
        let version = Version::new(clock, [0u8; 32]);

        let mut counter1 = GCounter::new(version.clone());
        counter1.increment(&peer1);
        counter1.increment(&peer1);

        let mut counter2 = GCounter::new(version);
        counter2.increment(&peer2);

        counter1.merge(&counter2).unwrap();

        assert_eq!(counter1.count(), 3); // 2 from peer1 + 1 from peer2
        assert_eq!(counter1.peer_count(&peer1), 2);
        assert_eq!(counter1.peer_count(&peer2), 1);
    }

    #[test]
    fn test_gcounter_merge_takes_max() {
        let peer1 = PeerId::random();

        let clock = VectorClock::new(peer1);
        let version = Version::new(clock, [0u8; 32]);

        let mut counter1 = GCounter::new(version.clone());
        counter1.increment(&peer1);
        counter1.increment(&peer1);

        let mut counter2 = GCounter::new(version);
        counter2.increment(&peer1);
        counter2.increment(&peer1);
        counter2.increment(&peer1);
        counter2.increment(&peer1);
        counter2.increment(&peer1);

        // Merge counter2 into counter1
        counter1.merge(&counter2).unwrap();

        // Should take the max (5, not 2+5)
        assert_eq!(counter1.peer_count(&peer1), 5);
        assert_eq!(counter1.count(), 5);
    }

    #[test]
    fn test_gcounter_merge_commutative() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let clock = VectorClock::new(peer1);
        let version = Version::new(clock, [0u8; 32]);

        let mut counter1a = GCounter::new(version.clone());
        counter1a.increment(&peer1);
        counter1a.increment(&peer1);

        let mut counter1b = counter1a.clone();

        let mut counter2 = GCounter::new(version);
        counter2.increment(&peer2);
        counter2.increment(&peer2);
        counter2.increment(&peer2);

        // Merge in both directions
        counter1a.merge(&counter2).unwrap();

        let mut counter2_clone = counter2.clone();
        counter2_clone.merge(&counter1b).unwrap();

        // Should get same result
        assert_eq!(counter1a.count(), counter2_clone.count());
    }

    #[test]
    fn test_gcounter_can_merge() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let counter1 = GCounter::new(version.clone());
        let counter2 = GCounter::new(version);

        // G-Counter can always merge
        assert!(counter1.can_merge(&counter2));
    }

    #[test]
    fn test_gcounter_value() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let mut counter = GCounter::new(version);
        counter.increment(&peer);
        counter.increment(&peer);
        counter.increment(&peer);

        let value_bytes = counter.value();
        let count_from_bytes = u64::from_le_bytes(value_bytes.try_into().unwrap());
        assert_eq!(count_from_bytes, 3);
    }

    // TODO: Fix serialization - serde_json doesn't support HashMap<PeerId, T>
    // #[test]
    // fn test_gcounter_serialization() {
    //     let peer1 = PeerId::random();
    //     let peer2 = PeerId::random();
    //
    //     let clock = VectorClock::new(peer1);
    //     let version = Version::new(clock, [0u8; 32]);
    //
    //     let mut counter = GCounter::new(version);
    //     counter.increment(&peer1);
    //     counter.increment(&peer1);
    //     counter.increment(&peer2);
    //
    //     // Test serialization
    //     let bytes = counter.to_bytes();
    //     assert!(bytes.is_ok());
    //     assert!(!bytes.unwrap().is_empty());
    // }
}
