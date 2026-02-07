//! Node-level metadata and identity
//!
//! Contains types for node identity, public metadata, and private state.

use libp2p::PeerId;

use crate::data::network::handshake::{
    keys::{StaticKeyPair, StaticPublicKey},
    state::EstablishedConnection,
};

/// Public metadata about a node that can be shared with peers
#[derive(Debug, Clone)]
pub struct NodePublicMetadata {
    /// Node's libp2p peer ID
    pub peer_id: PeerId,
    /// Node's static public key for handshakes
    pub static_key: StaticPublicKey,
    /// Hashes of subscriptions this node participates in (commitment, not full data)
    pub subscription_commitments: Vec<[u8; 32]>,
}

impl NodePublicMetadata {
    pub fn new(peer_id: PeerId, static_key: StaticPublicKey) -> Self {
        Self {
            peer_id,
            static_key,
            subscription_commitments: Vec::new(),
        }
    }

    pub fn with_subscriptions(mut self, commitments: Vec<[u8; 32]>) -> Self {
        self.subscription_commitments = commitments;
        self
    }
}

/// Private metadata that never leaves the node
pub struct NodePrivateMetadata {
    /// Node's static key pair (includes secret key)
    pub static_keys: StaticKeyPair,
    /// Full subscription interest hashes (not just commitments)
    pub subscription_interests: Vec<SubscriptionInterest>,
    /// Capability cache (for faster verification)
    pub capability_cache: CapabilityCache,
}

impl NodePrivateMetadata {
    pub fn new(static_keys: StaticKeyPair) -> Self {
        Self {
            static_keys,
            subscription_interests: Vec::new(),
            capability_cache: CapabilityCache::new(),
        }
    }

    pub fn add_interest(&mut self, interest: SubscriptionInterest) {
        self.subscription_interests.push(interest);
    }

    /// Get interest hashes for handshake
    pub fn interest_hashes(&self) -> Vec<[u8; 32]> {
        self.subscription_interests
            .iter()
            .map(|i| i.subscription_hash)
            .collect()
    }
}

/// Interest in a specific subscription
#[derive(Debug, Clone)]
pub struct SubscriptionInterest {
    /// Hash of the subscription
    pub subscription_hash: [u8; 32],
    /// Tables within the subscription we're interested in
    pub table_hashes: Vec<[u8; 32]>,
    /// Whether we have write access (via capability)
    pub has_write_capability: bool,
}

/// Cache of verified capabilities for faster repeated verification
pub struct CapabilityCache {
    /// Cached capability hashes and their verification results
    entries: Vec<CachedCapability>,
    /// Maximum cache size
    max_size: usize,
}

impl CapabilityCache {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_size: 1000,
        }
    }

    pub fn get(&self, capability_hash: &[u8; 32]) -> Option<&CachedCapability> {
        self.entries.iter().find(|e| &e.capability_hash == capability_hash)
    }

    pub fn insert(&mut self, entry: CachedCapability) {
        if self.entries.len() >= self.max_size {
            self.entries.remove(0); // Simple LRU
        }
        self.entries.push(entry);
    }
}

impl Default for CapabilityCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cached capability verification result
#[derive(Debug, Clone)]
pub struct CachedCapability {
    /// Hash of the capability
    pub capability_hash: [u8; 32],
    /// When the verification was performed
    pub verified_at: u64,
    /// Whether the capability was valid
    pub is_valid: bool,
    /// Expiry time (if any)
    pub expires_at: Option<u64>,
}

/// Connection state for a peer
pub struct PeerConnection {
    /// Established connection (if handshake complete)
    pub connection: Option<EstablishedConnection>,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Number of messages exchanged
    pub message_count: u64,
}

impl PeerConnection {
    pub fn new(connection: EstablishedConnection) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            connection: Some(connection),
            last_activity: now,
            message_count: 0,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub fn record_activity(&mut self) {
        self.last_activity = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.message_count += 1;
    }
}
