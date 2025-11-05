//! Test helper binary for P2P integration tests
//!
//! This binary can be spawned as a subprocess and controlled via commands
//! to test actual inter-process communication between netabase nodes.

use anyhow::Result;
use netabase::Netabase;
use netabase_store::*;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use tokio::time::Duration;

// Test schema for integration tests
#[netabase_definition_module(TestDefinition, TestKeys)]
mod test_schema {
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
    #[netabase(TestDefinition)]
    pub struct TestRecord {
        #[primary_key]
        pub id: String,
        pub data: String,
        pub timestamp: i64,
    }
}

use test_schema::*;

/// Commands that can be sent to the test node via stdin
#[derive(Debug, Serialize, Deserialize)]
enum Command {
    /// Start the swarm
    StartSwarm,

    /// Wait for peer discovery
    WaitForPeers { timeout_secs: u64 },

    /// Put a record in the DHT
    PutRecord { id: String, data: String },

    /// Get a record from local storage
    GetRecord { id: String },

    /// Get a record from DHT (network query)
    GetRecordFromDHT { id: String },

    /// Query local records
    QueryLocal { limit: Option<usize> },

    /// Start providing a key
    StartProviding { id: String },

    /// Stop providing a key
    StopProviding { id: String },

    /// Get providers for a key
    GetProviders { id: String },

    /// Bootstrap to join DHT
    Bootstrap,

    /// Bootstrap with explicit peer list
    BootstrapWithPeers { peers: Vec<String> },

    /// Get peer info
    GetPeers,

    /// Get local peer ID
    GetPeerId,

    /// Get listen addresses
    GetListenAddrs,

    /// Add a peer address
    AddPeerAddress { peer_id: String, multiaddr: String },

    /// Remove a peer
    RemovePeer { peer_id: String },

    /// Get routing table information
    GetRoutingTableInfo,

    /// Shutdown
    Shutdown,
}

/// Responses sent back via stdout
#[derive(Debug, Serialize, Deserialize)]
enum Response {
    Ok,
    Error(String),
    PeerId(String),
    ListenAddrs(Vec<String>),
    PeerDiscovered { peer_id: String },
    RecordStored { id: String },
    RecordRetrieved { id: String, data: String },
    RecordNotFound { id: String },
    LocalRecords { count: usize, records: Vec<(String, String)> },
    ProvidersFound { count: usize, providers: Vec<String> },
    PeersConnected { count: usize, peers: Vec<String> },
    BootstrapStarted,
    SwarmStarted,
    RoutingTableInfo { bucket_count: usize, peer_count: usize },
}

fn send_response(resp: Response) -> Result<()> {
    let json = serde_json::to_string(&resp)?;
    println!("{}", json);
    io::stdout().flush()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    // Get node name from args
    let node_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| format!("test_node_{}", std::process::id()));

    eprintln!("[{}] TEST_NODE STARTING", node_name);

    let db_path = format!("./test_data/{}", node_name);

    // Clean up old test data
    let _ = std::fs::remove_dir_all(&db_path);
    std::fs::create_dir_all(&db_path)?;

    eprintln!("[{}] Creating netabase instance at {}", node_name, db_path);

    // Create netabase instance
    let mut netabase = Netabase::<TestDefinition>::new_with_path(&db_path)?;

    eprintln!("[{}] Netabase instance created", node_name);

    // Subscribe to events
    let mut event_receiver = netabase.subscribe_to_broadcasts();

    // Spawn event handler
    let node_name_clone = node_name.clone();
    tokio::spawn(async move {
        eprintln!("[{}] Event handler task started", node_name_clone);
        while let Ok(event) = event_receiver.recv().await {
            match &event.0 {
                libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    eprintln!("[{}] Connection established with peer: {}", node_name_clone, peer_id);
                    let _ = send_response(Response::PeerDiscovered {
                        peer_id: peer_id.to_string(),
                    });
                }
                _ => {}
            }
        }
        eprintln!("[{}] Event handler task ended", node_name_clone);
    });

    eprintln!("[{}] Starting command loop", node_name);

    // Read commands from stdin
    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = line?;
        eprintln!("[{}] Received command: {}", node_name, line);

        let command: Command = match serde_json::from_str(&line) {
            Ok(cmd) => {
                eprintln!("[{}] Parsed command: {:?}", node_name, cmd);
                cmd
            },
            Err(e) => {
                eprintln!("[{}] Failed to parse command: {}", node_name, e);
                send_response(Response::Error(format!("Invalid command: {}", e)))?;
                continue;
            }
        };

        eprintln!("[{}] Handling command...", node_name);
        match handle_command(&mut netabase, command).await {
            Ok(()) => {
                eprintln!("[{}] Command completed successfully", node_name);
            }
            Err(e) => {
                eprintln!("[{}] Command failed: {}", node_name, e);
                send_response(Response::Error(format!("Command failed: {}", e)))?;
            }
        }
    }

    eprintln!("[{}] Command loop ended, cleaning up", node_name);

    // Cleanup
    netabase.stop_swarm().await?;
    let _ = std::fs::remove_dir_all(&db_path);

    eprintln!("[{}] TEST_NODE SHUTDOWN COMPLETE", node_name);

    Ok(())
}

async fn handle_command(netabase: &mut Netabase<TestDefinition>, command: Command) -> Result<()> {
    match command {
        Command::StartSwarm => {
            eprintln!("CMD: Starting swarm...");
            netabase.start_swarm().await?;
            eprintln!("CMD: Swarm started successfully");
            send_response(Response::SwarmStarted)?;
            eprintln!("CMD: SwarmStarted response sent");
        }

        Command::WaitForPeers { timeout_secs } => {
            let mut event_receiver = netabase.subscribe_to_broadcasts();
            let timeout = Duration::from_secs(timeout_secs);
            let start = tokio::time::Instant::now();

            while start.elapsed() < timeout {
                tokio::select! {
                    event_result = event_receiver.recv() => {
                        if let Ok(event) = event_result {
                            if let libp2p::swarm::SwarmEvent::Behaviour(behaviour) = &event.0 {
                                use netabase::NetabaseBehaviourEvent;
                                match behaviour {
                                    NetabaseBehaviourEvent::Mdns(mdns_event) => {
                                        use libp2p::mdns::Event;
                                        if let Event::Discovered(_) = mdns_event {
                                            send_response(Response::Ok)?;
                                            return Ok(());
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
            send_response(Response::Error("Timeout waiting for peers".to_string()))?;
        }

        Command::PutRecord { id, data } => {
            eprintln!("CMD: PutRecord - id={}, data={}", id, data);
            let record = TestRecord {
                id: id.clone(),
                data: data.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            };

            eprintln!("CMD: Calling netabase.put_record()...");
            let start = std::time::Instant::now();
            let result = netabase.put_record(record).await;
            eprintln!("CMD: put_record() returned after {:?}", start.elapsed());

            match result {
                Ok(_) => {
                    eprintln!("CMD: put_record() succeeded");
                    send_response(Response::RecordStored { id })?;
                    eprintln!("CMD: RecordStored response sent");
                }
                Err(e) => {
                    eprintln!("CMD: put_record() failed: {}", e);
                    return Err(e);
                }
            }
        }

        Command::GetRecord { id } => {
            // For now, query local records since get_record is async
            match netabase.query_local_records(None).await {
                Ok(records) => {
                    for record in records {
                        if let TestDefinition::TestRecord(rec) = record {
                            if rec.id == id {
                                send_response(Response::RecordRetrieved {
                                    id: rec.id,
                                    data: rec.data,
                                })?;
                                return Ok(());
                            }
                        }
                    }
                    send_response(Response::RecordNotFound { id })?;
                }
                Err(e) => {
                    send_response(Response::Error(format!("Query failed: {}", e)))?;
                }
            }
        }

        Command::QueryLocal { limit } => {
            match netabase.query_local_records(limit).await {
                Ok(records) => {
                    let records_data: Vec<(String, String)> = records
                        .into_iter()
                        .map(|def| match def {
                            TestDefinition::TestRecord(rec) => (rec.id, rec.data),
                        })
                        .collect();

                    let count = records_data.len();
                    send_response(Response::LocalRecords {
                        count,
                        records: records_data,
                    })?;
                }
                Err(e) => {
                    send_response(Response::Error(format!("Query failed: {}", e)))?;
                }
            }
        }

        Command::StartProviding { id } => {
            eprintln!("CMD: StartProviding - id={}", id);
            // Create a record key for providing
            use netabase_store::traits::model::NetabaseModelTrait;
            let primary_key = TestRecordPrimaryKey(id);
            let key = TestRecordKey::Primary(primary_key);

            eprintln!("CMD: Calling netabase.start_providing()...");
            let start = std::time::Instant::now();
            let result = netabase.start_providing(key).await;
            eprintln!("CMD: start_providing() returned after {:?}", start.elapsed());

            match result {
                Ok(_) => {
                    eprintln!("CMD: start_providing() succeeded");
                    send_response(Response::Ok)?;
                    eprintln!("CMD: Ok response sent");
                }
                Err(e) => {
                    eprintln!("CMD: start_providing() failed: {}", e);
                    return Err(e);
                }
            }
        }

        Command::GetProviders { id } => {
            use netabase_store::traits::model::NetabaseModelTrait;
            let primary_key = TestRecordPrimaryKey(id);
            let key = TestRecordKey::Primary(primary_key);

            let result = netabase.get_providers(key).await?;

            match result {
                libp2p::kad::QueryResult::GetProviders(Ok(get_providers_ok)) => {
                    use libp2p::kad::GetProvidersOk;
                    match get_providers_ok {
                        GetProvidersOk::FoundProviders { providers, .. } => {
                            let provider_ids: Vec<String> = providers.iter().map(|p| p.to_string()).collect();
                            send_response(Response::ProvidersFound {
                                count: provider_ids.len(),
                                providers: provider_ids,
                            })?;
                        }
                        GetProvidersOk::FinishedWithNoAdditionalRecord { .. } => {
                            send_response(Response::ProvidersFound { count: 0, providers: vec![] })?;
                        }
                    }
                }
                _ => {
                    send_response(Response::ProvidersFound { count: 0, providers: vec![] })?;
                }
            }
        }

        Command::GetRecordFromDHT { id } => {
            eprintln!("CMD: GetRecordFromDHT - id={}", id);
            use netabase_store::traits::model::NetabaseModelTrait;
            let primary_key = TestRecordPrimaryKey(id.clone());
            let key = TestRecordKey::Primary(primary_key);

            match netabase.get_record(key).await {
                Ok(result) => {
                    match result {
                        libp2p::kad::QueryResult::GetRecord(Ok(get_record_ok)) => {
                            use libp2p::kad::GetRecordOk;
                            match get_record_ok {
                                GetRecordOk::FoundRecord(peer_record) => {
                                    let record = &peer_record.record;
                                    // Try to deserialize as TestRecord
                                    match bincode::decode_from_slice::<TestRecord, _>(
                                        &record.value,
                                        bincode::config::standard(),
                                    ) {
                                        Ok((test_record, _)) => {
                                            send_response(Response::RecordRetrieved {
                                                id: test_record.id,
                                                data: test_record.data,
                                            })?;
                                        }
                                        Err(e) => {
                                            eprintln!("CMD: Failed to decode record: {}", e);
                                            send_response(Response::RecordNotFound { id })?;
                                        }
                                    }
                                }
                                GetRecordOk::FinishedWithNoAdditionalRecord { .. } => {
                                    send_response(Response::RecordNotFound { id })?;
                                }
                            }
                        }
                        _ => {
                            send_response(Response::RecordNotFound { id })?;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("CMD: get_record failed: {}", e);
                    send_response(Response::RecordNotFound { id })?;
                }
            }
        }

        Command::StopProviding { id } => {
            eprintln!("CMD: StopProviding - id={}", id);
            use netabase_store::traits::model::NetabaseModelTrait;
            let primary_key = TestRecordPrimaryKey(id);
            let key = TestRecordKey::Primary(primary_key);

            match netabase.stop_providing(key).await {
                Ok(_) => {
                    eprintln!("CMD: stop_providing() succeeded");
                    send_response(Response::Ok)?;
                }
                Err(e) => {
                    eprintln!("CMD: stop_providing() failed: {}", e);
                    return Err(e);
                }
            }
        }

        Command::BootstrapWithPeers { peers: _ } => {
            // For now, just do a regular bootstrap
            // In the future, we could add explicit peers to the routing table first
            netabase.bootstrap().await?;
            send_response(Response::BootstrapStarted)?;
        }

        Command::GetListenAddrs => {
            // This would need access to swarm internals
            // For now, return empty list with a note that this needs API exposure
            eprintln!("CMD: GetListenAddrs - API not yet exposed");
            send_response(Response::ListenAddrs(vec![]))?;
        }

        Command::AddPeerAddress { peer_id: _, multiaddr: _ } => {
            // This would need API exposure in Netabase
            eprintln!("CMD: AddPeerAddress - API not yet exposed");
            send_response(Response::Ok)?;
        }

        Command::RemovePeer { peer_id: _ } => {
            // This would need API exposure in Netabase
            eprintln!("CMD: RemovePeer - API not yet exposed");
            send_response(Response::Ok)?;
        }

        Command::GetRoutingTableInfo => {
            // This would need API exposure in Netabase
            eprintln!("CMD: GetRoutingTableInfo - API not yet exposed");
            send_response(Response::RoutingTableInfo {
                bucket_count: 0,
                peer_count: 0,
            })?;
        }

        Command::Bootstrap => {
            netabase.bootstrap().await?;
            send_response(Response::BootstrapStarted)?;
        }

        Command::GetPeers => {
            // This would need access to swarm internals
            // For now, just respond with empty list
            send_response(Response::PeersConnected { count: 0, peers: vec![] })?;
        }

        Command::GetPeerId => {
            // Would need to expose peer ID in Netabase API
            send_response(Response::PeerId("unknown".to_string()))?;
        }

        Command::Shutdown => {
            send_response(Response::Ok)?;
            std::process::exit(0);
        }
    }

    Ok(())
}
