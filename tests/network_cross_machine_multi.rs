//! Multi-Record Cross-Machine Test Module
//!
//! This module provides functionality for testing persistence across machines and sessions
//! with support for large numbers of test records and deterministic path generation.
//!
//! Key features:
//! - Deterministic path randomization via NETABASE_TEST_SEED environment variable
//! - Support for creating and testing large numbers of records
//! - Comprehensive success/failure reporting
//! - Configuration through environment variables 516or command-line arguments

use anyhow::{Context, Result, bail};
use clap::Parser;
use libp2p::Multiaddr;
use libp2p::kad::{QueryResult, RecordKey};
use log::{debug, error, info, warn};
use netabase::{get_test_temp_dir_str, network::swarm::generate_swarm};
use serde::Deserialize;
use std::future::Future;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::{Duration, Instant};
use libp2p::futures::StreamExt;
use tokio::time::timeout;

// Default configuration values
const DEFAULT_WRITER_ADDR: &str = "0.0.0.0:9901";
const DEFAULT_READER_CONNECT_ADDR: &str = "127.0.0.1:9901";
const DEFAULT_TEST_KEY: &str = "multi_record_test";
const DEFAULT_WRITER_TIMEOUT: u64 = 300; // 5 minutes
const DEFAULT_READER_TIMEOUT: u64 = 120; // 2 minutes
const DEFAULT_READER_RETRIES: u32 = 3;
const DEFAULT_RECORD_COUNT: usize = 50; // Default number of test records

/// Initialize logging with once-only guarantee
fn init_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();
    });
}

/// Generate test values with a specific pattern and count
fn generate_test_values(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("Multi-Record Test Value {}", i))
        .collect()
}

/// Writer configuration from environment variables and/or command-line arguments
#[derive(Debug, Deserialize, Clone)]
struct WriterConfig {
    #[serde(default)]
    address: Option<String>,

    #[serde(default)]
    test_key: Option<String>,

    #[serde(default)]
    record_count: Option<usize>,

    #[serde(default)]
    timeout: Option<u64>,

    #[serde(default)]
    verbose: bool,
}

/// Validated writer configuration
#[derive(Debug, Clone)]
struct ValidatedWriterConfig {
    address: SocketAddr,
    test_key: String,
    record_count: usize,
    timeout: Option<Duration>,
    verbose: bool,
}

impl WriterConfig {
    /// Parse configuration from environment variables with NETABASE_ prefix
    fn from_env() -> Result<Self> {
        envy::prefixed("NETABASE_")
            .from_env::<WriterConfig>()
            .context("Failed to parse writer configuration from environment")
    }

    /// Validate and apply defaults to writer configuration
    fn validate(self) -> Result<ValidatedWriterConfig> {
        // Apply defaults
        let address = match self.address {
            Some(addr) => {
                // Handle both SocketAddr and separate host:port formats
                if addr.contains(':') {
                    SocketAddr::from_str(&addr)
                        .with_context(|| format!("Invalid socket address: {}", addr))?
                } else {
                    bail!("Invalid address format: {}", addr);
                }
            }
            None => SocketAddr::from_str(DEFAULT_WRITER_ADDR)?,
        };

        let test_key = self
            .test_key
            .unwrap_or_else(|| DEFAULT_TEST_KEY.to_string());
        let record_count = self.record_count.unwrap_or(DEFAULT_RECORD_COUNT);

        let timeout = match self.timeout {
            Some(0) => None, // 0 means no timeout
            Some(secs) => Some(Duration::from_secs(secs)),
            None => Some(Duration::from_secs(DEFAULT_WRITER_TIMEOUT)),
        };

        Ok(ValidatedWriterConfig {
            address,
            test_key,
            record_count,
            timeout,
            verbose: self.verbose,
        })
    }
}

/// Parse writer configuration from environment variables with validation
fn writer_config_from_env() -> Result<ValidatedWriterConfig> {
    WriterConfig::from_env()?.validate()
}

/// Reader configuration from environment variables and/or command-line arguments
#[derive(Debug, Deserialize, Clone)]
struct ReaderConfig {
    #[serde(default)]
    connect_addr: Option<String>,

    #[serde(default)]
    test_key: Option<String>,

    #[serde(default)]
    record_count: Option<usize>,

    #[serde(default)]
    timeout: Option<u64>,

    #[serde(default)]
    retries: Option<u32>,

    #[serde(default)]
    verbose: bool,
}

/// Validated reader configuration
#[derive(Debug, Clone)]
struct ValidatedReaderConfig {
    connect_addr: SocketAddr,
    test_key: String,
    record_count: usize,
    timeout: Duration,
    retries: u32,
    verbose: bool,
}

impl ReaderConfig {
    /// Parse configuration from environment variables with NETABASE_ prefix
    fn from_env() -> Result<Self> {
        envy::prefixed("NETABASE_")
            .from_env::<ReaderConfig>()
            .context("Failed to parse reader configuration from environment")
    }

    /// Validate and apply defaults to reader configuration
    fn validate(self) -> Result<ValidatedReaderConfig> {
        // Apply defaults
        let connect_addr = match self.connect_addr {
            Some(addr) => {
                // Handle both SocketAddr and separate host:port formats
                if addr.contains(':') {
                    SocketAddr::from_str(&addr)
                        .with_context(|| format!("Invalid socket address: {}", addr))?
                } else {
                    bail!("Invalid address format: {}", addr);
                }
            }
            None => SocketAddr::from_str(DEFAULT_READER_CONNECT_ADDR)?,
        };

        let test_key = self
            .test_key
            .unwrap_or_else(|| DEFAULT_TEST_KEY.to_string());
        let record_count = self.record_count.unwrap_or(DEFAULT_RECORD_COUNT);

        let timeout = Duration::from_secs(self.timeout.unwrap_or(DEFAULT_READER_TIMEOUT));

        let retries = self.retries.unwrap_or(DEFAULT_READER_RETRIES);

        Ok(ValidatedReaderConfig {
            connect_addr,
            test_key,
            record_count,
            timeout,
            retries,
            verbose,
        })
    }
}

/// Parse reader configuration from environment variables with validation
fn reader_config_from_env() -> Result<ValidatedReaderConfig> {
    ReaderConfig::from_env()?.validate()
}

/// Run a writer node that stores multiple records with the specified key and values
async fn run_writer_node(
    listen_addr: SocketAddr,
    key: RecordKey,
    values: Vec<String>,
    timeout: Option<Duration>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        info!("Starting writer node on address: {}", listen_addr);
        info!(
            "Writer will store {} values under key base: {:?}",
            values.len(),
            key
        );
    }

    let temp_dir = get_test_temp_dir_str(Some("multi_writer"));
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

    // Store all records with unique keys
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

    if let Some(timeout_duration) = timeout {
        info!("Writer: Will run for {:?}", timeout_duration);
    } else {
        info!("Writer: Press Ctrl+C to stop the writer node");
    }

    let start_time = Instant::now();

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

/// Run a reader node that attempts to retrieve all records with the specified key prefix
async fn run_reader_node(
    connect_addr: SocketAddr,
    base_key: RecordKey,
    expected_values: Vec<String>,
    timeout_duration: Duration,
    retries: u32,
    verbose: bool,
) -> Result<ReaderTestResults> {
    info!(
        "Starting reader node, connecting to writer at: {}",
        connect_addr
    );

    let temp_dir = get_test_temp_dir_str(Some("multi_reader"));
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

    let total_records = records_to_find.len();
    let mut connected = false;
    let mut record_results = Vec::with_capacity(total_records);
    let mut current_record_index = 0;
    let mut record_attempts = 0;
    let max_attempts_per_record = retries + 1;

    let start_time = Instant::now();

    // Wait for mDNS discovery and connection establishment
    let mut discovered_peers = Vec::new();
    let mut connected_peer_id = None;

    info!("Reader: Waiting for mDNS discovery and peer connections...");

    while current_record_index < total_records && start_time.elapsed() < timeout_duration {
        let event_future = swarm.select_next_some();
        let event_result = timeout(Duration::from_secs(1), event_future).await;

        if let Ok(event) = event_result {
            match event {
                libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Reader: Listening on {}", address);
                }
                libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    info!("Reader: Connected to peer: {}", peer_id);
                    connected = true;
                    connected_peer_id = Some(peer_id);

                    // Try bootstrapping with the connected peer
                    swarm.behaviour_mut().kad.bootstrap()?;
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
                        discovered_peers.push(peer_id);

                        // Try to connect to discovered peers
                        if let Err(e) = swarm.dial(peer_id) {
                            if verbose {
                                warn!(
                                    "Reader: Failed to dial discovered peer {}: {:?}",
                                    peer_id, e
                                );
                            }
                        } else {
                            info!(
                                "Reader: Attempting to connect to discovered peer: {}",
                                peer_id
                            );
                        }
                    }
                }
                libp2p::swarm::SwarmEvent::Behaviour(
                    netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                        libp2p::kad::Event::RoutingUpdated { peer, .. },
                    ),
                ) => {
                    info!("Reader: Routing updated for peer: {}", peer);

                    // If we haven't started querying records yet, start now
                    if !connected {
                        info!("Reader: Network initialized, starting record queries");
                        connected = true;
                    }
                }
                libp2p::swarm::SwarmEvent::Behaviour(
                    netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                        libp2p::kad::Event::OutboundQueryProgressed {
                            result: QueryResult::GetRecord(Ok(result)),
                            ..
                        },
                    ),
                ) => {
                    if let Ok(record) = result {
                        let value_str = String::from_utf8_lossy(&record.record.value);
                        let expected = &expected_values[current_record_index];

                        if &value_str == expected {
                            info!(
                                "Reader: ✓ Record {}/{} found and matches expected value: '{}'",
                                current_record_index + 1,
                                total_records,
                                value_str
                            );
                            record_results.push(RecordResult {
                                index: current_record_index,
                                key: format!(
                                    "{}__{}",
                                    std::str::from_utf8(base_key.as_ref()).unwrap_or("test_key"),
                                    current_record_index
                                ),
                                value: value_str.to_string(),
                                expected: expected.clone(),
                                status: RecordStatus::Success,
                                attempts: record_attempts + 1,
                            });

                            // Move to next record
                            current_record_index += 1;
                            record_attempts = 0;

                            // Start querying the next record if there are more
                            if current_record_index < total_records {
                                info!(
                                    "Reader: Querying record {}/{}: key: {:?}",
                                    current_record_index + 1,
                                    total_records,
                                    records_to_find[current_record_index]
                                );
                                swarm
                                    .behaviour_mut()
                                    .kad
                                    .get_record(records_to_find[current_record_index].clone());
                            }
                        } else {
                            warn!(
                                "Reader: ✗ Record found but value mismatch: Expected '{}', Got '{}'",
                                expected, value_str
                            );
                            record_results.push(RecordResult {
                                index: current_record_index,
                                key: format!(
                                    "{}__{}",
                                    std::str::from_utf8(base_key.as_ref()).unwrap_or("test_key"),
                                    current_record_index
                                ),
                                value: value_str.to_string(),
                                expected: expected.clone(),
                                status: RecordStatus::ValueMismatch,
                                attempts: record_attempts + 1,
                            });

                            // Move to next record after a mismatch
                            current_record_index += 1;
                            record_attempts = 0;

                            // Start querying the next record if there are more
                            if current_record_index < total_records {
                                info!(
                                    "Reader: Querying record {}/{}: key: {:?}",
                                    current_record_index + 1,
                                    total_records,
                                    records_to_find[current_record_index]
                                );
                                swarm
                                    .behaviour_mut()
                                    .kad
                                    .get_record(records_to_find[current_record_index].clone());
                            }
                        }
                    } else {
                        warn!(
                            "Reader: Record found but no value returned for index {}",
                            current_record_index
                        );

                        // Retry if we haven't exceeded max attempts
                        if record_attempts < max_attempts_per_record - 1 {
                            record_attempts += 1;
                            info!(
                                "Reader: Retrying record {}/{} (attempt {}/{})",
                                current_record_index + 1,
                                total_records,
                                record_attempts + 1,
                                max_attempts_per_record
                            );
                            swarm
                                .behaviour_mut()
                                .kad
                                .get_record(records_to_find[current_record_index].clone());
                        } else {
                            // Max retries exceeded, mark as not found and move on
                            warn!(
                                "Reader: ✗ Record {}/{} not found after {} attempts",
                                current_record_index + 1,
                                total_records,
                                max_attempts_per_record
                            );
                            record_results.push(RecordResult {
                                index: current_record_index,
                                key: format!(
                                    "{}__{}",
                                    std::str::from_utf8(base_key.as_ref()).unwrap_or("test_key"),
                                    current_record_index
                                ),
                                value: String::new(),
                                expected: expected_values[current_record_index].clone(),
                                status: RecordStatus::NotFound,
                                attempts: record_attempts + 1,
                            });

                            // Move to next record
                            current_record_index += 1;
                            record_attempts = 0;

                            // Start querying the next record if there are more
                            if current_record_index < total_records {
                                info!(
                                    "Reader: Querying record {}/{}: key: {:?}",
                                    current_record_index + 1,
                                    total_records,
                                    records_to_find[current_record_index]
                                );
                                swarm
                                    .behaviour_mut()
                                    .kad
                                    .get_record(records_to_find[current_record_index].clone());
                            }
                        }
                    }
                }
                libp2p::swarm::SwarmEvent::Behaviour(
                    netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                        libp2p::kad::Event::OutboundQueryProgressed {
                            result: QueryResult::GetRecord(Err(e)),
                            ..
                        },
                    ),
                ) => {
                    warn!(
                        "Reader: ✗ Error querying record {}/{}: {:?}",
                        current_record_index + 1,
                        total_records,
                        e
                    );

                    // Retry if we haven't exceeded max attempts
                    if record_attempts < max_attempts_per_record - 1 {
                        record_attempts += 1;
                        info!(
                            "Reader: Retrying record {}/{} (attempt {}/{})",
                            current_record_index + 1,
                            total_records,
                            record_attempts + 1,
                            max_attempts_per_record
                        );
                        swarm
                            .behaviour_mut()
                            .kad
                            .get_record(records_to_find[current_record_index].clone());
                    } else {
                        // Max retries exceeded, mark as error and move on
                        warn!(
                            "Reader: ✗ Record {}/{} query failed after {} attempts",
                            current_record_index + 1,
                            total_records,
                            max_attempts_per_record
                        );
                        record_results.push(RecordResult {
                            index: current_record_index,
                            key: format!(
                                "{}__{}",
                                std::str::from_utf8(base_key.as_ref()).unwrap_or("test_key"),
                                current_record_index
                            ),
                            value: String::new(),
                            expected: expected_values[current_record_index].clone(),
                            status: RecordStatus::QueryError(e.to_string()),
                            attempts: record_attempts + 1,
                        });

                        // Move to next record
                        current_record_index += 1;
                        record_attempts = 0;

                        // Start querying the next record if there are more
                        if current_record_index < total_records {
                            info!(
                                "Reader: Querying record {}/{}: key: {:?}",
                                current_record_index + 1,
                                total_records,
                                records_to_find[current_record_index]
                            );
                            swarm
                                .behaviour_mut()
                                .kad
                                .get_record(records_to_find[current_record_index].clone());
                        }
                    }
                }
                _ => {}
            }
        } else {
            // Timeout on waiting for events
            if connected && current_record_index < total_records {
                // If we're connected but didn't get a response for the current record
                if current_record_index == 0 && !connected {
                    // We haven't found any peers yet, keep waiting
                    if discovered_peers.is_empty() {
                        info!("Reader: No peers discovered yet, continuing to wait...");
                        continue;
                    }

                    // Try bootstrapping with discovered peers
                    info!("Reader: Bootstrapping with discovered peers...");
                    if let Err(e) = swarm.behaviour_mut().kad.bootstrap() {
                        warn!("Reader: Bootstrap failed: {:?}", e);
                    }
                }

                // If we haven't yet issued a query for this record, do so now
                if record_attempts == 0 {
                    info!(
                        "Reader: Querying record {}/{}: key: {:?}",
                        current_record_index + 1,
                        total_records,
                        records_to_find[current_record_index]
                    );
                    swarm
                        .behaviour_mut()
                        .kad
                        .get_record(records_to_find[current_record_index].clone());
                    record_attempts += 1;
                } else if record_attempts < max_attempts_per_record {
                    // Retry if we haven't exceeded max attempts
                    record_attempts += 1;
                    info!(
                        "Reader: Retrying record {}/{} (attempt {}/{})",
                        current_record_index + 1,
                        total_records,
                        record_attempts,
                        max_attempts_per_record
                    );
                    swarm
                        .behaviour_mut()
                        .kad
                        .get_record(records_to_find[current_record_index].clone());
                } else {
                    // Max retries exceeded, mark as timeout and move on
                    warn!(
                        "Reader: ✗ Record {}/{} timed out after {} attempts",
                        current_record_index + 1,
                        total_records,
                        max_attempts_per_record
                    );
                    record_results.push(RecordResult {
                        index: current_record_index,
                        key: format!(
                            "{}__{}",
                            std::str::from_utf8(base_key.as_ref()).unwrap_or("test_key"),
                            current_record_index
                        ),
                        value: String::new(),
                        expected: expected_values[current_record_index].clone(),
                        status: RecordStatus::Timeout,
                        attempts: record_attempts,
                    });

                    // Move to next record
                    current_record_index += 1;
                    record_attempts = 0;

                    // Start querying the next record if there are more
                    if current_record_index < total_records {
                        info!(
                            "Reader: Querying record {}/{}: key: {:?}",
                            current_record_index + 1,
                            total_records,
                            records_to_find[current_record_index]
                        );
                        swarm
                            .behaviour_mut()
                            .kad
                            .get_record(records_to_find[current_record_index].clone());
                    }
                }
            }
        }
    }

    // Process any remaining records that weren't attempted due to timeout
    for i in current_record_index..total_records {
        record_results.push(RecordResult {
            index: i,
            key: format!(
                "{}__{}",
                std::str::from_utf8(base_key.as_ref()).unwrap_or("test_key"),
                i
            ),
            value: String::new(),
            expected: expected_values[i].clone(),
            status: RecordStatus::NotAttempted,
            attempts: 0,
        });
    }

    // Calculate summary statistics
    let successful = record_results
        .iter()
        .filter(|r| matches!(r.status, RecordStatus::Success))
        .count();
    let failed = record_results.len() - successful;

    let results = ReaderTestResults {
        total: record_results.len(),
        successful,
        failed,
        records: record_results,
        elapsed: start_time.elapsed(),
    };

    // Log summary
    info!(
        "Reader: Test completed. Results: {}/{} records successful",
        successful, total_records
    );

    Ok(results)
}

/// Status of a record retrieval attempt
#[derive(Debug, Clone)]
enum RecordStatus {
    Success,
    ValueMismatch,
    NotFound,
    Timeout,
    QueryError(String),
    NotAttempted,
}

/// Result of a single record retrieval attempt
#[derive(Debug, Clone)]
struct RecordResult {
    index: usize,
    key: String,
    value: String,
    expected: String,
    status: RecordStatus,
    attempts: u32,
}

/// Summary of all record retrieval results
#[derive(Debug)]
struct ReaderTestResults {
    total: usize,
    successful: usize,
    failed: usize,
    records: Vec<RecordResult>,
    elapsed: Duration,
}

/// Run the writer node with many test records
pub async fn cross_machine_writer_multi_records() -> Result<()> {
    init_logging();

    let config = writer_config_from_env().map_err(|e| {
        error!("Failed to parse writer configuration: {}", e);
        e
    })?;

    info!("=== CROSS-MACHINE MULTI-RECORD WRITER TEST ===");
    info!("Writer address: {}", config.address);
    info!("Test key: {:?}", RecordKey::new(&config.test_key));
    info!("Number of records: {}", config.record_count);

    // Generate test values based on count
    let test_values = generate_test_values(config.record_count);

    info!("Test values generated: {} records", test_values.len());
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
    )
    .await
}

/// Run the reader node to find and verify many test records
pub async fn cross_machine_reader_multi_records() -> Result<()> {
    init_logging();

    let config = reader_config_from_env().map_err(|e| {
        error!("Failed to parse reader configuration: {}", e);
        e
    })?;

    info!("=== CROSS-MACHINE MULTI-RECORD READER TEST ===");
    info!("Connecting to writer at: {}", config.connect_addr);
    info!("Test key: {:?}", RecordKey::new(&config.test_key));
    info!("Expected records: {}", config.record_count);

    // Generate expected values (same as writer would have generated)
    let expected_values = generate_test_values(config.record_count);

    info!(
        "Expected values generated: {} records",
        expected_values.len()
    );
    info!("Timeout: {:?}", config.timeout);
    info!("Retries: {}", config.retries);
    info!("===========================================");

    let key = RecordKey::new(&config.test_key);

    let results = run_reader_node(
        config.connect_addr,
        key,
        expected_values.clone(),
        config.timeout,
        config.retries,
        config.verbose,
    )
    .await?;

    // Report detailed results
    info!("=== CROSS-MACHINE MULTI-RECORD READER RESULTS ===");
    info!("Total records: {}", results.total);
    info!(
        "Successful: {} ({}%)",
        results.successful,
        (results.successful as f32 / results.total as f32 * 100.0) as u32
    );
    info!(
        "Failed: {} ({}%)",
        results.failed,
        (results.failed as f32 / results.total as f32 * 100.0) as u32
    );
    info!("Test duration: {:?}", results.elapsed);

    // List all failures
    if results.failed > 0 {
        info!("--- Failed Records ---");
        for record in &results.records {
            if !matches!(record.status, RecordStatus::Success) {
                let status_str = match &record.status {
                    RecordStatus::ValueMismatch => format!(
                        "Value mismatch (expected: '{}', got: '{}')",
                        record.expected, record.value
                    ),
                    RecordStatus::NotFound => "Not found".to_string(),
                    RecordStatus::Timeout => "Timeout".to_string(),
                    RecordStatus::QueryError(e) => format!("Query error: {}", e),
                    RecordStatus::NotAttempted => "Not attempted".to_string(),
                    _ => "Unknown error".to_string(),
                };
                info!("Record {}: {} - {}", record.index, record.key, status_str);
            }
        }
    }

    info!("===========================================");

    // If all records were found, return success
    if results.successful == results.total {
        info!("✓ Cross-machine multi-record reader test completed successfully!");
        info!(
            "✓ All {} records were found and retrieved correctly",
            results.total
        );
        Ok(())
    } else {
        error!("✗ Cross-machine multi-record reader test failed");
        error!(
            "✗ Only {}/{} records were found and verified",
            results.successful, results.total
        );
        bail!(
            "Cross-machine multi-record reader test failed: {}/{} records verified",
            results.successful,
            results.total
        )
    }
}

/// Local cross-machine test that runs both writer and reader in sequence
pub async fn cross_machine_local_test_multi_records() -> Result<()> {
    init_logging();

    // Get environment variables
    let writer_config = writer_config_from_env()?;
    let reader_config = reader_config_from_env()?;

    // First run the writer to store records
    info!("=== STARTING LOCAL MULTI-RECORD CROSS-MACHINE TEST ===");
    info!("Using seed: {:?}", std::env::var("NETABASE_TEST_SEED").ok());
    info!(
        "Step 1: Running writer to store {} records",
        writer_config.record_count
    );

    // Run writer with timeout
    let writer_timeout = Duration::from_secs(30);
    let writer_start = Instant::now();

    let key = RecordKey::new(&writer_config.test_key);
    let test_values = generate_test_values(writer_config.record_count);

    // Run writer with shorter timeout for local testing
    run_writer_node(
        writer_config.address,
        key.clone(),
        test_values.clone(),
        Some(writer_timeout),
        writer_config.verbose,
    )
    .await?;

    let writer_elapsed = writer_start.elapsed();
    info!("Writer completed in {:?}", writer_elapsed);

    // Allow some time for the network to settle
    info!("Waiting 5 seconds for network to stabilize...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Then run the reader to verify records
    info!(
        "Step 2: Running reader to verify {} records",
        reader_config.record_count
    );
    let reader_start = Instant::now();

    // Run reader with the same values
    let results = run_reader_node(
        reader_config.connect_addr,
        key,
        test_values,
        reader_config.timeout,
        reader_config.retries,
        reader_config.verbose,
    )
    .await?;

    let reader_elapsed = reader_start.elapsed();

    // Report results
    info!("=== LOCAL MULTI-RECORD TEST RESULTS ===");
    info!("Total records: {}", results.total);
    info!(
        "Successful: {} ({}%)",
        results.successful,
        (results.successful as f32 / results.total as f32 * 100.0) as u32
    );
    info!(
        "Failed: {} ({}%)",
        results.failed,
        (results.failed as f32 / results.total as f32 * 100.0) as u32
    );
    info!("Writer time: {:?}", writer_elapsed);
    info!("Reader time: {:?}", reader_elapsed);
    info!("Total test time: {:?}", writer_elapsed + reader_elapsed);

    // If all records were found, return success
    if results.successful == results.total {
        info!("✓ Local multi-record cross-machine test completed successfully!");
        info!(
            "✓ All {} records were found and retrieved correctly",
            results.total
        );
        Ok(())
    } else {
        error!("✗ Local multi-record cross-machine test failed");
        error!(
            "✗ Only {}/{} records were found and verified",
            results.successful, results.total
        );
        bail!(
            "Local multi-record cross-machine test failed: {}/{} records verified",
            results.successful,
            results.total
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_multi_record() {
        // Set a fixed seed for deterministic testing
        std::env::set_var("NETABASE_TEST_SEED", "12345");

        // Set small record count for quick testing
        std::env::set_var("NETABASE_RECORD_COUNT", "10");

        // Run local test
        let result = cross_machine_local_test_multi_records().await;
        assert!(
            result.is_ok(),
            "Local multi-record test failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_deterministic_paths() {
        // Test that paths are deterministic with same seed
        std::env::set_var("NETABASE_TEST_SEED", "54321");
        let path1 = netabase::get_test_temp_dir_str(Some("test_path"));
        let path2 = netabase::get_test_temp_dir_str(Some("test_path"));
        assert_eq!(path1, path2, "Paths with same seed should be identical");

        // Test that paths are different with different seeds
        std::env::set_var("NETABASE_TEST_SEED", "12345");
        let path3 = netabase::get_test_temp_dir_str(Some("test_path"));
        assert_ne!(
            path1, path3,
            "Paths with different seeds should be different"
        );
    }

    #[test]
    fn test_multi_record_config() {
        // Test writer config
        std::env::set_var("NETABASE_TEST_KEY", "test_multi_key");
        std::env::set_var("NETABASE_RECORD_COUNT", "25");

        let writer_config = writer_config_from_env().unwrap();
        assert_eq!(writer_config.test_key, "test_multi_key");
        assert_eq!(writer_config.record_count, 25);

        // Test reader config
        std::env::set_var("NETABASE_TEST_TIMEOUT", "60");
        std::env::set_var("NETABASE_READER_RETRIES", "5");

        let reader_config = reader_config_from_env().unwrap();
        assert_eq!(reader_config.test_key, "test_multi_key");
        assert_eq!(reader_config.record_count, 25);
        assert_eq!(reader_config.timeout, Duration::from_secs(60));
        assert_eq!(reader_config.retries, 5);
    }
}
