//! Validation example for all supported key types in netabase
//!
//! This example tests all the key types that are currently supported by the
//! NetabaseSchema derive macro, as documented in the README.

use bincode::{Decode, Encode};
use libp2p::identity::ed25519::Keypair;
use netabase::{Netabase, NetabaseConfig, NetabaseSchema};
use serde::{Deserialize, Serialize};

// Test u8 key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct U8Entity {
    #[key]
    id: u8,
    value: String,
}

// Test u16 key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct U16Entity {
    #[key]
    id: u16,
    value: String,
}

// Test u32 key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct U32Entity {
    #[key]
    id: u32,
    value: String,
}

// Test u64 key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct U64Entity {
    #[key]
    id: u64,
    value: String,
}

// Test i8 key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct I8Entity {
    #[key]
    id: i8,
    value: String,
}

// Test i16 key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct I16Entity {
    #[key]
    id: i16,
    value: String,
}

// Test i32 key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct I32Entity {
    #[key]
    id: i32,
    value: String,
}

// Test i64 key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct I64Entity {
    #[key]
    id: i64,
    value: String,
}

// Test String key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct StringEntity {
    #[key]
    id: String,
    value: String,
}

// Test bool key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct BoolEntity {
    #[key]
    enabled: bool,
    config_value: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 Testing all supported key types in netabase...\n");

    // Initialize logging
    netabase::init_logging();

    println!("📋 Testing key generation for all supported types");
    test_key_generation()?;

    println!("\n🔄 Testing serialization for all key types");
    test_serialization_all_types()?;

    println!("\n🌐 Testing network operations with different key types");
    test_network_operations_all_types().await?;

    println!("\n✅ All supported key types validated successfully!");
    Ok(())
}

fn test_key_generation() -> anyhow::Result<()> {
    // Test u8
    let u8_entity = U8Entity {
        id: 255,
        value: "u8 test".to_string(),
    };
    assert_eq!(u8_entity.key().to_string(), "255");
    println!("   ✓ u8 key: {} -> {}", u8_entity.id, u8_entity.key());

    // Test u16
    let u16_entity = U16Entity {
        id: 65535,
        value: "u16 test".to_string(),
    };
    assert_eq!(u16_entity.key().to_string(), "65535");
    println!("   ✓ u16 key: {} -> {}", u16_entity.id, u16_entity.key());

    // Test u32
    let u32_entity = U32Entity {
        id: 4294967295,
        value: "u32 test".to_string(),
    };
    assert_eq!(u32_entity.key().to_string(), "4294967295");
    println!("   ✓ u32 key: {} -> {}", u32_entity.id, u32_entity.key());

    // Test u64
    let u64_entity = U64Entity {
        id: 18446744073709551615,
        value: "u64 test".to_string(),
    };
    assert_eq!(u64_entity.key().to_string(), "18446744073709551615");
    println!("   ✓ u64 key: {} -> {}", u64_entity.id, u64_entity.key());

    // Test i8
    let i8_entity = I8Entity {
        id: -128,
        value: "i8 test".to_string(),
    };
    assert_eq!(i8_entity.key().to_string(), "-128");
    println!("   ✓ i8 key: {} -> {}", i8_entity.id, i8_entity.key());

    // Test i16
    let i16_entity = I16Entity {
        id: -32768,
        value: "i16 test".to_string(),
    };
    assert_eq!(i16_entity.key().to_string(), "-32768");
    println!("   ✓ i16 key: {} -> {}", i16_entity.id, i16_entity.key());

    // Test i32
    let i32_entity = I32Entity {
        id: -2147483648,
        value: "i32 test".to_string(),
    };
    assert_eq!(i32_entity.key().to_string(), "-2147483648");
    println!("   ✓ i32 key: {} -> {}", i32_entity.id, i32_entity.key());

    // Test i64
    let i64_entity = I64Entity {
        id: -9223372036854775808,
        value: "i64 test".to_string(),
    };
    assert_eq!(i64_entity.key().to_string(), "-9223372036854775808");
    println!("   ✓ i64 key: {} -> {}", i64_entity.id, i64_entity.key());

    // Test String
    let string_entity = StringEntity {
        id: "test_string_key".to_string(),
        value: "String test".to_string(),
    };
    assert_eq!(string_entity.key().to_string(), "test_string_key");
    println!(
        "   ✓ String key: {} -> {}",
        string_entity.id,
        string_entity.key()
    );

    // Test bool
    let bool_entity = BoolEntity {
        enabled: true,
        config_value: "bool test".to_string(),
    };
    assert_eq!(bool_entity.key().to_string(), "true");
    println!(
        "   ✓ bool key: {} -> {}",
        bool_entity.enabled,
        bool_entity.key()
    );

    let bool_entity_false = BoolEntity {
        enabled: false,
        config_value: "bool test false".to_string(),
    };
    assert_eq!(bool_entity_false.key().to_string(), "false");
    println!(
        "   ✓ bool key: {} -> {}",
        bool_entity_false.enabled,
        bool_entity_false.key()
    );

    Ok(())
}

fn test_serialization_all_types() -> anyhow::Result<()> {
    // Test u64 entity
    let u64_original = U64Entity {
        id: 12345,
        value: "test value".to_string(),
    };
    let serialized = bincode::encode_to_vec(&u64_original, bincode::config::standard())?;
    let (deserialized, _): (U64Entity, usize) =
        bincode::decode_from_slice(&serialized, bincode::config::standard())?;
    assert_eq!(u64_original, deserialized);
    println!("   ✓ u64 entity serialization roundtrip successful");

    // Test String entity
    let string_original = StringEntity {
        id: "test_key".to_string(),
        value: "test value".to_string(),
    };
    let serialized = bincode::encode_to_vec(&string_original, bincode::config::standard())?;
    let (deserialized, _): (StringEntity, usize) =
        bincode::decode_from_slice(&serialized, bincode::config::standard())?;
    assert_eq!(string_original, deserialized);
    println!("   ✓ String entity serialization roundtrip successful");

    // Test bool entity
    let bool_original = BoolEntity {
        enabled: true,
        config_value: "test config".to_string(),
    };
    let serialized = bincode::encode_to_vec(&bool_original, bincode::config::standard())?;
    let (deserialized, _): (BoolEntity, usize) =
        bincode::decode_from_slice(&serialized, bincode::config::standard())?;
    assert_eq!(bool_original, deserialized);
    println!("   ✓ bool entity serialization roundtrip successful");

    // Test i32 entity
    let i32_original = I32Entity {
        id: -42,
        value: "negative test".to_string(),
    };
    let serialized = bincode::encode_to_vec(&i32_original, bincode::config::standard())?;
    let (deserialized, _): (I32Entity, usize) =
        bincode::decode_from_slice(&serialized, bincode::config::standard())?;
    assert_eq!(i32_original, deserialized);
    println!("   ✓ i32 entity serialization roundtrip successful");

    Ok(())
}

async fn test_network_operations_all_types() -> anyhow::Result<()> {
    // Create configuration
    let config = NetabaseConfig::default();

    // Generate keypair
    let keypair = Keypair::generate();

    // Create netabase instance
    let mut netabase = Netabase::try_new(config, "supported-keys-test").await?;

    // Start the network
    netabase.start_swarm().await?;
    println!("   ✓ Network started successfully");

    // Wait for network initialization
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Test with u64 key
    let u64_entity = U64Entity {
        id: 42,
        value: "u64 network test".to_string(),
    };
    let put_result = netabase
        .put(
            u64_entity.clone(),
            None::<std::vec::IntoIter<libp2p::PeerId>>,
            libp2p::kad::Quorum::One,
        )
        .await?;
    println!("   ✓ u64 entity put operation: {:?}", put_result.key);

    let get_result = netabase.get(u64_entity.key()).await?;
    let record_count = match get_result {
        libp2p::kad::GetRecordOk::FoundRecord(_) => 1,
        libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. } => 0,
    };
    println!("   ✓ u64 entity get operation: {} records", record_count);

    // Test with String key
    let string_entity = StringEntity {
        id: "network_test_key".to_string(),
        value: "String network test".to_string(),
    };
    let put_result = netabase
        .put(
            string_entity.clone(),
            None::<std::vec::IntoIter<libp2p::PeerId>>,
            libp2p::kad::Quorum::One,
        )
        .await?;
    println!("   ✓ String entity put operation: {:?}", put_result.key);

    let get_result = netabase.get(string_entity.key()).await?;
    let record_count = match get_result {
        libp2p::kad::GetRecordOk::FoundRecord(_) => 1,
        libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. } => 0,
    };
    println!("   ✓ String entity get operation: {} records", record_count);

    // Test with bool key
    let bool_entity = BoolEntity {
        enabled: true,
        config_value: "bool network test".to_string(),
    };
    let put_result = netabase
        .put(
            bool_entity.clone(),
            None::<std::vec::IntoIter<libp2p::PeerId>>,
            libp2p::kad::Quorum::One,
        )
        .await?;
    println!("   ✓ bool entity put operation: {:?}", put_result.key);

    let get_result = netabase.get(bool_entity.key()).await?;
    let record_count = match get_result {
        libp2p::kad::GetRecordOk::FoundRecord(_) => 1,
        libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. } => 0,
    };
    println!("   ✓ bool entity get operation: {} records", record_count);

    // Test with i32 key (negative number)
    let i32_entity = I32Entity {
        id: -999,
        value: "i32 negative network test".to_string(),
    };
    let put_result = netabase
        .put(
            i32_entity.clone(),
            None::<std::vec::IntoIter<libp2p::PeerId>>,
            libp2p::kad::Quorum::One,
        )
        .await?;
    println!("   ✓ i32 entity put operation: {:?}", put_result.key);

    let get_result = netabase.get(i32_entity.key()).await?;
    let record_count = match get_result {
        libp2p::kad::GetRecordOk::FoundRecord(_) => 1,
        libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. } => 0,
    };
    println!("   ✓ i32 entity get operation: {} records", record_count);

    Ok(())
}
