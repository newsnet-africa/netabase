//! Test netabase macros without network operations
//!
//! This example tests only the macro functionality without any network operations
//! to verify that the NetabaseSchema derive macro works correctly.

use bincode::{Decode, Encode};
use netabase::NetabaseSchema;
use serde::{Deserialize, Serialize};

// Test basic schema with u64 key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct User {
    #[key]
    id: u64,
    name: String,
    email: String,
}

// Test schema with String key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct Document {
    #[key]
    doc_id: String,
    title: String,
    content: String,
}

// Test schema with boolean key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct Setting {
    #[key]
    enabled: bool,
    config_value: String,
}

// Test schema with i32 key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct Event {
    #[key]
    timestamp: i32,
    event_type: String,
    data: Vec<u8>,
}

fn main() -> anyhow::Result<()> {
    println!("🧪 Testing netabase macros (no network operations)\n");

    // Test 1: Basic macro compilation
    println!("📦 Test 1: Basic macro compilation");
    test_macro_compilation()?;

    // Test 2: Key generation
    println!("\n🔑 Test 2: Key generation");
    test_key_generation()?;

    // Test 3: Serialization
    println!("\n🔄 Test 3: Serialization");
    test_serialization()?;

    // Test 4: Record conversion (no network)
    println!("\n📝 Test 4: Record conversion");
    test_record_conversion()?;

    // Test 5: Edge cases
    println!("\n⚠️  Test 5: Edge cases");
    test_edge_cases()?;

    println!("\n✅ All macro tests passed!");
    Ok(())
}

fn test_macro_compilation() -> anyhow::Result<()> {
    // Just creating instances should prove macros compiled correctly
    let user = User {
        id: 42,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let doc = Document {
        doc_id: "doc_123".to_string(),
        title: "Test Document".to_string(),
        content: "Test content".to_string(),
    };

    let setting = Setting {
        enabled: true,
        config_value: "debug=true".to_string(),
    };

    let event = Event {
        timestamp: -1234567890,
        event_type: "user_login".to_string(),
        data: vec![1, 2, 3, 4, 5],
    };

    println!("   ✓ User struct created: {:?}", user);
    println!("   ✓ Document struct created: {:?}", doc);
    println!("   ✓ Setting struct created: {:?}", setting);
    println!("   ✓ Event struct created: {:?}", event);

    Ok(())
}

fn test_key_generation() -> anyhow::Result<()> {
    // Test u64 key
    let user = User {
        id: 12345,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };
    let user_key = user.key();
    assert_eq!(user_key.to_string(), "12345");
    println!("   ✓ u64 key: {} -> {}", user.id, user_key);

    // Test String key
    let doc = Document {
        doc_id: "my_document_456".to_string(),
        title: "My Document".to_string(),
        content: "Document content here".to_string(),
    };
    let doc_key = doc.key();
    assert_eq!(doc_key.to_string(), "my_document_456");
    println!("   ✓ String key: {} -> {}", doc.doc_id, doc_key);

    // Test boolean key (true)
    let setting_true = Setting {
        enabled: true,
        config_value: "feature_enabled".to_string(),
    };
    let setting_key = setting_true.key();
    assert_eq!(setting_key.to_string(), "true");
    println!(
        "   ✓ bool key (true): {} -> {}",
        setting_true.enabled, setting_key
    );

    // Test boolean key (false)
    let setting_false = Setting {
        enabled: false,
        config_value: "feature_disabled".to_string(),
    };
    let setting_key = setting_false.key();
    assert_eq!(setting_key.to_string(), "false");
    println!(
        "   ✓ bool key (false): {} -> {}",
        setting_false.enabled, setting_key
    );

    // Test i32 key (negative)
    let event = Event {
        timestamp: -999999,
        event_type: "system_error".to_string(),
        data: vec![],
    };
    let event_key = event.key();
    assert_eq!(event_key.to_string(), "-999999");
    println!(
        "   ✓ i32 key (negative): {} -> {}",
        event.timestamp, event_key
    );

    Ok(())
}

fn test_serialization() -> anyhow::Result<()> {
    // Test User serialization
    let user = User {
        id: 789,
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
    };

    let serialized = bincode::encode_to_vec(&user, bincode::config::standard())?;
    let (deserialized, _): (User, usize) =
        bincode::decode_from_slice(&serialized, bincode::config::standard())?;

    assert_eq!(user, deserialized);
    println!(
        "   ✓ User serialization roundtrip: {} bytes",
        serialized.len()
    );

    // Test Document serialization
    let doc = Document {
        doc_id: "test_doc".to_string(),
        title: "Test".to_string(),
        content: "This is a test document with some content.".to_string(),
    };

    let serialized = bincode::encode_to_vec(&doc, bincode::config::standard())?;
    let (deserialized, _): (Document, usize) =
        bincode::decode_from_slice(&serialized, bincode::config::standard())?;

    assert_eq!(doc, deserialized);
    println!(
        "   ✓ Document serialization roundtrip: {} bytes",
        serialized.len()
    );

    // Test complex data serialization
    let event = Event {
        timestamp: 1640995200, // Jan 1, 2022
        event_type: "data_sync".to_string(),
        data: (0..100).collect::<Vec<u8>>(), // 100 bytes of data
    };

    let serialized = bincode::encode_to_vec(&event, bincode::config::standard())?;
    let (deserialized, _): (Event, usize) =
        bincode::decode_from_slice(&serialized, bincode::config::standard())?;

    assert_eq!(event, deserialized);
    println!(
        "   ✓ Event serialization roundtrip: {} bytes",
        serialized.len()
    );

    Ok(())
}

fn test_record_conversion() -> anyhow::Result<()> {
    let user = User {
        id: 555,
        name: "Diana".to_string(),
        email: "diana@example.com".to_string(),
    };

    // Convert to libp2p Record
    let record: libp2p::kad::Record = user.clone().into();

    // Verify key
    let key_bytes = record.key.to_vec();
    let key_str = String::from_utf8_lossy(&key_bytes);
    assert_eq!(key_str, "555");
    println!("   ✓ Record key conversion: {}", key_str);

    // Verify data can be deserialized
    let (deserialized, _): (User, usize) =
        bincode::decode_from_slice(&record.value, bincode::config::standard())?;
    assert_eq!(user, deserialized);
    println!("   ✓ Record data conversion successful");

    // Test with String key
    let doc = Document {
        doc_id: "record_test_doc".to_string(),
        title: "Record Test".to_string(),
        content: "Testing record conversion".to_string(),
    };

    let doc_record: libp2p::kad::Record = doc.clone().into();
    let doc_key_bytes = doc_record.key.to_vec();
    let doc_key_str = String::from_utf8_lossy(&doc_key_bytes);
    assert_eq!(doc_key_str, "record_test_doc");
    println!("   ✓ String key record conversion: {}", doc_key_str);

    let (deserialized_doc, _): (Document, usize) =
        bincode::decode_from_slice(&doc_record.value, bincode::config::standard())?;
    assert_eq!(doc, deserialized_doc);
    println!("   ✓ Document record data conversion successful");

    Ok(())
}

fn test_edge_cases() -> anyhow::Result<()> {
    // Test zero values
    let zero_user = User {
        id: 0,
        name: "Zero User".to_string(),
        email: "zero@example.com".to_string(),
    };
    assert_eq!(zero_user.key().to_string(), "0");
    println!("   ✓ Zero key value: {}", zero_user.key());

    // Test empty string
    let empty_doc = Document {
        doc_id: "".to_string(),
        title: "Empty ID Document".to_string(),
        content: "This document has an empty doc_id".to_string(),
    };
    assert_eq!(empty_doc.key().to_string(), "");
    println!("   ✓ Empty string key: '{}'", empty_doc.key());

    // Test maximum values
    let max_user = User {
        id: u64::MAX,
        name: "Max User".to_string(),
        email: "max@example.com".to_string(),
    };
    assert_eq!(max_user.key().to_string(), "18446744073709551615");
    println!("   ✓ Maximum u64 key: {}", max_user.key());

    // Test minimum i32 value
    let min_event = Event {
        timestamp: i32::MIN,
        event_type: "minimum_time".to_string(),
        data: vec![],
    };
    assert_eq!(min_event.key().to_string(), "-2147483648");
    println!("   ✓ Minimum i32 key: {}", min_event.key());

    // Test special characters in string key
    let special_doc = Document {
        doc_id: "doc/with:special@chars#123!$%^&*()".to_string(),
        title: "Special Characters".to_string(),
        content: "Testing special characters in key".to_string(),
    };
    assert_eq!(
        special_doc.key().to_string(),
        "doc/with:special@chars#123!$%^&*()"
    );
    println!("   ✓ Special characters in key: {}", special_doc.key());

    // Test Unicode in string key
    let unicode_doc = Document {
        doc_id: "文档_документ_🔑_key".to_string(),
        title: "Unicode Test".to_string(),
        content: "Testing Unicode characters in key".to_string(),
    };
    assert_eq!(unicode_doc.key().to_string(), "文档_документ_🔑_key");
    println!("   ✓ Unicode in key: {}", unicode_doc.key());

    // Test large string key
    let large_key = "a".repeat(1000);
    let large_doc = Document {
        doc_id: large_key.clone(),
        title: "Large Key Test".to_string(),
        content: "Testing very large key".to_string(),
    };
    assert_eq!(large_doc.key().to_string(), large_key);
    println!(
        "   ✓ Large string key: {} characters",
        large_doc.key().to_string().len()
    );

    Ok(())
}
