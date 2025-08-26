use bincode::{Decode, Encode};

pub trait NetabaseSchema:
    Clone + From<libp2p::kad::Record> + Encode + Decode<()> + Into<libp2p::kad::Record>
{
    type Key: NetabaseSchemaKey;
    fn key(&self) -> Self::Key;
}

/// Refactored NetabaseSchemaKey trait that supports unwrapped key types
///
/// This trait now allows users to pass unwrapped key types directly to query methods,
/// while the library handles the wrapping/unwrapping internally during serialization.
pub trait NetabaseSchemaKey:
    Clone + From<libp2p::kad::RecordKey> + Encode + Decode<()> + Into<libp2p::kad::RecordKey>
{
}
