use libp2p::{PeerId, identity::Keypair, kad};
use netabase::{
    config::{BehaviourConfig, KadStoreConfig, NetabaseConfig},
    database::SledStoreConfig,
    network::behaviour::NetabaseBehaviour,
};

#[test]
fn test_memory_store_configuration() {
    let peer_id = PeerId::random();

    // Test default memory store configuration
    let config = NetabaseConfig::with_memory_store(peer_id)
        .build()
        .expect("Should create memory store config");

    match config.behaviour_config().store_config() {
        KadStoreConfig::MemoryStore {
            peer_id: store_peer_id,
            config,
        } => {
            assert_eq!(*store_peer_id, peer_id);
            assert!(config.is_none(), "Default config should be None");
        }
        _ => panic!("Expected MemoryStore configuration"),
    }
}

#[test]
fn test_memory_store_custom_configuration() {
    let peer_id = PeerId::random();
    let memory_config = kad::store::MemoryStoreConfig {
        max_records: 2048,
        max_value_bytes: 128 * 1024,
        max_providers_per_key: kad::store::MemoryStoreConfig::default().max_providers_per_key,
        max_provided_keys: kad::store::MemoryStoreConfig::default().max_provided_keys,
    };

    let config = NetabaseConfig::builder()
        .swarm_config(netabase::config::NetabaseSwarmConfig::default())
        .behaviour_config(
            BehaviourConfig::with_memory_store_config(peer_id, memory_config)
                .build()
                .expect("Should create custom memory config"),
        )
        .build()
        .expect("Should create complete config");

    match config.behaviour_config().store_config() {
        KadStoreConfig::MemoryStore {
            peer_id: store_peer_id,
            config,
        } => {
            assert_eq!(*store_peer_id, peer_id);
            assert!(config.is_some(), "Custom config should be Some");
        }
        _ => panic!("Expected MemoryStore configuration"),
    }
}

#[test]
fn test_sled_store_configuration() {
    let path = "./test_sled_database";

    // Test default sled store configuration
    let config = NetabaseConfig::with_sled_store(path)
        .build()
        .expect("Should create sled store config");

    match config.behaviour_config().store_config() {
        KadStoreConfig::SledStore {
            path: store_path,
            config,
        } => {
            assert_eq!(store_path, path);
            assert!(config.is_none(), "Default config should be None");
        }
        _ => panic!("Expected SledStore configuration"),
    }
}

#[test]
fn test_sled_store_custom_configuration() {
    let path = "./test_custom_sled_database";
    let custom_config = SledStoreConfig {
        max_records: 4096,
        max_value_bytes: 256 * 1024,
        max_provided_keys: 2048,
        max_providers_per_key: 50,
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
            assert_eq!(
                store_config.max_provided_keys,
                custom_config.max_provided_keys
            );
            assert_eq!(
                store_config.max_providers_per_key,
                custom_config.max_providers_per_key
            );
        }
        _ => panic!("Expected SledStore configuration with custom config"),
    }
}

#[test]
fn test_kad_store_config_builders() {
    let peer_id = PeerId::random();

    // Test memory store builders
    let memory_store = KadStoreConfig::memory_store(peer_id);
    match memory_store {
        KadStoreConfig::MemoryStore {
            peer_id: store_peer_id,
            config,
        } => {
            assert_eq!(store_peer_id, peer_id);
            assert!(config.is_none());
        }
        _ => panic!("Expected MemoryStore from memory_store builder"),
    }

    let memory_config = kad::store::MemoryStoreConfig {
        max_records: 1024,
        ..kad::store::MemoryStoreConfig::default()
    };
    let memory_store_custom = KadStoreConfig::memory_store_with_config(peer_id, memory_config);
    match memory_store_custom {
        KadStoreConfig::MemoryStore {
            peer_id: store_peer_id,
            config,
        } => {
            assert_eq!(store_peer_id, peer_id);
            assert!(config.is_some());
        }
        _ => panic!("Expected MemoryStore from memory_store_with_config builder"),
    }

    // Test sled store builders
    let sled_path = "./test_kad_sled";
    let sled_store = KadStoreConfig::sled_store(sled_path);
    match sled_store {
        KadStoreConfig::SledStore { path, config } => {
            assert_eq!(path, sled_path);
            assert!(config.is_none());
        }
        _ => panic!("Expected SledStore from sled_store builder"),
    }

    let sled_config = SledStoreConfig::default();
    let sled_store_custom = KadStoreConfig::sled_store_with_config(sled_path, sled_config);
    match sled_store_custom {
        KadStoreConfig::SledStore { path, config } => {
            assert_eq!(path, sled_path);
            assert!(config.is_some());
        }
        _ => panic!("Expected SledStore from sled_store_with_config builder"),
    }
}

#[tokio::test]
async fn test_behaviour_creation_with_different_stores() -> anyhow::Result<()> {
    let keypair = Keypair::generate_ed25519();
    let peer_id = PeerId::from_public_key(&keypair.public());

    // Test memory store behaviour creation
    let _memory_behaviour = NetabaseBehaviour::with_memory_store(&keypair, peer_id)?;

    // Test sled store behaviour creation
    let _sled_behaviour =
        NetabaseBehaviour::with_sled_store(&keypair, "./test_behaviour_database")?;

    // Test custom memory store behaviour creation
    let memory_config = kad::store::MemoryStoreConfig {
        max_records: 512,
        ..kad::store::MemoryStoreConfig::default()
    };
    let _custom_memory_behaviour =
        NetabaseBehaviour::with_memory_store_config(&keypair, peer_id, memory_config)?;

    // Test custom sled store behaviour creation
    let sled_config = SledStoreConfig {
        max_records: 1024,
        max_value_bytes: 64 * 1024,
        max_provided_keys: 512,
        max_providers_per_key: 20,
    };
    let _custom_sled_behaviour = NetabaseBehaviour::with_sled_store_config(
        &keypair,
        "./test_custom_behaviour_database",
        sled_config,
    )?;

    Ok(())
}

#[test]
fn test_behaviour_config_builders() {
    let peer_id = PeerId::random();

    // Test memory store behaviour config
    let memory_behaviour_config = BehaviourConfig::with_memory_store(peer_id)
        .protocol_version("/test/1.0.0".to_string())
        .build()
        .expect("Should build memory behaviour config");

    assert_eq!(memory_behaviour_config.protocol_version(), "/test/1.0.0");
    match memory_behaviour_config.store_config() {
        KadStoreConfig::MemoryStore { .. } => {}
        _ => panic!("Expected MemoryStore in behaviour config"),
    }

    // Test sled store behaviour config
    let sled_behaviour_config = BehaviourConfig::with_sled_store("./test_behaviour_db")
        .agent_version("TestAgent/1.0.0".to_string())
        .build()
        .expect("Should build sled behaviour config");

    assert_eq!(sled_behaviour_config.agent_version(), "TestAgent/1.0.0");
    match sled_behaviour_config.store_config() {
        KadStoreConfig::SledStore { .. } => {}
        _ => panic!("Expected SledStore in behaviour config"),
    }
}

#[test]
fn test_default_behaviour_config() {
    let default_config = netabase::config::DefaultBehaviourConfig::default();

    // Check default values
    assert_eq!(default_config.protocol_version(), "/p2p/newsnet/0.1.0");
    assert_eq!(default_config.agent_version(), "netabase/0.1.0");
    assert!(default_config.kad_config().is_none());
    assert!(default_config.identify_config().is_none());
    assert!(default_config.mdns_config().is_none());

    // Check default store config
    match default_config.store_config() {
        KadStoreConfig::SledStore { path, config } => {
            assert_eq!(path, "./database");
            assert!(config.is_none());
        }
        _ => panic!("Expected default SledStore configuration"),
    }
}
