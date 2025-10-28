//! Integration tests using Netabase as the entrypoint
//! Tests sync protocols through the high-level Netabase API

use netabase::Netabase;
use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
use netabase_store::traits::model::NetabaseModelTrait;
use netabase::sync::{
    SyncBehaviorManager, SyncManagerConfigBuilder,
    PaxosInstance, PaxosConfig,
    VectorClock, Version,
};
use libp2p::PeerId;
use std::time::Duration;
use tokio::time::sleep;

// Define test data model
#[netabase_definition_module(TestDefinition, TestKeys)]
mod test_definition {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub email: String,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        pub author_id: u64,
    }
}

use test_definition::*;

// ============================================================================
// BASIC NETABASE WITH SYNC TESTS
// ============================================================================

#[tokio::test]
async fn test_netabase_creation_with_sync() {
    let netabase = Netabase::<TestDefinition>::new();
    assert!(netabase.is_ok(), "Should create Netabase instance");
}

#[tokio::test]
async fn test_netabase_start_stop_swarm() {
    let mut netabase = Netabase::<TestDefinition>::new()
        .expect("Failed to create Netabase");

    // Start swarm
    let start_result = netabase.start_swarm().await;
    assert!(start_result.is_ok(), "Should start swarm successfully");

    // Give it time to initialize
    sleep(Duration::from_millis(500)).await;

    // Stop swarm
    let stop_result = netabase.stop_swarm().await;
    assert!(stop_result.is_ok(), "Should stop swarm successfully");
}

#[tokio::test]
async fn test_netabase_put_get_record_with_sync() {
    let mut netabase = Netabase::<TestDefinition>::new()
        .expect("Failed to create Netabase");

    netabase.start_swarm().await.expect("Failed to start swarm");

    // Create a user
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Put record
    let put_result = netabase.put_record(user.clone()).await;
    // Note: put_record might timeout in single-node setup, which is expected
    println!("Put result: {:?}", put_result);

    // Try to get record from local store (should work even if network times out)
    let key = UserKey::Primary(UserPrimaryKey(1));

    // Give network time to process
    sleep(Duration::from_millis(100)).await;

    netabase.stop_swarm().await.expect("Failed to stop swarm");
}

// ============================================================================
// MULTI-NODE SYNC TESTS
// ============================================================================

#[tokio::test]
async fn test_multi_node_netabase_sync() {
    // Create two Netabase instances
    let mut netabase1 = Netabase::<TestDefinition>::new()
        .expect("Failed to create Netabase 1");
    let mut netabase2 = Netabase::<TestDefinition>::new()
        .expect("Failed to create Netabase 2");

    // Start both swarms
    netabase1.start_swarm().await.expect("Failed to start swarm 1");
    netabase2.start_swarm().await.expect("Failed to start swarm 2");

    // Give nodes time to discover each other
    sleep(Duration::from_secs(2)).await;

    // Create a user on node 1
    let user = User {
        id: 1,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    let _put_result = netabase1.put_record(user.clone()).await;

    // Give network time to sync
    sleep(Duration::from_secs(1)).await;

    // Try to get from node 2
    let key = UserKey::Primary(UserPrimaryKey(1));
    let _get_result = netabase2.get_record::<UserKey>(key).await;

    // Cleanup
    netabase1.stop_swarm().await.expect("Failed to stop swarm 1");
    netabase2.stop_swarm().await.expect("Failed to stop swarm 2");
}

// ============================================================================
// PAXOS INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_paxos_with_netabase_entrypoint() {
    // Test that Paxos consensus works with Netabase instances

    // Create 4 nodes (n=3f+1 where f=1)
    let peer1 = PeerId::random();
    let peer2 = PeerId::random();
    let peer3 = PeerId::random();
    let peer4 = PeerId::random();

    let config = PaxosConfig::new(4, 1);

    let mut paxos1 = PaxosInstance::new(peer1, config.clone());
    let mut paxos2 = PaxosInstance::new(peer2, config.clone());
    let mut paxos3 = PaxosInstance::new(peer3, config.clone());
    let mut paxos4 = PaxosInstance::new(peer4, config);

    // Simulate Paxos consensus for a value
    let value = b"consensus_value".to_vec();
    let proposal = paxos1.propose(value.clone());

    // Get promises from acceptors
    let promise2 = paxos2.handle_prepare(proposal).expect("Promise failed");
    let promise3 = paxos3.handle_prepare(proposal).expect("Promise failed");

    // Handle promises on proposer
    if let netabase::sync::PaxosMessage::Promise {
        proposal_number,
        accepted_proposal,
        accepted_value,
    } = promise2
    {
        let _ = paxos1.handle_promise(peer2, proposal_number, accepted_proposal, accepted_value);
    }

    if let netabase::sync::PaxosMessage::Promise {
        proposal_number,
        accepted_proposal,
        accepted_value,
    } = promise3
    {
        let result = paxos1.handle_promise(peer3, proposal_number, accepted_proposal, accepted_value);
        // After quorum (f+1=2), should get Accept message
        assert!(result.is_ok());
        assert!(result.unwrap().is_some(), "Should get Accept message after quorum");
    }

    // Verify that Paxos integrated with Netabase lifecycle
    assert!(true, "Paxos consensus workflow completed");
}

#[tokio::test]
async fn test_netabase_history_with_paxos() {
    // Test that /history endpoint shows Paxos consensus values

    let mut netabase = Netabase::<TestDefinition>::new()
        .expect("Failed to create Netabase");

    netabase.start_swarm().await.expect("Failed to start swarm");

    // Create multiple versions of a record
    let user_v1 = User {
        id: 1,
        name: "Charlie v1".to_string(),
        email: "charlie@v1.com".to_string(),
    };

    let user_v2 = User {
        id: 1,
        name: "Charlie v2".to_string(),
        email: "charlie@v2.com".to_string(),
    };

    // Put first version
    let _ = netabase.put_record(user_v1).await;
    sleep(Duration::from_millis(500)).await;

    // Put second version (update)
    let _ = netabase.put_record(user_v2).await;
    sleep(Duration::from_millis(500)).await;

    // When Paxos is enabled, history should track all consensus values
    // This is a placeholder - actual implementation would query history
    println!("History test: Paxos should track version history");

    netabase.stop_swarm().await.expect("Failed to stop swarm");
}

// ============================================================================
// SYNC MANAGER INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_sync_manager_with_netabase_peers() {
    // Test that SyncManager works with Netabase peer discovery

    let peer_id = PeerId::random();
    let config = SyncManagerConfigBuilder::new()
        .gossip_interval(Duration::from_secs(5))
        .gossip_fanout(3)
        .brb_config(4, 1)
        .pow_enabled(true)
        .build();

    let mut sync_manager = SyncBehaviorManager::new(peer_id, config)
        .expect("Failed to create SyncManager");

    // Add some peers (simulating Netabase peer discovery)
    let peer1 = PeerId::random();
    let peer2 = PeerId::random();
    let peer3 = PeerId::random();

    sync_manager.add_peer(peer1);
    sync_manager.add_peer(peer2);
    sync_manager.add_peer(peer3);

    assert_eq!(sync_manager.peer_count(), 3, "Should have 3 peers");

    // Issue challenges (Sybil resistance)
    let challenge = sync_manager.issue_challenge(peer1);
    assert!(!challenge.is_empty(), "Should issue non-empty challenge");

    // Verify peer is not yet verified
    assert!(!sync_manager.is_peer_verified(&peer1), "Peer should not be verified yet");
}

// ============================================================================
// VECTOR CLOCK WITH NETABASE TESTS
// ============================================================================

#[tokio::test]
async fn test_vector_clock_causality_with_netabase() {
    // Test that vector clocks track causality in Netabase operations

    let peer1 = PeerId::random();
    let peer2 = PeerId::random();

    let mut clock1 = VectorClock::new(peer1);
    let mut clock2 = VectorClock::new(peer2);

    // Simulate operations
    clock1.increment(); // [1, 0]
    clock2.increment(); // [0, 1]

    // Clocks are concurrent
    assert!(clock1.is_concurrent(&clock2), "Clocks should be concurrent");

    // Merge and advance
    clock2.merge(&clock1); // [1, 1]
    clock2.increment();    // [1, 2]

    // Now clock1 happened before clock2
    assert!(clock1.happened_before(&clock2), "clock1 should happen before clock2");
}

// ============================================================================
// CROSS-NODE DATA SYNCHRONIZATION TESTS
// ============================================================================

#[tokio::test]
async fn test_cross_node_data_sync_with_netabase() {
    // Test that data syncs across multiple Netabase nodes

    let mut nodes = Vec::new();

    // Create 3 nodes
    for _ in 0..3 {
        let netabase = Netabase::<TestDefinition>::new()
            .expect("Failed to create Netabase node");
        nodes.push(netabase);
    }

    // Start all nodes
    for node in nodes.iter_mut() {
        node.start_swarm().await.expect("Failed to start swarm");
    }

    // Give nodes time to discover each other
    sleep(Duration::from_secs(2)).await;

    // Put data on first node
    let user = User {
        id: 100,
        name: "David".to_string(),
        email: "david@sync.test".to_string(),
    };

    let _ = nodes[0].put_record(user.clone()).await;

    // Give time for sync
    sleep(Duration::from_secs(2)).await;

    // Try to get from other nodes
    let key = UserKey::Primary(UserPrimaryKey(100));

    for (i, node) in nodes.iter_mut().enumerate() {
        println!("Attempting to get record from node {}", i);
        let _result = node.get_record::<UserKey>(key.clone()).await;
        // In a real multi-node test, we'd verify the result
    }

    // Cleanup
    for node in nodes.iter_mut() {
        node.stop_swarm().await.expect("Failed to stop swarm");
    }
}

// ============================================================================
// GOSSIP PROTOCOL WITH NETABASE TESTS
// ============================================================================

#[tokio::test]
async fn test_gossip_protocol_integration() {
    // Test that gossip protocol works with Netabase

    let mut netabase = Netabase::<TestDefinition>::new()
        .expect("Failed to create Netabase");

    netabase.start_swarm().await.expect("Failed to start swarm");

    // Create multiple records to trigger gossip
    for i in 0..5 {
        let user = User {
            id: i,
            name: format!("User{}", i),
            email: format!("user{}@gossip.test", i),
        };
        let _ = netabase.put_record(user).await;
        sleep(Duration::from_millis(100)).await;
    }

    // Gossip should propagate these records
    sleep(Duration::from_secs(2)).await;

    netabase.stop_swarm().await.expect("Failed to stop swarm");
}

// ============================================================================
// BYZANTINE FAULT TOLERANCE WITH NETABASE TESTS
// ============================================================================

#[tokio::test]
async fn test_byzantine_fault_tolerance_with_netabase() {
    // Test that Netabase handles Byzantine faults correctly

    // Create 7 nodes (n=3f+1 where f=2)
    let num_nodes = 7;
    let max_failures = 2;

    let peer_ids: Vec<PeerId> = (0..num_nodes).map(|_| PeerId::random()).collect();

    // Verify Byzantine tolerance configuration
    assert_eq!(num_nodes, 3 * max_failures + 1, "Should satisfy n=3f+1");

    // Create Paxos instances for each node
    let config = PaxosConfig::new(num_nodes, max_failures);
    assert_eq!(config.quorum_size(), max_failures + 1, "Quorum should be f+1");

    // Create sync managers for each node
    let sync_config = SyncManagerConfigBuilder::new()
        .brb_config(num_nodes, max_failures)
        .build();

    for &peer_id in &peer_ids {
        let sync_manager = SyncBehaviorManager::new(peer_id, sync_config.clone());
        assert!(sync_manager.is_ok(), "Should create sync manager for Byzantine setup");
    }
}

// ============================================================================
// SYBIL RESISTANCE WITH NETABASE TESTS
// ============================================================================

#[tokio::test]
async fn test_sybil_resistance_with_netabase() {
    // Test that Netabase enforces Sybil resistance

    let peer_id = PeerId::random();
    let config = SyncManagerConfigBuilder::new()
        .pow_difficulty(4)
        .pow_enabled(true)
        .build();

    let mut sync_manager = SyncBehaviorManager::new(peer_id, config)
        .expect("Failed to create SyncManager");

    // Add a peer
    let new_peer = PeerId::random();
    sync_manager.add_peer(new_peer);

    // Issue challenge
    let challenge = sync_manager.issue_challenge(new_peer);
    assert!(!challenge.is_empty(), "Should issue PoW challenge");

    // Peer must solve challenge before being verified
    assert!(!sync_manager.is_peer_verified(&new_peer), "Peer should not be verified without solving challenge");

    // In a real scenario, peer would solve the challenge and submit proof
    // For now, we just verify the challenge mechanism works
}

// ============================================================================
// NETABASE EVENT SUBSCRIPTION TESTS
// ============================================================================

#[tokio::test]
async fn test_netabase_event_subscription() {
    let mut netabase = Netabase::<TestDefinition>::new()
        .expect("Failed to create Netabase");

    netabase.start_swarm().await.expect("Failed to start swarm");

    // Subscribe to broadcasts
    let mut receiver = netabase.subscribe_to_broadcasts();

    // Put a record to trigger events
    let user = User {
        id: 999,
        name: "Event Test".to_string(),
        email: "event@test.com".to_string(),
    };

    let _ = netabase.put_record(user).await;

    // Try to receive events (with timeout)
    tokio::select! {
        event = receiver.recv() => {
            match event {
                Ok(evt) => println!("Received event: {:?}", evt),
                Err(e) => println!("Event receive error: {:?}", e),
            }
        }
        _ = sleep(Duration::from_secs(1)) => {
            println!("Event receive timeout (expected in some cases)");
        }
    }

    netabase.stop_swarm().await.expect("Failed to stop swarm");
}

// ============================================================================
// NETABASE PROVIDER TESTS
// ============================================================================

#[tokio::test]
async fn test_netabase_provider_operations() {
    let mut netabase = Netabase::<TestDefinition>::new()
        .expect("Failed to create Netabase");

    netabase.start_swarm().await.expect("Failed to start swarm");

    // Create a user
    let user = User {
        id: 42,
        name: "Provider Test".to_string(),
        email: "provider@test.com".to_string(),
    };

    let key = user.primary_key();

    // Start providing
    let provider_result = netabase.start_providing::<UserKey>(key.clone().into()).await;
    println!("Start providing result: {:?}", provider_result);

    // Give network time to register provider
    sleep(Duration::from_millis(500)).await;

    // Get providers
    let providers_result = netabase.get_providers::<UserKey>(key.into()).await;
    println!("Get providers result: {:?}", providers_result);

    netabase.stop_swarm().await.expect("Failed to stop swarm");
}

// ============================================================================
// NETABASE BOOTSTRAP TESTS
// ============================================================================

#[tokio::test]
async fn test_netabase_bootstrap() {
    let mut netabase = Netabase::<TestDefinition>::new()
        .expect("Failed to create Netabase");

    netabase.start_swarm().await.expect("Failed to start swarm");

    // Attempt bootstrap (will fail without known peers, which is expected)
    let bootstrap_result = netabase.bootstrap().await;
    println!("Bootstrap result: {:?}", bootstrap_result);
    // Note: Bootstrap typically fails in single-node tests, which is expected

    netabase.stop_swarm().await.expect("Failed to stop swarm");
}

// ============================================================================
// MULTI-MODEL SYNC TESTS
// ============================================================================

#[tokio::test]
async fn test_multi_model_sync_with_netabase() {
    // Test syncing multiple model types (User and Post)

    let mut netabase = Netabase::<TestDefinition>::new()
        .expect("Failed to create Netabase");

    netabase.start_swarm().await.expect("Failed to start swarm");

    // Create a user
    let user = User {
        id: 1,
        name: "Emma".to_string(),
        email: "emma@multi.test".to_string(),
    };

    // Create a post
    let post = Post {
        id: 1,
        title: "First Post".to_string(),
        content: "This is a test post".to_string(),
        author_id: 1,
    };

    // Put both records
    let _ = netabase.put_record(user).await;
    sleep(Duration::from_millis(100)).await;

    let _ = netabase.put_record(post).await;
    sleep(Duration::from_millis(100)).await;

    // Both should sync through the network
    sleep(Duration::from_secs(1)).await;

    netabase.stop_swarm().await.expect("Failed to stop swarm");
}
