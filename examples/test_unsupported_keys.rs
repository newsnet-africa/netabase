fn main() {}

// //! Test Unsupported Key Patterns
// //!
// //! This file contains examples of UNSUPPORTED key patterns that should
// //! generate clear compilation errors. These examples demonstrate what
// //! NOT to do and ensure users get helpful error messages.
// //!
// //! Note: This file is designed to fail compilation and show error messages.
// //! It serves as documentation for unsupported patterns.

// use bincode::{Decode, Encode};
// use netabase::NetabaseSchema;
// use serde::{Deserialize, Serialize};

// // ============================================================================
// // UNSUPPORTED: Multiple Key Fields (Composite Keys)
// // ============================================================================

// // This should fail with a clear error message about composite keys not being supported
// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UnsupportedCompositeKey {
//     #[key]
//     primary_id: u64,
//     #[key] // ERROR: Multiple #[key] fields not supported
//     secondary_id: String,
//     data: String,
// }

// // Another example of unsupported composite keys
// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UnsupportedTripleKey {
//     #[key]
//     part_a: String,
//     #[key] // ERROR: Multiple #[key] fields not supported
//     part_b: u32,
//     #[key] // ERROR: Multiple #[key] fields not supported
//     part_c: bool,
//     data: String,
// }

// // ============================================================================
// // UNSUPPORTED: Complex Key Types
// // ============================================================================

// // Vec as key field - should fail
// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UnsupportedVecKey {
//     #[key]
//     tags: Vec<String>, // ERROR: Collection types not supported as keys
//     data: String,
// }

// // HashMap as key field - should fail
// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UnsupportedHashMapKey {
//     #[key]
//     metadata: std::collections::HashMap<String, String>, // ERROR: Collection types not supported
//     data: String,
// }

// // Option as key field - should fail
// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UnsupportedOptionKey {
//     #[key]
//     optional_id: Option<u64>, // ERROR: Option types not supported as keys
//     data: String,
// }

// // Array as key field - should fail
// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UnsupportedArrayKey {
//     #[key]
//     bytes: [u8; 32], // ERROR: Array types not supported as keys
//     data: String,
// }

// // Tuple as key field - should fail
// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UnsupportedTupleKey {
//     #[key]
//     coordinate: (f64, f64), // ERROR: Tuple types not supported as keys
//     data: String,
// }

// // Custom struct as key field - should fail
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Encode, Decode)]
// struct CustomId {
//     namespace: String,
//     id: u64,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UnsupportedCustomStructKey {
//     #[key]
//     custom_id: CustomId, // ERROR: Custom struct types not supported as keys
//     data: String,
// }

// // Generic type as key field - should fail
// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UnsupportedGenericKey<T> {
//     #[key]
//     generic_field: T, // ERROR: Generic types not supported as keys
//     data: String,
// }

// // ============================================================================
// // UNSUPPORTED: Multiple Enum Variants with Keys
// // ============================================================================

// // Multiple enum variants with keys - should fail
// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// enum UnsupportedMultiVariantEnum {
//     #[key]
//     Variant1 {
//         id: u64,
//     }, // ERROR: Multiple variants with keys not supported
//     #[key]
//     Variant2 {
//         name: String,
//     }, // ERROR: Multiple variants with keys not supported
//     DataVariant {
//         data: String,
//     },
// }

// // ============================================================================
// // UNSUPPORTED: Complex Nested Structures as Keys
// // ============================================================================

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Encode, Decode)]
// struct NestedStruct {
//     inner: String,
//     value: u32,
// }

// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct UnsupportedNestedKey {
//     #[key]
//     nested: NestedStruct, // ERROR: Nested structs not supported as keys
//     data: String,
// }

// // ============================================================================
// // EXAMPLES OF WHAT TO DO INSTEAD
// // ============================================================================

// // ✅ SUPPORTED: Single primitive key
// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct SupportedSingleKey {
//     #[key]
//     id: u64, // ✅ Single primitive key works
//     primary_data: String,
//     secondary_data: String,
// }

// // ✅ SUPPORTED: String key
// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct SupportedStringKey {
//     #[key]
//     identifier: String, // ✅ String keys work
//     data: String,
// }

// // ✅ WORKAROUND: Composite key via String concatenation
// #[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
// struct CompositeKeyWorkaround {
//     #[key]
//     composite_key: String, // ✅ Use String and format manually
//     primary_id: u64,
//     secondary_id: String,
//     data: String,
// }

// impl CompositeKeyWorkaround {
//     pub fn new(primary_id: u64, secondary_id: String, data: String) -> Self {
//         Self {
//             composite_key: format!("{}::{}", primary_id, secondary_id),
//             primary_id,
//             secondary_id,
//             data,
//         }
//     }
// }

// // ✅ WORKAROUND: Custom key method (when macro doesn't work)
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Encode, Decode)]
// struct CustomKeyMethod {
//     primary_id: u64,
//     secondary_id: String,
//     data: String,
// }

// // Manual implementation when macro can't handle complexity
// impl CustomKeyMethod {
//     pub fn key(&self) -> String {
//         format!("{}::{}", self.primary_id, self.secondary_id)
//     }
// }

// // ============================================================================
// // MAIN FUNCTION (Won't compile due to errors above)
// // ============================================================================

// fn main() {
//     // This main function demonstrates usage patterns
//     // It won't compile due to the unsupported patterns above

//     println!("=== Testing Error Messages for Unsupported Key Patterns ===");

//     // If you want to test error messages, uncomment ONE unsupported
//     // pattern at a time to see the specific error message.

//     // Example of supported pattern:
//     let supported = SupportedSingleKey {
//         id: 123,
//         primary_data: "data1".to_string(),
//         secondary_data: "data2".to_string(),
//     };
//     println!("Supported key: {}", supported.key());

//     // Example of workaround:
//     let workaround = CompositeKeyWorkaround::new(123, "abc".to_string(), "some data".to_string());
//     println!("Workaround composite key: {}", workaround.key());

//     let custom = CustomKeyMethod {
//         primary_id: 456,
//         secondary_id: "def".to_string(),
//         data: "custom data".to_string(),
//     };
//     println!("Custom key method: {}", custom.key());
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_supported_patterns() {
//         // Test that supported patterns work
//         let item = SupportedSingleKey {
//             id: 999,
//             primary_data: "test".to_string(),
//             secondary_data: "data".to_string(),
//         };
//         assert_eq!(item.key(), "999");
//     }

//     #[test]
//     fn test_workaround_patterns() {
//         // Test workaround patterns
//         let composite = CompositeKeyWorkaround::new(123, "test".to_string(), "data".to_string());
//         assert_eq!(composite.key(), "123::test");

//         let custom = CustomKeyMethod {
//             primary_id: 789,
//             secondary_id: "custom".to_string(),
//             data: "test data".to_string(),
//         };
//         assert_eq!(custom.key(), "789::custom");
//     }
// }

// // ============================================================================
// // DOCUMENTATION COMMENTS
// // ============================================================================

// /*
// COMPILATION ERRORS YOU SHOULD SEE:

// 1. For UnsupportedCompositeKey:
//    "UNSUPPORTED: Multiple #[key] fields found: [primary_id, secondary_id].
//    Current netabase implementation only supports ONE key field per struct.
//    Composite keys are not yet implemented.
//    Please use only one #[key] field, or implement a custom key() method."

// 2. For UnsupportedVecKey:
//    "UNSUPPORTED: Collection type 'Vec' cannot be used as a key field.
//    Only primitive types (u8, u16, u32, u64, i8, i16, i32, i64, String, bool)
//    are currently supported as key fields.
//    Consider using a String representation or implement a custom key() method."

// 3. For UnsupportedOptionKey:
//    "UNSUPPORTED: Option<T> cannot be used as a key field.
//    Key fields must always have a value.
//    Consider using a default value or implement a custom key() method."

// 4. For UnsupportedMultiVariantEnum:
//    "UNSUPPORTED: Multiple enum variants with #[key] fields are not yet supported.
//    Current implementation only supports single-variant key extraction.
//    Please use only one variant with a #[key] field, or implement a custom key() method."

// These error messages help users understand:
// - What they're trying to do isn't supported yet
// - What IS supported (primitive types)
// - How to work around the limitation (custom key() method or String formatting)
// - That these features may be added in the future

// WORKAROUNDS:
// 1. Use String keys and format composite data manually
// 2. Implement custom key() methods when the macro can't handle your use case
// 3. Stick to primitive types (u8, u16, u32, u64, i8, i16, i32, i64, String, bool) for key fields
// 4. Use only one #[key] field per struct
// */
