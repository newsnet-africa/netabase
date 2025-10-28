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
    /// Synchronization configuration
    #[cfg(feature = "native")]
    pub sync: SyncConfig,
}

impl NetabaseConfig {
    /// Create a new config with the specified storage backend
    pub fn with_backend(backend: StorageBackend) -> Self {
        Self {
            storage_backend: backend,
            ..Default::default()
        }
    }

    /// Create a new config with sync enabled
    #[cfg(feature = "native")]
    pub fn with_sync(sync_config: SyncConfig) -> Self {
        Self {
            sync: sync_config,
            ..Default::default()
        }
    }

    /// Create a new config with custom storage backend and sync
    #[cfg(feature = "native")]
    pub fn with_backend_and_sync(backend: StorageBackend, sync_config: SyncConfig) -> Self {
        Self {
            storage_backend: backend,
            sync: sync_config,
            ..Default::default()
        }
    }
}

/// Synchronization configuration
#[cfg(feature = "native")]
#[derive(Clone)]
pub struct SyncConfig {
    /// Enable synchronization
    pub enabled: bool,

    /// Gossip protocol configuration
    pub gossip: GossipConfig,

    /// Byzantine Reliable Broadcast configuration
    pub brb: BrbConfig,

    /// Sybil resistance configuration
    pub sybil_resistance: SybilResistanceConfig,

    /// Paxos consensus configuration
    pub paxos: PaxosConfig,

    /// Auto-sync on record updates
    pub auto_sync: bool,

    /// Sync interval
    pub sync_interval: Duration,
}

#[cfg(feature = "native")]
impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            gossip: GossipConfig::default(),
            brb: BrbConfig::default(),
            sybil_resistance: SybilResistanceConfig::default(),
            paxos: PaxosConfig::default(),
            auto_sync: true,
            sync_interval: Duration::from_secs(30),
        }
    }
}

/// Gossip protocol configuration
#[cfg(feature = "native")]
#[derive(Clone)]
pub struct GossipConfig {
    /// Enable gossip protocol
    pub enabled: bool,
    /// Gossip interval
    pub interval: Duration,
    /// Number of peers to gossip with per round
    pub fanout: usize,
}

#[cfg(feature = "native")]
impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(10),
            fanout: 3,
        }
    }
}

/// Byzantine Reliable Broadcast configuration
#[cfg(feature = "native")]
#[derive(Clone)]
pub struct BrbConfig {
    /// Enable BRB
    pub enabled: bool,
    /// Total number of peers
    pub total_peers: usize,
    /// Maximum Byzantine faults to tolerate
    pub max_faulty: usize,
}

#[cfg(feature = "native")]
impl Default for BrbConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            total_peers: 7,
            max_faulty: 2,
        }
    }
}

/// Sybil resistance configuration
#[cfg(feature = "native")]
#[derive(Clone)]
pub struct SybilResistanceConfig {
    /// Enable Sybil resistance
    pub enabled: bool,
    /// Proof-of-Work difficulty
    pub pow_difficulty: u32,
    /// Challenge duration
    pub challenge_duration: Duration,
    /// Verification validity duration
    pub verification_duration: Duration,
    /// Enable reputation system
    pub reputation_enabled: bool,
}

#[cfg(feature = "native")]
impl Default for SybilResistanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pow_difficulty: 16,
            challenge_duration: Duration::from_secs(60),
            verification_duration: Duration::from_secs(3600),
            reputation_enabled: true,
        }
    }
}

/// Paxos consensus configuration
#[cfg(feature = "native")]
#[derive(Clone)]
pub struct PaxosConfig {
    /// Enable Paxos consensus
    pub enabled: bool,
    /// Number of acceptors
    pub num_acceptors: usize,
    /// Maximum failures to tolerate
    pub max_failures: usize,
}

#[cfg(feature = "native")]
impl Default for PaxosConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Off by default, use for critical operations only
            num_acceptors: 5,
            max_failures: 2,
        }
    }
}

#[derive(Default, Clone)]
pub struct DHTDiscoveryConfig {
    pub mdns_discovery: MDNSDiscoveryConfig,
}

#[derive(Clone)]
pub struct MDNSDiscoveryConfig {
    pub auto_connect: Option<Duration>,
}

impl Default for MDNSDiscoveryConfig {
    fn default() -> Self {
        Self {
            auto_connect: Some(Duration::from_secs(120)),
        }
    }
}
