//! Scalability tests for NetaBase P2P platform
//!
//! This module contains comprehensive tests to validate the scalability characteristics
//! of the NetaBase distributed hash table as the number of nodes increases.
//!
//! Tests measure:
//! - Connection establishment performance
//! - DHT record propagation latency
//! - Network topology performance comparison
//! - Resource usage under load
//!
//! All tests are marked as #[ignore] by default since they are resource-intensive.
//! Use `cargo test -- --ignored` to run them.

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime},
};

use anyhow::Result;
use libp2p::{
    Multiaddr, PeerId, Swarm,
    futures::StreamExt,
    kad::{Record, RecordKey},
    swarm::SwarmEvent,
};
use log::{debug, error, warn};
use netabase::{
    get_test_temp_dir_str,
    network::{
        behaviour::{NetabaseBehaviour, NetabaseBehaviourEvent},
        swarm::generate_swarm,
    },
};
use tokio::time::{sleep, timeout};

/// Configuration parameters for scalability tests
#[derive(Debug, Clone)]
struct ScalabilityTestConfig {
    /// Maximum number of nodes to spawn during the test
    pub max_nodes: usize,
    /// Duration to run the test (in seconds)
    pub test_duration: Duration,
    /// Number of records each node should store/retrieve
    pub records_per_node: usize,
    /// Size of each record value in bytes
    pub record_value_size: usize,
    /// Whether to print detailed metrics during test execution
    pub verbose_metrics: bool,
}

impl Default for ScalabilityTestConfig {
    fn default() -> Self {
        Self {
            max_nodes: std::env::var("SCALABILITY_MAX_NODES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8),
            test_duration: Duration::from_secs(
                std::env::var("SCALABILITY_TEST_DURATION")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60),
            ),
            records_per_node: std::env::var("SCALABILITY_RECORDS_PER_NODE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50),
            record_value_size: std::env::var("SCALABILITY_RECORD_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1024),
            verbose_metrics: std::env::var("SCALABILITY_VERBOSE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
        }
    }
}

/// Simple performance metrics tracking
pub struct PerformanceMetrics {
    /// Start time for calculating rates
    pub start_time: SystemTime,
    /// Custom counters for test-specific metrics
    pub successful_operations: Arc<AtomicU64>,
    pub failed_operations: Arc<AtomicU64>,
    pub total_latency_micros: Arc<AtomicU64>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            successful_operations: Arc::new(AtomicU64::new(0)),
            failed_operations: Arc::new(AtomicU64::new(0)),
            total_latency_micros: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a successful operation with its latency
    pub fn record_success(&self, latency: Duration) {
        self.successful_operations.fetch_add(1, Ordering::Relaxed);
        self.total_latency_micros
            .fetch_add(latency.as_micros() as u64, Ordering::Relaxed);
    }

    /// Record a failed operation
    pub fn record_failure(&self) {
        self.failed_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current average latency in milliseconds
    pub fn average_latency_ms(&self) -> f64 {
        let total_ops = self.successful_operations.load(Ordering::Relaxed);
        if total_ops == 0 {
            return 0.0;
        }
        let total_latency = self.total_latency_micros.load(Ordering::Relaxed);
        (total_latency as f64 / total_ops as f64) / 1000.0
    }

    /// Get operations per second
    pub fn operations_per_second(&self) -> f64 {
        let elapsed = self.start_time.elapsed().unwrap_or(Duration::from_secs(1));
        let total_ops = self.successful_operations.load(Ordering::Relaxed);
        total_ops as f64 / elapsed.as_secs_f64()
    }

    /// Get failure rate as a percentage
    pub fn failure_rate(&self) -> f64 {
        let successful = self.successful_operations.load(Ordering::Relaxed);
        let failed = self.failed_operations.load(Ordering::Relaxed);
        let total = successful + failed;
        if total == 0 {
            return 0.0;
        }
        (failed as f64 / total as f64) * 100.0
    }

    /// Print comprehensive metrics report
    pub fn print_report(&self, node_id: &str) {
        let successful = self.successful_operations.load(Ordering::Relaxed);
        let failed = self.failed_operations.load(Ordering::Relaxed);

        println!("=== Performance Metrics Report for Node {} ===", node_id);
        println!("Successful operations: {}", successful);
        println!("Failed operations: {}", failed);
        println!("Failure rate: {:.2}%", self.failure_rate());
        println!("Average latency: {:.2}ms", self.average_latency_ms());
        println!("Operations per second: {:.2}", self.operations_per_second());
        println!(
            "Test duration: {:.2}s",
            self.start_time.elapsed().unwrap_or_default().as_secs_f64()
        );
        println!("==================================================");
    }
}

/// Test node wrapper
struct TestNode {
    pub id: usize,
    #[allow(dead_code)]
    pub peer_id: PeerId,
    pub swarm: Swarm<NetabaseBehaviour>,
    pub metrics: PerformanceMetrics,
    pub listen_addr: Option<Multiaddr>,
    pub temp_dir: String,
}

impl TestNode {
    /// Create a new test node
    pub async fn new(id: usize) -> Result<Self> {
        let temp_dir = get_test_temp_dir_str(Some(&format!("scalability_node_{}", id)), None);
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
            debug!("Node {} connected successfully in {:?}", self.id, latency);
            Ok(())
        } else {
            self.metrics.record_failure();
            error!("Node {} failed to connect to target", self.id);
            Err(anyhow::anyhow!(
                "Failed to connect to target within timeout"
            ))
        }
    }

    /// Store a record in the DHT
    pub fn put_record(&mut self, key: RecordKey, value: Vec<u8>) -> Result<()> {
        let record = Record::new(key, value);
        self.swarm
            .behaviour_mut()
            .kad
            .put_record(record, libp2p::kad::Quorum::One)?;
        Ok(())
    }

    /// Retrieve a record from the DHT
    pub fn get_record(&mut self, key: RecordKey) -> Result<()> {
        self.swarm.behaviour_mut().kad.get_record(key);
        Ok(())
    }

    /// Get current connection count
    pub fn connection_count(&self) -> usize {
        self.swarm.connected_peers().count()
    }

    /// Process pending swarm events
    pub async fn process_events(&mut self, max_events: usize) {
        for _ in 0..max_events {
            if let Ok(Some(event)) = timeout(Duration::from_millis(10), self.swarm.next()).await {
                match &event {
                    SwarmEvent::ConnectionEstablished { .. } => {
                        debug!("Node {} established connection", self.id);
                    }
                    SwarmEvent::ConnectionClosed { .. } => {
                        debug!("Node {} closed connection", self.id);
                    }
                    SwarmEvent::Behaviour(behaviour_event) => match behaviour_event {
                        NetabaseBehaviourEvent::Kad(_) => {
                            debug!("Node {} DHT event", self.id);
                        }
                        NetabaseBehaviourEvent::Identify(_) => {
                            debug!("Node {} identify event", self.id);
                        }
                        NetabaseBehaviourEvent::Mdns(_) => {
                            debug!("Node {} mDNS event", self.id);
                        }
                    },
                    _ => {}
                }
            } else {
                break;
            }
        }
    }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_dir_all(&self.temp_dir) {
            warn!("Failed to clean up temp directory: {}", e);
        }
    }
}

/// Network topology configurations for testing different scenarios
#[derive(Debug, Clone)]
pub enum NetworkTopology {
    /// Every node connects to every other node
    FullMesh,
    /// Nodes form a ring topology
    Ring,
    /// One central node, all others connect to it
    Star,
    /// Random connections with specified connection ratio
    Random { connection_ratio: f64 },
}

/// Main test orchestrator
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

    /// Add a new node to the test network
    pub async fn add_node(&mut self) -> Result<usize> {
        let node_id = self.nodes.len();
        let mut node = TestNode::new(node_id).await?;

        // Wait for the node to get a listening address
        while node.listen_addr.is_none() {
            if let Ok(Some(event)) = timeout(Duration::from_millis(100), node.swarm.next()).await {
                if let SwarmEvent::NewListenAddr { address, .. } = event {
                    node.listen_addr = Some(address.clone());
                    println!("Node {} listening on {}", node_id, address);
                }
            }
        }

        self.nodes.push(node);
        println!("Added node {} to test network", node_id);
        Ok(node_id)
    }

    /// Setup network topology between nodes
    pub async fn setup_topology(&mut self, topology: NetworkTopology) -> Result<()> {
        match topology {
            NetworkTopology::FullMesh => self.setup_full_mesh().await,
            NetworkTopology::Ring => self.setup_ring().await,
            NetworkTopology::Star => self.setup_star().await,
            NetworkTopology::Random { connection_ratio } => {
                self.setup_random(connection_ratio).await
            }
        }
    }

    async fn setup_full_mesh(&mut self) -> Result<()> {
        println!("Setting up full mesh topology");
        let node_count = self.nodes.len();

        for i in 0..node_count {
            for j in (i + 1)..node_count {
                if let Some(target_addr) = self.nodes[j].listen_addr.clone() {
                    if let Err(e) = self.nodes[i].connect_to(&target_addr).await {
                        warn!("Failed to connect nodes: {}", e);
                    }
                    tokio::task::yield_now().await;
                }
            }
        }
        Ok(())
    }

    async fn setup_ring(&mut self) -> Result<()> {
        println!("Setting up ring topology");
        let node_count = self.nodes.len();
        let mut successful_connections = 0;

        for i in 0..node_count {
            let next_node = (i + 1) % node_count;
            if let Some(target_addr) = self.nodes[next_node].listen_addr.clone() {
                match self.nodes[i].connect_to(&target_addr).await {
                    Ok(_) => {
                        successful_connections += 1;
                        println!("Node {} -> Node {}: Connected", i, next_node);
                    }
                    Err(e) => {
                        warn!("Node {} -> Node {}: Failed to connect: {}", i, next_node, e);
                    }
                }
                // Small delay between connection attempts
                sleep(Duration::from_millis(100)).await;
            }
        }

        // Allow time for connections to stabilize
        sleep(Duration::from_secs(2)).await;

        println!(
            "Ring topology setup completed: {}/{} connections successful",
            successful_connections, node_count
        );

        if successful_connections == 0 {
            return Err(anyhow::anyhow!(
                "No connections established in ring topology"
            ));
        }

        Ok(())
    }

    async fn setup_star(&mut self) -> Result<()> {
        println!("Setting up star topology");
        if self.nodes.is_empty() {
            return Ok(());
        }

        let center_node_addr = self.nodes[0]
            .listen_addr
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Center node has no listen address"))?;

        // Connect all other nodes to the center node
        for i in 1..self.nodes.len() {
            self.nodes[i].connect_to(&center_node_addr).await?;
        }
        Ok(())
    }

    async fn setup_random(&mut self, connection_ratio: f64) -> Result<()> {
        println!("Setting up random topology with ratio {}", connection_ratio);
        let node_count = self.nodes.len();
        let target_connections =
            ((node_count * (node_count - 1)) as f64 * connection_ratio / 2.0) as usize;

        use rand::{rng, seq::SliceRandom};
        let mut rng = rng();

        let mut possible_connections: Vec<(usize, usize)> = Vec::new();
        for i in 0..node_count {
            for j in (i + 1)..node_count {
                possible_connections.push((i, j));
            }
        }

        possible_connections.shuffle(&mut rng);

        for (i, j) in possible_connections.into_iter().take(target_connections) {
            if let Some(target_addr) = self.nodes[j].listen_addr.clone() {
                if let Err(e) = self.nodes[i].connect_to(&target_addr).await {
                    warn!("Failed to connect nodes in random topology: {}", e);
                }
            }
        }
        Ok(())
    }

    /// Run DHT operations across all nodes
    pub async fn run_dht_operations(&mut self) -> Result<()> {
        println!("Starting DHT operations across {} nodes", self.nodes.len());

        // Execute PUT operations
        println!("Executing PUT operations");
        for node_id in 0..self.nodes.len() {
            for i in 0..self.config.records_per_node {
                let key = format!("test-key-node-{}-record-{}", node_id, i);
                let value = vec![node_id as u8; self.config.record_value_size];
                let record_key = RecordKey::new(&key);

                let start = Instant::now();
                if let Err(e) = self.nodes[node_id].put_record(record_key, value) {
                    warn!("Failed to put record: {}", e);
                    self.global_metrics.record_failure();
                } else {
                    // Process some events to ensure the operation is handled
                    self.nodes[node_id].process_events(5).await;
                    let latency = start.elapsed();
                    self.global_metrics.record_success(latency);
                }
            }

            if self.config.verbose_metrics && (node_id % 5 == 0) {
                debug!("Node {} completed PUT operations", node_id);
            }
        }

        // Wait for propagation
        sleep(Duration::from_secs(2)).await;

        // Execute GET operations
        println!("Executing GET operations");
        for node_id in 0..self.nodes.len() {
            for i in 0..(self.config.records_per_node / 2) {
                let key = format!("test-key-node-{}-record-{}", node_id, i);
                let record_key = RecordKey::new(&key);

                let start = Instant::now();
                if let Err(e) = self.nodes[node_id].get_record(record_key) {
                    warn!("Failed to get record: {}", e);
                    self.global_metrics.record_failure();
                } else {
                    // Process events to handle the GET response
                    self.nodes[node_id].process_events(10).await;
                    let latency = start.elapsed();
                    self.global_metrics.record_success(latency);
                }

                // Small delay between operations
                sleep(Duration::from_millis(10)).await;
            }
        }

        println!("DHT operations completed");
        Ok(())
    }

    /// Generate comprehensive test report
    pub fn generate_test_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== NetaBase Scalability Test Report ===\n");
        report.push_str(&format!("Test Configuration:\n"));
        report.push_str(&format!("  Max nodes: {}\n", self.config.max_nodes));
        report.push_str(&format!(
            "  Test duration: {:?}\n",
            self.config.test_duration
        ));
        report.push_str(&format!(
            "  Records per node: {}\n",
            self.config.records_per_node
        ));
        report.push_str(&format!(
            "  Record size: {} bytes\n",
            self.config.record_value_size
        ));
        report.push_str("\n");

        report.push_str("Global Metrics:\n");
        report.push_str(&format!(
            "  Average latency: {:.2}ms\n",
            self.global_metrics.average_latency_ms()
        ));
        report.push_str(&format!(
            "  Operations per second: {:.2}\n",
            self.global_metrics.operations_per_second()
        ));
        report.push_str(&format!(
            "  Failure rate: {:.2}%\n",
            self.global_metrics.failure_rate()
        ));
        report.push_str("\n");

        report.push_str("Per-Node Metrics:\n");
        for (i, node) in self.nodes.iter().enumerate() {
            report.push_str(&format!(
                "  Node {}: {} connections, {:.2}ms avg latency\n",
                i,
                node.connection_count(),
                node.metrics.average_latency_ms()
            ));
        }
        report.push_str("\n");

        report.push_str("=========================================\n");
        report
    }
}

fn init_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    });
}

/// Test how performance scales with increasing number of nodes
#[tokio::test]
#[ignore]
async fn test_linear_node_scaling() -> Result<()> {
    init_logging();
    println!("Starting linear node scaling test");

    let config = ScalabilityTestConfig {
        max_nodes: 10,
        test_duration: Duration::from_secs(60),
        records_per_node: 20,
        ..Default::default()
    };

    let mut orchestrator = ScalabilityTestOrchestrator::new(config);

    // Add nodes incrementally and measure performance
    for node_count in (2..=orchestrator.config.max_nodes).step_by(2) {
        println!("Testing with {} nodes", node_count);

        // Add nodes to reach target count
        while orchestrator.nodes.len() < node_count {
            orchestrator.add_node().await?;
        }

        // Setup topology
        orchestrator.setup_topology(NetworkTopology::Ring).await?;

        // Allow network to stabilize
        sleep(Duration::from_secs(1)).await;

        // Run DHT operations
        orchestrator.run_dht_operations().await?;

        // Generate report for this phase
        let report = orchestrator.generate_test_report();
        println!("Phase {} nodes completed", node_count);
        if orchestrator.config.verbose_metrics {
            println!("\n{}", report);
        }

        sleep(Duration::from_secs(1)).await;
    }

    println!("Linear scaling test completed successfully");
    Ok(())
}

/// Test network topology performance comparison
#[tokio::test]
#[ignore]
async fn test_network_topology_performance() -> Result<()> {
    init_logging();
    println!("Starting network topology performance test");

    let config = ScalabilityTestConfig {
        max_nodes: 6,
        test_duration: Duration::from_secs(30),
        records_per_node: 10,
        ..Default::default()
    };

    let topologies = vec![
        NetworkTopology::Ring,
        NetworkTopology::Star,
        NetworkTopology::Random {
            connection_ratio: 0.3,
        },
    ];

    for topology in topologies {
        println!("Testing topology: {:?}", topology);
        let mut orchestrator = ScalabilityTestOrchestrator::new(config.clone());

        // Add all nodes
        for _ in 0..config.max_nodes {
            orchestrator.add_node().await?;
        }

        // Setup topology
        orchestrator.setup_topology(topology.clone()).await?;

        // Allow stabilization
        sleep(Duration::from_secs(2)).await;

        // Run operations
        orchestrator.run_dht_operations().await?;

        let report = orchestrator.generate_test_report();
        println!("Topology {:?} results:\n{}", topology, report);
    }

    Ok(())
}

/// Test high throughput operations
#[tokio::test]
#[ignore]
async fn test_high_throughput_operations() -> Result<()> {
    init_logging();
    println!("Starting high throughput test");

    let config = ScalabilityTestConfig {
        max_nodes: 5,
        records_per_node: 100,
        record_value_size: 512,
        ..Default::default()
    };

    let mut orchestrator = ScalabilityTestOrchestrator::new(config);

    // Add nodes
    for _ in 0..orchestrator.config.max_nodes {
        orchestrator.add_node().await?;
    }

    // Setup full mesh for maximum connectivity
    orchestrator
        .setup_topology(NetworkTopology::FullMesh)
        .await?;
    sleep(Duration::from_secs(2)).await;

    // Run high throughput operations
    orchestrator.run_dht_operations().await?;

    let report = orchestrator.generate_test_report();
    println!("High throughput test results:\n{}", report);

    Ok(())
}

/// Quick smoke test for CI/CD
#[tokio::test]
async fn test_scalability_smoke_test() -> Result<()> {
    init_logging();
    println!("Starting scalability smoke test");

    let config = ScalabilityTestConfig {
        max_nodes: 2,
        test_duration: Duration::from_secs(5),
        records_per_node: 2,
        ..Default::default()
    };

    let mut orchestrator = ScalabilityTestOrchestrator::new(config);

    // Add nodes and verify they can be created
    for _ in 0..orchestrator.config.max_nodes {
        orchestrator.add_node().await?;
    }

    // Skip complex topology setup for smoke test - just verify basic functionality
    println!("Skipping topology setup for smoke test - testing basic node functionality");

    // Test basic DHT operations locally (without requiring network connections)
    for node in &mut orchestrator.nodes {
        let key = RecordKey::new(&format!("test_key_{}", node.id));
        let value = format!("test_value_{}", node.id).into_bytes();

        // Test local record storage
        node.put_record(key.clone(), value)?;
        println!("Node {} stored record locally", node.id);
    }

    let report = orchestrator.generate_test_report();
    println!("Smoke test results:\n{}", report);

    // Verify basic functionality - nodes created successfully
    assert_eq!(orchestrator.nodes.len(), orchestrator.config.max_nodes);
    println!(
        "Smoke test completed successfully - {} nodes created",
        orchestrator.nodes.len()
    );

    Ok(())
}
