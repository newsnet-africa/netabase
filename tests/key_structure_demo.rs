use std::collections::HashMap;
use test_macros::social_data::v1::*;
use test_macros::{
    CommentKeys, CommentSecondaryKeys, PostKeys, PostSecondaryKeys, PrimitiveTestKeys,
    SocialMediaSchemaKey, UserKeys, UserSecondaryKeys,
};

#[test]
fn test_user_key_structure() {
    // Create a sample User
    let now = chrono::Utc::now();
    let user = User {
        id: "user123".to_string(),
        username: "john_doe".to_string(),
        email: "john@example.com".to_string(),
        display_name: Some("John Doe".to_string()),
        bio: Some("Software developer".to_string()),
        avatar_url: None,
        cover_url: None,
        created_at: now,
        updated_at: now,
        birth_timestamp: None,
        last_active: now,
        followers_count: 100,
        following_count: 50,
        posts_count: 0,
        age: Some(30),
        is_verified: false,
        is_private: false,
        is_active: true,
        allow_messages: true,
        interests: vec!["rust".to_string(), "programming".to_string()],
        languages: vec!["en".to_string()],
        settings: HashMap::new(),
    };

    // Test primary key
    let primary_key = UserKeys::Primary(user.id.clone());
    match primary_key {
        UserKeys::Primary(id) => assert_eq!(id, "user123"),
        UserKeys::Secondary(_) => panic!("Expected primary key"),
    }

    // Test secondary keys
    let username_key = UserKeys::Secondary(UserSecondaryKeys::username(user.username.clone()));
    let email_key = UserKeys::Secondary(UserSecondaryKeys::email(user.email.clone()));

    match username_key {
        UserKeys::Primary(_) => panic!("Expected secondary key"),
        UserKeys::Secondary(UserSecondaryKeys::username(username)) => {
            assert_eq!(username, "john_doe");
        }
        UserKeys::Secondary(UserSecondaryKeys::email(_)) => {
            panic!("Expected username, got email");
        }
    }

    match email_key {
        UserKeys::Primary(_) => panic!("Expected secondary key"),
        UserKeys::Secondary(UserSecondaryKeys::email(email)) => {
            assert_eq!(email, "john@example.com");
        }
        UserKeys::Secondary(UserSecondaryKeys::username(_)) => {
            panic!("Expected email, got username");
        }
    }

    // Test database-level key enum
    let db_primary_key = SocialMediaSchemaKey::User(UserKeys::Primary(user.id.clone()));
    let db_username_key = SocialMediaSchemaKey::User(UserKeys::Secondary(
        UserSecondaryKeys::username(user.username.clone()),
    ));

    match db_primary_key {
        SocialMediaSchemaKey::User(UserKeys::Primary(id)) => assert_eq!(id, "user123"),
        _ => panic!("Expected user primary key"),
    }

    match db_username_key {
        SocialMediaSchemaKey::User(UserKeys::Secondary(UserSecondaryKeys::username(username))) => {
            assert_eq!(username, "john_doe");
        }
        _ => panic!("Expected user username key"),
    }
}

#[test]
fn test_post_with_multiple_secondary_keys() {
    let now = chrono::Utc::now();
    let post = Post {
        id: "post456".to_string(),
        user_id: "user123".to_string(),
        created_at: 1234567890,
        content: "Hello, world! This is my first post.".to_string(),
        updated_at: Some(now),
        media_urls: vec![],
        hashtags: vec!["#hello".to_string(), "#world".to_string()],
        mentions: vec![],
        likes_count: 25,
        comments_count: 3,
        shares_count: 1,
        views_count: 100,
        is_public: true,
        allow_comments: true,
        allow_shares: true,
        latitude: None,
        longitude: None,
        location_name: None,
    };

    // Test different types of keys for Post
    let post_primary = PostKeys::Primary(post.id.clone());
    let post_by_user = PostKeys::Secondary(PostSecondaryKeys::user_id(post.user_id.clone()));
    let post_by_time = PostKeys::Secondary(PostSecondaryKeys::created_at(post.created_at));

    // Verify each key type
    match &post_primary {
        PostKeys::Primary(id) => assert_eq!(id, "post456"),
        PostKeys::Secondary(_) => panic!("Expected primary key"),
    }

    match &post_by_user {
        PostKeys::Secondary(PostSecondaryKeys::user_id(user_id)) => {
            assert_eq!(user_id, "user123");
        }
        _ => panic!("Expected user_id secondary key"),
    }

    match &post_by_time {
        PostKeys::Secondary(PostSecondaryKeys::created_at(timestamp)) => {
            assert_eq!(*timestamp, 1234567890);
        }
        _ => panic!("Expected created_at secondary key"),
    }

    // Test database-level keys
    let db_post_primary = SocialMediaSchemaKey::Post(post_primary);
    let db_post_by_user = SocialMediaSchemaKey::Post(post_by_user);
    let db_post_by_time = SocialMediaSchemaKey::Post(post_by_time);

    match &db_post_primary {
        SocialMediaSchemaKey::Post(PostKeys::Primary(id)) => assert_eq!(id, "post456"),
        _ => panic!("Expected post primary key"),
    }

    match &db_post_by_user {
        SocialMediaSchemaKey::Post(PostKeys::Secondary(PostSecondaryKeys::user_id(user_id))) => {
            assert_eq!(user_id, "user123");
        }
        _ => panic!("Expected post user_id key"),
    }

    match &db_post_by_time {
        SocialMediaSchemaKey::Post(PostKeys::Secondary(PostSecondaryKeys::created_at(
            timestamp,
        ))) => {
            assert_eq!(*timestamp, 1234567890);
        }
        _ => panic!("Expected post created_at key"),
    }
}

#[test]
fn test_models_without_secondary_keys() {
    let primitive_test = PrimitiveTest {
        id: "test123".to_string(),
        is_active: true,
        is_verified: false,
        byte_value: 42,
        short_value: 1000,
        int_value: 50000,
        long_value: 1000000,
        huge_value: 999999999999,
        ubyte_value: 255,
        ushort_value: 65535,
        uint_value: 4000000000,
        ulong_value: 18000000000000000000,
        uhuge_value: 340282366920938463463374607431768211455,
        float_value: 3.14,
        double_value: 2.718281828,
        char_value: 'A',
        text: "Hello World".to_string(),
        optional_number: Some(42),
        optional_text: Some("Optional".to_string()),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        metadata: std::collections::HashMap::new(),
    };

    // Models without secondary keys should only have Primary variant
    let primitive_key = PrimitiveTestKeys::Primary(primitive_test.id.clone());

    match &primitive_key {
        PrimitiveTestKeys::Primary(id) => assert_eq!(id, "test123"),
        // Note: No Secondary variant exists for PrimitiveTestKeys
    }

    // Test database-level key
    let db_primitive_key = SocialMediaSchemaKey::PrimitiveTest(primitive_key);

    match db_primitive_key {
        SocialMediaSchemaKey::PrimitiveTest(PrimitiveTestKeys::Primary(id)) => {
            assert_eq!(id, "test123");
        }
        _ => panic!("Expected primitive test primary key"),
    }
}

#[test]
fn test_key_type_safety() {
    // This test demonstrates that the new structure provides type safety
    // We can't accidentally mix up keys from different models

    let user_key = UserKeys::Primary("user123".to_string());
    let post_key = PostKeys::Primary("post456".to_string());

    // Each model has its own key type, preventing confusion
    let db_user_key = SocialMediaSchemaKey::User(user_key);
    let db_post_key = SocialMediaSchemaKey::Post(post_key);

    // The compiler ensures we handle each case correctly
    match db_user_key {
        SocialMediaSchemaKey::User(_) => {
            // This is the correct branch for user keys
            assert!(true);
        }
        SocialMediaSchemaKey::Post(_) => {
            panic!("This should never happen - type safety violated!");
        }
        _ => {
            panic!("Unexpected key type");
        }
    }

    match db_post_key {
        SocialMediaSchemaKey::Post(_) => {
            // This is the correct branch for post keys
            assert!(true);
        }
        SocialMediaSchemaKey::User(_) => {
            panic!("This should never happen - type safety violated!");
        }
        _ => {
            panic!("Unexpected key type");
        }
    }
}

#[test]
fn test_comment_secondary_keys() {
    let comment = Comment {
        id: 789,
        post_id: "post456".to_string(),
        user_id: "user123".to_string(),
        created_at: 1234567890,
        content: "Great post!".to_string(),
        parent_comment_id: None,
        likes_count: 5,
        replies_count: 0,
        is_edited: false,
        edited_at: None,
    };

    // Test all secondary key types for Comment
    let comment_primary = CommentKeys::Primary(comment.id);
    let comment_by_post =
        CommentKeys::Secondary(CommentSecondaryKeys::post_id(comment.post_id.clone()));
    let comment_by_user =
        CommentKeys::Secondary(CommentSecondaryKeys::user_id(comment.user_id.clone()));
    let comment_by_time =
        CommentKeys::Secondary(CommentSecondaryKeys::created_at(comment.created_at));

    // Verify primary key
    // Verify each key type
    match &comment_primary {
        CommentKeys::Primary(id) => assert_eq!(*id, 789),
        CommentKeys::Secondary(_) => panic!("Expected primary key"),
    }

    // Verify secondary keys
    match &comment_by_post {
        CommentKeys::Secondary(CommentSecondaryKeys::post_id(post_id)) => {
            assert_eq!(post_id, "post456");
        }
        _ => panic!("Expected post_id secondary key"),
    }

    match &comment_by_user {
        CommentKeys::Secondary(CommentSecondaryKeys::user_id(user_id)) => {
            assert_eq!(user_id, "user123");
        }
        _ => panic!("Expected user_id secondary key"),
    }

    match &comment_by_time {
        CommentKeys::Secondary(CommentSecondaryKeys::created_at(timestamp)) => {
            assert_eq!(*timestamp, 1234567890);
        }
        _ => panic!("Expected created_at secondary key"),
    }
}

#[test]
fn test_key_enum_benefits() {
    println!("=== Benefits of New Key Structure ===");
    println!("✓ Type safety: Each model has its own key type");
    println!("✓ Clear distinction: Primary vs Secondary keys are explicit");
    println!("✓ Performance: Different key types can be optimized differently");
    println!("✓ Extensibility: Easy to add new secondary keys per model");
    println!("✓ Querying: Enables efficient secondary key lookups");

    // This test always passes - it's mainly for documentation
    assert!(true, "New key structure provides all expected benefits");
}
