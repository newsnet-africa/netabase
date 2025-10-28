use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use netabase::Netabase;
use netabase::network::config::{NetabaseConfig, SyncConfig, GossipConfig, BrbConfig, SybilResistanceConfig, PaxosConfig};
use netabase_store::*;
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;

// Define the chat schema with Message model
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
        pub id: String, // UUID as string
        pub sender: String,
        pub content: String,
        pub timestamp: i64, // Unix timestamp
    }
}

use chat::*;

/// Simple mDNS Chat Application with Byzantine Fault Tolerant Synchronization
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Username for the chat (defaults to user_<pid>)
    #[arg(short, long)]
    username: Option<String>,

    /// Database path (defaults to ./chat_data/<username>)
    #[arg(short, long)]
    db_path: Option<String>,

    // ===== Sync Configuration =====
    /// Enable synchronization
    #[arg(long, default_value_t = true)]
    sync_enabled: bool,

    /// Auto-sync on record updates
    #[arg(long, default_value_t = true)]
    auto_sync: bool,

    /// Sync interval in seconds
    #[arg(long, default_value_t = 30)]
    sync_interval: u64,

    // ===== Gossip Configuration =====
    /// Enable gossip protocol
    #[arg(long, default_value_t = true)]
    gossip_enabled: bool,

    /// Gossip interval in seconds
    #[arg(long, default_value_t = 10)]
    gossip_interval: u64,

    /// Gossip fanout (number of peers to gossip with per round)
    #[arg(long, default_value_t = 3)]
    gossip_fanout: usize,

    // ===== Byzantine Reliable Broadcast (BRB) Configuration =====
    /// Enable Byzantine Reliable Broadcast for critical messages
    #[arg(long, default_value_t = true)]
    brb_enabled: bool,

    /// Total number of peers in BRB quorum
    #[arg(long, default_value_t = 7)]
    brb_total_peers: usize,

    /// Maximum Byzantine faults to tolerate (f in 3f+1 formula)
    #[arg(long, default_value_t = 2)]
    brb_max_faulty: usize,

    // ===== Proof-of-Work (PoW) Sybil Resistance =====
    /// Enable Proof-of-Work Sybil resistance
    #[arg(long, default_value_t = true)]
    pow_enabled: bool,

    /// PoW difficulty (number of leading zero bits required)
    /// Recommended: 16 for testing, 20 for production, 24+ for high security
    #[arg(long, default_value_t = 16)]
    pow_difficulty: u32,

    /// Challenge duration in seconds (how long peer has to solve challenge)
    #[arg(long, default_value_t = 60)]
    pow_challenge_duration: u64,

    /// Verification validity duration in seconds (how long verification lasts)
    #[arg(long, default_value_t = 3600)]
    pow_verification_duration: u64,

    // ===== Reputation System =====
    /// Enable reputation-based peer filtering
    #[arg(long, default_value_t = true)]
    reputation_enabled: bool,

    // ===== Paxos Consensus =====
    /// Enable Paxos consensus for critical operations
    #[arg(long, default_value_t = false)]
    paxos_enabled: bool,

    /// Number of Paxos acceptors
    #[arg(long, default_value_t = 5)]
    paxos_acceptors: usize,

    /// Maximum failures Paxos can tolerate
    #[arg(long, default_value_t = 2)]
    paxos_max_failures: usize,

    // ===== Presets =====
    /// Use a preset configuration (overrides individual flags)
    /// Options: development, production, high-security, low-latency
    #[arg(long)]
    preset: Option<String>,
}

impl Args {
    /// Build SyncConfig from command-line arguments
    fn build_sync_config(&self) -> SyncConfig {
        // Apply preset if specified
        if let Some(preset) = &self.preset {
            return match preset.as_str() {
                "development" => SyncConfig {
                    enabled: true,
                    gossip: GossipConfig {
                        enabled: true,
                        interval: Duration::from_secs(5),
                        fanout: 2,
                    },
                    brb: BrbConfig {
                        enabled: false,
                        total_peers: 4,
                        max_faulty: 1,
                    },
                    sybil_resistance: SybilResistanceConfig {
                        enabled: true,
                        pow_difficulty: 12,
                        challenge_duration: Duration::from_secs(30),
                        verification_duration: Duration::from_secs(1800),
                        reputation_enabled: true,
                    },
                    paxos: PaxosConfig {
                        enabled: false,
                        num_acceptors: 3,
                        max_failures: 1,
                    },
                    auto_sync: true,
                    sync_interval: Duration::from_secs(15),
                },
                "production" => SyncConfig {
                    enabled: true,
                    gossip: GossipConfig {
                        enabled: true,
                        interval: Duration::from_secs(10),
                        fanout: 3,
                    },
                    brb: BrbConfig {
                        enabled: true,
                        total_peers: 7,
                        max_faulty: 2,
                    },
                    sybil_resistance: SybilResistanceConfig {
                        enabled: true,
                        pow_difficulty: 20,
                        challenge_duration: Duration::from_secs(60),
                        verification_duration: Duration::from_secs(3600),
                        reputation_enabled: true,
                    },
                    paxos: PaxosConfig {
                        enabled: false,
                        num_acceptors: 5,
                        max_failures: 2,
                    },
                    auto_sync: true,
                    sync_interval: Duration::from_secs(30),
                },
                "high-security" => SyncConfig {
                    enabled: true,
                    gossip: GossipConfig {
                        enabled: true,
                        interval: Duration::from_secs(15),
                        fanout: 4,
                    },
                    brb: BrbConfig {
                        enabled: true,
                        total_peers: 10,
                        max_faulty: 3,
                    },
                    sybil_resistance: SybilResistanceConfig {
                        enabled: true,
                        pow_difficulty: 24,
                        challenge_duration: Duration::from_secs(120),
                        verification_duration: Duration::from_secs(1800),
                        reputation_enabled: true,
                    },
                    paxos: PaxosConfig {
                        enabled: true,
                        num_acceptors: 7,
                        max_failures: 3,
                    },
                    auto_sync: true,
                    sync_interval: Duration::from_secs(30),
                },
                "low-latency" => SyncConfig {
                    enabled: true,
                    gossip: GossipConfig {
                        enabled: true,
                        interval: Duration::from_secs(1),
                        fanout: 5,
                    },
                    brb: BrbConfig {
                        enabled: true,
                        total_peers: 7,
                        max_faulty: 2,
                    },
                    sybil_resistance: SybilResistanceConfig {
                        enabled: true,
                        pow_difficulty: 16,
                        challenge_duration: Duration::from_secs(30),
                        verification_duration: Duration::from_secs(3600),
                        reputation_enabled: true,
                    },
                    paxos: PaxosConfig {
                        enabled: false,
                        num_acceptors: 5,
                        max_failures: 2,
                    },
                    auto_sync: true,
                    sync_interval: Duration::from_secs(5),
                },
                _ => {
                    eprintln!("Unknown preset '{}', using defaults", preset);
                    SyncConfig::default()
                }
            };
        }

        // Build from individual flags
        SyncConfig {
            enabled: self.sync_enabled,
            gossip: GossipConfig {
                enabled: self.gossip_enabled,
                interval: Duration::from_secs(self.gossip_interval),
                fanout: self.gossip_fanout,
            },
            brb: BrbConfig {
                enabled: self.brb_enabled,
                total_peers: self.brb_total_peers,
                max_faulty: self.brb_max_faulty,
            },
            sybil_resistance: SybilResistanceConfig {
                enabled: self.pow_enabled,
                pow_difficulty: self.pow_difficulty,
                challenge_duration: Duration::from_secs(self.pow_challenge_duration),
                verification_duration: Duration::from_secs(self.pow_verification_duration),
                reputation_enabled: self.reputation_enabled,
            },
            paxos: PaxosConfig {
                enabled: self.paxos_enabled,
                num_acceptors: self.paxos_acceptors,
                max_failures: self.paxos_max_failures,
            },
            auto_sync: self.auto_sync,
            sync_interval: Duration::from_secs(self.sync_interval),
        }
    }
}

fn print_sync_config(config: &SyncConfig) {
    println!("\n=== Sync Configuration ===");
    println!("Sync enabled: {}", config.enabled);
    println!("Auto-sync: {}", config.auto_sync);
    println!("Sync interval: {:?}", config.sync_interval);

    println!("\nGossip Protocol:");
    println!("  Enabled: {}", config.gossip.enabled);
    println!("  Interval: {:?}", config.gossip.interval);
    println!("  Fanout: {}", config.gossip.fanout);

    println!("\nByzantine Reliable Broadcast:");
    println!("  Enabled: {}", config.brb.enabled);
    println!("  Total peers: {}", config.brb.total_peers);
    println!("  Max faulty: {} (tolerates up to {} Byzantine peers)",
             config.brb.max_faulty, config.brb.max_faulty);

    println!("\nSybil Resistance:");
    println!("  PoW enabled: {}", config.sybil_resistance.enabled);
    println!("  PoW difficulty: {} (avg ~{:.1}s to solve)",
             config.sybil_resistance.pow_difficulty,
             2f64.powi(config.sybil_resistance.pow_difficulty as i32) / 1_000_000.0);
    println!("  Challenge duration: {:?}", config.sybil_resistance.challenge_duration);
    println!("  Verification duration: {:?}", config.sybil_resistance.verification_duration);
    println!("  Reputation enabled: {}", config.sybil_resistance.reputation_enabled);

    println!("\nPaxos Consensus:");
    println!("  Enabled: {}", config.paxos.enabled);
    if config.paxos.enabled {
        println!("  Acceptors: {}", config.paxos.num_acceptors);
        println!("  Max failures: {}", config.paxos.max_failures);
    }
    println!("==========================\n");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    println!("=== Simple mDNS Chat Application ===");
    println!("Initializing netabase with sync...\n");

    // Get username
    let username = args.username.clone().unwrap_or_else(|| {
        format!("user_{}", std::process::id())
    });

    // Get database path
    let db_path = args.db_path.clone().unwrap_or_else(|| {
        format!("./chat_data/{}", username)
    });

    // Build sync configuration
    let sync_config = args.build_sync_config();

    // Display configuration
    print_sync_config(&sync_config);

    // Create netabase config
    let netabase_config = NetabaseConfig {
        sync: sync_config.clone(),
        ..Default::default()
    };

    // Create netabase instance with configuration
    let mut netabase = Netabase::<ChatDefinition>::new_with_path_and_config(
        &db_path,
        netabase_config,
    )?;

    println!("Welcome, {}!", username);
    println!("Database path: {}\n", db_path);

    // Start the swarm
    println!("Starting P2P network with sync...");
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
                        use netabase::NetabaseBehaviourEvent;

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
                            #[cfg(feature = "native")]
                            NetabaseBehaviourEvent::Sync(sync_event) => {
                                // Log sync events
                                use libp2p::request_response::Event;
                                match sync_event {
                                    Event::Message { peer, .. } => {
                                        println!("🔄 Sync message with peer: {}", peer);
                                    }
                                    Event::OutboundFailure { peer, error, .. } => {
                                        eprintln!("⚠ Sync failed with {}: {:?}", peer, error);
                                    }
                                    Event::InboundFailure { peer, error, .. } => {
                                        eprintln!("⚠ Inbound sync failed with {}: {:?}", peer, error);
                                    }
                                    _ => {}
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
        println!("\n✓ Connected to peers! Messages will be synced.\n");
    } else {
        println!(
            "\n⚠ No peers discovered. Messages will be stored locally.\n"
        );
    }

    // Start chat loop
    println!("Commands:");
    println!("  /history        - View all messages in local store");
    println!("  /paxos-history  - View Paxos consensus history");
    println!("  /config         - Show current sync configuration");
    println!("  /help           - Show this help message");
    println!("  quit            - Exit the chat\n");

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

        if message_content == "/config" {
            print_sync_config(&sync_config);
            continue;
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

        if message_content == "/paxos-history" {
            println!("\n=== Paxos Consensus History ===");
            if sync_config.paxos.enabled {
                println!("Paxos Consensus: ENABLED");
                println!("\nConfiguration:");
                println!("  Acceptors: {}", sync_config.paxos.num_acceptors);
                println!("  Max failures tolerated: {}", sync_config.paxos.max_failures);
                println!("  Quorum size: {} (f+1)", sync_config.paxos.max_failures + 1);
                println!("\nAbout Paxos:");
                println!("  Paxos is a consensus protocol that ensures agreement on");
                println!("  critical operations even when some peers fail or act");
                println!("  maliciously. With {} acceptors, the system can tolerate",
                         sync_config.paxos.num_acceptors);
                println!("  up to {} Byzantine failures.", sync_config.paxos.max_failures);
                println!("\n  When enabled, Paxos provides:");
                println!("    • Strong consistency guarantees");
                println!("    • Byzantine fault tolerance");
                println!("    • History tracking of consensus decisions");
                println!("\n  Note: Full Paxos integration with message consensus");
                println!("  is available through the Netabase sync API.");
            } else {
                println!("Paxos Consensus: DISABLED");
                println!("\nTo enable Paxos, use: --paxos-enabled");
                println!("Example:");
                println!("  cargo run --example simple_mdns_chat -- --paxos-enabled");
                println!("\nOr use a preset with Paxos:");
                println!("  cargo run --example simple_mdns_chat -- --preset high-security");
            }
            println!("===============================\n");
            continue;
        }

        if message_content == "/help" {
            println!("\n=== Commands ===");
            println!("  /history        - View all messages in local store");
            println!("  /paxos-history  - View Paxos consensus status and configuration");
            println!("  /config         - Show current sync configuration");
            println!("  /help           - Show this help message");
            println!("  quit            - Exit the chat");
            println!("\n=== Sync Features ===");
            if sync_config.enabled {
                println!("✓ Sync is ENABLED");
                if sync_config.gossip.enabled {
                    println!("  ✓ Gossip protocol active (interval: {:?})", sync_config.gossip.interval);
                }
                if sync_config.brb.enabled {
                    println!("  ✓ Byzantine Reliable Broadcast active");
                }
                if sync_config.sybil_resistance.enabled {
                    println!("  ✓ Sybil resistance (PoW difficulty: {})", sync_config.sybil_resistance.pow_difficulty);
                }
                if sync_config.paxos.enabled {
                    println!("  ✓ Paxos consensus enabled ({} acceptors, f={})",
                             sync_config.paxos.num_acceptors,
                             sync_config.paxos.max_failures);
                    println!("     Use /paxos-history for details");
                }
            } else {
                println!("✗ Sync is DISABLED");
            }
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

        // Store message in DHT (will also persist locally and sync if enabled)
        match netabase.put_record(message.clone()).await {
            Ok(_) => {
                println!("📤 {}: {}\n", username, message_content);
                if sync_config.enabled {
                    println!("   (Message queued for sync)");
                    if sync_config.paxos.enabled {
                        println!("   ✓ Paxos consensus enabled for critical operations");
                    }
                }
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
