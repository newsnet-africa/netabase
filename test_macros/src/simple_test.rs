use crate::social_data::v1::{Post, PrimitiveTest, User};
use crate::{SocialMediaSchema, SocialMediaSchemaDBIter, SocialMediaSchemaRef};
use native_db::{Builder, Models};
use std::collections::HashMap;
use std::sync::LazyLock;

static MODELS: LazyLock<Models> = LazyLock::new(|| {
    let mut models = Models::new();
    models.define::<User>().unwrap();
    models.define::<Post>().unwrap();
    models.define::<PrimitiveTest>().unwrap();
    models
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator_lifetime_resolution() {
        // Create an in-memory database
        let db = Builder::new().create_in_memory(&MODELS).unwrap();

        // Create some test data
        let user = User {
            id: "user1".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            bio: Some("A test user".to_string()),
            avatar_url: None,
            cover_url: None,
            created_at: 1234567890,
            updated_at: 1234567890,
            birth_timestamp: Some(946684800), // Y2K
            last_active: Some(1234567890),
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
            content: "Hello, world!".to_string(),
            updated_at: Some(1234567890),
            media_urls: vec![],
            hashtags: vec!["hello".to_string()],
            mentions: vec![],
            likes_count: 0,
            comments_count: 0,
            shares_count: 0,
            views_count: 0,
            is_public: true,
            allow_comments: true,
            allow_shares: true,
            latitude: None,
            longitude: None,
            location_name: None,
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
            float_value: 3.14,
            double_value: 2.718281828,
            char_value: '🦀',
            text: "Hello, Rust!".to_string(),
            optional_number: Some(42),
            optional_text: Some("Optional text".to_string()),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            metadata: HashMap::from([
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string()),
            ]),
        };

        // Insert data into database
        {
            let rw = db.rw_transaction().unwrap();
            rw.insert(user.clone()).unwrap();
            rw.insert(post.clone()).unwrap();
            rw.insert(primitive_test.clone()).unwrap();
            rw.commit().unwrap();
        }

        // Test the iterator functionality
        let iter = SocialMediaSchemaDBIter::new(&db);

        // Test individual scan methods
        let users = iter.scan_user().unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].id, "user1");

        let posts = iter.scan_post().unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, "post1");

        let primitive_tests = iter.scan_primitivetest().unwrap();
        assert_eq!(primitive_tests.len(), 1);
        assert_eq!(primitive_tests[0].id, "test1");

        // Test unified scan method
        let all_items = iter.scan_all_types().unwrap();
        assert_eq!(all_items.len(), 3);

        // Verify the items are correctly wrapped in the enum
        let mut found_user = false;
        let mut found_post = false;
        let mut found_primitive = false;

        for item in &all_items {
            match item {
                SocialMediaSchema::User(u) => {
                    assert_eq!(u.id, "user1");
                    found_user = true;
                }
                SocialMediaSchema::Post(p) => {
                    assert_eq!(p.id, "post1");
                    found_post = true;
                }
                SocialMediaSchema::PrimitiveTest(pt) => {
                    assert_eq!(pt.id, "test1");
                    found_primitive = true;
                }
                _ => {}
            }
        }

        assert!(found_user, "User not found in unified scan");
        assert!(found_post, "Post not found in unified scan");
        assert!(found_primitive, "PrimitiveTest not found in unified scan");
    }

    #[test]
    fn test_from_conversions() {
        let user = User {
            id: "user1".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            bio: Some("A test user".to_string()),
            avatar_url: None,
            cover_url: None,
            created_at: 1234567890,
            updated_at: 1234567890,
            birth_timestamp: Some(946684800),
            last_active: Some(1234567890),
            followers_count: 10,
            following_count: 5,
            posts_count: 3,
            age: Some(25),
            is_verified: false,
            is_private: false,
            is_active: true,
            allow_messages: true,
            interests: vec!["rust".to_string()],
            languages: vec!["en".to_string()],
            settings: HashMap::new(),
        };

        // Test From implementation
        let schema_enum: SocialMediaSchema = user.clone().into();
        match schema_enum {
            SocialMediaSchema::User(u) => assert_eq!(u.id, "user1"),
            _ => panic!("Wrong enum variant"),
        }

        // Test reference conversion
        let schema_ref: SocialMediaSchemaRef = (&user).into();
        match schema_ref {
            SocialMediaSchemaRef::User(u) => assert_eq!(u.id, "user1"),
            _ => panic!("Wrong enum variant"),
        }

        // Test conversion from base enum to ref enum
        let base_enum = SocialMediaSchema::User(user);
        let ref_enum: SocialMediaSchemaRef = (&base_enum).into();
        match ref_enum {
            SocialMediaSchemaRef::User(u) => assert_eq!(u.id, "user1"),
            _ => panic!("Wrong enum variant"),
        }
    }

    #[test]
    fn test_try_from_conversions() {
        let user = User {
            id: "user1".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            bio: Some("A test user".to_string()),
            avatar_url: None,
            cover_url: None,
            created_at: 1234567890,
            updated_at: 1234567890,
            birth_timestamp: Some(946684800),
            last_active: Some(1234567890),
            followers_count: 10,
            following_count: 5,
            posts_count: 3,
            age: Some(25),
            is_verified: false,
            is_private: false,
            is_active: true,
            allow_messages: true,
            interests: vec!["rust".to_string()],
            languages: vec!["en".to_string()],
            settings: HashMap::new(),
        };

        let schema_enum = SocialMediaSchema::User(user.clone());

        // Test successful TryFrom
        let extracted_user: User = schema_enum.try_into().unwrap();
        assert_eq!(extracted_user.id, "user1");

        // Test failed TryFrom
        let post_schema = SocialMediaSchema::Post(Post {
            id: "post1".to_string(),
            user_id: "user1".to_string(),
            created_at: 1234567890,
            content: "Hello".to_string(),
            updated_at: None,
            media_urls: vec![],
            hashtags: vec![],
            mentions: vec![],
            likes_count: 0,
            comments_count: 0,
            shares_count: 0,
            views_count: 0,
            is_public: true,
            allow_comments: true,
            allow_shares: true,
            latitude: None,
            longitude: None,
            location_name: None,
        });

        let user_result: Result<User, SocialMediaSchema> = post_schema.try_into();
        assert!(user_result.is_err());
    }
}
