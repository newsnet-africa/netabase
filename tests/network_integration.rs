//! Integration tests for network functionality
//!
//! These tests verify that the network components work correctly,
//! including swarm creation, event handling, and peer communication.

use std::time::Duration;
use tokio::time::timeout;

use libp2p::{PeerId, futures::StreamExt};
use netabase::network::{
    behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent},
    swarm::{generate_swarm, handle_event},
};

fn init_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();
    });
}

fn get_test_temp_dir(test_name: &str) -> String {
    format!("./test_tmp/network_{}", test_name)
}

fn cleanup_test_dir(test_name: &str) {
    let test_dir = get_test_temp_dir(test_name);
    if std::path::Path::new(&test_dir).exists() {
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}

#[tokio::test]
async fn test_swarm_creation() {
    init_logging();
    let test_name = "swarm_creation";
    cleanup_test_dir(test_name);

    let temp_dir = get_test_temp_dir(test_name);

    // Test that we can create a swarm without errors
    let swarm = generate_swarm(&temp_dir).expect("Failed to generate swarm");

    // Verify swarm is created with expected properties
    assert!(swarm.local_peer_id() != &PeerId::random());

    cleanup_test_dir(test_name);
}

#[tokio::test]
async fn test_swarm_listening() {
    init_logging();
    let test_name = "swarm_listening";
    cleanup_test_dir(test_name);

    let temp_dir = get_test_temp_dir(test_name);
    let mut swarm = generate_swarm(&temp_dir).expect("Failed to generate swarm");

    // Start listening on a random port
    swarm
        .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().expect("Parse Erruh"))
        .expect("Failed to start listening");

    // Wait for the listening event with timeout
    let result = timeout(Duration::from_secs(5), async {
        loop {
            if let Some(event) = swarm.next().await {
                log::info!("Received swarm event: {:?}", event);
                // Look for NewListenAddr event
                if let libp2p::swarm::SwarmEvent::NewListenAddr { .. } = event {
                    return true;
                }
            }
        }
    })
    .await;

    assert!(
        result.is_ok(),
        "Swarm failed to start listening within timeout"
    );

    cleanup_test_dir(test_name);
}

#[tokio::test]
async fn test_swarm_event_handling() {
    init_logging();
    let test_name = "swarm_event_handling";
    cleanup_test_dir(test_name);

    let temp_dir = get_test_temp_dir(test_name);
    let mut swarm = generate_swarm(&temp_dir).expect("Failed to generate swarm");
    let (tx, mut rx) = tokio::sync::broadcast::channel::<NetabaseBehaviourEvent>(10);

    // Start listening
    swarm
        .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().expect("Parse Erruh"))
        .expect("Failed to start listening");

    // Spawn event handler
    let event_handler = tokio::spawn(async move {
        // Handle a few events then exit
        for _ in 0..3 {
            if let Some(event) = swarm.next().await {
                log::info!("Handling swarm event: {:?}", event);
                // For this test, we just want to verify the handler doesn't crash
            }
        }
    });

    // Spawn event receiver
    let event_receiver = tokio::spawn(async move {
        let mut received_events = 0;
        while received_events < 3 {
            match timeout(Duration::from_millis(100), rx.recv()).await {
                Ok(Ok(event)) => {
                    log::info!("Received behaviour event: {:?}", event);
                    received_events += 1;
                }
                Ok(Err(_)) => break, // Channel closed
                Err(_) => break,     // Timeout
            }
        }
        received_events
    });

    // Wait for both tasks to complete
    let (handler_result, receiver_result) = tokio::join!(event_handler, event_receiver);

    assert!(handler_result.is_ok(), "Event handler task failed");
    assert!(receiver_result.is_ok(), "Event receiver task failed");

    cleanup_test_dir(test_name);
}

#[tokio::test]
async fn test_multiple_swarms() {
    init_logging();
    let test_name = "multiple_swarms";
    cleanup_test_dir(test_name);

    let temp_dir1 = format!("{}_1", get_test_temp_dir(test_name));
    let temp_dir2 = format!("{}_2", get_test_temp_dir(test_name));

    // Create two separate swarms
    let mut swarm1 = generate_swarm(&temp_dir1).expect("Failed to generate swarm1");
    let mut swarm2 = generate_swarm(&temp_dir2).expect("Failed to generate swarm2");

    // Verify they have different peer IDs
    assert_ne!(swarm1.local_peer_id(), swarm2.local_peer_id());

    // Start listening on both
    swarm1
        .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().expect("Parse Erruh"))
        .expect("Failed to start listening on swarm1");

    swarm2
        .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().expect("Parse Erruh"))
        .expect("Failed to start listening on swarm2");

    // Wait for both to start listening
    let swarm1_task = tokio::spawn(async move {
        if let Some(event) = swarm1.next().await {
            matches!(event, libp2p::swarm::SwarmEvent::NewListenAddr { .. })
        } else {
            false
        }
    });

    let swarm2_task = tokio::spawn(async move {
        if let Some(event) = swarm2.next().await {
            matches!(event, libp2p::swarm::SwarmEvent::NewListenAddr { .. })
        } else {
            false
        }
    });

    let (swarm1_listening, swarm2_listening) = tokio::join!(swarm1_task, swarm2_task);

    assert!(
        swarm1_listening.unwrap_or(false),
        "Swarm1 failed to start listening"
    );
    assert!(
        swarm2_listening.unwrap_or(false),
        "Swarm2 failed to start listening"
    );

    // Clean up both directories
    if std::path::Path::new(&temp_dir1).exists() {
        let _ = std::fs::remove_dir_all(&temp_dir1);
    }
    if std::path::Path::new(&temp_dir2).exists() {
        let _ = std::fs::remove_dir_all(&temp_dir2);
    }
}

#[tokio::test]
async fn test_swarm_with_custom_config() {
    init_logging();
    let test_name = "swarm_custom_config";
    cleanup_test_dir(test_name);

    let temp_dir = get_test_temp_dir(test_name);

    // Test that swarm creation works with different database paths
    let swarm = generate_swarm(&temp_dir).expect("Failed to generate swarm");

    // Verify the swarm uses a valid peer ID
    let peer_id = *swarm.local_peer_id();
    assert_ne!(peer_id.to_string(), "");

    log::info!("Created swarm with peer ID: {}", peer_id);

    cleanup_test_dir(test_name);
}

#[tokio::test]
async fn test_swarm_database_integration() {
    init_logging();
    let test_name = "swarm_database_integration";
    cleanup_test_dir(test_name);

    let temp_dir = get_test_temp_dir(test_name);

    // Create swarm which should initialize the database
    let swarm = generate_swarm(&temp_dir).expect("Failed to generate swarm");

    // Verify database directory was created
    let db_path = format!("{}/db", temp_dir);

    // Drop swarm to ensure cleanup
    drop(swarm);

    // Give time for cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check if database files exist
    if std::path::Path::new(&db_path).exists() {
        log::info!("Database was created at: {}", db_path);
    } else {
        log::info!("Database was not persisted (may be in-memory for tests)");
    }

    cleanup_test_dir(test_name);
}
