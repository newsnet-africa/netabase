//! Integration layer between sync module and Netabase types
//!
//! This module bridges the gap between the generic sync functionality and
//! Netabase's typed records, replacing byte arrays with proper types.

use crate::sync::{
    types::{SyncMessage as GenericSyncMessage, Version, SignedMessage, MessageSignature},
    behavior::SyncManagerConfig,
};
use libp2p::PeerId;
use netabase_store::traits::definition::NetabaseDefinitionTrait;
use serde::{Serialize, Deserialize};
use std::marker::PhantomData;

/// Typed sync message that uses Netabase records instead of byte arrays
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetabaseSyncMessage<D>
where
    D: NetabaseDefinitionTrait,
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
    /// Request state synchronization
    SyncRequest {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        version: Version,
    },

    /// Response with state records
    SyncResponse {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        /// Typed records to sync
        records: Vec<NetabaseRecord<D>>,
        version: Version,
    },

    /// Byzantine Reliable Broadcast - Init phase
    BrbInit {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        /// Typed payload
        payload: NetabaseRecord<D>,
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

    /// Challenge for Sybil resistance
    Challenge {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        challenge: Vec<u8>,
    },

    /// Challenge response with proof of work
    ChallengeResponse {
        #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
        #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
        peer_id: PeerId,
        proof: crate::sync::ProofOfWork,
    },
}

/// Wrapper for Netabase records in sync messages
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetabaseRecord<D>
where
    D: NetabaseDefinitionTrait,
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
    /// The discriminant (table/model type)
    pub discriminant: <D as strum::IntoDiscriminant>::Discriminant,

    /// The key for this record
    pub key: Vec<u8>,

    /// The serialized value
    pub value: Vec<u8>,

    /// Version/timestamp
    pub version: Version,

    /// Optional signature
    pub signature: Option<MessageSignature>,

    /// PhantomData to maintain type parameter
    #[serde(skip)]
    _phantom: PhantomData<D>,
}

impl<D> NetabaseRecord<D>
where
    D: NetabaseDefinitionTrait,
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
    /// Create a new record
    pub fn new(
        discriminant: <D as strum::IntoDiscriminant>::Discriminant,
        key: Vec<u8>,
        value: Vec<u8>,
        version: Version,
    ) -> Self {
        Self {
            discriminant,
            key,
            value,
            version,
            signature: None,
            _phantom: PhantomData,
        }
    }

    /// Add a signature to the record
    pub fn with_signature(mut self, signature: MessageSignature) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Get the hash of this record for BRB
    pub fn hash(&self) -> [u8; 32] {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&bincode::encode_to_vec(&self.discriminant, bincode::config::standard()).unwrap_or_default());
        hasher.update(&self.key);
        hasher.update(&self.value);
        hasher.update(&self.version.to_le_bytes());
        hasher.finalize().into()
    }
}

/// Conversion between generic and typed sync messages
impl<D> NetabaseSyncMessage<D>
where
    D: NetabaseDefinitionTrait,
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
    /// Convert to generic sync message (for backward compatibility)
    pub fn to_generic(&self) -> GenericSyncMessage {
        match self {
            Self::SyncRequest { peer_id, version } => GenericSyncMessage::SyncRequest {
                peer_id: *peer_id,
                version: *version,
            },
            Self::SyncResponse { peer_id, records, version } => {
                // Serialize records to bytes
                let payload = bincode::encode_to_vec(records, bincode::config::standard())
                    .unwrap_or_default();
                GenericSyncMessage::SyncResponse {
                    peer_id: *peer_id,
                    payload,
                    version: *version,
                }
            },
            Self::BrbInit { peer_id, payload, version } => {
                let bytes = bincode::encode_to_vec(payload, bincode::config::standard())
                    .unwrap_or_default();
                GenericSyncMessage::BrbInit {
                    peer_id: *peer_id,
                    payload: bytes,
                    version: *version,
                }
            },
            Self::BrbEcho { peer_id, message_hash, original_sender } => {
                GenericSyncMessage::BrbEcho {
                    peer_id: *peer_id,
                    message_hash: *message_hash,
                    original_sender: *original_sender,
                }
            },
            Self::BrbReady { peer_id, message_hash, original_sender } => {
                GenericSyncMessage::BrbReady {
                    peer_id: *peer_id,
                    message_hash: *message_hash,
                    original_sender: *original_sender,
                }
            },
            Self::Challenge { peer_id, challenge } => {
                GenericSyncMessage::SyncRequest {
                    peer_id: *peer_id,
                    version: 0, // Use version field for challenge
                }
            },
            Self::ChallengeResponse { peer_id, proof } => {
                GenericSyncMessage::SyncRequest {
                    peer_id: *peer_id,
                    version: proof.nonce,
                }
            },
        }
    }
}

/// Configuration for typed sync
#[derive(Clone, Debug)]
pub struct NetabaseSyncConfig {
    /// Enable synchronization
    pub enabled: bool,

    /// Base sync configuration
    pub sync_config: SyncManagerConfig,

    /// Auto-sync on record updates
    pub auto_sync: bool,

    /// Sync interval in seconds
    pub sync_interval_secs: u64,
}

impl Default for NetabaseSyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sync_config: SyncManagerConfig::default(),
            auto_sync: true,
            sync_interval_secs: 30,
        }
    }
}

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
    fn test_netabase_record_creation() {
        use strum::IntoEnumIterator;

        let discriminant = TestDefinitionDiscriminant::iter().next().unwrap();
        let record = NetabaseRecord::<TestDefinition>::new(
            discriminant,
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            42,
        );

        assert_eq!(record.key, vec![1, 2, 3, 4]);
        assert_eq!(record.value, vec![5, 6, 7, 8]);
        assert_eq!(record.version, 42);
        assert!(record.signature.is_none());
    }

    #[test]
    fn test_record_hash() {
        use strum::IntoEnumIterator;

        let discriminant = TestDefinitionDiscriminant::iter().next().unwrap();
        let record1 = NetabaseRecord::<TestDefinition>::new(
            discriminant,
            vec![1, 2, 3],
            vec![4, 5, 6],
            1,
        );
        let record2 = NetabaseRecord::<TestDefinition>::new(
            discriminant,
            vec![1, 2, 3],
            vec![4, 5, 6],
            1,
        );

        // Same data should produce same hash
        assert_eq!(record1.hash(), record2.hash());

        // Different data should produce different hash
        let record3 = NetabaseRecord::<TestDefinition>::new(
            discriminant,
            vec![1, 2, 3],
            vec![4, 5, 7], // Different value
            1,
        );
        assert_ne!(record1.hash(), record3.hash());
    }
}
