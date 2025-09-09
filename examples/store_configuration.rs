//! Example demonstrating different Kademlia store configurations
//!
//! This example shows how to configure Netabase with different store types:
//! - MemoryStore: In-memory storage for testing and temporary use
//! - SledStore: Persistent file-based storage
//! - Custom configurations for both store types

use libp2p::{PeerId, identity::Keypair, kad};
use netabase::{
    config::{BehaviourConfig, KadStoreConfig, NetabaseConfig, NetabaseSwarmConfig},
    database::SledStoreConfig,
    network::behaviour::NetabaseBehaviour,
};

fn main() -> anyhow::Result<()> {
    println!("=== Netabase Store Configuration Examples ===\n");

    // Generate a keypair for examples
    let keypair = Keypair::generate_ed25519();
    let peer_id = PeerId::from_public_key(&keypair.public());

    // Example 1: Memory Store (Default Configuration)
    println!("1. Creating configuration with MemoryStore (default):");
    let memory_config = NetabaseConfig::with_memory_store(peer_id)
        .build()
        .expect("Should create memory store config");

    println!("   ✓ Memory store configuration created successfully");
    match memory_config.behaviour_config().store_config() {
        KadStoreConfig::MemoryStore { peer_id, config } => {
            println!("   - Store type: MemoryStore");
            println!("   - Peer ID: {}", peer_id);
            println!("   - Using default config: {}", config.is_none());
        }
        _ => println!("   - Unexpected store type"),
    }

    // Example 2: Memory Store with Custom Configuration
    println!("\n2. Creating configuration with MemoryStore (custom config):");
    let memory_store_config = kad::store::MemoryStoreConfig {
        max_records: 2048,
        max_value_bytes: 128 * 1024,
        max_providers_per_key: kad::store::MemoryStoreConfig::default().max_providers_per_key,
        max_provided_keys: kad::store::MemoryStoreConfig::default().max_provided_keys,
    };

    let custom_memory_config = NetabaseConfig::builder()
        .swarm_config(NetabaseSwarmConfig::default())
        .behaviour_config(
            BehaviourConfig::with_memory_store_config(peer_id, memory_store_config)
                .build()
                .expect("Should create custom memory config"),
        )
        .build()
        .expect("Should create complete config");

    println!("   ✓ Custom memory store configuration created successfully");
    match custom_memory_config.behaviour_config().store_config() {
        KadStoreConfig::MemoryStore { peer_id, config } => {
            println!("   - Store type: MemoryStore");
            println!("   - Peer ID: {}", peer_id);
            println!("   - Using custom config: {}", config.is_some());
        }
        _ => println!("   - Unexpected store type"),
    }

    // Example 3: Sled Store (Default Configuration)
    println!("\n3. Creating configuration with SledStore (default):");
    let sled_config = NetabaseConfig::with_sled_store("./example_database")
        .build()
        .expect("Should create sled store config");

    println!("   ✓ Sled store configuration created successfully");
    match sled_config.behaviour_config().store_config() {
        KadStoreConfig::SledStore { path, config } => {
            println!("   - Store type: SledStore");
            println!("   - Database path: {}", path);
            println!("   - Using default config: {}", config.is_none());
        }
        _ => println!("   - Unexpected store type"),
    }

    // Example 4: Sled Store with Custom Configuration
    println!("\n4. Creating configuration with SledStore (custom config):");
    let sled_store_config = SledStoreConfig {
        max_records: 4096,
        max_value_bytes: 256 * 1024,
        max_provided_keys: 2048,
        max_providers_per_key: 50,
    };

    let custom_sled_config =
        NetabaseConfig::with_sled_store_config("./custom_database", sled_store_config)
            .build()
            .expect("Should create custom sled config");

    println!("   ✓ Custom sled store configuration created successfully");
    match custom_sled_config.behaviour_config().store_config() {
        KadStoreConfig::SledStore { path, config } => {
            println!("   - Store type: SledStore");
            println!("   - Database path: {}", path);
            println!("   - Using custom config: {}", config.is_some());
            if let Some(conf) = config {
                println!("   - Max records: {}", conf.max_records);
                println!("   - Max value bytes: {}", conf.max_value_bytes);
            }
        }
        _ => println!("   - Unexpected store type"),
    }

    // Example 5: Creating NetabaseBehaviour directly with different stores
    println!("\n5. Creating NetabaseBehaviour instances:");

    // Memory store behaviour
    let memory_behaviour = NetabaseBehaviour::with_memory_store(&keypair, peer_id)?;
    println!("   ✓ NetabaseBehaviour with MemoryStore created");

    // Sled store behaviour
    let sled_behaviour = NetabaseBehaviour::with_sled_store(&keypair, "./behaviour_database")?;
    println!("   ✓ NetabaseBehaviour with SledStore created");

    // Custom memory store behaviour with config
    let memory_config = kad::store::MemoryStoreConfig {
        max_records: 1024,
        ..kad::store::MemoryStoreConfig::default()
    };
    let custom_memory_behaviour =
        NetabaseBehaviour::with_memory_store_config(&keypair, peer_id, memory_config)?;
    println!("   ✓ NetabaseBehaviour with custom MemoryStore config created");

    // Custom sled store behaviour with config
    let custom_sled_config = SledStoreConfig {
        max_records: 8192,
        max_value_bytes: 512 * 1024,
        max_provided_keys: 4096,
        max_providers_per_key: 100,
    };
    let custom_sled_behaviour = NetabaseBehaviour::with_sled_store_config(
        &keypair,
        "./custom_behaviour_database",
        custom_sled_config,
    )?;
    println!("   ✓ NetabaseBehaviour with custom SledStore config created");

    // Example 6: Using KadStoreConfig directly
    println!("\n6. Using KadStoreConfig builders:");

    let memory_store_config = KadStoreConfig::memory_store(peer_id);
    println!("   ✓ KadStoreConfig::memory_store created");

    let memory_store_with_config = KadStoreConfig::memory_store_with_config(
        peer_id,
        kad::store::MemoryStoreConfig {
            max_records: 512,
            ..kad::store::MemoryStoreConfig::default()
        },
    );
    println!("   ✓ KadStoreConfig::memory_store_with_config created");

    let sled_store_config = KadStoreConfig::sled_store("./direct_database");
    println!("   ✓ KadStoreConfig::sled_store created");

    let sled_store_with_config = KadStoreConfig::sled_store_with_config(
        "./direct_custom_database",
        SledStoreConfig::default(),
    );
    println!("   ✓ KadStoreConfig::sled_store_with_config created");

    println!("\n=== All examples completed successfully! ===");
    println!("\nKey benefits of the new store configuration system:");
    println!("• No need to specify generic types when using builder methods");
    println!("• Support for MemoryStore, SledStore, and custom configurations");
    println!("• Easy-to-use builder pattern with sensible defaults");
    println!("• Type-safe configuration with compile-time guarantees");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_configuration() {
        let peer_id = PeerId::random();

        // Test default memory store
        let config = NetabaseConfig::with_memory_store(peer_id)
            .build()
            .expect("Should create memory store config");

        match config.behaviour_config().store_config() {
            KadStoreConfig::MemoryStore {
                peer_id: store_peer_id,
                config,
            } => {
                assert_eq!(*store_peer_id, peer_id);
                assert!(config.is_none()); // Default config
            }
            _ => panic!("Expected MemoryStore configuration"),
        }
    }

    #[test]
    fn test_sled_store_configuration() {
        let path = "./test_database";

        // Test default sled store
        let config = NetabaseConfig::with_sled_store(path)
            .build()
            .expect("Should create sled store config");

        match config.behaviour_config().store_config() {
            KadStoreConfig::SledStore {
                path: store_path,
                config,
            } => {
                assert_eq!(store_path, path);
                assert!(config.is_none()); // Default config
            }
            _ => panic!("Expected SledStore configuration"),
        }
    }

    #[test]
    fn test_custom_sled_store_configuration() {
        let path = "./custom_test_database";
        let custom_config = SledStoreConfig {
            max_records: 1024,
            max_value_bytes: 64 * 1024,
            max_provided_keys: 512,
            max_providers_per_key: 25,
        };

        let config = NetabaseConfig::with_sled_store_config(path, custom_config.clone())
            .build()
            .expect("Should create custom sled store config");

        match config.behaviour_config().store_config() {
            KadStoreConfig::SledStore {
                path: store_path,
                config: Some(store_config),
            } => {
                assert_eq!(store_path, path);
                assert_eq!(store_config.max_records, custom_config.max_records);
                assert_eq!(store_config.max_value_bytes, custom_config.max_value_bytes);
            }
            _ => panic!("Expected SledStore configuration with custom config"),
        }
    }

    #[test]
    fn test_behaviour_creation_with_different_stores() -> anyhow::Result<()> {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from_public_key(&keypair.public());

        // Test memory store behaviour creation
        let _memory_behaviour = NetabaseBehaviour::with_memory_store(&keypair, peer_id)?;

        // Test sled store behaviour creation
        let _sled_behaviour = NetabaseBehaviour::with_sled_store(&keypair, "./test_behaviour_db")?;

        Ok(())
    }
}
