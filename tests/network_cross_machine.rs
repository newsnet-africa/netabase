use std::env;
use std::time::Duration;

use anyhow::Result;
use libp2p::futures::StreamExt;
use libp2p::kad::{QueryResult, RecordKey};
use libp2p::{Multiaddr, PeerId};
use log::info;
use netabase::{get_test_temp_dir_str, network::swarm::generate_swarm};
use tokio::time::{sleep, timeout};

fn init_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();
    });
}

/// Get writer listen address from environment or use default
fn get_writer_address() -> String {
    env::var("NETABASE_WRITER_ADDR").unwrap_or_else(|_| "0.0.0.0:9901".to_string())
}

/// Get reader connect address from environment or use default
fn get_reader_connect_address() -> String {
    env::var("NETABASE_READER_CONNECT_ADDR").unwrap_or_else(|_| "127.0.0.1:9901".to_string())
}

/// Get test key from environment or use default
fn get_test_key() -> String {
    env::var("NETABASE_TEST_KEY").unwrap_or_else(|_| "cross_machine_key".to_string())
}

/// Get test values from environment (comma-separated) or use defaults
fn get_test_values() -> Vec<String> {
    env::var("NETABASE_TEST_VALUES")
        .unwrap_or_else(|_| "Value1,Value2,Value3".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect()
}

/// Get test timeout from environment or use default (in seconds)
fn get_test_timeout() -> u64 {
    env::var("NETABASE_TEST_TIMEOUT")
        .unwrap_or_else(|_| "120".to_string())
        .parse()
        .unwrap_or(120)
}

async fn run_writer_node(listen_addr: String, key: RecordKey, values: Vec<String>) -> Result<()> {
    info!("Starting writer node on address: {}", listen_addr);

    let temp_dir = get_test_temp_dir_str(Some("cross_writer"));
    let mut swarm = generate_swarm(&temp_dir)?;

    // Parse and listen on the specified address
    let parts: Vec<&str> = listen_addr.split(':').collect();
    let ip = parts[0];
    let port = parts[1];
    let listen_multiaddr: Multiaddr = format!("/ip4/{}/udp/{}/quic-v1", ip, port).parse()?;
    swarm.listen_on(listen_multiaddr)?;

    // Set to server mode to accept and store records
    swarm
        .behaviour_mut()
        .kad
        .set_mode(Some(libp2p::kad::Mode::Server));

    info!("Writer node peer ID: {}", swarm.local_peer_id());

    // Wait for listening address to be established
    let mut _actual_listen_addr = None;
    let mut put_query_ids = Vec::new();
    let mut stored_count = 0;
    let total_values = values.len();

    // Store all records first
    for (i, value) in values.iter().enumerate() {
        let record = libp2p::kad::Record::new(key.clone(), value.as_bytes().to_vec());
        match swarm
            .behaviour_mut()
            .kad
            .put_record(record, libp2p::kad::Quorum::One)
        {
            Ok(query_id) => {
                info!(
                    "Writer: Stored record {}/{}: '{}' (QueryId: {:?})",
                    i + 1,
                    total_values,
                    value,
                    query_id
                );
                put_query_ids.push(query_id);
            }
            Err(e) => {
                info!("Writer: Failed to initiate put for record {}: {:?}", i, e);
            }
        }
    }

    info!("Writer: All {} records queued for storage", total_values);
    info!("Writer: Now listening for connections and serving requests...");
    info!("Writer: Press Ctrl+C to stop the writer node");

    // Main event loop
    loop {
        let event = swarm.select_next_some().await;

        match event {
            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                info!("Writer: Listening on {}", address);
                _actual_listen_addr = Some(address.clone());
            }
            libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Writer: Connected to peer: {}", peer_id);
            }
            libp2p::swarm::SwarmEvent::Behaviour(
                netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                    libp2p::kad::Event::OutboundQueryProgressed { result, .. },
                ),
            ) => match result {
                QueryResult::PutRecord(Ok(_)) => {
                    stored_count += 1;
                    info!(
                        "Writer: Successfully stored record ({}/{})",
                        stored_count, total_values
                    );
                }
                QueryResult::PutRecord(Err(e)) => {
                    info!("Writer: Failed to store record: {:?}", e);
                }
                _ => {}
            },
            libp2p::swarm::SwarmEvent::Behaviour(
                netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                    libp2p::kad::Event::InboundRequest { request },
                ),
            ) => {
                info!("Writer: Received inbound request: {:?}", request);
            }
            _ => {}
        }

        // Add a small delay to prevent busy waiting
        tokio::task::yield_now().await;
    }
}

async fn run_writer_node_with_ready_signal(
    listen_addr: String,
    key: RecordKey,
    values: Vec<String>,
    ready_tx: tokio::sync::mpsc::Sender<bool>,
) -> Result<()> {
    info!("Starting writer node on address: {}", listen_addr);

    let temp_dir = get_test_temp_dir_str(Some("cross_writer"));
    let mut swarm = generate_swarm(&temp_dir)?;

    // Parse and listen on the specified address
    let parts: Vec<&str> = listen_addr.split(':').collect();
    let ip = parts[0];
    let port = parts[1];
    let listen_multiaddr: Multiaddr = format!("/ip4/{}/udp/{}/quic-v1", ip, port).parse()?;
    swarm.listen_on(listen_multiaddr)?;

    // Set to server mode to accept and store records
    swarm
        .behaviour_mut()
        .kad
        .set_mode(Some(libp2p::kad::Mode::Server));

    info!("Writer node peer ID: {}", swarm.local_peer_id());

    // Wait for listening address to be established and store records
    let mut _actual_listen_addr = None;
    let mut put_query_ids = Vec::new();
    let mut stored_count = 0;
    let total_values = values.len();
    let mut ready_sent = false;

    // Store all records first
    for (i, value) in values.iter().enumerate() {
        let record = libp2p::kad::Record::new(key.clone(), value.as_bytes().to_vec());
        match swarm
            .behaviour_mut()
            .kad
            .put_record(record, libp2p::kad::Quorum::One)
        {
            Ok(query_id) => {
                info!(
                    "Writer: Stored record {}/{}: '{}' (QueryId: {:?})",
                    i + 1,
                    total_values,
                    value,
                    query_id
                );
                put_query_ids.push(query_id);
            }
            Err(e) => {
                info!("Writer: Failed to initiate put for record {}: {:?}", i, e);
            }
        }
    }

    info!("Writer: All {} records queued for storage", total_values);

    // Main event loop with timeout for local testing
    let start_time = tokio::time::Instant::now();
    let max_runtime = Duration::from_secs(90); // Run for 90 seconds max

    loop {
        if start_time.elapsed() > max_runtime {
            info!("Writer: Max runtime reached, shutting down");
            break;
        }

        let event_result = timeout(Duration::from_secs(1), swarm.select_next_some()).await;

        match event_result {
            Ok(event) => {
                match event {
                    libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Writer: Listening on {}", address);
                        _actual_listen_addr = Some(address.clone());

                        // Signal ready after we start listening
                        if !ready_sent {
                            let _ = ready_tx.send(true).await;
                            ready_sent = true;
                            info!("Writer: Ready signal sent");
                        }
                    }
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("Writer: Connected to peer: {}", peer_id);
                    }
                    libp2p::swarm::SwarmEvent::Behaviour(
                        netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                            libp2p::kad::Event::OutboundQueryProgressed { result, .. },
                        ),
                    ) => match result {
                        QueryResult::PutRecord(Ok(_)) => {
                            stored_count += 1;
                            info!(
                                "Writer: Successfully stored record ({}/{})",
                                stored_count, total_values
                            );
                        }
                        QueryResult::PutRecord(Err(e)) => {
                            info!("Writer: Failed to store record: {:?}", e);
                        }
                        _ => {}
                    },
                    libp2p::swarm::SwarmEvent::Behaviour(
                        netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                            libp2p::kad::Event::InboundRequest { request },
                        ),
                    ) => {
                        info!("Writer: Received inbound request: {:?}", request);
                    }
                    _ => {}
                }
            }
            Err(_) => {
                // Timeout on select_next_some - continue loop
                tokio::task::yield_now().await;
            }
        }
    }

    info!("Writer: Shutting down");
    Ok(())
}

async fn run_reader_node(
    writer_addr: String,
    key: RecordKey,
    expected_values: Vec<String>,
) -> Result<Vec<String>> {
    info!(
        "Starting reader node, connecting to writer at: {}",
        writer_addr
    );

    let temp_dir = get_test_temp_dir_str(Some("cross_reader"));
    let mut swarm = generate_swarm(&temp_dir)?;

    // Listen on any available port
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;

    // Set to client mode
    swarm
        .behaviour_mut()
        .kad
        .set_mode(Some(libp2p::kad::Mode::Client));

    info!("Reader node peer ID: {}", swarm.local_peer_id());

    // Parse writer address and peer ID (we'll discover the peer ID through connection)
    let parts: Vec<&str> = writer_addr.split(':').collect();
    let ip = parts[0];
    let port = parts[1];
    let writer_multiaddr: Multiaddr = format!("/ip4/{}/udp/{}/quic-v1", ip, port).parse()?;

    info!("Reader: Attempting to dial writer at: {}", writer_multiaddr);
    swarm.dial(writer_multiaddr.clone())?;

    let mut found_records = Vec::new();
    let mut connected_to_writer = false;
    let mut writer_peer_id: Option<PeerId> = None;
    let timeout_duration = Duration::from_secs(get_test_timeout());

    let start_time = tokio::time::Instant::now();

    // Main event loop with timeout
    loop {
        if start_time.elapsed() > timeout_duration {
            info!("Reader: Timeout reached, stopping");
            break;
        }

        let event_result = timeout(Duration::from_secs(1), swarm.select_next_some()).await;

        match event_result {
            Ok(event) => {
                match event {
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("Reader: Connected to peer: {}", peer_id);
                        connected_to_writer = true;
                        writer_peer_id = Some(peer_id);

                        // Add the peer to our routing table
                        swarm
                            .behaviour_mut()
                            .kad
                            .add_address(&peer_id, writer_multiaddr.clone());

                        // Wait a bit for the connection to stabilize
                        sleep(Duration::from_secs(2)).await;

                        // Now try to get the record
                        info!("Reader: Attempting to get record with key: {:?}", key);
                        swarm.behaviour_mut().kad.get_record(key.clone());
                    }
                    libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                        info!("Reader: Connection to {} closed: {:?}", peer_id, cause);
                        if Some(peer_id) == writer_peer_id {
                            connected_to_writer = false;
                        }
                    }
                    libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Reader: Listening on {}", address);
                    }
                    libp2p::swarm::SwarmEvent::Behaviour(
                        netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                            libp2p::kad::Event::OutboundQueryProgressed { result, .. },
                        ),
                    ) => {
                        match result {
                            QueryResult::GetRecord(Ok(libp2p::kad::GetRecordOk::FoundRecord(
                                peer_record,
                            ))) => {
                                let value =
                                    String::from_utf8_lossy(&peer_record.record.value).to_string();
                                info!("Reader: Found record: '{}'", value);
                                found_records.push(value.clone());

                                // Check if we've found all expected records
                                if found_records.len() >= expected_values.len() {
                                    info!("Reader: Found all expected records, stopping");
                                    break;
                                }

                                // Try to get more records by querying again
                                sleep(Duration::from_millis(500)).await;
                                swarm.behaviour_mut().kad.get_record(key.clone());
                            }
                            QueryResult::GetRecord(Ok(
                                libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. },
                            )) => {
                                info!("Reader: Get query finished with no additional records");

                                // If we haven't found any records yet, try again
                                if found_records.is_empty() && connected_to_writer {
                                    sleep(Duration::from_secs(2)).await;
                                    info!("Reader: Retrying get record query");
                                    swarm.behaviour_mut().kad.get_record(key.clone());
                                }
                            }
                            QueryResult::GetRecord(Err(e)) => {
                                info!("Reader: Get record failed: {:?}", e);

                                // If we're connected, try again after a delay
                                if connected_to_writer && found_records.is_empty() {
                                    sleep(Duration::from_secs(2)).await;
                                    info!("Reader: Retrying after error");
                                    swarm.behaviour_mut().kad.get_record(key.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Err(_) => {
                // Timeout on event, continue loop
                if connected_to_writer && found_records.is_empty() {
                    // Periodically retry getting records
                    info!("Reader: No events, retrying get record");
                    swarm.behaviour_mut().kad.get_record(key.clone());
                }
            }
        }

        tokio::task::yield_now().await;
    }

    info!("Reader: Finished, found {} records", found_records.len());
    for (i, record) in found_records.iter().enumerate() {
        info!("Reader: Record {}: '{}'", i + 1, record);
    }

    Ok(found_records)
}

/// Cross-machine writer test
///
/// Environment variables:
/// - NETABASE_WRITER_ADDR: Address to listen on (default: "0.0.0.0:9901")
/// - NETABASE_TEST_KEY: Key to store records under (default: "cross_machine_key")
/// - NETABASE_TEST_VALUES: Comma-separated values to store (default: "Value1,Value2,Value3")
///
/// Example usage:
/// ```bash
/// # On writer machine:
/// NETABASE_WRITER_ADDR=0.0.0.0:9901 NETABASE_TEST_KEY=mykey NETABASE_TEST_VALUES="Hello,World,Test" cargo test cross_machine_writer -- --nocapture
/// ```
#[tokio::test]
#[ignore] // Use --ignored to run cross-machine tests
async fn cross_machine_writer() {
    init_logging();

    let writer_addr = get_writer_address();
    let key = RecordKey::new(&get_test_key());
    let values = get_test_values();

    info!("=== CROSS-MACHINE WRITER TEST ===");
    info!("Writer address: {}", writer_addr);
    info!("Test key: {:?}", key);
    info!("Test values: {:?}", values);
    info!("=====================================");

    match run_writer_node(writer_addr, key, values).await {
        Ok(_) => info!("Writer node completed successfully"),
        Err(e) => {
            info!("Writer node failed: {:?}", e);
            panic!("Writer node failed: {:?}", e);
        }
    }
}

/// Cross-machine reader test
///
/// Environment variables:
/// - NETABASE_READER_CONNECT_ADDR: Writer address to connect to (default: "127.0.0.1:9901")
/// - NETABASE_TEST_KEY: Key to retrieve records from (default: "cross_machine_key")
/// - NETABASE_TEST_VALUES: Expected comma-separated values (default: "Value1,Value2,Value3")
/// - NETABASE_TEST_TIMEOUT: Timeout in seconds (default: "120")
///
/// Example usage:
/// ```bash
/// # On reader machine:
/// NETABASE_READER_CONNECT_ADDR=192.168.1.100:9901 NETABASE_TEST_KEY=mykey NETABASE_TEST_VALUES="Hello,World,Test" cargo test cross_machine_reader -- --nocapture
/// ```
#[tokio::test]
#[ignore] // Use --ignored to run cross-machine tests
async fn cross_machine_reader() {
    init_logging();

    let writer_addr = get_reader_connect_address();
    let key = RecordKey::new(&get_test_key());
    let expected_values = get_test_values();
    let timeout_secs = get_test_timeout();

    info!("=== CROSS-MACHINE READER TEST ===");
    info!("Connecting to writer at: {}", writer_addr);
    info!("Test key: {:?}", key);
    info!("Expected values: {:?}", expected_values);
    info!("Timeout: {} seconds", timeout_secs);
    info!("===================================");

    match run_reader_node(writer_addr, key, expected_values.clone()).await {
        Ok(found_records) => {
            info!("Reader completed successfully");
            info!("Found {} records", found_records.len());

            // Verify we found at least one record
            assert!(
                !found_records.is_empty(),
                "Should have found at least one record"
            );

            // Log which expected values were found
            for expected in &expected_values {
                if found_records.contains(expected) {
                    info!("✓ Found expected value: '{}'", expected);
                } else {
                    info!("✗ Missing expected value: '{}'", expected);
                }
            }

            info!("Cross-machine reader test completed successfully!");
        }
        Err(e) => {
            info!("Reader node failed: {:?}", e);
            panic!("Reader node failed: {:?}", e);
        }
    }
}

/// Combined test that runs both writer and reader on the same machine (for testing the cross-machine setup)
#[tokio::test]
// #[ignore]
async fn cross_machine_local_test() {
    init_logging();

    let key = RecordKey::new(&"local_test_key");
    let values = vec!["LocalValue1".to_string(), "LocalValue2".to_string()];

    info!("=== CROSS-MACHINE LOCAL TEST ===");
    info!("Testing cross-machine setup on local machine");
    info!("Test key: {:?}", key);
    info!("Test values: {:?}", values);
    info!("=================================");

    // Use a channel to coordinate between writer and reader
    let (writer_ready_tx, mut writer_ready_rx) = tokio::sync::mpsc::channel::<bool>(1);

    // Start writer in background
    let writer_values = values.clone();
    let writer_key = key.clone();
    let writer_handle = tokio::spawn(async move {
        match run_writer_node_with_ready_signal(
            "127.0.0.1:9902".to_string(),
            writer_key,
            writer_values,
            writer_ready_tx,
        )
        .await
        {
            Ok(_) => info!("Writer completed successfully"),
            Err(e) => info!("Writer failed: {:?}", e),
        }
    });

    // Wait for writer to be ready (with timeout)
    let writer_ready = timeout(Duration::from_secs(15), writer_ready_rx.recv()).await;
    match writer_ready {
        Ok(Some(true)) => {
            info!("Writer is ready, starting reader");
        }
        _ => {
            info!("Writer didn't signal ready in time, proceeding anyway");
        }
    }

    // Give writer a bit more time to ensure it's fully ready
    sleep(Duration::from_secs(2)).await;

    // Start reader
    let reader_result = timeout(Duration::from_secs(60), async {
        run_reader_node("127.0.0.1:9902".to_string(), key, values.clone()).await
    })
    .await;

    // Stop writer
    writer_handle.abort();

    match reader_result {
        Ok(Ok(found_records)) => {
            info!("Local test completed successfully");
            info!("Found {} records", found_records.len());
            assert!(
                !found_records.is_empty(),
                "Should have found at least one record"
            );

            for record in &found_records {
                info!("Found record: '{}'", record);
            }
        }
        Ok(Err(e)) => {
            panic!("Reader failed: {:?}", e);
        }
        Err(_) => {
            panic!("Local test timed out");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_parsing() {
        // Test default values
        unsafe {
            std::env::remove_var("NETABASE_WRITER_ADDR");
        }
        assert_eq!(get_writer_address(), "0.0.0.0:9901");

        unsafe {
            std::env::remove_var("NETABASE_READER_CONNECT_ADDR");
        }
        assert_eq!(get_reader_connect_address(), "127.0.0.1:9901");

        unsafe {
            std::env::remove_var("NETABASE_TEST_KEY");
        }
        assert_eq!(get_test_key(), "cross_machine_key");

        unsafe {
            std::env::remove_var("NETABASE_TEST_VALUES");
        }
        assert_eq!(get_test_values(), vec!["Value1", "Value2", "Value3"]);

        // Test custom values
        unsafe {
            std::env::set_var("NETABASE_WRITER_ADDR", "192.168.1.100:8080");
        }
        assert_eq!(get_writer_address(), "192.168.1.100:8080");

        unsafe {
            std::env::set_var("NETABASE_TEST_VALUES", "A,B,C,D");
        }
        assert_eq!(get_test_values(), vec!["A", "B", "C", "D"]);

        // Clean up
        unsafe {
            std::env::remove_var("NETABASE_WRITER_ADDR");
        }
        unsafe {
            std::env::remove_var("NETABASE_TEST_VALUES");
        }
    }
}
