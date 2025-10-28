//! Multi-process network test node for testing sync protocols
//! This binary can be run as separate processes to test network communication

use clap::Parser;
use libp2p::{
    identity,
    Multiaddr, PeerId,
};
use netabase::sync::{
    SyncBehaviorManager, SyncManagerConfigBuilder,
    PaxosInstance, PaxosConfig, PaxosMessage, ProposalNumber,
    BrbManager, BrbConfig,
    VectorClock, SyncRecord, StateDigest, Version,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use log::{info, warn, error};

#[derive(Parser, Debug)]
#[command(name = "sync-network-node")]
#[command(about = "Network node for testing sync protocols", long_about = None)]
struct Args {
    /// Node ID (0, 1, 2, ...)
    #[arg(short, long)]
    node_id: u8,

    /// Listen port
    #[arg(short, long)]
    port: u16,

    /// Bootstrap peers (comma-separated multiaddrs)
    #[arg(short, long)]
    bootstrap: Option<String>,

    /// Test mode: paxos, brb, gossip, sybil, full
    #[arg(short, long, default_value = "full")]
    test_mode: String,

    /// Number of faulty nodes to tolerate
    #[arg(short, long, default_value = "1")]
    max_failures: usize,

    /// Paxos enabled
    #[arg(long, default_value = "true")]
    paxos_enabled: bool,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestData {
    key: String,
    value: String,
    timestamp: u64,
}

/// Test state for the node
struct NodeState {
    peer_id: PeerId,
    node_id: u8,
    paxos: Option<PaxosInstance>,
    brb: Option<BrbManager>,
    sync_manager: SyncBehaviorManager,
    vector_clock: VectorClock,
    data_store: HashMap<Vec<u8>, SyncRecord>,
    test_results: TestResults,
    paxos_enabled: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct TestResults {
    paxos_proposals: u32,
    paxos_consensuses: u32,
    brb_broadcasts: u32,
    brb_deliveries: u32,
    gossip_rounds: u32,
    state_syncs: u32,
    pow_challenges_issued: u32,
    pow_challenges_verified: u32,
    errors: Vec<String>,
}

impl NodeState {
    fn new(
        peer_id: PeerId,
        node_id: u8,
        max_failures: usize,
        paxos_enabled: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let num_nodes = 3 * max_failures + 1;

        // Initialize Paxos if enabled
        let paxos = if paxos_enabled {
            let paxos_config = PaxosConfig::new(num_nodes, max_failures);
            Some(PaxosInstance::new(peer_id, paxos_config))
        } else {
            None
        };

        // Initialize BRB
        let brb_config = BrbConfig::new(num_nodes, max_failures)?;
        let brb = Some(BrbManager::new(brb_config, peer_id)?);

        // Initialize SyncManager
        let sync_config = SyncManagerConfigBuilder::new()
            .gossip_interval(Duration::from_secs(3))
            .gossip_fanout(2)
            .brb_config(num_nodes, max_failures)
            .pow_difficulty(4)
            .pow_enabled(true)
            .challenge_duration(Duration::from_secs(60))
            .build();

        let sync_manager = SyncBehaviorManager::new(peer_id, sync_config)?;

        // Initialize vector clock
        let vector_clock = VectorClock::new(peer_id);

        Ok(Self {
            peer_id,
            node_id,
            paxos,
            brb,
            sync_manager,
            vector_clock,
            data_store: HashMap::new(),
            test_results: TestResults::default(),
            paxos_enabled,
        })
    }

    /// Test Paxos consensus
    async fn test_paxos_propose(&mut self, value: Vec<u8>) -> Result<(), Box<dyn Error>> {
        if !self.paxos_enabled {
            warn!("Paxos is not enabled");
            return Ok(());
        }

        let paxos = self.paxos.as_mut().ok_or("Paxos not initialized")?;

        info!("Node {} proposing value via Paxos", self.node_id);
        let proposal = paxos.propose(value);
        self.test_results.paxos_proposals += 1;

        info!("Node {} created proposal: {:?}", self.node_id, proposal);
        Ok(())
    }

    /// Handle Paxos prepare message
    fn handle_paxos_prepare(
        &mut self,
        proposal_number: ProposalNumber,
    ) -> Result<PaxosMessage, Box<dyn Error>> {
        if !self.paxos_enabled {
            return Err("Paxos is not enabled".into());
        }

        let paxos = self.paxos.as_mut().ok_or("Paxos not initialized")?;
        let response = paxos.handle_prepare(proposal_number)?;
        info!("Node {} responded to prepare with promise", self.node_id);
        Ok(response)
    }

    /// Handle Paxos promise message
    fn handle_paxos_promise(
        &mut self,
        from: PeerId,
        proposal_number: ProposalNumber,
        accepted_proposal: Option<ProposalNumber>,
        accepted_value: Option<Vec<u8>>,
    ) -> Result<Option<PaxosMessage>, Box<dyn Error>> {
        if !self.paxos_enabled {
            return Err("Paxos is not enabled".into());
        }

        let paxos = self.paxos.as_mut().ok_or("Paxos not initialized")?;
        let result = paxos.handle_promise(from, proposal_number, accepted_proposal, accepted_value)?;

        if result.is_some() {
            info!("Node {} reached quorum, moving to accept phase", self.node_id);
        }

        Ok(result)
    }

    /// Handle Paxos accept message
    fn handle_paxos_accept(
        &mut self,
        proposal_number: ProposalNumber,
        value: Vec<u8>,
    ) -> Result<PaxosMessage, Box<dyn Error>> {
        if !self.paxos_enabled {
            return Err("Paxos is not enabled".into());
        }

        let paxos = self.paxos.as_mut().ok_or("Paxos not initialized")?;
        let response = paxos.handle_accept(proposal_number, value)?;
        info!("Node {} accepted proposal", self.node_id);
        Ok(response)
    }

    /// Handle Paxos accepted message
    fn handle_paxos_accepted(
        &mut self,
        from: PeerId,
        proposal_number: ProposalNumber,
        value: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        if !self.paxos_enabled {
            return Err("Paxos is not enabled".into());
        }

        let paxos = self.paxos.as_mut().ok_or("Paxos not initialized")?;
        paxos.handle_accepted(from, proposal_number, value)?;

        // Check if consensus reached
        if paxos.is_proposal_successful(&proposal_number) {
            info!("Node {} reached consensus!", self.node_id);
            self.test_results.paxos_consensuses += 1;
        }

        Ok(())
    }

    /// Test BRB broadcast
    async fn test_brb_broadcast(&mut self, message: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let brb = self.brb.as_mut().ok_or("BRB not initialized")?;

        info!("Node {} initiating BRB broadcast", self.node_id);

        // Create version
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let content_hash = blake3::hash(&message);
        let version = Version {
            clock: self.vector_clock.clone(),
            content_hash: *content_hash.as_bytes(),
            timestamp,
        };

        let (hash, _peers) = brb.initiate_broadcast(message, version)?;
        self.test_results.brb_broadcasts += 1;

        info!("Node {} initiated BRB broadcast with hash: {:?}", self.node_id, &hash[..8]);

        Ok(())
    }

    /// Handle BRB echo message
    fn handle_brb_echo(
        &mut self,
        from: PeerId,
        message_hash: [u8; 32],
        sender: PeerId,
    ) -> Result<(), Box<dyn Error>> {
        let brb = self.brb.as_mut().ok_or("BRB not initialized")?;

        // Handle echo - returns BrbAction
        let _action = brb.handle_echo(&from, message_hash, sender)?;

        info!("Node {} handled BRB echo from {}", self.node_id, from);

        Ok(())
    }

    /// Handle BRB ready message
    fn handle_brb_ready(
        &mut self,
        from: PeerId,
        message_hash: [u8; 32],
    ) -> Result<(), Box<dyn Error>> {
        let brb = self.brb.as_mut().ok_or("BRB not initialized")?;

        // Handle ready - returns BrbAction
        let _action = brb.handle_ready(&from, message_hash, from)?;

        info!("Node {} handled BRB ready from {}", self.node_id, from);

        Ok(())
    }

    /// Initiate gossip round
    fn initiate_gossip(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Node {} initiating gossip round", self.node_id);

        // Get state digest
        let digest = self.get_state_digest();

        self.test_results.gossip_rounds += 1;

        Ok(())
    }

    /// Get state digest for gossip
    fn get_state_digest(&self) -> StateDigest {
        // Calculate merkle root from data store
        let keys: Vec<Vec<u8>> = self.data_store.keys().cloned().collect();
        let merkle_root = self.calculate_merkle_root(&keys);

        StateDigest {
            merkle_root,
            record_count: self.data_store.len(),
            clock: self.vector_clock.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Calculate merkle root (simplified)
    fn calculate_merkle_root(&self, keys: &[Vec<u8>]) -> [u8; 32] {
        if keys.is_empty() {
            return [0u8; 32];
        }

        let mut combined = Vec::new();
        for key in keys {
            combined.extend_from_slice(key);
        }

        let hash = blake3::hash(&combined);
        *hash.as_bytes()
    }

    /// Store data
    fn put_data(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Box<dyn Error>> {
        // Increment vector clock
        self.vector_clock.increment();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let record = SyncRecord {
            key: key.clone(),
            value,
            version: timestamp,
            peer_id: self.peer_id.to_bytes(),
            vector_clock: self.vector_clock.clone(),
        };

        self.data_store.insert(key, record);

        Ok(())
    }

    /// Get data history
    fn get_history(&self, key: &[u8]) -> Vec<SyncRecord> {
        // If paxos is enabled, history should include all consensus values
        if self.paxos_enabled {
            if let Some(paxos) = &self.paxos {
                let learned = paxos.learned_values();
                info!("Node {} has {} learned values from Paxos", self.node_id, learned.len());
            }
        }

        // Return history for the key
        if let Some(record) = self.data_store.get(key) {
            vec![record.clone()]
        } else {
            vec![]
        }
    }

    /// Issue PoW challenge
    fn issue_pow_challenge(&mut self, peer: PeerId) -> Vec<u8> {
        let challenge = self.sync_manager.issue_challenge(peer);
        self.test_results.pow_challenges_issued += 1;
        challenge
    }

    /// Get test results
    fn get_results(&self) -> TestResults {
        self.test_results.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Setup logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&args.log_level))
        .init();

    info!("Starting node {} on port {}", args.node_id, args.port);
    info!("Test mode: {}", args.test_mode);
    info!("Paxos enabled: {}", args.paxos_enabled);

    // Create identity
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    info!("Node {} peer ID: {}", args.node_id, local_peer_id);

    // Initialize node state
    let mut state = NodeState::new(
        local_peer_id,
        args.node_id,
        args.max_failures,
        args.paxos_enabled,
    )?;

    // Parse bootstrap peers
    let bootstrap_peers: Vec<Multiaddr> = if let Some(bootstrap) = args.bootstrap {
        bootstrap
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect()
    } else {
        vec![]
    };

    info!("Bootstrap peers: {:?}", bootstrap_peers);

    // Run test based on mode
    match args.test_mode.as_str() {
        "paxos" => run_paxos_test(&mut state).await?,
        "brb" => run_brb_test(&mut state).await?,
        "gossip" => run_gossip_test(&mut state).await?,
        "sybil" => run_sybil_test(&mut state).await?,
        "full" => run_full_test(&mut state).await?,
        _ => {
            error!("Unknown test mode: {}", args.test_mode);
            return Err("Unknown test mode".into());
        }
    }

    // Print results
    let results = state.get_results();
    println!("\n=== Node {} Test Results ===", args.node_id);
    println!("{}", serde_json::to_string_pretty(&results)?);

    Ok(())
}

async fn run_paxos_test(state: &mut NodeState) -> Result<(), Box<dyn Error>> {
    info!("Running Paxos consensus test");

    if !state.paxos_enabled {
        warn!("Paxos is not enabled - skipping test");
        return Ok(());
    }

    // Test 1: Propose a value
    let value = format!("test_value_node_{}", state.node_id).into_bytes();
    state.test_paxos_propose(value.clone()).await?;

    // Test 2: Simulate receiving prepare and responding
    let other_peer = PeerId::random();
    let proposal = ProposalNumber::new(1, other_peer);

    match state.handle_paxos_prepare(proposal) {
        Ok(promise) => {
            info!("Successfully handled prepare, sent promise");
        }
        Err(e) => {
            warn!("Failed to handle prepare: {}", e);
            state.test_results.errors.push(format!("Paxos prepare error: {}", e));
        }
    }

    // Test 3: Check learned values
    if let Some(paxos) = &state.paxos {
        let learned = paxos.learned_values();
        info!("Learned values: {} entries", learned.len());
    }

    Ok(())
}

async fn run_brb_test(state: &mut NodeState) -> Result<(), Box<dyn Error>> {
    info!("Running BRB test");

    // Test 1: Broadcast a message
    let message = format!("brb_test_node_{}", state.node_id).into_bytes();
    state.test_brb_broadcast(message.clone()).await?;

    // Test 2: Simulate receiving echo
    let message_hash = blake3::hash(&message);
    let other_peer = PeerId::random();

    match state.handle_brb_echo(other_peer, *message_hash.as_bytes(), state.peer_id) {
        Ok(()) => {
            info!("BRB echo handled successfully");
        }
        Err(e) => {
            warn!("Failed to handle echo: {}", e);
            state.test_results.errors.push(format!("BRB echo error: {}", e));
        }
    }

    Ok(())
}

async fn run_gossip_test(state: &mut NodeState) -> Result<(), Box<dyn Error>> {
    info!("Running gossip test");

    // Test 1: Store some data
    for i in 0..5 {
        let key = format!("key_{}_node_{}", i, state.node_id).into_bytes();
        let value = format!("value_{}", i).into_bytes();
        state.put_data(key, value)?;
    }

    info!("Stored {} records", state.data_store.len());

    // Test 2: Get state digest
    let digest = state.get_state_digest();
    info!("State digest: merkle_root={:?}, count={}",
          &digest.merkle_root[..8], digest.record_count);

    // Test 3: Initiate gossip
    state.initiate_gossip()?;

    Ok(())
}

async fn run_sybil_test(state: &mut NodeState) -> Result<(), Box<dyn Error>> {
    info!("Running Sybil resistance test");

    // Test 1: Issue challenge
    let peer = PeerId::random();
    let challenge = state.issue_pow_challenge(peer);
    info!("Issued challenge: {} bytes", challenge.len());

    // Test 2: Check if peer is verified
    let is_verified = state.sync_manager.is_peer_verified(&peer);
    info!("Peer verified: {}", is_verified);

    Ok(())
}

async fn run_full_test(state: &mut NodeState) -> Result<(), Box<dyn Error>> {
    info!("Running full integration test");

    // Run all tests
    run_paxos_test(state).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    run_brb_test(state).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    run_gossip_test(state).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    run_sybil_test(state).await?;

    // Test history with Paxos
    if state.paxos_enabled {
        info!("Testing history retrieval with Paxos enabled");
        let test_key = b"test_key";
        let history = state.get_history(test_key);
        info!("History entries: {}", history.len());
    }

    Ok(())
}
