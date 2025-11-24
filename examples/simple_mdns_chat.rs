//! Simple mDNS Chat Example
//!
//! A basic peer-to-peer chat application demonstrating Netabase sync capabilities
//! and persistent message storage.
//!
//! ## Usage
//!
//! ```bash
//! # Terminal 1 - Alice
//! cargo run --example simple_mdns_chat --all-features -- alice
//!
//! # Terminal 2 - Bob
//! cargo run --example simple_mdns_chat --all-features -- bob
//! ```
//!
//! ## Commands
//! - Type a message to send (stored locally)
//! - `/history` - View all stored messages
//! - `/peers` - Show connected peer count
//! - `/config` - Show sync configuration
//! - `/sync` - Manually trigger sync with peers
//! - `/help` - Show help
//! - `quit` or `exit` - Exit the chat
//!
//! ## Note
//! This example demonstrates:
//! - High-level sync API methods (trigger_sync, peer_count, etc.)
//! - Local message storage and retrieval
//! - Database querying with query_local_records()

use anyhow::Result;
use netabase::Netabase;
use netabase::network::config::{NetabaseConfig, SyncConfig};
use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
use std::env;
use std::io::{self, Write};
use tokio::time::{sleep, Duration};

// Define chat data model using the macro system
#[netabase_definition_module(ChatDefinition, ChatKeys)]
mod chat {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(ChatDefinition)]
    pub struct Message {
        /// Unique message ID (timestamp + sender)
        #[primary_key]
        pub id: String,
        /// Username of the sender
        #[secondary_key]
        pub sender: String,
        /// Message content
        pub content: String,
        /// Unix timestamp
        pub timestamp: i64,
    }
}

use chat::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Get username from command line args
    let args: Vec<String> = env::args().collect();
    let username = if args.len() > 1 {
        args[1].clone()
    } else {
        format!("user_{}", std::process::id())
    };

    println!("\n╔═══════════════════════════════════════╗");
    println!("║   Netabase P2P Chat Demo              ║");
    println!("╚═══════════════════════════════════════╝\n");
    println!("Welcome, {}!", username);
    println!("Setting up your node...\n");

    // Configure with sync enabled - demonstrates SyncConfig API
    let sync_config = SyncConfig::development(); // Fast sync for demo
    let config = NetabaseConfig::default();
    let mut config_with_sync = config.clone();
    config_with_sync.sync = Some(sync_config);

    // Create database path
    let db_path = format!("./chat_data/{}", username);

    // Initialize Netabase
    let mut netabase = Netabase::<ChatDefinition>::new_with_path_and_config(
        &db_path,
        config_with_sync,
    )?;

    // Start the network swarm
    println!("Starting network...");
    netabase.start_swarm().await?;

    // Give network time to initialize
    sleep(Duration::from_secs(1)).await;

    // Demonstrate peer_count() API
    match netabase.peer_count().await {
        Ok(count) => {
            if count > 0 {
                println!("✓ Connected to {} peer(s)", count);

                // Demonstrate is_sync_enabled() and trigger_sync() APIs
                if netabase.is_sync_enabled() {
                    println!("⟳ Sync is enabled - triggering initial sync...");
                    match netabase.trigger_sync().await {
                        Ok(_) => {
                            println!("✓ Sync completed successfully");
                        }
                        Err(e) => eprintln!("Warning: Sync failed: {}", e),
                    }
                }
            } else {
                println!("⚠ No peers discovered yet");
                println!("  Start another instance to connect peers");
            }
        }
        Err(e) => eprintln!("Warning: Could not get peer count: {}", e),
    }

    // Load and display existing messages
    match load_history(&netabase).await {
        Ok(count) => {
            if count > 0 {
                println!("\n✓ Loaded {} message(s) from history", count);
                println!("  Type /history to view all messages");
            }
        }
        Err(e) => eprintln!("Warning: Could not load history: {}", e),
    }

    println!("\n────────────────────────────────────────");
    println!("Type /help for commands");
    println!("────────────────────────────────────────\n");

    // Main interaction loop
    loop {
        // Show prompt
        print!("{} > ", username);
        io::stdout().flush()?;

        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let message_content = input.trim();

        // Skip empty messages
        if message_content.is_empty() {
            continue;
        }

        // Handle exit commands
        if message_content == "quit" || message_content == "exit" {
            println!("\nGoodbye!");
            break;
        }

        // Handle slash commands
        if message_content.starts_with('/') {
            match message_content {
                "/help" => print_help(),
                "/history" => show_history(&netabase).await?,
                "/peers" => show_peers(&netabase).await?,
                "/config" => show_config(&netabase),
                "/sync" => trigger_manual_sync(&netabase).await?,
                _ => println!("Unknown command. Type /help for available commands."),
            }
            continue;
        }

        // Store message in the database
        match store_message(&netabase, &username, message_content).await {
            Ok(_) => println!("✓ Message stored in database"),
            Err(e) => eprintln!("✗ Failed to store message: {}", e),
        }
    }

    // Cleanup
    netabase.stop_swarm().await?;
    Ok(())
}

/// Store a message in the local database
async fn store_message(
    netabase: &Netabase<ChatDefinition>,
    sender: &str,
    content: &str,
) -> Result<()> {
    // Create message with unique ID
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    let message_id = format!("{}_{}", timestamp, sender);

    let message = Message {
        id: message_id,
        sender: sender.to_string(),
        content: content.to_string(),
        timestamp,
    };

    // Store using put_record (now works with macro-generated RecordStoreExt!)
    netabase.put_record(message).await?;

    Ok(())
}

/// Show all messages from local database
async fn show_history(netabase: &Netabase<ChatDefinition>) -> Result<()> {
    println!("\n═══ Message History ═══");

    // Query all local records using the query_local_records API
    match netabase.query_local_records(None).await {
        Ok(records) => {
            if records.is_empty() {
                println!("No messages in history.");
            } else {
                // Collect messages and sort by timestamp
                let mut messages: Vec<(String, String, i64)> = Vec::new();

                for record in records {
                    if let ChatDefinition::Message(msg) = record {
                        messages.push((msg.sender.clone(), msg.content.clone(), msg.timestamp));
                    }
                }

                messages.sort_by_key(|(_, _, ts)| *ts);

                println!("Found {} message(s):\n", messages.len());

                for (sender, content, timestamp) in messages {
                    let time = format_timestamp(timestamp);
                    println!("[{}] {}: {}", time, sender, content);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to query history: {}", e);
        }
    }

    println!("═══════════════════════\n");
    Ok(())
}

/// Load history count (for startup message)
async fn load_history(netabase: &Netabase<ChatDefinition>) -> Result<usize> {
    match netabase.query_local_records(None).await {
        Ok(records) => {
            let count = records
                .iter()
                .filter(|m| matches!(m, ChatDefinition::Message(_)))
                .count();
            Ok(count)
        }
        Err(e) => Err(e),
    }
}

/// Show connected peers
async fn show_peers(netabase: &Netabase<ChatDefinition>) -> Result<()> {
    match netabase.peer_count().await {
        Ok(count) => {
            println!("\nConnected peers: {}", count);
            if count == 0 {
                println!("Start another instance to connect peers.");
            }
            println!();
        }
        Err(e) => eprintln!("Error getting peer count: {}", e),
    }
    Ok(())
}

/// Show sync configuration
fn show_config(netabase: &Netabase<ChatDefinition>) {
    if let Some(sync_config) = netabase.sync_config() {
        println!("\n═══ Sync Configuration ═══");
        println!("Enabled: {}", sync_config.enabled);
        println!("Auto-sync: {}", sync_config.auto_sync);
        println!("Sync interval: {:?}", sync_config.sync_interval);
        println!("Gossip enabled: {}", sync_config.gossip.enabled);
        println!("Gossip interval: {:?}", sync_config.gossip.interval);
        println!("Gossip fanout: {}", sync_config.gossip.fanout);
        println!("BRB enabled: {}", sync_config.brb.enabled);
        println!("BRB total peers: {}", sync_config.brb.total_peers);
        println!("BRB max faulty: {}", sync_config.brb.max_faulty);
        println!("Sybil resistance: {}", sync_config.sybil_resistance.enabled);
        println!("═════════════════════════\n");
    } else {
        println!("\nSync is not enabled\n");
    }
}

/// Manually trigger sync
async fn trigger_manual_sync(netabase: &Netabase<ChatDefinition>) -> Result<()> {
    println!("\n⟳ Triggering sync...");
    match netabase.trigger_sync().await {
        Ok(_) => {
            println!("✓ Sync initiated successfully\n");
        }
        Err(e) => eprintln!("✗ Sync failed: {}\n", e),
    }
    Ok(())
}

/// Format Unix timestamp as readable time
fn format_timestamp(timestamp: i64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let duration = Duration::from_secs(timestamp as u64);
    let datetime = UNIX_EPOCH + duration;

    // Simple formatting showing relative time
    let now = SystemTime::now();
    let elapsed = now.duration_since(datetime).ok();

    if let Some(elapsed) = elapsed {
        let secs = elapsed.as_secs();
        if secs < 60 {
            format!("{}s ago", secs)
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h ago", secs / 3600)
        } else {
            format!("{}d ago", secs / 86400)
        }
    } else {
        "just now".to_string()
    }
}

fn print_help() {
    println!("\n═══ Available Commands ═══");
    println!("  <message>   - Send a message and store it in the database");
    println!("  /history    - View all stored messages from the database");
    println!("  /peers      - Show number of connected peers");
    println!("  /config     - Show sync configuration");
    println!("  /sync       - Manually trigger sync with peers");
    println!("  /help       - Show this help");
    println!("  quit        - Exit the application");
    println!("═══════════════════════════\n");
}
