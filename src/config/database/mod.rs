pub mod wrappers;

use std::{borrow::Cow, io::Read, iter};

use libp2p::kad::{
    ProviderRecord, Record,
    store::{Error, RecordStore},
};
use sled::{Db, IVec};
use wrappers::{ProviderRecordWrapper, RecordWrapper};

pub struct LocalDatabase {
    sled: Db,
    max_providers: usize,
}

impl LocalDatabase {
    pub fn new(test: bool, max_providers: usize) -> Self {
        let sled = if test {
            sled::Config::new()
                .temporary(true)
                .open()
                .expect("Could not open temp tree")
        } else {
            sled::Config::default()
                .path("./data/kademlia.db")
                .open()
                .expect("Could not open sled tree")
        };
        Self {
            sled,
            max_providers,
        }
    }
}

impl RecordStore for LocalDatabase {
    type RecordsIter<'a> = iter::Map<sled::Iter, fn(sled::Result<(IVec, IVec)>) -> Cow<'a, Record>>;

    type ProvidedIter<'a> =
        iter::Map<sled::Iter, fn(sled::Result<(IVec, IVec)>) -> Cow<'a, ProviderRecord>>;

    fn get(&self, k: &libp2p::kad::RecordKey) -> Option<std::borrow::Cow<'_, libp2p::kad::Record>> {
        let records = self.sled.open_tree("RECORDS").expect("Could not open tree");
        match records.get(k) {
            Ok(item) => item.map(|ivec| {
                Cow::Owned(Record::from(
                    bincode::deserialize::<RecordWrapper>(ivec.to_vec().as_slice())
                        .expect("Could not deserialise Wrapper"),
                ))
            }),
            Err(_) => None,
        }
    }

    fn put(&mut self, r: libp2p::kad::Record) -> libp2p::kad::store::Result<()> {
        let records = self.sled.open_tree("RECORDS").expect("Could not open tree");
        let key = &r.key;
        records.insert(key.clone(), RecordWrapper::from(r));
        Ok(())
    }

    fn remove(&mut self, k: &libp2p::kad::RecordKey) {
        let records = self.sled.open_tree("RECORDS").expect("Could not open tree");
        records.remove(k).expect("Error removing item from Records");
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        self.sled
            .open_tree("RECORDS")
            .expect("Could not open tree")
            .iter()
            .map(|item| {
                let (_, item) = item.expect("Iteration error");
                Cow::Owned(Record::from(RecordWrapper::from(item)))
            })
    }

    fn add_provider(
        &mut self,
        record: libp2p::kad::ProviderRecord,
    ) -> libp2p::kad::store::Result<()> {
        let providers = self
            .sled
            .open_tree("PROVIDERS")
            .expect("Could not open tree");
        let key = record.key.clone();
        let update_providers: Box<dyn FnMut(Option<&[u8]>) -> Option<Vec<u8>>> =
            Box::new(|old: Option<&[u8]>| -> Option<Vec<u8>> {
                match old {
                    Some(bytes) => {
                        let mut vect = bincode::deserialize::<Vec<ProviderRecordWrapper>>(bytes)
                            .expect("Could not deserialise vect");
                        if vect.len().le(&self.max_providers) {
                            vect.push(ProviderRecordWrapper::from(record.clone()));
                            Some(bincode::serialize(&vect).expect("Could not serialise Vect"))
                        } else {
                            None
                        }
                    }
                    None => {
                        let mut vect = vec![];
                        if vect.len().le(&self.max_providers) {
                            vect.push(ProviderRecordWrapper::from(record.clone()));
                            Some(bincode::serialize(&vect).expect("Could not serialise Vect"))
                        } else {
                            None
                        }
                    }
                }
            });

        enum UpdateSuccess {
            Success,
            Failed,
        }

        let update_success = match providers.update_and_fetch(key, update_providers) {
            Ok(val) => match val {
                Some(ivect) => {
                    let vec = ivect.to_vec();
                    let sliced = vec.as_slice();
                    let providers = bincode::deserialize::<Vec<ProviderRecordWrapper>>(sliced)
                        .expect("Cannot deserialise vector");
                    match providers.last() {
                        Some(item) => {
                            if item.eq(&ProviderRecordWrapper::from(record)) {
                                UpdateSuccess::Success
                            } else {
                                UpdateSuccess::Failed
                            }
                        }
                        None => {
                            UpdateSuccess::Failed //TODO: Handle this better
                        }
                    }
                }
                None => {
                    UpdateSuccess::Failed //TODO: Handle this better
                }
            },
            Err(_) => UpdateSuccess::Failed,
        };

        match update_success {
            UpdateSuccess::Success => Ok(()),
            UpdateSuccess::Failed => libp2p::kad::store::Result::Err(Error::ValueTooLarge),
        }

        // Ok(())
    }

    fn providers(&self, key: &libp2p::kad::RecordKey) -> Vec<libp2p::kad::ProviderRecord> {
        let providers = self
            .sled
            .open_tree("PROVIDERS")
            .expect("Could not open tree");
        match providers.get(key) {
            Ok(vect) => match vect {
                Some(ivect) => {
                    let deserialize_result = match bincode::deserialize::<Vec<ProviderRecordWrapper>>(
                        ivect.to_vec().as_slice(),
                    ) {
                        Ok(vect) => vect,
                        Err(_) => vec![],
                    };

                    deserialize_result
                        .iter()
                        .map(|wrapper| ProviderRecord::from(wrapper.clone()))
                        .collect()
                }
                None => vec![],
            },
            Err(_) => vec![],
        }
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        self.sled
            .open_tree("PROVIDERS")
            .expect("Could not open tree")
            .iter()
            .map(|item| {
                let (_, item) = item.expect("Iteration error");
                Cow::Owned(ProviderRecord::from(ProviderRecordWrapper::from(item)))
            })
    }

    fn remove_provider(&mut self, k: &libp2p::kad::RecordKey, p: &libp2p::PeerId) {
        let providers = self
            .sled
            .open_tree("PROVIDERS")
            .expect("Could not open tree");
        let update_providers: Box<dyn FnMut(Option<&[u8]>) -> Option<Vec<u8>>> =
            Box::new(|old: Option<&[u8]>| -> Option<Vec<u8>> {
                match old {
                    Some(bytes) => {
                        let mut vect = bincode::deserialize::<Vec<ProviderRecordWrapper>>(bytes)
                            .expect("Could not deserialise vect");
                        Some(
                            bincode::serialize(&vect.iter().find(|item| item.provider.eq(p)))
                                .expect("Could not serialise Wrapper"),
                        )
                    }
                    None => None,
                }
            });

        enum UpdateSuccess {
            Success,
            Failed,
        }

        let _ = match providers.update_and_fetch(k, update_providers) {
            Ok(val) => match val {
                Some(ivect) => {
                    let vec = ivect.to_vec();
                    let sliced = vec.as_slice();
                    let providers = bincode::deserialize::<Vec<ProviderRecordWrapper>>(sliced);
                    match providers {
                        Ok(vect) => match vect.iter().find(|item| item.provider.eq(p)) {
                            Some(_) => UpdateSuccess::Failed,
                            None => UpdateSuccess::Success,
                        },
                        Err(_) => UpdateSuccess::Failed,
                    }
                }
                None => {
                    UpdateSuccess::Failed //TODO: Handle this better
                }
            },
            Err(_) => UpdateSuccess::Failed,
        };
    }
}

#[cfg(test)]
mod local_database {
    use super::*;
    use libp2p::{
        PeerId,
        kad::{ProviderRecord, Record},
    };
    use log::info;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_new() {
        init_logger();
        info!("Starting test_new");
        let db = LocalDatabase::new(true, 10);
        assert_eq!(db.max_providers, 10);
        info!("Finished test_new");
    }

    #[test]
    fn test_put_and_get_record() {
        init_logger();
        info!("Starting test_put_and_get_record");
        let mut db = LocalDatabase::new(true, 10);
        let record = Record {
            key: b"key1".to_vec().into(),
            value: b"value1".to_vec(),
            publisher: None,
            expires: None,
        };
        db.put(record.clone()).unwrap();
        let retrieved = db.get(&record.key).unwrap();
        assert_eq!(retrieved.into_owned(), record);
        info!("Finished test_put_and_get_record");
    }

    #[test]
    fn test_remove_record() {
        init_logger();
        info!("Starting test_remove_record");
        let mut db = LocalDatabase::new(true, 10);
        let record = Record {
            key: b"key1".to_vec().into(),
            value: b"value1".to_vec(),
            publisher: None,
            expires: None,
        };
        db.put(record.clone()).unwrap();
        db.remove(&record.key);
        let rec = db.get(&record.key);
        println!("Record Get: {:?}", rec);
        assert!(db.get(&record.key).is_none());
        info!("Finished test_remove_record");
    }

    #[test]
    fn test_add_and_get_provider() {
        init_logger();
        info!("Starting test_add_and_get_provider");
        let mut db = LocalDatabase::new(true, 10);
        let provider = ProviderRecord {
            key: b"key1".to_vec().into(),
            provider: PeerId::random(),
            expires: None,
            addresses: vec![],
        };
        db.add_provider(provider.clone()).unwrap();
        let providers = db.providers(&provider.key);
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0], provider);
        info!("Finished test_add_and_get_provider");
    }

    #[test]
    fn test_remove_provider() {
        init_logger();
        info!("Starting test_remove_provider");
        let mut db = LocalDatabase::new(true, 10);
        let provider = ProviderRecord {
            key: b"key1".to_vec().into(),
            provider: PeerId::random(),
            expires: None,
            addresses: vec![],
        };
        db.add_provider(provider.clone()).unwrap();
        db.remove_provider(&provider.key, &provider.provider);
        let providers = db.providers(&provider.key);
        println!("{providers:?}");
        assert!(!providers.iter().any(|prov| { prov.eq(&provider) }));
        info!("Finished test_remove_provider");
    }
}
