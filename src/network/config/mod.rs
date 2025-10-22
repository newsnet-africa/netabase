use std::time::Duration;

/// Storage backend options for Netabase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBackend {
    /// Sled embedded database (native only, default)
    Sled,
    /// Redb embedded database (native only)
    Redb,
    /// IndexedDB browser storage (WASM only)
    #[cfg(feature = "wasm")]
    IndexedDB,
}

impl Default for StorageBackend {
    fn default() -> Self {
        #[cfg(feature = "native")]
        return StorageBackend::Sled;

        #[cfg(all(feature = "wasm", not(feature = "native")))]
        return StorageBackend::IndexedDB;
    }
}

#[derive(Default, Clone)]
pub struct NetabaseConfig {
    pub dht_discovery: DHTDiscoveryConfig,
    /// Storage backend to use (sled, redb, or indexeddb)
    pub storage_backend: StorageBackend,
}

impl NetabaseConfig {
    /// Create a new config with the specified storage backend
    pub fn with_backend(backend: StorageBackend) -> Self {
        Self {
            storage_backend: backend,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone)]
pub struct DHTDiscoveryConfig {
    pub mdns_discovery: MDNSDiscoveryConfig,
}

#[derive(Default, Clone)]
pub struct MDNSDiscoveryConfig {
    pub auto_connect: Option<Duration>,
}
