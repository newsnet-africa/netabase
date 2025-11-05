//! Test helper binary for chat example integration tests
//!
//! This binary can be spawned as a subprocess and controlled via JSON commands
//! to test the actual chat application behavior in real-world scenarios.

use anyhow::Result;
use chrono::Utc;
use netabase::Netabase;
use netabase_store::*;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use tokio::time::Duration;

// Chat schema
#[netabase_definition_module(ChatDefinition, ChatKeys)]
mod chat {
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
    #[netabase(ChatDefinition)]
    pub struct Message {
        #[primary_key]
        pub id: String,
        pub sender: String,
        pub content: String,
        pub timestamp: i64,
    }
}

use chat::*;

/// Commands that can be sent to the test chat node
#[derive(Debug, Serialize, Deserialize)]
enum Command {
    /// Start the swarm and wait for peer discovery
    StartSwarm { username: String, wait_for_peers_secs: u64 },

    /// Send a message (to DHT)
    SendMessage { content: String },

    /// Store a message locally (for testing)
    StoreMessageLocally { sender: String, content: String },

    /// Get message history
    GetHistory,

    /// Wait for specific number of peers
    WaitForPeers { count: usize, timeout_secs: u64 },

    /// Wait for a message to appear in history
    WaitForMessage { sender: String, content: String, timeout_secs: u64 },

    /// Get peer count
    GetPeerCount,

    /// Shutdown
    Shutdown,
}

/// Responses sent back via stdout
#[derive(Debug, Serialize, Deserialize)]
enum Response {
    Ok,
    Error(String),
    SwarmStarted { peer_count: usize },
    MessageSent { id: String },
    History { messages: Vec<MessageInfo> },
    MessageReceived { sender: String, content: String },
    PeerCount { count: usize },
    PeersDiscovered { count: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageInfo {
    id: String,
    sender: String,
    content: String,
    timestamp: i64,
}

fn send_response(resp: Response) -> Result<()> {
    let json = serde_json::to_string(&resp)?;
    println!("{}", json);
    io::stdout().flush()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Suppress netabase logs in test mode
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "off");
        }
    }
    env_logger::init();

    eprintln!("[CHAT_TEST_NODE] Starting");

    let stdin = io::stdin();
    let reader = stdin.lock();

    let mut netabase: Option<Netabase<ChatDefinition>> = None;
    let mut username = String::new();

    for line in reader.lines() {
        let line = line?;
        eprintln!("[CHAT_TEST_NODE] Received command: {}", line);

        let command: Command = match serde_json::from_str(&line) {
            Ok(cmd) => cmd,
            Err(e) => {
                send_response(Response::Error(format!("Invalid command: {}", e)))?;
                continue;
            }
        };

        match handle_command(&mut netabase, &mut username, command).await {
            Ok(()) => {}
            Err(e) => {
                send_response(Response::Error(format!("Command failed: {}", e)))?;
            }
        }
    }

    // Cleanup
    if let Some(mut nb) = netabase {
        nb.stop_swarm().await?;
    }

    eprintln!("[CHAT_TEST_NODE] Shutdown complete");
    Ok(())
}

async fn handle_command(
    netabase: &mut Option<Netabase<ChatDefinition>>,
    username: &mut String,
    command: Command,
) -> Result<()> {
    match command {
        Command::StartSwarm { username: user, wait_for_peers_secs } => {
            *username = user.clone();
            let db_path = format!("./test_chat_data/{}", user);

            // Clean up old data
            let _ = std::fs::remove_dir_all(&db_path);
            std::fs::create_dir_all(&db_path)?;

            eprintln!("[CHAT_TEST_NODE] Creating netabase at {}", db_path);
            let mut nb = Netabase::<ChatDefinition>::new_with_path(&db_path)?;

            eprintln!("[CHAT_TEST_NODE] Starting swarm");
            nb.start_swarm().await?;

            // Wait for peer discovery
            let mut event_receiver = nb.subscribe_to_broadcasts();
            let mut peer_count = 0;
            let start = tokio::time::Instant::now();
            let timeout = Duration::from_secs(wait_for_peers_secs);

            eprintln!("[CHAT_TEST_NODE] Waiting for peers (timeout: {:?})", timeout);

            while start.elapsed() < timeout {
                tokio::select! {
                    event_result = event_receiver.recv() => {
                        if let Ok(event) = event_result {
                            if let libp2p::swarm::SwarmEvent::Behaviour(behaviour) = &event.0 {
                                use netabase::NetabaseBehaviourEvent;
                                match behaviour {
                                    NetabaseBehaviourEvent::Mdns(mdns_event) => {
                                        use libp2p::mdns::Event;
                                        if let Event::Discovered(peers) = mdns_event {
                                            peer_count += peers.len();
                                            eprintln!("[CHAT_TEST_NODE] Discovered {} peers", peers.len());
                                        }
                                    }
                                    NetabaseBehaviourEvent::Identify(identify_event) => {
                                        use libp2p::identify::Event;
                                        if let Event::Received { .. } = identify_event {
                                            eprintln!("[CHAT_TEST_NODE] Peer identified");
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                }
            }

            *netabase = Some(nb);
            send_response(Response::SwarmStarted { peer_count })?;
        }

        Command::SendMessage { content } => {
            if let Some(nb) = netabase {
                let message = Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    sender: username.clone(),
                    content: content.clone(),
                    timestamp: Utc::now().timestamp(),
                };

                eprintln!("[CHAT_TEST_NODE] Sending message: {}", content);
                nb.put_record(message.clone()).await?;
                send_response(Response::MessageSent { id: message.id })?;
            } else {
                return Err(anyhow::anyhow!("Swarm not started"));
            }
        }

        Command::StoreMessageLocally { sender: msg_sender, content } => {
            if let Some(nb) = netabase {
                let message = Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    sender: msg_sender,
                    content: content.clone(),
                    timestamp: Utc::now().timestamp(),
                };

                eprintln!("[CHAT_TEST_NODE] Storing message locally: {}", content);
                // Store directly to local Kademlia store (bypasses DHT publication)
                nb.put_record_locally(ChatDefinition::Message(message.clone())).await?;
                send_response(Response::MessageSent { id: message.id })?;
            } else {
                return Err(anyhow::anyhow!("Swarm not started"));
            }
        }

        Command::GetHistory => {
            if let Some(nb) = netabase {
                let records = nb.query_local_records(None).await?;
                let mut messages: Vec<MessageInfo> = records
                    .into_iter()
                    .map(|def| match def {
                        ChatDefinition::Message(msg) => MessageInfo {
                            id: msg.id,
                            sender: msg.sender,
                            content: msg.content,
                            timestamp: msg.timestamp,
                        },
                    })
                    .collect();

                messages.sort_by_key(|m| m.timestamp);

                eprintln!("[CHAT_TEST_NODE] History has {} messages", messages.len());
                send_response(Response::History { messages })?;
            } else {
                return Err(anyhow::anyhow!("Swarm not started"));
            }
        }

        Command::WaitForPeers { count, timeout_secs } => {
            if let Some(nb) = netabase {
                let mut event_receiver = nb.subscribe_to_broadcasts();
                let mut discovered_peers = 0;
                let start = tokio::time::Instant::now();
                let timeout = Duration::from_secs(timeout_secs);

                eprintln!("[CHAT_TEST_NODE] Waiting for {} peers", count);

                while discovered_peers < count && start.elapsed() < timeout {
                    tokio::select! {
                        event_result = event_receiver.recv() => {
                            if let Ok(event) = event_result {
                                if let libp2p::swarm::SwarmEvent::Behaviour(behaviour) = &event.0 {
                                    use netabase::NetabaseBehaviourEvent;
                                    if let NetabaseBehaviourEvent::Mdns(mdns_event) = behaviour {
                                        use libp2p::mdns::Event;
                                        if let Event::Discovered(peers) = mdns_event {
                                            discovered_peers += peers.len();
                                            eprintln!("[CHAT_TEST_NODE] Total peers: {}", discovered_peers);
                                        }
                                    }
                                }
                            }
                        }
                        _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                    }
                }

                send_response(Response::PeersDiscovered { count: discovered_peers })?;
            } else {
                return Err(anyhow::anyhow!("Swarm not started"));
            }
        }

        Command::WaitForMessage { sender, content, timeout_secs } => {
            if let Some(nb) = netabase {
                let start = tokio::time::Instant::now();
                let timeout = Duration::from_secs(timeout_secs);

                eprintln!("[CHAT_TEST_NODE] Waiting for message from {} containing '{}'", sender, content);

                while start.elapsed() < timeout {
                    let records = nb.query_local_records(None).await?;
                    let messages: Vec<Message> = records
                        .into_iter()
                        .map(|def| match def {
                            ChatDefinition::Message(msg) => msg,
                        })
                        .collect();

                    if let Some(msg) = messages.iter().find(|m|
                        m.sender == sender && m.content.contains(&content)
                    ) {
                        eprintln!("[CHAT_TEST_NODE] Found message!");
                        send_response(Response::MessageReceived {
                            sender: msg.sender.clone(),
                            content: msg.content.clone(),
                        })?;
                        return Ok(());
                    }

                    tokio::time::sleep(Duration::from_millis(500)).await;
                }

                return Err(anyhow::anyhow!("Timeout waiting for message"));
            } else {
                return Err(anyhow::anyhow!("Swarm not started"));
            }
        }

        Command::GetPeerCount => {
            // For now, return a simple response
            // In a full implementation, we'd query the swarm for connected peers
            send_response(Response::PeerCount { count: 0 })?;
        }

        Command::Shutdown => {
            send_response(Response::Ok)?;
            std::process::exit(0);
        }
    }

    Ok(())
}
