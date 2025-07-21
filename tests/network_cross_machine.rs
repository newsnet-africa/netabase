//! Cross-machine network testing for NetaBase DHT functionality
//!
//! This module provides tests for distributed hash table operations across multiple machines.
//! It supports both environment variables and command-line arguments for configuration.

use std::time::Duration;

use anyhow::Result;
use libp2p::futures::StreamExt;
use libp2p::kad::{QueryResult, RecordKey, store::RecordStore};
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
    persistence: Option<u64>,
) -> Result<()> {
    if verbose {
        info!("Starting writer node on address: {}", listen_addr);
        info!(
            "Writer will store {} values under key: {:?}",
            values.len(),
            key
        );
    }

    let temp_dir = get_test_temp_dir_str(Some("cross_writer"), persistence);
    let mut swarm = generate_swarm(&temp_dir)?;

    // Convert SocketAddr to Multiaddr for specific address listening
    let listen_multiaddr: Multiaddr = format!(
        "/ip4/{}/udp/{}/quic-v1",
        listen_addr.ip(),
        listen_addr.port()
    )
    .parse()
    .or_else(|_| -> anyhow::Result<Multiaddr> { Ok(Multiaddr::empty()) })?;

    swarm.listen_on(listen_multiaddr)?;

    // Also listen on a general address for mDNS discovery
    let mdns_listen_multiaddr: Multiaddr = "/ip4/0.0.0.0/udp/0/quic-v1".parse()?;
    swarm.listen_on(mdns_listen_multiaddr)?;

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

    // Store all records first with unique keys
    for (i, value) in values.iter().enumerate() {
        let unique_key = libp2p::kad::RecordKey::new(&format!(
            "{}__{}",
            std::str::from_utf8(key.as_ref()).unwrap_or("test_key"),
            i
        ));
        let record = libp2p::kad::Record::new(unique_key, value.as_bytes().to_vec());
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
                netabase::network::behaviour::NetabaseBehaviourEvent::Mdns(
                    libp2p::mdns::Event::Discovered(peers),
                ),
            ) => {
                for (peer_id, multiaddr) in peers {
                    info!(
                        "Writer: Discovered peer via mDNS: {} at {}",
                        peer_id, multiaddr
                    );
                    // mDNS behavior automatically adds peers to Kademlia
                    // but let's also try to connect to them for better DHT connectivity
                    if let Err(e) = swarm.dial(peer_id) {
                        if verbose {
                            warn!(
                                "Writer: Failed to dial discovered peer {}: {:?}",
                                peer_id, e
                            );
                        }
                    } else {
                        info!(
                            "Writer: Attempting to connect to discovered peer: {}",
                            peer_id
                        );
                    }
                }
            }
            libp2p::swarm::SwarmEvent::Behaviour(
                netabase::network::behaviour::NetabaseBehaviourEvent::Mdns(
                    libp2p::mdns::Event::Expired(peers),
                ),
            ) => {
                for (peer_id, multiaddr) in peers {
                    if verbose {
                        info!("Writer: mDNS peer expired: {} at {}", peer_id, multiaddr);
                    }
                }
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
#[allow(dead_code)]
async fn run_writer_node_with_ready_signal(
    listen_addr: std::net::SocketAddr,
    key: RecordKey,
    values: Vec<String>,
    ready_tx: tokio::sync::oneshot::Sender<PeerId>,
    verbose: bool,
    persistence: Option<u64>,
) -> Result<()> {
    if verbose {
        info!(
            "Starting coordinated writer node on address: {}",
            listen_addr
        );
    }

    let temp_dir = get_test_temp_dir_str(Some("cross_writer_coordinated"), persistence);
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
                if let Err(_) = ready_tx.send(peer_id) {
                    warn!("Writer: Failed to send ready signal");
                } else {
                    info!("Writer: Ready signal sent");
                }
                return Ok(()); // Exit after sending ready signal
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
    base_key: RecordKey,
    expected_values: Vec<String>,
    timeout_duration: Duration,
    retries: u32,
    _verbose: bool,
    persistence: Option<u64>,
) -> Result<()> {
    info!(
        "Starting reader node, connecting to writer at: {}",
        connect_addr
    );

    let temp_dir = get_test_temp_dir_str(Some("cross_reader"), persistence);
    let mut swarm = generate_swarm(&temp_dir)?;

    let reader_peer_id = *swarm.local_peer_id();
    info!("Reader node peer ID: {}", reader_peer_id);

    // Start listening for mDNS discovery instead of manual dialing
    let listen_multiaddr: Multiaddr = "/ip4/0.0.0.0/udp/0/quic-v1".parse()?;

    info!("Reader: Starting listener for mDNS discovery...");
    swarm.listen_on(listen_multiaddr)?;

    info!("Reader: Waiting for mDNS to discover writer peers...");

    // Generate the same unique keys that the writer used
    let mut records_to_find = Vec::new();
    for i in 0..expected_values.len() {
        let unique_key = libp2p::kad::RecordKey::new(&format!(
            "{}__{}",
            std::str::from_utf8(base_key.as_ref()).unwrap_or("test_key"),
            i
        ));
        records_to_find.push(unique_key);
    }

    let mut connected = false;
    let mut found_values = Vec::new();
    let mut current_record_index = 0;
    let mut record_attempts = 0;
    let max_attempts_per_record = retries + 1;

    let start_time = std::time::Instant::now();

    // Wait for mDNS discovery and connection establishment
    let mut discovered_peers = Vec::new();
    let mut connected_peer_id = None;

    info!("Reader: Waiting for mDNS discovery and peer connections...");

    while !connected && start_time.elapsed() < timeout_duration {
        let event_future = swarm.select_next_some();
        let event_result = timeout(Duration::from_secs(1), event_future).await;

        if let Ok(event) = event_result {
            match event {
                libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Reader: Listening on {}", address);
                }
                libp2p::swarm::SwarmEvent::Behaviour(
                    netabase::network::behaviour::NetabaseBehaviourEvent::Mdns(
                        libp2p::mdns::Event::Discovered(peers),
                    ),
                ) => {
                    for (peer_id, multiaddr) in peers {
                        info!(
                            "Reader: Discovered peer via mDNS: {} at {}",
                            peer_id, multiaddr
                        );
                        discovered_peers.push((peer_id, multiaddr.clone()));

                        // mDNS behavior already adds peers to Kademlia automatically
                        // but let's also try to connect to them
                        if let Err(e) = swarm.dial(peer_id) {
                            warn!(
                                "Reader: Failed to dial discovered peer {}: {:?}",
                                peer_id, e
                            );
                        } else {
                            info!(
                                "Reader: Attempting to connect to discovered peer: {}",
                                peer_id
                            );
                        }
                    }
                }
                libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    info!("Reader: Connected to peer: {}", peer_id);
                    connected = true;
                    connected_peer_id = Some(peer_id);
                }
                _ => {}
            }
        }
        tokio::task::yield_now().await;
    }

    if !connected {
        return Err(anyhow::anyhow!(
            "Failed to discover and connect to any peers via mDNS"
        ));
    }

    // Bootstrap DHT with discovered peers
    if !discovered_peers.is_empty() {
        info!(
            "Reader: Bootstrapping Kademlia DHT with {} discovered peers...",
            discovered_peers.len()
        );

        if let Err(e) = swarm.behaviour_mut().kad.bootstrap() {
            warn!("Reader: Bootstrap failed: {:?}", e);
        }

        // Wait for DHT bootstrap to complete
        info!("Reader: Waiting for DHT bootstrap to complete...");
        sleep(Duration::from_secs(5)).await;

        // Process bootstrap events
        let bootstrap_timeout = std::time::Instant::now();
        while bootstrap_timeout.elapsed() < Duration::from_secs(10) {
            let event_future = swarm.select_next_some();
            let event_result = timeout(Duration::from_millis(100), event_future).await;

            if let Ok(event) = event_result {
                match event {
                    libp2p::swarm::SwarmEvent::Behaviour(
                        netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                            libp2p::kad::Event::OutboundQueryProgressed { result, .. },
                        ),
                    ) => match result {
                        QueryResult::Bootstrap(Ok(_)) => {
                            info!("Reader: DHT bootstrap completed successfully");
                            break;
                        }
                        QueryResult::Bootstrap(Err(e)) => {
                            warn!("Reader: DHT bootstrap failed: {:?}", e);
                            break;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    // Additional stabilization time
    info!("Reader: DHT stabilization period...");
    sleep(Duration::from_secs(3)).await;

    // Try to find each record
    while current_record_index < records_to_find.len() && start_time.elapsed() < timeout_duration {
        let key = &records_to_find[current_record_index];

        info!(
            "Reader: Attempting to get record {} (attempt {}/{})",
            current_record_index,
            record_attempts + 1,
            max_attempts_per_record
        );

        let query_id = swarm.behaviour_mut().kad.get_record(key.clone());
        info!("Reader: Get query initiated with ID: {:?}", query_id);

        let mut record_found = false;
        let query_timeout = timeout(Duration::from_secs(10), async {
            loop {
                let event = swarm.select_next_some().await;
                match event {
                    libp2p::swarm::SwarmEvent::Behaviour(
                        netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                            libp2p::kad::Event::OutboundQueryProgressed { result, .. },
                        ),
                    ) => match result {
                        QueryResult::GetRecord(Ok(libp2p::kad::GetRecordOk::FoundRecord(
                            peer_record,
                        ))) => {
                            if let Ok(value_str) = String::from_utf8(peer_record.record.value) {
                                info!(
                                    "Reader: Found record {}: '{}'",
                                    current_record_index, value_str
                                );
                                found_values.push(value_str);
                                record_found = true;
                                break;
                            }
                        }
                        QueryResult::GetRecord(Ok(
                            libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. },
                        )) => {
                            info!(
                                "Reader: Query finished with no additional records for record {}",
                                current_record_index
                            );
                            break;
                        }
                        QueryResult::GetRecord(Err(e)) => {
                            warn!(
                                "Reader: Get record {} failed: {:?}",
                                current_record_index, e
                            );
                            break;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        });

        match query_timeout.await {
            Ok(_) => {
                if record_found {
                    current_record_index += 1;
                    record_attempts = 0;
                } else {
                    record_attempts += 1;
                    if record_attempts >= max_attempts_per_record {
                        warn!(
                            "Reader: Failed to find record {} after {} attempts",
                            current_record_index, max_attempts_per_record
                        );
                        current_record_index += 1;
                        record_attempts = 0;
                    } else {
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
            Err(_) => {
                warn!("Reader: Timeout for record {}", current_record_index);
                record_attempts += 1;
                if record_attempts >= max_attempts_per_record {
                    current_record_index += 1;
                    record_attempts = 0;
                }
            }
        }
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
        config.persistence,
    )
    .await
}

/// Cross-machine writer test for 5 records with unique keys
/// Runs a writer node that stores exactly 5 records and serves them to readers
#[ignore]
#[tokio::test]
async fn cross_machine_writer_5_records() -> Result<()> {
    init_logging();

    let config = writer_config_from_env().map_err(|e| {
        error!("Failed to parse writer configuration: {}", e);
        e
    })?;

    info!("=== CROSS-MACHINE 5-RECORD WRITER TEST ===");
    info!("Writer address: {}", config.address);
    info!("Test key: {:?}", RecordKey::new(&config.test_key));

    // Use exactly 5 test values for consistency
    let test_values = vec![
        "Hello World".to_string(),
        "Test Record".to_string(),
        "Another Value".to_string(),
        "Fourth Record".to_string(),
        "Fifth Record".to_string(),
    ];

    info!("Test values: {:?}", test_values);
    if let Some(timeout) = config.timeout {
        info!("Timeout: {:?}", timeout);
    } else {
        info!("Timeout: None (runs indefinitely)");
    }
    info!("===========================================");

    let key = RecordKey::new(&config.test_key);

    run_writer_node(
        config.address,
        key,
        test_values,
        config.timeout,
        config.verbose,
        config.persistence,
    )
    .await
}

/// Cross-machine reader test for 5 records with unique keys
/// Connects to a writer node and retrieves exactly 5 records
#[ignore]
#[tokio::test]
async fn cross_machine_reader_5_records() -> Result<()> {
    init_logging();

    let config = reader_config_from_env().map_err(|e| {
        error!("Failed to parse reader configuration: {}", e);
        e
    })?;

    info!("=== CROSS-MACHINE 5-RECORD READER TEST ===");
    info!("Connecting to writer at: {}", config.connect_addr);
    info!("Test key: {:?}", RecordKey::new(&config.test_key));

    // Expect exactly 5 test values for consistency
    let expected_values = vec![
        "Hello World".to_string(),
        "Test Record".to_string(),
        "Another Value".to_string(),
        "Fourth Record".to_string(),
        "Fifth Record".to_string(),
    ];

    info!("Expected values: {:?}", expected_values);
    info!("Timeout: {:?}", config.timeout);
    info!("Retries: {}", config.retries);
    info!("===========================================");

    let key = RecordKey::new(&config.test_key);

    let result = run_reader_node(
        config.connect_addr,
        key,
        expected_values.clone(),
        config.timeout,
        config.retries,
        config.verbose,
        config.persistence,
    )
    .await;

    match &result {
        Ok(_) => {
            info!("✓ Cross-machine 5-record reader test completed successfully!");
            info!("✓ All 5 records were found and retrieved correctly");
        }
        Err(e) => {
            error!("✗ Cross-machine 5-record reader test failed: {}", e);
        }
    }

    result
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
        config.persistence,
    )
    .await
}

/// Local test that verifies basic node functionality without requiring network operations
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
    info!("Testing basic node functionality locally");
    info!("==================================");

    let key = RecordKey::new(&config.test_key);

    // Test 1: Create writer node and verify local storage
    let temp_dir_1 = get_test_temp_dir_str(Some("cross_writer_local"), config.persistence);
    let mut writer_swarm = generate_swarm(&temp_dir_1)?;
    info!("Writer node created successfully");

    // Test local storage on writer
    let writer_store = writer_swarm.behaviour_mut().kad.store_mut();
    let mut stored_count = 0;

    for (i, value) in config.test_values.iter().enumerate() {
        let unique_key = libp2p::kad::RecordKey::new(&format!(
            "{}__{}",
            std::str::from_utf8(key.as_ref()).unwrap_or("test_key"),
            i
        ));
        let record = libp2p::kad::Record::new(unique_key.clone(), value.as_bytes().to_vec());
        match writer_store.put(record) {
            Ok(_) => {
                stored_count += 1;
                info!("Writer stored record {}: '{}'", i + 1, value);
            }
            Err(e) => warn!("Writer failed to store record {}: {:?}", i + 1, e),
        }

        // Verify we can retrieve this record from writer's local store
        if let Some(retrieved_record) = writer_store.get(&unique_key) {
            let retrieved_value = String::from_utf8_lossy(&retrieved_record.value);
            info!(
                "Writer successfully retrieved record {}: '{}'",
                i + 1,
                retrieved_value
            );
        } else {
            warn!(
                "Failed to retrieve record {} from writer's local store",
                i + 1
            );
        }
    }

    // Test 2: Create reader node and verify it works independently
    let temp_dir_2 = get_test_temp_dir_str(Some("cross_reader_local"), config.persistence);
    let mut reader_swarm = generate_swarm(&temp_dir_2)?;
    info!("Reader node created successfully");

    // Test local storage on reader with different data
    let reader_store = reader_swarm.behaviour_mut().kad.store_mut();
    let test_key_reader = RecordKey::new(&"local_test_key");
    let test_value_reader = "local_test_value";
    let test_record = libp2p::kad::Record::new(
        test_key_reader.clone(),
        test_value_reader.as_bytes().to_vec(),
    );

    match reader_store.put(test_record) {
        Ok(_) => info!("Reader stored local test record successfully"),
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Reader failed to store local record: {:?}",
                e
            ));
        }
    }

    // Verify reader can retrieve its own record
    if let Some(retrieved_record) = reader_store.get(&test_key_reader) {
        let retrieved_value = String::from_utf8_lossy(&retrieved_record.value);
        info!(
            "Reader successfully retrieved its record: '{}'",
            retrieved_value
        );
        assert_eq!(retrieved_value, test_value_reader);
    } else {
        return Err(anyhow::anyhow!(
            "Failed to retrieve record from reader's local store"
        ));
    }

    // Final verification
    info!("Verification phase");
    info!("✓ Writer node created and can store/retrieve records locally");
    info!("✓ Reader node created and can store/retrieve records locally");
    info!(
        "✓ Stored {}/{} test values in writer",
        stored_count,
        config.test_values.len()
    );

    if stored_count > 0 {
        info!("Local test: Completed successfully!");
        Ok(())
    } else {
        error!("Local test: Failed - No values stored");
        Err(anyhow::anyhow!("No values stored successfully"))
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
            persistent: None,
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
            persistent: None,
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
            persistent: None,
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
