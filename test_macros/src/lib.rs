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
    }
}

fn main() {
    println!("Macro compilation test successful!");

    let user = User {
        id: 1,
        name: "Test User".to_string(),
    };
}
