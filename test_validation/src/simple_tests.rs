//! Simple test cases to verify basic validation functionality
//!
//! This module contains minimal test cases to verify that the key field
//! validation constraints are working correctly.

use bincode::{Decode, Encode};
use netabase::{netabase_trait, NetabaseSchema};

// Test 1: Valid struct with exactly 1 key field
#[derive(NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct UserStruct {
    #[key]
    id: String,
    name: String,
}

// Test 2: Valid enum with exactly 1 key field per variant
#[derive(NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
enum UserEnum {
    Regular {
        #[key]
        user_id: String,
        name: String,
    },
    Admin {
        #[key]
        admin_id: String,
        permissions: Vec<String>,
    },
}

// Test 3: Valid struct with no key field (should generate error but compile for testing)
// Commented out because it should fail compilation
// #[derive(NetabaseSchema, Clone, Debug, PartialEq)]
// struct NoKeyStruct {
//     name: String,
//     email: String,
// }

// Test 4: Valid enum variant with unit variant (no key needed for unit)
#[derive(NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
enum MixedEnum {
    User {
        #[key]
        id: String,
        name: String,
    },
    Anonymous, // Unit variant - no key field required
}

// Test 5: Simple tuple struct with key
#[derive(NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct TupleStruct(#[key] String, i32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_struct_compiles() {
        let user = UserStruct {
            id: "123".to_string(),
            name: "Alice".to_string(),
        };

        // Test that we can call the key() method
        let key = user.key();
        assert_eq!(key.as_str(), "123");
    }

    #[test]
    fn test_user_enum_compiles() {
        let user = UserEnum::Regular {
            user_id: "user123".to_string(),
            name: "Bob".to_string(),
        };

        let admin = UserEnum::Admin {
            admin_id: "admin456".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
        };

        // Test that we can call the key() method
        let key = user.key();
        assert_eq!(key.as_str(), "user123");
    }

    // #[test]
    // fn test_no_key_struct_compiles() {
    //     let user = NoKeyStruct {
    //         name: "Charlie".to_string(),
    //         email: "charlie@example.com".to_string(),
    //     };
    //
    //     // This should fail compilation due to validation
    //     // let key = user.key();
    // }

    #[test]
    fn test_mixed_enum_compiles() {
        let user = MixedEnum::User {
            id: "user789".to_string(),
            name: "Dave".to_string(),
        };

        let anon = MixedEnum::Anonymous;

        let key = user.key();
        assert_eq!(key.as_str(), "user789");
    }

    #[test]
    fn test_tuple_struct_compiles() {
        let data = TupleStruct("key123".to_string(), 42);

        let key = data.key();
        assert_eq!(key.as_str(), "key123");
    }
}
