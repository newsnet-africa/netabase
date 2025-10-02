use bincode::{Decode, Encode};
use log::{debug, error, info, warn};
use netabase_macros::NetabaseModel;
use netabase_store::traits::NetabaseModel;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    });
}

// Simple model without relations for baseline
#[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq)]
#[key_name(UserKey)]
pub struct User {
    #[key]
    pub id: u64,
    pub name: String,
}

// Model with a single relation field
#[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq)]
#[key_name(PostKey)]
pub struct Post {
    #[key]
    pub id: u64,
    pub title: String,
    // Using generated type alias
    pub author: UserLink,
}

// Model with optional relation
#[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq)]
#[key_name(ProfileKey)]
pub struct Profile {
    #[key]
    pub id: u64,
    pub bio: String,
    // Using generated type alias
    pub user: Option<UserLink>,
}

// Model with vector relation
#[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq)]
#[key_name(CommentKey)]
pub struct Comment {
    #[key]
    pub id: u64,
    pub content: String,
    // Using generated type alias
    pub related_posts: Vec<PostLink>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_no_relations() {
        init_logger();
        info!("Starting test_baseline_no_relations");

        let user = User {
            id: 1,
            name: "Test User".to_string(),
        };
        debug!("Created user: id={}, name={}", user.id, user.name);

        // User should have no relations
        debug!("Testing user relations");
        let relations = User::relations();
        assert!(relations.is_empty());
        info!("✓ User has no relations as expected");

        // Should work normally
        debug!("Testing user key generation");
        let key = user.key();
        assert_eq!(key, UserKey::Primary(UserPrimaryKey(1)));
        info!("✓ User key generation works correctly");

        info!("test_baseline_no_relations completed successfully");
    }

    #[test]
    fn test_single_relation_field() {
        init_logger();
        info!("Starting test_single_relation_field");

        // Create a post with an unresolved author relation
        let post = Post {
            id: 1,
            title: "Test Post".to_string(),
            // The macro should have transformed this to RelationalLink<UserKey, User>
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
        };
        debug!("Created post: id={}, title={}", post.id, post.title);

        // Post should have no relations (since we removed the relation macro)
        debug!("Testing post relations");
        let relations = Post::relations();
        assert_eq!(relations.len(), 0);
        info!("✓ Post has no relations as expected (new behavior)");

        // The author field should be a RelationalLink
        debug!("Testing author relational link state");
        assert!(post.author.is_unresolved());
        let expected_key = UserKey::Primary(UserPrimaryKey(1));
        assert_eq!(post.author.key(), Some(&expected_key));
        info!("✓ Author link is unresolved with correct key");

        // Test resolving the relation
        debug!("Testing relation resolution");
        let user = User {
            id: 1,
            name: "Author".to_string(),
        };
        debug!("Created author user: id={}, name={}", user.id, user.name);

        let resolved_author = post.author.resolve(user.clone());
        assert!(resolved_author.is_resolved());
        assert_eq!(resolved_author.object().unwrap().name, "Author");
        info!("✓ Author relation resolved successfully");

        info!("test_single_relation_field completed successfully");
    }

    #[test]
    fn test_optional_relation_field() {
        init_logger();
        info!("Starting test_optional_relation_field");

        // Profile without user relation
        let profile_empty = Profile {
            id: 1,
            bio: "Empty profile".to_string(),
            user: None,
        };
        debug!(
            "Created empty profile: id={}, bio={}",
            profile_empty.id, profile_empty.bio
        );

        // Profile with user relation
        let profile_with_user = Profile {
            id: 2,
            bio: "Profile with user".to_string(),
            user: Some(UserLink::from_key(UserKey::Primary(UserPrimaryKey(2)))),
        };
        debug!(
            "Created profile with user: id={}, bio={}",
            profile_with_user.id, profile_with_user.bio
        );

        // Should have no relations (since we removed the relation macro)
        debug!("Testing profile relations");
        let relations = Profile::relations();
        assert_eq!(relations.len(), 0);
        info!("✓ Profile has no relations as expected (new behavior)");

        // Test empty optional relation
        debug!("Testing empty optional relation");
        assert!(profile_empty.user.is_none());
        info!("✓ Empty profile has no user relation as expected");

        // Test filled optional relation
        debug!("Testing filled optional relation");
        assert!(profile_with_user.user.is_some());
        let user_link = profile_with_user.user.as_ref().unwrap();
        assert!(user_link.is_unresolved());
        let expected_key = UserKey::Primary(UserPrimaryKey(2));
        assert_eq!(user_link.key(), Some(&expected_key));
        info!("✓ Profile with user has correct unresolved user link");

        info!("test_optional_relation_field completed successfully");
    }

    #[test]
    fn test_vector_relation_field() {
        init_logger();
        info!("Starting test_vector_relation_field");

        let comment = Comment {
            id: 1,
            content: "Test comment".to_string(),
            related_posts: vec![
                PostLink::from_key(PostKey::Primary(PostPrimaryKey(1))),
                PostLink::from_key(PostKey::Primary(PostPrimaryKey(2))),
            ],
        };
        debug!(
            "Created comment: id={}, content={}",
            comment.id, comment.content
        );

        // Should have no relations (since we removed the relation macro)
        debug!("Testing comment relations");
        let relations = Comment::relations();
        assert_eq!(relations.len(), 0);
        info!("✓ Comment has no relations as expected (new behavior)");

        // Test vector relation
        debug!(
            "Testing vector relation with {} posts",
            comment.related_posts.len()
        );
        assert_eq!(comment.related_posts.len(), 2);
        for (i, post_link) in comment.related_posts.iter().enumerate() {
            assert!(post_link.is_unresolved());
            assert!(post_link.key().is_some());
            debug!(
                "Post link {} is unresolved with key: {:?}",
                i,
                post_link.key()
            );
        }
        info!("✓ Vector relation with 2 posts verified successfully");

        info!("test_vector_relation_field completed successfully");
    }

    #[test]
    fn test_serialization_with_relations() {
        init_logger();
        info!("Starting test_serialization_with_relations");

        let post = Post {
            id: 1,
            title: "Serializable Post".to_string(),
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
        };
        debug!(
            "Created post for serialization: id={}, title={}",
            post.id, post.title
        );

        // Test bincode serialization
        debug!("Testing bincode serialization");
        let encoded = bincode::encode_to_vec(&post, bincode::config::standard()).unwrap();
        debug!("Post encoded successfully, size: {} bytes", encoded.len());

        let (decoded, _): (Post, usize) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        debug!(
            "Post decoded successfully: id={}, title={}",
            decoded.id, decoded.title
        );

        assert_eq!(post.title, decoded.title);
        assert!(decoded.author.is_unresolved());
        assert_eq!(post.author.key(), decoded.author.key());
        info!("✓ Serialization with relations verified successfully");

        info!("test_serialization_with_relations completed successfully");
    }

    #[test]
    fn test_empty_collections() {
        init_logger();
        info!("Starting test_empty_collections");

        let comment_empty = Comment {
            id: 1,
            content: "Empty comment".to_string(),
            related_posts: vec![],
        };
        debug!(
            "Created empty comment: id={}, content={}",
            comment_empty.id, comment_empty.content
        );

        // Should work with empty vector
        debug!("Testing empty vector relation");
        assert!(comment_empty.related_posts.is_empty());
        info!("✓ Empty vector relation works correctly");

        // Should still serialize/deserialize correctly
        debug!("Testing serialization of empty collections");
        let encoded = bincode::encode_to_vec(&comment_empty, bincode::config::standard()).unwrap();
        debug!(
            "Empty comment encoded successfully, size: {} bytes",
            encoded.len()
        );

        let (decoded, _): (Comment, usize) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(comment_empty, decoded);
        info!("✓ Empty collections serialize/deserialize correctly");

        info!("test_empty_collections completed successfully");
    }
}
