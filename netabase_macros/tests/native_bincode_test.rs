//! Test suite for native bincode functionality in NetabaseSchema
//!
//! This test verifies that the derive macro correctly generates basic implementations
//! and key functionality.

use netabase_macros::NetabaseSchema;

// Test basic struct with native bincode (default behavior)
#[derive(NetabaseSchema, Clone, Debug, PartialEq)]
struct TestUserNative {
    #[key]
    id: String,
    name: String,
    email: String,
    age: u32,
}

// Test enum with native bincode
#[derive(NetabaseSchema, Clone, Debug, PartialEq)]
enum TestDocumentNative {
    #[key]
    Post { id: String, title: String },
    #[key]
    Comment { id: String, text: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_struct_functionality() {
        let user = TestUserNative {
            id: "test123".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            age: 30,
        };

        // Test basic functionality
        let cloned_user = user.clone();
        assert_eq!(user, cloned_user);

        println!("User: {:?}", user);
        println!("Basic struct functionality test passed");
    }

    #[test]
    fn test_key_generation() {
        let user = TestUserNative {
            id: "key_test".to_string(),
            name: "Key Test".to_string(),
            email: "key@test.com".to_string(),
            age: 25,
        };

        // Test key generation
        let key = user.key();
        assert_eq!(key.as_str(), "key_test");

        // Test key properties
        assert_eq!(format!("{}", key), "key_test");
        let key_ref: &str = key.as_ref();
        assert_eq!(key_ref, "key_test");

        // Test key conversions
        let key_bytes: Vec<u8> = key.clone().into();
        let key_from_bytes = TestUserNativeKey::from(key_bytes);
        assert_eq!(key, key_from_bytes);

        println!("Key generation test passed");
    }

    #[test]
    fn test_enum_functionality() {
        let post = TestDocumentNative::Post {
            id: "post456".to_string(),
            title: "Test Post Title".to_string(),
        };

        let comment = TestDocumentNative::Comment {
            id: "comment789".to_string(),
            text: "Test comment text".to_string(),
        };

        // Test enum cloning and equality
        let cloned_post = post.clone();
        assert_eq!(post, cloned_post);

        // Test key generation for different enum variants
        assert_eq!(post.key().as_str(), "post456");
        assert_eq!(comment.key().as_str(), "comment789");

        println!("Enum functionality test passed");
    }

    #[test]
    fn test_key_utility_methods() {
        // Test key creation methods
        let key1 = TestUserNativeKey::new("test_key".to_string());
        let key2: TestUserNativeKey = "test_key".to_string().into();

        assert_eq!(key1, key2);
        assert_eq!(key1.as_str(), "test_key");
        assert_eq!(key1.into_string(), "test_key".to_string());

        // Test key generation
        let generated_key = TestUserNativeKey::generate_key();
        assert!(!generated_key.as_str().is_empty());

        println!("Key utility methods test passed");
    }

    #[test]
    fn test_thread_safety() {
        let user = TestUserNative {
            id: "thread_test".to_string(),
            name: "Thread Test".to_string(),
            email: "thread@test.com".to_string(),
            age: 40,
        };

        // Test Send
        let user_clone = user.clone();
        std::thread::spawn(move || {
            assert_eq!(user_clone.id, "thread_test");
        })
        .join()
        .unwrap();

        // Test Sync with scoped threads
        let user_ref = &user;
        std::thread::scope(|s| {
            s.spawn(|| {
                assert_eq!(user_ref.name, "Thread Test");
            });
        });

        println!("Thread safety test passed");
    }
}
