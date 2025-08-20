fn main() {}

// //! Validate Supported Netabase Key Types
// //!
// //! This example demonstrates all currently SUPPORTED key patterns in netabase.
// //! These examples are guaranteed to compile and work correctly.
// //!
// //! SUPPORTED PATTERNS:
// //! - Single field keys with primitive types
// //! - String keys
// //! - Integer keys (u8, u16, u32, u64, i8, i16, i32, i64)
// //! - Boolean keys
// //! - Schema attributes (prefix, version)
// //! - Optional non-key fields
// //! - Single enum variant with key
// //!
// //! WORKAROUNDS for unsupported patterns are also demonstrated.

// use bincode::{Decode, Encode};
// use netabase::NetabaseSchema;
// use serde::{Deserialize, Serialize};

// // ============================================================================
// // SUPPORTED: Primitive Key Types
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
// struct U16Key {
//     #[key]
//     id: u16,
//     value: i32,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct U8Key {
//     #[key]
//     id: u8,
//     flag: bool,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct I64Key {
//     #[key]
//     id: i64,
//     content: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct I32Key {
//     #[key]
//     id: i32,
//     metadata: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct I16Key {
//     #[key]
//     id: i16,
//     description: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct I8Key {
//     #[key]
//     id: i8,
//     status: String,
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
// // SUPPORTED: Schema Attributes
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
// #[schema(version = "v1")]
// struct VersionedDocument {
//     #[key]
//     doc_id: String,
//     title: String,
//     content: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// #[schema(prefix = "msg", version = "v2")]
// struct PrefixedAndVersioned {
//     #[key]
//     message_id: String,
//     sender: String,
//     content: String,
//     timestamp: u64,
// }

// // ============================================================================
// // SUPPORTED: Optional Non-Key Fields
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct WithOptionalFields {
//     #[key]
//     id: u64,
//     required_name: String,
//     optional_description: Option<String>,
//     optional_count: Option<i32>,
//     optional_tags: Option<Vec<String>>,
// }

// // ============================================================================
// // SUPPORTED: Nested Structures (Non-Key Fields)
// // ============================================================================

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Encode, Decode)]
// struct Address {
//     street: String,
//     city: String,
//     country: String,
//     postal_code: Option<String>,
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Encode, Decode)]
// struct ContactInfo {
//     email: String,
//     phone: Option<String>,
//     address: Address,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UserProfile {
//     #[key]
//     user_id: u64,
//     username: String,
//     contact: ContactInfo,
//     preferences: Vec<String>,
//     metadata: std::collections::HashMap<String, String>,
// }

// // ============================================================================
// // SUPPORTED: Single Enum Variant (Only one variant should have #[key])
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// enum SingleVariantEnum {
//     #[key]
//     Active {
//         user_id: u64,
//     },
//     Inactive {
//         reason: String,
//     },
//     Pending {
//         expires_at: u64,
//     },
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// #[schema(prefix = "session")]
// enum PrefixedSingleVariant {
//     #[key]
//     Online {
//         session_id: String,
//     },
//     Offline {
//         last_seen: u64,
//     },
// }

// // ============================================================================
// // WORKAROUNDS: Composite Keys via String Concatenation
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct CompositeKeyWorkaround {
//     #[key]
//     composite_key: String, // Manually formatted composite key
//     // Original components stored separately for easy access
//     org_id: u64,
//     user_id: u64,
//     session_id: String,
//     data: String,
// }

// impl CompositeKeyWorkaround {
//     pub fn new(org_id: u64, user_id: u64, session_id: String, data: String) -> Self {
//         Self {
//             composite_key: format!("{}::{}::{}", org_id, user_id, session_id),
//             org_id,
//             user_id,
//             session_id,
//             data,
//         }
//     }

//     pub fn org_id(&self) -> u64 {
//         self.org_id
//     }

//     pub fn user_id(&self) -> u64 {
//         self.user_id
//     }

//     pub fn session_id(&self) -> &str {
//         &self.session_id
//     }
// }

// // ============================================================================
// // WORKAROUNDS: Manual Key Implementation
// // ============================================================================

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Encode, Decode)]
// struct ManualKeyImplementation {
//     primary_id: u64,
//     secondary_id: String,
//     data: String,
//     tags: Vec<String>,
// }

// impl ManualKeyImplementation {
//     pub fn new(primary_id: u64, secondary_id: String, data: String, tags: Vec<String>) -> Self {
//         Self {
//             primary_id,
//             secondary_id,
//             data,
//             tags,
//         }
//     }

//     /// Custom key method for complex key generation
//     pub fn key(&self) -> String {
//         format!("manual::{}::{}", self.primary_id, self.secondary_id)
//     }

//     /// Alternative key based on content hash
//     pub fn content_key(&self) -> String {
//         use std::collections::hash_map::DefaultHasher;
//         use std::hash::{Hash, Hasher};

//         let mut hasher = DefaultHasher::new();
//         self.data.hash(&mut hasher);
//         self.tags.hash(&mut hasher);
//         format!("content::{:x}", hasher.finish())
//     }
// }

// // ============================================================================
// // MAIN VALIDATION FUNCTION
// // ============================================================================

// fn main() {
//     println!("=== Netabase Supported Key Types Validation ===\n");

//     test_primitive_keys();
//     test_schema_attributes();
//     test_optional_fields();
//     test_nested_structures();
//     test_enum_variants();
//     test_workarounds();
//     test_serialization();

//     println!("\n🎉 All supported key patterns validated successfully!");
//     print_summary();
// }

// fn test_primitive_keys() {
//     println!("🔑 Testing Primitive Key Types...");

//     // Test all integer types
//     let u64_item = U64Key {
//         id: 18446744073709551615u64,
//         data: "max u64".to_string(),
//     };
//     assert_eq!(u64_item.key().to_string(), "18446744073709551615");

//     let u32_item = U32Key {
//         id: 4294967295u32,
//         name: "max u32".to_string(),
//     };
//     assert_eq!(u32_item.key().to_string(), "4294967295");

//     let u16_item = U16Key {
//         id: 65535u16,
//         value: -1,
//     };
//     assert_eq!(u16_item.key().to_string(), "65535");

//     let u8_item = U8Key {
//         id: 255u8,
//         flag: true,
//     };
//     assert_eq!(u8_item.key().to_string(), "255");

//     let i64_item = I64Key {
//         id: -9223372036854775808i64,
//         content: "min i64".to_string(),
//     };
//     assert_eq!(i64_item.key().to_string(), "-9223372036854775808");

//     let i32_item = I32Key {
//         id: -2147483648i32,
//         metadata: "min i32".to_string(),
//     };
//     assert_eq!(i32_item.key().to_string(), "-2147483648");

//     let i16_item = I16Key {
//         id: -32768i16,
//         description: "min i16".to_string(),
//     };
//     assert_eq!(i16_item.key().to_string(), "-32768");

//     let i8_item = I8Key {
//         id: -128i8,
//         status: "min i8".to_string(),
//     };
//     assert_eq!(i8_item.key().to_string(), "-128");

//     // Test string and bool
//     let string_item = StringKey {
//         identifier: "unique_string_id".to_string(),
//         payload: "some data".to_string(),
//     };
//     assert_eq!(string_item.key().to_string(), "unique_string_id");

//     let bool_item = BoolKey {
//         active: true,
//         info: "active state".to_string(),
//     };
//     assert_eq!(bool_item.key().to_string(), "true");

//     let bool_false_item = BoolKey {
//         active: false,
//         info: "inactive state".to_string(),
//     };
//     assert_eq!(bool_false_item.key().to_string(), "false");

//     println!("  ✅ All primitive key types work correctly");
// }

// fn test_schema_attributes() {
//     println!("⚙️ Testing Schema Attributes...");

//     let prefixed = PrefixedUser {
//         id: 123,
//         name: "Alice".to_string(),
//         email: "alice@example.com".to_string(),
//     };
//     assert_eq!(prefixed.key().to_string(), "user::123");

//     let versioned = VersionedDocument {
//         doc_id: "doc_001".to_string(),
//         title: "Important Document".to_string(),
//         content: "Document content here".to_string(),
//     };
//     assert_eq!(versioned.key().to_string(), "doc_001");

//     let both = PrefixedAndVersioned {
//         message_id: "msg_123".to_string(),
//         sender: "alice".to_string(),
//         content: "Hello world!".to_string(),
//         timestamp: 1640995200,
//     };
//     assert_eq!(both.key().to_string(), "msg::msg_123");

//     println!("  ✅ Schema attributes work correctly");
// }

// fn test_optional_fields() {
//     println!("❓ Testing Optional Fields...");

//     let with_some = WithOptionalFields {
//         id: 456,
//         required_name: "Test Item".to_string(),
//         optional_description: Some("This has a description".to_string()),
//         optional_count: Some(42),
//         optional_tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
//     };
//     assert_eq!(with_some.key().to_string(), "456");

//     let with_none = WithOptionalFields {
//         id: 789,
//         required_name: "Test Item 2".to_string(),
//         optional_description: None,
//         optional_count: None,
//         optional_tags: None,
//     };
//     assert_eq!(with_none.key().to_string(), "789");

//     println!("  ✅ Optional fields work correctly");
// }

// fn test_nested_structures() {
//     println!("🏗️ Testing Nested Structures...");

//     let address = Address {
//         street: "123 Main St".to_string(),
//         city: "Anytown".to_string(),
//         country: "USA".to_string(),
//         postal_code: Some("12345".to_string()),
//     };

//     let contact = ContactInfo {
//         email: "user@example.com".to_string(),
//         phone: Some("+1-555-0123".to_string()),
//         address,
//     };

//     let mut metadata = std::collections::HashMap::new();
//     metadata.insert("theme".to_string(), "dark".to_string());
//     metadata.insert("lang".to_string(), "en".to_string());

//     let profile = UserProfile {
//         user_id: 999,
//         username: "testuser".to_string(),
//         contact,
//         preferences: vec!["email_notifications".to_string(), "dark_mode".to_string()],
//         metadata,
//     };
//     assert_eq!(profile.key().to_string(), "999");

//     println!("  ✅ Nested structures work correctly");
// }

// fn test_enum_variants() {
//     println!("🏷️ Testing Enum Variants...");

//     let active_user = SingleVariantEnum::Active { user_id: 123 };
//     // Note: The exact key format for enums may vary based on implementation
//     println!("  Active user key: {}", active_user.key());

//     let inactive_user = SingleVariantEnum::Inactive {
//         reason: "On vacation".to_string(),
//     };
//     println!("  Inactive user key: {}", inactive_user.key());

//     let prefixed_online = PrefixedSingleVariant::Online {
//         session_id: "sess_abc123".to_string(),
//     };
//     println!("  Prefixed online key: {}", prefixed_online.key());

//     println!("  ✅ Enum variants work correctly");
// }

// fn test_workarounds() {
//     println!("🔧 Testing Workarounds...");

//     // Test composite key workaround
//     let composite = CompositeKeyWorkaround::new(
//         100,
//         200,
//         "session_xyz".to_string(),
//         "composite data".to_string(),
//     );
//     assert_eq!(composite.key().to_string(), "100::200::session_xyz");
//     assert_eq!(composite.org_id(), 100);
//     assert_eq!(composite.user_id(), 200);
//     assert_eq!(composite.session_id(), "session_xyz");

//     // Test manual key implementation
//     let manual = ManualKeyImplementation::new(
//         300,
//         "manual_key".to_string(),
//         "manual data".to_string(),
//         vec!["tag1".to_string(), "tag2".to_string()],
//     );
//     assert_eq!(manual.key(), "manual::300::manual_key");

//     let content_key = manual.content_key();
//     assert!(content_key.starts_with("content::"));

//     println!("  ✅ Workarounds function correctly");
// }

// fn test_serialization() {
//     println!("🔄 Testing Serialization...");

//     // Test serialization roundtrip for various types
//     let string_key = StringKey {
//         identifier: "serialize_test".to_string(),
//         payload: "test payload".to_string(),
//     };

//     let serialized = bincode::serialize(&string_key).expect("Serialization failed");
//     let deserialized: StringKey =
//         bincode::deserialize(&serialized).expect("Deserialization failed");
//     assert_eq!(string_key, deserialized);
//     assert_eq!(string_key.key().to_string(), deserialized.key().to_string());

//     // Test prefixed schema serialization
//     let prefixed = PrefixedUser {
//         id: 999,
//         name: "Serialization Test".to_string(),
//         email: "test@example.com".to_string(),
//     };

//     let serialized = bincode::serialize(&prefixed).expect("Serialization failed");
//     let deserialized: PrefixedUser =
//         bincode::deserialize(&serialized).expect("Deserialization failed");
//     assert_eq!(prefixed, deserialized);
//     assert_eq!(prefixed.key().to_string(), deserialized.key().to_string());

//     println!("  ✅ Serialization works correctly");
// }

// fn print_summary() {
//     println!("📊 VALIDATION SUMMARY:");
//     println!("========================");
//     println!("✅ SUPPORTED Key Types:");
//     println!("   - Primitive integers: u8, u16, u32, u64, i8, i16, i32, i64");
//     println!("   - String keys");
//     println!("   - Boolean keys");
//     println!("   - Single #[key] field per struct");
//     println!("");
//     println!("✅ SUPPORTED Schema Features:");
//     println!("   - #[schema(prefix = \"name\")] attribute");
//     println!("   - #[schema(version = \"v1\")] attribute");
//     println!("   - Combined prefix and version");
//     println!("   - Optional non-key fields");
//     println!("   - Nested structures in non-key fields");
//     println!("   - Collections in non-key fields");
//     println!("");
//     println!("✅ SUPPORTED Enum Features:");
//     println!("   - Single variant with #[key] field");
//     println!("   - Multiple variants (only one with #[key])");
//     println!("   - Prefixed enums");
//     println!("");
//     println!("✅ WORKAROUNDS for Unsupported Features:");
//     println!("   - Composite keys via String concatenation");
//     println!("   - Manual key() method implementation");
//     println!("   - Content-based key generation");
//     println!("");
//     println!("⚠️ LIMITATIONS (Use workarounds above):");
//     println!("   - Multiple #[key] fields per struct");
//     println!("   - Collection types as key fields");
//     println!("   - Option<T> as key fields");
//     println!("   - Custom structs as key fields");
//     println!("   - Multiple enum variants with #[key]");
//     println!("");
//     println!("🎯 ALL SUPPORTED PATTERNS VALIDATED!");
// }

// // #[cfg(test)]
// // mod tests {
// //     use super::*;

// //     #[test]
// //     fn test_primitive_key_types() {
// //         let u64_key = U64Key {
// //             id: 123,
// //             data: "test".to_string(),
// //         };
// //         assert_eq!(u64_key.key().to_string(), "123");

// //         let string_key = StringKey {
// //             identifier: "test_id".to_string(),
// //             payload: "payload".to_string(),
// //         };
// //         assert_eq!(string_key.key().to_string(), "test_id");

// //         let bool_key = BoolKey {
// //             active: true,
// //             info: "test".to_string(),
// //         };
// //         assert_eq!(bool_key.key().to_string(), "true");
// //     }

// //     #[test]
// //     fn test_schema_attributes() {
// //         let prefixed = PrefixedUser {
// //             id: 456,
// //             name: "Test".to_string(),
// //             email: "test@example.com".to_string(),
// //         };
// //         assert_eq!(prefixed.key().to_string(), "user::456");

// //         let versioned = VersionedDocument {
// //             doc_id: "test_doc".to_string(),
// //             title: "Test".to_string(),
// //             content: "Content".to_string(),
// //         };
// //         assert_eq!(versioned.key().to_string(), "test_doc");
// //     }

// //     #[test]
// //     fn test_optional_fields() {
// //         let with_opts = WithOptionalFields {
// //             id: 789,
// //             required_name: "Test".to_string(),
// //             optional_description: Some("Desc".to_string()),
// //             optional_count: None,
// //             optional_tags: Some(vec!["tag".to_string()]),
// //         };
// //         assert_eq!(with_opts.key().to_string(), "789");
// //     }

// //     #[test]
// //     fn test_workarounds() {
// //         let composite =
// //             CompositeKeyWorkaround::new(1, 2, "session".to_string(), "data".to_string());
// //         assert_eq!(composite.key().to_string(), "1::2::session");

// //         let manual =
// //             ManualKeyImplementation::new(10, "manual".to_string(), "data".to_string(), vec![]);
// //         assert_eq!(manual.key(), "manual::10::manual");
// //     }

// //     #[test]
// //     fn test_serialization_roundtrip() {
// //         let original = StringKey {
// //             identifier: "roundtrip_test".to_string(),
// //             payload: "test data".to_string(),
// //         };

// //         let serialized = bincode::serialize(&original).unwrap();
// //         let deserialized: StringKey = bincode::deserialize(&serialized).unwrap();

// //         assert_eq!(original, deserialized);
// //         assert_eq!(original.key().to_string(), deserialized.key().to_string());
// //     }

// //     #[test]
// //     fn test_key_consistency() {
// //         let item1 = U64Key {
// //             id: 100,
// //             data: "data1".to_string(),
// //         };
// //         let item2 = U64Key {
// //             id: 100,
// //             data: "data2".to_string(),
// //         };

// //         // Same key field should produce same key, regardless of other fields
// //         assert_eq!(item1.key().to_string(), item2.key().to_string());
// //     }

// //     #[test]
// //     fn test_unicode_support() {
// //         let unicode_key = StringKey {
// //             identifier: "🔑_test_🌍".to_string(),
// //             payload: "unicode test".to_string(),
// //         };
// //         assert_eq!(unicode_key.key().to_string(), "🔑_test_🌍");

// //         // Test serialization with unicode
// //         let serialized = bincode::serialize(&unicode_key).unwrap();
// //         let deserialized: StringKey = bincode::deserialize(&serialized).unwrap();
// //         assert_eq!(
// //             unicode_key.key().to_string(),
// //             deserialized.key().to_string()
// //         );
// //     }
// // }
