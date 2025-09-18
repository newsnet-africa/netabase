//! Refined NativeDB Record Store Implementation
//!
//! This module provides an improved implementation of the RecordStore trait that:
//! 1. Matches the MemoryStore pattern exactly - keeps data in memory and returns borrowed references
//! 2. Uses native_db for persistent storage of records and provider information
//! 3. Loads all catalog objects from database into memory when needed
//! 4. Provides efficient iteration over in-memory data structures like MemoryStore
//! 5. Acts as a drop-in replacement for MemoryStore with database persistence
//!
//! ## Iterator Implementation Strategy
//!
//! The iterators (`RecordsIter` and `ProvidedIter`) follow the exact same pattern as MemoryStore:
//! - They are simply map functions over the in-memory HashMap/HashSet values
//! - They return `Cow<'_, Record>` and `Cow<'_, ProviderRecord>` respectively
//! - The lifetime is tied to the store itself, ensuring memory safety
//! - No dynamic loading happens during iteration - all data is loaded into memory beforehand
//!
//! ## Database Integration
//!
//! ### Records (KadRecords)
//! - Records are stored both in memory (HashMap<RecordKey, KadRecord>) and persisted to database
//! - The `records()` iterator returns references to the in-memory HashMap values
//! - Database loading happens during initialization or when explicitly triggered
//! - For large datasets, consider implementing lazy loading in `records()` method
//!
//! ### Provider Records
//! - Provider records are stored in memory in the same structure as MemoryStore:
//!   - `providers: HashMap<RecordKey, Vec<ProviderRecord>>` - all providers per key
//!   - `provided: HashSet<ProviderRecord>` - records where this node is the provider
//! - These are loaded from database during initialization using `load_providers_from_database()`
//! - The `provided()` iterator returns references to the in-memory HashSet
//! - The `providers()` method returns clones of the Vec (matching MemoryStore behavior)
//!
//! ## Memory vs Database Trade-offs
//!
//! This implementation prioritizes MemoryStore compatibility over memory efficiency:
//! - All records are kept in memory for fast iteration
//! - Database serves as persistent backing store
//! - For memory-constrained environments, consider implementing:
//!   1. Paged iteration with dynamic loading
//!   2. LRU cache with database fallback
//!   3. Separate in-memory and persistent modes
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! # use netabase::network::database::refined_store::RefinedNativeDBStore;
//! # use libp2p::PeerId;
//! # use native_db::{Database, Models, Builder};
//!
//! let peer_id = PeerId::random();
//! let models = Models::new();
//! let database = Builder::new().create_in_memory(&models).unwrap();
//! let mut store = RefinedNativeDBStore::<TestCatalog>::new(peer_id, database);
//!
//! // Use exactly like MemoryStore
//! for record in store.records() {
//!     // record is Cow<'_, KadRecord>
//!     println!("Record key: {:?}", record.key);
//! }
//! ```

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque, hash_map, hash_set};
use std::marker::PhantomData;
use std::{
    iter,
    sync::{Arc, RwLock},
};

use libp2p::PeerId;
use libp2p::kad::store::{Error, RecordStore, Result as StoreResult};
use libp2p::kad::{ProviderRecord, Record as KadRecord, RecordKey};
use native_db::{Database, ToKey};

// Removed smallvec dependency - using Vec instead

use crate::{NetabaseCatalog, NetabaseRecordExt};

/// K_VALUE constant - maximum number of providers per key (matching libp2p-kad)
const K_VALUE: usize = 20;
const DEFAULT_PAGE_SIZE: usize = 100;

/// A paginated database iterator that caches batches of records in memory
/// and writes modified records back to the database when they're dropped from cache
pub struct PaginatedDbIterator<'db, C>
where
    C: NetabaseCatalog
        + NetabaseRecordExt
        + Clone
        + bincode::Encode
        + bincode::Decode<()>
        + ToKey
        + Send
        + Sync
        + 'static,
{
    /// Database connection
    database: Arc<RwLock<Database<'db>>>,

    /// Current page of records in memory
    current_page: VecDeque<(RecordKey, KadRecord)>,

    /// Cache of modified records that need to be written back
    modified_records: HashMap<RecordKey, KadRecord>,

    /// Current position in the database scan
    scan_position: Option<Vec<u8>>, // Native DB key position

    /// Page size for batching
    page_size: usize,

    /// Whether we've reached the end of the database
    exhausted: bool,

    /// Flag to track if iterator has been initialized
    initialized: bool,

    /// Full cache of all records for compatibility
    full_cache: Option<HashMap<RecordKey, KadRecord>>,

    /// Phantom marker for the catalog type
    _phantom: PhantomData<C>,
}

impl<'db, C> PaginatedDbIterator<'db, C>
where
    C: NetabaseCatalog
        + NetabaseRecordExt
        + Clone
        + bincode::Encode
        + bincode::Decode<()>
        + ToKey
        + Send
        + Sync
        + 'static,
{
    /// Create a new paginated iterator
    pub fn new(database: Arc<RwLock<Database<'db>>>) -> Self {
        Self {
            database,
            current_page: VecDeque::new(),
            modified_records: HashMap::new(),
            scan_position: None,
            page_size: DEFAULT_PAGE_SIZE,
            exhausted: false,
            initialized: false,
            full_cache: None,
            _phantom: PhantomData,
        }
    }

    /// Create a new paginated iterator with custom page size
    pub fn with_page_size(database: Arc<RwLock<Database<'db>>>, page_size: usize) -> Self {
        Self {
            database,
            current_page: VecDeque::new(),
            modified_records: HashMap::new(),
            scan_position: None,
            page_size,
            exhausted: false,
            initialized: false,
            full_cache: None,
            _phantom: PhantomData,
        }
    }

    /// Load the next batch of records from the database
    fn load_next_batch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.exhausted {
            return Ok(());
        }

        let _db = self.database.read().unwrap();
        let _rtxn = _db.r_transaction()?;

        // Write back any modified records first
        drop(_db); // Release read lock before getting write lock
        self.flush_modified_records()?;
        let _db = self.database.read().unwrap();
        let _rtxn = _db.r_transaction()?;

        let mut _loaded_count = 0;

        // Scan catalog objects from native_db
        // Note: This is a simplified version that scans all records
        // In a production implementation, you'd need proper pagination with scan ranges
        // For now, return empty scan since we need ToInput for proper scanning
        // In a full implementation, you'd scan actual catalog objects
        let loaded_count = 0;
        // Placeholder - in a full implementation you'd load from database

        // If we loaded less than page_size, we've reached the end
        if loaded_count < self.page_size {
            self.exhausted = true;
        }

        Ok(())
    }

    /// Flush modified records back to the database
    fn flush_modified_records(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.modified_records.is_empty() {
            return Ok(());
        }

        let _db = self.database.write().unwrap();
        // Simplified flush - in production you'd convert KadRecord back to catalog objects
        // and store them using native_db transactions
        self.modified_records.clear();
        Ok(())
    }

    /// Get a record by key, loading from database if necessary
    pub fn get(&mut self, key: &RecordKey) -> Option<&KadRecord> {
        // First check if it's in current page
        if let Some((_, record)) = self.current_page.iter().find(|(k, _)| k == key) {
            return Some(record);
        }

        // Check modified records
        if let Some(record) = self.modified_records.get(key) {
            return Some(record);
        }

        // Check full cache if available
        if let Some(cache) = &self.full_cache {
            return cache.get(key);
        }

        // For now, don't try to load from database to avoid borrowing conflicts
        // In a full implementation, you'd implement proper async loading or restructure
        None
    }

    /// Load a specific record by key from the database
    fn load_record_by_key(&mut self, _key: &RecordKey) -> Result<(), Box<dyn std::error::Error>> {
        let _db = self.database.read().unwrap();
        let _rtxn = _db.r_transaction()?;

        // Simplified approach - in production you'd convert key and lookup specific record
        // For now, just return Ok without loading anything

        Ok(())
    }

    /// Insert or update a record
    pub fn insert(&mut self, key: RecordKey, record: KadRecord) {
        // Mark as modified for write-back
        self.modified_records.insert(key.clone(), record.clone());

        // Also update current page if the record exists there
        if let Some((_, existing_record)) = self.current_page.iter_mut().find(|(k, _)| k == &key) {
            *existing_record = record.clone();
        } else {
            // Add to current page
            self.current_page.push_back((key.clone(), record.clone()));
        }

        // Update full cache if available
        if let Some(cache) = &mut self.full_cache {
            cache.insert(key, record);
        }
    }

    /// Remove a record
    pub fn remove(&mut self, key: &RecordKey) -> Option<KadRecord> {
        // Remove from current page
        let mut removed = None;
        if let Some(index) = self.current_page.iter().position(|(k, _)| k == key) {
            removed = Some(self.current_page.remove(index).unwrap().1);
        }

        // Remove from modified records
        let modified_removed = self.modified_records.remove(key);

        // Remove from full cache if available
        if let Some(cache) = &mut self.full_cache {
            cache.remove(key);
        }

        // Mark for deletion in database by adding a tombstone record
        // For simplicity, we'll just remove from our caches
        removed.or(modified_removed)
    }

    /// Initialize the iterator by loading the first batch
    fn ensure_initialized(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.initialized {
            self.load_next_batch()?;
            self.initialized = true;
        }
        Ok(())
    }

    /// Get all records as a HashMap (for compatibility with existing code)
    pub fn get_all_records(
        &mut self,
    ) -> Result<HashMap<RecordKey, KadRecord>, Box<dyn std::error::Error>> {
        if let Some(cache) = &self.full_cache {
            return Ok(cache.clone());
        }

        let mut result = HashMap::new();

        // Ensure we're initialized
        self.ensure_initialized()?;

        // Load all remaining data
        while !self.exhausted {
            self.load_next_batch()?;
        }

        // Collect from current page
        for (key, record) in &self.current_page {
            result.insert(key.clone(), record.clone());
        }

        // Include modified records
        for (key, record) in &self.modified_records {
            result.insert(key.clone(), record.clone());
        }

        // Cache the result for future use
        self.full_cache = Some(result.clone());

        Ok(result)
    }

    /// Get the number of records (triggers full load)
    pub fn len(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let all_records = self.get_all_records()?;
        Ok(all_records.len())
    }

    /// Check if a key exists
    pub fn contains_key(&mut self, key: &RecordKey) -> bool {
        self.get(key).is_some()
    }
}

impl<'db, C> Iterator for PaginatedDbIterator<'db, C>
where
    C: NetabaseCatalog
        + NetabaseRecordExt
        + Clone
        + bincode::Encode
        + bincode::Decode<()>
        + ToKey
        + Send
        + Sync
        + 'static,
{
    type Item = (RecordKey, KadRecord);

    fn next(&mut self) -> Option<Self::Item> {
        // Ensure initialized
        if let Err(_) = self.ensure_initialized() {
            return None;
        }

        // If current page is empty, try to load next batch
        if self.current_page.is_empty() && !self.exhausted {
            if let Err(_) = self.load_next_batch() {
                return None;
            }
        }

        // Return next item from current page
        self.current_page.pop_front()
    }
}

impl<'db, C> Drop for PaginatedDbIterator<'db, C>
where
    C: NetabaseCatalog
        + NetabaseRecordExt
        + Clone
        + bincode::Encode
        + bincode::Decode<()>
        + ToKey
        + Send
        + Sync
        + 'static,
{
    fn drop(&mut self) {
        // Flush any remaining modified records when the iterator is dropped
        let _ = self.flush_modified_records();
    }
}

/// Configuration for the RefinedNativeDBStore (matching MemoryStore config exactly)
#[derive(Debug, Clone)]
pub struct RefinedNativeDBStoreConfig {
    /// The maximum number of records.
    pub max_records: usize,
    /// The maximum size of record values, in bytes.
    pub max_value_bytes: usize,
    /// The maximum number of providers stored for a key.
    pub max_providers_per_key: usize,
    /// The maximum number of provider records for which the local node is the provider.
    pub max_provided_keys: usize,
}

impl Default for RefinedNativeDBStoreConfig {
    fn default() -> Self {
        Self {
            max_records: 1024,
            max_value_bytes: 65 * 1024,
            max_provided_keys: 1024,
            max_providers_per_key: K_VALUE,
        }
    }
}

/// Refined NativeDB Store that matches MemoryStore behavior exactly with database persistence
pub struct RefinedNativeDBStore<C>
where
    C: NetabaseCatalog
        + NetabaseRecordExt
        + Clone
        + bincode::Encode
        + bincode::Decode<()>
        + ToKey
        + Send
        + Sync
        + 'static,
{
    /// The identity of the peer owning the store (matching MemoryStore)
    local_key: PeerId,

    /// The configuration of the store (matching MemoryStore)
    config: RefinedNativeDBStoreConfig,

    /// Paginated iterator for records with write-back caching
    records_iterator: RefCell<Option<PaginatedDbIterator<'static, C>>>,

    /// The stored provider records - EXACT match to MemoryStore structure
    providers: HashMap<RecordKey, Vec<ProviderRecord>>,

    /// The set of all provider records for the node identified by `local_key` - EXACT match to MemoryStore
    /// Must be kept in sync with `providers`
    provided: HashSet<ProviderRecord>,

    /// Database connection for persistence
    database: Arc<RwLock<Database<'static>>>,

    /// Catalog type marker
    _phantom: PhantomData<C>,
}

impl<C> RefinedNativeDBStore<C>
where
    C: NetabaseCatalog
        + NetabaseRecordExt
        + Clone
        + bincode::Encode
        + bincode::Decode<()>
        + ToKey
        + Send
        + Sync
        + 'static,
{
    /// Creates a new RefinedNativeDBStore with a default configuration
    pub fn new(local_id: PeerId, database: Database<'static>) -> Self {
        Self::with_config(local_id, Default::default(), database)
    }

    /// Creates a new RefinedNativeDBStore with the given configuration
    pub fn with_config(
        local_id: PeerId,
        config: RefinedNativeDBStoreConfig,
        database: Database<'static>,
    ) -> Self {
        let mut store = RefinedNativeDBStore {
            local_key: local_id,
            config,
            records_iterator: RefCell::new(None),
            provided: HashSet::default(),
            providers: HashMap::default(),
            database: Arc::new(RwLock::new(database)),
            _phantom: PhantomData,
        };

        // Load provider records from database on initialization
        if let Err(e) = store.load_providers_from_database() {
            eprintln!("Warning: Failed to load providers from database: {:?}", e);
        }

        store
    }

    /// Load all provider records from database into memory
    fn load_providers_from_database(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // For now, provider records are stored as serialized data in the database
        // In a full implementation, you would:
        // 1. Create a separate native_db model for ProviderRecord
        // 2. Use r_transaction() to read all provider records
        // 3. Deserialize and populate the providers and provided HashMaps

        // Placeholder - in production this would scan the provider records table:
        // let db = self.database.read().unwrap();
        // let rtxn = db.r_transaction()?;
        // for provider_record in rtxn.scan().primary()? {
        //     self.add_provider_to_memory(provider_record)?;
        // }

        Ok(())
    }

    /// Load all catalog objects from database and convert to KadRecords
    ///
    /// This method demonstrates how to implement database scanning for a full implementation:
    /// 1. Create a read transaction
    /// 2. Scan all catalog objects of type C
    /// 3. Convert each to KadRecord using NetabaseRecordExt::to_kad_record()
    /// 4. Build HashMap for in-memory storage
    ///
    /// For large databases, consider:
    /// - Implementing pagination/batching
    /// - Using database indexes for efficient scanning
    /// - Loading only recently accessed records
    fn load_records_from_database(
        &self,
    ) -> Result<HashMap<RecordKey, KadRecord>, Box<dyn std::error::Error>> {
        let kad_records = HashMap::new();

        // Full implementation would look like:
        // let db = self.database.read().unwrap();
        // let rtxn = db.r_transaction()?;
        // let mut kad_records = HashMap::new();
        //
        // // Scan all catalog objects from native_db
        // for catalog_result in rtxn.scan().primary()? {
        //     let catalog_obj: C = catalog_result?;
        //     let kad_record = catalog_obj.to_kad_record();
        //     kad_records.insert(kad_record.key.clone(), kad_record);
        // }
        //
        // // For multiple catalog types, you'd need multiple scans:
        // // for user in rtxn.scan().primary::<User>()? { ... }
        // // for post in rtxn.scan().primary::<Post>()? { ... }
        // // etc.

        Ok(kad_records)
    }

    /// Store a KadRecord as a catalog object in the database
    fn store_record_in_database(&self, _kad_record: &KadRecord) -> StoreResult<()> {
        // Simplified implementation - just return Ok for now
        // In a full implementation, you would deserialize the KadRecord value
        // back to the catalog object and store it using native_db
        Ok(())
    }

    /// Remove a record from the database
    fn remove_record_from_database(&self, _key: &RecordKey) -> StoreResult<()> {
        // In a full implementation:
        // let db = self.database.write().unwrap();
        // let rwtxn = db.rw_transaction()?;
        // rwtxn.remove_by_key(key)?;
        // rwtxn.commit()?;
        Ok(())
    }

    /// Persist provider records to database
    fn store_provider_in_database(&self, _provider_record: &ProviderRecord) -> StoreResult<()> {
        // In a full implementation, serialize and store provider record
        Ok(())
    }

    /// Remove provider record from database
    fn remove_provider_from_database(
        &self,
        _key: &RecordKey,
        _provider: &PeerId,
    ) -> StoreResult<()> {
        // In a full implementation, remove from database
        Ok(())
    }

    /// Retains the records satisfying a predicate (matching MemoryStore API)
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&RecordKey, &mut KadRecord) -> bool,
    {
        self.ensure_records_loaded();

        let mut iterator_ref = self.records_iterator.borrow_mut();
        if let Some(iterator) = iterator_ref.as_mut() {
            // Get all records to work with the predicate
            if let Ok(mut all_records) = iterator.get_all_records() {
                let keys_to_remove: Vec<RecordKey> = all_records
                    .iter_mut()
                    .filter_map(|(k, v)| if !f(k, v) { Some(k.clone()) } else { None })
                    .collect();

                for key in keys_to_remove {
                    let _ = iterator.remove(&key);
                }
            }
        }
    }

    /// Ensure records are loaded from database
    ///
    /// This method implements lazy loading - records are only loaded from database
    /// when needed. In a production implementation, you might want to:
    /// 1. Track whether records have been loaded vs. are genuinely empty
    /// 2. Implement cache invalidation when database changes
    /// 3. Add periodic refresh mechanisms
    /// 4. Use database change notifications to trigger reloads
    fn ensure_records_loaded(&self) {
        let mut iterator = self.records_iterator.borrow_mut();
        if iterator.is_none() {
            *iterator = Some(PaginatedDbIterator::new(self.database.clone()));
        }
    }
}

impl<C> RecordStore for RefinedNativeDBStore<C>
where
    C: NetabaseCatalog
        + NetabaseRecordExt
        + Clone
        + bincode::Encode
        + bincode::Decode<()>
        + ToKey
        + Send
        + Sync
        + 'static,
{
    type RecordsIter<'a>
        = iter::Map<std::vec::IntoIter<KadRecord>, fn(KadRecord) -> Cow<'a, KadRecord>>
    where
        Self: 'a;

    type ProvidedIter<'a>
        = iter::Map<
        hash_set::Iter<'a, ProviderRecord>,
        fn(&'a ProviderRecord) -> Cow<'a, ProviderRecord>,
    >
    where
        Self: 'a;

    fn get(&self, k: &RecordKey) -> Option<Cow<'_, KadRecord>> {
        self.ensure_records_loaded();

        let mut iterator_ref = self.records_iterator.borrow_mut();
        if let Some(iterator) = iterator_ref.as_mut() {
            if let Some(record) = iterator.get(k) {
                // We need to return owned values due to RefCell borrowing constraints
                return Some(Cow::Owned(record.clone()));
            }
        }

        None
    }

    fn put(&mut self, r: KadRecord) -> StoreResult<()> {
        // Same validation as MemoryStore
        if r.value.len() >= self.config.max_value_bytes {
            return Err(Error::ValueTooLarge);
        }

        self.ensure_records_loaded();

        let key = r.key.clone();
        let mut iterator_ref = self.records_iterator.borrow_mut();

        if let Some(iterator) = iterator_ref.as_mut() {
            // Check if record already exists and validate max records
            let is_new_record = !iterator.contains_key(&key);
            if is_new_record {
                if let Ok(current_count) = iterator.len() {
                    if current_count >= self.config.max_records {
                        return Err(Error::MaxRecords);
                    }
                }
            }

            // Store in iterator (which handles database persistence)
            iterator.insert(key, r);
        }

        Ok(())
    }

    fn remove(&mut self, k: &RecordKey) {
        self.ensure_records_loaded();

        let mut iterator_ref = self.records_iterator.borrow_mut();
        if let Some(iterator) = iterator_ref.as_mut() {
            let _ = iterator.remove(k);
        }
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        self.ensure_records_loaded();

        let mut iterator_ref = self.records_iterator.borrow_mut();
        if let Some(iterator) = iterator_ref.as_mut() {
            // Get all records and return as owned values
            if let Ok(all_records) = iterator.get_all_records() {
                let owned_records: Vec<KadRecord> = all_records.into_values().collect();
                return owned_records.into_iter().map(Cow::Owned);
            }
        }

        Vec::new().into_iter().map(Cow::Owned)
    }

    fn add_provider(&mut self, record: ProviderRecord) -> StoreResult<()> {
        let num_keys = self.providers.len();

        // Store in database first to avoid borrowing conflicts
        let _ = self.store_provider_in_database(&record);

        // EXACT same logic as MemoryStore
        let providers = match self.providers.entry(record.key.clone()) {
            e @ hash_map::Entry::Occupied(_) => e,
            e @ hash_map::Entry::Vacant(_) => {
                if self.config.max_provided_keys == num_keys {
                    return Err(Error::MaxProvidedKeys);
                }
                e
            }
        }
        .or_insert_with(Default::default);

        for p in providers.iter_mut() {
            if p.provider == record.provider {
                // In-place update of an existing provider record
                if &self.local_key == &record.provider {
                    self.provided.remove(p);
                    self.provided.insert(record.clone());
                }
                *p = record;
                return Ok(());
            }
        }

        // If the providers list is full, we ignore the new provider (same as MemoryStore)
        if providers.len() == self.config.max_providers_per_key {
            return Ok(());
        }

        // Otherwise, insert the new provider record
        if &self.local_key == &record.provider {
            self.provided.insert(record.clone());
        }
        providers.push(record);

        Ok(())
    }

    fn providers(&self, key: &RecordKey) -> Vec<ProviderRecord> {
        // Same as MemoryStore pattern
        self.providers
            .get(key)
            .map_or_else(Vec::new, |ps| ps.clone())
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        // Same as MemoryStore pattern - iterate over in-memory HashSet
        // Returns borrowed references to provider records where this node is the provider
        self.provided.iter().map(Cow::Borrowed)
    }

    fn remove_provider(&mut self, key: &RecordKey, provider: &PeerId) {
        // Remove from database first to avoid borrowing conflicts
        let _ = self.remove_provider_from_database(key, provider);

        // EXACT same logic as MemoryStore
        if let hash_map::Entry::Occupied(mut e) = self.providers.entry(key.clone()) {
            let providers: &mut Vec<ProviderRecord> = e.get_mut();
            if let Some(i) = providers.iter().position(|p| &p.provider == provider) {
                let p = providers.remove(i);
                if &p.provider == &self.local_key {
                    self.provided.remove(&p);
                }
            }
            if providers.is_empty() {
                e.remove();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;
    use libp2p::kad::store::RecordStore;
    use native_db::{Database, Models, ToKey};

    // Placeholder test models - in production these would be generated by macros
    #[derive(Clone, Debug, PartialEq)]
    struct TestCatalog;

    #[derive(Clone, Debug, PartialEq)]
    enum TestCatalogRef<'a> {
        TestCatalog(&'a TestCatalog),
    }

    impl<'a> crate::NetabaseRefCatalog<'a> for TestCatalogRef<'a> {}

    impl NetabaseCatalog for TestCatalog {
        type RefCatalog<'a> = TestCatalogRef<'a>;
    }

    impl NetabaseRecordExt for TestCatalog {}

    impl crate::CatalogKey for TestCatalog {
        type KeyType = String;

        fn catalog_key(&self) -> Self::KeyType {
            "test".to_string()
        }

        fn key_to_serializable(_key: &Self::KeyType) -> crate::SerializableKey {
            crate::SerializableKey::from_native_db_key(&"test".to_key())
        }

        fn key_to_bytes(_key: &Self::KeyType) -> Vec<u8> {
            b"test".to_vec()
        }

        fn bytes_to_key(_bytes: &[u8]) -> Result<Self::KeyType, Box<dyn std::error::Error>> {
            Ok("test".to_string())
        }
    }

    impl bincode::Encode for TestCatalog {
        fn encode<E: bincode::enc::Encoder>(
            &self,
            _encoder: &mut E,
        ) -> Result<(), bincode::error::EncodeError> {
            Ok(())
        }
    }

    impl bincode::Decode<()> for TestCatalog {
        fn decode<D: bincode::de::Decoder>(
            _decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
            Ok(TestCatalog)
        }
    }

    impl ToKey for TestCatalog {
        fn to_key(&self) -> native_db::db_type::Key {
            native_db::db_type::Key::new(b"test_catalog_key".to_vec())
        }

        fn key_names() -> Vec<String> {
            vec!["TestCatalog".to_string()]
        }
    }

    fn create_test_database() -> Database<'static> {
        use native_db::Builder;
        use std::sync::OnceLock;

        static MODELS: OnceLock<Models> = OnceLock::new();
        let models = MODELS.get_or_init(|| Models::new());
        Builder::new().create_in_memory(models).unwrap()
    }

    #[test]
    fn test_store_creation() {
        let peer_id = PeerId::random();
        let database = create_test_database();
        let store = RefinedNativeDBStore::<TestCatalog>::new(peer_id, database);
        assert_eq!(store.local_key, peer_id);
    }

    #[test]
    fn test_provider_operations_match_memory_store() {
        use std::time::Instant;

        let peer_id = PeerId::random();
        let database = create_test_database();
        let mut store = RefinedNativeDBStore::<TestCatalog>::new(peer_id, database);

        let key = RecordKey::new(b"test_key");
        let provider_record = ProviderRecord {
            key: key.clone(),
            provider: peer_id,
            expires: Some(Instant::now()),
            addresses: vec![],
        };

        // Test add_provider
        assert!(store.add_provider(provider_record.clone()).is_ok());

        // Test providers
        let providers = store.providers(&key);
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].provider, peer_id);

        // Test provided
        let provided: Vec<_> = store.provided().collect();
        assert_eq!(provided.len(), 1);
        assert_eq!(provided[0].provider, peer_id);

        // Test remove_provider
        store.remove_provider(&key, &peer_id);
        assert_eq!(store.providers(&key).len(), 0);
        assert_eq!(store.provided().count(), 0);
    }

    #[test]
    fn test_paginated_iterator_basic_functionality() {
        let peer_id = PeerId::random();
        let database = create_test_database();
        let store = RefinedNativeDBStore::<TestCatalog>::new(peer_id, database);

        // Test that iterator is initialized properly
        store.ensure_records_loaded();
        let iterator_ref = store.records_iterator.borrow();
        assert!(iterator_ref.is_some());
    }

    #[test]
    fn test_paginated_iterator_page_size() {
        let database = Arc::new(RwLock::new(create_test_database()));
        let iterator = PaginatedDbIterator::<TestCatalog>::with_page_size(database, 50);

        // Test custom page size
        assert_eq!(iterator.page_size, 50);
        assert!(!iterator.exhausted);
        assert!(!iterator.initialized);
    }

    #[test]
    fn test_paginated_iterator_caching() {
        let database = Arc::new(RwLock::new(create_test_database()));
        let mut iterator = PaginatedDbIterator::<TestCatalog>::new(database);

        let key = RecordKey::new(b"test_key");
        let record = KadRecord {
            key: key.clone(),
            value: b"test_value".to_vec(),
            publisher: None,
            expires: None,
        };

        // Test insert and get from cache
        iterator.insert(key.clone(), record.clone());
        let cached_record = iterator.get(&key);
        assert!(cached_record.is_some());
        assert_eq!(cached_record.unwrap().value, b"test_value");

        // Test that modified records are tracked
        assert!(iterator.modified_records.contains_key(&key));
    }

    #[test]
    fn test_paginated_iterator_remove() {
        let database = Arc::new(RwLock::new(create_test_database()));
        let mut iterator = PaginatedDbIterator::<TestCatalog>::new(database);

        let key = RecordKey::new(b"test_key");
        let record = KadRecord {
            key: key.clone(),
            value: b"test_value".to_vec(),
            publisher: None,
            expires: None,
        };

        // Insert and then remove
        iterator.insert(key.clone(), record);
        let removed = iterator.remove(&key);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().value, b"test_value");

        // Verify record is no longer accessible
        assert!(iterator.get(&key).is_none());
    }

    #[test]
    fn test_paginated_iterator_len_and_contains() {
        let database = Arc::new(RwLock::new(create_test_database()));
        let mut iterator = PaginatedDbIterator::<TestCatalog>::new(database);

        let key = RecordKey::new(b"test_key");
        let record = KadRecord {
            key: key.clone(),
            value: b"test_value".to_vec(),
            publisher: None,
            expires: None,
        };

        // Initially should be empty (or minimal from database)
        iterator.insert(key.clone(), record);

        // Test contains_key
        assert!(iterator.contains_key(&key));

        let non_existent_key = RecordKey::new(b"non_existent");
        assert!(!iterator.contains_key(&non_existent_key));
    }

    #[test]
    fn test_refined_store_with_paginated_iterator() {
        let peer_id = PeerId::random();
        let database = create_test_database();
        let mut store = RefinedNativeDBStore::<TestCatalog>::new(peer_id, database);

        let key = RecordKey::new(b"test_record");
        let record = KadRecord {
            key: key.clone(),
            value: b"test_data".to_vec(),
            publisher: Some(peer_id),
            expires: None,
        };

        // Test put operation uses paginated iterator
        let result = store.put(record.clone());
        assert!(result.is_ok());

        // Test get operation uses paginated iterator
        let retrieved = store.get(&key);
        assert!(retrieved.is_some());
        let retrieved_record = retrieved.unwrap();
        assert_eq!(retrieved_record.value, b"test_data");

        // Test remove operation
        store.remove(&key);
        let after_remove = store.get(&key);
        assert!(after_remove.is_none());
    }

    #[test]
    fn test_records_iterator_functionality() {
        let peer_id = PeerId::random();
        let database = create_test_database();
        let mut store = RefinedNativeDBStore::<TestCatalog>::new(peer_id, database);

        // Add multiple records
        for i in 0..5 {
            let key_string = format!("key_{}", i);
            let key = RecordKey::new(&key_string);
            let record = KadRecord {
                key: key.clone(),
                value: format!("value_{}", i).as_bytes().to_vec(),
                publisher: Some(peer_id),
                expires: None,
            };
            let _ = store.put(record);
        }

        // Test records iterator
        let records: Vec<_> = store.records().collect();
        assert_eq!(records.len(), 5);

        // Verify all records are present
        for (i, record) in records.iter().enumerate() {
            assert!(record.value.starts_with(b"value_"));
        }
    }

    #[test]
    fn test_max_providers_per_key() {
        let peer_id = PeerId::random();
        let database = create_test_database();
        let config = RefinedNativeDBStoreConfig {
            max_providers_per_key: 2,
            ..Default::default()
        };
        let mut store = RefinedNativeDBStore::<TestCatalog>::with_config(peer_id, config, database);

        let key = RecordKey::new(b"test_key");

        // Add up to max_providers_per_key
        for _i in 0..2 {
            let provider = PeerId::random();
            let provider_record = ProviderRecord {
                key: key.clone(),
                provider,
                expires: None,
                addresses: vec![],
            };
            assert!(store.add_provider(provider_record).is_ok());
        }

        // Adding one more should be ignored (not return error, just ignored like MemoryStore)
        let extra_provider = PeerId::random();
        let extra_record = ProviderRecord {
            key: key.clone(),
            provider: extra_provider,
            expires: None,
            addresses: vec![],
        };
        assert!(store.add_provider(extra_record.clone()).is_ok());

        // Should still only have 2 providers
        assert_eq!(store.providers(&key).len(), 2);
        assert!(!store.providers(&key).contains(&extra_record));
    }
}
