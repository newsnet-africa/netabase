use bincode::{Decode, Encode};
use netabase::{
    database::{NetabaseSledDatabase, NetabaseSledTree},
    traits::{NetabaseModel, NetabaseSchema},
};
use netabase_macros::NetabaseModel;
use netabase_macros::netabase_schema_module;
use serde::{Deserialize, Serialize};

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
        println!("\n=== Testing Schema Discriminants ===");

        // Get all discriminants
        let discriminants = TestSchema::all_schema_discriminants();
        println!("Number of discriminants: {}", discriminants.len());

        for (i, discriminant) in discriminants.iter().enumerate() {
            println!(
                "Discriminant {}: {:?} -> '{}'",
                i,
                discriminant,
                discriminant.as_ref()
            );
        }

        // Test discriminant enum iteration
        println!("\n=== Testing Enum Iteration ===");
        for (i, discriminant) in
            <TestSchemaDiscriminants as strum::IntoEnumIterator>::iter().enumerate()
        {
            println!(
                "Enum variant {}: {:?} -> '{}'",
                i,
                discriminant,
                discriminant.as_ref()
            );
        }
    }

    #[test]
    fn test_model_based_tree_access() -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Testing Model-based Tree Access ===");

        let db = NetabaseSledDatabase::<TestSchema>::new_with_name("test_model_trees")?;

        // Test User tree creation
        println!("Creating User tree...");
        let user_tree: NetabaseSledTree<test_schema::User, test_schema::UserKey> =
            db.get_main_tree()?;
        println!("  User tree length: {}", user_tree.len());

        // Test Post tree creation
        println!("Creating Post tree...");
        let post_tree: NetabaseSledTree<test_schema::Post, test_schema::PostKey> =
            db.get_main_tree()?;
        println!("  Post tree length: {}", post_tree.len());

        Ok(())
    }

    #[test]
    fn test_model_discriminants() -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Testing Model-specific Discriminant Access ===");

        // Test that each model returns its own discriminant
        let user_discriminant = test_schema::User::tree_name();
        let post_discriminant = test_schema::Post::tree_name();

        println!("User discriminant: {:?}", user_discriminant);
        println!("Post discriminant: {:?}", post_discriminant);

        assert_eq!(user_discriminant, "User");
        assert_eq!(post_discriminant, "Post");

        // Test that trees can be created using the model types directly
        let db = NetabaseSledDatabase::<TestSchema>::new_with_name("test_model_discriminants")?;

        let user_tree: NetabaseSledTree<test_schema::User, test_schema::UserKey> =
            db.get_main_tree()?;
        let post_tree: NetabaseSledTree<test_schema::Post, test_schema::PostKey> =
            db.get_main_tree()?;

        println!("User tree length: {}", user_tree.len());
        println!("Post tree length: {}", post_tree.len());

        Ok(())
    }
}
