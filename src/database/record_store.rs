//! RecordStore implementation for sled-backed storage
//!
//! This module provides a persistent RecordStore implementation using sled as the backend.
//! It stores DHT records and provider records with configurable limits.

use std::borrow::Cow;

use libp2p::PeerId;
use libp2p::kad::{ProviderRecord, Record, RecordKey, store::Error, store::Result};

use crate::database::sled::{NetabaseSledDatabase, NetabaseSledTreeLegacy};
// use crate::database::{ProvidedIter, RecordsIter, StoredProviderRecord};
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

/// Sled-backed implementation of a `RecordStore`.
///
/// This store persists DHT records and provider records to disk using sled.
/// It maintains three trees:
/// - `records`: Stores regular DHT records (Key enum -> Value enum)
/// - `providers`: Stores provider lists for each key (Key enum -> Vec<ProviderRecord>)
/// - `provided`: Stores provider records that the local node has published (Key enum -> ProviderRecord)
///
/// Both K and V are enum types (NetabaseKeys and NetabaseSchema) that represent
/// all possible key and value types in the schema.
pub struct SledRecordStore<K, V>
where
    K: NetabaseKeys,
    V: NetabaseSchema,
    sled::IVec: TryFrom<K>,
    K: TryFrom<sled::IVec>,
    sled::IVec: TryFrom<V>,
    V: TryFrom<sled::IVec>,
    RecordKey: TryFrom<K>,
    K: TryFrom<RecordKey>,
    Record: TryFrom<V>,
    V: TryFrom<Record>,
    <K as TryInto<RecordKey>>::Error: std::error::Error + Send + Sync + 'static,
    <V as TryInto<Record>>::Error: std::error::Error + Send + Sync + 'static,
{
    /// The identity of the peer owning the store.
    local_peer_id: PeerId,
    /// The configuration of the store.
    config: SledRecordStoreConfig,
    /// Tree for storing regular records (enum-based storage)
    records_tree: NetabaseSledTreeLegacy<K, V>,
    /// Tree for storing provider lists (Key enum -> Vec<ProviderRecord>)
    providers_tree: NetabaseSledTreeLegacy<K, ProvidersListValue>,
    /// Tree for storing what the local node provides (Key enum -> ProviderRecord)
    provided_tree: NetabaseSledTreeLegacy<K, StoredProviderRecord>,
}

/// Wrapper for storing a list of provider records
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct ProvidersListValue {
    pub providers: Vec<StoredProviderRecord>,
}

// Dummy key type for ProvidersListValue
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct ProvidersListKey;

impl NetabaseModelKey for ProvidersListKey {
    type SecondaryKeysDiscriminants = String;

    fn secondary_key_discriminants() -> Vec<Self::SecondaryKeysDiscriminants> {
        vec![]
    }
}

impl NetabaseModel for ProvidersListValue {
    type Key = ProvidersListKey;
    type RelationsDiscriminants = String;
    type SchemaDiscriminant = &'static str;

    fn key(&self) -> Self::Key {
        ProvidersListKey
    }

    fn schema_discriminant() -> Self::SchemaDiscriminant {
        "ProvidersListValue"
    }
}

// Dummy key type for StoredProviderRecord
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct StoredProviderKey;

impl NetabaseModelKey for StoredProviderKey {
    type SecondaryKeysDiscriminants = String;

    fn secondary_key_discriminants() -> Vec<Self::SecondaryKeysDiscriminants> {
        vec![]
    }
}

impl NetabaseModel for StoredProviderRecord {
    type Key = StoredProviderKey;
    type RelationsDiscriminants = String;
    type SchemaDiscriminant = &'static str;

    fn key(&self) -> Self::Key {
        StoredProviderKey
    }

    fn schema_discriminant() -> Self::SchemaDiscriminant {
        "StoredProviderRecord"
    }
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

impl TryFrom<sled::IVec> for ProvidersListValue {
    type Error = NetabaseError;

    fn try_from(ivec: sled::IVec) -> std::result::Result<Self, Self::Error> {
        bincode::decode_from_slice(&ivec, bincode::config::standard())
            .map(|(val, _)| val)
            .map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })
    }
}

impl TryFrom<ProvidersListValue> for sled::IVec {
    type Error = NetabaseError;

    fn try_from(value: ProvidersListValue) -> std::result::Result<Self, Self::Error> {
        bincode::encode_to_vec(&value, bincode::config::standard())
            .map(|vec| sled::IVec::from(vec))
            .map_err(|_| {
                NetabaseError::Conversion(
                    crate::errors::conversion::ConversionError::TraitConversion,
                )
            })
    }
}

impl<K, V> SledRecordStore<K, V>
where
    K: NetabaseKeys,
    V: NetabaseSchema,
    sled::IVec: TryFrom<K>,
    K: TryFrom<sled::IVec>,
    sled::IVec: TryFrom<V>,
    V: TryFrom<sled::IVec>,
    RecordKey: TryFrom<K>,
    K: TryFrom<RecordKey>,
    Record: TryFrom<V>,
    V: TryFrom<Record>,
    <K as TryInto<RecordKey>>::Error: std::error::Error + Send + Sync + 'static,
    <V as TryInto<Record>>::Error: std::error::Error + Send + Sync + 'static,
{
    /// Creates a new `SledRecordStore` with a default configuration.
    pub fn new(db_path: &str, local_peer_id: PeerId) -> std::result::Result<Self, NetabaseError> {
        Self::with_config(db_path, local_peer_id, Default::default())
    }

    /// Creates a new `SledRecordStore` with the given configuration.
    pub fn with_config(
        db_path: &str,
        local_peer_id: PeerId,
        config: SledRecordStoreConfig,
    ) -> std::result::Result<Self, NetabaseError> {
        let db = NetabaseSledDatabase::new_with_name(db_path)?;

        let records_tree = db.open_tree_with_discriminant(&db_path.parse().unwrap())?;
        let providers_tree = db.open_tree_with_discriminant(&db_path.parse().unwrap())?;
        let provided_tree = db.open_tree_with_discriminant(&db_path.parse().unwrap())?;

        Ok(Self {
            local_peer_id,
            config,
            records_tree,
            providers_tree,
            provided_tree,
        })
    }

    /// Retains the records satisfying a predicate.
    pub fn retain<F>(&mut self, mut f: F) -> std::result::Result<(), NetabaseError>
    where
        F: FnMut(&RecordKey, &mut Record) -> bool,
    {
        let keys_to_remove: Vec<K> = self
            .records_tree
            .iter()
            .filter_map(|result| {
                if let Ok((key, value)) = result {
                    // Convert enum key to RecordKey and enum value to Record
                    if let Ok(record_key) = key.to_record_key() {
                        if let Ok(mut record) = value.to_record() {
                            if !f(&record_key, &mut record) {
                                return Some(key);
                            }
                        }
                    }
                }
                None
            })
            .collect();

        for key in keys_to_remove {
            self.records_tree.remove(key)?;
        }

        Ok(())
    }

    /// Get the number of records stored
    pub fn records_count(&self) -> usize {
        self.records_tree.len()
    }

    /// Get the number of provider keys stored
    pub fn providers_count(&self) -> usize {
        self.providers_tree.len()
    }

    /// Get the number of keys the local node provides
    pub fn provided_count(&self) -> usize {
        self.provided_tree.len()
    }
}

impl<K, V> libp2p::kad::store::RecordStore for SledRecordStore<K, V>
where
    K: NetabaseKeys,
    V: NetabaseSchema,
    sled::IVec: TryFrom<K>,
    K: TryFrom<sled::IVec>,
    sled::IVec: TryFrom<V>,
    V: TryFrom<sled::IVec>,
    RecordKey: TryFrom<K>,
    K: TryFrom<RecordKey>,
    Record: TryFrom<V>,
    V: TryFrom<Record>,
    <K as TryInto<RecordKey>>::Error: std::error::Error + Send + Sync + 'static,
    <V as TryInto<Record>>::Error: std::error::Error + Send + Sync + 'static,
{
    type RecordsIter<'a>
        = RecordsIter<'a, K, V>
    where
        K: 'a,
        V: 'a;
    type ProvidedIter<'a>
        = ProvidedIter<'a, K>
    where
        K: 'a;

    fn get(&self, k: &RecordKey) -> Option<Cow<'_, Record>> {
        // Convert RecordKey to enum K
        let key = K::from_record_key(k.clone()).ok()?;

        // Get the value from the tree
        let value = self.records_tree.get(key).ok()??;

        // Convert enum V to Record
        let record = value.to_record().ok()?;

        Some(Cow::Owned(record))
    }

    fn put(&mut self, r: Record) -> Result<()> {
        if r.value.len() >= self.config.max_value_bytes {
            return Err(Error::ValueTooLarge);
        }

        let num_records = self.records_tree.len();

        // Convert Record to enum V
        let value = V::from_record(r.clone()).map_err(|_| Error::ValueTooLarge)?;

        // Convert RecordKey to enum K
        let key = K::from_record_key(r.key.clone()).map_err(|_| Error::ValueTooLarge)?;

        // Check if key exists
        let exists = self
            .records_tree
            .contains_key(key.clone())
            .map_err(|_| Error::MaxRecords)?;

        if !exists && num_records >= self.config.max_records {
            return Err(Error::MaxRecords);
        }

        self.records_tree
            .insert(key, value)
            .map_err(|_| Error::MaxRecords)?;

        Ok(())
    }

    fn remove(&mut self, k: &RecordKey) {
        if let Ok(key) = K::from_record_key(k.clone()) {
            let _ = self.records_tree.remove(key);
        }
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        RecordsIter::new(self.records_tree.iter())
    }

    fn add_provider(&mut self, record: ProviderRecord) -> Result<()> {
        let num_keys = self.providers_tree.len();

        // Convert RecordKey to enum K
        let key = K::from_record_key(record.key.clone()).map_err(|_| Error::MaxProvidedKeys)?;

        // Get or create the providers list
        let mut providers_list = self
            .providers_tree
            .get(key.clone())
            .ok()
            .flatten()
            .unwrap_or_else(|| ProvidersListValue::new());

        // Check if we're adding a new key and if we've hit the limit
        let is_new_key = providers_list.providers.is_empty();
        if is_new_key && num_keys >= self.config.max_provided_keys {
            return Err(Error::MaxProvidedKeys);
        }

        // Check if this provider already exists and update it
        let stored_record = StoredProviderRecord::from(record.clone());
        let mut found = false;
        for p in providers_list.providers.iter_mut() {
            if p.provider == stored_record.provider {
                // Update existing provider
                if self.local_peer_id == record.provider {
                    // Update in provided tree as well
                    let _ = self
                        .provided_tree
                        .insert(key.clone(), stored_record.clone());
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
                let _ = self.providers_tree.insert(key.clone(), providers_list);
                return Ok(());
            }

            // Add new provider
            if self.local_peer_id == record.provider {
                // Track in provided tree
                let _ = self
                    .provided_tree
                    .insert(key.clone(), stored_record.clone());
            }
            providers_list.providers.push(stored_record);
        }

        // Save the updated providers list
        self.providers_tree
            .insert(key, providers_list)
            .map_err(|_| Error::MaxProvidedKeys)?;

        Ok(())
    }

    fn providers(&self, key: &RecordKey) -> Vec<ProviderRecord> {
        // Convert RecordKey to enum K
        let k = match K::from_record_key(key.clone()) {
            Ok(k) => k,
            Err(_) => return Vec::new(),
        };

        // Get the providers list
        let providers_list = match self.providers_tree.get(k) {
            Ok(Some(list)) => list,
            _ => return Vec::new(),
        };

        providers_list.to_records()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        ProvidedIter::new(self.provided_tree.iter())
    }

    fn remove_provider(&mut self, key: &RecordKey, provider: &PeerId) {
        // Convert RecordKey to enum K
        let k = match K::from_record_key(key.clone()) {
            Ok(k) => k,
            Err(_) => return,
        };

        // Get the providers list
        let mut providers_list = match self.providers_tree.get(k.clone()) {
            Ok(Some(list)) => list,
            _ => return,
        };

        // Find and remove the provider
        if let Some(pos) = providers_list.providers.iter().position(|p| {
            PeerId::from_bytes(&p.provider)
                .map(|pid| &pid == provider)
                .unwrap_or(false)
        }) {
            let removed = providers_list.providers.remove(pos);

            // If this is the local peer, remove from provided tree
            if PeerId::from_bytes(&removed.provider)
                .map(|pid| pid == self.local_peer_id)
                .unwrap_or(false)
            {
                let _ = self.provided_tree.remove(k.clone());
            }
        }

        // Update or remove the providers list
        if providers_list.providers.is_empty() {
            let _ = self.providers_tree.remove(k);
        } else {
            let _ = self.providers_tree.insert(k, providers_list);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::record_store::tests::test_schema::TestKey;
    use crate::database::record_store::tests::test_schema::TestSchema;
    use crate::database::record_store::tests::test_schema::TestSchemaKeys;
    use libp2p::kad::store::RecordStore;
    use libp2p::multihash::Multihash;
    use netabase_macros::netabase_schema_module;
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
        use crate::database::record_store::tests::test_schema::TestRecord;
        use crate::traits::NetabaseKeys;

        let test_record = TestRecord {
            key: key_data.to_vec(),
            value: b"test_value".to_vec(),
            publisher: b"test_publisher".to_vec(),
        };
        let test_key = test_record.key();
        let test_schema_key = TestSchemaKeys::TestKey(test_key);
        test_schema_key.to_record_key().unwrap()
    }

    #[netabase_schema_module(TestSchema, TestSchemaKeys)]
    pub mod test_schema {
        use crate as netabase;
        use crate::traits::NetabaseModel;
        use crate::traits::NetabaseModelKey;
        use bincode::{Decode, Encode};
        use netabase_macros::{NetabaseModel, key_name, netabase_schema_module};

        #[derive(NetabaseModel, Clone, Encode, Decode, Debug)]
        #[key_name(TestKey)]
        pub struct TestRecord {
            #[key]
            pub key: Vec<u8>,
            pub value: Vec<u8>,
            pub publisher: Vec<u8>,
        }
    }

    #[test]
    fn put_get_remove_record() {
        use crate::database::record_store::tests::test_schema::TestRecord;
        use crate::traits::{NetabaseKeys, NetabaseSchema};

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut store: SledRecordStore<TestSchemaKeys, TestSchema> =
            SledRecordStore::new(db_path.to_str().unwrap(), PeerId::random()).unwrap();

        // Create a proper TestRecord and convert to Record
        let test_record = TestRecord {
            key: b"test_key".to_vec(),
            value: b"test_value".to_vec(),
            publisher: b"test_publisher".to_vec(),
        };
        let test_schema = TestSchema::TestRecord(test_record);
        let record = test_schema.to_record().unwrap();

        assert!(store.put(record.clone()).is_ok());

        let retrieved = store.get(&record.key);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.key, record.key);
        assert_eq!(retrieved.value, record.value);

        store.remove(&record.key);
        assert!(store.get(&record.key).is_none());
    }

    #[test]
    fn add_get_remove_provider() {
        use crate::database::record_store::tests::test_schema::TestRecord;
        use crate::traits::NetabaseKeys;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut store: SledRecordStore<TestSchemaKeys, TestSchema> =
            SledRecordStore::new(db_path.to_str().unwrap(), PeerId::random()).unwrap();

        // Create a proper TestRecord and get its key
        let test_record = TestRecord {
            key: b"test_key".to_vec(),
            value: b"test_value".to_vec(),
            publisher: b"test_publisher".to_vec(),
        };
        let test_key = test_record.key();
        let test_schema_key = TestSchemaKeys::TestKey(test_key);
        let key = test_schema_key.to_record_key().unwrap();

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
        use crate::database::record_store::tests::test_schema::TestRecord;
        use crate::traits::NetabaseKeys;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let id = PeerId::random();
        let mut store: SledRecordStore<TestSchemaKeys, TestSchema> =
            SledRecordStore::new(db_path.to_str().unwrap(), id).unwrap();

        // Create a proper TestRecord and get its key
        let test_record = TestRecord {
            key: b"test_key".to_vec(),
            value: b"test_value".to_vec(),
            publisher: b"test_publisher".to_vec(),
        };
        let test_key = test_record.key();
        let test_schema_key = TestSchemaKeys::TestKey(test_key);
        let key = test_schema_key.to_record_key().unwrap();

        let rec = ProviderRecord {
            key: key.clone(),
            provider: id,
            expires: None,
            addresses: vec![],
        };
        let add_res = store.add_provider(rec.clone());
        assert!(add_res.is_ok());

        let provided: Vec<_> = store.provided().collect();
        assert_eq!(provided.len(), 1);

        store.remove_provider(&key, &id);
        assert_eq!(store.provided().count(), 0);
    }

    #[test]
    fn update_provider() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut store: SledRecordStore<TestSchemaKeys, TestSchema> =
            SledRecordStore::new(db_path.to_str().unwrap(), PeerId::random()).unwrap();

        let key = create_test_key(b"update_provider_key");
        let prv = PeerId::random();
        let mut rec = ProviderRecord {
            key: key.clone(),
            provider: prv,
            expires: None,
            addresses: vec![],
        };

        assert!(store.add_provider(rec.clone()).is_ok());
        let providers = store.providers(&rec.key);
        assert_eq!(providers.len(), 1);

        rec.expires = Some(Instant::now());
        assert!(store.add_provider(rec.clone()).is_ok());
        let providers = store.providers(&rec.key);
        assert_eq!(providers.len(), 1);
    }

    #[test]
    fn update_provided() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let prv = PeerId::random();
        let mut store: SledRecordStore<TestSchemaKeys, TestSchema> =
            SledRecordStore::new(db_path.to_str().unwrap(), prv).unwrap();

        let key = create_test_key(b"update_provided_key");
        let mut rec = ProviderRecord {
            key: key.clone(),
            provider: prv,
            expires: None,
            addresses: vec![],
        };

        assert!(store.add_provider(rec.clone()).is_ok());
        assert_eq!(store.provided().count(), 1);

        rec.expires = Some(Instant::now());
        assert!(store.add_provider(rec.clone()).is_ok());
        assert_eq!(store.provided().count(), 1);
    }

    #[test]
    fn max_providers_per_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let config = SledRecordStoreConfig::default();
        let mut store: SledRecordStore<TestSchemaKeys, TestSchema> = SledRecordStore::with_config(
            db_path.to_str().unwrap(),
            PeerId::random(),
            config.clone(),
        )
        .unwrap();

        let key = create_test_key(b"max_providers_key");

        // Add max providers
        for _ in 0..config.max_providers_per_key {
            let peer = PeerId::random();
            let rec = ProviderRecord {
                key: key.clone(),
                provider: peer,
                expires: None,
                addresses: vec![],
            };
            assert!(store.add_provider(rec).is_ok());
        }

        // Try to add one more - should succeed but not be added
        let peer = PeerId::random();
        let rec = ProviderRecord {
            key: key.clone(),
            provider: peer,
            expires: None,
            addresses: vec![],
        };
        assert!(store.add_provider(rec.clone()).is_ok());
        let providers = store.providers(&key);
        assert_eq!(providers.len(), config.max_providers_per_key);
        assert!(!providers.iter().any(|p| p.provider == peer));
    }

    #[test]
    fn max_provided_keys() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut config = SledRecordStoreConfig::default();
        config.max_provided_keys = 5;

        let mut store: SledRecordStore<TestSchemaKeys, TestSchema> = SledRecordStore::with_config(
            db_path.to_str().unwrap(),
            PeerId::random(),
            config.clone(),
        )
        .unwrap();

        // Fill up to the limit
        for _ in 0..config.max_provided_keys {
            let key = RecordKey::from(random_multihash().to_bytes());
            let prv = PeerId::random();
            let rec = ProviderRecord {
                key,
                provider: prv,
                expires: None,
                addresses: vec![],
            };
            let _ = store.add_provider(rec);
        }

        // Try to add one more key - should fail
        let key = RecordKey::from(random_multihash().to_bytes());
        let prv = PeerId::random();
        let rec = ProviderRecord {
            key,
            provider: prv,
            expires: None,
            addresses: vec![],
        };
        match store.add_provider(rec) {
            Err(Error::MaxProvidedKeys) => {}
            _ => panic!("Expected MaxProvidedKeys error"),
        }
    }

    #[test]
    fn max_records() {
        use crate::database::record_store::tests::test_schema::TestRecord;
        use crate::traits::NetabaseSchema;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut config = SledRecordStoreConfig::default();
        config.max_records = 5;

        let mut store: SledRecordStore<TestSchemaKeys, TestSchema> = SledRecordStore::with_config(
            db_path.to_str().unwrap(),
            PeerId::random(),
            config.clone(),
        )
        .unwrap();

        for i in 0..5 {
            let test_record = TestRecord {
                key: format!("key_{}", i).into_bytes(),
                value: format!("value_{}", i).into_bytes(),
                publisher: b"test_publisher".to_vec(),
            };
            let test_schema = TestSchema::TestRecord(test_record);
            let record = test_schema.to_record().unwrap();
            assert!(store.put(record).is_ok());
        }

        // Try to add one more - should fail
        let extra_test_record = TestRecord {
            key: b"extra_key".to_vec(),
            value: b"extra_value".to_vec(),
            publisher: b"test_publisher".to_vec(),
        };
        let extra_test_schema = TestSchema::TestRecord(extra_test_record);
        let extra_record = extra_test_schema.to_record().unwrap();
        match store.put(extra_record) {
            Err(Error::MaxRecords) => {}
            _ => panic!("Expected MaxRecords error"),
        }
    }

    #[test]
    fn value_too_large() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let config = SledRecordStoreConfig::default();

        let mut store: SledRecordStore<TestSchemaKeys, TestSchema> = SledRecordStore::with_config(
            db_path.to_str().unwrap(),
            PeerId::random(),
            config.clone(),
        )
        .unwrap();

        let record = Record {
            key: RecordKey::from(b"test_key".to_vec()),
            value: vec![0u8; config.max_value_bytes + 1],
            publisher: Some(PeerId::random()),
            expires: None,
        };

        match store.put(record) {
            Err(Error::ValueTooLarge) => {}
            _ => panic!("Expected ValueTooLarge error"),
        }
    }

    #[test]
    fn records_iter() {
        use crate::database::record_store::tests::test_schema::TestRecord;
        use crate::traits::NetabaseSchema;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let mut store: SledRecordStore<TestSchemaKeys, TestSchema> =
            SledRecordStore::new(db_path.to_str().unwrap(), PeerId::random()).unwrap();

        // Add some records
        for i in 0..5 {
            let test_record = TestRecord {
                key: format!("key_{}", i).into_bytes(),
                value: format!("value_{}", i).into_bytes(),
                publisher: b"test_publisher".to_vec(),
            };
            let test_schema = TestSchema::TestRecord(test_record);
            let record = test_schema.to_record().unwrap();
            assert!(store.put(record).is_ok());
        }
        // Iterate and count
        let count = store.records().count();
        assert_eq!(count, 5);
    }
}
