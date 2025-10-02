use bincode::{Decode, Encode};
use netabase::{
    database::{NetabaseSledDatabase, NetabaseSledTree},
    traits::{NetabaseModel, NetabaseModelKey, NetabaseSchema},
};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[netabase_schema_module(BasicTestSchema, BasicTestSchemaKey)]
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
        pub published: bool,
        pub created_at: u64,
    }
}

use test_schema::*;

type TestResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn create_test_database() -> TestResult<(NetabaseSledDatabase<BasicTestSchema>, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("basic_test_db");
    let db = NetabaseSledDatabase::new_with_name(&db_path.to_string_lossy())?;
    Ok((db, temp_dir))
}

fn create_sample_user(id: u64) -> User {
    User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
        created_at: 1234567890,
    }
}

fn create_sample_post(id: u64, author_id: u64) -> Post {
    Post {
        id,
        title: format!("Post {}", id),
        content: format!("Content of post {}", id),
        author_id,
        published: true,
        created_at: 1234567891,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_initialization() -> TestResult<()> {
        let (db, _temp_dir) = create_test_database()?;

        // Test that database was created successfully
        assert!(!db.db().was_recovered());

        // Test tree name generation
        let tree_names = db.tree_names();
        assert!(!tree_names.is_empty());

        Ok(())
    }

    #[test]
    fn test_model_discriminants() -> TestResult<()> {
        // Test that each model returns its own discriminant
        let user_discriminant = User::tree_name();
        let post_discriminant = Post::tree_name();

        assert_eq!(user_discriminant, "User");
        assert_eq!(post_discriminant, "Post");

        Ok(())
    }

    #[test]
    fn test_model_based_tree_creation() -> TestResult<()> {
        let (db, _temp_dir) = create_test_database()?;

        // Test User tree creation using the new API
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
        assert_eq!(user_tree.len(), 0);

        // Test Post tree creation using the new API
        let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;
        assert_eq!(post_tree.len(), 0);

        Ok(())
    }

    #[test]
    fn test_basic_crud_operations() -> TestResult<()> {
        let (db, _temp_dir) = create_test_database()?;

        // Create trees
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
        let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;

        // Create sample data
        let user = create_sample_user(1);
        let post = create_sample_post(1, user.id);

        // Test insert operations
        user_tree.insert(user.key(), user.clone())?;
        post_tree.insert(post.key(), post.clone())?;

        assert_eq!(user_tree.len(), 1);
        assert_eq!(post_tree.len(), 1);

        // Test get operations
        let loaded_user = user_tree.get(user.key())?.unwrap();
        assert_eq!(loaded_user.id, user.id);
        assert_eq!(loaded_user.name, user.name);
        assert_eq!(loaded_user.email, user.email);

        let loaded_post = post_tree.get(post.key())?.unwrap();
        assert_eq!(loaded_post.id, post.id);
        assert_eq!(loaded_post.title, post.title);
        assert_eq!(loaded_post.author_id, user.id);

        // Test contains_key
        assert!(user_tree.contains_key(user.key())?);
        assert!(post_tree.contains_key(post.key())?);

        // Test remove operations
        let removed_user = user_tree.remove(user.key())?.unwrap();
        assert_eq!(removed_user.id, user.id);
        assert_eq!(user_tree.len(), 0);

        let removed_post = post_tree.remove(post.key())?.unwrap();
        assert_eq!(removed_post.id, post.id);
        assert_eq!(post_tree.len(), 0);

        Ok(())
    }

    #[test]
    fn test_multiple_models_same_database() -> TestResult<()> {
        let (db, _temp_dir) = create_test_database()?;

        // Create trees for different models
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
        let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;

        // Create multiple users and posts
        let user1 = create_sample_user(1);
        let user2 = create_sample_user(2);
        let post1 = create_sample_post(1, user1.id);
        let post2 = create_sample_post(2, user2.id);

        // Store all data
        user_tree.insert(user1.key(), user1.clone())?;
        user_tree.insert(user2.key(), user2.clone())?;
        post_tree.insert(post1.key(), post1.clone())?;
        post_tree.insert(post2.key(), post2.clone())?;

        // Verify storage
        assert_eq!(user_tree.len(), 2);
        assert_eq!(post_tree.len(), 2);

        // Verify data integrity
        let loaded_user1 = user_tree.get(user1.key())?.unwrap();
        let loaded_user2 = user_tree.get(user2.key())?.unwrap();
        let loaded_post1 = post_tree.get(post1.key())?.unwrap();
        let loaded_post2 = post_tree.get(post2.key())?.unwrap();

        assert_eq!(loaded_user1.id, 1);
        assert_eq!(loaded_user2.id, 2);
        assert_eq!(loaded_post1.author_id, 1);
        assert_eq!(loaded_post2.author_id, 2);

        Ok(())
    }

    #[test]
    fn test_tree_iteration() -> TestResult<()> {
        let (db, _temp_dir) = create_test_database()?;
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

        // Insert multiple users
        for i in 1..=5 {
            let user = create_sample_user(i);
            user_tree.insert(user.key(), user)?;
        }

        // Test iteration
        let mut count = 0;
        for result in user_tree.iter() {
            let (key, user) = result?;
            assert!(user.id >= 1 && user.id <= 5);
            count += 1;
        }
        assert_eq!(count, 5);

        Ok(())
    }

    #[test]
    fn test_key_extraction_methods() -> TestResult<()> {
        // Test that the new primary_keys() and secondary_keys() methods work correctly
        let user = create_sample_user(1);
        let user_key = user.key();

        // Test primary key extraction
        if let Some(primary_key) = user_key.primary_keys() {
            // We can access the primary key, but it's a UserPrimaryKey newtype
            println!("Found primary key: {:?}", primary_key);
        } else {
            panic!("Expected primary key but got None");
        }

        // Test secondary key extraction - should be None for primary key variant
        assert!(user_key.secondary_keys().is_none());

        // Create a secondary key variant to test
        use test_schema::UserSecondaryKeys;
        let secondary_key_variant = UserSecondaryKeys::EmailKey(user.email.clone());
        let secondary_user_key = test_schema::UserKey::Secondary(secondary_key_variant);

        // Test secondary key extraction
        if let Some(secondary_key) = secondary_user_key.secondary_keys() {
            println!("Found secondary key: {:?}", secondary_key);
        } else {
            panic!("Expected secondary key but got None");
        }

        // Test primary key extraction - should be None for secondary key variant
        assert!(secondary_user_key.primary_keys().is_none());

        Ok(())
    }
}
