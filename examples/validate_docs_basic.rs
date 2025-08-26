//! Basic validation example for netabase documentation examples
//!
//! This example tests the basic functionality described in the netabase README,
//! including schema definition, key generation, and basic operations.

use bincode::{Decode, Encode};
use libp2p::identity::ed25519::Keypair;
use netabase::{Netabase, NetabaseConfig, NetabaseSchema};
use serde::{Deserialize, Serialize};

// Example from the basic documentation - User struct
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct User {
    #[key]
    id: u64,
    name: String,
    email: String,
}

// Example from the documentation - Message struct
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct Message {
    #[key]
    id: String,
    sender: String,
    recipient: String,
    content: String,
    timestamp: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 Testing basic netabase documentation examples...\n");

    // Initialize logging
    netabase::init_logging();

    // Test 1: Basic schema validation
    println!("📋 Test 1: Basic schema validation");
    test_basic_schemas().await?;

    // Test 2: Key generation
    println!("\n🔑 Test 2: Key generation");
    test_key_generation()?;

    // Test 3: Serialization roundtrip
    println!("\n🔄 Test 3: Serialization roundtrip");
    test_serialization_roundtrip()?;

    // Test 4: Network operations (basic)
    println!("\n🌐 Test 4: Basic network operations");
    test_network_operations().await?;

    println!("\n✅ All basic documentation examples validated successfully!");
    Ok(())
}

async fn test_basic_schemas() -> anyhow::Result<()> {
    // Create User instance as shown in documentation
    let user = User {
        id: 42,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    println!("   Created user: {:?}", user);
    println!("   User key: {}", user.key());

    // Create Message instance
    let message = Message {
        id: "msg_123".to_string(),
        sender: "alice@example.com".to_string(),
        recipient: "bob@example.com".to_string(),
        content: "Hello, world!".to_string(),
        timestamp: 1234567890,
    };

    println!("   Created message: {:?}", message);
    println!("   Message key: {}", message.key());

    Ok(())
}

fn test_key_generation() -> anyhow::Result<()> {
    let user = User {
        id: 42,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let message = Message {
        id: "test_message".to_string(),
        sender: "sender".to_string(),
        recipient: "recipient".to_string(),
        content: "content".to_string(),
        timestamp: 12345,
    };

    // Test key generation
    assert_eq!(user.key().to_string(), "42");
    assert_eq!(message.key().to_string(), "test_message");

    println!("   ✓ User key generation: {} -> {}", user.id, user.key());
    println!(
        "   ✓ Message key generation: {} -> {}",
        message.id,
        message.key()
    );

    Ok(())
}

fn test_serialization_roundtrip() -> anyhow::Result<()> {
    let original_user = User {
        id: 123,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    // Test bincode serialization (used internally by netabase)
    let serialized = bincode::encode_to_vec(&original_user, bincode::config::standard())?;
    let (deserialized_user, _): (User, usize) =
        bincode::decode_from_slice(&serialized, bincode::config::standard())?;

    assert_eq!(original_user, deserialized_user);
    println!("   ✓ User serialization roundtrip successful");

    // Test Record conversion (netabase internal format)
    let record: libp2p::kad::Record = original_user.clone().into();
    println!(
        "   ✓ Record key: {:?}",
        String::from_utf8_lossy(&record.key.to_vec())
    );
    println!("   ✓ Record value length: {} bytes", record.value.len());

    Ok(())
}

async fn test_network_operations() -> anyhow::Result<()> {
    // Create configuration
    let config = NetabaseConfig::default();

    // Generate keypair
    let keypair = Keypair::generate();

    // Create netabase instance
    let mut netabase = Netabase::try_new(config, "docs-basic-test").await?;

    // Start the network
    netabase.start_swarm().await?;
    println!("   ✓ Network started successfully");

    // Wait a moment for network initialization
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Create test data
    let user = User {
        id: 42,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Test put operation
    let put_result = netabase
        .put(
            user.clone(),
            None::<std::vec::IntoIter<libp2p::PeerId>>,
            libp2p::kad::Quorum::One,
        )
        .await?;
    println!("   ✓ Put operation successful: {:?}", put_result);

    // Test get operation
    let get_result = netabase.get(user.key()).await?;
    let record_count = match get_result {
        libp2p::kad::GetRecordOk::FoundRecord(_) => 1,
        libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. } => 0,
    };
    println!(
        "   ✓ Get operation successful: {} records found",
        record_count
    );

    Ok(())
}
