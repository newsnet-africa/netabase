use bincode::{Decode, Encode};

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

<<<<<<< HEAD
    #[derive(Clone)]
<<<<<<< HEAD
    struct User {
        #[key]
=======
    #[key = |i: User2| i ]
    struct User2 {
>>>>>>> 9ebb163c7b1984ab70d5bbe2ab7aa48824850724
        id: u128,
        name: String,
        another: String
=======
pub trait NetabaseSchemaKey:
    Clone
    + From<libp2p::kad::RecordKey>
    + Encode
    + Decode<()>
    + for<'de> bincode::BorrowDecode<'de, ()>
    + Into<libp2p::kad::RecordKey>
{
}

// Simple struct with field key
#[derive(Clone, Encode, Decode, netabase_macros::NetabaseSchema)]
pub struct User {
    #[key]
    pub id: u64,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_compilation() {
        let user = User {
            id: 1,
            name: "Test".to_string(),
        };

        // Test that get_key method exists
        let _key_bytes = user.key().as_bytes();
>>>>>>> 4740b930844447b717a06adb472169f5fb202c37
    }
}

fn main() {
    println!("Macro compilation test successful!");

    let user = User {
        id: 1,
        name: "Test User".to_string(),
    };
}
