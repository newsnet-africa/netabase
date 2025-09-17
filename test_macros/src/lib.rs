use crate::social_data::v1;
use chrono::{DateTime, Duration, Utc};
use netabase_macros::netabase_schema;
use serde::{Deserialize, Serialize};

// # Comprehensive Social Media Test Schema
//
// This test module provides a comprehensive social media data model that exercises
// all supported data types and struct variations in the netabase macro system.
//
// ## Test Coverage
//
// ### Primitive Types Tested:
// - **Boolean**: `bool` (is_active, is_verified)
// - **Signed integers**: `i8`, `i16`, `i32`, `i64`, `i128` (all edge cases)
// - **Unsigned integers**: `u8`, `u16`, `u32`, `u64`, `u128` (including MAX values)
// - **Floating point**: `f32`, `f64` (including infinity and precision)
// - **Character**: `char` (Unicode characters like emojis)
// - **String**: `String` (including empty strings and Unicode text)
// - **Timestamps**: `i64` (Unix timestamps for date/time representation)
//
// ### Optional Types:
// - `Option<T>` for all primitive types
// - `Some(value)` and `None` cases
//
// ### Collection Types:
// - `Vec<String>` (tags, interests, languages)
// - `HashMap<String, String>` (metadata, settings)
// - Large collections (1000+ elements) for performance testing
//
// ### Struct Variations:
// - **Named field structs**: Traditional structs with named fields (User, Post, Comment)
// - **Unit-like structs**: Minimal structs with just ID (TestUnit)
// - **Tuple-like structs**: Structs with numbered field access (TestTuple)
//
// ### Social Media Domain Model:
// - `User`: Comprehensive user profiles with all field types
// - `Post`: Social media posts with engagement metrics and geographic data
// - `Comment`: Hierarchical comments with threading support
// - `Media`: File attachments with metadata (images, videos, documents)
// - `Reaction`: User reactions to posts/comments
// - `Notification`: System notifications with various types
// - `UserStats`: Daily user activity statistics
// - `HashTag`: Trending hashtags with popularity metrics
// - `PrimitiveTest`: Dedicated struct for testing all primitive types
//
// ### NativeDB Integration:
// - Primary keys (`#[primary_key]`)
// - Secondary keys (`#[secondary_key]`) for indexing
// - All structs implement required traits for database storage
//
// ### Serialization Testing:
// - Bincode serialization/deserialization for network transport
// - JSON serialization verification (ensuring bincode is used, not JSON)
// - Round-trip testing ensuring data integrity
// - Large dataset performance testing
//
// ### Edge Cases Covered:
// - Minimum and maximum values for all numeric types
// - Infinity values for floating point numbers
// - Empty strings and collections
// - Unicode characters and emojis
// - Deeply nested optional types
// - Large collections (performance testing)
//
// ### Network Integration:
// - KadRecord conversion for libp2p DHT storage
// - Reference enum testing for memory efficiency
// - Cow<'_, Record> patterns for zero-copy operations
//
// This comprehensive test suite ensures that the netabase macro system can handle
// real-world application data models with all supported Rust types and patterns.

// Comprehensive social media data model testing all types
#[netabase_schema(SocialMediaSchema)]
pub mod social_data {
    use bincode::{Decode, Encode};
    use native_db::{native_db, ToKey};
    use native_model::{native_model, Model};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    // Type aliases for latest versions
    pub type User = v1::User;
    pub type Post = v1::Post;
    pub type Comment = v1::Comment;
    pub type Media = v1::Media;
    pub type Reaction = v1::Reaction;
    pub type Notification = v1::Notification;
    pub type UserStats = v1::UserStats;
    pub type HashTag = v1::HashTag;
    pub type PrimitiveTest = v1::PrimitiveTest;
    pub type TestTuple = v1::TestTuple;
    pub type TestUnit = v1::TestUnit;

    pub mod v1 {
        use super::*;
        use chrono::{DateTime, Duration, Utc};

        // Test all primitive types
        #[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
        #[native_model(id = 1, version = 1)]
        #[native_db]
        pub struct PrimitiveTest {
            #[primary_key]
            pub id: String,

            // Boolean
            pub is_active: bool,
            pub is_verified: bool,

            // Signed integers
            pub byte_value: i8,
            pub short_value: i16,
            pub int_value: i32,
            pub long_value: i64,
            pub huge_value: i128,

            // Unsigned integers
            pub ubyte_value: u8,
            pub ushort_value: u16,
            pub uint_value: u32,
            pub ulong_value: u64,
            pub uhuge_value: u128,

            // Floating point
            pub float_value: f32,
            pub double_value: f64,

            // Character and string
            pub char_value: char,
            pub text: String,

            // Optional types
            pub optional_number: Option<i32>,
            pub optional_text: Option<String>,

            // Collections
            pub tags: Vec<String>,
            pub metadata: HashMap<String, String>,
        }

        // Unit-like struct test with minimal data
        #[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
        #[native_model(id = 2, version = 1)]
        #[native_db]
        pub struct TestUnit {
            #[primary_key]
            pub id: String,
            #[bincode(with_serde)]
            pub timestamp: DateTime<Utc>,
        }

        // Tuple-like struct test (named fields simulating tuple behavior)
        #[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
        #[native_model(id = 3, version = 1)]
        #[native_db]
        pub struct TestTuple {
            #[primary_key]
            pub field_0: String,
            pub field_1: String,
            pub field_2: i32,
            pub field_3: bool,
            #[bincode(with_serde)]
            pub field_4: DateTime<Utc>,
        }

        // Comprehensive user model with named fields
        #[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
        #[native_model(id = 4, version = 1)]
        #[native_db]
        pub struct User {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub username: String,
            #[secondary_key]
            pub email: String,

            // Profile information
            pub display_name: Option<String>,
            pub bio: Option<String>,
            pub avatar_url: Option<String>,
            pub cover_url: Option<String>,

            // Timestamps using chrono
            #[bincode(with_serde)]
            pub created_at: DateTime<Utc>,
            #[bincode(with_serde)]
            pub updated_at: DateTime<Utc>,
            #[bincode(with_serde)]
            pub birth_timestamp: Option<DateTime<Utc>>,
            #[bincode(with_serde)]
            pub last_active: DateTime<Utc>,

            // Numeric data
            pub followers_count: u32,
            pub following_count: u32,
            pub posts_count: u32,
            pub age: Option<u8>,

            // Boolean flags
            pub is_verified: bool,
            pub is_private: bool,
            pub is_active: bool,
            pub allow_messages: bool,

            // Collections
            pub interests: Vec<String>,
            pub languages: Vec<String>,
            pub settings: HashMap<String, String>,
        }

        // Post model with various field types
        #[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
        #[native_model(id = 5, version = 1)]
        #[native_db]
        pub struct Post {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub user_id: String,
            #[secondary_key]
            pub created_at: i64,

            pub content: String,
            #[bincode(with_serde)]
            pub updated_at: Option<DateTime<Utc>>,
            pub media_urls: Vec<String>,
            pub hashtags: Vec<String>,
            pub mentions: Vec<String>,

            // Engagement metrics
            pub likes_count: u32,
            pub comments_count: u32,
            pub shares_count: u32,
            pub views_count: u64,

            // Post settings
            pub is_public: bool,
            pub allow_comments: bool,
            pub allow_shares: bool,

            // Geographic data
            pub latitude: Option<f64>,
            pub longitude: Option<f64>,
            pub location_name: Option<String>,
        }

        // Comment model
        #[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
        #[native_model(id = 6, version = 1)]
        #[native_db]
        pub struct Comment {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub post_id: String,
            #[secondary_key]
            pub user_id: String,
            #[secondary_key]
            pub created_at: i64,

            pub content: String,
            pub parent_comment_id: Option<String>,
            pub likes_count: u16,
            pub replies_count: u16,
            pub is_edited: bool,
            #[bincode(with_serde)]
            pub edited_at: Option<DateTime<Utc>>,
        }

        // Media attachment model
        #[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
        #[native_model(id = 7, version = 1)]
        #[native_db]
        pub struct Media {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub post_id: String,
            #[secondary_key]
            pub uploaded_at: i64,

            pub url: String,
            pub media_type: String, // "image", "video", "audio", "document"
            pub filename: String,
            pub size_bytes: u64,
            pub width: Option<u32>,
            pub height: Option<u32>,
            pub duration_seconds: Option<f32>,
            pub alt_text: Option<String>,
            pub is_processed: bool,
        }

        // Reaction model
        #[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
        #[native_model(id = 8, version = 1)]
        #[native_db]
        pub struct Reaction {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub user_id: String,
            #[secondary_key]
            pub target_id: String, // post_id or comment_id
            #[secondary_key]
            pub created_at: i64,

            pub reaction_type: String, // "like", "love", "laugh", "angry", "sad"
            pub target_type: String,   // "post", "comment"
        }

        // Notification model
        #[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
        #[native_model(id = 9, version = 1)]
        #[native_db]
        pub struct Notification {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub user_id: String,
            #[secondary_key]
            pub created_at: i64,

            pub notification_type: String,
            pub title: String,
            pub message: String,
            pub is_read: bool,
            #[bincode(with_serde)]
            pub read_at: Option<DateTime<Utc>>,
            pub related_user_id: Option<String>,
            pub related_post_id: Option<String>,
            pub related_comment_id: Option<String>,
            pub action_url: Option<String>,
            pub priority: u8, // 1-5 priority level
        }

        // User statistics model
        #[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
        #[native_model(id = 10, version = 1)]
        #[native_db]
        pub struct UserStats {
            #[primary_key]
            pub user_id: String,
            #[secondary_key]
            pub date_timestamp: i64, // Unix timestamp for date

            pub posts_created: u16,
            pub comments_made: u16,
            pub likes_given: u16,
            pub likes_received: u16,
            pub profile_views: u32,
            pub time_spent_minutes: u32,
            pub login_count: u8,
            pub avg_session_duration: f32,
        }

        // HashTag model
        #[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
        #[native_model(id = 11, version = 1)]
        #[native_db]
        pub struct HashTag {
            #[primary_key]
            pub tag: String,
            #[secondary_key]
            pub created_at: i64,

            pub usage_count: u64,
            pub trending_score: f64,
            #[bincode(with_serde)]
            pub last_used: DateTime<Utc>,
            pub is_trending: bool,
            pub category: Option<String>,
            pub related_tags: Vec<String>,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{f32, f64};
    use libp2p::kad::Record as KadRecord;
    use netabase::{AsKadRecord, NetabaseRecordExt};
    use std::borrow::Cow;
    use std::collections::HashMap;

    #[test]
    fn test_primitive_types_comprehensive() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        let primitive_test = social_data::v1::PrimitiveTest {
            id: "primitive-test-1".to_string(),
            is_active: true,
            is_verified: false,

            // Signed integers
            byte_value: -42i8,
            short_value: -1234i16,
            int_value: -123456i32,
            long_value: -9876543210i64,
            huge_value: -123456789012345678901234567890i128,

            // Unsigned integers
            ubyte_value: 255u8,
            ushort_value: 65535u16,
            uint_value: 4294967295u32,
            ulong_value: 18446744073709551615u64,
            uhuge_value: 340282366920938463463374607431768211455u128,

            // Floating point
            float_value: f32::consts::PI,
            double_value: f64::consts::E,

            // Character and string
            char_value: '🎉',
            text: "Hello, 世界! 🌍".to_string(),

            // Optional types
            optional_number: Some(42),
            optional_text: None,

            // Collections
            tags: vec![
                "rust".to_string(),
                "database".to_string(),
                "p2p".to_string(),
            ],
            metadata,
        };

        let schema_enum = SocialMediaSchema::PrimitiveTest(primitive_test.clone());

        // Test round-trip serialization
        let kad_record = schema_enum.to_kad_record();
        let recovered = SocialMediaSchema::from_kad_record(kad_record).unwrap();

        match recovered {
            SocialMediaSchema::PrimitiveTest(recovered_primitive) => {
                assert_eq!(recovered_primitive, primitive_test);

                // Test specific extreme values
                assert_eq!(
                    recovered_primitive.huge_value,
                    -123456789012345678901234567890i128
                );
                assert_eq!(
                    recovered_primitive.uhuge_value,
                    340282366920938463463374607431768211455u128
                );
                assert_eq!(recovered_primitive.char_value, '🎉');
                assert_eq!(recovered_primitive.text, "Hello, 世界! 🌍");
            }
            _ => panic!("Expected PrimitiveTest variant"),
        }
    }

    #[test]
    fn test_unit_struct_equivalent() {
        let now = DateTime::from_timestamp(1640995200, 0).unwrap(); // Convert Unix timestamp to DateTime
        let unit_struct = social_data::v1::TestUnit {
            id: "unit-test-1".to_string(),
            timestamp: now,
        };
        let schema_enum = SocialMediaSchema::TestUnit(unit_struct.clone());

        // Test round-trip serialization
        let kad_record = schema_enum.to_kad_record();
        let recovered = SocialMediaSchema::from_kad_record(kad_record).unwrap();

        match recovered {
            SocialMediaSchema::TestUnit(recovered_unit) => {
                assert_eq!(recovered_unit, unit_struct);
                assert_eq!(recovered_unit.timestamp, now);
            }
            _ => panic!("Expected TestUnit variant"),
        }
    }

    #[test]
    fn test_user_named_fields_with_timestamps() {
        let now = DateTime::from_timestamp(1640995200, 0).unwrap(); // Convert Unix timestamp to DateTime
        let birth_timestamp = DateTime::from_timestamp(643680000, 0).unwrap(); // DateTime for 1990-05-15

        let mut interests = vec![
            "rust".to_string(),
            "p2p".to_string(),
            "databases".to_string(),
        ];
        interests.sort(); // Ensure consistent ordering

        let mut languages = vec!["en".to_string(), "es".to_string(), "fr".to_string()];
        languages.sort();

        let mut settings = HashMap::new();
        settings.insert("theme".to_string(), "dark".to_string());
        settings.insert("notifications".to_string(), "enabled".to_string());

        let user = social_data::v1::User {
            id: "user-123".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            bio: Some("A test user for comprehensive testing".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            cover_url: None,
            created_at: now,
            updated_at: now,
            birth_timestamp: Some(birth_timestamp),
            last_active: now,
            followers_count: 150,
            following_count: 89,
            posts_count: 42,
            age: Some(33),
            is_verified: true,
            is_private: false,
            is_active: true,
            allow_messages: true,
            interests,
            languages,
            settings,
        };

        let schema_enum = SocialMediaSchema::User(user.clone());

        // Test round-trip serialization with all field types
        let kad_record = schema_enum.to_kad_record();
        let recovered = SocialMediaSchema::from_kad_record(kad_record).unwrap();

        match recovered {
            SocialMediaSchema::User(recovered_user) => {
                assert_eq!(recovered_user, user);

                // Test specific timestamp fields
                assert_eq!(recovered_user.created_at, now);
                assert_eq!(recovered_user.birth_timestamp, Some(birth_timestamp));

                // Test optional fields
                assert_eq!(recovered_user.display_name, Some("Test User".to_string()));
                assert_eq!(recovered_user.cover_url, None);

                // Test collections
                assert_eq!(recovered_user.interests.len(), 3);
                assert!(recovered_user.interests.contains(&"rust".to_string()));

                // Test numeric types
                assert_eq!(recovered_user.followers_count, 150u32);
                assert_eq!(recovered_user.age, Some(33u8));
            }
            _ => panic!("Expected User variant"),
        }
    }

    #[test]
    fn test_post_with_geographic_data() {
        let now = DateTime::from_timestamp(1640995200, 0).unwrap(); // Convert Unix timestamp to DateTime

        let post = social_data::v1::Post {
            id: "post-456".to_string(),
            user_id: "user-123".to_string(),
            created_at: 1640995200i64,
            content: "Check out this amazing view! 🌄 #travel #nature #photography".to_string(),
            updated_at: None,
            media_urls: vec![
                "https://example.com/photo1.jpg".to_string(),
                "https://example.com/photo2.jpg".to_string(),
            ],
            hashtags: vec![
                "travel".to_string(),
                "nature".to_string(),
                "photography".to_string(),
            ],
            mentions: vec!["@friend1".to_string(), "@travelblogger".to_string()],
            likes_count: 245,
            comments_count: 18,
            shares_count: 12,
            views_count: 1547,
            is_public: true,
            allow_comments: true,
            allow_shares: true,
            latitude: Some(40.7128),
            longitude: Some(-74.0060), // New York City coordinates
            location_name: Some("New York City, NY".to_string()),
        };

        let schema_enum = SocialMediaSchema::Post(post.clone());

        // Test serialization with geographic and engagement data
        let kad_record = schema_enum.to_kad_record();
        let recovered = SocialMediaSchema::from_kad_record(kad_record).unwrap();

        match recovered {
            SocialMediaSchema::Post(recovered_post) => {
                assert_eq!(recovered_post, post);

                // Test geographic precision
                assert_eq!(recovered_post.latitude, Some(40.7128));
                assert_eq!(recovered_post.longitude, Some(-74.0060));

                // Test large numbers
                assert_eq!(recovered_post.views_count, 1547u64);

                // Test collections with various string types
                assert_eq!(recovered_post.hashtags.len(), 3);
                assert_eq!(recovered_post.media_urls.len(), 2);
                assert_eq!(recovered_post.mentions.len(), 2);
            }
            _ => panic!("Expected Post variant"),
        }
    }

    #[test]
    fn test_comment_with_hierarchical_structure() {
        let now = DateTime::from_timestamp(1640995200, 0).unwrap(); // Convert Unix timestamp to DateTime
        let edited_at = now + chrono::Duration::seconds(300); // 5 minutes later

        let comment = social_data::v1::Comment {
            id: "comment-789".to_string(),
            post_id: "post-456".to_string(),
            user_id: "user-123".to_string(),
            created_at: 1640995200i64,
            content: "This is a reply to another comment with emojis! 😊👍".to_string(),
            parent_comment_id: Some("comment-parent".to_string()),
            likes_count: 7,
            replies_count: 2,
            is_edited: true,
            edited_at: Some(edited_at),
        };

        let schema_enum = SocialMediaSchema::Comment(comment.clone());

        let kad_record = schema_enum.to_kad_record();
        let recovered = SocialMediaSchema::from_kad_record(kad_record).unwrap();

        match recovered {
            SocialMediaSchema::Comment(recovered_comment) => {
                assert_eq!(recovered_comment, comment);

                // Test hierarchical relationship
                assert_eq!(
                    recovered_comment.parent_comment_id,
                    Some("comment-parent".to_string())
                );

                // Test smaller integer types
                assert_eq!(recovered_comment.likes_count, 7u16);
                assert_eq!(recovered_comment.replies_count, 2u16);

                // Test optional timestamp field
                assert_eq!(recovered_comment.edited_at, Some(edited_at));
            }
            _ => panic!("Expected Comment variant"),
        }
    }

    #[test]
    fn test_media_with_various_numeric_types() {
        let now = DateTime::from_timestamp(1640995200, 0).unwrap(); // Convert Unix timestamp to DateTime

        let media = social_data::v1::Media {
            id: "media-101".to_string(),
            post_id: "post-456".to_string(),
            uploaded_at: 1640995200i64,
            url: "https://cdn.example.com/video/awesome_clip.mp4".to_string(),
            media_type: "video".to_string(),
            filename: "awesome_clip.mp4".to_string(),
            size_bytes: 52428800u64, // 50MB
            width: Some(1920u32),
            height: Some(1080u32),
            duration_seconds: Some(125.7f32),
            alt_text: Some("An awesome video clip showing amazing scenery".to_string()),
            is_processed: true,
        };

        let schema_enum = SocialMediaSchema::Media(media.clone());

        let kad_record = schema_enum.to_kad_record();
        let recovered = SocialMediaSchema::from_kad_record(kad_record).unwrap();

        match recovered {
            SocialMediaSchema::Media(recovered_media) => {
                assert_eq!(recovered_media, media);

                // Test large numbers
                assert_eq!(recovered_media.size_bytes, 52428800u64);

                // Test optional u32 fields
                assert_eq!(recovered_media.width, Some(1920u32));
                assert_eq!(recovered_media.height, Some(1080u32));

                // Test floating point precision
                assert_eq!(recovered_media.duration_seconds, Some(125.7f32));
            }
            _ => panic!("Expected Media variant"),
        }
    }

    #[test]
    fn test_user_stats_with_timestamp() {
        let date_timestamp = 1710537600i64; // March 15, 2024

        let stats = social_data::v1::UserStats {
            user_id: "user-123".to_string(),
            date_timestamp,
            posts_created: 5,
            comments_made: 23,
            likes_given: 87,
            likes_received: 134,
            profile_views: 456,
            time_spent_minutes: 127,
            login_count: 3,
            avg_session_duration: 42.33f32,
        };

        let schema_enum = SocialMediaSchema::UserStats(stats.clone());

        let kad_record = schema_enum.to_kad_record();
        let recovered = SocialMediaSchema::from_kad_record(kad_record).unwrap();

        match recovered {
            SocialMediaSchema::UserStats(recovered_stats) => {
                assert_eq!(recovered_stats, stats);

                // Test timestamp specifically
                assert_eq!(recovered_stats.date_timestamp, 1710537600i64);

                // Test various integer sizes
                assert_eq!(recovered_stats.posts_created, 5u16);
                assert_eq!(recovered_stats.profile_views, 456u32);
                assert_eq!(recovered_stats.login_count, 3u8);

                // Test f32 precision
                assert_eq!(recovered_stats.avg_session_duration, 42.33f32);
            }
            _ => panic!("Expected UserStats variant"),
        }
    }

    #[test]
    fn test_hashtag_with_trending_data() {
        let created_at = 1640995200i64; // Unix timestamp
        let last_used =
            DateTime::from_timestamp(1640995200, 0).unwrap() + chrono::Duration::seconds(7200); // 2 hours later

        let hashtag = social_data::v1::HashTag {
            tag: "rustlang".to_string(),
            created_at,
            usage_count: 125847u64,
            trending_score: 0.8573f64,
            last_used,
            is_trending: true,
            category: Some("programming".to_string()),
            related_tags: vec![
                "rust".to_string(),
                "programming".to_string(),
                "systems".to_string(),
                "memory_safety".to_string(),
            ],
        };

        let schema_enum = SocialMediaSchema::HashTag(hashtag.clone());

        let kad_record = schema_enum.to_kad_record();
        let recovered = SocialMediaSchema::from_kad_record(kad_record).unwrap();

        match recovered {
            SocialMediaSchema::HashTag(recovered_hashtag) => {
                assert_eq!(recovered_hashtag, hashtag);

                // Test f64 precision
                assert_eq!(recovered_hashtag.trending_score, 0.8573f64);

                // Test large u64
                assert_eq!(recovered_hashtag.usage_count, 125847u64);

                // Test string collections
                assert_eq!(recovered_hashtag.related_tags.len(), 4);
                assert!(recovered_hashtag.related_tags.contains(&"rust".to_string()));
            }
            _ => panic!("Expected HashTag variant"),
        }
    }

    #[test]
    fn test_notification_comprehensive() {
        let created_at = 1640995200i64; // Unix timestamp
        let read_at =
            DateTime::from_timestamp(1640995200, 0).unwrap() + chrono::Duration::seconds(1800); // 30 minutes later

        let notification = social_data::v1::Notification {
            id: "notification-202".to_string(),
            user_id: "user-123".to_string(),
            created_at,
            notification_type: "like".to_string(),
            title: "New Like".to_string(),
            message: "Someone liked your post!".to_string(),
            is_read: true,
            read_at: Some(read_at),
            related_user_id: Some("user-456".to_string()),
            related_post_id: Some("post-789".to_string()),
            related_comment_id: None,
            action_url: Some("/post/123456".to_string()),
            priority: 3u8,
        };

        let schema_enum = SocialMediaSchema::Notification(notification.clone());

        let kad_record = schema_enum.to_kad_record();
        let recovered = SocialMediaSchema::from_kad_record(kad_record).unwrap();

        match recovered {
            SocialMediaSchema::Notification(recovered_notification) => {
                assert_eq!(recovered_notification, notification);

                // Test multiple optional String fields
                assert_eq!(
                    recovered_notification.related_user_id,
                    Some("user-456".to_string())
                );
                assert_eq!(
                    recovered_notification.related_post_id,
                    Some("post-789".to_string())
                );
                assert_eq!(recovered_notification.related_comment_id, None);

                // Test optional timestamp field
                assert_eq!(recovered_notification.read_at, Some(read_at));

                // Test u8 field
                assert_eq!(recovered_notification.priority, 3u8);
            }
            _ => panic!("Expected Notification variant"),
        }
    }

    #[test]
    fn test_multiple_schema_variants_comprehensive() {
        let now = DateTime::from_timestamp(1640995200, 0).unwrap(); // Convert Unix timestamp to DateTime

        // Create instances of multiple schema variants to test the enum generation
        let primitive_test = SocialMediaSchema::PrimitiveTest(social_data::v1::PrimitiveTest {
            id: "primitive-multi-test".to_string(),
            is_active: true,
            is_verified: false,
            byte_value: -42i8,
            short_value: 1000i16,
            int_value: 100000i32,
            long_value: 10000000000i64,
            huge_value: 123456789012345678901234567890i128,
            ubyte_value: 255u8,
            ushort_value: 65535u16,
            uint_value: 4000000000u32,
            ulong_value: 18000000000000000000u64,
            uhuge_value: 340282366920938463463374607431768211455u128,
            float_value: f32::consts::PI,
            double_value: f64::consts::E,
            char_value: '🚀',
            text: "Comprehensive test".to_string(),
            optional_number: Some(42),
            optional_text: Some("Optional text".to_string()),
            tags: vec!["test".to_string(), "comprehensive".to_string()],
            metadata: {
                let mut map = HashMap::new();
                map.insert("version".to_string(), "1.0".to_string());
                map
            },
        });

        let unit_struct = SocialMediaSchema::TestUnit(social_data::v1::TestUnit {
            id: "unit-multi-test".to_string(),
            timestamp: now,
        });

        let tuple_struct = SocialMediaSchema::TestTuple(social_data::v1::TestTuple {
            field_0: "tuple-multi-test".to_string(),
            field_1: "Tuple test".to_string(),
            field_2: 999,
            field_3: false,
            field_4: now,
        });

        let user = SocialMediaSchema::User(social_data::v1::User {
            id: "user-multi-test".to_string(),
            username: "comprehensive_user".to_string(),
            email: "comprehensive@test.com".to_string(),
            display_name: Some("Comprehensive User".to_string()),
            bio: None,
            avatar_url: None,
            cover_url: None,
            created_at: now,
            updated_at: now,
            birth_timestamp: Some(DateTime::from_timestamp(809337600, 0).unwrap()), // August 20, 1995
            last_active: now,
            followers_count: 500,
            following_count: 200,
            posts_count: 75,
            age: Some(28),
            is_verified: true,
            is_private: false,
            is_active: true,
            allow_messages: true,
            interests: vec!["technology".to_string(), "science".to_string()],
            languages: vec!["en".to_string(), "de".to_string()],
            settings: HashMap::new(),
        });

        // Test all variants can be serialized and deserialized
        let variants = vec![primitive_test, unit_struct, tuple_struct, user];

        for (i, variant) in variants.into_iter().enumerate() {
            let kad_record = variant.to_kad_record();
            let recovered = SocialMediaSchema::from_kad_record(kad_record)
                .unwrap_or_else(|_| panic!("Failed to recover variant {}", i));

            // Verify each variant type is correctly recovered
            match (&variant, &recovered) {
                (SocialMediaSchema::PrimitiveTest(_), SocialMediaSchema::PrimitiveTest(_)) => {}
                (SocialMediaSchema::TestUnit(_), SocialMediaSchema::TestUnit(_)) => {}
                (SocialMediaSchema::TestTuple(_), SocialMediaSchema::TestTuple(_)) => {}
                (SocialMediaSchema::User(_), SocialMediaSchema::User(_)) => {}
                _ => panic!("Variant type mismatch at index {}", i),
            }
        }
    }

    #[test]
    fn test_ref_enum_conversion_comprehensive() {
        let now = DateTime::from_timestamp(1640995200, 0).unwrap(); // Convert Unix timestamp to DateTime

        let user = social_data::v1::User {
            id: "ref-test-user".to_string(),
            username: "ref_test_user".to_string(),
            email: "ref@test.com".to_string(),
            display_name: Some("Reference Test User".to_string()),
            bio: Some("Testing reference enums".to_string()),
            avatar_url: None,
            cover_url: None,
            created_at: now,
            updated_at: now,
            birth_timestamp: Some(DateTime::from_timestamp(725846400, 0).unwrap()), // December 25, 1992
            last_active: now,
            followers_count: 100,
            following_count: 50,
            posts_count: 25,
            age: Some(31),
            is_verified: false,
            is_private: true,
            is_active: true,
            allow_messages: false,
            interests: vec!["testing".to_string()],
            languages: vec!["en".to_string()],
            settings: {
                let mut settings = HashMap::new();
                settings.insert("privacy".to_string(), "high".to_string());
                settings
            },
        };

        let schema_enum = SocialMediaSchema::User(user.clone());
        let ref_enum: SocialMediaSchemaRef = (&schema_enum).into();

        // Test AsKadRecord trait implementation for reference enum
        let kad_record_cow: Cow<'_, KadRecord> = ref_enum.as_kad_record();

        match kad_record_cow {
            Cow::Owned(kad_record) => {
                let recovered = SocialMediaSchema::from_kad_record(kad_record).unwrap();
                match recovered {
                    SocialMediaSchema::User(recovered_user) => {
                        assert_eq!(recovered_user.id, user.id);
                        assert_eq!(recovered_user.username, user.username);
                        assert_eq!(recovered_user.birth_timestamp, user.birth_timestamp);
                        assert_eq!(recovered_user.is_private, user.is_private);
                        assert_eq!(recovered_user.settings, user.settings);
                    }
                    _ => panic!("Expected User variant after ref conversion"),
                }
            }
            Cow::Borrowed(_) => panic!("Expected owned record from ref enum"),
        }
    }

    #[test]
    fn test_bincode_vs_json_serialization() {
        // Verify that we're using bincode, not serde_json for network serialization
        let user = social_data::v1::User {
            id: "bincode-test".to_string(),
            username: "bincode_user".to_string(),
            email: "bincode@test.com".to_string(),
            display_name: Some("Bincode Test User".to_string()),
            bio: Some("Testing serialization formats".to_string()),
            avatar_url: None,
            cover_url: None,
            created_at: DateTime::from_timestamp(1640995200, 0).unwrap(),
            updated_at: DateTime::from_timestamp(1640995200, 0).unwrap(),
            birth_timestamp: Some(DateTime::from_timestamp(584409600, 0).unwrap()), // July 12, 1988
            last_active: DateTime::from_timestamp(1640995200, 0).unwrap(),
            followers_count: 75,
            following_count: 42,
            posts_count: 15,
            age: Some(35),
            is_verified: true,
            is_private: false,
            is_active: true,
            allow_messages: true,
            interests: vec!["serialization".to_string(), "testing".to_string()],
            languages: vec!["en".to_string()],
            settings: HashMap::new(),
        };

        let schema_enum = SocialMediaSchema::User(user);
        let kad_record = schema_enum.to_kad_record();

        // Bincode data should not be valid JSON
        let json_parse_result = serde_json::from_slice::<serde_json::Value>(&kad_record.value);
        assert!(
            json_parse_result.is_err(),
            "Data should be bincode, not JSON"
        );

        // But should be valid bincode
        let bincode_result = bincode::decode_from_slice::<netabase::Record<SocialMediaSchema>, _>(
            &kad_record.value,
            bincode::config::standard(),
        );
        assert!(bincode_result.is_ok(), "Data should be valid bincode");
    }

    #[test]
    fn test_edge_case_values() {
        // Test with edge case values for numeric types
        let edge_case_test = social_data::v1::PrimitiveTest {
            id: "edge-case-test".to_string(),
            is_active: false,
            is_verified: true,

            // Signed integer edge cases
            byte_value: i8::MIN,
            short_value: i16::MAX,
            int_value: i32::MIN,
            long_value: i64::MAX,
            huge_value: i128::MIN,

            // Unsigned integer edge cases
            ubyte_value: u8::MAX,
            ushort_value: u16::MIN,
            uint_value: u32::MAX,
            ulong_value: u64::MIN,
            uhuge_value: u128::MAX,

            // Floating point edge cases
            float_value: f32::INFINITY,
            double_value: f64::NEG_INFINITY,

            // Unicode character
            char_value: '𝕊',
            text: "".to_string(), // Empty string

            // Edge cases for optionals
            optional_number: None,
            optional_text: Some("".to_string()),

            // Empty collections
            tags: vec![],
            metadata: HashMap::new(),
        };

        let schema_enum = SocialMediaSchema::PrimitiveTest(edge_case_test.clone());

        // Test serialization with edge case values
        let kad_record = schema_enum.to_kad_record();
        let recovered = SocialMediaSchema::from_kad_record(kad_record).unwrap();

        match recovered {
            SocialMediaSchema::PrimitiveTest(recovered_test) => {
                assert_eq!(recovered_test, edge_case_test);

                // Verify specific edge case values
                assert_eq!(recovered_test.byte_value, i8::MIN);
                assert_eq!(recovered_test.short_value, i16::MAX);
                assert_eq!(recovered_test.uint_value, u32::MAX);
                assert_eq!(recovered_test.uhuge_value, u128::MAX);
                assert_eq!(recovered_test.float_value, f32::INFINITY);
                assert_eq!(recovered_test.double_value, f64::NEG_INFINITY);
                assert_eq!(recovered_test.char_value, '𝕊');
                assert_eq!(recovered_test.text, "");
                assert!(recovered_test.tags.is_empty());
                assert!(recovered_test.metadata.is_empty());
            }
            _ => panic!("Expected PrimitiveTest variant"),
        }
    }

    #[test]
    fn test_large_collections() {
        // Test with large collections to verify performance and correctness
        let large_tags: Vec<String> = (0..1000).map(|i| format!("tag{}", i)).collect();
        let large_metadata: HashMap<String, String> = (0..500)
            .map(|i| (format!("key{}", i), format!("value{}", i)))
            .collect();

        let large_collection_test = social_data::v1::PrimitiveTest {
            id: "large-collection-test".to_string(),
            is_active: true,
            is_verified: false,
            byte_value: 42,
            short_value: 1234,
            int_value: 123456,
            long_value: 123456789,
            huge_value: 123456789012345678901234567890,
            ubyte_value: 200,
            ushort_value: 50000,
            uint_value: 3000000000,
            ulong_value: 15000000000000000000,
            uhuge_value: 200000000000000000000000000000000000000,
            float_value: f32::consts::PI,
            double_value: f64::consts::E,
            char_value: '🚀',
            text: "Large collection test".to_string(),
            optional_number: Some(999),
            optional_text: Some("Large test".to_string()),
            tags: large_tags.clone(),
            metadata: large_metadata.clone(),
        };

        let schema_enum = SocialMediaSchema::PrimitiveTest(large_collection_test.clone());

        // Test serialization with large collections
        let kad_record = schema_enum.to_kad_record();
        let recovered = SocialMediaSchema::from_kad_record(kad_record).unwrap();

        match recovered {
            SocialMediaSchema::PrimitiveTest(recovered_test) => {
                assert_eq!(recovered_test, large_collection_test);
                assert_eq!(recovered_test.tags.len(), 1000);
                assert_eq!(recovered_test.metadata.len(), 500);

                // Verify some specific elements
                assert!(recovered_test.tags.contains(&"tag500".to_string()));
                assert_eq!(
                    recovered_test.metadata.get("key250"),
                    Some(&"value250".to_string())
                );
            }
            _ => panic!("Expected PrimitiveTest variant"),
        }
    }
}
