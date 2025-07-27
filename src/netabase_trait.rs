use bincode::{Decode, Encode};

pub trait NetabaseSchema: Clone + From<libp2p::kad::Record> + Encode + Decode<()> {
    type Key: NetabaseSchemaKey;
    fn key(&self) -> Self::Key;
}

pub trait NetabaseSchemaKey: Clone + From<libp2p::kad::Record> + Encode + Decode<()> {
    fn generate_key() -> Self;
}
