#[cfg(feature = "native")]
use libp2p::websocket::error;
use netabase_store::error::NetabaseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("There was a netabase store error")]
    NetabaseStoreError(#[from] NetabaseError),
    #[error("There was an IO error")]
    IoError(#[from] std::io::Error),
    #[cfg(feature = "native")]
    #[error("Websocket error")]
    WebsocketError(#[from] error::Error<std::io::Error>),
    #[error("Database error: {0}")]
    Database(String),
}
