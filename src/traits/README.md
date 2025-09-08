# Netabase Traits System

This module contains trait definitions that separate the database, network, configuration, and core functionality of Netabase into distinct concerns.

## Overview

The Netabase traits system is designed to:

- **Provide clear separation between different subsystems**
- **Enable easier testing through dependency injection**
- **Allow for different implementations of each subsystem**
- **Support extensibility and customization**
- **Facilitate modular development and maintenance**

## Architecture

The traits are organized into four main categories:

### Core Traits

- **[`NetabaseCore`](core.rs)** - The main interface that combines all subsystems
- **[`NetabaseCoreExt`](core.rs)** - Extension trait for advanced core operations
- **[`NetabaseTransaction`](core.rs)** - Trait for atomic transactions
- **[`NetabaseEventHandler`](core.rs)** - Event handling interface

### Database Traits

- **[`NetabaseDatabase`](database.rs)** - Core database operations and storage management
- **[`NetabaseDatabaseExt`](database.rs)** - Extension trait for advanced database operations
- **[`NetabaseDatabaseIterator`](database.rs)** - Trait for creating database iterators
- **[`DatabaseIterator`](database.rs)** - Iterator interface for streaming large datasets

### Network Traits

- **[`NetabaseNetwork`](network.rs)** - Core network operations and peer communication
- **[`NetabaseNetworkExt`](network.rs)** - Extension trait for advanced network operations
- **[`NetworkEventHandler`](network.rs)** - Network event handling interface

### Configuration Traits

- **[`NetabaseConfiguration`](configuration.rs)** - Configuration management and validation
- **[`NetabaseConfigurationExt`](configuration.rs)** - Extension trait for typed configuration access
- **[`ConfigurationValidator`](configuration.rs)** - Configuration validation interface
- **[`ConfigurationProvider`](configuration.rs)** - Configuration source providers

## Usage Examples

### Basic Usage with the Main Interface

```rust
use netabase::traits::prelude::*;
use netabase::netabase_trait::{NetabaseSchema, NetabaseSchemaKey};

async fn example<K, V>(mut netabase: impl NetabaseCore<K, V>) -> NetabaseResult<()>
where
    K: NetabaseSchemaKey,
    V: NetabaseSchema,
{
    // Initialize and start the system
    let config = NetabaseConfig::default();
    netabase.initialize(config).await?;
    netabase.start().await?;

    // High-level operations
    netabase.put(key, value).await?;
    let result = netabase.get(&key).await?;
    
    // Batch operations
    let entries = vec![(key1, value1), (key2, value2)];
    netabase.put_batch(entries).await?;
    
    // Network operations
    netabase.publish(key, value).await?;
    let peers = netabase.connected_peers().await?;
    
    // Health monitoring
    let health = netabase.health_check().await?;
    let stats = netabase.stats().await?;

    Ok(())
}
```

### Accessing Subsystems Directly

```rust
async fn subsystem_access<K, V>(mut netabase: impl NetabaseCore<K, V>) -> NetabaseResult<()>
where
    K: NetabaseSchemaKey,
    V: NetabaseSchema,
{
    // Access database subsystem
    let db_stats = netabase.database().stats().await?;
    netabase.database_mut().compact().await?;
    
    // Access network subsystem
    let network_stats = netabase.network().stats().await?;
    netabase.network_mut().bootstrap().await?;
    
    // Access configuration subsystem
    let config_value: String = netabase.configuration()
        .get("database.path")?
        .unwrap_or_else(|| "./default".to_string());

    Ok(())
}
```

### Dependency Injection for Testing

```rust
use netabase::traits::{NetabaseDatabase, NetabaseNetwork, NetabaseConfiguration};

// Mock implementations for testing
struct MockDatabase { /* ... */ }
struct MockNetwork { /* ... */ }
struct MockConfiguration { /* ... */ }

#[async_trait]
impl<K: NetabaseSchemaKey, V: NetabaseSchema> NetabaseDatabase<K, V> for MockDatabase {
    // Implement all required methods...
}

// Create a testable instance
async fn create_test_instance() -> impl NetabaseCore<MyKey, MyValue> {
    let mut netabase = ExampleNetabase::new();
    netabase
        .with_database(Box::new(MockDatabase::new()))
        .with_network(Box::new(MockNetwork::new()))
        .with_configuration(Box::new(MockConfiguration::new()))
}
```

### Event Handling

```rust
struct MyEventHandler {
    id: String,
}

#[async_trait]
impl<K: NetabaseSchemaKey, V: NetabaseSchema> NetabaseEventHandler<K, V> for MyEventHandler {
    fn id(&self) -> &str {
        &self.id
    }

    async fn handle_event(&mut self, event: NetabaseEvent<K, V>) -> NetabaseResult<()> {
        match event {
            NetabaseEvent::DataOperation { operation, success, duration } => {
                println!("Data operation completed: success={}, duration={:?}", success, duration);
            }
            NetabaseEvent::PeerActivity { peer_id, activity } => {
                println!("Peer {} activity: {:?}", peer_id, activity);
            }
            NetabaseEvent::HealthChanged { component, old_status, new_status } => {
                println!("Health changed for {}: {:?} -> {:?}", component, old_status, new_status);
            }
            _ => {}
        }
        Ok(())
    }

    fn interested_events(&self) -> Vec<String> {
        vec!["DataOperation".to_string(), "PeerActivity".to_string()]
    }
}

// Register the event handler
let handler = Box::new(MyEventHandler { id: "my_handler".to_string() });
netabase.register_event_handler(handler).await?;
```

### Configuration Management

```rust
use netabase::traits::configuration::*;

async fn configuration_example() -> ConfigurationResult<()> {
    let config = ConfigurationBuilder::new()
        .add_source(ConfigurationSource::File {
            path: "config.toml".to_string(),
            format: FileFormat::Toml,
        })
        .add_source(ConfigurationSource::Environment {
            prefix: Some("NETABASE_".to_string()),
        })
        .validation_level(ValidationLevel::Strict)
        .watch_changes(true)
        .auto_reload(true)
        .build()
        .await?;

    // Use the configuration
    let db_path: String = config.get_required("database.path")?;
    let port: u16 = config.get_or("network.port", 4001);
    
    Ok(())
}
```

### Database Operations

```rust
use netabase::traits::database::*;

async fn database_example<K, V>(mut db: impl NetabaseDatabase<K, V>) -> DatabaseResult<()>
where
    K: NetabaseSchemaKey,
    V: NetabaseSchema,
{
    // Initialize
    let config = DatabaseConfig::default();
    db.initialize(config).await?;
    
    // Basic operations
    db.put(key, value).await?;
    let result = db.get(&key).await?;
    let exists = db.contains_key(&key).await?;
    
    // Batch operations
    let entries = vec![(key1, value1), (key2, value2)];
    db.put_batch(entries).await?;
    
    // Advanced operations
    let keys = db.keys(Some(QueryOptions {
        limit: Some(100),
        offset: Some(0),
        ..Default::default()
    })).await?;
    
    // Transaction
    let mut tx = db.begin_transaction().await?;
    // ... perform operations ...
    db.commit_transaction(tx).await?;
    
    // Statistics and maintenance
    let stats = db.stats().await?;
    db.compact().await?;
    db.backup("./backup.db").await?;
    
    Ok(())
}
```

### Network Operations

```rust
use netabase::traits::network::*;

async fn network_example<K, V>(mut network: impl NetabaseNetwork<K, V>) -> NetworkResult<()>
where
    K: NetabaseSchemaKey,
    V: NetabaseSchema,
{
    // Initialize and start
    let config = NetworkConfig::default();
    network.initialize(config).await?;
    network.start().await?;
    
    // Connection management
    let peer_id = PeerId::random();
    let address = "/ip4/127.0.0.1/tcp/4001".parse()?;
    network.connect_peer(peer_id, address).await?;
    
    // Messaging
    let message = NetworkMessage::StoreRequest { key, value };
    network.send_message(&peer_id, message).await?;
    
    // Broadcasting
    let broadcast_options = BroadcastOptions {
        topic: Some("updates".to_string()),
        min_peers: Some(3),
        ..Default::default()
    };
    network.broadcast_message(message, broadcast_options).await?;
    
    // DHT operations
    network.dht_put("my_key".to_string(), b"my_value".to_vec()).await?;
    let dht_value = network.dht_get("my_key").await?;
    
    // Statistics
    let stats = network.stats().await?;
    let peers = network.connected_peers().await?;
    
    Ok(())
}
```

## Command System

The traits work with a comprehensive command system that allows for type-safe operations:

### Available Commands

- **System Commands**: `Initialize`, `Start`, `Stop`, `Shutdown`, `HealthCheck`
- **Database Commands**: `Put`, `Get`, `Delete`, `Batch operations`, `Transactions`
- **Network Commands**: `Connect`, `Disconnect`, `Send`, `Broadcast`, `DHT operations`
- **Configuration Commands**: `Load`, `Save`, `Get/Set values`, `Validation`

### Command Usage

```rust
use netabase::network::event_messages::command_messages::*;

// Create commands
let put_command = NetabaseCommand::Database(DatabaseCommand::Put { key, value });
let get_command = NetabaseCommand::Database(DatabaseCommand::Get { key });
let connect_command = NetabaseCommand::Network(NetworkCommand::ConnectPeer { 
    peer_id, 
    address 
});

// Commands with responses
let (tx, rx) = tokio::sync::oneshot::channel();
let command_with_response = CommandWithResponse {
    command: get_command,
    response_sender: tx,
};

// Send command and wait for response
command_sender.send(command_with_response)?;
let response = rx.await?;

match response {
    CommandResponse::Database(DatabaseResponse::GetResult(value)) => {
        println!("Retrieved value: {:?}", value);
    }
    _ => {}
}
```

## Error Handling

The traits use a comprehensive error system:

```rust
use netabase::traits::prelude::*;

async fn error_handling_example() {
    match netabase.get(&key).await {
        Ok(Some(value)) => println!("Found value: {:?}", value),
        Ok(None) => println!("Key not found"),
        Err(NetabaseError::Database { source }) => {
            match source {
                DatabaseError::KeyNotFound { key } => {
                    println!("Key not found: {}", key);
                }
                DatabaseError::StorageError { message } => {
                    println!("Storage error: {}", message);
                }
                _ => println!("Other database error: {}", source),
            }
        }
        Err(NetabaseError::Network { source }) => {
            println!("Network error: {}", source);
        }
        Err(e) => println!("Other error: {}", e),
    }
}
```

## Implementation Guidelines

When implementing these traits:

1. **Follow async patterns**: All I/O operations should be async
2. **Handle errors gracefully**: Use the provided error types
3. **Emit events**: Use event channels to notify of state changes
4. **Support configuration**: Make behavior configurable where appropriate
5. **Provide metrics**: Implement statistics and health reporting
6. **Enable testing**: Design for dependency injection

## Thread Safety

All traits are designed to be `Send + Sync` and can be used safely across threads. Internal state should be protected with appropriate synchronization primitives.

## Extensibility

The trait system supports extensibility through:

- **Extension traits** for adding new functionality without breaking existing code
- **Event handlers** for custom behavior
- **Configuration providers** for different configuration sources
- **Custom implementations** for specific use cases

See the [example implementations](example.rs) for complete working examples.