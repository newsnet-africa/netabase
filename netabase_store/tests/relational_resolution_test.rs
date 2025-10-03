use bincode::{Decode, Encode};
use log::{debug, error, info, warn};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::{
    database::{NetabaseSledDatabase, NetabaseSledTree},
    relational::RelationalLink,
    traits::NetabaseModel,
};
use serde::{Deserialize, Serialize};
use std::sync::Once;
use tempfile::TempDir;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    });
}

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
    init_logger();
    info!("Creating relational test database in temporary directory");

    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("relational_test_db");
    debug!("Relational test database path: {}", db_path.display());

    let db = NetabaseSledDatabase::new_with_path(&db_path.to_string_lossy())?;
    info!("Relational test database created successfully");

    Ok((db, temp_dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_mut_basic() -> TestResult<()> {
        info!("Starting test_resolve_mut_basic");

        // Create a user and a relational link
        let user = User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        debug!("Created user: id={}, name={}", user.id, user.name);

        let mut user_link = UserLink::from_key(UserKey::Primary(UserPrimaryKey(1)));
        debug!("Created user link from key");

        // Verify initial state
        assert!(user_link.is_unresolved());
        assert!(user_link.object().is_none());
        info!("✓ Initial state verified - link is unresolved");

        // Resolve in-place and get reference
        debug!("Resolving user link in-place");
        {
            let resolved_user_ref = user_link.resolve_mut(user.clone());
            // Verify reference points to correct data
            assert_eq!(resolved_user_ref.id, 1);
            assert_eq!(resolved_user_ref.name, "Alice");
            debug!(
                "Reference verified: id={}, name={}",
                resolved_user_ref.id, resolved_user_ref.name
            );
        }

        // Verify mutation happened (after reference is out of scope)
        assert!(user_link.is_resolved());
        assert_eq!(user_link.object().unwrap().id, 1);
        info!("✓ Mutation verified - link is now resolved");

        info!("test_resolve_mut_basic completed successfully");
        Ok(())
    }

    #[test]
    fn test_resolve_if_unresolved() -> TestResult<()> {
        info!("Starting test_resolve_if_unresolved");

        let user = User {
            id: 1,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        };
        debug!("Created user: id={}, name={}", user.id, user.name);

        let mut user_link = UserLink::from_key(UserKey::Primary(UserPrimaryKey(1)));
        debug!("Created user link from key");

        // First resolution
        debug!("First resolution attempt");
        {
            let resolved_ref1 = user_link.resolve_if_unresolved(user.clone());
            assert_eq!(resolved_ref1.name, "Bob");
            info!(
                "✓ First resolution successful - name: {}",
                resolved_ref1.name
            );
        }
        assert!(user_link.is_resolved());

        // Second call should return existing object, not re-resolve
        let different_user = User {
            id: 1,
            name: "Charlie".to_string(), // Different name
            email: "charlie@example.com".to_string(),
        };
        debug!(
            "Created different user for second resolution test: name={}",
            different_user.name
        );

        debug!("Second resolution attempt (should not replace existing)");
        {
            let resolved_ref2 = user_link.resolve_if_unresolved(different_user);
            assert_eq!(resolved_ref2.name, "Bob"); // Should still be Bob, not Charlie
            info!(
                "✓ Second resolution correctly returned existing object - name: {}",
                resolved_ref2.name
            );
        }

        info!("test_resolve_if_unresolved completed successfully");
        Ok(())
    }

    #[test]
    fn test_post_with_author_resolution() -> TestResult<()> {
        info!("Starting test_post_with_author_resolution");

        let (db, _temp_dir) = create_test_database()?;
        info!("Database created successfully for post author resolution test");

        // Create trees
        debug!("Creating trees for post author resolution test");
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
        let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;
        info!("✓ Trees created successfully");

        // Create and store user
        let user = User {
            id: 1,
            name: "Author Alice".to_string(),
            email: "alice@blog.com".to_string(),
        };
        debug!("Created user: id={}, name={}", user.id, user.name);
        user_tree.insert(user.key(), user.clone())?;
        info!("✓ User stored successfully");

        // Create post with unresolved author link
        let mut post = Post {
            id: 1,
            title: "My First Post".to_string(),
            content: "Hello world!".to_string(),
            author_id: 1,
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
        };
        debug!("Created post: id={}, title={}", post.id, post.title);

        // Verify author is unresolved initially
        assert!(post.author.is_unresolved());
        info!("✓ Post author link is initially unresolved as expected");

        // Manually resolve the author from database
        debug!("Resolving author from database");
        let author_key = post.author.key().unwrap().clone();
        let fetched_author = user_tree.get(author_key)?.unwrap();
        debug!("Fetched author from database: name={}", fetched_author.name);
        {
            let author_ref = post.author.resolve_mut(fetched_author);
            // Verify reference points to correct data
            assert_eq!(author_ref.name, "Author Alice");
            debug!("Author resolved with name: {}", author_ref.name);
        }

        // Verify resolution worked (after reference is out of scope)
        assert!(post.author.is_resolved());
        assert_eq!(post.author.object().unwrap().email, "alice@blog.com");
        info!("✓ Post author resolution verified successfully");

        // Store the post with resolved author
        debug!("Storing post with resolved author");
        post_tree.insert(post.key(), post.clone())?;
        info!("✓ Post stored successfully");

        // Load post back and verify author is still resolved
        debug!("Loading post back to verify persistence");
        let loaded_post = post_tree.get(post.key())?.unwrap();
        assert!(loaded_post.author.is_resolved());
        assert_eq!(loaded_post.author.object().unwrap().name, "Author Alice");
        info!("✓ Post loaded back with author still resolved");

        info!("test_post_with_author_resolution completed successfully");
        Ok(())
    }

    #[test]
    fn test_comment_with_multiple_relations() -> TestResult<()> {
        info!("Starting test_comment_with_multiple_relations");

        let (db, _temp_dir) = create_test_database()?;
        info!("Database created successfully for comment multiple relations test");

        // Create trees
        debug!("Creating trees for comment multiple relations test");
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
        let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;
        let comment_tree: NetabaseSledTree<Comment, CommentKey> = db.get_main_tree()?;
        info!("✓ Trees created successfully");

        // Create and store user
        let user = User {
            id: 1,
            name: "Commenter".to_string(),
            email: "commenter@example.com".to_string(),
        };
        debug!("Created user: id={}, name={}", user.id, user.name);
        user_tree.insert(user.key(), user.clone())?;
        info!("✓ User stored successfully");

        // Create and store post (with unresolved author for now)
        let post = Post {
            id: 1,
            title: "Original Post".to_string(),
            content: "This is the original post".to_string(),
            author_id: 1,
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
        };
        debug!("Created post: id={}, title={}", post.id, post.title);
        post_tree.insert(post.key(), post.clone())?;
        info!("✓ Post stored successfully");

        // Create comment with unresolved relations
        let mut comment = Comment {
            id: 1,
            content: "Great post!".to_string(),
            post_id: 1,
            author_id: 1,
            post: PostLink::from_key(PostKey::Primary(PostPrimaryKey(1))),
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
        };
        debug!(
            "Created comment: id={}, content={}",
            comment.id, comment.content
        );

        // Verify both relations are unresolved
        assert!(comment.post.is_unresolved());
        assert!(comment.author.is_unresolved());
        info!("✓ Both comment relations are initially unresolved as expected");

        // Resolve post relation
        debug!("Resolving comment post relation");
        let post_key = comment.post.key().unwrap().clone();
        let fetched_post = post_tree.get(post_key)?.unwrap();
        let post_ref = comment.post.resolve_mut(fetched_post);
        assert_eq!(post_ref.title, "Original Post");
        assert!(comment.post.is_resolved());
        info!("✓ Comment post relation resolved successfully");

        // Resolve author relation
        debug!("Resolving comment author relation");
        let author_key = comment.author.key().unwrap().clone();
        let fetched_author = user_tree.get(author_key)?.unwrap();
        let author_ref = comment.author.resolve_mut(fetched_author);
        assert_eq!(author_ref.name, "Commenter");
        assert!(comment.author.is_resolved());
        info!("✓ Comment author relation resolved successfully");

        // Both relations should now be resolved
        assert!(comment.post.is_resolved());
        assert!(comment.author.is_resolved());
        info!("✓ Both comment relations are now resolved");

        // Store and reload comment
        debug!("Storing comment with resolved relations");
        comment_tree.insert(comment.key(), comment.clone())?;
        let loaded_comment = comment_tree.get(comment.key())?.unwrap();
        info!("✓ Comment stored and reloaded successfully");

        // Verify persistence of resolved relations
        debug!("Verifying persistence of resolved relations");
        assert!(loaded_comment.post.is_resolved());
        assert!(loaded_comment.author.is_resolved());
        assert_eq!(loaded_comment.post.object().unwrap().title, "Original Post");
        assert_eq!(loaded_comment.author.object().unwrap().name, "Commenter");
        info!("✓ Persistence of resolved relations verified successfully");

        info!("test_comment_with_multiple_relations completed successfully");
        Ok(())
    }

    #[test]
    fn test_chained_resolution_workflow() -> TestResult<()> {
        info!("Starting test_chained_resolution_workflow");

        let (db, _temp_dir) = create_test_database()?;
        info!("Database created successfully for chained resolution workflow test");

        debug!("Creating trees for chained resolution test");
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
        let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;
        info!("✓ Trees created successfully");

        // Setup data
        debug!("Setting up test data");
        let user = User {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };
        debug!("Created user: id={}, name={}", user.id, user.name);
        user_tree.insert(user.key(), user.clone())?;
        info!("✓ User stored successfully");

        // Create multiple posts by the same author
        debug!("Creating multiple posts for chained resolution");
        let mut posts = vec![];
        for i in 1..=3 {
            let mut post = Post {
                id: i,
                title: format!("Post {}", i),
                content: format!("Content of post {}", i),
                author_id: 1,
                author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
            };
            debug!("Created post: id={}, title={}", post.id, post.title);

            // Resolve author for each post
            debug!("Resolving author for post {}", i);
            let author_key = post.author.key().unwrap().clone();
            let fetched_author = user_tree.get(author_key)?.unwrap();
            post.author.resolve_mut(fetched_author);
            debug!("Author resolved for post {}", i);

            posts.push(post);
        }
        info!("✓ Created and resolved 3 posts successfully");

        // Verify all posts have resolved authors
        debug!("Verifying all posts have resolved authors");
        for post in &posts {
            assert!(post.author.is_resolved());
            assert_eq!(post.author.object().unwrap().name, "John Doe");
            debug!(
                "Post {} has resolved author: {}",
                post.id,
                post.author.object().unwrap().name
            );
        }
        info!("✓ All posts have resolved authors verified");

        // Store all posts
        debug!("Storing all posts with resolved authors");
        for post in &posts {
            post_tree.insert(post.key(), post.clone())?;
            debug!("Stored post with id: {}", post.id);
        }
        info!("✓ All posts stored successfully");

        // Load them back and verify resolution persisted
        debug!("Loading posts back to verify persistence");
        for i in 1..=3 {
            let loaded_post = post_tree.get(PostKey::Primary(PostPrimaryKey(i)))?.unwrap();
            assert!(loaded_post.author.is_resolved());
            assert_eq!(loaded_post.author.object().unwrap().name, "John Doe");
            debug!(
                "Loaded post {} with persistent resolved author: {}",
                i,
                loaded_post.author.object().unwrap().name
            );
        }
        info!("✓ Resolution persistence verified for all posts");

        info!("test_chained_resolution_workflow completed successfully");
        Ok(())
    }

    #[test]
    fn test_resolution_error_handling() -> TestResult<()> {
        info!("Starting test_resolution_error_handling");

        // Test what happens when we try to resolve with wrong object
        debug!("Creating users for error handling test");
        let _user1 = User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        debug!("Created user1: id=1, name=Alice (not used in resolution)");

        let user2 = User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        };
        debug!("Created user2: id=2, name=Bob (will be used for resolution)");

        // Create link pointing to user 1, but resolve with user 2
        debug!("Creating link pointing to user 1 but resolving with user 2");
        let mut user_link = UserLink::from_key(UserKey::Primary(UserPrimaryKey(1)));
        debug!("Created link pointing to user with id=1");
        {
            let resolved_ref = user_link.resolve_mut(user2.clone());
            // Verify reference points to correct data
            assert_eq!(resolved_ref.id, 2); // Should be Bob's data
            assert_eq!(resolved_ref.name, "Bob");
            debug!(
                "Resolution used user2 data: id={}, name={}",
                resolved_ref.id, resolved_ref.name
            );
        }

        // The resolution should work (it doesn't validate key consistency)
        assert!(user_link.is_resolved());
        info!("✓ Resolution works without key consistency validation");

        info!("test_resolution_error_handling completed successfully");
        Ok(())
    }

    #[test]
    fn test_consuming_vs_mutating_resolve() -> TestResult<()> {
        info!("Starting test_consuming_vs_mutating_resolve");

        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        debug!("Created user: id={}, name={}", user.id, user.name);

        // Test consuming resolve (original behavior)
        debug!("Testing consuming resolve (original behavior)");
        let link1 = UserLink::from_key(UserKey::Primary(UserPrimaryKey(1)));
        debug!("Created link1 for consuming resolve");
        let resolved_link1 = link1.resolve(user.clone()); // Consumes link1
        assert!(resolved_link1.is_resolved());
        info!("✓ Consuming resolve works correctly - link1 consumed");
        // link1 is no longer accessible here

        // Test mutating resolve (new behavior)
        debug!("Testing mutating resolve (new behavior)");
        let mut link2 = UserLink::from_key(UserKey::Primary(UserPrimaryKey(1)));
        debug!("Created link2 for mutating resolve");
        {
            let user_ref = link2.resolve_mut(user.clone()); // Mutates link2
            // Verify reference points to correct data
            assert_eq!(user_ref.name, "Test User");
            debug!("Reference verified: name={}", user_ref.name);
        }

        // Verify link2 is still accessible and mutated (after reference is out of scope)
        assert!(link2.is_resolved());
        info!("✓ Mutating resolve works correctly - link2 still accessible and resolved");

        info!("test_consuming_vs_mutating_resolve completed successfully");
        Ok(())
    }
}
