use bincode::{Decode, Encode};

pub trait NetabaseSchema:
    Clone
    + Send
    + From<::macro_exports::__netabase_libp2p_kad::Record>
    + Encode
    + Decode<()>
    + Into<::macro_exports::__netabase_libp2p_kad::Record> // TODO: TryFrom
{
    type Key: NetabaseSchemaKey;
    fn key(&self) -> Self::Key;
}

/// Refactored NetabaseSchemaKey trait that supports unwrapped key types
///
/// This trait now allows users to pass unwrapped key types directly to query methods,
/// while the library handles the wrapping/unwrapping internally during serialization.
pub trait NetabaseSchemaKey:
    Clone
    + Send
    + From<::macro_exports::__netabase_libp2p_kad::RecordKey>
    + Encode
    + Decode<()>
    + Into<::macro_exports::__netabase_libp2p_kad::RecordKey>
{
}
