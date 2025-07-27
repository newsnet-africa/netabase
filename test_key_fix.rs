//! Test to verify that the key generation fix works correctly
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

impl From<libp2p::kad::Record> for UserStruct {
    fn from(record: libp2p::kad::Record) -> Self {
        let data = record.value;
        bincode::decode_from_slice(&data, bincode::config::standard())
            .map(|(value, _)| value)
            .unwrap_or_else(|_| UserStruct {
                id: "default".to_string(),
                name: "default".to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netabase::netabase_trait::NetabaseSchema;

    #[test]
    fn test_different_instances_have_different_keys() {
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
    fn test_key_method_returns_owned_value() {
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
    fn test_key_reflects_current_field_value() {
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
}

fn main() {
    println!("Running key generation tests...");

    let user1 = UserStruct {
        id: "user_1".to_string(),
        name: "First User".to_string(),
    };

    let user2 = UserStruct {
        id: "user_2".to_string(),
        name: "Second User".to_string(),
    };

    println!("User 1 key: {}", user1.key().as_str());
    println!("User 2 key: {}", user2.key().as_str());

    // Verify they're different
    if user1.key().as_str() != user2.key().as_str() {
        println!("✅ SUCCESS: Different users have different keys!");
    } else {
        println!("❌ FAILURE: Users have the same key!");
    }

    // Test same ID users
    let user3 = UserStruct {
        id: "user_1".to_string(), // Same as user1
        name: "Third User".to_string(),
    };

    println!("User 3 key: {}", user3.key().as_str());

    if user1.key().as_str() == user3.key().as_str() {
        println!("✅ SUCCESS: Users with same ID have same keys!");
    } else {
        println!("❌ FAILURE: Users with same ID have different keys!");
    }
}
