use std::time::Duration;

use libp2p::PeerId;

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

#[derive(Default, Clone, Debug)]
pub struct NetabaseConfig {
    pub node: NodeConfig,
    pub dht_discovery: DHTDiscoveryConfig,
    /// Storage backend to use (sled, redb, or indexeddb)
    pub storage_backend: StorageBackend,
    /// Synchronization configuration
    pub sync: Option<SyncConfig>,
    /// Paxos consensus configuration (only used when paxos feature is enabled)
    #[cfg(feature = "paxos")]
    pub paxos: PaxosConfig,
}

#[derive(Clone, Debug)]
pub struct NodeConfig {
    pub peer_id: PeerId,
    pub database_path: Option<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            peer_id: PeerId::random(),
            database_path: Default::default(),
        }
    }
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
    pub fn with_sync(sync: SyncConfig) -> Self {
        Self {
            sync: Some(sync),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DHTDiscoveryConfig {
    pub mdns_discovery: MDNSDiscoveryConfig,
}

#[derive(Clone, Debug)]
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

/// Paxos consensus configuration
///
/// This configures cluster membership and consensus parameters when the paxos feature is enabled.
///
/// # Examples
///
/// ## Simple 3-node cluster with defaults:
/// ```ignore
/// let config = PaxosConfig {
///     cluster_members: vec![peer_id_1, peer_id_2, peer_id_3],
///     ..Default::default()
/// };
/// ```
///
/// ## Custom timeouts and retry behavior:
/// ```ignore
/// let config = PaxosConfig {
///     cluster_members: vec![peer_id_1, peer_id_2, peer_id_3],
///     operation: PaxosOperationConfig {
///         proposal_timeout: Duration::from_secs(10),
///         max_retries: 5,
///         ..Default::default()
///     },
///     ..Default::default()
/// };
/// ```
#[cfg(feature = "paxos")]
#[derive(Clone, Debug)]
pub struct PaxosConfig {
    /// Static list of cluster member peer IDs
    ///
    /// This defines the fixed set of nodes that participate in consensus.
    /// For dynamic membership, leave this empty and use discovery mechanisms.
    ///
    /// **Default**: Empty vector (single-node mode)
    pub cluster_members: Vec<PeerId>,

    /// Enable dynamic cluster membership via peer discovery
    ///
    /// When true, nodes discovered via mDNS/Identify are automatically added to the cluster.
    /// When false, only statically configured cluster_members participate in consensus.
    ///
    /// **Default**: false (static membership only)
    pub dynamic_membership: bool,

    /// Minimum number of nodes required for quorum
    ///
    /// If None, uses the standard Paxos majority: (cluster_size / 2) + 1
    ///
    /// **Default**: None (use Paxos majority)
    pub min_quorum: Option<usize>,

    /// Operation-level configuration for consensus proposals
    pub operation: PaxosOperationConfig,
}

/// Configuration for individual Paxos consensus operations
///
/// These settings control timeouts, retries, and other operational parameters
/// for consensus proposals. All values have sensible defaults optimized for
/// typical network conditions.
#[cfg(feature = "paxos")]
#[derive(Clone, Debug)]
pub struct PaxosOperationConfig {
    /// Maximum time to wait for a proposal to be accepted by the cluster
    ///
    /// This includes time for:
    /// - Prepare phase (requesting promises from acceptors)
    /// - Accept phase (getting accepts from acceptors)
    /// - Learn phase (confirming the decision)
    ///
    /// **Default**: 30 seconds
    pub proposal_timeout: Duration,

    /// Maximum number of times to retry a failed proposal
    ///
    /// A proposal may fail due to:
    /// - Network timeouts
    /// - Competing proposals (promise conflicts)
    /// - Insufficient quorum
    ///
    /// **Default**: 3 retries
    pub max_retries: usize,

    /// Delay between retry attempts
    ///
    /// This delay helps prevent thundering herd problems when multiple
    /// nodes retry proposals simultaneously.
    ///
    /// **Default**: 1 second
    pub retry_delay: Duration,

    /// Enable automatic retry with exponential backoff
    ///
    /// When true, retry delays increase exponentially: 1s, 2s, 4s, etc.
    /// When false, uses fixed retry_delay for all attempts.
    ///
    /// **Default**: true
    pub exponential_backoff: bool,

    /// Maximum backoff delay when using exponential backoff
    ///
    /// Prevents retry delays from growing too large.
    ///
    /// **Default**: 30 seconds
    pub max_backoff: Duration,

    /// Timeout for individual prepare/accept phase messages
    ///
    /// This is the timeout for a single message round-trip to one node.
    /// The total proposal_timeout should be significantly larger to account
    /// for multiple phases and retries.
    ///
    /// **Default**: 5 seconds
    pub message_timeout: Duration,

    /// Whether to fail fast if the cluster doesn't have quorum
    ///
    /// When true, immediately returns an error if the cluster size is
    /// below the minimum quorum requirement.
    /// When false, attempts the operation anyway (may timeout).
    ///
    /// **Default**: true
    pub fail_fast_no_quorum: bool,
}

#[cfg(feature = "paxos")]
impl Default for PaxosOperationConfig {
    fn default() -> Self {
        Self {
            proposal_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            exponential_backoff: true,
            max_backoff: Duration::from_secs(30),
            message_timeout: Duration::from_secs(5),
            fail_fast_no_quorum: true,
        }
    }
}

#[cfg(feature = "paxos")]
impl Default for PaxosConfig {
    fn default() -> Self {
        Self {
            // Empty cluster = single-node mode (useful for testing)
            cluster_members: Vec::new(),
            // Disabled by default for predictable behavior
            dynamic_membership: false,
            // Use standard Paxos majority by default
            min_quorum: None,
            // Use default operation settings
            operation: PaxosOperationConfig::default(),
        }
    }
}

// ============================================================================
// Sync Configuration
// ============================================================================

/// Comprehensive synchronization configuration for distributed data replication
///
/// Netabase supports multiple sync protocols that can be combined for different
/// consistency and performance characteristics:
///
/// - **Gossip**: Fast eventual consistency with configurable fanout
/// - **BRB (Byzantine Reliable Broadcast)**: Byzantine fault-tolerant broadcast
/// - **Paxos**: Strong consistency via distributed consensus
/// - **Sybil Resistance**: Proof-of-Work based identity verification
#[derive(Clone, Debug)]
pub struct SyncConfig {
    /// Enable automatic synchronization
    pub enabled: bool,

    /// Automatically sync when new peers are discovered
    pub auto_sync: bool,

    /// Interval between automatic sync operations
    pub sync_interval: Duration,

    /// Gossip protocol configuration for eventual consistency
    pub gossip: GossipConfig,

    /// Byzantine Reliable Broadcast configuration
    pub brb: BrbConfig,

    /// Sybil resistance configuration
    pub sybil_resistance: SybilResistanceConfig,

    /// Paxos consensus configuration
    #[cfg(feature = "paxos")]
    pub paxos: PaxosConfig,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_sync: true,
            sync_interval: Duration::from_secs(30),
            gossip: GossipConfig::default(),
            brb: BrbConfig::default(),
            sybil_resistance: SybilResistanceConfig::default(),
            #[cfg(feature = "paxos")]
            paxos: PaxosConfig::default(),
        }
    }
}

impl SyncConfig {
    /// Development preset: fast sync, minimal security
    pub fn development() -> Self {
        Self {
            enabled: true,
            auto_sync: true,
            sync_interval: Duration::from_secs(10),
            gossip: GossipConfig {
                enabled: true,
                interval: Duration::from_secs(5),
                fanout: 3,
            },
            brb: BrbConfig {
                enabled: false,
                total_peers: 3,
                max_faulty: 1,
            },
            sybil_resistance: SybilResistanceConfig {
                enabled: false,
                pow_difficulty: 8,
                challenge_duration: Duration::from_secs(30),
                verification_validity: Duration::from_secs(3600),
            },
            #[cfg(feature = "paxos")]
            paxos: PaxosConfig::default(),
        }
    }

    /// Production preset: balanced security and performance
    pub fn production() -> Self {
        Self {
            enabled: true,
            auto_sync: true,
            sync_interval: Duration::from_secs(30),
            gossip: GossipConfig {
                enabled: true,
                interval: Duration::from_secs(10),
                fanout: 3,
            },
            brb: BrbConfig {
                enabled: true,
                total_peers: 7,
                max_faulty: 2,
            },
            sybil_resistance: SybilResistanceConfig {
                enabled: true,
                pow_difficulty: 16,
                challenge_duration: Duration::from_secs(60),
                verification_validity: Duration::from_secs(3600),
            },
            #[cfg(feature = "paxos")]
            paxos: PaxosConfig::default(),
        }
    }

    /// High security preset: maximum Byzantine fault tolerance
    pub fn high_security() -> Self {
        Self {
            enabled: true,
            auto_sync: true,
            sync_interval: Duration::from_secs(60),
            gossip: GossipConfig {
                enabled: true,
                interval: Duration::from_secs(15),
                fanout: 5,
            },
            brb: BrbConfig {
                enabled: true,
                total_peers: 10,
                max_faulty: 3,
            },
            sybil_resistance: SybilResistanceConfig {
                enabled: true,
                pow_difficulty: 20,
                challenge_duration: Duration::from_secs(120),
                verification_validity: Duration::from_secs(7200),
            },
            #[cfg(feature = "paxos")]
            paxos: PaxosConfig {
                cluster_members: Vec::new(),
                dynamic_membership: true,
                min_quorum: None,
                operation: PaxosOperationConfig::default(),
            },
        }
    }

    /// Low latency preset: optimize for speed
    pub fn low_latency() -> Self {
        Self {
            enabled: true,
            auto_sync: true,
            sync_interval: Duration::from_secs(5),
            gossip: GossipConfig {
                enabled: true,
                interval: Duration::from_secs(2),
                fanout: 5,
            },
            brb: BrbConfig {
                enabled: false,
                total_peers: 5,
                max_faulty: 1,
            },
            sybil_resistance: SybilResistanceConfig {
                enabled: false,
                pow_difficulty: 12,
                challenge_duration: Duration::from_secs(30),
                verification_validity: Duration::from_secs(1800),
            },
            #[cfg(feature = "paxos")]
            paxos: PaxosConfig::default(),
        }
    }
}

/// Gossip protocol configuration for epidemic-style data propagation
///
/// Gossip provides eventual consistency with low latency by having each node
/// periodically share data with a random subset of peers.
#[derive(Clone, Debug)]
pub struct GossipConfig {
    /// Enable gossip protocol
    pub enabled: bool,

    /// Interval between gossip rounds
    pub interval: Duration,

    /// Number of random peers to gossip with each round
    pub fanout: usize,
}

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
///
/// BRB provides Byzantine fault-tolerant broadcast guarantees, ensuring that
/// all correct nodes deliver the same messages even in the presence of
/// malicious nodes.
///
/// The system can tolerate up to `max_faulty` Byzantine nodes out of
/// `total_peers` nodes, requiring `total_peers >= 2 * max_faulty + 1`.
#[derive(Clone, Debug)]
pub struct BrbConfig {
    /// Enable Byzantine Reliable Broadcast
    pub enabled: bool,

    /// Total number of peers in the system
    pub total_peers: usize,

    /// Maximum number of Byzantine (faulty) peers to tolerate
    /// Formula: total_peers >= 2 * max_faulty + 1
    pub max_faulty: usize,
}

impl Default for BrbConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            total_peers: 7,
            max_faulty: 2,
        }
    }
}

impl BrbConfig {
    /// Validate BRB configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.enabled {
            if self.total_peers < 2 * self.max_faulty + 1 {
                return Err(format!(
                    "BRB requires total_peers >= 2 * max_faulty + 1. Got {} peers with {} max_faulty",
                    self.total_peers, self.max_faulty
                ));
            }
        }
        Ok(())
    }

    /// Calculate required quorum size (2f + 1)
    pub fn quorum_size(&self) -> usize {
        2 * self.max_faulty + 1
    }
}

/// Sybil resistance configuration using Proof-of-Work
///
/// Prevents Sybil attacks where an adversary creates many fake identities
/// by requiring peers to solve computational puzzles before joining.
#[derive(Clone, Debug)]
pub struct SybilResistanceConfig {
    /// Enable Sybil resistance
    pub enabled: bool,

    /// Difficulty of the PoW challenge (number of leading zero bits required)
    pub pow_difficulty: u32,

    /// Maximum time allowed to solve the PoW challenge
    pub challenge_duration: Duration,

    /// How long a PoW verification remains valid
    pub verification_validity: Duration,
}

impl Default for SybilResistanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pow_difficulty: 16,
            challenge_duration: Duration::from_secs(60),
            verification_validity: Duration::from_secs(3600),
        }
    }
}
