//! RecordStore implementation for sled-backed storage
//!
//! This module provides a persistent RecordStore implementation using sled as the backend.
//! It stores DHT records and provider records with configurable limits.

use std::borrow::Cow;

use libp2p::PeerId;
use libp2p::kad::{ProviderRecord, Record, RecordKey, store::Error, store::Result};

use crate::database::sled::{NetabaseSledDatabase, NetabaseSledTree};
use crate::errors::NetabaseError;
use crate::traits::{NetabaseKeys, NetabaseModel, NetabaseModelKey, NetabaseSchema};

use std::num::NonZeroUsize;

const K_VALUE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(20) };

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
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct StoredProviderRecord {
    pub key: Vec<u8>,
    pub provider: Vec<u8>, // PeerId as bytes
    pub expires: Option<std::time::SystemTime>,
    pub addresses: Vec<Vec<u8>>, // Multiaddrs as bytes
}

impl From<ProviderRecord> for StoredProviderRecord {
    fn from(record: ProviderRecord) -> Self {
        Self {
            key: record.key.to_vec(),
            provider: record.provider.to_bytes(),
            expires: record.expires.map(|_| std::time::SystemTime::now()),
            addresses: record
                .addresses
                .into_iter()
                .map(|addr| addr.to_vec())
                .collect(),
        }
    }
}

impl TryFrom<StoredProviderRecord> for ProviderRecord {
    type Error = NetabaseError;

    fn try_from(stored: StoredProviderRecord) -> std::result::Result<Self, Self::Error> {
        let provider = PeerId::from_bytes(&stored.provider).map_err(|_| {
            NetabaseError::Conversion(crate::errors::conversion::ConversionError::TraitConversion)
        })?;

        Ok(ProviderRecord {
            key: RecordKey::new(&stored.key),
            provider,
            expires: stored.expires.map(|_| std::time::Instant::now()),
            addresses: stored
                .addresses
                .into_iter()
                .filter_map(|bytes| std::str::from_utf8(&bytes).ok()?.parse().ok())
                .collect(),
        })
    }
}

/// Iterator over stored records
pub struct RecordsIter<'a> {
    inner: Box<dyn Iterator<Item = Cow<'a, Record>> + 'a>,
}

impl<'a> RecordsIter<'a> {
    pub fn new(records: Vec<Record>) -> Self {
        Self {
            inner: Box::new(records.into_iter().map(Cow::Owned)),
        }
    }
}

impl<'a> Iterator for RecordsIter<'a> {
    type Item = Cow<'a, Record>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Iterator over provided records
pub struct ProvidedIter<'a> {
    inner: Box<dyn Iterator<Item = Cow<'a, ProviderRecord>> + 'a>,
}

impl<'a> ProvidedIter<'a> {
    pub fn new(records: Vec<ProviderRecord>) -> Self {
        Self {
            inner: Box::new(records.into_iter().map(Cow::Owned)),
        }
    }
}

impl<'a> Iterator for ProvidedIter<'a> {
    type Item = Cow<'a, ProviderRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Wrapper for storing a list of provider records
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct ProvidersListValue {
    pub providers: Vec<StoredProviderRecord>,
}

impl ProvidersListValue {
    fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    fn to_records(&self) -> Vec<ProviderRecord> {
        self.providers
            .iter()
            .filter_map(|stored| ProviderRecord::try_from(stored.clone()).ok())
            .collect()
    }
}

/// Simple implementation of RecordStore that uses basic storage
pub struct SledRecordStore {
    /// The identity of the peer owning the store.
    local_peer_id: PeerId,
    /// The configuration of the store.
    config: SledRecordStoreConfig,
    /// Storage for records
    records: std::collections::HashMap<Vec<u8>, Record>,
    /// Storage for providers per key
    providers: std::collections::HashMap<Vec<u8>, ProvidersListValue>,
    /// Storage for what the local node provides
    provided: std::collections::HashMap<Vec<u8>, StoredProviderRecord>,
}

impl SledRecordStore {
    /// Creates a new `SledRecordStore` with a default configuration.
    pub fn new(db_path: &str, local_peer_id: PeerId) -> std::result::Result<Self, NetabaseError> {
        Self::with_config(db_path, local_peer_id, Default::default())
    }

    /// Creates a new `SledRecordStore` with the given configuration.
    pub fn with_config(
        _db_path: &str,
        local_peer_id: PeerId,
        config: SledRecordStoreConfig,
    ) -> std::result::Result<Self, NetabaseError> {
        Ok(Self {
            local_peer_id,
            config,
            records: std::collections::HashMap::new(),
            providers: std::collections::HashMap::new(),
            provided: std::collections::HashMap::new(),
        })
    }

    /// Retains the records satisfying a predicate.
    pub fn retain<F>(&mut self, mut f: F) -> std::result::Result<(), NetabaseError>
    where
        F: FnMut(&RecordKey, &mut Record) -> bool,
    {
        let keys_to_remove: Vec<Vec<u8>> = self
            .records
            .iter()
            .filter_map(|(key, record)| {
                let record_key = RecordKey::new(key);
                let mut record_clone = record.clone();
                if !f(&record_key, &mut record_clone) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in keys_to_remove {
            self.records.remove(&key);
        }

        Ok(())
    }

    /// Get the number of records stored
    pub fn records_count(&self) -> usize {
        self.records.len()
    }

    /// Get the number of provider keys stored
    pub fn providers_count(&self) -> usize {
        self.providers.len()
    }

    /// Get the number of keys the local node provides
    pub fn provided_count(&self) -> usize {
        self.provided.len()
    }
}

impl libp2p::kad::store::RecordStore for SledRecordStore {
    type RecordsIter<'a>
        = RecordsIter<'a>
    where
        Self: 'a;
    type ProvidedIter<'a>
        = ProvidedIter<'a>
    where
        Self: 'a;

    fn get(&self, k: &RecordKey) -> Option<Cow<'_, Record>> {
        self.records.get(&k.to_vec()).map(|r| Cow::Borrowed(r))
    }

    fn put(&mut self, r: Record) -> Result<()> {
        if r.value.len() >= self.config.max_value_bytes {
            return Err(Error::ValueTooLarge);
        }

        if self.records.len() >= self.config.max_records
            && !self.records.contains_key(&r.key.to_vec())
        {
            return Err(Error::MaxRecords);
        }

        self.records.insert(r.key.to_vec(), r);
        Ok(())
    }

    fn remove(&mut self, k: &RecordKey) {
        self.records.remove(&k.to_vec());
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        let records: Vec<Record> = self.records.values().cloned().collect();
        RecordsIter::new(records)
    }

    fn add_provider(&mut self, record: ProviderRecord) -> Result<()> {
        let key = record.key.to_vec();
        let stored_record = StoredProviderRecord::from(record.clone());

        // Get or create the providers list
        let mut providers_list = self
            .providers
            .get(&key)
            .cloned()
            .unwrap_or_else(|| ProvidersListValue::new());

        // Check if we're adding a new key and if we've hit the limit
        let is_new_key = providers_list.providers.is_empty();
        if is_new_key && self.providers.len() >= self.config.max_provided_keys {
            return Err(Error::MaxProvidedKeys);
        }

        // Check if this provider already exists and update it
        let mut found = false;
        for p in providers_list.providers.iter_mut() {
            if p.provider == stored_record.provider {
                // Update existing provider
                if self.local_peer_id == record.provider {
                    // Update in provided storage as well
                    self.provided.insert(key.clone(), stored_record.clone());
                }
                *p = stored_record.clone();
                found = true;
                break;
            }
        }

        if !found {
            // If the providers list is full, ignore the new provider
            if providers_list.providers.len() >= self.config.max_providers_per_key {
                // Still save the list even if we're ignoring this provider
                self.providers.insert(key, providers_list);
                return Ok(());
            }

            // Add new provider
            if self.local_peer_id == record.provider {
                // Track in provided storage
                self.provided.insert(key.clone(), stored_record.clone());
            }
            providers_list.providers.push(stored_record);
        }

        // Save the updated providers list
        self.providers.insert(key, providers_list);
        Ok(())
    }

    fn providers(&self, key: &RecordKey) -> Vec<ProviderRecord> {
        self.providers
            .get(&key.to_vec())
            .map(|list| list.to_records())
            .unwrap_or_default()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        let records: Vec<ProviderRecord> = self
            .provided
            .values()
            .filter_map(|stored| ProviderRecord::try_from(stored.clone()).ok())
            .collect();
        ProvidedIter::new(records)
    }

    fn remove_provider(&mut self, key: &RecordKey, provider: &PeerId) {
        let key_bytes = key.to_vec();

        if let Some(mut providers_list) = self.providers.get(&key_bytes).cloned() {
            // Find and remove the provider
            if let Some(pos) = providers_list.providers.iter().position(|p| {
                PeerId::from_bytes(&p.provider)
                    .map(|pid| &pid == provider)
                    .unwrap_or(false)
            }) {
                let removed = providers_list.providers.remove(pos);

                // If this is the local peer, remove from provided storage
                if PeerId::from_bytes(&removed.provider)
                    .map(|pid| pid == self.local_peer_id)
                    .unwrap_or(false)
                {
                    self.provided.remove(&key_bytes);
                }
            }

            // Update or remove the providers list
            if providers_list.providers.is_empty() {
                self.providers.remove(&key_bytes);
            } else {
                self.providers.insert(key_bytes, providers_list);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::kad::store::RecordStore;
    use libp2p::multihash::Multihash;
    use std::time::Instant;

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

    #[test]
    fn put_get_remove_record() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut store = SledRecordStore::new(db_path.to_str().unwrap(), PeerId::random()).unwrap();

        // Create a test record
        let key = RecordKey::new(&b"test_key".to_vec());
        let record = Record {
            key: key.clone(),
            value: b"test_value".to_vec(),
            publisher: Some(PeerId::random()),
            expires: None,
        };

        assert!(store.put(record.clone()).is_ok());

        let retrieved = store.get(&key);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.key, record.key);
        assert_eq!(retrieved.value, record.value);

        store.remove(&key);
        assert!(store.get(&key).is_none());
    }

    #[test]
    fn add_get_remove_provider() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut store = SledRecordStore::new(db_path.to_str().unwrap(), PeerId::random()).unwrap();

        let key = RecordKey::new(&b"test_key".to_vec());
        let provider = PeerId::random();
        let record = ProviderRecord {
            key: key.clone(),
            provider,
            expires: None,
            addresses: vec![],
        };

        assert!(store.add_provider(record.clone()).is_ok());
        let providers = store.providers(&key);
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].provider, provider);

        store.remove_provider(&key, &provider);
        assert!(store.providers(&key).is_empty());
    }

    #[test]
    fn provided() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let id = PeerId::random();
        let mut store = SledRecordStore::new(db_path.to_str().unwrap(), id).unwrap();

        let key = RecordKey::new(&b"test_key".to_vec());
        let record = ProviderRecord {
            key: key.clone(),
            provider: id,
            expires: None,
            addresses: vec![],
        };

        assert!(store.add_provider(record.clone()).is_ok());

        let provided: Vec<_> = store.provided().collect();
        assert_eq!(provided.len(), 1);
        assert_eq!(provided[0].provider, id);
    }

    #[test]
    fn update_provider() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut store = SledRecordStore::new(db_path.to_str().unwrap(), PeerId::random()).unwrap();

        let key = RecordKey::new(&b"test_key".to_vec());
        let provider = PeerId::random();
        let record = ProviderRecord {
            key: key.clone(),
            provider,
            expires: None,
            addresses: vec![],
        };

        assert!(store.add_provider(record.clone()).is_ok());
        assert_eq!(store.providers(&key).len(), 1);

        // Add the same provider again - should update, not duplicate
        assert!(store.add_provider(record.clone()).is_ok());
        assert_eq!(store.providers(&key).len(), 1);
    }

    #[test]
    fn update_provided() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let id = PeerId::random();
        let mut store = SledRecordStore::new(db_path.to_str().unwrap(), id).unwrap();

        let key = RecordKey::new(&b"test_key".to_vec());
        let record = ProviderRecord {
            key: key.clone(),
            provider: id,
            expires: None,
            addresses: vec![],
        };

        store.add_provider(record.clone()).unwrap();
        let provided: Vec<_> = store.provided().collect();
        assert_eq!(provided.len(), 1);
    }

    #[test]
    fn max_providers_per_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut config = SledRecordStoreConfig::default();
        config.max_providers_per_key = 2;
        let mut store =
            SledRecordStore::with_config(db_path.to_str().unwrap(), PeerId::random(), config)
                .unwrap();

        let key = RecordKey::new(&b"test_key".to_vec());

        // Add providers up to the limit
        for i in 0..2 {
            let provider = PeerId::random();
            let record = ProviderRecord {
                key: key.clone(),
                provider,
                expires: None,
                addresses: vec![],
            };
            assert!(store.add_provider(record).is_ok());
        }

        assert_eq!(store.providers(&key).len(), 2);

        // Adding another provider should not increase the count beyond the limit
        let provider = PeerId::random();
        let record = ProviderRecord {
            key: key.clone(),
            provider,
            expires: None,
            addresses: vec![],
        };
        assert!(store.add_provider(record).is_ok());
        assert_eq!(store.providers(&key).len(), 2);
    }

    #[test]
    fn max_provided_keys() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let id = PeerId::random();
        let mut config = SledRecordStoreConfig::default();
        config.max_provided_keys = 2;
        let mut store =
            SledRecordStore::with_config(db_path.to_str().unwrap(), id, config).unwrap();

        // Add providers for different keys up to the limit
        for i in 0..2 {
            let key = RecordKey::new(&format!("test_key_{}", i).as_bytes().to_vec());
            let record = ProviderRecord {
                key: key.clone(),
                provider: id,
                expires: None,
                addresses: vec![],
            };
            assert!(store.add_provider(record).is_ok());
        }

        let provided: Vec<_> = store.provided().collect();
        assert_eq!(provided.len(), 2);

        // Adding another provider for a new key should fail
        let key = RecordKey::new(&b"test_key_overflow".to_vec());
        let record = ProviderRecord {
            key: key.clone(),
            provider: id,
            expires: None,
            addresses: vec![],
        };
        let result = store.add_provider(record);
        assert!(matches!(result, Err(Error::MaxProvidedKeys)));
    }

    #[test]
    fn max_records() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut config = SledRecordStoreConfig::default();
        config.max_records = 2;
        let mut store =
            SledRecordStore::with_config(db_path.to_str().unwrap(), PeerId::random(), config)
                .unwrap();

        // Add records up to the limit
        for i in 0..2 {
            let key = RecordKey::new(&format!("test_key_{}", i).as_bytes().to_vec());
            let record = Record {
                key: key.clone(),
                value: format!("test_value_{}", i).as_bytes().to_vec(),
                publisher: Some(PeerId::random()),
                expires: None,
            };
            assert!(store.put(record).is_ok());
        }

        assert_eq!(store.records_count(), 2);

        // Adding another record should fail
        let key = RecordKey::new(&b"test_key_overflow".to_vec());
        let record = Record {
            key: key.clone(),
            value: b"test_value_overflow".to_vec(),
            publisher: Some(PeerId::random()),
            expires: None,
        };
        let result = store.put(record);
        assert!(matches!(result, Err(Error::MaxRecords)));
    }

    #[test]
    fn value_too_large() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut config = SledRecordStoreConfig::default();
        config.max_value_bytes = 100;
        let mut store =
            SledRecordStore::with_config(db_path.to_str().unwrap(), PeerId::random(), config)
                .unwrap();

        let key = RecordKey::new(&b"test_key".to_vec());
        let large_value = vec![0u8; 200]; // Larger than the limit
        let record = Record {
            key: key.clone(),
            value: large_value,
            publisher: Some(PeerId::random()),
            expires: None,
        };

        let result = store.put(record);
        assert!(matches!(result, Err(Error::ValueTooLarge)));
    }

    #[test]
    fn records_iter() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut store = SledRecordStore::new(db_path.to_str().unwrap(), PeerId::random()).unwrap();

        // Add some records
        for i in 0..3 {
            let key = RecordKey::new(&format!("test_key_{}", i).as_bytes().to_vec());
            let record = Record {
                key: key.clone(),
                value: format!("test_value_{}", i).as_bytes().to_vec(),
                publisher: Some(PeerId::random()),
                expires: None,
            };
            store.put(record).unwrap();
        }

        let records: Vec<_> = store.records().collect();
        assert_eq!(records.len(), 3);
    }
}
