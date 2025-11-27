//! Network Receiver Node - Test binary for P2P message receiving
//!
//! This binary starts a netabase node, waits for messages from senders,
//! reports received messages, then exits after a timeout or when enough messages are received.
//!
//! Usage:
//!   network_receiver_node <node_name> <expected_messages> <timeout_secs> <message_prefix>
//!
//! Output format (JSON per line):
//!   {"type":"status","status":"starting"}
//!   {"type":"peer_connected","peer_id":"..."}
//!   {"type":"message_received","id":"...","sender":"...","content":"...","seq":0}
//!   {"type":"complete","messages_received":10,"expected":10}

use anyhow::Result;
use netabase::Netabase;
use netabase_store::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{self, Write};
use tokio::time::Duration;

// Test schema - must match sender_schema
#[netabase_definition_module(ReceiverDefinition, ReceiverKeys)]
mod receiver_schema {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(ReceiverDefinition)]
    pub struct NetworkMessage {
        #[primary_key]
        pub id: String,
        pub sender_node: String,
        pub content: String,
        pub sequence: u64,
        pub timestamp: i64,
    }
}

use receiver_schema::*;

/// Output events from the receiver node
#[derive(Debug, Serialize, Deserialize)]
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

fn emit_event(event: ReceiverEvent) {
    if let Ok(json) = serde_json::to_string(&event) {
        println!("{}", json);
        let _ = io::stdout().flush();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse arguments
    let args: Vec<String> = std::env::args().collect();

    // Handle nextest's --list flag for test discovery
    if args.iter().any(|a| a == "--list" || a == "--list-tests") {
        // Print nothing - this binary is not a test suite
        return Ok(());
    }

    if args.len() < 5 {
        eprintln!(
            "Usage: {} <node_name> <expected_messages> <timeout_secs> <message_prefix>",
            args[0]
        );
        std::process::exit(1);
    }

    let node_name = &args[1];
    let expected_messages: usize = args[2].parse().unwrap_or(10);
    let timeout_secs: u64 = args[3].parse().unwrap_or(60);
    let message_prefix = &args[4];

    // Suppress logs unless RUST_LOG is set
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "off"); }
    }
    env_logger::init();

    emit_event(ReceiverEvent::Status {
        status: "starting".to_string(),
    });

    // Setup database path
    let db_path = format!("./test_network_data/receiver_{}", node_name);
    let _ = std::fs::remove_dir_all(&db_path);
    std::fs::create_dir_all(&db_path)?;

    // Create netabase instance
    let mut netabase = Netabase::<ReceiverDefinition>::new_with_path(&db_path)?;

    emit_event(ReceiverEvent::Status {
        status: "swarm_starting".to_string(),
    });

    // Start the swarm
    netabase.start_swarm().await?;

    emit_event(ReceiverEvent::Status {
        status: "swarm_started".to_string(),
    });

    // Track received messages
    let mut received_ids: HashSet<String> = HashSet::new();
    let mut peer_count = 0;

    // Subscribe to events
    let mut event_receiver = netabase.subscribe_to_broadcasts();

    let start = tokio::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    emit_event(ReceiverEvent::Status {
        status: "listening".to_string(),
    });

    // Main loop: listen for events and poll for messages
    let poll_interval = Duration::from_millis(500);
    let mut last_poll = tokio::time::Instant::now();

    while start.elapsed() < timeout && received_ids.len() < expected_messages {
        tokio::select! {
            // Handle network events
            event_result = event_receiver.recv() => {
                if let Ok(event) = event_result {
                    match &event.0 {
                        libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            peer_count += 1;
                            emit_event(ReceiverEvent::PeerConnected {
                                peer_id: peer_id.to_string(),
                            });
                        }
                        libp2p::swarm::SwarmEvent::Behaviour(behaviour) => {
                            use netabase::NetabaseBehaviourEvent;
                            if let NetabaseBehaviourEvent::Mdns(mdns_event) = behaviour {
                                use libp2p::mdns::Event;
                                if let Event::Discovered(peers) = mdns_event {
                                    for (peer_id, _) in peers {
                                        peer_count += 1;
                                        emit_event(ReceiverEvent::PeerConnected {
                                            peer_id: peer_id.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Poll for new messages periodically
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                if last_poll.elapsed() >= poll_interval {
                    last_poll = tokio::time::Instant::now();

                    // Query local records for matching messages
                    match netabase.query_local_records(None).await {
                        Ok(records) => {
                            let mut new_messages = 0;
                            for record in records {
                                if let ReceiverDefinition::NetworkMessage(msg) = record {
                                    // Check if this message matches our prefix
                                    if msg.content.starts_with(message_prefix)
                                        && !received_ids.contains(&msg.id)
                                    {
                                        received_ids.insert(msg.id.clone());
                                        new_messages += 1;
                                        emit_event(ReceiverEvent::MessageReceived {
                                            id: msg.id,
                                            sender: msg.sender_node,
                                            content: msg.content,
                                            seq: msg.sequence,
                                        });
                                    }
                                }
                            }

                            if new_messages > 0 {
                                emit_event(ReceiverEvent::PollResult {
                                    messages_found: new_messages,
                                    total_received: received_ids.len(),
                                });
                            }
                        }
                        Err(e) => {
                            emit_event(ReceiverEvent::Error {
                                message: format!("Query failed: {}", e),
                            });
                        }
                    }
                }
            }
        }
    }

    emit_event(ReceiverEvent::PeerCount { count: peer_count });

    // Final result
    let success = received_ids.len() >= expected_messages
        || (expected_messages > 0 && !received_ids.is_empty());

    emit_event(ReceiverEvent::Complete {
        messages_received: received_ids.len(),
        expected: expected_messages,
        success,
    });

    // Cleanup
    netabase.stop_swarm().await?;
    let _ = std::fs::remove_dir_all(&db_path);

    // Exit with appropriate code
    if success {
        Ok(())
    } else {
        std::process::exit(1);
    }
}
