use bincode::{Decode, Encode};
use netabase::{
    database::{NetabaseSledDatabase, NetabaseSledTree},
    relational::RelationalLink,
    traits::NetabaseModel,
};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[netabase_schema_module(TestSchema, TestSchemaKey)]
pub mod test_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(UserKey)]
    pub struct User {
        #[key]
        pub id: u64,
        pub name: String,
        pub email: String,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(PostKey)]
    pub struct Post {
        #[key]
        pub id: u64,
        pub title: String,
        pub content: String,
        pub author_id: u64,
        // Relational field that will be resolved
        pub author: UserLink,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(CommentKey)]
    pub struct Comment {
        #[key]
        pub id: u64,
        pub content: String,
        pub post_id: u64,
        pub author_id: u64,
        // Multiple relational fields
        pub post: PostLink,
        pub author: UserLink,
    }
}

use test_schema::*;

type TestResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn create_test_database() -> TestResult<(NetabaseSledDatabase<TestSchema>, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("relational_test_db");
    let db = NetabaseSledDatabase::new_with_name(&db_path.to_string_lossy())?;
    Ok((db, temp_dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_mut_basic() -> TestResult<()> {
        // Create a user and a relational link
        let user = User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        let mut user_link = UserLink::from_key(UserKey::Primary(UserPrimaryKey(1)));

        // Verify initial state
        assert!(user_link.is_unresolved());
        assert!(user_link.object().is_none());

        // Resolve in-place and get reference
        {
            let resolved_user_ref = user_link.resolve_mut(user.clone());
            // Verify reference points to correct data
            assert_eq!(resolved_user_ref.id, 1);
            assert_eq!(resolved_user_ref.name, "Alice");
        }

        // Verify mutation happened (after reference is out of scope)
        assert!(user_link.is_resolved());
        assert_eq!(user_link.object().unwrap().id, 1);

        Ok(())
    }

    #[test]
    fn test_resolve_if_unresolved() -> TestResult<()> {
        let user = User {
            id: 1,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        };

        let mut user_link = UserLink::from_key(UserKey::Primary(UserPrimaryKey(1)));

        // First resolution
        let resolved_ref1 = user_link.resolve_if_unresolved(user.clone());
        assert_eq!(resolved_ref1.name, "Bob");
        assert!(user_link.is_resolved());

        // Second call should return existing object, not re-resolve
        let different_user = User {
            id: 1,
            name: "Charlie".to_string(), // Different name
            email: "charlie@example.com".to_string(),
        };

        let resolved_ref2 = user_link.resolve_if_unresolved(different_user);
        assert_eq!(resolved_ref2.name, "Bob"); // Should still be Bob, not Charlie

        Ok(())
    }

    #[test]
    fn test_post_with_author_resolution() -> TestResult<()> {
        let (db, _temp_dir) = create_test_database()?;

        // Create trees
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
        let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;

        // Create and store user
        let user = User {
            id: 1,
            name: "Author Alice".to_string(),
            email: "alice@blog.com".to_string(),
        };
        user_tree.insert(user.key(), user.clone())?;

        // Create post with unresolved author link
        let mut post = Post {
            id: 1,
            title: "My First Post".to_string(),
            content: "Hello world!".to_string(),
            author_id: 1,
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
        };

        // Verify author is unresolved initially
        assert!(post.author.is_unresolved());

        // Manually resolve the author from database
        let author_key = post.author.key().unwrap().clone();
        let fetched_author = user_tree.get(author_key)?.unwrap();
        {
            let author_ref = post.author.resolve_mut(fetched_author);
            // Verify reference points to correct data
            assert_eq!(author_ref.name, "Author Alice");
        }

        // Verify resolution worked (after reference is out of scope)
        assert!(post.author.is_resolved());
        assert_eq!(post.author.object().unwrap().email, "alice@blog.com");

        // Store the post with resolved author
        post_tree.insert(post.key(), post.clone())?;

        // Load post back and verify author is still resolved
        let loaded_post = post_tree.get(post.key())?.unwrap();
        assert!(loaded_post.author.is_resolved());
        assert_eq!(loaded_post.author.object().unwrap().name, "Author Alice");

        Ok(())
    }

    #[test]
    fn test_comment_with_multiple_relations() -> TestResult<()> {
        let (db, _temp_dir) = create_test_database()?;

        // Create trees
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
        let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;
        let comment_tree: NetabaseSledTree<Comment, CommentKey> = db.get_main_tree()?;

        // Create and store user
        let user = User {
            id: 1,
            name: "Commenter".to_string(),
            email: "commenter@example.com".to_string(),
        };
        user_tree.insert(user.key(), user.clone())?;

        // Create and store post (with unresolved author for now)
        let post = Post {
            id: 1,
            title: "Original Post".to_string(),
            content: "This is the original post".to_string(),
            author_id: 1,
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
        };
        post_tree.insert(post.key(), post.clone())?;

        // Create comment with unresolved relations
        let mut comment = Comment {
            id: 1,
            content: "Great post!".to_string(),
            post_id: 1,
            author_id: 1,
            post: PostLink::from_key(PostKey::Primary(PostPrimaryKey(1))),
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
        };

        // Verify both relations are unresolved
        assert!(comment.post.is_unresolved());
        assert!(comment.author.is_unresolved());

        // Resolve post relation
        let post_key = comment.post.key().unwrap().clone();
        let fetched_post = post_tree.get(post_key)?.unwrap();
        let post_ref = comment.post.resolve_mut(fetched_post);
        assert_eq!(post_ref.title, "Original Post");
        assert!(comment.post.is_resolved());

        // Resolve author relation
        let author_key = comment.author.key().unwrap().clone();
        let fetched_author = user_tree.get(author_key)?.unwrap();
        let author_ref = comment.author.resolve_mut(fetched_author);
        assert_eq!(author_ref.name, "Commenter");
        assert!(comment.author.is_resolved());

        // Both relations should now be resolved
        assert!(comment.post.is_resolved());
        assert!(comment.author.is_resolved());

        // Store and reload comment
        comment_tree.insert(comment.key(), comment.clone())?;
        let loaded_comment = comment_tree.get(comment.key())?.unwrap();

        // Verify persistence of resolved relations
        assert!(loaded_comment.post.is_resolved());
        assert!(loaded_comment.author.is_resolved());
        assert_eq!(loaded_comment.post.object().unwrap().title, "Original Post");
        assert_eq!(loaded_comment.author.object().unwrap().name, "Commenter");

        Ok(())
    }

    #[test]
    fn test_chained_resolution_workflow() -> TestResult<()> {
        let (db, _temp_dir) = create_test_database()?;

        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
        let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;

        // Setup data
        let user = User {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };
        user_tree.insert(user.key(), user.clone())?;

        // Create multiple posts by the same author
        let mut posts = vec![];
        for i in 1..=3 {
            let mut post = Post {
                id: i,
                title: format!("Post {}", i),
                content: format!("Content of post {}", i),
                author_id: 1,
                author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
            };

            // Resolve author for each post
            let author_key = post.author.key().unwrap().clone();
            let fetched_author = user_tree.get(author_key)?.unwrap();
            post.author.resolve_mut(fetched_author);

            posts.push(post);
        }

        // Verify all posts have resolved authors
        for post in &posts {
            assert!(post.author.is_resolved());
            assert_eq!(post.author.object().unwrap().name, "John Doe");
        }

        // Store all posts
        for post in &posts {
            post_tree.insert(post.key(), post.clone())?;
        }

        // Load them back and verify resolution persisted
        for i in 1..=3 {
            let loaded_post = post_tree.get(PostKey::Primary(PostPrimaryKey(i)))?.unwrap();
            assert!(loaded_post.author.is_resolved());
            assert_eq!(loaded_post.author.object().unwrap().name, "John Doe");
        }

        Ok(())
    }

    #[test]
    fn test_resolution_error_handling() -> TestResult<()> {
        // Test what happens when we try to resolve with wrong object
        let _user1 = User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        let user2 = User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        };

        // Create link pointing to user 1, but resolve with user 2
        let mut user_link = UserLink::from_key(UserKey::Primary(UserPrimaryKey(1)));
        {
            let resolved_ref = user_link.resolve_mut(user2.clone());
            // Verify reference points to correct data
            assert_eq!(resolved_ref.id, 2); // Should be Bob's data
            assert_eq!(resolved_ref.name, "Bob");
        }

        // The resolution should work (it doesn't validate key consistency)
        assert!(user_link.is_resolved());

        Ok(())
    }

    #[test]
    fn test_consuming_vs_mutating_resolve() -> TestResult<()> {
        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        // Test consuming resolve (original behavior)
        let link1 = UserLink::from_key(UserKey::Primary(UserPrimaryKey(1)));
        let resolved_link1 = link1.resolve(user.clone()); // Consumes link1
        assert!(resolved_link1.is_resolved());
        // link1 is no longer accessible here

        // Test mutating resolve (new behavior)
        let mut link2 = UserLink::from_key(UserKey::Primary(UserPrimaryKey(1)));
        {
            let user_ref = link2.resolve_mut(user.clone()); // Mutates link2
            // Verify reference points to correct data
            assert_eq!(user_ref.name, "Test User");
        }

        // Verify link2 is still accessible and mutated (after reference is out of scope)
        assert!(link2.is_resolved());

        Ok(())
    }
}
