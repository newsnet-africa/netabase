//! Basic Netabase Example Validation
//!
//! This example validates that the basic netabase examples from the documentation
//! compile and work correctly. It demonstrates the fundamental schema definition
//! and key generation functionality.

use bincode::{Decode, Encode};
use netabase::NetabaseSchema;
use serde::{Deserialize, Serialize};

// Define your data structure - from basic documentation example
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct User {
    #[key]
    id: u64,
    name: String,
    email: String,
}

// Session example with single key - updated to work with current implementation
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct UserSession {
    #[key]
    session_id: String, // Single key field only
    user_id: u64, // Non-key field
    expires_at: u64,
    data: String,
}

// Message example without schema attributes
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct Message {
    #[key]
    id: String,
    sender: String,
    recipient: String,
    content: String,
    timestamp: u64,
}

fn main() {
    println!("=== Netabase Documentation Examples Validation ===");

    // Test 1: Basic User schema
    let user = User {
        id: 42,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    println!("User key: {}", user.key());
    assert_eq!(user.key().to_string(), "42");
    println!("✓ Basic user schema works correctly");

    // Test 2: Single key session schema
    let session = UserSession {
        session_id: "abc123".to_string(),
        user_id: 123,
        expires_at: 1234567890,
        data: "session data".to_string(),
    };

    println!("Session key: {}", session.key());
    assert_eq!(session.key().to_string(), "abc123");
    println!("✓ Single key session schema works correctly");

    // Test 3: Message schema
    let message = Message {
        id: "msg_001".to_string(),
        sender: "alice".to_string(),
        recipient: "bob".to_string(),
        content: "Hello!".to_string(),
        timestamp: 1234567890,
    };

    println!("Message key: {}", message.key());
    assert_eq!(message.key().to_string(), "msg_001");
    println!("✓ Message schema works correctly");

    // Test 4: Serialization roundtrip using bincode 2.x API
    let original_user = user.clone();
    let serialized = bincode::encode_to_vec(&original_user, bincode::config::standard())
        .expect("Serialization failed");
    let (deserialized, _): (User, usize) =
        bincode::decode_from_slice(&serialized, bincode::config::standard())
            .expect("Deserialization failed");

    assert_eq!(original_user, deserialized);
    println!("✓ Serialization roundtrip works correctly");

    // Test 5: Key generation consistency
    let user_copy = User {
        id: 42,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    assert_eq!(user.key().to_string(), user_copy.key().to_string());
    println!("✓ Key generation is consistent");

    // Test 6: Clone functionality
    let cloned_session = session.clone();
    assert_eq!(session.key().to_string(), cloned_session.key().to_string());
    assert_eq!(session, cloned_session);
    println!("✓ Clone functionality works correctly");

    println!("\n🎉 All documentation examples validated successfully!");
    println!("   - Basic schema definition ✓");
    println!("   - Single key schemas ✓");
    println!("   - Serialization/deserialization ✓");
    println!("   - Key generation consistency ✓");
    println!("   - Clone functionality ✓");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_user_schema() {
        let user = User {
            id: 42,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        assert_eq!(user.key().to_string(), "42");
    }

    #[test]
    fn test_single_key_session_schema() {
        let session = UserSession {
            session_id: "test_session".to_string(),
            user_id: 999,
            expires_at: 0,
            data: "test".to_string(),
        };

        assert_eq!(session.key().to_string(), "test_session");
    }

    #[test]
    fn test_message_schema() {
        let message = Message {
            id: "test_msg".to_string(),
            sender: "sender".to_string(),
            recipient: "recipient".to_string(),
            content: "test content".to_string(),
            timestamp: 0,
        };

        assert_eq!(message.key().to_string(), "test_msg");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let user = User {
            id: 123,
            name: "Roundtrip Test".to_string(),
            email: "roundtrip@example.com".to_string(),
        };

        let serialized = bincode::encode_to_vec(&user, bincode::config::standard())
            .expect("Serialization failed");
        let (deserialized, _): (User, usize) =
            bincode::decode_from_slice(&serialized, bincode::config::standard())
                .expect("Deserialization failed");

        assert_eq!(user, deserialized);
        assert_eq!(user.key().to_string(), deserialized.key().to_string());
    }

    #[test]
    fn test_key_consistency() {
        let user1 = User {
            id: 456,
            name: "Consistency Test 1".to_string(),
            email: "test1@example.com".to_string(),
        };

        let user2 = User {
            id: 456,
            name: "Consistency Test 2".to_string(),
            email: "test2@example.com".to_string(),
        };

        // Same ID should generate same key, regardless of other fields
        assert_eq!(user1.key().to_string(), user2.key().to_string());
    }
}
