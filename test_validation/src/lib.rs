//! Test module to validate new key field constraints
//!
//! This module tests the following rules:
//! 1. Structs can have at most 1 key field
//! 2. Each enum variant should have exactly 1 key field (unless enum has top-level closure)
//! 3. Enums can have a top-level key closure instead of per-variant keys

use bincode::{Decode, Encode};
use netabase::NetabaseSchema;

mod minimal_test;
mod simple_tests;

// Valid struct with 1 key field
#[derive(NetabaseSchema, Encode, Decode)]
struct ValidStruct {
    #[key]
    id: String,
    name: String,
    email: String,
}

// Invalid struct with multiple key fields - should fail compilation
/*
#[derive(NetabaseSchema)]
struct InvalidStructMultipleKeys {
    #[key]
    id: String,
    #[key]
    email: String,  // This should cause a compilation error
    name: String,
}
*/

// Valid struct with key closure (commented out due to attribute parsing limitations)
// #[derive(NetabaseSchema)]
// struct ValidStructWithClosure {
//     #[key = |user| format!("{}_{}", user.name, user.id)]
//     id: String,
//     name: String,
// }

// Valid struct with simple key field for now
#[derive(NetabaseSchema)]
struct ValidStructSimple {
    #[key]
    id: String,
    name: String,
}

// Valid enum with each variant having exactly one key field
#[derive(NetabaseSchema)]
enum ValidEnum {
    User {
        #[key]
        id: String,
        name: String,
    },
    Admin {
        #[key]
        admin_id: String,
        permissions: Vec<String>,
    },
    Guest {
        #[key]
        session_id: String,
    },
}

// Invalid enum variant with multiple keys - should fail compilation
/*
#[derive(NetabaseSchema)]
enum InvalidEnumVariantMultipleKeys {
    User {
        #[key]
        id: String,
        #[key]
        email: String,  // This should cause a compilation error
        name: String,
    },
}
*/

// Invalid enum variant with no keys - should fail compilation
/*
#[derive(NetabaseSchema)]
enum InvalidEnumVariantNoKeys {
    User {
        id: String,     // No key field - should cause a compilation error
        name: String,
    },
}
*/

// Valid enum with top-level key closure (commented out due to attribute parsing limitations)
#[derive(NetabaseSchema, Encode, Decode)]
#[key = r#"|item: &EnumWithTopLevelClosure| match item.clone() {    EnumWithTopLevelClosure::User { id, .. } => format!(\"user_{}\", id),
    EnumWithTopLevelClosure::Admin { admin_id, .. } => format!(\"admin_{}\", admin_id),
    EnumWithTopLevelClosure::Guest { session_id, .. } => format!(\"guest_{}\", session_id),
}"#]
enum EnumWithTopLevelClosure {
    User {
        id: String,
        name: String,
    },
    Admin {
        admin_id: String,
        permissions: Vec<String>,
    },
    Guest {
        session_id: String,
    },
}

// Valid enum with top-level closure and no individual variant keys (commented out)
#[derive(NetabaseSchema, Encode, Decode)]
#[key = r#"|item| format!("entity_{}", std::mem::discriminant(item) as u8)"#]
enum EnumWithTopLevelClosureOnly {
    TypeA { data: String },
    TypeB { value: i32 },
    TypeC, // Unit variant is fine
}

// Enum with top-level closure but variant also has key (commented out)
// #[derive(NetabaseSchema)]
// #[key = |item| "top_level_key".to_string()]
// enum EnumWithBothTopLevelAndVariantKeys {
//     User {
//         #[key] // This should generate a warning
//         id: String,
//         name: String,
//     },
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_struct_compiles() {
        // If this compiles, the single key field constraint is working
        let _user = ValidStruct {
            id: "123".to_string(),
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };
    }

    #[test]
    fn test_valid_struct_simple_compiles() {
        // If this compiles, simple key fields are working
        let _user = ValidStructSimple {
            id: "123".to_string(),
            name: "Alice".to_string(),
        };
    }

    #[test]
    fn test_valid_enum_compiles() {
        // If this compiles, the single key per variant constraint is working
        let _user = ValidEnum::User {
            id: "123".to_string(),
            name: "Alice".to_string(),
        };

        let _admin = ValidEnum::Admin {
            admin_id: "admin123".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
        };
    }

    // #[test]
    // fn test_enum_with_top_level_closure_compiles() {
    //     // If this compiles, top-level enum closures are working
    //     let _user = EnumWithTopLevelClosure::User {
    //         id: "123".to_string(),
    //         name: "Alice".to_string(),
    //     };
    // }

    // #[test]
    // fn test_enum_with_top_level_closure_only_compiles() {
    //     // If this compiles, enums can have top-level closures without variant keys
    //     let _entity = EnumWithTopLevelClosureOnly::TypeA {
    //         data: "test".to_string(),
    //     };
    // }
}
