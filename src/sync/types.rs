//! Core types for synchronization

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use super::clock::VectorClock;

/// Version identifier for a record or state
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    /// Vector clock timestamp
    pub clock: VectorClock,

    /// Hash of the content
    pub content_hash: [u8; 32],

    /// Timestamp when this version was created
    pub timestamp: u64,
}

impl Version {
    /// Create a new version
    pub fn new(clock: VectorClock, content_hash: [u8; 32]) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            clock,
            content_hash,
            timestamp,
        }
    }

    /// Check if this version happened before another
    pub fn happened_before(&self, other: &Self) -> bool {
        self.clock.happened_before(&other.clock)
    }

    /// Check if this version is concurrent with another
    pub fn is_concurrent(&self, other: &Self) -> bool {
        self.clock.is_concurrent(&other.clock)
    }
}

/// Digital signature for Byzantine fault tolerance
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageSignature {
    /// Public key of signer
    pub public_key: Vec<u8>,

    /// Signature bytes
    pub signature: Vec<u8>,

    /// Peer ID of signer
    #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
    #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
    pub signer: PeerId,
}

/// Signed message wrapper
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedMessage<T> {
    /// The actual message payload
    pub payload: T,

    /// Signature of the payload
    pub signature: MessageSignature,

    /// Timestamp when message was signed
    pub timestamp: u64,
}

impl<T: Serialize> SignedMessage<T> {
    /// Create a new signed message (signature will be added later by crypto module)
    pub fn new(payload: T, signature: MessageSignature) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            payload,
            signature,
            timestamp,
        }
    }

    /// Verify the signature (to be implemented with crypto)
    pub fn verify(&self) -> bool {
        // TODO: Implement actual signature verification
        // For now, always return true
        true
    }
}

/// Sync protocol messages
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Request sync with a peer - send state digest
    SyncRequest {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        state_digest: StateDigest,
    },

    /// Response to sync request - send missing keys
    SyncResponse {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        missing_keys: Vec<Vec<u8>>,
        extra_keys: Vec<Vec<u8>>,
    },

    /// Transfer actual data
    DataTransfer {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        records: Vec<SyncRecord>,
    },

    /// Gossip state update
    GossipUpdate {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        version: Version,
        merkle_root: [u8; 32],
    },

    /// Byzantine Reliable Broadcast - Initial message
    BrbInit {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        payload: Vec<u8>,
        version: Version,
    },

    /// Byzantine Reliable Broadcast - Echo phase
    BrbEcho {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        message_hash: [u8; 32],
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        original_sender: PeerId,
    },

    /// Byzantine Reliable Broadcast - Ready phase
    BrbReady {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        message_hash: [u8; 32],
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        original_sender: PeerId,
    },

    /// Request full state from peer
    FullStateRequest {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
    },

    /// Full state response
    FullStateResponse {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        state: HashMap<Vec<u8>, SyncRecord>,
    },
}

/// A record being synchronized
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncRecord {
    /// Key (record identifier)
    pub key: Vec<u8>,

    /// Value (serialized data)
    pub value: Vec<u8>,

    /// Version information
    pub version: Version,

    /// Signature for Byzantine verification
    pub signature: Option<MessageSignature>,
}

impl SyncRecord {
    /// Create a new sync record
    pub fn new(key: Vec<u8>, value: Vec<u8>, version: Version) -> Self {
        Self {
            key,
            value,
            version,
            signature: None,
        }
    }

    /// Add signature to the record
    pub fn with_signature(mut self, signature: MessageSignature) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Verify the record's signature
    pub fn verify(&self) -> bool {
        if let Some(_sig) = &self.signature {
            // TODO: Implement actual signature verification
            true
        } else {
            false
        }
    }
}

/// Compact representation of state for comparison
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateDigest {
    /// Merkle tree root hash
    pub merkle_root: [u8; 32],

    /// Number of records
    pub record_count: usize,

    /// Vector clock summary
    pub clock: VectorClock,

    /// Timestamp of digest creation
    pub timestamp: u64,
}

impl StateDigest {
    /// Create a new state digest
    pub fn new(merkle_root: [u8; 32], record_count: usize, clock: VectorClock) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            merkle_root,
            record_count,
            clock,
            timestamp,
        }
    }

    /// Check if states are likely different
    pub fn differs_from(&self, other: &Self) -> bool {
        self.merkle_root != other.merkle_root
    }
}

/// Merkle tree node for efficient state comparison
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleNode {
    /// Hash of this node
    pub hash: [u8; 32],

    /// Left child hash (if not leaf)
    pub left: Option<Box<MerkleNode>>,

    /// Right child hash (if not leaf)
    pub right: Option<Box<MerkleNode>>,

    /// Range of keys this node covers
    pub key_range: Option<(Vec<u8>, Vec<u8>)>,
}

impl MerkleNode {
    /// Create a leaf node
    pub fn leaf(hash: [u8; 32], key: Vec<u8>) -> Self {
        Self {
            hash,
            left: None,
            right: None,
            key_range: Some((key.clone(), key)),
        }
    }

    /// Create an internal node from two children
    pub fn internal(left: MerkleNode, right: MerkleNode) -> Self {
        // Combine hashes
        let mut hasher = blake3::Hasher::new();
        hasher.update(&left.hash);
        hasher.update(&right.hash);
        let hash: [u8; 32] = hasher.finalize().into();

        Self {
            hash,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            key_range: None,
        }
    }

    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

/// Per-peer sync state tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerSyncState {
    /// Peer ID
    #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
    #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
    pub peer_id: PeerId,

    /// Last known state digest from this peer
    pub last_digest: Option<StateDigest>,

    /// Last successful sync time
    pub last_sync: Option<u64>,

    /// Number of successful syncs
    pub sync_count: usize,

    /// Number of failed syncs
    pub failure_count: usize,

    /// Reputation score (for Byzantine filtering)
    pub reputation: f64,

    /// Is this peer currently being synced?
    pub syncing: bool,
}

impl PeerSyncState {
    /// Create a new peer sync state
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id,
            last_digest: None,
            last_sync: None,
            sync_count: 0,
            failure_count: 0,
            reputation: 1.0, // Start with neutral reputation
            syncing: false,
        }
    }

    /// Mark sync as started
    pub fn start_sync(&mut self) {
        self.syncing = true;
    }

    /// Mark sync as successful
    pub fn sync_success(&mut self) {
        self.syncing = false;
        self.sync_count += 1;
        self.last_sync = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        // Improve reputation slightly on success
        self.reputation = (self.reputation + 0.1).min(1.0);
    }

    /// Mark sync as failed
    pub fn sync_failure(&mut self) {
        self.syncing = false;
        self.failure_count += 1;

        // Decrease reputation on failure
        self.reputation = (self.reputation - 0.2).max(0.0);
    }

    /// Update state digest
    pub fn update_digest(&mut self, digest: StateDigest) {
        self.last_digest = Some(digest);
    }

    /// Check if peer is trusted (reputation above threshold)
    pub fn is_trusted(&self, threshold: f64) -> bool {
        self.reputation >= threshold
    }
}

/// Overall sync state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncState {
    /// Local state digest
    pub local_digest: StateDigest,

    /// Per-peer states
    #[serde(with = "crate::sync::serde_helper::peer_id_map")]
    pub peer_states: HashMap<PeerId, PeerSyncState>,

    /// Currently syncing with these peers
    #[serde(with = "crate::sync::serde_helper::peer_id_set")]
    pub active_syncs: HashSet<PeerId>,

    /// Total number of records synchronized
    pub total_synced: usize,

    /// Total number of sync operations
    pub sync_operations: usize,
}

impl SyncState {
    /// Create a new sync state
    pub fn new(local_digest: StateDigest) -> Self {
        Self {
            local_digest,
            peer_states: HashMap::new(),
            active_syncs: HashSet::new(),
            total_synced: 0,
            sync_operations: 0,
        }
    }
}

/// Current synchronization status
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Not actively syncing
    Idle,

    /// Actively syncing
    Active,

    /// Syncing with specific peer
    SyncingWith(
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        PeerId
    ),

    /// Error occurred
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_creation() {
        let peer_id = PeerId::random();
        let clock = VectorClock::new(peer_id);
        let hash = [0u8; 32];
        let version = Version::new(clock, hash);

        assert_eq!(version.content_hash, hash);
        assert!(version.timestamp > 0);
    }

    #[test]
    fn test_peer_sync_state() {
        let peer_id = PeerId::random();
        let mut state = PeerSyncState::new(peer_id);

        assert_eq!(state.reputation, 1.0);
        assert_eq!(state.sync_count, 0);

        state.sync_success();
        assert_eq!(state.sync_count, 1);
        assert_eq!(state.reputation, 1.0); // Capped at max

        state.sync_failure();
        assert_eq!(state.failure_count, 1);
        assert!((state.reputation - 0.8).abs() < 0.001); // 1.0 - 0.2 = 0.8
    }

    #[test]
    fn test_merkle_leaf() {
        let hash = [1u8; 32];
        let key = vec![1, 2, 3];
        let node = MerkleNode::leaf(hash, key.clone());

        assert!(node.is_leaf());
        assert_eq!(node.hash, hash);
        assert_eq!(node.key_range, Some((key.clone(), key)));
    }
}
