//! Integration tests for NetabaseSchema derive macro and generated trait implementations
//!
//! These tests verify that the syn-generated code works correctly for various schema types
//! and provides proper integration with Kademlia distributed hash tables.

use bincode::{Decode, Encode};
use netabase::netabase_trait::{NetabaseSchema as NetabaseSchemaTrait, NetabaseSchemaKey};
use netabase_macros::NetabaseSchema;

/// Simple struct with a string key field
#[derive(Clone, Debug, PartialEq, NetabaseSchema, Encode, Decode)]
struct User {
    #[key]
    id: String,
    name: String,
    email: String,
    age: u32,
}

/// Struct with numeric key field
#[derive(Clone, Debug, PartialEq, NetabaseSchema, Encode, Decode)]
struct Product {
    #[key]
    sku: u64,
    name: String,
    price: f64,
}

/// Enum with key fields in different variants
#[derive(Clone, Debug, PartialEq, NetabaseSchema, Encode, Decode)]
enum Document {
    Text {
        #[key]
        hash: String,
        content: String,
    },
    Image {
        #[key]
        checksum: String,
        data: Vec<u8>,
    },
    Video {
        #[key]
        id: u64,
        duration: u32,
    },
}

/// Unit enum for testing unit variants
#[derive(Clone, Debug, PartialEq, NetabaseSchema, Encode, Decode)]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_key_extraction() {
        let user = User {
            id: "user123".to_string(),
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };

        // Test key extraction
        let key = user.key();
        assert_eq!(key.as_str(), "user123");
    }

    #[test]
    fn test_numeric_key_extraction() {
        let product = Product {
            sku: 12345,
            name: "Laptop".to_string(),
            price: 999.99,
        };

        // Test numeric key formatting
        let key = product.key();
        assert_eq!(key.as_str(), "12345");
    }

    #[test]
    fn test_enum_key_extraction() {
        let text_doc = Document::Text {
            hash: "abc123".to_string(),
            content: "Hello world".to_string(),
        };

        let image_doc = Document::Image {
            checksum: "def456".to_string(),
            data: vec![1, 2, 3, 4],
        };

        let video_doc = Document::Video {
            id: 789,
            duration: 3600,
        };

        // Test key extraction from different enum variants
        assert_eq!(text_doc.key().as_str(), "abc123");
        assert_eq!(image_doc.key().as_str(), "def456");
        assert_eq!(video_doc.key().as_str(), "789");
    }

    #[test]
    fn test_unit_enum_keys() {
        let active = Status::Active;
        let inactive = Status::Inactive;
        let pending = Status::Pending;

        // Unit variants should use their name as key
        assert_eq!(active.key().as_str(), "Active");
        assert_eq!(inactive.key().as_str(), "Inactive");
        assert_eq!(pending.key().as_str(), "Pending");
    }

    #[test]
    fn test_key_generation() {
        let key1 = UserKey::generate_key();
        let key2 = UserKey::generate_key();

        // Generated keys should be different
        assert_ne!(key1.as_str(), key2.as_str());

        // Keys should be non-empty
        assert!(!key1.as_str().is_empty());
        assert!(!key2.as_str().is_empty());

        // Keys should contain timestamp and hash parts
        assert!(key1.as_str().contains('_'));
        assert!(key2.as_str().contains('_'));
    }

    #[test]
    fn test_key_type_utilities() {
        let custom_key = UserKey::new("custom123".to_string());

        // Test utility methods
        assert_eq!(custom_key.as_str(), "custom123");

        let key_string = custom_key.clone().into_string();
        assert_eq!(key_string, "custom123");
    }

    #[test]
    fn test_kad_record_conversion() -> Result<(), Box<dyn std::error::Error>> {
        let user = User {
            id: "test456".to_string(),
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 25,
        };

        // Convert to Kademlia record
        let record = user.to_kad_record()?;

        // Verify key matches
        let key_bytes = record.key.to_vec();
        let key_string = String::from_utf8(key_bytes)?;
        assert_eq!(key_string, user.key().as_str());

        // Verify record has data
        assert!(!record.value.is_empty());

        Ok(())
    }

    #[test]
    fn test_kad_key_generation() {
        let product = Product {
            sku: 99999,
            name: "Test Product".to_string(),
            price: 19.99,
        };

        let kad_key = product.kad_key();
        let key_bytes = kad_key.to_vec();
        let key_string = String::from_utf8(key_bytes).unwrap();

        assert_eq!(key_string, "99999");
    }

    #[test]
    fn test_serialization_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let original_user = User {
            id: "round_trip_test".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            age: 42,
        };

        // Convert to record and back
        let record = original_user.to_kad_record()?;
        let recovered_user = User::from(record);

        // Verify data integrity
        assert_eq!(original_user, recovered_user);
        assert_eq!(original_user.key().as_str(), recovered_user.key().as_str());

        Ok(())
    }

    #[test]
    fn test_enum_serialization_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let original_doc = Document::Text {
            hash: "serialization_test".to_string(),
            content: "This is test content".to_string(),
        };

        // Convert to record and back
        let record = original_doc.to_kad_record()?;
        let recovered_doc = Document::from(record);

        // Verify data integrity
        assert_eq!(original_doc, recovered_doc);
        assert_eq!(original_doc.key().as_str(), recovered_doc.key().as_str());

        Ok(())
    }

    #[test]
    fn test_key_from_record() -> Result<(), Box<dyn std::error::Error>> {
        let user = User {
            id: "key_from_record_test".to_string(),
            name: "Key Test User".to_string(),
            email: "keytest@example.com".to_string(),
            age: 33,
        };

        let record = user.to_kad_record()?;
        let key_from_record = UserKey::from(record);

        assert_eq!(key_from_record.as_str(), user.key().as_str());

        Ok(())
    }

    #[test]
    fn test_bincode_key_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let key = UserKey::new("bincode_test".to_string());

        // Serialize and deserialize the key
        let serialized = bincode::encode_to_vec(&key, bincode::config::standard())?;
        let (deserialized, _) =
            bincode::decode_from_slice::<UserKey, _>(&serialized, bincode::config::standard())?;

        assert_eq!(key.as_str(), deserialized.as_str());

        Ok(())
    }

    #[test]
    fn test_multiple_schema_types() {
        // Test that different schema types can coexist
        let user = User {
            id: "multi_test_user".to_string(),
            name: "Multi User".to_string(),
            email: "multi@example.com".to_string(),
            age: 28,
        };

        let product = Product {
            sku: 54321,
            name: "Multi Product".to_string(),
            price: 49.99,
        };

        let document = Document::Video {
            id: 12345,
            duration: 7200,
        };

        // Each should have their own key type and extraction logic
        assert_eq!(user.key().as_str(), "multi_test_user");
        assert_eq!(product.key().as_str(), "54321");
        assert_eq!(document.key().as_str(), "12345");

        // Keys should be different types (this is ensured by the type system)
        let _user_key: &UserKey = user.key();
        let _product_key: &ProductKey = product.key();
        let _document_key: &DocumentKey = document.key();
    }

    #[test]
    fn test_key_uniqueness_over_time() {
        // Generate multiple keys and ensure they're unique
        let mut keys = std::collections::HashSet::new();

        for _ in 0..100 {
            let key = UserKey::generate_key();
            assert!(
                keys.insert(key.as_str().to_string()),
                "Generated duplicate key"
            );
        }

        #[test]
        fn test_record_from_schema_conversion() -> Result<(), Box<dyn std::error::Error>> {
            let user = User {
                id: "from_conversion_test".to_string(),
                name: "From User".to_string(),
                email: "from@example.com".to_string(),
                age: 27,
            };

            // Test From<User> for libp2p::kad::Record
            let record: libp2p::kad::Record = user.clone().into();

            // Verify key matches
            let key_bytes = record.key.to_vec();
            let key_string = String::from_utf8(key_bytes)?;
            assert_eq!(key_string, user.key().as_str());

            // Verify record has data
            assert!(!record.value.is_empty());

            // Verify we can deserialize back
            let recovered_user = User::from(record);
            assert_eq!(user, recovered_user);

            Ok(())
        }

        #[test]
        fn test_record_key_from_key_conversion() -> Result<(), Box<dyn std::error::Error>> {
            let custom_key = UserKey::new("key_conversion_test".to_string());

            // Test From<UserKey> for libp2p::kad::RecordKey
            let record_key: libp2p::kad::RecordKey = custom_key.clone().into();

            // Verify the conversion
            let key_bytes = record_key.to_vec();
            let key_string = String::from_utf8(key_bytes)?;
            assert_eq!(key_string, custom_key.as_str());

            Ok(())
        }

        #[test]
        fn test_enum_record_from_conversion() -> Result<(), Box<dyn std::error::Error>> {
            let document = Document::Text {
                hash: "enum_from_test".to_string(),
                content: "Test content for From conversion".to_string(),
            };

            // Test From<Document> for libp2p::kad::Record
            let record: libp2p::kad::Record = document.clone().into();

            // Verify key matches
            let key_bytes = record.key.to_vec();
            let key_string = String::from_utf8(key_bytes)?;
            assert_eq!(key_string, document.key().as_str());

            // Verify round-trip conversion
            let recovered_document = Document::from(record);
            assert_eq!(document, recovered_document);

            Ok(())
        }

        #[test]
        fn test_multiple_type_conversions() -> Result<(), Box<dyn std::error::Error>> {
            let user = User {
                id: "multi_from_user".to_string(),
                name: "Multi From User".to_string(),
                email: "multiFrom@example.com".to_string(),
                age: 31,
            };

            let product = Product {
                sku: 99887,
                name: "Multi From Product".to_string(),
                price: 29.99,
            };

            // Test From conversions for different types
            let user_record: libp2p::kad::Record = user.clone().into();
            let product_record: libp2p::kad::Record = product.clone().into();

            // Test key conversions
            let user_key = UserKey::new("test_user_key".to_string());
            let product_key = ProductKey::new("test_product_key".to_string());

            let user_record_key: libp2p::kad::RecordKey = user_key.clone().into();
            let product_record_key: libp2p::kad::RecordKey = product_key.clone().into();

            // Verify conversions
            assert_eq!(
                String::from_utf8(user_record.key.to_vec())?,
                user.key().as_str()
            );
            assert_eq!(
                String::from_utf8(product_record.key.to_vec())?,
                product.key().as_str()
            );
            assert_eq!(
                String::from_utf8(user_record_key.to_vec())?,
                user_key.as_str()
            );
            assert_eq!(
                String::from_utf8(product_record_key.to_vec())?,
                product_key.as_str()
            );

            Ok(())
        }

        #[test]
        fn test_idiomatic_conversion_patterns() -> Result<(), Box<dyn std::error::Error>> {
            let user = User {
                id: "idiomatic_test".to_string(),
                name: "Idiomatic User".to_string(),
                email: "idiomatic@example.com".to_string(),
                age: 24,
            };

            // Test that both patterns work equivalently
            let record1 = user.to_kad_record()?;
            let record2: libp2p::kad::Record = user.clone().into();

            // Both should produce equivalent records
            assert_eq!(record1.key, record2.key);
            assert_eq!(record1.value, record2.value);

            // Test key conversion patterns
            let key = user.key().clone();
            let record_key1 = user.kad_key();
            let record_key2: libp2p::kad::RecordKey = key.into();

            assert_eq!(record_key1, record_key2);

            Ok(())
        }

        #[test]
        fn test_conversion_with_generated_keys() {
            // Test From conversions with generated keys
            let generated_key = UserKey::generate_key();
            let record_key: libp2p::kad::RecordKey = generated_key.clone().into();

            // Verify the conversion preserves the key content
            let key_bytes = record_key.to_vec();
            let key_string = String::from_utf8(key_bytes).expect("Key should be valid UTF-8");
            assert_eq!(key_string, generated_key.as_str());

            // Test with multiple generated keys
            for _ in 0..10 {
                let key = ProductKey::generate_key();
                let record_key: libp2p::kad::RecordKey = key.clone().into();
                let recovered_string = String::from_utf8(record_key.to_vec()).unwrap();
                assert_eq!(recovered_string, key.as_str());
            }
        }
    }
}
