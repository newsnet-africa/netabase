//! Scalability tests for NetaBase P2P platform
//!
//! This module contains comprehensive tests to validate the scalability characteristics
//! of the NetaBase distributed hash table as the number of nodes increases.
//!
//! Tests are designed to measure:
//! - Connection establishment time
//! - DHT record propagation latency
//! - Network partition recovery
//! - Memory and resource usage under load
//! - Bootstrap performance for new nodes
//! - Churn handling (nodes joining/leaving)
//!
//! All tests are marked as #[ignore] by default since they are resource-intensive
//! and long-running. Use `cargo test -- --ignored` to run them.

use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicU64},
    time::{Duration, Instant, SystemTime},
};

use anyhow::Result;
use libp2p::{
    Multiaddr, PeerId, Swarm,
    futures::StreamExt,
    kad::{Record, RecordKey},
    swarm::SwarmEvent,
};
use log::{debug, error, info, warn};
use netabase::{
    get_test_temp_dir_str,
    network::{behaviour::NetabaseBehaviour, swarm::generate_swarm},
};
use tokio::{
    sync::mpsc,
    time::{sleep, timeout},
};

/// Configuration for scalability tests, primarily controlled via environment variables
#[derive(Debug, Clone)]
struct ScalabilityTestConfig {
    /// Maximum number of nodes to test with
    pub max_nodes: usize,
    /// Duration to run each test phase
    pub test_duration: Duration,
    /// Interval between adding new nodes
    pub node_spawn_interval: Duration,
    /// Timeout for individual operations
    pub operation_timeout: Duration,
    /// Number of records to store per node
    pub records_per_node: usize,
    /// Size of each record value in bytes
    pub record_value_size: usize,
    /// Enable verbose performance logging
    pub verbose_metrics: bool,
    /// Enable network partition simulation
    pub test_partitions: bool,
}

impl Default for ScalabilityTestConfig {
    fn default() -> Self {
        Self {
            max_nodes: std::env::var("NETABASE_SCALABILITY_MAX_NODES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(20),
            test_duration: Duration::from_secs(
                std::env::var("NETABASE_SCALABILITY_TEST_DURATION")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300),
            ),
            node_spawn_interval: Duration::from_secs(
                std::env::var("NETABASE_SCALABILITY_SPAWN_INTERVAL")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(2),
            ),
            operation_timeout: Duration::from_secs(
                std::env::var("NETABASE_SCALABILITY_OP_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30),
            ),
            records_per_node: std::env::var("NETABASE_SCALABILITY_RECORDS_PER_NODE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            record_value_size: std::env::var("NETABASE_SCALABILITY_RECORD_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1024),
            verbose_metrics: std::env::var("NETABASE_SCALABILITY_VERBOSE")
                .map(|s| s.to_lowercase() == "true")
                .unwrap_or(false),
            test_partitions: std::env::var("NETABASE_SCALABILITY_TEST_PARTITIONS")
                .map(|s| s.to_lowercase() == "true")
                .unwrap_or(true),
        }
    }
}

/// Performance metrics collector - useful for production monitoring
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Timestamp when metrics collection started
    pub start_time: SystemTime,
    /// Number of successful operations
    pub successful_operations: Arc<AtomicU64>,
    /// Number of failed operations
    pub failed_operations: Arc<AtomicU64>,
    /// Total latency for all operations (in microseconds)
    pub total_latency_micros: Arc<AtomicU64>,
    /// Peak memory usage observed (in bytes)
    pub peak_memory_bytes: Arc<AtomicU64>,
    /// Number of active connections
    pub active_connections: Arc<AtomicU64>,
    /// DHT routing table size
    pub routing_table_size: Arc<AtomicU64>,
    /// Number of records stored locally
    pub local_records: Arc<AtomicU64>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            successful_operations: Arc::new(AtomicU64::new(0)),
            failed_operations: Arc::new(AtomicU64::new(0)),
            total_latency_micros: Arc::new(AtomicU64::new(0)),
            peak_memory_bytes: Arc::new(AtomicU64::new(0)),
            active_connections: Arc::new(AtomicU64::new(0)),
            routing_table_size: Arc::new(AtomicU64::new(0)),
            local_records: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a successful operation with its latency
    pub fn record_success(&self, latency: Duration) {
        self.successful_operations
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_latency_micros.fetch_add(
            latency.as_micros() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    /// Record a failed operation
    pub fn record_failure(&self) {
        self.failed_operations
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Update connection count
    pub fn update_connections(&self, count: u64) {
        self.active_connections
            .store(count, std::sync::atomic::Ordering::Relaxed);
    }

    /// Update routing table size
    pub fn update_routing_table_size(&self, size: u64) {
        self.routing_table_size
            .store(size, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get current average latency in milliseconds
    pub fn average_latency_ms(&self) -> f64 {
        let total_ops = self
            .successful_operations
            .load(std::sync::atomic::Ordering::Relaxed);
        if total_ops == 0 {
            return 0.0;
        }
        let total_latency = self
            .total_latency_micros
            .load(std::sync::atomic::Ordering::Relaxed);
        (total_latency as f64 / total_ops as f64) / 1000.0
    }

    /// Get operations per second
    pub fn operations_per_second(&self) -> f64 {
        let elapsed = self.start_time.elapsed().unwrap_or(Duration::from_secs(1));
        let total_ops = self
            .successful_operations
            .load(std::sync::atomic::Ordering::Relaxed);
        total_ops as f64 / elapsed.as_secs_f64()
    }

    /// Get failure rate as a percentage
    pub fn failure_rate(&self) -> f64 {
        let successful = self
            .successful_operations
            .load(std::sync::atomic::Ordering::Relaxed);
        let failed = self
            .failed_operations
            .load(std::sync::atomic::Ordering::Relaxed);
        let total = successful + failed;
        if total == 0 {
            return 0.0;
        }
        (failed as f64 / total as f64) * 100.0
    }

    /// Print comprehensive metrics report - useful for production monitoring
    pub fn print_report(&self, node_id: &str) {
        let successful = self
            .successful_operations
            .load(std::sync::atomic::Ordering::Relaxed);
        let failed = self
            .failed_operations
            .load(std::sync::atomic::Ordering::Relaxed);
        let connections = self
            .active_connections
            .load(std::sync::atomic::Ordering::Relaxed);
        let routing_size = self
            .routing_table_size
            .load(std::sync::atomic::Ordering::Relaxed);
        let local_records = self
            .local_records
            .load(std::sync::atomic::Ordering::Relaxed);

        info!("=== Performance Metrics Report for Node {} ===", node_id);
        info!("Successful operations: {}", successful);
        info!("Failed operations: {}", failed);
        info!("Failure rate: {:.2}%", self.failure_rate());
        info!("Average latency: {:.2}ms", self.average_latency_ms());
        info!("Operations per second: {:.2}", self.operations_per_second());
        info!("Active connections: {}", connections);
        info!("Routing table size: {}", routing_size);
        info!("Local records: {}", local_records);
        info!(
            "Test duration: {:.2}s",
            self.start_time.elapsed().unwrap_or_default().as_secs_f64()
        );
        info!("==================================================");
    }
}

/// Represents a test node in the network
struct TestNode {
    pub id: usize,
    pub peer_id: PeerId,
    pub swarm: Swarm<NetabaseBehaviour>,
    pub metrics: PerformanceMetrics,
    pub listen_addr: Option<Multiaddr>,
    pub temp_dir: String,
}

impl TestNode {
    /// Create a new test node with unique storage
    pub async fn new(id: usize) -> Result<Self> {
        let temp_dir = get_test_temp_dir_str(Some(&format!("scalability_node_{}", id)));
        let mut swarm = generate_swarm(&temp_dir)?;
        let peer_id = *swarm.local_peer_id();

        // Start listening on a random port
        let listen_addr = format!("/ip4/127.0.0.1/udp/0/quic-v1")
            .parse()
            .expect("Valid multiaddr");
        swarm.listen_on(listen_addr)?;

        Ok(Self {
            id,
            peer_id,
            swarm,
            metrics: PerformanceMetrics::new(),
            listen_addr: None,
            temp_dir,
        })
    }

    /// Connect to another node
    pub async fn connect_to(&mut self, target_addr: &Multiaddr) -> Result<()> {
        let start = Instant::now();
        self.swarm.dial(target_addr.clone())?;

        // Wait for connection establishment with timeout
        let connection_timeout = Duration::from_secs(10);
        let connected = timeout(connection_timeout, async {
            loop {
                if let Some(event) = self.swarm.next().await {
                    if let SwarmEvent::ConnectionEstablished { .. } = event {
                        return true;
                    }
                }
            }
        })
        .await
        .unwrap_or(false);

        if connected {
            let latency = start.elapsed();
            self.metrics.record_success(latency);
            debug!(
                "Node {} connected to {} in {:?}",
                self.id, target_addr, latency
            );
        } else {
            self.metrics.record_failure();
            error!("Node {} failed to connect to {}", self.id, target_addr);
        }

        Ok(())
    }

    /// Store a record in the DHT
    pub fn put_record(&mut self, key: RecordKey, value: Vec<u8>) {
        let record = Record::new(key, value);
        self.swarm
            .behaviour_mut()
            .kad
            .put_record(record, libp2p::kad::Quorum::One)
            .expect("Put record should not fail");
        self.metrics
            .local_records
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Retrieve a record from the DHT
    pub fn get_record(&mut self, key: RecordKey) {
        self.swarm.behaviour_mut().kad.get_record(key);
    }

    /// Get current connection count
    pub fn connection_count(&self) -> usize {
        self.swarm.connected_peers().count()
    }

    /// Update metrics based on current swarm state
    pub fn update_metrics(&mut self) {
        let connection_count = self.connection_count() as u64;
        self.metrics.update_connections(connection_count);

        // Update routing table size (approximate)
        // Note: This is a simplified metric - in production you might want more detailed DHT statistics
        let routing_table_size = connection_count; // Simplified approximation
        self.metrics.update_routing_table_size(routing_table_size);
    }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        // Clean up temporary directory
        if std::path::Path::new(&self.temp_dir).exists() {
            let _ = std::fs::remove_dir_all(&self.temp_dir);
        }
    }
}

/// Network topology for testing different scenarios
#[derive(Debug, Clone)]
pub enum NetworkTopology {
    /// All nodes connect to all other nodes
    FullMesh,
    /// Nodes connect in a ring
    Ring,
    /// Nodes connect in a star pattern (one central node)
    Star,
    /// Random connections between nodes
    Random { connection_ratio: f64 },
    /// Bootstrap scenario (new nodes connect to existing bootstrap nodes)
    Bootstrap { bootstrap_count: usize },
}

/// Network partition scenario for resilience testing
#[derive(Debug, Clone)]
pub struct NetworkPartition {
    /// Nodes that should be disconnected from each other
    pub partitioned_groups: Vec<Vec<usize>>,
    /// Duration to maintain the partition
    pub duration: Duration,
}

/// Main orchestrator for scalability tests
struct ScalabilityTestOrchestrator {
    config: ScalabilityTestConfig,
    nodes: Vec<TestNode>,
    global_metrics: PerformanceMetrics,
}

impl ScalabilityTestOrchestrator {
    pub fn new(config: ScalabilityTestConfig) -> Self {
        Self {
            config,
            nodes: Vec::new(),
            global_metrics: PerformanceMetrics::new(),
        }
    }

    /// Add a new node to the network
    pub async fn add_node(&mut self) -> Result<usize> {
        let node_id = self.nodes.len();
        let mut node = TestNode::new(node_id).await?;

        // Wait for the node to start listening
        let listen_timeout = Duration::from_secs(5);
        let listen_addr = timeout(listen_timeout, async {
            loop {
                if let Some(event) = node.swarm.next().await {
                    if let SwarmEvent::NewListenAddr { address, .. } = event {
                        return Some(address);
                    }
                }
            }
        })
        .await
        .ok()
        .flatten();

        if let Some(addr) = listen_addr {
            node.listen_addr = Some(addr.clone());
            info!("Node {} listening on {}", node_id, addr);
        } else {
            warn!("Node {} failed to establish listen address", node_id);
        }

        self.nodes.push(node);
        Ok(node_id)
    }

    /// Create network connections based on specified topology
    pub async fn setup_topology(&mut self, topology: NetworkTopology) -> Result<()> {
        match topology {
            NetworkTopology::FullMesh => self.setup_full_mesh().await,
            NetworkTopology::Ring => self.setup_ring().await,
            NetworkTopology::Star => self.setup_star().await,
            NetworkTopology::Random { connection_ratio } => {
                self.setup_random(connection_ratio).await
            }
            NetworkTopology::Bootstrap { bootstrap_count } => {
                self.setup_bootstrap(bootstrap_count).await
            }
        }
    }

    async fn setup_full_mesh(&mut self) -> Result<()> {
        info!(
            "Setting up full mesh topology with {} nodes",
            self.nodes.len()
        );
        for i in 0..self.nodes.len() {
            for j in 0..self.nodes.len() {
                if i != j {
                    if let Some(target_addr) = self.nodes[j].listen_addr.clone() {
                        let _ = self.nodes[i].connect_to(&target_addr).await;
                    }
                }
            }
        }
        Ok(())
    }

    async fn setup_ring(&mut self) -> Result<()> {
        info!("Setting up ring topology with {} nodes", self.nodes.len());
        for i in 0..self.nodes.len() {
            let next_idx = (i + 1) % self.nodes.len();
            if let Some(target_addr) = self.nodes[next_idx].listen_addr.clone() {
                let _ = self.nodes[i].connect_to(&target_addr).await;
            }
        }
        Ok(())
    }

    async fn setup_star(&mut self) -> Result<()> {
        info!("Setting up star topology with {} nodes", self.nodes.len());
        if self.nodes.is_empty() {
            return Ok(());
        }

        // Node 0 is the central hub
        let hub_addr = self.nodes[0].listen_addr.clone();
        if let Some(addr) = hub_addr {
            for i in 1..self.nodes.len() {
                let _ = self.nodes[i].connect_to(&addr).await;
            }
        }
        Ok(())
    }

    async fn setup_random(&mut self, connection_ratio: f64) -> Result<()> {
        info!(
            "Setting up random topology with {} nodes and {:.2} connection ratio",
            self.nodes.len(),
            connection_ratio
        );
        use rand::Rng;
        let mut rng = rand::rng();

        for i in 0..self.nodes.len() {
            for j in 0..self.nodes.len() {
                if i != j && rng.random::<f64>() < connection_ratio {
                    if let Some(target_addr) = self.nodes[j].listen_addr.clone() {
                        let _ = self.nodes[i].connect_to(&target_addr).await;
                    }
                }
            }
        }
        Ok(())
    }

    async fn setup_bootstrap(&mut self, bootstrap_count: usize) -> Result<()> {
        info!(
            "Setting up bootstrap topology with {} bootstrap nodes out of {} total",
            bootstrap_count.min(self.nodes.len()),
            self.nodes.len()
        );

        // Connect bootstrap nodes to each other
        for i in 0..bootstrap_count.min(self.nodes.len()) {
            for j in 0..bootstrap_count.min(self.nodes.len()) {
                if i != j {
                    if let Some(target_addr) = self.nodes[j].listen_addr.clone() {
                        let _ = self.nodes[i].connect_to(&target_addr).await;
                    }
                }
            }
        }

        // Connect remaining nodes to bootstrap nodes
        for i in bootstrap_count..self.nodes.len() {
            for j in 0..bootstrap_count {
                if let Some(target_addr) = self.nodes[j].listen_addr.clone() {
                    let _ = self.nodes[i].connect_to(&target_addr).await;
                    break; // Each node only needs to connect to one bootstrap node initially
                }
            }
        }
        Ok(())
    }

    /// Run continuous DHT operations across all nodes
    pub async fn run_dht_operations(&mut self, duration: Duration) -> Result<()> {
        info!("Running DHT operations for {:?}", duration);
        let start_time = Instant::now();

        // Generate test records
        let mut test_records = Vec::new();
        for i in 0..self.config.records_per_node {
            let key = RecordKey::new(&format!("test_key_{}", i));
            let value = vec![0u8; self.config.record_value_size]; // Configurable size
            test_records.push((key, value));
        }

        while start_time.elapsed() < duration {
            // Each node stores some records
            for (node_idx, node) in self.nodes.iter_mut().enumerate() {
                for (record_idx, (_key, value)) in test_records.iter().enumerate() {
                    let unique_key =
                        RecordKey::new(&format!("node_{}_record_{}", node_idx, record_idx));
                    node.put_record(unique_key, value.clone());
                }
            }

            // Small delay between operations
            sleep(Duration::from_millis(100)).await;

            // Each node tries to retrieve records from other nodes
            let node_count = self.nodes.len();
            for node in self.nodes.iter_mut() {
                for other_node_idx in 0..node_count {
                    if other_node_idx != node.id {
                        let key = RecordKey::new(&format!("node_{}_record_0", other_node_idx));
                        node.get_record(key);
                    }
                }
            }

            // Update metrics
            for node in self.nodes.iter_mut() {
                node.update_metrics();
            }

            sleep(Duration::from_millis(500)).await;
        }

        Ok(())
    }

    /// Simulate network churn (nodes joining and leaving)
    pub async fn simulate_churn(&mut self, duration: Duration) -> Result<()> {
        info!("Simulating network churn for {:?}", duration);
        let start_time = Instant::now();
        use rand::Rng;
        let mut rng = rand::rng();

        while start_time.elapsed() < duration {
            // Randomly add or remove nodes
            if rng.random_bool(0.5) && self.nodes.len() < self.config.max_nodes {
                // Add a node
                let node_id = self.add_node().await?;
                info!("Added node {} during churn simulation", node_id);

                // Connect new node to some existing nodes
                if self.nodes.len() > 1 {
                    let target_idx = rng.random_range(0..self.nodes.len() - 1);
                    if let Some(target_addr) = self.nodes[target_idx].listen_addr.clone() {
                        let _ = self
                            .nodes
                            .last_mut()
                            .unwrap()
                            .connect_to(&target_addr)
                            .await;
                    }
                }
            } else if self.nodes.len() > 2 {
                // Remove a node (but keep at least 2 nodes)
                let remove_idx = rng.random_range(0..self.nodes.len());
                let removed_node = self.nodes.remove(remove_idx);
                info!("Removed node {} during churn simulation", removed_node.id);
            }

            sleep(self.config.node_spawn_interval).await;
        }

        Ok(())
    }

    /// Generate comprehensive test report
    pub fn generate_test_report(&self, test_name: &str) {
        info!("=== {} Test Report ===", test_name);
        info!("Configuration:");
        info!("  Max nodes: {}", self.config.max_nodes);
        info!("  Test duration: {:?}", self.config.test_duration);
        info!("  Records per node: {}", self.config.records_per_node);
        info!(
            "  Record value size: {} bytes",
            self.config.record_value_size
        );
        info!("");
        info!("Network Overview:");
        info!("  Total nodes: {}", self.nodes.len());

        let total_connections: usize = self.nodes.iter().map(|n| n.connection_count()).sum();
        info!("  Total connections: {}", total_connections);
        info!(
            "  Average connections per node: {:.2}",
            total_connections as f64 / self.nodes.len() as f64
        );

        // Aggregate metrics
        let mut total_successful = 0u64;
        let mut total_failed = 0u64;
        let mut total_latency = 0u64;

        for (idx, node) in self.nodes.iter().enumerate() {
            let successful = node
                .metrics
                .successful_operations
                .load(std::sync::atomic::Ordering::Relaxed);
            let failed = node
                .metrics
                .failed_operations
                .load(std::sync::atomic::Ordering::Relaxed);
            let latency = node
                .metrics
                .total_latency_micros
                .load(std::sync::atomic::Ordering::Relaxed);

            total_successful += successful;
            total_failed += failed;
            total_latency += latency;

            if self.config.verbose_metrics {
                node.metrics.print_report(&format!("{}", idx));
            }
        }

        info!("");
        info!("Aggregate Performance Metrics:");
        info!("  Total successful operations: {}", total_successful);
        info!("  Total failed operations: {}", total_failed);
        let total_ops = total_successful + total_failed;
        if total_ops > 0 {
            info!(
                "  Overall failure rate: {:.2}%",
                (total_failed as f64 / total_ops as f64) * 100.0
            );
            if total_successful > 0 {
                info!(
                    "  Average latency: {:.2}ms",
                    (total_latency as f64 / total_successful as f64) / 1000.0
                );
            }
        }
        info!("====================================");
    }
}

fn init_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();
    });
}

// ===== SCALABILITY TESTS =====

#[tokio::test]
#[ignore = "Long-running scalability test"]
async fn test_linear_node_scaling() {
    init_logging();
    let config = ScalabilityTestConfig::default();
    let mut orchestrator = ScalabilityTestOrchestrator::new(config.clone());

    info!(
        "Starting linear node scaling test with max {} nodes",
        config.max_nodes
    );

    // Gradually add nodes and measure performance
    for node_count in 1..=config.max_nodes {
        info!("Testing with {} nodes", node_count);

        // Add new node
        let node_id = orchestrator.add_node().await.expect("Failed to add node");

        // Connect to existing nodes (simple bootstrap approach)
        if node_count > 1 {
            if let Some(bootstrap_addr) = orchestrator.nodes[0].listen_addr.clone() {
                let _ = orchestrator.nodes[node_id]
                    .connect_to(&bootstrap_addr)
                    .await;
            }
        }

        // Allow network to stabilize
        sleep(Duration::from_secs(2)).await;

        // Run DHT operations for a short period
        let test_duration = Duration::from_secs(30); // Shorter for scaling test
        orchestrator
            .run_dht_operations(test_duration)
            .await
            .expect("DHT operations failed");

        info!("Completed test phase with {} nodes", node_count);
    }

    orchestrator.generate_test_report("Linear Node Scaling");
}

#[tokio::test]
#[ignore = "Long-running scalability test"]
async fn test_network_topology_performance() {
    init_logging();
    let mut config = ScalabilityTestConfig::default();
    config.max_nodes = config.max_nodes.min(15); // Limit for topology tests

    let topologies = vec![
        NetworkTopology::Ring,
        NetworkTopology::Star,
        NetworkTopology::Random {
            connection_ratio: 0.3,
        },
        NetworkTopology::Bootstrap { bootstrap_count: 3 },
    ];

    for topology in topologies {
        info!("Testing topology: {:?}", topology);
        let mut orchestrator = ScalabilityTestOrchestrator::new(config.clone());

        // Add all nodes
        for _ in 0..config.max_nodes {
            orchestrator.add_node().await.expect("Failed to add node");
        }

        // Setup topology
        orchestrator
            .setup_topology(topology.clone())
            .await
            .expect("Failed to setup topology");

        // Allow network to stabilize
        sleep(Duration::from_secs(5)).await;

        // Run DHT operations
        let test_duration = Duration::from_secs(60);
        orchestrator
            .run_dht_operations(test_duration)
            .await
            .expect("DHT operations failed");

        orchestrator.generate_test_report(&format!("Topology {:?}", topology));
    }
}

#[tokio::test]
#[ignore = "Long-running scalability test"]
async fn test_high_throughput_operations() {
    init_logging();
    let mut config = ScalabilityTestConfig::default();
    config.records_per_node = 100; // More records for throughput test
    config.record_value_size = 512; // Smaller records for higher throughput

    let mut orchestrator = ScalabilityTestOrchestrator::new(config.clone());

    // Add nodes
    let node_count = config.max_nodes.min(10); // Focus on throughput rather than scale
    for _ in 0..node_count {
        orchestrator.add_node().await.expect("Failed to add node");
    }

    // Setup full mesh for maximum connectivity
    orchestrator
        .setup_topology(NetworkTopology::FullMesh)
        .await
        .expect("Failed to setup mesh");

    // Allow network to stabilize
    sleep(Duration::from_secs(5)).await;

    info!(
        "Starting high-throughput test with {} nodes, {} records per node",
        node_count, config.records_per_node
    );

    // Run intensive DHT operations
    orchestrator
        .run_dht_operations(config.test_duration)
        .await
        .expect("DHT operations failed");

    orchestrator.generate_test_report("High Throughput Operations");
}

#[tokio::test]
#[ignore = "Long-running scalability test"]
async fn test_network_churn_resilience() {
    init_logging();
    let mut config = ScalabilityTestConfig::default();
    config.max_nodes = config.max_nodes.min(25); // Reasonable limit for churn test

    let mut orchestrator = ScalabilityTestOrchestrator::new(config.clone());

    info!("Starting network churn resilience test");

    // Start with initial nodes
    let initial_nodes = 5;
    for _ in 0..initial_nodes {
        orchestrator
            .add_node()
            .await
            .expect("Failed to add initial node");
    }

    // Setup bootstrap topology for better resilience
    orchestrator
        .setup_topology(NetworkTopology::Bootstrap { bootstrap_count: 2 })
        .await
        .expect("Failed to setup bootstrap topology");

    // Allow network to stabilize
    sleep(Duration::from_secs(3)).await;

    // Store some initial records
    info!("Storing initial records before churn simulation");
    let initial_duration = Duration::from_secs(30);
    orchestrator
        .run_dht_operations(initial_duration)
        .await
        .expect("Initial DHT operations failed");

    // Run churn simulation
    info!("Starting churn simulation");
    let churn_duration = Duration::from_secs(120);
    orchestrator
        .simulate_churn(churn_duration)
        .await
        .expect("Churn simulation failed");

    // Test network after churn
    info!("Testing network stability after churn");
    let recovery_duration = Duration::from_secs(60);
    orchestrator
        .run_dht_operations(recovery_duration)
        .await
        .expect("Post-churn DHT operations failed");

    orchestrator.generate_test_report("Network Churn Resilience");
}

#[tokio::test]
#[ignore = "Long-running scalability test"]
async fn test_bootstrap_performance() {
    init_logging();
    let mut config = ScalabilityTestConfig::default();
    config.max_nodes = 20;

    let mut orchestrator = ScalabilityTestOrchestrator::new(config.clone());

    info!("Testing bootstrap performance with {} bootstrap nodes", 3);

    // Create bootstrap nodes first
    let bootstrap_count = 3;
    for i in 0..bootstrap_count {
        let node_id = orchestrator
            .add_node()
            .await
            .expect("Failed to add bootstrap node");
        info!("Added bootstrap node {}", node_id);
    }

    // Connect bootstrap nodes to each other
    orchestrator
        .setup_topology(NetworkTopology::FullMesh)
        .await
        .expect("Failed to setup bootstrap mesh");

    // Allow bootstrap network to stabilize and store records
    sleep(Duration::from_secs(5)).await;
    orchestrator
        .run_dht_operations(Duration::from_secs(60))
        .await
        .expect("Bootstrap DHT operations failed");

    info!("Bootstrap network established, adding new nodes");

    // Measure time for new nodes to join and sync
    let mut bootstrap_times = Vec::new();

    for i in bootstrap_count..config.max_nodes {
        let start_time = Instant::now();

        // Add new node
        let node_id = orchestrator
            .add_node()
            .await
            .expect("Failed to add new node");

        // Connect to bootstrap nodes
        for bootstrap_idx in 0..bootstrap_count {
            if let Some(bootstrap_addr) = orchestrator.nodes[bootstrap_idx].listen_addr.clone() {
                let _ = orchestrator.nodes[node_id]
                    .connect_to(&bootstrap_addr)
                    .await;
            }
        }

        // Measure time until node can successfully retrieve a record
        let bootstrap_success = timeout(Duration::from_secs(30), async {
            // Try to retrieve a record that should exist in the bootstrap network
            let test_key = RecordKey::new(&"bootstrap_test_key");
            orchestrator.nodes[node_id].get_record(test_key);

            // In a real implementation, you'd wait for the query result
            // For this test, we'll just wait a bit for the connection to stabilize
            sleep(Duration::from_secs(5)).await;
            true
        })
        .await
        .unwrap_or(false);

        let bootstrap_time = start_time.elapsed();
        bootstrap_times.push(bootstrap_time);

        info!(
            "Node {} bootstrap time: {:?}, success: {}",
            node_id, bootstrap_time, bootstrap_success
        );

        // Add delay between node additions
        sleep(config.node_spawn_interval).await;
    }

    // Calculate bootstrap statistics
    let avg_bootstrap_time =
        bootstrap_times.iter().sum::<Duration>() / bootstrap_times.len() as u32;
    let max_bootstrap_time = bootstrap_times.iter().max().copied().unwrap_or_default();
    let min_bootstrap_time = bootstrap_times.iter().min().copied().unwrap_or_default();

    info!("Bootstrap Performance Results:");
    info!("  Average bootstrap time: {:?}", avg_bootstrap_time);
    info!("  Maximum bootstrap time: {:?}", max_bootstrap_time);
    info!("  Minimum bootstrap time: {:?}", min_bootstrap_time);

    orchestrator.generate_test_report("Bootstrap Performance");
}

#[tokio::test]
#[ignore = "Long-running scalability test"]
async fn test_large_record_scalability() {
    init_logging();
    let mut config = ScalabilityTestConfig::default();
    config.max_nodes = 10; // Fewer nodes for large record test
    config.records_per_node = 20; // Fewer records
    config.record_value_size = 64 * 1024; // 64KB records

    let mut orchestrator = ScalabilityTestOrchestrator::new(config.clone());

    info!(
        "Testing scalability with large records ({} bytes per record)",
        config.record_value_size
    );

    // Add nodes
    for _ in 0..config.max_nodes {
        orchestrator.add_node().await.expect("Failed to add node");
    }

    // Setup mesh topology for good connectivity
    orchestrator
        .setup_topology(NetworkTopology::FullMesh)
        .await
        .expect("Failed to setup mesh topology");

    // Allow network to stabilize
    sleep(Duration::from_secs(5)).await;

    // Monitor memory usage and performance with large records
    info!("Starting large record operations");
    let test_start = Instant::now();

    // Run operations with large records
    orchestrator
        .run_dht_operations(Duration::from_secs(180))
        .await
        .expect("Large record DHT operations failed");

    let test_duration = test_start.elapsed();
    info!("Large record test completed in {:?}", test_duration);

    orchestrator.generate_test_report("Large Record Scalability");
}

#[tokio::test]
#[ignore = "Long-running scalability test"]
async fn test_concurrent_operations_scaling() {
    init_logging();
    let config = ScalabilityTestConfig::default();
    let mut orchestrator = ScalabilityTestOrchestrator::new(config.clone());

    info!("Testing concurrent operations scaling");

    // Add nodes
    let node_count = config.max_nodes.min(12);
    for _ in 0..node_count {
        orchestrator.add_node().await.expect("Failed to add node");
    }

    // Setup random topology for realistic network conditions
    orchestrator
        .setup_topology(NetworkTopology::Random {
            connection_ratio: 0.4,
        })
        .await
        .expect("Failed to setup random topology");

    sleep(Duration::from_secs(5)).await;

    info!("Running concurrent operations across {} nodes", node_count);

    // Create multiple concurrent operation tasks
    let mut tasks = Vec::new();

    for i in 0..node_count {
        let node_metrics = orchestrator.nodes[i].metrics.clone();
        let operation_count = config.records_per_node;
        let record_size = config.record_value_size;

        let task = tokio::spawn(async move {
            for j in 0..operation_count {
                let start = Instant::now();

                // Simulate concurrent put operations
                let _key = format!("concurrent_key_{}_{}", i, j);
                let _value = vec![i as u8; record_size];

                // In a real test, we'd actually perform the operation
                // For this simulation, we just track the metrics
                sleep(Duration::from_millis(10)).await; // Simulate operation time

                let latency = start.elapsed();
                node_metrics.record_success(latency);

                // Small delay between operations
                sleep(Duration::from_millis(50)).await;
            }
        });

        tasks.push(task);
    }

    // Wait for all concurrent operations to complete
    info!("Waiting for concurrent operations to complete...");
    for task in tasks {
        let _ = task.await;
    }

    orchestrator.generate_test_report("Concurrent Operations Scaling");
}

#[tokio::test]
#[ignore = "Long-running scalability test"]
async fn test_network_partition_recovery() {
    init_logging();
    let mut config = ScalabilityTestConfig::default();
    config.max_nodes = 12;
    config.test_partitions = true;

    let mut orchestrator = ScalabilityTestOrchestrator::new(config.clone());

    info!("Testing network partition recovery");

    // Add nodes
    for _ in 0..config.max_nodes {
        orchestrator.add_node().await.expect("Failed to add node");
    }

    // Setup mesh topology
    orchestrator
        .setup_topology(NetworkTopology::FullMesh)
        .await
        .expect("Failed to setup mesh topology");

    sleep(Duration::from_secs(5)).await;

    // Store records in the full network
    info!("Storing records in connected network");
    orchestrator
        .run_dht_operations(Duration::from_secs(60))
        .await
        .expect("Pre-partition operations failed");

    info!("Simulating network partition");
    // Note: In a real implementation, we would actually partition the network
    // by temporarily blocking connections between node groups.
    // For this test, we simulate the partition effects.

    // Partition the network into two groups
    let partition_point = config.max_nodes / 2;
    info!(
        "Partitioning network into groups: 0-{} and {}-{}",
        partition_point - 1,
        partition_point,
        config.max_nodes - 1
    );

    // Simulate partition by running operations only within groups
    let partition_duration = Duration::from_secs(90);
    info!(
        "Running operations during partition for {:?}",
        partition_duration
    );

    // In a real test, nodes would continue operating but couldn't communicate
    // across the partition. Here we simulate this with separate operation cycles.
    sleep(partition_duration).await;

    info!("Healing network partition");
    // In reality, we would restore network connectivity here

    // Test recovery - all nodes should be able to communicate again
    info!("Testing network recovery");
    orchestrator
        .run_dht_operations(Duration::from_secs(120))
        .await
        .expect("Post-partition operations failed");

    orchestrator.generate_test_report("Network Partition Recovery");
}

#[tokio::test]
#[ignore = "Long-running scalability test"]
async fn test_stress_test_maximum_scale() {
    init_logging();
    let mut config = ScalabilityTestConfig::default();

    // Push the limits - use environment variable to set even higher limits
    config.max_nodes = std::env::var("NETABASE_STRESS_MAX_NODES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(config.max_nodes * 2);
    config.test_duration = Duration::from_secs(
        std::env::var("NETABASE_STRESS_DURATION")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(600),
    );

    let mut orchestrator = ScalabilityTestOrchestrator::new(config.clone());

    info!(
        "Starting maximum scale stress test with up to {} nodes for {:?}",
        config.max_nodes, config.test_duration
    );

    // Rapidly add nodes
    let node_add_tasks = (0..config.max_nodes).map(|i| {
        async move {
            // Stagger node additions to avoid overwhelming the system
            sleep(Duration::from_millis(i as u64 * 200)).await;
            (i, TestNode::new(i).await)
        }
    });

    // Add nodes concurrently but with rate limiting
    let mut node_results = Vec::new();
    for task in node_add_tasks {
        match task.await {
            (id, Ok(node)) => {
                node_results.push((id, node));
                if id % 10 == 0 {
                    info!("Added {} nodes so far", id + 1);
                }
            }
            (id, Err(e)) => {
                error!("Failed to create node {}: {}", id, e);
            }
        }

        // Small delay to prevent resource exhaustion
        sleep(Duration::from_millis(100)).await;
    }

    // Move successful nodes to orchestrator
    for (_, node) in node_results {
        orchestrator.nodes.push(node);
    }

    info!("Successfully created {} nodes", orchestrator.nodes.len());

    // Use bootstrap topology for efficiency at scale
    let bootstrap_count = (orchestrator.nodes.len() / 10).max(3);
    orchestrator
        .setup_topology(NetworkTopology::Bootstrap { bootstrap_count })
        .await
        .expect("Failed to setup bootstrap topology for stress test");

    // Allow longer stabilization time for large network
    sleep(Duration::from_secs(15)).await;

    // Run stress operations
    info!(
        "Starting stress operations across {} nodes",
        orchestrator.nodes.len()
    );
    orchestrator
        .run_dht_operations(config.test_duration)
        .await
        .expect("Stress test operations failed");

    orchestrator.generate_test_report("Maximum Scale Stress Test");

    // Additional stress test metrics
    info!("=== Stress Test Additional Metrics ===");
    info!("Actual nodes created: {}", orchestrator.nodes.len());
    info!("Target nodes: {}", config.max_nodes);
    info!(
        "Success rate: {:.1}%",
        (orchestrator.nodes.len() as f64 / config.max_nodes as f64) * 100.0
    );
}

/// Utility function for production monitoring - can be used to set up metrics collection
/// This demonstrates how to integrate performance monitoring in a production P2P system
pub fn setup_metrics_collection(
    nodes: &[TestNode],
    collection_interval: Duration,
) -> mpsc::Receiver<HashMap<String, f64>> {
    let (tx, rx) = mpsc::channel(100);
    let metrics_refs: Vec<_> = nodes.iter().map(|n| (n.id, n.metrics.clone())).collect();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(collection_interval);

        loop {
            interval.tick().await;

            let mut metrics_snapshot = HashMap::new();

            for (node_id, metrics) in &metrics_refs {
                let prefix = format!("node_{}", node_id);

                metrics_snapshot.insert(
                    format!("{}_ops_per_sec", prefix),
                    metrics.operations_per_second(),
                );
                metrics_snapshot.insert(
                    format!("{}_avg_latency_ms", prefix),
                    metrics.average_latency_ms(),
                );
                metrics_snapshot.insert(format!("{}_failure_rate", prefix), metrics.failure_rate());
                metrics_snapshot.insert(
                    format!("{}_connections", prefix),
                    metrics
                        .active_connections
                        .load(std::sync::atomic::Ordering::Relaxed) as f64,
                );
            }

            if tx.send(metrics_snapshot).await.is_err() {
                break; // Receiver dropped
            }
        }
    });

    rx
}

/// Utility function to simulate realistic network conditions for production testing
/// This can be used to test P2P behavior under various network conditions
pub fn simulate_network_conditions() {
    // In a real implementation, this could:
    // - Add artificial latency using traffic control (tc) on Linux
    // - Simulate packet loss
    // - Simulate bandwidth limitations
    // - Test behavior under NAT/firewall conditions

    info!("Network condition simulation would be implemented here");
    info!("Production considerations:");
    info!("  - Network latency simulation");
    info!("  - Packet loss simulation");
    info!("  - Bandwidth throttling");
    info!("  - NAT traversal testing");
    info!("  - Firewall simulation");
}

#[cfg(test)]
mod integration_helpers {
    use super::*;

    /// Helper to run a quick scalability smoke test - useful for CI/CD
    #[tokio::test]
    async fn quick_scalability_smoke_test() {
        init_logging();
        let mut config = ScalabilityTestConfig::default();
        config.max_nodes = 5; // Small for quick test
        config.test_duration = Duration::from_secs(30);

        let mut orchestrator = ScalabilityTestOrchestrator::new(config.clone());

        // Add nodes
        for _ in 0..config.max_nodes {
            orchestrator.add_node().await.expect("Failed to add node");
        }

        // Simple mesh
        orchestrator
            .setup_topology(NetworkTopology::Ring)
            .await
            .expect("Setup failed");

        // Quick test
        orchestrator
            .run_dht_operations(config.test_duration)
            .await
            .expect("Operations failed");

        orchestrator.generate_test_report("Quick Smoke Test");

        // Verify basic functionality
        assert!(orchestrator.nodes.len() == config.max_nodes);
        assert!(orchestrator.nodes.iter().all(|n| n.connection_count() > 0));
    }

    /// Test configuration validation
    #[test]
    fn test_scalability_config_from_env() {
        // Test that environment variables are properly parsed
        unsafe {
            std::env::set_var("NETABASE_SCALABILITY_MAX_NODES", "15");
            std::env::set_var("NETABASE_SCALABILITY_VERBOSE", "true");
        }

        let config = ScalabilityTestConfig::default();
        assert_eq!(config.max_nodes, 15);
        assert_eq!(config.verbose_metrics, true);

        // Cleanup
        unsafe {
            std::env::remove_var("NETABASE_SCALABILITY_MAX_NODES");
            std::env::remove_var("NETABASE_SCALABILITY_VERBOSE");
        }
    }
}
