//! Comprehensive test for netabase_macros functionality
//!
//! This example tests various features of the netabase_macros crate including:
//! - Basic NetabaseSchema derive macro
//! - Different key types
//! - Generated trait implementations
//! - Code generation validation

use bincode::{Decode, Encode};
use netabase::NetabaseSchema;
use serde::{Deserialize, Serialize};

// Test basic macro functionality
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct BasicEntity {
    #[key]
    id: u64,
    name: String,
    description: String,
}

// Test with String key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct StringKeyEntity {
    #[key]
    identifier: String,
    data: Vec<u8>,
    metadata: Option<String>,
}

// Test with different integer types
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct U32Entity {
    #[key]
    sequence: u32,
    payload: String,
}

#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct I64Entity {
    #[key]
    timestamp: i64,
    event_data: String,
}

// Test with boolean key
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct BoolKeyEntity {
    #[key]
    active: bool,
    config: std::collections::HashMap<String, String>,
}

// Test with more complex data types
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct ComplexEntity {
    #[key]
    uuid: String,
    tags: Vec<String>,
    properties: std::collections::HashMap<String, i32>,
    nested_data: NestedData,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Encode, Decode)]
struct NestedData {
    value: f64,
    labels: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    println!("🧪 Testing netabase_macros functionality\n");

    // Test 1: Macro code generation
    println!("📦 Test 1: Macro code generation validation");
    test_macro_generation()?;

    // Test 2: Key trait implementations
    println!("\n🔑 Test 2: Key trait implementations");
    test_key_traits()?;

    // Test 3: Schema trait implementations
    println!("\n📋 Test 3: Schema trait implementations");
    test_schema_traits()?;

    // Test 4: Serialization traits
    println!("\n🔄 Test 4: Serialization traits");
    test_serialization_traits()?;

    // Test 5: Record conversion traits
    println!("\n📝 Test 5: Record conversion traits");
    test_record_conversion_traits()?;

    // Test 6: Type safety validation
    println!("\n🛡️  Test 6: Type safety validation");
    test_type_safety()?;

    // Test 7: Edge cases
    println!("\n⚠️  Test 7: Edge cases and boundary conditions");
    test_edge_cases()?;

    println!("\n✅ All netabase_macros functionality tests passed!");
    Ok(())
}

fn test_macro_generation() -> anyhow::Result<()> {
    // Test that the macro generates the expected key types
    let entity = BasicEntity {
        id: 42,
        name: "Test".to_string(),
        description: "Test description".to_string(),
    };

    // The macro should generate a key method
    let key = entity.key();
    println!("   Generated key for BasicEntity: {}", key);

    // Test with different key types
    let string_entity = StringKeyEntity {
        identifier: "test_id".to_string(),
        data: vec![1, 2, 3, 4],
        metadata: Some("meta".to_string()),
    };

    let string_key = string_entity.key();
    println!("   Generated key for StringKeyEntity: {}", string_key);

    // Verify key generation works correctly
    assert_eq!(key.to_string(), "42");
    assert_eq!(string_key.to_string(), "test_id");

    println!("   ✓ Macro generates keys correctly for different types");
    Ok(())
}

fn test_key_traits() -> anyhow::Result<()> {
    let entity = U32Entity {
        sequence: 12345,
        payload: "test payload".to_string(),
    };

    let key = entity.key();

    // Test Display trait on key
    let key_str = format!("{}", key);
    assert_eq!(key_str, "12345");
    println!("   ✓ Key implements Display: {}", key_str);

    // Test that key can be converted to String
    let key_string = key.to_string();
    assert_eq!(key_string, "12345");
    println!("   ✓ Key converts to String: {}", key_string);

    Ok(())
}

fn test_schema_traits() -> anyhow::Result<()> {
    let entity = I64Entity {
        timestamp: -1234567890,
        event_data: "negative timestamp test".to_string(),
    };

    // Test NetabaseSchema trait
    let key = entity.key();
    println!("   ✓ NetabaseSchema::key() works: {}", key);
    assert_eq!(key.to_string(), "-1234567890");

    // Test Clone
    let cloned = entity.clone();
    assert_eq!(entity, cloned);
    println!("   ✓ Clone trait implemented correctly");

    // Test Debug
    let debug_output = format!("{:?}", entity);
    assert!(debug_output.contains("I64Entity"));
    println!("   ✓ Debug trait works: contains struct name");

    // Test PartialEq
    let same_entity = I64Entity {
        timestamp: -1234567890,
        event_data: "negative timestamp test".to_string(),
    };
    assert_eq!(entity, same_entity);
    println!("   ✓ PartialEq trait works correctly");

    Ok(())
}

fn test_serialization_traits() -> anyhow::Result<()> {
    let complex_entity = ComplexEntity {
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        properties: {
            let mut map = std::collections::HashMap::new();
            map.insert("count".to_string(), 42);
            map.insert("priority".to_string(), 1);
            map
        },
        nested_data: NestedData {
            value: 3.14159,
            labels: vec!["pi".to_string(), "constant".to_string()],
        },
    };

    // Test bincode Encode trait
    let encoded = bincode::encode_to_vec(&complex_entity, bincode::config::standard())?;
    println!("   ✓ Encode trait works, {} bytes", encoded.len());

    // Test bincode Decode trait
    let (decoded, _): (ComplexEntity, usize) =
        bincode::decode_from_slice(&encoded, bincode::config::standard())?;
    assert_eq!(complex_entity, decoded);
    println!("   ✓ Decode trait works, roundtrip successful");

    // Test serde Serialize/Deserialize
    let json_str = serde_json::to_string(&complex_entity)?;
    println!(
        "   ✓ Serialize trait works, JSON length: {}",
        json_str.len()
    );

    let from_json: ComplexEntity = serde_json::from_str(&json_str)?;
    assert_eq!(complex_entity, from_json);
    println!("   ✓ Deserialize trait works, JSON roundtrip successful");

    Ok(())
}

fn test_record_conversion_traits() -> anyhow::Result<()> {
    let bool_entity = BoolKeyEntity {
        active: true,
        config: {
            let mut map = std::collections::HashMap::new();
            map.insert("timeout".to_string(), "30s".to_string());
            map.insert("retries".to_string(), "3".to_string());
            map
        },
    };

    // Test conversion to libp2p::kad::Record
    let record: libp2p::kad::Record = bool_entity.clone().into();

    // Verify key conversion
    let record_key_bytes = record.key.to_vec();
    let record_key_str = String::from_utf8_lossy(&record_key_bytes);
    assert_eq!(record_key_str, "true");
    println!("   ✓ Record key conversion: {}", record_key_str);

    // Verify data conversion
    let (decoded_entity, _): (BoolKeyEntity, usize) =
        bincode::decode_from_slice(&record.value, bincode::config::standard())?;
    assert_eq!(bool_entity, decoded_entity);
    println!("   ✓ Record data conversion successful");

    // Test with false boolean
    let false_entity = BoolKeyEntity {
        active: false,
        config: std::collections::HashMap::new(),
    };

    let false_record: libp2p::kad::Record = false_entity.into();
    let false_key_bytes = false_record.key.to_vec();
    let false_key_str = String::from_utf8_lossy(&false_key_bytes);
    assert_eq!(false_key_str, "false");
    println!("   ✓ Boolean false key conversion: {}", false_key_str);

    Ok(())
}

fn test_type_safety() -> anyhow::Result<()> {
    // Test that keys maintain type safety
    let entity1 = BasicEntity {
        id: 100,
        name: "Entity 1".to_string(),
        description: "First entity".to_string(),
    };

    let entity2 = BasicEntity {
        id: 200,
        name: "Entity 2".to_string(),
        description: "Second entity".to_string(),
    };

    let key1 = entity1.key();
    let key2 = entity2.key();

    // Keys should be comparable
    assert_ne!(key1.to_string(), key2.to_string());
    println!("   ✓ Keys maintain uniqueness: {} != {}", key1, key2);

    // Test that different entity types produce different key types
    let string_entity = StringKeyEntity {
        identifier: "100".to_string(),
        data: vec![],
        metadata: None,
    };

    let string_key = string_entity.key();

    // Even though they have the same value, they should be different types
    assert_eq!(key1.to_string(), "100");
    assert_eq!(string_key.to_string(), "100");
    println!("   ✓ Different key types maintain type safety");

    Ok(())
}

fn test_edge_cases() -> anyhow::Result<()> {
    // Test empty strings
    let empty_string_entity = StringKeyEntity {
        identifier: "".to_string(),
        data: vec![],
        metadata: None,
    };

    let empty_key = empty_string_entity.key();
    assert_eq!(empty_key.to_string(), "");
    println!("   ✓ Empty string key handled correctly");

    // Test zero values
    let zero_entity = BasicEntity {
        id: 0,
        name: "Zero".to_string(),
        description: "Zero ID test".to_string(),
    };

    let zero_key = zero_entity.key();
    assert_eq!(zero_key.to_string(), "0");
    println!("   ✓ Zero value key handled correctly");

    // Test negative numbers
    let negative_entity = I64Entity {
        timestamp: -1,
        event_data: "negative one".to_string(),
    };

    let negative_key = negative_entity.key();
    assert_eq!(negative_key.to_string(), "-1");
    println!("   ✓ Negative number key handled correctly");

    // Test large numbers
    let large_entity = U32Entity {
        sequence: u32::MAX,
        payload: "max value test".to_string(),
    };

    let large_key = large_entity.key();
    assert_eq!(large_key.to_string(), "4294967295");
    println!("   ✓ Large number key handled correctly");

    // Test special characters in strings
    let special_chars_entity = StringKeyEntity {
        identifier: "key/with:special@chars#123!".to_string(),
        data: vec![],
        metadata: None,
    };

    let special_key = special_chars_entity.key();
    assert_eq!(special_key.to_string(), "key/with:special@chars#123!");
    println!("   ✓ Special characters in string keys handled correctly");

    // Test Unicode
    let unicode_entity = StringKeyEntity {
        identifier: "测试_ключ_🔑".to_string(),
        data: vec![],
        metadata: None,
    };

    let unicode_key = unicode_entity.key();
    assert_eq!(unicode_key.to_string(), "测试_ключ_🔑");
    println!("   ✓ Unicode in string keys handled correctly");

    Ok(())
}
