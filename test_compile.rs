//! Direct compilation test for netabase macros
//!
//! This is a simple test file that verifies the NetabaseSchema macro
//! compiles correctly and generates the expected code without any
//! network operations or complex dependencies.

use bincode::{Decode, Encode};
use netabase::NetabaseSchema;
use serde::{Deserialize, Serialize};

// Test basic u64 key schema
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct User {
    #[key]
    id: u64,
    name: String,
    email: String,
}

// Test String key schema
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct Document {
    #[key]
    doc_id: String,
    title: String,
    content: String,
}

// Test boolean key schema
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct Setting {
    #[key]
    enabled: bool,
    config_name: String,
    config_value: String,
}

// Test i32 key schema
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct Event {
    #[key]
    timestamp: i32,
    event_type: String,
    data: Vec<u8>,
}

fn main() {
    println!("🧪 Netabase Macro Compilation Test\n");

    // Test 1: Create instances and verify they compile
    let user = User {
        id: 42,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    println!("✓ User struct compiled and created: {:?}", user);

    let doc = Document {
        doc_id: "doc_123".to_string(),
        title: "Test Document".to_string(),
        content: "This is test content".to_string(),
    };
    println!("✓ Document struct compiled and created: {:?}", doc);

    let setting = Setting {
        enabled: true,
        config_name: "debug_mode".to_string(),
        config_value: "enabled".to_string(),
    };
    println!("✓ Setting struct compiled and created: {:?}", setting);

    let event = Event {
        timestamp: -1234567890,
        event_type: "user_login".to_string(),
        data: vec![1, 2, 3, 4, 5],
    };
    println!("✓ Event struct compiled and created: {:?}", event);

    // Test 2: Verify key generation works
    println!("\n🔑 Testing key generation:");

    let user_key = user.key();
    println!("   User key ({}): {}", user.id, user_key);
    assert_eq!(user_key.to_string(), "[42]");

    let doc_key = doc.key();
    println!("   Document key ({}): {}", doc.doc_id, doc_key);
    assert_eq!(doc_key.to_string(), "[8, 100, 111, 99, 95, 49, 50, 51]");

    let setting_key = setting.key();
    println!("   Setting key ({}): {}", setting.enabled, setting_key);
    assert_eq!(setting_key.to_string(), "[1]");

    let event_key = event.key();
    println!("   Event key ({}): {}", event.timestamp, event_key);
    assert_eq!(event_key.to_string(), "[46, 21, 205, 181]");

    // Test 3: Verify serialization works
    println!("\n🔄 Testing serialization:");

    match bincode::encode_to_vec(&user, bincode::config::standard()) {
        Ok(serialized) => {
            println!("   User serialized: {} bytes", serialized.len());
            match bincode::decode_from_slice::<User, _>(&serialized, bincode::config::standard()) {
                Ok((deserialized, _)) => {
                    assert_eq!(user, deserialized);
                    println!("   User deserialization: ✓");
                }
                Err(e) => panic!("User deserialization failed: {}", e),
            }
        }
        Err(e) => panic!("User serialization failed: {}", e),
    }

    match bincode::encode_to_vec(&doc, bincode::config::standard()) {
        Ok(serialized) => {
            println!("   Document serialized: {} bytes", serialized.len());
            match bincode::decode_from_slice::<Document, _>(
                &serialized,
                bincode::config::standard(),
            ) {
                Ok((deserialized, _)) => {
                    assert_eq!(doc, deserialized);
                    println!("   Document deserialization: ✓");
                }
                Err(e) => panic!("Document deserialization failed: {}", e),
            }
        }
        Err(e) => panic!("Document serialization failed: {}", e),
    }

    // Test 4: Verify record conversion works
    println!("\n📝 Testing record conversion:");

    let user_record: libp2p::kad::Record = user.clone().into();
    let user_key_bytes = user_record.key.to_vec();
    let record_key_str = String::from_utf8_lossy(&user_key_bytes);
    println!("   User record key: {}", record_key_str);
    assert_eq!(record_key_str, "*");

    let doc_record: libp2p::kad::Record = doc.clone().into();
    let doc_key_bytes = doc_record.key.to_vec();
    let doc_record_key_str = String::from_utf8_lossy(&doc_key_bytes);
    println!("   Document record key: {}", doc_record_key_str);
    assert_eq!(doc_record_key_str, "doc_123");

    // Test 5: Edge cases
    println!("\n⚠️ Testing edge cases:");

    // Zero value
    let zero_user = User {
        id: 0,
        name: "Zero".to_string(),
        email: "zero@test.com".to_string(),
    };
    assert_eq!(zero_user.key().to_string(), "[0]");
    println!("   Zero value key: ✓");

    // Empty string
    let empty_doc = Document {
        doc_id: "".to_string(),
        title: "Empty ID".to_string(),
        content: "Empty doc_id test".to_string(),
    };
    assert_eq!(empty_doc.key().to_string(), "[0]");
    println!("   Empty string key: ✓");

    // Negative number
    let negative_event = Event {
        timestamp: -999,
        event_type: "error".to_string(),
        data: vec![],
    };
    assert_eq!(negative_event.key().to_string(), "[25, 252, 255, 255]");
    println!("   Negative number key: ✓");

    // Boolean false
    let false_setting = Setting {
        enabled: false,
        config_name: "feature_x".to_string(),
        config_value: "disabled".to_string(),
    };
    assert_eq!(false_setting.key().to_string(), "[0]");
    println!("   Boolean false key: ✓");

    // Special characters in string
    let special_doc = Document {
        doc_id: "doc:with/special@chars#123!".to_string(),
        title: "Special".to_string(),
        content: "Testing special chars".to_string(),
    };
    assert_eq!(
        special_doc.key().to_string(),
        "[27, 100, 111, 99, 58, 119, 105, 116, 104, 47, 115, 112, 101, 99, 105, 97, 108, 64, 99, 104, 97, 114, 115, 35, 49, 50, 51, 33]"
    );
    println!("   Special characters key: ✓");

    // Unicode characters
    let unicode_doc = Document {
        doc_id: "文档_тест_🔑".to_string(),
        title: "Unicode".to_string(),
        content: "Testing unicode".to_string(),
    };
    assert_eq!(
        unicode_doc.key().to_string(),
        "[23, 230, 150, 135, 230, 161, 163, 95, 209, 130, 208, 181, 209, 129, 209, 130, 95, 240, 159, 148, 145]"
    );
    println!("   Unicode characters key: ✓");

    println!("\n✅ All macro compilation tests passed!");
    println!("\n📊 Test Summary:");
    println!("   • Basic schema compilation: ✓");
    println!("   • Key generation (u64, String, bool, i32): ✓");
    println!("   • Serialization/deserialization: ✓");
    println!("   • Record conversion: ✓");
    println!("   • Edge cases: ✓");
    println!("\n🎉 NetabaseSchema macro is working correctly!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_schema() {
        let user = User {
            id: 123,
            name: "Test".to_string(),
            email: "test@example.com".to_string(),
        };

        assert_eq!(user.key().to_string(), "[123]");

        let serialized = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();
        let (deserialized, _): (User, usize) =
            bincode::decode_from_slice(&serialized, bincode::config::standard()).unwrap();
        assert_eq!(user, deserialized);
    }

    #[test]
    fn test_document_schema() {
        let doc = Document {
            doc_id: "test_doc".to_string(),
            title: "Test".to_string(),
            content: "Content".to_string(),
        };

        assert_eq!(
            doc.key().to_string(),
            "[8, 116, 101, 115, 116, 95, 100, 111, 99]"
        );

        let record: libp2p::kad::Record = doc.clone().into();
        let key_bytes = record.key.to_vec();
        let key_str = String::from_utf8_lossy(&key_bytes);
        assert_eq!(key_str, "\u{8}test_doc");
    }

    #[test]
    fn test_setting_schema() {
        let setting = Setting {
            enabled: true,
            config_name: "test".to_string(),
            config_value: "value".to_string(),
        };

        assert_eq!(setting.key().to_string(), "[1]");

        let setting_false = Setting {
            enabled: false,
            config_name: "test".to_string(),
            config_value: "value".to_string(),
        };

        assert_eq!(setting_false.key().to_string(), "[0]");
    }

    #[test]
    fn test_event_schema() {
        let event = Event {
            timestamp: -12345,
            event_type: "test".to_string(),
            data: vec![1, 2, 3],
        };

        assert_eq!(event.key().to_string(), "[251, 113, 96]");
    }

    #[test]
    fn test_edge_cases() {
        // Zero
        let zero_user = User {
            id: 0,
            name: "".to_string(),
            email: "".to_string(),
        };
        assert_eq!(zero_user.key().to_string(), "[0]");

        // Empty string
        let empty_doc = Document {
            doc_id: "".to_string(),
            title: "".to_string(),
            content: "".to_string(),
        };
        assert_eq!(empty_doc.key().to_string(), "[0]");

        // Max values
        let max_user = User {
            id: u64::MAX,
            name: "".to_string(),
            email: "".to_string(),
        };
        assert_eq!(
            max_user.key().to_string(),
            "[253, 255, 255, 255, 255, 255, 255, 255, 255]"
        );

        // Min i32
        let min_event = Event {
            timestamp: i32::MIN,
            event_type: "".to_string(),
            data: vec![],
        };
        assert_eq!(min_event.key().to_string(), "[252, 255, 255, 255, 255]");
    }
}
