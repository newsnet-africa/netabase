//! Simple test to verify that the NetabaseModel derive macro works with re-exported dependencies

use netabase_deps::{bincode, serde};
use netabase_macros::NetabaseModel;

// This uses re-exported dependencies for hygiene
#[derive(
    NetabaseModel,
    Clone,
    Debug,
    bincode::Encode,
    bincode::Decode,
    serde::Serialize,
    serde::Deserialize,
)]
#[key_name(TestKey)]
pub struct TestModel {
    #[key]
    pub id: u64,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use netabase_store::traits::NetabaseModel;

    #[test]
    fn test_simple_model() {
        let model = TestModel {
            id: 1,
            name: "test".to_string(),
        };

        // Test that the model can be converted to key (requires NetabaseModel trait)
        let key = model.key();

        // Basic assertion to ensure compilation works
        assert_eq!(model.id, 1);
        assert_eq!(model.name, "test");
    }
}
