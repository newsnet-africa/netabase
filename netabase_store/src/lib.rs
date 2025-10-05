//! # Netabase Store
//!
//! The core storage layer for Netabase, providing embedded database functionality
//! built on top of [sled](https://github.com/spacejam/sled). This crate handles
//! local data persistence, indexing, querying, and relational operations with
//! optional peer-to-peer networking capabilities through libp2p integration.
//!
//! ## Overview
//!
//! Netabase Store operates in two modes:
//!
//! ### Local Mode (Core Features)
//! - Type-safe database operations through generated traits
//! - Primary and secondary key indexing
//! - Relational queries and foreign key support
//! - Advanced filtering and aggregation capabilities
//! - Batch operations for performance
//! - Direct model-to-storage serialization
//!
//! ### Distributed Mode (libp2p Feature)
//! - Schema-based network serialization
//! - libp2p Kademlia DHT integration
//! - Provider record management
//! - Automatic peer discovery and data replication
//! - Compatible with libp2p RecordStore trait
//!
//! ## Data Flow Architecture
//!
//! ### Local Storage Flow (without libp2p)
//!
//! ```text
//! User Struct ──► NetabaseModel ──► IVec ──► Sled Database
//!     │               │             │           │
//!     │               │             │           │
//!     ▼               ▼             ▼           ▼
//! [User {         [Generated    [Binary     [Persistent
//!  id: 1,          Traits]       Data]       Storage]
//!  name: "Alice"}]
//!
//!                   GET OPERATION
//!
//! Sled Database ──► IVec ──► NetabaseModel ──► User Struct
//! ```
//!
//! ### Distributed Storage Flow (with libp2p)
//!
//! ```text
//! User Struct ──► NetabaseSchema ──► Record ──► DHT Network
//!     │               │              │            │
//!     │               │              │            │
//!     ▼               ▼              ▼            ▼
//! [User {         [BlogSchema::  [libp2p::kad  [Distributed
//!  id: 1,          User(user)]    ::Record]     Storage]
//!  name: "Alice"}]     │              │            │
//!     │               │              │            │
//!     ▼               ▼              ▼            ▼
//! [Local Cache] ◄─ [IVec] ◄───── [Schema] ◄──────┘
//!
//!                   NETWORK GET OPERATION
//!
//! DHT Network ──► Record ──► NetabaseSchema ──► User Struct
//!     │             │            │                │
//!     │             │            │                │
//!     ▼             ▼            ▼                ▼
//! [Remote Peer] [Network    [BlogSchema::     [User {
//!                Data]       User(user)]       id: 1, ...}]
//! ```
//!
//! ### Schema Discriminant Routing (libp2p)
//!
//! When libp2p is enabled, data is organized by schema discriminants:
//!
//! ```text
//! NetabaseSchema ──► Discriminant ──► Tree Selection ──► Storage
//!       │                 │               │               │
//!       ▼                 ▼               ▼               ▼
//! [BlogSchema::User] [UserDiscriminant] [user_tree]   [IVec Data]
//! [BlogSchema::Post] [PostDiscriminant] [post_tree]   [IVec Data]
//! [BlogSchema::Tag]  [TagDiscriminant]  [tag_tree]    [IVec Data]
//! ```
//!
//! ## Architecture
//!
//! The crate is organized into several modules:
//!
//! - [`database`] - Core database types and implementations
//! - [`traits`] - Trait definitions for models, schemas, and operations
//! - [`errors`] - Error types and handling
//! - [`relational`] - Relational query support and foreign key handling
//!
//! ## Usage
//!
//! ### Local Database Operations
//!
//! ```rust,no_run
//! use netabase_store::database::{NetabaseSledDatabase, NetabaseSledTree};
//! use netabase_store::traits::{NetabaseModel, NetabaseSecondaryKeyQuery};
//! // Note: No need to import serde, bincode, etc. - macros handle this automatically!
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Direct model operations (local mode)
//! let db = NetabaseSledDatabase::<MySchema>::new_with_name("my_database")?;
//! let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
//!
//! // Insert and retrieve using generated traits
//! user_tree.insert(user.key(), user.clone())?;
//! let retrieved = user_tree.get(user.key())?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Network Operations (libp2p feature)
//!
//! ```rust,no_run
//! #[cfg(feature = "libp2p")]
//! use netabase_store::traits::{NetabaseSchemaQuery, NetabaseRecordStoreQuery};
//! use libp2p::kad::store::RecordStore;
//! // Note: All serialization dependencies are automatically handled by macros!
//!
//! # #[cfg(feature = "libp2p")]
//! # fn network_example() -> Result<(), Box<dyn std::error::Error>> {
//! // Schema-based operations for network compatibility
//! let mut db = NetabaseSledDatabase::<BlogSchema>::new_with_name("blog_db")?;
//!
//! // Create schema instance
//! let user_schema = BlogSchema::User(user);
//!
//! // Store locally using schema
//! db.put_schema(&user_schema)?;
//!
//! // Convert to network record
//! let record = user_schema.to_record()?;
//!
//! // Use as libp2p RecordStore
//! db.put(record)?;
//! let retrieved_record = db.get(&record.key);
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! ### Core Features (always available)
//! - **Local Storage**: Fast embedded database using sled
//! - **Type Safety**: Generated traits prevent runtime errors
//! - **Indexing**: Primary and secondary key indexes
//! - **Queries**: Range queries, filters, and relational operations
//! - **Performance**: O(log n) primary key access, batch operations
//! - **Thread Safety**: Concurrent read/write operations
//!
//! ### Network Features (libp2p feature only)
//! - **Schema Serialization**: Automatic network format conversion
//! - **DHT Integration**: Compatible with libp2p Kademlia DHT
//! - **Provider Records**: Advertise and discover data providers
//! - **Record Store**: Implements `libp2p::kad::store::RecordStore`
//! - **Network Routing**: Discriminant-based data organization
//! - **Distributed Storage**: Automatic replication and discovery
//!
//! ### Optional Features
//! - `libp2p` - Enable peer-to-peer networking capabilities
//! - `record-store` - Additional DHT record storage features
//!
//! ## Performance Characteristics
//!
//! ### Local Operations
//! - **Primary Key Access**: O(log n) using sled B+ trees
//! - **Secondary Key Queries**: O(m) where m = matching records
//! - **Range Queries**: O(log n + m) for prefix searches
//! - **Custom Filters**: O(n) - full tree scan required
//! - **Batch Operations**: ~10x faster than individual operations
//! - **Memory Usage**: ~50MB baseline + data size
//!
//! ### Network Operations (libp2p feature)
//! - **Schema Conversion**: ~1-5μs overhead per conversion
//! - **DHT Operations**: O(log n) where n = network size
//! - **Record Serialization**: ~3-4μs per operation
//! - **Provider Discovery**: Average 3-5 network hops
//! - **Memory Overhead**: +~20MB for networking stack
//!
//! ### Conversion Performance
//!
//! | Operation | Local Mode | Network Mode | Notes |
//! |-----------|------------|--------------|-------|
//! | Model → IVec | ~1μs | ~1μs | Direct serialization |
//! | IVec → Model | ~2μs | ~2μs | Includes validation |
//! | Schema → Record | N/A | ~3μs | Network format |
//! | Record → Schema | N/A | ~4μs | Network parsing |
//! | Key → RecordKey | N/A | ~1μs | Simple conversion |
//!
//! ## Thread Safety
//!
//! All database operations are thread-safe with the following guarantees:
//! - Multiple concurrent readers supported
//! - Writers are properly synchronized
//! - Schema conversions are stateless and thread-safe
//! - Network operations use async-safe primitives
//!
//! **Note**: The same database path cannot be opened by multiple processes
//! simultaneously due to sled's single-writer constraint.
//!
//! ## Error Handling
//!
//! The crate uses comprehensive error types defined in the [`errors`] module:
//!
//! ```rust,no_run
//! use netabase_store::errors::NetabaseError;
//! // Note: Error handling works seamlessly with hygienic macros!
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let user_tree: netabase_store::database::NetabaseSledTree<(), ()> = todo!();
//! # let user_key = ();
//! match user_tree.get(user_key) {
//!     Ok(Some(user)) => println!("Found: {:?}", user),
//!     Ok(None) => println!("User not found"),
//!     Err(NetabaseError::Database) => eprintln!("Database error"),
//!     Err(NetabaseError::Conversion(_)) => eprintln!("Conversion error"),
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Conversion Errors
//!
//! When using libp2p features, additional conversion errors may occur:
//!
//! ```rust,no_run
//! #[cfg(feature = "libp2p")]
//! use netabase_store::traits::NetabaseRecordStoreQuery;
//!
//! # #[cfg(feature = "libp2p")]
//! # fn conversion_example() -> Result<(), Box<dyn std::error::Error>> {
//! # let schema: () = ();
//! // Schema to Record conversion
//! match NetabaseSledDatabase::<MySchema>::schema_to_record(&schema) {
//!     Ok(record) => println!("Converted to network record"),
//!     Err(e) => eprintln!("Network serialization failed: {}", e),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Storage Organization
//!
//! ### Local Mode Structure
//! ```text
//! Database/
//! ├── model_user/           # Direct model storage
//! ├── model_post/           # One tree per model type
//! ├── secondary_email/      # Secondary key indexes
//! └── relational_author/    # Foreign key relationships
//! ```
//!
//! ### Network Mode Structure (libp2p feature)
//! ```text
//! Database/
//! ├── schema_user/          # Schema-discriminant trees
//! ├── schema_post/          # Network-compatible storage
//! ├── schema_comment/       # Organized by discriminants
//! ├── dht_providers/        # Provider record storage
//! ├── dht_provided/         # Local provider cache
//! ├── secondary_email/      # Secondary key indexes
//! └── relational_author/    # Relational query support
//! ```

pub mod database;
pub mod errors;
pub mod relational;

pub mod traits;

// Re-export macros with hygienic dependencies
pub use netabase_macros;

/// Re-exports for macro hygiene - provides all dependencies needed by generated code.
///
/// This module ensures that all macros are hygienic and don't require users to manually
/// import dependencies like `serde`, `bincode`, `strum`, etc. The macros automatically
/// use these re-exported dependencies through absolute paths.
pub use netabase_deps as __macro_deps;

/// Re-export macro dependencies for user convenience.
/// Users can access these through `netabase_store::serde`, `netabase_store::bincode`, etc.
/// but the macros will work even without manual imports thanks to hygiene.
pub use netabase_deps::*;
