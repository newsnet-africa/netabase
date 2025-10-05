use bincode::{Decode, Encode};
use log::{debug, info};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::{
    database::{NetabaseSledDatabase, NetabaseSledTree},
    relational::{RelationalResolver, utils},
    traits::{
        NetabaseAdvancedQuery, NetabaseModel, NetabaseRelationalQuery, NetabaseSecondaryKeyQuery,
    },
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
        #[secondary_key]
        pub email: String,
        #[secondary_key]
        pub department: String,
        pub age: u32,
        pub created_at: u64,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(PostKey)]
    pub struct Post {
        #[key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
        #[secondary_key]
        pub category: String,
        pub published: bool,
        pub created_at: u64,
        // Relational field
        pub author: UserLink,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(CommentKey)]
    pub struct Comment {
        #[key]
        pub id: u64,
        pub content: String,
        #[secondary_key]
        pub post_id: u64,
        #[secondary_key]
        pub author_id: u64,
        pub likes: u32,
        pub created_at: u64,
        // Relational fields
        pub post: PostLink,
        pub author: UserLink,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(ProfileKey)]
    pub struct Profile {
        #[key]
        pub id: u64,
        pub bio: String,
        #[secondary_key]
        pub user_id: u64,
        pub avatar_url: Option<String>,
        pub social_links: Vec<String>,
        // Optional relational field
        pub user: Option<UserLink>,
    }
}

use test_schema::*;

type TestResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn create_test_database() -> TestResult<(NetabaseSledDatabase<TestSchema>, TempDir)> {
    init_logger();
    info!("Creating test database for secondary key and relational queries");

    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("secondary_relational_test_db");
    debug!("Test database path: {}", db_path.display());

    let db = NetabaseSledDatabase::new_with_path(&db_path)?;
    info!("Test database created successfully");

    Ok((db, temp_dir))
}

fn create_sample_users() -> Vec<User> {
    vec![
        User {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@tech.com".to_string(),
            department: "Engineering".to_string(),
            age: 28,
            created_at: 1234567890,
        },
        User {
            id: 2,
            name: "Bob Smith".to_string(),
            email: "bob@tech.com".to_string(),
            department: "Marketing".to_string(),
            age: 32,
            created_at: 1234567891,
        },
        User {
            id: 3,
            name: "Carol Davis".to_string(),
            email: "carol@tech.com".to_string(),
            department: "Engineering".to_string(),
            age: 29,
            created_at: 1234567892,
        },
        User {
            id: 4,
            name: "David Wilson".to_string(),
            email: "david@tech.com".to_string(),
            department: "Sales".to_string(),
            age: 35,
            created_at: 1234567893,
        },
    ]
}

fn create_sample_posts(users: &[User]) -> Vec<Post> {
    vec![
        Post {
            id: 1,
            title: "Rust Best Practices".to_string(),
            content: "Here are some Rust best practices...".to_string(),
            author_id: users[0].id,
            category: "Programming".to_string(),
            published: true,
            created_at: 1234567894,
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(users[0].id))),
        },
        Post {
            id: 2,
            title: "Database Design Patterns".to_string(),
            content: "Let's explore database design patterns...".to_string(),
            author_id: users[2].id,
            category: "Database".to_string(),
            published: true,
            created_at: 1234567895,
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(users[2].id))),
        },
        Post {
            id: 3,
            title: "Marketing Strategies".to_string(),
            content: "Effective marketing strategies for tech companies...".to_string(),
            author_id: users[1].id,
            category: "Marketing".to_string(),
            published: false,
            created_at: 1234567896,
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(users[1].id))),
        },
    ]
}

fn create_sample_comments(users: &[User], posts: &[Post]) -> Vec<Comment> {
    vec![
        Comment {
            id: 1,
            content: "Great article!".to_string(),
            post_id: posts[0].id,
            author_id: users[1].id,
            likes: 5,
            created_at: 1234567897,
            post: PostLink::from_key(PostKey::Primary(PostPrimaryKey(posts[0].id))),
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(users[1].id))),
        },
        Comment {
            id: 2,
            content: "Thanks for sharing!".to_string(),
            post_id: posts[0].id,
            author_id: users[2].id,
            likes: 3,
            created_at: 1234567898,
            post: PostLink::from_key(PostKey::Primary(PostPrimaryKey(posts[0].id))),
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(users[2].id))),
        },
        Comment {
            id: 3,
            content: "Very informative".to_string(),
            post_id: posts[1].id,
            author_id: users[3].id,
            likes: 7,
            created_at: 1234567899,
            post: PostLink::from_key(PostKey::Primary(PostPrimaryKey(posts[1].id))),
            author: UserLink::from_key(UserKey::Primary(UserPrimaryKey(users[3].id))),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secondary_key_querying() -> TestResult<()> {
        info!("Starting test_secondary_key_querying");

        let (db, _temp_dir) = create_test_database()?;
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

        // Insert sample users
        let users = create_sample_users();
        for user in &users {
            user_tree.insert(user.key(), user.clone())?;
        }
        info!("✓ {} users inserted", users.len());

        // Test querying by department (secondary key)
        debug!("Testing secondary key query by department");
        let engineering_users = user_tree
            .query_by_secondary_key(UserSecondaryKeys::DepartmentKey("Engineering".to_string()))?;

        debug!("Found {} engineering users", engineering_users.len());
        assert_eq!(engineering_users.len(), 2); // Alice and Carol

        for user in &engineering_users {
            assert_eq!(user.department, "Engineering");
            debug!("Engineering user: {} ({})", user.name, user.email);
        }
        info!("✓ Secondary key query by department successful");

        // Test querying by email (secondary key)
        debug!("Testing secondary key query by email");
        let alice_users = user_tree
            .query_by_secondary_key(UserSecondaryKeys::EmailKey("alice@tech.com".to_string()))?;

        assert_eq!(alice_users.len(), 1);
        assert_eq!(alice_users[0].name, "Alice Johnson");
        debug!("Found user by email: {}", alice_users[0].name);
        info!("✓ Secondary key query by email successful");

        // Test getting all secondary key values
        debug!("Testing secondary key value extraction");
        let department_values = user_tree.get_secondary_key_values("department")?;
        debug!("Found {} department values", department_values.len());
        assert!(!department_values.is_empty());
        info!("✓ Secondary key value extraction successful");

        info!("test_secondary_key_querying completed successfully");
        Ok(())
    }

    #[test]
    fn test_advanced_querying() -> TestResult<()> {
        info!("Starting test_advanced_querying");

        let (db, _temp_dir) = create_test_database()?;
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

        // Insert sample users
        let users = create_sample_users();
        for user in &users {
            user_tree.insert(user.key(), user.clone())?;
        }
        info!("✓ {} users inserted", users.len());

        // Test custom filter query
        debug!("Testing custom filter query");
        let senior_users = user_tree.query_with_filter(|user| user.age >= 30)?;

        debug!("Found {} senior users", senior_users.len());
        assert_eq!(senior_users.len(), 2); // Bob and David

        for (_key, user) in &senior_users {
            assert!(user.age >= 30);
            debug!("Senior user: {} (age: {})", user.name, user.age);
        }
        info!("✓ Custom filter query successful");

        // Test count with condition
        debug!("Testing count with condition");
        let engineering_count = user_tree.count_where(|user| user.department == "Engineering")?;
        assert_eq!(engineering_count, 2);
        debug!("Engineering users count: {}", engineering_count);
        info!("✓ Count with condition successful");

        // Test batch insert with indexing
        debug!("Testing batch insert with indexing");
        let new_users = vec![
            (
                UserKey::Primary(UserPrimaryKey(5)),
                User {
                    id: 5,
                    name: "Eve Brown".to_string(),
                    email: "eve@tech.com".to_string(),
                    department: "HR".to_string(),
                    age: 26,
                    created_at: 1234567900,
                },
            ),
            (
                UserKey::Primary(UserPrimaryKey(6)),
                User {
                    id: 6,
                    name: "Frank Miller".to_string(),
                    email: "frank@tech.com".to_string(),
                    department: "Engineering".to_string(),
                    age: 31,
                    created_at: 1234567901,
                },
            ),
        ];

        user_tree.batch_insert_with_indexing(new_users)?;
        assert_eq!(user_tree.len(), 6);
        debug!("Users after batch insert: {}", user_tree.len());
        info!("✓ Batch insert with indexing successful");

        info!("test_advanced_querying completed successfully");
        Ok(())
    }

    #[test]
    fn test_relational_queries() -> TestResult<()> {
        info!("Starting test_relational_queries");

        let (db, _temp_dir) = create_test_database()?;
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
        let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;
        let comment_tree: NetabaseSledTree<Comment, CommentKey> = db.get_main_tree()?;

        // Insert sample data
        let users = create_sample_users();
        let posts = create_sample_posts(&users);
        let comments = create_sample_comments(&users, &posts);

        for user in &users {
            user_tree.insert(user.key(), user.clone())?;
        }
        for post in &posts {
            post_tree.insert(post.key(), post.clone())?;
        }
        for comment in &comments {
            comment_tree.insert(comment.key(), comment.clone())?;
        }
        info!(
            "✓ Sample data inserted (users: {}, posts: {}, comments: {})",
            users.len(),
            posts.len(),
            comments.len()
        );

        // Test finding unresolved relations
        debug!("Testing unresolved relations detection");
        let unresolved_posts = post_tree.get_unresolved_relations()?;
        assert_eq!(unresolved_posts.len(), posts.len());

        for (_key, post) in &unresolved_posts {
            assert!(post.author.is_unresolved());
            debug!("Post '{}' has unresolved author relation", post.title);
        }
        info!("✓ Unresolved relations detection successful");

        // Test finding models that reference a specific key
        debug!("Testing referencing models query");
        let alice_key = UserKey::Primary(UserPrimaryKey(1));
        let posts_by_alice = post_tree.find_referencing_models(alice_key)?;
        debug!("Found {} posts referencing Alice", posts_by_alice.len());
        info!("✓ Referencing models query successful");

        info!("test_relational_queries completed successfully");
        Ok(())
    }

    #[test]
    fn test_relational_resolver() -> TestResult<()> {
        info!("Starting test_relational_resolver");

        let (db, _temp_dir) = create_test_database()?;
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

        // Insert sample users
        let users = create_sample_users();
        for user in &users {
            user_tree.insert(user.key(), user.clone())?;
        }

        // Create a resolver for User relations
        debug!("Creating relational resolver");
        let mut resolver = RelationalResolver::new(move |key: &UserKey| {
            if let UserKey::Primary(UserPrimaryKey(id)) = key {
                let user = users.iter().find(|u| u.id == *id).cloned();
                Ok(user)
            } else {
                Ok(None)
            }
        });

        // Test resolving a single link
        debug!("Testing single link resolution");
        let user_link = UserLink::from_key(UserKey::Primary(UserPrimaryKey(1)));
        let resolved_user = resolver.resolve(&user_link)?;

        assert!(resolved_user.is_some());
        let user = resolved_user.unwrap();
        assert_eq!(user.name, "Alice Johnson");
        debug!("Resolved user: {}", user.name);
        info!("✓ Single link resolution successful");

        // Test resolving multiple links
        debug!("Testing multiple link resolution");
        let links = vec![
            UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
            UserLink::from_key(UserKey::Primary(UserPrimaryKey(2))),
            UserLink::from_key(UserKey::Primary(UserPrimaryKey(999))), // Non-existent
        ];

        let resolved_users = resolver.resolve_many(&links)?;
        assert_eq!(resolved_users.len(), 3);
        assert!(resolved_users[0].is_some());
        assert!(resolved_users[1].is_some());
        assert!(resolved_users[2].is_none()); // Non-existent user

        debug!(
            "Resolved {} users, {} were found",
            resolved_users.len(),
            resolved_users.iter().filter(|u| u.is_some()).count()
        );
        info!("✓ Multiple link resolution successful");

        // Test cache statistics
        debug!("Testing cache statistics");
        let (cache_size, cache_capacity) = resolver.cache_stats();
        debug!("Cache size: {}, capacity: {}", cache_size, cache_capacity);
        assert!(cache_size > 0);
        info!("✓ Cache statistics successful");

        info!("test_relational_resolver completed successfully");
        Ok(())
    }

    #[test]
    fn test_complex_relational_scenarios() -> TestResult<()> {
        info!("Starting test_complex_relational_scenarios");

        let (db, _temp_dir) = create_test_database()?;
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
        let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;
        let comment_tree: NetabaseSledTree<Comment, CommentKey> = db.get_main_tree()?;

        // Insert sample data
        let users = create_sample_users();
        let posts = create_sample_posts(&users);
        let comments = create_sample_comments(&users, &posts);

        for user in &users {
            user_tree.insert(user.key(), user.clone())?;
        }
        for post in &posts {
            post_tree.insert(post.key(), post.clone())?;
        }
        for comment in &comments {
            comment_tree.insert(comment.key(), comment.clone())?;
        }

        // Test complex query: Find all comments by engineering department users
        debug!("Testing complex relational query");
        let engineering_users: Vec<_> = user_tree
            .query_by_secondary_key(UserSecondaryKeys::DepartmentKey("Engineering".to_string()))?
            .into_iter()
            .map(|u| u.id)
            .collect();

        debug!("Found {} engineering users", engineering_users.len());

        let engineering_comments = comment_tree
            .query_with_filter(|comment| engineering_users.contains(&comment.author_id))?;

        debug!(
            "Found {} comments by engineering users",
            engineering_comments.len()
        );
        assert!(engineering_comments.len() > 0);

        for (_key, comment) in &engineering_comments {
            assert!(engineering_users.contains(&comment.author_id));
            debug!("Engineering comment: '{}'", comment.content);
        }
        info!("✓ Complex relational query successful");

        // Test query: Find all posts in "Programming" category with their comments
        debug!("Testing posts with comments query");
        let programming_posts = post_tree
            .query_by_secondary_key(PostSecondaryKeys::CategoryKey("Programming".to_string()))?;

        for post in &programming_posts {
            let post_comments =
                comment_tree.query_by_secondary_key(CommentSecondaryKeys::Post_idKey(post.id))?;

            debug!("Post '{}' has {} comments", post.title, post_comments.len());

            for comment in &post_comments {
                assert_eq!(comment.post_id, post.id);
                debug!("  Comment: '{}'", comment.content);
            }
        }
        info!("✓ Posts with comments query successful");

        info!("test_complex_relational_scenarios completed successfully");
        Ok(())
    }

    #[test]
    fn test_range_queries_and_prefixes() -> TestResult<()> {
        info!("Starting test_range_queries_and_prefixes");

        let (db, _temp_dir) = create_test_database()?;
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

        // Insert users with specific ID patterns
        let users = vec![
            User {
                id: 1000,
                name: "User A".to_string(),
                email: "a@test.com".to_string(),
                department: "Dept A".to_string(),
                age: 25,
                created_at: 1234567890,
            },
            User {
                id: 1001,
                name: "User B".to_string(),
                email: "b@test.com".to_string(),
                department: "Dept A".to_string(),
                age: 26,
                created_at: 1234567891,
            },
            User {
                id: 2000,
                name: "User C".to_string(),
                email: "c@test.com".to_string(),
                department: "Dept B".to_string(),
                age: 27,
                created_at: 1234567892,
            },
        ];

        for user in &users {
            user_tree.insert(user.key(), user.clone())?;
        }
        info!("✓ Users with specific ID patterns inserted");

        // Test range queries (this is a simplified test since we don't have
        // direct access to the key serialization format)
        debug!("Testing range query by prefix");
        let prefix = b""; // Empty prefix to get all
        let all_users = user_tree.range_by_prefix(prefix)?;

        assert_eq!(all_users.len(), users.len());
        debug!("Range query returned {} users", all_users.len());
        info!("✓ Range query by prefix successful");

        info!("test_range_queries_and_prefixes completed successfully");
        Ok(())
    }

    #[test]
    fn test_database_level_queries() -> TestResult<()> {
        info!("Starting test_database_level_queries");

        let (db, _temp_dir) = create_test_database()?;

        // Test database-level secondary key indexing
        debug!("Testing database-level secondary key indexing");
        db.create_secondary_key_index::<User, UserKey, UserSecondaryKeys>("email")?;
        db.create_secondary_key_index::<User, UserKey, UserSecondaryKeys>("department")?;
        info!("✓ Database-level secondary key indexes created");

        // Insert sample data
        let users = create_sample_users();
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

        for user in &users {
            user_tree.insert(user.key(), user.clone())?;
        }

        // Test database-level secondary key querying
        debug!("Testing database-level secondary key querying");
        let engineering_users = db.query_by_secondary_key::<User, UserKey, UserSecondaryKeys>(
            UserSecondaryKeys::DepartmentKey("Engineering".to_string()),
        )?;

        assert_eq!(engineering_users.len(), 2);
        debug!(
            "Database-level query found {} engineering users",
            engineering_users.len()
        );
        info!("✓ Database-level secondary key querying successful");

        info!("test_database_level_queries completed successfully");
        Ok(())
    }

    #[test]
    fn test_relational_link_utilities() -> TestResult<()> {
        info!("Starting test_relational_link_utilities");

        // Test RelationalLink utility functions
        debug!("Testing RelationalLink utilities");

        let links = vec![
            UserLink::from_key(UserKey::Primary(UserPrimaryKey(1))),
            UserLink::from_object(User {
                id: 2,
                name: "Resolved User".to_string(),
                email: "resolved@test.com".to_string(),
                department: "Test".to_string(),
                age: 30,
                created_at: 1234567890,
            }),
            UserLink::from_key(UserKey::Primary(UserPrimaryKey(3))),
        ];

        // Test utility functions
        let keys = utils::extract_keys(&links);
        assert_eq!(keys.len(), 2); // Two unresolved links
        debug!("Extracted {} keys from links", keys.len());

        let has_unresolved = utils::has_unresolved_links(&links);
        assert!(has_unresolved);
        debug!("Has unresolved links: {}", has_unresolved);

        let unresolved_count = utils::count_unresolved(&links);
        assert_eq!(unresolved_count, 2);
        debug!("Unresolved links count: {}", unresolved_count);

        info!("✓ RelationalLink utilities successful");

        info!("test_relational_link_utilities completed successfully");
        Ok(())
    }
}
