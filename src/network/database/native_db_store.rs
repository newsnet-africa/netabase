use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use libp2p::{
    PeerId,
    kad::{
        ProviderRecord, Record as KadRecord, RecordKey,
        store::{RecordStore, Result as StoreResult},
    },
};
use native_db::Database;

use crate::{NetabaseCatalog, NetabaseRecordExt, Record};

/// NativeDB implementation of the libp2p_kad RecordStore trait
pub struct NativeDBStore<C>
where
    C: NetabaseCatalog,
{
    local_key: PeerId,
    database: Arc<RwLock<Database<'static>>>,
    // In-memory cache for records (key -> catalog object)
    record_cache: Arc<RwLock<HashMap<String, C>>>,
    // Provider records cache
    provider_cache: Arc<RwLock<HashMap<RecordKey, Vec<ProviderRecord>>>>,
    _phantom: std::marker::PhantomData<C>,
}

impl<C> NativeDBStore<C>
where
    C: NetabaseCatalog + NetabaseRecordExt + Clone + 'static + bincode::Encode,
{
    /// Create a new NativeDBStore
    pub fn new(local_key: PeerId, database: Database<'static>) -> Self {
        Self {
            local_key,
            database: Arc::new(RwLock::new(database)),
            record_cache: Arc::new(RwLock::new(HashMap::new())),
            provider_cache: Arc::new(RwLock::new(HashMap::new())),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Store a native object and return its KadRecord representation
    pub fn store_native_object(&mut self, object: C) -> StoreResult<KadRecord> {
        // Get the catalog key for the object and convert to bytes
        let catalog_key = object.catalog_key();
        let key_bytes = C::key_to_bytes(&catalog_key);

        // Convert key to string for cache
        let key_string = String::from_utf8_lossy(&key_bytes).to_string();

        // Create record with bytes key
        let record = Record::new(key_bytes, object.clone());
        let kad_record = record.clone().into();

        {
            let mut cache = self.record_cache.write().unwrap();
            cache.insert(key_string, object.clone());
        }

        Ok(kad_record)
    }

    /// Retrieve a native object by key
    pub fn get_native_object(&self, key: &str) -> Option<C> {
        // Check cache first
        if let Some(catalog_obj) = self.record_cache.read().unwrap().get(key) {
            return Some(catalog_obj.clone());
        }

        None
    }

    /// Remove native object by key
    pub fn remove_native_object(&mut self, key: &str) {
        // Remove from cache
        self.record_cache.write().unwrap().remove(key);
    }

    /// Get the local peer ID
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_key
    }

    /// Clean up expired records from cache
    pub fn cleanup_expired(&mut self) {
        // Note: With catalog objects stored directly, expiry tracking would need
        // to be implemented at the catalog level or through separate metadata.
        // For now, this is a no-op as catalog objects don't have expiry info.
        // In a production implementation, you might store additional metadata
        // about when objects were cached and implement cleanup based on that.
    }
}

impl<C> RecordStore for NativeDBStore<C>
where
    C: NetabaseCatalog + NetabaseRecordExt + Clone + bincode::Encode,
{
    type RecordsIter<'iter>
        = std::vec::IntoIter<Cow<'iter, KadRecord>>
    where
        Self: 'iter;

    type ProvidedIter<'iter>
        = std::vec::IntoIter<Cow<'iter, ProviderRecord>>
    where
        Self: 'iter;

    fn get(&self, key: &RecordKey) -> Option<Cow<'_, KadRecord>> {
        let key_str = String::from_utf8_lossy(key.as_ref());

        if let Some(catalog_obj) = self
            .record_cache
            .read()
            .unwrap()
            .get(key_str.as_ref())
            .cloned()
        {
            // Convert catalog object to KadRecord using NetabaseRecordExt
            let kad_record = catalog_obj.to_kad_record();
            Some(Cow::Owned(kad_record))
        } else {
            None
        }
    }

    fn put(&mut self, kad_record: KadRecord) -> StoreResult<()> {
        let key_str = String::from_utf8_lossy(kad_record.key.as_ref()).to_string();

        // Convert KadRecord back to catalog object
        if let Ok(catalog_obj) = C::from_kad_record(kad_record) {
            self.record_cache
                .write()
                .unwrap()
                .insert(key_str, catalog_obj);
        }

        Ok(())
    }

    fn remove(&mut self, key: &RecordKey) {
        let key_str = String::from_utf8_lossy(key.as_ref());
        self.record_cache.write().unwrap().remove(key_str.as_ref());
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        let cache = self.record_cache.read().unwrap();
        let records: Vec<Cow<'_, KadRecord>> = cache
            .values()
            .map(|catalog_obj| {
                let kad_record = catalog_obj.to_kad_record();
                Cow::Owned(kad_record)
            })
            .collect();

        records.into_iter()
    }

    fn add_provider(&mut self, provider_record: ProviderRecord) -> StoreResult<()> {
        let mut providers = self.provider_cache.write().unwrap();
        let key = provider_record.key.clone();

        providers
            .entry(key)
            .or_insert_with(Vec::new)
            .push(provider_record);

        Ok(())
    }

    fn providers(&self, key: &RecordKey) -> Vec<ProviderRecord> {
        self.provider_cache
            .read()
            .unwrap()
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        let providers = self.provider_cache.read().unwrap();
        let provided: Vec<Cow<'_, ProviderRecord>> = providers
            .values()
            .flat_map(|provider_list| {
                provider_list
                    .iter()
                    .filter(|p| p.provider == self.local_key)
            })
            .map(|p| Cow::Owned(p.clone()))
            .collect();

        provided.into_iter()
    }

    fn remove_provider(&mut self, key: &RecordKey, provider: &PeerId) {
        let mut providers = self.provider_cache.write().unwrap();
        if let Some(provider_list) = providers.get_mut(key) {
            provider_list.retain(|p| p.provider != *provider);

            if provider_list.is_empty() {
                providers.remove(key);
            }
        }
    }
}

/// Builder for NativeDBStore
pub struct NativeDBStoreBuilder<C>
where
    C: NetabaseCatalog,
{
    local_key: Option<PeerId>,
    database: Option<Database<'static>>,
    _phantom: std::marker::PhantomData<C>,
}

impl<C> Default for NativeDBStoreBuilder<C>
where
    C: NetabaseCatalog,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C> NativeDBStoreBuilder<C>
where
    C: NetabaseCatalog,
{
    pub fn new() -> Self {
        Self {
            local_key: None,
            database: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_local_key(mut self, local_key: PeerId) -> Self {
        self.local_key = Some(local_key);
        self
    }

    pub fn with_database(mut self, database: Database<'static>) -> Self {
        self.database = Some(database);
        self
    }

    pub fn build(self) -> Result<NativeDBStore<C>, &'static str>
    where
        C: NetabaseRecordExt + Clone + 'static,
    {
        let local_key = self.local_key.ok_or("Local key is required")?;
        let database = self.database.ok_or("Database is required")?;

        Ok(NativeDBStore::new(local_key, database))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CatalogConstructor, CatalogKey};
    use bincode::{Decode, Encode};
    use libp2p::PeerId;

    // Simple test model that doesn't depend on native_db macros
    #[derive(Debug, Clone, PartialEq, Encode, Decode)]
    struct TestModel {
        pub id: String,
        pub data: String,
    }

    #[derive(Debug, Clone, Encode, Decode)]
    enum TestCatalog {
        TestModel(TestModel),
    }

    impl NetabaseCatalog for TestCatalog {
        type RefCatalog<'a> = TestCatalogRef<'a>;
    }

    #[derive(Debug, Clone, Copy)]
    enum TestCatalogRef<'a> {
        TestModel(&'a TestModel),
    }

    impl<'a> crate::NetabaseRefCatalog<'a> for TestCatalogRef<'a> {}

    #[derive(Debug, Clone, Encode, Decode)]
    enum TestCatalogKey {
        TestModelKey(crate::SerializableKey),
    }

    impl crate::CatalogKey for TestCatalog {
        type KeyType = TestCatalogKey;

        fn catalog_key(&self) -> Self::KeyType {
            match self {
                TestCatalog::TestModel(model) => {
                    use native_db::ToKey;
                    let native_key = model.id.to_key();
                    let serializable_key = crate::SerializableKey::from_native_db_key_with_hint(
                        &native_key,
                        "TestCatalog::TestModel".to_string(),
                    );
                    TestCatalogKey::TestModelKey(serializable_key)
                }
            }
        }

        fn key_to_serializable(key: &Self::KeyType) -> crate::SerializableKey {
            match key {
                TestCatalogKey::TestModelKey(serializable_key) => serializable_key.clone(),
            }
        }

        fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
            Self::key_to_serializable(key).as_bytes().to_vec()
        }

        fn bytes_to_key(bytes: &[u8]) -> Result<Self::KeyType, Box<dyn std::error::Error>> {
            let serializable_key = crate::SerializableKey {
                key_bytes: bytes.to_vec(),
                type_hint: Some("TestCatalog::TestModel".to_string()),
            };
            Ok(TestCatalogKey::TestModelKey(serializable_key))
        }
    }

    impl CatalogConstructor<TestModel> for TestCatalog {
        fn from_native_db(data: TestModel) -> Self {
            TestCatalog::TestModel(data)
        }

        fn to_native_db(self) -> TestModel {
            match self {
                TestCatalog::TestModel(model) => model,
            }
        }
    }

    impl NetabaseRecordExt for TestCatalog {}

    fn create_test_database() -> Database<'static> {
        // Create a simple in-memory database for testing
        // Note: In a real implementation, this would include proper native_db setup
        use native_db::{Builder, Models};
        use std::sync::OnceLock;

        static MODELS: OnceLock<Models> = OnceLock::new();
        let models = MODELS.get_or_init(|| Models::new());
        Builder::new().create_in_memory(models).unwrap()
    }

    #[test]
    fn test_store_creation() {
        let peer_id = PeerId::random();
        let database = create_test_database();

        let store = NativeDBStore::<TestCatalog>::new(peer_id, database);
        assert_eq!(store.local_peer_id(), &peer_id);
    }

    #[test]
    fn test_builder_pattern() {
        let peer_id = PeerId::random();
        let database = create_test_database();

        let store = NativeDBStoreBuilder::<TestCatalog>::new()
            .with_local_key(peer_id)
            .with_database(database)
            .build()
            .unwrap();

        assert_eq!(store.local_peer_id(), &peer_id);
    }

    #[test]
    fn test_record_store_operations() {
        let peer_id = PeerId::random();
        let database = create_test_database();
        let mut store = NativeDBStore::<TestCatalog>::new(peer_id, database);

        // Create test catalog object
        let test_model = TestModel {
            id: "test_id".to_string(),
            data: "test_data".to_string(),
        };
        let catalog_obj = TestCatalog::TestModel(test_model.clone());
        let kad_record = catalog_obj.to_kad_record();

        // Test put
        store.put(kad_record.clone()).unwrap();

        // Test get
        let retrieved = store.get(&kad_record.key);
        assert!(retrieved.is_some());

        let retrieved_record = retrieved.unwrap();
        match retrieved_record {
            Cow::Owned(record) => {
                assert_eq!(record.key, kad_record.key);
            }
            Cow::Borrowed(_) => {
                // This could also be valid depending on implementation
            }
        }

        // Test remove
        store.remove(&kad_record.key);
        let after_remove = store.get(&kad_record.key);
        assert!(after_remove.is_none());
    }

    #[test]
    fn test_provider_operations() {
        let peer_id = PeerId::random();
        let database = create_test_database();
        let mut store = NativeDBStore::<TestCatalog>::new(peer_id, database);

        let record_key = RecordKey::new(&"provider_test".as_bytes());
        let provider_record = ProviderRecord {
            key: record_key.clone(),
            provider: peer_id,
            expires: None,
            addresses: vec![],
        };

        // Test add provider
        store.add_provider(provider_record.clone()).unwrap();

        // Test get providers
        let providers = store.providers(&record_key);
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].provider, peer_id);

        // Test remove provider
        store.remove_provider(&record_key, &peer_id);
        let providers_after_remove = store.providers(&record_key);
        assert!(providers_after_remove.is_empty());
    }

    #[test]
    fn test_native_object_storage() {
        let peer_id = PeerId::random();
        let database = create_test_database();
        let mut store = NativeDBStore::<TestCatalog>::new(peer_id, database);

        let test_model = TestModel {
            id: "test_id".to_string(),
            data: "test_data".to_string(),
        };
        let catalog_obj = TestCatalog::TestModel(test_model.clone());

        // Store native object
        let kad_record = store.store_native_object(catalog_obj.clone()).unwrap();
        assert!(!kad_record.key.as_ref().is_empty());

        // Get the key string for retrieval
        let catalog_key = catalog_obj.catalog_key();
        let key_bytes = TestCatalog::key_to_bytes(&catalog_key);
        let key_string = String::from_utf8_lossy(&key_bytes).to_string();
        let retrieved = store.get_native_object(&key_string);
        assert!(retrieved.is_some());

        if let TestCatalog::TestModel(retrieved_model) = retrieved.unwrap() {
            assert_eq!(retrieved_model, test_model);
        } else {
            panic!("Retrieved wrong variant");
        }
    }

    #[test]
    fn test_records_iterator() {
        let peer_id = PeerId::random();
        let database = create_test_database();
        let mut store = NativeDBStore::<TestCatalog>::new(peer_id, database);

        // Add multiple catalog objects
        let test_model1 = TestModel {
            id: "test_id1".to_string(),
            data: "test_data1".to_string(),
        };
        let catalog_obj1 = TestCatalog::TestModel(test_model1);
        let kad_record1 = catalog_obj1.to_kad_record();

        let test_model2 = TestModel {
            id: "test_id2".to_string(),
            data: "test_data2".to_string(),
        };
        let catalog_obj2 = TestCatalog::TestModel(test_model2);
        let kad_record2 = catalog_obj2.to_kad_record();

        store.put(kad_record1).unwrap();
        store.put(kad_record2).unwrap();

        // Test records iterator
        let records: Vec<_> = store.records().collect();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_bincode_data_flow() {
        let peer_id = PeerId::random();
        let database = create_test_database();
        let mut store = NativeDBStore::<TestCatalog>::new(peer_id, database);

        let test_model = TestModel {
            id: "bincode_test".to_string(),
            data: "bincode_data".to_string(),
        };
        let catalog_obj = TestCatalog::TestModel(test_model.clone());

        // Step 1: Store as native object (uses bincode internally)
        let kad_record = store.store_native_object(catalog_obj).unwrap();

        // Step 2: Simulate network transmission by converting back and forth
        let record_back: Result<Record<TestCatalog>, _> = kad_record.try_into();
        assert!(record_back.is_ok());
        let recovered_obj = record_back.unwrap().data;

        // Verify bincode round-trip worked
        if let TestCatalog::TestModel(recovered_model) = recovered_obj {
            assert_eq!(recovered_model.id, test_model.id);
            assert_eq!(recovered_model.data, test_model.data);
        } else {
            panic!("Bincode round-trip failed");
        }
    }
}
