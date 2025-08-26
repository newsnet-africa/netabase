use bincode::{Decode, Encode};

pub trait NetabaseSchema:
    Clone
    + From<libp2p::kad::Record>
    + Encode
    + for<'de> bincode::BorrowDecode<'de, ()>
    + Into<libp2p::kad::Record>
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
    /// The inner key type that this key struct wraps
    type Inner: Clone + Encode + Decode<()>;

    /// Create a new key from the unwrapped inner type
    fn from_inner(inner: Self::Inner) -> Self;

    /// Extract the inner key value
    fn into_inner(self) -> Self::Inner;

    /// Get a reference to the inner key value
    fn inner(&self) -> &Self::Inner;
}
