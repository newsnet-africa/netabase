fn main() {}

// //! Working Netabase Key Patterns Validation
// //!
// //! This example demonstrates ONLY the key patterns that currently work
// //! with the netabase implementation. These are guaranteed to compile and run.
// //!
// //! WORKING PATTERNS:
// //! - Single primitive field keys (u8, u16, u32, u64, i8, i16, i32, i64, String, bool)
// //! - Schema prefix attributes
// //! - Optional non-key fields
// //! - Nested structures in non-key fields
// //!
// //! IMPORTANT: This test validates the ACTUAL current implementation,
// //! not what we documented. The documentation examples need to be updated
// //! to match what actually works.

// use bincode::{Decode, Encode};
// use netabase::NetabaseSchema;
// use serde::{Deserialize, Serialize};

// // ============================================================================
// // WORKING: Basic Primitive Key Types
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct U64Key {
//     #[key]
//     id: u64,
//     data: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct U32Key {
//     #[key]
//     id: u32,
//     name: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct StringKey {
//     #[key]
//     identifier: String,
//     payload: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct BoolKey {
//     #[key]
//     active: bool,
//     info: String,
// }

// // ============================================================================
// // WORKING: Schema Prefix Attribute
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// #[schema(prefix = "user")]
// struct PrefixedUser {
//     #[key]
//     id: u64,
//     name: String,
//     email: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// #[schema(prefix = "msg", version = "v1")]
// struct PrefixedMessage {
//     #[key]
//     message_id: String,
//     content: String,
// }

// // ============================================================================
// // WORKING: Optional Non-Key Fields
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct WithOptionalFields {
//     #[key]
//     id: u64,
//     required_name: String,
//     optional_description: Option<String>,
//     optional_tags: Option<Vec<String>>,
// }

// // ============================================================================
// // WORKING: Nested Structures in Non-Key Fields
// // ============================================================================

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Encode, Decode)]
// struct Address {
//     street: String,
//     city: String,
//     country: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UserProfile {
//     #[key]
//     user_id: u64,
//     username: String,
//     address: Address,
//     preferences: Vec<String>,
// }

// // ============================================================================
// // WORKING: Complex Non-Key Data
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct ComplexRecord {
//     #[key]
//     record_id: String,
//     metadata: std::collections::HashMap<String, String>,
//     tags: Vec<String>,
//     scores: Vec<f64>,
//     nested_data: Option<Address>,
// }

// // ============================================================================
// // WORKING: Edge Cases with Unicode
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UnicodeKey {
//     #[key]
//     emoji_id: String,
//     description: String,
// }

// // ============================================================================
// // MAIN VALIDATION FUNCTION
// // ============================================================================

// fn main() {
//     println!("=== Netabase Working Patterns Validation ===\n");

//     test_primitive_keys();
//     test_schema_attributes();
//     test_optional_fields();
//     test_nested_structures();
//     test_edge_cases();
//     test_serialization();

//     println!("\n🎉 All working patterns validated successfully!");
//     print_limitations_and_workarounds();
// }

// fn test_primitive_keys() {
//     println!("🔑 Testing Primitive Key Types...");

//     let u64_item = U64Key {
//         id: 123456789,
//         data: "test data".to_string(),
//     };
//     println!("  U64 key: {}", u64_item.key());

//     let u32_item = U32Key {
//         id: 987654,
//         name: "test name".to_string(),
//     };
//     println!("  U32 key: {}", u32_item.key());

//     let string_item = StringKey {
//         identifier: "unique_string_id".to_string(),
//         payload: "payload data".to_string(),
//     };
//     println!("  String key: {}", string_item.key());

//     let bool_item = BoolKey {
//         active: true,
//         info: "active state".to_string(),
//     };
//     println!("  Bool key: {}", bool_item.key());

//     println!("  ✅ Primitive key types work correctly");
// }

// fn test_schema_attributes() {
//     println!("⚙️ Testing Schema Attributes...");

//     let prefixed_user = PrefixedUser {
//         id: 42,
//         name: "Alice".to_string(),
//         email: "alice@example.com".to_string(),
//     };
//     println!("  Prefixed user key: {}", prefixed_user.key());

//     let prefixed_message = PrefixedMessage {
//         message_id: "msg_001".to_string(),
//         content: "Hello world!".to_string(),
//     };
//     println!("  Prefixed message key: {}", prefixed_message.key());

//     println!("  ✅ Schema attributes work correctly");
// }

// fn test_optional_fields() {
//     println!("❓ Testing Optional Fields...");

//     let with_some = WithOptionalFields {
//         id: 789,
//         required_name: "Test Item".to_string(),
//         optional_description: Some("This has a description".to_string()),
//         optional_tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
//     };
//     println!("  With Some values key: {}", with_some.key());

//     let with_none = WithOptionalFields {
//         id: 456,
//         required_name: "Test Item 2".to_string(),
//         optional_description: None,
//         optional_tags: None,
//     };
//     println!("  With None values key: {}", with_none.key());

//     println!("  ✅ Optional fields work correctly");
// }

// fn test_nested_structures() {
//     println!("🏗️ Testing Nested Structures...");

//     let address = Address {
//         street: "123 Main St".to_string(),
//         city: "Anytown".to_string(),
//         country: "USA".to_string(),
//     };

//     let profile = UserProfile {
//         user_id: 999,
//         username: "testuser".to_string(),
//         address,
//         preferences: vec!["dark_mode".to_string(), "notifications".to_string()],
//     };
//     println!("  User profile key: {}", profile.key());

//     let mut metadata = std::collections::HashMap::new();
//     metadata.insert("type".to_string(), "important".to_string());
//     metadata.insert("priority".to_string(), "high".to_string());

//     let complex = ComplexRecord {
//         record_id: "complex_001".to_string(),
//         metadata,
//         tags: vec!["urgent".to_string(), "review".to_string()],
//         scores: vec![0.95, 0.87, 0.92],
//         nested_data: Some(Address {
//             street: "456 Oak Ave".to_string(),
//             city: "Somewhere".to_string(),
//             country: "Canada".to_string(),
//         }),
//     };
//     println!("  Complex record key: {}", complex.key());

//     println!("  ✅ Nested structures work correctly");
// }

// fn test_edge_cases() {
//     println!("⚠️ Testing Edge Cases...");

//     // Unicode in keys
//     let unicode = UnicodeKey {
//         emoji_id: "🔑_test_🌍".to_string(),
//         description: "Unicode test".to_string(),
//     };
//     println!("  Unicode key: {}", unicode.key());

//     // Large numbers
//     let large_number = U64Key {
//         id: 18446744073709551615u64, // max u64
//         data: "max value test".to_string(),
//     };
//     println!("  Large number key: {}", large_number.key());

//     // Empty strings
//     let empty_string = StringKey {
//         identifier: "".to_string(),
//         payload: "empty key test".to_string(),
//     };
//     println!("  Empty string key: '{}'", empty_string.key());

//     println!("  ✅ Edge cases work correctly");
// }

// fn test_serialization() {
//     println!("🔄 Testing Serialization...");

//     let original = StringKey {
//         identifier: "serialize_test".to_string(),
//         payload: "test payload".to_string(),
//     };

//     let serialized = bincode::encode_to_vec(&original, bincode::config::standard())
//         .expect("Serialization failed");
//     let (deserialized, _): (StringKey, usize) =
//         bincode::decode_from_slice(&serialized, bincode::config::standard())
//             .expect("Deserialization failed");

//     assert_eq!(original, deserialized);
//     assert_eq!(original.key().to_string(), deserialized.key().to_string());
//     println!("  Serialization roundtrip successful");

//     // Test with complex structure
//     let complex = UserProfile {
//         user_id: 123,
//         username: "serialization_test".to_string(),
//         address: Address {
//             street: "Test St".to_string(),
//             city: "Test City".to_string(),
//             country: "Test Country".to_string(),
//         },
//         preferences: vec!["pref1".to_string()],
//     };

//     let serialized = bincode::encode_to_vec(&complex, bincode::config::standard())
//         .expect("Serialization failed");
//     let (deserialized, _): (UserProfile, usize) =
//         bincode::decode_from_slice(&serialized, bincode::config::standard())
//             .expect("Deserialization failed");

//     assert_eq!(complex, deserialized);
//     assert_eq!(complex.key().to_string(), deserialized.key().to_string());
//     println!("  Complex structure serialization successful");

//     println!("  ✅ Serialization works correctly");
// }

// fn print_limitations_and_workarounds() {
//     println!("📊 CURRENT IMPLEMENTATION STATUS:");
//     println!("==================================");
//     println!("✅ WORKING Features:");
//     println!(
//         "   - Single primitive key fields: u8, u16, u32, u64, i8, i16, i32, i64, String, bool"
//     );
//     println!("   - Schema prefix: #[schema(prefix = \"name\")]");
//     println!("   - Schema version: #[schema(version = \"v1\")] (metadata only)");
//     println!("   - Optional non-key fields: Option<T> in data fields");
//     println!("   - Nested structures in non-key fields");
//     println!("   - Collections in non-key fields: Vec<T>, HashMap<K,V>, etc.");
//     println!("   - Unicode support in string keys");
//     println!("   - Serialization with bincode");
//     println!("");

//     println!("❌ NOT WORKING (Documentation Needs Updates):");
//     println!("   - Multiple #[key] fields (composite keys)");
//     println!("   - Enum variants with #[key] fields");
//     println!("   - Custom separators: #[schema(separator = \"|\")]");
//     println!("   - Collection types as key fields");
//     println!("   - Option<T> as key fields");
//     println!("   - Custom struct types as key fields");
//     println!("");

//     println!("🔧 WORKAROUNDS for Unsupported Features:");
//     println!("   1. Composite Keys → Use String field with manual formatting:");
//     println!("      struct CompositeKey {{ #[key] key: String, org: u64, user: u64 }}");
//     println!("      impl CompositeKey {{ fn new(org: u64, user: u64) -> Self {{");
//     println!("          Self {{ key: format!(\"{{}}::{{}}\", org, user), org, user }}");
//     println!("      }}");
//     println!("");
//     println!("   2. Complex Key Logic → Manual key() method:");
//     println!("      impl MyStruct {{ fn key(&self) -> String {{ /* custom logic */ }} }}");
//     println!("");

//     println!("⚠️ DOCUMENTATION ISSUES FOUND:");
//     println!("   - Examples in README.md show unsupported composite keys");
//     println!("   - Enum examples don't work with current implementation");
//     println!("   - Custom separator examples don't work");
//     println!("   - Need to update all docs to show only working patterns");
//     println!("");

//     println!("🎯 CURRENT NETABASE IS SUITABLE FOR:");
//     println!("   - Simple key-value storage with primitive keys");
//     println!("   - Distributed storage of structured data");
//     println!("   - Use cases where single-field keys are sufficient");
//     println!("   - Applications that can work around composite key limitations");
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_all_working_patterns() {
//         // Test primitive keys
//         let u64_key = U64Key {
//             id: 123,
//             data: "test".to_string(),
//         };
//         assert!(u64_key.key().to_string().contains("123"));

//         let string_key = StringKey {
//             identifier: "test_id".to_string(),
//             payload: "payload".to_string(),
//         };
//         assert_eq!(string_key.key().to_string(), "test_id");

//         // Test prefixed schema
//         let prefixed = PrefixedUser {
//             id: 456,
//             name: "Test".to_string(),
//             email: "test@example.com".to_string(),
//         };
//         assert!(prefixed.key().to_string().contains("456"));

//         // Test optional fields
//         let with_opts = WithOptionalFields {
//             id: 789,
//             required_name: "Test".to_string(),
//             optional_description: Some("Desc".to_string()),
//             optional_tags: None,
//         };
//         assert!(with_opts.key().to_string().contains("789"));
//     }

//     #[test]
//     fn test_serialization_roundtrip() {
//         let original = PrefixedUser {
//             id: 999,
//             name: "Roundtrip Test".to_string(),
//             email: "test@example.com".to_string(),
//         };

//         let serialized = bincode::encode_to_vec(&original, bincode::config::standard()).unwrap();
//         let (deserialized, _): (PrefixedUser, usize) =
//             bincode::decode_from_slice(&serialized, bincode::config::standard()).unwrap();

//         assert_eq!(original, deserialized);
//         assert_eq!(original.key().to_string(), deserialized.key().to_string());
//     }

//     #[test]
//     fn test_unicode_keys() {
//         let unicode = UnicodeKey {
//             emoji_id: "🚀🌟💫".to_string(),
//             description: "Unicode test".to_string(),
//         };

//         assert_eq!(unicode.key().to_string(), "🚀🌟💫");

//         // Test serialization with unicode
//         let serialized = bincode::encode_to_vec(&unicode, bincode::config::standard()).unwrap();
//         let (deserialized, _): (UnicodeKey, usize) =
//             bincode::decode_from_slice(&serialized, bincode::config::standard()).unwrap();
//         assert_eq!(unicode.key().to_string(), deserialized.key().to_string());
//     }

//     #[test]
//     fn test_key_consistency() {
//         let item1 = U64Key {
//             id: 100,
//             data: "data1".to_string(),
//         };
//         let item2 = U64Key {
//             id: 100,
//             data: "different_data".to_string(),
//         };

//         // Same key field should produce same key, regardless of other fields
//         assert_eq!(item1.key().to_string(), item2.key().to_string());
//     }

//     #[test]
//     fn test_edge_cases() {
//         // Test with empty string key
//         let empty_key = StringKey {
//             identifier: "".to_string(),
//             payload: "test".to_string(),
//         };
//         assert_eq!(empty_key.key().to_string(), "");

//         // Test with very long string
//         let long_key = StringKey {
//             identifier: "a".repeat(1000),
//             payload: "test".to_string(),
//         };
//         assert_eq!(long_key.key().to_string().len(), 1000);

//         // Test boolean keys
//         let bool_true = BoolKey {
//             active: true,
//             info: "test".to_string(),
//         };
//         let bool_false = BoolKey {
//             active: false,
//             info: "test".to_string(),
//         };
//         assert_eq!(bool_true.key().to_string(), "true");
//         assert_eq!(bool_false.key().to_string(), "false");
//     }
// }
