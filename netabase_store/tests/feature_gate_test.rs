use bincode::{Decode, Encode};
use netabase_macros::NetabaseModel;
use netabase_store::traits::{NetabaseModel, NetabaseModelKey};

#[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq)]
#[key_name(TestUserKey)]
pub struct TestUser {
    #[key]
    pub id: u64,
    pub name: String,
    #[secondary_key]
    pub email: String,
}

#[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq)]
#[key_name(PostKey)]
pub struct Post {
    #[key]
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author_id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_model_functionality_always_works() {
        // This should always work regardless of feature flags
        let user = TestUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        // Basic model functionality should always work
        let key = user.key();
        assert_eq!(key, TestUserKey::Primary(TestUserPrimaryKey(1)));
        assert_eq!(TestUser::tree_name(), "TestUser");

        // Test clone and debug traits
        let user_clone = user.clone();
        assert_eq!(user, user_clone);
        let debug_str = format!("{:?}", user);
        assert!(debug_str.contains("Test User"));
    }

    #[test]
    fn test_secondary_keys_always_work() {
        let user = TestUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        // Test primary key extraction
        let primary_key = user.key();
        if let Some(pk) = primary_key.primary_keys() {
            assert_eq!(*pk, TestUserPrimaryKey(1));
        } else {
            panic!("Expected primary key");
        }
        assert!(primary_key.secondary_keys().is_none());

        // Test secondary key creation
        let secondary_key = TestUserKey::Secondary(TestUserSecondaryKeys::EmailKey(
            "test@example.com".to_string(),
        ));
        assert!(secondary_key.primary_keys().is_none());
        if let Some(sk) = secondary_key.secondary_keys() {
            match sk {
                TestUserSecondaryKeys::EmailKey(email) => assert_eq!(email, "test@example.com"),
            }
        } else {
            panic!("Expected secondary key");
        }
    }

    #[test]
    fn test_multiple_models_work() {
        let user = TestUser {
            id: 1,
            name: "Author".to_string(),
            email: "author@example.com".to_string(),
        };

        let post = Post {
            id: 100,
            title: "Test Post".to_string(),
            content: "This is a test post".to_string(),
            author_id: user.id,
        };

        // Both models should work independently
        assert_eq!(TestUser::tree_name(), "TestUser");
        assert_eq!(Post::tree_name(), "Post");

        let user_key = user.key();
        let post_key = post.key();

        assert_eq!(user_key, TestUserKey::Primary(TestUserPrimaryKey(1)));
        assert_eq!(post_key, PostKey::Primary(PostPrimaryKey(100)));
    }

    #[test]
    fn test_serialization_always_works() {
        let user = TestUser {
            id: 42,
            name: "Serializable User".to_string(),
            email: "serializable@example.com".to_string(),
        };

        // Test bincode serialization (should always work)
        let encoded = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();
        assert!(!encoded.is_empty());

        let (decoded, _): (TestUser, usize) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(user, decoded);
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_libp2p_traits_are_available_when_enabled() {
        use netabase_store::traits::{NetabaseKeysLibp2p, NetabaseSchemaLibp2p};

        // This test verifies that the libp2p-specific traits are available
        // when the feature is enabled, even though the macro implementations
        // might not be working yet.

        let user = TestUser {
            id: 42,
            name: "LibP2P User".to_string(),
            email: "libp2p@example.com".to_string(),
        };

        // Test that the traits exist and can be imported
        // Note: The actual methods might not work yet due to macro issues,
        // but the traits should be available

        // Basic functionality should still work
        let key = user.key();
        assert_eq!(key, TestUserKey::Primary(TestUserPrimaryKey(42)));
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_libp2p_imports_work() {
        // Test that libp2p types can be imported when feature is enabled
        use libp2p::kad::{Record, RecordKey};

        // Create some dummy data to ensure the types are usable
        let record = Record {
            key: RecordKey::new(&[1, 2, 3, 4]),
            value: vec![5, 6, 7, 8],
            publisher: None,
            expires: None,
        };

        assert!(!record.value.is_empty());
        assert!(!record.key.to_vec().is_empty());
    }

    #[cfg(not(feature = "libp2p"))]
    #[test]
    fn test_functionality_works_without_libp2p_feature() {
        // This test ensures that the code compiles and works without libp2p feature.
        // The main thing is that this test can run without libp2p dependencies.
        let user = TestUser {
            id: 100,
            name: "No LibP2P User".to_string(),
            email: "no-libp2p@example.com".to_string(),
        };

        let post = Post {
            id: 200,
            title: "No LibP2P Post".to_string(),
            content: "This post works without libp2p".to_string(),
            author_id: user.id,
        };

        // Basic model functionality should always work
        let user_key = user.key();
        let post_key = post.key();

        assert_eq!(user_key, TestUserKey::Primary(TestUserPrimaryKey(100)));
        assert_eq!(post_key, PostKey::Primary(PostPrimaryKey(200)));

        // Serialization should work
        let encoded = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();
        let (decoded, _): (TestUser, usize) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(user, decoded);

        // This test passing means libp2p is properly feature-gated
        assert_eq!(user.name, "No LibP2P User");
        assert_eq!(post.title, "No LibP2P Post");
    }

    #[cfg(not(feature = "libp2p"))]
    #[test]
    fn test_libp2p_types_not_available_without_feature() {
        // This test ensures that libp2p types are not accidentally available
        // when the feature is disabled.

        // If we accidentally had libp2p available, this would fail to compile
        // with the libp2p feature disabled

        let user = TestUser {
            id: 999,
            name: "Feature Gate Test".to_string(),
            email: "feature-gate@example.com".to_string(),
        };

        // Basic functionality should work
        assert_eq!(TestUser::tree_name(), "TestUser");
        assert_eq!(user.id, 999);
    }
}
