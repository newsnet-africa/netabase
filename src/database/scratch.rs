use std::{
    borrow::Cow,
    collections::{HashMap, HashSet, hash_map},
};

use libp2p::{
    PeerId,
    kad::{
        K_VALUE, ProviderRecord, Record, RecordKey as Key,
        kbucket::KBucketKey,
        store::{Error, RecordStore, Result},
    },
};
use smallvec::SmallVec;

pub struct MemoryStore {
    local_key: KBucketKey<PeerId>,
    config: MemoryStoreConfig,
    records: HashMap<Key, Record>,
    providers: HashMap<Key, SmallVec<[ProviderRecord; K_VALUE.get()]>>,
    provided: HashSet<ProviderRecord>,
}

#[derive(Debug, Clone)]
pub struct MemoryStoreConfig {
    pub max_records: usize,
    pub max_value_bytes: usize,
    pub max_providers_per_key: usize,
    pub max_provided_keys: usize,
}

impl Default for MemoryStoreConfig {
    fn default() -> Self {
        Self {
            max_records: 1024,
            max_value_bytes: 65 * 1024,
            max_providers_per_key: K_VALUE.get(),
            max_provided_keys: 1024,
        }
    }
}

impl MemoryStore {
    pub fn new(local_id: PeerId) -> Self {
        Self::with_config(local_id, Default::default())
    }

    pub fn with_config(local_id: PeerId, config: MemoryStoreConfig) -> Self {
        MemoryStore {
            local_key: KBucketKey::from(local_id),
            config,
            records: HashMap::default(),
            providers: HashMap::default(),
            provided: HashSet::default(),
        }
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Key, &mut Record) -> bool,
    {
        self.records.retain(f);
    }
}

impl RecordStore for MemoryStore {
    type RecordsIter<'a> = std::iter::Map<
        std::collections::hash_map::Values<'a, Key, Record>,
        fn(&'a Record) -> Cow<'a, Record>,
    >;

    type ProvidedIter<'a> = std::iter::Map<
        std::collections::hash_set::Iter<'a, ProviderRecord>,
        fn(&'a ProviderRecord) -> Cow<'a, ProviderRecord>,
    >;

    fn get(&self, k: &Key) -> Option<Cow<'_, Record>> {
        self.records.get(k).map(Cow::Borrowed)
    }

    fn put(&mut self, r: Record) -> Result<()> {
        Ok(())
    }

    fn remove(&mut self, k: &Key) {
        self.records.remove(k);
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        self.records.values().map(Cow::Borrowed)
    }

    fn add_provider(&mut self, record: ProviderRecord) -> Result<()> {
        let num_keys = self.providers.len();

        let providers = match self.providers.entry(record.key.clone()) {
            e @ hash_map::Entry::Occupied(_) => e,
            e @ hash_map::Entry::Vacant(_) => {
                if self.config.max_provided_keys == num_keys {
                    return Err(Error::MaxProvidedKeys);
                }
                e
            }
        }
        .or_default();

        for p in providers.iter_mut() {
            if p.provider == record.provider {
                if self.local_key.preimage() == &record.provider {
                    self.provided.remove(p);
                    self.provided.insert(record.clone());
                }
                *p = record;
                return Ok(());
            }
        }

        if providers.len() == self.config.max_providers_per_key {
            return Ok(());
        }

        if self.local_key.preimage() == &record.provider {
            self.provided.insert(record.clone());
        }

        providers.push(record);
        Ok(())
    }

    fn providers(&self, key: &Key) -> Vec<ProviderRecord> {
        self.providers
            .get(key)
            .map_or_else(Vec::new, |ps| ps.to_vec())
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        self.provided.iter().map(Cow::Borrowed)
    }

    fn remove_provider(&mut self, k: &Key, p: &PeerId) {
        if let hash_map::Entry::Occupied(mut e) = self.providers.entry(k.clone()) {
            let providers = e.get_mut();
            if let Some(i) = providers.iter().position(|r| &r.provider == p) {
                let provider_record = providers.remove(i);
                if &provider_record.provider == self.local_key.preimage() {
                    self.provided.remove(&provider_record);
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
    use libp2p::multihash::{Code, MultihashDigest};

    fn random_multihash() -> libp2p::multihash::Multihash<64> {
        Code::Sha2_256.digest(&rand::random::<[u8; 32]>())
    }

    #[test]
    fn put_get_remove_record() {
        let mut store = MemoryStore::new(PeerId::random());
        let record = Record::new(random_multihash(), "Hello World".into());

        assert_eq!(None, store.get(&record.key));
        assert!(store.put(record.clone()).is_ok());
        assert_eq!(Some(Cow::Borrowed(&record)), store.get(&record.key));
        store.remove(&record.key);
        assert_eq!(None, store.get(&record.key));
    }

    #[test]
    fn add_get_remove_provider() {
        let local_id = PeerId::random();
        let mut store = MemoryStore::new(local_id);
        let record = ProviderRecord::new(random_multihash(), local_id, Vec::new());

        assert!(store.add_provider(record.clone()).is_ok());
        assert_eq!(vec![record.clone()], store.providers(&record.key));
        assert_eq!(1, store.provided().count());

        store.remove_provider(&record.key, &record.provider);
        assert!(store.providers(&record.key).is_empty());
        assert_eq!(0, store.provided().count());
    }

    #[test]
    fn provided() {
        let local_id = PeerId::random();
        let mut store = MemoryStore::new(local_id);
        let key = random_multihash();
        let rec = ProviderRecord::new(key.clone(), local_id, Vec::new());

        assert!(store.add_provider(rec.clone()).is_ok());
        assert_eq!(
            vec![Cow::Borrowed(&rec)],
            store.provided().collect::<Vec<_>>()
        );

        store.remove_provider(&RecordKey::from(key), &local_id);
        assert_eq!(store.provided().count(), 0);
    }

    #[test]
    fn update_provider() {
        let local_id = PeerId::random();
        let mut store = MemoryStore::new(local_id);
        let key = random_multihash();
        let rec = ProviderRecord::new(key, local_id, Vec::new());

        assert!(store.add_provider(rec.clone()).is_ok());
        assert_eq!(vec![rec.clone()], store.providers(&rec.key));

        let mut updated_rec = rec.clone();
        updated_rec.expires = Some(std::time::Instant::now());
        assert!(store.add_provider(updated_rec.clone()).is_ok());
        assert_eq!(vec![updated_rec], store.providers(&rec.key));
    }

    #[test]
    fn max_providers_per_key() {
        let mut store = MemoryStore::new(PeerId::random());
        let key = random_multihash();
        let peers = (0..store.config.max_providers_per_key)
            .map(|_| PeerId::random())
            .collect::<Vec<_>>();

        for peer in peers {
            let rec = ProviderRecord::new(key.clone(), peer, Vec::new());
            assert!(store.add_provider(rec).is_ok());
        }

        let peer = PeerId::random();
        let rec = ProviderRecord::new(key.preimage().clone(), peer, Vec::new());
        assert!(store.add_provider(rec.clone()).is_ok());
        assert!(!store.providers(&rec.key).contains(&rec));
    }

    #[test]
    fn max_provided_keys() {
        let local_id = PeerId::random();
        let mut store = MemoryStore::new(local_id);

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
