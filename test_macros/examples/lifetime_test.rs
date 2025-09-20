//! Standalone example demonstrating that the lifetime issues in the iterator have been resolved.
//!
//! This example shows that:
//! 1. The SocialMediaSchemaDBIter can be created with proper lifetime management
//! 2. Individual scan methods work correctly with Vec<T> return types
//! 3. The unified scan_all_types method works and returns the correct enum variants
//! 4. From/TryFrom conversions work correctly between individual types and enum variants
//! 5. Reference conversions work correctly between owned and reference enums

use chrono::{DateTime, Utc};
use native_db::{Builder, Models};
use std::collections::HashMap;
use std::sync::LazyLock;
use test_macros::social_data::v1::{Post, PrimitiveTest, User};
use test_macros::{SocialMediaSchema, SocialMediaSchemaDBIter, SocialMediaSchemaRef};

static MODELS: LazyLock<Models> = LazyLock::new(|| {
    let mut models = Models::new();
    models.define::<User>().unwrap();
    models.define::<Post>().unwrap();
    models.define::<PrimitiveTest>().unwrap();
    models
});

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing lifetime resolution in iterator...");

    // Create an in-memory database
    let db = Builder::new().create_in_memory(&MODELS)?;
    println!("✓ Database created successfully");

    // Create test data
    let user = User {
        id: "user1".to_string(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        display_name: Some("Test User".to_string()),
        bio: Some("A test user".to_string()),
        avatar_url: None,
        cover_url: None,
        created_at: DateTime::from_timestamp(1234567890, 0).unwrap(),
        updated_at: DateTime::from_timestamp(1234567890, 0).unwrap(),
        birth_timestamp: Some(DateTime::from_timestamp(946684800, 0).unwrap()), // Y2K
        last_active: DateTime::from_timestamp(1234567890, 0).unwrap(),
        followers_count: 10,
        following_count: 5,
        posts_count: 3,
        age: Some(25),
        is_verified: false,
        is_private: false,
        is_active: true,
        allow_messages: true,
        interests: vec!["rust".to_string(), "programming".to_string()],
        languages: vec!["en".to_string()],
        settings: HashMap::new(),
    };

    let post = Post {
        id: "post1".to_string(),
        user_id: "user1".to_string(),
        created_at: 1234567890,
        content: "Hello, world! 🦀".to_string(),
        updated_at: Some(DateTime::from_timestamp(1234567890, 0).unwrap()),
        media_urls: vec![],
        hashtags: vec!["hello".to_string(), "rust".to_string()],
        mentions: vec!["@testuser".to_string()],
        likes_count: 42,
        comments_count: 3,
        shares_count: 1,
        views_count: 100,
        is_public: true,
        allow_comments: true,
        allow_shares: true,
        latitude: Some(37.7749),
        longitude: Some(-122.4194),
        location_name: Some("San Francisco, CA".to_string()),
    };

    let primitive_test = PrimitiveTest {
        id: "test1".to_string(),
        is_active: true,
        is_verified: false,
        byte_value: 42,
        short_value: 1000,
        int_value: 100000,
        long_value: 1000000000,
        huge_value: 1000000000000000000,
        ubyte_value: 200,
        ushort_value: 50000,
        uint_value: 4000000000,
        ulong_value: 18000000000000000000,
        uhuge_value: 340282366920938463463374607431768211455,
        float_value: 3.14159,
        double_value: 2.718281828,
        char_value: '🦀',
        text: "Hello, Rust! 🦀".to_string(),
        optional_number: Some(42),
        optional_text: Some("This is optional text".to_string()),
        tags: vec![
            "test".to_string(),
            "example".to_string(),
            "rust".to_string(),
        ],
        metadata: HashMap::from([
            ("author".to_string(), "test_user".to_string()),
            ("version".to_string(), "1.0.0".to_string()),
            ("language".to_string(), "rust".to_string()),
        ]),
    };

    // Insert data into the database
    {
        let rw = db.rw_transaction()?;
        rw.insert(user.clone())?;
        rw.insert(post.clone())?;
        rw.insert(primitive_test.clone())?;
        rw.commit()?;
    }
    println!("✓ Test data inserted successfully");

    // Test the iterator functionality - this demonstrates resolved lifetime issues
    let iter = SocialMediaSchemaDBIter::new(&db);
    println!("✓ Iterator created successfully (lifetimes resolved!)");

    // Test individual scan methods
    println!("\n--- Testing Individual Scan Methods ---");

    let users = iter.scan_user()?;
    println!("✓ Found {} users", users.len());
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, "user1");
    assert_eq!(users[0].username, "testuser");

    let posts = iter.scan_post()?;
    println!("✓ Found {} posts", posts.len());
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, "post1");
    assert_eq!(posts[0].content, "Hello, world! 🦀");

    let primitive_tests = iter.scan_primitivetest()?;
    println!("✓ Found {} primitive tests", primitive_tests.len());
    assert_eq!(primitive_tests.len(), 1);
    assert_eq!(primitive_tests[0].id, "test1");
    assert_eq!(primitive_tests[0].char_value, '🦀');

    // Test unified scan method
    println!("\n--- Testing Unified Scan Method ---");
    let all_items = iter.scan_all_types()?;
    println!("✓ Found {} total items across all types", all_items.len());
    assert_eq!(all_items.len(), 3);

    // Verify the items are correctly wrapped in the enum
    let mut found_user = false;
    let mut found_post = false;
    let mut found_primitive = false;

    for (i, item) in all_items.iter().enumerate() {
        match item {
            SocialMediaSchema::User(u) => {
                println!("  Item {}: User(id={})", i + 1, u.id);
                assert_eq!(u.id, "user1");
                found_user = true;
            }
            SocialMediaSchema::Post(p) => {
                println!(
                    "  Item {}: Post(id={}, content='{}')",
                    i + 1,
                    p.id,
                    p.content
                );
                assert_eq!(p.id, "post1");
                found_post = true;
            }
            SocialMediaSchema::PrimitiveTest(pt) => {
                println!(
                    "  Item {}: PrimitiveTest(id={}, char='{}')",
                    i + 1,
                    pt.id,
                    pt.char_value
                );
                assert_eq!(pt.id, "test1");
                found_primitive = true;
            }
            _ => {
                println!("  Item {}: Other variant", i + 1);
            }
        }
    }

    assert!(found_user, "User not found in unified scan");
    assert!(found_post, "Post not found in unified scan");
    assert!(found_primitive, "PrimitiveTest not found in unified scan");
    println!("✓ All expected items found in unified scan");

    // Test From/Into conversions
    println!("\n--- Testing Conversions ---");

    // Test From implementation for individual types to enum
    let user_enum: SocialMediaSchema = user.clone().into();
    match user_enum {
        SocialMediaSchema::User(u) => {
            println!("✓ User -> SocialMediaSchema conversion works");
            assert_eq!(u.id, "user1");
        }
        _ => panic!("Wrong enum variant after conversion"),
    }

    // Test reference conversion to ref enum
    let user_ref: SocialMediaSchemaRef = (&user).into();
    match user_ref {
        SocialMediaSchemaRef::User(u) => {
            println!("✓ &User -> SocialMediaSchemaRef conversion works");
            assert_eq!(u.id, "user1");
        }
        _ => panic!("Wrong ref enum variant after conversion"),
    }

    // Test conversion from base enum to ref enum
    let base_enum = SocialMediaSchema::User(user);
    let ref_enum: SocialMediaSchemaRef = (&base_enum).into();
    match ref_enum {
        SocialMediaSchemaRef::User(u) => {
            println!("✓ &SocialMediaSchema -> SocialMediaSchemaRef conversion works");
            assert_eq!(u.id, "user1");
        }
        _ => panic!("Wrong ref enum variant after base->ref conversion"),
    }

    // Test TryFrom conversions
    let post_enum = SocialMediaSchema::Post(post);
    let extracted_post: Post = post_enum
        .try_into()
        .expect("Failed to extract Post from enum");
    println!("✓ SocialMediaSchema -> Post extraction works");
    assert_eq!(extracted_post.id, "post1");

    // Test failed TryFrom
    let user_enum = SocialMediaSchema::User(User {
        id: "user2".to_string(),
        username: "user2".to_string(),
        email: "user2@example.com".to_string(),
        display_name: None,
        bio: None,
        avatar_url: None,
        cover_url: None,
        created_at: DateTime::from_timestamp(1234567890, 0).unwrap(),
        updated_at: DateTime::from_timestamp(1234567890, 0).unwrap(),
        birth_timestamp: None,
        last_active: DateTime::from_timestamp(1234567890, 0).unwrap(),
        followers_count: 0,
        following_count: 0,
        posts_count: 0,
        age: None,
        is_verified: false,
        is_private: false,
        is_active: true,
        allow_messages: true,
        interests: vec![],
        languages: vec![],
        settings: HashMap::new(),
    });

    let post_result: Result<Post, SocialMediaSchema> = user_enum.try_into();
    assert!(
        post_result.is_err(),
        "TryFrom should fail for wrong variant"
    );
    println!("✓ TryFrom correctly fails for wrong variant");

    println!("\n🎉 All tests passed! Lifetime issues have been successfully resolved!");
    println!("\nKey improvements:");
    println!(
        "  - Iterator now uses simplified lifetime '<db> instead of complex '<db: 'stack_db, 'stack_db>"
    );
    println!("  - Methods return Vec<T> instead of complex iterator types with lifetime issues");
    println!("  - From/TryFrom implementations work correctly with proper lifetime management");
    println!("  - The unified scan method efficiently collects all types into a single enum");
    println!("  - Reference enums allow zero-copy access patterns for the RecordStore trait");

    Ok(())
}
