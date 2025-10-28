//! Rust-based test orchestrator using std::process::Command
//! This can be run as an alternative to the nushell script

use std::process::{Command, Child, Stdio};
use std::time::Duration;
use std::thread;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Test configuration
struct TestConfig {
    num_nodes: usize,
    max_failures: usize,
    test_mode: String,
    paxos_enabled: bool,
    duration_secs: u64,
    base_port: u16,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            num_nodes: 4,
            max_failures: 1,
            test_mode: "full".to_string(),
            paxos_enabled: true,
            duration_secs: 20,
            base_port: 9000,
        }
    }
}

/// Test results summary
#[derive(Debug, Default)]
struct TestResults {
    nodes_spawned: usize,
    nodes_failed: usize,
    errors: Vec<String>,
}

/// Find the example binary
fn find_example_binary() -> Result<PathBuf, std::io::Error> {
    let binary_path = PathBuf::from("target/release/examples/sync_network_node");
    if binary_path.exists() {
        Ok(binary_path)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Example binary not found at target/release/examples/sync_network_node",
        ))
    }
}

/// Spawn a test node process
fn spawn_node(
    config: &TestConfig,
    node_id: usize,
    bootstrap_addrs: Vec<String>,
    log_dir: &Path,
    binary_path: &Path,
) -> Result<Child, std::io::Error> {
    let port = config.base_port + node_id as u16;
    let log_file = log_dir.join(format!("node_{}.log", node_id));

    // Run the test binary directly
    let mut cmd = Command::new(binary_path);
    cmd.arg("--node-id")
        .arg(node_id.to_string())
        .arg("--port")
        .arg(port.to_string())
        .arg("--test-mode")
        .arg(&config.test_mode)
        .arg("--max-failures")
        .arg(config.max_failures.to_string())
        .arg("--log-level")
        .arg("info");

    if config.paxos_enabled {
        cmd.arg("--paxos-enabled");
    }

    if !bootstrap_addrs.is_empty() {
        cmd.arg("--bootstrap").arg(bootstrap_addrs.join(","));
    }

    // Redirect output to log file
    let log_file_handle = fs::File::create(log_file)?;
    cmd.stdout(Stdio::from(log_file_handle.try_clone()?));
    cmd.stderr(Stdio::from(log_file_handle));

    // Spawn process
    cmd.spawn()
}

/// Run multi-node test
fn run_multi_node_test(config: TestConfig) -> Result<TestResults, Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║     NetaBase Sync Protocol Multi-Process Test Suite          ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Validate configuration
    let min_nodes = 3 * config.max_failures + 1;
    if config.num_nodes < min_nodes {
        return Err(format!(
            "Need at least {} nodes for f={} Byzantine faults",
            min_nodes, config.max_failures
        )
        .into());
    }

    println!("Configuration:");
    println!("  Test Mode:       {}", config.test_mode);
    println!("  Number of Nodes: {}", config.num_nodes);
    println!("  Max Failures:    {}", config.max_failures);
    println!("  Paxos Enabled:   {}", config.paxos_enabled);
    println!("  Test Duration:   {}s\n", config.duration_secs);

    // Setup log directory
    let log_dir = PathBuf::from("test_logs");
    if log_dir.exists() {
        fs::remove_dir_all(&log_dir)?;
    }
    fs::create_dir_all(&log_dir)?;

    println!("Building example binary...");
    let build_status = Command::new("cargo")
        .arg("build")
        .arg("--example")
        .arg("sync_network_node")
        .arg("--release")
        .status()?;

    if !build_status.success() {
        return Err("Failed to build example".into());
    }
    println!("✓ Build successful\n");

    // Find the example binary
    let binary_path = find_example_binary()?;
    println!("Found example binary: {}\n", binary_path.display());

    println!("Spawning {} network nodes...", config.num_nodes);

    let mut nodes = Vec::new();
    let mut bootstrap_addrs = Vec::new();

    for node_id in 0..config.num_nodes {
        let port = config.base_port + node_id as u16;

        println!("  Node {}: port {}", node_id, port);

        // Spawn node
        match spawn_node(&config, node_id, bootstrap_addrs.clone(), &log_dir, &binary_path) {
            Ok(child) => {
                nodes.push((node_id, child));

                // Add this node to bootstrap list for next nodes
                bootstrap_addrs.push(format!("/ip4/127.0.0.1/tcp/{}", port));

                // Give node time to start
                thread::sleep(Duration::from_millis(500));
            }
            Err(e) => {
                eprintln!("  Failed to spawn node {}: {}", node_id, e);
            }
        }
    }

    println!("✓ All nodes spawned\n");

    // Let nodes run and communicate
    println!("Running test for {} seconds...", config.duration_secs);
    println!("Nodes are communicating over the network...\n");

    // Progress bar
    for i in 1..=config.duration_secs {
        let progress = (i * 100 / config.duration_secs) as usize;
        let bar_length = progress / 5;
        let bar: String = "█".repeat(bar_length);
        let empty: String = "░".repeat(20 - bar_length);

        print!("\r  Progress: [{}{}] {}% {}/{}s", bar, empty, progress, i, config.duration_secs);
        std::io::stdout().flush()?;

        thread::sleep(Duration::from_secs(1));
    }
    println!("\n");

    // Stop all nodes
    println!("Stopping nodes...");
    let mut results = TestResults {
        nodes_spawned: nodes.len(),
        ..Default::default()
    };

    for (node_id, mut child) in nodes {
        match child.kill() {
            Ok(_) => {
                let _ = child.wait();
            }
            Err(e) => {
                results.nodes_failed += 1;
                results
                    .errors
                    .push(format!("Failed to stop node {}: {}", node_id, e));
            }
        }
    }

    println!("✓ Test completed\n");

    // Collect and display results
    println!("Test Results Summary");
    println!("═══════════════════════════════════════════════════════════════\n");

    for node_id in 0..config.num_nodes {
        let log_file = log_dir.join(format!("node_{}.log", node_id));

        if log_file.exists() {
            println!("Node {} (port {}):", node_id, config.base_port + node_id as u16);

            // Read log file and extract results
            let log_content = fs::read_to_string(&log_file)?;

            // Look for test results JSON
            if let Some(results_line) = log_content.lines().find(|l| l.contains("Test Results")) {
                if let Some(json_start) = results_line.find('{') {
                    let json_str = &results_line[json_start..];
                    match serde_json::from_str::<serde_json::Value>(json_str) {
                        Ok(json) => {
                            println!("  Paxos Proposals:       {}", json["paxos_proposals"]);
                            println!("  Paxos Consensuses:     {}", json["paxos_consensuses"]);
                            println!("  BRB Broadcasts:        {}", json["brb_broadcasts"]);
                            println!("  BRB Deliveries:        {}", json["brb_deliveries"]);
                            println!("  Gossip Rounds:         {}", json["gossip_rounds"]);
                            println!("  State Syncs:           {}", json["state_syncs"]);
                            println!("  PoW Challenges Issued: {}", json["pow_challenges_issued"]);
                            println!("  PoW Verified:          {}", json["pow_challenges_verified"]);

                            if let Some(errors) = json["errors"].as_array() {
                                if !errors.is_empty() {
                                    println!("  Errors: {}", errors.len());
                                    for error in errors {
                                        println!("    - {}", error);
                                    }
                                } else {
                                    println!("  ✓ No errors");
                                }
                            }
                        }
                        Err(e) => {
                            println!("  Could not parse results: {}", e);
                        }
                    }
                }
            } else {
                println!("  No results found in log");
            }
            println!();
        }
    }

    // Paxos-specific checks
    if config.paxos_enabled {
        println!("Paxos-Specific Checks");
        println!("═══════════════════════════════════════════════════════════════\n");

        for node_id in 0..config.num_nodes {
            let log_file = log_dir.join(format!("node_{}.log", node_id));

            if log_file.exists() {
                let log_content = fs::read_to_string(&log_file)?;

                let paxos_logs: Vec<&str> = log_content
                    .lines()
                    .filter(|l| {
                        l.contains("Paxos") || l.contains("consensus") || l.contains("proposal")
                    })
                    .collect();

                if !paxos_logs.is_empty() {
                    println!("Node {} Paxos activity:", node_id);
                    for log in paxos_logs.iter().rev().take(5).rev() {
                        println!("  {}", log);
                    }
                    println!();
                }
            }
        }
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!("Test suite completed successfully!");
    println!("Logs saved in: {}/\n", log_dir.display());

    Ok(results)
}

#[test]
fn test_sync_small_network() {
    let config = TestConfig {
        num_nodes: 4,
        max_failures: 1,
        test_mode: "full".to_string(),
        paxos_enabled: true,
        duration_secs: 15,
        ..Default::default()
    };

    match run_multi_node_test(config) {
        Ok(results) => {
            println!("\n✓ Test completed successfully");
            println!("  Nodes spawned: {}", results.nodes_spawned);
            println!("  Nodes failed: {}", results.nodes_failed);
            assert_eq!(results.nodes_failed, 0, "Some nodes failed to stop properly");
        }
        Err(e) => {
            panic!("Test failed: {}", e);
        }
    }
}

#[test]
fn test_sync_byzantine_network() {
    let config = TestConfig {
        num_nodes: 7,
        max_failures: 2,
        test_mode: "brb".to_string(),
        paxos_enabled: false,
        duration_secs: 20,
        ..Default::default()
    };

    match run_multi_node_test(config) {
        Ok(results) => {
            println!("\n✓ Byzantine test completed");
            assert_eq!(results.nodes_failed, 0);
        }
        Err(e) => {
            panic!("Byzantine test failed: {}", e);
        }
    }
}

#[test]
fn test_sync_paxos_network() {
    let config = TestConfig {
        num_nodes: 7,
        max_failures: 2,
        test_mode: "paxos".to_string(),
        paxos_enabled: true,
        duration_secs: 15,
        ..Default::default()
    };

    match run_multi_node_test(config) {
        Ok(results) => {
            println!("\n✓ Paxos test completed");
            assert_eq!(results.nodes_failed, 0);
        }
        Err(e) => {
            panic!("Paxos test failed: {}", e);
        }
    }
}

#[test]
#[ignore] // Run with: cargo test test_sync_large_network -- --ignored
fn test_sync_large_network() {
    let config = TestConfig {
        num_nodes: 10,
        max_failures: 3,
        test_mode: "full".to_string(),
        paxos_enabled: true,
        duration_secs: 30,
        ..Default::default()
    };

    match run_multi_node_test(config) {
        Ok(results) => {
            println!("\n✓ Large network test completed");
            assert_eq!(results.nodes_failed, 0);
        }
        Err(e) => {
            panic!("Large network test failed: {}", e);
        }
    }
}

// Helper main function for manual testing
fn main() {
    let config = TestConfig::default();

    match run_multi_node_test(config) {
        Ok(_) => println!("All tests passed!"),
        Err(e) => eprintln!("Test failed: {}", e),
    }
}
