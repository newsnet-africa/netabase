//! Example demonstrating mDNS auto-connect functionality
//!
//! This example shows how to configure Netabase with mDNS auto-connect enabled.
//! When enabled, peers discovered via mDNS will be automatically added to the
//! Kademlia routing table, allowing for automatic peer discovery on the local network.
//!
//! Run this example with:
//! ```
//! cargo run --example mdns_auto_connect
//! ```
//!
//! To test the functionality, run multiple instances of this example on the same
//! local network. Peers should automatically discover each other via mDNS and
//! add each other to their Kademlia routing tables.

use bincode::{Decode, Encode};
use netabase::{
    Netabase,
    config::{DefaultBehaviourConfig, DefaultNetabaseConfig, KadStoreConfig, NetabaseSwarmConfig},
    netabase_trait::NetabaseSchema,
};
use netabase_macros::{NetabaseSchema, NetabaseSchemaKey};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(NetabaseSchema, Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
struct ExampleValue {
    #[key]
    pub id: String,
    pub data: String,
    pub timestamp: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("Starting mDNS Auto-Connect Example");
    println!("===================================");

    // Create a configuration with mDNS auto-connect enabled
    let config_with_auto_connect = DefaultNetabaseConfig::builder()
        .swarm_config(
            NetabaseSwarmConfig::builder()
                .mdns_enabled(true)
                .mdns_auto_connect(true) // Enable auto-connect
                .build()?,
        )
        .behaviour_config(
            DefaultBehaviourConfig::builder()
                .store_config(KadStoreConfig::sled_store("./example_db_auto_connect"))
                .build()?,
        )
        .build()?;

    println!("✓ Configuration created with mDNS auto-connect enabled");

    // Create Netabase instance
    let mut netabase: Netabase<ExampleValueKey, ExampleValue> =
        Netabase::new(config_with_auto_connect);
    println!("✓ Netabase instance created (swarm not started yet)");

    // Start the swarm
    netabase.start_swarm()?;
    println!("✓ Swarm started - now listening for mDNS discoveries");

    // Create some test data
    let test_value = ExampleValue {
        id: "test_key_001".to_string(),
        data: "Hello from mDNS auto-connect example!".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    };

    // Store the test data
    netabase.put(test_value.key(), test_value.clone()).await?;
    println!("✓ Stored test data in DHT");

    println!("\nWaiting for mDNS peer discovery...");
    println!("(Run multiple instances of this example to see auto-connect in action)");

    // Monitor events for a while to see mDNS discoveries
    let mut event_receiver = netabase.swarm_event_listener.resubscribe();

    let monitoring_duration = Duration::from_secs(30);
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < monitoring_duration {
        tokio::select! {
            event = event_receiver.recv() => {
                match event {
                    Ok(event) => {
                        // Log interesting events
                        match &event.0 {
                            libp2p::swarm::SwarmEvent::Behaviour(behaviour_event) => {
                                match behaviour_event {
                                    netabase::network::behaviour::NetabaseBehaviourEvent::Mdns(mdns_event) => {
                                        println!("🔍 mDNS Event: {:?}", mdns_event);
                                    }
                                    netabase::network::behaviour::NetabaseBehaviourEvent::Kad(kad_event) => {
                                        println!("🗺️  Kademlia Event: {:?}", kad_event);
                                    }
                                    _ => {}
                                }
                            }
                            libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                                println!("🤝 Connection established with peer: {}", peer_id);
                            }
                            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                                println!("🎧 Listening on: {}", address);
                            }
                            _ => {}
                        }
                    }
                    Err(_) => break,
                }
            }
            _ = sleep(Duration::from_secs(5)) => {
                // Periodically try to retrieve the test data
                match netabase.get(test_value.key()).await {
                    Ok(Some(retrieved_value)) => {
                        if retrieved_value == test_value {
                            println!("✓ Successfully retrieved test data from DHT");
                        }
                    }
                    Ok(None) => {
                        println!("⚠️  Test data not found in DHT (this is expected initially)");
                    }
                    Err(e) => {
                        println!("❌ Error retrieving test data: {}", e);
                    }
                }
            }
        }
    }

    println!("\nExample completed. Key points about mDNS auto-connect:");
    println!(
        "1. When mDNS discovers peers, they are automatically added to Kademlia routing table"
    );
    println!("2. This enables automatic peer discovery without manual configuration");
    println!(
        "3. Useful for local network scenarios where nodes should find each other automatically"
    );
    println!("4. Can be disabled by setting mdns_auto_connect to false in configuration");

    // Clean shutdown
    netabase.close().await;
    println!("✓ Netabase closed cleanly");

    Ok(())
}

// Comparison example showing the difference between auto-connect enabled and disabled
#[allow(dead_code)]
async fn comparison_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Comparison: Auto-Connect vs Manual ===");

    // Configuration WITHOUT auto-connect (default behavior)
    let _config_manual = DefaultNetabaseConfig::builder()
        .swarm_config(
            NetabaseSwarmConfig::builder()
                .mdns_enabled(true)
                .mdns_auto_connect(false) // Disabled - manual peer management required
                .build()?,
        )
        .behaviour_config(DefaultBehaviourConfig::default())
        .build()?;

    // Configuration WITH auto-connect
    let _config_auto = DefaultNetabaseConfig::builder()
        .swarm_config(
            NetabaseSwarmConfig::builder()
                .mdns_enabled(true)
                .mdns_auto_connect(true) // Enabled - automatic peer management
                .build()?,
        )
        .behaviour_config(DefaultBehaviourConfig::default())
        .build()?;

    println!(
        "Manual mode: mDNS discoveries are logged but peers are not automatically added to Kademlia"
    );
    println!("Auto mode: mDNS discoveries automatically add peers to Kademlia routing table");

    Ok(())
}
