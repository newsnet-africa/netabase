#[cfg(all(feature = "libp2p", feature = "native"))]
pub mod record_store;

// Conditional compilation for native vs WASM
#[cfg(all(feature = "wasm", not(feature = "native")))]
pub mod memory;
#[cfg(feature = "native")]
pub mod sled;

// Re-export the appropriate database implementation based on features
#[cfg(all(feature = "native", not(feature = "wasm")))]
pub use sled::{
    NetabaseIter, NetabaseSledDatabase as NetabaseDatabase, NetabaseSledTree as NetabaseTree,
    NetabaseTreeCompatible,
};

#[cfg(all(feature = "wasm", not(feature = "native")))]
pub use memory::{
    NetabaseIter, NetabaseMemoryDatabase as NetabaseDatabase, NetabaseMemoryTree as NetabaseTree,
    NetabaseTreeCompatible,
};

// Re-export libp2p-specific functionality when feature is enabled
#[cfg(all(feature = "libp2p", feature = "native"))]
pub use sled::NetabaseRecordStoreExt;

#[cfg(all(feature = "libp2p", feature = "native"))]
pub use record_store::{
    ProvidedIter, ProvidersListValue, RecordsIter, SledRecordStoreConfig, StoredProviderRecord,
};
