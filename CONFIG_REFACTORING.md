# Netabase Configuration Refactoring

This document outlines the refactoring of the Netabase configuration system from a monolithic structure to a modular, component-based approach.

## Overview

The configuration system has been successfully refactored to use smaller, dedicated configuration structs for each aspect of libp2p networking: Swarm, Kademlia, Identify, and mDNS. This modular approach provides better separation of concerns, improved maintainability, and enhanced flexibility for users.

## Configuration Structure

### Individual Configuration Modules

#### 1. SwarmConfig
Handles libp2p swarm-specific configuration:

```rust
pub struct SwarmConfig {
    pub connection_idle_timeout: Duration,
    pub notify_handler_buffer_size: NonZero<usize>,
    pub per_connection_event_buffer_size: usize,
    pub dial_concurrency_factor: NonZero<u8>,
    pub substream_upgrade_protocol_override: Option<Version>,
    pub max_negotiating_inbound_streams: usize,
}
```

**Features:**
- Builder pattern with fluent API
- Sensible defaults based on libp2p recommendations
- Full configurability of connection and stream management

#### 2. KademliaConfig
Manages Kademlia DHT configuration:

```rust
pub struct KademliaConfig {
    pub replication_factor: usize,
    pub query_timeout: Duration,
}
```

**Features:**
- Configurable replication factor for data redundancy
- Adjustable query timeouts for network conditions
- Builder pattern for easy configuration

#### 3. IdentifyConfig
Controls the libp2p Identify protocol:

```rust
pub struct IdentifyConfig {
    pub agent_version: String,
    pub protocol_version: String,
    pub interval: Duration,
    pub push_listen_addr_updates: bool,
    pub cache_size: usize,
    pub hide_listen_addrs: bool,
}
```

**Features:**
- Customizable agent and protocol versions
- Configurable identification intervals
- Address update and caching options

#### 4. MdnsConfig
Manages mDNS discovery settings:

```rust
pub struct MdnsConfig {
    pub enabled: bool,
    pub ttl: Duration,
    pub query_interval: Duration,
    pub enable_ipv6: bool,
}
```

**Features:**
- Enable/disable mDNS discovery
- Configurable TTL and query intervals
- IPv6 support toggle

### Main Configuration Structure

The `NetabaseConfig` now encapsulates all individual configurations:

```rust
pub struct NetabaseConfig {
    pub storage_path: PathBuf,
    pub keypair_path: PathBuf,
    pub listen_addresses: Vec<Multiaddr>,
    pub bootstrap_addresses: Vec<Multiaddr>,
    pub swarm: SwarmConfig,
    pub kademlia: KademliaConfig,
    pub identify: IdentifyConfig,
    pub mdns: MdnsConfig,
}
```

## Usage Examples

### Basic Configuration
```rust
let config = NetabaseConfig::new()
    .with_storage_path(PathBuf::from("/tmp/netabase"))
    .with_kad_replication_factor(10)
    .with_mdns_enabled(true);
```

### Advanced Modular Configuration
```rust
let config = NetabaseConfig::new()
    .with_storage_path(PathBuf::from("/opt/netabase/data"))
    .add_listen_address("/ip4/0.0.0.0/tcp/0".parse()?)
    .add_bootstrap_address("/ip4/192.168.1.100/tcp/4001/p2p/12D3KooWExample".parse()?)
    .with_swarm_config(
        SwarmConfig::new()
            .with_connection_idle_timeout(Duration::from_secs(600))
            .with_dial_concurrency_factor(NonZero::new(4).unwrap())
            .with_notify_handler_buffer_size(NonZero::new(16).unwrap())
    )
    .with_kademlia_config(
        KademliaConfig::new()
            .with_replication_factor(20)
            .with_query_timeout(Duration::from_secs(120))
    )
    .with_identify_config(
        IdentifyConfig::new()
            .with_agent_version("my-custom-agent/1.0.0".to_string())
            .with_interval(Duration::from_secs(600))
    )
    .with_mdns_config(
        MdnsConfig::new()
            .with_enabled(false)
            .with_ttl(Duration::from_secs(300))
    );
```

### Using Individual Config Structs
```rust
let custom_swarm_config = SwarmConfig::new()
    .with_connection_idle_timeout(Duration::from_secs(180))
    .with_max_negotiating_inbound_streams(512);

let custom_kad_config = KademliaConfig::new()
    .with_replication_factor(25)
    .with_query_timeout(Duration::from_secs(30));

let config = NetabaseConfig::new()
    .with_swarm_config(custom_swarm_config)
    .with_kademlia_config(custom_kad_config);
```

## Backward Compatibility

Convenience methods are provided for backward compatibility with the old flat structure:

```rust
let config = NetabaseConfig::new()
    .with_kad_replication_factor(15)           // Updates kademlia.replication_factor
    .with_kad_query_timeout(Duration::from_secs(45))  // Updates kademlia.query_timeout
    .with_mdns_enabled(false)                  // Updates mdns.enabled
    .with_connection_idle_timeout(Duration::from_secs(20))  // Updates swarm.connection_idle_timeout
    .with_dial_concurrency_factor(NonZero::new(6).unwrap()); // Updates swarm.dial_concurrency_factor
```

## Implementation Details

### Key Changes Made

1. **Modular Structure**: Separated configuration concerns into dedicated structs
2. **Builder Patterns**: Each config struct implements a fluent builder API
3. **Type Safety**: Leveraged Rust's type system for compile-time validation
4. **Default Values**: Sensible defaults based on libp2p best practices
5. **Testing**: Comprehensive test suite covering all configuration scenarios

### Integration Points

1. **NetabaseBehaviour**: Updated to use the new modular configuration
2. **Swarm Generation**: Modified to apply individual config modules correctly
3. **Type Compatibility**: Fixed libp2p API compatibility issues (StreamProtocol, NonZero types)

### Testing

The refactoring includes comprehensive tests for:
- Individual config struct builders
- Default value validation
- Modular configuration composition
- Backward compatibility methods
- Configuration cloning and serialization

## Benefits

1. **Separation of Concerns**: Each config struct handles a specific aspect of networking
2. **Maintainability**: Easier to understand, modify, and extend individual components
3. **Flexibility**: Users can configure only what they need
4. **Type Safety**: Compile-time validation of configuration parameters
5. **Discoverability**: IDE auto-completion helps users find relevant configuration options
6. **Testing**: Individual components can be tested in isolation
7. **Future-Proofing**: Easy to add new libp2p protocols without affecting existing configuration

## Migration Guide

### From Old Flat Structure
```rust
// Old way
let config = NetabaseConfig {
    kad_replication_factor: 20,
    kad_query_timeout: Duration::from_secs(60),
    enable_mdns: true,
    connection_idle_timeout: Duration::from_secs(300),
    // ... other fields
};
```

### To New Modular Structure
```rust
// New way - using convenience methods
let config = NetabaseConfig::new()
    .with_kad_replication_factor(20)
    .with_kad_query_timeout(Duration::from_secs(60))
    .with_mdns_enabled(true)
    .with_connection_idle_timeout(Duration::from_secs(300));

// Or using explicit modular configuration
let config = NetabaseConfig::new()
    .with_kademlia_config(
        KademliaConfig::new()
            .with_replication_factor(20)
            .with_query_timeout(Duration::from_secs(60))
    )
    .with_mdns_config(MdnsConfig::new().with_enabled(true))
    .with_swarm_config(
        SwarmConfig::new()
            .with_connection_idle_timeout(Duration::from_secs(300))
    );
```

## Future Enhancements

1. **Configuration Validation**: Add validation methods to ensure configuration consistency
2. **Serialization**: Add serde support for configuration persistence
3. **Environment Variables**: Support loading configuration from environment variables
4. **Configuration Profiles**: Predefined configuration profiles for common use cases
5. **Hot Reloading**: Support for runtime configuration updates where possible
6. **Documentation**: Enhanced documentation with more examples and use cases

## Conclusion

The modular configuration refactoring successfully achieved the goals of improved maintainability, flexibility, and type safety while maintaining backward compatibility. The new structure provides a solid foundation for future enhancements and makes the Netabase configuration system more user-friendly and robust.