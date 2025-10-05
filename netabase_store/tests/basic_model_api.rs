use bincode::{Decode, Encode};
use log::{debug, error, info, warn};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::{
    database::{NetabaseDatabase, NetabaseTree},
    traits::{NetabaseModel, NetabaseModelKey, NetabaseSchema},
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

fn create_test_database() -> TestResult<(NetabaseDatabase<BasicTestSchema>, TempDir)> {
    init_logger();
    info!("Creating basic test database in temporary directory");

    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("basic_test_db");
    debug!("Basic test database path: {}", db_path.display());

    let db = NetabaseDatabase::new_with_path(&db_path)?;
    info!("Basic test database created successfully");

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
        info!("Starting test_database_initialization");

        let (db, _temp_dir) = create_test_database()?;
        info!("Database created successfully for initialization test");

        // Test that database was created successfully
        let was_recovered = db.db().was_recovered();
        debug!("Database was_recovered: {}", was_recovered);
        assert!(!was_recovered);
        info!("✓ Database initialization check passed");

        // Test tree name generation
        let tree_names = db.tree_names();
        debug!("Tree names: {:?}", tree_names);
        assert!(!tree_names.is_empty());
        info!("✓ Tree name generation check passed");

        info!("test_database_initialization completed successfully");
        Ok(())
    }

    #[test]
    fn test_model_discriminants() -> TestResult<()> {
        info!("Starting test_model_discriminants");

        // Test that each model returns its own discriminant
        debug!("Testing User discriminant");
        let user_discriminant = User::tree_name();
        assert_eq!(user_discriminant, "User");
        info!("✓ User discriminant is correct: {}", user_discriminant);

        debug!("Testing Post discriminant");
        let post_discriminant = Post::tree_name();
        assert_eq!(post_discriminant, "Post");
        info!("✓ Post discriminant is correct: {}", post_discriminant);

        info!("test_model_discriminants completed successfully");
        Ok(())
    }

    #[test]
    fn test_model_based_tree_creation() -> TestResult<()> {
        info!("Starting test_model_based_tree_creation");

        let (db, _temp_dir) = create_test_database()?;
        info!("Database created successfully for tree creation test");

        // Test User tree creation using the new API
        debug!("Creating User tree");
        let user_tree: NetabaseTree<User, UserKey> = db.get_main_tree()?;
        assert_eq!(user_tree.len(), 0);
        info!("✓ User tree created and verified empty");

        // Test Post tree creation using the new API
        debug!("Creating Post tree");
        let post_tree: NetabaseTree<Post, PostKey> = db.get_main_tree()?;
        assert_eq!(post_tree.len(), 0);
        info!("✓ Post tree created and verified empty");

        info!("test_model_based_tree_creation completed successfully");
        Ok(())
    }

    #[test]
    fn test_basic_crud_operations() -> TestResult<()> {
        info!("Starting test_basic_crud_operations");

        let (db, _temp_dir) = create_test_database()?;
        info!("Database created successfully for CRUD operations test");

        // Create trees
        debug!("Creating trees for CRUD operations");
        let user_tree: NetabaseTree<User, UserKey> = db.get_main_tree()?;
        let post_tree: NetabaseTree<Post, PostKey> = db.get_main_tree()?;
        info!("✓ Trees created successfully");

        // Create sample data
        debug!("Creating sample data");
        let user = create_sample_user(1);
        let post = create_sample_post(1, user.id);
        info!(
            "✓ Sample data created (user id: {}, post id: {})",
            user.id, post.id
        );

        // Test insert operations
        debug!("Testing insert operations");
        user_tree.insert(user.key(), user.clone())?;
        debug!("User inserted with id: {}", user.id);
        post_tree.insert(post.key(), post.clone())?;
        debug!("Post inserted with id: {}", post.id);

        assert_eq!(user_tree.len(), 1);
        assert_eq!(post_tree.len(), 1);
        info!("✓ Insert operations completed successfully");

        // Test get operations
        debug!("Testing get operations");
        let loaded_user = user_tree.get(user.key())?.unwrap();
        assert_eq!(loaded_user.id, user.id);
        assert_eq!(loaded_user.name, user.name);
        assert_eq!(loaded_user.email, user.email);
        info!("✓ User loaded and verified successfully");

        let loaded_post = post_tree.get(post.key())?.unwrap();
        assert_eq!(loaded_post.id, post.id);
        assert_eq!(loaded_post.title, post.title);
        assert_eq!(loaded_post.author_id, user.id);
        info!("✓ Post loaded and verified successfully");

        // Test contains_key
        debug!("Testing contains_key operations");
        assert!(user_tree.contains_key(user.key())?);
        assert!(post_tree.contains_key(post.key())?);
        info!("✓ Contains_key operations verified successfully");

        // Test remove operations
        debug!("Testing remove operations");
        let removed_user = user_tree.remove(user.key())?.unwrap();
        assert_eq!(removed_user.id, user.id);
        assert_eq!(user_tree.len(), 0);
        debug!("User removed successfully");

        let removed_post = post_tree.remove(post.key())?.unwrap();
        assert_eq!(removed_post.id, post.id);
        assert_eq!(post_tree.len(), 0);
        debug!("Post removed successfully");
        info!("✓ Remove operations completed successfully");

        info!("test_basic_crud_operations completed successfully");
        Ok(())
    }

    #[test]
    fn test_multiple_models_same_database() -> TestResult<()> {
        info!("Starting test_multiple_models_same_database");

        let (db, _temp_dir) = create_test_database()?;
        info!("Database created successfully for multiple models test");

        // Create trees for different models
        debug!("Creating trees for multiple models");
        let user_tree: NetabaseTree<User, UserKey> = db.get_main_tree()?;
        let post_tree: NetabaseTree<Post, PostKey> = db.get_main_tree()?;
        info!("✓ Trees created for multiple models");

        // Create multiple users and posts
        debug!("Creating multiple entities");
        let user1 = create_sample_user(1);
        let user2 = create_sample_user(2);
        let post1 = create_sample_post(1, user1.id);
        let post2 = create_sample_post(2, user2.id);
        info!("✓ Multiple entities created (2 users, 2 posts)");

        // Store all data
        debug!("Storing multiple entities");
        user_tree.insert(user1.key(), user1.clone())?;
        user_tree.insert(user2.key(), user2.clone())?;
        post_tree.insert(post1.key(), post1.clone())?;
        post_tree.insert(post2.key(), post2.clone())?;
        info!("✓ All entities stored successfully");

        // Verify storage
        debug!("Verifying storage counts");
        assert_eq!(user_tree.len(), 2);
        assert_eq!(post_tree.len(), 2);
        info!("✓ Storage counts verified");

        // Verify data integrity
        debug!("Verifying data integrity");
        let loaded_user1 = user_tree.get(user1.key())?.unwrap();
        let loaded_user2 = user_tree.get(user2.key())?.unwrap();
        let loaded_post1 = post_tree.get(post1.key())?.unwrap();
        let loaded_post2 = post_tree.get(post2.key())?.unwrap();

        assert_eq!(loaded_user1.id, 1);
        assert_eq!(loaded_user2.id, 2);
        assert_eq!(loaded_post1.author_id, 1);
        assert_eq!(loaded_post2.author_id, 2);
        info!("✓ Data integrity verified for all entities");

        info!("test_multiple_models_same_database completed successfully");
        Ok(())
    }

    #[test]
    fn test_tree_iteration() -> TestResult<()> {
        info!("Starting test_tree_iteration");

        let (db, _temp_dir) = create_test_database()?;
        info!("Database created successfully for tree iteration test");

        let user_tree: NetabaseTree<User, UserKey> = db.get_main_tree()?;
        debug!("User tree created for iteration test");

        // Insert multiple users
        debug!("Inserting multiple users for iteration test");
        for i in 1..=5 {
            let user = create_sample_user(i);
            user_tree.insert(user.key(), user)?;
            debug!("Inserted user with id: {}", i);
        }
        info!("✓ 5 users inserted successfully");

        // Test iteration
        debug!("Testing tree iteration");
        let mut count = 0;
        for result in user_tree.iter() {
            let (_key, user) = result?;
            assert!(user.id >= 1 && user.id <= 5);
            debug!("Iterated over user with id: {}", user.id);
            count += 1;
        }
        assert_eq!(count, 5);
        info!(
            "✓ Tree iteration completed successfully, processed {} users",
            count
        );

        info!("test_tree_iteration completed successfully");
        Ok(())
    }

    #[test]
    fn test_key_extraction_methods() -> TestResult<()> {
        info!("Starting test_key_extraction_methods");

        // Test that the new primary_keys() and secondary_keys() methods work correctly
        debug!("Creating sample user for key extraction test");
        let user = create_sample_user(1);
        let user_key = user.key();
        info!("✓ Sample user created with id: {}", user.id);

        // Test primary key extraction
        debug!("Testing primary key extraction");
        if let Some(primary_key) = user_key.primary_keys() {
            // We can access the primary key, but it's a UserPrimaryKey newtype
            debug!("Found primary key: {:?}", primary_key);
            info!("✓ Primary key extraction successful");
        } else {
            error!("Expected primary key but got None");
            panic!("Expected primary key but got None");
        }

        // Test secondary key extraction - should be None for primary key variant
        debug!("Testing secondary key extraction for primary key variant");
        assert!(user_key.secondary_keys().is_none());
        info!("✓ Secondary key extraction correctly returned None for primary key");

        // Create a secondary key variant to test
        debug!("Creating secondary key variant for testing");
        use test_schema::UserSecondaryKeys;
        let secondary_key_variant = UserSecondaryKeys::EmailKey(user.email.clone());
        let secondary_user_key = test_schema::UserKey::Secondary(secondary_key_variant);
        debug!("Secondary key variant created with email: {}", user.email);

        // Test secondary key extraction
        debug!("Testing secondary key extraction for secondary key variant");
        if let Some(secondary_key) = secondary_user_key.secondary_keys() {
            debug!("Found secondary key: {:?}", secondary_key);
            info!("✓ Secondary key extraction successful");
        } else {
            error!("Expected secondary key but got None");
            panic!("Expected secondary key but got None");
        }

        // Test primary key extraction - should be None for secondary key variant
        debug!("Testing primary key extraction for secondary key variant");
        assert!(secondary_user_key.primary_keys().is_none());
        info!("✓ Primary key extraction correctly returned None for secondary key");

        info!("test_key_extraction_methods completed successfully");
        Ok(())
    }
}
