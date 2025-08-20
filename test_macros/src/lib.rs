use bincode::{Decode, Encode};
use netabase::{NetabaseSchema, NetabaseSchemaKey};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Encode, Decode, NetabaseSchema)]
pub struct UserRandom {
    #[key]
    pub id: u128,
    pub name: String,
    pub another: String,
}

fn main() {
    println!("Macro compilation test successful!");

    let user = UserRandom {
        id: 1,
        name: "Test User".to_string(),
        another: "Another field".to_string(),
    };

    test_schema_implementation();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_implementation() {
        let user = UserRandom {
            id: 123,
            name: "Test User".to_string(),
            another: "Another field".to_string(),
        };

        // Test that the NetabaseSchema trait is implemented
        let key = user.key();
        println!("Generated key: {}", key);

        // Test that Clone is working
        let user_clone = user.clone();
        assert_eq!(user_clone.id, user.id);
        assert_eq!(user_clone.name, user.name);
        assert_eq!(user_clone.another, user.another);
    }
}

fn test_schema_implementation() {
    let user = UserRandom {
        id: 456,
        name: "Runtime Test".to_string(),
        another: "Another field".to_string(),
    };

    let key = user.key();
    println!("Generated key: {}", key);
    println!("Schema implementation test passed!");
}
