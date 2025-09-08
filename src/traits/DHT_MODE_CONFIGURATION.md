# Kademlia DHT Mode Configuration in Netabase

## Overview

Netabase provides dynamic Kademlia DHT mode configuration that allows nodes to operate in different participation modes within the distributed hash table. This flexibility enables nodes to optimize their resource usage, network participation, and performance based on their capabilities and network conditions.

## DHT Modes

### 1. Server Mode (Default)

**Characteristics:**
- Full DHT participation
- Stores and serves records to other nodes
- Responds to DHT queries from peers
- Participates in DHT maintenance operations
- Provides content to the network

**Use Cases:**
- Full nodes with sufficient resources
- Always-on servers and infrastructure nodes
- Nodes with stable network connections
- High-capacity devices

**Resource Impact:**
- Higher CPU usage (query processing)
- Higher memory usage (record storage)
- Higher bandwidth usage (serving requests)
- Increased network traffic

```rust
// Configure server mode
let config = NetworkConfig::builder()
    .server_mode()
    .user_agent("My-DHT-Server/1.0.0")
    .build();
```

### 2. Client Mode

**Characteristics:**
- Limited DHT participation
- Can query and retrieve records
- Does not store records for others
- Does not respond to DHT queries
- Consumes content without providing

**Use Cases:**
- Mobile devices with limited resources
- Temporary or ephemeral nodes
- Nodes behind restrictive NATs/firewalls
- Resource-constrained environments
- Privacy-focused applications

**Resource Impact:**
- Lower CPU usage (no query processing)
- Lower memory usage (no record storage)
- Lower bandwidth usage (query-only)
- Reduced network overhead

```rust
// Configure client mode
let config = NetworkConfig::builder()
    .client_mode()
    .user_agent("My-DHT-Client/1.0.0")
    .max_connections_per_peer(2)
    .build();
```

### 3. Auto Mode

**Characteristics:**
- Dynamic mode switching based on conditions
- Adapts to changing network conditions
- Balances participation vs resource usage
- Intelligent decision making

**Use Cases:**
- Nodes with variable resources
- Applications with changing requirements
- Adaptive network protocols
- Load balancing scenarios

**Auto-Switch Triggers:**
- Peer count thresholds
- Available bandwidth
- Battery level (mobile devices)
- Network stability
- Resource availability

```rust
// Configure auto mode
let config = NetworkConfig::builder()
    .auto_mode()
    .user_agent("My-Adaptive-Node/1.0.0")
    .build();
```

## Configuration Examples

### Basic Configuration

```rust
use netabase::traits::network::{NetworkConfig, KademliaDhtMode};

// Simple server mode
let server_config = NetworkConfig::default(); // Server mode is default

// Simple client mode
let client_config = NetworkConfig::builder()
    .client_mode()
    .build();

// Auto mode with custom settings
let auto_config = NetworkConfig::builder()
    .auto_mode()
    .connection_timeout(Duration::from_secs(30))
    .max_connections_per_peer(8)
    .build();
```

### Production Configurations

```rust
// High-capacity server node
let server_config = NetworkConfig::builder()
    .server_mode()
    .user_agent("NewsNet-Server/1.0.0")
    .add_listen_address("/ip4/0.0.0.0/tcp/4001".parse()?)
    .add_listen_address("/ip4/0.0.0.0/udp/4001/quic-v1".parse()?)
    .max_connections_per_peer(32)
    .connection_timeout(Duration::from_secs(60))
    .kademlia(true)
    .gossipsub(true)
    .mdns(false) // Disable for production
    .build();

// Mobile client node
let mobile_config = NetworkConfig::builder()
    .client_mode()
    .user_agent("NewsNet-Mobile/1.0.0")
    .add_listen_address("/ip4/0.0.0.0/tcp/0".parse()?) // Random port
    .max_connections_per_peer(4)
    .connection_timeout(Duration::from_secs(15))
    .kademlia(true)
    .gossipsub(false) // Reduce bandwidth
    .mdns(true) // Good for local discovery
    .build();

// Edge node with adaptive behavior
let edge_config = NetworkConfig::builder()
    .auto_mode()
    .user_agent("NewsNet-Edge/1.0.0")
    .add_listen_address("/ip4/0.0.0.0/tcp/4001".parse()?)
    .max_connections_per_peer(16)
    .build();
```

## Dynamic Mode Switching

### Manual Mode Changes

```rust
// Initialize network
let mut network = create_network().await?;

// Switch to client mode
network.set_dht_mode(KademliaDhtMode::Client).await?;
println!("Switched to client mode");

// Check current mode
let current_mode = network.get_dht_mode()?;
println!("Current DHT mode: {:?}", current_mode);

// Check mode capabilities
if network.is_dht_server()? {
    println!("Node can store and serve records");
} else if network.is_dht_client()? {
    println!("Node can only query records");
}
```

### Automatic Mode Switching

```rust
// Enable auto mode
network.set_dht_mode(KademliaDhtMode::Auto).await?;

// Trigger automatic evaluation
let selected_mode = network.toggle_dht_mode_auto().await?;
println!("Auto mode selected: {:?}", selected_mode);

// Get statistics about mode switches
let stats = network.get_dht_mode_stats().await?;
println!("Mode switches: {}", stats.mode_switches_count);
println!("Time in server mode: {:?}", stats.time_in_server_mode);
```

### Force Mode Changes

```rust
// Force server mode regardless of conditions
network.force_dht_server_mode().await?;

// Force client mode for resource conservation
network.force_dht_client_mode().await?;
```

## Practical Implications

### Server Mode Operations

```rust
// In server mode, you can:

// Store records for the network
network.dht_put("my-key".to_string(), data).await?;

// Provide content
network.dht_start_providing("content-key").await?;

// Respond to queries (automatic)
// Other nodes can query records from this node
```

### Client Mode Limitations

```rust
// In client mode:

// Can query records
let result = network.dht_get("some-key").await?;

// Cannot provide records
match network.dht_start_providing("key").await {
    Err(NetworkError::ProtocolError { message }) => {
        println!("Expected: {}", message); // DHT is in client mode
    }
    _ => unreachable!(),
}
```

### Performance Monitoring

```rust
// Monitor DHT performance
let stats = network.get_dht_mode_stats().await?;
println!("Records stored: {}", stats.records_stored);
println!("Queries answered: {}", stats.queries_answered);

// Check network health
let health = network.health_check().await?;
println!("DHT Status: {:?}", health.dht_status);
println!("Current Mode: {:?}", health.dht_mode);
```

## Configuration via Commands

### Using the Command System

```rust
use netabase::network::event_messages::command_messages::*;

// Set DHT mode via command
let set_mode_cmd = NetabaseCommand::Network(NetworkCommand::SetDhtMode {
    mode: KademliaDhtMode::Client,
});
command_sender.send(set_mode_cmd)?;

// Query current mode
let query_cmd = NetabaseCommand::Network(NetworkCommand::GetDhtMode);
let (response_tx, response_rx) = oneshot::channel();
command_sender.send(CommandWithResponse {
    command: query_cmd,
    response_sender: response_tx,
})?;

match response_rx.await? {
    CommandResponse::Network(NetworkResponse::DhtMode(mode)) => {
        println!("Current DHT mode: {:?}", mode);
    }
    _ => {}
}
```

## Best Practices

### 1. Mode Selection Guidelines

**Use Server Mode When:**
- Node has stable network connection
- Sufficient CPU and memory resources
- Long-running or always-on deployment
- Contributing to network health is desired

**Use Client Mode When:**
- Resource constraints (mobile, IoT devices)
- Temporary or short-lived connections
- Privacy concerns (reduce network footprint)
- Behind restrictive network configurations

**Use Auto Mode When:**
- Variable network conditions
- Adaptive resource management needed
- Uncertain deployment environment
- Want automatic optimization

### 2. Configuration Tips

```rust
// Good: Configure based on deployment environment
let config = if is_mobile_device() {
    NetworkConfig::builder().client_mode()
} else if is_server_deployment() {
    NetworkConfig::builder().server_mode()
} else {
    NetworkConfig::builder().auto_mode()
}.build();

// Good: Monitor and adapt
tokio::spawn(async move {
    loop {
        let health = network.health_check().await?;
        if health.connected_peer_count < 3 {
            network.force_dht_client_mode().await?;
        } else if health.connected_peer_count > 10 {
            network.force_dht_server_mode().await?;
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
});
```

### 3. Monitoring and Observability

```rust
// Set up DHT mode monitoring
let mut event_receiver = network.event_receiver()?;
tokio::spawn(async move {
    while let Ok(event) = event_receiver.recv().await {
        match event {
            NetworkEvent::NetworkError { error } if error.to_string().contains("DHT mode") => {
                log::info!("DHT mode change detected: {}", error);
            }
            _ => {}
        }
    }
});

// Regular statistics collection
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(300));
    loop {
        interval.tick().await;
        let stats = network.get_dht_mode_stats().await?;
        log::info!("DHT stats: {:?}", stats);
    }
});
```

## Migration and Compatibility

### Upgrading Existing Configurations

```rust
// Old configuration (no DHT mode specified)
let old_config = NetworkConfig {
    // ... existing fields
    ..Default::default()
};

// New configuration (explicit server mode - same behavior)
let new_config = NetworkConfig::builder()
    .server_mode() // Explicitly set what was default
    // ... copy other fields
    .build();

// Or upgrade to auto mode for better adaptability
let adaptive_config = NetworkConfig::builder()
    .auto_mode()
    // ... copy other fields
    .build();
```

### Backward Compatibility

- Default mode remains `Server` to maintain existing behavior
- All existing APIs continue to work unchanged
- New mode-specific features are opt-in
- Configuration builders provide smooth migration path

## Troubleshooting

### Common Issues

**Issue: Records not being stored**
```rust
// Check if node is in client mode
if network.is_dht_client()? {
    println!("Client mode: cannot store records");
    network.set_dht_mode(KademliaDhtMode::Server).await?;
}
```

**Issue: High resource usage**
```rust
// Switch to client mode to reduce load
if system_resources_low() {
    network.set_dht_mode(KademliaDhtMode::Client).await?;
}
```

**Issue: Auto mode not switching**
```rust
// Check auto mode triggers
let stats = network.get_dht_mode_stats().await?;
println!("Auto triggers: {:?}", stats.auto_mode_triggers);

// Force evaluation
network.toggle_dht_mode_auto().await?;
```

### Debugging Commands

```rust
// Enable detailed DHT logging
env_logger::Builder::from_env(Env::default().default_filter_or("netabase=debug")).init();

// Monitor mode changes
let mut event_stream = network.event_receiver()?;
while let Ok(event) = event_stream.recv().await {
    if let NetworkEvent::NetworkError { error } = event {
        if error.to_string().contains("DHT mode") {
            log::debug!("DHT mode event: {}", error);
        }
    }
}
```

## Conclusion

Kademlia DHT mode configuration in Netabase provides powerful flexibility for optimizing network participation based on node capabilities and requirements. By choosing the appropriate mode and leveraging dynamic switching capabilities, applications can achieve optimal performance while contributing effectively to the distributed network.

The three-mode system (Server/Client/Auto) covers the full spectrum from full participation to minimal resource usage, with intelligent adaptation capabilities for changing conditions. Combined with comprehensive monitoring and configuration options, this system enables building robust, efficient, and adaptive P2P applications.