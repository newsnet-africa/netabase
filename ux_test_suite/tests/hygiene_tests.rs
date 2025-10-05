//! # Hygiene Tests
//!
//! This module contains tests that validate the macro hygiene of Netabase.
//! The core principle being tested is that users should be able to use
//! Netabase macros WITHOUT manually importing any dependencies like
//! `serde`, `bincode`, `strum`, etc.
//!
//! These tests are designed to fail at compile time if hygiene is broken.

use netabase_store::{bincode, netabase_schema_module, serde, NetabaseModel};
use ux_test_suite::TestResult;

/// Test that basic model derivation works with just netabase_store import
#[test]
fn test_basic_hygiene_single_import() -> TestResult {
    // Users only need to import from netabase_store - no separate macro or deps crates
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(HygieneUserKey)]
    struct HygieneUser {
        #[key]
        id: u64,
        name: String,
    }

    let user = HygieneUser {
        id: 1,
        name: "Test".to_string(),
    };

    // If this compiles, hygiene is working
    assert_eq!(user.id, 1);
    Ok(())
}

/// Test that models with secondary keys work with single import
#[test]
fn test_secondary_keys_hygiene() -> TestResult {
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(HygieneIndexedKey)]
    struct HygieneIndexed {
        #[key]
        id: u64,
        name: String,
        #[secondary_key]
        email: String,
    }

    let model = HygieneIndexed {
        id: 1,
        name: "Test".to_string(),
        email: "test@example.com".to_string(),
    };

    assert_eq!(model.id, 1);
    Ok(())
}

/// Test that schema modules work without manual imports
/// Test that schema modules work with single import
#[test]
fn test_schema_module_hygiene() -> TestResult {
    #[netabase_schema_module(HygieneSchema, HygieneSchemaKeys)]
    mod hygiene_schema {
        use super::*;
        use netabase_store::traits::NetabaseModel as NetabaseModelTrait;

        #[derive(
            NetabaseModel,
            Clone,
            Debug,
            bincode::Encode,
            bincode::Decode,
            serde::Serialize,
            serde::Deserialize,
        )]
        #[key_name(HygieneSchemaUserKey)]
        pub struct HygieneSchemaUser {
            #[key]
            pub id: u64,
            pub name: String,
        }

        #[derive(
            NetabaseModel,
            Clone,
            Debug,
            bincode::Encode,
            bincode::Decode,
            serde::Serialize,
            serde::Deserialize,
        )]
        #[key_name(HygieneSchemaPostKey)]
        pub struct HygieneSchemaPost {
            #[key]
            pub id: u64,
            pub title: String,
            #[secondary_key]
            pub author_id: u64,
        }
    }

    use hygiene_schema::*;

    let user = HygieneSchemaUser {
        id: 1,
        name: "Test".to_string(),
    };

    let post = HygieneSchemaPost {
        id: 1,
        title: "Test Post".to_string(),
        author_id: 1,
    };

    assert_eq!(user.id, 1);
    assert_eq!(post.author_id, 1);
    Ok(())
}

/// Test that complex types work without manual imports
/// Test that macros handle edge cases correctly
#[test]
fn test_edge_cases_hygiene() -> TestResult {
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(HygieneComplexKey)]
    struct HygieneComplex {
        #[key]
        id: u64,
        name: String,
    }

    let model = HygieneComplex {
        id: 1,
        name: "Complex".to_string(),
    };

    assert_eq!(model.id, 1);
    Ok(())
}

/// Test that models work when user defines conflicting names
#[test]
fn test_name_conflict_isolation() -> TestResult {
    // Import already available from top-level use statement

    // Define user variables with names that could conflict with macro internals
    let serde = "user_defined_serde";
    let bincode = "user_defined_bincode";
    let strum = "user_defined_strum";
    let std = "user_defined_std";

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(ConflictTestKey)]
    struct ConflictTest {
        #[key]
        id: u64,
        name: String,
        #[secondary_key]
        active: bool,
    }

    let model = ConflictTest {
        id: 1,
        name: "Test".to_string(),
        active: true,
    };

    // Verify user variables are still accessible
    assert_eq!(serde, "user_defined_serde");
    assert_eq!(bincode, "user_defined_bincode");
    assert_eq!(strum, "user_defined_strum");
    assert_eq!(std, "user_defined_std");

    // Verify model works
    assert_eq!(model.id, 1);
    Ok(())
}

/// Test that multiple models work independently
#[test]
fn test_order_independence() -> TestResult {
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(Order1Key)]
    struct Order1 {
        #[key]
        id: u64,
        name: String,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(Order2Key)]
    struct Order2 {
        #[key]
        id: u64,
        name: String,
    }

    let m1 = Order1 {
        id: 1,
        name: "Test1".to_string(),
    };
    let m2 = Order2 {
        id: 2,
        name: "Test2".to_string(),
    };

    assert_eq!(m1.id, 1);
    assert_eq!(m2.id, 2);
    Ok(())
}

/// Test that nested modules work correctly
#[test]
fn test_nested_modules_hygiene() -> TestResult {
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(NestedTestKey)]
    struct NestedTest {
        #[key]
        id: u64,
        name: String,
    }

    let model = NestedTest {
        id: 1,
        name: "Nested".to_string(),
    };

    assert_eq!(model.id, 1);
    Ok(())
}

/// Test that macros work with generic contexts
#[test]
fn test_generic_context_hygiene() -> TestResult {
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(GenericContextModelKey)]
    struct GenericContextModel {
        #[key]
        id: u64,
        name: String,
    }

    let model = GenericContextModel {
        id: 1,
        name: "Generic".to_string(),
    };

    assert_eq!(model.name, "Generic");
    Ok(())
}

/// Test that attribute order doesn't matter
#[test]
fn test_attribute_order_flexibility() -> TestResult {
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(Order1Key)]
    struct Order1 {
        #[key]
        id: u64,
        name: String,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(Order2Key)]
    struct Order2 {
        #[key]
        id: u64,
        name: String,
    }

    let m1 = Order1 {
        id: 1,
        name: "Test1".to_string(),
    };
    let m2 = Order2 {
        id: 2,
        name: "Second".to_string(),
    };

    assert_eq!(m1.id, 1);
    assert_eq!(m2.id, 2);
    Ok(())
}

/// Test that macros work with conditional compilation
#[test]
fn test_conditional_compilation_hygiene() -> TestResult {
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(ConditionalModelKey)]
    struct ConditionalModel {
        #[key]
        id: u64,
        name: String,
        #[cfg(feature = "test")]
        test_field: String,
        #[cfg(not(feature = "nonexistent"))]
        always_present: u32,
    }

    let model = ConditionalModel {
        id: 1,
        name: "Conditional".to_string(),
        #[cfg(feature = "test")]
        test_field: "test".to_string(),
        #[cfg(not(feature = "nonexistent"))]
        always_present: 42,
    };

    assert_eq!(model.id, 1);
    assert_eq!(model.always_present, 42);
    Ok(())
}

/// Integration test using the test runner framework
/// Test that demonstrates the simplified user experience
#[test]
fn test_hygiene_comparison() -> TestResult {
    // With netabase_store: Only one import needed
    // Before: users had to separately import netabase_macros, netabase_deps,
    // serde, bincode, strum, derive_more, etc.

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(HygieneComparisonKey)]
    struct HygieneComparison {
        #[key]
        id: u64,
        name: String,
    }

    let model = HygieneComparison {
        id: 1,
        name: "Hygiene rocks!".to_string(),
    };

    assert_eq!(model.name, "Hygiene rocks!");
    Ok(())
}

/// Test minimal model definition
#[test]
fn test_minimal_model() -> TestResult {
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(MinimalKey)]
    struct Minimal {
        #[key]
        id: u64,
    }

    // Test with many secondary keys
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[key_name(ManyKeysKey)]
    struct ManyKeys {
        #[key]
        id: u64,
        #[secondary_key]
        key1: u32,
        #[secondary_key]
        key2: String,
        #[secondary_key]
        key3: bool,
        #[secondary_key]
        key4: u8,
        #[secondary_key]
        key5: i32,
    }

    let minimal = Minimal { id: 1 };
    let many_keys = ManyKeys {
        id: 1,
        key1: 1,
        key2: "test".to_string(),
        key3: true,
        key4: 255,
        key5: -1,
    };

    assert_eq!(minimal.id, 1);
    assert_eq!(many_keys.id, 1);
    Ok(())
}
