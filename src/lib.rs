#![feature(impl_trait_in_assoc_type)]

//! # Netabase - Distributed Database System
//!
//! Netabase is a distributed, peer-to-peer database system built on top of [sled](https://github.com/spacejam/sled)
//! with [libp2p](https://libp2p.io/) integration. It provides a type-safe, macro-driven approach to
//! defining database definitions and models with support for primary keys, secondary keys, and relational queries.
//!
//! This crate is an attempt to provide a persistent Object mapped store for use in the libp2p implementation of the kademlia protocol
//!
//! ## Key Features
//!
//! - **Type-Safe Models**: Automatic code generation for database models using derive macros
//! - **Primary & Secondary Keys**: Efficient indexing and querying capabilities
//! - **Distributed Architecture**: Peer-to-peer networking with DHT-based record storage
//! - **Network Transparency**: Seamless data synchronization across network nodes
//!
//! ## Quick Start
//!
//! ```rust
//! use netabase::Netabase;
//! use netabase_store::{ netabase_definition_module};
//! use netabase_store::traits::model::NetabaseModelTrait;
//! use netabase_store::{bincode, serde}; // Re-exported for convenience
//!
//! // Define your data models
//! #[netabase_definition_module(BlogDefinition, BlogKeys)]
//! mod blog {
//!     use netabase_store::{NetabaseModel, netabase};
//!
//!     #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
//!     #[netabase(BlogDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub name: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//!
//! use blog::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a distributed database instance
//!     let mut netabase = Netabase::<BlogDefinition>::new()?;
//!     netabase.start_swarm().await?;
//!
//!     // Create and store a user
//!     let user = User {
//!         id: 1,
//!         name: "Alice".to_string(),
//!         email: "alice@example.com".to_string(),
//!     };
//!
//!     // Store in the distributed hash table
//!     netabase.put_record(user).await?;
//!
//!     // Retrieve from the network
//!     let user_key = UserKey::Primary(UserPrimaryKey(1));
//!     let result = netabase.get_record(user_key).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture Overview
//!
//! Netabase consists of three main layers:
//!
//! ### 1. Storage Layer (`netabase_store`)
//! - Embedded database operations using sled
//! - CRUD operations and indexing
//! - Secondary key and relational queries
//! - Local data persistence
//!
//! ### 2. Macro Layer (`netabase_macros`)
//! - Procedural macros for code generation
//! - Type-safe model and schema definitions
//! - Automatic key type generation
//! - Serialization trait implementations
//!
//! ### 3. Network Layer (`netabase`)
//! - Peer-to-peer networking with libp2p
//! - Distributed hash table (DHT) operations
//! - Record replication and discovery
//! - Event broadcasting and subscription
//!
//! ## Local Database Usage
//!
//! For local-only database operations without networking:
//!
//! ```rust
//! use netabase_store::databases::sled_store::SledStore;
//! use netabase_store::traits::model::NetabaseModelTrait;
//! use netabase_store::traits::tree::NetabaseTreeSync;
//! use netabase_store::netabase_definition_module;
//!
//! // Define your data models
//! #[netabase_definition_module(BlogDefinition, BlogKeys)]
//! mod blog {
//!     use netabase_store::{NetabaseModel, netabase};
//!
//!     #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
//!     #[netabase(BlogDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub name: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//!
//! use blog::*;
//!
//! // Create local database
//! let db = SledStore::<BlogDefinition>::temp().unwrap();
//! let user_tree = db.open_tree::<User>();
//!
//! let user = User { id: 1, name: "Some Name".to_string(), email: "some@email.com".to_string() };
//!
//! // Standard CRUD operations
//! user_tree.put(user.clone()).unwrap();
//! let retrieved = user_tree.get(user.primary_key()).unwrap().unwrap();
//! assert_eq!(retrieved.name, "Some Name");
//!
//! // Secondary key queries
//! let users_by_email = user_tree.get_by_secondary_key(
//!     UserSecondaryKeys::Email(EmailSecondaryKey("some@email.com".to_string()))
//! ).unwrap();
//! assert_eq!(users_by_email.len(), 1);
//! ```
//!
//! ## Distributed Network Usage
//!
//! For distributed operations across multiple nodes:
//!
//! ```rust
//! use netabase::Netabase;
//! use netabase_store::*;
//! use netabase_store::traits::model::NetabaseModelTrait;
//!
//! /// Example definition module for testing netabase functionality
//! #[netabase_definition_module(TestDefinition, TestDefinitionKeys)]
//! pub mod test_definition {
//!     use super::*;
//!     use netabase_store::{NetabaseModel, netabase};
//!
//!     /// Test user model
//!     #[derive(
//!         NetabaseModel,
//!         Clone,
//!         Debug,
//!         PartialEq,
//!         bincode::Encode,
//!         bincode::Decode,
//!         serde::Serialize,
//!         serde::Deserialize,
//!     )]
//!     #[netabase(TestDefinition)]
//!     pub struct TestUser {
//!         #[primary_key]
//!         pub id: u64,
//!         pub name: String,
//!     }
//! }
//! pub use test_definition::*;
//! #[tokio::main]
//! pub async fn main() {
//!     // Create distributed instance
//!     let mut netabase = Netabase::<test_definition::TestDefinition>::new().expect("Failed to initialise database for some reason");
//!     let user = test_definition::TestUser { id:1, name:"Some Name".to_string() };
//!     let key = user.primary_key();
//!     let start_swarm_result = netabase.start_swarm().await;
//!
//!     // Network operations
//!     let bootstrap_result = netabase.bootstrap().await; // Join the network
//!     let put_record_result = netabase.put_record(user).await; // Store data
//!     let get_record_result = netabase.get_record::<TestUserKey>(key.clone().into()).await; // Retrieve data
//!
//!     // Provider operations
//!     let provider_result = netabase.start_providing::<TestUserKey>(key.clone().into()).await;
//!     let providers_result = netabase.get_providers::<TestUserKey>(key.into()).await;
//!
//!     // Subscribe to network events
//!     let mut receiver = netabase.subscribe_to_broadcasts();
//!     use tokio::time::{timeout, Duration};
//!     
//!     let duration = Duration::from_secs(5);
//!     
//!     loop {
//!         match timeout(duration, receiver.recv()).await {
//!             Ok(Ok(event)) => {
//!                 println!("Network event: {:?}", event);
//!             }
//!             Ok(Err(_closed)) => {
//!                 // channel closed
//!                 break;
//!             }
//!             Err(_) => {
//!                 // timeout elapsed
//!                 println!("No event received for {}s — timing out", duration.as_secs());
//!                 break; // or continue, or handle timeout
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Data Modeling Best Practices
//!
//! ### Primary Keys
//! - Use simple, immutable types (u64, String, UUID)
//! - Ensure uniqueness across all records
//! - Consider using auto-incrementing integers or UUIDs
//!
//! ### Secondary Keys
//! - Index frequently queried fields
//! - Balance query performance vs. storage/write overhead
//! - Use for fields with reasonable cardinality
//!
//! ### TODO
//! #### Relations
//! [ ] Use foreign key fields to reference other models
//! [ ] Enable efficient joins and referential integrity
//! [ ] Consider using `NetabaseRelationalQuery` for complex relationships
///
/// ## Performance Considerations
///
/// - **Secondary Key Queries**: O(m) where m is matching records
/// - **Primary Key Access**: O(log n) where n is total records
/// - **Range Queries**: O(log n + m) for efficient prefix searches
/// - **Batch Operations**: Much faster than individual operations
/// - **Network Operations**: May timeout in single-node setups
///
/// ## Testing
///
/// Use unique database paths for tests to avoid conflicts:
///
/// ```rust
/// use tempfile::TempDir;
///
/// #[cfg(test)]
/// mod tests {
///     #[test]
///     fn test_database_operations() {
///         let temp_dir = TempDir::new().unwrap();
///         let db_path = temp_dir.path().join("test_db");
///         let db = NetabaseSledDatabase::new_with_name(&db_path.to_string_lossy()).unwrap();
///         // Test operations...
///     }
/// }
/// ```
///
/// This crate re-exports the core functionality from `netabase_store` for convenience.
///
// ## Macro Hygiene
//
// All macros in this crate are hygienic - they use absolute paths to reference all internal
// dependencies like `serde`, `bincode`, `strum`, etc. Users need to add these as derives
// to their structs but can import them conveniently through the re-exports provided.
//
// ### Required Derives
//
// When using `#[derive(NetabaseModel)]`, you must also include:
// - `bincode::Encode` and `bincode::Decode` for serialization
// - `serde::Serialize` and `serde::Deserialize` for JSON support
// - Standard derives like `Clone`, `Debug` as needed
//
// ### Convenience Re-exports
//
// All necessary dependencies are re-exported for easy access:
// ```rust
// use netabase::{bincode, serde, strum, derive_more, sled};
// ```
pub use netabase_store;
/// Re-export macro dependencies for user convenience.
/// Users can access these through `netabase::serde`, `netabase::bincode`, etc.
/// but the macros will work even without manual imports thanks to hygiene.
// Re-export macro dependencies conditionally when macros are used
pub mod errors;
pub mod routing;

#[cfg(feature = "native")]
pub mod network;

/// Synchronization protocols (gossip, BRB, PoW)
#[cfg(feature = "native")]
pub mod sync;

#[cfg(feature = "native")]
pub use network::behaviour::NetabaseBehaviourEvent;
#[cfg(feature = "native")]
pub use network::behaviour::clone_impl::NetabaseSwarmEvent;

#[cfg(feature = "native")]
use libp2p::{PeerId, kad::QueryResult};
use netabase_store::traits::{
    definition::NetabaseDefinitionTrait,
    model::{NetabaseModelTrait, NetabaseModelTraitKey},
};
#[cfg(feature = "native")]
use tokio::sync::{broadcast, mpsc, oneshot};

#[cfg(feature = "native")]
use crate::network::config::NetabaseConfig;

/// Main Netabase instance that manages the distributed database.
///
/// This is the primary interface for interacting with a Netabase distributed database.
/// It manages both local storage (via sled) and peer-to-peer networking (via libp2p),
/// providing a unified API for distributed data operations.
#[cfg(feature = "native")]
///
/// ## Architecture
///
/// The `Netabase` instance coordinates several components:
/// - **Local Database**: Embedded sled database for local storage
/// - **P2P Swarm**: libp2p swarm for network communication
/// - **DHT**: Kademlia DHT for distributed record storage and discovery
/// - **Event System**: Broadcast channels for network event notifications
///
/// ## Lifecycle
///
/// 1. **Creation**: Use `new()` or `new_with_path()` to create an instance
/// 2. **Startup**: Call `start_swarm()` to begin network operations
/// 3. **Operations**: Use `put_record()`, `get_record()`, etc. for data operations
/// 4. **Shutdown**: Call `stop_swarm()` or let the instance drop for cleanup
///
/// ## Thread Safety
///
/// The `Netabase` instance is thread-safe and can be shared across async tasks.
/// Network operations are handled by a background task that communicates via
/// message passing channels.
///
/// ## Example
///
/// ```rust,no_run
/// use netabase::Netabase;
/// use netabase_store::netabase_definition_module;
/// use netabase_store::{NetabaseModel, netabase};
/// use netabase_store::traits::model::NetabaseModelTrait;
///
/// #[netabase_definition_module(MyDefinition, MyKeys)]
/// mod definition {
///     use netabase_store::{NetabaseModel, netabase};
///
///     #[derive(NetabaseModel, Clone, Debug)]
///     #[derive(bincode::Encode, bincode::Decode)]
///     #[derive(serde::Serialize, serde::Deserialize)]
///     #[netabase(MyDefinition)]
///     pub struct User {
///         #[primary_key] pub id: u64,
///         pub name: String,
///     }
/// }
///
/// use definition::*;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create and start the distributed database
/// let mut netabase = Netabase::<MyDefinition>::new().unwrap();
/// netabase.start_swarm().await.unwrap();
///
/// // Store data in the distributed network
/// let user = User { id: 1, name: "Alice".to_string() };
/// netabase.put_record(user).await.unwrap();
///
/// // Retrieve data from the network
/// let key = UserKey::Primary(UserPrimaryKey(1));
/// let result = netabase.get_record(key).await.unwrap();
///
/// netabase.stop_swarm().await.unwrap();
/// # Ok(())
/// # }
/// ```

#[derive(Debug)]
pub struct Netabase<D: NetabaseDefinitionTrait + Send + Sync>
where
    D: netabase_store::convert::ToIVec + serde::Serialize + for<'de> serde::Deserialize<'de>,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    config: NetabaseConfig,
    /// Handle to the background swarm task
    swarm_thread: Option<tokio::task::JoinHandle<anyhow::Result<()>>>,
    /// Channel for sending commands to the swarm
    command_sender: mpsc::Sender<network::swarm::handlers::command_events::Command<D>>,
    /// Channel for receiving broadcast events from the network
    broadcast_receiver: broadcast::Receiver<network::behaviour::clone_impl::NetabaseSwarmEvent<D>>,
    /// Optional custom database path
    database_path: Option<String>,
    peer_id: NetabaseNodeInfo,
}

#[derive(Debug, Clone)]
pub struct NetabaseNodeInfo(PeerId);

// /// WASM-compatible Netabase instance for local database operations only.
// ///
// /// This version provides only local database functionality without networking,
// /// suitable for WebAssembly environments where networking capabilities are limited.
// #[cfg(all(feature = "wasm", not(feature = "native")))]
// pub struct Netabase<D: NetabaseDefinitionTrait + Send + Sync> {
//     /// Local database instance
//     database: database::NetabaseDatabase<D>,
//     /// Optional custom database name for WASM storage
//     database_name: Option<String>,
// }

#[cfg(feature = "native")]
impl<D: NetabaseDefinitionTrait + Send + Sync + 'static> Netabase<D>
where
    D: netabase_store::traits::convert::ToIVec + serde::Serialize + for<'de> serde::Deserialize<'de>,
    D::Keys: netabase_store::traits::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    /// Create a new Netabase instance with default settings.
    ///
    /// This creates a new distributed database instance with:
    /// - Default database path (system-dependent)
    /// - Uninitialized network swarm (call `start_swarm()` to activate)
    /// - Fresh command and broadcast channels
    ///
    /// # Returns
    ///
    /// A new `Netabase` instance ready for network startup.
    ///
    /// # Errors
    ///
    /// Currently this method doesn't fail, but returns `Result` for future compatibility.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use netabase::Netabase;
    /// use netabase_store::netabase_definition_module;
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_definition {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
    ///     #[derive(NetabaseModel, Clone, Debug)]
    ///     #[derive(bincode::Encode, bincode::Decode)]
    ///     #[derive(serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key] pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// use my_definition::*;
    ///
    /// let netabase = Netabase::<MyDefinition>::new().unwrap();
    /// ```
    pub fn new() -> anyhow::Result<Self> {
        let (command_sender, _command_receiver) = mpsc::channel(100);
        let (_broadcast_sender, broadcast_receiver) = broadcast::channel(1000);

        Ok(Self {
            config: NetabaseConfig::default(),
            swarm_thread: None,
            command_sender,
            broadcast_receiver,
            database_path: None,
            peer_id: NetabaseNodeInfo(PeerId::random()),
        })
    }

    /// Create a new Netabase instance with a custom configuration.
    ///
    /// This allows you to specify the storage backend and other configuration options.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the Netabase instance
    ///
    /// # Returns
    ///
    /// A new `Netabase` instance configured with the specified settings.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use netabase::Netabase;
    /// use netabase::network::config::{NetabaseConfig, StorageBackend};
    /// use netabase_store::netabase_definition_module;
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_definition {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
    ///     #[derive(NetabaseModel, Clone, Debug)]
    ///     #[derive(bincode::Encode, bincode::Decode)]
    ///     #[derive(serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key] pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// use my_definition::*;
    ///
    /// // Use redb backend instead of default sled
    /// let config = NetabaseConfig::with_backend(StorageBackend::Redb);
    /// let netabase = Netabase::<MyDefinition>::new_with_config(config).unwrap();
    /// ```
    pub fn new_with_config(config: NetabaseConfig) -> anyhow::Result<Self> {
        let (command_sender, _command_receiver) = mpsc::channel(100);
        let (_broadcast_sender, broadcast_receiver) = broadcast::channel(1000);
        let peer_id = config.node.peer_id;
        Ok(Self {
            swarm_thread: None,
            config,
            command_sender,
            broadcast_receiver,
            database_path: None,
            peer_id: NetabaseNodeInfo(peer_id),
        })
    }

    /// Create a new Netabase instance with a custom database path.
    ///
    /// This allows you to specify where the local database will be stored,
    /// which is useful for:
    /// - Custom data directories
    /// - Testing with isolated databases
    /// - Multiple database instances
    /// - Docker volume mounting
    ///
    /// Uses the default storage backend (sled on native, IndexedDB on WASM).
    ///
    /// # Arguments
    ///
    /// * `path` - The filesystem path where the database should be stored
    ///
    /// # Returns
    ///
    /// A new `Netabase` instance configured to use the specified database path.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use netabase::Netabase;
    /// use std::path::Path;
    /// use netabase_store::netabase_definition_module;
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_definition {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
    ///     #[derive(NetabaseModel, Clone, Debug)]
    ///     #[derive(bincode::Encode, bincode::Decode)]
    ///     #[derive(serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key] pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// use my_definition::*;
    ///
    /// // Use a custom database path
    /// let netabase = Netabase::<MyDefinition>::new_with_path("./my_app_data").unwrap();
    ///
    /// // For testing with temporary directories
    /// let temp_dir = tempfile::TempDir::new().unwrap();
    /// let netabase = Netabase::<MyDefinition>::new_with_path(temp_dir.path()).unwrap();
    /// ```
    pub fn new_with_path<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let (command_sender, _command_receiver) = mpsc::channel(100);
        let (_broadcast_sender, broadcast_receiver) = broadcast::channel(1000);
        let config = NetabaseConfig::default();
        let peer_id = config.node.peer_id;
        Ok(Self {
            swarm_thread: None,
            config,
            command_sender,
            broadcast_receiver,
            database_path: Some(path.as_ref().to_string_lossy().to_string()),
            peer_id: NetabaseNodeInfo(peer_id),
        })
    }

    /// Create a new Netabase instance with both a custom path and backend.
    ///
    /// This combines the functionality of `new_with_path` and `new_with_config`,
    /// allowing you to specify both the storage location and the backend type.
    ///
    /// # Arguments
    ///
    /// * `path` - The filesystem path where the database should be stored
    /// * `backend` - The storage backend to use (sled, redb, or indexeddb)
    ///
    /// # Returns
    ///
    /// A new `Netabase` instance configured with the specified path and backend.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use netabase::Netabase;
    /// use netabase::network::config::StorageBackend;
    /// use netabase_store::netabase_definition_module;
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_definition {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
    ///     #[derive(NetabaseModel, Clone, Debug)]
    ///     #[derive(bincode::Encode, bincode::Decode)]
    ///     #[derive(serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key] pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// use my_definition::*;
    ///
    /// // Use redb backend with custom path
    /// let netabase = Netabase::<MyDefinition>::new_with_path_and_backend(
    ///     "./my_app_data",
    ///     StorageBackend::Redb
    /// ).unwrap();
    /// ```
    pub fn new_with_path_and_backend<P: AsRef<std::path::Path>>(
        path: P,
        backend: network::config::StorageBackend,
    ) -> anyhow::Result<Self> {
        let (command_sender, _command_receiver) = mpsc::channel(100);
        let (_broadcast_sender, broadcast_receiver) = broadcast::channel(1000);

        let config = NetabaseConfig::with_backend(backend);

        let peer_id = config.node.peer_id;
        Ok(Self {
            swarm_thread: None,
            config,
            command_sender,
            broadcast_receiver,
            database_path: Some(path.as_ref().to_string_lossy().to_string()),
            peer_id: NetabaseNodeInfo(peer_id),
        })
    }

    /// Create a new Netabase instance with a custom path and full configuration.
    ///
    /// This method allows you to specify both the database path and complete
    /// network/sync configuration, providing maximum control over the instance.
    ///
    /// # Arguments
    ///
    /// * `path` - Custom database storage path
    /// * `config` - Complete NetabaseConfig including sync settings
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use netabase::Netabase;
    /// use netabase::network::config::{NetabaseConfig, SyncConfig};
    /// use netabase_store::netabase_definition_module;
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_definition {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///     #[derive(NetabaseModel, Clone, Debug)]
    ///     #[derive(bincode::Encode, bincode::Decode)]
    ///     #[derive(serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key] pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// use my_definition::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let config = NetabaseConfig {
    ///         sync: SyncConfig::default(),
    ///         ..Default::default()
    ///     };
    ///
    ///     let mut netabase = Netabase::<MyDefinition>::new_with_path_and_config(
    ///         "./my_db",
    ///         config,
    ///     )?;
    ///
    ///     netabase.start_swarm().await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new_with_path_and_config<P: AsRef<std::path::Path>>(
        path: P,
        config: NetabaseConfig,
    ) -> anyhow::Result<Self> {
        let (command_sender, _command_receiver) = mpsc::channel(100);
        let (_broadcast_sender, broadcast_receiver) = broadcast::channel(1000);

        let peer_id = config.node.peer_id;
        Ok(Self {
            swarm_thread: None,
            config,
            command_sender,
            broadcast_receiver,
            database_path: Some(path.as_ref().to_string_lossy().to_string()),
            peer_id: NetabaseNodeInfo(peer_id),
        })
    }

    /// Start the swarm thread to enable network operations.
    ///
    /// This method initializes and starts the background libp2p swarm that handles
    /// all network communication, including:
    /// - DHT participation for record storage and discovery
    /// - Peer discovery and connection management
    /// - Network event processing and broadcasting
    ///
    /// After calling this method, the instance can perform distributed operations
    /// like `put_record()`, `get_record()`, and provider management.
    ///
    /// # State Changes
    ///
    /// - Spawns a background async task for swarm management
    /// - Initializes fresh command and broadcast channels
    /// - Starts listening for network connections
    /// - Begins DHT bootstrap process (if peers are available)
    ///
    /// # Errors
    ///
    /// - Returns error if swarm is already running
    /// - May fail if unable to bind to network ports
    /// - Database path issues (if custom path was specified)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use netabase::Netabase;
    /// use netabase_store::netabase_definition_module;
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_definition {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
    ///     #[derive(NetabaseModel, Clone, Debug)]
    ///     #[derive(bincode::Encode, bincode::Decode)]
    ///     #[derive(serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key] pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// use my_definition::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    ///
    /// // Start network operations
    /// netabase.start_swarm().await.unwrap();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Thread Safety
    ///
    /// The swarm runs in a separate async task, so this method doesn't block.
    /// Multiple concurrent operations can be performed safely after startup.
    pub async fn start_swarm(&mut self) -> anyhow::Result<()> {
        if self.swarm_thread.is_some() {
            return Err(anyhow::anyhow!("Swarm is already running"));
        }

        let (command_sender, command_receiver) = mpsc::channel(100);
        let (broadcast_sender, broadcast_receiver) = broadcast::channel(1000);

        // Update internal channels
        self.command_sender = command_sender;
        self.broadcast_receiver = broadcast_receiver;

        // Generate and start swarm with configured backend
        let swarm = network::swarm::generate_swarm_with_name::<D>(
            self.database_path.clone(),
            self.config.storage_backend,
        )?;

        // Setup swarm with listening addresses (required for mDNS discovery)
        let swarm = network::swarm::setup_swarm(swarm).await?;

        let config = self.config.clone();
        let handle = tokio::spawn(async move {
            network::swarm::handlers::start_swarm_loop(
                config,
                swarm,
                broadcast_sender,
                command_receiver,
            )
            .await;
            Ok(())
        });

        self.swarm_thread = Some(handle);
        Ok(())
    }

    /// Stop the swarm thread and shutdown network operations.
    ///
    /// This method gracefully shuts down the background swarm task and terminates
    /// all network operations. After calling this method, distributed operations
    /// will no longer work until `start_swarm()` is called again.
    ///
    /// # Cleanup Operations
    ///
    /// - Aborts the background swarm task
    /// - Closes network connections
    /// - Stops DHT participation
    /// - Flushes any pending operations
    ///
    /// # Graceful Shutdown
    ///
    /// The method waits for the swarm task to complete or handles cancellation
    /// gracefully. It's safe to call this method multiple times.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use netabase::Netabase;
    /// use netabase_store::netabase_definition_module;
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_definition {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
    ///     #[derive(NetabaseModel, Clone, Debug)]
    ///     #[derive(bincode::Encode, bincode::Decode)]
    ///     #[derive(serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key] pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// use my_definition::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// netabase.start_swarm().await.unwrap();
    ///
    /// // Perform operations...
    ///
    /// // Shutdown gracefully
    /// netabase.stop_swarm().await.unwrap();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Automatic Cleanup
    ///
    /// The swarm is also automatically stopped when the `Netabase` instance
    /// is dropped, so explicit shutdown is optional but recommended for
    /// graceful application termination.
    pub async fn stop_swarm(&mut self) -> anyhow::Result<()> {
        if let Some(handle) = self.swarm_thread.take() {
            handle.abort();
            match handle.await {
                Ok(result) => result?,
                Err(e) if e.is_cancelled() => {
                    // Expected when we abort the task
                }
                Err(e) => return Err(anyhow::anyhow!("Swarm thread error: {}", e)),
            }
        }
        Ok(())
    }

    /// Subscribe to network event broadcasts.
    ///
    /// This method returns a receiver that can be used to monitor network events
    /// such as peer connections, DHT operations, record discoveries, and other
    /// swarm activities. Each subscription creates an independent receiver.
    ///
    /// # Event Types
    ///
    /// The receiver will get events including:
    /// - Peer connection/disconnection events
    /// - DHT query results and updates
    /// - Record put/get operations
    /// - Provider advertisement events
    /// - Network errors and status changes
    ///
    /// # Multiple Subscriptions
    ///
    /// Multiple receivers can be created and will each receive all events
    /// independently. This allows different parts of your application to
    /// monitor network activity simultaneously.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use netabase::Netabase;
    /// use netabase_store::netabase_definition_module;
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_definition {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
    ///     #[derive(NetabaseModel, Clone, Debug)]
    ///     #[derive(bincode::Encode, bincode::Decode)]
    ///     #[derive(serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key] pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// use my_definition::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// let mut receiver = netabase.subscribe_to_broadcasts();
    ///
    /// // Start the swarm first
    /// netabase.start_swarm().await.unwrap();
    ///
    /// // Monitor network events in a background task
    /// tokio::spawn(async move {
    ///     while let Ok(event) = receiver.recv().await {
    ///         println!("Network event: {:?}", event);
    ///     }
    /// });
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance Note
    ///
    /// If events are not consumed fast enough, the receiver may lag behind
    /// and miss events. Use bounded channels and appropriate buffering for
    /// high-throughput scenarios.
    pub fn subscribe_to_broadcasts(
        &self,
    ) -> broadcast::Receiver<network::behaviour::clone_impl::NetabaseSwarmEvent<D>> {
        self.broadcast_receiver.resubscribe()
    }

    /// Store a record in the distributed hash table (DHT).
    ///
    /// This method takes any model that implements `NetabaseModelTrait` and stores it
    /// in the distributed network. The model is automatically wrapped in the
    /// schema enum and serialized for network transmission.
    ///
    /// # Type Safety
    ///
    /// The model type `M` must be convertible to the schema type `S` via the
    /// automatically generated `From` implementations. This ensures type safety
    /// across the network.
    ///
    /// # Network Behavior
    ///
    /// - The record is stored in the local DHT and propagated to nearby peers
    /// - Multiple nodes may store copies for redundancy
    /// - The operation may timeout if no peers are available
    /// - Records have a TTL and may expire if not refreshed
    ///
    /// # Arguments
    ///
    /// * `model` - The model instance to store in the network
    ///
    /// # Returns
    ///
    /// A `QueryResult` containing the outcome of the DHT operation, including
    /// information about which peers stored the record.
    ///
    /// # Errors
    ///
    /// - Network timeout if no peers respond
    /// - Serialization errors if the model can't be encoded
    /// - Swarm not started (must call `start_swarm()` first)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # use netabase_store::traits::model::NetabaseModelTrait;
    /// # use libp2p::kad::QueryResult;
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #         #[secondary_key] pub email: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// # netabase.start_swarm().await.unwrap();
    /// let user = User {
    ///     id: 1,
    ///     name: "Alice".to_string(),
    ///     email: "alice@example.com".to_string(),
    /// };
    ///
    /// match netabase.put_record(user).await {
    ///     Ok(result) => println!("Stored successfully: {:?}", result),
    ///     Err(e) => eprintln!("Storage failed: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance Note
    ///
    /// DHT operations are asynchronous and may take time to complete,
    /// especially during network partitions or with limited peers.
    pub async fn put_record<M: NetabaseModelTrait<D>>(
        &self,
        model: M,
    ) -> anyhow::Result<QueryResult>
    where
        D: From<M>,
        D: netabase_store::convert::ToIVec,
        <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
            + Clone
            + Copy
            + std::fmt::Debug
            + std::fmt::Display
            + PartialEq
            + Eq
            + std::hash::Hash
            + strum::IntoEnumIterator
            + Send
            + Sync
            + 'static
            + std::str::FromStr,
        <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
        <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
        <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
        <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
        <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
        <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
        <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
        <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    {
        let definition = D::from(model);

        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::PutRecord {
                record: definition,
                response_channel: response_tx,
            },
        );

        self.command_sender.send(command).await?;

        match response_rx.await? {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow::anyhow!("Put record failed: {:?}", e)),
        }
    }

    /// Retrieve a record from the distributed hash table (DHT).
    ///
    /// This method searches the distributed network for a record matching the
    /// specified key. It queries multiple peers and returns the first valid
    /// record found, or an error if the record cannot be located.
    ///
    /// # Type Safety
    ///
    /// The key type `K` must be convertible to the schema keys type `S::Keys`
    /// via automatically generated `From` implementations.
    ///
    /// # Network Behavior
    ///
    /// - Queries are sent to multiple peers simultaneously
    /// - Returns as soon as the first valid record is found
    /// - May timeout if no peers have the record or are reachable
    /// - Checks record validity and authenticity
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the record to retrieve from the network
    ///
    /// # Returns
    ///
    /// A `QueryResult` containing either the found record or information
    /// about why the query failed.
    ///
    /// # Errors
    ///
    /// - Record not found in the network
    /// - Network timeout if no peers respond
    /// - Deserialization errors if record data is corrupted
    /// - Swarm not started (must call `start_swarm()` first)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # use netabase_store::traits::model::NetabaseModelTrait;
    /// # use libp2p::kad::QueryResult;
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// # netabase.start_swarm().await.unwrap();
    /// # let user_key = UserKey::Primary(UserPrimaryKey(1));
    /// match netabase.get_record(user_key).await {
    ///     Ok(result) => println!("Query completed: {:?}", result),
    ///     Err(e) => eprintln!("Query failed: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Caching Behavior
    ///
    /// Retrieved records may be cached locally for faster future access.
    /// The cache TTL depends on network configuration.
    pub async fn get_record<K: NetabaseModelTraitKey<D>>(
        &self,
        key: K,
    ) -> anyhow::Result<QueryResult>
    where
        D::Keys: From<K>,
    {
        let definition_key = D::Keys::from(key);

        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::GetRecord {
                key: definition_key,
                response_channel: response_tx,
            },
        );

        self.command_sender.send(command).await?;
        Ok(response_rx.await?)
    }

    /// Find all peers that are providing a specific key in the DHT.
    ///
    /// This method queries the distributed network to discover which peers are
    /// advertising themselves as providers for a given key. Providers are nodes
    /// that claim to have the data associated with a key and can serve it to
    /// other peers.
    ///
    /// # Provider System
    ///
    /// - Nodes automatically become providers when they store records
    /// - Providers can be manually advertised using `start_providing()`
    /// - Provider records have a TTL and expire if not refreshed
    /// - Multiple nodes can provide the same key for redundancy
    ///
    /// # Arguments
    ///
    /// * `key` - The key to find providers for
    ///
    /// # Returns
    ///
    /// A `QueryResult` containing a list of peer IDs that are providing the key.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # use netabase_store::traits::model::NetabaseModelTrait;
    /// # use libp2p::kad::QueryResult;
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// # netabase.start_swarm().await.unwrap();
    /// # let user_key = UserKey::Primary(UserPrimaryKey(1));
    /// match netabase.get_providers(user_key).await {
    ///     Ok(result) => println!("Query completed: {:?}", result),
    ///     Err(e) => eprintln!("Provider query failed: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_providers<K: NetabaseModelTraitKey<D>>(
        &self,
        key: K,
    ) -> anyhow::Result<QueryResult>
    where
        D::Keys: From<K>,
    {
        let definition_key = D::Keys::from(key);

        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::GetProviders {
                key: definition_key,
                response_channel: response_tx,
            },
        );

        self.command_sender.send(command).await?;
        Ok(response_rx.await?)
    }

    /// Advertise this node as a provider for a specific key.
    ///
    /// This method announces to the DHT network that this node can provide
    /// data for the specified key. Other nodes can then discover this node
    /// when searching for providers of that key.
    ///
    /// # Provider Lifecycle
    ///
    /// - Provider records are periodically refreshed automatically
    /// - Records expire if the node goes offline or stops providing
    /// - Multiple nodes can provide the same key simultaneously
    /// - Providing continues until explicitly stopped or node shuts down
    ///
    /// # Use Cases
    ///
    /// - Content distribution and caching
    /// - Service discovery
    /// - Load balancing across multiple data sources
    /// - Redundancy and fault tolerance
    ///
    /// # Arguments
    ///
    /// * `key` - The key to start providing
    ///
    /// # Returns
    ///
    /// A `QueryResult` indicating the success of the provider advertisement.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # use netabase_store::traits::model::NetabaseModelTrait;
    /// # use libp2p::kad::QueryResult;
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// # netabase.start_swarm().await.unwrap();
    /// # let user = User { id: 1, name: "Alice".to_string() };
    /// // Store the record first
    /// netabase.put_record(user).await.unwrap();
    ///
    /// // Advertise as a provider
    /// let user_key = UserKey::Primary(UserPrimaryKey(1));
    /// match netabase.start_providing(user_key).await {
    ///     Ok(_) => {
    ///         println!("Now providing the key");
    ///         // Other nodes can now discover us as a provider
    ///     }
    ///     Err(e) => eprintln!("Failed to start providing: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_providing<K: NetabaseModelTraitKey<D>>(
        &self,
        key: K,
    ) -> anyhow::Result<QueryResult>
    where
        D::Keys: From<K>,
    {
        let definition_key = D::Keys::from(key);

        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::StartProviding {
                key: definition_key,
                response_channel: response_tx,
            },
        );

        self.command_sender.send(command).await?;

        match response_rx.await? {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow::anyhow!("Start providing failed: {:?}", e)),
        }
    }

    /// Stop advertising this node as a provider for a specific key.
    ///
    /// This method removes the provider advertisement for the specified key,
    /// so other nodes will no longer discover this node when searching for
    /// providers of that key.
    ///
    /// # Behavior
    ///
    /// - Immediately stops advertising the key
    /// - Existing provider records will expire naturally
    /// - Does not affect locally stored data
    /// - Other nodes may still have cached provider information temporarily
    ///
    /// # Arguments
    ///
    /// * `key` - The key to stop providing
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # use netabase_store::traits::model::NetabaseModelTrait;
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// # netabase.start_swarm().await.unwrap();
    /// # let user_key = UserKey::Primary(UserPrimaryKey(1));
    /// // Previously started providing this key
    /// netabase.start_providing(user_key.clone()).await.unwrap();
    ///
    /// // Later, stop providing it
    /// netabase.stop_providing(user_key).await.unwrap();
    /// println!("No longer providing this key");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stop_providing<K: NetabaseModelTraitKey<D>>(&self, key: K) -> anyhow::Result<()>
    where
        D::Keys: From<K>,
    {
        let definition_key = D::Keys::from(key);

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::StopProviding {
                key: definition_key,
            },
        );

        self.command_sender.send(command).await?;
        Ok(())
    }

    /// Bootstrap the node to join the DHT network.
    ///
    /// This method initiates the bootstrap process to join the distributed hash table
    /// network. It attempts to connect to known peers and populate the routing table
    /// to enable efficient DHT operations.
    ///
    /// # Bootstrap Process
    ///
    /// 1. Connects to any known bootstrap peers
    /// 2. Performs initial DHT queries to discover more peers
    /// 3. Populates the Kademlia routing table
    /// 4. Begins participating in DHT maintenance
    ///
    /// # When to Bootstrap
    ///
    /// - After starting the swarm for the first time
    /// - When reconnecting after network isolation
    /// - Periodically to refresh the routing table
    /// - When few peers are known and connectivity is poor
    ///
    /// # Network Requirements
    ///
    /// - At least one reachable bootstrap peer must be known
    /// - Network connectivity to discover additional peers
    /// - May fail in isolated or single-node environments
    ///
    /// # Returns
    ///
    /// A `QueryResult` indicating the outcome of the bootstrap process.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # use libp2p::kad::QueryResult;
    /// # use libp2p::{Multiaddr, PeerId};
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// // Start the swarm first
    /// netabase.start_swarm().await.unwrap();
    ///
    /// // Add known bootstrap peers (optional, but recommended)
    /// let bootstrap_addr: Multiaddr = "/ip4/192.168.1.100/tcp/4001".parse().unwrap();
    /// let bootstrap_peer = PeerId::random(); // In practice, use known peer ID
    /// netabase.add_address(bootstrap_peer, bootstrap_addr).await.unwrap();
    ///
    /// // Bootstrap to join the network
    /// match netabase.bootstrap().await {
    ///     Ok(QueryResult::Bootstrap(Ok(result))) => {
    ///         if result.num_remaining == 0 {
    ///             println!("Successfully joined the DHT network");
    ///         } else {
    ///             println!("Bootstrap in progress, {} peers remaining", result.num_remaining);
    ///         }
    ///     }
    ///     Ok(QueryResult::Bootstrap(Err(e))) => eprintln!("Bootstrap failed: {:?}", e),
    ///     Ok(_) => println!("Unexpected result"),
    ///     Err(e) => eprintln!("Bootstrap error: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bootstrap(&self) -> anyhow::Result<QueryResult> {
        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::Bootstrap {
                response_channel: response_tx,
            },
        );

        self.command_sender.send(command).await?;

        match response_rx.await? {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow::anyhow!("Bootstrap failed: {:?}", e)),
        }
    }

    /// Add a known network address for a peer.
    ///
    /// This method adds a peer's network address to the routing table, enabling
    /// direct connections to that peer. This is essential for bootstrapping
    /// and maintaining connectivity in the DHT network.
    ///
    /// # Address Types
    ///
    /// Supports various libp2p multiaddress formats:
    /// - `/ip4/192.168.1.100/tcp/4001` - IPv4 TCP
    /// - `/ip6/::1/tcp/4001` - IPv6 TCP
    /// - `/dns/example.com/tcp/4001` - DNS resolution
    /// - Complex addresses with multiple protocols
    ///
    /// # Use Cases
    ///
    /// - Adding bootstrap peers for network discovery
    /// - Maintaining connections to known reliable peers
    /// - Manual peer management for private networks
    /// - Reconnecting to previously known peers
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The unique identifier of the peer
    /// * `address` - The network address where the peer can be reached
    ///
    /// # Returns
    ///
    /// A `RoutingUpdate` indicating how the routing table was modified.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    /// use netabase::Netabase;
    /// use netabase_store::{ netabase_definition_module};
    /// use netabase_store::traits::model::NetabaseModelTrait;
    /// use netabase_store::traits::definition::NetabaseDefinitionTrait;
    /// use netabase_store::{bincode, serde}; // Re-exported for convenience
    /// use libp2p::{PeerId, Multiaddr};
    ///
    /// // Define your data models
    /// #[netabase_definition_module(BlogDefinition, BlogKeys)]
    /// pub mod blog {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
    ///     #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
    ///     #[netabase(BlogDefinition)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///         #[secondary_key]
    ///         pub email: String,
    ///     }
    /// }
    ///
    /// use blog::*;
    ///
    /// #[tokio::main]
    /// pub async fn main() {
    ///     // Add a bootstrap peer
    ///     let netabase = Netabase::<blog::BlogDefinition>::new().expect("Netabase creation failed for some reason");
    ///     let peer_id = PeerId::random(); // In practice, use a known peer ID
    ///     let address: Multiaddr = "/ip4/192.168.1.100/tcp/4001".parse().unwrap();
    ///
    ///     match netabase.add_address(peer_id, address).await {
    ///         Ok(update) => println!("Routing table updated: {:?}", update),
    ///         Err(e) => eprintln!("Failed to add address: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn add_address(
        &self,
        peer_id: libp2p::PeerId,
        address: libp2p::Multiaddr,
    ) -> anyhow::Result<libp2p::kad::RoutingUpdate> {
        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::AddAddress {
                peer: peer_id,
                address,
                response_channel: response_tx,
            },
        );

        self.command_sender.send(command).await?;
        Ok(response_rx.await?)
    }

    /// Remove a specific network address for a peer.
    ///
    /// This method removes a peer's network address from the routing table.
    /// If the peer has multiple addresses, only the specified address is removed.
    /// If it's the peer's last address, the peer may be removed from the routing table.
    ///
    /// # Behavior
    ///
    /// - Removes only the exact address specified
    /// - Peer remains in routing table if other addresses exist
    /// - May trigger routing table reorganization
    /// - Does not affect active connections using other addresses
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The unique identifier of the peer
    /// * `address` - The specific address to remove
    ///
    /// # Returns
    ///
    /// An `Option<EntryView>` of the peer's routing table entry if it still exists,
    /// or `None` if the peer was completely removed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # use libp2p::{PeerId, Multiaddr};
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// # netabase.start_swarm().await.unwrap();
    /// let peer_id = PeerId::random(); // In practice, use a known peer ID
    /// let old_address: Multiaddr = "/ip4/192.168.1.100/tcp/4001".parse().unwrap();
    ///
    /// match netabase.remove_address(peer_id, old_address).await {
    ///     Ok(Some(entry)) => println!("Address removed, peer still has addresses"),
    ///     Ok(None) => println!("Address removed, peer completely removed from routing table"),
    ///     Err(e) => eprintln!("Failed to remove address: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_address(
        &self,
        peer_id: libp2p::PeerId,
        address: libp2p::Multiaddr,
    ) -> anyhow::Result<
        Option<
            libp2p::kad::EntryView<libp2p::kad::KBucketKey<libp2p::PeerId>, libp2p::kad::Addresses>,
        >,
    > {
        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::RemoveAddress {
                peer: peer_id,
                address,
                response_channel: response_tx,
            },
        );

        self.command_sender.send(command).await?;
        Ok(response_rx.await?)
    }

    /// Remove a peer completely from the routing table.
    ///
    /// This method removes all addresses and routing information for the specified
    /// peer. The peer will no longer be considered for DHT operations until it's
    /// re-added or discovered again through normal network processes.
    ///
    /// # Behavior
    ///
    /// - Removes all addresses for the peer
    /// - Terminates any active connections to the peer
    /// - Removes peer from Kademlia k-buckets
    /// - May trigger routing table rebalancing
    ///
    /// # Use Cases
    ///
    /// - Removing misbehaving or unreliable peers
    /// - Manual network topology management
    /// - Cleaning up stale routing entries
    /// - Enforcing network access policies
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The unique identifier of the peer to remove
    ///
    /// # Returns
    ///
    /// An `Option<EntryView>` containing the removed peer's routing information,
    /// or `None` if the peer wasn't in the routing table.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # use libp2p::PeerId;
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// # netabase.start_swarm().await.unwrap();
    /// let problematic_peer = PeerId::random(); // In practice, use a known peer ID
    ///
    /// match netabase.remove_peer(problematic_peer).await {
    ///     Ok(Some(entry)) => {
    ///         println!("Removed peer from routing table");
    ///     }
    ///     Ok(None) => println!("Peer was not in routing table"),
    ///     Err(e) => eprintln!("Failed to remove peer: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_peer(
        &self,
        peer_id: libp2p::PeerId,
    ) -> anyhow::Result<
        Option<
            libp2p::kad::EntryView<libp2p::kad::KBucketKey<libp2p::PeerId>, libp2p::kad::Addresses>,
        >,
    > {
        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::RemovePeer {
                peer: peer_id,
                response_channel: response_tx,
            },
        );

        self.command_sender.send(command).await?;
        Ok(response_rx.await?)
    }

    /// Get the current DHT operating mode.
    ///
    /// This method returns the current Kademlia DHT mode that determines how
    /// this node participates in the network and what operations it can perform.
    ///
    /// # DHT Modes
    ///
    /// - **`Mode::Client`**: Read-only participation, can query but not store records
    /// - **`Mode::Server`**: Full participation, can both query and store records
    ///
    /// # Mode Implications
    ///
    /// **Client Mode:**
    /// - Lower resource usage
    /// - Cannot store records for other peers
    /// - Cannot participate in DHT maintenance
    /// - Suitable for lightweight applications
    ///
    /// **Server Mode:**
    /// - Higher resource usage
    /// - Stores records for the network
    /// - Participates in DHT maintenance
    /// - Required for network health
    ///
    /// # Returns
    ///
    /// The current `libp2p::kad::Mode` of the DHT.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// # netabase.start_swarm().await.unwrap();
    /// match netabase.get_mode().await {
    ///     Ok(libp2p::kad::Mode::Client) => {
    ///         println!("Operating in client mode");
    ///     }
    ///     Ok(libp2p::kad::Mode::Server) => {
    ///         println!("Operating in server mode");
    ///     }
    ///     Err(e) => eprintln!("Failed to get mode: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_mode(&self) -> anyhow::Result<libp2p::kad::Mode> {
        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::Mode {
                response_channel: response_tx,
            },
        );

        self.command_sender.send(command).await?;
        Ok(response_rx.await?)
    }

    /// Set the DHT operating mode.
    ///
    /// This method changes how this node participates in the Kademlia DHT network.
    /// The mode determines whether the node can store records and participate in
    /// DHT maintenance operations.
    ///
    /// # Mode Options
    ///
    /// - **`Some(Mode::Client)`**: Switch to client mode (read-only)
    /// - **`Some(Mode::Server)`**: Switch to server mode (full participation)
    /// - **`None`**: Use automatic mode selection based on network conditions
    ///
    /// # Performance Impact
    ///
    /// Changing modes can affect:
    /// - Network resource usage
    /// - Storage requirements
    /// - Query performance
    /// - Network responsibilities
    ///
    /// # Arguments
    ///
    /// * `mode` - The new DHT mode, or `None` for automatic selection
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # use libp2p::kad::Mode;
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// # netabase.start_swarm().await.unwrap();
    /// // Switch to server mode for full participation
    /// netabase.set_mode(Some(Mode::Server)).await.unwrap();
    /// println!("Now running in server mode");
    ///
    /// // Switch to client mode for lower resource usage
    /// netabase.set_mode(Some(Mode::Client)).await.unwrap();
    /// println!("Now running in client mode");
    ///
    /// // Let the system choose automatically
    /// netabase.set_mode(None).await.unwrap();
    /// println!("Using automatic mode selection");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_mode(&self, mode: Option<libp2p::kad::Mode>) -> anyhow::Result<()> {
        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::SetMode { mode },
        );

        self.command_sender.send(command).await?;
        Ok(())
    }

    /// Get the protocol names used by the DHT.
    ///
    /// This method returns the libp2p stream protocol identifier used for
    /// Kademlia DHT communication. This is primarily useful for debugging,
    /// monitoring, and ensuring protocol compatibility.
    ///
    /// # Protocol Information
    ///
    /// - Returns the exact protocol string used for DHT communication
    /// - Useful for network analysis and debugging
    /// - Can help identify protocol version incompatibilities
    /// - May be used for custom protocol handling
    ///
    /// # Returns
    ///
    /// A `StreamProtocol` representing the DHT protocol identifier.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// # netabase.start_swarm().await.unwrap();
    /// match netabase.get_protocol_names().await {
    ///     Ok(protocol) => {
    ///         println!("Using protocol: {:?}", protocol);
    ///     }
    ///     Err(e) => eprintln!("Failed to get protocol: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_protocol_names(&self) -> anyhow::Result<libp2p::StreamProtocol> {
        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::ProtocolNames {
                response_channel: response_tx,
            },
        );

        self.command_sender.send(command).await?;
        Ok(response_rx.await?)
    }

    /// Remove a record from local storage.
    ///
    /// This method removes a record from the local DHT storage. Note that this
    /// only affects the local node's storage - other nodes in the network may
    /// still have copies of the record.
    ///
    /// # Behavior
    ///
    /// - Removes record only from local storage
    /// - Does not affect copies on other network nodes
    /// - Stops providing the record if currently being provided
    /// - The record may be re-stored if received from other peers
    ///
    /// # Use Cases
    ///
    /// - Freeing up local storage space
    /// - Removing outdated or invalid data
    /// - Cache management and cleanup
    /// - Privacy and data retention compliance
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the record to remove from local storage
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # use netabase_store::traits::model::NetabaseModelTrait;
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// # netabase.start_swarm().await.unwrap();
    /// # let user_key = UserKey::Primary(UserPrimaryKey(1));
    /// // Remove from local storage
    /// match netabase.remove_record(user_key).await {
    ///     Ok(()) => println!("Record removed from local storage"),
    ///     Err(e) => eprintln!("Failed to remove record: {}", e),
    /// }
    ///
    /// // Note: Other nodes may still have this record
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Network Implications
    ///
    /// Removing a record locally doesn't remove it from the network. To fully
    /// remove data from a distributed system, you would need to coordinate
    /// with other nodes or implement a distributed deletion protocol.
    pub async fn remove_record<K: NetabaseModelTraitKey<D>>(&self, key: K) -> anyhow::Result<()>
    where
        D::Keys: From<K>,
    {
        let definition_key = D::Keys::from(key);

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::RemoveRecord {
                key: definition_key,
            },
        );

        self.command_sender.send(command).await?;
        Ok(())
    }

    /// Query records from the local database store.
    ///
    /// This method retrieves records directly from the local Kademlia store
    /// managed by the swarm thread. It's useful for getting a snapshot of
    /// locally stored data without performing network queries.
    ///
    /// # Arguments
    ///
    /// * `limit` - Optional maximum number of records to retrieve. If `None`, returns all records.
    ///
    /// # Returns
    ///
    /// A `Vec<D>` containing the requested records from local storage.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// #
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// #
    /// # use my_definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new().unwrap();
    /// # netabase.start_swarm().await.unwrap();
    /// // Get all records from local store
    /// match netabase.query_local_records(None).await {
    ///     Ok(records) => {
    ///         println!("Found {} records in local store", records.len());
    ///         for record in records {
    ///             println!("Record: {:?}", record);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Query failed: {}", e),
    /// }
    ///
    /// // Get only the first 10 records
    /// let recent_records = netabase.query_local_records(Some(10)).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// This method only queries the local store. It does not perform network
    /// queries to retrieve records from other peers.
    pub async fn query_local_records(&self, limit: Option<usize>) -> anyhow::Result<Vec<D>> {
        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::LocalStore(
                network::swarm::handlers::command_events::LocalStoreCommand::QueryRecords {
                    limit,
                    response_channel: response_tx,
                },
            ),
        );

        self.command_sender.send(command).await?;

        match response_rx.await? {
            Ok(records) => Ok(records),
            Err(e) => Err(anyhow::anyhow!("Query local records failed: {}", e)),
        }
    }

    /// Store a definition record in the local database and DHT
    ///
    /// This method stores a definition (enum variant) directly without requiring
    /// the NetabaseModelTrait implementation. Useful for simple use cases and examples.
    ///
    /// # Arguments
    ///
    /// * `definition` - The definition enum variant to store
    ///
    /// # Returns
    ///
    /// Returns `Ok(QueryResult)` on success, containing the result of the put operation.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let message = ChatModel::Message {
    ///     id: "msg1".to_string(),
    ///     content: "Hello!".to_string(),
    ///     timestamp: 12345,
    /// };
    ///
    /// netabase.put_definition(message).await?;
    /// ```
    pub async fn put_definition(&self, definition: D) -> anyhow::Result<QueryResult> {
        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::PutRecord {
                record: definition,
                response_channel: response_tx,
            },
        );

        self.command_sender.send(command).await?;

        match response_rx.await? {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow::anyhow!("Put definition failed: {}", e)),
        }
    }

    /// Trigger synchronization with peers
    ///
    /// Initiates a sync operation to exchange data with connected peers.
    /// This uses the configured sync protocols (Gossip, BRB, Paxos) to:
    /// - Query peers for their data digests
    /// - Request missing records
    /// - Share local records with peers
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the sync was initiated successfully. Note that this
    /// does not wait for sync to complete - sync happens asynchronously in the
    /// background.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Manually trigger a sync operation
    /// netabase.trigger_sync().await?;
    ///
    /// // Wait a moment for sync to complete
    /// tokio::time::sleep(Duration::from_secs(2)).await;
    ///
    /// // Query updated local records
    /// let records = netabase.query_local_records(None).await?;
    /// ```
    pub async fn trigger_sync(&self) -> anyhow::Result<()> {
        if let Some(sync_config) = &self.config.sync {
            if !sync_config.enabled {
                return Err(anyhow::anyhow!("Sync is not enabled in configuration"));
            }
        } else {
            return Err(anyhow::anyhow!("Sync configuration not found"));
        }

        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Sync(
            network::swarm::handlers::command_events::SyncCommand::TriggerSync {
                response_channel: response_tx,
            },
        );

        self.command_sender.send(command).await?;

        match response_rx.await? {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Trigger sync failed: {}", e)),
        }
    }

    /// Get the current peer count
    ///
    /// Returns the number of peers currently connected to this node.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let peer_count = netabase.peer_count().await?;
    /// println!("Connected to {} peers", peer_count);
    /// ```
    pub async fn peer_count(&self) -> anyhow::Result<usize> {
        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::GetPeerCount {
            response_channel: response_tx,
        };

        self.command_sender.send(command).await?;

        match response_rx.await {
            Ok(count) => Ok(count),
            Err(e) => Err(anyhow::anyhow!("Get peer count failed: {}", e)),
        }
    }

    /// Get sync configuration
    ///
    /// Returns a reference to the sync configuration if sync is enabled.
    pub fn sync_config(&self) -> Option<&network::config::SyncConfig> {
        self.config.sync.as_ref()
    }

    /// Check if sync is enabled
    pub fn is_sync_enabled(&self) -> bool {
        self.config.sync.as_ref().map(|s| s.enabled).unwrap_or(false)
    }

    // /// Get direct access to the local database.
    // ///
    // /// This provides access to the underlying database for local operations
    // /// without routing through the Kademlia swarm. Useful for:
    // /// - Direct local reads and writes
    // /// - Bypassing network overhead for local operations
    // /// - Database administration and maintenance
    // /// - Testing and debugging
    // ///
    // /// # Returns
    // ///
    // /// A reference to the underlying `NetabaseDatabase` instance.
    // ///
    // /// # Example
    // ///
    // /// ```rust,ignore
    // /// use netabase::Netabase;
    // /// use netabase_store::traits::NetabaseSchemaQuery;
    // /// use netabase_store::netabase_definition_module;
    // ///
    // /// #[netabase_definition_module(MyDefinition, MyKeys)]
    // /// mod my_definition {
    // ///     // Define your models here
    // /// }
    // ///
    // /// let netabase = Netabase::<MyDefinition>::new().unwrap();
    // ///
    // /// // Direct database access
    // /// let db = netabase.database().unwrap();
    // /// let result = db.get_definition(&my_key);
    // /// ```
    // ///
    // /// # Errors
    // ///
    // /// Returns an error if the database cannot be opened with the configured path.
    // pub fn database(&self) -> anyhow::Result<database::NetabaseDatabase<D>> {
    //     match &self.database_path {
    //         Some(path) => database::NetabaseDatabase::<D>::new_with_path(path),
    //         None => database::NetabaseDatabase::<D>::new(),
    //     }
    //     .map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))
    // }

    // /// Get mutable access to a new database instance.
    // ///
    // /// This creates a new database instance with mutable access for operations
    // /// that require writing to the database. Each call creates a new connection,
    // /// so this should be used sparingly for write-heavy operations.
    // ///
    // /// # Returns
    // ///
    // /// A mutable `NetabaseDatabase` instance.
    // ///
    // /// # Example
    // ///
    // /// ```rust,ignore
    // /// use netabase::Netabase;
    // /// use netabase_store::traits::NetabaseSchemaQuery;
    // /// use netabase_store::netabase_definition_module;
    // ///
    // /// #[netabase_definition_module(MyDefinition, MyKeys)]
    // /// mod my_definition {
    // ///     // Define your models here
    // /// }
    // ///
    // /// let netabase = Netabase::<MyDefinition>::new().unwrap();
    // ///
    // /// // Direct mutable database access
    // /// let mut db = netabase.database_mut().unwrap();
    // /// db.put_definition(&my_definition).unwrap();
    // /// ```
    // ///
    // /// # Errors
    // ///
    // /// Returns an error if the database cannot be opened with the configured path.
    // pub fn database_mut(&self) -> anyhow::Result<database::NetabaseDatabase<D>> {
    //     match &self.database_path {
    //         Some(path) => database::NetabaseDatabase::<D>::new_with_path(path),
    //         None => database::NetabaseDatabase::<D>::new(),
    //     }
    //     .map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))
    // }
}

// /// WASM-specific implementation with local database operations only
// #[cfg(all(feature = "wasm", not(feature = "native")))]
// impl<D: NetabaseDefinitionTrait + Send + Sync + 'static> Netabase<D> {
//     /// Create a new WASM Netabase instance for local operations.
//     ///
//     /// This creates a local-only database instance suitable for WASM environments.
//     /// No networking functionality is available.
//     pub fn new() -> anyhow::Result<Self> {
//         let database = database::NetabaseDatabase::<D>::new()?;
//         Ok(Self {
//             database,
//             database_name: None,
//         })
//     }

//     /// Create a new WASM Netabase instance with a custom name.
//     pub fn new_with_name(name: String) -> anyhow::Result<Self> {
//         let database = database::NetabaseDatabase::<D>::new()?;
//         Ok(Self {
//             database,
//             database_name: Some(name),
//         })
//     }

//     /// Get direct access to the local database for WASM environments.
//     ///
//     /// Since networking is not available in WASM, this provides direct access
//     /// to the underlying database for local operations.
//     pub fn database(&self) -> &database::NetabaseDatabase<D> {
//         &self.database
//     }

//     /// Get mutable access to the local database for WASM environments.
//     pub fn database_mut(&mut self) -> &mut database::NetabaseDatabase<D> {
//         &mut self.database
//     }
// }

#[cfg(feature = "native")]
impl<D: NetabaseDefinitionTrait + Send + Sync> Drop for Netabase<D>
where
    D: netabase_store::convert::ToIVec + serde::Serialize + for<'de> serde::Deserialize<'de>,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    fn drop(&mut self) {
        if let Some(handle) = self.swarm_thread.take() {
            handle.abort();
        }
    }
}

// ============================================================================
// Paxos-Specific API Methods
// ============================================================================

#[cfg(all(feature = "native", feature = "paxos"))]
impl<D: NetabaseDefinitionTrait + Send + Sync + 'static> Netabase<D>
where
    D: netabase_store::convert::ToIVec + serde::Serialize + for<'de> serde::Deserialize<'de>,
    D::Keys: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    /// Propose a database update through Paxos consensus.
    ///
    /// This method submits a database operation (put, delete, etc.) to the Paxos cluster
    /// for consensus. The operation will only be applied once a majority of nodes in the
    /// cluster agree on it.
    ///
    /// # Consensus Process
    ///
    /// 1. **Prepare Phase**: Request promises from acceptors with a unique proposal number
    /// 2. **Accept Phase**: Ask acceptors to accept the proposed value
    /// 3. **Learn Phase**: Once accepted by a quorum, the value is learned and applied
    ///
    /// # Timeout and Retry Behavior
    ///
    /// The operation respects the configuration in [`PaxosOperationConfig`]:
    /// - `proposal_timeout`: Maximum time to wait for consensus
    /// - `max_retries`: Number of retry attempts on failure
    /// - `exponential_backoff`: Whether to use exponential backoff between retries
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The cluster doesn't have quorum (if `fail_fast_no_quorum` is true)
    /// - The proposal times out waiting for consensus
    /// - Maximum retries are exhausted
    /// - Network communication fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use netabase::Netabase;
    /// use netabase_store::netabase_definition_module;
    ///
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use definition::*;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut netabase = Netabase::<MyDefinition>::new()?;
    /// netabase.start_swarm().await?;
    ///
    /// let user = User { id: 1, name: "Alice".to_string() };
    ///
    /// // Propose the update through consensus
    /// netabase.propose_update(user).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn propose_update<M: NetabaseModelTrait<D>>(
        &self,
        record: M,
    ) -> anyhow::Result<ConsensusResult> {
        // Get operation config from PaxosConfig
        let operation_config = &self.config.paxos.operation;

        // Check if cluster has quorum if fail_fast is enabled
        if operation_config.fail_fast_no_quorum {
            let cluster_size = self.config.paxos.cluster_members.len();
            let min_quorum = self.config.paxos.min_quorum
                .unwrap_or((cluster_size / 2) + 1);

            if cluster_size < min_quorum {
                return Err(anyhow::anyhow!(
                    "Cluster size ({}) is below minimum quorum ({})",
                    cluster_size,
                    min_quorum
                ));
            }
        }

        // TODO: Implement actual Paxos proposal submission
        // This is a placeholder that will be implemented when we integrate
        // the proposal submission mechanism into the command channel

        // For now, return a placeholder indicating the feature is under development
        Err(anyhow::anyhow!(
            "Paxos consensus proposals are not yet fully implemented. \
             This will be completed in the integration phase."
        ))
    }

    /// Get information about the current Paxos cluster state.
    ///
    /// Returns details about cluster membership, quorum status, and consensus state.
    ///
    /// # Returns
    ///
    /// A [`ClusterInfo`] struct containing:
    /// - List of cluster members
    /// - Current cluster size
    /// - Required quorum size
    /// - Whether the cluster has quorum
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use definition::*;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new()?;
    /// let cluster_info = netabase.get_cluster_info()?;
    ///
    /// println!("Cluster size: {}", cluster_info.cluster_size);
    /// println!("Quorum: {}", cluster_info.quorum_size);
    /// println!("Has quorum: {}", cluster_info.has_quorum);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_cluster_info(&self) -> anyhow::Result<ClusterInfo> {
        let cluster_members = self.config.paxos.cluster_members.clone();
        let cluster_size = cluster_members.len();
        let quorum_size = self.config.paxos.min_quorum
            .unwrap_or((cluster_size / 2) + 1);
        let has_quorum = cluster_size >= quorum_size;

        Ok(ClusterInfo {
            cluster_members,
            cluster_size,
            quorum_size,
            has_quorum,
            dynamic_membership: self.config.paxos.dynamic_membership,
        })
    }

    /// Get the current Paxos operation configuration.
    ///
    /// Returns a copy of the [`PaxosOperationConfig`] that controls timeout,
    /// retry, and other operational parameters for consensus proposals.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use definition::*;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new()?;
    /// let op_config = netabase.get_operation_config();
    ///
    /// println!("Proposal timeout: {:?}", op_config.proposal_timeout);
    /// println!("Max retries: {}", op_config.max_retries);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_operation_config(&self) -> network::config::PaxosOperationConfig {
        self.config.paxos.operation.clone()
    }

    /// Check if the cluster currently has sufficient nodes for quorum.
    ///
    /// This is a convenience method that checks whether the cluster size meets
    /// the minimum quorum requirement. Useful for pre-flight checks before
    /// attempting consensus operations.
    ///
    /// # Returns
    ///
    /// - `true` if cluster_size >= quorum_size
    /// - `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use netabase::Netabase;
    /// # use netabase_store::netabase_definition_module;
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod definition {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug)]
    /// #     #[derive(bincode::Encode, bincode::Decode)]
    /// #     #[derive(serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key] pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use definition::*;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut netabase = Netabase::<MyDefinition>::new()?;
    /// if netabase.has_quorum() {
    ///     println!("Cluster has quorum - ready for consensus operations");
    /// } else {
    ///     println!("Cluster lacks quorum - waiting for more nodes");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn has_quorum(&self) -> bool {
        let cluster_size = self.config.paxos.cluster_members.len();
        let quorum_size = self.config.paxos.min_quorum
            .unwrap_or((cluster_size / 2) + 1);
        cluster_size >= quorum_size
    }
}

/// Result of a successful consensus proposal.
///
/// Contains information about the consensus round and the accepted value.
#[cfg(all(feature = "native", feature = "paxos"))]
#[derive(Debug, Clone)]
pub struct ConsensusResult {
    /// The round number in which consensus was reached
    pub round: u64,
    /// Number of acceptors that accepted the proposal
    pub acceptors: usize,
    /// Whether the proposal was unanimously accepted
    pub unanimous: bool,
}

/// Information about the Paxos cluster state.
///
/// Provides a snapshot of cluster membership and quorum status.
#[cfg(all(feature = "native", feature = "paxos"))]
#[derive(Debug, Clone)]
pub struct ClusterInfo {
    /// List of peer IDs in the cluster
    pub cluster_members: Vec<PeerId>,
    /// Total number of nodes in the cluster
    pub cluster_size: usize,
    /// Number of nodes required for quorum
    pub quorum_size: usize,
    /// Whether the cluster currently has quorum
    pub has_quorum: bool,
    /// Whether dynamic membership is enabled
    pub dynamic_membership: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bincode::{Decode, Encode};
    use netabase_store::traits::definition::{NetabaseDefinitionTrait, NetabaseDefinitionTraitKey};
    use serde::{Deserialize, Serialize};
    use std::hash::Hash;
    use strum::EnumDiscriminants;

    // Manual test definition to avoid macro issues
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, EnumDiscriminants)]
    #[strum_discriminants(derive(
        strum::EnumIter,
        strum::Display,
        strum::AsRefStr,
        strum::EnumString,
        Hash,
        Encode,
        Decode
    ))]
    #[strum_discriminants(name(TestDefinition))]
    enum TestModel {
        TestUser {
            id: u64,
            name: String,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Encode, Decode, EnumDiscriminants)]
    #[strum_discriminants(derive(
        strum::EnumIter,
        strum::Display,
        strum::AsRefStr,
        strum::EnumString,
        Hash,
        Encode,
        Decode
    ))]
    #[strum_discriminants(name(TestDefinitionKeysDiscriminant))]
    enum TestDefinitionKeys {
        Id(u64),
    }

    impl NetabaseDefinitionTraitKey for TestDefinitionKeys {}

    impl NetabaseDefinitionTrait for TestModel {
        type Keys = TestDefinitionKeys;
        type Tables = TestDefinition;

        fn tables() -> Self::Tables {
            TestDefinition::TestUser
        }

        #[cfg(all(feature = "paxos", feature = "libp2p", not(target_arch = "wasm32")))]
        fn apply_to_store<S>(&self, _store: &mut S) -> Result<(), String>
        where
            S: libp2p::kad::store::RecordStore,
        {
            Ok(())
        }
    }

    impl netabase_store::convert::ToIVec for TestModel {}
    impl netabase_store::convert::ToIVec for TestDefinitionKeys {}

    #[test]
    fn test_subscribe_to_broadcasts_is_not_async() {
        let netabase = Netabase::<TestModel>::new().unwrap();

        // This should compile without .await - proving the method is synchronous
        let _receiver = netabase.subscribe_to_broadcasts();
    }

    #[test]
    fn test_multiple_broadcast_subscriptions() {
        let netabase = Netabase::<TestModel>::new().unwrap();

        // Test that we can create multiple receivers without Arc wrapping
        let receiver1 = netabase.subscribe_to_broadcasts();
        let receiver2 = netabase.subscribe_to_broadcasts();
        let receiver3 = netabase.subscribe_to_broadcasts();

        // Verify they are independent instances
        let addr1 = &receiver1 as *const _ as usize;
        let addr2 = &receiver2 as *const _ as usize;
        let addr3 = &receiver3 as *const _ as usize;

        assert_ne!(addr1, addr2);
        assert_ne!(addr2, addr3);
        assert_ne!(addr1, addr3);
    }

    #[test]
    fn test_broadcast_receiver_cloning() {
        let netabase = Netabase::<TestModel>::new().unwrap();

        // Get a receiver
        let mut receiver1 = netabase.subscribe_to_broadcasts();

        // Clone it using resubscribe - this proves broadcast receivers are cloneable
        let mut receiver2 = receiver1.resubscribe();

        // Both should be empty initially
        assert!(receiver1.try_recv().is_err());
        assert!(receiver2.try_recv().is_err());
    }
}
