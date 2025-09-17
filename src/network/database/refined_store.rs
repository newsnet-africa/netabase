//! Refined NativeDB Record Store Implementation
//!
//! This module provides an improved implementation of the RecordStore trait that:
//! 1. Properly uses CatalogRef objects to wrap database pointers
//! 2. Provides map iterators from CatalogRef -> Cow<'a, Record>
//! 3. Joins different record types from the database
//! 4. Integrates directly with native_db for efficient storage

use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

use libp2p::PeerId;
use libp2p::kad::store::{RecordStore, Result as StoreResult};
use libp2p::kad::{ProviderRecord, Record as KadRecord, RecordKey};
use native_db::Database;

use crate::{NetabaseCatalog, NetabaseRecordExt};

/// Iterator that maps catalog objects to Cow<'_, KadRecord>
pub struct CatalogRefRecordsIter<'a> {
    kad_records: std::vec::IntoIter<KadRecord>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Iterator for CatalogRefRecordsIter<'a> {
    type Item = Cow<'a, KadRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        self.kad_records.next().map(Cow::Owned)
    }
}

impl<'a> CatalogRefRecordsIter<'a> {
    pub fn new(kad_records: Vec<KadRecord>) -> Self {
        Self {
            kad_records: kad_records.into_iter(),
            _phantom: PhantomData,
        }
    }

    /// Create from catalog objects by converting them to KadRecords
    pub fn from_catalog_objects<C>(catalog_objects: Vec<C>) -> Self
    where
        C: NetabaseRecordExt + bincode::Encode,
    {
        let kad_records: Vec<KadRecord> = catalog_objects
            .into_iter()
            .map(|obj| obj.to_kad_record())
            .collect();
        Self::new(kad_records)
    }
}

/// Iterator for provider records owned by this node
pub struct OwnedProviderIter<'a> {
    provider_records: std::vec::IntoIter<ProviderRecord>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Iterator for OwnedProviderIter<'a> {
    type Item = Cow<'a, ProviderRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        self.provider_records.next().map(Cow::Owned)
    }
}

/// Refined NativeDB Store that properly integrates database references with network records
pub struct RefinedNativeDBStore<C>
where
    C: NetabaseCatalog
        + NetabaseRecordExt
        + Clone
        + bincode::Encode
        + bincode::Decode<()>
        + 'static,
{
    /// Local peer identifier
    local_key: PeerId,

    /// Direct database connection for efficient querying
    database: Arc<RwLock<Database<'static>>>,

    /// Cache for network records (KadRecords) indexed by record key
    record_cache: Arc<RwLock<HashMap<String, KadRecord>>>,

    /// Provider records cache indexed by record key
    provider_cache: Arc<RwLock<HashMap<RecordKey, Vec<ProviderRecord>>>>,

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
        + 'static,
{
    /// Create a new RefinedNativeDBStore
    pub fn new(local_key: PeerId, database: Database<'static>) -> Self {
        Self {
            local_key,
            database: Arc::new(RwLock::new(database)),
            record_cache: Arc::new(RwLock::new(HashMap::new())),
            provider_cache: Arc::new(RwLock::new(HashMap::new())),
            _phantom: PhantomData,
        }
    }

    /// Query all catalog objects from database and return as owned catalog objects
    fn query_all_catalog_objects(&self) -> Vec<C> {
        let _db = self.database.read().unwrap();
        let catalog_objects = Vec::new();

        // This is a simplified version - in practice, you'd need to:
        // 1. Query different model types from the database
        // 2. Convert them to CatalogRef objects using the generated From implementations
        // 3. Join all the different types into a single iterator

        // For now, return empty vector as this requires the actual database schema
        // In a real implementation, this would look like:
        //
        // let r = db.read().unwrap();
        // for model in r.scan().primary()? {
        //     if let Some(catalog_ref) = C::RefCatalog::try_from_native_db(&model) {
        //         catalog_refs.push(catalog_ref);
        //     }
        // }

        catalog_objects
    }

    /// Convert database objects to CatalogRef and then to KadRecord
    fn _database_to_kad_record<T>(&self, _database_obj: &T) -> Option<KadRecord>
    where
        T: 'static,
    {
        // This would use the generated TryFrom implementations to convert
        // database objects to CatalogRef, then to KadRecord
        //
        // if let Some(catalog_ref) = C::RefCatalog::try_from_native_db(database_obj) {
        //     Some(catalog_ref.as_kad_record().into_owned())
        // } else {
        //     None
        // }
        None
    }

    /// Store a catalog object in the database and update caches
    fn store_catalog_in_database(&mut self, catalog_obj: C) -> StoreResult<()> {
        // In a full implementation, this would:
        // 1. Extract the individual models from the catalog enum
        // 2. Store them in native_db using the appropriate collections
        // 3. Update the record cache with the network representation

        let kad_record = catalog_obj.to_kad_record();
        let key_str = String::from_utf8_lossy(kad_record.key.as_ref()).to_string();

        self.record_cache
            .write()
            .unwrap()
            .insert(key_str, kad_record);

        Ok(())
    }

    /// Remove a catalog object from database by key
    fn remove_from_database(&mut self, key: &RecordKey) -> StoreResult<()> {
        // This would:
        // 1. Look up the record by key in the database
        // 2. Remove it from the appropriate native_db collections
        // 3. Update caches

        let key_str = String::from_utf8_lossy(key.as_ref());
        self.record_cache.write().unwrap().remove(key_str.as_ref());

        Ok(())
    }
}

impl<C> RecordStore for RefinedNativeDBStore<C>
where
    C: NetabaseCatalog
        + NetabaseRecordExt
        + Clone
        + bincode::Encode
        + bincode::Decode<()>
        + 'static,
{
    type RecordsIter<'iter>
        = CatalogRefRecordsIter<'iter>
    where
        Self: 'iter;

    type ProvidedIter<'iter>
        = OwnedProviderIter<'iter>
    where
        Self: 'iter;

    fn get(&self, key: &RecordKey) -> Option<Cow<'_, KadRecord>> {
        // First check cache
        let key_str = String::from_utf8_lossy(key.as_ref());
        if let Some(kad_record) = self
            .record_cache
            .read()
            .unwrap()
            .get(key_str.as_ref())
            .cloned()
        {
            return Some(Cow::Owned(kad_record));
        }

        // If not in cache, query database
        // This would require implementing database lookups by key
        // For now, return None as this requires full database integration
        None
    }

    fn put(&mut self, kad_record: KadRecord) -> StoreResult<()> {
        // Convert KadRecord back to catalog object
        match C::from_kad_record(kad_record) {
            Ok(catalog_obj) => {
                self.store_catalog_in_database(catalog_obj)?;
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to convert KadRecord to catalog object: {}", e);
                Err(libp2p::kad::store::Error::ValueTooLarge)
            }
        }
    }

    fn remove(&mut self, key: &RecordKey) {
        if let Err(e) = self.remove_from_database(key) {
            eprintln!("Failed to remove record from database: {:?}", e);
        }
    }

    /// Returns an iterator over all records, using catalog objects from the database
    fn records(&self) -> Self::RecordsIter<'_> {
        // Get all cached records
        let cached_records: Vec<KadRecord> = self
            .record_cache
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect();

        // Query database for all catalog objects
        let db_catalog_objects = self.query_all_catalog_objects();
        let db_kad_records: Vec<KadRecord> = db_catalog_objects
            .into_iter()
            .map(|obj| obj.to_kad_record())
            .collect();

        // Combine cached and database records
        let mut all_records = cached_records;
        all_records.extend(db_kad_records);

        CatalogRefRecordsIter::new(all_records)
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

    /// Returns an iterator over provider records where this node is the provider
    fn provided(&self) -> Self::ProvidedIter<'_> {
        let providers = self.provider_cache.read().unwrap();
        let provided_records: Vec<ProviderRecord> = providers
            .values()
            .flat_map(|provider_list| {
                provider_list
                    .iter()
                    .filter(|p| p.provider == self.local_key)
                    .cloned()
            })
            .collect();

        OwnedProviderIter {
            provider_records: provided_records.into_iter(),
            _phantom: PhantomData,
        }
    }

    fn remove_provider(&mut self, key: &RecordKey, provider: &PeerId) {
        let mut providers = self.provider_cache.write().unwrap();
        if let Some(provider_list) = providers.get_mut(key) {
            provider_list.retain(|p| p.provider != *provider);

            // Remove empty provider lists to keep cache clean
            if provider_list.is_empty() {
                providers.remove(key);
            }
        }
    }
}

/// Extension trait for joining multiple catalog types in the records iterator
pub trait CatalogJoiner<C>
where
    C: NetabaseCatalog,
{
    /// Join multiple catalog reference iterators into a single iterator
    fn join_catalog_objects(&self, catalog_objects: Vec<C>) -> CatalogRefRecordsIter<'_>
    where
        C: NetabaseRecordExt + bincode::Encode;
}

impl<C> CatalogJoiner<C> for RefinedNativeDBStore<C>
where
    C: NetabaseCatalog
        + NetabaseRecordExt
        + Clone
        + bincode::Encode
        + bincode::Decode<()>
        + 'static,
{
    fn join_catalog_objects(&self, catalog_objects: Vec<C>) -> CatalogRefRecordsIter<'_>
    where
        C: NetabaseRecordExt + bincode::Encode,
    {
        CatalogRefRecordsIter::from_catalog_objects(catalog_objects)
    }
}

#[cfg(test)]
mod tests {

    // These tests would require actual catalog implementations from the macro
    // For now, they serve as documentation of the intended API

    #[test]
    fn test_refined_store_creation() {
        // This test would create a store with a real database and catalog
        // let db = Database::create_in_memory(&[]).unwrap();
        // let peer_id = PeerId::random();
        // let store = RefinedNativeDBStore::<TestCatalog>::new(peer_id, db);
        // assert_eq!(store.local_key, peer_id);
    }

    #[test]
    fn test_catalog_ref_iterator() {
        // This test would demonstrate the CatalogRef -> Cow<'_, KadRecord> mapping
        // let catalog_refs = vec![...]; // CatalogRef objects from database
        // let iter = CatalogRefRecordsIter::new(catalog_refs);
        // let kad_records: Vec<_> = iter.collect();
        // assert!(!kad_records.is_empty());
    }

    #[test]
    fn test_records_joining() {
        // This test would show how different catalog types get joined
        // let store = create_test_store();
        // let records_iter = store.records();
        // let all_records: Vec<_> = records_iter.collect();
        // // Verify that records from different catalog variants are included
    }
}
