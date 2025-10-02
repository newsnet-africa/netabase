//! RecordStore support types for libp2p integration
//!
//! This module provides the necessary support types for implementing
//! libp2p RecordStore functionality on NetabaseSledDatabase.

use std::borrow::Cow;

#[cfg(feature = "libp2p")]
use libp2p::PeerId;
#[cfg(feature = "libp2p")]
use libp2p::kad::{ProviderRecord, Record, RecordKey};

use crate::errors::NetabaseError;

use std::num::NonZeroUsize;

const K_VALUE: NonZeroUsize = NonZeroUsize::new(20).unwrap();

/// Configuration for a `SledRecordStore`.
#[derive(Debug, Clone)]
pub struct SledRecordStoreConfig {
    /// The maximum number of records.
    pub max_records: usize,
    /// The maximum size of record values, in bytes.
    pub max_value_bytes: usize,
    /// The maximum number of providers stored for a key.
    ///
    /// This should match up with the chosen replication factor.
    pub max_providers_per_key: usize,
    /// The maximum number of provider records for which the
    /// local node is the provider.
    pub max_provided_keys: usize,
}

impl Default for SledRecordStoreConfig {
    fn default() -> Self {
        Self {
            max_records: 1024,
            max_value_bytes: 65 * 1024,
            max_provided_keys: 1024,
            max_providers_per_key: K_VALUE.get(),
        }
    }
}

/// Wrapper for storing a provider record with serialization support
#[cfg(feature = "libp2p")]
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct StoredProviderRecord {
    pub key: Vec<u8>,
    pub provider: Vec<u8>, // PeerId as bytes
    pub expires: Option<std::time::SystemTime>,
    pub addresses: Vec<Vec<u8>>, // Multiaddr as bytes
}

#[cfg(feature = "libp2p")]
impl From<ProviderRecord> for StoredProviderRecord {
    fn from(record: ProviderRecord) -> Self {
        Self {
            key: record.key.to_vec(),
            provider: record.provider.to_bytes(),
            expires: record.expires.map(|_instant| std::time::SystemTime::now()),
            addresses: record
                .addresses
                .into_iter()
                .map(|addr| addr.to_vec())
                .collect(),
        }
    }
}

#[cfg(feature = "libp2p")]
impl TryFrom<StoredProviderRecord> for ProviderRecord {
    type Error = NetabaseError;

    fn try_from(stored: StoredProviderRecord) -> std::result::Result<Self, Self::Error> {
        let provider = PeerId::from_bytes(&stored.provider).map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        let expires = stored.expires.map(|sys_time| {
            sys_time
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|duration| std::time::Instant::now() + duration)
                .unwrap_or_else(|_| std::time::Instant::now())
        });

        Ok(ProviderRecord {
            key: RecordKey::new(&stored.key),
            provider,
            expires,
            addresses: stored
                .addresses
                .into_iter()
                .filter_map(|bytes| std::str::from_utf8(&bytes).ok()?.parse().ok())
                .collect(),
        })
    }
}

/// Iterator over stored records
#[cfg(feature = "libp2p")]
pub struct RecordsIter<'a> {
    inner: Box<dyn Iterator<Item = Cow<'a, Record>> + 'a>,
}

#[cfg(feature = "libp2p")]
impl<'a> RecordsIter<'a> {
    pub fn new(records: Vec<Record>) -> Self {
        Self {
            inner: Box::new(records.into_iter().map(Cow::Owned)),
        }
    }
}

#[cfg(feature = "libp2p")]
impl<'a> Iterator for RecordsIter<'a> {
    type Item = Cow<'a, Record>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Iterator over provided records
#[cfg(feature = "libp2p")]
pub struct ProvidedIter<'a> {
    inner: Box<dyn Iterator<Item = Cow<'a, ProviderRecord>> + 'a>,
}

#[cfg(feature = "libp2p")]
impl<'a> ProvidedIter<'a> {
    pub fn new(records: Vec<ProviderRecord>) -> Self {
        Self {
            inner: Box::new(records.into_iter().map(Cow::Owned)),
        }
    }
}

#[cfg(feature = "libp2p")]
impl<'a> Iterator for ProvidedIter<'a> {
    type Item = Cow<'a, ProviderRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Wrapper for storing a list of provider records
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
#[cfg(feature = "libp2p")]
pub struct ProvidersListValue {
    pub providers: Vec<StoredProviderRecord>,
}

#[cfg(feature = "libp2p")]
impl ProvidersListValue {
    fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn to_records(&self) -> Vec<ProviderRecord> {
        self.providers
            .iter()
            .filter_map(|stored| ProviderRecord::try_from(stored.clone()).ok())
            .collect()
    }
}

#[cfg(all(test, feature = "libp2p"))]
mod tests {
    use super::*;
    use libp2p::kad::store::RecordStore;
    use libp2p::multihash::Multihash;
    use tempfile::tempdir;

    const SHA_256_MH: u64 = 0x12;

    fn random_multihash() -> Multihash<64> {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        Multihash::wrap(SHA_256_MH, &bytes).unwrap()
    }

    fn create_test_key(key_data: &[u8]) -> RecordKey {
        RecordKey::new(&key_data.to_vec())
    }

    // Note: The RecordStore functionality is now implemented directly on NetabaseSledDatabase
    // These tests demonstrate the basic functionality that the consolidated implementation provides

    #[test]
    fn test_record_store_types_exist() {
        // Test that our helper types can be created
        let _config = SledRecordStoreConfig::default();
        assert_eq!(_config.max_records, 1024);
        assert_eq!(_config.max_value_bytes, 65 * 1024);
    }

    #[test]
    fn test_provider_record_conversion() {
        let provider_id = PeerId::random();
        let provider_record = ProviderRecord {
            key: RecordKey::new(&b"test_key".to_vec()),
            provider: provider_id,
            expires: None,
            addresses: vec![],
        };

        // Test conversion to stored format
        let stored = StoredProviderRecord::from(provider_record.clone());
        assert_eq!(stored.key, b"test_key");
        assert_eq!(stored.provider, provider_id.to_bytes());

        // Test conversion back
        let converted: ProviderRecord = stored.try_into().unwrap();
        assert_eq!(converted.key, provider_record.key);
        assert_eq!(converted.provider, provider_record.provider);
    }
}
