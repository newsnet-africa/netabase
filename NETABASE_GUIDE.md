# Netabase Documentation & Developer Guide

Netabase is a distributed database system built on top of [sled](https://github.com/spacejam/sled) with [libp2p](https://libp2p.io/) integration for peer-to-peer networking. It provides a type-safe, macro-driven approach to defining database schemas and models with support for primary keys, secondary keys, and relational queries.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Core Concepts](#core-concepts)
3. [Macro Reference](#macro-reference)
4. [Database Operations](#database-operations)
5. [Distributed Features](#distributed-features)
6. [Advanced Usage](#advanced-usage)
7. [Best Practices](#best-practices)
8. [Examples](#examples)
9. [API Reference](#api-reference)

## Quick Start

### Basic Setup

Add Netabase to your `Cargo.toml`:

```toml
[dependencies]
netabase = { path = "path/to/netabase" }
netabase_store = { path = "path/to/netabase/netabase_store" }
netabase_macros = { path = "path/to/netabase/netabase_store/netabase_macros" }
bincode = { version = "2.0", features = ["derive", "serde"] }
serde = { version = "1.0", features = ["derive"] }
```

### Defining Your First Model

```rust
use netabase_macros::{NetabaseModel, netabase_schema_module};
use bincode::{Encode, Decode};
use serde::{Serialize, Deserialize};

#[netabase_schema_module(MySchema, MySchemaKeys)]
mod my_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(UserKey)]
    pub struct User {
        #[key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
        pub created_at: u64,
    }
}
```

### Basic Database Operations

```rust
use netabase_store::database::{NetabaseSledDatabase, NetabaseSledTree};
use my_schema::*;

async fn basic_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Create database
    let db = NetabaseSledDatabase::<MySchema>::new_with_name("my_database")?;
    let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;

    // Create a user
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: 1234567890,
    };

    // Insert user
    user_tree.insert(user.key(), user.clone())?;

    // Get user by primary key
    let retrieved_user = user_tree.get(user.key())?.unwrap();
    println!("Retrieved: {:?}", retrieved_user);

    Ok(())
}
```

## Core Concepts

### Models and Keys

**Models** are the core data structures in Netabase. Every model must have:
- A **primary key** field marked with `#[key]`
- Optional **secondary key** fields marked with `#[secondary_key]`
- The `NetabaseModel` derive macro

**Keys** are automatically generated type-safe identifiers that allow efficient querying and indexing.

### Schemas

**Schemas** are collections of related models organized into modules using the `#[netabase_schema_module]` attribute. They provide:
- Type-safe model collections
- Unified key types
- Network serialization support

### Trees

**Trees** are the storage structures that hold your data. Each model type gets its own tree, providing:
- CRUD operations
- Secondary key indexing
- Range queries
- Iteration support

## Macro Reference

### `#[derive(NetabaseModel)]`

The primary derive macro that generates all necessary traits and types for a database model.

**Required Attributes:**
- `#[key]` - Marks the primary key field
- `#[key_name(KeyTypeName)]` - Specifies the name for the generated key type

**Optional Attributes:**
- `#[secondary_key]` - Marks fields as secondary keys for indexing

**Generated Types:**
- `{ModelName}Key` - Enum containing primary and secondary key variants
- `{ModelName}PrimaryKey` - Newtype wrapper for the primary key value
- `{ModelName}SecondaryKeys` - Enum of all secondary key variants
- `{ModelName}Relations` - Enum for relational keys (if any)

**Example:**
```rust
#[derive(NetabaseModel, Clone, Debug, Serialize, Deserialize)]
#[key_name(PostKey)]
pub struct Post {
    #[key]
    pub id: u64,
    pub title: String,
    #[secondary_key]
    pub author_id: u64,
    #[secondary_key]
    pub published: bool,
    pub content: String,
}
```

### `#[netabase_schema_module]`

Attribute macro that transforms a module into a Netabase schema.

**Syntax:**
```rust
#[netabase_schema_module(SchemaName, SchemaKeysName)]
mod module_name {
    // Model definitions
}
```

**Generated Types:**
- `SchemaName` - Enum containing all models in the schema
- `SchemaKeysName` - Enum containing all key types in the schema

**Features:**
- Automatic `From` implementations between models and schema
- Network serialization support with libp2p
- Unified querying interface

### `#[derive(NetabaseModelKey)]`

Used for custom key types (advanced usage).

### Attribute Reference

| Attribute | Usage | Description |
|-----------|-------|-------------|
| `#[key]` | Field | Marks the primary key field |
| `#[secondary_key]` | Field | Marks secondary key fields |
| `#[key_name(Name)]` | Struct | Names the generated key type |
| `#[key_schema]` | Field | Advanced relational key marking |

## Database Operations

### CRUD Operations

```rust
use netabase_store::traits::NetabaseModel;

// CREATE
let user = User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() };
user_tree.insert(user.key(), user.clone())?;

// READ
let retrieved = user_tree.get(user.key())?.unwrap();

// UPDATE
let mut updated_user = retrieved;
updated_user.name = "Alice Smith".to_string();
user_tree.insert(updated_user.key(), updated_user)?;

// DELETE
let removed = user_tree.remove(user.key())?.unwrap();
```

### Secondary Key Queries

```rust
use netabase_store::traits::NetabaseSecondaryKeyQuery;

// Query by email
let users = user_tree.query_by_secondary_key(
    UserSecondaryKeys::EmailKey("alice@example.com".to_string())
)?;

// Query by boolean secondary key
let published_posts = post_tree.query_by_secondary_key(
    PostSecondaryKeys::PublishedKey(true)
)?;
```

### Advanced Queries

```rust
use netabase_store::traits::NetabaseAdvancedQuery;

// Filter with custom condition
let adults = user_tree.query_with_filter(|user| user.age >= 18)?;

// Count with condition
let count = user_tree.count_where(|user| user.active)?;

// Range queries
let range_results = user_tree.range_by_prefix(b"prefix")?;

// Batch operations
let batch_data = vec![
    (UserKey::Primary(UserPrimaryKey(1)), user1),
    (UserKey::Primary(UserPrimaryKey(2)), user2),
];
user_tree.batch_insert_with_indexing(batch_data)?;
```

### Tree Iteration

```rust
// Iterate over all entries
for result in user_tree.iter() {
    let (key, user) = result?;
    println!("User: {} ({})", user.name, user.email);
}

// Collect results
let all_users: Vec<(UserKey, User)> = user_tree.iter().collect_results()?;

// Filter during iteration
let active_users: Vec<User> = user_tree.iter()
    .filter_ok(|(_, user)| user.active)
    .values()
    .collect_results()?;
```

## Distributed Features

### Setting Up P2P Networking

```rust
use netabase::Netabase;

// Create Netabase instance
let mut netabase = Netabase::<MySchema>::new()?;

// Start the swarm
netabase.start_swarm().await?;

// Subscribe to network events
let mut receiver = netabase.subscribe_to_broadcasts();
```

### DHT Operations

```rust
// Put a record into the DHT
let user = User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() };
let result = netabase.put_record(user).await?;

// Get a record from the DHT
let key = UserKey::Primary(UserPrimaryKey(1));
let result = netabase.get_record(key).await?;

// Provider operations
let key = UserKey::Primary(UserPrimaryKey(1));
netabase.start_providing(key.clone()).await?;
let providers = netabase.get_providers(key.clone()).await?;
netabase.stop_providing(key).await?;
```

### Network Management

```rust
// Bootstrap to join the network
netabase.bootstrap().await?;

// Peer management
let peer_id = PeerId::random();
let address: Multiaddr = "/ip4/127.0.0.1/tcp/4001".parse()?;
netabase.add_address(peer_id, address.clone()).await?;
netabase.remove_address(peer_id, address).await?;

// DHT mode management
netabase.set_mode(Some(libp2p::kad::Mode::Server)).await?;
let current_mode = netabase.get_mode().await?;
```

## Advanced Usage

### Relational Models

```rust
#[derive(NetabaseModel, Clone, Debug, Serialize, Deserialize)]
#[key_name(CommentKey)]
pub struct Comment {
    #[key]
    pub id: u64,
    pub content: String,
    #[secondary_key]
    pub post_id: u64,  // Foreign key to Post
    #[secondary_key]
    pub author_id: u64, // Foreign key to User
}

// Query comments by post
let post_comments = comment_tree.query_by_secondary_key(
    CommentSecondaryKeys::Post_idKey(post.id)
)?;
```

### Custom Database Paths

```rust
// Create database with custom path
let netabase = Netabase::<MySchema>::new_with_path("./my_custom_db")?;

// For local database
let db = NetabaseSledDatabase::<MySchema>::new_with_name("./local_db")?;
```

### Error Handling

```rust
use netabase_store::errors::NetabaseStoreError;

match user_tree.get(user_key) {
    Ok(Some(user)) => println!("Found user: {:?}", user),
    Ok(None) => println!("User not found"),
    Err(NetabaseStoreError::Database(e)) => eprintln!("Database error: {}", e),
    Err(NetabaseStoreError::Serialization(e)) => eprintln!("Serialization error: {}", e),
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Working with Multiple Models

```rust
#[netabase_schema_module(BlogSchema, BlogSchemaKeys)]
mod blog_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Debug, Serialize, Deserialize)]
    #[key_name(UserKey)]
    pub struct User {
        #[key] pub id: u64,
        pub name: String,
        #[secondary_key] pub email: String,
    }

    #[derive(NetabaseModel, Clone, Debug, Serialize, Deserialize)]
    #[key_name(PostKey)]
    pub struct Post {
        #[key] pub id: u64,
        pub title: String,
        #[secondary_key] pub author_id: u64,
    }

    #[derive(NetabaseModel, Clone, Debug, Serialize, Deserialize)]
    #[key_name(CommentKey)]
    pub struct Comment {
        #[key] pub id: u64,
        pub content: String,
        #[secondary_key] pub post_id: u64,
        #[secondary_key] pub author_id: u64,
    }
}

use blog_schema::*;

// Work with multiple trees
let db = NetabaseSledDatabase::<BlogSchema>::new()?;
let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree()?;
let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;
let comment_tree: NetabaseSledTree<Comment, CommentKey> = db.get_main_tree()?;
```

## Best Practices

### 1. Model Design

**Do:**
- Use meaningful, descriptive field names
- Keep primary keys simple (prefer integers or UUIDs)
- Index frequently queried fields with `#[secondary_key]`
- Use appropriate data types (u64 for IDs, String for text, etc.)

**Don't:**
- Make complex composite primary keys
- Over-index (too many secondary keys can impact performance)
- Use mutable references in model fields

### 2. Key Naming

```rust
// Good: Descriptive key names
#[key_name(UserAccountKey)]
pub struct UserAccount { ... }

#[key_name(BlogPostKey)]
pub struct BlogPost { ... }

// Avoid: Generic or unclear names
#[key_name(DataKey)]
pub struct Data { ... }
```

### 3. Schema Organization

```rust
// Good: Logical grouping
#[netabase_schema_module(UserManagementSchema, UserManagementKeys)]
mod user_management {
    pub struct User { ... }
    pub struct UserSession { ... }
    pub struct UserPreferences { ... }
}

#[netabase_schema_module(ContentSchema, ContentKeys)]
mod content {
    pub struct Post { ... }
    pub struct Comment { ... }
    pub struct Tag { ... }
}
```

### 4. Error Handling

```rust
// Good: Specific error handling
match operation_result {
    Ok(value) => handle_success(value),
    Err(NetabaseStoreError::Database(e)) => handle_db_error(e),
    Err(NetabaseStoreError::Serialization(e)) => handle_serialization_error(e),
    Err(e) => handle_generic_error(e),
}

// Good: Propagate errors with context
fn create_user(name: &str) -> Result<User, Box<dyn std::error::Error>> {
    let user = User::new(name)?;
    user_tree.insert(user.key(), user.clone())
        .map_err(|e| format!("Failed to create user '{}': {}", name, e))?;
    Ok(user)
}
```

### 5. Testing

```rust
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> (NetabaseSledDatabase<MySchema>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let db = NetabaseSledDatabase::new_with_name(&db_path.to_string_lossy()).unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_user_creation() {
        let (db, _temp_dir) = create_test_db();
        let user_tree: NetabaseSledTree<User, UserKey> = db.get_main_tree().unwrap();
        
        // Test implementation
    }
}
```

## Examples

### Example 1: Simple Blog System

```rust
use netabase_macros::{NetabaseModel, netabase_schema_module};
use bincode::{Encode, Decode};
use serde::{Serialize, Deserialize};

#[netabase_schema_module(BlogSchema, BlogKeys)]
mod blog {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(AuthorKey)]
    pub struct Author {
        #[key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
        pub bio: String,
        pub created_at: u64,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(PostKey)]
    pub struct Post {
        #[key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
        #[secondary_key]
        pub published: bool,
        #[secondary_key]
        pub category: String,
        pub created_at: u64,
    }
}

use blog::*;

async fn blog_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create database
    let db = NetabaseSledDatabase::<BlogSchema>::new_with_name("blog_db")?;
    let author_tree: NetabaseSledTree<Author, AuthorKey> = db.get_main_tree()?;
    let post_tree: NetabaseSledTree<Post, PostKey> = db.get_main_tree()?;

    // Create an author
    let author = Author {
        id: 1,
        name: "Alice Johnson".to_string(),
        email: "alice@blog.com".to_string(),
        bio: "Tech blogger and developer".to_string(),
        created_at: chrono::Utc::now().timestamp() as u64,
    };
    author_tree.insert(author.key(), author.clone())?;

    // Create posts
    let post1 = Post {
        id: 1,
        title: "Getting Started with Rust".to_string(),
        content: "Rust is a systems programming language...".to_string(),
        author_id: author.id,
        published: true,
        category: "Programming".to_string(),
        created_at: chrono::Utc::now().timestamp() as u64,
    };

    let post2 = Post {
        id: 2,
        title: "Advanced Rust Patterns".to_string(),
        content: "In this post, we'll explore...".to_string(),
        author_id: author.id,
        published: false,
        category: "Programming".to_string(),
        created_at: chrono::Utc::now().timestamp() as u64,
    };

    post_tree.insert(post1.key(), post1)?;
    post_tree.insert(post2.key(), post2)?;

    // Query published posts
    let published_posts = post_tree.query_by_secondary_key(
        PostSecondaryKeys::PublishedKey(true)
    )?;
    println!("Published posts: {}", published_posts.len());

    // Query posts by author
    let author_posts = post_tree.query_by_secondary_key(
        PostSecondaryKeys::Author_idKey(author.id)
    )?;
    println!("Posts by {}: {}", author.name, author_posts.len());

    // Query posts by category
    let programming_posts = post_tree.query_by_secondary_key(
        PostSecondaryKeys::CategoryKey("Programming".to_string())
    )?;
    println!("Programming posts: {}", programming_posts.len());

    Ok(())
}
```

### Example 2: Distributed Chat System

```rust
use netabase::Netabase;
use tokio::time::{interval, Duration};

#[netabase_schema_module(ChatSchema, ChatKeys)]
mod chat {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(UserKey)]
    pub struct User {
        #[key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub online: bool,
        pub last_seen: u64,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(MessageKey)]
    pub struct Message {
        #[key]
        pub id: u64,
        pub content: String,
        #[secondary_key]
        pub sender_id: u64,
        #[secondary_key]
        pub channel: String,
        pub timestamp: u64,
    }
}

use chat::*;

async fn chat_example() -> Result<(), Box<dyn std::error::Error>> {
    // Setup distributed chat node
    let mut netabase = Netabase::<ChatSchema>::new_with_path("./chat_node")?;
    netabase.start_swarm().await?;

    // Subscribe to network events
    let mut receiver = netabase.subscribe_to_broadcasts();

    // Create user and announce presence
    let user = User {
        id: 1,
        username: "alice".to_string(),
        online: true,
        last_seen: chrono::Utc::now().timestamp() as u64,
    };

    // Put user record in DHT
    netabase.put_record(user.clone()).await?;
    println!("User {} is now online", user.username);

    // Send a message
    let message = Message {
        id: 1,
        content: "Hello, distributed world!".to_string(),
        sender_id: user.id,
        channel: "general".to_string(),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    netabase.put_record(message.clone()).await?;
    println!("Message sent: {}", message.content);

    // Listen for network events
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            println!("Network event: {:?}", event);
        }
    });

    // Keep the node running
    let mut heartbeat = interval(Duration::from_secs(30));
    loop {
        heartbeat.tick().await;
        
        // Update user's last_seen timestamp
        let mut updated_user = user.clone();
        updated_user.last_seen = chrono::Utc::now().timestamp() as u64;
        
        if let Err(e) = netabase.put_record(updated_user).await {
            eprintln!("Failed to update presence: {}", e);
        }
    }
}
```

## API Reference

### Traits

#### `NetabaseModel`
Core trait for all database models.

**Associated Types:**
- `Key` - The key type for this model
- `RelationsDiscriminants` - Enum of relation types

**Methods:**
- `key(&self) -> Self::Key` - Get the primary key
- `tree_name() -> &'static str` - Get the tree name
- `secondary_keys(&self) -> Vec<Self::SecondaryKeys>` - Get secondary keys
- `relations(&self) -> Vec<Self::Relations>` - Get relational keys

#### `NetabaseSchema`
Trait for schema enums containing multiple models.

**Associated Types:**
- `SchemaDiscriminants` - Enum variants
- `Keys` - Union of all key types

**Methods:**
- `keys(&self) -> Self::Keys` - Extract keys from schema
- `to_ivec(&self) -> Result<sled::IVec>` - Serialize for storage
- `from_ivec(ivec: sled::IVec) -> Result<Self>` - Deserialize from storage

#### `NetabaseSecondaryKeyQuery`
Provides secondary key querying capabilities.

**Methods:**
- `query_by_secondary_key<SK>(&self, key: SK) -> Result<Vec<M>>` - Query by secondary key
- `get_secondary_key_values(&self, key_name: &str) -> Result<Vec<sled::IVec>>` - Get all values for a secondary key
- `create_secondary_key_index<M, K, SK>(&self, key_name: &str) -> Result<()>` - Create index
- `remove_secondary_key_index<M, K, SK>(&self, key_name: &str) -> Result<()>` - Remove index

#### `NetabaseAdvancedQuery`
Advanced querying capabilities.

**Methods:**
- `query_with_filter<F>(&self, filter: F) -> Result<Vec<(K, M)>>` - Custom filter queries
- `count_where<F>(&self, predicate: F) -> Result<usize>` - Count with condition
- `range_by_prefix(&self, prefix: &[u8]) -> Result<Vec<(K, M)>>` - Range queries
- `batch_insert_with_indexing(&self, items: Vec<(K, M)>) -> Result<()>` - Batch operations

### Types

#### `NetabaseSledDatabase<S: NetabaseSchema>`
Main database type managing multiple trees.

**Methods:**
- `new() -> Result<Self>` - Create with default path
- `new_with_name(name: &str) -> Result<Self>` - Create with custom path
- `get_main_tree<M, K>() -> Result<NetabaseSledTree<M, K>>` - Get tree for model type

#### `NetabaseSledTree<M: NetabaseModel, K: NetabaseModelKey>`
Tree storage for a specific model type.

**Methods:**
- `insert(&self, key: K, value: M) -> Result<Option<M>>` - Insert/update
- `get(&self, key: K) -> Result<Option<M>>` - Retrieve by key
- `remove(&self, key: K) -> Result<Option<M>>` - Remove by key
- `contains_key(&self, key: K) -> Result<bool>` - Check existence
- `len(&self) -> usize` - Get count
- `is_empty(&self) -> bool` - Check if empty
- `iter(&self) -> NetabaseIter<K, M>` - Iterate over all entries
- `clear(&self) -> Result<()>` - Remove all entries

#### `Netabase<S: NetabaseSchema>`
Distributed database instance with P2P networking.

**Methods:**
- `new() -> Result<Self>` - Create instance
- `new_with_path<P: AsRef<Path>>(path: P) -> Result<Self>` - Create with custom path
- `start_swarm(&mut self) -> Result<()>` - Start P2P networking
- `stop_swarm(&mut self) -> Result<()>` - Stop P2P networking
- `put_record<M: NetabaseModel>(&self, model: M) -> Result<QueryResult>` - Store in DHT
- `get_record<K: NetabaseModelKey>(&self, key: K) -> Result<QueryResult>` - Retrieve from DHT
- `subscribe_to_broadcasts(&self) -> broadcast::Receiver<NetabaseSwarmEvent<S>>` - Subscribe to events

## Troubleshooting

### Common Issues

1. **"NetabaseModel requires a field marked with #[key]"**
   - Ensure exactly one field has the `#[key]` attribute
   - The key field must be a simple type (u64, String, etc.)

2. **Database path conflicts**
   - Use unique database paths for each test/instance
   - Use `tempfile::TempDir` for tests

3. **Serialization errors**
   - Ensure all model fields implement `Serialize` and `Deserialize`
   - Add required derives: `Encode`, `Decode` for bincode

4. **Network timeout issues**
   - DHT operations may timeout in single-node setups
   - Bootstrap with known peers for production use

### Performance Tips

1. **Indexing Strategy**
   - Only add `#[secondary_key]` to frequently queried fields
   - Consider the trade-off between query speed and write performance

2. **Batch Operations**
   - Use `batch_insert_with_indexing` for bulk data operations
   - Group related operations together

3. **Memory Usage**
   - Sled databases are memory-mapped
   - Monitor memory usage with large datasets
   - Use range queries instead of loading all data

### Debugging

Enable detailed logging:

```rust
env_logger::Builder::from_default_env()
    .filter_level(log::LevelFilter::Debug)
    .init();
```

Use the debug discriminants test to verify macro generation:

```bash
cargo test debug_discriminants -- --nocapture
```

## Contributing

See the main repository for contribution guidelines. When adding new features:

1. Update macro tests in `netabase_store/tests/`
2. Add integration tests in `netabase/tests/`
3. Update this documentation
4. Ensure all tests pass with `cargo test --test-threads=1`

## License

See the LICENSE file in the repository root.