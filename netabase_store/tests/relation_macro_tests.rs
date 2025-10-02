use bincode::{Decode, Encode};
use log::{debug, info};
use netabase_macros::NetabaseModel;
use netabase_store::{
    database::NetabaseSledDatabase,
    traits::{NetabaseModel, NetabaseSchema},
};
use std::collections::HashMap;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    });
}

// Simple model without relations (baseline)
#[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq)]
#[key_name(UserKey)]
pub struct User {
    #[key]
    pub id: u64,
    pub name: String,
    #[secondary_key]
    pub email: String,
}

// Model with single relation
#[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq)]
#[key_name(PostKey)]
pub struct Post {
    #[key]
    pub id: u64,
    pub title: String,
    pub content: String,
    #[secondary_key]
    pub author_id: u64,
    // Single relation - using generated type alias
    pub author: UserLink,
}

// Model with optional relation
#[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq)]
#[key_name(ProfileKey)]
pub struct Profile {
    #[key]
    pub id: u64,
    pub bio: String,
    #[secondary_key]
    pub user_id: u64,
    // Optional relation - using generated type alias
    pub user: Option<UserLink>,
}

// Model with vector relation
#[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq)]
#[key_name(CommentKey)]
pub struct Comment {
    #[key]
    pub id: u64,
    pub content: String,
    #[secondary_key]
    pub post_id: u64,
    // Vector relation - using generated type alias
    pub related_posts: Vec<PostLink>,
}

// Model with HashMap relation
#[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq)]
#[key_name(CategoryKey)]
pub struct Category {
    #[key]
    pub id: u64,
    pub name: String,
    // HashMap relation - using generated type alias
    pub featured_posts: HashMap<String, PostLink>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_relations_baseline() {
        init_logger();
        info!("Starting test_no_relations_baseline");

        // Test that models without relations work normally
        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        debug!("Created user: id={}, name={}", user.id, user.name);

        // Should have no relations
        debug!("Testing user relations");
        let relations = User::relations();
        assert!(relations.is_empty());
        info!("✓ User has no relations as expected");

        // Should encode/decode normally
        debug!("Testing serialization");
        let encoded = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();
        let (decoded, _): (User, usize) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(user, decoded);
        info!("✓ Serialization works correctly");

        info!("test_no_relations_baseline completed successfully");
    }

    #[test]
    fn test_single_relation_transformation() {
        // Test that single relation field is properly transformed

        // Create a post with unresolved author relation
        let post = Post {
            id: 1,
            title: "Test Post".to_string(),
            content: "This is a test post".to_string(),
            author_id: 1,
            // Using the generated UserLink type alias
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
        };

        // Should have no relations (since we removed the relation macro)
        let relations = Post::relations();
        assert_eq!(relations.len(), 0);

        // The author field should be a RelationalLink
        assert!(post.author.is_unresolved());
        let expected_key = UserKey::Primary(UserPrimaryKey(1));
        assert_eq!(post.author.key(), Some(&expected_key));

        // Test resolving the relation
        debug!("Testing relation resolution");
        let user = User {
            id: 123,
            name: "Author".to_string(),
            email: "author@example.com".to_string(),
        };
        debug!("Created author user: id={}, name={}", user.id, user.name);

        let resolved_author = post.author.resolve(user.clone());
        assert!(resolved_author.is_resolved());
        assert_eq!(resolved_author.object().unwrap().name, "Author");
        info!("✓ Author relation resolved successfully");

        info!("test_single_relation_transformation completed successfully");
    }

    #[test]
    fn test_optional_relation_transformation() {
        // Test that Optional relation field is properly transformed

        // Profile without user relation
        let profile_no_user = Profile {
            id: 1,
            bio: "Empty bio".to_string(),
            user_id: 1,
            user: None,
        };

        // Profile with user relation
        let profile_with_user = Profile {
            id: 2,
            bio: "Bio with user".to_string(),
            user_id: 2,
            user: Some(UserLink::from_key(UserKey::Primary(UserPrimaryKey(2)))),
        };

        // Should have no relations (since we removed the relation macro)
        let relations = Profile::relations();
        assert_eq!(relations.len(), 0);

        // Test empty optional relation
        assert!(profile_no_user.user.is_none());

        // Test filled optional relation
        assert!(profile_with_user.user.is_some());
        let user_link = profile_with_user.user.as_ref().unwrap();
        assert!(user_link.is_unresolved());
        let expected_key = UserKey::Primary(UserPrimaryKey(2));
        assert_eq!(user_link.key(), Some(&expected_key));
    }

    #[test]
    fn test_vector_relation_transformation() {
        // Test that Vec relation field is properly transformed

        let comment = Comment {
            id: 1,
            content: "Original comment".to_string(),
            post_id: 1,
            related_posts: vec![
                PostLink::from_key(PostKey::Primary(PostPrimaryKey(2))),
                PostLink::from_key(PostKey::Primary(PostPrimaryKey(3))),
            ],
        };

        // Should have no relations (since we removed the relation macro)
        let relations = Comment::relations();
        assert_eq!(relations.len(), 0);

        // Test vector relation
        assert_eq!(comment.related_posts.len(), 2);
        for post_link in &comment.related_posts {
            assert!(post_link.is_unresolved());
            assert!(post_link.key().is_some());
        }
    }

    #[test]
    fn test_hashmap_relation_transformation() {
        // Test that HashMap relation field is properly transformed

        let category = Category {
            id: 1,
            name: "Tech".to_string(),
            featured_posts: HashMap::from([
                (
                    "popular".to_string(),
                    PostLink::from_key(PostKey::Primary(PostPrimaryKey(1))),
                ),
                (
                    "trending".to_string(),
                    PostLink::from_key(PostKey::Primary(PostPrimaryKey(2))),
                ),
            ]),
        };

        // Should have no relations (since we removed the relation macro)
        let relations = Category::relations();
        assert_eq!(relations.len(), 0);

        // Test HashMap relation
        assert_eq!(category.featured_posts.len(), 2);
        for (key, post_link) in &category.featured_posts {
            assert!(key == "popular" || key == "trending");
            assert!(post_link.is_unresolved());
            assert!(post_link.key().is_some());
        }
    }

    #[test]
    fn test_serialization_with_relations() {
        // Test that models with relations can be serialized/deserialized

        let post = Post {
            id: 1,
            title: "Serializable Post".to_string(),
            content: "Content".to_string(),
            author_id: 1,
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
        };

        // Test bincode serialization
        let encoded = bincode::encode_to_vec(&post, bincode::config::standard()).unwrap();
        let (decoded, _): (Post, usize) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(post.title, decoded.title);
        assert_eq!(post.author_id, decoded.author_id);
        assert!(decoded.author.is_unresolved());
    }

    #[test]
    fn test_relation_discriminants_generation() {
        // Test that relation discriminants are properly generated

        // Since we removed the relation macro, all models should have empty relations
        assert!(User::relations().is_empty());
        assert!(Post::relations().is_empty());
        assert!(Profile::relations().is_empty());
        assert!(Comment::relations().is_empty());
        assert!(Category::relations().is_empty());

        // All models should have zero relations now
        assert_eq!(User::relations().len(), 0);
        assert_eq!(Post::relations().len(), 0);
        assert_eq!(Profile::relations().len(), 0);
        assert_eq!(Comment::relations().len(), 0);
        assert_eq!(Category::relations().len(), 0);
    }

    #[test]
    fn test_empty_collections() {
        // Test that empty collections work correctly

        let comment_no_posts = Comment {
            id: 1,
            content: "No related posts".to_string(),
            post_id: 1,
            related_posts: vec![],
        };

        let category_no_posts = Category {
            id: 1,
            name: "Empty".to_string(),
            featured_posts: HashMap::new(),
        };

        // Should still work with empty collections
        assert!(comment_no_posts.related_posts.is_empty());
        assert!(category_no_posts.featured_posts.is_empty());

        // Should still encode/decode correctly
        let encoded =
            bincode::encode_to_vec(&comment_no_posts, bincode::config::standard()).unwrap();
        let (decoded, _): (Comment, usize) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(comment_no_posts, decoded);
    }
}
