use bincode::{Decode, Encode};
use netabase::traits::NetabaseModel;
use netabase_macros::NetabaseModel;

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
        let user = User {
            id: 1,
            name: "Test User".to_string(),
        };

        // User should have no relations
        let relations = User::relations();
        assert!(relations.is_empty());

        // Should work normally
        let key = user.key();
        assert_eq!(key, UserKey::Primary(UserPrimaryKey(1)));
    }

    #[test]
    fn test_single_relation_field() {
        // Create a post with an unresolved author relation
        let post = Post {
            id: 1,
            title: "Test Post".to_string(),
            // The macro should have transformed this to RelationalLink<UserKey, User>
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
        };

        // Post should have no relations (since we removed the relation macro)
        let relations = Post::relations();
        assert_eq!(relations.len(), 0);

        // The author field should be a RelationalLink
        assert!(post.author.is_unresolved());
        let expected_key = UserKey::Primary(UserPrimaryKey(1));
        assert_eq!(post.author.key(), Some(&expected_key));

        // Test resolving the relation
        let user = User {
            id: 1,
            name: "Author".to_string(),
        };
        let resolved_author = post.author.resolve(user.clone());
        assert!(resolved_author.is_resolved());
        assert_eq!(resolved_author.object().unwrap().name, "Author");
    }

    #[test]
    fn test_optional_relation_field() {
        // Profile without user relation
        let profile_empty = Profile {
            id: 1,
            bio: "Empty profile".to_string(),
            user: None,
        };

        // Profile with user relation
        let profile_with_user = Profile {
            id: 2,
            bio: "Profile with user".to_string(),
            user: Some(UserLink::from_key(UserKey::Primary(UserPrimaryKey(2)))),
        };

        // Should have no relations (since we removed the relation macro)
        let relations = Profile::relations();
        assert_eq!(relations.len(), 0);

        // Test empty optional relation
        assert!(profile_empty.user.is_none());

        // Test filled optional relation
        assert!(profile_with_user.user.is_some());
        let user_link = profile_with_user.user.as_ref().unwrap();
        assert!(user_link.is_unresolved());
        let expected_key = UserKey::Primary(UserPrimaryKey(2));
        assert_eq!(user_link.key(), Some(&expected_key));
    }

    #[test]
    fn test_vector_relation_field() {
        let comment = Comment {
            id: 1,
            content: "Test comment".to_string(),
            related_posts: vec![
                PostLink::from_key(PostKey::Primary(PostPrimaryKey(1))),
                PostLink::from_key(PostKey::Primary(PostPrimaryKey(2))),
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
    fn test_serialization_with_relations() {
        let post = Post {
            id: 1,
            title: "Serializable Post".to_string(),
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
        };

        // Test bincode serialization
        let encoded = bincode::encode_to_vec(&post, bincode::config::standard()).unwrap();
        let (decoded, _): (Post, usize) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(post.title, decoded.title);
        assert!(decoded.author.is_unresolved());
        assert_eq!(post.author.key(), decoded.author.key());
    }

    #[test]
    fn test_empty_collections() {
        let comment_empty = Comment {
            id: 1,
            content: "Empty comment".to_string(),
            related_posts: vec![],
        };

        // Should work with empty vector
        assert!(comment_empty.related_posts.is_empty());

        // Should still serialize/deserialize correctly
        let encoded = bincode::encode_to_vec(&comment_empty, bincode::config::standard()).unwrap();
        let (decoded, _): (Comment, usize) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(comment_empty, decoded);
    }
}
