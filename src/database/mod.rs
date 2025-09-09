use std::{borrow::Cow, collections::HashSet, iter, path::Path};

use libp2p::{
    PeerId,
    kad::{K_VALUE, ProviderRecord, Record, store::RecordStore},
};

use sled::{Db, IVec, Tree};
use smallvec::SmallVec;

use crate::database::wrappers::{try_ivec_to_record, try_record_to_ivec};

pub mod wrappers;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
        let sled_db = sled::open(&path)?;
        let records = sled_db.open_tree("Records")?;
        let providers = sled_db.open_tree("Providers")?;
        let mut provided = HashSet::new();

        for res in providers.iter() {
            if let Ok((key, value)) = res {
                if key.as_ref() == b"Provided" {
                    continue;
                }
                if let Ok(provider_records) = wrappers::try_ivec_to_providers_smallvec(value) {
                    for record in provider_records {
                        if record.provider == local_id {
                            provided.insert(record);
                        }
                    }
                }
            }
        }

        let store = Self {
            local_key: local_id,
            config,
            records,
            sled_db,
            providers,
            provided,
            path: path.as_ref().to_path_buf(),
        };

        Ok(store)
    }
}

impl Drop for SledStore {
    fn drop(&mut self) {
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
        match try_record_to_ivec(r.clone()) {
            Ok(record_ivec) => {
                match self.records.insert(r.key.clone(), record_ivec) {
                    Ok(_) => Ok(()),
                    Err(_) => Ok(()), // Silently handle the error by returning Ok
                }
            }
            Err(_) => Ok(()),
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

        if let Some(pos) = providers.iter().position(|p| p.provider == record.provider) {
            if self.local_key == record.provider {
                self.provided.remove(&providers[pos]);
                self.provided.insert(record.clone());
            }
            providers[pos] = record;
        } else {
            if providers.len() < self.config.max_providers_per_key {
                if self.local_key == record.provider {
                    self.provided.insert(record.clone());
                }
                providers.push(record);
            }
        }

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
