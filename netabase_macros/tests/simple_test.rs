//! Minimal test to debug NetabaseSchema macro behavior

use netabase_macros::NetabaseSchema;

// Very simple struct to test basic macro functionality
#[derive(NetabaseSchema, Clone, Debug, PartialEq, bincode::Encode, bincode::Decode)]
struct SimpleUser {
    #[key]
    id: String,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_struct_creation() {
        let user = SimpleUser {
            id: "test123".to_string(),
            name: "Test User".to_string(),
        };

        // Test basic functionality
        let cloned_user = user.clone();
        assert_eq!(user, cloned_user);

        println!("Simple user: {:?}", user);
        println!("Simple struct creation test passed");
    }

    #[test]
    fn test_key_generation_basic() {
        let user = SimpleUser {
            id: "key_test".to_string(),
            name: "Key Test".to_string(),
        };

        // Test key generation
        let key = user.key();
        assert_eq!(key.as_str(), "key_test");

        println!("Key generation basic test passed");
    }

    #[test]
    fn test_key_type_creation() {
        // Test key type creation methods
        let key1 = SimpleUserKey::new("test_key".to_string());
        let key2: SimpleUserKey = "test_key".to_string().into();

        assert_eq!(key1, key2);
        assert_eq!(key1.as_str(), "test_key");

        println!("Key type creation test passed");
    }
}
