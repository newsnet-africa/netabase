//! Network Topology Tests - P2P communication tests with various network configurations
//!
//! These tests spawn sender and receiver nodes as subprocesses and verify
//! that messages are properly transmitted across the network.
//!
//! Since there's no consistency guarantee, tests pass if nodes can read
//! messages sent since they joined the network.
//!
//! Run with:
//!   cargo test --test network_topology_tests -- --test-threads=1

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Events from sender node
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum SenderEvent {
    #[serde(rename = "status")]
    Status { status: String },
    #[serde(rename = "peer_connected")]
    PeerConnected { peer_id: String },
    #[serde(rename = "peer_count")]
    PeerCount { count: usize },
    #[serde(rename = "message_sent")]
    MessageSent { id: String, content: String, seq: u64 },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "complete")]
    Complete { messages_sent: u64 },
}

/// Events from receiver node
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ReceiverEvent {
    #[serde(rename = "status")]
    Status { status: String },
    #[serde(rename = "peer_connected")]
    PeerConnected { peer_id: String },
    #[serde(rename = "peer_count")]
    PeerCount { count: usize },
    #[serde(rename = "message_received")]
    MessageReceived {
        id: String,
        sender: String,
        content: String,
        seq: u64,
    },
    #[serde(rename = "poll_result")]
    PollResult { messages_found: usize, total_received: usize },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "complete")]
    Complete {
        messages_received: usize,
        expected: usize,
        success: bool,
    },
}

/// Results from a test run
#[derive(Debug, Default)]
struct TestResults {
    sender_messages_sent: u64,
    receiver_messages_received: usize,
    sender_peer_count: usize,
    receiver_peer_count: usize,
    sender_errors: Vec<String>,
    receiver_errors: Vec<String>,
}

/// Build the test binaries
fn build_test_binaries() -> bool {
    eprintln!("Building test binaries...");

    let output = Command::new("cargo")
        .args([
            "build",
            "--test",
            "network_sender_node",
            "--test",
            "network_receiver_node",
            "--features",
            "native",
        ])
        .output()
        .expect("Failed to build test binaries");

    if !output.status.success() {
        eprintln!(
            "Build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return false;
    }

    eprintln!("Test binaries built successfully");
    true
}

/// Spawn a sender node
fn spawn_sender(
    name: &str,
    message_count: u64,
    wait_for_peers_secs: u64,
    message_prefix: &str,
) -> Child {
    let binary = format!(
        "./target/debug/deps/network_sender_node-{}",
        find_binary_hash("network_sender_node")
    );

    // Fallback to test binary path
    let binary = if std::path::Path::new(&binary).exists() {
        binary
    } else {
        // Try to find the binary
        find_test_binary("network_sender_node")
    };

    eprintln!("Starting sender {} with binary: {}", name, binary);

    Command::new(&binary)
        .args([
            name,
            &message_count.to_string(),
            &wait_for_peers_secs.to_string(),
            message_prefix,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn sender node")
}

/// Spawn a receiver node
fn spawn_receiver(
    name: &str,
    expected_messages: usize,
    timeout_secs: u64,
    message_prefix: &str,
) -> Child {
    let binary = find_test_binary("network_receiver_node");

    eprintln!("Starting receiver {} with binary: {}", name, binary);

    Command::new(&binary)
        .args([
            name,
            &expected_messages.to_string(),
            &timeout_secs.to_string(),
            message_prefix,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn receiver node")
}

/// Find a test binary by name
fn find_test_binary(name: &str) -> String {
    // Look in deps directory
    let deps_dir = std::path::Path::new("./target/debug/deps");
    if deps_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(deps_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name() {
                    let file_name = file_name.to_string_lossy();
                    if file_name.starts_with(name)
                        && !file_name.ends_with(".d")
                        && !file_name.ends_with(".rmeta")
                        && !file_name.ends_with(".rlib")
                    {
                        // Check if it's executable
                        if path.is_file() {
                            return path.to_string_lossy().to_string();
                        }
                    }
                }
            }
        }
    }

    // Fallback
    format!("./target/debug/deps/{}", name)
}

/// Find binary hash (not always needed)
fn find_binary_hash(name: &str) -> String {
    let deps_dir = std::path::Path::new("./target/debug/deps");
    if let Ok(entries) = std::fs::read_dir(deps_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if file_name.starts_with(name)
                && !file_name.ends_with(".d")
                && !file_name.ends_with(".rmeta")
                && !file_name.ends_with(".rlib")
            {
                // Extract hash from filename like "network_sender_node-abc123"
                if let Some(hash) = file_name.strip_prefix(&format!("{}-", name)) {
                    return hash.to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

/// Collect events from a sender process
fn collect_sender_events(child: &mut Child) -> Vec<SenderEvent> {
    let mut events = Vec::new();

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if let Ok(event) = serde_json::from_str::<SenderEvent>(&line) {
                events.push(event);
            }
        }
    }

    events
}

/// Collect events from a receiver process
fn collect_receiver_events(child: &mut Child) -> Vec<ReceiverEvent> {
    let mut events = Vec::new();

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if let Ok(event) = serde_json::from_str::<ReceiverEvent>(&line) {
                events.push(event);
            }
        }
    }

    events
}

/// Run a basic two-node test: one sender, one receiver
fn run_two_node_test(
    message_count: u64,
    timeout_secs: u64,
    test_name: &str,
) -> TestResults {
    let message_prefix = format!("test_{}", test_name);

    // Start receiver first so it's ready for connections
    let mut receiver = spawn_receiver(
        &format!("{}_receiver", test_name),
        message_count as usize,
        timeout_secs,
        &message_prefix,
    );

    // Give receiver time to start
    std::thread::sleep(Duration::from_secs(2));

    // Start sender
    let mut sender = spawn_sender(
        &format!("{}_sender", test_name),
        message_count,
        15, // wait for peers
        &message_prefix,
    );

    // Wait for both to complete
    let sender_status = sender.wait().expect("Failed to wait for sender");
    let receiver_status = receiver.wait().expect("Failed to wait for receiver");

    eprintln!(
        "Sender exit status: {:?}, Receiver exit status: {:?}",
        sender_status, receiver_status
    );

    // Collect events
    let sender_events = collect_sender_events(&mut sender);
    let receiver_events = collect_receiver_events(&mut receiver);

    // Build results
    let mut results = TestResults::default();

    for event in sender_events {
        match event {
            SenderEvent::Complete { messages_sent } => {
                results.sender_messages_sent = messages_sent;
            }
            SenderEvent::PeerCount { count } => {
                results.sender_peer_count = count;
            }
            SenderEvent::Error { message } => {
                results.sender_errors.push(message);
            }
            _ => {}
        }
    }

    for event in receiver_events {
        match event {
            ReceiverEvent::Complete {
                messages_received, ..
            } => {
                results.receiver_messages_received = messages_received;
            }
            ReceiverEvent::PeerCount { count } => {
                results.receiver_peer_count = count;
            }
            ReceiverEvent::Error { message } => {
                results.receiver_errors.push(message);
            }
            _ => {}
        }
    }

    results
}

/// Run a multi-sender, single-receiver test
fn run_multi_sender_test(
    sender_count: usize,
    messages_per_sender: u64,
    timeout_secs: u64,
    test_name: &str,
) -> TestResults {
    let message_prefix = format!("test_{}", test_name);
    let expected_total = sender_count * messages_per_sender as usize;

    // Start receiver first
    let mut receiver = spawn_receiver(
        &format!("{}_receiver", test_name),
        expected_total,
        timeout_secs,
        &message_prefix,
    );

    // Give receiver time to start
    std::thread::sleep(Duration::from_secs(2));

    // Start all senders
    let mut senders: Vec<Child> = Vec::new();
    for i in 0..sender_count {
        let sender = spawn_sender(
            &format!("{}_sender_{}", test_name, i),
            messages_per_sender,
            15,
            &message_prefix,
        );
        senders.push(sender);

        // Stagger sender starts slightly
        std::thread::sleep(Duration::from_millis(500));
    }

    // Wait for all senders to complete
    let mut total_sent = 0u64;
    for mut sender in senders {
        let _ = sender.wait();
        let events = collect_sender_events(&mut sender);
        for event in events {
            if let SenderEvent::Complete { messages_sent } = event {
                total_sent += messages_sent;
            }
        }
    }

    // Wait for receiver
    let _ = receiver.wait();
    let receiver_events = collect_receiver_events(&mut receiver);

    let mut results = TestResults::default();
    results.sender_messages_sent = total_sent;

    for event in receiver_events {
        match event {
            ReceiverEvent::Complete {
                messages_received, ..
            } => {
                results.receiver_messages_received = messages_received;
            }
            ReceiverEvent::PeerCount { count } => {
                results.receiver_peer_count = count;
            }
            ReceiverEvent::Error { message } => {
                results.receiver_errors.push(message);
            }
            _ => {}
        }
    }

    results
}

// ============================================================================
// Tests
// ============================================================================

/// Test: Simple two-node communication
#[test]
#[ignore] // Run with --ignored flag
fn test_two_node_basic() {
    if !build_test_binaries() {
        panic!("Failed to build test binaries");
    }

    eprintln!("\n=== Test: Two Node Basic Communication ===\n");

    let results = run_two_node_test(5, 60, "two_node_basic");

    eprintln!("Results:");
    eprintln!("  Sender messages sent: {}", results.sender_messages_sent);
    eprintln!(
        "  Receiver messages received: {}",
        results.receiver_messages_received
    );
    eprintln!("  Sender peer count: {}", results.sender_peer_count);
    eprintln!("  Receiver peer count: {}", results.receiver_peer_count);

    if !results.sender_errors.is_empty() {
        eprintln!("  Sender errors: {:?}", results.sender_errors);
    }
    if !results.receiver_errors.is_empty() {
        eprintln!("  Receiver errors: {:?}", results.receiver_errors);
    }

    // Success criteria: receiver got at least some messages
    // (no consistency guarantee, but should get messages sent since joining)
    assert!(
        results.receiver_messages_received > 0,
        "Receiver should have received at least some messages"
    );
}

/// Test: Two nodes with more messages
#[test]
#[ignore]
fn test_two_node_many_messages() {
    if !build_test_binaries() {
        panic!("Failed to build test binaries");
    }

    eprintln!("\n=== Test: Two Node Many Messages ===\n");

    let results = run_two_node_test(20, 90, "two_node_many");

    eprintln!("Results:");
    eprintln!("  Sender messages sent: {}", results.sender_messages_sent);
    eprintln!(
        "  Receiver messages received: {}",
        results.receiver_messages_received
    );

    // Should receive at least 50% of messages
    let min_expected = (results.sender_messages_sent as f64 * 0.5) as usize;
    assert!(
        results.receiver_messages_received >= min_expected,
        "Receiver should have received at least {}% of messages (got {}/{})",
        50,
        results.receiver_messages_received,
        results.sender_messages_sent
    );
}

/// Test: Multiple senders, one receiver
#[test]
#[ignore]
fn test_multi_sender_single_receiver() {
    if !build_test_binaries() {
        panic!("Failed to build test binaries");
    }

    eprintln!("\n=== Test: Multi-Sender Single Receiver ===\n");

    let sender_count = 3;
    let messages_per_sender = 5;

    let results = run_multi_sender_test(
        sender_count,
        messages_per_sender,
        120, // longer timeout for multi-node
        "multi_sender",
    );

    eprintln!("Results:");
    eprintln!("  Total messages sent: {}", results.sender_messages_sent);
    eprintln!(
        "  Receiver messages received: {}",
        results.receiver_messages_received
    );

    // Should receive at least some messages from the network
    assert!(
        results.receiver_messages_received > 0,
        "Receiver should have received at least some messages from {} senders",
        sender_count
    );
}

/// Test: Verify message content integrity
#[test]
#[ignore]
fn test_message_content_integrity() {
    if !build_test_binaries() {
        panic!("Failed to build test binaries");
    }

    eprintln!("\n=== Test: Message Content Integrity ===\n");

    // Use a unique prefix to identify messages
    let message_prefix = format!("integrity_{}", std::process::id());

    // Start receiver
    let mut receiver = spawn_receiver("integrity_receiver", 3, 60, &message_prefix);

    std::thread::sleep(Duration::from_secs(2));

    // Start sender
    let mut sender = spawn_sender("integrity_sender", 3, 15, &message_prefix);

    // Wait for completion
    let _ = sender.wait();
    let _ = receiver.wait();

    // Check receiver events for message content
    let receiver_events = collect_receiver_events(&mut receiver);
    let mut received_contents: Vec<String> = Vec::new();

    for event in receiver_events {
        if let ReceiverEvent::MessageReceived { content, .. } = event {
            received_contents.push(content);
        }
    }

    eprintln!("Received message contents: {:?}", received_contents);

    // Verify at least one message was received with correct prefix
    let valid_messages: Vec<_> = received_contents
        .iter()
        .filter(|c| c.starts_with(&message_prefix))
        .collect();

    assert!(
        !valid_messages.is_empty(),
        "Should have received at least one message with prefix '{}'",
        message_prefix
    );
}

/// Quick smoke test that can run without --ignored
#[test]
fn test_binaries_build() {
    assert!(
        build_test_binaries(),
        "Test binaries should build successfully"
    );
}

/// Test that cleans up test data directories
#[test]
fn test_cleanup_test_data() {
    let test_dir = "./test_network_data";
    if std::path::Path::new(test_dir).exists() {
        let _ = std::fs::remove_dir_all(test_dir);
    }
}
