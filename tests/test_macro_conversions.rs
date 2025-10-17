//! Comprehensive tests for Netabase macro conversions
//!
//! This test suite verifies the 8 key conversion types used in the Netabase system:
//!
//! ## Schema Conversions (for data storage/transport):
//! 1. NetabaseSchema::from(NetabaseModel) - API convenience conversions
//! 2. NetabaseSchema::try_into(NetabaseModel) - API convenience conversions
//! 3. NetabaseSchema::try_into(libp2p::kad::Record) - Kademlia communication
//! 4. NetabaseSchema::try_from(libp2p::kad::Record) - Kademlia communication
//!
//! ## Key Conversions (for key management/lookup):
//! 5. NetabaseSchemaKeys::from(NetabaseModelKey) - API convenience conversions
//! 6. NetabaseSchemaKeys::try_into(NetabaseModelKey) - API convenience conversions
//! 7. NetabaseSchemaKeys::try_into(libp2p::kad::RecordKey) - Kademlia communication
//! 8. NetabaseSchemaKeys::try_from(libp2p::kad::RecordKey) - Kademlia communication
//!
//! ## Important Design Principles:
//! - All data stored in the database should be NetabaseSchema variants, never individual models
//! - All data transported between threads should be NetabaseSchema variants
//! - Individual NetabaseModel types should only exist at API boundaries for convenience
//! - Schema keys and model keys are different wrapper types around the same underlying data
//! - They should NOT be equal to each other, but contain equivalent underlying data
//! - The Record key is always the serialized schema key, never the entire schema
//! - Conversions must be robust and handle serialization round-trips correctly

use bincode::{Decode, Encode};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::{NetabaseModel as NetabaseModelTrait, NetabaseSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test schema for comprehensive conversion testing
#[netabase_schema_module(TestConversionSchema, TestConversionKeys)]
mod test_conversion_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(UserKey)]
    pub struct User {
        #[key]
        pub user_id: u64,
        pub username: String,
        pub email: String,
        pub created_at: u64,
        pub is_active: bool,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(PostKey)]
    pub struct Post {
        #[key]
        pub post_id: String,
        pub author_id: u64,
        pub title: String,
        pub content: String,
        pub tags: Vec<String>,
        pub published_at: Option<u64>,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(CommentKey)]
    pub struct Comment {
        #[key]
        pub comment_id: u128,
        pub post_id: String,
        pub author_id: u64,
        pub content: String,
        pub replies: HashMap<String, String>,
        pub metadata: CommentMetadata,
    }

    #[derive(Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    pub struct CommentMetadata {
        pub likes: u32,
        pub reports: u32,
        pub edited: bool,
    }
}

use test_conversion_schema::{
    Comment, CommentMetadata, Post, TestConversionKeys, TestConversionSchema, User,
};

#[cfg(test)]
mod conversion_tests {
    use super::*;

    /// Test data creation helpers
    fn create_test_user() -> User {
        User {
            user_id: 12345,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            created_at: 1640995200, // 2022-01-01
            is_active: true,
        }
    }

    fn create_test_post() -> Post {
        Post {
            post_id: "post_abc123".to_string(),
            author_id: 12345,
            title: "Test Post Title".to_string(),
            content: "This is a test post with some content.".to_string(),
            tags: vec!["test".to_string(), "example".to_string()],
            published_at: Some(1640995200),
        }
    }

    fn create_test_comment() -> Comment {
        let mut replies = HashMap::new();
        replies.insert("reply1".to_string(), "First reply".to_string());
        replies.insert("reply2".to_string(), "Second reply".to_string());

        Comment {
            comment_id: 999888777666555444u128,
            post_id: "post_abc123".to_string(),
            author_id: 12345,
            content: "This is a test comment.".to_string(),
            replies,
            metadata: CommentMetadata {
                likes: 42,
                reports: 0,
                edited: true,
            },
        }
    }

    // ==================================================================================
    // SCHEMA CONVERSIONS (1-4): For data storage and transport
    // ==================================================================================

    // ==================================================================================
    // 1. NetabaseSchema::from(NetabaseModel) - API convenience conversions
    // ==================================================================================

    #[test]
    fn test_model_to_schema_from_conversion_user() {
        let user = create_test_user();
        let original_user = user.clone();

        // Test From<User> for TestConversionSchema
        let schema: TestConversionSchema = TestConversionSchema::from(user);

        match schema {
            TestConversionSchema::User(converted_user) => {
                assert_eq!(converted_user, original_user);
                assert_eq!(converted_user.user_id, 12345);
                assert_eq!(converted_user.username, "testuser");
                assert_eq!(converted_user.email, "test@example.com");
                assert_eq!(converted_user.created_at, 1640995200);
                assert!(converted_user.is_active);
            }
            _ => panic!("User model was not converted to User schema variant"),
        }
    }

    #[test]
    fn test_model_to_schema_from_conversion_post() {
        let post = create_test_post();
        let original_post = post.clone();

        // Test From<Post> for TestConversionSchema
        let schema: TestConversionSchema = TestConversionSchema::from(post);

        match schema {
            TestConversionSchema::Post(converted_post) => {
                assert_eq!(converted_post, original_post);
                assert_eq!(converted_post.post_id, "post_abc123");
                assert_eq!(converted_post.author_id, 12345);
                assert_eq!(converted_post.title, "Test Post Title");
                assert_eq!(converted_post.tags.len(), 2);
                assert!(converted_post.tags.contains(&"test".to_string()));
                assert!(converted_post.tags.contains(&"example".to_string()));
            }
            _ => panic!("Post model was not converted to Post schema variant"),
        }
    }

    #[test]
    fn test_model_to_schema_from_conversion_comment() {
        let comment = create_test_comment();
        let original_comment = comment.clone();

        // Test From<Comment> for TestConversionSchema
        let schema: TestConversionSchema = TestConversionSchema::from(comment);

        match schema {
            TestConversionSchema::Comment(converted_comment) => {
                assert_eq!(converted_comment, original_comment);
                assert_eq!(converted_comment.comment_id, 999888777666555444u128);
                assert_eq!(converted_comment.post_id, "post_abc123");
                assert_eq!(converted_comment.replies.len(), 2);
                assert_eq!(converted_comment.metadata.likes, 42);
                assert_eq!(converted_comment.metadata.reports, 0);
                assert!(converted_comment.metadata.edited);
            }
            _ => panic!("Comment model was not converted to Comment schema variant"),
        }
    }

    #[test]
    fn test_model_to_schema_from_conversion_with_into() {
        let user = create_test_user();
        let post = create_test_post();
        let comment = create_test_comment();

        // Test using .into() which calls From
        let user_schema: TestConversionSchema = user.clone().into();
        let post_schema: TestConversionSchema = post.clone().into();
        let comment_schema: TestConversionSchema = comment.clone().into();

        // Verify all conversions
        assert!(matches!(user_schema, TestConversionSchema::User(_)));
        assert!(matches!(post_schema, TestConversionSchema::Post(_)));
        assert!(matches!(comment_schema, TestConversionSchema::Comment(_)));
    }

    // ==================================================================================
    // 2. NetabaseSchema::try_into(NetabaseModel) - API convenience conversions
    // ==================================================================================

    #[test]
    fn test_schema_to_model_try_into_conversion_user() {
        let user = create_test_user();
        let schema: TestConversionSchema = TestConversionSchema::from(user.clone());

        // Test TryInto<User> for TestConversionSchema
        let converted_user: Result<User, _> = schema.try_into();

        match converted_user {
            Ok(extracted_user) => {
                assert_eq!(extracted_user, user);
                assert_eq!(extracted_user.user_id, 12345);
                assert_eq!(extracted_user.username, "testuser");
            }
            Err(_) => panic!("Failed to convert schema back to User model"),
        }
    }

    #[test]
    fn test_schema_to_model_try_into_conversion_post() {
        let post = create_test_post();
        let schema: TestConversionSchema = TestConversionSchema::from(post.clone());

        // Test TryInto<Post> for TestConversionSchema
        let converted_post: Result<Post, _> = schema.try_into();

        match converted_post {
            Ok(extracted_post) => {
                assert_eq!(extracted_post, post);
                assert_eq!(extracted_post.post_id, "post_abc123");
                assert_eq!(extracted_post.author_id, 12345);
            }
            Err(_) => panic!("Failed to convert schema back to Post model"),
        }
    }

    #[test]
    fn test_schema_to_model_try_into_conversion_comment() {
        let comment = create_test_comment();
        let schema: TestConversionSchema = TestConversionSchema::from(comment.clone());

        // Test TryInto<Comment> for TestConversionSchema
        let converted_comment: Result<Comment, _> = schema.try_into();

        match converted_comment {
            Ok(extracted_comment) => {
                assert_eq!(extracted_comment, comment);
                assert_eq!(extracted_comment.comment_id, 999888777666555444u128);
                assert_eq!(extracted_comment.post_id, "post_abc123");
            }
            Err(_) => panic!("Failed to convert schema back to Comment model"),
        }
    }

    #[test]
    fn test_schema_to_model_try_into_wrong_variant() {
        let user = create_test_user();
        let schema: TestConversionSchema = TestConversionSchema::from(user);

        // Try to extract a Post from a User schema - should fail
        let wrong_conversion: Result<Post, _> = schema.try_into();
        assert!(
            wrong_conversion.is_err(),
            "Should fail when extracting wrong variant"
        );

        // Create a new schema for the next test
        let post = create_test_post();
        let schema: TestConversionSchema = TestConversionSchema::from(post);

        // Try to extract a Comment from a Post schema - should fail
        let wrong_conversion: Result<Comment, _> = schema.try_into();
        assert!(
            wrong_conversion.is_err(),
            "Should fail when extracting wrong variant"
        );
    }

    #[test]
    fn test_roundtrip_model_schema_model() {
        let original_user = create_test_user();
        let original_post = create_test_post();
        let original_comment = create_test_comment();

        // User roundtrip: Model -> Schema -> Model
        let user_schema: TestConversionSchema = original_user.clone().into();
        let recovered_user: User = user_schema.try_into().expect("User roundtrip failed");
        assert_eq!(recovered_user, original_user);

        // Post roundtrip: Model -> Schema -> Model
        let post_schema: TestConversionSchema = original_post.clone().into();
        let recovered_post: Post = post_schema.try_into().expect("Post roundtrip failed");
        assert_eq!(recovered_post, original_post);

        // Comment roundtrip: Model -> Schema -> Model
        let comment_schema: TestConversionSchema = original_comment.clone().into();
        let recovered_comment: Comment =
            comment_schema.try_into().expect("Comment roundtrip failed");
        assert_eq!(recovered_comment, original_comment);
    }

    // ==================================================================================
    // 3. & 4. Schema Kademlia Record Conversions (libp2p feature required)
    // ==================================================================================

    #[cfg(feature = "libp2p")]
    mod schema_kademlia_record_tests {
        use super::*;
        use libp2p::kad::Record;

        #[test]
        fn test_schema_to_record_conversion_user() {
            let user = create_test_user();
            let schema: TestConversionSchema = TestConversionSchema::from(user.clone());

            // Test TryInto<Record> for TestConversionSchema
            let record_result: Result<Record, _> = schema.clone().try_into();

            match record_result {
                Ok(record) => {
                    // Verify the record was created properly
                    assert!(!record.value.is_empty(), "Record value should not be empty");
                    assert!(
                        !record.key.to_vec().is_empty(),
                        "Record key should not be empty"
                    );

                    // The key should be the serialized schema key, not the entire schema
                    let schema_key = schema.keys();
                    let expected_key_bytes =
                        bincode::encode_to_vec(&schema_key, bincode::config::standard())
                            .expect("Failed to serialize schema key");
                    assert_eq!(
                        record.key.to_vec(),
                        expected_key_bytes,
                        "Record key should be serialized schema key"
                    );

                    // The value should be the serialized schema
                    let expected_value_bytes =
                        bincode::encode_to_vec(&schema, bincode::config::standard())
                            .expect("Failed to serialize schema");
                    assert_eq!(
                        record.value, expected_value_bytes,
                        "Record value should be serialized schema"
                    );
                }
                Err(e) => panic!("Failed to convert schema to record: {:?}", e),
            }
        }

        #[test]
        fn test_schema_to_record_conversion_post() {
            let post = create_test_post();
            let schema: TestConversionSchema = TestConversionSchema::from(post);

            // Test TryInto<Record> for TestConversionSchema
            let record_result: Result<Record, _> = schema.try_into();

            assert!(
                record_result.is_ok(),
                "Post schema to record conversion should succeed"
            );
            let record = record_result.unwrap();
            assert!(!record.value.is_empty());
            assert!(!record.key.to_vec().is_empty());
        }

        #[test]
        fn test_schema_to_record_conversion_comment() {
            let comment = create_test_comment();
            let schema: TestConversionSchema = TestConversionSchema::from(comment);

            // Test TryInto<Record> for TestConversionSchema
            let record_result: Result<Record, _> = schema.try_into();

            assert!(
                record_result.is_ok(),
                "Comment schema to record conversion should succeed"
            );
            let record = record_result.unwrap();
            assert!(!record.value.is_empty());
            assert!(!record.key.to_vec().is_empty());
        }

        #[test]
        fn test_record_to_schema_conversion_user() {
            let user = create_test_user();
            let schema: TestConversionSchema = TestConversionSchema::from(user.clone());

            // Convert to record first
            let record: Record = schema
                .clone()
                .try_into()
                .expect("Failed to convert schema to record");

            // Test TryFrom<Record> for TestConversionSchema
            let schema_result: Result<TestConversionSchema, _> = record.try_into();

            match schema_result {
                Ok(recovered_schema) => {
                    // Should be the same as the original schema
                    assert_eq!(recovered_schema, schema);

                    // Extract the user and verify it matches
                    let recovered_user: User = recovered_schema
                        .try_into()
                        .expect("Failed to extract user from recovered schema");
                    assert_eq!(recovered_user, user);
                }
                Err(e) => panic!("Failed to convert record back to schema: {:?}", e),
            }
        }

        #[test]
        fn test_record_to_schema_conversion_post() {
            let post = create_test_post();
            let schema: TestConversionSchema = TestConversionSchema::from(post.clone());

            // Convert to record and back
            let record: Record = schema
                .clone()
                .try_into()
                .expect("Failed to convert schema to record");
            let recovered_schema: TestConversionSchema = record
                .try_into()
                .expect("Failed to convert record back to schema");

            assert_eq!(recovered_schema, schema);
            let recovered_post: Post = recovered_schema
                .try_into()
                .expect("Failed to extract post from recovered schema");
            assert_eq!(recovered_post, post);
        }

        #[test]
        fn test_record_to_schema_conversion_comment() {
            let comment = create_test_comment();
            let schema: TestConversionSchema = TestConversionSchema::from(comment.clone());

            // Convert to record and back
            let record: Record = schema
                .clone()
                .try_into()
                .expect("Failed to convert schema to record");
            let recovered_schema: TestConversionSchema = record
                .try_into()
                .expect("Failed to convert record back to schema");

            assert_eq!(recovered_schema, schema);
            let recovered_comment: Comment = recovered_schema
                .try_into()
                .expect("Failed to extract comment from recovered schema");
            assert_eq!(recovered_comment, comment);
        }

        #[test]
        fn test_kademlia_record_roundtrip_all_models() {
            let user = create_test_user();
            let post = create_test_post();
            let comment = create_test_comment();

            // Test User: Model -> Schema -> Record -> Schema -> Model
            let user_schema: TestConversionSchema = user.clone().into();
            let user_record: Record = user_schema
                .clone()
                .try_into()
                .expect("User schema to record failed");
            let recovered_user_schema: TestConversionSchema = user_record
                .try_into()
                .expect("User record to schema failed");
            let recovered_user: User = recovered_user_schema
                .try_into()
                .expect("User schema to model failed");
            assert_eq!(recovered_user, user);

            // Test Post: Model -> Schema -> Record -> Schema -> Model
            let post_schema: TestConversionSchema = post.clone().into();
            let post_record: Record = post_schema
                .clone()
                .try_into()
                .expect("Post schema to record failed");
            let recovered_post_schema: TestConversionSchema = post_record
                .try_into()
                .expect("Post record to schema failed");
            let recovered_post: Post = recovered_post_schema
                .try_into()
                .expect("Post schema to model failed");
            assert_eq!(recovered_post, post);

            // Test Comment: Model -> Schema -> Record -> Schema -> Model
            let comment_schema: TestConversionSchema = comment.clone().into();
            let comment_record: Record = comment_schema
                .clone()
                .try_into()
                .expect("Comment schema to record failed");
            let recovered_comment_schema: TestConversionSchema = comment_record
                .try_into()
                .expect("Comment record to schema failed");
            let recovered_comment: Comment = recovered_comment_schema
                .try_into()
                .expect("Comment schema to model failed");
            assert_eq!(recovered_comment, comment);
        }

        #[test]
        fn test_record_key_generation_is_consistent() {
            let user = create_test_user();
            let schema: TestConversionSchema = TestConversionSchema::from(user);

            // Convert to record multiple times
            let record1: Record = schema.clone().try_into().unwrap();
            let record2: Record = schema.clone().try_into().unwrap();
            let record3: Record = schema.clone().try_into().unwrap();

            // All records should have the same key
            assert_eq!(record1.key, record2.key);
            assert_eq!(record2.key, record3.key);
            assert_eq!(record1.key, record3.key);

            // All records should have the same value
            assert_eq!(record1.value, record2.value);
            assert_eq!(record2.value, record3.value);
            assert_eq!(record1.value, record3.value);
        }

        #[test]
        fn test_record_key_uses_schema_key_not_entire_schema() {
            let user = create_test_user();
            let post = create_test_post();

            let user_schema: TestConversionSchema = TestConversionSchema::from(user);
            let post_schema: TestConversionSchema = TestConversionSchema::from(post);

            // Get the schema keys
            let user_key = user_schema.keys();
            let post_key = post_schema.keys();

            // Convert schemas to records
            let user_record: Record = user_schema.clone().try_into().unwrap();
            let post_record: Record = post_schema.clone().try_into().unwrap();

            // Verify record keys match the serialized schema keys
            let user_key_bytes =
                bincode::encode_to_vec(&user_key, bincode::config::standard()).unwrap();
            let post_key_bytes =
                bincode::encode_to_vec(&post_key, bincode::config::standard()).unwrap();

            assert_eq!(user_record.key.to_vec(), user_key_bytes);
            assert_eq!(post_record.key.to_vec(), post_key_bytes);

            // Verify record keys are different (since they're different models)
            assert_ne!(user_record.key, post_record.key);

            // Verify record keys are NOT the entire serialized schema
            let user_schema_bytes =
                bincode::encode_to_vec(&user_schema, bincode::config::standard()).unwrap();
            let post_schema_bytes =
                bincode::encode_to_vec(&post_schema, bincode::config::standard()).unwrap();

            assert_ne!(
                user_record.key.to_vec(),
                user_schema_bytes,
                "Record key should not be the entire schema"
            );
            assert_ne!(
                post_record.key.to_vec(),
                post_schema_bytes,
                "Record key should not be the entire schema"
            );
        }

        #[test]
        fn test_corrupted_record_handling() {
            let user = create_test_user();
            let schema: TestConversionSchema = TestConversionSchema::from(user);
            let mut record: Record = schema.try_into().unwrap();

            // Corrupt the record value
            record.value = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Invalid bincode

            // Try to convert back - should fail gracefully
            let corrupted_result: Result<TestConversionSchema, _> = record.try_into();
            assert!(
                corrupted_result.is_err(),
                "Corrupted record should fail to deserialize"
            );
        }

        #[test]
        fn test_empty_record_handling() {
            // Create an empty record
            let empty_record = Record {
                key: libp2p::kad::RecordKey::new(&[]),
                value: vec![],
                publisher: None,
                expires: None,
            };

            // Try to convert - should fail gracefully
            let empty_result: Result<TestConversionSchema, _> = empty_record.try_into();
            assert!(
                empty_result.is_err(),
                "Empty record should fail to deserialize"
            );
        }
    }

    // ==================================================================================
    // KEY CONVERSIONS (5-8): For key management and lookup
    // ==================================================================================

    // ==================================================================================
    // 5. NetabaseSchemaKeys::from(NetabaseModelKey) - API convenience conversions
    // ==================================================================================

    #[test]
    fn test_model_key_to_schema_key_from_conversion_user() {
        let user = create_test_user();
        let model_key = user.key();

        // Test From<ModelKey> for SchemaKeys
        let schema_key: TestConversionKeys = TestConversionKeys::from(model_key.clone());

        // Verify it's the correct variant
        match schema_key {
            TestConversionKeys::UserKey(inner_key) => {
                assert_eq!(inner_key, model_key);
            }
            _ => panic!("Model key was not converted to correct schema key variant"),
        }
    }

    #[test]
    fn test_model_key_to_schema_key_from_conversion_post() {
        let post = create_test_post();
        let model_key = post.key();

        // Test From<ModelKey> for SchemaKeys
        let schema_key: TestConversionKeys = TestConversionKeys::from(model_key.clone());

        // Verify it's the correct variant
        match schema_key {
            TestConversionKeys::PostKey(inner_key) => {
                assert_eq!(inner_key, model_key);
            }
            _ => panic!("Model key was not converted to correct schema key variant"),
        }
    }

    #[test]
    fn test_model_key_to_schema_key_from_conversion_comment() {
        let comment = create_test_comment();
        let model_key = comment.key();

        // Test From<ModelKey> for SchemaKeys
        let schema_key: TestConversionKeys = TestConversionKeys::from(model_key.clone());

        // Verify it's the correct variant
        match schema_key {
            TestConversionKeys::CommentKey(inner_key) => {
                assert_eq!(inner_key, model_key);
            }
            _ => panic!("Model key was not converted to correct schema key variant"),
        }
    }

    #[test]
    fn test_model_key_to_schema_key_with_into() {
        let user = create_test_user();
        let post = create_test_post();
        let comment = create_test_comment();

        // Test using .into() which calls From
        let user_schema_key: TestConversionKeys = user.key().into();
        let post_schema_key: TestConversionKeys = post.key().into();
        let comment_schema_key: TestConversionKeys = comment.key().into();

        // Verify all conversions
        assert!(matches!(user_schema_key, TestConversionKeys::UserKey(_)));
        assert!(matches!(post_schema_key, TestConversionKeys::PostKey(_)));
        assert!(matches!(
            comment_schema_key,
            TestConversionKeys::CommentKey(_)
        ));
    }

    // ==================================================================================
    // 6. NetabaseSchemaKeys::try_into(NetabaseModelKey) - API convenience conversions
    // ==================================================================================

    #[test]
    fn test_schema_key_to_model_key_try_into_conversion_user() {
        let user = create_test_user();
        let model_key = user.key();
        let schema_key: TestConversionKeys = TestConversionKeys::from(model_key.clone());

        // Test TryInto<ModelKey> for SchemaKeys
        let converted_key: Result<test_conversion_schema::UserKey, _> = schema_key.try_into();

        match converted_key {
            Ok(extracted_key) => {
                assert_eq!(extracted_key, model_key);
            }
            Err(_) => panic!("Failed to convert schema key back to model key"),
        }
    }

    #[test]
    fn test_schema_key_to_model_key_try_into_conversion_post() {
        let post = create_test_post();
        let model_key = post.key();
        let schema_key: TestConversionKeys = TestConversionKeys::from(model_key.clone());

        // Test TryInto<ModelKey> for SchemaKeys
        let converted_key: Result<test_conversion_schema::PostKey, _> = schema_key.try_into();

        match converted_key {
            Ok(extracted_key) => {
                assert_eq!(extracted_key, model_key);
            }
            Err(_) => panic!("Failed to convert schema key back to model key"),
        }
    }

    #[test]
    fn test_schema_key_to_model_key_try_into_conversion_comment() {
        let comment = create_test_comment();
        let model_key = comment.key();
        let schema_key: TestConversionKeys = TestConversionKeys::from(model_key.clone());

        // Test TryInto<ModelKey> for SchemaKeys
        let converted_key: Result<test_conversion_schema::CommentKey, _> = schema_key.try_into();

        match converted_key {
            Ok(extracted_key) => {
                assert_eq!(extracted_key, model_key);
            }
            Err(_) => panic!("Failed to convert schema key back to model key"),
        }
    }

    #[test]
    fn test_schema_key_to_model_key_try_into_wrong_variant() {
        let user = create_test_user();
        let model_key = user.key();
        let schema_key: TestConversionKeys = TestConversionKeys::from(model_key);

        // Try to extract a PostKey from a UserKey schema - should fail
        let wrong_conversion: Result<test_conversion_schema::PostKey, _> = schema_key.try_into();
        assert!(
            wrong_conversion.is_err(),
            "Should fail when extracting wrong key variant"
        );
    }

    #[test]
    fn test_roundtrip_model_key_schema_key_model_key() {
        let original_user = create_test_user();
        let original_post = create_test_post();
        let original_comment = create_test_comment();

        // User key roundtrip: ModelKey -> SchemaKey -> ModelKey
        let user_model_key = original_user.key();
        let user_schema_key: TestConversionKeys = user_model_key.clone().into();
        let recovered_user_key: test_conversion_schema::UserKey = user_schema_key
            .try_into()
            .expect("User key roundtrip failed");
        assert_eq!(recovered_user_key, user_model_key);

        // Post key roundtrip: ModelKey -> SchemaKey -> ModelKey
        let post_model_key = original_post.key();
        let post_schema_key: TestConversionKeys = post_model_key.clone().into();
        let recovered_post_key: test_conversion_schema::PostKey = post_schema_key
            .try_into()
            .expect("Post key roundtrip failed");
        assert_eq!(recovered_post_key, post_model_key);

        // Comment key roundtrip: ModelKey -> SchemaKey -> ModelKey
        let comment_model_key = original_comment.key();
        let comment_schema_key: TestConversionKeys = comment_model_key.clone().into();
        let recovered_comment_key: test_conversion_schema::CommentKey = comment_schema_key
            .try_into()
            .expect("Comment key roundtrip failed");
        assert_eq!(recovered_comment_key, comment_model_key);
    }

    // ==================================================================================
    // 7. & 8. Key Kademlia RecordKey Conversions (libp2p feature required)
    // ==================================================================================

    #[cfg(feature = "libp2p")]
    mod key_kademlia_record_tests {
        use super::*;
        use libp2p::kad::RecordKey;

        #[test]
        fn test_schema_key_to_record_key_conversion_user() {
            let user = create_test_user();
            let model_key = user.key();
            let schema_key: TestConversionKeys = TestConversionKeys::from(model_key);

            // Test TryInto<RecordKey> for SchemaKeys
            let record_key_result: Result<RecordKey, _> = schema_key.clone().try_into();

            match record_key_result {
                Ok(record_key) => {
                    // Verify the record key was created properly
                    assert!(
                        !record_key.to_vec().is_empty(),
                        "RecordKey should not be empty"
                    );

                    // The RecordKey should be the serialized schema key
                    let expected_key_bytes =
                        bincode::encode_to_vec(&schema_key, bincode::config::standard())
                            .expect("Failed to serialize schema key");
                    assert_eq!(
                        record_key.to_vec(),
                        expected_key_bytes,
                        "RecordKey should be serialized schema key"
                    );
                }
                Err(e) => panic!("Failed to convert schema key to RecordKey: {:?}", e),
            }
        }

        #[test]
        fn test_schema_key_to_record_key_conversion_post() {
            let post = create_test_post();
            let model_key = post.key();
            let schema_key: TestConversionKeys = TestConversionKeys::from(model_key);

            // Test TryInto<RecordKey> for SchemaKeys
            let record_key_result: Result<RecordKey, _> = schema_key.try_into();

            assert!(
                record_key_result.is_ok(),
                "Post schema key to RecordKey conversion should succeed"
            );
            let record_key = record_key_result.unwrap();
            assert!(!record_key.to_vec().is_empty());
        }

        #[test]
        fn test_schema_key_to_record_key_conversion_comment() {
            let comment = create_test_comment();
            let model_key = comment.key();
            let schema_key: TestConversionKeys = TestConversionKeys::from(model_key);

            // Test TryInto<RecordKey> for SchemaKeys
            let record_key_result: Result<RecordKey, _> = schema_key.try_into();

            assert!(
                record_key_result.is_ok(),
                "Comment schema key to RecordKey conversion should succeed"
            );
            let record_key = record_key_result.unwrap();
            assert!(!record_key.to_vec().is_empty());
        }

        #[test]
        fn test_record_key_to_schema_key_conversion_user() {
            let user = create_test_user();
            let model_key = user.key();
            let schema_key: TestConversionKeys = TestConversionKeys::from(model_key);

            // Convert to RecordKey first
            let record_key: RecordKey = schema_key
                .clone()
                .try_into()
                .expect("Failed to convert schema key to RecordKey");

            // Test TryFrom<RecordKey> for SchemaKeys
            let schema_key_result: Result<TestConversionKeys, _> = record_key.try_into();

            match schema_key_result {
                Ok(recovered_schema_key) => {
                    // Should be the same as the original schema key
                    assert_eq!(recovered_schema_key, schema_key);
                }
                Err(e) => panic!("Failed to convert RecordKey back to schema key: {:?}", e),
            }
        }

        #[test]
        fn test_record_key_to_schema_key_conversion_post() {
            let post = create_test_post();
            let model_key = post.key();
            let schema_key: TestConversionKeys = TestConversionKeys::from(model_key);

            // Convert to RecordKey and back
            let record_key: RecordKey = schema_key
                .clone()
                .try_into()
                .expect("Failed to convert schema key to RecordKey");
            let recovered_schema_key: TestConversionKeys = record_key
                .try_into()
                .expect("Failed to convert RecordKey back to schema key");

            assert_eq!(recovered_schema_key, schema_key);
        }

        #[test]
        fn test_record_key_to_schema_key_conversion_comment() {
            let comment = create_test_comment();
            let model_key = comment.key();
            let schema_key: TestConversionKeys = TestConversionKeys::from(model_key);

            // Convert to RecordKey and back
            let record_key: RecordKey = schema_key
                .clone()
                .try_into()
                .expect("Failed to convert schema key to RecordKey");
            let recovered_schema_key: TestConversionKeys = record_key
                .try_into()
                .expect("Failed to convert RecordKey back to schema key");

            assert_eq!(recovered_schema_key, schema_key);
        }

        #[test]
        fn test_kademlia_record_key_roundtrip_all_models() {
            let user = create_test_user();
            let post = create_test_post();
            let comment = create_test_comment();

            // Test User: ModelKey -> SchemaKey -> RecordKey -> SchemaKey -> ModelKey
            let user_model_key = user.key();
            let user_schema_key: TestConversionKeys = user_model_key.clone().into();
            let user_record_key: RecordKey = user_schema_key
                .clone()
                .try_into()
                .expect("User schema key to RecordKey failed");
            let recovered_user_schema_key: TestConversionKeys = user_record_key
                .try_into()
                .expect("User RecordKey to schema key failed");
            let recovered_user_model_key: test_conversion_schema::UserKey =
                recovered_user_schema_key
                    .try_into()
                    .expect("User schema key to model key failed");
            assert_eq!(recovered_user_model_key, user_model_key);

            // Test Post: ModelKey -> SchemaKey -> RecordKey -> SchemaKey -> ModelKey
            let post_model_key = post.key();
            let post_schema_key: TestConversionKeys = post_model_key.clone().into();
            let post_record_key: RecordKey = post_schema_key
                .clone()
                .try_into()
                .expect("Post schema key to RecordKey failed");
            let recovered_post_schema_key: TestConversionKeys = post_record_key
                .try_into()
                .expect("Post RecordKey to schema key failed");
            let recovered_post_model_key: test_conversion_schema::PostKey =
                recovered_post_schema_key
                    .try_into()
                    .expect("Post schema key to model key failed");
            assert_eq!(recovered_post_model_key, post_model_key);

            // Test Comment: ModelKey -> SchemaKey -> RecordKey -> SchemaKey -> ModelKey
            let comment_model_key = comment.key();
            let comment_schema_key: TestConversionKeys = comment_model_key.clone().into();
            let comment_record_key: RecordKey = comment_schema_key
                .clone()
                .try_into()
                .expect("Comment schema key to RecordKey failed");
            let recovered_comment_schema_key: TestConversionKeys = comment_record_key
                .try_into()
                .expect("Comment RecordKey to schema key failed");
            let recovered_comment_model_key: test_conversion_schema::CommentKey =
                recovered_comment_schema_key
                    .try_into()
                    .expect("Comment schema key to model key failed");
            assert_eq!(recovered_comment_model_key, comment_model_key);
        }

        #[test]
        fn test_record_key_generation_is_consistent() {
            let user = create_test_user();
            let model_key = user.key();
            let schema_key: TestConversionKeys = TestConversionKeys::from(model_key);

            // Convert to RecordKey multiple times
            let record_key1: RecordKey = schema_key.clone().try_into().unwrap();
            let record_key2: RecordKey = schema_key.clone().try_into().unwrap();
            let record_key3: RecordKey = schema_key.clone().try_into().unwrap();

            // All RecordKeys should be the same
            assert_eq!(record_key1, record_key2);
            assert_eq!(record_key2, record_key3);
            assert_eq!(record_key1, record_key3);
        }

        #[test]
        fn test_corrupted_record_key_handling() {
            // Create a corrupted RecordKey
            let corrupted_record_key = RecordKey::new(&[0xFF, 0xFF, 0xFF, 0xFF]); // Invalid bincode

            // Try to convert back - should fail gracefully
            let corrupted_result: Result<TestConversionKeys, _> = corrupted_record_key.try_into();
            assert!(
                corrupted_result.is_err(),
                "Corrupted RecordKey should fail to deserialize"
            );
        }

        #[test]
        fn test_empty_record_key_handling() {
            // Create an empty RecordKey
            let empty_record_key = RecordKey::new(&[]);

            // Try to convert - should fail gracefully
            let empty_result: Result<TestConversionKeys, _> = empty_record_key.try_into();
            assert!(
                empty_result.is_err(),
                "Empty RecordKey should fail to deserialize"
            );
        }
    }

    // ==================================================================================
    // Key Consistency and Integration Tests
    // ==================================================================================

    #[test]
    fn test_key_relationships_are_consistent() {
        let user = create_test_user();
        let post = create_test_post();
        let comment = create_test_comment();

        // Get keys directly from models
        let user_model_key = user.key();
        let post_model_key = post.key();
        let comment_model_key = comment.key();

        // Convert to schemas and get keys
        let user_schema: TestConversionSchema = user.into();
        let post_schema: TestConversionSchema = post.into();
        let comment_schema: TestConversionSchema = comment.into();

        let user_schema_key = user_schema.keys();
        let post_schema_key = post_schema.keys();
        let comment_schema_key = comment_schema.keys();

        // The schema keys should contain the model keys as inner values
        match user_schema_key {
            TestConversionKeys::UserKey(inner_key) => {
                assert_eq!(
                    inner_key, user_model_key,
                    "User schema key should contain model key"
                );
            }
            _ => panic!("User schema key should be UserKey variant"),
        }

        match post_schema_key {
            TestConversionKeys::PostKey(inner_key) => {
                assert_eq!(
                    inner_key, post_model_key,
                    "Post schema key should contain model key"
                );
            }
            _ => panic!("Post schema key should be PostKey variant"),
        }

        match comment_schema_key {
            TestConversionKeys::CommentKey(inner_key) => {
                assert_eq!(
                    inner_key, comment_model_key,
                    "Comment schema key should contain model key"
                );
            }
            _ => panic!("Comment schema key should be CommentKey variant"),
        }
    }

    #[test]
    fn test_different_model_keys_have_different_schema_keys() {
        let user = create_test_user();
        let post = create_test_post();
        let comment = create_test_comment();

        let user_schema_key: TestConversionKeys = user.key().into();
        let post_schema_key: TestConversionKeys = post.key().into();
        let comment_schema_key: TestConversionKeys = comment.key().into();

        // Serialize keys for comparison
        let user_key_bytes =
            bincode::encode_to_vec(&user_schema_key, bincode::config::standard()).unwrap();
        let post_key_bytes =
            bincode::encode_to_vec(&post_schema_key, bincode::config::standard()).unwrap();
        let comment_key_bytes =
            bincode::encode_to_vec(&comment_schema_key, bincode::config::standard()).unwrap();

        // All keys should be different
        assert_ne!(user_key_bytes, post_key_bytes);
        assert_ne!(post_key_bytes, comment_key_bytes);
        assert_ne!(user_key_bytes, comment_key_bytes);
    }

    #[test]
    fn test_schema_keys_and_model_keys_are_not_equal() {
        let user = create_test_user();
        let model_key = user.key();
        let schema_key: TestConversionKeys = TestConversionKeys::from(model_key.clone());

        // The types are different, so they can't be equal
        // This test verifies that we don't accidentally make them equal in some way
        let model_key_bytes =
            bincode::encode_to_vec(&model_key, bincode::config::standard()).unwrap();
        let schema_key_bytes =
            bincode::encode_to_vec(&schema_key, bincode::config::standard()).unwrap();

        // The serialized forms should be different because schema key is wrapped in an enum
        assert_ne!(
            model_key_bytes, schema_key_bytes,
            "Schema keys and model keys should have different serialized forms"
        );
    }

    // ==================================================================================
    // Edge Case and Performance Tests
    // ==================================================================================

    #[test]
    fn test_large_data_conversion() {
        // Create a comment with large data
        let mut large_replies = HashMap::new();
        for i in 0..1000 {
            large_replies.insert(
                format!("reply_{}", i),
                format!("This is a long reply content for reply number {} with lots of text to make it substantial", i)
            );
        }

        let large_comment = Comment {
            comment_id: 999888777666555444u128,
            post_id: "post_with_many_replies".to_string(),
            author_id: 12345,
            content: "A".repeat(10000), // Large content
            replies: large_replies,
            metadata: CommentMetadata {
                likes: u32::MAX,
                reports: 0,
                edited: true,
            },
        };

        // Test schema conversion with large data
        let schema: TestConversionSchema = TestConversionSchema::from(large_comment.clone());
        let recovered_comment: Comment = schema
            .try_into()
            .expect("Large comment conversion should succeed");

        assert_eq!(recovered_comment, large_comment);
        assert_eq!(recovered_comment.replies.len(), 1000);
        assert_eq!(recovered_comment.content.len(), 10000);

        // Test key conversion with large data
        let model_key = large_comment.key();
        let schema_key: TestConversionKeys = TestConversionKeys::from(model_key.clone());
        let recovered_model_key: test_conversion_schema::CommentKey = schema_key
            .try_into()
            .expect("Large comment key conversion should succeed");

        assert_eq!(recovered_model_key, model_key);
    }

    #[test]
    fn test_edge_case_values() {
        // Test with edge case values
        let edge_user = User {
            user_id: u64::MAX,
            username: "".to_string(), // Empty string
            email: "a".repeat(1000),  // Very long email
            created_at: 0,            // Epoch
            is_active: false,
        };

        let edge_post = Post {
            post_id: "".to_string(),       // Empty ID
            author_id: 0,                  // Zero ID
            title: "🚀🎉💯".to_string(),   // Unicode
            content: "\n\t\r".to_string(), // Whitespace
            tags: vec![],                  // Empty vec
            published_at: None,            // None option
        };

        // Test schema conversions with edge cases
        let user_schema: TestConversionSchema = TestConversionSchema::from(edge_user.clone());
        let post_schema: TestConversionSchema = TestConversionSchema::from(edge_post.clone());

        let recovered_user: User = user_schema.try_into().unwrap();
        let recovered_post: Post = post_schema.try_into().unwrap();

        assert_eq!(recovered_user, edge_user);
        assert_eq!(recovered_post, edge_post);

        // Test key conversions with edge cases
        let user_model_key = edge_user.key();
        let post_model_key = edge_post.key();

        let user_schema_key: TestConversionKeys = TestConversionKeys::from(user_model_key.clone());
        let post_schema_key: TestConversionKeys = TestConversionKeys::from(post_model_key.clone());

        let recovered_user_key: test_conversion_schema::UserKey =
            user_schema_key.try_into().unwrap();
        let recovered_post_key: test_conversion_schema::PostKey =
            post_schema_key.try_into().unwrap();

        assert_eq!(recovered_user_key, user_model_key);
        assert_eq!(recovered_post_key, post_model_key);
    }

    #[test]
    fn test_multiple_conversion_cycles() {
        let mut user = create_test_user();

        // Perform multiple conversion cycles for schemas
        for i in 0..10 {
            user.created_at += i; // Slightly modify the data

            // Model -> Schema -> Model -> Schema -> Model
            let schema1: TestConversionSchema = TestConversionSchema::from(user.clone());
            let user1: User = schema1.try_into().unwrap();
            let schema2: TestConversionSchema = TestConversionSchema::from(user1.clone());
            let user2: User = schema2.try_into().unwrap();

            // Should remain consistent
            assert_eq!(user, user1);
            assert_eq!(user1, user2);
            assert_eq!(user, user2);

            user = user2; // Use the recovered user for the next iteration
        }

        // Perform multiple conversion cycles for keys
        let mut model_key = user.key();
        for _i in 0..10 {
            // ModelKey -> SchemaKey -> ModelKey -> SchemaKey -> ModelKey
            let schema_key1: TestConversionKeys = TestConversionKeys::from(model_key.clone());
            let model_key1: test_conversion_schema::UserKey = schema_key1.try_into().unwrap();
            let schema_key2: TestConversionKeys = TestConversionKeys::from(model_key1.clone());
            let model_key2: test_conversion_schema::UserKey = schema_key2.try_into().unwrap();

            // Should remain consistent
            assert_eq!(model_key, model_key1);
            assert_eq!(model_key1, model_key2);
            assert_eq!(model_key, model_key2);

            model_key = model_key2; // Use the recovered key for the next iteration
        }
    }

    #[test]
    fn test_serialization_consistency_across_conversions() {
        let user = create_test_user();
        let schema: TestConversionSchema = TestConversionSchema::from(user.clone());

        // Serialize the schema directly
        let direct_schema_bytes =
            bincode::encode_to_vec(&schema, bincode::config::standard()).unwrap();

        // Convert schema back to model and then to schema again
        let recovered_user: User = schema.clone().try_into().unwrap();
        let recovered_schema: TestConversionSchema = TestConversionSchema::from(recovered_user);
        let recovered_schema_bytes =
            bincode::encode_to_vec(&recovered_schema, bincode::config::standard()).unwrap();

        // Should be identical
        assert_eq!(
            direct_schema_bytes, recovered_schema_bytes,
            "Schema serialization should be consistent across conversions"
        );

        // Test key serialization consistency
        let model_key = user.key();
        let schema_key: TestConversionKeys = TestConversionKeys::from(model_key.clone());

        // Serialize the schema key directly
        let direct_key_bytes =
            bincode::encode_to_vec(&schema_key, bincode::config::standard()).unwrap();

        // Convert schema key back to model key and then to schema key again
        let recovered_model_key: test_conversion_schema::UserKey =
            schema_key.clone().try_into().unwrap();
        let recovered_schema_key: TestConversionKeys =
            TestConversionKeys::from(recovered_model_key);
        let recovered_key_bytes =
            bincode::encode_to_vec(&recovered_schema_key, bincode::config::standard()).unwrap();

        // Should be identical
        assert_eq!(
            direct_key_bytes, recovered_key_bytes,
            "Key serialization should be consistent across conversions"
        );
    }

    #[test]
    fn test_wrong_variant_extraction_errors() {
        let user = create_test_user();
        let user_schema: TestConversionSchema = TestConversionSchema::from(user.clone());

        // Try to extract wrong types from schema - all should fail
        let post_result: Result<Post, _> = user_schema.clone().try_into();
        let comment_result: Result<Comment, _> = user_schema.clone().try_into();

        assert!(
            post_result.is_err(),
            "Should not be able to extract Post from User schema"
        );
        assert!(
            comment_result.is_err(),
            "Should not be able to extract Comment from User schema"
        );

        // Try to extract wrong types from schema key - all should fail
        let user_model_key = user.key();
        let user_schema_key: TestConversionKeys = TestConversionKeys::from(user_model_key);

        let post_key_result: Result<test_conversion_schema::PostKey, _> =
            user_schema_key.clone().try_into();
        let comment_key_result: Result<test_conversion_schema::CommentKey, _> =
            user_schema_key.try_into();

        assert!(
            post_key_result.is_err(),
            "Should not be able to extract PostKey from UserKey schema"
        );
        assert!(
            comment_key_result.is_err(),
            "Should not be able to extract CommentKey from UserKey schema"
        );
    }
}
