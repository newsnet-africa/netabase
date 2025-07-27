//! Integration test to verify that the key generation fix works correctly
//!
//! This test ensures that each struct instance returns its own unique key
//! based on its field values, rather than all instances returning the same key.

use bincode::{Decode, Encode};
use netabase_macros::NetabaseSchema;

#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
struct UserStruct {
    #[key]
    id: String,
    name: String,
}

#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
enum ProductVariant {
    Digital {
        #[key]
        sku: String,
        download_url: String,
    },
    Physical {
        #[key]
        barcode: String,
        weight: f32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use netabase::netabase_trait::NetabaseSchema;

    #[test]
    fn test_struct_different_instances_have_different_keys() {
        let user1 = UserStruct {
            id: "123".to_string(),
            name: "Alice".to_string(),
        };

        let user2 = UserStruct {
            id: "456".to_string(),
            name: "Bob".to_string(),
        };

        let user3 = UserStruct {
            id: "123".to_string(), // Same ID as user1
            name: "Charlie".to_string(),
        };

        // Get keys from each instance
        let key1 = user1.key();
        let key2 = user2.key();
        let key3 = user3.key();

        // Verify that different IDs produce different keys
        assert_ne!(
            key1.as_str(),
            key2.as_str(),
            "Users with different IDs should have different keys"
        );

        // Verify that same IDs produce same keys
        assert_eq!(
            key1.as_str(),
            key3.as_str(),
            "Users with same IDs should have same keys"
        );

        // Verify keys match the expected format
        assert_eq!(key1.as_str(), "123");
        assert_eq!(key2.as_str(), "456");
        assert_eq!(key3.as_str(), "123");
    }

    #[test]
    fn test_struct_key_method_returns_owned_value() {
        let user = UserStruct {
            id: "test_id".to_string(),
            name: "Test User".to_string(),
        };

        // Call key() multiple times to ensure each call returns a new owned value
        let key1 = user.key();
        let key2 = user.key();

        // Both keys should have the same content
        assert_eq!(key1.as_str(), key2.as_str());
        assert_eq!(key1.as_str(), "test_id");
    }

    #[test]
    fn test_struct_key_reflects_current_field_value() {
        let mut user = UserStruct {
            id: "original_id".to_string(),
            name: "Test User".to_string(),
        };

        let original_key = user.key();
        assert_eq!(original_key.as_str(), "original_id");

        // Modify the field and verify the key changes
        user.id = "new_id".to_string();
        let new_key = user.key();
        assert_eq!(new_key.as_str(), "new_id");

        // Keys should be different
        assert_ne!(original_key.as_str(), new_key.as_str());
    }

    #[test]
    fn test_enum_different_variants_have_different_keys() {
        let digital1 = ProductVariant::Digital {
            sku: "SOFT123".to_string(),
            download_url: "https://example.com/download1".to_string(),
        };

        let digital2 = ProductVariant::Digital {
            sku: "SOFT456".to_string(),
            download_url: "https://example.com/download2".to_string(),
        };

        let physical1 = ProductVariant::Physical {
            barcode: "BAR123".to_string(),
            weight: 1.5,
        };

        let physical2 = ProductVariant::Physical {
            barcode: "BAR456".to_string(),
            weight: 2.0,
        };

        // Get keys from each variant
        let key_d1 = digital1.key();
        let key_d2 = digital2.key();
        let key_p1 = physical1.key();
        let key_p2 = physical2.key();

        // Verify that different SKUs/barcodes produce different keys
        assert_ne!(
            key_d1.as_str(),
            key_d2.as_str(),
            "Different digital products should have different keys"
        );
        assert_ne!(
            key_p1.as_str(),
            key_p2.as_str(),
            "Different physical products should have different keys"
        );
        assert_ne!(
            key_d1.as_str(),
            key_p1.as_str(),
            "Digital and physical products should have different keys"
        );

        // Verify keys match expected format
        assert_eq!(key_d1.as_str(), "SOFT123");
        assert_eq!(key_d2.as_str(), "SOFT456");
        assert_eq!(key_p1.as_str(), "BAR123");
        assert_eq!(key_p2.as_str(), "BAR456");
    }

    #[test]
    fn test_enum_same_keys_for_same_values() {
        let digital1 = ProductVariant::Digital {
            sku: "SAME123".to_string(),
            download_url: "https://example.com/download1".to_string(),
        };

        let digital2 = ProductVariant::Digital {
            sku: "SAME123".to_string(),                                // Same SKU
            download_url: "https://example.com/download2".to_string(), // Different URL
        };

        let key1 = digital1.key();
        let key2 = digital2.key();

        // Keys should be the same since they have the same SKU (key field)
        assert_eq!(
            key1.as_str(),
            key2.as_str(),
            "Products with same key field should have same keys"
        );
        assert_eq!(key1.as_str(), "SAME123");
    }

    #[test]
    fn test_key_trait_implementation() {
        let user = UserStruct {
            id: "trait_test".to_string(),
            name: "Trait Test User".to_string(),
        };

        // Test that the key implements the required methods
        let key = user.key();

        // Test as_str method
        assert_eq!(key.as_str(), "trait_test");

        // Test into_string method
        let key_string = key.clone().into_string();
        assert_eq!(key_string, "trait_test");

        // Test that we can create a new key with the same value
        let manual_key = UserStructKey::new("trait_test".to_string());
        assert_eq!(manual_key.as_str(), key.as_str());
    }

    #[test]
    fn test_key_serialization() {
        let user = UserStruct {
            id: "serialize_test".to_string(),
            name: "Serialize Test User".to_string(),
        };

        let key = user.key();

        // Test that the key can be encoded and decoded
        let encoded = bincode::encode_to_vec(&key, bincode::config::standard()).unwrap();
        let decoded: UserStructKey =
            bincode::decode_from_slice(&encoded, bincode::config::standard())
                .unwrap()
                .0;

        assert_eq!(key.as_str(), decoded.as_str());
    }

    #[test]
    fn test_no_static_key_sharing_bug() {
        // This test specifically checks for the bug that was fixed:
        // where all instances would return the key from the first instance

        let first_user = UserStruct {
            id: "first".to_string(),
            name: "First User".to_string(),
        };

        // Get the key from the first user
        let first_key = first_user.key();
        assert_eq!(first_key.as_str(), "first");

        // Create a second user with a different ID
        let second_user = UserStruct {
            id: "second".to_string(),
            name: "Second User".to_string(),
        };

        // Get the key from the second user
        let second_key = second_user.key();

        // This should NOT be "first" (which was the bug)
        assert_eq!(second_key.as_str(), "second");
        assert_ne!(first_key.as_str(), second_key.as_str());

        // Create a third user to double-check
        let third_user = UserStruct {
            id: "third".to_string(),
            name: "Third User".to_string(),
        };

        let third_key = third_user.key();
        assert_eq!(third_key.as_str(), "third");
        assert_ne!(first_key.as_str(), third_key.as_str());
        assert_ne!(second_key.as_str(), third_key.as_str());
    }

    #[test]
    fn test_automatic_from_record_implementation() {
        // Test that the derive macro automatically implements From<libp2p::kad::Record>
        let original_user = UserStruct {
            id: "from_record_test".to_string(),
            name: "From Record Test User".to_string(),
        };

        // Convert to kad record
        let record = libp2p::kad::Record::from(original_user.clone());

        // Convert back from record using the auto-generated From implementation
        let recovered_user = UserStruct::from(record);

        // Verify the data is the same
        assert_eq!(original_user.id, recovered_user.id);
        assert_eq!(original_user.name, recovered_user.name);
        assert_eq!(original_user.key().as_str(), recovered_user.key().as_str());
    }

    #[test]
    fn test_key_from_record_implementation() {
        // Test that keys can also be recovered from records
        let original_user = UserStruct {
            id: "key_from_record".to_string(),
            name: "Key From Record User".to_string(),
        };

        let key = original_user.key();

        // Convert key to record key
        let record_key = libp2p::kad::RecordKey::from(key.clone());

        // Create a dummy record with this key
        let record = libp2p::kad::Record {
            key: record_key,
            value: vec![],
            publisher: None,
            expires: None,
        };

        // Convert record key back to our key type
        let recovered_key = UserStructKey::from(record);

        // Verify the keys are the same
        assert_eq!(key.as_str(), recovered_key.as_str());
    }

    #[test]
    fn test_enum_from_record_implementation() {
        // Test that enum schemas also get automatic From implementations
        let original_product = ProductVariant::Digital {
            sku: "AUTO_FROM_TEST".to_string(),
            download_url: "https://example.com/auto-test".to_string(),
        };

        // Convert to kad record
        let record = libp2p::kad::Record::from(original_product.clone());

        // Convert back from record using the auto-generated From implementation
        let recovered_product = ProductVariant::from(record);

        // Verify the data is the same
        match (original_product, recovered_product) {
            (
                ProductVariant::Digital {
                    sku: orig_sku,
                    download_url: orig_url,
                },
                ProductVariant::Digital {
                    sku: rec_sku,
                    download_url: rec_url,
                },
            ) => {
                assert_eq!(orig_sku, rec_sku);
                assert_eq!(orig_url, rec_url);
            }
            _ => panic!("Product variants don't match after conversion"),
        }
    }

    #[test]
    fn test_round_trip_conversion() {
        // Test complete round-trip: struct -> record -> struct
        let original = UserStruct {
            id: "round_trip_test".to_string(),
            name: "Round Trip User".to_string(),
        };

        // First conversion: struct -> record
        let record = original
            .to_kad_record()
            .expect("Failed to convert to record");

        // Second conversion: record -> struct
        let recovered = UserStruct::from(record);

        // Verify everything matches
        assert_eq!(original.id, recovered.id);
        assert_eq!(original.name, recovered.name);
        assert_eq!(original.key().as_str(), recovered.key().as_str());

        // Verify keys are functionally equivalent
        let orig_record_key = original.kad_key();
        let recovered_record_key = recovered.kad_key();
        assert_eq!(orig_record_key.to_vec(), recovered_record_key.to_vec());
    }
}
