//! Subscription room types
//!
//! Subscriptions are the netabase equivalent of Willow namespaces.
//! They represent a collection of tables that sync together.

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::data::{
    network::capability::OwnedCapability,
    store::network::NetworkDefinition,
    util::encryption::{NamespacePublicKey, NamespaceSecretKey, NamespaceSignature},
};

/// Unique identifier for a subscription room
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionRoomID(pub [u8; 32]);

impl SubscriptionRoomID {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Derive from namespace public key
    pub fn from_namespace_key(key: &NamespacePublicKey) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"subscription_room_id");
        hasher.update(key.as_bytes());
        Self(hasher.finalize().into())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// A subscription room (namespace equivalent)
#[derive(Clone)]
pub enum SubscriptionRoom<D: NetworkDefinition>
where
    D::Discriminant: std::fmt::Debug + 'static,
{
    /// Owned subscription - we have namespace authority
    Owned {
        /// Room identifier
        id: SubscriptionRoomID,
        /// Namespace public key (proves ownership)
        namespace_key: NamespacePublicKey,
        /// Signature proving we own this namespace
        ownership_signature: NamespaceSignature,
        /// Owner peer ID
        owner: PeerId,
        /// Which tables are included in this subscription
        tables: Vec<D::Discriminant>,
        /// Secret key for signing (private, never transmitted)
        #[doc(hidden)]
        secret_key: Option<NamespaceSecretKey>,
    },

    /// Communal subscription - shared ownership, no single authority
    Communal {
        /// Room identifier
        id: SubscriptionRoomID,
        /// Namespace public key
        namespace_key: NamespacePublicKey,
        /// Which tables are included in this subscription
        tables: Vec<D::Discriminant>,
    },

    /// Delegated subscription - we have capability from owner
    Delegated {
        /// Room identifier
        id: SubscriptionRoomID,
        /// Namespace public key
        namespace_key: NamespacePublicKey,
        /// Our capability proving access (OwnedCapability since delegated from owner)
        capability: OwnedCapability,
        /// Which tables we have access to
        tables: Vec<D::Discriminant>,
    },
}

impl<D: NetworkDefinition> SubscriptionRoom<D>
where
    D::Discriminant: std::fmt::Debug + 'static + Clone,
{
    /// Create a new owned subscription
    pub fn new_owned(
        namespace_key: NamespacePublicKey,
        secret_key: NamespaceSecretKey,
        owner: PeerId,
        tables: Vec<D::Discriminant>,
    ) -> Self {
        let id = SubscriptionRoomID::from_namespace_key(&namespace_key);
        
        // Sign ownership proof
        let mut message = Vec::new();
        message.extend_from_slice(b"subscription_ownership");
        message.extend_from_slice(id.as_bytes());
        message.extend_from_slice(&owner.to_bytes());
        let ownership_signature = secret_key.sign(&message);

        Self::Owned {
            id,
            namespace_key,
            ownership_signature,
            owner,
            tables,
            secret_key: Some(secret_key),
        }
    }

    /// Create a new communal subscription
    pub fn new_communal(namespace_key: NamespacePublicKey, tables: Vec<D::Discriminant>) -> Self {
        let id = SubscriptionRoomID::from_namespace_key(&namespace_key);
        Self::Communal { id, namespace_key, tables }
    }

    /// Get the room ID
    pub fn id(&self) -> &SubscriptionRoomID {
        match self {
            Self::Owned { id, .. } => id,
            Self::Communal { id, .. } => id,
            Self::Delegated { id, .. } => id,
        }
    }

    /// Get the tables in this subscription
    pub fn tables(&self) -> &[D::Discriminant] {
        match self {
            Self::Owned { tables, .. } => tables,
            Self::Communal { tables, .. } => tables,
            Self::Delegated { tables, .. } => tables,
        }
    }

    /// Check if we have ownership (can delegate)
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned { .. })
    }

    /// Check if this is communal (no owner)
    pub fn is_communal(&self) -> bool {
        matches!(self, Self::Communal { .. })
    }

    /// Get the namespace key
    pub fn namespace_key(&self) -> &NamespacePublicKey {
        match self {
            Self::Owned { namespace_key, .. } => namespace_key,
            Self::Delegated { namespace_key, .. } => namespace_key,
            Self::Communal { namespace_key, .. } => namespace_key,
        }
    }

    /// Compute hash for interest commitment
    pub fn interest_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.id().as_bytes());
        // Include tables in hash
        // TODO: Hash table discriminants properly
        hasher.finalize().into()
    }

    /// Get table hashes for this subscription
    pub fn table_hashes(&self) -> Vec<[u8; 32]> {
        // TODO: Implement proper table hashing
        self.tables()
            .iter()
            .map(|_t| {
                let hasher = Sha256::new();
                // hasher.update(t.to_bytes()); // Would need discriminant to implement ToBytes
                hasher.finalize().into()
            })
            .collect()
    }
}
