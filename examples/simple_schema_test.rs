//! Simple schema test for netabase macros
//!
//! This example tests the basic functionality of the NetabaseSchema derive macro
//! and validates that the generated code works as expected.

use bincode::{Decode, Encode};
use netabase::NetabaseSchema;
use serde::{Deserialize, Serialize};

// Simple struct to test basic schema functionality
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct SimpleUser {
    #[key]
    id: u64,
    name: String,
}

// Test with String key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct Document {
    #[key]
    doc_id: String,
    title: String,
    content: String,
}

// Test with boolean key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct Setting {
    #[key]
    enabled: bool,
    config_name: String,
    config_value: String,
}

fn main() -> anyhow::Result<()> {
    println!("🧪 Simple schema test for netabase macros\n");

    // Test 1: Basic schema creation and key generation
    println!("📋 Test 1: Basic schema creation");
    test_basic_schema_creation()?;

    // Test 2: Key generation for different types
    println!("\n🔑 Test 2: Key generation");
    test_key_generation()?;

    // Test 3: Schema trait implementations
    println!("\n🛠️  Test 3: Schema trait implementations");
    test_schema_traits()?;

    // Test 4: Record conversion
    println!("\n🔄 Test 4: Record conversion");
    test_record_conversion()?;

    println!("\n✅ All simple schema tests passed!");
    Ok(())
}

fn test_basic_schema_creation() -> anyhow::Result<()> {
    // Create instances of each schema type
    let user = SimpleUser {
        id: 123,
        name: "Alice".to_string(),
    };
    println!("   Created SimpleUser: {:?}", user);

    let doc = Document {
        doc_id: "doc_456".to_string(),
        title: "Test Document".to_string(),
        content: "This is test content".to_string(),
    };
    println!("   Created Document: {:?}", doc);

    let setting = Setting {
        enabled: true,
        config_name: "debug_mode".to_string(),
        config_value: "enabled".to_string(),
    };
    println!("   Created Setting: {:?}", setting);

    Ok(())
}

fn test_key_generation() -> anyhow::Result<()> {
    let user = SimpleUser {
        id: 42,
        name: "Bob".to_string(),
    };
    let user_key = user.key();
    println!("   SimpleUser key: {}", user_key);
    assert_eq!(user_key.to_string(), "42");

    let doc = Document {
        doc_id: "my_document".to_string(),
        title: "My Document".to_string(),
        content: "Content here".to_string(),
    };
    let doc_key = doc.key();
    println!("   Document key: {}", doc_key);
    assert_eq!(doc_key.to_string(), "my_document");

    let setting = Setting {
        enabled: false,
        config_name: "feature_x".to_string(),
        config_value: "disabled".to_string(),
    };
    let setting_key = setting.key();
    println!("   Setting key: {}", setting_key);
    assert_eq!(setting_key.to_string(), "false");

    println!("   ✓ All key generations work correctly");
    Ok(())
}

fn test_schema_traits() -> anyhow::Result<()> {
    let user = SimpleUser {
        id: 999,
        name: "Charlie".to_string(),
    };

    // Test that the NetabaseSchema trait is implemented
    let key = user.key();
    println!("   NetabaseSchema::key() works: {}", key);

    // Test Clone
    let user_clone = user.clone();
    assert_eq!(user, user_clone);
    println!("   ✓ Clone trait works");

    // Test Debug
    let debug_str = format!("{:?}", user);
    assert!(debug_str.contains("SimpleUser"));
    println!("   ✓ Debug trait works: {}", debug_str);

    // Test PartialEq
    let user2 = SimpleUser {
        id: 999,
        name: "Charlie".to_string(),
    };
    assert_eq!(user, user2);
    println!("   ✓ PartialEq trait works");

    Ok(())
}

fn test_record_conversion() -> anyhow::Result<()> {
    let user = SimpleUser {
        id: 777,
        name: "Diana".to_string(),
    };

    // Test conversion to libp2p Record
    let record: libp2p::kad::Record = user.clone().into();

    // Verify the key is correct
    let key_bytes = record.key.to_vec();
    let key_str = String::from_utf8_lossy(&key_bytes);
    assert_eq!(key_str, "777");
    println!("   ✓ Record key conversion: {}", key_str);

    // Verify we can deserialize the data back
    let (deserialized_user, _): (SimpleUser, usize) =
        bincode::decode_from_slice(&record.value, bincode::config::standard())?;
    assert_eq!(user, deserialized_user);
    println!("   ✓ Record data roundtrip successful");

    // Test with String key
    let doc = Document {
        doc_id: "test_doc_123".to_string(),
        title: "Test".to_string(),
        content: "Test content".to_string(),
    };

    let doc_record: libp2p::kad::Record = doc.clone().into();
    let doc_key_bytes = doc_record.key.to_vec();
    let doc_key_str = String::from_utf8_lossy(&doc_key_bytes);
    assert_eq!(doc_key_str, "test_doc_123");
    println!("   ✓ String key record conversion: {}", doc_key_str);

    let (deserialized_doc, _): (Document, usize) =
        bincode::decode_from_slice(&doc_record.value, bincode::config::standard())?;
    assert_eq!(doc, deserialized_doc);
    println!("   ✓ Document record data roundtrip successful");

    Ok(())
}
