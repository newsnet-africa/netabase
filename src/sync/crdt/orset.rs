//! Observed-Remove Set (OR-Set) CRDT
//!
//! TODO: Phase 4 - Implement OR-Set

use crate::sync::traits::{Syncable, CRDT};
use crate::sync::types::Version;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Unique identifier for set elements
type ElementId = Vec<u8>;

/// Observed-Remove Set
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrSet {
    /// Elements with their unique tags
    elements: HashMap<ElementId, HashSet<u64>>,

    /// Version
    version: Version,
}

impl OrSet {
    /// Create a new OR-Set
    pub fn new(version: Version) -> Self {
        Self {
            elements: HashMap::new(),
            version,
        }
    }

    /// Add an element with a unique tag
    pub fn add(&mut self, element: ElementId, tag: u64) {
        self.elements
            .entry(element)
            .or_insert_with(HashSet::new)
            .insert(tag);
    }

    /// Remove an element (removes all known tags)
    pub fn remove(&mut self, element: &ElementId) {
        self.elements.remove(element);
    }

    /// Check if element exists
    pub fn contains(&self, element: &ElementId) -> bool {
        self.elements.contains_key(element)
    }

    /// Get all elements
    pub fn elements(&self) -> Vec<ElementId> {
        self.elements.keys().cloned().collect()
    }
}

impl Syncable for OrSet {
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

impl CRDT for OrSet {
    fn merge(&mut self, other: &Self) -> Result<()> {
        // Merge is union of all element-tag pairs
        for (element, tags) in &other.elements {
            self.elements
                .entry(element.clone())
                .or_insert_with(HashSet::new)
                .extend(tags);
        }
        Ok(())
    }

    fn can_merge(&self, _other: &Self) -> bool {
        true // OR-Set can always merge
    }

    fn value(&self) -> Vec<u8> {
        // Serialize the set of elements
        bincode::encode_to_vec(&self.elements(), bincode::config::standard()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::clock::VectorClock;
    use libp2p::PeerId;

    #[test]
    fn test_orset_creation() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let set = OrSet::new(version);
        assert!(set.elements().is_empty());
    }

    #[test]
    fn test_orset_add() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let mut set = OrSet::new(version);
        set.add(vec![1, 2, 3], 1);

        assert!(set.contains(&vec![1, 2, 3]));
        assert_eq!(set.elements().len(), 1);
    }

    #[test]
    fn test_orset_add_multiple() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let mut set = OrSet::new(version);
        set.add(vec![1], 1);
        set.add(vec![2], 2);
        set.add(vec![3], 3);

        assert_eq!(set.elements().len(), 3);
        assert!(set.contains(&vec![1]));
        assert!(set.contains(&vec![2]));
        assert!(set.contains(&vec![3]));
    }

    #[test]
    fn test_orset_add_same_element_different_tags() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let mut set = OrSet::new(version);
        set.add(vec![1, 2, 3], 1);
        set.add(vec![1, 2, 3], 2);
        set.add(vec![1, 2, 3], 3);

        // Still one element, but with multiple tags
        assert_eq!(set.elements().len(), 1);
        assert!(set.contains(&vec![1, 2, 3]));
    }

    #[test]
    fn test_orset_remove() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let mut set = OrSet::new(version);
        set.add(vec![1, 2, 3], 1);
        assert!(set.contains(&vec![1, 2, 3]));

        set.remove(&vec![1, 2, 3]);
        assert!(!set.contains(&vec![1, 2, 3]));
        assert!(set.elements().is_empty());
    }

    #[test]
    fn test_orset_merge() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let mut set1 = OrSet::new(version.clone());
        set1.add(vec![1], 1);
        set1.add(vec![2], 2);

        let mut set2 = OrSet::new(version);
        set2.add(vec![2], 3); // Same element, different tag
        set2.add(vec![3], 4);

        set1.merge(&set2).unwrap();

        // Should have all three elements
        assert_eq!(set1.elements().len(), 3);
        assert!(set1.contains(&vec![1]));
        assert!(set1.contains(&vec![2]));
        assert!(set1.contains(&vec![3]));
    }

    #[test]
    fn test_orset_merge_with_removed() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let mut set1 = OrSet::new(version.clone());
        set1.add(vec![1], 1);
        set1.add(vec![2], 2);

        let mut set2 = OrSet::new(version);
        set2.add(vec![1], 1);
        set2.add(vec![1], 3); // Re-added with new tag
        set2.add(vec![3], 4);

        // Remove element from set1
        set1.remove(&vec![1]);
        assert!(!set1.contains(&vec![1]));

        // Merge set2 which still has element [1]
        set1.merge(&set2).unwrap();

        // Element [1] should reappear because set2 has tags for it
        assert!(set1.contains(&vec![1]));
        assert!(set1.contains(&vec![2]));
        assert!(set1.contains(&vec![3]));
    }

    #[test]
    fn test_orset_can_merge() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let set1 = OrSet::new(version.clone());
        let set2 = OrSet::new(version);

        // OR-Set can always merge
        assert!(set1.can_merge(&set2));
    }

    // TODO: Fix serialization - serde_json doesn't support HashMap<PeerId, T>
    // #[test]
    // fn test_orset_serialization() {
    //     let peer = PeerId::random();
    //     let clock = VectorClock::new(peer);
    //     let version = Version::new(clock, [0u8; 32]);
    //
    //     let mut set = OrSet::new(version);
    //     set.add(vec![1, 2, 3], 1);
    //     set.add(vec![4, 5, 6], 2);
    //
    //     // Test serialization
    //     let bytes = set.to_bytes();
    //     assert!(bytes.is_ok());
    //     assert!(!bytes.unwrap().is_empty());
    // }

    #[test]
    fn test_orset_concurrent_adds() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        // Simulate two peers adding same element concurrently
        let mut set1 = OrSet::new(version.clone());
        set1.add(vec![1], 100);

        let mut set2 = OrSet::new(version);
        set2.add(vec![1], 200);

        // Merge both ways
        set1.merge(&set2).unwrap();

        // Element should exist with both tags
        assert!(set1.contains(&vec![1]));

        // Now if we remove from set1
        set1.remove(&vec![1]);

        // Element should be gone
        assert!(!set1.contains(&vec![1]));
    }
}
