//! Kademlia Memory Storage Test
//!
//! Tests Kademlia DHT functionality using in-memory storage to focus on
//! communication logic without persistence overhead. This test specifically
//! targets the reported bug where "data does not seem to be shared across
//! processes correctly."
//!
//! Run with:
//! ```bash
//! cargo test --features memory test_kademlia_memory_swarm -- --nocapture
//! ```

use std::collections::HashSet;
use std::time::{Duration, Instant};

use bincode::{Decode, Encode};
use libp2p::PeerId;
use log::{debug, error, info, warn};
use netabase::Netabase;
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::{NetabaseModel as NetabaseModelTrait, NetabaseSchema};
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

static INIT: std::sync::Once = std::sync::Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_secs()
            .init();
    });
}

// Define test schema using the correct macro pattern
#[netabase_schema_module(KademliaTestSchema, KademliaTestKeys)]
mod kademlia_test_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TestMessageKey)]
    pub struct TestMessage {
        #[key]
        pub id: u64,
        pub content: String,
        pub sender_node: String,
        pub timestamp: u64,
        pub test_phase: String,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(ProviderTestKey)]
    pub struct ProviderTest {
        #[key]
        pub provider_id: String,
        pub data: String,
        pub timestamp: u64,
    }
}

use kademlia_test_schema::{KademliaTestSchema, ProviderTest, TestMessage};

/// Create a test message for a specific node and phase
fn create_test_message(id: u64, sender: &str, content: &str, phase: &str) -> TestMessage {
    TestMessage {
        id,
        content: content.to_string(),
        sender_node: sender.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        test_phase: phase.to_string(),
    }
}

/// Create a provider test record
fn create_provider_test(provider_id: &str, data: &str) -> ProviderTest {
    ProviderTest {
        provider_id: provider_id.to_string(),
        data: data.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

/// Wait for mDNS peer discovery and connection establishment
async fn wait_for_peer_connections(
    netabase: &Netabase<KademliaTestSchema>,
    node_name: &str,
    expected_peers: usize,
    timeout_duration: Duration,
) -> Result<HashSet<PeerId>, String> {
    info!(
        "🔍 [{}] Waiting for peer discovery and connections...",
        node_name
    );

    let mut discovered_peers = HashSet::new();
    let mut connected_peers = HashSet::new();
    let mut events = netabase.subscribe_to_broadcasts();
    let start_time = Instant::now();

    while start_time.elapsed() < timeout_duration {
        match tokio::time::timeout(Duration::from_millis(100), events.recv()).await {
            Ok(Ok(event)) => {
                match &event.0 {
                    libp2p::swarm::SwarmEvent::Behaviour(behaviour_event) => {
                        match behaviour_event {
                            netabase::network::behaviour::NetabaseBehaviourEvent::Mdns(
                                mdns_event,
                            ) => match mdns_event {
                                libp2p::mdns::Event::Discovered(peers) => {
                                    for (peer_id, addr) in peers {
                                        info!(
                                            "🔍 [{}] mDNS discovered: {} at {}",
                                            node_name, peer_id, addr
                                        );
                                        discovered_peers.insert(*peer_id);
                                    }
                                }
                                libp2p::mdns::Event::Expired(peers) => {
                                    for (peer_id, addr) in peers {
                                        warn!(
                                            "⏰ [{}] mDNS expired: {} at {}",
                                            node_name, peer_id, addr
                                        );
                                        discovered_peers.remove(peer_id);
                                    }
                                }
                            },
                            netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                                kad_event,
                            ) => {
                                debug!("🕸️ [{}] Kademlia event: {:?}", node_name, kad_event);
                            }
                            _ => {
                                debug!(
                                    "📡 [{}] Other behaviour event: {:?}",
                                    node_name, behaviour_event
                                );
                            }
                        }
                    }
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!(
                            "🤝 [{}] Connection established with: {}",
                            node_name, peer_id
                        );
                        connected_peers.insert(*peer_id);
                    }
                    libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                        warn!(
                            "💔 [{}] Connection closed with: {} (cause: {:?})",
                            node_name, peer_id, cause
                        );
                        connected_peers.remove(peer_id);
                    }
                    _ => {
                        // Other swarm events
                    }
                }

                // Success condition: we have connections to the expected number of peers
                if connected_peers.len() >= expected_peers {
                    info!(
                        "✅ [{}] Connected to {} peers: {:?}",
                        node_name,
                        connected_peers.len(),
                        connected_peers
                    );
                    return Ok(connected_peers);
                }
            }
            Ok(Err(e)) => {
                error!("❌ [{}] Event stream error: {:?}", node_name, e);
                break;
            }
            Err(_) => {
                // Timeout waiting for events, continue
            }
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    if connected_peers.is_empty() {
        Err(format!(
            "No connections established by {} within {:?}",
            node_name, timeout_duration
        ))
    } else {
        warn!(
            "⚠️ [{}] Only connected to {} out of {} expected peers",
            node_name,
            connected_peers.len(),
            expected_peers
        );
        Ok(connected_peers)
    }
}

/// Test data sharing between connected nodes
async fn test_cross_node_data_sharing(
    nodes: &[(String, Netabase<KademliaTestSchema>)],
    test_phase: &str,
) -> Result<(usize, usize), String> {
    info!(
        "🔄 Testing cross-node data sharing for phase: {}",
        test_phase
    );

    let mut test_messages = Vec::new();
    let mut successful_puts = 0;

    // Phase 1: Each node stores a message
    for (i, (node_name, netabase)) in nodes.iter().enumerate() {
        let message = create_test_message(
            (i + 1) as u64,
            node_name,
            &format!("Hello from {} in {}", node_name, test_phase),
            test_phase,
        );

        info!("📤 [{}] Storing message: {:?}", node_name, message);

        match timeout(
            Duration::from_secs(10),
            netabase.put_record(message.clone()),
        )
        .await
        {
            Ok(Ok(_)) => {
                info!("✅ [{}] Successfully stored message", node_name);
                test_messages.push((node_name.clone(), message));
                successful_puts += 1;
            }
            Ok(Err(e)) => {
                error!("❌ [{}] Failed to store message: {:?}", node_name, e);
            }
            Err(_) => {
                error!("⏰ [{}] Store operation timed out", node_name);
            }
        }
    }

    // Wait for DHT propagation
    info!("⏳ Waiting for DHT propagation...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Phase 2: Each node tries to retrieve messages from other nodes
    let mut successful_retrievals = 0;
    let mut total_attempts = 0;

    for (retriever_name, retriever_netabase) in nodes.iter() {
        for (sender_name, test_message) in &test_messages {
            // Skip retrieving own message
            if retriever_name == sender_name {
                continue;
            }

            total_attempts += 1;
            info!(
                "📥 [{}] Attempting to retrieve message from {}",
                retriever_name, sender_name
            );

            match timeout(
                Duration::from_secs(15),
                retriever_netabase.get_record(test_message.key()),
            )
            .await
            {
                Ok(Ok(query_result)) => {
                    match query_result {
                        libp2p::kad::QueryResult::GetRecord(get_record_result) => {
                            match get_record_result {
                                Ok(get_record_ok) => {
                                    match get_record_ok {
                                        libp2p::kad::GetRecordOk::FoundRecord(peer_record) => {
                                            // Try to decode the record data
                                            match bincode::decode_from_slice::<TestMessage, _>(
                                                &peer_record.record.value,
                                                bincode::config::standard(),
                                            ) {
                                                Ok((retrieved_message, _)) => {
                                                    if retrieved_message == *test_message {
                                                        info!(
                                                            "✅ [{}] Successfully retrieved message from {}",
                                                            retriever_name, sender_name
                                                        );
                                                        successful_retrievals += 1;
                                                    } else {
                                                        error!(
                                                            "❌ [{}] Retrieved message differs from original from {}",
                                                            retriever_name, sender_name
                                                        );
                                                    }
                                                }
                                                Err(e) => {
                                                    error!(
                                                        "❌ [{}] Failed to deserialize message from {}: {:?}",
                                                        retriever_name, sender_name, e
                                                    );
                                                }
                                            }
                                        }
                                        libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. } => {
                                            debug!(
                                                "🔍 [{}] Query finished but no record found from {}",
                                                retriever_name, sender_name
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "❌ [{}] Get record error from {}: {:?}",
                                        retriever_name, sender_name, e
                                    );
                                }
                            }
                        }
                        _ => {
                            error!(
                                "❌ [{}] Unexpected query result type from {}",
                                retriever_name, sender_name
                            );
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!(
                        "❌ [{}] Error retrieving message from {}: {:?}",
                        retriever_name, sender_name, e
                    );
                }
                Err(_) => {
                    error!(
                        "⏰ [{}] Retrieval timed out when getting message from {}",
                        retriever_name, sender_name
                    );
                }
            }
        }
    }

    info!(
        "📊 Phase {} results: {}/{} retrievals successful",
        test_phase, successful_retrievals, total_attempts
    );

    Ok((successful_retrievals, total_attempts))
}

/// Test provider functionality between nodes
async fn test_provider_functionality(
    nodes: &[(String, Netabase<KademliaTestSchema>)],
) -> Result<bool, String> {
    if nodes.len() < 2 {
        return Ok(true); // Skip if not enough nodes
    }

    info!("🔄 Testing provider functionality...");

    let provider_node = &nodes[0];
    let seeker_node = &nodes[1];

    let provider_test = create_provider_test(&provider_node.0, "Provider test data");

    // Provider starts providing
    info!(
        "📤 [{}] Starting to provide: {:?}",
        provider_node.0, provider_test
    );

    match timeout(
        Duration::from_secs(10),
        provider_node.1.start_providing(provider_test.key()),
    )
    .await
    {
        Ok(Ok(_)) => {
            info!("✅ [{}] Successfully started providing", provider_node.0);
        }
        Ok(Err(e)) => {
            error!(
                "❌ [{}] Failed to start providing: {:?}",
                provider_node.0, e
            );
            return Err(format!("Failed to start providing: {:?}", e));
        }
        Err(_) => {
            error!("⏰ [{}] Start providing timed out", provider_node.0);
            return Err("Start providing timed out".to_string());
        }
    }

    // Wait for provider record propagation
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Seeker looks for providers
    info!("🔍 [{}] Looking for providers", seeker_node.0);

    match timeout(
        Duration::from_secs(15),
        seeker_node.1.get_providers(provider_test.key()),
    )
    .await
    {
        Ok(Ok(query_result)) => match query_result {
            libp2p::kad::QueryResult::GetProviders(get_providers_result) => {
                match get_providers_result {
                    Ok(get_providers_ok) => match get_providers_ok {
                        libp2p::kad::GetProvidersOk::FoundProviders { providers, .. } => {
                            if providers.is_empty() {
                                error!("❌ [{}] No providers found", seeker_node.0);
                                Ok(false)
                            } else {
                                info!(
                                    "✅ [{}] Found {} providers: {:?}",
                                    seeker_node.0,
                                    providers.len(),
                                    providers
                                );
                                Ok(true)
                            }
                        }
                        libp2p::kad::GetProvidersOk::FinishedWithNoAdditionalRecord { .. } => {
                            error!(
                                "❌ [{}] Query finished but no providers found",
                                seeker_node.0
                            );
                            Ok(false)
                        }
                    },
                    Err(e) => {
                        error!("❌ [{}] Get providers error: {:?}", seeker_node.0, e);
                        Ok(false)
                    }
                }
            }
            _ => {
                error!("❌ [{}] Unexpected query result type", seeker_node.0);
                Ok(false)
            }
        },
        Ok(Err(e)) => {
            error!("❌ [{}] Error getting providers: {:?}", seeker_node.0, e);
            Ok(false)
        }
        Err(_) => {
            error!("⏰ [{}] Get providers timed out", seeker_node.0);
            Ok(false)
        }
    }
}

/// Main comprehensive test function
#[tokio::test]
#[cfg(feature = "memory")]
async fn test_kademlia_memory_swarm() {
    init_logger();

    info!("🚀 Starting Kademlia Memory Storage Test");
    info!("📋 Testing DHT communication logic with in-memory storage");

    const NUM_NODES: usize = 3;
    let node_names: Vec<String> = (1..=NUM_NODES).map(|i| format!("MemNode{}", i)).collect();

    let mut nodes = Vec::new();

    // Create nodes with memory storage
    for node_name in &node_names {
        info!("🔧 Creating {} with memory storage", node_name);

        let netabase = match Netabase::<KademliaTestSchema>::new_with_memory() {
            Ok(n) => n,
            Err(e) => {
                error!("❌ Failed to create {}: {:?}", node_name, e);
                panic!("Failed to create memory-based netabase instance");
            }
        };

        nodes.push((node_name.clone(), netabase));
    }

    info!("✅ Created {} nodes with memory storage", NUM_NODES);

    // Start all swarms
    for (node_name, netabase) in &mut nodes {
        info!("🔌 Starting swarm for {}", node_name);
        if let Err(e) = netabase.start_swarm().await {
            error!("❌ Failed to start swarm for {}: {:?}", node_name, e);
            panic!("Failed to start swarm for {}", node_name);
        }

        // Small delay between swarm starts
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    info!("🔗 All swarms started, waiting for peer discovery...");

    // Wait for peer discovery and connections
    let discovery_timeout = Duration::from_secs(30);
    let mut all_connected = true;

    for (node_name, netabase) in &nodes {
        match wait_for_peer_connections(netabase, node_name, NUM_NODES - 1, discovery_timeout).await
        {
            Ok(connected_peers) => {
                info!(
                    "✅ {} connected to {} peers",
                    node_name,
                    connected_peers.len()
                );
            }
            Err(e) => {
                warn!("⚠️ {} connection issue: {}", node_name, e);
                all_connected = false;
            }
        }
    }

    if !all_connected {
        warn!("⚠️ Not all nodes fully connected, but continuing with test...");
    }

    // Additional time for Kademlia bootstrap
    info!("⏳ Allowing time for Kademlia bootstrap...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Test data sharing in multiple phases
    let test_phases = vec!["initial", "secondary", "final"];
    let mut total_successful = 0;
    let mut total_attempts = 0;

    for phase in &test_phases {
        match test_cross_node_data_sharing(&nodes, phase).await {
            Ok((successful, attempts)) => {
                total_successful += successful;
                total_attempts += attempts;
                info!(
                    "✅ Phase '{}': {}/{} successful",
                    phase, successful, attempts
                );
            }
            Err(e) => {
                error!("❌ Phase '{}' failed: {}", phase, e);
            }
        }

        // Brief pause between phases
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // Test provider functionality
    match test_provider_functionality(&nodes).await {
        Ok(true) => {
            info!("✅ Provider functionality test passed");
        }
        Ok(false) => {
            warn!("⚠️ Provider functionality test failed");
        }
        Err(e) => {
            error!("❌ Provider functionality test error: {}", e);
        }
    }

    // Final results
    info!("📊 Final Test Results:");
    info!("   • Total data sharing attempts: {}", total_attempts);
    info!("   • Successful retrievals: {}", total_successful);

    if total_attempts > 0 {
        let success_rate = (total_successful as f64 / total_attempts as f64) * 100.0;
        info!("   • Success rate: {:.1}%", success_rate);

        if total_successful == 0 {
            error!("🚨 CRITICAL: No cross-node data sharing worked!");
            error!("   This confirms the reported Kademlia data sharing bug.");
        } else if total_successful < total_attempts {
            warn!("⚠️ PARTIAL: Some cross-node data sharing failed.");
            warn!("   This suggests intermittent issues with Kademlia DHT.");
        } else {
            info!("🎉 SUCCESS: All cross-node data sharing operations worked!");
        }
    } else {
        warn!("❓ No data sharing attempts were made");
    }

    // Cleanup
    info!("🧹 Cleaning up nodes...");
    for (node_name, netabase) in &mut nodes {
        if let Err(e) = netabase.stop_swarm().await {
            error!("❌ Failed to stop {}: {:?}", node_name, e);
        } else {
            info!("🛑 {} stopped successfully", node_name);
        }
    }

    info!("🏁 Kademlia Memory Storage Test completed");

    // Assert for test framework
    if total_attempts > 0 {
        assert!(
            total_successful > 0,
            "No cross-node data sharing succeeded - Kademlia bug confirmed"
        );
    }
}

/// Minimal two-node test for focused debugging
#[tokio::test]
#[cfg(feature = "memory")]
async fn test_minimal_two_node_memory() {
    init_logger();

    info!("🚀 Starting Minimal Two-Node Memory Test");

    // Create two nodes
    let mut node1 = Netabase::<KademliaTestSchema>::new_with_memory().unwrap();
    let mut node2 = Netabase::<KademliaTestSchema>::new_with_memory().unwrap();

    // Start swarms
    node1.start_swarm().await.unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;

    node2.start_swarm().await.unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;

    info!("🔍 Waiting for peer discovery...");

    // Wait for connections
    let (result1, result2) = tokio::join!(
        wait_for_peer_connections(&node1, "Node1", 1, Duration::from_secs(30)),
        wait_for_peer_connections(&node2, "Node2", 1, Duration::from_secs(30))
    );

    match (result1, result2) {
        (Ok(_), Ok(_)) => {
            info!("✅ Both nodes connected successfully");
        }
        _ => {
            warn!("⚠️ Connection issues, but continuing test...");
        }
    }

    // Wait for Kademlia bootstrap
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Test simple data exchange
    let msg1 = create_test_message(1, "Node1", "Hello from Node1", "minimal");
    let msg2 = create_test_message(2, "Node2", "Hello from Node2", "minimal");

    // Store messages
    info!("📤 Storing messages...");
    let put1 = node1.put_record(msg1.clone()).await;
    let put2 = node2.put_record(msg2.clone()).await;

    info!("Put results: {:?}, {:?}", put1, put2);

    // Wait for propagation
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Cross-retrieve
    info!("📥 Cross-retrieving messages...");

    let get1 = timeout(Duration::from_secs(10), node2.get_record(msg1.key())).await;
    let get2 = timeout(Duration::from_secs(10), node1.get_record(msg2.key())).await;

    match get1 {
        Ok(Ok(query_result)) => match query_result {
            libp2p::kad::QueryResult::GetRecord(Ok(get_record_ok)) => match get_record_ok {
                libp2p::kad::GetRecordOk::FoundRecord(peer_record) => {
                    match bincode::decode_from_slice::<TestMessage, _>(
                        &peer_record.record.value,
                        bincode::config::standard(),
                    ) {
                        Ok((retrieved, _)) if retrieved == msg1 => {
                            info!("✅ Node2 successfully retrieved Node1's message");
                        }
                        _ => {
                            error!("❌ Node2 retrieved incorrect data from Node1");
                        }
                    }
                }
                libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. } => {
                    error!("❌ Node2 query finished but no record found");
                }
            },
            _ => {
                error!("❌ Node2 failed to retrieve Node1's message");
            }
        },
        _ => {
            error!("❌ Node2 failed to retrieve Node1's message");
        }
    }

    match get2 {
        Ok(Ok(query_result)) => match query_result {
            libp2p::kad::QueryResult::GetRecord(Ok(get_record_ok)) => match get_record_ok {
                libp2p::kad::GetRecordOk::FoundRecord(peer_record) => {
                    match bincode::decode_from_slice::<TestMessage, _>(
                        &peer_record.record.value,
                        bincode::config::standard(),
                    ) {
                        Ok((retrieved, _)) if retrieved == msg2 => {
                            info!("✅ Node1 successfully retrieved Node2's message");
                        }
                        _ => {
                            error!("❌ Node1 retrieved incorrect data from Node2");
                        }
                    }
                }
                libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. } => {
                    error!("❌ Node1 query finished but no record found");
                }
            },
            _ => {
                error!("❌ Node1 failed to retrieve Node2's message");
            }
        },
        _ => {
            error!("❌ Node1 failed to retrieve Node2's message");
        }
    }

    // Cleanup
    node1.stop_swarm().await.unwrap();
    node2.stop_swarm().await.unwrap();

    info!("🏁 Minimal Two-Node Memory Test completed");
}
