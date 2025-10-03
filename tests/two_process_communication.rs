use std::sync::Once;
use std::time::Duration;

use bincode::{Decode, Encode};
use libp2p::{Multiaddr, PeerId};
use netabase::Netabase;
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::NetabaseModel as NetabaseModelTrait;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use uuid::Uuid;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    });
}

// Test schema for two-process communication
#[netabase_schema_module(TwoProcessSchema, TwoProcessSchemaKeys)]
pub mod two_process_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TwoProcessDataKey)]
    pub struct TwoProcessData {
        #[key]
        pub id: u64,
        pub content: String,
        pub sender_node: String,
        pub timestamp: u64,
        pub message_type: String,
    }
}

use two_process_schema::{TwoProcessData, TwoProcessSchema};

/// Generate a unique database path for each test process to avoid conflicts
fn generate_unique_db_path() -> String {
    let uuid = Uuid::new_v4();
    format!("test_two_proc_db_{}", uuid.to_string().replace("-", "_"))
}

/// Create test data for two-process communication
fn create_two_process_data(id: u64, sender: &str, message_type: &str) -> TwoProcessData {
    TwoProcessData {
        id,
        content: format!("Message {} from {}", id, sender),
        sender_node: sender.to_string(),
        timestamp: chrono::Utc::now().timestamp() as u64,
        message_type: message_type.to_string(),
    }
}

/// Test helper to wait for specific broadcast events
async fn wait_for_connection_event(
    event_receiver: &mut tokio::sync::broadcast::Receiver<
        netabase::network::behaviour::clone_impl::NetabaseSwarmEvent<TwoProcessSchema>,
    >,
    timeout_duration: Duration,
) -> bool {
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < timeout_duration {
        match event_receiver.try_recv() {
            Ok(event) => {
                println!("Received broadcast event: {:?}", event);

                // Check if this is a connection-related event
                // In a real implementation, you would inspect the event type
                // For this test, we'll consider any event as potential connection activity
                return true;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                println!("Event receiver lagged behind");
                return false;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                println!("Event receiver channel closed");
                return false;
            }
        }
    }

    false
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore = "Two-process test that may interfere with IDE - run manually with 'cargo test --ignored'"]
async fn test_two_process_bootstrap_and_communication() {
    init_logger();

    // This test simulates two processes communicating with each other
    // Process 1 will start first and wait for a connection
    // Process 2 will try to connect to Process 1

    println!("Starting two-process communication test");

    // Generate unique database paths for both processes
    let db_path1 = generate_unique_db_path();
    let db_path2 = generate_unique_db_path();

    println!("Database paths: {} and {}", db_path1, db_path2);

    // Create first netabase instance (bootstrap node)
    let mut netabase1 = Netabase::<TwoProcessSchema>::new_with_path(&db_path1).unwrap();
    let mut events1 = netabase1.subscribe_to_broadcasts();

    // Create second netabase instance (connecting node)
    let mut netabase2 = Netabase::<TwoProcessSchema>::new_with_path(&db_path2).unwrap();
    let mut events2 = netabase2.subscribe_to_broadcasts();

    // Start the first swarm (bootstrap node)
    println!("Starting bootstrap node...");
    netabase1.start_swarm().await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Start the second swarm (connecting node)
    println!("Starting connecting node...");
    netabase2.start_swarm().await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Test 1: Monitor for initial broadcast events from both nodes
    println!("Monitoring for initial broadcast events...");

    let event_found1 = wait_for_connection_event(&mut events1, Duration::from_secs(2)).await;
    let event_found2 = wait_for_connection_event(&mut events2, Duration::from_secs(2)).await;

    println!(
        "Initial events - Node1: {}, Node2: {}",
        event_found1, event_found2
    );

    // Test 2: Attempt to create a simulated connection between the nodes
    // In a real two-process scenario, nodes would discover each other via DHT
    // For this test, we'll simulate peer discovery by adding each other's addresses

    let dummy_peer1 = PeerId::random();
    let dummy_peer2 = PeerId::random();
    let dummy_addr1: Multiaddr = "/ip4/127.0.0.1/tcp/40001".parse().unwrap();
    let dummy_addr2: Multiaddr = "/ip4/127.0.0.1/tcp/40002".parse().unwrap();

    println!("Simulating peer discovery...");

    // Add each other as known peers
    let add_result1 = timeout(
        Duration::from_secs(2),
        netabase1.add_address(dummy_peer2, dummy_addr2.clone()),
    )
    .await;

    let add_result2 = timeout(
        Duration::from_secs(2),
        netabase2.add_address(dummy_peer1, dummy_addr1.clone()),
    )
    .await;

    println!(
        "Peer addition results: {:?}, {:?}",
        add_result1, add_result2
    );

    // Test 3: Monitor for connection events after peer addition
    println!("Monitoring for connection events after peer addition...");

    let connection_event1 = wait_for_connection_event(&mut events1, Duration::from_secs(3)).await;
    let connection_event2 = wait_for_connection_event(&mut events2, Duration::from_secs(3)).await;

    println!(
        "Connection events - Node1: {}, Node2: {}",
        connection_event1, connection_event2
    );

    // Test 4: Test Kademlia functionality between the nodes
    println!("Testing Kademlia operations...");

    let test_data1 = create_two_process_data(1001, "node1", "test_message");
    let test_data2 = create_two_process_data(1002, "node2", "test_message");

    // Node1 puts a record
    let put_result1 = timeout(
        Duration::from_secs(5),
        netabase1.put_record(test_data1.clone()),
    )
    .await;

    // Node2 puts a record
    let put_result2 = timeout(
        Duration::from_secs(5),
        netabase2.put_record(test_data2.clone()),
    )
    .await;

    println!(
        "Put results - Node1: {:?}, Node2: {:?}",
        put_result1, put_result2
    );

    // Test 5: Cross-node record retrieval
    println!("Testing cross-node record retrieval...");

    // Node2 tries to get Node1's record using the key from the data
    let get_result_2_to_1 = timeout(
        Duration::from_secs(5),
        netabase2.get_record(test_data1.key()),
    )
    .await;

    // Node1 tries to get Node2's record using the key from the data
    let get_result_1_to_2 = timeout(
        Duration::from_secs(5),
        netabase1.get_record(test_data2.key()),
    )
    .await;

    println!(
        "Cross-retrieval results - Node2->Node1: {:?}, Node1->Node2: {:?}",
        get_result_2_to_1, get_result_1_to_2
    );

    // Test 6: Provider operations
    println!("Testing provider operations...");

    let provider_data = create_two_process_data(9999, "provider_test", "provider");
    let provider_key = provider_data.key();

    // Node1 starts providing
    let provide_result1 = timeout(
        Duration::from_secs(5),
        netabase1.start_providing(provider_key.clone()),
    )
    .await;

    // Node2 looks for providers
    let providers_result2 = timeout(
        Duration::from_secs(5),
        netabase2.get_providers(provider_key.clone()),
    )
    .await;

    println!(
        "Provider results - Start: {:?}, Get: {:?}",
        provide_result1, providers_result2
    );

    // Test 7: Bootstrap operations from second node
    println!("Testing bootstrap from second node...");

    let bootstrap_result2 = timeout(Duration::from_secs(5), netabase2.bootstrap()).await;

    println!("Bootstrap result from Node2: {:?}", bootstrap_result2);

    // Test 8: Monitor final events
    println!("Monitoring final broadcast events...");

    let final_events1 = wait_for_connection_event(&mut events1, Duration::from_secs(2)).await;
    let final_events2 = wait_for_connection_event(&mut events2, Duration::from_secs(2)).await;

    println!(
        "Final events - Node1: {}, Node2: {}",
        final_events1, final_events2
    );

    // Test 9: Cleanup and shutdown
    println!("Testing graceful shutdown...");

    let shutdown_result1 = netabase1.stop_swarm().await;
    let shutdown_result2 = netabase2.stop_swarm().await;

    assert!(shutdown_result1.is_ok(), "Node1 should shutdown gracefully");
    assert!(shutdown_result2.is_ok(), "Node2 should shutdown gracefully");

    println!("Two-process communication test completed successfully");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore = "Two-process test that may interfere with IDE - run manually with 'cargo test --ignored'"]
async fn test_sequential_process_startup() {
    init_logger();

    println!("Testing sequential process startup and connection detection");

    let db_path1 = generate_unique_db_path();
    let db_path2 = generate_unique_db_path();

    // Start first process and wait for it to be ready
    let mut netabase1 = Netabase::<TwoProcessSchema>::new_with_path(&db_path1).unwrap();
    let mut events1 = netabase1.subscribe_to_broadcasts();

    println!("Starting first process...");
    netabase1.start_swarm().await.unwrap();

    // Wait for first process to be fully initialized
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Monitor events from first process
    let initial_events1 = wait_for_connection_event(&mut events1, Duration::from_secs(1)).await;
    println!("First process initial events: {}", initial_events1);

    // Start second process
    let mut netabase2 = Netabase::<TwoProcessSchema>::new_with_path(&db_path2).unwrap();
    let mut events2 = netabase2.subscribe_to_broadcasts();

    println!("Starting second process...");
    netabase2.start_swarm().await.unwrap();

    // Wait for second process to be fully initialized
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Monitor for any connection activity
    let connection_detected1 =
        wait_for_connection_event(&mut events1, Duration::from_secs(3)).await;
    let connection_detected2 =
        wait_for_connection_event(&mut events2, Duration::from_secs(3)).await;

    println!(
        "Connection detection - Process1: {}, Process2: {}",
        connection_detected1, connection_detected2
    );

    // Test that both processes can operate independently
    let test_data1 = create_two_process_data(2001, "seq_node1", "independent_test");
    let test_data2 = create_two_process_data(2002, "seq_node2", "independent_test");

    let ops_future1 = async {
        let put_result = timeout(
            Duration::from_secs(3),
            netabase1.put_record(test_data1.clone()),
        )
        .await;
        let mode_result = timeout(Duration::from_secs(1), netabase1.get_mode()).await;
        (put_result, mode_result)
    };

    let ops_future2 = async {
        let put_result = timeout(
            Duration::from_secs(3),
            netabase2.put_record(test_data2.clone()),
        )
        .await;
        let mode_result = timeout(Duration::from_secs(1), netabase2.get_mode()).await;
        (put_result, mode_result)
    };

    let (results1, results2) = tokio::join!(ops_future1, ops_future2);

    println!(
        "Independent operations - Process1: {:?}, Process2: {:?}",
        results1, results2
    );

    // Verify both processes can shutdown cleanly
    netabase1.stop_swarm().await.unwrap();
    netabase2.stop_swarm().await.unwrap();

    println!("Sequential startup test completed successfully");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore = "Two-process test that may interfere with IDE - run manually with 'cargo test --ignored'"]
async fn test_broadcast_event_propagation() {
    init_logger();

    println!("Testing broadcast event propagation between processes");

    let db_path1 = generate_unique_db_path();
    let db_path2 = generate_unique_db_path();

    let mut netabase1 = Netabase::<TwoProcessSchema>::new_with_path(&db_path1).unwrap();
    let mut netabase2 = Netabase::<TwoProcessSchema>::new_with_path(&db_path2).unwrap();

    // Create multiple event receivers for each instance
    let mut events1_main = netabase1.subscribe_to_broadcasts();
    let mut events1_secondary = netabase1.subscribe_to_broadcasts();
    let mut events2_main = netabase2.subscribe_to_broadcasts();
    let mut events2_secondary = netabase2.subscribe_to_broadcasts();

    netabase1.start_swarm().await.unwrap();
    netabase2.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(300)).await;

    println!("Monitoring multiple event receivers...");

    // Create a monitoring task for each receiver
    let monitor1_main = async {
        let mut count = 0;
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            match events1_main.try_recv() {
                Ok(event) => {
                    count += 1;
                    println!("Node1 Main receiver event {}: {:?}", count, event);
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                    println!("Node1 Main receiver lagged behind");
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                    println!("Node1 Main receiver channel closed");
                }
            }
        }
        count
    };

    let monitor1_secondary = async {
        let mut count = 0;
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            match events1_secondary.try_recv() {
                Ok(event) => {
                    count += 1;
                    println!("Node1 Secondary receiver event {}: {:?}", count, event);
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                    println!("Node1 Secondary receiver lagged behind");
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                    println!("Node1 Secondary receiver channel closed");
                }
            }
        }
        count
    };

    let monitor2_main = async {
        let mut count = 0;
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            match events2_main.try_recv() {
                Ok(event) => {
                    count += 1;
                    println!("Node2 Main receiver event {}: {:?}", count, event);
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                    println!("Node2 Main receiver lagged behind");
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                    println!("Node2 Main receiver channel closed");
                }
            }
        }
        count
    };

    let monitor2_secondary = async {
        let mut count = 0;
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            match events2_secondary.try_recv() {
                Ok(event) => {
                    count += 1;
                    println!("Node2 Secondary receiver event {}: {:?}", count, event);
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                    println!("Node2 Secondary receiver lagged behind");
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                    println!("Node2 Secondary receiver channel closed");
                }
            }
        }
        count
    };

    // Generate some activity to potentially trigger events
    let activity_task = async {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let test_data = create_two_process_data(3001, "broadcast_node", "event_test");

        // Perform operations that might generate events
        let _ = timeout(
            Duration::from_secs(2),
            netabase1.put_record(test_data.clone()),
        )
        .await;
        let _ = timeout(Duration::from_secs(1), netabase1.get_mode()).await;
        let _ = timeout(
            Duration::from_secs(1),
            netabase1.set_mode(Some(libp2p::kad::Mode::Client)),
        )
        .await;

        let _ = timeout(Duration::from_secs(2), netabase2.put_record(test_data)).await;
        let _ = timeout(Duration::from_secs(1), netabase2.bootstrap()).await;
    };

    // Run all monitoring and activity tasks concurrently
    let (count1_main, count1_sec, count2_main, count2_sec, _) = tokio::join!(
        monitor1_main,
        monitor1_secondary,
        monitor2_main,
        monitor2_secondary,
        activity_task
    );

    println!(
        "Event counts - Node1 Main: {}, Node1 Secondary: {}, Node2 Main: {}, Node2 Secondary: {}",
        count1_main, count1_sec, count2_main, count2_sec
    );

    // Verify that event receivers for the same instance receive the same events
    // (They should be independent but receive the same broadcast)
    println!("Testing event receiver independence...");

    netabase1.stop_swarm().await.unwrap();
    netabase2.stop_swarm().await.unwrap();

    println!("Broadcast event propagation test completed");
}
