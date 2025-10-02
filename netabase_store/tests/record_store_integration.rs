#[cfg(feature = "libp2p")]
use bincode::{Decode, Encode};
#[cfg(feature = "libp2p")]
use libp2p::PeerId;
#[cfg(feature = "libp2p")]
use libp2p::kad::{ProviderRecord, Record};
#[cfg(feature = "libp2p")]
use libp2p::multihash::Multihash;
#[cfg(feature = "libp2p")]
use log::{debug, info};
#[cfg(feature = "libp2p")]
use netabase_macros::{NetabaseModel, netabase_schema_module};
#[cfg(feature = "libp2p")]
use netabase_store::traits::{
    NetabaseKeys, NetabaseKeysLibp2p, NetabaseModel, NetabaseSchema, NetabaseSchemaLibp2p,
};
#[cfg(feature = "libp2p")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "libp2p")]
use std::sync::Once;

#[cfg(feature = "libp2p")]
static INIT: Once = Once::new();

#[cfg(feature = "libp2p")]
fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    });
}

#[cfg(feature = "libp2p")]
const SHA_256_MH: u64 = 0x12;

#[cfg(feature = "libp2p")]
fn random_multihash() -> Multihash<64> {
    use rand::RngCore;

    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);

    Multihash::wrap(SHA_256_MH, &bytes).unwrap()
}

#[cfg(feature = "libp2p")]
#[netabase_schema_module(TestSchema, TestSchemaKeys)]
pub mod test_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TestRecordKey)]
    pub struct TestRecord {
        #[key]
        pub key: Vec<u8>,
        pub value: Vec<u8>,
        pub publisher: Option<Vec<u8>>,
        pub timestamp: u64,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TestDocumentKey)]
    pub struct TestDocument {
        #[key]
        pub id: String,
        pub content: String,
        pub metadata: std::collections::HashMap<String, String>,
        pub created_at: u64,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TestMessageKey)]
    pub struct TestMessage {
        #[key]
        pub id: u64,
        pub sender: String,
        pub recipient: String,
        pub content: String,
        pub encrypted: bool,
    }
}

#[cfg(feature = "libp2p")]
use test_schema::*;

#[cfg(test)]
mod tests {
    #[cfg(feature = "libp2p")]
    use super::*;

    #[cfg(not(feature = "libp2p"))]
    #[test]
    fn test_libp2p_feature_not_enabled() {
        // This test ensures the module compiles when libp2p feature is disabled
        // The actual libp2p tests are conditionally compiled and won't run
        assert!(
            true,
            "libp2p feature is not enabled - this is expected for some test runs"
        );
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_record_conversion_single_type() -> Result<(), Box<dyn std::error::Error>> {
        init_logger();
        info!("Starting test_record_conversion_single_type");

        // Create a test record using proper API
        debug!("Creating test record");
        let test_record = TestRecord {
            key: b"test_key_123".to_vec(),
            value: b"test_value_data".to_vec(),
            publisher: Some(b"test_publisher".to_vec()),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        debug!(
            "Created test record with key: {:?}",
            String::from_utf8_lossy(&test_record.key)
        );

        // Convert to schema and then to Record using proper API
        debug!("Converting to Record using proper API");
        let test_schema = TestSchema::TestRecord(test_record.clone());
        let record = test_schema.to_record()?;
        info!("✓ Record conversion successful");

        // Verify the record data
        debug!("Verifying record data");
        assert!(!record.key.to_vec().is_empty());
        assert!(!record.value.is_empty());
        info!("✓ Record data verification successful");

        // Convert back from Record to Schema
        debug!("Converting back from Record to Schema");
        let recovered_schema = TestSchema::from_record(record)?;
        info!("✓ Record back-conversion successful");

        // Verify the data matches
        debug!("Verifying data integrity after round-trip conversion");
        if let TestSchema::TestRecord(recovered_record) = recovered_schema {
            assert_eq!(recovered_record.key, test_record.key);
            assert_eq!(recovered_record.value, test_record.value);
            assert_eq!(recovered_record.publisher, test_record.publisher);
            assert_eq!(recovered_record.timestamp, test_record.timestamp);
            info!("✓ Round-trip conversion data integrity verified");
        } else {
            panic!("Expected TestRecord variant");
        }

        info!("test_record_conversion_single_type completed successfully");
        Ok(())
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_record_key_conversion() -> Result<(), Box<dyn std::error::Error>> {
        init_logger();
        info!("Starting test_record_key_conversion");

        // Create different types of records
        debug!("Creating test record for key conversion");
        let test_record = TestRecord {
            key: b"key_test_123".to_vec(),
            value: b"value_for_key_test".to_vec(),
            publisher: Some(b"key_publisher".to_vec()),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        // Test key conversion via schema keys
        debug!("Testing key conversion via schema keys");
        let test_key = test_record.key();
        let test_schema_key = TestSchemaKeys::TestRecordKey(test_key);
        let record_key = test_schema_key.to_record_key()?;
        info!("✓ Key conversion to RecordKey successful");

        // Verify the key is not empty and has proper format
        debug!("Verifying record key format");
        assert!(!record_key.to_vec().is_empty());
        info!("✓ Record key format verification successful");

        // Test round-trip key conversion
        debug!("Testing round-trip key conversion");
        let recovered_schema_key = TestSchemaKeys::from_record_key(record_key)?;
        info!("✓ Key back-conversion successful");

        // Verify key data integrity
        debug!("Verifying key data integrity");
        if let TestSchemaKeys::TestRecordKey(recovered_key) = recovered_schema_key {
            assert_eq!(recovered_key, test_record.key());
            info!("✓ Key round-trip conversion data integrity verified");
        } else {
            panic!("Expected TestRecordKey variant");
        }

        info!("test_record_key_conversion completed successfully");
        Ok(())
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_multiple_record_types_conversion() -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting test_multiple_record_types_conversion");

        // Create a TestRecord
        debug!("Creating TestRecord");
        let test_record = TestRecord {
            key: b"multi_test_record".to_vec(),
            value: b"multi_test_value".to_vec(),
            publisher: Some(b"multi_publisher".to_vec()),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        // Create a TestDocument
        debug!("Creating TestDocument");
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("author".to_string(), "test_author".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());

        let test_document = TestDocument {
            id: "doc_multi_123".to_string(),
            content: "This is multi-type test document content".to_string(),
            metadata,
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        // Create a TestMessage
        debug!("Creating TestMessage");
        let test_message = TestMessage {
            id: 42,
            sender: "alice@example.com".to_string(),
            recipient: "bob@example.com".to_string(),
            content: "Hello, this is a test message!".to_string(),
            encrypted: false,
        };

        // Convert all to Records
        debug!("Converting all types to Records");
        let record_schema = TestSchema::TestRecord(test_record.clone());
        let record1 = record_schema.to_record()?;

        let doc_schema = TestSchema::TestDocument(test_document.clone());
        let record2 = doc_schema.to_record()?;

        let msg_schema = TestSchema::TestMessage(test_message.clone());
        let record3 = msg_schema.to_record()?;
        info!("✓ All record types converted to Records successfully");

        // Convert back from Records
        debug!("Converting back from Records to Schemas");
        let recovered_schema1 = TestSchema::from_record(record1)?;
        let recovered_schema2 = TestSchema::from_record(record2)?;
        let recovered_schema3 = TestSchema::from_record(record3)?;
        info!("✓ All Records converted back to Schemas successfully");

        // Verify data integrity for each type
        debug!("Verifying TestRecord data integrity");
        if let TestSchema::TestRecord(recovered_record) = recovered_schema1 {
            assert_eq!(recovered_record.key, test_record.key);
            assert_eq!(recovered_record.value, test_record.value);
            assert_eq!(recovered_record.publisher, test_record.publisher);
            info!("✓ TestRecord data integrity verified");
        } else {
            panic!("Expected TestRecord variant");
        }

        debug!("Verifying TestDocument data integrity");
        if let TestSchema::TestDocument(recovered_doc) = recovered_schema2 {
            assert_eq!(recovered_doc.id, test_document.id);
            assert_eq!(recovered_doc.content, test_document.content);
            assert_eq!(recovered_doc.metadata, test_document.metadata);
            info!("✓ TestDocument data integrity verified");
        } else {
            panic!("Expected TestDocument variant");
        }

        debug!("Verifying TestMessage data integrity");
        if let TestSchema::TestMessage(recovered_msg) = recovered_schema3 {
            assert_eq!(recovered_msg.id, test_message.id);
            assert_eq!(recovered_msg.sender, test_message.sender);
            assert_eq!(recovered_msg.recipient, test_message.recipient);
            assert_eq!(recovered_msg.content, test_message.content);
            assert_eq!(recovered_msg.encrypted, test_message.encrypted);
            info!("✓ TestMessage data integrity verified");
        } else {
            panic!("Expected TestMessage variant");
        }

        info!("test_multiple_record_types_conversion completed successfully");
        Ok(())
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_record_serialization_formats() -> Result<(), Box<dyn std::error::Error>> {
        init_logger();
        info!("Starting test_record_serialization_formats");

        // Create a test record with various data types
        debug!("Creating test record with complex data");
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), "integration_test".to_string());
        metadata.insert("format".to_string(), "json".to_string());
        metadata.insert("version".to_string(), "2.1".to_string());

        let test_document = TestDocument {
            id: "format_test_doc".to_string(),
            content: "This document tests serialization formats with special chars: äöü, emoji: 🚀, numbers: 12345".to_string(),
            metadata,
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        // Convert to Record
        debug!("Converting to Record");
        let doc_schema = TestSchema::TestDocument(test_document.clone());
        let record = doc_schema.to_record()?;
        info!("✓ Complex document converted to Record");

        // Convert back from Record to verify data integrity
        debug!("Converting back from Record to schema");
        let recovered_schema = TestSchema::from_record(record)?;
        info!("✓ Record converted back to schema successfully");

        // Verify data integrity
        debug!("Verifying data integrity");
        if let TestSchema::TestDocument(recovered_doc) = recovered_schema {
            assert_eq!(recovered_doc.id, test_document.id);
            assert_eq!(recovered_doc.content, test_document.content);
            assert_eq!(recovered_doc.metadata, test_document.metadata);
            assert_eq!(recovered_doc.created_at, test_document.created_at);
            info!("✓ Data integrity verified - all special characters and data preserved");
        } else {
            panic!("Expected TestDocument variant");
        }

        info!("test_record_serialization_formats completed successfully");
        Ok(())
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_provider_record_integration() -> Result<(), Box<dyn std::error::Error>> {
        init_logger();
        info!("Starting test_provider_record_integration");

        // Create a test record
        debug!("Creating test record for provider integration");
        let test_record = TestRecord {
            key: b"provider_integration_key".to_vec(),
            value: b"provider_integration_value".to_vec(),
            publisher: Some(b"provider_publisher".to_vec()),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        // Get the record key for provider operations
        debug!("Getting record key for provider operations");
        let test_key = test_record.key();
        let test_schema_key = TestSchemaKeys::TestRecordKey(test_key);
        let record_key = test_schema_key.to_record_key()?;
        info!("✓ Record key obtained for provider operations");

        // Create a provider record
        debug!("Creating provider record");
        let provider_id = PeerId::random();
        let provider_record = ProviderRecord {
            key: record_key.clone(),
            provider: provider_id,
            expires: None,
            addresses: vec![],
        };
        info!(
            "✓ Provider record created with provider ID: {}",
            provider_id
        );

        // Test that the key can be converted back
        debug!("Testing provider record key conversion back to schema key");
        let key_copy = provider_record.key.clone();
        let recovered_schema_key = TestSchemaKeys::from_record_key(key_copy)?;
        if let TestSchemaKeys::TestRecordKey(recovered_key) = recovered_schema_key {
            assert_eq!(recovered_key, test_record.key());
            info!("✓ Provider record key conversion integrity verified");
        } else {
            panic!("Expected TestRecordKey variant");
        }

        // Verify provider record basic properties
        debug!("Verifying provider record properties");
        assert_eq!(provider_record.provider, provider_id);
        assert!(provider_record.expires.is_none());
        assert!(provider_record.addresses.is_empty());
        info!("✓ Provider record properties verified");

        info!("test_provider_record_integration completed successfully");
        Ok(())
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_edge_cases_and_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        init_logger();
        info!("Starting test_edge_cases_and_error_handling");

        // Test with empty data
        debug!("Testing with empty key data");
        let empty_record = TestRecord {
            key: vec![],
            value: b"non_empty_value".to_vec(),
            publisher: None,
            timestamp: 0,
        };

        let empty_schema = TestSchema::TestRecord(empty_record.clone());
        let empty_result = empty_schema.to_record();

        // This should still work as empty keys might be valid in some contexts
        match empty_result {
            Ok(record) => {
                info!("✓ Empty key conversion handled successfully");
                debug!(
                    "Empty key record created with value length: {}",
                    record.value.len()
                );
            }
            Err(e) => {
                info!("✓ Empty key conversion properly rejected: {}", e);
            }
        }

        // Test with very large data
        debug!("Testing with large data");
        let large_data = vec![42u8; 10000]; // 10KB of data
        let large_record = TestRecord {
            key: b"large_data_key".to_vec(),
            value: large_data.clone(),
            publisher: Some(b"large_publisher".to_vec()),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        let large_schema = TestSchema::TestRecord(large_record.clone());
        let large_result = large_schema.to_record()?;
        info!("✓ Large data conversion successful");

        // Verify large data integrity
        debug!("Verifying large data integrity");
        let recovered_large = TestSchema::from_record(large_result)?;
        if let TestSchema::TestRecord(recovered) = recovered_large {
            assert_eq!(recovered.value.len(), large_data.len());
            assert_eq!(recovered.value, large_data);
            info!("✓ Large data integrity verified");
        } else {
            panic!("Expected TestRecord variant");
        }

        // Test with special characters in string fields
        debug!("Testing with special characters");
        let special_doc = TestDocument {
            id: "special_chars_测试_🌟".to_string(),
            content: "Content with special chars: äöüß, 中文, русский, العربية, 🚀🌟⭐".to_string(),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("emoji".to_string(), "🎯🔥💡".to_string());
                map.insert("unicode".to_string(), "测试数据".to_string());
                map
            },
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        let special_schema = TestSchema::TestDocument(special_doc.clone());
        let special_result = special_schema.to_record()?;
        info!("✓ Special characters conversion successful");

        // Verify special characters integrity
        debug!("Verifying special characters integrity");
        let recovered_special = TestSchema::from_record(special_result)?;
        if let TestSchema::TestDocument(recovered) = recovered_special {
            assert_eq!(recovered.id, special_doc.id);
            assert_eq!(recovered.content, special_doc.content);
            assert_eq!(recovered.metadata, special_doc.metadata);
            info!("✓ Special characters integrity verified");
        } else {
            panic!("Expected TestDocument variant");
        }

        info!("test_edge_cases_and_error_handling completed successfully");
        Ok(())
    }
}
