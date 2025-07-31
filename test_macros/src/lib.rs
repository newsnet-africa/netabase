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

#[derive(Clone)]
struct User {
    #[key]
    id: u128,
    name: String,
    another: String,
}

fn main() {
    println!("Macro compilation test successful!");

    let user = User {
        id: 1,
        name: "Test User".to_string(),
        another: "Another field".to_string(),
    };
}
