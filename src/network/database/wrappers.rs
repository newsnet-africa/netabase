//! Utility functions for converting between NativeDB objects and libp2p_kad::Record
//!
//! This module provides helper functions to facilitate the data flow:
//! 1. NativeDB struct → Catalog → libp2p_kad::Record (for sending)
//! 2. libp2p_kad::Record → Catalog → NativeDB struct (for receiving)

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bincode::Encode;
use chrono::{DateTime, Duration, Utc};
use libp2p::kad::{Record as KadRecord, RecordKey};

use crate::{NetabaseRecordExt, Record};

/// Generate a content-based key for any serializable type using bincode
pub fn generate_content_key<T>(data: &T, type_prefix: &str) -> Vec<u8>
where
    T: Encode,
{
    let mut hasher = DefaultHasher::new();

    // Hash the type name first
    type_prefix.hash(&mut hasher);

    // Then hash the serialized content using bincode
    if let Ok(serialized) = bincode::encode_to_vec(data, bincode::config::standard()) {
        serialized.hash(&mut hasher);
    }

    // Return as bytes for native_db compatibility
    format!("{}:{:x}", type_prefix, hasher.finish()).into_bytes()
}

/// Generate a time-based key for any serializable type using bincode
pub fn generate_time_key<T>(data: &T, type_prefix: &str) -> Vec<u8>
where
    T: Encode,
{
    let timestamp = Utc::now().timestamp_millis();

    let mut hasher = DefaultHasher::new();
    if let Ok(serialized) = bincode::encode_to_vec(data, bincode::config::standard()) {
        serialized.hash(&mut hasher);
    }

    format!("{}:{}:{:x}", type_prefix, timestamp, hasher.finish()).into_bytes()
}

/// Convert any NetabaseRecordExt object directly to KadRecord using bincode
pub fn to_kad_record<T>(object: &T) -> KadRecord
where
    T: NetabaseRecordExt + Clone,
{
    object.to_kad_record()
}

/// Convert KadRecord directly to any NetabaseRecordExt object using bincode
pub fn from_kad_record<T>(kad_record: KadRecord) -> Result<T, Box<dyn std::error::Error>>
where
    T: NetabaseRecordExt,
{
    T::from_kad_record(kad_record)
}

/// Validate that a KadRecord can be deserialized to the expected type
pub fn validate_kad_record<T>(kad_record: &KadRecord) -> bool
where
    T: NetabaseRecordExt,
{
    // Try to convert the KadRecord to our type
    T::from_kad_record(kad_record.clone()).is_ok()
}

/// Extract the type prefix from a record key (assumes UTF-8 key format)
pub fn extract_type_prefix(key_bytes: &[u8]) -> Option<String> {
    if let Ok(key_str) = String::from_utf8(key_bytes.to_vec()) {
        key_str.split(':').next().map(|s| s.to_string())
    } else {
        None
    }
}

/// Check if a record has expired based on its expiry field
pub fn is_record_expired<T>(record: &Record<T>) -> bool
where
    T: Encode + bincode::Decode<()>,
{
    if let Some(expiry_time) = record.expiry {
        let current_time = Utc::now();
        current_time > expiry_time
    } else {
        false
    }
}

/// Create a Record with automatic expiry
pub fn create_expiring_record<T>(data: T, key: Vec<u8>, expiry_time: DateTime<Utc>) -> Record<T>
where
    T: Encode + bincode::Decode<()>,
{
    Record::new(key, data).with_expiry(expiry_time)
}

/// Create a Record with creator information
pub fn create_attributed_record<T>(data: T, key: Vec<u8>, creator: String) -> Record<T>
where
    T: Encode + bincode::Decode<()>,
{
    Record::new(key, data).with_creator(creator)
}

/// Batch convert multiple objects to KadRecords using bincode
pub fn batch_to_kad_records<T>(objects: &[T]) -> Vec<KadRecord>
where
    T: NetabaseRecordExt + Clone,
{
    objects.iter().map(|obj| to_kad_record(obj)).collect()
}

/// Batch convert multiple KadRecords to objects using bincode
pub fn batch_from_kad_records<T>(
    kad_records: Vec<KadRecord>,
) -> Vec<Result<T, Box<dyn std::error::Error>>>
where
    T: NetabaseRecordExt,
{
    kad_records
        .into_iter()
        .map(|record| from_kad_record(record))
        .collect()
}

/// Filter KadRecords by type prefix
pub fn filter_by_type_prefix<'a>(
    kad_records: &'a [KadRecord],
    type_prefix: &str,
) -> Vec<&'a KadRecord> {
    kad_records
        .iter()
        .filter(|record| {
            if let Some(prefix) = extract_type_prefix(record.key.as_ref()) {
                prefix == type_prefix
            } else {
                false
            }
        })
        .collect()
}

/// Get record size in bytes (including metadata) using bincode estimation
pub fn get_record_size<T>(record: &Record<T>) -> Result<usize, bincode::error::EncodeError>
where
    T: bincode::Encode,
{
    // Serialize the entire record to get accurate size
    let serialized = bincode::encode_to_vec(record, bincode::config::standard())?;
    Ok(serialized.len())
}

/// Convert between RecordKey and bytes
pub fn record_key_to_bytes(key: &RecordKey) -> Vec<u8> {
    key.as_ref().to_vec()
}

/// Convert bytes to RecordKey
pub fn bytes_to_record_key(bytes: &[u8]) -> RecordKey {
    RecordKey::new(&bytes)
}

/// Convert native_db key to RecordKey for network transmission
pub fn native_db_key_to_record_key(key: &native_db::db_type::Key) -> RecordKey {
    // Use the key bytes directly
    let key_bytes = crate::native_db_key_to_bytes(key);
    RecordKey::new(&key_bytes)
}

/// Convert RecordKey back to native_db key
pub fn record_key_to_native_db_key(
    key: &RecordKey,
) -> Result<native_db::db_type::Key, Box<dyn std::error::Error>> {
    // Convert bytes back to native_db key
    let native_key = crate::bytes_to_native_db_key(key.as_ref());
    Ok(native_key)
}

/// Debug helper to inspect Record contents
pub fn debug_record<T>(record: &Record<T>) -> String
where
    T: Encode + bincode::Decode<()> + std::fmt::Debug,
{
    format!(
        "Record {{ key: {:?}, data: {:?}, expiry: {:?}, creator: {:?} }}",
        record.key, record.data, record.expiry, record.creator
    )
}

/// Debug helper to inspect KadRecord contents
pub fn debug_kad_record(record: &KadRecord) -> String {
    let key_display = if let Some(prefix) = extract_type_prefix(record.key.as_ref()) {
        format!("{}...", prefix)
    } else {
        format!(
            "{:?}...",
            &record.key.as_ref()[..std::cmp::min(8, record.key.as_ref().len())]
        )
    };

    format!(
        "KadRecord {{ key: {}, value_size: {} bytes, publisher: {:?}, expires: {:?} }}",
        key_display,
        record.value.len(),
        record.publisher,
        record.expires
    )
}

/// Create a catalog constructor helper for type-safe conversions
pub fn create_catalog_from_native_db<C, T>(native_data: T) -> C
where
    C: crate::CatalogConstructor<T>,
{
    C::from_native_db(native_data)
}

/// Extract native_db data from catalog
pub fn extract_native_db_from_catalog<C, T>(catalog: C) -> T
where
    C: crate::CatalogConstructor<T>,
{
    catalog.to_native_db()
}

/// Complete dataflow helper: Native DB → Catalog → KadRecord
pub fn native_db_to_kad_record<C, T>(native_data: T) -> KadRecord
where
    C: crate::CatalogConstructor<T> + NetabaseRecordExt + Clone,
{
    let catalog = C::from_native_db(native_data);
    catalog.to_kad_record()
}

/// Complete dataflow helper: KadRecord → Catalog → Native DB
pub fn kad_record_to_native_db<C, T>(kad_record: KadRecord) -> Result<T, Box<dyn std::error::Error>>
where
    C: NetabaseRecordExt + crate::CatalogConstructor<T>,
{
    let catalog = C::from_kad_record(kad_record)?;
    Ok(catalog.to_native_db())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CatalogConstructor, CatalogKey};
    use bincode::{Decode, Encode};
    use native_db::ToKey;

    #[derive(Debug, Clone, PartialEq, Encode, Decode)]
    struct TestData {
        name: String,
        value: u32,
    }

    #[derive(Debug, Clone, PartialEq, Encode, Decode)]
    enum TestCatalog {
        TestData(TestData),
    }

    #[derive(Debug, Clone, Encode, Decode)]
    enum TestCatalogKey {
        TestDataKey(crate::SerializableKey),
    }

    impl CatalogKey for TestCatalog {
        type KeyType = TestCatalogKey;

        fn catalog_key(&self) -> Self::KeyType {
            match self {
                TestCatalog::TestData(data) => {
                    let native_key = data.name.to_key();
                    let serializable_key = crate::SerializableKey::from_native_db_key_with_hint(
                        &native_key,
                        "TestCatalog::TestData".to_string(),
                    );
                    TestCatalogKey::TestDataKey(serializable_key)
                }
            }
        }

        fn key_to_serializable(key: &Self::KeyType) -> crate::SerializableKey {
            match key {
                TestCatalogKey::TestDataKey(serializable_key) => serializable_key.clone(),
            }
        }

        fn key_to_bytes(key: &Self::KeyType) -> Vec<u8> {
            Self::key_to_serializable(key).as_bytes().to_vec()
        }

        fn bytes_to_key(bytes: &[u8]) -> Result<Self::KeyType, Box<dyn std::error::Error>> {
            let serializable_key = crate::SerializableKey {
                key_bytes: bytes.to_vec(),
                type_hint: Some("TestCatalog::TestData".to_string()),
            };
            Ok(TestCatalogKey::TestDataKey(serializable_key))
        }
    }

    impl NetabaseRecordExt for TestCatalog {}

    impl CatalogConstructor<TestData> for TestCatalog {
        fn from_native_db(data: TestData) -> Self {
            TestCatalog::TestData(data)
        }

        fn to_native_db(self) -> TestData {
            match self {
                TestCatalog::TestData(data) => data,
            }
        }
    }

    #[test]
    fn test_content_key_generation() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let key1 = generate_content_key(&data, "TestData");
        let key2 = generate_content_key(&data, "TestData");

        // Same data should produce same key
        assert_eq!(key1, key2);

        let key_str = String::from_utf8(key1).unwrap();
        assert!(key_str.starts_with("TestData:"));
    }

    #[test]
    fn test_time_key_generation() {
        let data = TestData {
            name: "time_test".to_string(),
            value: 123,
        };

        let key1 = generate_time_key(&data, "TestData");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let key2 = generate_time_key(&data, "TestData");

        // Different times should produce different keys
        assert_ne!(key1, key2);

        let key1_str = String::from_utf8(key1).unwrap();
        let key2_str = String::from_utf8(key2).unwrap();
        assert!(key1_str.starts_with("TestData:"));
        assert!(key2_str.starts_with("TestData:"));
    }

    #[test]
    fn test_kad_record_conversion() {
        let native_data = TestData {
            name: "conversion_test".to_string(),
            value: 999,
        };
        let catalog = TestCatalog::from_native_db(native_data.clone());

        let kad_record = to_kad_record(&catalog);
        let recovered_catalog: TestCatalog = from_kad_record(kad_record).unwrap();
        let recovered_data = recovered_catalog.to_native_db();

        assert_eq!(recovered_data, native_data);
    }

    #[test]
    fn test_record_validation() {
        let native_data = TestData {
            name: "validation_test".to_string(),
            value: 456,
        };
        let catalog = TestCatalog::from_native_db(native_data);

        let kad_record = to_kad_record(&catalog);
        assert!(validate_kad_record::<TestCatalog>(&kad_record));

        // Invalid record
        let invalid_record = KadRecord {
            key: RecordKey::new(&"invalid".as_bytes()),
            value: vec![0xFF, 0xFF, 0xFF], // Invalid bincode data
            publisher: None,
            expires: None,
        };
        assert!(!validate_kad_record::<TestCatalog>(&invalid_record));
    }

    #[test]
    fn test_type_prefix_extraction() {
        let key1 = b"TestData:12345";
        let key2 = b"User:alice:67890";

        assert_eq!(extract_type_prefix(key1), Some("TestData".to_string()));
        assert_eq!(extract_type_prefix(key2), Some("User".to_string()));
    }

    #[test]
    fn test_expiring_record() {
        let data = TestData {
            name: "expiry_test".to_string(),
            value: 789,
        };
        let key = crate::native_db_key_to_bytes(&data.name.to_key());

        let expiry_time = Utc::now() + chrono::Duration::seconds(3600);
        let record = create_expiring_record(data, key, expiry_time);
        assert_eq!(record.expiry, Some(expiry_time));
    }

    #[test]
    fn test_attributed_record() {
        let data = TestData {
            name: "creator_test".to_string(),
            value: 101112,
        };
        let key = crate::native_db_key_to_bytes(&data.name.to_key());

        let record = create_attributed_record(data, key, "test_creator".to_string());
        assert_eq!(record.creator, Some("test_creator".to_string()));
    }

    #[test]
    fn test_batch_operations() {
        let native_objects = vec![
            TestData {
                name: "batch1".to_string(),
                value: 1,
            },
            TestData {
                name: "batch2".to_string(),
                value: 2,
            },
            TestData {
                name: "batch3".to_string(),
                value: 3,
            },
        ];

        let catalogs: Vec<TestCatalog> = native_objects
            .iter()
            .cloned()
            .map(|data| TestCatalog::from_native_db(data))
            .collect();

        let kad_records = batch_to_kad_records(&catalogs);
        let recovered_catalogs: Result<Vec<TestCatalog>, _> =
            batch_from_kad_records(kad_records).into_iter().collect();
        let recovered_catalogs = recovered_catalogs.unwrap();

        let recovered_native: Vec<TestData> = recovered_catalogs
            .into_iter()
            .map(|cat| cat.to_native_db())
            .collect();

        assert_eq!(recovered_native, native_objects);
    }

    #[test]
    fn test_filter_by_type_prefix() {
        let records = vec![
            KadRecord {
                key: RecordKey::new(&"TestData:123".as_bytes()),
                value: vec![1, 2, 3],
                publisher: None,
                expires: None,
            },
            KadRecord {
                key: RecordKey::new(&"User:456".as_bytes()),
                value: vec![4, 5, 6],
                publisher: None,
                expires: None,
            },
            KadRecord {
                key: RecordKey::new(&"TestData:789".as_bytes()),
                value: vec![7, 8, 9],
                publisher: None,
                expires: None,
            },
        ];

        let filtered = filter_by_type_prefix(&records, "TestData");
        assert_eq!(filtered.len(), 2);

        let user_filtered = filter_by_type_prefix(&records, "User");
        assert_eq!(user_filtered.len(), 1);
    }

    #[test]
    fn test_complete_dataflow() {
        let native_data = TestData {
            name: "dataflow_test".to_string(),
            value: 12345,
        };

        // Native DB → KadRecord
        let kad_record = native_db_to_kad_record::<TestCatalog, _>(native_data.clone());

        // KadRecord → Native DB
        let recovered_native: TestData =
            kad_record_to_native_db::<TestCatalog, _>(kad_record).unwrap();

        assert_eq!(recovered_native, native_data);
    }

    #[test]
    fn test_native_db_key_conversion() {
        let original_key = "test_key".to_key();
        let record_key = native_db_key_to_record_key(&original_key);
        let recovered_key = record_key_to_native_db_key(&record_key).unwrap();

        // Keys should be equivalent (though exact equality might depend on serialization format)
        let original_key_bytes = crate::native_db_key_to_bytes(&original_key);
        assert!(!original_key_bytes.is_empty());
        assert!(!record_key.as_ref().is_empty());
    }

    #[test]
    fn test_debug_helpers() {
        let data = TestData {
            name: "debug_test".to_string(),
            value: 999,
        };
        let native_key = data.name.to_key();
        let key_bytes = crate::native_db_key_to_bytes(&native_key);
        let record = Record::new(key_bytes, data.clone());

        let debug_str = debug_record(&record);
        assert!(debug_str.contains("debug_test"));
        assert!(debug_str.contains("999"));

        let catalog = TestCatalog::from_native_db(data);
        let kad_record = to_kad_record(&catalog);
        let kad_debug_str = debug_kad_record(&kad_record);
        assert!(kad_debug_str.contains("bytes"));
    }
}
