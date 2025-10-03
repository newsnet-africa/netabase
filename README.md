# Netabase

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Netabase** is a distributed, peer-to-peer database system built on top of [sled](https://github.com/spacejam/sled) with [libp2p](https://libp2p.io/) integration. It provides a type-safe, macro-driven approach to defining database schemas and models with support for primary keys, secondary keys, and relational queries.


# ! This crate is a work in progress, and some features might be buggy, behave weirdly or have not been fully implemented. Please let me know in the issues if you notice something that is not already there
If you do make an issue, it may be moved to the [netabase_store](https://github.com/nzuzo-newsnet/netabase_store) repo but if possible, try create issues at the relevant repos.

*DO NOT* use this in a production environment as:
1. There will definately be breaking changes
2. This crate has not been extensively tested

## 🚀 Features

- **Type-Safe Models**: Automatic code generation for database models using derive macros
- **Primary & Secondary Keys**: Efficient indexing and querying capabilities
- **Distributed Architecture**: Peer-to-peer networking with DHT-based record storage
- **Relational Support**: Foreign key relationships and join-like operations
- **Network Transparency**: Seamless data synchronization across network nodes
- **Advanced Queries**: Complex filtering, range queries, and analytics
- **Batch Operations**: High-performance bulk operations

### TODO:
- **Libp2p Kademlia**: Complete integration with the libp2p kademlia implementation

## 📦 Installation //TODO: Re-Export

Add Netabase to your `Cargo.toml`:

//TODO: test and publish to crates.io
```toml
[dependencies]
netabase = { path = "path/to/netabase" }
netabase_store = { path = "path/to/netabase/netabase_store" }
netabase_macros = { path = "path/to/netabase/netabase_store/netabase_macros" }
bincode = { version = "2.0", features = ["derive", "serde"] }
serde = { version = "1.0", features = ["derive"] } # Optional if you use serde for serialising and deserialising
tokio = { version = "1.0", features = ["full"] }
```

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

## 📚 Core Concepts

### Models and Keys

**Models** are your data structures marked with `#[derive(NetabaseModel)]`:
- Must have exactly one `#[key]` field (primary key)
- Can have multiple `#[secondary_key]` fields for efficient queries
- Automatically generate type-safe key enums

### Schemas

**Schemas** organize related models using `#[netabase_schema_module]`:
- Group multiple model types together
- Enable network serialization
- Provide unified database interfaces

### Trees

**Trees** are storage structures for each model type:
- One tree per model type
- Support CRUD operations
- Maintain secondary key indexes automatically

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

- **[getting_started.rs](examples/getting_started.rs)** - Basic usage and CRUD operations
- **[blog_system.rs](examples/blog_system.rs)** - Complete blog system with multiple models
- **[advanced_queries.rs](examples/advanced_queries.rs)** - Complex queries and analytics

Run examples with:
```bash
cargo run --example getting_started
cargo run --example blog_system
cargo run --example advanced_queries
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
- Embedded database operations using sled
- CRUD operations and indexing
- Secondary key and relational queries
- Local data persistence

### 2. Macro Layer (`netabase_macros`)
- Procedural macros for code generation
- Type-safe model and schema definitions
- Automatic key type generation
- Serialization trait implementations

### 3. Network Layer (`netabase`)
- Peer-to-peer networking with libp2p
- Distributed hash table (DHT) operations
- Record replication and discovery
- Event broadcasting and subscription

## 🎯 Use Cases

Netabase is perfect for:

- **Decentralized Applications**: Build apps without central servers
- **Local-First Software**: Offline-capable applications with sync
- **P2P Content Sharing**: Distributed content networks
- **IoT Networks**: Device-to-device data sharing
- **Collaborative Tools**: Real-time collaboration without servers
- **Research Projects**: Distributed systems research and prototyping

## 📊 Performance

### Query Performance
- **Primary Key Access**: O(log n)
- **Secondary Key Queries**: O(m) where m = matching records
- **Range Queries**: O(log n + m) for prefix searches
- **Custom Filters**: O(n) - use secondary keys when possible

### Network Performance
- **DHT Operations**: Dependent on network size and connectivity
- **Record Replication**: Automatic with configurable redundancy
- **Peer Discovery**: Efficient Kademlia-based routing

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

**Database Path Conflicts**
```rust
// ❌ Don't reuse paths in tests
let db = NetabaseSledDatabase::new_with_name("test_db")?;

// ✅ Use unique paths
let temp_dir = tempfile::TempDir::new()?;
let db_path = temp_dir.path().join("unique_test_db");
let db = NetabaseSledDatabase::new_with_name(&db_path.to_string_lossy())?;
```

**Network Timeouts**
```rust
// DHT operations may timeout in single-node setups
match timeout(Duration::from_secs(10), netabase.put_record(data)).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(_) => println!("Timeout - normal for single-node testing"),
}
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
```

### Debugging

Enable debug logging:
```rust
env_logger::Builder::from_default_env()
    .filter_level(log::LevelFilter::Debug)
    .init();
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

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
