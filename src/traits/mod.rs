//! Traits for separating concerns in Netabase
//!
//! This module contains trait definitions that separate the database, network,
//! configuration, and core functionality of Netabase into distinct concerns.
//!
//! The trait system is designed to:
//! - Provide clear separation between different subsystems
//! - Enable easier testing through dependency injection
//! - Allow for different implementations of each subsystem
//! - Support extensibility and customization
//!
//! # Main Traits
//!
//! - [`NetabaseCore`] - The main interface that combines all subsystems
//! - [`NetabaseDatabase`] - Database operations and storage management
//! - [`NetabaseNetwork`] - Network operations and peer communication
//! - [`NetabaseConfiguration`] - Configuration management and validation
//!
//! # Usage
//!
//! ```rust
//! use netabase::traits::{NetabaseCore, NetabaseDatabase, NetabaseNetwork};
//!
//! // Use the main interface
//! async fn example<K, V>(mut netabase: impl NetabaseCore<K, V>)
//! where
//!     K: NetabaseSchemaKey,
//!     V: NetabaseSchema,
//! {
//!     // High-level operations
//!     netabase.put(key, value).await?;
//!     let result = netabase.get(&key).await?;
//!
//!     // Access subsystems when needed
//!     let db_stats = netabase.database().stats().await?;
//!     let network_stats = netabase.network().stats().await?;
//! }
//! ```

pub mod configuration;
pub mod core;
pub mod database;
pub mod network;

// Re-export commonly used traits and types for easier access
pub use configuration::{
    ConfigurationBuilder, ConfigurationChange, ConfigurationError, ConfigurationEvent,
    ConfigurationMetadata, ConfigurationOptions, ConfigurationProvider, ConfigurationResult,
    ConfigurationSource, ConfigurationValidator, FileFormat, MergeStrategy, NetabaseConfiguration,
    NetabaseConfigurationExt, ValidationLevel,
};

pub use database::{
    DatabaseConfig, DatabaseError, DatabaseEvent, DatabaseIterator, DatabaseResult, DatabaseStats,
    DatabaseTransaction, NetabaseDatabase, NetabaseDatabaseExt, NetabaseDatabaseIterator,
    QueryOptions, StoreCapacityInfo, StoreMaintenanceResult,
};

pub use network::{
    BootstrapStatus, BroadcastOptions, DhtModeStats, DhtStatus, KademliaDhtMode, MessagePriority,
    NetabaseNetwork, NetabaseNetworkExt, NetworkConfig, NetworkConfigBuilder, NetworkError,
    NetworkEvent, NetworkEventHandler, NetworkHealth, NetworkMessage, NetworkResult, NetworkStats,
    PeerInfo, ProtocolConfig,
};

pub use core::{
    ComponentHealth, DataOperationType, ErrorCounts, ExportFormat, HealthIssue, HealthStatus,
    KeyStats, MemoryUsage, NetabaseConfig, NetabaseCore, NetabaseCoreExt, NetabaseError,
    NetabaseEvent, NetabaseEventHandler, NetabaseResult, NetabaseSnapshot, NetabaseTransaction,
    OperationCounts, PeerActivityType, PerformanceMetrics, SyncEventType, SyncStrategy,
    SystemHealth, SystemState, SystemStats,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::configuration::{
        ConfigurationError, ConfigurationResult, NetabaseConfiguration,
    };
    pub use super::core::{NetabaseCore, NetabaseCoreExt, NetabaseError, NetabaseResult};
    pub use super::database::{DatabaseError, DatabaseResult, NetabaseDatabase};
    pub use super::network::{NetabaseNetwork, NetworkError, NetworkResult};
}
