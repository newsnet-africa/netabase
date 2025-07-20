//! Cross-machine network testing for NetaBase DHT functionality
//!
//! This module provides tests for distributed hash table operations across multiple machines.
//! It supports both environment variables and command-line arguments for configuration.

use std::time::Duration;

use anyhow::Result;
use libp2p::futures::StreamExt;
use libp2p::kad::{QueryResult, RecordKey};
use libp2p::{Multiaddr, PeerId};
use log::{error, info, warn};
use netabase::{
    config::{local_config_from_env, reader_config_from_env, writer_config_from_env},
    get_test_temp_dir_str,
    network::swarm::generate_swarm,
};
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

/// Run a writer node that stores records in the DHT
async fn run_writer_node(
    listen_addr: std::net::SocketAddr,
    key: RecordKey,
    values: Vec<String>,
    timeout: Option<Duration>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        info!("Starting writer node on address: {}", listen_addr);
        info!(
            "Writer will store {} values under key: {:?}",
            values.len(),
            key
        );
    }

    let temp_dir = get_test_temp_dir_str(Some("cross_writer"));
    let mut swarm = generate_swarm(&temp_dir)?;

    // Convert SocketAddr to Multiaddr
    let listen_multiaddr: Multiaddr = format!(
        "/ip4/{}/udp/{}/quic-v1",
        listen_addr.ip(),
        listen_addr.port()
    )
    .parse()?;

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
                warn!("Writer: Failed to initiate put for record {}: {:?}", i, e);
            }
        }
    }

    info!("Writer: All {} records queued for storage", total_values);
    info!("Writer: Now listening for connections and serving requests...");

    if timeout.is_some() {
        info!("Writer: Will run for {:?}", timeout.unwrap());
    } else {
        info!("Writer: Press Ctrl+C to stop the writer node");
    }

    let start_time = std::time::Instant::now();

    // Main event loop
    loop {
        // Check timeout if specified
        if let Some(timeout_duration) = timeout {
            if start_time.elapsed() > timeout_duration {
                info!("Writer: Timeout reached, shutting down");
                break;
            }
        }

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
                    warn!("Writer: Failed to store record: {:?}", e);
                }
                _ => {}
            },
            libp2p::swarm::SwarmEvent::Behaviour(
                netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                    libp2p::kad::Event::InboundRequest { request },
                ),
            ) => {
                if verbose {
                    info!("Writer: Received inbound request: {:?}", request);
                }
            }
            _ => {}
        }

        // Add a small delay to prevent busy waiting
        tokio::task::yield_now().await;
    }

    Ok(())
}

/// Run a writer node with a ready signal for coordination with reader
async fn run_writer_node_with_ready_signal(
    listen_addr: std::net::SocketAddr,
    key: RecordKey,
    values: Vec<String>,
    ready_tx: tokio::sync::oneshot::Sender<PeerId>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        info!(
            "Starting coordinated writer node on address: {}",
            listen_addr
        );
    }

    let temp_dir = get_test_temp_dir_str(Some("cross_writer_coordinated"));
    let mut swarm = generate_swarm(&temp_dir)?;

    // Convert SocketAddr to Multiaddr
    let listen_multiaddr: Multiaddr = format!(
        "/ip4/{}/udp/{}/quic-v1",
        listen_addr.ip(),
        listen_addr.port()
    )
    .parse()?;

    swarm.listen_on(listen_multiaddr)?;

    // Set to server mode
    swarm
        .behaviour_mut()
        .kad
        .set_mode(Some(libp2p::kad::Mode::Server));

    let peer_id = *swarm.local_peer_id();
    info!("Writer node peer ID: {}", peer_id);

    let mut ready_signal_sent = false;
    let mut stored_count = 0;
    let total_values = values.len();

    // Store all records
    for (i, value) in values.iter().enumerate() {
        let record = libp2p::kad::Record::new(key.clone(), value.as_bytes().to_vec());
        match swarm
            .behaviour_mut()
            .kad
            .put_record(record, libp2p::kad::Quorum::One)
        {
            Ok(query_id) => {
                info!(
                    "Writer: Queued record {}/{}: '{}' (QueryId: {:?})",
                    i + 1,
                    total_values,
                    value,
                    query_id
                );
            }
            Err(e) => {
                warn!("Writer: Failed to queue record {}: {:?}", i, e);
            }
        }
    }

    // Main event loop
    loop {
        let event = swarm.select_next_some().await;

        match event {
            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                info!("Writer: Listening on {}", address);

                // Send ready signal after we start listening
                if !ready_signal_sent {
                    if let Err(_) = ready_tx.send(peer_id) {
                        warn!("Writer: Failed to send ready signal");
                    } else {
                        info!("Writer: Ready signal sent");
                    }
                    ready_signal_sent = true;
                    return Ok(()); // Exit after sending ready signal
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
                    warn!("Writer: Failed to store record: {:?}", e);
                }
                _ => {}
            },
            libp2p::swarm::SwarmEvent::Behaviour(
                netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                    libp2p::kad::Event::InboundRequest { request },
                ),
            ) => {
                if verbose {
                    info!("Writer: Received inbound request: {:?}", request);
                }
            }
            _ => {}
        }

        tokio::task::yield_now().await;
    }
}

/// Run a reader node that retrieves records from the DHT
async fn run_reader_node(
    connect_addr: std::net::SocketAddr,
    key: RecordKey,
    expected_values: Vec<String>,
    timeout_duration: Duration,
    retries: u32,
    verbose: bool,
) -> Result<()> {
    info!(
        "Starting reader node, connecting to writer at: {}",
        connect_addr
    );

    let temp_dir = get_test_temp_dir_str(Some("cross_reader"));
    let mut swarm = generate_swarm(&temp_dir)?;

    let reader_peer_id = *swarm.local_peer_id();
    info!("Reader node peer ID: {}", reader_peer_id);

    // Convert SocketAddr to Multiaddr for dialing
    let dial_multiaddr: Multiaddr = format!(
        "/ip4/{}/udp/{}/quic-v1",
        connect_addr.ip(),
        connect_addr.port()
    )
    .parse()?;

    info!("Reader: Attempting to dial writer at: {}", dial_multiaddr);

    // Dial the writer
    match swarm.dial(dial_multiaddr) {
        Ok(_) => info!("Reader: Dial initiated successfully"),
        Err(e) => {
            error!("Reader: Failed to initiate dial: {:?}", e);
            return Err(e.into());
        }
    }

    let mut connected = false;
    let mut query_sent = false;
    let mut found_values = Vec::new();
    let mut attempts = 0;
    let max_attempts = retries + 1;

    let start_time = std::time::Instant::now();

    // Main event loop with timeout
    loop {
        if start_time.elapsed() > timeout_duration {
            error!("Reader: Timeout reached after {:?}", timeout_duration);
            break;
        }

        let event_future = swarm.select_next_some();
        let event_result = timeout(Duration::from_secs(1), event_future).await;

        match event_result {
            Ok(event) => match event {
                libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    info!("Reader: Connected to peer: {}", peer_id);
                    connected = true;
                }
                libp2p::swarm::SwarmEvent::Behaviour(
                    netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                        libp2p::kad::Event::OutboundQueryProgressed { result, .. },
                    ),
                ) => match result {
                    QueryResult::GetRecord(Ok(libp2p::kad::GetRecordOk::FoundRecord(
                        peer_record,
                    ))) => {
                        info!("Reader: Query completed successfully");
                        if let Ok(value_str) = String::from_utf8(peer_record.record.value) {
                            info!("Reader: Found record: '{}'", value_str);
                            found_values.push(value_str);
                        }
                        break;
                    }
                    QueryResult::GetRecord(Ok(
                        libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. },
                    )) => {
                        info!("Reader: Query finished with no additional records");
                        break;
                    }
                    QueryResult::GetRecord(Err(e)) => {
                        warn!("Reader: Get record failed: {:?}", e);
                        attempts += 1;

                        if attempts < max_attempts {
                            info!("Reader: Retrying ({}/{})", attempts, max_attempts - 1);
                            sleep(Duration::from_secs(2)).await;
                            query_sent = false; // Allow retry
                        } else {
                            error!("Reader: All retry attempts exhausted");
                            break;
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            Err(_) => {
                // Timeout on event, check if we need to send query
                if connected && !query_sent && attempts < max_attempts {
                    info!("Reader: Attempting to get record with key: {:?}", key);

                    let query_id = swarm.behaviour_mut().kad.get_record(key.clone());
                    info!("Reader: Get query initiated with ID: {:?}", query_id);
                    query_sent = true;
                }
            }
        }

        tokio::task::yield_now().await;
    }

    // Verify results
    info!("Reader: Verification phase");
    let mut missing_values = Vec::new();
    let mut found_expected = 0;

    for expected in &expected_values {
        if found_values.contains(expected) {
            info!("Reader: ✓ Found expected value: '{}'", expected);
            found_expected += 1;
        } else {
            warn!("Reader: ✗ Missing expected value: '{}'", expected);
            missing_values.push(expected.clone());
        }
    }

    info!(
        "Reader: Found {}/{} expected values",
        found_expected,
        expected_values.len()
    );

    if found_expected == expected_values.len() {
        info!("Reader: Cross-machine reader test completed successfully!");
        Ok(())
    } else {
        error!(
            "Reader: Test failed - missing {} values: {:?}",
            missing_values.len(),
            missing_values
        );
        Err(anyhow::anyhow!(
            "Expected {} values but found only {}",
            expected_values.len(),
            found_expected
        ))
    }
}

/// Cross-machine writer test
/// Runs a writer node that stores records and serves them to readers
#[ignore]
#[tokio::test]
async fn cross_machine_writer() -> Result<()> {
    init_logging();

    let config = writer_config_from_env().map_err(|e| {
        error!("Failed to parse writer configuration: {}", e);
        e
    })?;

    info!("=== CROSS-MACHINE WRITER TEST ===");
    info!("Writer address: {}", config.address);
    info!("Test key: {:?}", RecordKey::new(&config.test_key));
    info!("Test values: {:?}", config.test_values);
    if let Some(timeout) = config.timeout {
        info!("Timeout: {:?}", timeout);
    } else {
        info!("Timeout: None (runs indefinitely)");
    }
    info!("=====================================");

    let key = RecordKey::new(&config.test_key);

    run_writer_node(
        config.address,
        key,
        config.test_values,
        config.timeout,
        config.verbose,
    )
    .await
}

/// Cross-machine reader test
/// Connects to a writer node and retrieves records
#[ignore]
#[tokio::test]
async fn cross_machine_reader() -> Result<()> {
    init_logging();

    let config = reader_config_from_env().map_err(|e| {
        error!("Failed to parse reader configuration: {}", e);
        e
    })?;

    info!("=== CROSS-MACHINE READER TEST ===");
    info!("Connecting to writer at: {}", config.connect_addr);
    info!("Test key: {:?}", RecordKey::new(&config.test_key));
    info!("Expected values: {:?}", config.test_values);
    info!("Timeout: {:?}", config.timeout);
    info!("Retries: {}", config.retries);
    info!("===================================");

    let key = RecordKey::new(&config.test_key);

    run_reader_node(
        config.connect_addr,
        key,
        config.test_values,
        config.timeout,
        config.retries,
        config.verbose,
    )
    .await
}

/// Local test that runs both writer and reader on the same machine
#[ignore]
#[tokio::test]
async fn cross_machine_local_test() -> Result<()> {
    init_logging();

    let config = local_config_from_env().map_err(|e| {
        error!("Failed to parse local test configuration: {}", e);
        e
    })?;

    info!("=== CROSS-MACHINE LOCAL TEST ===");
    info!("Test key: {}", config.test_key);
    info!("Test values: {:?}", config.test_values);
    info!("Timeout: {:?}", config.timeout);
    info!("==================================");

    let key = RecordKey::new(&config.test_key);
    let listen_addr = "127.0.0.1:0".parse::<std::net::SocketAddr>().unwrap();

    // Channel for coordination between writer and reader
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<PeerId>();

    let writer_values = config.test_values.clone();
    let reader_values = config.test_values.clone();
    let writer_key = key.clone();
    let reader_key = key.clone();
    let reader_timeout = config.timeout;
    let verbose = config.verbose;

    // Start writer in a separate task
    let writer_task = tokio::spawn(async move {
        run_writer_node_with_ready_signal(listen_addr, writer_key, writer_values, ready_tx, verbose)
            .await
    });

    // Wait for writer to be ready and get its peer ID
    let _writer_peer_id = timeout(Duration::from_secs(10), ready_rx)
        .await
        .map_err(|_| anyhow::anyhow!("Writer ready timeout"))??;

    info!("Local test: Writer is ready, starting reader");

    // Small delay to ensure writer is fully ready
    sleep(Duration::from_secs(1)).await;

    // Run reader (this will complete and return)
    let reader_result = run_reader_node(
        "127.0.0.1:9901".parse().unwrap(), // Use fixed port for local test
        reader_key,
        reader_values,
        reader_timeout,
        3, // Default retries for local test
        verbose,
    )
    .await;

    // Cancel writer task
    writer_task.abort();

    match reader_result {
        Ok(_) => {
            info!("Local test: Completed successfully!");
            Ok(())
        }
        Err(e) => {
            error!("Local test: Failed - {}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netabase::config::*;

    #[test]
    fn test_environment_parsing() {
        // Test that configuration parsing works with various inputs

        // Test writer config with defaults
        let writer_config = WriterConfig {
            address: None,
            test_key: None,
            test_values: None,
            timeout: None,
            verbose: false,
        };

        let validated = writer_config.validate();
        assert!(validated.is_ok());

        let config = validated.unwrap();
        assert_eq!(config.address.to_string(), "0.0.0.0:9901");
        assert_eq!(config.test_key, "cross_machine_key");

        // Test reader config with custom values
        let reader_config = ReaderConfig {
            connect_addr: Some("192.168.1.100:8080".to_string()),
            test_key: Some("test_key".to_string()),
            test_values: Some("val1,val2,val3".to_string()),
            timeout: Some(30),
            retries: Some(5),
            verbose: true,
        };

        let validated = reader_config.validate();
        assert!(validated.is_ok());

        let config = validated.unwrap();
        assert_eq!(config.connect_addr.to_string(), "192.168.1.100:8080");
        assert_eq!(config.test_key, "test_key");
        assert_eq!(config.test_values, vec!["val1", "val2", "val3"]);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.retries, 5);
        assert!(config.verbose);
    }

    #[test]
    fn test_config_validation_errors() {
        // Test invalid timeout
        let reader_config = ReaderConfig {
            connect_addr: Some("127.0.0.1:9901".to_string()),
            test_key: Some("test".to_string()),
            test_values: Some("val1,val2".to_string()),
            timeout: Some(0), // Invalid: zero timeout
            retries: Some(1),
            verbose: false,
        };

        let result = reader_config.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Timeout must be greater than 0")
        );
    }
}
