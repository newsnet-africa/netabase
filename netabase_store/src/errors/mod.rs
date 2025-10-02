use crate::errors::conversion::ConversionError;
use thiserror::Error;

pub mod conversion;

#[derive(Error, Debug)]
pub enum NetabaseError {
    #[error("There was an error converting types: {0}")]
    Conversion(#[from] ConversionError),
    #[error("There was an error with the database")]
    Database,
    #[error("There was an error with serialization")]
    Serialization,
}

impl From<bincode::error::DecodeError> for NetabaseError {
    fn from(value: bincode::error::DecodeError) -> Self {
        Self::Conversion(ConversionError::MacroConversion)
    }
}
impl From<bincode::error::EncodeError> for NetabaseError {
    fn from(value: bincode::error::EncodeError) -> Self {
        Self::Conversion(ConversionError::MacroConversion)
    }
}

impl From<sled::Error> for NetabaseError {
    fn from(_value: sled::Error) -> Self {
        Self::Database
    }
}
