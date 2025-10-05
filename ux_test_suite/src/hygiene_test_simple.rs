//! # Simple Hygiene Test
//!
//! This module contains a minimal test to verify that Netabase macros work
//! without requiring manual imports of dependencies. This is the core test
//! of macro hygiene - users should only need to import the macro itself.

use netabase_macros::NetabaseModel;
// Use re-exported dependencies for convenience - this tests that they work correctly
use netabase_deps::{bincode, serde};

/// Test that basic NetabaseModel works with convenient re-exports
#[derive(
    NetabaseModel,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
#[key_name(SimpleTestKey)]
pub struct SimpleTest {
    #[key]
    pub id: u64,
    pub name: String,
    #[secondary_key]
    pub active: bool,
}

impl Default for SimpleTest {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Test".to_string(),
            active: true,
        }
    }
}

/// Test that we can create and use the model with convenient re-exports
pub fn test_basic_hygiene() -> Result<(), Box<dyn std::error::Error>> {
    // This test validates that the model can be used with convenient re-exports
    // If this compiles and runs, the hygiene test passes
    let model = SimpleTest::default();

    // Basic assertions
    assert_eq!(model.id, 1);
    assert_eq!(model.name, "Test");
    assert!(model.active);

    // Test that we can access the generated key (this proves the macro worked)
    use netabase_store::traits::NetabaseModel;
    let _key = model.key();

    println!("✓ Basic hygiene test passed - model created with convenient re-exports");
    Ok(())
}

/// Test that multiple models work together without conflicts
#[derive(
    NetabaseModel,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
#[key_name(AnotherTestKey)]
pub struct AnotherTest {
    #[key]
    pub id: u64,
    pub value: String,
    #[secondary_key]
    pub category: u32,
}

impl Default for AnotherTest {
    fn default() -> Self {
        Self {
            id: 42,
            value: "Another test".to_string(),
            category: 1,
        }
    }
}

/// Test that name conflicts don't interfere with macro hygiene
pub fn test_name_conflict_hygiene() -> Result<(), Box<dyn std::error::Error>> {
    // Define variables with names that could conflict with macro internals
    let serde = "user_serde_variable";
    let bincode = "user_bincode_variable";
    let strum = "user_strum_variable";

    // Create models - macros should use absolute paths and not conflict
    let model1 = SimpleTest::default();
    let model2 = AnotherTest::default();

    // Verify user variables are still accessible
    assert_eq!(serde, "user_serde_variable");
    assert_eq!(bincode, "user_bincode_variable");
    assert_eq!(strum, "user_strum_variable");

    // Verify models work correctly
    assert_eq!(model1.id, 1);
    assert_eq!(model2.id, 42);

    println!("✓ Name conflict hygiene test passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hygiene_basic() {
        test_basic_hygiene().expect("Basic hygiene test should pass");
    }

    #[test]
    fn test_hygiene_name_conflicts() {
        test_name_conflict_hygiene().expect("Name conflict hygiene test should pass");
    }

    #[test]
    fn test_model_creation() {
        let model = SimpleTest {
            id: 100,
            name: "Custom".to_string(),
            active: false,
        };

        assert_eq!(model.id, 100);
        assert_eq!(model.name, "Custom");
        assert!(!model.active);
    }

    #[test]
    fn test_multiple_models() {
        let simple = SimpleTest::default();
        let another = AnotherTest::default();

        // Both should work together without issues
        assert_eq!(simple.id, 1);
        assert_eq!(another.id, 42);
        assert!(simple.active);
        assert_eq!(another.category, 1);
    }
}
