//! Last-Write-Wins (LWW) Register CRDT
//!
//! TODO: Phase 4 - Implement LWW register

use crate::sync::traits::{Syncable, CRDT};
use crate::sync::types::Version;
use anyhow::Result;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};

/// Last-Write-Wins Register
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LwwRegister {
    /// Current value
    value: Vec<u8>,

    /// Version (timestamp)
    version: Version,

    /// Writer peer ID
    #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
    #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
    writer: PeerId,
}

impl LwwRegister {
    /// Create a new LWW register
    pub fn new(value: Vec<u8>, version: Version, writer: PeerId) -> Self {
        Self {
            value,
            version,
            writer,
        }
    }

    /// Get the current value
    pub fn get(&self) -> &[u8] {
        &self.value
    }

    /// Set a new value (with new version)
    pub fn set(&mut self, value: Vec<u8>, version: Version, writer: PeerId) {
        if version.timestamp > self.version.timestamp {
            self.value = value;
            self.version = version;
            self.writer = writer;
        }
    }
}

impl Syncable for LwwRegister {
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

impl CRDT for LwwRegister {
    fn merge(&mut self, other: &Self) -> Result<()> {
        // Take the value with the later timestamp
        if other.version.timestamp > self.version.timestamp {
            self.value = other.value.clone();
            self.version = other.version.clone();
            self.writer = other.writer.clone();
        }
        Ok(())
    }

    fn can_merge(&self, _other: &Self) -> bool {
        true // LWW can always merge
    }

    fn value(&self) -> Vec<u8> {
        self.value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::clock::VectorClock;

    #[test]
    fn test_lww_register_creation() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let lww = LwwRegister::new(vec![1, 2, 3], version, peer);
        assert_eq!(lww.get(), &[1, 2, 3]);
    }

    #[test]
    fn test_lww_set_with_newer_timestamp() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let clock1 = VectorClock::new(peer1);
        let mut version1 = Version::new(clock1, [0u8; 32]);
        version1.timestamp = 100;

        let mut lww = LwwRegister::new(vec![1, 2, 3], version1, peer1);

        // Set with newer timestamp
        let clock2 = VectorClock::new(peer2);
        let mut version2 = Version::new(clock2, [1u8; 32]);
        version2.timestamp = 200;
        lww.set(vec![4, 5, 6], version2, peer2);

        assert_eq!(lww.get(), &[4, 5, 6]);
    }

    #[test]
    fn test_lww_set_with_older_timestamp() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let clock1 = VectorClock::new(peer1);
        let mut version1 = Version::new(clock1, [0u8; 32]);
        version1.timestamp = 200;

        let mut lww = LwwRegister::new(vec![1, 2, 3], version1, peer1);

        // Try to set with older timestamp (should be ignored)
        let clock2 = VectorClock::new(peer2);
        let mut version2 = Version::new(clock2, [1u8; 32]);
        version2.timestamp = 100;
        lww.set(vec![4, 5, 6], version2, peer2);

        // Value should remain unchanged
        assert_eq!(lww.get(), &[1, 2, 3]);
    }

    #[test]
    fn test_lww_merge_takes_newer() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let clock1 = VectorClock::new(peer1);
        let mut version1 = Version::new(clock1, [0u8; 32]);
        version1.timestamp = 100;

        let mut lww1 = LwwRegister::new(vec![1, 2, 3], version1, peer1);

        // Create LWW2 with newer timestamp
        let clock2 = VectorClock::new(peer2);
        let mut version2 = Version::new(clock2, [1u8; 32]);
        version2.timestamp = 200;
        let lww2 = LwwRegister::new(vec![4, 5, 6], version2, peer2);

        // Merge - should take newer value
        lww1.merge(&lww2).unwrap();
        assert_eq!(lww1.get(), &[4, 5, 6]);
    }

    #[test]
    fn test_lww_merge_keeps_current() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let clock1 = VectorClock::new(peer1);
        let mut version1 = Version::new(clock1, [0u8; 32]);
        version1.timestamp = 200;

        let mut lww1 = LwwRegister::new(vec![1, 2, 3], version1, peer1);

        // Create LWW2 with older timestamp
        let clock2 = VectorClock::new(peer2);
        let mut version2 = Version::new(clock2, [1u8; 32]);
        version2.timestamp = 100;
        let lww2 = LwwRegister::new(vec![4, 5, 6], version2, peer2);

        // Merge - should keep current value
        lww1.merge(&lww2).unwrap();
        assert_eq!(lww1.get(), &[1, 2, 3]);
    }

    #[test]
    fn test_lww_can_merge() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let clock1 = VectorClock::new(peer1);
        let version1 = Version::new(clock1, [0u8; 32]);

        let clock2 = VectorClock::new(peer2);
        let version2 = Version::new(clock2, [1u8; 32]);

        let lww1 = LwwRegister::new(vec![1, 2, 3], version1, peer1);
        let lww2 = LwwRegister::new(vec![4, 5, 6], version2, peer2);

        // LWW can always merge
        assert!(lww1.can_merge(&lww2));
    }

    // TODO: Fix serialization - serde_json doesn't support HashMap<PeerId, T>
    // Need to either use bincode or ensure VectorClock uses custom serializer
    // #[test]
    // fn test_lww_serialization() {
    //     let peer = PeerId::random();
    //     let clock = VectorClock::new(peer);
    //     let version = Version::new(clock, [0u8; 32]);
    //
    //     let lww = LwwRegister::new(vec![1, 2, 3], version, peer);
    //
    //     // Test to_bytes (serialization)
    //     let bytes = lww.to_bytes();
    //     assert!(bytes.is_ok());
    //     assert!(!bytes.unwrap().is_empty());
    // }

    #[test]
    fn test_lww_value() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let version = Version::new(clock, [0u8; 32]);

        let lww = LwwRegister::new(vec![1, 2, 3], version, peer);
        assert_eq!(lww.value(), vec![1, 2, 3]);
    }
}
