use crate::error::{EncodingDecodingError, NetabaseError};
use bincode::{config::standard, decode_from_slice, encode_to_vec};

/// Trait for types that can be converted to/from byte vectors.
///
/// Only NetabaseDefinitionTrait and NetabaseDefinitionTraitKey should implement this trait.
/// This ensures type safety by preventing direct byte manipulation at lower levels.
pub trait ToIVec: bincode::Encode + bincode::Decode<()> + Sized {
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    /// Convert this type to a sled::IVec
    fn to_ivec(&self) -> Result<sled::IVec, NetabaseError> {
        let bytes = encode_to_vec(self, standard())
            .map_err(EncodingDecodingError::from)?;
        Ok(sled::IVec::from(bytes))
    }

    #[cfg(all(feature = "native", not(feature = "wasm")))]
    /// Convert from a sled::IVec to this type
    fn from_ivec(ivec: &sled::IVec) -> Result<Self, NetabaseError> {
        let (decoded, _) = decode_from_slice(&ivec[..], standard())
            .map_err(EncodingDecodingError::from)?;
        Ok(decoded)
    }

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    /// Convert this type to bytes (WASM version)
    fn to_ivec(&self) -> Result<Vec<u8>, NetabaseError> {
        let bytes = encode_to_vec(self, standard())
            .map_err(EncodingDecodingError::from)?;
        Ok(bytes)
    }

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    /// Convert from bytes to this type (WASM version)
    fn from_ivec(bytes: &[u8]) -> Result<Self, NetabaseError> {
        let (decoded, _) = decode_from_slice(bytes, standard())
            .map_err(EncodingDecodingError::from)?;
        Ok(decoded)
    }

    #[cfg(all(feature = "native", feature = "wasm"))]
    /// Convert this type to a sled::IVec (when both features are enabled, prefer native)
    fn to_ivec(&self) -> Result<sled::IVec, NetabaseError> {
        let bytes = encode_to_vec(self, standard())
            .map_err(EncodingDecodingError::from)?;
        Ok(sled::IVec::from(bytes))
    }

    #[cfg(all(feature = "native", feature = "wasm"))]
    /// Convert from a sled::IVec to this type (when both features are enabled, prefer native)
    fn from_ivec(ivec: &sled::IVec) -> Result<Self, NetabaseError> {
        let (decoded, _) = decode_from_slice(&ivec[..], standard())
            .map_err(EncodingDecodingError::from)?;
        Ok(decoded)
    }
}
