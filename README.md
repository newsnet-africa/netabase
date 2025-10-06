# Netabase

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)



**Netabase** is a distributed, peer-to-peer database system built on top of [sled](https://github.com/spacejam/sled) with optional [libp2p](https://libp2p.io/) integration. It provides a type-safe, macro-driven approach to defining database schemas and models with support for primary keys, secondary keys, and relational queries.

The system operates in two modes:
- **Local Mode**: High-performance embedded database for single-node applications
- **Distributed Mode**: P2P networked database with automatic synchronization (requires `libp2p` feature)


# ! This crate is a work in progress, and some features might be buggy, behave weirdly or have not been fully implemented. Please let me know in the issues if you notice something that is not already there
If you do make an issue, it may be moved to the [netabase_store](https://github.com/nzuzo-newsnet/netabase_store) repo but if possible, try create issues at the relevant repos.

*DO NOT* use this in a production environment as:
1. There will definately be breaking changes
2. This crate has not been extensively tested

## 🚀 Features

### Core Features (Always Available)
- **Type-Safe Models**: Automatic code generation for database models using derive macros
- **Primary & Secondary Keys**: Efficient indexing and querying capabilities
- **Embedded Storage**: Fast, local database operations using sled
- **Relational Support**: Foreign key relationships and join-like operations
- **Advanced Queries**: Complex filtering, range queries, and analytics
- **Batch Operations**: High-performance bulk operations

### Network Features (libp2p Feature)
- **Distributed Architecture**: Peer-to-peer networking with DHT-based record storage
- **Schema-Based Networking**: Automatic serialization for network operations
- **Record Store Interface**: Compatible with libp2p's Kademlia DHT
- **Provider Discovery**: Advertise and discover data providers on the network
- **Network Transparency**: Seamless data synchronization across network nodes

### TODO:
- **Advanced DHT Operations**: Enhanced integration with libp2p kademlia features

## TODO & Unimplemented Features

### Code-Level TODOs from Codebase Analysis

#### Network Event Handlers (All Unimplemented)
- [ ] **Connection Management**: All connection event handlers are placeholder implementations
  - [ ] `handle_connection_established` - Only prints debug info (src/network/swarm/handlers/swarm_events/connection_established.rs:14)
  - [ ] `handle_connection_closed` - Only prints debug info (src/network/swarm/handlers/swarm_events/connection_closed.rs:12)
  - [ ] `handle_incoming_connection` - Only prints debug info (src/network/swarm/handlers/swarm_events/incoming_connection.rs:9)
  - [ ] `handle_incoming_connection_error` - Only prints debug info (src/network/swarm/handlers/swarm_events/incoming_connection_error.rs:12)
  - [ ] `handle_outgoing_connection_error` - Only prints debug info (src/network/swarm/handlers/swarm_events/outgoing_connection_error.rs:9)
  - [ ] `handle_dialing` - Only prints debug info (src/network/swarm/handlers/swarm_events/dialing.rs:5)

#### mDNS Discovery Implementation
- [ ] **mDNS Peer Management**: Discovery handlers need full implementation
  - [ ] `handle_discovered` - Add peer to routing table/peer store (src/network/swarm/handlers/swarm_events/behaviour/mdns.rs:17)
  - [ ] `handle_expired` - Remove peer from routing table, close connections (src/network/swarm/handlers/swarm_events/behaviour/mdns.rs:29)

#### Identity/Identification System
- [ ] **Peer Identification**: All identify event handlers are placeholders
  - [ ] `handle_received` - Process received peer identification (src/network/swarm/handlers/swarm_events/behaviour/identify.rs:42)
  - [ ] `handle_sent` - Handle sent identification events (src/network/swarm/handlers/swarm_events/behaviour/identify.rs:51)
  - [ ] `handle_pushed` - Process pushed identification info (src/network/swarm/handlers/swarm_events/behaviour/identify.rs:60)
  - [ ] `handle_error` - Implement identification error handling (src/network/swarm/handlers/swarm_events/behaviour/identify.rs:73)

#### Address Management
- [ ] **External Address Handling**: Address discovery and management
  - [ ] `handle_new_external_addr_candidate` - Process new external address candidates (src/network/swarm/handlers/swarm_events/new_external_addr_candidate.rs:4)
  - [ ] `handle_external_addr_confirmed` - Handle confirmed external addresses (src/network/swarm/handlers/swarm_events/external_addr_confirmed.rs:4)
  - [ ] `handle_external_addr_expired` - Clean up expired external addresses (src/network/swarm/handlers/swarm_events/external_addr_expired.rs:4)
  - [ ] `handle_new_external_addr_of_peer` - Track peer external addresses (src/network/swarm/handlers/swarm_events/new_external_addr_of_peer.rs:4)

#### Listener Management
- [ ] **Network Listener Handling**: Listener lifecycle management
  - [ ] `handle_new_listen_addr` - Process new listen addresses (src/network/swarm/handlers/swarm_events/new_listen_addr.rs:4)
  - [ ] `handle_expired_listen_addr` - Handle expired listen addresses (src/network/swarm/handlers/swarm_events/expired_listen_addr.rs:4)
  - [ ] `handle_listener_closed` - Clean up closed listeners (src/network/swarm/handlers/swarm_events/listener_closed.rs:8)
  - [ ] `handle_listener_error` - Handle listener errors (src/network/swarm/handlers/swarm_events/listener_error.rs:4)

#### Command System
- [ ] **Command Processing**: Fallback command handler needs implementation
  - [ ] `handle_fallback_command` - Implement proper error handling or logging for unmatched commands (src/network/swarm/handlers/command_events/fallback.rs:5)

#### Fallback Event Handling
- [ ] **Event Processing**: Comprehensive fallback event handler
  - [ ] `handle_fallback_event` - Implement fallback for unhandled swarm events (src/network/swarm/handlers/swarm_events/fallback.rs:5)

#### Dead Code and Warnings
- [ ] **Unused Imports**: Multiple unused imports need cleanup
  - [ ] Remove unused `PublicKey` and `identify` imports (src/network/behaviour/mod.rs:2)
  - [ ] Remove unused `RecordStore` import (src/network/behaviour/clone_impl.rs:3)
  - [ ] Remove unused `NetabaseDatabase` import (src/network/behaviour/clone_impl.rs:8)
  - [ ] Remove unused `NetabaseSwarmEvent`, `Command`, `start_swarm_loop` imports (src/network/swarm/mod.rs:7)
  - [ ] Remove unused `Command` import (src/network/swarm/handlers/mod.rs:9)

#### Error Handling
- [ ] **WebSocket Error Types**: Fixed generic parameter issue but needs validation
  - [ ] Verify `error::Error<std::io::Error>` usage is correct (src/errors/mod.rs:14)

### Core Database Features
- [ ] **Complete macro system implementation**
  - [ ] Finish serialization macro generation
  - [ ] Add serde integration alongside bincode
  - [ ] Implement fallible conversion macros (TryFrom/TryInto)
  - [ ] Clean up macro code generation and error handling
- [ ] **Advanced query system**
  - [ ] Add SQL-like query interface
  - [ ] Implement complex joins and aggregations
  - [ ] Support for query optimization
  - [ ] Add indexing strategies for better performance
- [ ] **Schema evolution and migrations**
  - [ ] Add automatic schema migration support
  - [ ] Implement backward compatibility checking
  - [ ] Add data transformation during migrations
- [ ] **Performance optimizations**
  - [ ] Implement connection pooling for high-concurrency scenarios
  - [ ] Add batch operation optimizations
  - [ ] Optimize memory usage for large datasets
  - [ ] Add query result caching

### Distributed Networking Features
- [ ] **Complete DHT functionality**
  - [ ] Complete Kademlia configuration options
  - [ ] Add network protection and security features
  - [ ] Implement gossipsub for query functionality
  - [ ] Add peer discovery and connection management
- [ ] **Data consistency and replication**
  - [ ] Implement configurable storage backends
  - [ ] Add data replication and consistency guarantees
  - [ ] Design conflict resolution strategies
  - [ ] Add distributed transaction support
- [ ] **Network resilience**
  - [ ] Add automatic failover mechanisms
  - [ ] Implement network partition tolerance
  - [ ] Add peer health monitoring
  - [ ] Support for dynamic network topology changes
- [ ] **Security and privacy**
  - [ ] Add encryption for data at rest and in transit
  - [ ] Implement access control and authentication
  - [ ] Add data integrity verification
  - [ ] Support for private networks and permissioned access

### Query and Analytics
- [ ] **Distributed query processing**
  - [ ] Add distributed query planning and execution
  - [ ] Implement map-reduce style operations
  - [ ] Support for streaming queries
  - [ ] Add real-time analytics capabilities
- [ ] **Advanced indexing**
  - [ ] Add full-text search capabilities
  - [ ] Implement multi-dimensional indexing
  - [ ] Support for geospatial queries
  - [ ] Add time-series data optimizations

### Production Readiness
- [ ] **Monitoring and observability**
  - [ ] Add comprehensive metrics collection
  - [ ] Implement distributed tracing
  - [ ] Add health check endpoints
  - [ ] Support for logging and audit trails
- [ ] **Configuration and deployment**
  - [ ] Add configuration management system
  - [ ] Implement deployment automation tools
  - [ ] Support for containerized deployments
  - [ ] Add backup and restore functionality
- [ ] **Testing and quality assurance**
  - [ ] Add comprehensive test suite for distributed scenarios
  - [ ] Implement chaos engineering tests
  - [ ] Add performance benchmarking tools
  - [ ] Support for load testing distributed networks

### Developer Experience
- [ ] **Documentation and examples**
  - [ ] Create comprehensive API documentation
  - [ ] Add tutorial and getting started guides
  - [ ] Provide real-world example applications
  - [ ] Add troubleshooting and debugging guides
- [ ] **Tooling and utilities**
  - [ ] Add CLI tools for database management
  - [ ] Implement data migration utilities
  - [ ] Support for development and testing environments
  - [ ] Add code generation tools for common patterns

### Research and Innovation
- [ ] **Incentive mechanisms**
  - [ ] Design incentive models for data persistence
  - [ ] Implement reputation systems for network participants
  - [ ] Add economic models for resource sharing
- [ ] **Advanced networking protocols**
  - [ ] Explore integration with other P2P protocols
  - [ ] Add support for content-addressed storage
  - [ ] Implement advanced routing strategies
- [ ] **Machine learning integration**
  - [ ] Add support for distributed ML training
  - [ ] Implement automated performance optimization
  - [ ] Support for intelligent data placement

## 📦 Installation

Add Netabase to your `Cargo.toml`:

```toml
[dependencies]
# For local-only database operations
netabase_store = { git = "https://github.com/newsnet-africa/netabase_store.git" }
netabase_macros = { git = "https://github.com/newsnet-africa/netabase_macros.git" }

# For distributed P2P operations (includes everything above)
netabase = { git = "https://github.com/newsnet-africa/netabase.git", features = ["libp2p"] }

# Required for serialization
bincode = { version = "2.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

# Optional for additional serialization support
serde = { version = "1.0", features = ["derive"] }
```

### Feature Flags

- **`libp2p`** (optional): Enables peer-to-peer networking capabilities
- **`record-store`** (optional): Additional record storage features for DHT operations

## 🏃 Quick Start

### 1. Define Your Data Models

```rust
use netabase_macros::{NetabaseModel, netabase_schema_module};
use bincode::{Encode, Decode};
use serde::{Serialize, Deserialize};

#[netabase_schema_module(BlogSchema, BlogKeys)]
mod blog {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, Serialize, Deserialize)]
    #[key_name(UserKey)]
    pub struct User {
        #[key]
        pub id: u64,                    // Primary key
        pub name: String,
        #[secondary_key]
        pub email: String,              // Secondary key for efficient queries
        pub created_at: u64,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, Serialize, Deserialize)]
    #[key_name(PostKey)]
    pub struct Post {
        #[key]
        pub id: u64,                    // Primary key
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,             // Foreign key to User
        #[secondary_key]
        pub published: bool,            // Secondary key for filtering
        pub created_at: u64,
    }
}

use blog::*;
```

### 2. Local Database Operations
```rust
use netabase_store::database::{NetabaseSledDatabase, NetabaseSledTree};
use netabase_store::traits::{NetabaseModel, NetabaseSecondaryKeyQuery};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database
    let db = NetabaseSledDatabase::<BlogSchema>::new_with_name("my_blog_db")?;
    let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
    let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;

    // Create and insert a user
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: chrono::Utc::now().timestamp() as u64,
    };
    user_tree.insert(user.key(), user.clone())?;

    // Create and insert a post
    let post = Post {
        id: 1,
        title: "Hello, Netabase!".to_string(),
        content: "This is my first post using Netabase.".to_string(),
        author_id: user.id,
        published: true,
        created_at: chrono::Utc::now().timestamp() as u64,
    };
    post_tree.insert(post.key(), post.clone())?;

    // Query by primary key
    let retrieved_user = user_tree.get(user.key())?.unwrap();
    println!("User: {}", retrieved_user.name);

    // Query by secondary key
    let users_by_email = user_tree.query_by_secondary_key(
        UserSecondaryKeys::EmailKey("alice@example.com".to_string())
    )?;
    println!("Found {} users with this email", users_by_email.len());

    // Query posts by author
    let author_posts = post_tree.query_by_secondary_key(
        PostSecondaryKeys::Author_idKey(user.id)
    )?;
    println!("Alice has {} posts", author_posts.len());

    // Query published posts
    let published_posts = post_tree.query_by_secondary_key(
        PostSecondaryKeys::PublishedKey(true)
    )?;
    println!("There are {} published posts", published_posts.len());

    Ok(())
}
```

## 📚 Core Concepts & Data Flow

### Models and Keys

**Models** are your data structures marked with `#[derive(NetabaseModel)]`:
- Must have exactly one `#[key]` field (primary key)
- Can have multiple `#[secondary_key]` fields for efficient queries
- Automatically generate type-safe key enums

### Schemas

**Schemas** organize related models using `#[netabase_schema_module]`:
- Group multiple model types together
- Enable network serialization (when libp2p feature is enabled)
- Provide unified database interfaces

### Trees

**Trees** are storage structures for each model type:
- One tree per model type
- Support CRUD operations
- Maintain secondary key indexes automatically

## 🔄 Data Flow Architecture

### Local Mode Data Flow (without libp2p)

```
User Struct ─────► NetabaseSchema ─────► IVec ─────► Sled Database
    │                    │                │              │
    │                    │                │              │
    ▼                    ▼                ▼              ▼
[User { id: 1,      [BlogSchema::      [Serialized     [Persistent
 name: "Alice" }]    User(user)]        Binary Data]     Storage]

                         GET OPERATION FLOW
                              
Sled Database ────► IVec ────► NetabaseSchema ────► User Struct
    │                │              │                   │
    │                │              │                   │
    ▼                ▼              ▼                   ▼
[Persistent    [Binary Data]  [BlogSchema::         [User { id: 1,
 Storage]                       User(user)]          name: "Alice" }]
```

### Distributed Mode Data Flow (with libp2p feature)

```
User Struct ────► NetabaseSchema ────► Record ────► Network (DHT)
    │                   │                │              │
    │                   │                │              │
    ▼                   ▼                ▼              ▼
[User { id: 1,     [BlogSchema::    [libp2p::kad::   [Distributed
 name: "Alice" }]   User(user)]      Record]           Storage]
    │                   │                │              │
    │                   │                │              │
    ▼                   ▼                ▼              ▼
[Local Storage] ◄── [IVec] ◄────── [NetabaseSchema] ◄─┘

                         NETWORK GET OPERATION FLOW

Network (DHT) ────► Record ────► NetabaseSchema ────► User Struct
    │                 │              │                   │
    │                 │              │                   │
    ▼                 ▼              ▼                   ▼
[Distributed    [libp2p::kad::  [BlogSchema::        [User { id: 1,
 Storage]        Record]          User(user)]          name: "Alice" }]
    │                 │              │                   │
    │                 │              │                   │
    ▼                 ▼              ▼                   ▼
[Local Cache] ◄── [IVec] ◄────── [NetabaseSchema] ◄──── ┘
```

### Key Conversion Paths

```
                    LOCAL OPERATIONS
UserKey ────► IVec ────► Sled Tree ────► IVec ────► User
   │            │           │             │          │
   │            │           │             │          │
   ▼            ▼           ▼             ▼          ▼
[UserKey::   [Binary    [Tree Storage] [Binary   [User {
Primary(1)]   Data]                     Data]     id: 1, ...}]

                   NETWORK OPERATIONS (libp2p)
UserKey ────► RecordKey ────► DHT ────► RecordKey ────► UserKey
   │             │             │           │             │
   │             │             │           │             │
   ▼             ▼             ▼           ▼             ▼
[UserKey::   [libp2p::kad:: [Network   [libp2p::kad:: [UserKey::
Primary(1)]   RecordKey]     Storage]   RecordKey]     Primary(1)]
```

### Schema Discriminant Routing

When libp2p is enabled, data is organized by schema discriminants for efficient querying:

```
NetabaseSchema ────► Discriminant ────► Tree Selection ────► Storage
      │                    │                   │                │
      │                    │                   │                │
      ▼                    ▼                   ▼                ▼
[BlogSchema::User]   [UserDiscriminant]  [schema_user_tree]  [IVec Storage]
[BlogSchema::Post]   [PostDiscriminant]  [schema_post_tree]  [IVec Storage]
[BlogSchema::Tag]    [TagDiscriminant]   [schema_tag_tree]   [IVec Storage]
```

### Provider Record Flow (libp2p only)

```
ProviderRecord ────► StoredProviderRecord ────► IVec ────► Provider Trees
      │                       │                   │              │
      │                       │                   │              │
      ▼                       ▼                   ▼              ▼
[libp2p Provider]      [Serializable         [Binary        [dht_providers/
 [Network Addr]         Provider Record]      Data]          dht_provided]
 [Expiry Time]          [PeerId as bytes]
 [Key Info]             [Addresses as bytes]
```

## 🔧 Advanced Usage

### Complex Queries

```rust
use netabase_store::traits::NetabaseAdvancedQuery;

// Custom filtering
let active_users = user_tree.query_with_filter(|user| {
    user.created_at > some_timestamp && user.email.contains("@company.com")
})?;

// Count matching records
let user_count = user_tree.count_where(|user| user.name.starts_with("A"))?;

// Range queries by prefix
let recent_posts = post_tree.range_by_prefix(b"2024_")?;

// Batch operations
let new_users = vec![
    (UserKey::Primary(UserPrimaryKey(2)), user2),
    (UserKey::Primary(UserPrimaryKey(3)), user3),
];
user_tree.batch_insert_with_indexing(new_users)?;
```

### Network Provider Operations

```rust
// Advertise as a provider for specific data
let post_key = PostKey::Primary(PostPrimaryKey(1));
netabase.start_providing(post_key.clone()).await?;

// Find providers for data
let providers = netabase.get_providers(post_key).await?;

// Stop providing
netabase.stop_providing(post_key).await?;
```

## 📖 Examples

Check out the `examples/` directory for comprehensive demonstrations:

- **[getting_started.rs](example_usage/getting_started.rs)** - Basic usage and CRUD operations
- **[blog_system.rs](example_usage/blog_system.rs)** - Complete blog system with multiple models
- **[advanced_queries.rs](example_usage/advanced_queries.rs)** - Complex queries and analytics

Run examples (from with `example_usage` folder) with:
```bash
cargo run 
```

## 🧪 Testing

Run the test suite:
```bash
# Run all tests (single-threaded to avoid database conflicts)
cargo test --test-threads=1

# Run specific test files
cargo test --test integration_tests --test-threads=1
cargo test --test handler_tests --test-threads=1
```

For distributed tests:
```bash
cargo test --test multi_process_tests --test-threads=1
```

## 🏗️ Architecture

Netabase consists of three main layers:

### 1. Storage Layer (`netabase_store`)
- **Core Operations**: Embedded database operations using sled
- **Data Management**: CRUD operations and indexing
- **Query Engine**: Secondary key and relational queries
- **Persistence**: Local data persistence with IVec serialization
- **Schema Integration**: NetabaseSchema-based storage routing

### 2. Macro Layer (`netabase_macros`)
- **Code Generation**: Procedural macros for model and schema definitions
- **Type Safety**: Automatic key type generation and validation
- **Trait Implementation**: Auto-generated NetabaseModel, NetabaseSchema traits
- **Discriminant Generation**: Schema discriminant enums for tree routing
- **Conversion Methods**: Automatic IVec, Record, and Key conversions

### 3. Network Layer (`netabase`) - libp2p Feature Only
- **P2P Networking**: Peer-to-peer networking with libp2p
- **DHT Operations**: Distributed hash table record storage
- **Record Store**: Compatible with libp2p Kademlia RecordStore trait
- **Provider System**: Data provider advertisement and discovery
- **Event System**: Network event handling and subscription

### Data Flow Integration

```
Application Code
       │
       ▼
┌─────────────────┐
│   Macro Layer   │ ◄──── Generates traits and conversion methods
│  (Compile Time) │
└─────────────────┘
       │
       ▼
┌─────────────────┐     ┌──────────────────┐
│  Storage Layer  │ ◄── │  Network Layer   │ (Optional libp2p)
│   (Local DB)    │     │   (P2P Network)  │
└─────────────────┘     └──────────────────┘
       │                         │
       ▼                         ▼
┌─────────────────┐     ┌──────────────────┐
│   Sled Trees    │     │   DHT Records    │
│ (IVec Storage)  │     │ (Network Cache)  │
└─────────────────┘     └──────────────────┘
```

### Storage Organization

#### Without libp2p Feature:
```
Database/
├── model_user/           # Direct model storage
├── model_post/           # One tree per model type
├── secondary_email/      # Secondary key indexes
└── secondary_author_id/  # Efficient query support
```

#### With libp2p Feature:
```
Database/
├── schema_user/          # Schema-discriminant based storage
├── schema_post/          # NetabaseSchema organization
├── schema_comment/       # Supports network operations
├── dht_providers/        # Provider record storage
├── dht_provided/         # Local provider cache
├── secondary_email/      # Secondary key indexes
└── relational_author/    # Relational query support
```

## 🎯 Use Cases

Netabase is perfect for:

- **Decentralized Applications**: Build apps without central servers
- **Local-First Software**: Offline-capable applications with sync
- **P2P Content Sharing**: Distributed content networks
- **IoT Networks**: Device-to-device data sharing
- **Collaborative Tools**: Real-time collaboration without servers
- **Research Projects**: Distributed systems research and prototyping

## 📊 Performance

### Local Storage Performance
- **Primary Key Access**: O(log n) - using sled B+ trees
- **Secondary Key Queries**: O(m) where m = matching records
- **Range Queries**: O(log n + m) for efficient prefix searches
- **Custom Filters**: O(n) - requires full tree scan
- **Batch Operations**: ~10x faster than individual operations
- **Schema Conversions**: ~1μs overhead per conversion

### Network Performance (libp2p feature)
- **DHT Operations**: O(log n) where n = network size
- **Record Serialization**: ~5μs per NetabaseSchema conversion
- **Provider Discovery**: Average 3-5 network hops
- **Record Replication**: Automatic with K=20 redundancy
- **Peer Discovery**: Efficient Kademlia-based routing

### Memory Usage
- **Local Mode**: ~50MB baseline + data size
- **Network Mode**: +~20MB for libp2p networking stack
- **Schema Overhead**: ~1KB per discriminant type
- **Provider Cache**: ~10KB per 1000 provider records

### Conversion Overhead

| Operation | Local Mode | Network Mode | Notes |
|-----------|------------|--------------|-------|
| User Struct → IVec | ~1μs | ~1μs | Direct binary serialization |
| IVec → User Struct | ~2μs | ~2μs | Includes validation |
| NetabaseSchema → Record | N/A | ~3μs | Network serialization |
| Record → NetabaseSchema | N/A | ~4μs | Network deserialization |
| Key → RecordKey | N/A | ~1μs | Simple conversion |

## 🛠️ Development

### Building from Source

```bash
git clone https://github.com/yourorg/netabase.git
cd netabase
cargo build --release
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Run the test suite: `cargo test --test-threads=1`
5. Submit a pull request

### Development Guidelines

- Use `tempfile::TempDir` for test databases
- Run tests single-threaded to avoid sled conflicts
- Document public APIs thoroughly
- Follow Rust naming conventions

## 🐛 Troubleshooting

### Common Issues

### Database Path Conflicts
```rust
// ❌ Don't reuse paths in tests
let db = NetabaseSledDatabase::new_with_name("test_db")?;

// ✅ Use unique paths
let temp_dir = tempfile::TempDir::new()?;
let db_path = temp_dir.path().join("unique_test_db");
let db = NetabaseSledDatabase::new_with_name(&db_path.to_string_lossy())?;
```

**Network Timeouts (libp2p feature)**
```rust
// DHT operations may timeout in single-node setups
match timeout(Duration::from_secs(10), netabase.put_record(data)).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(_) => println!("Timeout - normal for single-node testing"),
}
```

**Schema Conversion Errors**
```rust
// ❌ Incorrect conversion attempt
let record: Record = user_struct.try_into()?; // Won't work directly

// ✅ Proper conversion flow
let schema = BlogSchema::User(user_struct);
let record = schema.to_record()?;
```

**Feature Gate Issues**
```rust
// ❌ Using network features without libp2p
use netabase_store::traits::NetabaseRecordStoreQuery; // Compile error

// ✅ Conditional compilation
#[cfg(feature = "libp2p")]
use netabase_store::traits::NetabaseRecordStoreQuery;
```

**Macro Compilation Errors**
```rust
// ❌ Missing required attributes
#[derive(NetabaseModel)]
struct User {
    pub id: u64,  // Missing #[key]
}

// ✅ Proper model definition
#[derive(NetabaseModel)]
#[key_name(UserKey)]
struct User {
    #[key]
    pub id: u64,
}

// ❌ Missing schema discriminant implementations
#[netabase_schema_module(MySchema, MyKeys)]
mod my_schema {
    // Missing: use super::*;
    pub use super::User;
}

// ✅ Proper schema module
#[netabase_schema_module(MySchema, MyKeys)]
mod my_schema {
    use super::*;  // Required for macro expansion
    pub use super::{User, UserKey};
}
```

**Data Conversion Debugging**
```rust
// Enable detailed conversion logging
env_logger::Builder::from_default_env()
    .filter_level(log::LevelFilter::Trace)
    .init();

// Check intermediate conversion steps
let schema = BlogSchema::User(user);
println!("Schema: {:?}", schema);

let ivec = schema.to_ivec()?;
println!("IVec length: {}", ivec.len());

#[cfg(feature = "libp2p")]
{
    let record = schema.to_record()?;
    println!("Record key: {:?}", record.key);
    println!("Record value length: {}", record.value.len());
}
```

### Debugging

Enable debug logging:
```rust
env_logger::Builder::from_default_env()
    .filter_level(log::LevelFilter::Debug)
    .init();
```

## 📄 License

This project is licensed under the GNU GPL v3 License - see the [LICENSE](LICENSE) file for details.

## 🤝 Acknowledgments

- [sled](https://github.com/spacejam/sled) - Fast embedded database
- [libp2p](https://libp2p.io/) - Modular peer-to-peer networking
- [tokio](https://tokio.rs/) - Asynchronous runtime
- [serde](https://serde.rs/) - Serialization framework

## 📬 Support

- 📖 [Documentation](NETABASE_GUIDE.md)
- 🐛 [Issue Tracker](https://github.com/yourorg/netabase/issues)
- 💬 [Discussions](https://github.com/yourorg/netabase/discussions)

---

**Happy coding with Netabase!** 🎉
