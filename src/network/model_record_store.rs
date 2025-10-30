//! ModelRecordStore - Bridge between NetabaseModelTrait and RecordStore
//!
//! This module provides traits and utilities for converting between type-safe
//! NetabaseModelTrait instances and the opaque byte-based libp2p RecordStore format.

use anyhow::{Context, Result};
use libp2p::kad::{Record, RecordKey};
use netabase_store::traits::{definition::NetabaseDefinitionTrait, model::NetabaseModelTrait};

/// Trait for encoding and decoding models to/from RecordStore format
///
/// This trait bridges the gap between type-safe NetabaseModelTrait instances
/// and the opaque byte arrays used by libp2p's RecordStore.
///
/// # Example
///
/// ```ignore
/// // Encode a model to a Record
/// let user = User { id: 1, name: "Alice".to_string() };
/// let record = UserRecordStore::encode_to_record(&user)?;
///
/// // Decode a Record back to a model
/// let decoded_user = UserRecordStore::decode_from_record(&record)?;
/// ```
pub trait ModelRecordStore<D>
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
    /// The model type this store works with
    type Model: NetabaseModelTrait<D> + bincode::Encode + bincode::Decode<()>;

    /// Encode a model instance to bytes for RecordStore
    ///
    /// Uses bincode for efficient binary serialization.
    fn encode_model(model: &Self::Model) -> Result<Vec<u8>>
    where
        <Self::Model as NetabaseModelTrait<D>>::PrimaryKey: bincode::Encode + bincode::Decode<()>,
    {
        bincode::encode_to_vec(model, bincode::config::standard())
            .context("Failed to encode model to bytes")
    }

    /// Decode bytes from RecordStore to model instance
    ///
    /// Uses bincode for deserialization.
    fn decode_model(bytes: &[u8]) -> Result<Self::Model>
    where
        <Self::Model as NetabaseModelTrait<D>>::PrimaryKey: bincode::Encode + bincode::Decode<()>,
    {
        let (model, _) = bincode::decode_from_slice(bytes, bincode::config::standard())
            .context("Failed to decode model from bytes")?;
        Ok(model)
    }

    /// Convert a model instance to a libp2p Record
    ///
    /// This creates a Record with:
    /// - Key: Derived from the model's primary key
    /// - Value: Bincode-encoded model
    /// - Publisher: None (will be set by Kademlia)
    /// - Expires: None (persistent)
    fn encode_to_record(model: &Self::Model) -> Result<Record>
    where
        <Self::Model as NetabaseModelTrait<D>>::PrimaryKey: bincode::Encode + bincode::Decode<()>,
    {
        let key = Self::model_to_record_key(model)?;
        let value = Self::encode_model(model)?;

        Ok(Record {
            key,
            value,
            publisher: None,
            expires: None,
        })
    }

    /// Decode a libp2p Record to a model instance
    ///
    /// Extracts the value bytes from the Record and deserializes them.
    fn decode_from_record(record: &Record) -> Result<Self::Model>
    where
        <Self::Model as NetabaseModelTrait<D>>::PrimaryKey: bincode::Encode + bincode::Decode<()>,
    {
        Self::decode_model(&record.value)
    }

    /// Convert a model's primary key to a RecordKey
    ///
    /// The key is created by:
    /// 1. Encoding the primary key with bincode
    /// 2. Wrapping in a RecordKey
    fn primary_key_to_record_key(
        key: &<Self::Model as NetabaseModelTrait<D>>::PrimaryKey,
    ) -> Result<RecordKey>
    where
        <Self::Model as NetabaseModelTrait<D>>::PrimaryKey: bincode::Encode + bincode::Decode<()>,
    {
        let key_bytes = bincode::encode_to_vec(key, bincode::config::standard())
            .context("Failed to encode primary key")?;
        Ok(RecordKey::new(&key_bytes))
    }

    /// Convert a RecordKey back to a model's primary key
    ///
    /// Decodes the bytes from the RecordKey using bincode.
    fn record_key_to_primary_key(
        key: &RecordKey,
    ) -> Result<<Self::Model as NetabaseModelTrait<D>>::PrimaryKey>
    where
        <Self::Model as NetabaseModelTrait<D>>::PrimaryKey: bincode::Encode + bincode::Decode<()>,
    {
        let (primary_key, _) =
            bincode::decode_from_slice(key.as_ref(), bincode::config::standard())
                .context("Failed to decode primary key from RecordKey")?;
        Ok(primary_key)
    }

    /// Convert a model instance to a RecordKey
    ///
    /// Convenience method that extracts the primary key and converts it.
    fn model_to_record_key(model: &Self::Model) -> Result<RecordKey>
    where
        <Self::Model as NetabaseModelTrait<D>>::PrimaryKey: bincode::Encode + bincode::Decode<()>,
    {
        let primary_key = model.primary_key();
        Self::primary_key_to_record_key(&primary_key)
    }
}

/// Encode/decode utilities for working with NetabaseDefinition and RecordStore
///
/// These functions help bridge between the definition enum and individual models.
pub mod codec {
    use super::*;

    /// Encode a NetabaseDefinition to bytes
    ///
    /// This is the primary encoding format for Paxos log entries.
    pub fn encode_definition<D>(definition: &D) -> Result<Vec<u8>>
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
        bincode::encode_to_vec(definition, bincode::config::standard())
            .context("Failed to encode definition")
    }

    /// Decode bytes to a NetabaseDefinition
    pub fn decode_definition<D>(bytes: &[u8]) -> Result<D>
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
        let (definition, _) = bincode::decode_from_slice(bytes, bincode::config::standard())
            .context("Failed to decode definition")?;
        Ok(definition)
    }

    /// Create a RecordKey from a NetabaseDefinition
    ///
    /// This uses the content hash of the definition as the key,
    /// making it content-addressable.
    pub fn definition_to_record_key<D>(definition: &D) -> Result<RecordKey>
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
        let bytes = encode_definition(definition)?;
        let hash = blake3::hash(&bytes);
        Ok(RecordKey::new(hash.as_bytes()))
    }

    /// Convert a NetabaseDefinition to a Record
    pub fn definition_to_record<D>(definition: &D) -> Result<Record>
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
        let key = definition_to_record_key(definition)?;
        let value = encode_definition(definition)?;

        Ok(Record {
            key,
            value,
            publisher: None,
            expires: None,
        })
    }

    /// Decode a Record to a NetabaseDefinition
    pub fn record_to_definition<D>(record: &Record) -> Result<D>
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
        decode_definition(&record.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper macros would be defined by netabase_store
    // These tests would be integration tests with actual model types

    #[test]
    fn test_codec_encode_decode_roundtrip() {
        // This would be tested with actual definition types in integration tests
        // Example:
        // let definition = BlogDefinition::User(user);
        // let bytes = codec::encode_definition(&definition).unwrap();
        // let decoded = codec::decode_definition::<BlogDefinition>(&bytes).unwrap();
        // assert_eq!(definition, decoded);
    }
}
