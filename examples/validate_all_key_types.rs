fn main() {}

// //! Comprehensive Netabase Key Types Validation
// //!
// //! This example validates all supported key types in the netabase system:
// //! - Single field keys (primitive types)
// //! - Composite field keys (multiple fields)
// //! - String keys
// //! - Numeric keys (u8, u16, u32, u64, i8, i16, i32, i64, f32, f64)
// //! - Custom key types
// //! - Schema prefixes and versions
// //! - Optional field handling
// //! - Enum variants with keys
// //! - Complex nested structures

// use bincode::{Decode, Encode};
// use netabase::NetabaseSchema;
// use serde::{Deserialize, Serialize};

// // ============================================================================
// // PRIMITIVE KEY TYPES
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
//     data: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct U16Key {
//     #[key]
//     id: u16,
//     data: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct U8Key {
//     #[key]
//     id: u8,
//     data: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct I64Key {
//     #[key]
//     id: i64,
//     data: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct I32Key {
//     #[key]
//     id: i32,
//     data: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct I16Key {
//     #[key]
//     id: i16,
//     data: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct I8Key {
//     #[key]
//     id: i8,
//     data: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct StringKey {
//     #[key]
//     id: String,
//     data: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct BoolKey {
//     #[key]
//     active: bool,
//     data: String,
// }

// // ============================================================================
// // COMPOSITE KEYS
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct DualKey {
//     #[key]
//     primary_id: u64,
//     #[key]
//     secondary_id: String,
//     data: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct TripleKey {
//     #[key]
//     org_id: u64,
//     #[key]
//     user_id: u64,
//     #[key]
//     session_id: String,
//     data: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct MixedTypeCompositeKey {
//     #[key]
//     numeric_part: u32,
//     #[key]
//     string_part: String,
//     #[key]
//     bool_part: bool,
//     data: String,
// }

// // ============================================================================
// // SCHEMA ATTRIBUTES (PREFIX, VERSION, SEPARATOR)
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// #[schema(prefix = "user")]
// struct PrefixedUser {
//     #[key]
//     id: u64,
//     name: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// #[schema(version = "v1")]
// struct VersionedDocument {
//     #[key]
//     doc_id: String,
//     content: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// #[schema(prefix = "msg", version = "v2")]
// struct PrefixedAndVersioned {
//     #[key]
//     id: String,
//     content: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// #[schema(separator = "|")]
// struct CustomSeparator {
//     #[key]
//     part_a: String,
//     #[key]
//     part_b: String,
//     data: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// #[schema(prefix = "complex", version = "v1", separator = "_")]
// struct ComplexSchema {
//     #[key]
//     region: String,
//     #[key]
//     cluster: String,
//     #[key]
//     node_id: u32,
//     status: String,
// }

// // ============================================================================
// // OPTIONAL FIELDS
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct WithOptionalFields {
//     #[key]
//     id: u64,
//     required_field: String,
//     optional_field: Option<String>,
//     optional_number: Option<i32>,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct OptionalCompositeKey {
//     #[key]
//     primary: String,
//     #[key]
//     secondary: u64,
//     optional_data: Option<Vec<String>>,
//     metadata: Option<String>,
// }

// // ============================================================================
// // ENUM TYPES
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// enum StatusEnum {
//     #[key]
//     Active { user_id: u64 },
//     #[key]
//     Inactive { user_id: u64, reason: String },
//     #[key]
//     Pending { user_id: u64, expires_at: u64 },
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// #[schema(prefix = "state")]
// enum PrefixedEnum {
//     #[key]
//     Online { session_id: String },
//     #[key]
//     Offline { last_seen: u64 },
// }

// // ============================================================================
// // NESTED STRUCTURES
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
//     name: String,
//     address: Address,
//     preferences: Vec<String>,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct NestedCompositeKey {
//     #[key]
//     organization: String,
//     #[key]
//     department: String,
//     #[key]
//     employee_id: u64,
//     personal_info: UserProfile,
// }

// // ============================================================================
// // EDGE CASES
// // ============================================================================

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct EmptyDataStruct {
//     #[key]
//     id: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UnicodeKey {
//     #[key]
//     emoji_id: String, // Will test with unicode content
//     description: String,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct VeryLongCompositeKey {
//     #[key]
//     region: String,
//     #[key]
//     availability_zone: String,
//     #[key]
//     cluster_id: String,
//     #[key]
//     node_id: u64,
//     #[key]
//     instance_id: String,
//     #[key]
//     container_id: String,
//     metadata: String,
// }

// // ============================================================================
// // MAIN VALIDATION FUNCTION
// // ============================================================================

// fn main() {
//     println!("=== Netabase Comprehensive Key Types Validation ===\n");

//     // Test primitive types
//     test_primitive_keys();

//     // Test composite keys
//     test_composite_keys();

//     // Test schema attributes
//     test_schema_attributes();

//     // Test optional fields
//     test_optional_fields();

//     // Test enum types
//     test_enum_types();

//     // Test nested structures
//     test_nested_structures();

//     // Test edge cases
//     test_edge_cases();

//     // Test serialization for all types
//     test_serialization_roundtrips();

//     println!("\n🎉 All key type validations passed successfully!");
//     print_summary();
// }

// fn test_primitive_keys() {
//     println!("📋 Testing Primitive Key Types...");

//     // u64
//     let u64_item = U64Key {
//         id: 18446744073709551615u64,
//         data: "max u64".to_string(),
//     };
//     assert_eq!(u64_item.key(), "18446744073709551615");
//     println!("  ✓ u64 key: {}", u64_item.key());

//     // u32
//     let u32_item = U32Key {
//         id: 4294967295u32,
//         data: "max u32".to_string(),
//     };
//     assert_eq!(u32_item.key(), "4294967295");
//     println!("  ✓ u32 key: {}", u32_item.key());

//     // u16
//     let u16_item = U16Key {
//         id: 65535u16,
//         data: "max u16".to_string(),
//     };
//     assert_eq!(u16_item.key(), "65535");
//     println!("  ✓ u16 key: {}", u16_item.key());

//     // u8
//     let u8_item = U8Key {
//         id: 255u8,
//         data: "max u8".to_string(),
//     };
//     assert_eq!(u8_item.key(), "255");
//     println!("  ✓ u8 key: {}", u8_item.key());

//     // i64
//     let i64_item = I64Key {
//         id: -9223372036854775808i64,
//         data: "min i64".to_string(),
//     };
//     assert_eq!(i64_item.key(), "-9223372036854775808");
//     println!("  ✓ i64 key: {}", i64_item.key());

//     // i32
//     let i32_item = I32Key {
//         id: -2147483648i32,
//         data: "min i32".to_string(),
//     };
//     assert_eq!(i32_item.key(), "-2147483648");
//     println!("  ✓ i32 key: {}", i32_item.key());

//     // i16
//     let i16_item = I16Key {
//         id: -32768i16,
//         data: "min i16".to_string(),
//     };
//     assert_eq!(i16_item.key(), "-32768");
//     println!("  ✓ i16 key: {}", i16_item.key());

//     // i8
//     let i8_item = I8Key {
//         id: -128i8,
//         data: "min i8".to_string(),
//     };
//     assert_eq!(i8_item.key(), "-128");
//     println!("  ✓ i8 key: {}", i8_item.key());

//     // String
//     let string_item = StringKey {
//         id: "test_string_key".to_string(),
//         data: "data".to_string(),
//     };
//     assert_eq!(string_item.key(), "test_string_key");
//     println!("  ✓ String key: {}", string_item.key());

//     // Bool
//     let bool_item = BoolKey {
//         active: true,
//         data: "active".to_string(),
//     };
//     assert_eq!(bool_item.key(), "true");
//     println!("  ✓ bool key: {}", bool_item.key());

//     println!("  ✅ All primitive key types work correctly\n");
// }

// fn test_composite_keys() {
//     println!("🔗 Testing Composite Key Types...");

//     // Dual key
//     let dual = DualKey {
//         primary_id: 123,
//         secondary_id: "abc".to_string(),
//         data: "dual key test".to_string(),
//     };
//     assert_eq!(dual.key(), "123::abc");
//     println!("  ✓ Dual key: {}", dual.key());

//     // Triple key
//     let triple = TripleKey {
//         org_id: 1,
//         user_id: 2,
//         session_id: "sess_123".to_string(),
//         data: "triple key test".to_string(),
//     };
//     assert_eq!(triple.key(), "1::2::sess_123");
//     println!("  ✓ Triple key: {}", triple.key());

//     // Mixed type composite key
//     let mixed = MixedTypeCompositeKey {
//         numeric_part: 42,
//         string_part: "mixed".to_string(),
//         bool_part: false,
//         data: "mixed types".to_string(),
//     };
//     assert_eq!(mixed.key(), "42::mixed::false");
//     println!("  ✓ Mixed type composite key: {}", mixed.key());

//     println!("  ✅ All composite key types work correctly\n");
// }

// fn test_schema_attributes() {
//     println!("⚙️ Testing Schema Attributes...");

//     // Prefix
//     let prefixed = PrefixedUser {
//         id: 456,
//         name: "Alice".to_string(),
//     };
//     assert_eq!(prefixed.key(), "user::456");
//     println!("  ✓ Prefixed schema: {}", prefixed.key());

//     // Version
//     let versioned = VersionedDocument {
//         doc_id: "doc_001".to_string(),
//         content: "content".to_string(),
//     };
//     assert_eq!(versioned.key(), "doc_001");
//     println!("  ✓ Versioned schema: {}", versioned.key());

//     // Prefix and version
//     let both = PrefixedAndVersioned {
//         id: "msg_001".to_string(),
//         content: "hello".to_string(),
//     };
//     assert_eq!(both.key(), "msg::msg_001");
//     println!("  ✓ Prefix + version schema: {}", both.key());

//     // Custom separator
//     let custom_sep = CustomSeparator {
//         part_a: "A".to_string(),
//         part_b: "B".to_string(),
//         data: "separated".to_string(),
//     };
//     assert_eq!(custom_sep.key(), "A|B");
//     println!("  ✓ Custom separator: {}", custom_sep.key());

//     // Complex schema
//     let complex = ComplexSchema {
//         region: "us-east".to_string(),
//         cluster: "prod".to_string(),
//         node_id: 42,
//         status: "running".to_string(),
//     };
//     assert_eq!(complex.key(), "complex::us-east_prod_42");
//     println!("  ✓ Complex schema: {}", complex.key());

//     println!("  ✅ All schema attributes work correctly\n");
// }

// fn test_optional_fields() {
//     println!("❓ Testing Optional Fields...");

//     // With optional fields
//     let with_opts = WithOptionalFields {
//         id: 789,
//         required_field: "required".to_string(),
//         optional_field: Some("optional".to_string()),
//         optional_number: None,
//     };
//     assert_eq!(with_opts.key(), "789");
//     println!("  ✓ Optional fields (Some/None): {}", with_opts.key());

//     // Optional composite key
//     let opt_composite = OptionalCompositeKey {
//         primary: "primary".to_string(),
//         secondary: 999,
//         optional_data: Some(vec!["a".to_string(), "b".to_string()]),
//         metadata: None,
//     };
//     assert_eq!(opt_composite.key(), "primary::999");
//     println!(
//         "  ✓ Optional fields with composite key: {}",
//         opt_composite.key()
//     );

//     println!("  ✅ Optional fields work correctly\n");
// }

// fn test_enum_types() {
//     println!("🏷️ Testing Enum Types...");

//     // Status enum variants
//     let active = StatusEnum::Active { user_id: 123 };
//     println!("  ✓ Enum Active variant key: {}", active.key());

//     let inactive = StatusEnum::Inactive {
//         user_id: 456,
//         reason: "vacation".to_string(),
//     };
//     println!("  ✓ Enum Inactive variant key: {}", inactive.key());

//     let pending = StatusEnum::Pending {
//         user_id: 789,
//         expires_at: 1234567890,
//     };
//     println!("  ✓ Enum Pending variant key: {}", pending.key());

//     // Prefixed enum
//     let online = PrefixedEnum::Online {
//         session_id: "sess_abc".to_string(),
//     };
//     assert!(online.key().starts_with("state::"));
//     println!("  ✓ Prefixed enum Online: {}", online.key());

//     let offline = PrefixedEnum::Offline {
//         last_seen: 1234567890,
//     };
//     assert!(offline.key().starts_with("state::"));
//     println!("  ✓ Prefixed enum Offline: {}", offline.key());

//     println!("  ✅ Enum types work correctly\n");
// }

// fn test_nested_structures() {
//     println!("🏗️ Testing Nested Structures...");

//     let address = Address {
//         street: "123 Main St".to_string(),
//         city: "Anytown".to_string(),
//         country: "USA".to_string(),
//     };

//     let profile = UserProfile {
//         user_id: 12345,
//         name: "John Doe".to_string(),
//         address: address.clone(),
//         preferences: vec!["dark_mode".to_string(), "notifications".to_string()],
//     };
//     assert_eq!(profile.key(), "12345");
//     println!("  ✓ Nested structure: {}", profile.key());

//     let nested_composite = NestedCompositeKey {
//         organization: "ACME Corp".to_string(),
//         department: "Engineering".to_string(),
//         employee_id: 98765,
//         personal_info: profile,
//     };
//     assert_eq!(nested_composite.key(), "ACME Corp::Engineering::98765");
//     println!("  ✓ Nested composite key: {}", nested_composite.key());

//     println!("  ✅ Nested structures work correctly\n");
// }

// fn test_edge_cases() {
//     println!("⚠️ Testing Edge Cases...");

//     // Empty data struct
//     let empty = EmptyDataStruct {
//         id: "only_key".to_string(),
//     };
//     assert_eq!(empty.key(), "only_key");
//     println!("  ✓ Empty data struct: {}", empty.key());

//     // Unicode key
//     let unicode = UnicodeKey {
//         emoji_id: "🚀_rocket_🌟".to_string(),
//         description: "unicode test".to_string(),
//     };
//     assert_eq!(unicode.key(), "🚀_rocket_🌟");
//     println!("  ✓ Unicode key: {}", unicode.key());

//     // Very long composite key
//     let long_key = VeryLongCompositeKey {
//         region: "us-west-2".to_string(),
//         availability_zone: "us-west-2a".to_string(),
//         cluster_id: "prod-cluster-001".to_string(),
//         node_id: 42,
//         instance_id: "i-0123456789abcdef0".to_string(),
//         container_id: "container-abcd1234".to_string(),
//         metadata: "long key test".to_string(),
//     };
//     let expected_long_key =
//         "us-west-2::us-west-2a::prod-cluster-001::42::i-0123456789abcdef0::container-abcd1234";
//     assert_eq!(long_key.key(), expected_long_key);
//     println!("  ✓ Very long composite key: {}", long_key.key());

//     println!("  ✅ Edge cases work correctly\n");
// }

// fn test_serialization_roundtrips() {
//     println!("🔄 Testing Serialization Roundtrips...");

//     // Test a few representative types
//     let string_key = StringKey {
//         id: "serialize_test".to_string(),
//         data: "test data".to_string(),
//     };

//     let serialized = bincode::serialize(&string_key).expect("Serialization failed");
//     let deserialized: StringKey =
//         bincode::deserialize(&serialized).expect("Deserialization failed");
//     assert_eq!(string_key, deserialized);
//     assert_eq!(string_key.key(), deserialized.key());
//     println!("  ✓ String key serialization roundtrip");

//     let composite = DualKey {
//         primary_id: 123,
//         secondary_id: "test".to_string(),
//         data: "composite test".to_string(),
//     };

//     let serialized = bincode::serialize(&composite).expect("Serialization failed");
//     let deserialized: DualKey = bincode::deserialize(&serialized).expect("Deserialization failed");
//     assert_eq!(composite, deserialized);
//     assert_eq!(composite.key(), deserialized.key());
//     println!("  ✓ Composite key serialization roundtrip");

//     let prefixed = PrefixedUser {
//         id: 456,
//         name: "Serialization Test".to_string(),
//     };

//     let serialized = bincode::serialize(&prefixed).expect("Serialization failed");
//     let deserialized: PrefixedUser =
//         bincode::deserialize(&serialized).expect("Deserialization failed");
//     assert_eq!(prefixed, deserialized);
//     assert_eq!(prefixed.key(), deserialized.key());
//     println!("  ✓ Prefixed schema serialization roundtrip");

//     println!("  ✅ All serialization roundtrips work correctly\n");
// }

// fn print_summary() {
//     println!("📊 VALIDATION SUMMARY:");
//     println!("========================");
//     println!("✅ Primitive Key Types:");
//     println!("   - u8, u16, u32, u64");
//     println!("   - i8, i16, i32, i64");
//     println!("   - String");
//     println!("   - bool");
//     println!("");
//     println!("✅ Composite Key Types:");
//     println!("   - Dual keys (2 fields)");
//     println!("   - Triple keys (3 fields)");
//     println!("   - Mixed type keys");
//     println!("   - Very long composite keys (6+ fields)");
//     println!("");
//     println!("✅ Schema Attributes:");
//     println!("   - prefix attribute");
//     println!("   - version attribute");
//     println!("   - separator attribute");
//     println!("   - Complex combinations");
//     println!("");
//     println!("✅ Optional Fields:");
//     println!("   - Some/None values");
//     println!("   - Optional in composite keys");
//     println!("");
//     println!("✅ Enum Types:");
//     println!("   - Multiple variants with keys");
//     println!("   - Prefixed enums");
//     println!("");
//     println!("✅ Nested Structures:");
//     println!("   - Structs containing other structs");
//     println!("   - Vec and Option fields");
//     println!("");
//     println!("✅ Edge Cases:");
//     println!("   - Empty data structs");
//     println!("   - Unicode keys");
//     println!("   - Very long keys");
//     println!("");
//     println!("✅ Serialization:");
//     println!("   - bincode roundtrips");
//     println!("   - Key consistency after serialization");
//     println!("");
//     println!("🎯 ALL NETABASE KEY TYPES VALIDATED SUCCESSFULLY!");
// }

// // #[cfg(test)]
// // mod tests {
// //     use super::*;

// //     #[test]
// //     #[ignore]
// //     fn test_all_primitive_types() {
// //         test_primitive_keys();
// //     }

// //     #[test]
// //     #[ignore]
// //     fn test_all_composite_types() {
// //         test_composite_keys();
// //     }

// //     #[test]
// //     #[ignore]
// //     fn test_all_schema_attributes() {
// //         test_schema_attributes();
// //     }

// //     #[test]
// //     #[ignore]
// //     fn test_all_optional_fields() {
// //         test_optional_fields();
// //     }

// //     #[test]
// //     #[ignore]
// //     fn test_all_enum_types() {
// //         test_enum_types();
// //     }

// //     #[test]
// //     #[ignore]
// //     fn test_all_nested_structures() {
// //         test_nested_structures();
// //     }

// //     #[test]
// //     #[ignore]
// //     fn test_all_edge_cases() {
// //         test_edge_cases();
// //     }

// //     #[test]
// //     #[ignore]
// //     fn test_all_serialization() {
// //         test_serialization_roundtrips();
// //     }

// //     #[test]
// //     #[ignore]
// //     fn test_key_consistency() {
// //         // Test that same data produces same key
// //         let item1 = StringKey {
// //             id: "consistency_test".to_string(),
// //             data: "data1".to_string(),
// //         };

// //         let item2 = StringKey {
// //             id: "consistency_test".to_string(),
// //             data: "data2".to_string(), // Different data, same key field
// //         };

// //         assert_eq!(item1.key(), item2.key());
// //     }

// //     #[test]
// //     #[ignore]
// //     fn test_unicode_keys() {
// //         let unicode_item = UnicodeKey {
// //             emoji_id: "🔑🌍💫".to_string(),
// //             description: "test".to_string(),
// //         };

// //         assert_eq!(unicode_item.key(), "🔑🌍💫");

// //         // Test serialization with unicode
// //         let serialized = bincode::serialize(&unicode_item).unwrap();
// //         let deserialized: UnicodeKey = bincode::deserialize(&serialized).unwrap();
// //         assert_eq!(unicode_item.key(), deserialized.key());
// //     }
// // }
