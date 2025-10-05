#[cfg(feature = "libp2p")]
pub mod record_store;
pub mod sled;

// Re-export the enhanced database as the primary implementation
pub use sled::{NetabaseIter, NetabaseSledDatabase, NetabaseSledTree, NetabaseTreeCompatible};

// Re-export libp2p-specific functionality when feature is enabled
#[cfg(feature = "libp2p")]
pub use sled::NetabaseRecordStoreExt;

#[cfg(feature = "libp2p")]
pub use record_store::{
    ProvidedIter, ProvidersListValue, RecordsIter, SledRecordStoreConfig, StoredProviderRecord,
};
