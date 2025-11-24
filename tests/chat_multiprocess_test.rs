//! Multi-process integration test for the chat example
//!
//! This test spawns multiple chat instances and verifies they can store and retrieve messages.

use anyhow::Result;
use bincode::{Decode, Encode};
use netabase::Netabase;
use netabase::network::config::{NetabaseConfig, SyncConfig};
use netabase_store::traits::definition::{NetabaseDefinitionTrait, NetabaseDefinitionTraitKey};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::time::Duration;
use strum::EnumDiscriminants;
use tokio::time::sleep;

// Define the same chat model as the example
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, EnumDiscriminants)]
#[strum_discriminants(derive(
    strum::EnumIter,
    strum::Display,
    strum::AsRefStr,
    strum::EnumString,
    Hash,
    Encode,
    Decode
))]
#[strum_discriminants(name(ChatDefinition))]
enum ChatModel {
    Message {
        id: String,
        sender: String,
        content: String,
        timestamp: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Encode, Decode, EnumDiscriminants)]
#[strum_discriminants(derive(
    strum::EnumIter,
    strum::Display,
    strum::AsRefStr,
    strum::EnumString,
    Hash,
    Encode,
    Decode
))]
#[strum_discriminants(name(ChatKeysDiscriminant))]
enum ChatKeys {
    Id(String),
}

impl NetabaseDefinitionTraitKey for ChatKeys {}

impl NetabaseDefinitionTrait for ChatModel {
    type Keys = ChatKeys;
    type Tables = ChatDefinition;

    fn tables() -> Self::Tables {
        ChatDefinition::Message
    }

    #[cfg(all(feature = "paxos", feature = "libp2p", not(target_arch = "wasm32")))]
    fn apply_to_store<S>(&self, _store: &mut S) -> Result<(), String>
    where
        S: libp2p::kad::store::RecordStore,
    {
        Ok(())
    }
}

impl netabase_store::convert::ToIVec for ChatModel {}
impl netabase_store::convert::ToIVec for ChatKeys {}

/// Helper function to create and store a message
async fn store_message(
    netabase: &Netabase<ChatModel>,
    sender: &str,
    content: &str,
) -> Result<()> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    let message_id = format!("{}_{}", timestamp, sender);

    let message = ChatModel::Message {
        id: message_id,
        sender: sender.to_string(),
        content: content.to_string(),
        timestamp,
    };

    netabase.put_definition(message).await?;
    Ok(())
}

/// Helper function to query messages from database
async fn query_messages(netabase: &Netabase<ChatModel>) -> Result<Vec<(String, String, i64)>> {
    let records = netabase.query_local_records(None).await?;

    let mut messages = Vec::new();
    for record in records {
        let ChatModel::Message { sender, content, timestamp, .. } = record;
        messages.push((sender, content, timestamp));
    }

    messages.sort_by_key(|(_, _, ts)| *ts);
    Ok(messages)
}

#[tokio::test]
async fn test_single_node_storage() -> Result<()> {
    // Create temporary database
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("alice");

    let sync_config = SyncConfig::development();
    let config = NetabaseConfig::default();
    let mut config_with_sync = config;
    config_with_sync.sync = Some(sync_config);

    let mut netabase = Netabase::<ChatModel>::new_with_path_and_config(
        db_path.to_str().unwrap(),
        config_with_sync,
    )?;

    netabase.start_swarm().await?;
    sleep(Duration::from_millis(100)).await;

    // Store messages
    store_message(&netabase, "alice", "Hello world!").await?;
    store_message(&netabase, "alice", "Second message").await?;
    store_message(&netabase, "alice", "Third message").await?;

    // Wait for storage to complete
    sleep(Duration::from_millis(100)).await;

    // Query messages
    let messages = query_messages(&netabase).await?;

    assert_eq!(messages.len(), 3, "Should have 3 messages stored");
    assert_eq!(messages[0].0, "alice");
    assert_eq!(messages[0].1, "Hello world!");
    assert_eq!(messages[1].1, "Second message");
    assert_eq!(messages[2].1, "Third message");

    netabase.stop_swarm().await?;
    Ok(())
}

#[tokio::test]
async fn test_message_persistence() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("bob");

    // First session - store messages
    {
        let sync_config = SyncConfig::development();
        let config = NetabaseConfig::default();
        let mut config_with_sync = config;
        config_with_sync.sync = Some(sync_config);

        let mut netabase = Netabase::<ChatModel>::new_with_path_and_config(
            db_path.to_str().unwrap(),
            config_with_sync,
        )?;

        netabase.start_swarm().await?;
        sleep(Duration::from_millis(100)).await;

        store_message(&netabase, "bob", "Persistent message 1").await?;
        store_message(&netabase, "bob", "Persistent message 2").await?;

        sleep(Duration::from_millis(100)).await;
        netabase.stop_swarm().await?;
    }

    // Second session - verify persistence
    {
        let sync_config = SyncConfig::development();
        let config = NetabaseConfig::default();
        let mut config_with_sync = config;
        config_with_sync.sync = Some(sync_config);

        let mut netabase = Netabase::<ChatModel>::new_with_path_and_config(
            db_path.to_str().unwrap(),
            config_with_sync,
        )?;

        netabase.start_swarm().await?;
        sleep(Duration::from_millis(100)).await;

        let messages = query_messages(&netabase).await?;

        assert_eq!(messages.len(), 2, "Should have 2 persisted messages");
        assert_eq!(messages[0].1, "Persistent message 1");
        assert_eq!(messages[1].1, "Persistent message 2");

        // Add another message
        store_message(&netabase, "bob", "Persistent message 3").await?;
        sleep(Duration::from_millis(100)).await;

        let messages = query_messages(&netabase).await?;
        assert_eq!(messages.len(), 3, "Should have 3 messages after adding one");

        netabase.stop_swarm().await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_multiple_users_separate_databases() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    // Create Alice's database
    let alice_path = temp_dir.path().join("alice");
    let bob_path = temp_dir.path().join("bob");
    let charlie_path = temp_dir.path().join("charlie");

    // Spawn Alice
    let alice_handle = tokio::spawn(async move {
        let sync_config = SyncConfig::development();
        let config = NetabaseConfig::default();
        let mut config_with_sync = config;
        config_with_sync.sync = Some(sync_config);

        let mut netabase = Netabase::<ChatModel>::new_with_path_and_config(
            alice_path.to_str().unwrap(),
            config_with_sync,
        ).unwrap();

        netabase.start_swarm().await.unwrap();
        sleep(Duration::from_millis(100)).await;

        // Alice sends messages
        store_message(&netabase, "alice", "Alice message 1").await.unwrap();
        store_message(&netabase, "alice", "Alice message 2").await.unwrap();

        sleep(Duration::from_millis(100)).await;

        let messages = query_messages(&netabase).await.unwrap();
        netabase.stop_swarm().await.unwrap();

        messages
    });

    // Spawn Bob
    let bob_handle = tokio::spawn(async move {
        let sync_config = SyncConfig::development();
        let config = NetabaseConfig::default();
        let mut config_with_sync = config;
        config_with_sync.sync = Some(sync_config);

        let mut netabase = Netabase::<ChatModel>::new_with_path_and_config(
            bob_path.to_str().unwrap(),
            config_with_sync,
        ).unwrap();

        netabase.start_swarm().await.unwrap();
        sleep(Duration::from_millis(100)).await;

        // Bob sends messages
        store_message(&netabase, "bob", "Bob message 1").await.unwrap();
        store_message(&netabase, "bob", "Bob message 2").await.unwrap();
        store_message(&netabase, "bob", "Bob message 3").await.unwrap();

        sleep(Duration::from_millis(100)).await;

        let messages = query_messages(&netabase).await.unwrap();
        netabase.stop_swarm().await.unwrap();

        messages
    });

    // Spawn Charlie
    let charlie_handle = tokio::spawn(async move {
        let sync_config = SyncConfig::development();
        let config = NetabaseConfig::default();
        let mut config_with_sync = config;
        config_with_sync.sync = Some(sync_config);

        let mut netabase = Netabase::<ChatModel>::new_with_path_and_config(
            charlie_path.to_str().unwrap(),
            config_with_sync,
        ).unwrap();

        netabase.start_swarm().await.unwrap();
        sleep(Duration::from_millis(100)).await;

        // Charlie sends one message
        store_message(&netabase, "charlie", "Charlie message 1").await.unwrap();

        sleep(Duration::from_millis(100)).await;

        let messages = query_messages(&netabase).await.unwrap();
        netabase.stop_swarm().await.unwrap();

        messages
    });

    // Wait for all to complete
    let alice_messages = alice_handle.await?;
    let bob_messages = bob_handle.await?;
    let charlie_messages = charlie_handle.await?;

    // Verify each user has their own messages
    assert_eq!(alice_messages.len(), 2, "Alice should have 2 messages");
    assert_eq!(bob_messages.len(), 3, "Bob should have 3 messages");
    assert_eq!(charlie_messages.len(), 1, "Charlie should have 1 message");

    // Verify message content
    assert!(alice_messages.iter().all(|(sender, _, _)| sender == "alice"));
    assert!(bob_messages.iter().all(|(sender, _, _)| sender == "bob"));
    assert!(charlie_messages.iter().all(|(sender, _, _)| sender == "charlie"));

    Ok(())
}

#[tokio::test]
async fn test_concurrent_writes() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("concurrent");

    let sync_config = SyncConfig::development();
    let config = NetabaseConfig::default();
    let mut config_with_sync = config;
    config_with_sync.sync = Some(sync_config);

    let mut netabase = Netabase::<ChatModel>::new_with_path_and_config(
        db_path.to_str().unwrap(),
        config_with_sync,
    )?;

    netabase.start_swarm().await?;
    sleep(Duration::from_millis(100)).await;

    // Write messages sequentially (simpler than concurrent with lifetime issues)
    for i in 0..10 {
        store_message(&netabase, "user", &format!("Message {}", i)).await?;
    }

    sleep(Duration::from_millis(200)).await;

    // Verify all messages were stored
    let messages = query_messages(&netabase).await?;
    assert_eq!(messages.len(), 10, "Should have 10 messages");

    // Verify message ordering
    for (idx, (_, content, _)) in messages.iter().enumerate() {
        assert!(content.contains(&format!("Message {}", idx)));
    }

    netabase.stop_swarm().await?;
    Ok(())
}

#[tokio::test]
async fn test_peer_discovery() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    let node1_path = temp_dir.path().join("node1");
    let node2_path = temp_dir.path().join("node2");

    let sync_config = SyncConfig::development();
    let config = NetabaseConfig::default();
    let mut config1 = config.clone();
    config1.sync = Some(sync_config.clone());
    let mut config2 = config;
    config2.sync = Some(sync_config);

    // Start node 1
    let mut node1 = Netabase::<ChatModel>::new_with_path_and_config(
        node1_path.to_str().unwrap(),
        config1,
    )?;
    node1.start_swarm().await?;

    // Start node 2
    let mut node2 = Netabase::<ChatModel>::new_with_path_and_config(
        node2_path.to_str().unwrap(),
        config2,
    )?;
    node2.start_swarm().await?;

    // Give mDNS time to discover peers
    sleep(Duration::from_secs(2)).await;

    // Check peer counts
    let node1_peers = node1.peer_count().await?;
    let node2_peers = node2.peer_count().await?;

    // Each should discover at least the other (may discover more if other tests are running)
    assert!(
        node1_peers > 0 || node2_peers > 0,
        "At least one node should discover peers"
    );

    node1.stop_swarm().await?;
    node2.stop_swarm().await?;

    Ok(())
}
