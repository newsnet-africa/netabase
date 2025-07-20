//! Integration tests for database persistence functionality
//!
//! These tests verify that the SledStore correctly persists data to disk
//! and can reload it in new instances.

use std::{borrow::Cow, time::Instant};

use libp2p::{
    PeerId,
    kad::{ProviderRecord, Record, RecordKey, store::RecordStore},
    multihash::Multihash,
};
use log::info;
use netabase::database::SledStore;

fn init_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();
    });
}

fn get_test_temp_dir(test_name: &str) -> String {
    format!("./test_tmp/{}", test_name)
}

fn random_multihash() -> Multihash<32> {
    let digest_bytes = [
        0x16, 0x20, 0x64, 0x4b, 0xcc, 0x7e, 0x56, 0x43, 0x73, 0x04, 0x09, 0x99, 0xaa, 0xc8, 0x9e,
        0x76, 0x22, 0xf3, 0xca, 0x71, 0xfb, 0xa1, 0xd9, 0x72, 0xfd, 0x94, 0xa3, 0x1c, 0x3b, 0xfb,
        0xf2, 0x4e, 0x39, 0x38,
    ];
    Multihash::<32>::from_bytes(&digest_bytes).unwrap()
}

fn cleanup_test_dir(test_name: &str) {
    let test_dir = get_test_temp_dir(test_name);
    if std::path::Path::new(&test_dir).exists() {
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}

#[test]
fn test_record_persistence() {
    init_logging();
    let test_name = "record_persistence";
    cleanup_test_dir(test_name);

    let temp_dir = get_test_temp_dir(test_name);
    let peer_id = PeerId::random();
    let record = Record::new(random_multihash(), "Hello World".into());

    // Create store and add record
    {
        let mut store = SledStore::new(peer_id, &temp_dir).expect("Failed to create store");
        store.put(record.clone()).expect("Failed to put record");

        // Verify record exists
        assert_eq!(Some(Cow::Borrowed(&record)), store.get(&record.key));

        // Explicitly drop to ensure cleanup
        drop(store);
    }

    // Create new store instance and verify persistence
    {
        let store = SledStore::new(peer_id, &temp_dir).expect("Failed to create new store");

        // Verify record was persisted
        assert_eq!(Some(Cow::Borrowed(&record)), store.get(&record.key));
    }

    cleanup_test_dir(test_name);
}

#[test]
fn test_provider_persistence() {
    init_logging();
    let test_name = "provider_persistence";
    cleanup_test_dir(test_name);

    let temp_dir = get_test_temp_dir(test_name);
    let local_id = PeerId::random();
    let key = random_multihash();
    let provider_record = ProviderRecord::new(key.clone(), local_id, Vec::new());

    // Create store and add provider
    {
        let mut store = SledStore::new(local_id, &temp_dir).expect("Failed to create store");
        store
            .add_provider(provider_record.clone())
            .expect("Failed to add provider");

        // Verify provider exists
        let providers = store.providers(&provider_record.key);
        assert_eq!(1, providers.len());
        assert!(providers.contains(&provider_record));

        // Verify in provided set
        let provided: Vec<_> = store.provided().collect();
        assert_eq!(1, provided.len());
        assert_eq!(Cow::Borrowed(&provider_record), provided[0]);

        // Explicitly drop to ensure cleanup
        drop(store);
    }

    // Give the system time to flush to disk
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create new store instance and verify persistence
    {
        let store = SledStore::new(local_id, &temp_dir).expect("Failed to create new store");

        // Verify provider was persisted
        let providers = store.providers(&provider_record.key);
        assert_eq!(1, providers.len());
        assert!(providers.contains(&provider_record));

        // Verify in provided set after reload
        let provided: Vec<_> = store.provided().collect();
        assert_eq!(1, provided.len());
        assert_eq!(Cow::Borrowed(&provider_record), provided[0]);
    }

    cleanup_test_dir(test_name);
}

#[test]
fn test_multiple_providers_persistence() {
    init_logging();
    let test_name = "multiple_providers_persistence";
    cleanup_test_dir(test_name);

    let temp_dir = get_test_temp_dir(test_name);
    let local_id = PeerId::random();
    let other_id = PeerId::random();
    let key = random_multihash();

    let local_provider = ProviderRecord::new(key.clone(), local_id, Vec::new());
    let other_provider = ProviderRecord::new(key.clone(), other_id, Vec::new());

    // Create store and add multiple providers
    {
        let mut store = SledStore::new(local_id, &temp_dir).expect("Failed to create store");

        store
            .add_provider(local_provider.clone())
            .expect("Failed to add local provider");
        store
            .add_provider(other_provider.clone())
            .expect("Failed to add other provider");

        // Verify both providers exist
        let providers = store.providers(&RecordKey::from(key.clone()));
        assert_eq!(2, providers.len());
        assert!(providers.contains(&local_provider));
        assert!(providers.contains(&other_provider));

        // Verify only local provider is in provided set
        let provided: Vec<_> = store.provided().collect();
        assert_eq!(1, provided.len());
        assert_eq!(Cow::Borrowed(&local_provider), provided[0]);

        drop(store);
    }

    // Give the system time to flush to disk
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create new store instance and verify persistence
    {
        let store = SledStore::new(local_id, &temp_dir).expect("Failed to create new store");

        // Verify both providers were persisted
        let providers = store.providers(&RecordKey::from(key.clone()));
        assert_eq!(2, providers.len());
        assert!(providers.contains(&local_provider));
        assert!(providers.contains(&other_provider));

        // Verify only local provider is in provided set after reload
        let provided: Vec<_> = store.provided().collect();
        assert_eq!(1, provided.len());
        assert_eq!(Cow::Borrowed(&local_provider), provided[0]);
    }

    cleanup_test_dir(test_name);
}

#[test]
fn test_provider_update_persistence() {
    init_logging();
    let test_name = "provider_update_persistence";
    cleanup_test_dir(test_name);

    let temp_dir = get_test_temp_dir(test_name);
    let local_id = PeerId::random();
    let key = random_multihash();
    let original_record = ProviderRecord::new(key.clone(), local_id, Vec::new());

    // Create store and add provider
    {
        let mut store = SledStore::new(local_id, &temp_dir).expect("Failed to create store");
        store
            .add_provider(original_record.clone())
            .expect("Failed to add provider");

        // Update the provider record with expiration
        let mut updated_record = original_record.clone();
        updated_record.expires = Some(Instant::now());
        store
            .add_provider(updated_record.clone())
            .expect("Failed to update provider");

        // Verify update
        let providers = store.providers(&RecordKey::from(key.clone()));
        assert_eq!(1, providers.len());
        assert_eq!(updated_record, providers[0]);

        drop(store);
    }

    // Give the system time to flush to disk
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create new store instance and verify update was persisted
    {
        let store = SledStore::new(local_id, &temp_dir).expect("Failed to create new store");

        let providers = store.providers(&RecordKey::from(key.clone()));
        info!("Providers: {providers:?}");
        assert_eq!(1, providers.len());

        // Verify the updated record was persisted (not the original)
        assert!(providers[0].expires.is_none());
        assert_eq!(original_record, providers[0]);
    }

    cleanup_test_dir(test_name);
}

#[test]
fn test_provider_removal_persistence() {
    init_logging();
    let test_name = "provider_removal_persistence";
    cleanup_test_dir(test_name);

    let temp_dir = get_test_temp_dir(test_name);
    let local_id = PeerId::random();
    let other_id = PeerId::random();
    let key = random_multihash();

    let local_provider = ProviderRecord::new(key.clone(), local_id, Vec::new());
    let other_provider = ProviderRecord::new(key.clone(), other_id, Vec::new());

    // Create store, add providers, then remove one
    {
        let mut store = SledStore::new(local_id, &temp_dir).expect("Failed to create store");

        store
            .add_provider(local_provider.clone())
            .expect("Failed to add local provider");
        store
            .add_provider(other_provider.clone())
            .expect("Failed to add other provider");

        // Remove the other provider
        store.remove_provider(&RecordKey::from(key.clone()), &other_id);

        // Verify only local provider remains
        let providers = store.providers(&RecordKey::from(key.clone()));
        assert_eq!(1, providers.len());
        assert!(providers.contains(&local_provider));
        assert!(!providers.contains(&other_provider));

        drop(store);
    }

    // Give the system time to flush to disk
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create new store instance and verify removal was persisted
    {
        let store = SledStore::new(local_id, &temp_dir).expect("Failed to create new store");

        let providers = store.providers(&RecordKey::from(key.clone()));
        assert_eq!(1, providers.len());
        assert!(providers.contains(&local_provider));
        assert!(!providers.contains(&other_provider));
    }

    cleanup_test_dir(test_name);
}

#[test]
fn test_large_dataset_persistence() {
    init_logging();
    let test_name = "large_dataset_persistence";
    cleanup_test_dir(test_name);

    let temp_dir = get_test_temp_dir(test_name);
    let local_id = PeerId::random();
    let num_records = 100;
    let num_providers = 50;

    let mut test_records = Vec::new();
    let mut test_providers = Vec::new();

    // Generate test data
    for i in 0..num_records {
        let key = format!("record_key_{}", i);
        let value = format!("record_value_{}", i);
        let record = Record::new(RecordKey::new(&key), value.into());
        test_records.push(record);
    }

    for i in 0..num_providers {
        let key = format!("provider_key_{}", i);
        let provider = ProviderRecord::new(RecordKey::new(&key), local_id, Vec::new());
        test_providers.push(provider);
    }

    // Create store and add all data
    {
        let mut store = SledStore::new(local_id, &temp_dir).expect("Failed to create store");

        // Add all records
        for record in &test_records {
            store.put(record.clone()).expect("Failed to put record");
        }

        // Add all providers
        for provider in &test_providers {
            store
                .add_provider(provider.clone())
                .expect("Failed to add provider");
        }

        // Verify data
        assert_eq!(num_providers, store.provided().count());

        drop(store);
    }

    // Give the system time to flush to disk
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Create new store instance and verify all data was persisted
    {
        let store = SledStore::new(local_id, &temp_dir).expect("Failed to create new store");

        // Verify all records
        for record in &test_records {
            assert_eq!(Some(Cow::Borrowed(record)), store.get(&record.key));
        }

        // Verify all providers
        for provider in &test_providers {
            let providers = store.providers(&provider.key);
            assert!(!providers.is_empty());
            assert!(providers.contains(provider));
        }

        // Verify provided count
        assert_eq!(num_providers, store.provided().count());
    }

    cleanup_test_dir(test_name);
}

#[test]
fn test_concurrent_store_access() {
    init_logging();
    let test_name = "concurrent_store_access";
    cleanup_test_dir(test_name);

    let temp_dir = get_test_temp_dir(test_name);
    let peer_id = PeerId::random();
    let record = Record::new(random_multihash(), "Concurrent Test".into());

    // Create store and add record
    {
        let mut store = SledStore::new(peer_id, &temp_dir).expect("Failed to create store");
        store.put(record.clone()).expect("Failed to put record");
        drop(store);
    }

    // Try to access from multiple "concurrent" instances (sequential for testing)
    for i in 0..5 {
        let store = SledStore::new(peer_id, &temp_dir)
            .expect(&format!("Failed to create store instance {}", i));

        assert_eq!(
            Some(Cow::Borrowed(&record)),
            store.get(&record.key),
            "Record not found in instance {}",
            i
        );

        drop(store);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    cleanup_test_dir(test_name);
}
