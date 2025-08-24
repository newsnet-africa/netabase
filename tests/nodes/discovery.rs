use std::time::Duration;

use libp2p::identity::ed25519::Keypair;
use netabase::{Netabase, config::NetabaseConfig, init_logging};
use tokio::time::sleep;

/// Tests for Netabase swarm event handling and message passing
/// These tests verify that the refactored architecture properly handles:
/// - Swarm thread initialization and event broadcasting
/// - Message passing between main thread and swarm thread
/// - Event reception from broadcast channels
/// - Proper cleanup and shutdown

#[tokio::test]
async fn swarm_creation_and_basic_lifecycle() {
    let netabase_config =
        NetabaseConfig::default().add_listen_address("/ip4/127.0.0.1/tcp/0".parse().unwrap());
    let mut netabase = Netabase::try_new(netabase_config, "netabase_test_creation")
        .await
        .expect("faileed to start netabase");
    println!("Starting Swarm");
    netabase.start_swarm();

    netabase.close_swarm().await;
}

#[tokio::test]
async fn event_reception_with_timeout_collection() {
    let config = NetabaseConfig::default()
        .with_storage_path("./netabase_test_connection1".into())
        .add_listen_address("/ip4/127.0.0.1/tcp/0".parse().unwrap());
    let mut netabase1 = Netabase::try_new(config, "/netabase_test_connection1")
        .await
        .expect("Failed");
    netabase1
        .start_swarm()
        .await
        .expect("Failed to start swarm");

    let listener_options = netabase1.swarm_event_listener.as_mut().unwrap();
    let mut event_receiver = listener_options.resubscribe();

    // Wait for swarm to initialize and start listening
    let mut received_events = Vec::new();
    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(10);

    // Collect events for analysis
    while start_time.elapsed() < timeout && received_events.len() < 5 {
        match tokio::time::timeout(Duration::from_millis(500), event_receiver.recv()).await {
            Ok(Ok(event)) => {
                println!("Received event: {:?}", event);
                received_events.push(event);
            }
            Ok(Err(e)) => {
                println!("Error receiving event: {:?}", e);
                break;
            }
            Err(_) => {
                println!("Timeout waiting for event");
                // Continue to collect more events
            }
        }
    }

    println!("Total events received: {}", received_events.len());

    // We should have received at least one event (typically NewListenAddr)
    assert!(
        !received_events.is_empty(),
        "Should have received at least one swarm event"
    );

    netabase1
        .close_swarm()
        .await
        .expect("Failed to close swarm");
}
#[tokio::test]
async fn event_reception_with_blocking_wait() {
    let config = NetabaseConfig::default()
        .with_storage_path("./netabase_test_connection2".into())
        .add_listen_address("/ip4/127.0.0.1/tcp/0".parse().unwrap());
    let mut netabase2 = Netabase::try_new(config, "/netabase_test_connection2")
        .await
        .expect("Failed");
    netabase2
        .start_swarm()
        .await
        .expect("Failed to start swarm");

    let listener_options = netabase2.swarm_event_listener.as_mut().unwrap();
    let mut event_receiver = listener_options.resubscribe();

    // Wait for the first event with timeout
    match tokio::time::timeout(Duration::from_secs(5), event_receiver.recv()).await {
        Ok(Ok(event)) => {
            println!("Received first event: {:?}", event);
            // Test that we can receive events successfully
        }
        Ok(Err(e)) => {
            panic!("Error receiving event: {:?}", e);
        }
        Err(_) => {
            panic!("Timeout waiting for first event - swarm may not be generating events properly");
        }
    }

    netabase2
        .close_swarm()
        .await
        .expect("Failed to close swarm");
}

#[tokio::test]
async fn dual_node_event_generation() {
    // Create two netabase instances with different storage paths
    let config1 = NetabaseConfig::default()
        .with_storage_path("./netabase_test_peer1".into())
        .add_listen_address("/ip4/127.0.0.1/tcp/0".parse().unwrap());
    let mut netabase1 = Netabase::try_new(config1, "/netabase_test_peer1")
        .await
        .expect("Failed to create netabase1");
    let config2 = NetabaseConfig::default()
        .with_storage_path("./netabase_test_peer2".into())
        .add_listen_address("/ip4/127.0.0.1/tcp/0".parse().unwrap());
    let mut netabase2 = Netabase::try_new(config2, "/netabase_test_peer2")
        .await
        .expect("Failed to create netabase2");

    // Start both swarms
    netabase1
        .start_swarm()
        .await
        .expect("Failed to start swarm1");

    // Get event listener for netabase1 immediately after starting
    let mut event_receiver1 = netabase1
        .swarm_event_listener
        .as_mut()
        .unwrap()
        .resubscribe();

    netabase2
        .start_swarm()
        .await
        .expect("Failed to start swarm2");

    // Get event listener for netabase2 immediately after starting
    let mut event_receiver2 = netabase2
        .swarm_event_listener
        .as_mut()
        .unwrap()
        .resubscribe();

    // Wait for initial setup events (like NewListenAddr)
    sleep(Duration::from_millis(1000)).await;

    // Get the listening addresses of netabase1 to connect netabase2 to it
    let listeners1 = netabase1
        .listeners()
        .await
        .expect("Failed to get listeners");

    println!("Netabase1 listeners: {:?}", listeners1);

    // Verify we have listeners
    assert!(
        !listeners1.is_empty(),
        "Netabase1 should have at least one listener"
    );

    // Collect events from both nodes - try to get at least one event from each
    let mut events1 = Vec::new();
    let mut events2 = Vec::new();

    // Use blocking recv with timeout to ensure we get initial events
    for i in 0..10 {
        // Try to get an event from node1
        if events1.is_empty() {
            match tokio::time::timeout(Duration::from_millis(200), event_receiver1.recv()).await {
                Ok(Ok(event)) => {
                    println!("Node1 event {}: {:?}", i, event);
                    events1.push(event);
                }
                Ok(Err(_)) => break,
                Err(_) => {} // timeout, continue
            }
        }

        // Try to get an event from node2
        if events2.is_empty() {
            match tokio::time::timeout(Duration::from_millis(200), event_receiver2.recv()).await {
                Ok(Ok(event)) => {
                    println!("Node2 event {}: {:?}", i, event);
                    events2.push(event);
                }
                Ok(Err(_)) => break,
                Err(_) => {} // timeout, continue
            }
        }

        // If we have at least one event from each, we're good
        if !events1.is_empty() && !events2.is_empty() {
            break;
        }
    }

    println!("Node1 total events: {}", events1.len());
    println!("Node2 total events: {}", events2.len());

    // Verify that both nodes are generating events
    assert!(
        !events1.is_empty(),
        "Node1 should generate swarm events (like NewListenAddr)"
    );
    assert!(
        !events2.is_empty(),
        "Node2 should generate swarm events (like NewListenAddr)"
    );

    // Clean shutdown
    let _ = netabase1.close_swarm().await;
    let _ = netabase2.close_swarm().await;
}

#[tokio::test]
async fn swarm_event_categorization() {
    // Test to verify we're receiving the expected types of events
    let config = NetabaseConfig::default()
        .with_storage_path("./netabase_test_events".into())
        .add_listen_address("/ip4/127.0.0.1/tcp/0".parse().unwrap());
    let mut netabase = Netabase::try_new(config, "/netabase_test_events")
        .await
        .expect("Failed to create netabase");

    netabase.start_swarm().await.expect("Failed to start swarm");

    let mut event_receiver = netabase
        .swarm_event_listener
        .as_mut()
        .unwrap()
        .resubscribe();

    let mut new_listen_addr_events = 0;
    let mut mdns_events = 0;
    let mut identify_events = 0;
    let mut other_events = 0;

    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(8);

    // Collect and categorize events
    while start_time.elapsed() < timeout {
        match tokio::time::timeout(Duration::from_millis(200), event_receiver.recv()).await {
            Ok(Ok(event)) => {
                println!("Event received: {:?}", event);

                // Categorize the event
                match &event {
                    netabase::network::behaviour::NetabaseEvent(
                        libp2p::swarm::SwarmEvent::NewListenAddr { .. },
                    ) => {
                        new_listen_addr_events += 1;
                    }
                    netabase::network::behaviour::NetabaseEvent(
                        libp2p::swarm::SwarmEvent::Behaviour(
                            netabase::network::behaviour::NetabaseBehaviourEvent::Mdns(..),
                        ),
                    ) => {
                        mdns_events += 1;
                    }
                    netabase::network::behaviour::NetabaseEvent(
                        libp2p::swarm::SwarmEvent::Behaviour(
                            netabase::network::behaviour::NetabaseBehaviourEvent::Identify(..),
                        ),
                    ) => {
                        identify_events += 1;
                    }
                    _ => {
                        other_events += 1;
                    }
                }
            }
            Ok(Err(e)) => {
                println!("Error receiving event: {:?}", e);
                break;
            }
            Err(_) => {
                // Timeout - continue collecting
            }
        }
    }

    println!("Event summary:");
    println!("  NewListenAddr events: {}", new_listen_addr_events);
    println!("  mDNS events: {}", mdns_events);
    println!("  Identify events: {}", identify_events);
    println!("  Other events: {}", other_events);

    // We should have at least received NewListenAddr events when the swarm starts
    assert!(
        new_listen_addr_events > 0,
        "Should receive at least one NewListenAddr event when swarm starts listening"
    );

    let _ = netabase.close_swarm().await;
}
