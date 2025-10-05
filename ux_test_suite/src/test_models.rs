//! # Test Models
//!
//! This module contains various test models used to validate macro hygiene,
//! dependency auto-export, and user experience across different scenarios.
//! These models are designed to test edge cases and common usage patterns.

use netabase_macros::{netabase_schema_module, NetabaseModel};

// Re-export dependencies for convenience in tests
pub use netabase_deps::{bincode, derive_more, serde, sled, strum};

/// Basic model with minimal fields for hygiene testing
#[derive(
    NetabaseModel,
    Clone,
    Debug,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
#[key_name(BasicUserKey)]
pub struct BasicUser {
    #[key]
    pub id: u64,
    pub name: String,
}

impl Default for BasicUser {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Test User".to_string(),
        }
    }
}

/// Model with secondary keys for testing indexing
#[derive(
    NetabaseModel,
    Clone,
    Debug,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
#[key_name(IndexedUserKey)]
pub struct IndexedUser {
    #[key]
    pub id: u64,
    pub name: String,
    #[secondary_key]
    pub email: String,
    #[secondary_key]
    pub department: String,
    #[secondary_key]
    pub active: bool,
    pub created_at: u64,
}

impl Default for IndexedUser {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            department: "Engineering".to_string(),
            active: true,
            created_at: 1600000000,
        }
    }
}

/// Model with complex types to test serialization edge cases
#[derive(
    NetabaseModel,
    Clone,
    Debug,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
#[key_name(ComplexModelKey)]
pub struct ComplexModel {
    #[key]
    pub id: u64,
    pub name: String,
    #[secondary_key]
    pub category: ModelCategory,
    pub metadata: std::collections::HashMap<String, String>,
    pub tags: Vec<String>,
    pub created_at: Option<u64>,
    pub scores: [f64; 3],
}

impl Default for ComplexModel {
    fn default() -> Self {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());

        Self {
            id: 1,
            name: "Complex Test Model".to_string(),
            category: ModelCategory::TypeA,
            metadata,
            tags: vec!["test".to_string(), "complex".to_string()],
            created_at: Some(1600000000),
            scores: [1.0, 2.0, 3.0],
        }
    }
}

/// Enum for testing complex secondary keys
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
    strum::EnumString,
    strum::Display,
)]
pub enum ModelCategory {
    #[default]
    TypeA,
    TypeB,
    TypeC,
}

/// Model with foreign key relationships
#[derive(
    NetabaseModel,
    Clone,
    Debug,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
#[key_name(RelationalModelKey)]
pub struct RelationalModel {
    #[key]
    pub id: u64,
    pub title: String,
    #[secondary_key]
    pub owner_id: u64, // Foreign key to BasicUser
    #[secondary_key]
    pub category_id: u32,
    #[secondary_key]
    pub status: RelationalStatus,
    pub description: String,
}

impl Default for RelationalModel {
    fn default() -> Self {
        Self {
            id: 1,
            title: "Test Relational Model".to_string(),
            owner_id: 1,
            category_id: 100,
            status: RelationalStatus::Active,
            description: "A test model with relationships".to_string(),
        }
    }
}

/// Status enum for relational model
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
    strum::EnumString,
    strum::Display,
)]
pub enum RelationalStatus {
    #[default]
    Active,
    Inactive,
    Pending,
    Archived,
}

/// Model with derive_more features to test compatibility
#[derive(
    NetabaseModel,
    Clone,
    Debug,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
    derive_more::From,
    derive_more::Into,
)]
#[key_name(DerivedModelKey)]
pub struct DerivedModel {
    #[key]
    pub id: u64,
    pub value: String,
}

impl Default for DerivedModel {
    fn default() -> Self {
        Self {
            id: 1,
            value: "derived".to_string(),
        }
    }
}

/// Schema module for testing multiple models together
#[netabase_schema_module(TestSchema, TestSchemaKeys)]
mod test_schema {
    use super::*;
    use netabase_store::traits::NetabaseModel;

    /// User model for schema testing
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
        bincode::Encode,
        bincode::Decode,
    )]
    #[key_name(SchemaUserKey)]
    pub struct SchemaUser {
        #[key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub email: String,
        #[secondary_key]
        pub role: UserRole,
        pub created_at: u64,
    }

    /// Post model for schema testing
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
        bincode::Encode,
        bincode::Decode,
    )]
    #[key_name(SchemaPostKey)]
    pub struct SchemaPost {
        #[key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64, // Foreign key to SchemaUser
        #[secondary_key]
        pub published: bool,
        #[secondary_key]
        pub category: PostCategory,
        pub created_at: u64,
        pub updated_at: Option<u64>,
    }

    /// Comment model for schema testing
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
        bincode::Encode,
        bincode::Decode,
    )]
    #[key_name(SchemaCommentKey)]
    pub struct SchemaComment {
        #[key]
        pub id: u64,
        pub content: String,
        #[secondary_key]
        pub post_id: u64, // Foreign key to SchemaPost
        #[secondary_key]
        pub author_id: u64, // Foreign key to SchemaUser
        #[secondary_key]
        pub approved: bool,
        pub created_at: u64,
    }

    /// User role enum
    #[derive(
        Clone,
        Debug,
        PartialEq,
        Eq,
        Hash,
        Default,
        serde::Serialize,
        serde::Deserialize,
        bincode::Encode,
        bincode::Decode,
        strum::EnumString,
        strum::Display,
    )]
    pub enum UserRole {
        Admin,
        Moderator,
        #[default]
        User,
        Guest,
    }

    /// Post category enum
    #[derive(
        Clone,
        Debug,
        PartialEq,
        Eq,
        Hash,
        Default,
        serde::Serialize,
        serde::Deserialize,
        bincode::Encode,
        bincode::Decode,
        strum::EnumString,
        strum::Display,
    )]
    pub enum PostCategory {
        #[default]
        Technology,
        Science,
        Politics,
        Entertainment,
        Sports,
        Other,
    }
}

// Re-export schema types for use in tests
pub use test_schema::*;

/// Model factory for creating test instances
pub struct TestModelFactory;

impl TestModelFactory {
    /// Create a basic user with specified ID
    pub fn basic_user(id: u64) -> BasicUser {
        BasicUser {
            id,
            name: format!("User{}", id),
        }
    }

    /// Create an indexed user with specified ID
    pub fn indexed_user(id: u64) -> IndexedUser {
        IndexedUser {
            id,
            name: format!("User{}", id),
            email: format!("user{}@example.com", id),
            department: if id % 2 == 0 {
                "Engineering".to_string()
            } else {
                "Marketing".to_string()
            },
            active: id % 3 != 0,
            created_at: 1600000000 + (id * 3600),
        }
    }

    /// Create a complex model with specified ID
    pub fn complex_model(id: u64) -> ComplexModel {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), format!("1.{}", id));
        metadata.insert("type".to_string(), "test".to_string());

        ComplexModel {
            id,
            name: format!("Complex{}", id),
            category: match id % 3 {
                0 => ModelCategory::TypeA,
                1 => ModelCategory::TypeB,
                _ => ModelCategory::TypeC,
            },
            metadata,
            tags: vec![format!("tag{}", id), "test".to_string()],
            created_at: if id % 2 == 0 {
                Some(1600000000 + id)
            } else {
                None
            },
            scores: [id as f64, (id * 2) as f64, (id * 3) as f64],
        }
    }

    /// Create a relational model with specified ID and owner
    pub fn relational_model(id: u64, owner_id: u64) -> RelationalModel {
        RelationalModel {
            id,
            title: format!("Relational{}", id),
            owner_id,
            category_id: (100 + id % 10) as u32,
            status: match id % 4 {
                0 => RelationalStatus::Active,
                1 => RelationalStatus::Inactive,
                2 => RelationalStatus::Pending,
                _ => RelationalStatus::Archived,
            },
            description: format!("Description for relational model {}", id),
        }
    }

    /// Create a schema user with specified ID
    pub fn schema_user(id: u64) -> SchemaUser {
        SchemaUser {
            id,
            username: format!("user{}", id),
            email: format!("user{}@example.com", id),
            role: match id % 4 {
                0 => UserRole::Admin,
                1 => UserRole::Moderator,
                2 => UserRole::User,
                _ => UserRole::Guest,
            },
            created_at: 1600000000 + (id * 3600),
        }
    }

    /// Create a schema post with specified ID and author
    pub fn schema_post(id: u64, author_id: u64) -> SchemaPost {
        SchemaPost {
            id,
            title: format!("Post {}", id),
            content: format!("This is the content of post {}", id),
            author_id,
            published: id % 2 == 0,
            category: match id % 6 {
                0 => PostCategory::Technology,
                1 => PostCategory::Science,
                2 => PostCategory::Politics,
                3 => PostCategory::Entertainment,
                4 => PostCategory::Sports,
                _ => PostCategory::Other,
            },
            created_at: 1600000000 + (id * 1800),
            updated_at: if id % 3 == 0 {
                Some(1600000000 + (id * 1800) + 900)
            } else {
                None
            },
        }
    }

    /// Create a schema comment with specified IDs
    pub fn schema_comment(id: u64, post_id: u64, author_id: u64) -> SchemaComment {
        SchemaComment {
            id,
            content: format!("This is comment {} on post {}", id, post_id),
            post_id,
            author_id,
            approved: id % 3 != 0,
            created_at: 1600000000 + (id * 900),
        }
    }

    /// Create multiple users for bulk testing
    pub fn multiple_indexed_users(count: usize) -> Vec<IndexedUser> {
        (1..=count).map(|i| Self::indexed_user(i as u64)).collect()
    }

    /// Create a complete blog scenario (users, posts, comments)
    pub fn blog_scenario() -> (Vec<SchemaUser>, Vec<SchemaPost>, Vec<SchemaComment>) {
        let users = vec![
            Self::schema_user(1),
            Self::schema_user(2),
            Self::schema_user(3),
        ];

        let posts = vec![
            Self::schema_post(1, 1), // User 1's post
            Self::schema_post(2, 1), // User 1's second post
            Self::schema_post(3, 2), // User 2's post
            Self::schema_post(4, 3), // User 3's post
        ];

        let comments = vec![
            Self::schema_comment(1, 1, 2), // User 2 comments on post 1
            Self::schema_comment(2, 1, 3), // User 3 comments on post 1
            Self::schema_comment(3, 2, 3), // User 3 comments on post 2
            Self::schema_comment(4, 3, 1), // User 1 comments on post 3
        ];

        (users, posts, comments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netabase_store::traits::NetabaseModel;

    #[test]
    fn test_basic_user_creation() {
        let user = BasicUser::default();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Test User");

        // Test key extraction
        let _key = user.key();
    }

    #[test]
    fn test_indexed_user_creation() {
        let user = IndexedUser::default();
        assert_eq!(user.id, 1);
        assert!(user.active);

        // Test key extraction
        let _key = user.key();
    }

    #[test]
    fn test_complex_model_creation() {
        let model = ComplexModel::default();
        assert_eq!(model.id, 1);
        assert_eq!(model.tags.len(), 2);
        assert_eq!(model.metadata.len(), 1);

        // Test key extraction
        let _key = model.key();
    }

    #[test]
    fn test_model_factory() {
        let user = TestModelFactory::indexed_user(42);
        assert_eq!(user.id, 42);
        assert_eq!(user.name, "User42");
        assert_eq!(user.email, "user42@example.com");

        let post = TestModelFactory::schema_post(1, 42);
        assert_eq!(post.author_id, 42);
    }

    #[test]
    fn test_blog_scenario() {
        let (users, posts, comments) = TestModelFactory::blog_scenario();
        assert_eq!(users.len(), 3);
        assert_eq!(posts.len(), 4);
        assert_eq!(comments.len(), 4);

        // Verify relationships
        assert_eq!(posts[0].author_id, users[0].id);
        assert_eq!(comments[0].post_id, posts[0].id);
        assert_eq!(comments[0].author_id, users[1].id);
    }

    #[test]
    fn test_enum_serialization() {
        use bincode::{Decode, Encode};

        let category = ModelCategory::TypeA;
        let encoded = bincode::encode_to_vec(&category, bincode::config::standard()).unwrap();
        let decoded: ModelCategory =
            bincode::decode_from_slice(&encoded, bincode::config::standard())
                .unwrap()
                .0;
        assert_eq!(category, decoded);
    }
}
