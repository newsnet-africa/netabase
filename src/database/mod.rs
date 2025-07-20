use std::{borrow::Cow, collections::HashSet, iter, path::Path};

use libp2p::{
    PeerId,
    kad::{K_VALUE, ProviderRecord, Record, store::RecordStore},
};

use sled::{Db, IVec, Tree};
use smallvec::SmallVec;

use crate::database::wrappers::{try_ivec_to_record, try_record_to_ivec};

pub mod wrappers;

#[derive(Debug)]
pub struct SledStore {
    local_key: PeerId,
    config: SledStoreConfig,
    sled_db: Db,
    records: Tree,
    providers: Tree,
    provided: HashSet<ProviderRecord>,
    #[allow(dead_code)]
    path: std::path::PathBuf,
}

#[derive(Debug)]
pub struct SledStoreConfig {
    pub max_records: usize,
    pub max_value_bytes: usize,
    pub max_providers_per_key: usize,
    pub max_provided_keys: usize,
}

impl Default for SledStoreConfig {
    fn default() -> Self {
        Self {
            max_records: 1024,
            max_value_bytes: 65 * 1024,
            max_provided_keys: 1024,
            max_providers_per_key: K_VALUE.get(),
        }
    }
}

impl SledStore {
    pub fn new<P: AsRef<Path>>(local_id: PeerId, path: P) -> anyhow::Result<Self> {
        Self::with_config(local_id, path, SledStoreConfig::default())
    }

    pub fn with_config<P: AsRef<Path>>(
        local_id: PeerId,
        path: P,
        config: SledStoreConfig,
    ) -> anyhow::Result<Self> {
        log::info!("Creating SledStore at path: {:?}", path.as_ref());
        let sled_db = sled::open(&path).map_err(|e| {
            log::error!("Failed to open sled database at {:?}: {}", path.as_ref(), e);
            e
        })?;
        log::info!("Sled database opened successfully");

        let records = sled_db.open_tree("Records")?;
        log::info!("Records tree opened");

        let providers = sled_db.open_tree("Providers")?;
        log::info!("Providers tree opened");

        // Initialize an empty provided set
        let mut provided = HashSet::new();

        // Scan through all provider records to find local ones
        log::info!("Scanning providers tree for local provider records...");
        let mut provider_count = 0;
        let mut local_provider_count = 0;

        for res in providers.iter() {
            if let Ok((key, value)) = res {
                provider_count += 1;

                if key.as_ref() == b"Provided" {
                    continue; // Skip the "Provided" special key
                }

                // Try to deserialize the providers for this key
                if let Ok(provider_records) = wrappers::try_ivec_to_providers_smallvec(value) {
                    // Add any provider records that match our local ID
                    for record in provider_records {
                        if record.provider == local_id {
                            provided.insert(record);
                            local_provider_count += 1;
                        }
                    }
                } else {
                    log::warn!("Failed to deserialize provider records for key: {:?}", key);
                }
            }
        }

        log::info!(
            "Found {} total provider entries, {} local provider records for peer {:?}",
            provider_count,
            local_provider_count,
            local_id
        );

        let store = Self {
            local_key: local_id,
            config,
            records,
            sled_db,
            providers,
            provided,
            path: path.as_ref().to_path_buf(),
        };

        log::info!(
            "SledStore created successfully with {} provided records",
            store.provided.len()
        );
        Ok(store)
    }
}

impl Drop for SledStore {
    fn drop(&mut self) {
        // Ensure all data is flushed before cleanup
        let _ = self.providers.flush();
        let _ = self.records.flush();
        let _ = self.sled_db.flush();
    }
}
impl RecordStore for SledStore {
    type RecordsIter<'a> = iter::FilterMap<
        sled::Iter,
        fn(Result<(IVec, IVec), sled::Error>) -> Option<Cow<'a, Record>>,
    >;

    type ProvidedIter<'a> = iter::Map<
        std::collections::hash_set::Iter<'a, ProviderRecord>,
        fn(&'a ProviderRecord) -> Cow<'a, ProviderRecord>,
    >;

    fn get(&self, k: &libp2p::kad::RecordKey) -> Option<std::borrow::Cow<'_, libp2p::kad::Record>> {
        match self.records.get(k) {
            Ok(Some(ivec)) => try_ivec_to_record(ivec).ok().map(Cow::Owned),
            Ok(None) => None,
            Err(_) => None,
        }
    }

    fn put(&mut self, r: libp2p::kad::Record) -> libp2p::kad::store::Result<()> {
        //TODO: Error handling for the trait errors for the Kad implementation. check for shit like max length etc.
        match try_record_to_ivec(r.clone()) {
            Ok(record_ivec) => {
                match self.records.insert(r.key.clone(), record_ivec) {
                    Ok(_) => Ok(()),
                    Err(_) => Ok(()), // Silently handle the error by returning Ok
                }
            }
            Err(_) => Ok(()), // Silently handle the conversion error by returning Ok
        }
    }

    fn remove(&mut self, k: &libp2p::kad::RecordKey) {
        self.records.remove(k).expect("Remove Erruh");
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        self.records.iter().filter_map(|res| {
            res.ok()
                .map(|(_, v)| Cow::Owned(try_ivec_to_record(v).expect("Conversion Erruh")))
        })
    }

    fn add_provider(&mut self, record: ProviderRecord) -> libp2p::kad::store::Result<()> {
        // Check max provided keys limit for local providers
        if self.local_key == record.provider && self.provided.len() >= self.config.max_provided_keys
        {
            return Err(libp2p::kad::store::Error::MaxProvidedKeys);
        }

        let key = record.key.clone();
        let mut providers = match self
            .providers
            .get(key.clone())
            .expect("Failed to get providers")
        {
            Some(ivec) => wrappers::try_ivec_to_providers_smallvec(ivec)
                .expect("Failed to deserialize providers"),
            None => SmallVec::new(),
        };

        // Check if we're updating an existing provider
        if let Some(pos) = providers.iter().position(|p| p.provider == record.provider) {
            // Update the existing provider record
            if self.local_key == record.provider {
                self.provided.remove(&providers[pos]);
                self.provided.insert(record.clone());
            }
            providers[pos] = record;
        } else {
            // Add new provider if we haven't reached the limit
            if providers.len() < self.config.max_providers_per_key {
                if self.local_key == record.provider {
                    self.provided.insert(record.clone());
                }
                providers.push(record);
            }
        }

        // Update providers tree
        self.providers
            .insert(
                key,
                wrappers::try_providers_smallvec_to_ivec(providers)
                    .expect("Failed to serialize providers"),
            )
            .expect("Failed to update providers");

        Ok(())
    }

    fn providers(&self, key: &libp2p::kad::RecordKey) -> Vec<ProviderRecord> {
        match self.providers.get(key) {
            Ok(Some(ivec)) => match wrappers::try_ivec_to_providers_smallvec(ivec) {
                Ok(providers) => providers.to_vec(),
                Err(_) => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        self.provided.iter().map(Cow::Borrowed)
    }

    fn remove_provider(&mut self, k: &libp2p::kad::RecordKey, provider: &PeerId) {
        if let Ok(Some(ivec)) = self.providers.get(k) {
            let mut providers = wrappers::try_ivec_to_providers_smallvec(ivec)
                .expect("Failed to deserialize providers");

            if let Some(idx) = providers.iter().position(|p| &p.provider == provider) {
                let removed = providers.remove(idx);
                if removed.provider == self.local_key {
                    self.provided.remove(&removed);
                }

                // Update or remove the key in providers tree
                if providers.is_empty() {
                    let _ = self.providers.remove(k);
                } else {
                    let _ = self.providers.insert(
                        k,
                        wrappers::try_providers_smallvec_to_ivec(providers)
                            .expect("Failed to serialize providers"),
                    );
                }
            }
        }
    }
}

pub mod tests {
    use log::info;
    use std::time::Instant;

    use libp2p::{
        kad::{RecordKey, store::Error},
        multihash::Multihash,
    };

    use super::*;
    use crate::{get_test_temp_dir, init_logging};

    fn random_multihash() -> Multihash<32> {
        let digest_bytes = [
            0x16, 0x20, 0x64, 0x4b, 0xcc, 0x7e, 0x56, 0x43, 0x73, 0x04, 0x09, 0x99, 0xaa, 0xc8,
            0x9e, 0x76, 0x22, 0xf3, 0xca, 0x71, 0xfb, 0xa1, 0xd9, 0x72, 0xfd, 0x94, 0xa3, 0x1c,
            0x3b, 0xfb, 0xf2, 0x4e, 0x39, 0x38,
        ];
        Multihash::<32>::from_bytes(&digest_bytes).unwrap()
    }

    #[test]
    fn put_get_remove_record() {
        put_get_remove_record_with_number(Some(1));
    }

    pub fn put_get_remove_record_with_number(test_number: Option<u32>) {
        init_logging();
        let r = Record::new(random_multihash(), "Hello".into());
        let temp_dir = get_test_temp_dir(test_number);
        let mut store = SledStore::new(PeerId::random(), &temp_dir).expect("Creation Erruh");
        info!("Record: {r:?}\nStore: {store:?}");
        assert!(store.put(r.clone()).is_ok());
        assert_eq!(Some(Cow::Borrowed(&r)), store.get(&r.key));
        store.remove(&r.key);
        assert!(store.get(&r.key).is_none());

        // Cleanup test directory
        drop(store);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn add_get_remove_provider() {
        add_get_remove_provider_with_number(Some(2));
    }

    pub fn add_get_remove_provider_with_number(test_number: Option<u32>) {
        init_logging();
        let local_id = PeerId::random();
        let temp_dir = get_test_temp_dir(test_number);
        let mut store = SledStore::new(local_id, &temp_dir).expect("Creation Erruh");

        // Add provider
        let r = ProviderRecord::new(random_multihash(), local_id, vec![]);
        assert!(store.add_provider(r.clone()).is_ok());

        // Verify provider is stored
        let providers = store.providers(&r.key);
        assert_eq!(1, providers.len());
        assert!(providers.contains(&r));

        // Verify it's in the provided set
        assert_eq!(1, store.provided().count());
        assert_eq!(Cow::Borrowed(&r), store.provided().next().unwrap());

        // Remove provider
        store.remove_provider(&r.key, &r.provider);
        assert!(store.providers(&r.key).is_empty());
        assert_eq!(0, store.provided().count());

        // Cleanup test directory
        drop(store);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn provided() {
        provided_with_number(Some(3));
    }

    pub fn provided_with_number(test_number: Option<u32>) {
        init_logging();
        let local_id = PeerId::random();
        let temp_dir = get_test_temp_dir(test_number);
        let key = random_multihash();
        let rec = ProviderRecord::new(key.clone(), local_id, Vec::new());

        // Test sled trees persistence specifically
        info!("Testing sled trees persistence at: {}", temp_dir);
        {
            let db = sled::open(&temp_dir).expect("Failed to open sled db");
            let providers_tree = db
                .open_tree("Providers")
                .expect("Failed to open providers tree");

            // Insert test data in the providers tree
            providers_tree
                .insert("test_provider_key", "test_provider_value")
                .expect("Failed to insert provider test data");

            // Flush both tree and database
            providers_tree
                .flush()
                .expect("Failed to flush providers tree");
            db.flush().expect("Failed to flush test db");
            info!("Providers tree test data inserted and flushed");
        }

        // Verify sled trees persistence
        {
            let db = sled::open(&temp_dir).expect("Failed to reopen sled db");
            let providers_tree = db
                .open_tree("Providers")
                .expect("Failed to reopen providers tree");

            if let Ok(Some(value)) = providers_tree.get("test_provider_key") {
                info!(
                    "Sled trees persistence works: {}",
                    String::from_utf8_lossy(&value)
                );
            } else {
                info!("Sled trees persistence FAILED - no test data found");
            }
        }

        {
            let mut store = SledStore::new(local_id, &temp_dir).expect("Creation Erruh");
            info!("Initial store created");
            assert!(store.add_provider(rec.clone()).is_ok());
            info!("Added provider record");

            // Verify the record is in the provided set
            let provided: Vec<_> = store.provided().collect();
            info!("Initial provided count: {}", provided.len());
            assert_eq!(1, provided.len());
            assert_eq!(Cow::Borrowed(&rec), provided[0]);

            // Check providers tree directly
            if let Ok(Some(ivec)) = store.providers.get(&rec.key) {
                if let Ok(providers) = wrappers::try_ivec_to_providers_smallvec(ivec) {
                    info!("Providers in tree: {:?}", providers);
                }
            }

            // Ensure data is flushed to disk
            store.providers.flush().expect("Failed to flush providers");
            store.sled_db.flush().expect("Failed to flush database");
            info!("Flushed database to disk");
        }

        // Give the OS a moment to release the lock
        std::thread::sleep(std::time::Duration::from_millis(100));

        info!("Creating new store instance");
        // Verify persistence by creating a new store instance
        let mut new_store = SledStore::new(local_id, &temp_dir).expect("Creation Erruh");

        // Check providers tree in new store
        if let Ok(Some(ivec)) = new_store.providers.get(&rec.key) {
            if let Ok(providers) = wrappers::try_ivec_to_providers_smallvec(ivec) {
                info!("Providers after reload: {:?}", providers);
            }
        } else {
            info!("No providers found after reload");
        }

        let provided_after_reload: Vec<_> = new_store.provided().collect();
        info!(
            "Provided count after reload: {}",
            provided_after_reload.len()
        );
        assert_eq!(1, provided_after_reload.len());
        assert_eq!(Cow::Borrowed(&rec), provided_after_reload[0]);

        // Test removal
        new_store.remove_provider(&RecordKey::from(key), &local_id);
        assert_eq!(new_store.provided().count(), 0);

        // Cleanup test directory
        drop(new_store);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn update_provider() {
        update_provider_with_number(Some(4));
    }

    pub fn update_provider_with_number(test_number: Option<u32>) {
        init_logging();
        let local_id = PeerId::random();
        let temp_dir = get_test_temp_dir(test_number);
        let mut store = SledStore::new(local_id, &temp_dir).expect("Creation Erruh");
        let key = random_multihash();
        let rec = ProviderRecord::new(key, local_id, Vec::new());

        info!("Store: {store:?}");
        assert!(store.add_provider(rec.clone()).is_ok());
        assert_eq!(vec![rec.clone()], store.providers(&rec.key));

        let mut updated_rec = rec.clone();
        updated_rec.expires = Some(Instant::now());
        assert!(store.add_provider(updated_rec.clone()).is_ok());
        assert_eq!(vec![updated_rec], store.providers(&rec.key));

        // Cleanup test directory
        drop(store);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn update_provided() {
        update_provided_with_number(Some(5));
    }

    pub fn update_provided_with_number(test_number: Option<u32>) {
        init_logging();
        let local_id = PeerId::random();
        let temp_dir = get_test_temp_dir(test_number);
        let mut store = SledStore::new(local_id, &temp_dir).expect("Creation Erruh");
        let key = random_multihash();
        let rec = ProviderRecord::new(key, local_id, Vec::new());

        info!("Record: {rec:?}, Store: {store:?}");
        assert!(store.add_provider(rec.clone()).is_ok());
        assert_eq!(
            vec![Cow::Borrowed(&rec)],
            store.provided().collect::<Vec<_>>()
        );

        let mut updated_rec = rec.clone();
        updated_rec.expires = Some(Instant::now());
        assert!(store.add_provider(updated_rec.clone()).is_ok());
        assert_eq!(
            vec![Cow::Borrowed(&updated_rec)],
            store.provided().collect::<Vec<_>>()
        );

        // Cleanup test directory
        drop(store);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn max_providers_per_key() {
        max_providers_per_key_with_number(Some(6));
    }

    pub fn max_providers_per_key_with_number(test_number: Option<u32>) {
        init_logging();
        let config = SledStoreConfig::default();
        let key = random_multihash();

        let temp_dir = get_test_temp_dir(test_number);
        let mut store = SledStore::new(PeerId::random(), &temp_dir).expect("Creation Erruh");
        let peers = (0..config.max_providers_per_key)
            .map(|_| PeerId::random())
            .collect::<Vec<_>>();
        for peer in peers {
            let rec = ProviderRecord::new(key.clone(), peer, Vec::new());
            assert!(store.add_provider(rec).is_ok());
        }

        // The new provider cannot be added because the key is already saturated.
        let peer = PeerId::random();
        let rec = ProviderRecord::new(key.clone(), peer, Vec::new());
        assert!(store.add_provider(rec.clone()).is_ok());
        assert!(!store.providers(&rec.key).contains(&rec));

        // Cleanup test directory
        drop(store);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn max_provided_keys() {
        max_provided_keys_with_number(Some(7));
    }

    pub fn max_provided_keys_with_number(test_number: Option<u32>) {
        init_logging();
        let temp_dir = get_test_temp_dir(test_number);
        let local_id = PeerId::random();
        let mut store = SledStore::new(local_id, &temp_dir).expect("Creation Erruh");
        for i in 0..store.config.max_provided_keys {
            let key = format!("{:?}, {}", random_multihash(), i);
            let rec = ProviderRecord::new(RecordKey::new(&key), local_id, Vec::new());
            assert!(store.add_provider(rec).is_ok());
        }
        let key = random_multihash();
        let rec = ProviderRecord::new(key, local_id, Vec::new());
        match store.add_provider(rec) {
            Err(Error::MaxProvidedKeys) => {}
            _ => panic!("Unexpected result"),
        }

        // Cleanup test directory
        drop(store);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    // Functions that use tmp0 as default for predictable testing
    pub fn put_get_remove_record_with_tmp0() {
        init_logging();
        let r = Record::new(random_multihash(), "Hello".into());
        let temp_dir = get_test_temp_dir(None);
        let mut store = SledStore::new(PeerId::random(), &temp_dir).expect("Creation Erruh");
        info!("Record: {r:?}\nStore: {store:?}");
        assert!(store.put(r.clone()).is_ok());
        assert_eq!(Some(Cow::Borrowed(&r)), store.get(&r.key));
        store.remove(&r.key);
        assert!(store.get(&r.key).is_none());
    }

    pub fn add_get_remove_provider_with_tmp0() {
        init_logging();
        let local_id = PeerId::random();
        let temp_dir = get_test_temp_dir(None);
        let mut store = SledStore::new(local_id, &temp_dir).expect("Creation Erruh");

        // Add provider
        let r = ProviderRecord::new(random_multihash(), local_id, vec![]);
        assert!(store.add_provider(r.clone()).is_ok());

        // Verify provider is stored
        let providers = store.providers(&r.key);
        assert_eq!(1, providers.len());
        assert!(providers.contains(&r));

        // Verify it's in the provided set
        assert_eq!(1, store.provided().count());
        assert_eq!(Cow::Borrowed(&r), store.provided().next().unwrap());

        // Remove provider
        store.remove_provider(&r.key, &r.provider);
        assert!(store.providers(&r.key).is_empty());
        assert_eq!(0, store.provided().count());
    }

    pub fn provided_with_tmp0() {
        init_logging();
        let local_id = PeerId::random();
        let temp_dir = get_test_temp_dir(None);
        let key = random_multihash();
        let rec = ProviderRecord::new(key.clone(), local_id, Vec::new());
        {
            let mut store = SledStore::new(local_id, &temp_dir).expect("Creation Erruh");
            info!("Initial store created");
            assert!(store.add_provider(rec.clone()).is_ok());
            info!("Added provider record");

            // Verify the record is in the provided set
            let provided: Vec<_> = store.provided().collect();
            info!("Initial provided count: {}", provided.len());
            assert_eq!(1, provided.len());
            assert_eq!(Cow::Borrowed(&rec), provided[0]);

            // Check providers tree directly
            if let Ok(Some(ivec)) = store.providers.get(&rec.key) {
                if let Ok(providers) = wrappers::try_ivec_to_providers_smallvec(ivec) {
                    info!("Providers in tree: {:?}", providers);
                }
            }

            // Ensure data is flushed to disk
            store.providers.flush().expect("Failed to flush providers");
            store.sled_db.flush().expect("Failed to flush database");
            info!("Flushed database to disk");
        }

        // Give the OS a moment to release the lock
        std::thread::sleep(std::time::Duration::from_millis(100));

        info!("Creating new store instance");
        // Verify persistence by creating a new store instance
        let mut new_store = SledStore::new(local_id, &temp_dir).expect("Creation Erruh");

        // Check providers tree in new store
        if let Ok(Some(ivec)) = new_store.providers.get(&rec.key) {
            if let Ok(providers) = wrappers::try_ivec_to_providers_smallvec(ivec) {
                info!("Providers after reload: {:?}", providers);
            }
        } else {
            info!("No providers found after reload");
        }

        let provided_after_reload: Vec<_> = new_store.provided().collect();
        info!(
            "Provided count after reload: {}",
            provided_after_reload.len()
        );
        assert_eq!(1, provided_after_reload.len());
        assert_eq!(Cow::Borrowed(&rec), provided_after_reload[0]);

        // Test removal
        new_store.remove_provider(&RecordKey::from(key), &local_id);
        assert_eq!(new_store.provided().count(), 0);
    }

    pub fn update_provider_with_tmp0() {
        init_logging();
        let local_id = PeerId::random();
        let temp_dir = get_test_temp_dir(None);
        let mut store = SledStore::new(local_id, &temp_dir).expect("Creation Erruh");
        let key = random_multihash();
        let rec = ProviderRecord::new(key, local_id, Vec::new());

        info!("Store: {store:?}");
        assert!(store.add_provider(rec.clone()).is_ok());
        assert_eq!(vec![rec.clone()], store.providers(&rec.key));

        let mut updated_rec = rec.clone();
        updated_rec.expires = Some(Instant::now());
        assert!(store.add_provider(updated_rec.clone()).is_ok());
        assert_eq!(vec![updated_rec], store.providers(&rec.key));
    }

    pub fn update_provided_with_tmp0() {
        init_logging();
        let local_id = PeerId::random();
        let temp_dir = get_test_temp_dir(None);
        let mut store = SledStore::new(local_id, &temp_dir).expect("Creation Erruh");
        let key = random_multihash();
        let rec = ProviderRecord::new(key, local_id, Vec::new());

        info!("Record: {rec:?}, Store: {store:?}");
        assert!(store.add_provider(rec.clone()).is_ok());
        assert_eq!(
            vec![Cow::Borrowed(&rec)],
            store.provided().collect::<Vec<_>>()
        );

        let mut updated_rec = rec.clone();
        updated_rec.expires = Some(Instant::now());
        assert!(store.add_provider(updated_rec.clone()).is_ok());
        assert_eq!(
            vec![Cow::Borrowed(&updated_rec)],
            store.provided().collect::<Vec<_>>()
        );
    }

    pub fn max_providers_per_key_with_tmp0() {
        init_logging();
        let config = SledStoreConfig::default();
        let key = random_multihash();

        let temp_dir = get_test_temp_dir(None);
        let mut store = SledStore::new(PeerId::random(), &temp_dir).expect("Creation Erruh");
        let peers = (0..config.max_providers_per_key)
            .map(|_| PeerId::random())
            .collect::<Vec<_>>();
        for peer in peers {
            let rec = ProviderRecord::new(key.clone(), peer, Vec::new());
            assert!(store.add_provider(rec).is_ok());
        }

        // The new provider cannot be added because the key is already saturated.
        let peer = PeerId::random();
        let rec = ProviderRecord::new(key.clone(), peer, Vec::new());
        assert!(store.add_provider(rec.clone()).is_ok());
        assert!(!store.providers(&rec.key).contains(&rec));
    }

    pub fn max_provided_keys_with_tmp0() {
        init_logging();
        let temp_dir = get_test_temp_dir(None);
        let local_id = PeerId::random();
        let mut store = SledStore::new(local_id, &temp_dir).expect("Creation Erruh");
        for i in 0..store.config.max_provided_keys {
            let key = format!("{:?}, {}", random_multihash(), i);
            let rec = ProviderRecord::new(RecordKey::new(&key), local_id, Vec::new());
            assert!(store.add_provider(rec).is_ok());
        }
        let key = random_multihash();
        let rec = ProviderRecord::new(key, local_id, Vec::new());
        match store.add_provider(rec) {
            Err(Error::MaxProvidedKeys) => {}
            _ => panic!("Unexpected result"),
        }
    }
}
