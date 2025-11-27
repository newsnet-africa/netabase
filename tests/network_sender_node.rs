//! Network Sender Node - Test binary for P2P message sending
//!
//! This binary starts a netabase node, waits for peer connections,
//! sends a configurable number of messages, then exits.
//!
//! Usage:
//!   network_sender_node <node_name> <message_count> <wait_for_peers_secs> <message_prefix>
//!
//! Output format (JSON per line):
//!   {"type":"status","status":"starting"}
//!   {"type":"peer_connected","peer_id":"..."}
//!   {"type":"message_sent","id":"...","content":"...","seq":0}
//!   {"type":"complete","messages_sent":10}

use anyhow::Result;
use chrono::Utc;
use netabase::Netabase;
use netabase_store::*;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use tokio::time::Duration;

// Test schema
#[netabase_definition_module(SenderDefinition, SenderKeys)]
mod sender_schema {
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
    #[netabase(SenderDefinition)]
    pub struct NetworkMessage {
        #[primary_key]
        pub id: String,
        pub sender_node: String,
        pub content: String,
        pub sequence: u64,
        pub timestamp: i64,
    }
}

use sender_schema::*;

/// Output events from the sender node
#[derive(Debug, Serialize, Deserialize)]
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

fn emit_event(event: SenderEvent) {
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
            "Usage: {} <node_name> <message_count> <wait_for_peers_secs> <message_prefix>",
            args[0]
        );
        std::process::exit(1);
    }

    let node_name = &args[1];
    let message_count: u64 = args[2].parse().unwrap_or(10);
    let wait_for_peers_secs: u64 = args[3].parse().unwrap_or(30);
    let message_prefix = &args[4];

    // Suppress logs unless RUST_LOG is set
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "off"); }
    }
    env_logger::init();

    emit_event(SenderEvent::Status {
        status: "starting".to_string(),
    });

    // Setup database path
    let db_path = format!("./test_network_data/sender_{}", node_name);
    let _ = std::fs::remove_dir_all(&db_path);
    std::fs::create_dir_all(&db_path)?;

    // Create netabase instance
    let mut netabase = Netabase::<SenderDefinition>::new_with_path(&db_path)?;

    emit_event(SenderEvent::Status {
        status: "swarm_starting".to_string(),
    });

    // Start the swarm
    netabase.start_swarm().await?;

    emit_event(SenderEvent::Status {
        status: "swarm_started".to_string(),
    });

    // Wait for peer connections
    let mut event_receiver = netabase.subscribe_to_broadcasts();
    let mut peer_count = 0;
    let start = tokio::time::Instant::now();
    let timeout = Duration::from_secs(wait_for_peers_secs);

    emit_event(SenderEvent::Status {
        status: "waiting_for_peers".to_string(),
    });

    // Wait until at least one peer is discovered or timeout
    while peer_count == 0 && start.elapsed() < timeout {
        tokio::select! {
            event_result = event_receiver.recv() => {
                if let Ok(event) = event_result {
                    match &event.0 {
                        libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            peer_count += 1;
                            emit_event(SenderEvent::PeerConnected {
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
                                        emit_event(SenderEvent::PeerConnected {
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
            _ = tokio::time::sleep(Duration::from_millis(100)) => {}
        }
    }

    emit_event(SenderEvent::PeerCount { count: peer_count });

    // Give peers a moment to fully establish
    tokio::time::sleep(Duration::from_millis(500)).await;

    emit_event(SenderEvent::Status {
        status: "sending_messages".to_string(),
    });

    // Send messages
    let mut messages_sent = 0;
    for seq in 0..message_count {
        let msg_id = format!("{}_{}_msg_{}", node_name, message_prefix, seq);
        let content = format!("{}_{}", message_prefix, seq);

        let message = NetworkMessage {
            id: msg_id.clone(),
            sender_node: node_name.clone(),
            content: content.clone(),
            sequence: seq,
            timestamp: Utc::now().timestamp(),
        };

        match netabase.put_record(message).await {
            Ok(_) => {
                emit_event(SenderEvent::MessageSent {
                    id: msg_id,
                    content,
                    seq,
                });
                messages_sent += 1;
            }
            Err(e) => {
                emit_event(SenderEvent::Error {
                    message: format!("Failed to send message {}: {}", seq, e),
                });
            }
        }

        // Small delay between messages to avoid overwhelming the network
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Give some time for messages to propagate
    tokio::time::sleep(Duration::from_secs(2)).await;

    emit_event(SenderEvent::Complete { messages_sent });

    // Cleanup
    netabase.stop_swarm().await?;
    let _ = std::fs::remove_dir_all(&db_path);

    Ok(())
}
