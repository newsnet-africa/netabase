#![feature(impl_trait_in_assoc_type)]

pub mod network;

// Legacy traits for compatibility with existing code
pub trait NetabaseRefCatalog<'a> {}

pub trait NetabaseCatalog {
    type RefCatalog<'a>: NetabaseRefCatalog<'a>;
}

use chrono::{DateTime, Utc};
pub use libp2p::kad::{Record as KadRecord, RecordKey};
use serde::{Deserialize, Serialize};

use std::time::Instant;

/// Trait for types that can provide a key for database operations.
/// This trait is implemented by both individual models and schema enums.
pub trait GetKey {
    type KeyType: Clone + Send + Sync;

    /// Get the key for this item
    fn key(&self) -> Self::KeyType;
}

/// Trait for types that can be sent safely across thread boundaries.
/// This is primarily used for schema enums that will be transmitted as messages.
pub trait ThreadSafe: Send + Sync + Clone {}

/// Blanket implementation for types that already implement the required bounds
impl<T> ThreadSafe for T where T: Send + Sync + Clone {}

/// Trait for conversion between netabase types and libp2p Record types.
/// Provides customizable expiry calculation for records.
pub trait RecordConversion: bincode::Encode + bincode::Decode<()> + GetKey + Clone {
    /// Calculate when this record should expire.
    /// Return None for records that don't expire.
    fn calculate_expiry(&self) -> Option<Instant>;

    /// Convert key to bytes for network transmission
    fn key_to_bytes(key: &Self::KeyType) -> Vec<u8>;

    /// Convert bytes back to key
    fn bytes_to_key(bytes: &[u8]) -> Result<Self::KeyType, Box<dyn std::error::Error>>;

    /// Convert to libp2p Record with calculated expiry
    fn to_record(&self) -> KadRecord {
        let key = self.key();
        let key_bytes = Self::key_to_bytes(&key);
        let record = Record::new(key_bytes, self.clone());
        let kad_record = KadRecord::from(record);

        // Apply expiry if calculated
        if let Some(expiry) = self.calculate_expiry() {
            KadRecord {
                key: kad_record.key,
                value: kad_record.value,
                publisher: kad_record.publisher,
                expires: Some(expiry),
            }
        } else {
            kad_record
        }
    }

    /// Convert from libp2p Record
    fn from_record(record: KadRecord) -> Result<Self, Box<dyn std::error::Error>> {
        let netabase_record: Record<Self> = record.try_into()?;
        Ok(netabase_record.data)
    }
}

/// Trait for ref enums to support conversion from native_db types
pub trait FromNativeDb<'a> {
    /// Try to convert from a native_db ToInput type
    fn try_from_native_db<T>(data: &'a T) -> Option<Self>
    where
        T: std::any::Any,
        Self: Sized,
        T: native_db::ToInput + 'a;
}

/// Custom Record wrapper that contains metadata along with the actual data payload.
/// This is used to wrap catalog data before network transmission, providing:
/// - key: The record identifier (as bytes for network compatibility)
/// - data: The actual data payload
/// - expiry: Optional expiration time for the record
/// - creator: Optional identifier of who created this record
///
/// The Record acts as a metadata envelope around the data before it gets
/// serialized with bincode and transmitted over the network via libp2p kad.
#[derive(Debug, Clone, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub struct Record<T> {
    pub key: Vec<u8>,
    pub data: T,
    #[bincode(with_serde)]
    pub expiry: Option<DateTime<Utc>>,
    pub creator: Option<String>,
}

impl<T> Record<T> {
    pub fn new(key: Vec<u8>, data: T) -> Self {
        Self {
            key,
            data,
            expiry: None,
            creator: None,
        }
    }

    pub fn with_expiry(mut self, expiry: DateTime<Utc>) -> Self {
        self.expiry = Some(expiry);
        self
    }

    pub fn with_creator(mut self, creator: String) -> Self {
        self.creator = Some(creator);
        self
    }

    /// Create a Record from native_db key and data
    pub fn from_native_db_key(key: native_db::db_type::Key, data: T) -> Self {
        let key_bytes = native_db_key_to_bytes(&key);
        Self::new(key_bytes, data)
    }

    /// Get the key as native_db::db_type::Key
    pub fn to_native_db_key(&self) -> native_db::db_type::Key {
        bytes_to_native_db_key(&self.key)
    }
}

// Direct conversion from Record<T> to libp2p_kad::Record using bincode
impl<T> From<Record<T>> for KadRecord
where
    T: bincode::Encode,
{
    fn from(record: Record<T>) -> Self {
        // Serialize the entire Record (including metadata) with bincode
        let serialized_record = bincode::encode_to_vec(&record, bincode::config::standard())
            .expect("Failed to serialize record with bincode");

        KadRecord {
            key: RecordKey::new(&record.key),
            value: serialized_record,
            publisher: None,
            expires: record.expiry.and_then(|expiry_time| {
                let duration_until_expiry = expiry_time.signed_duration_since(Utc::now());
                if duration_until_expiry > chrono::Duration::zero() {
                    Some(
                        std::time::Instant::now()
                            + std::time::Duration::from_secs(
                                duration_until_expiry.num_seconds() as u64
                            ),
                    )
                } else {
                    None
                }
            }),
        }
    }
}

// Direct conversion from libp2p_kad::Record to Record<T> using bincode
impl<T> TryFrom<KadRecord> for Record<T>
where
    T: bincode::Decode<()>,
{
    type Error = Box<dyn std::error::Error>;

    fn try_from(kad_record: KadRecord) -> Result<Self, Self::Error> {
        // Try to deserialize the entire Record from the value using bincode
        let (record, _): (Record<T>, usize) =
            bincode::decode_from_slice(&kad_record.value, bincode::config::standard())
                .map_err(|e| format!("Failed to deserialize record with bincode: {}", e))?;

        Ok(record)
    }
}

// Constructor trait for creating catalog items from native_db types
// This uses the constructor pattern instead of a generator as requested
pub trait CatalogConstructor<T> {
    /// Create a catalog item from a native_db type (constructor pattern)
    fn from_native_db(data: T) -> Self;

    /// Extract the native_db type from the catalog item
    fn to_native_db(self) -> T;
}

// Helper functions for working with native_db keys
pub fn native_db_key_to_bytes(key: &native_db::db_type::Key) -> Vec<u8> {
    // Convert to string for debug representation, then to bytes
    format!("{:?}", key).into_bytes()
}

pub fn bytes_to_native_db_key(bytes: &[u8]) -> native_db::db_type::Key {
    use native_db::ToKey;
    // Convert bytes back to key using ToKey
    bytes.to_key()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[derive(Debug, Clone)]
    struct TestKey {
        composite_key: String,
    }

    impl GetKey for TestData {
        type KeyType = TestKey;

        fn key(&self) -> TestKey {
            TestKey {
                composite_key: format!("test_{}_{}", self.name, self.value),
            }
        }
    }

    impl RecordConversion for TestData {
        fn calculate_expiry(&self) -> Option<Instant> {
            None
        }

        fn key_to_bytes(key: &TestKey) -> Vec<u8> {
            key.composite_key.as_bytes().to_vec()
        }

        fn bytes_to_key(bytes: &[u8]) -> Result<TestKey, Box<dyn std::error::Error>> {
            let debug_str = String::from_utf8(bytes.to_vec())?;
            Ok(TestKey {
                composite_key: debug_str,
            })
        }
    }

    impl CatalogConstructor<TestData> for TestData {
        fn from_native_db(data: TestData) -> Self {
            data
        }

        fn to_native_db(self) -> TestData {
            self
        }
    }

    #[test]
    fn test_record_creation() {
        let test_data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let key_bytes = vec![1, 2, 3, 4];
        let record = Record::new(key_bytes.clone(), test_data.clone());

        assert_eq!(record.key, key_bytes);
        assert_eq!(record.data, test_data);
        assert!(record.expiry.is_none());
        assert!(record.creator.is_none());
    }

    #[test]
    fn test_bincode_record_conversion() {
        let test_data = TestData {
            name: "bincode_test".to_string(),
            value: 200,
        };

        let record = Record::new(vec![5, 6, 7, 8], test_data.clone());
        let kad_record: KadRecord = record.clone().into();

        let recovered_record: Record<TestData> = kad_record.try_into().unwrap();
        assert_eq!(recovered_record.data, test_data);
        assert_eq!(recovered_record.key, record.key);
    }

    #[test]
    fn test_record_conversion() {
        let test_data = TestData {
            name: "record_test".to_string(),
            value: 300,
        };

        let kad_record = test_data.to_record();
        assert!(!kad_record.value.is_empty());

        let recovered_data = TestData::from_record(kad_record).unwrap();
        assert_eq!(recovered_data, test_data);
    }

    #[test]
    fn test_record_with_metadata() {
        use chrono::Utc;

        let test_data = TestData {
            name: "metadata_test".to_string(),
            value: 400,
        };

        let expiry_time = Utc::now() + chrono::Duration::hours(1);
        let record = Record::new(vec![9, 10, 11, 12], test_data.clone())
            .with_expiry(expiry_time)
            .with_creator("test_creator".to_string());

        assert_eq!(record.expiry, Some(expiry_time));
        assert_eq!(record.creator, Some("test_creator".to_string()));
    }

    #[test]
    fn test_record_to_kad_record() {
        let test_data = TestData {
            name: "kad_test".to_string(),
            value: 500,
        };

        let record = Record::new(vec![13, 14, 15, 16], test_data);
        let kad_record: KadRecord = record.into();
        assert!(!kad_record.value.is_empty());
    }

    #[test]
    fn test_key_to_bytes() {
        let test_data = TestData {
            name: "key_test".to_string(),
            value: 600,
        };

        let key = test_data.key();
        let key_bytes = TestData::key_to_bytes(&key);
        assert!(!key_bytes.is_empty());

        let recovered_key = TestData::bytes_to_key(&key_bytes).unwrap();
        assert_eq!(recovered_key.composite_key, key.composite_key);
    }

    #[test]
    fn test_get_key_trait() {
        let test_data = TestData {
            name: "get_key_test".to_string(),
            value: 700,
        };

        let key = test_data.key();
        assert!(key.composite_key.contains("get_key_test"));
        assert!(key.composite_key.contains("700"));
    }

    #[test]
    fn test_complete_dataflow() {
        let test_data = TestData {
            name: "complete_flow".to_string(),
            value: 800,
        };

        // Test complete flow: data -> Record -> KadRecord -> Record -> data
        let key_bytes = TestData::key_to_bytes(&test_data.key());
        let record = Record::new(key_bytes, test_data.clone());

        // Convert to KadRecord and back
        let kad_record: KadRecord = record.into();
        let recovered_record: Record<TestData> = kad_record.try_into().unwrap();

        // Verify data integrity
        assert_eq!(recovered_record.data, test_data);
    }

    #[test]
    fn test_record_conversion_with_expiry() {
        let test_data = TestData {
            name: "expiry_test".to_string(),
            value: 800,
        };

        // Test the to_record method which should use calculate_expiry
        let kad_record = test_data.to_record();
        assert!(!kad_record.value.is_empty());
        assert!(kad_record.expires.is_none()); // Since calculate_expiry returns None

        // Test from_record
        let recovered_data = TestData::from_record(kad_record).unwrap();
        assert_eq!(recovered_data, test_data);
    }

    #[test]
    fn test_new_trait_system_integration() {
        let test_data = TestData {
            name: "integration_test".to_string(),
            value: 900,
        };

        // Test GetKey trait
        let key = test_data.key();
        assert!(matches!(key, TestKey { .. }));
        assert!(key.composite_key.contains("integration_test"));
        assert!(key.composite_key.contains("900"));

        // Test RecordConversion trait
        assert!(test_data.calculate_expiry().is_none()); // Default implementation

        // Test key serialization roundtrip
        let key_bytes = <TestData as RecordConversion>::key_to_bytes(&key);
        let recovered_key = <TestData as RecordConversion>::bytes_to_key(&key_bytes).unwrap();
        assert_eq!(key.composite_key, recovered_key.composite_key);

        // Test RecordConversion::to_record and from_record
        let kad_record = test_data.to_record();
        let recovered_data = TestData::from_record(kad_record).unwrap();
        assert_eq!(test_data, recovered_data);

        // Test ThreadSafe trait (blanket implementation)
        fn assert_thread_safe<T: ThreadSafe>(_: &T) {}
        assert_thread_safe(&test_data);

        // Test RecordConversion trait (blanket implementation)
        let kad_record_via_conversion = test_data.to_record();
        assert!(!kad_record_via_conversion.value.is_empty());
    }
}
