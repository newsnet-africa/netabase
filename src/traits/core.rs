use crate::netabase_trait::{NetabaseRegistery, NetabaseRegistryKey, NetabaseSchema, NetabaseSchemaKey};
use crate::traits::{
    configuration::{ConfigurationError, NetabaseConfiguration},
    database::{DatabaseError, DatabaseStats, NetabaseDatabase},
    network::{NetabaseNetwork, NetworkError, NetworkStats, PeerInfo},
};
use async_trait::async_trait;
use libp2p::PeerId;
use std::collections::HashMap;
use std::time::Duration;

/// Main result type for all Netabase operations
pub type NetabaseResult<T> = Result<T, NetabaseError>;

/// Comprehensive error type that encompasses all possible Netabase errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum NetabaseError {
    /// Database-related errors
    #[error("Database error: {source}")]
    Database {
        #[from]
        source: DatabaseError,
    },

    /// Network-related errors
    #[error("Network error: {source}")]
    Network {
        #[from]
        source: NetworkError,
    },

    /// Configuration-related errors
    #[error("Configuration error: {source}")]
    Configuration {
        #[from]
        source: ConfigurationError,
    },

    /// System is not initialized
    #[error("System not initialized - call initialize() first")]
    NotInitialized,

    /// System is already initialized
    #[error("System already initialized")]
    AlreadyInitialized,

    /// System is not running
    #[error("System not running - call start() first")]
    NotRunning,

    /// System is already running
    #[error("System already running")]
    AlreadyRunning,

    /// Operation timed out
    #[error("Operation timeout: {operation} after {duration:?}")]
    Timeout {
        operation: String,
        duration: Duration,
    },

    /// Synchronization error between subsystems
    #[error("Synchronization error: {message}")]
    SyncError { message: String },

    /// Resource exhaustion
    #[error("Resource exhausted: {resource} - {message}")]
    ResourceExhausted { resource: String, message: String },

    /// Invalid state transition
    #[error("Invalid state transition from {from} to {to}: {reason}")]
    InvalidStateTransition {
        from: String,
        to: String,
        reason: String,
    },

    /// Dependency error
    #[error("Dependency error: {component} - {message}")]
    DependencyError { component: String, message: String },

    /// Internal consistency error
    #[error("Internal consistency error: {message}")]
    ConsistencyError { message: String },
}

/// System states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemState {
    Uninitialized,
    Initializing,
    Initialized,
    Starting,
    Running,
    Stopping,
    Stopped,
    Error { message: String },
    Shutdown,
}

/// Comprehensive system health information
#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub overall_status: HealthStatus,
    pub database_health: ComponentHealth,
    pub network_health: ComponentHealth,
    pub configuration_health: ComponentHealth,
    pub uptime: Duration,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub issues: Vec<HealthIssue>,
    pub recommendations: Vec<String>,
}

/// Health status levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Health information for individual components
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub metrics: HashMap<String, f64>,
    pub issues: Vec<String>,
}

/// Specific health issues
#[derive(Debug, Clone)]
pub struct HealthIssue {
    pub component: String,
    pub severity: HealthStatus,
    pub message: String,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub suggested_action: Option<String>,
}

/// Comprehensive system statistics
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub state: SystemState,
    pub uptime: Duration,
    pub database_stats: DatabaseStats,
    pub network_stats: NetworkStats,
    pub memory_usage: MemoryUsage,
    pub performance_metrics: PerformanceMetrics,
    pub operation_counts: OperationCounts,
    pub error_counts: ErrorCounts,
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub total_allocated: usize,
    pub database_cache: usize,
    pub network_buffers: usize,
    pub configuration_cache: usize,
    pub other: usize,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub average_put_latency: Duration,
    pub average_get_latency: Duration,
    pub average_network_latency: Duration,
    pub throughput_ops_per_second: f64,
    pub network_throughput_bytes_per_second: f64,
    pub cache_hit_ratio: f64,
}

/// Operation counters
#[derive(Debug, Clone)]
pub struct OperationCounts {
    pub total_puts: u64,
    pub total_gets: u64,
    pub total_deletes: u64,
    pub total_network_messages_sent: u64,
    pub total_network_messages_received: u64,
    pub total_config_reloads: u64,
}

/// Error counters
#[derive(Debug, Clone)]
pub struct ErrorCounts {
    pub database_errors: u64,
    pub network_errors: u64,
    pub configuration_errors: u64,
    pub timeout_errors: u64,
    pub sync_errors: u64,
    pub total_errors: u64,
}

/// Events that can occur in the Netabase system
#[derive(Debug, Clone)]
pub enum NetabaseEvent<K: NetabaseSchemaKey, V: NetabaseSchema> {
    /// System state changed
    StateChanged {
        old_state: SystemState,
        new_state: SystemState,
    },

    /// Data operation completed
    DataOperation {
        operation: DataOperationType<K, V>,
        success: bool,
        duration: Duration,
    },

    /// Peer activity
    PeerActivity {
        peer_id: PeerId,
        activity: PeerActivityType,
    },

    /// Configuration changed
    ConfigurationChanged {
        changes: Vec<String>,
        source: String,
    },

    /// Health status changed
    HealthChanged {
        component: String,
        old_status: HealthStatus,
        new_status: HealthStatus,
    },

    /// Error occurred
    Error {
        error: NetabaseError,
        component: String,
        recoverable: bool,
    },

    /// Performance threshold crossed
    PerformanceAlert {
        metric: String,
        threshold: f64,
        current_value: f64,
        severity: HealthStatus,
    },

    /// Synchronization event
    SyncEvent {
        key: K,
        sync_type: SyncEventType<V>,
        peer_id: Option<PeerId>,
    },
}

/// Types of data operations
#[derive(Debug, Clone)]
pub enum DataOperationType<K: NetabaseSchemaKey, V: NetabaseSchema> {
    Put {
        key: K,
        value: V,
    },
    Get {
        key: K,
    },
    Delete {
        key: K,
    },
    Batch {
        operations: Vec<DataOperationType<K, V>>,
    },
}

/// Types of peer activities
#[derive(Debug, Clone)]
pub enum PeerActivityType {
    Connected,
    Disconnected { reason: Option<String> },
    MessageSent { message_type: String, size: usize },
    MessageReceived { message_type: String, size: usize },
    Error { error: String },
}

/// Types of synchronization events
#[derive(Debug, Clone)]
pub enum SyncEventType<V: NetabaseSchema> {
    LocalUpdate { new_value: V },
    RemoteUpdate { new_value: V, source: PeerId },
    Conflict { local_value: V, remote_value: V },
    Resolution { resolved_value: V },
}

/// Configuration for the entire Netabase system
#[derive(Debug, Clone)]
pub struct NetabaseConfig {
    pub database_config: crate::traits::database::DatabaseConfig,
    pub network_config: crate::traits::network::NetworkConfig,
    pub configuration_options: crate::traits::configuration::ConfigurationOptions,
    pub health_check_interval: Duration,
    pub stats_collection_interval: Duration,
    pub event_buffer_size: usize,
    pub operation_timeout: Duration,
    pub auto_recovery_enabled: bool,
    pub sync_strategy: SyncStrategy,
}

/// Strategies for data synchronization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStrategy {
    /// No automatic synchronization
    None,
    /// Eventually consistent with last-write-wins
    EventualConsistency,
    /// Strong consistency with consensus
    StrongConsistency,
    /// Custom synchronization logic
    Custom { strategy_name: String },
}

impl Default for NetabaseConfig {
    fn default() -> Self {
        Self {
            database_config: Default::default(),
            network_config: Default::default(),
            configuration_options: Default::default(),
            health_check_interval: Duration::from_secs(30),
            stats_collection_interval: Duration::from_secs(60),
            event_buffer_size: 1000,
            operation_timeout: Duration::from_secs(30),
            auto_recovery_enabled: true,
            sync_strategy: SyncStrategy::EventualConsistency,
        }
    }
}

/// Core trait that provides the main Netabase interface
#[async_trait]
pub trait NetabaseCore<K: NetabaseRegistryKey, V: NetabaseRegistery, D, N, C>: Send + Sync
where
    K: NetabaseSchemaKey,
    V: NetabaseSchema,
    D: NetabaseDatabase<K, V>,
    N: NetabaseNetwork<K, V>,
    C: NetabaseConfiguration,
{
    // === Lifecycle Management ===

    /// Initialize the Netabase system with the given configuration
    async fn initialize(&mut self, config: NetabaseConfig) -> NetabaseResult<()>;

    /// Start all subsystems (network, database, etc.)
    async fn start(&mut self) -> NetabaseResult<()>;

    /// Stop all subsystems gracefully
    async fn stop(&mut self) -> NetabaseResult<()>;

    /// Shutdown the system and clean up all resources
    async fn shutdown(self) -> NetabaseResult<()>;

    /// Get the current system state
    fn state(&self) -> SystemState;

    /// Check if the system is initialized
    fn is_initialized(&self) -> bool;

    /// Check if the system is running
    fn is_running(&self) -> bool;

    // === High-Level Data Operations ===

    /// Store a value locally and optionally sync with the network
    async fn put(&mut self, key: K, value: V) -> NetabaseResult<()>;

    /// Store a value with specific sync behavior
    async fn put_with_sync(
        &mut self,
        key: K,
        value: V,
        sync_immediately: bool,
    ) -> NetabaseResult<()>;

    /// Retrieve a value, checking local storage first then network if needed
    async fn get(&self, key: &K) -> NetabaseResult<Option<V>>;

    /// Retrieve a value with timeout
    async fn get_with_timeout(&self, key: &K, timeout: Duration) -> NetabaseResult<Option<V>>;

    /// Delete a value locally and propagate to network
    async fn delete(&mut self, key: &K) -> NetabaseResult<bool>;

    /// Check if a key exists (local first, then network)
    async fn contains_key(&self, key: &K) -> NetabaseResult<bool>;

    /// Get all keys (local only for performance)
    async fn keys(&self) -> NetabaseResult<Vec<K>>;

    /// Get the number of entries in local storage
    async fn len(&self) -> NetabaseResult<usize>;

    /// Check if local storage is empty
    async fn is_empty(&self) -> NetabaseResult<bool>;

    // === Batch Operations ===

    /// Perform multiple operations as a batch
    async fn batch(&mut self, operations: Vec<DataOperationType<K, V>>) -> NetabaseResult<()>;

    /// Store multiple key-value pairs
    async fn put_batch(&mut self, entries: Vec<(K, V)>) -> NetabaseResult<()>;

    /// Retrieve multiple values
    async fn get_batch(&self, keys: &[K]) -> NetabaseResult<HashMap<K, V>>;

    /// Delete multiple keys
    async fn delete_batch(&mut self, keys: &[K]) -> NetabaseResult<Vec<K>>;

    // === Network-Aware Operations ===

    /// Publish a value to the network (gossip)
    async fn publish(&mut self, key: K, value: V) -> NetabaseResult<()>;

    /// Subscribe to updates for a specific key
    async fn subscribe(&mut self, key: &K) -> NetabaseResult<tokio::sync::broadcast::Receiver<V>>;

    /// Unsubscribe from updates for a key
    async fn unsubscribe(&mut self, key: &K) -> NetabaseResult<()>;

    /// Force synchronization of a specific key with the network
    async fn sync_key(&mut self, key: &K) -> NetabaseResult<()>;

    /// Force synchronization of all local data with the network
    async fn sync_all(&mut self) -> NetabaseResult<()>;

    /// Get connected peers
    async fn connected_peers(&self) -> NetabaseResult<Vec<PeerInfo>>;

    /// Connect to a specific peer
    async fn connect_peer(
        &mut self,
        peer_id: PeerId,
        address: libp2p::Multiaddr,
    ) -> NetabaseResult<()>;

    /// Disconnect from a specific peer
    async fn disconnect_peer(&mut self, peer_id: &PeerId) -> NetabaseResult<()>;

    // === Monitoring and Health ===

    /// Perform a comprehensive health check
    async fn health_check(&self) -> NetabaseResult<SystemHealth>;

    /// Get detailed system statistics
    async fn stats(&self) -> NetabaseResult<SystemStats>;

    /// Get performance metrics
    async fn performance_metrics(&self) -> NetabaseResult<PerformanceMetrics>;

    /// Start monitoring and automatic health checks
    async fn start_monitoring(&mut self) -> NetabaseResult<()>;

    /// Stop monitoring
    async fn stop_monitoring(&mut self) -> NetabaseResult<()>;

    // === Event Handling ===

    /// Get a receiver for system events
    fn event_receiver(&self) -> tokio::sync::broadcast::Receiver<NetabaseEvent<K, V>>;

    /// Register an event handler
    async fn register_event_handler(
        &mut self,
        handler: Box<dyn NetabaseEventHandler<K, V>>,
    ) -> NetabaseResult<()>;

    /// Unregister an event handler
    async fn unregister_event_handler(&mut self, handler_id: &str) -> NetabaseResult<bool>;

    // === Access to Subsystems ===

    /// Get read-only access to the database subsystem
    fn database(&self) -> &D;

    /// Get mutable access to the database subsystem
    fn database_mut(&mut self) -> &mut D;

    /// Get read-only access to the network subsystem
    fn network(&self) -> &N;

    /// Get mutable access to the network subsystem
    fn network_mut(&mut self) -> &mut N;

    /// Get read-only access to the configuration subsystem
    fn configuration(&self) -> &C;

    /// Get mutable access to the configuration subsystem
    fn configuration_mut(&mut self) -> &mut C;

    // === Backup and Recovery ===

    /// Create a backup of all data
    async fn backup<P: AsRef<str> + Send>(&self, backup_path: P) -> NetabaseResult<()>;

    /// Restore data from a backup
    async fn restore<P: AsRef<str> + Send>(&mut self, backup_path: P) -> NetabaseResult<()>;

    /// Export data in a specific format
    async fn export(&self, format: ExportFormat) -> NetabaseResult<Vec<u8>>;

    /// Import data from a specific format
    async fn import(&mut self, data: Vec<u8>, format: ExportFormat) -> NetabaseResult<()>;

    // === Advanced Operations ===

    /// Create a transaction for atomic operations
    async fn begin_transaction(&mut self) -> NetabaseResult<Box<dyn NetabaseTransaction<K, V>>>;

    /// Execute a closure with automatic retry on failure
    async fn with_retry<F, T>(&mut self, operation: F, max_retries: usize) -> NetabaseResult<T>
    where
        F: Fn(
                &mut Self,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = NetabaseResult<T>> + Send + '_>,
            > + Send,
        T: Send;

    /// Wait for a specific condition to be met
    async fn wait_for_condition<F>(&self, condition: F, timeout: Duration) -> NetabaseResult<()>
    where
        F: Fn(&Self) -> bool + Send;
}

/// Data export/import formats
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    MessagePack,
    Protobuf,
    Custom { format_name: String },
}

/// Trait for handling system events
#[async_trait]
pub trait NetabaseEventHandler<K: NetabaseSchemaKey, V: NetabaseSchema>: Send + Sync {
    /// Get the unique identifier for this handler
    fn id(&self) -> &str;

    /// Handle a system event
    async fn handle_event(&mut self, event: NetabaseEvent<K, V>) -> NetabaseResult<()>;

    /// Get the event types this handler is interested in
    fn interested_events(&self) -> Vec<String>;

    /// Check if this handler should receive a specific event
    fn should_handle(&self, event: &NetabaseEvent<K, V>) -> bool {
        true // Default: handle all events
    }
}

/// Trait for atomic transactions
#[async_trait]
pub trait NetabaseTransaction<K: NetabaseSchemaKey, V: NetabaseSchema>: Send + Sync {
    /// Add a put operation to the transaction
    fn put(&mut self, key: K, value: V) -> NetabaseResult<()>;

    /// Add a delete operation to the transaction
    fn delete(&mut self, key: &K) -> NetabaseResult<()>;

    /// Commit all operations in the transaction
    async fn commit(self: Box<Self>) -> NetabaseResult<()>;

    /// Rollback the transaction (cancel all operations)
    async fn rollback(self: Box<Self>) -> NetabaseResult<()>;

    /// Check if the transaction is still valid
    fn is_valid(&self) -> bool;

    /// Get the operations in this transaction
    fn operations(&self) -> &[DataOperationType<K, V>];
}

/// Extension trait for advanced Netabase operations
#[async_trait]
pub trait NetabaseCoreExt<K: NetabaseRegistryKey, V: NetabaseRegistery, D, N, C>: NetabaseCore<K, V, D, N, C>
where
    K: NetabaseSchemaKey,
    V: NetabaseSchema,
    D: NetabaseDatabase<K, V>,
    N: NetabaseNetwork<K, V>,
    C: NetabaseConfiguration,
{
    /// Update a value using a closure
    async fn update<F>(&mut self, key: &K, updater: F) -> NetabaseResult<bool>
    where
        F: FnOnce(V) -> V + Send + Sync;

    /// Insert a value only if the key doesn't exist
    async fn insert(&mut self, key: K, value: V) -> NetabaseResult<bool>;

    /// Update if exists, insert if not
    async fn upsert(&mut self, key: K, value: V) -> NetabaseResult<()>;

    /// Get and remove a value atomically
    async fn take(&mut self, key: &K) -> NetabaseResult<Option<V>>;

    /// Compare and swap operation
    async fn compare_and_swap(
        &mut self,
        key: &K,
        expected: Option<V>,
        new_value: V,
    ) -> NetabaseResult<bool>;

    /// Watch for changes to a key
    async fn watch(&mut self, key: &K) -> NetabaseResult<tokio::sync::broadcast::Receiver<V>>;

    /// Scan keys with a prefix
    async fn scan_prefix(&self, prefix: &str) -> NetabaseResult<Vec<K>>;

    /// Get statistics for a specific key
    async fn key_stats(&self, key: &K) -> NetabaseResult<KeyStats>;

    /// Optimize storage (compact, defragment, etc.)
    async fn optimize(&mut self) -> NetabaseResult<()>;

    /// Create a consistent snapshot
    async fn snapshot(&self) -> NetabaseResult<Box<dyn NetabaseSnapshot<K, V>>>;
}

/// Statistics for a specific key
#[derive(Debug, Clone)]
pub struct KeyStats {
    pub key_size: usize,
    pub value_size: usize,
    pub access_count: u64,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub replication_count: usize,
    pub conflicts: u64,
}

/// Trait for database snapshots
pub trait NetabaseSnapshot<K: NetabaseSchemaKey, V: NetabaseSchema>: Send + Sync {
    /// Get a value from the snapshot
    fn get(&self, key: &K) -> NetabaseResult<Option<V>>;

    /// Get all keys in the snapshot
    fn keys(&self) -> NetabaseResult<Vec<K>>;

    /// Get the timestamp when this snapshot was created
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;

    /// Check if the snapshot is still valid
    fn is_valid(&self) -> bool;
}
