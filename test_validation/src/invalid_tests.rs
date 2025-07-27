//! Invalid test cases that should fail compilation
//!
//! These tests demonstrate cases that should be rejected by the macro validation.
//! Most are commented out to prevent compilation failures, but can be uncommented
//! to test that the validation is working correctly.

use netabase_macros::NetabaseSchema;

// INVALID: Struct with multiple key fields - should fail compilation
/*
#[derive(NetabaseSchema)]
struct InvalidStructMultipleKeys {
    #[key]
    id: String,
    #[key]
    email: String,  // ERROR: Structs can have at most 1 key field
    name: String,
}
*/

// INVALID: Enum variant with multiple key fields - should fail compilation
/*
#[derive(NetabaseSchema)]
enum InvalidEnumMultipleKeysPerVariant {
    User {
        #[key]
        id: String,
        #[key]
        email: String,  // ERROR: Enum variants can have at most 1 key field
        name: String,
    },
}
*/

// INVALID: Enum variant with no key fields - should fail compilation
/*
#[derive(NetabaseSchema)]
enum InvalidEnumNoKeys {
    User {
        id: String,     // ERROR: No key field - each variant needs exactly 1 key
        name: String,
    },
}
*/

// INVALID: Mixed - some variants have keys, some don't (without top-level closure)
/*
#[derive(NetabaseSchema)]
enum InvalidEnumMixedKeys {
    User {
        #[key]
        id: String,
        name: String,
    },
    Admin {
        // ERROR: This variant has no key field
        admin_name: String,
        permissions: Vec<String>,
    },
}
*/

// VALID: Enum with top-level key closure - variants don't need individual keys
#[derive(NetabaseSchema)]
#[key = |item| match item {
    ValidEnumWithTopLevelKey::User { name, .. } => format!("user_{}", name),
    ValidEnumWithTopLevelKey::Admin { admin_name, .. } => format!("admin_{}", admin_name),
}]
enum ValidEnumWithTopLevelKey {
    User {
        // No key needed here because enum has top-level key
        name: String,
        email: String,
    },
    Admin {
        // No key needed here either
        admin_name: String,
        permissions: Vec<String>,
    },
}

// Test cases that should work (for comparison)
#[derive(NetabaseSchema)]
struct ValidStruct {
    #[key]
    id: String,
    name: String,
}

#[derive(NetabaseSchema)]
enum ValidEnum {
    User {
        #[key]
        user_id: String,
        name: String,
    },
    Admin {
        #[key]
        admin_id: String,
        permissions: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_cases_compile() {
        // These should compile without issues
        let _struct = ValidStruct {
            id: "123".to_string(),
            name: "Alice".to_string(),
        };

        let _enum = ValidEnum::User {
            user_id: "user123".to_string(),
            name: "Bob".to_string(),
        };

        let _enum_with_closure = ValidEnumWithTopLevelKey::User {
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
        };
    }

    // To test invalid cases, uncomment the structs/enums above and try to compile.
    // They should produce compilation errors with descriptive messages about
    // violating the key field constraints.

    #[test]
    #[ignore] // Ignored by default since these are meant to test compilation failures
    fn test_invalid_cases() {
        // Uncomment the invalid examples above and run:
        // cargo test test_invalid_cases -- --ignored
        //
        // Expected errors:
        // - InvalidStructMultipleKeys: "Schema has multiple key fields: [id, email]. Structs can have at most 1 key field"
        // - InvalidEnumMultipleKeysPerVariant: "Schema has multiple key fields: [id, email]. Enum variants can have at most 1 key field"
        // - InvalidEnumNoKeys: "Schema has no key fields. Each schema must have at least one field marked with #[key]"
        // - InvalidEnumMixedKeys: "Schema has no key fields" for the Admin variant

        println!("To test validation, uncomment the invalid examples and attempt compilation.");
        println!("They should fail with descriptive error messages about key field constraints.");
    }
}

// Additional edge cases for testing

// VALID: Unit enum variant (should not require key)
#[derive(NetabaseSchema)]
enum EnumWithUnitVariant {
    User {
        #[key]
        id: String,
        name: String,
    },
    Anonymous, // Unit variant - no key required
}

// VALID: Tuple struct with key
#[derive(NetabaseSchema)]
struct TupleWithKey(#[key] String, i32, bool);

// INVALID: Tuple struct with multiple keys - should fail
/*
#[derive(NetabaseSchema)]
struct InvalidTupleMultipleKeys(
    #[key] String,
    #[key] i32,    // ERROR: Multiple keys in tuple struct
    bool
);
*/

// Edge case: Empty struct (should probably fail for having no key)
/*
#[derive(NetabaseSchema)]
struct EmptyStruct;
*/

// Edge case: Enum with only unit variants
#[derive(NetabaseSchema)]
#[key = |item| format!("variant_{}", std::mem::discriminant(item) as u8)]
enum OnlyUnitVariants {
    A,
    B,
    C,
}
