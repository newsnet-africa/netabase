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
//!     UserSecondaryKeys::Email(UserEmailSecondaryKey("some@email.com".to_string()))
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
//! use log::{debug, info, warn};
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
//!                 debug!("Network event: {:?}", event);
//!             }
//!             Ok(Err(_closed)) => {
//!                 // channel closed
//!                 break;
//!             }
//!             Err(_) => {
//!                 // timeout elapsed
//!                 debug!("No event received for {}s — timing out", duration.as_secs());
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

#[cfg(feature = "native")]
pub mod network;

#[cfg(feature = "native")]
pub use network::behaviour::NetabaseBehaviourEvent;
#[cfg(feature = "native")]
pub use network::behaviour::clone_impl::NetabaseSwarmEvent;

#[cfg(feature = "native")]
use libp2p::kad::QueryResult;
#[cfg(feature = "native")]
use netabase_store::traits::{
    definition::{NetabaseDefinitionTrait, RecordStoreExt},
    model::{NetabaseModelTrait, NetabaseModelTraitKey},
};

#[cfg(not(feature = "native"))]
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
pub struct Netabase<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync>
where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as netabase_store::traits::definition::NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant: netabase_store::traits::definition::NetabaseKeyDiscriminant,
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
}

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
impl<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static> Netabase<D>
where
    D: netabase_store::traits::convert::ToIVec,
    D::Keys: netabase_store::traits::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as netabase_store::traits::definition::NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant: netabase_store::traits::definition::NetabaseKeyDiscriminant,
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
    /// use log::{debug, info, warn};
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

        Ok(Self {
            swarm_thread: None,
            config,
            command_sender,
            broadcast_receiver,
            database_path: None,
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

        Ok(Self {
            swarm_thread: None,
            config: NetabaseConfig::default(),
            command_sender,
            broadcast_receiver,
            database_path: Some(path.as_ref().to_string_lossy().to_string()),
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

        Ok(Self {
            swarm_thread: None,
            config,
            command_sender,
            broadcast_receiver,
            database_path: Some(path.as_ref().to_string_lossy().to_string()),
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
    /// use log::{debug, info, warn};
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
    /// use log::{debug, info, warn};
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
    /// use log::{debug, info, warn};
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
    ///         debug!("Network event: {:?}", event);
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
    /// # use log::{debug, info, warn};
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
    ///     Ok(result) => debug!("Stored successfully: {:?}", result),
    ///     Err(e) => warn!("Storage failed: {}", e),
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
    /// # use log::{debug, info, warn};
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
    ///     Ok(result) => debug!("Query completed: {:?}", result),
    ///     Err(e) => warn!("Query failed: {}", e),
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
    /// # use log::{debug, info, warn};
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
    ///     Ok(result) => debug!("Query completed: {:?}", result),
    ///     Err(e) => warn!("Provider query failed: {}", e),
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
    /// # use log::{debug, info, warn};
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
    ///         debug!("Now providing the key");
    ///         // Other nodes can now discover us as a provider
    ///     }
    ///     Err(e) => warn!("Failed to start providing: {}", e),
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
    /// # use log::{debug, info, warn};
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
    /// debug!("No longer providing this key");
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
    /// # use log::{debug, info, warn};
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
    ///             debug!("Successfully joined the DHT network");
    ///         } else {
    ///             debug!("Bootstrap in progress, {} peers remaining", result.num_remaining);
    ///         }
    ///     }
    ///     Ok(QueryResult::Bootstrap(Err(e))) => warn!("Bootstrap failed: {:?}", e),
    ///     Ok(_) => debug!("Unexpected result"),
    ///     Err(e) => warn!("Bootstrap error: {}", e),
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
    /// use log::{debug, info, warn};
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
    ///         Ok(update) => debug!("Routing table updated: {:?}", update),
    ///         Err(e) => warn!("Failed to add address: {}", e),
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
    /// # use log::{debug, info, warn};
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
    ///     Ok(Some(entry)) => debug!("Address removed, peer still has addresses"),
    ///     Ok(None) => debug!("Address removed, peer completely removed from routing table"),
    ///     Err(e) => warn!("Failed to remove address: {}", e),
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
    /// # use log::{debug, info, warn};
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
    ///         debug!("Removed peer from routing table");
    ///     }
    ///     Ok(None) => debug!("Peer was not in routing table"),
    ///     Err(e) => warn!("Failed to remove peer: {}", e),
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
    /// # use log::{debug, info, warn};
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
    ///         debug!("Operating in client mode");
    ///     }
    ///     Ok(libp2p::kad::Mode::Server) => {
    ///         debug!("Operating in server mode");
    ///     }
    ///     Err(e) => warn!("Failed to get mode: {}", e),
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
    /// # use log::{debug, info, warn};
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
    /// debug!("Now running in server mode");
    ///
    /// // Switch to client mode for lower resource usage
    /// netabase.set_mode(Some(Mode::Client)).await.unwrap();
    /// debug!("Now running in client mode");
    ///
    /// // Let the system choose automatically
    /// netabase.set_mode(None).await.unwrap();
    /// debug!("Using automatic mode selection");
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
    /// # use log::{debug, info, warn};
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
    ///         debug!("Using protocol: {:?}", protocol);
    ///     }
    ///     Err(e) => warn!("Failed to get protocol: {}", e),
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
    /// # use log::{debug, info, warn};
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
    ///     Ok(()) => debug!("Record removed from local storage"),
    ///     Err(e) => warn!("Failed to remove record: {}", e),
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
    /// # use log::{debug, info, warn};
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
    ///         debug!("Found {} records in local store", records.len());
    ///         for record in records {
    ///             debug!("Record: {:?}", record);
    ///         }
    ///     }
    ///     Err(e) => warn!("Query failed: {}", e),
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

    /// Store a record directly to the local store without publishing to the DHT.
    ///
    /// This method bypasses the DHT and stores the record only in the local Kademlia
    /// store. This is useful for:
    /// - Testing and debugging
    /// - Caching data locally
    /// - Pre-populating the local store
    ///
    /// # Parameters
    ///
    /// - `record`: The record to store locally
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the record was stored successfully, or an error if storage failed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use netabase::Netabase;
    /// use netabase_store::netabase_definition_module;
    /// use log::{debug, info, warn};
    ///
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod my_module {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct MyRecord {
    /// #         #[primary_key]
    /// #         pub id: String,
    /// #         pub data: String,
    /// #     }
    /// # }
    /// # use my_module::*;
    /// # async fn example() -> anyhow::Result<()> {
    /// let mut netabase = Netabase::<MyDefinition>::new()?;
    /// netabase.start_swarm().await?;
    ///
    /// let record = MyRecord {
    ///     id: "test-1".to_string(),
    ///     data: "test data".to_string(),
    /// };
    ///
    /// // Store locally without publishing to DHT
    /// // Convert the model to a Definition enum
    /// let definition: MyDefinition = record.into();
    /// netabase.put_record_locally(definition).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_record_locally(&self, record: D) -> anyhow::Result<()> {
        let (response_tx, response_rx) = oneshot::channel();

        let command = network::swarm::handlers::command_events::Command::Kademlia(
            network::swarm::handlers::command_events::KademliaCommand::LocalStore(
                network::swarm::handlers::command_events::LocalStoreCommand::PutRecordLocally {
                    record,
                    response_channel: response_tx,
                },
            ),
        );

        self.command_sender.send(command).await?;

        match response_rx.await? {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Put record locally failed: {}", e)),
        }
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
// impl<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static> Netabase<D> {
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
impl<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync> Drop for Netabase<D>
where
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
    fn drop(&mut self) {
        if let Some(handle) = self.swarm_thread.take() {
            handle.abort();
        }
    }
}

// Note: Tests have been moved to tests/broadcast_tests.rs
//
// Inline test modules using netabase_definition_module can encounter trait resolution issues
// in certain Rust compiler contexts. External test files work correctly and are preferred.
