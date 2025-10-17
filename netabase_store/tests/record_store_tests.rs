#![cfg(all(feature = "libp2p", feature = "native"))]

use libp2p::kad::store::RecordStore;
use libp2p::kad::{ProviderRecord, Record, RecordKey as Key};
use libp2p::{multihash::Multihash, PeerId};
use netabase_macros::netabase_definition_module;
use netabase_store::databases::sled_store::SledStore;
use std::time::Instant;

// Test schema
#[netabase_definition_module(TestDefinition, TestKeys)]
mod test_schema {
    use netabase_deps::{bincode, serde};
    use netabase_macros::NetabaseModel;

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    pub struct TestModel {
        #[primary_key]
        pub id: u64,
        pub name: String,
    }
}

use test_schema::*;

const SHA_256_MH: u64 = 0x12;

fn random_multihash() -> Multihash<64> {
    use rand::Rng;
    Multihash::wrap(SHA_256_MH, &rand::thread_rng().r#gen::<[u8; 32]>()).unwrap()
}

#[test]
fn put_get_remove_record() {
    let mut store = SledStore::<TestDefinition>::temp().unwrap();

    // Create a test model
    let test_model = TestModel {
        id: 1,
        name: "test".to_string(),
    };

    // Create a test key based on TestModel
    let test_model_key = TestModelPrimaryKey(1);
    let key_bytes = bincode::encode_to_vec(&TestKeys::TestModelKey(
        test_schema::TestModelKey::Primary(test_model_key)
    ), bincode::config::standard()).unwrap();
    let record_key = Key::from(key_bytes);

    // Encode the Definition as the record value
    let definition = TestDefinition::TestModel(test_model);
    let value_bytes = bincode::encode_to_vec(&definition, bincode::config::standard()).unwrap();

    let record = Record {
        key: record_key.clone(),
        value: value_bytes,
        publisher: None,
        expires: None,
    };

    assert!(store.put(record.clone()).is_ok());
    assert_eq!(Some(std::borrow::Cow::Owned(record.clone())), store.get(&record_key));
    store.remove(&record_key);
    assert!(store.get(&record_key).is_none());
}

#[test]
fn add_get_remove_provider() {
    let mut store = SledStore::<TestDefinition>::temp().unwrap();

    let key = random_multihash();
    let provider_id = PeerId::random();
    let provider_record = ProviderRecord::new(key.clone(), provider_id, Vec::new());

    assert!(store.add_provider(provider_record.clone()).is_ok());
    assert!(store.providers(&provider_record.key).contains(&provider_record));
    store.remove_provider(&provider_record.key, &provider_id);
    assert!(!store.providers(&provider_record.key).contains(&provider_record));
}

#[test]
fn provided() {
    let id = PeerId::random();
    let mut store = SledStore::<TestDefinition>::temp().unwrap();
    let key = random_multihash();
    let rec = ProviderRecord::new(key, id, Vec::new());
    assert!(store.add_provider(rec.clone()).is_ok());

    let provided: Vec<_> = store.provided().collect();
    assert!(!provided.is_empty());
    assert!(provided.iter().any(|p| **p == rec));

    store.remove_provider(&rec.key, &id);
    assert_eq!(store.provided().count(), 0);
}

#[test]
fn update_provider() {
    let mut store = SledStore::<TestDefinition>::temp().unwrap();
    let key = random_multihash();
    let prv = PeerId::random();
    let mut rec = ProviderRecord::new(key, prv, Vec::new());
    assert!(store.add_provider(rec.clone()).is_ok());
    assert_eq!(vec![rec.clone()], store.providers(&rec.key).to_vec());
    rec.expires = Some(Instant::now());
    assert!(store.add_provider(rec.clone()).is_ok());
    assert_eq!(vec![rec.clone()], store.providers(&rec.key).to_vec());
}

#[test]
fn max_providers_per_key() {
    let mut store = SledStore::<TestDefinition>::temp().unwrap();
    let config = store.record_store_config();
    let key = random_multihash();

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
    assert!(store.add_provider(rec.clone()).is_ok()); // Should silently ignore
    assert!(!store.providers(&rec.key).contains(&rec));
}

#[test]
fn max_provided_keys() {
    let mut store = SledStore::<TestDefinition>::temp().unwrap();
    let config = store.record_store_config();

    for _ in 0..config.max_provided_keys {
        let key = random_multihash();
        let prv = PeerId::random();
        let rec = ProviderRecord::new(key, prv, Vec::new());
        let _ = store.add_provider(rec);
    }
    let key = random_multihash();
    let prv = PeerId::random();
    let rec = ProviderRecord::new(key, prv, Vec::new());
    match store.add_provider(rec) {
        Err(libp2p::kad::store::Error::MaxProvidedKeys) => {}
        _ => panic!("Expected MaxProvidedKeys error"),
    }
}

#[test]
fn records_iterator() {
    let mut store = SledStore::<TestDefinition>::temp().unwrap();

    let records: Vec<Record> = (0..10)
        .map(|i| {
            // Create a test model
            let test_model = TestModel {
                id: i,
                name: format!("test{}", i),
            };

            // Create proper keys using TestModelPrimaryKey
            let test_model_key = TestModelPrimaryKey(i);
            let key_bytes = bincode::encode_to_vec(&TestKeys::TestModelKey(
                test_schema::TestModelKey::Primary(test_model_key)
            ), bincode::config::standard()).unwrap();

            // Encode the Definition as the record value
            let definition = TestDefinition::TestModel(test_model);
            let value_bytes = bincode::encode_to_vec(&definition, bincode::config::standard()).unwrap();

            Record {
                key: Key::from(key_bytes),
                value: value_bytes,
                publisher: None,
                expires: None,
            }
        })
        .collect();

    for r in &records {
        store.put(r.clone()).unwrap();
    }

    let stored: Vec<_> = store.records().collect();
    assert_eq!(stored.len(), records.len());

    for r in &records {
        assert!(stored.iter().any(|s| s.key == r.key));
    }
}
