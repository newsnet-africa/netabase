use bincode::{Decode, Encode};
use netabase::{database::NetabaseSledDatabase, traits::NetabaseSchema};
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
    use strum::IntoEnumIterator;

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
    fn test_discriminant_based_tree_access() -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Testing Discriminant-based Tree Access ===");

        let mut db = NetabaseSledDatabase::<TestSchema>::new_with_name("test_discriminants")?;

        let discriminants = TestSchema::all_schema_discriminants();
        db.initialize_trees_from_discriminants(&discriminants)?;

        for discriminant in &discriminants {
            println!("Testing discriminant: {:?}", discriminant);
            let tree = db.get_main_tree(discriminant)?;
            println!("  Tree length: {}", tree.len());
        }

        Ok(())
    }

    #[test]
    fn test_discriminant_access_by_model() -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Testing Model-specific Discriminant Access ===");

        let discriminants = TestSchema::all_schema_discriminants();

        // Find User discriminant
        let user_discriminant = discriminants
            .iter()
            .find(|d| d.as_ref() == "User")
            .ok_or("User discriminant not found")?;

        // Find Post discriminant
        let post_discriminant = discriminants
            .iter()
            .find(|d| d.as_ref() == "Post")
            .ok_or("Post discriminant not found")?;

        println!("User discriminant: {:?}", user_discriminant);
        println!("Post discriminant: {:?}", post_discriminant);

        // Test tree access with specific discriminants
        let mut db = NetabaseSledDatabase::<TestSchema>::new_with_name("test_model_discriminants")?;
        db.initialize_trees_from_discriminants(&discriminants)?;

        let user_tree = db.get_main_tree(user_discriminant)?;
        let post_tree = db.get_main_tree(post_discriminant)?;

        println!("User tree length: {}", user_tree.len());
        println!("Post tree length: {}", post_tree.len());

        Ok(())
    }
}
