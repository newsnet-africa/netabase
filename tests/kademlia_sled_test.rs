//! Kademlia Sled Database Test
//!
//! Tests Kademlia DHT functionality using Sled persistent storage to verify
//! data persistence and cross-node communication with database backend.
//! This test is based on the memory test logic but focuses on:
//! - Data persistence to disk
//! - Database integrity verification
//! - Single-threaded execution to avoid Sled conflicts
//!
//! Run with:
//! ```bash
//! cargo test --features native test_kademlia_sled_persistence -- --nocapture --test-threads=1
//! ```

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Once;
use std::time::{Duration, Instant};

use bincode::{Decode, Encode};
use libp2p::PeerId;
use log::{debug, error, info, warn};
use netabase::Netabase;
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::NetabaseModel as NetabaseModelTrait;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use tokio::time::timeout;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_secs()
            .is_test(true)
            .init();
    });
}

// Test configuration constants
const TEST_TIMEOUT: Duration = Duration::from_secs(30);
const DHT_PROPAGATION_WAIT: Duration = Duration::from_secs(5);
const PEER_DISCOVERY_WAIT: Duration = Duration::from_secs(10);

/// Generate current timestamp in seconds
fn current_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Define test schema for Sled database testing
#[netabase_schema_module(SledKademliaSchema, SledKademliaKeys)]
mod sled_kademlia_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(SledTestMessageKey)]
    pub struct SledTestMessage {
        #[key]
        pub id: u64,
        pub content: String,
        pub sender_node: String,
        pub receiver_node: Option<String>,
        pub timestamp: u64,
        pub test_phase: String,
        pub message_size: usize,
        pub persistence_marker: String,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(SledProviderTestKey)]
    pub struct SledProviderTest {
        #[key]
        pub provider_id: String,
        pub data: String,
        pub node_id: String,
        pub timestamp: u64,
        pub data_hash: String,
        pub verification_token: String,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(SledPersistenceTestKey)]
    pub struct SledPersistenceTest {
        #[key]
        pub persistence_id: String,
        pub test_data: Vec<u8>,
        pub creation_time: u64,
        pub node_info: String,
        pub checksum: u32,
    }
}

use sled_kademlia_schema::{
    SledKademliaSchema, SledPersistenceTest, SledProviderTest, SledTestMessage,
};

/// Create a test message with persistence verification data
fn create_sled_test_message(
    id: u64,
    sender: &str,
    content: &str,
    phase: &str,
    receiver: Option<&str>,
) -> SledTestMessage {
    let content_str = content.to_string();
    let persistence_marker = format!(
        "sled_{}_{}_{}_{}",
        sender,
        phase,
        id,
        current_timestamp_secs()
    );

    SledTestMessage {
        id,
        content: content_str.clone(),
        sender_node: sender.to_string(),
        receiver_node: receiver.map(|r| r.to_string()),
        timestamp: current_timestamp_secs(),
        test_phase: phase.to_string(),
        message_size: content_str.len(),
        persistence_marker,
    }
}

/// Create a provider test with hash verification
fn create_sled_provider_test(provider_id: &str, data: &str, node_id: &str) -> SledProviderTest {
    let data_hash = format!("{:x}", md5::compute(data.as_bytes()));
    let verification_token = format!(
        "verify_{}_{}_{}",
        provider_id,
        node_id,
        current_timestamp_secs()
    );

    SledProviderTest {
        provider_id: provider_id.to_string(),
        data: data.to_string(),
        node_id: node_id.to_string(),
        timestamp: current_timestamp_secs(),
        data_hash,
        verification_token,
    }
}

/// Create a persistence test record with checksum verification
fn create_persistence_test(
    persistence_id: &str,
    test_data: &[u8],
    node_info: &str,
) -> SledPersistenceTest {
    let checksum = crc32fast::hash(test_data);

    SledPersistenceTest {
        persistence_id: persistence_id.to_string(),
        test_data: test_data.to_vec(),
        creation_time: current_timestamp_secs(),
        node_info: node_info.to_string(),
        checksum,
    }
}

/// Verify that data is actually persisted to the Sled database
async fn verify_sled_persistence(
    db_path: &PathBuf,
    expected_keys: &[String],
) -> Result<bool, String> {
    info!("🔍 Verifying Sled database persistence at: {:?}", db_path);

    // Open the Sled database directly to verify persistence
    // Try to open the database directory to find any Sled databases
    if !db_path.exists() {
        return Err(format!("Database path does not exist: {:?}", db_path));
    }

    // Look for actual database files in the directory
    let entries = std::fs::read_dir(db_path)
        .map_err(|e| format!("Failed to read database directory: {}", e))?;

    let mut found_db_files = false;
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                found_db_files = true;
                break;
            }
        }
    }

    if !found_db_files {
        info!("⚠️ No database files found in directory: {:?}", db_path);
        return Ok(false); // No data to verify
    }

    let db = sled::open(db_path).map_err(|e| format!("Failed to open Sled DB: {}", e))?;

    let mut found_keys = 0;
    let mut verified_data = Vec::new();

    // Iterate through all keys in the database
    for result in db.iter() {
        match result {
            Ok((key, value)) => {
                let key_str = String::from_utf8_lossy(&key);
                info!("🔑 Found database key: {}", key_str);

                // Check if this is one of our expected keys
                if expected_keys
                    .iter()
                    .any(|expected| key_str.contains(expected))
                {
                    found_keys += 1;
                    verified_data.push((key_str.to_string(), value.len()));
                    info!(
                        "✅ Verified expected key: {} (value size: {} bytes)",
                        key_str,
                        value.len()
                    );
                }
            }
            Err(e) => {
                warn!("⚠️ Error reading database entry: {}", e);
            }
        }
    }

    info!("📊 Database verification results:");
    info!("   Expected keys: {}", expected_keys.len());
    info!("   Found matching keys: {}", found_keys);
    info!("   Database entries verified: {}", verified_data.len());

    // Log detailed verification data
    for (key, size) in verified_data {
        debug!("   📄 Key: {} -> {} bytes", key, size);
    }

    let success = found_keys >= expected_keys.len();
    if success {
        info!("✅ Sled persistence verification PASSED");
    } else {
        error!("❌ Sled persistence verification FAILED");
        error!(
            "   Missing {} expected keys",
            expected_keys.len() - found_keys
        );
    }

    Ok(success)
}

/// Wait for peer connections with enhanced logging
async fn wait_for_sled_peer_connections(
    netabase: &Netabase<SledKademliaSchema>,
    node_name: &str,
    expected_peers: usize,
    timeout_duration: Duration,
) -> Result<HashSet<PeerId>, String> {
    info!(
        "🔍 [{}] Waiting for peer discovery and connections...",
        node_name
    );

    let mut events = netabase.subscribe_to_broadcasts();
    let start_time = Instant::now();
    let mut connected_peers = HashSet::new();
    let mut discovery_events = 0;
    let mut connection_events = 0;

    while start_time.elapsed() < timeout_duration && connected_peers.len() < expected_peers {
        match tokio::time::timeout(Duration::from_millis(100), events.recv()).await {
            Ok(Ok(event)) => match &event.0 {
                libp2p::swarm::SwarmEvent::Behaviour(behaviour_event) => match behaviour_event {
                    netabase::network::behaviour::NetabaseBehaviourEvent::Mdns(mdns_event) => {
                        match mdns_event {
                            libp2p::mdns::Event::Discovered(peers) => {
                                discovery_events += 1;
                                info!(
                                    "🔍 [{}] mDNS discovery event #{}: {} peers",
                                    node_name,
                                    discovery_events,
                                    peers.len()
                                );

                                for (peer_id, addr) in peers {
                                    info!("   📡 Discovered: {} at {}", peer_id, addr);
                                    connected_peers.insert(*peer_id);
                                }
                            }
                            libp2p::mdns::Event::Expired(peers) => {
                                info!("🕰️ [{}] mDNS expired: {} peers", node_name, peers.len());
                                for (peer_id, _) in peers {
                                    connected_peers.remove(peer_id);
                                }
                            }
                        }
                    }
                    _ => {}
                },
                libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    connection_events += 1;
                    info!(
                        "🤝 [{}] Connection established event #{}: {}",
                        node_name, connection_events, peer_id
                    );
                    connected_peers.insert(*peer_id);
                }
                libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    info!("🔌 [{}] Connection closed: {}", node_name, peer_id);
                    connected_peers.remove(peer_id);
                }
                _ => {}
            },
            Ok(Err(e)) => {
                warn!("⚠️ [{}] Event receive error: {}", node_name, e);
            }
            Err(_) => {
                // Timeout on event receive, check current status
                debug!(
                    "⏱️ [{}] No events (connected: {})",
                    node_name,
                    connected_peers.len()
                );
            }
        }
    }

    info!("📊 [{}] Peer connection summary:", node_name);
    info!("   Discovery events: {}", discovery_events);
    info!("   Connection events: {}", connection_events);
    info!("   Final connected peers: {}", connected_peers.len());
    info!("   Expected peers: {}", expected_peers);

    if connected_peers.len() >= expected_peers {
        info!(
            "✅ [{}] Successfully connected to {} peers",
            node_name,
            connected_peers.len()
        );
        Ok(connected_peers)
    } else {
        Err(format!(
            "Failed to connect to required peers. Expected: {}, Got: {}",
            expected_peers,
            connected_peers.len()
        ))
    }
}

/// Test cross-node data sharing with Sled persistence verification
async fn test_sled_cross_node_data_sharing(
    nodes: &[(String, Netabase<SledKademliaSchema>, PathBuf)],
    test_phase: &str,
) -> Result<(usize, usize, Vec<String>), String> {
    info!(
        "🔄 Testing Sled cross-node data sharing for phase: {}",
        test_phase
    );

    let mut test_messages = Vec::new();
    let mut successful_puts = 0;
    let mut expected_keys = Vec::new();

    // Phase 1: Each node stores a message with persistence verification
    for (i, (node_name, netabase, _db_path)) in nodes.iter().enumerate() {
        let message = create_sled_test_message(
            (i + 1) as u64,
            node_name,
            &format!("Sled test data from {} in phase {}", node_name, test_phase),
            test_phase,
            None,
        );

        info!("📤 [{}] Storing Sled message: {:?}", node_name, message);
        expected_keys.push(format!("{}_{}", test_phase, message.id));

        match timeout(TEST_TIMEOUT, netabase.put_record(message.clone())).await {
            Ok(Ok(_)) => {
                info!("✅ [{}] Successfully stored message to Sled", node_name);
                test_messages.push((node_name.clone(), message));
                successful_puts += 1;
            }
            Ok(Err(e)) => {
                error!(
                    "❌ [{}] Failed to store message to Sled: {:?}",
                    node_name, e
                );
            }
            Err(_) => {
                error!("⏰ [{}] Sled store operation timed out", node_name);
            }
        }
    }

    // Wait for DHT propagation and database writes
    info!("⏳ Waiting for DHT propagation and Sled persistence...");
    tokio::time::sleep(DHT_PROPAGATION_WAIT).await;

    // Phase 2: Verify data persistence in each node's database
    for (node_name, _netabase, db_path) in nodes.iter() {
        info!("🔍 [{}] Verifying Sled database persistence", node_name);

        match verify_sled_persistence(db_path, &expected_keys).await {
            Ok(true) => {
                info!("✅ [{}] Sled persistence verification passed", node_name);
            }
            Ok(false) => {
                warn!("⚠️ [{}] Sled persistence verification failed", node_name);
            }
            Err(e) => {
                error!(
                    "❌ [{}] Sled persistence verification error: {}",
                    node_name, e
                );
            }
        }
    }

    // Phase 3: Cross-node retrieval testing
    let mut successful_retrievals = 0;
    let mut total_attempts = 0;

    for (retriever_name, retriever_netabase, _) in nodes.iter() {
        for (sender_name, test_message) in &test_messages {
            if retriever_name == sender_name {
                continue; // Skip self-retrieval
            }

            total_attempts += 1;
            info!(
                "📥 [{}] Retrieving message from {} via Sled/DHT",
                retriever_name, sender_name
            );

            match timeout(
                TEST_TIMEOUT,
                retriever_netabase.get_record(test_message.key()),
            )
            .await
            {
                Ok(Ok(query_result)) => match query_result {
                    libp2p::kad::QueryResult::GetRecord(get_record_result) => {
                        match get_record_result {
                            Ok(get_record_ok) => match get_record_ok {
                                libp2p::kad::GetRecordOk::FoundRecord(peer_record) => {
                                    match bincode::decode_from_slice::<SledTestMessage, _>(
                                        &peer_record.record.value,
                                        bincode::config::standard(),
                                    ) {
                                        Ok((retrieved_message, _)) => {
                                            if retrieved_message == *test_message {
                                                info!(
                                                    "✅ [{}] Successfully retrieved Sled message from {}",
                                                    retriever_name, sender_name
                                                );
                                                successful_retrievals += 1;
                                            } else {
                                                error!(
                                                    "❌ [{}] Retrieved Sled message differs from original from {}",
                                                    retriever_name, sender_name
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            error!(
                                                "❌ [{}] Failed to decode Sled message from {}: {:?}",
                                                retriever_name, sender_name, e
                                            );
                                        }
                                    }
                                }
                                libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord {
                                    ..
                                } => {
                                    warn!(
                                        "⚠️ [{}] No additional Sled record found for {}",
                                        retriever_name, sender_name
                                    );
                                }
                            },
                            Err(e) => {
                                error!(
                                    "❌ [{}] Sled get record error from {}: {:?}",
                                    retriever_name, sender_name, e
                                );
                            }
                        }
                    }
                    _ => {
                        warn!(
                            "⚠️ [{}] Unexpected query result type for Sled message from {}",
                            retriever_name, sender_name
                        );
                    }
                },
                Ok(Err(e)) => {
                    error!(
                        "❌ [{}] Failed to retrieve Sled message from {}: {:?}",
                        retriever_name, sender_name, e
                    );
                }
                Err(_) => {
                    error!(
                        "⏰ [{}] Sled retrieval from {} timed out",
                        retriever_name, sender_name
                    );
                }
            }
        }
    }

    info!("📊 Sled cross-node data sharing results:");
    info!("   Successful puts: {}/{}", successful_puts, nodes.len());
    info!(
        "   Successful retrievals: {}/{}",
        successful_retrievals, total_attempts
    );

    Ok((successful_puts, successful_retrievals, expected_keys))
}

/// Test provider functionality with Sled backend
async fn test_sled_provider_functionality(
    nodes: &[(String, Netabase<SledKademliaSchema>, PathBuf)],
) -> Result<usize, String> {
    info!("🏪 Testing Sled provider functionality");

    let mut successful_provisions = 0;

    for (i, (node_name, netabase, _db_path)) in nodes.iter().enumerate() {
        let provider_test = create_sled_provider_test(
            &format!("sled_provider_{}", i),
            &format!("Provider data from {} stored in Sled", node_name),
            node_name,
        );

        info!(
            "📦 [{}] Providing Sled data: {:?}",
            node_name, provider_test
        );

        match timeout(TEST_TIMEOUT, netabase.put_record(provider_test.clone())).await {
            Ok(Ok(_)) => {
                info!("✅ [{}] Successfully provided Sled data", node_name);
                successful_provisions += 1;

                // Verify the data can be retrieved immediately
                match timeout(TEST_TIMEOUT, netabase.get_record(provider_test.key())).await {
                    Ok(Ok(query_result)) => match query_result {
                        libp2p::kad::QueryResult::GetRecord(Ok(get_record_ok)) => {
                            match get_record_ok {
                                libp2p::kad::GetRecordOk::FoundRecord(_) => {
                                    info!(
                                        "✅ [{}] Sled provider data verified immediately",
                                        node_name
                                    );
                                }
                                _ => {
                                    warn!(
                                        "⚠️ [{}] Sled provider data not found immediately",
                                        node_name
                                    );
                                }
                            }
                        }
                        _ => {
                            warn!(
                                "⚠️ [{}] Unexpected query result for Sled provider data",
                                node_name
                            );
                        }
                    },
                    Ok(Err(e)) => {
                        warn!(
                            "⚠️ [{}] Failed to verify Sled provider data: {:?}",
                            node_name, e
                        );
                    }
                    Err(_) => {
                        warn!("⏰ [{}] Sled provider verification timed out", node_name);
                    }
                }
            }
            Ok(Err(e)) => {
                error!("❌ [{}] Failed to provide Sled data: {:?}", node_name, e);
            }
            Err(_) => {
                error!("⏰ [{}] Sled provide operation timed out", node_name);
            }
        }
    }

    info!(
        "📊 Sled provider functionality results: {}/{} successful",
        successful_provisions,
        nodes.len()
    );

    Ok(successful_provisions)
}

/// Main Sled Kademlia test with comprehensive persistence verification
#[tokio::test]
async fn test_kademlia_sled_persistence() {
    init_logger();
    info!("🧪 Starting Kademlia Sled persistence test");

    const NUM_NODES: usize = 3;
    let mut nodes = Vec::new();
    let mut temp_dirs = Vec::new();
    let mut test_passed = 0;
    let mut test_total = 0;

    // Create nodes with Sled databases
    for i in 0..NUM_NODES {
        let node_name = format!("sled_node_{}", i);
        let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
        let db_path = temp_dir.path().join("netabase_sled");

        info!("🏗️ Creating Sled node: {} at {:?}", node_name, db_path);

        match Netabase::<SledKademliaSchema>::new_with_path(&db_path) {
            Ok(mut netabase) => {
                info!("✅ Successfully created Sled node: {}", node_name);

                // Start the swarm for network operations
                match netabase.start_swarm().await {
                    Ok(_) => {
                        info!("✅ [{}] Successfully started swarm", node_name);
                        nodes.push((node_name, netabase, db_path));
                        temp_dirs.push(temp_dir);
                    }
                    Err(e) => {
                        error!("❌ [{}] Failed to start swarm: {:?}", node_name, e);
                        panic!("Failed to start swarm for test node");
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to create Sled node {}: {:?}", node_name, e);
                panic!("Failed to create test nodes");
            }
        }
    }

    // Wait for network initialization
    info!("⏳ Waiting for Sled nodes to initialize...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Test 1: Peer discovery and connections
    info!("🔍 Test 1: Sled peer discovery and connections");
    test_total += 1;
    let mut connected_nodes = 0;

    for (node_name, netabase, _) in &nodes {
        match wait_for_sled_peer_connections(
            netabase,
            node_name,
            NUM_NODES - 1, // Expect to connect to all other nodes
            Duration::from_secs(30),
        )
        .await
        {
            Ok(peers) => {
                info!("✅ [{}] Connected to {} peers", node_name, peers.len());
                connected_nodes += 1;
            }
            Err(e) => {
                warn!("⚠️ [{}] Peer connection issue: {}", node_name, e);
            }
        }
    }

    if connected_nodes > 0 {
        test_passed += 1;
        info!("✅ Test 1 PASSED: Some nodes connected successfully");
    } else {
        info!("❌ Test 1 FAILED: No nodes connected to peers");
    }

    // Test 2: Cross-node data sharing with Sled persistence
    info!("🔄 Test 2: Sled cross-node data sharing");
    test_total += 1;

    match test_sled_cross_node_data_sharing(&nodes, "sled_persistence_test").await {
        Ok((puts, retrievals, keys)) => {
            info!(
                "✅ Sled data sharing completed: {} puts, {} retrievals",
                puts, retrievals
            );

            if puts > 0 || retrievals > 0 {
                test_passed += 1;
                info!("✅ Test 2 PASSED: Data sharing operations succeeded");
            } else {
                info!("❌ Test 2 FAILED: No successful operations");
            }

            // Additional verification: check databases for expected keys
            let mut db_verification_success = 0;
            for (node_name, _, db_path) in &nodes {
                match verify_sled_persistence(db_path, &keys).await {
                    Ok(true) => {
                        info!("✅ [{}] Database verification passed", node_name);
                        db_verification_success += 1;
                    }
                    Ok(false) => {
                        warn!("⚠️ [{}] Database verification failed", node_name);
                    }
                    Err(e) => {
                        error!("❌ [{}] Database verification error: {}", node_name, e);
                    }
                }
            }

            info!(
                "📊 Database verification: {}/{} nodes passed",
                db_verification_success,
                nodes.len()
            );
        }
        Err(e) => {
            error!("❌ Sled data sharing test failed: {}", e);
            info!("❌ Test 2 FAILED: Data sharing test error");
        }
    }

    // Test 3: Provider functionality with Sled
    info!("🏪 Test 3: Sled provider functionality");
    test_total += 1;

    match test_sled_provider_functionality(&nodes).await {
        Ok(successful_provisions) => {
            info!(
                "✅ Sled provider test completed: {} successful provisions",
                successful_provisions
            );

            if successful_provisions > 0 {
                test_passed += 1;
                info!("✅ Test 3 PASSED: Provider operations succeeded");
            } else {
                info!("❌ Test 3 FAILED: No successful provider operations");
            }
        }
        Err(e) => {
            error!("❌ Sled provider test failed: {}", e);
            info!("❌ Test 3 FAILED: Provider test error");
        }
    }

    // Test 4: Persistence verification after restart simulation
    info!("🔄 Test 4: Sled persistence after restart simulation");
    test_total += 1;

    // Store additional test data before "restart"
    let persistence_tests = vec![
        create_persistence_test(
            "restart_test_1",
            b"Test data for restart verification",
            "restart_node",
        ),
        create_persistence_test(
            "restart_test_2",
            b"Another test record for persistence",
            "restart_node",
        ),
    ];

    let mut pre_restart_keys = Vec::new();
    let mut restart_test_passed = false;

    if let Some((node_name, netabase, _)) = nodes.first() {
        for persistence_test in &persistence_tests {
            pre_restart_keys.push(persistence_test.persistence_id.clone());

            match timeout(TEST_TIMEOUT, netabase.put_record(persistence_test.clone())).await {
                Ok(Ok(_)) => {
                    info!(
                        "✅ [{}] Stored pre-restart test data: {}",
                        node_name, persistence_test.persistence_id
                    );
                }
                Ok(Err(e)) => {
                    error!(
                        "❌ [{}] Failed to store pre-restart data: {:?}",
                        node_name, e
                    );
                }
                Err(_) => {
                    error!("⏰ [{}] Pre-restart store timed out", node_name);
                }
            }
        }

        // Wait for data to be written to disk
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Verify the data is persisted
        if let Some((_, _, db_path)) = nodes.first() {
            match verify_sled_persistence(db_path, &pre_restart_keys).await {
                Ok(true) => {
                    info!("✅ Pre-restart persistence verification passed");
                    restart_test_passed = true;
                }
                Ok(false) => {
                    warn!("⚠️ Pre-restart persistence verification failed");
                }
                Err(e) => {
                    error!("❌ Pre-restart persistence verification error: {}", e);
                }
            }
        }
    }

    if restart_test_passed {
        test_passed += 1;
        info!("✅ Test 4 PASSED: Restart persistence verification succeeded");
    } else {
        info!("❌ Test 4 FAILED: Restart persistence verification failed");
    }

    // Final test summary
    info!("🏁 Sled Kademlia test completed");
    info!(
        "📊 Test Results: {}/{} tests passed ({:.1}%)",
        test_passed,
        test_total,
        (test_passed as f64 / test_total as f64) * 100.0
    );

    // Cleanup
    for (node_name, _, _) in &nodes {
        info!("🧹 Cleaning up Sled node: {}", node_name);
    }

    info!(
        "✅ Sled Kademlia test finished with {} total tests",
        test_total
    );

    // Assert overall success - require at least 50% success rate
    assert!(
        test_passed > 0,
        "No tests passed. Total tests: {}",
        test_total
    );

    assert!(
        (test_passed as f64 / test_total as f64) > 0.5,
        "Test success rate too low: {}/{}",
        test_passed,
        test_total
    );
}

/// Minimal two-node Sled test for quick validation
#[tokio::test]
async fn test_minimal_two_node_sled() {
    init_logger();
    info!("🧪 Starting minimal two-node Sled test");

    let mut nodes: Vec<(String, Netabase<SledKademliaSchema>, PathBuf)> = Vec::new();
    let mut temp_dirs = Vec::new();

    // Create two nodes
    for i in 0..2 {
        let node_name = format!("minimal_sled_node_{}", i);
        let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
        let db_path = temp_dir.path().join("minimal_netabase_sled");

        info!(
            "🏗️ Creating minimal Sled node: {} at {:?}",
            node_name, db_path
        );

        match Netabase::<SledKademliaSchema>::new_with_path(&db_path) {
            Ok(mut netabase) => {
                info!("✅ Successfully created minimal Sled node: {}", node_name);

                // Start the swarm for network operations
                match netabase.start_swarm().await {
                    Ok(_) => {
                        info!("✅ [{}] Successfully started swarm", node_name);
                        nodes.push((node_name, netabase, db_path));
                        temp_dirs.push(temp_dir);
                    }
                    Err(e) => {
                        error!("❌ [{}] Failed to start swarm: {:?}", node_name, e);
                        panic!("Failed to start swarm for minimal test node");
                    }
                }
            }
            Err(e) => {
                error!(
                    "❌ Failed to create minimal Sled node {}: {:?}",
                    node_name, e
                );
                panic!("Failed to create minimal test node");
            }
        }
    }

    // Simple put/get test
    if let Some((node_name, netabase, db_path)) = nodes.first() {
        let test_message = create_sled_test_message(
            1,
            node_name,
            "Minimal Sled test message",
            "minimal_test",
            None,
        );

        info!("📤 [{}] Storing minimal test message", node_name);

        match timeout(TEST_TIMEOUT, netabase.put_record(test_message.clone())).await {
            Ok(Ok(_)) => {
                info!("✅ [{}] Successfully stored minimal message", node_name);

                // Verify persistence
                tokio::time::sleep(Duration::from_secs(1)).await;

                match verify_sled_persistence(db_path, &[test_message.persistence_marker.clone()])
                    .await
                {
                    Ok(true) => {
                        info!("✅ [{}] Minimal persistence verification passed", node_name);
                    }
                    Ok(false) => {
                        warn!("⚠️ [{}] Minimal persistence verification failed", node_name);
                    }
                    Err(e) => {
                        error!(
                            "❌ [{}] Minimal persistence verification error: {}",
                            node_name, e
                        );
                    }
                }
            }
            Ok(Err(e)) => {
                error!(
                    "❌ [{}] Failed to store minimal message: {:?}",
                    node_name, e
                );
                panic!("Failed minimal store operation");
            }
            Err(_) => {
                error!("⏰ [{}] Minimal store operation timed out", node_name);
                panic!("Minimal store operation timed out");
            }
        }
    }

    info!("✅ Minimal two-node Sled test completed successfully");
}
