#[derive(Debug, thiserror::Error)]
pub enum NetabaseError {
    #[error("There was a conversion Error")]
    Conversion(#[from] EncodingDecodingError),
    #[cfg(feature = "native")]
    #[error("There was an error with the Sled database")]
    DatabaseError(#[from] sled::Error),
    #[error("Storage error: {0}")]
    Storage(String),
}

#[derive(Debug, thiserror::Error)]
pub enum EncodingDecodingError {
    #[error("There was an error encoding type")]
    Encoding(#[from] bincode::error::EncodeError),
    #[error("There was an error decoding type")]
    Decoding(#[from] bincode::error::DecodeError),
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("There was an error opening the store tree")]
    OpenTreeError,
    #[error("There was an error opening the store DB")]
    OpenDBError,
}
