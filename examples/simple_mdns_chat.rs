use anyhow::Result;
use chrono::Utc;
use netabase::Netabase;
use netabase_store::*;
use std::io::{self, Write};
use tokio::time::{Duration, sleep};

// Define the chat schema with Message model
#[netabase_definition_module(ChatDefinition, ChatKeys)]
mod chat {
    use netabase_store::*;

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
    pub struct Message {
        #[primary_key]
        pub id: String, // UUID as string
        pub sender: String,
        pub content: String,
        pub timestamp: i64, // Unix timestamp
    }
}

use chat::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Simple mDNS Chat Application ===");
    println!("Initializing netabase...\n");

    // EXAMPLE SETUP

    // Get a unique database path for this instance
    let username = std::env::args()
        .nth(1)
        .unwrap_or_else(|| format!("user_{}", std::process::id()));

    let db_path = format!("./chat_data/{}", username);

    // Create netabase instance
    let mut netabase = Netabase::<ChatDefinition>::new_with_path(&db_path)?;

    println!("Welcome, {}!", username);
    println!("Database path: {}\n", db_path);

    // Start the swarm
    println!("Starting P2P network...");
    netabase.start_swarm().await?;
    println!("Network started! Listening for peers...\n");

    // Subscribe to network events for peer discovery
    let mut event_receiver = netabase.subscribe_to_broadcasts();

    // Wait for mDNS peer discovery
    println!("Waiting for peer discovery via mDNS...");
    let mut peer_connected = false;
    let mut attempts = 0;
    let max_wait_time = Duration::from_secs(30);
    let start_time = tokio::time::Instant::now();

    while !peer_connected && start_time.elapsed() < max_wait_time {
        tokio::select! {
            event_result = event_receiver.recv() => {
                if let Ok(event) = event_result {
                    let inner_event = &event.0;

                    // Check for mDNS discovered peers
                    if let libp2p::swarm::SwarmEvent::Behaviour(behaviour_event) = inner_event {
                        use netabase::network::behaviour::NetabaseBehaviourEvent;

                        match behaviour_event {
                            NetabaseBehaviourEvent::Mdns(mdns_event) => {
                                use libp2p::mdns::Event;
                                match mdns_event {
                                    Event::Discovered(peers) => {
                                        for (peer_id, _addr) in peers {
                                            println!("✓ Discovered peer via mDNS: {}", peer_id);
                                            peer_connected = true;
                                        }
                                    }
                                    Event::Expired(_) => {}
                                }
                            }
                            NetabaseBehaviourEvent::Identify(identify_event) => {
                                use libp2p::identify::Event;
                                if let Event::Received { peer_id, .. } = identify_event {
                                    println!("✓ Identified peer: {}", peer_id);
                                    peer_connected = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ = sleep(Duration::from_millis(100)) => {
                attempts += 1;
                if attempts % 10 == 0 {
                    print!(".");
                    io::stdout().flush().unwrap();
                }
            }
        }
    }

    if peer_connected {
        println!("\n✓ Connected to peers! You can now send messages.\n");
    } else {
        println!(
            "\n⚠ No peers discovered. You can still send messages (they'll be stored locally).\n"
        );
    }

    // Start chat loop
    println!("Commands:");
    println!("  /history - View all messages in local store");
    println!("  /help    - Show this help message");
    println!("  quit     - Exit the chat\n");

    let stdin = io::stdin();
    let mut input_buffer = String::new();

    loop {
        print!("{}: ", username);
        io::stdout().flush()?;

        input_buffer.clear();
        stdin.read_line(&mut input_buffer)?;
        let message_content = input_buffer.trim().to_string();

        if message_content.is_empty() {
            continue;
        }

        // Handle commands
        if message_content == "quit" || message_content == "exit" {
            println!("\nGoodbye!\n");
            break;
        }

        if message_content == "/history" {
            println!("\n=== Message History ===");
            match netabase.query_local_records(None).await {
                Ok(records) => {
                    if records.is_empty() {
                        println!("(No messages yet)");
                    } else {
                        // Extract messages and sort by timestamp
                        let mut messages: Vec<Message> = records
                            .into_iter()
                            .map(|def| match def {
                                ChatDefinition::Message(msg) => msg,
                            })
                            .collect();

                        messages.sort_by_key(|m| m.timestamp);

                        for msg in messages {
                            let dt = chrono::DateTime::<Utc>::from_timestamp(msg.timestamp, 0)
                                .unwrap_or_else(Utc::now);
                            println!(
                                "[{}] {}: {}",
                                dt.format("%H:%M:%S"),
                                msg.sender,
                                msg.content
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to query history: {}", e);
                }
            }
            println!("=======================\n");
            continue;
        }

        if message_content == "/help" {
            println!("\n=== Commands ===");
            println!("  /history - View all messages in local store");
            println!("  /help    - Show this help message");
            println!("  quit     - Exit the chat");
            println!("================\n");
            continue;
        }

        // Create message with UUID
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            sender: username.clone(),
            content: message_content.clone(),
            timestamp: Utc::now().timestamp(),
        };

        // Store message in DHT (will also persist locally)
        match netabase.put_record(message.clone()).await {
            Ok(_) => {
                println!("📤 {}: {}\n", username, message_content);
            }
            Err(e) => {
                eprintln!("✗ Failed to send message: {}\n", e);
            }
        }

        // Small delay to avoid flooding
        sleep(Duration::from_millis(100)).await;
    }

    // Cleanup
    netabase.stop_swarm().await?;
    println!("Chat session ended.");

    Ok(())
}
