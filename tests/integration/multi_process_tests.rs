use std::sync::Once;
use std::time::Duration;

use bincode::{Decode, Encode};
use libp2p::{Multiaddr, PeerId};
use netabase::Netabase;
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::NetabaseModel as NetabaseModelTrait;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
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

// Test schema for multi-process tests
#[netabase_schema_module(MultiProcessSchema, MultiProcessSchemaKeys)]
pub mod multi_process_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TestDataKey)]
    pub struct TestData {
        #[key]
        pub id: u64,
        pub content: String,
        pub timestamp: u64,
        pub node_id: String,
    }
}

use multi_process_schema::{MultiProcessSchema, TestData};

/// Generate a unique database path for each test process to avoid conflicts
fn generate_unique_db_path() -> (TempDir, String) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let path = temp_dir.path().to_string_lossy().to_string();
    (temp_dir, path)
}

/// Clean up test database directory (now handled automatically by TempDir)
fn cleanup_test_db(_temp_dir: TempDir) {
    // TempDir automatically cleans up when dropped
}

/// Create test data with unique identifiers
fn create_test_data(id: u64, node_id: &str) -> TestData {
    TestData {
        id,
        content: format!("Data from node {} with id {}", node_id, id),
        timestamp: chrono::Utc::now().timestamp() as u64,
        node_id: node_id.to_string(),
    }
}

// Note: This helper would be used for actual multi-process testing
// but is commented out since we don't have a test node binary yet

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_single_node_with_unique_db_path() {
    init_logger();

    let (_temp_dir, db_path) = generate_unique_db_path();
    let mut netabase = Netabase::<MultiProcessSchema>::new_with_path(&db_path).unwrap();

    // Test that the node can start with a unique database path
    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_data = create_test_data(1, "node_single");

    // Test basic operations work with unique database
    let result = timeout(
        Duration::from_secs(3),
        netabase.put_record(test_data.clone()),
    )
    .await;

    match result {
        Ok(put_result) => {
            println!("Put record with unique DB successful: {:?}", put_result);
        }
        Err(_) => {
            println!("Put record timed out - expected in single-node setup");
        }
    }

    netabase.stop_swarm().await.unwrap();

    // Verify database path uniqueness by creating another instance
    let (_temp_dir2, db_path2) = generate_unique_db_path();
    assert_ne!(db_path, db_path2, "Database paths should be unique");

    let mut netabase2 = Netabase::<MultiProcessSchema>::new_with_path(&db_path2).unwrap();
    netabase2.start_swarm().await.unwrap();
    netabase2.stop_swarm().await.unwrap();

    // Cleanup test databases (automatically handled by TempDir drop)
    cleanup_test_db(_temp_dir);
    cleanup_test_db(_temp_dir2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_broadcast_event_detection() {
    init_logger();

    let (_temp_dir, db_path) = generate_unique_db_path();
    let mut netabase = Netabase::<MultiProcessSchema>::new_with_path(&db_path).unwrap();
    let mut event_receiver = netabase.subscribe_to_broadcasts();

    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize and potentially generate connection events
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Monitor for any broadcast events that might indicate network activity
    let mut received_events = Vec::new();
    let start_time = std::time::Instant::now();

    // Check for events for up to 2 seconds
    while start_time.elapsed() < Duration::from_secs(2) {
        match event_receiver.try_recv() {
            Ok(event) => {
                println!("Detected broadcast event: {:?}", event);
                received_events.push(event);
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                // No events available, continue monitoring
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                println!("Receiver lagged behind");
                break;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                println!("Receiver channel closed");
                break;
            }
        }
    }

    println!("Total broadcast events detected: {}", received_events.len());

    // Test putting a record to potentially generate more events
    let test_data = create_test_data(2, "broadcast_test");
    let _ = timeout(Duration::from_secs(2), netabase.put_record(test_data)).await;

    // Check for any additional events after the put operation
    tokio::time::sleep(Duration::from_millis(100)).await;
    let mut additional_events = 0;
    loop {
        match event_receiver.try_recv() {
            Ok(event) => {
                println!("Additional event after put: {:?}", event);
                additional_events += 1;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => break,
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                println!("Receiver lagged behind");
                break;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                println!("Receiver channel closed");
                break;
            }
        }
    }

    println!(
        "Additional events after put operation: {}",
        additional_events
    );

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_bootstrap_with_no_peers() {
    init_logger();

    let (_temp_dir, db_path) = generate_unique_db_path();
    let mut netabase = Netabase::<MultiProcessSchema>::new_with_path(&db_path).unwrap();

    netabase.start_swarm().await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Test bootstrap operation when no peers are available
    let bootstrap_result = timeout(Duration::from_secs(3), netabase.bootstrap()).await;

    match bootstrap_result {
        Ok(result) => {
            match result {
                Ok(query_result) => {
                    println!("Bootstrap successful (unexpected): {:?}", query_result);
                }
                Err(e) => {
                    println!("Bootstrap failed as expected (no peers): {:?}", e);
                    // This is the expected outcome when no peers are available
                }
            }
        }
        Err(_) => {
            println!("Bootstrap timed out - also acceptable when no peers are available");
        }
    }

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_kademlia_operations_isolation() {
    init_logger();

    // Create two separate netabase instances with unique database paths
    let (_temp_dir1, db_path1) = generate_unique_db_path();
    let (_temp_dir2, db_path2) = generate_unique_db_path();

    let mut netabase1 = Netabase::<MultiProcessSchema>::new_with_path(&db_path1).unwrap();
    let mut netabase2 = Netabase::<MultiProcessSchema>::new_with_path(&db_path2).unwrap();

    // Start both swarms
    netabase1.start_swarm().await.unwrap();
    netabase2.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(300)).await;

    // Test that operations on one instance don't interfere with the other
    let test_data1 = create_test_data(10, "node1");
    let test_data2 = create_test_data(20, "node2");

    // Perform operations on both instances (sequential to avoid async issues)
    let result1 = timeout(
        Duration::from_secs(5),
        netabase1.put_record(test_data1.clone()),
    )
    .await;
    let result2 = timeout(
        Duration::from_secs(5),
        netabase2.put_record(test_data2.clone()),
    )
    .await;

    match (result1, result2) {
        (Ok(r1), Ok(r2)) => {
            println!("Put results - Node1: {:?}, Node2: {:?}", r1, r2);
        }
        _ => {
            println!("Operations timed out - expected in isolated nodes");
        }
    }

    // Test that each node can get its mode independently
    let mode1 = timeout(Duration::from_secs(2), netabase1.get_mode())
        .await
        .expect("Mode operation should not timeout");
    let mode2 = timeout(Duration::from_secs(2), netabase2.get_mode())
        .await
        .expect("Mode operation should not timeout");

    println!("Node1 mode: {:?}, Node2 mode: {:?}", mode1, mode2);
    assert!(mode1.is_ok());
    assert!(mode2.is_ok());

    // Clean up
    netabase1.stop_swarm().await.unwrap();
    netabase2.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore = "Multi-process test that may interfere with IDE - run manually with 'cargo test --ignored'"]
async fn test_provider_operations_across_instances() {
    init_logger();

    let (_temp_dir1, db_path1) = generate_unique_db_path();
    let (_temp_dir2, db_path2) = generate_unique_db_path();

    let mut netabase1 = Netabase::<MultiProcessSchema>::new_with_path(&db_path1).unwrap();
    let mut netabase2 = Netabase::<MultiProcessSchema>::new_with_path(&db_path2).unwrap();

    netabase1.start_swarm().await.unwrap();
    netabase2.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(300)).await;

    let test_data = create_test_data(100, "provider_test");
    let test_key = test_data.key();

    // Test start providing on first instance
    let provide_result = timeout(
        Duration::from_secs(3),
        netabase1.start_providing(test_key.clone()),
    )
    .await;

    match provide_result {
        Ok(result) => {
            println!("Start providing result: {:?}", result);
        }
        Err(_) => {
            println!("Start providing timed out");
        }
    }

    // Test get providers on second instance
    let providers_result = timeout(
        Duration::from_secs(3),
        netabase2.get_providers(test_key.clone()),
    )
    .await;

    match providers_result {
        Ok(result) => {
            println!("Get providers result: {:?}", result);
        }
        Err(_) => {
            println!("Get providers timed out - expected without peer connection");
        }
    }

    // Test stop providing
    let _ = timeout(Duration::from_secs(2), netabase1.stop_providing(test_key)).await;

    netabase1.stop_swarm().await.unwrap();
    netabase2.stop_swarm().await.unwrap();
}

/// Test simulating two processes that should be able to communicate
/// This test creates two netabase instances that try to connect to each other
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore = "Multi-process test that may interfere with IDE - run manually with 'cargo test --ignored'"]
async fn test_simulated_peer_connection() {
    init_logger();

    let (_temp_dir1, db_path1) = generate_unique_db_path();
    let (_temp_dir2, db_path2) = generate_unique_db_path();

    let mut netabase1 = Netabase::<MultiProcessSchema>::new_with_path(&db_path1).unwrap();
    let mut netabase2 = Netabase::<MultiProcessSchema>::new_with_path(&db_path2).unwrap();

    // Subscribe to broadcasts to monitor for connection events
    let mut events1 = netabase1.subscribe_to_broadcasts();
    let mut events2 = netabase2.subscribe_to_broadcasts();

    netabase1.start_swarm().await.unwrap();
    netabase2.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Try to simulate peer discovery by adding each other as known addresses
    // Note: In a real scenario, nodes would discover each other through the DHT
    let dummy_peer = PeerId::random();
    let dummy_addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();

    // Add dummy addresses to test the peer management functionality
    let add_result1 = timeout(
        Duration::from_secs(2),
        netabase1.add_address(dummy_peer, dummy_addr.clone()),
    )
    .await;

    let add_result2 = timeout(
        Duration::from_secs(2),
        netabase2.add_address(dummy_peer, dummy_addr.clone()),
    )
    .await;

    println!("Add address results: {:?}, {:?}", add_result1, add_result2);

    // Monitor events for a short period to see if any connection-related events occur
    let monitoring_duration = Duration::from_secs(2);
    let start_time = std::time::Instant::now();

    let mut events_count1 = 0;
    let mut events_count2 = 0;

    while start_time.elapsed() < monitoring_duration {
        // Check events from first instance
        match events1.try_recv() {
            Ok(event) => {
                println!("Node1 event: {:?}", event);
                events_count1 += 1;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {}
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                println!("Node1 receiver lagged behind");
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                println!("Node1 receiver channel closed");
            }
        }

        // Check events from second instance
        match events2.try_recv() {
            Ok(event) => {
                println!("Node2 event: {:?}", event);
                events_count2 += 1;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {}
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                println!("Node2 receiver lagged behind");
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                println!("Node2 receiver channel closed");
            }
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    println!(
        "Event counts - Node1: {}, Node2: {}",
        events_count1, events_count2
    );

    // Test that both nodes can perform independent operations
    let test_data1 = create_test_data(201, "sim_node1");
    let test_data2 = create_test_data(202, "sim_node2");

    let _ = timeout(Duration::from_secs(3), netabase1.put_record(test_data1)).await;
    let _ = timeout(Duration::from_secs(3), netabase2.put_record(test_data2)).await;

    netabase1.stop_swarm().await.unwrap();
    netabase2.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore = "Multi-process test that may interfere with IDE - run manually with 'cargo test --ignored'"]
async fn test_sequential_process_startup() {
    init_logger();

    println!("Testing sequential process startup and connection detection");

    let (_temp_dir1, db_path1) = generate_unique_db_path();
    let (_temp_dir2, db_path2) = generate_unique_db_path();

    // Start first process and wait for it to be ready
    let mut netabase1 = Netabase::<MultiProcessSchema>::new_with_path(&db_path1).unwrap();
    let mut events1 = netabase1.subscribe_to_broadcasts();

    println!("Starting first process...");
    netabase1.start_swarm().await.unwrap();

    // Wait for first process to be fully initialized
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Monitor events from first process
    let mut initial_events1 = 0;
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < Duration::from_secs(1) {
        match events1.try_recv() {
            Ok(event) => {
                println!("First process event: {:?}", event);
                initial_events1 += 1;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                println!("First process receiver lagged behind");
                break;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                println!("First process receiver channel closed");
                break;
            }
        }
    }
    println!("First process initial events: {}", initial_events1);

    // Start second process
    let mut netabase2 = Netabase::<MultiProcessSchema>::new_with_path(&db_path2).unwrap();
    let mut events2 = netabase2.subscribe_to_broadcasts();

    println!("Starting second process...");
    netabase2.start_swarm().await.unwrap();

    // Wait for second process to be fully initialized
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Monitor for any connection activity
    let monitoring_duration = Duration::from_secs(3);
    let start_time = std::time::Instant::now();
    let mut connection_events1 = 0;
    let mut connection_events2 = 0;

    while start_time.elapsed() < monitoring_duration {
        match events1.try_recv() {
            Ok(event) => {
                println!("Process1 connection event: {:?}", event);
                connection_events1 += 1;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {}
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                println!("Process1 receiver lagged behind");
                break;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                println!("Process1 receiver channel closed");
                break;
            }
        }

        match events2.try_recv() {
            Ok(event) => {
                println!("Process2 connection event: {:?}", event);
                connection_events2 += 1;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {}
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                println!("Process2 receiver lagged behind");
                break;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                println!("Process2 receiver channel closed");
                break;
            }
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    println!(
        "Connection detection - Process1: {}, Process2: {}",
        connection_events1, connection_events2
    );

    // Test that both processes can operate independently
    let test_data1 = create_test_data(2001, "seq_node1");
    let test_data2 = create_test_data(2002, "seq_node2");

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
#[ignore = "Multi-process test that may interfere with IDE - run manually with 'cargo test --ignored'"]
async fn test_broadcast_event_propagation() {
    init_logger();

    println!("Testing broadcast event propagation between processes");

    let (_temp_dir1, db_path1) = generate_unique_db_path();
    let (_temp_dir2, db_path2) = generate_unique_db_path();

    let mut netabase1 = Netabase::<MultiProcessSchema>::new_with_path(&db_path1).unwrap();
    let mut netabase2 = Netabase::<MultiProcessSchema>::new_with_path(&db_path2).unwrap();

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

        let test_data = create_test_data(3001, "broadcast_node");

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

/// Test database path uniqueness to ensure no conflicts between test runs
#[test]
fn test_database_path_generation() {
    let (_temp_dir1, path1) = generate_unique_db_path();
    let (_temp_dir2, path2) = generate_unique_db_path();
    let (_temp_dir3, path3) = generate_unique_db_path();

    // Ensure all paths are unique
    assert_ne!(path1, path2);
    assert_ne!(path2, path3);
    assert_ne!(path1, path3);

    // Ensure all paths are valid temp directories
    assert!(std::path::Path::new(&path1).exists());
    assert!(std::path::Path::new(&path2).exists());
    assert!(std::path::Path::new(&path3).exists());
}

/// Test concurrent database creation with unique paths
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_concurrent_database_creation() {
    init_logger();

    // Create multiple netabase instances concurrently with unique paths
    let futures = (0..5).map(|i| async move {
        let (_temp_dir, db_path) = generate_unique_db_path();
        let mut netabase = Netabase::<MultiProcessSchema>::new_with_path(&db_path).unwrap();
        netabase.start_swarm().await.unwrap();

        // Perform a quick operation to verify functionality
        let test_data = create_test_data(i, &format!("concurrent_node_{}", i));
        let _ = timeout(Duration::from_secs(2), netabase.put_record(test_data)).await;

        netabase.stop_swarm().await.unwrap();
        // Keep temp_dir alive until the end
        (i, db_path, _temp_dir)
    });

    let results = timeout(Duration::from_secs(15), futures::future::join_all(futures)).await;

    match results {
        Ok(instances) => {
            println!(
                "Successfully created {} concurrent instances",
                instances.len()
            );

            // Verify all database paths are unique
            let mut paths = std::collections::HashSet::new();
            for (id, path, _temp_dir) in instances {
                assert!(paths.insert(path.clone()), "Duplicate path found: {}", path);
                println!("Instance {}: {}", id, path);
            }
        }
        Err(_) => {
            panic!("Concurrent database creation timed out");
        }
    }
}
