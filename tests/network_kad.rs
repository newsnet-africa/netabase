use std::time::Duration;

use anyhow::{Result, anyhow};
use libp2p::futures::StreamExt;
use libp2p::kad::{QueryResult, RecordKey, store::RecordStore};
use libp2p::{Multiaddr, PeerId};
use log::info;
use netabase::{
    get_test_temp_dir, get_test_temp_dir_str,
    network::swarm::{SwarmAction, generate_swarm},
};
use tokio::time::{sleep, timeout};

#[allow(dead_code)]
fn cleanup_test_dir(_test_name: &str) {
    let test_dir = get_test_temp_dir(None);
    if std::path::Path::new(&test_dir).exists() {
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}

fn init_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();
    });
}

async fn setup_connected_swarms() -> Result<(
    libp2p::Swarm<netabase::network::behaviour::NetabaseBehaviour>,
    libp2p::Swarm<netabase::network::behaviour::NetabaseBehaviour>,
    Multiaddr,
    PeerId,
)> {
    // Create first swarm (writer)
    let temp_dir_1 = get_test_temp_dir_str(Some("writer"));
    let mut swarm1 = generate_swarm(&temp_dir_1)?;

    // Listen on a specific port for the first swarm
    swarm1.listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse()?)?;
    swarm1
        .behaviour_mut()
        .kad
        .set_mode(Some(libp2p::kad::Mode::Server));

    // Wait for the listener to be established
    let mut listen_addr = None;
    let peer1_id = *swarm1.local_peer_id();

    // Poll until we get a listen address
    for _ in 0..10 {
        let event = swarm1.select_next_some().await;
        match event {
            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                listen_addr = Some(address.clone());
                info!("Writer listening on: {}", address);
                break;
            }
            _ => {}
        }
        sleep(Duration::from_millis(100)).await;
    }

    let listen_addr = listen_addr.ok_or_else(|| anyhow!("Failed to get listen address"))?;

    // Create second swarm (reader)
    let temp_dir_2 = get_test_temp_dir_str(Some("reader"));
    let mut swarm2 = generate_swarm(&temp_dir_2)?;
    swarm2.listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse()?)?;
    swarm2
        .behaviour_mut()
        .kad
        .set_mode(Some(libp2p::kad::Mode::Client));

    // Add the first swarm's address to the second swarm's address book
    swarm2
        .behaviour_mut()
        .kad
        .add_address(&peer1_id, listen_addr.clone());

    // Try to dial the first swarm
    swarm2.dial(listen_addr.clone())?;

    Ok((swarm1, swarm2, listen_addr, peer1_id))
}

async fn record_writer(
    mut swarm: libp2p::Swarm<netabase::network::behaviour::NetabaseBehaviour>,
    key: libp2p::kad::RecordKey,
    values: Vec<String>,
    mut command_receiver: tokio::sync::mpsc::Receiver<SwarmAction>,
) -> Result<()> {
    info!("Starting record writer");

    // Put all records first
    for (i, value) in values.iter().enumerate() {
        let record = libp2p::kad::Record::new(key.clone(), value.as_bytes().to_vec());
        let query_id = swarm
            .behaviour_mut()
            .kad
            .put_record(record, libp2p::kad::Quorum::One)?;
        info!("Writer Put Record {}: QueryId({:?})", i, query_id);
    }

    // Now handle events
    let mut put_confirmations = 0;
    let expected_puts = values.len();

    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                match event {
                    libp2p::swarm::SwarmEvent::Behaviour(
                        netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                            libp2p::kad::Event::OutboundQueryProgressed { result, .. }
                        )
                    ) => {
                        match result {
                            QueryResult::PutRecord(Ok(_)) => {
                                put_confirmations += 1;
                                info!("Writer: Put confirmed ({}/{})", put_confirmations, expected_puts);
                                if put_confirmations >= expected_puts {
                                    info!("Writer: All records stored successfully");
                                    return Ok(());
                                }
                            }
                            QueryResult::PutRecord(Err(e)) => {
                                info!("Writer: Put failed: {:?}", e);
                            }
                            _ => {}
                        }
                    }
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("Writer: Connected to peer: {}", peer_id);
                    }
                    libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Writer: Listening on {}", address);
                    }
                    _ => {}
                }
            }
            command = command_receiver.recv() => {
                match command {
                    Some(SwarmAction::EndLoop) => {
                        info!("Writer: Received end command");
                        break;
                    }
                    Some(cmd) => {
                        info!("Writer: Received command: {:?}", cmd);
                    }
                    None => {
                        info!("Writer: Command channel closed");
                        break;
                    }
                }
            }
            _ = sleep(Duration::from_secs(30)) => {
                info!("Writer: Timeout waiting for put confirmations");
                break;
            }
        }
    }

    Ok(())
}

async fn record_reader(
    mut swarm: libp2p::Swarm<netabase::network::behaviour::NetabaseBehaviour>,
    key: libp2p::kad::RecordKey,
    _expected_records: usize,
    mut command_receiver: tokio::sync::mpsc::Receiver<SwarmAction>,
) -> Result<Vec<String>> {
    info!("Starting record reader");

    // Wait a bit for connections to establish
    sleep(Duration::from_secs(2)).await;

    let mut found_records = Vec::new();
    let mut query_attempts = 0;
    let max_attempts = 10;

    loop {
        // Try to get the record
        let query_id = swarm.behaviour_mut().kad.get_record(key.clone());
        query_attempts += 1;
        info!(
            "Reader: Attempting to get record (attempt {}/{}): {:?}",
            query_attempts, max_attempts, query_id
        );

        let mut got_response = false;
        let query_timeout = timeout(Duration::from_secs(5), async {
            loop {
                tokio::select! {
                    event = swarm.select_next_some() => {
                        match event {
                            libp2p::swarm::SwarmEvent::Behaviour(
                                netabase::network::behaviour::NetabaseBehaviourEvent::Kad(
                                    libp2p::kad::Event::OutboundQueryProgressed { result, .. }
                                )
                            ) => {
                                match result {
                                    QueryResult::GetRecord(Ok(libp2p::kad::GetRecordOk::FoundRecord(peer_record))) => {
                                        let value = String::from_utf8_lossy(&peer_record.record.value).to_string();
                                        info!("Reader: Found record: {}", value);
                                        found_records.push(value);
                                        got_response = true;
                                        break;
                                    }
                                    QueryResult::GetRecord(Ok(libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. })) => {
                                        info!("Reader: Get finished with no additional record");
                                        got_response = true;
                                        break;
                                    }
                                    QueryResult::GetRecord(Err(e)) => {
                                        info!("Reader: Get record failed: {:?}", e);
                                        got_response = true;
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                                info!("Reader: Connected to peer: {}", peer_id);
                            }
                            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                                info!("Reader: Listening on {}", address);
                            }
                            _ => {}
                        }
                    }
                    command = command_receiver.recv() => {
                        match command {
                            Some(SwarmAction::EndLoop) => {
                                info!("Reader: Received end command");
                                break;
                            }
                            Some(cmd) => {
                                info!("Reader: Received command: {:?}", cmd);
                            }
                            None => {
                                info!("Reader: Command channel closed");
                                break;
                            }
                        }
                    }
                }
            }
        });

        match query_timeout.await {
            Ok(_) => {
                if !found_records.is_empty() {
                    info!("Reader: Successfully found {} records", found_records.len());
                    return Ok(found_records);
                }
            }
            Err(_) => {
                info!("Reader: Query timeout on attempt {}", query_attempts);
            }
        }

        if query_attempts >= max_attempts {
            info!(
                "Reader: Max attempts reached, found {} records",
                found_records.len()
            );
            break;
        }

        // Wait before next attempt
        sleep(Duration::from_secs(1)).await;
    }

    Ok(found_records)
}

#[tokio::test]
async fn test_connected_writer_reader() {
    init_logging();

    let key = RecordKey::new(&"test_key");
    let test_values = vec![
        "Hello World".to_string(),
        "Test Record".to_string(),
        "Another Value".to_string(),
    ];

    // Setup connected swarms
    let (writer_swarm, reader_swarm, writer_addr, writer_peer_id) = setup_connected_swarms()
        .await
        .expect("Failed to setup swarms");

    info!("Writer PeerID: {}", writer_peer_id);
    info!("Writer Address: {}", writer_addr);

    // Create command channels
    let (writer_cmd_tx, writer_cmd_rx) = tokio::sync::mpsc::channel(10);
    let (reader_cmd_tx, reader_cmd_rx) = tokio::sync::mpsc::channel(10);

    // Start writer task
    let writer_values = test_values.clone();
    let writer_key = key.clone();
    let writer_task = tokio::spawn(async move {
        record_writer(writer_swarm, writer_key, writer_values, writer_cmd_rx).await
    });

    // Give writer time to start and put records
    sleep(Duration::from_secs(3)).await;

    // Start reader task
    let reader_key = key.clone();
    let expected_count = test_values.len();
    let reader_task = tokio::spawn(async move {
        record_reader(reader_swarm, reader_key, expected_count, reader_cmd_rx).await
    });

    // Wait for both tasks with timeout
    let writer_result = timeout(Duration::from_secs(60), writer_task).await;
    let reader_result = timeout(Duration::from_secs(60), reader_task).await;

    // Send end commands
    let _ = writer_cmd_tx.send(SwarmAction::EndLoop).await;
    let _ = reader_cmd_tx.send(SwarmAction::EndLoop).await;

    match writer_result {
        Ok(Ok(Ok(()))) => info!("Writer completed successfully"),
        Ok(Ok(Err(e))) => info!("Writer failed: {:?}", e),
        Ok(Err(e)) => info!("Writer task panicked: {:?}", e),
        Err(_) => info!("Writer timed out"),
    }

    match reader_result {
        Ok(Ok(Ok(records))) => {
            info!(
                "Reader completed successfully, found {} records",
                records.len()
            );
            for (i, record) in records.iter().enumerate() {
                info!("Record {}: {}", i, record);
            }
            assert!(!records.is_empty(), "Should have found at least one record");
        }
        Ok(Ok(Err(e))) => {
            info!("Reader failed: {:?}", e);
            panic!("Reader failed: {:?}", e);
        }
        Ok(Err(e)) => {
            info!("Reader task panicked: {:?}", e);
            panic!("Reader task panicked: {:?}", e);
        }
        Err(_) => {
            info!("Reader timed out");
            panic!("Reader timed out");
        }
    }
}

#[tokio::test]
async fn test_simple_put_get() {
    init_logging();

    let key = RecordKey::new(&"simple_key");
    let value = "Simple Value".to_string();

    // Create a single swarm for local testing
    let temp_dir = get_test_temp_dir_str(Some("simple"));
    let mut swarm = generate_swarm(&temp_dir).expect("Failed to generate swarm");

    // Test local storage functionality directly instead of DHT
    // since single-node DHT operations fail due to quorum requirements
    let record = libp2p::kad::Record::new(key.clone(), value.as_bytes().to_vec());

    // Access the store directly to test local storage
    let store = swarm.behaviour_mut().kad.store_mut();

    // Put record into local store
    let put_result = store.put(record.clone());
    match put_result {
        Ok(_) => info!("Successfully stored record locally"),
        Err(e) => panic!("Failed to store record locally: {:?}", e),
    }

    // Get record from local store
    let get_result = store.get(&key);
    match get_result {
        Some(found_record) => {
            let found_value = String::from_utf8_lossy(&found_record.value);
            info!("Found record: {}", found_value);
            assert_eq!(
                found_value, value,
                "Retrieved value should match stored value"
            );
            info!("Local storage test completed successfully");
        }
        None => panic!("Failed to retrieve record from local storage"),
    }
}
