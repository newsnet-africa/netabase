//! Traits for synchronizable objects and stores

use crate::sync::types::{MessageSignature, SyncRecord, Version};
use crate::sync::clock::VectorClock;
use anyhow::Result;
use async_trait::async_trait;
use libp2p::PeerId;

/// Trait for objects that can be synchronized across peers
pub trait Syncable: Send + Sync {
    /// Get the key for this object
    fn key(&self) -> Vec<u8>;

    /// Serialize to bytes for transmission
    fn to_bytes(&self) -> Result<Vec<u8>>;

    /// Deserialize from bytes
    fn from_bytes(data: &[u8]) -> Result<Self>
    where
        Self: Sized;

    /// Get the version of this object
    fn version(&self) -> &Version;

    /// Get content hash for verification
    fn content_hash(&self) -> [u8; 32] {
        let data = self.to_bytes().unwrap_or_default();
        blake3::hash(&data).into()
    }

    /// Check if this object's version happened before another
    fn happened_before(&self, other: &Self) -> bool {
        self.version().happened_before(other.version())
    }

    /// Check if this object's version is concurrent with another
    fn is_concurrent(&self, other: &Self) -> bool {
        self.version().is_concurrent(other.version())
    }
}

/// Trait for stores that can synchronize data
#[async_trait]
pub trait SyncableStore: Send + Sync {
    /// Get a record by key
    async fn get(&self, key: &[u8]) -> Result<Option<SyncRecord>>;

    /// Put a record (local operation)
    async fn put(&self, record: SyncRecord) -> Result<()>;

    /// Remove a record
    async fn remove(&self, key: &[u8]) -> Result<()>;

    /// Get all keys in the store
    async fn keys(&self) -> Result<Vec<Vec<u8>>>;

    /// Get all records in the store
    async fn records(&self) -> Result<Vec<SyncRecord>>;

    /// Get records in a specific key range
    async fn range(&self, start: &[u8], end: &[u8]) -> Result<Vec<SyncRecord>>;

    /// Get the number of records
    async fn count(&self) -> Result<usize>;

    /// Compute merkle root of all data
    async fn merkle_root(&self) -> Result<[u8; 32]>;

    /// Get the local vector clock
    async fn vector_clock(&self) -> Result<VectorClock>;

    /// Update the local vector clock
    async fn update_vector_clock(&self, clock: VectorClock) -> Result<()>;

    /// Merge a record using conflict resolution strategy
    async fn merge(&self, remote: SyncRecord, local: Option<SyncRecord>) -> Result<SyncRecord>;

    /// Batch put multiple records
    async fn batch_put(&self, records: Vec<SyncRecord>) -> Result<()> {
        for record in records {
            self.put(record).await?;
        }
        Ok(())
    }

    /// Clear all records (dangerous!)
    async fn clear(&self) -> Result<()>;
}

/// Trait for Byzantine validation
pub trait ByzantineValidator: Send + Sync {
    /// Verify a message signature
    fn verify_signature(&self, message: &[u8], signature: &MessageSignature) -> bool;

    /// Sign a message
    fn sign_message(&self, message: &[u8]) -> Result<MessageSignature>;

    /// Check if a peer is trusted based on reputation
    fn is_trusted(&self, peer_id: &PeerId) -> bool;

    /// Update peer reputation after an interaction
    fn update_reputation(&mut self, peer_id: &PeerId, success: bool);

    /// Check if a quorum has been reached (for BRB)
    /// Returns true if at least 2f+1 out of 3f+1 peers have responded
    fn has_quorum(&self, responding_peers: usize, total_peers: usize, max_faulty: usize) -> bool {
        let required = 2 * max_faulty + 1;
        responding_peers >= required && total_peers >= 3 * max_faulty + 1
    }

    /// Check if enough echoes received (for BRB echo phase)
    fn has_echo_threshold(&self, echoes: usize, total_peers: usize, max_faulty: usize) -> bool {
        let required = (total_peers + max_faulty) / 2 + 1;
        echoes >= required
    }

    /// Check if enough ready messages received (for BRB ready phase)
    fn has_ready_threshold(&self, ready: usize, total_peers: usize, max_faulty: usize) -> bool {
        let required = 2 * max_faulty + 1;
        ready >= required
    }
}

/// Trait for conflict-free replicated data types (CRDTs)
pub trait CRDT: Syncable {
    /// Merge this CRDT with another (commutative and associative)
    fn merge(&mut self, other: &Self) -> Result<()>;

    /// Check if this CRDT can be merged with another
    fn can_merge(&self, other: &Self) -> bool;

    /// Get the current state as a value
    fn value(&self) -> Vec<u8>;
}

/// Trait for gossip participants
#[async_trait]
pub trait GossipParticipant: Send + Sync {
    /// Get a list of peers to gossip with
    async fn select_gossip_peers(&self, count: usize) -> Vec<PeerId>;

    /// Send a gossip message to a peer
    async fn send_gossip(&self, peer: &PeerId, message: Vec<u8>) -> Result<()>;

    /// Handle an incoming gossip message
    async fn handle_gossip(&mut self, from: &PeerId, message: Vec<u8>) -> Result<()>;

    /// Get the current state digest for gossip
    async fn state_digest(&self) -> Result<Vec<u8>>;
}

/// Trait for Sybil resistance mechanisms
pub trait SybilResistance: Send + Sync {
    /// Verify that a peer is legitimate (not a Sybil)
    fn verify_peer(&self, peer_id: &PeerId) -> bool;

    /// Verify proof-of-work for a message
    fn verify_pow(&self, message: &[u8], nonce: u64, difficulty: u32) -> bool {
        let mut hasher = blake3::Hasher::new();
        hasher.update(message);
        hasher.update(&nonce.to_le_bytes());
        let hash = hasher.finalize();

        // Check if hash has required number of leading zeros
        let leading_zeros = hash.as_bytes()
            .iter()
            .take_while(|&&b| b == 0)
            .count() * 8;

        leading_zeros >= difficulty as usize
    }

    /// Generate proof-of-work for a message
    fn generate_pow(&self, message: &[u8], difficulty: u32) -> u64 {
        let mut nonce = 0u64;
        while !self.verify_pow(message, nonce, difficulty) {
            nonce += 1;
        }
        nonce
    }

    /// Check if a peer has sufficient stake
    fn has_sufficient_stake(&self, peer_id: &PeerId, minimum: u64) -> bool;
}

/// Trait for reputation systems
pub trait ReputationSystem: Send + Sync {
    /// Get the reputation score for a peer (0.0 to 1.0)
    fn reputation(&self, peer_id: &PeerId) -> f64;

    /// Update reputation after a successful interaction
    fn reward(&mut self, peer_id: &PeerId, amount: f64);

    /// Update reputation after a failed interaction
    fn penalize(&mut self, peer_id: &PeerId, amount: f64);

    /// Check if a peer's reputation is above threshold
    fn is_reputable(&self, peer_id: &PeerId, threshold: f64) -> bool {
        self.reputation(peer_id) >= threshold
    }

    /// Get the top N most reputable peers
    fn top_peers(&self, n: usize) -> Vec<PeerId>;

    /// Remove a peer from the reputation system
    fn remove_peer(&mut self, peer_id: &PeerId);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSybilResistance;

    impl SybilResistance for MockSybilResistance {
        fn verify_peer(&self, _peer_id: &PeerId) -> bool {
            true
        }

        fn has_sufficient_stake(&self, _peer_id: &PeerId, _minimum: u64) -> bool {
            true
        }
    }

    #[test]
    fn test_pow_verification() {
        let resistance = MockSybilResistance;
        let message = b"test message";
        let difficulty = 4; // 4 leading zero bits

        let nonce = resistance.generate_pow(message, difficulty);
        assert!(resistance.verify_pow(message, nonce, difficulty));
    }

    #[test]
    fn test_quorum_calculation() {
        struct MockValidator;

        impl ByzantineValidator for MockValidator {
            fn verify_signature(&self, _: &[u8], _: &MessageSignature) -> bool {
                true
            }

            fn sign_message(&self, _: &[u8]) -> Result<MessageSignature> {
                unimplemented!()
            }

            fn is_trusted(&self, _: &PeerId) -> bool {
                true
            }

            fn update_reputation(&mut self, _: &PeerId, _: bool) {}
        }

        let validator = MockValidator;

        // With f=1, need 3 out of 4 total peers
        assert!(validator.has_quorum(3, 4, 1));
        assert!(!validator.has_quorum(2, 4, 1));

        // With f=2, need 5 out of 7 total peers
        assert!(validator.has_quorum(5, 7, 2));
        assert!(!validator.has_quorum(4, 7, 2));
    }
}
