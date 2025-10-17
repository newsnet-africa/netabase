//! Test schema conversion and serialization/deserialization
//!
//! This test verifies that the schema conversion between models and schema enums
//! works correctly, including serialization to bincode and back.

use bincode::{Decode, Encode};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::{NetabaseKeys, NetabaseModel as NetabaseModelTrait, NetabaseSchema};
use serde::{Deserialize, Serialize};

/// Test schema for conversion verification
#[netabase_schema_module(TestSchema, TestKeys)]
mod test_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TestDataKey)]
    pub struct TestData {
        #[key]
        pub id: u64,
        pub content: String,
        pub timestamp: u64,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TestEventKey)]
    pub struct TestEvent {
        #[key]
        pub event_id: String,
        pub event_type: String,
        pub data: String,
    }
}

use test_schema::{TestData, TestEvent, TestKeys, TestSchema};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_to_schema_conversion() {
        // Create test models
        let test_data = TestData {
            id: 42,
            content: "Hello, World!".to_string(),
            timestamp: 1234567890,
        };

        let test_event = TestEvent {
            event_id: "event_123".to_string(),
            event_type: "test".to_string(),
            data: "test data".to_string(),
        };

        // Test From conversions
        let schema_from_data: TestSchema = test_data.clone().into();
        let schema_from_event: TestSchema = test_event.clone().into();

        // Verify the conversions worked
        match schema_from_data {
            TestSchema::TestData(data) => assert_eq!(data, test_data),
            _ => panic!("Conversion to TestData variant failed"),
        }

        match schema_from_event {
            TestSchema::TestEvent(event) => assert_eq!(event, test_event),
            _ => panic!("Conversion to TestEvent variant failed"),
        }
    }

    #[test]
    fn test_schema_serialization_roundtrip() {
        // Create test model
        let test_data = TestData {
            id: 42,
            content: "Hello, World!".to_string(),
            timestamp: 1234567890,
        };

        // Convert to schema enum
        let schema: TestSchema = test_data.clone().into();

        // Serialize to bincode
        let serialized = bincode::encode_to_vec(&schema, bincode::config::standard())
            .expect("Failed to serialize schema");

        // Deserialize back
        let (deserialized, _): (TestSchema, _) =
            bincode::decode_from_slice(&serialized, bincode::config::standard())
                .expect("Failed to deserialize schema");

        // Verify roundtrip
        assert_eq!(schema, deserialized);

        // Verify the inner data is intact
        match deserialized {
            TestSchema::TestData(data) => assert_eq!(data, test_data),
            _ => panic!("Deserialized wrong variant"),
        }
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_record_conversion() {
        use libp2p::kad::Record;

        // Create test model
        let test_data = TestData {
            id: 42,
            content: "Hello, World!".to_string(),
            timestamp: 1234567890,
        };

        // Convert to schema enum
        let schema: TestSchema = test_data.clone().into();

        // Convert to libp2p Record
        let record = schema.to_record().expect("Failed to convert to record");

        // Convert back from Record
        let schema_back = TestSchema::from_record(record).expect("Failed to convert from record");

        // Verify roundtrip
        match schema_back {
            TestSchema::TestData(data) => assert_eq!(data, test_data),
            _ => panic!("Record conversion failed"),
        }
    }

    #[test]
    fn test_key_extraction() {
        // Create test model
        let test_data = TestData {
            id: 42,
            content: "Hello, World!".to_string(),
            timestamp: 1234567890,
        };

        // Get key from model
        let model_key = test_data.key();

        // Convert to schema and get key
        let schema: TestSchema = test_data.clone().into();
        let schema_key = schema.keys();

        // Verify keys match (they should both be TestDataKey variants)
        println!("Model key: {:?}", model_key);
        println!("Schema key: {:?}", schema_key);

        // Both should serialize to the same bytes
        let model_key_bytes = bincode::encode_to_vec(&model_key, bincode::config::standard())
            .expect("Failed to serialize model key");
        let schema_key_bytes = bincode::encode_to_vec(&schema_key, bincode::config::standard())
            .expect("Failed to serialize schema key");

        assert_eq!(model_key_bytes, schema_key_bytes);
    }

    #[test]
    fn test_multiple_models_in_schema() {
        // Create different model types
        let test_data = TestData {
            id: 42,
            content: "Hello, World!".to_string(),
            timestamp: 1234567890,
        };

        let test_event = TestEvent {
            event_id: "event_123".to_string(),
            event_type: "test".to_string(),
            data: "test data".to_string(),
        };

        // Convert both to same schema enum
        let schema_data: TestSchema = test_data.clone().into();
        let schema_event: TestSchema = test_event.clone().into();

        // Serialize both
        let serialized_data = bincode::encode_to_vec(&schema_data, bincode::config::standard())
            .expect("Failed to serialize data schema");
        let serialized_event = bincode::encode_to_vec(&schema_event, bincode::config::standard())
            .expect("Failed to serialize event schema");

        // Deserialize and verify types
        let (deserialized_data, _): (TestSchema, _) =
            bincode::decode_from_slice(&serialized_data, bincode::config::standard())
                .expect("Failed to deserialize data schema");
        let (deserialized_event, _): (TestSchema, _) =
            bincode::decode_from_slice(&serialized_event, bincode::config::standard())
                .expect("Failed to deserialize event schema");

        // Verify correct variants
        match deserialized_data {
            TestSchema::TestData(data) => assert_eq!(data, test_data),
            _ => panic!("Wrong variant for data"),
        }

        match deserialized_event {
            TestSchema::TestEvent(event) => assert_eq!(event, test_event),
            _ => panic!("Wrong variant for event"),
        }
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_record_key_roundtrip() {
        // Create test model
        let test_data = TestData {
            id: 42,
            content: "Hello, World!".to_string(),
            timestamp: 1234567890,
        };

        // Convert to schema and get schema key
        let schema: TestSchema = test_data.clone().into();
        let schema_key = schema.keys();

        // Convert to RecordKey
        let record_key = schema_key
            .to_record_key()
            .expect("Failed to convert to record key");

        // Convert back from RecordKey
        let key_back =
            TestKeys::from_record_key(record_key).expect("Failed to convert from record key");

        // Verify roundtrip
        assert_eq!(schema_key, key_back);
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_schema_to_record_uses_correct_key() {
        // Create test model
        let test_data = TestData {
            id: 42,
            content: "Hello, World!".to_string(),
            timestamp: 1234567890,
        };

        // Convert to schema enum
        let schema: TestSchema = test_data.clone().into();

        // Convert to libp2p Record
        let record = schema.to_record().expect("Failed to convert to record");

        // The record key should be the serialized schema key, not the entire schema
        let schema_key = schema.keys();
        let expected_key = schema_key
            .to_record_key()
            .expect("Failed to get expected key");

        assert_eq!(record.key, expected_key);

        // Verify we can convert back
        let schema_back = TestSchema::from_record(record).expect("Failed to convert from record");
        match schema_back {
            TestSchema::TestData(data) => assert_eq!(data, test_data),
            _ => panic!("Record roundtrip failed"),
        }
    }

    #[test]
    fn test_derive_more_conversions() {
        // Test that derive_more From/TryInto traits work
        let test_data = TestData {
            id: 42,
            content: "Hello, World!".to_string(),
            timestamp: 1234567890,
        };

        // Test From conversion (should work due to derive_more::From)
        let schema: TestSchema = TestSchema::from(test_data.clone());

        match schema {
            TestSchema::TestData(data) => assert_eq!(data, test_data),
            _ => panic!("derive_more From conversion failed"),
        }

        // Test TryInto conversion (should work due to derive_more::TryInto)
        let schema: TestSchema = test_data.clone().into();
        let extracted: Result<TestData, _> = schema.try_into();

        match extracted {
            Ok(data) => assert_eq!(data, test_data),
            Err(_) => panic!("derive_more TryInto conversion failed"),
        }

        #[cfg(feature = "libp2p")]
        #[test]
        fn test_schema_key_handling() {
            use libp2p::kad::{Record, RecordKey};

            // Create test model
            let test_data = TestData {
                id: 42,
                content: "Hello, World!".to_string(),
                timestamp: 1234567890,
            };

            // Convert to schema enum
            let schema: TestSchema = test_data.clone().into();

            // Get the key from the schema
            let schema_key = schema.keys();

            // Convert schema to record
            let record = schema.to_record().expect("Failed to convert to record");

            // Verify that the record key was created from schema.keys(), not from the entire schema
            let expected_key_bytes =
                bincode::encode_to_vec(&schema_key, bincode::config::standard())
                    .expect("Failed to serialize schema key");
            let actual_key_bytes = record.key.to_vec();

            assert_eq!(
                expected_key_bytes, actual_key_bytes,
                "Record key should be created from schema.keys()"
            );

            // Test round-trip: record key back to schema key
            let recovered_key = TestKeys::from_record_key(record.key.clone())
                .expect("Failed to convert record key back to schema key");

            // Verify the recovered key matches the original
            let recovered_key_bytes =
                bincode::encode_to_vec(&recovered_key, bincode::config::standard())
                    .expect("Failed to serialize recovered key");
        }
    }
}
