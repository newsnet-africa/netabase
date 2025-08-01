use bincode::{Decode, Encode};
use netabase_macros::NetabaseSchema;

// Local trait definitions for testing
pub trait NetabaseSchema:
    Clone
    + From<libp2p::kad::Record>
    + Encode
    + Decode<()>
    + for<'de> bincode::BorrowDecode<'de, ()>
    + Into<libp2p::kad::Record>
{
    type Key: NetabaseSchemaKey;
    fn key(&self) -> Self::Key;
}

pub trait NetabaseSchemaKey:
    Clone
    + From<libp2p::kad::RecordKey>
    + Encode
    + Decode<()>
    + for<'de> bincode::BorrowDecode<'de, ()>
    + Into<libp2p::kad::RecordKey>
{
}

#[derive(Clone, Encode, Decode, NetabaseSchema)]
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
        assert_eq!(key, "placeholder".to_string());

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
    assert_eq!(key, "placeholder".to_string());
    println!("Schema implementation test passed!");
}
