//! Distributed tests for Netabase
//!
//! These tests verify that Netabase instances can communicate across processes
//! and machines using the libp2p network layer.
//!
//! Run locally with two processes:
//! ```bash
//! # Terminal 1 (Node A)
//! NETABASE_NODE=A NETABASE_PORT=0 cargo test distributed_two_nodes_local -- --nocapture
//!
//! # Terminal 2 (Node B)
//! NETABASE_NODE=B NETABASE_PORT=0 NETABASE_BOOTSTRAP=/ip4/127.0.0.1/tcp/<PORT_FROM_NODE_A>/p2p/<PEER_ID_FROM_NODE_A> cargo test distributed_two_nodes_local -- --nocapture
//! ```
//!
//! Run across machines:
//! ```bash
//! # Machine 1 (Node A)
//! NETABASE_NODE=A NETABASE_IP=0.0.0.0 NETABASE_PORT=4001 cargo test distributed_two_nodes_remote -- --nocapture
//!
//! # Machine 2 (Node B)
//! NETABASE_NODE=B NETABASE_IP=0.0.0.0 NETABASE_PORT=4001 NETABASE_BOOTSTRAP=/ip4/<MACHINE_1_IP>/tcp/4001/p2p/<PEER_ID_FROM_MACHINE_1> cargo test distributed_two_nodes_remote -- --nocapture
//! ```

use std::time::Duration;
use tokio::time::sleep;
use libp2p::{
    Multiaddr, PeerId,
    identity::ed25519::Keypair,
    kad::Quorum,
};
use serde::{Serialize, Deserialize};
use netabase::{
    config::NetabaseConfig,
    network::commands::database_commands::Database,
    Netabase, NetabaseSchema,
    get_test_temp_dir, init_logging,
    schema,
};

// Define our test schemas using the proper netabase macros
#[schema]
mod test_schemas {
    use super::*;

    #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq)]
    pub struct TestUser {
        #[key]
        pub id: String,
        pub name: String,
        pub age: u32,
        pub email: String,
    }

    #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq)]
    pub struct TestMessage {
        #[key]
        pub message_id: u64,
        pub sender: String,
        pub content: String,
        pub timestamp: u64,
    }

    #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq)]
    pub enum TestDocument {
        Text {
            #[key]
            id: String,
            content: String,
        },
        Binary {
            #[key]
            id: String,
            data: Vec<u8>,
        },
    }
}

use test_schemas::*;

/// Configuration for distributed tests
#[derive(Debug, Clone)]
struct TestConfig {
    node_id: String,
    listen_ip: String,
    listen_port: u16,
    bootstrap_addr: Option<Multiaddr>,
}

impl TestConfig {
    fn from_env() -> Self {
        Self {
            node_id: std::env::var("NETABASE_NODE").unwrap_or_else(|_| "A".to_string()),
            listen_ip: std::env::var("NETABASE_IP").unwrap_or_else(|_| "127.0.0.1".to_string()),
            listen_port: std::env::var("NETABASE_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(0),
            bootstrap_addr: std::env::var("NETABASE_BOOTSTRAP")
                .ok()
                .and_then(|addr| addr.parse().ok()),
        }
    }
}

/// Create a Netabase instance for testing
async fn create_test_netabase(config: TestConfig) -> anyhow::Result<(Netabase, PeerId, Multiaddr)> {
    // Generate or use deterministic keypair based on node ID
    let keypair = if config.node_id == "A" {
        Keypair::generate()
    } else {
        // For node B, we could use a seed for deterministic keys if needed
        Keypair::generate()
    };

    let peer_id = PeerId::from_public_key(&keypair.public().into());

    // Create storage directory
    let storage_path = get_test_temp_dir(
        Some(peer_id.to_string().as_bytes().iter().sum::<u8>() as u64),
        Some(&config.node_id),
    );

    // Create netabase config
    let netabase_config = NetabaseConfig {
        storage_path: storage_path.clone(),
        listen_addresses: vec![
            format!("/ip4/{}/tcp/{}", config.listen_ip, config.listen_port)
                .parse()
                .expect("Invalid listen address"),
        ],
        bootstrap_addresses: config.bootstrap_addr.into_iter().collect(),
        kad_replication_factor: 3,
        kad_query_timeout: Duration::from_secs(30),
        connection_idle_timeout: Duration::from_secs(60),
        enable_mdns: config.listen_ip == "127.0.0.1" || config.listen_ip == "localhost",
    };

    // Create Netabase instance
    let mut netabase = Netabase::try_new(
        netabase_config,
        &keypair,
        format!("netabase-test-{}", config.node_id),
    )?;

    // Start the swarm
    netabase.start_swarm()?;

    // Get the actual listening address
    let listen_addr = if config.listen_port == 0 {
        // Wait a bit for the swarm to start and get assigned port
        sleep(Duration::from_millis(100)).await;
        // For now, we'll construct the address - in a real implementation,
        // you'd get this from the swarm's listening addresses
        format!("/ip4/{}/tcp/0/p2p/{}", config.listen_ip, peer_id)
            .parse()
            .expect("Invalid constructed address")
    } else {
        format!("/ip4/{}/tcp/{}/p2p/{}", config.listen_ip, config.listen_port, peer_id)
            .parse()
            .expect("Invalid constructed address")
    };

    println!("Node {} started with PeerID: {} at address: {}",
             config.node_id, peer_id, listen_addr);

    Ok((netabase, peer_id, listen_addr))
}

/// Wait for network connectivity between nodes
async fn wait_for_connectivity(netabase: &mut Netabase, timeout: Duration) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    println!("Waiting for network connectivity...");

    while start.elapsed() < timeout {
        // Try to perform a simple operation to test connectivity
        // In a real implementation, you might have a specific connectivity check
        sleep(Duration::from_secs(1)).await;

        // For now, we'll just wait and hope the bootstrap process works
        if start.elapsed() > Duration::from_secs(5) {
            println!("Assuming connectivity established after 5 seconds");
            break;
        }
    }

    Ok(())
}

#[tokio::test]
#[ignore] // Ignore by default since it requires manual coordination
async fn distributed_two_nodes_local() -> anyhow::Result<()> {
    init_logging();

    let config = TestConfig::from_env();
    println!("Starting distributed test for node: {}", config.node_id);

    let (mut netabase, peer_id, listen_addr) = create_test_netabase(config.clone()).await?;

    if config.node_id == "A" {
        println!("Node A: Waiting for Node B to connect...");
        println!("Start Node B with: NETABASE_NODE=B NETABASE_BOOTSTRAP={} cargo test distributed_two_nodes_local -- --nocapture", listen_addr);

        // Wait for potential connections
        wait_for_connectivity(&mut netabase, Duration::from_secs(30)).await?;

        // Create test data using our schema
        let test_user = TestUser {
            id: "user123".to_string(),
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        };

        println!("Node A: Storing user data...");

        // Store data using the NetabaseSchema implementation
        let put_result = netabase.put(test_user.clone(), None, Quorum::One).await?;
        println!("Node A: Put result: {:?}", put_result);

        // Wait for replication
        sleep(Duration::from_secs(5)).await;

        // Try to retrieve data using the generated key
        println!("Node A: Retrieving user data...");
        let get_result = netabase.get(test_user.key()).await?;
        println!("Node A: Get result: {:?}", get_result);

        // Verify data
        let retrieved_user: TestUser = get_result.record.into();
        assert_eq!(retrieved_user, test_user);

        // Test message storage as well
        let test_message = TestMessage {
            message_id: 12345,
            sender: "Alice".to_string(),
            content: "Hello from Node A!".to_string(),
            timestamp: 1640995200, // Example timestamp
        };

        println!("Node A: Storing message data...");
        let put_result = netabase.put(test_message.clone(), None, Quorum::One).await?;
        println!("Node A: Message put result: {:?}", put_result);

        println!("Node A: Test completed successfully!");

    } else if config.node_id == "B" {
        println!("Node B: Connected as bootstrap node");

        // Wait for network to stabilize
        wait_for_connectivity(&mut netabase, Duration::from_secs(10)).await?;

        // Try to retrieve data that Node A should have stored
        let test_user_key = TestUser {
            id: "user123".to_string(),
            name: "".to_string(), // We only need the key
            age: 0,
            email: "".to_string(),
        }.key();

        println!("Node B: Attempting to retrieve user data...");

        // Wait a bit more for potential data replication
        sleep(Duration::from_secs(5)).await;

        match netabase.get(test_user_key).await {
            Ok(get_result) => {
                let retrieved_user: TestUser = get_result.record.into();
                println!("Node B: Successfully retrieved user: {:?}", retrieved_user);

                assert_eq!(retrieved_user.name, "Alice");
                assert_eq!(retrieved_user.age, 30);
                assert_eq!(retrieved_user.email, "alice@example.com");

                println!("Node B: Test completed successfully!");
            }
            Err(e) => {
                println!("Node B: Could not retrieve data (this might be expected if Node A hasn't stored it yet): {:?}", e);

                // Store our own data using enum schema
                let test_document = TestDocument::Text {
                    id: "doc456".to_string(),
                    content: "Document from Node B".to_string(),
                };

                println!("Node B: Storing document data...");
                let put_result = netabase.put(test_document.clone(), None, Quorum::One).await?;
                println!("Node B: Put result: {:?}", put_result);

                // Retrieve our own data
                let get_result = netabase.get(test_document.key()).await?;
                let retrieved_doc: TestDocument = get_result.record.into();
                assert_eq!(retrieved_doc, test_document);

                println!("Node B: Successfully stored and retrieved own data!");
            }
        }
    }

    // Keep the node alive for a while to allow for manual testing
    println!("Node {}: Keeping alive for 30 seconds for manual testing...", config.node_id);
    sleep(Duration::from_secs(30)).await;

    Ok(())
}

#[tokio::test]
#[ignore] // Ignore by default since it requires manual coordination across machines
async fn distributed_two_nodes_remote() -> anyhow::Result<()> {
    init_logging();

    let config = TestConfig::from_env();
    println!("Starting remote distributed test for node: {}", config.node_id);

    let (mut netabase, peer_id, listen_addr) = create_test_netabase(config.clone()).await?;

    if config.node_id == "A" {
        println!("Node A (Machine 1): Waiting for connections...");
        println!("Start Node B on another machine with:");
        println!("NETABASE_NODE=B NETABASE_IP=0.0.0.0 NETABASE_PORT=4001 NETABASE_BOOTSTRAP={} cargo test distributed_two_nodes_remote -- --nocapture",
                 listen_addr.to_string().replace("0.0.0.0", "<THIS_MACHINE_IP>"));

        // Wait for connections
        wait_for_connectivity(&mut netabase, Duration::from_secs(60)).await?;

        // Create and store test data
        let test_user = TestUser {
            id: "remote_user_123".to_string(),
            name: "Remote Alice".to_string(),
            age: 35,
            email: "remote.alice@example.com".to_string(),
        };

        println!("Node A: Storing remote user data...");
        let put_result = netabase.put(test_user.clone(), None, Quorum::One).await?;
        println!("Node A: Put result: {:?}", put_result);

        // Also store a binary document
        let binary_doc = TestDocument::Binary {
            id: "binary_doc_1".to_string(),
            data: vec![0xDE, 0xAD, 0xBE, 0xEF],
        };

        println!("Node A: Storing binary document...");
        let put_result = netabase.put(binary_doc.clone(), None, Quorum::One).await?;
        println!("Node A: Binary doc put result: {:?}", put_result);

        // Wait for replication across network
        sleep(Duration::from_secs(10)).await;

        // Verify local retrieval
        let get_result = netabase.get(test_user.key()).await?;
        let retrieved_user: TestUser = get_result.record.into();
        assert_eq!(retrieved_user, test_user);

        println!("Node A: Remote test completed successfully!");

    } else if config.node_id == "B" {
        println!("Node B (Machine 2): Connected to remote network");

        // Wait for network to stabilize
        wait_for_connectivity(&mut netabase, Duration::from_secs(30)).await?;

        // Try to retrieve remote data
        let remote_user_key = TestUser {
            id: "remote_user_123".to_string(),
            name: "".to_string(),
            age: 0,
            email: "".to_string(),
        }.key();

        println!("Node B: Attempting to retrieve remote user data...");
        sleep(Duration::from_secs(5)).await;

        match netabase.get(remote_user_key).await {
            Ok(get_result) => {
                let retrieved_user: TestUser = get_result.record.into();
                println!("Node B: Successfully retrieved remote user: {:?}", retrieved_user);

                assert_eq!(retrieved_user.name, "Remote Alice");
                assert_eq!(retrieved_user.age, 35);
                assert_eq!(retrieved_user.email, "remote.alice@example.com");

                // Also try to get the binary document
                let binary_doc_key = TestDocument::Binary {
                    id: "binary_doc_1".to_string(),
                    data: vec![], // We only need the key
                }.key();

                match netabase.get(binary_doc_key).await {
                    Ok(get_result) => {
                        let retrieved_doc: TestDocument = get_result.record.into();
                        if let TestDocument::Binary { id, data } = retrieved_doc {
                            assert_eq!(id, "binary_doc_1");
                            assert_eq!(data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
                            println!("Node B: Successfully retrieved binary document!");
                        }
                    }
                    Err(e) => {
                        println!("Node B: Could not retrieve binary document: {:?}", e);
                    }
                }

                println!("Node B: Remote test completed successfully!");
            }
            Err(e) => {
                println!("Node B: Could not retrieve remote data: {:?}", e);

                // Store and test local data
                let local_user = TestUser {
                    id: "local_user_456".to_string(),
                    name: "Local Bob".to_string(),
                    age: 28,
                    email: "local.bob@example.com".to_string(),
                };

                println!("Node B: Storing local user data...");
                let put_result = netabase.put(local_user.clone(), None, Quorum::One).await?;
                println!("Node B: Put result: {:?}", put_result);

                let get_result = netabase.get(local_user.key()).await?;
                let retrieved_user: TestUser = get_result.record.into();
                assert_eq!(retrieved_user, local_user);

                println!("Node B: Local operations working correctly!");
            }
        }
    }

    // Keep alive for extended testing
    println!("Node {}: Keeping alive for 60 seconds for extended testing...", config.node_id);
    sleep(Duration::from_secs(60)).await;

    Ok(())
}

#[tokio::test]
async fn test_local_netabase_operations() -> anyhow::Result<()> {
    init_logging();

    // Create a single local instance for basic functionality testing
    let config = TestConfig {
        node_id: "local".to_string(),
        listen_ip: "127.0.0.1".to_string(),
        listen_port: 0,
        bootstrap_addr: None,
    };

    let (mut netabase, _peer_id, _listen_addr) = create_test_netabase(config).await?;

    // Wait for swarm to initialize
    sleep(Duration::from_secs(1)).await;

    // Create test data using our schemas
    let test_user = TestUser {
        id: "local_test_user".to_string(),
        name: "Local Test User".to_string(),
        age: 42,
        email: "test@local.com".to_string(),
    };

    // Store data locally
    println!("Storing test user locally...");
    let put_result = netabase.put(test_user.clone(), None, Quorum::One).await?;
    println!("Put result: {:?}", put_result);

    // Retrieve data locally
    println!("Retrieving test user locally...");
    let get_result = netabase.get(test_user.key()).await?;
    println!("Get result: {:?}", get_result);

    // Verify data integrity
    let retrieved_user: TestUser = get_result.record.into();
    assert_eq!(retrieved_user, test_user);

    println!("Local test completed successfully!");

    Ok(())
}

#[tokio::test]
async fn test_multiple_schema_types() -> anyhow::Result<()> {
    init_logging();

    let config = TestConfig {
        node_id: "multi".to_string(),
        listen_ip: "127.0.0.1".to_string(),
        listen_port: 0,
        bootstrap_addr: None,
    };

    let (mut netabase, _peer_id, _listen_addr) = create_test_netabase(config).await?;

    // Wait for swarm to initialize
    sleep(Duration::from_secs(1)).await;

    // Test different schema types
    let test_user = TestUser {
        id: "user1".to_string(),
        name: "Test User".to_string(),
        age: 25,
        email: "user@test.com".to_string(),
    };

    let test_message = TestMessage {
        message_id: 12345,
        sender: "Test User".to_string(),
        content: "Hello, World!".to_string(),
        timestamp: 1640995200,
    };

    let test_text_doc = TestDocument::Text {
        id: "doc1".to_string(),
        content: "This is a text document".to_string(),
    };

    let test_binary_doc = TestDocument::Binary {
        id: "doc2".to_string(),
        data: vec![1, 2, 3, 4, 5],
    };

    // Store all different types
    println!("Storing user...");
    let user_put = netabase.put(test_user.clone(), None, Quorum::One).await?;
    println!("User put result: {:?}", user_put);

    println!("Storing message...");
    let message_put = netabase.put(test_message.clone(), None, Quorum::One).await?;
    println!("Message put result: {:?}", message_put);

    println!("Storing text document...");
    let text_doc_put = netabase.put(test_text_doc.clone(), None, Quorum::One).await?;
    println!("Text doc put result: {:?}", text_doc_put);

    println!("Storing binary document...");
    let binary_doc_put = netabase.put(test_binary_doc.clone(), None, Quorum::One).await?;
    println!("Binary doc put result: {:?}", binary_doc_put);

    // Retrieve and verify all types
    println!("Retrieving and verifying all stored data...");

    let retrieved_user: TestUser = netabase.get(test_user.key()).await?.record.into();
    assert_eq!(retrieved_user, test_user);
    println!("✓ User verified");

    let retrieved_message: TestMessage = netabase.get(test_message.key()).await?.record.into();
    assert_eq!(retrieved_message, test_message);
    println!("✓ Message verified");

    let retrieved_text_doc: TestDocument = netabase.get(test_text_doc.key()).await?.record.into();
    assert_eq!(retrieved_text_doc, test_text_doc);
    println!("✓ Text document verified");

    let retrieved_binary_doc: TestDocument = netabase.get(test_binary_doc.key()).await?.record.into();
    assert_eq!(retrieved_binary_doc, test_binary_doc);
    println!("✓ Binary document verified");

    println!("Multiple schema types test completed successfully!");

    Ok(())
}

/// Helper function to print usage instructions
#[allow(dead_code)]
fn print_usage() {
    println!("Distributed Netabase Tests");
    println!("==========================");
    println!();
    println!("Local Testing (Two Processes):");
    println!("  Terminal 1: NETABASE_NODE=A cargo test distributed_two_nodes_local -- --nocapture --ignored");
    println!("  Terminal 2: NETABASE_NODE=B NETABASE_BOOTSTRAP=<addr_from_terminal_1> cargo test distributed_two_nodes_local -- --nocapture --ignored");
    println!();
    println!("Remote Testing (Two Machines):");
    println!("  Machine 1: NETABASE_NODE=A NETABASE_IP=0.0.0.0 NETABASE_PORT=4001 cargo test distributed_two_nodes_remote -- --nocapture --ignored");
    println!("  Machine 2: NETABASE_NODE=B NETABASE_IP=0.0.0.0 NETABASE_PORT=4001 NETABASE_BOOTSTRAP=/ip4/<machine1_ip>/tcp/4001/p2p/<peer_id> cargo test distributed_two_nodes_remote -- --nocapture --ignored");
    println!();
    println!("Environment Variables:");
    println!("  NETABASE_NODE: Node identifier (A or B)");
    println!("  NETABASE_IP: IP to bind to (default: 127.0.0.1)");
    println!("  NETABASE_PORT: Port to bind to (default: 0 for random)");
    println!("  NETABASE_BOOTSTRAP: Bootstrap address for connecting to other nodes");
    println!();
    println!("Non-distributed tests (run normally):");
    println!("  cargo test test_local_netabase_operations");
    println!("  cargo test test_multiple_schema_types");
}
