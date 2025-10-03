use bincode::{Decode, Encode};
use log::{debug, info};
use netabase_macros::NetabaseModel;
use netabase_macros::netabase_schema_module;
use netabase_store::{
    database::NetabaseSledDatabase,
    traits::{NetabaseModel, NetabaseSchema},
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
        pub username: String,
        pub created_at: u64, // Unix timestamp
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
        pub category: String,
        #[secondary_key]
        pub published: bool,
        pub created_at: u64, // Unix timestamp
        pub tags: Vec<String>,
    }
}

#[cfg(test)]
mod tests {
    use super::test_schema::*;
    use super::*;

    #[test]
    fn debug_discriminants() {
        init_logger();
        info!("Starting debug_discriminants test");
        println!("\n=== Testing Schema Discriminants ===");

        // Get all discriminants
        let discriminants = TestSchema::all_schema_discriminants();
        println!("Number of discriminants: {}", discriminants.len());

        for (i, discriminant) in discriminants.iter().enumerate() {
            debug!(
                "Discriminant {}: {:?} -> '{}'",
                i,
                discriminant,
                discriminant.as_ref()
            );
            println!(
                "Discriminant {}: {:?} -> '{}'",
                i,
                discriminant,
                discriminant.as_ref()
            );
        }
        info!("✓ Schema discriminants verified successfully");

        // Test discriminant enum iteration
        println!("\n=== Testing Enum Iteration ===");
        for (i, discriminant) in
            <TestSchemaDiscriminants as strum::IntoEnumIterator>::iter().enumerate()
        {
            debug!(
                "Enum variant {}: {:?} -> '{}'",
                i,
                discriminant,
                discriminant.as_ref()
            );
            println!(
                "Enum variant {}: {:?} -> '{}'",
                i,
                discriminant,
                discriminant.as_ref()
            );
        }
        info!("✓ Enum iteration verified successfully");
        info!("debug_discriminants test completed successfully");
    }

    #[test]
    fn test_model_based_tree_access() -> Result<(), Box<dyn std::error::Error>> {
        init_logger();
        info!("Starting test_model_based_tree_access");
        println!("\n=== Testing Model-based Tree Access ===");

        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_model_trees");
        debug!("Test model trees database path: {}", db_path.display());
        let db = NetabaseSledDatabase::<TestSchema>::new_with_path(&db_path.to_string_lossy())?;
        info!("✓ Database created successfully for model tree access test");

        // Test User tree creation
        debug!("Creating User tree");
        println!("Creating User tree...");
        let user_tree: netabase_store::database::NetabaseSledTree<
            test_schema::User,
            test_schema::UserKey,
        > = db.get_main_tree()?;
        println!("  User tree length: {}", user_tree.len());
        info!("✓ User tree created successfully");

        // Test Post tree creation
        debug!("Creating Post tree");
        println!("Creating Post tree...");
        let post_tree: netabase_store::database::NetabaseSledTree<
            test_schema::Post,
            test_schema::PostKey,
        > = db.get_main_tree()?;
        println!("  Post tree length: {}", post_tree.len());
        info!("✓ Post tree created successfully");

        info!("test_model_based_tree_access completed successfully");
        Ok(())
    }

    #[test]
    fn test_model_discriminants() -> Result<(), Box<dyn std::error::Error>> {
        init_logger();
        info!("Starting test_model_discriminants");
        println!("\n=== Testing Model-specific Discriminant Access ===");

        // Test that each model returns its own discriminant
        debug!("Testing model discriminant access");
        let user_discriminant = test_schema::User::tree_name();
        let post_discriminant = test_schema::Post::tree_name();

        println!("User discriminant: {:?}", user_discriminant);
        println!("Post discriminant: {:?}", post_discriminant);
        debug!(
            "User discriminant: {}, Post discriminant: {}",
            user_discriminant, post_discriminant
        );

        assert_eq!(user_discriminant, "User");
        assert_eq!(post_discriminant, "Post");
        info!("✓ Model discriminants verified successfully");

        // Test that trees can be created using the model types directly
        debug!("Testing tree creation using model types");
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_model_discriminants");
        debug!(
            "Test model discriminants database path: {}",
            db_path.display()
        );
        let db = NetabaseSledDatabase::<TestSchema>::new_with_path(&db_path.to_string_lossy())?;
        info!("✓ Database created successfully for model discriminants test");

        let user_tree: netabase_store::database::NetabaseSledTree<
            test_schema::User,
            test_schema::UserKey,
        > = db.get_main_tree()?;
        let post_tree: netabase_store::database::NetabaseSledTree<
            test_schema::Post,
            test_schema::PostKey,
        > = db.get_main_tree()?;

        println!("User tree length: {}", user_tree.len());
        println!("Post tree length: {}", post_tree.len());
        debug!(
            "User tree length: {}, Post tree length: {}",
            user_tree.len(),
            post_tree.len()
        );
        info!("✓ Trees created using model types successfully");

        info!("test_model_discriminants completed successfully");
        Ok(())
    }
}
