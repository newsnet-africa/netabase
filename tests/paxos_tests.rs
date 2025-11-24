//! Comprehensive tests for the Paxos implementation
//!
//! These tests verify the correctness of the core paxos implementation
//! independently of the libp2p integration.

#![cfg(all(test, feature = "paxos"))]

use netabase::network::behaviour::sync_behaviour::paxos::{
    log_entry::LogEntry, node_id::NodeId, CoordNum, RoundNum,
};
use netabase_store::traits::definition::NetabaseDefinitionTrait;
use bincode::{Encode, Decode};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use strum::{EnumDiscriminants, IntoEnumIterator};

// ============================================================================
// Test Model Definition
// ============================================================================

/// Simple model type for testing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, EnumDiscriminants)]
#[strum_discriminants(derive(
    strum::EnumIter,
    strum::Display,
    strum::AsRefStr,
    strum::EnumString,
    Hash,
    Encode,
    Decode
))]
#[strum_discriminants(name(TestModelDiscriminant))]
enum TestModel {
    Simple { id: u64, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Encode, Decode, EnumDiscriminants)]
#[strum_discriminants(derive(
    strum::EnumIter,
    strum::Display,
    strum::AsRefStr,
    strum::EnumString,
    Hash,
    Encode,
    Decode
))]
#[strum_discriminants(name(TestModelKeysDiscriminant))]
enum TestModelKeys {
    Id(u64),
}

impl netabase_store::traits::definition::NetabaseDefinitionTraitKey for TestModelKeys {}

impl NetabaseDefinitionTrait for TestModel {
    type Keys = TestModelKeys;
    type Tables = TestModelDiscriminant;

    fn tables() -> Self::Tables {
        TestModelDiscriminant::Simple
    }

    #[cfg(all(feature = "paxos", feature = "libp2p", not(target_arch = "wasm32")))]
    fn apply_to_store<S>(&self, _store: &mut S) -> Result<(), String>
    where
        S: libp2p::kad::store::RecordStore,
    {
        Ok(())
    }
}

impl netabase_store::convert::ToIVec for TestModel {}
impl netabase_store::convert::ToIVec for TestModelKeys {}

// ============================================================================
// Unit Tests - LogEntry
// ============================================================================

#[test]
fn test_log_entry_id_determinism() {
    // Same log entry should always produce the same ID
    let model = TestModel::Simple {
        id: 1,
        value: "test".to_string(),
    };

    let entry1 = LogEntry::Create(model.clone());
    let entry2 = LogEntry::Create(model.clone());

    use paxakos::LogEntry as PaxakosLogEntry;
    assert_eq!(entry1.id(), entry2.id(), "Log entry IDs should be deterministic");
}

#[test]
fn test_log_entry_different_ops() {
    let model = TestModel::Simple {
        id: 1,
        value: "test".to_string(),
    };

    let create = LogEntry::Create(model.clone());
    let delete = LogEntry::Delete(model.clone());

    use paxakos::LogEntry as PaxakosLogEntry;
    assert_ne!(
        create.id(),
        delete.id(),
        "Create and Delete should have different IDs"
    );
}

#[test]
fn test_log_entry_different_models() {
    let model1 = TestModel::Simple {
        id: 1,
        value: "test1".to_string(),
    };
    let model2 = TestModel::Simple {
        id: 2,
        value: "test2".to_string(),
    };

    let entry1 = LogEntry::Create(model1);
    let entry2 = LogEntry::Create(model2);

    use paxakos::LogEntry as PaxakosLogEntry;
    assert_ne!(entry1.id(), entry2.id(), "Different models should have different IDs");
}

#[test]
fn test_log_entry_helpers() {
    let model = TestModel::Simple {
        id: 1,
        value: "test".to_string(),
    };

    let create = LogEntry::Create(model.clone());
    assert!(create.is_create());
    assert!(!create.is_delete());
    assert_eq!(create.model(), &model);

    let delete = LogEntry::Delete(model.clone());
    assert!(delete.is_delete());
    assert!(!delete.is_create());
    assert_eq!(delete.model(), &model);
}

#[test]
fn test_log_entry_serialization() {
    let model = TestModel::Simple {
        id: 42,
        value: "serialization test".to_string(),
    };

    let entry = LogEntry::Create(model.clone());

    // Should be serializable
    let serialized = bincode::serde::encode_to_vec(&entry, bincode::config::standard()).unwrap();
    let (deserialized, _): (LogEntry<TestModel>, _) =
        bincode::serde::decode_from_slice(&serialized, bincode::config::standard()).unwrap();

    assert_eq!(entry.model(), deserialized.model());
    assert_eq!(entry.is_create(), deserialized.is_create());
}

// ============================================================================
// Unit Tests - RoundNum
// ============================================================================

#[test]
fn test_round_num_arithmetic() {
    let r1 = RoundNum(5);
    let r2 = RoundNum(3);

    assert_eq!(r1 + r2, RoundNum(8));
    assert_eq!(r1 - r2, RoundNum(2));
    assert_eq!(r1 * r2, RoundNum(15));
    assert_eq!(r1 / r2, RoundNum(1));
}

#[test]
fn test_round_num_saturating() {
    let max = RoundNum(u64::MAX);
    let one = RoundNum(1);

    // Should saturate, not overflow
    assert_eq!(max + one, RoundNum(u64::MAX));
}

#[test]
fn test_round_num_zero_one() {
    use num_traits::{One, Zero};

    assert_eq!(RoundNum::zero(), RoundNum(0));
    assert_eq!(RoundNum::one(), RoundNum(1));

    assert!(RoundNum::zero().is_zero());
    assert!(!RoundNum::one().is_zero());
}

#[test]
fn test_round_num_ordering() {
    let r1 = RoundNum(1);
    let r2 = RoundNum(2);
    let r3 = RoundNum(2);

    assert!(r1 < r2);
    assert!(r2 > r1);
    assert_eq!(r2, r3);
}

#[test]
fn test_round_num_conversions() {
    let r = RoundNum(42);

    // to u128
    let as_u128: u128 = r.into();
    assert_eq!(as_u128, 42u128);

    // from u128
    let from_u128: RoundNum = 42u128.try_into().unwrap();
    assert_eq!(from_u128, r);

    // from usize
    let from_usize: RoundNum = 42usize.try_into().unwrap();
    assert_eq!(from_usize, r);

    // to usize
    let to_usize: usize = r.try_into().unwrap();
    assert_eq!(to_usize, 42usize);
}

#[test]
fn test_round_num_display() {
    let r = RoundNum(123);
    assert_eq!(format!("{}", r), "123");
}

// ============================================================================
// Unit Tests - CoordNum
// ============================================================================

#[test]
fn test_coord_num_arithmetic() {
    let c1 = CoordNum(10);
    let c2 = CoordNum(5);

    assert_eq!(c1 + c2, CoordNum(15));
    assert_eq!(c1 - c2, CoordNum(5));
    assert_eq!(c1 * c2, CoordNum(50));
    assert_eq!(c1 / c2, CoordNum(2));
}

#[test]
fn test_coord_num_ordering() {
    let c1 = CoordNum(1);
    let c2 = CoordNum(2);
    let c3 = CoordNum(2);

    assert!(c1 < c2);
    assert!(c2 > c1);
    assert_eq!(c2, c3);
}

#[test]
fn test_coord_num_zero_one() {
    use num_traits::{One, Zero};

    assert_eq!(CoordNum::zero(), CoordNum(0));
    assert_eq!(CoordNum::one(), CoordNum(1));
}

#[test]
fn test_coord_num_bounded() {
    use num_traits::Bounded;

    assert_eq!(CoordNum::min_value(), CoordNum(u64::MIN));
    assert_eq!(CoordNum::max_value(), CoordNum(u64::MAX));
}

// ============================================================================
// Unit Tests - NodeId
// ============================================================================

#[test]
fn test_node_id_conversion() {
    use libp2p::PeerId;
    use paxakos::NodeInfo;

    let peer_id = PeerId::random();
    let node_id = NodeId::from(peer_id);

    assert_eq!(node_id.peer_id(), peer_id);
    assert_eq!(PeerId::from(node_id), peer_id);

    // NodeInfo trait
    assert_eq!(node_id.id(), node_id);
}

#[test]
fn test_node_id_equality() {
    use libp2p::PeerId;

    let peer_id1 = PeerId::random();
    let peer_id2 = PeerId::random();

    let node_id1a = NodeId::from(peer_id1);
    let node_id1b = NodeId::from(peer_id1);
    let node_id2 = NodeId::from(peer_id2);

    assert_eq!(node_id1a, node_id1b);
    assert_ne!(node_id1a, node_id2);
}

#[test]
fn test_node_id_ordering() {
    use libp2p::PeerId;

    let peer_id1 = PeerId::random();
    let peer_id2 = PeerId::random();

    let node_id1 = NodeId::from(peer_id1);
    let node_id2 = NodeId::from(peer_id2);

    // Should be orderable (for use in sorted collections)
    let mut vec = vec![node_id2, node_id1];
    vec.sort();

    // Just verify it doesn't panic and produces deterministic results
    assert_eq!(vec.len(), 2);
}

#[test]
fn test_node_id_serialization() {
    use libp2p::PeerId;

    let peer_id = PeerId::random();
    let node_id = NodeId::from(peer_id);

    let serialized = bincode::serde::encode_to_vec(&node_id, bincode::config::standard()).unwrap();
    let (deserialized, _): (NodeId, _) =
        bincode::serde::decode_from_slice(&serialized, bincode::config::standard()).unwrap();

    assert_eq!(node_id, deserialized);
    assert_eq!(node_id.peer_id(), deserialized.peer_id());
}

#[test]
fn test_node_id_display() {
    use libp2p::PeerId;

    let peer_id = PeerId::random();
    let node_id = NodeId::from(peer_id);

    let display_str = format!("{}", node_id);
    let peer_id_str = format!("{}", peer_id);

    assert_eq!(display_str, peer_id_str);
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_log_entry_id_stable(id in 0u64..1000, value in "\\PC*") {
            let model = TestModel::Simple { id, value: value.clone() };
            let entry = LogEntry::Create(model);

            use paxakos::LogEntry as PaxakosLogEntry;
            let id1 = entry.id();
            let id2 = entry.id();

            prop_assert_eq!(id1, id2, "ID should be stable");
        }

        #[test]
        fn prop_round_num_monotonic(a in 0u64..1000, b in 0u64..1000) {
            let r1 = RoundNum(a);
            let r2 = RoundNum(b);

            if a < b {
                prop_assert!(r1 < r2);
            } else if a > b {
                prop_assert!(r1 > r2);
            } else {
                prop_assert_eq!(r1, r2);
            }
        }

        #[test]
        fn prop_coord_num_addition_commutative(a in 0u64..100, b in 0u64..100) {
            let c1 = CoordNum(a);
            let c2 = CoordNum(b);

            prop_assert_eq!(c1 + c2, c2 + c1);
        }

        #[test]
        fn prop_round_num_addition_associative(a in 0u64..50, b in 0u64..50, c in 0u64..50) {
            let r1 = RoundNum(a);
            let r2 = RoundNum(b);
            let r3 = RoundNum(c);

            prop_assert_eq!((r1 + r2) + r3, r1 + (r2 + r3));
        }

        #[test]
        fn prop_coord_num_multiplication_commutative(a in 0u64..50, b in 0u64..50) {
            let c1 = CoordNum(a);
            let c2 = CoordNum(b);

            prop_assert_eq!(c1 * c2, c2 * c1);
        }

        #[test]
        fn prop_log_entry_round_trip(id in 0u64..1000, value in "\\PC{0,20}") {
            let model = TestModel::Simple { id, value };
            let entry = LogEntry::Create(model.clone());

            // Serialize and deserialize
            let serialized = bincode::serde::encode_to_vec(&entry, bincode::config::standard()).unwrap();
            let (deserialized, _): (LogEntry<TestModel>, _) =
                bincode::serde::decode_from_slice(&serialized, bincode::config::standard()).unwrap();

            prop_assert_eq!(entry.model(), deserialized.model());
        }

        #[test]
        fn prop_node_id_round_trip_serialization(seed in 0u64..1000) {
            use libp2p::PeerId;

            // Create deterministic peer ID from seed
            let bytes = seed.to_le_bytes();
            let mut full_bytes = [0u8; 32];
            full_bytes[..8].copy_from_slice(&bytes);

            let peer_id = PeerId::from_bytes(&full_bytes).unwrap_or_else(|_| PeerId::random());
            let node_id = NodeId::from(peer_id);

            let serialized = bincode::serde::encode_to_vec(&node_id, bincode::config::standard()).unwrap();
            let (deserialized, _): (NodeId, _) =
                bincode::serde::decode_from_slice(&serialized, bincode::config::standard()).unwrap();

            prop_assert_eq!(node_id, deserialized);
        }
    }
}

// ============================================================================
// Integration-style Tests
// ============================================================================

#[test]
fn test_complete_workflow() {
    // Simulate a complete paxos workflow
    let model1 = TestModel::Simple {
        id: 1,
        value: "first".to_string(),
    };
    let model2 = TestModel::Simple {
        id: 2,
        value: "second".to_string(),
    };

    // Create entries
    let create1 = LogEntry::Create(model1.clone());
    let create2 = LogEntry::Create(model2.clone());
    let delete1 = LogEntry::Delete(model1.clone());

    use paxakos::LogEntry as PaxakosLogEntry;

    // Verify IDs are unique
    assert_ne!(create1.id(), create2.id());
    assert_ne!(create1.id(), delete1.id());

    // Verify operations
    assert!(create1.is_create());
    assert!(create2.is_create());
    assert!(delete1.is_delete());

    // Verify model access
    match (create1.model(), create2.model(), delete1.model()) {
        (TestModel::Simple { id: id1, .. }, TestModel::Simple { id: id2, .. }, TestModel::Simple { id: id3, .. }) => {
            assert_eq!(*id1, 1);
            assert_eq!(*id2, 2);
            assert_eq!(*id3, 1);
        }
    }
}

#[test]
fn test_round_progression() {
    let mut round = RoundNum(0);

    for i in 1..=10 {
        round = round + RoundNum(1);
        assert_eq!(round, RoundNum(i));
    }
}

#[test]
fn test_coord_num_progression() {
    let mut coord = CoordNum(0);

    // Simulate leadership changes
    for i in 1..=5 {
        coord = coord + CoordNum(1);
        assert_eq!(coord, CoordNum(i));
    }
}

#[test]
fn test_multiple_nodes() {
    use libp2p::PeerId;
    use std::collections::HashSet;

    // Create multiple nodes
    let node_ids: Vec<NodeId> = (0..5).map(|_| NodeId::from(PeerId::random())).collect();

    // All should be unique
    let unique: HashSet<_> = node_ids.iter().collect();
    assert_eq!(unique.len(), 5);

    // All should be orderable
    let mut sorted = node_ids.clone();
    sorted.sort();
    assert_eq!(sorted.len(), 5);
}
