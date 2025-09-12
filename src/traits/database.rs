use crate::netabase_trait::{NetabaseRegistery, NetabaseSchema, NetabaseSchemaKey};
use async_trait::async_trait;
use macro_exports::__netabase_libp2p_kad::{Record, RecordKey as KadKey};
use std::collections::HashMap;

/// Result type for database operations
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Storage backend types available for the database
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StorageBackend {
    /// Use libp2p MemoryStore (in-memory, kad::Record native)
    Memory,
    /// Use Sled embedded database (persistent, file-based)
    #[default]
    Sled,
    /// Custom storage backend with specified name
    Custom(String),
}

/// Errors that can occur during database operations
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Key not found: {key}")]
    KeyNotFound { key: String },

    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Storage error: {message}")]
    StorageError { message: String },

    #[error("Connection error: {message}")]
    ConnectionError { message: String },

    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },

    #[error("Database is closed")]
    DatabaseClosed,

    #[error("Transaction error: {message}")]
    TransactionError { message: String },
}

impl Clone for DatabaseError {
    fn clone(&self) -> Self {
        match self {
            DatabaseError::KeyNotFound { key } => DatabaseError::KeyNotFound { key: key.clone() },
            DatabaseError::SerializationError { source } => DatabaseError::SerializationError {
                source: format!("{}", source).into(),
            },
            DatabaseError::StorageError { message } => DatabaseError::StorageError {
                message: message.clone(),
            },
            DatabaseError::ConnectionError { message } => DatabaseError::ConnectionError {
                message: message.clone(),
            },
            DatabaseError::InvalidOperation { message } => DatabaseError::InvalidOperation {
                message: message.clone(),
            },
            DatabaseError::DatabaseClosed => DatabaseError::DatabaseClosed,
            DatabaseError::TransactionError { message } => DatabaseError::TransactionError {
                message: message.clone(),
            },
        }
    }
}

/// Configuration for database operations
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_path: String,
    pub storage_backend: StorageBackend,
    pub max_connections: Option<usize>,
    pub connection_timeout: Option<std::time::Duration>,
    pub enable_compression: bool,
    pub cache_size: Option<usize>,
    pub enable_encryption: bool,
    /// Memory store specific options
    pub memory_store_config: MemoryStoreConfig,
    /// Sled specific options
    pub sled_config: SledConfig,
}

/// Configuration options specific to MemoryStore backend
#[derive(Debug, Clone)]
pub struct MemoryStoreConfig {
    /// Maximum number of records to store
    pub max_records: Option<usize>,
    /// Maximum total size in bytes
    pub max_size_bytes: Option<usize>,
    /// Enable automatic expiration cleanup
    pub enable_expiration_cleanup: bool,
    /// Interval for maintenance tasks
    pub maintenance_interval: std::time::Duration,
}

impl Default for MemoryStoreConfig {
    fn default() -> Self {
        Self {
            max_records: None,
            max_size_bytes: None,
            enable_expiration_cleanup: true,
            maintenance_interval: std::time::Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Configuration options specific to Sled backend
#[derive(Debug, Clone)]
pub struct SledConfig {
    /// Enable compression for stored data
    pub use_compression: bool,
    /// Cache size for Sled
    pub cache_capacity: Option<u64>,
    /// Enable periodic compaction
    pub enable_compaction: bool,
    /// Flush every write (durability vs performance trade-off)
    pub flush_every_ms: Option<u64>,
    /// Print profile information on drop
    pub print_profile: bool,
}

impl Default for SledConfig {
    fn default() -> Self {
        Self {
            use_compression: true,
            cache_capacity: Some(1024 * 1024 * 100), // 100MB
            enable_compaction: true,
            flush_every_ms: Some(1000),
            print_profile: false,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_path: "./netabase_data".to_string(),
            storage_backend: StorageBackend::default(),
            max_connections: Some(10),
            connection_timeout: Some(std::time::Duration::from_secs(30)),
            enable_compression: false,
            cache_size: Some(1000),
            enable_encryption: false,
            memory_store_config: MemoryStoreConfig::default(),
            sled_config: SledConfig::default(),
        }
    }
}

impl DatabaseConfig {
    /// Create a new DatabaseConfig builder
    pub fn builder() -> DatabaseConfigBuilder {
        DatabaseConfigBuilder::default()
    }

    /// Set the storage backend
    pub fn with_storage_backend(mut self, backend: StorageBackend) -> Self {
        self.storage_backend = backend;
        self
    }

    /// Set the database path
    pub fn with_path<P: AsRef<str>>(mut self, path: P) -> Self {
        self.database_path = path.as_ref().to_string();
        self
    }

    /// Configure for in-memory storage using MemoryStore
    pub fn in_memory() -> Self {
        Self {
            storage_backend: StorageBackend::Memory,
            database_path: ":memory:".to_string(),
            ..Default::default()
        }
    }

    /// Configure for persistent storage using Sled
    pub fn persistent<P: AsRef<str>>(path: P) -> Self {
        Self {
            storage_backend: StorageBackend::Sled,
            database_path: path.as_ref().to_string(),
            ..Default::default()
        }
    }

    /// Configure memory store options
    pub fn with_memory_config(mut self, config: MemoryStoreConfig) -> Self {
        self.memory_store_config = config;
        self
    }

    /// Configure sled options
    pub fn with_sled_config(mut self, config: SledConfig) -> Self {
        self.sled_config = config;
        self
    }

    /// Enable compression
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.enable_compression = enabled;
        self
    }

    /// Set cache size
    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.cache_size = Some(size);
        self
    }
}

/// Builder for DatabaseConfig
#[derive(Debug, Clone)]
pub struct DatabaseConfigBuilder {
    config: DatabaseConfig,
}

impl Default for DatabaseConfigBuilder {
    fn default() -> Self {
        Self {
            config: DatabaseConfig::default(),
        }
    }
}

impl DatabaseConfigBuilder {
    /// Set the storage backend
    pub fn storage_backend(mut self, backend: StorageBackend) -> Self {
        self.config.storage_backend = backend;
        self
    }

    /// Set the database path
    pub fn path<P: AsRef<str>>(mut self, path: P) -> Self {
        self.config.database_path = path.as_ref().to_string();
        self
    }

    /// Use in-memory storage
    pub fn in_memory(mut self) -> Self {
        self.config.storage_backend = StorageBackend::Memory;
        self.config.database_path = ":memory:".to_string();
        self
    }

    /// Use persistent storage
    pub fn persistent<P: AsRef<str>>(mut self, path: P) -> Self {
        self.config.storage_backend = StorageBackend::Sled;
        self.config.database_path = path.as_ref().to_string();
        self
    }

    /// Configure memory store options
    pub fn memory_config(mut self, config: MemoryStoreConfig) -> Self {
        self.config.memory_store_config = config;
        self
    }

    /// Configure sled options
    pub fn sled_config(mut self, config: SledConfig) -> Self {
        self.config.sled_config = config;
        self
    }

    /// Set max records for memory store
    pub fn max_records(mut self, max: usize) -> Self {
        self.config.memory_store_config.max_records = Some(max);
        self
    }

    /// Set max size for memory store
    pub fn max_size_bytes(mut self, max: usize) -> Self {
        self.config.memory_store_config.max_size_bytes = Some(max);
        self
    }

    /// Enable or disable compression
    pub fn compression(mut self, enabled: bool) -> Self {
        self.config.enable_compression = enabled;
        self.config.sled_config.use_compression = enabled;
        self
    }

    /// Set cache size
    pub fn cache_size(mut self, size: usize) -> Self {
        self.config.cache_size = Some(size);
        self
    }

    /// Set sled cache capacity
    pub fn sled_cache_capacity(mut self, capacity: u64) -> Self {
        self.config.sled_config.cache_capacity = Some(capacity);
        self
    }

    /// Enable encryption
    pub fn encryption(mut self, enabled: bool) -> Self {
        self.config.enable_encryption = enabled;
        self
    }

    /// Build the configuration
    pub fn build(self) -> DatabaseConfig {
        self.config
    }
}

/// Options for database queries
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub include_metadata: bool,
    pub timeout: Option<std::time::Duration>,
}

/// Transaction handle for batch operations
pub struct DatabaseTransaction<'a, V: NetabaseSchema<R>, R: NetabaseRegistery> {
    _phantom: std::marker::PhantomData<(&'a (), V::Key, V)>,
}

/// Core database operations trait
#[async_trait]
pub trait NetabaseDatabase<V: NetabaseSchema<R>, R: NetabaseRegistery>: Send + Sync {
    /// Initialize the database with the given configuration
    async fn initialize(&mut self, config: DatabaseConfig) -> DatabaseResult<()>;

    /// Check if the database is initialized and ready
    fn is_initialized(&self) -> bool;

    /// Close the database and clean up resources
    async fn close(&mut self) -> DatabaseResult<()>;

    /// Store a key-value pair in the database
    async fn put(&mut self, key: V::Key, value: V) -> DatabaseResult<()>;

    /// Store multiple key-value pairs in a single operation
    async fn put_batch(&mut self, entries: Vec<(V::Key, V)>) -> DatabaseResult<()>;

    /// Retrieve a value by its key
    async fn get(&self, key: &V::Key) -> DatabaseResult<Option<V>>;

    /// Retrieve multiple values by their keys
    async fn get_batch(&self, keys: &[V::Key]) -> DatabaseResult<HashMap<V::Key, V>>;

    /// Check if a key exists in the database
    async fn contains_key(&self, key: &V::Key) -> DatabaseResult<bool>;

    /// Remove a key-value pair from the database
    async fn delete(&mut self, key: &V::Key) -> DatabaseResult<bool>;

    /// Remove multiple keys in a single operation
    async fn delete_batch(&mut self, keys: &[V::Key]) -> DatabaseResult<Vec<V::Key>>;

    /// Get all keys in the database
    async fn keys(&self, options: Option<QueryOptions>) -> DatabaseResult<Vec<V::Key>>;

    /// Get all values in the database
    async fn values(&self, options: Option<QueryOptions>) -> DatabaseResult<Vec<V>>;

    /// Get all key-value pairs in the database
    async fn entries(&self, options: Option<QueryOptions>) -> DatabaseResult<Vec<(V::Key, V)>>;

    /// Get the number of entries in the database
    async fn len(&self) -> DatabaseResult<usize>;

    /// Check if the database is empty
    async fn is_empty(&self) -> DatabaseResult<bool>;

    /// Clear all entries from the database
    async fn clear(&mut self) -> DatabaseResult<()>;

    /// Create a transaction for batch operations
    async fn begin_transaction(&mut self) -> DatabaseResult<DatabaseTransaction<'_, V, R>>;

    /// Commit a transaction
    async fn commit_transaction(
        &mut self,
        transaction: DatabaseTransaction<'_, V, R>,
    ) -> DatabaseResult<()>;

    /// Rollback a transaction
    async fn rollback_transaction(
        &mut self,
        transaction: DatabaseTransaction<'_, V, R>,
    ) -> DatabaseResult<()>;

    /// Compact the database to reclaim space
    async fn compact(&mut self) -> DatabaseResult<()>;

    /// Get database statistics
    async fn stats(&self) -> DatabaseResult<DatabaseStats>;

    /// Store a kad::Record directly (for network integration)
    async fn put_record(&mut self, record: Record) -> DatabaseResult<()>;

    /// Retrieve a kad::Record directly (for network integration)
    async fn get_record(&self, key: &V::Key) -> DatabaseResult<Option<Record>>;

    /// Convert a value to a kad::Record using auto-generated TryInto trait
    fn to_record(&self, value: V) -> Result<Record, DatabaseError> {
        value
            .try_into()
            .map_err(|e| DatabaseError::SerializationError {
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to convert value to record",
                )),
            })
    }

    /// Convert a kad::Record back to a value using auto-generated TryFrom trait
    fn from_record(&self, record: Record) -> Result<V, DatabaseError> {
        record
            .try_into()
            .map_err(|e| DatabaseError::SerializationError {
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to convert record to value",
                )),
            })
    }

    /// Get all records that should be republished (for DHT maintenance)
    async fn get_republish_records(&self) -> DatabaseResult<Vec<Record>>;

    /// Mark records as needing republish after network events
    async fn mark_for_republish(&mut self, keys: &[V::Key]) -> DatabaseResult<()>;

    /// Get records that are about to expire in the DHT
    async fn get_expiring_records(
        &self,
        within: std::time::Duration,
    ) -> DatabaseResult<Vec<Record>>;

    /// Update record expiration time based on DHT feedback
    async fn update_record_expiry(
        &mut self,
        key: &V::Key,
        expires: Option<std::time::Instant>,
    ) -> DatabaseResult<()>;

    /// Convert NetabaseSchemaKey to kad::record::Key using auto-generated TryInto trait
    fn schema_key_to_kad_key(&self, key: V::Key) -> Result<KadKey, DatabaseError> {
        key.try_into()
            .map_err(|e| DatabaseError::SerializationError {
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to convert key to record key",
                )),
            })
    }

    /// Convert kad::record::Key to NetabaseSchemaKey using auto-generated TryInto trait
    fn kad_key_to_schema_key(&self, key: KadKey) -> Result<V::Key, DatabaseError> {
        key.try_into()
            .map_err(|e| DatabaseError::SerializationError {
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to convert record key to key",
                )),
            })
    }

    /// Backup the database to a specified path
    async fn backup<P: AsRef<str> + Send>(&self, backup_path: P) -> DatabaseResult<()>;

    /// Restore the database from a backup
    async fn restore<P: AsRef<str> + Send>(&mut self, backup_path: P) -> DatabaseResult<()>;
}

/// Statistics about the database
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_entries: usize,
    pub total_size: usize,
    pub last_compaction: Option<chrono::DateTime<chrono::Utc>>,
    pub cache_hit_rate: f64,
    pub average_entry_size: usize,
    pub total_kad_records: usize,
    pub records_pending_republish: usize,
    pub records_expiring_soon: usize,
}

/// Extension trait for advanced database operations
#[async_trait]
pub trait NetabaseDatabaseExt<V: NetabaseSchema<R>, R: NetabaseRegistery>:
    NetabaseDatabase<V, R>
{
    /// Update a value if the key exists
    async fn update(&mut self, key: &V::Key, value: V) -> DatabaseResult<bool>;

    /// Insert a value only if the key doesn't exist
    async fn insert(&mut self, key: V::Key, value: V) -> DatabaseResult<bool>;

    /// Update a value if it exists, or insert it if it doesn't
    async fn upsert(&mut self, key: V::Key, value: V) -> DatabaseResult<()>;

    /// Get and remove a value in a single operation
    async fn take(&mut self, key: &V::Key) -> DatabaseResult<Option<V>>;

    /// Update a value using a closure
    async fn update_with<F>(&mut self, key: &V::Key, updater: F) -> DatabaseResult<bool>
    where
        F: FnOnce(V) -> V + Send + Sync;

    /// Scan keys with a prefix
    async fn scan_prefix(
        &self,
        prefix: &str,
        options: Option<QueryOptions>,
    ) -> DatabaseResult<Vec<V::Key>>;

    /// Scan keys within a range
    async fn scan_range(
        &self,
        start: &V::Key,
        end: &V::Key,
        options: Option<QueryOptions>,
    ) -> DatabaseResult<Vec<V::Key>>;

    /// Subscribe to changes for a specific key
    async fn subscribe_key(
        &self,
        key: &V::Key,
    ) -> DatabaseResult<tokio::sync::broadcast::Receiver<DatabaseEvent<V, R>>>;

    /// Subscribe to all database changes
    async fn subscribe_all(
        &self,
    ) -> DatabaseResult<tokio::sync::broadcast::Receiver<DatabaseEvent<V, R>>>;

    /// Sync with network - put local records to DHT
    async fn sync_to_network(&mut self) -> DatabaseResult<Vec<Record>>;

    /// Sync from network - get records from DHT and store locally
    async fn sync_from_network(&mut self, records: Vec<Record>) -> DatabaseResult<()>;

    /// Get records by key prefix (useful for DHT queries)
    async fn get_records_by_prefix(&self, prefix: &[u8]) -> DatabaseResult<Vec<Record>>;

    /// Store multiple kad records atomically (for network sync)
    async fn put_records_batch(&mut self, records: Vec<Record>) -> DatabaseResult<()>;

    /// Check if a record is locally newer than a network record
    async fn is_local_newer(&self, network_record: &Record) -> DatabaseResult<bool>;

    /// Handle record conflicts when syncing with network
    async fn resolve_record_conflict(
        &mut self,
        local_record: Record,
        network_record: Record,
    ) -> DatabaseResult<Record>;

    /// Get all records as an iterator (efficient for large datasets)
    async fn records_iter(&self) -> DatabaseResult<Vec<Record>>;

    /// Get records that match a specific publisher
    async fn get_records_by_publisher(
        &self,
        publisher: Option<libp2p::PeerId>,
    ) -> DatabaseResult<Vec<Record>>;

    /// Remove expired records from the store
    async fn remove_expired_records(&mut self) -> DatabaseResult<usize>;

    /// Get the closest records to a given key (useful for DHT operations)
    async fn get_closest_records(&self, key: &V::Key, limit: usize) -> DatabaseResult<Vec<Record>>;

    /// Check if the store has reached its capacity limit
    async fn is_at_capacity(&self) -> DatabaseResult<bool>;

    /// Get store capacity information
    async fn capacity_info(&self) -> DatabaseResult<StoreCapacityInfo>;

    /// Perform store maintenance (cleanup, optimization, etc.)
    async fn maintain_store(&mut self) -> DatabaseResult<StoreMaintenanceResult>;
}

/// Store capacity information
#[derive(Debug, Clone)]
pub struct StoreCapacityInfo {
    pub current_records: usize,
    pub max_records: Option<usize>,
    pub current_size_bytes: usize,
    pub max_size_bytes: Option<usize>,
    pub is_full: bool,
    pub utilization_percentage: f64,
}

/// Store maintenance result
#[derive(Debug, Clone)]
pub struct StoreMaintenanceResult {
    pub records_removed: usize,
    pub bytes_freed: usize,
    pub expired_records_cleaned: usize,
    pub maintenance_duration: std::time::Duration,
}

/// Events that can occur in the database
#[derive(Debug, Clone)]
pub enum DatabaseEvent<V: NetabaseSchema<R>, R: NetabaseRegistery> {
    Inserted {
        key: V::Key,
        value: V,
    },
    Updated {
        key: V::Key,
        old_value: V,
        new_value: V,
    },
    Deleted {
        key: V::Key,
        value: V,
    },
    Cleared,
    Error {
        error: String,
    },
    /// Store maintenance was performed
    MaintenancePerformed {
        result: StoreMaintenanceResult,
    },
    /// Store reached capacity limit
    CapacityLimitReached {
        current_utilization: f64,
    },
    /// Records were expired and removed
    RecordsExpired {
        count: usize,
    },
}

/// Iterator trait for streaming large datasets
pub trait DatabaseIterator<V: NetabaseSchema<R>, R: NetabaseRegistery>: Send + Sync {
    /// Get the next batch of entries
    async fn next_batch(&mut self, batch_size: usize) -> DatabaseResult<Vec<(V::Key, V)>>;

    /// Check if there are more entries available
    fn has_more(&self) -> bool;

    /// Reset the iterator to the beginning
    async fn reset(&mut self) -> DatabaseResult<()>;
}

/// Trait for creating database iterators
#[async_trait]
pub trait NetabaseDatabaseIterator<V: NetabaseSchema<R>, R: NetabaseRegistery>:
    NetabaseDatabase<V, R>
{
    type Iterator: DatabaseIterator<V, R>;

    /// Create an iterator over all entries
    async fn iter(&self) -> DatabaseResult<Self::Iterator>;

    /// Create an iterator over entries with a key prefix
    async fn iter_prefix(&self, prefix: &str) -> DatabaseResult<Self::Iterator>;

    /// Create an iterator over entries in a key range
    async fn iter_range(&self, start: &V::Key, end: &V::Key) -> DatabaseResult<Self::Iterator>;
}
