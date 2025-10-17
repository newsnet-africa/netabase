use bincode::{Decode, Encode};
use libp2p::PeerId;
use log::{debug, error, info, warn};
use netabase::Netabase;
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::NetabaseModel as NetabaseModelTrait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Once;
use std::time::{Duration, Instant};
use tokio::time::timeout;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .is_test(true)
            .init();
    });
}

// Schema for interprocess messaging tests
#[netabase_schema_module(InterprocessMessagingSchema, InterprocessMessagingKeys)]
mod interprocess_messaging_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(InterprocessMessageKey)]
    pub struct InterprocessMessage {
        #[key]
        pub id: String,
        pub content: String,
        pub sender_node: String,
        pub receiver_node: String,
        pub timestamp: u64,
        pub message_type: MessageType,
        pub sequence_number: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq)]
    pub enum MessageType {
        Handshake,
        Data,
        Acknowledgment,
        Heartbeat,
        Shutdown,
    }
}

use interprocess_messaging_schema::{
    InterprocessMessage, InterprocessMessagingSchema, MessageType,
};

fn create_message(
    id: u64,
    content: &str,
    sender: &str,
    receiver: &str,
    msg_type: MessageType,
    sequence: u32,
) -> InterprocessMessage {
    InterprocessMessage {
        id: format!("msg_{}_{}_{}_{}", sender, receiver, sequence, id),
        content: content.to_string(),
        sender_node: sender.to_string(),
        receiver_node: receiver.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        message_type: msg_type,
        sequence_number: sequence,
    }
}

async fn wait_for_mdns_connection(
    netabase: &Netabase<InterprocessMessagingSchema>,
    node_name: &str,
    timeout_duration: Duration,
) -> Result<PeerId, String> {
    info!("🔍 [{}] Waiting for mDNS peer discovery...", node_name);

    let mut events = netabase.subscribe_to_broadcasts();
    let start_time = Instant::now();
    let mut discovered_peers = HashSet::new();

    while start_time.elapsed() < timeout_duration {
        match tokio::time::timeout(Duration::from_millis(100), events.recv()).await {
            Ok(Ok(event)) => {
                match &event.0 {
                    libp2p::swarm::SwarmEvent::Behaviour(behaviour_event) => {
                        match behaviour_event {
                            netabase::network::behaviour::NetabaseBehaviourEvent::Mdns(
                                mdns_event,
                            ) => match mdns_event {
                                libp2p::mdns::Event::Discovered(peers) => {
                                    for (peer_id, addr) in peers {
                                        info!(
                                            "🔍 [{}] mDNS discovered peer: {} at {}",
                                            node_name, peer_id, addr
                                        );
                                        discovered_peers.insert(peer_id.clone());
                                    }
                                }
                                libp2p::mdns::Event::Expired(peers) => {
                                    for (peer_id, _addr) in peers {
                                        warn!("⏰ [{}] mDNS peer expired: {}", node_name, peer_id);
                                        discovered_peers.remove(&peer_id);
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!(
                            "🤝 [{}] Connection established with: {}",
                            node_name, peer_id
                        );
                        // Return the first peer we successfully connect to
                        return Ok(peer_id.clone());
                    }
                    libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                        warn!(
                            "💔 [{}] Connection closed with: {} (cause: {:?})",
                            node_name, peer_id, cause
                        );
                    }
                    _ => {}
                }
            }
            Ok(Err(e)) => {
                error!("❌ [{}] Event stream error: {:?}", node_name, e);
                break;
            }
            Err(_) => {
                // Timeout waiting for events, continue
            }
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    Err(format!(
        "No mDNS connections established by {} within {:?}",
        node_name, timeout_duration
    ))
}

async fn wait_for_peer_disconnection(
    netabase: &Netabase<InterprocessMessagingSchema>,
    node_name: &str,
    target_peer: PeerId,
    timeout_duration: Duration,
) -> Result<(), String> {
    info!(
        "👀 [{}] Waiting for peer {} to disconnect...",
        node_name, target_peer
    );

    let mut events = netabase.subscribe_to_broadcasts();
    let start_time = Instant::now();

    while start_time.elapsed() < timeout_duration {
        match tokio::time::timeout(Duration::from_millis(100), events.recv()).await {
            Ok(Ok(event)) => match &event.0 {
                libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    if peer_id == &target_peer {
                        info!(
                            "✅ [{}] Target peer {} disconnected (cause: {:?})",
                            node_name, peer_id, cause
                        );
                        return Ok(());
                    }
                }
                _ => {}
            },
            Ok(Err(e)) => {
                error!("❌ [{}] Event stream error: {:?}", node_name, e);
                break;
            }
            Err(_) => {
                // Timeout waiting for events, continue
            }
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    Err(format!(
        "Peer {} did not disconnect within {:?}",
        target_peer, timeout_duration
    ))
}

async fn send_messages_to_peer(
    netabase: &Netabase<InterprocessMessagingSchema>,
    sender_name: &str,
    receiver_name: &str,
    num_messages: u32,
) -> Result<Vec<InterprocessMessage>, String> {
    info!(
        "📤 [{}] Sending {} messages to {}...",
        sender_name, num_messages, receiver_name
    );

    let mut sent_messages = Vec::new();

    // Send handshake first
    let handshake = create_message(
        0,
        "HANDSHAKE",
        sender_name,
        receiver_name,
        MessageType::Handshake,
        0,
    );

    match timeout(
        Duration::from_secs(5),
        netabase.put_record(handshake.clone()),
    )
    .await
    {
        Ok(Ok(_)) => {
            info!("🤝 [{}] Handshake sent successfully", sender_name);
            sent_messages.push(handshake);
        }
        Ok(Err(e)) => {
            error!("❌ [{}] Failed to send handshake: {:?}", sender_name, e);
            return Err(format!("Failed to send handshake: {:?}", e));
        }
        Err(_) => {
            error!("⏰ [{}] Handshake send timeout", sender_name);
            return Err("Handshake send timeout".to_string());
        }
    }

    // Send data messages
    for i in 1..=num_messages {
        let message = create_message(
            i as u64,
            &format!("Data message {} from {}", i, sender_name),
            sender_name,
            receiver_name,
            MessageType::Data,
            i,
        );

        match timeout(Duration::from_secs(5), netabase.put_record(message.clone())).await {
            Ok(Ok(_)) => {
                info!("📤 [{}] Message {} sent successfully", sender_name, i);
                sent_messages.push(message);

                // Small delay between messages to avoid overwhelming
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            Ok(Err(e)) => {
                error!("❌ [{}] Failed to send message {}: {:?}", sender_name, i, e);
                return Err(format!("Failed to send message {}: {:?}", i, e));
            }
            Err(_) => {
                error!("⏰ [{}] Message {} send timeout", sender_name, i);
                return Err(format!("Message {} send timeout", i));
            }
        }
    }

    // Send shutdown message
    let shutdown = create_message(
        (num_messages + 1) as u64,
        "SHUTDOWN",
        sender_name,
        receiver_name,
        MessageType::Shutdown,
        num_messages + 1,
    );

    match timeout(
        Duration::from_secs(5),
        netabase.put_record(shutdown.clone()),
    )
    .await
    {
        Ok(Ok(_)) => {
            info!("🛑 [{}] Shutdown message sent successfully", sender_name);
            sent_messages.push(shutdown);
        }
        Ok(Err(e)) => {
            error!("❌ [{}] Failed to send shutdown: {:?}", sender_name, e);
            return Err(format!("Failed to send shutdown: {:?}", e));
        }
        Err(_) => {
            error!("⏰ [{}] Shutdown send timeout", sender_name);
            return Err("Shutdown send timeout".to_string());
        }
    }

    Ok(sent_messages)
}

async fn receive_and_verify_messages(
    netabase: &Netabase<InterprocessMessagingSchema>,
    receiver_name: &str,
    sender_name: &str,
    expected_count: u32,
    timeout_duration: Duration,
) -> Result<Vec<InterprocessMessage>, String> {
    info!(
        "📥 [{}] Waiting to receive {} messages from {}...",
        receiver_name, expected_count, sender_name
    );

    let mut received_messages = Vec::new();
    let start_time = Instant::now();
    let mut shutdown_received = false;

    while start_time.elapsed() < timeout_duration && !shutdown_received {
        // Try to retrieve messages by checking different sequence numbers
        for seq in 0..=expected_count + 1 {
            // Create the message we expect to find
            let expected_message = create_message(
                seq as u64,
                "dummy", // content doesn't matter for key generation
                sender_name,
                receiver_name,
                MessageType::Data, // type doesn't matter for key generation
                seq,
            );

            match timeout(
                Duration::from_secs(2),
                netabase.get_record(expected_message.key()),
            )
            .await
            {
                Ok(Ok(libp2p::kad::QueryResult::GetRecord(Ok(get_record_ok)))) => {
                    match get_record_ok {
                        libp2p::kad::GetRecordOk::FoundRecord(peer_record) => {
                            match bincode::decode_from_slice::<InterprocessMessage, _>(
                                &peer_record.record.value,
                                bincode::config::standard(),
                            ) {
                                Ok((message, _)) => {
                                    if !received_messages.iter().any(|m: &InterprocessMessage| {
                                        m.sequence_number == message.sequence_number
                                    }) {
                                        info!(
                                            "📥 [{}] Received message {}: {} (type: {:?})",
                                            receiver_name,
                                            message.sequence_number,
                                            message.content,
                                            message.message_type
                                        );

                                        if matches!(message.message_type, MessageType::Shutdown) {
                                            shutdown_received = true;
                                        }

                                        received_messages.push(message);
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "❌ [{}] Failed to deserialize message: {:?}",
                                        receiver_name, e
                                    );
                                }
                            }
                        }
                        libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. } => {
                            // Query finished but no record found
                            debug!(
                                "🔍 [{}] Query finished for seq {} but no record found",
                                receiver_name, seq
                            );
                        }
                    }
                }
                Ok(Ok(_)) => {
                    // Message not found yet or other QueryResult type, continue
                }
                Ok(Err(e)) => {
                    debug!(
                        "🔍 [{}] Get record error for seq {}: {:?}",
                        receiver_name, seq, e
                    );
                }
                Err(_) => {
                    debug!("⏰ [{}] Get record timeout for seq {}", receiver_name, seq);
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Sort messages by sequence number
    received_messages.sort_by_key(|m| m.sequence_number);

    info!(
        "✅ [{}] Received {} total messages from {}",
        receiver_name,
        received_messages.len(),
        sender_name
    );

    for msg in &received_messages {
        info!(
            "  📋 [{}] Seq {}: {} (type: {:?})",
            receiver_name, msg.sequence_number, msg.content, msg.message_type
        );
    }

    Ok(received_messages)
}

/// SENDER PROCESS TEST
/// Run this test in one terminal/process
#[tokio::test]
#[cfg(feature = "memory")]
async fn test_interprocess_sender() {
    init_logger();

    info!("🚀 Starting SENDER process for interprocess messaging test");

    // Create sender node
    let mut sender_node = Netabase::<InterprocessMessagingSchema>::new_with_memory()
        .expect("Failed to create sender node");

    // Start swarm
    info!("🌐 Starting sender swarm...");
    sender_node
        .start_swarm()
        .await
        .expect("Failed to start sender swarm");

    // Wait a bit for the swarm to initialize
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Wait for mDNS discovery and connection to receiver
    info!("🔍 Sender waiting for receiver peer discovery...");
    let receiver_peer = wait_for_mdns_connection(&sender_node, "Sender", Duration::from_secs(30))
        .await
        .expect("Sender failed to discover receiver peer");

    info!("✅ Sender discovered receiver peer: {}", receiver_peer);

    // Give time for connection to stabilize
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Send messages to receiver
    const MESSAGE_COUNT: u32 = 5;
    info!(
        "📤 Sender starting to send {} messages to receiver",
        MESSAGE_COUNT
    );

    let sent_messages = send_messages_to_peer(&sender_node, "sender", "receiver", MESSAGE_COUNT)
        .await
        .expect("Sender failed to send messages");

    info!(
        "✅ Sender successfully sent {} messages",
        sent_messages.len()
    );

    // Wait a bit to ensure all messages are propagated
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Shutdown sender
    info!("🛑 Sender shutting down...");
    drop(sender_node);

    info!("🎉 SENDER process completed successfully!");
}

/// RECEIVER PROCESS TEST
/// Run this test in another terminal/process
#[tokio::test]
#[cfg(feature = "memory")]
async fn test_interprocess_receiver() {
    init_logger();

    info!("🚀 Starting RECEIVER process for interprocess messaging test");

    // Create receiver node
    let mut receiver_node = Netabase::<InterprocessMessagingSchema>::new_with_memory()
        .expect("Failed to create receiver node");

    // Start swarm
    info!("🌐 Starting receiver swarm...");
    receiver_node
        .start_swarm()
        .await
        .expect("Failed to start receiver swarm");

    // Wait a bit for the swarm to initialize
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Wait for mDNS discovery and connection to sender
    info!("🔍 Receiver waiting for sender peer discovery...");
    let sender_peer = wait_for_mdns_connection(&receiver_node, "Receiver", Duration::from_secs(30))
        .await
        .expect("Receiver failed to discover sender peer");

    info!("✅ Receiver discovered sender peer: {}", sender_peer);

    // Wait for sender to disconnect (indicating it finished sending)
    info!("👀 Receiver waiting for sender to finish and disconnect...");
    wait_for_peer_disconnection(
        &receiver_node,
        "Receiver",
        sender_peer,
        Duration::from_secs(60),
    )
    .await
    .expect("Sender did not disconnect properly");

    info!("✅ Sender has disconnected, now checking retained messages...");

    // Check if data was retained in memory after sender disconnection
    const MESSAGE_COUNT: u32 = 5;
    info!("📥 Receiver checking for retained messages in memory...");

    let received_messages = receive_and_verify_messages(
        &receiver_node,
        "receiver",
        "sender",
        MESSAGE_COUNT,
        Duration::from_secs(30),
    )
    .await
    .expect("Receiver failed to retrieve messages");

    // Verify message integrity
    info!("🔍 Verifying message integrity...");

    // Should have handshake + data messages + shutdown
    let expected_total = MESSAGE_COUNT + 2; // +1 for handshake, +1 for shutdown
    assert_eq!(
        received_messages.len(),
        expected_total as usize,
        "Expected {} messages, but received {}",
        expected_total,
        received_messages.len()
    );

    // Verify message types and sequence
    assert!(matches!(
        received_messages[0].message_type,
        MessageType::Handshake
    ));
    assert_eq!(received_messages[0].sequence_number, 0);

    for i in 1..=MESSAGE_COUNT {
        assert!(matches!(
            received_messages[i as usize].message_type,
            MessageType::Data
        ));
        assert_eq!(received_messages[i as usize].sequence_number, i);
        assert_eq!(received_messages[i as usize].sender_node, "sender");
        assert_eq!(received_messages[i as usize].receiver_node, "receiver");
    }

    let last_idx = (MESSAGE_COUNT + 1) as usize;
    assert!(matches!(
        received_messages[last_idx].message_type,
        MessageType::Shutdown
    ));
    assert_eq!(
        received_messages[last_idx].sequence_number,
        MESSAGE_COUNT + 1
    );

    info!(
        "✅ All {} messages verified successfully!",
        received_messages.len()
    );
    info!("✅ Data was successfully retained in memory after sender disconnection!");

    // Receiver shuts down
    info!("🛑 Receiver shutting down...");
    drop(receiver_node);

    info!("🎉 RECEIVER process completed successfully!");
}

/// COMBINED TEST - for testing both in the same process (optional)
#[tokio::test]
#[cfg(feature = "memory")]
async fn test_interprocess_messaging_combined() {
    init_logger();

    info!("🚀 Starting combined interprocess messaging test (both nodes in same process)");

    // Create both nodes
    let mut sender_node = Netabase::<InterprocessMessagingSchema>::new_with_memory()
        .expect("Failed to create sender node");
    let mut receiver_node = Netabase::<InterprocessMessagingSchema>::new_with_memory()
        .expect("Failed to create receiver node");

    // Start both swarms
    info!("🌐 Starting swarms...");
    sender_node
        .start_swarm()
        .await
        .expect("Failed to start sender swarm");
    tokio::time::sleep(Duration::from_millis(500)).await;

    receiver_node
        .start_swarm()
        .await
        .expect("Failed to start receiver swarm");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Wait for mDNS discovery and connection
    info!("🔍 Waiting for peer discovery...");

    let sender_discovered_peer =
        wait_for_mdns_connection(&sender_node, "Sender", Duration::from_secs(10))
            .await
            .expect("Sender failed to discover peers");

    let receiver_discovered_peer =
        wait_for_mdns_connection(&receiver_node, "Receiver", Duration::from_secs(10))
            .await
            .expect("Receiver failed to discover peers");

    info!(
        "✅ Peers discovered: Sender -> {}, Receiver -> {}",
        sender_discovered_peer, receiver_discovered_peer
    );

    // Sender sends messages
    const MESSAGE_COUNT: u32 = 5;
    info!("📤 Sender sending {} messages to receiver", MESSAGE_COUNT);

    let _sent_messages = send_messages_to_peer(&sender_node, "sender", "receiver", MESSAGE_COUNT)
        .await
        .expect("Sender failed to send messages");

    // Give time for messages to propagate
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Sender shuts down
    info!("🛑 Sender shutting down...");
    drop(sender_node);

    // Receiver waits for sender disconnection
    info!("👀 Receiver waiting for sender disconnection...");
    wait_for_peer_disconnection(
        &receiver_node,
        "Receiver",
        receiver_discovered_peer,
        Duration::from_secs(10),
    )
    .await
    .expect("Sender did not disconnect properly");

    // Receiver checks retained messages
    info!("📥 Receiver checking retained messages");
    let received_messages = receive_and_verify_messages(
        &receiver_node,
        "receiver",
        "sender",
        MESSAGE_COUNT,
        Duration::from_secs(15),
    )
    .await
    .expect("Receiver failed to receive messages");

    // Verify message integrity
    let expected_total = MESSAGE_COUNT + 2;
    assert_eq!(received_messages.len(), expected_total as usize);

    info!("✅ Combined test completed successfully!");

    // Cleanup
    drop(receiver_node);
}
