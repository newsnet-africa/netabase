use libp2p::websocket::error;
use netabase_store::errors::NetabaseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("There was a netabase store error")]
    NetabaseStoreError(#[from] NetabaseError),
    #[error("There was an IO error")]
    IoError(#[from] std::io::Error),
}
