#![cfg(feature = "paxos")]

use netabase::network::behaviour::sync_behaviour::paxos::{
    log_entry::LogEntry, state::Frozen, CoordNum, NodeId, PaxosBehaviour, RoundNum,
};
use netabase_store::traits::definition::NetabaseDefinitionTrait;
use bincode::{Encode, Decode};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use strum::{EnumDiscriminants, IntoEnumIterator};

/// Test model for integration testing
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
pub enum TestModel {
    User { id: u64, name: String },
    Post { id: u64, content: String },
    Comment { id: u64, post_id: u64, text: String },
}

impl NetabaseDefinitionTrait for TestModel {
    type Keys = TestModelKeys;
    type Tables = TestModelDiscriminant;

    fn tables() -> Self::Tables {
        // Return default discriminant for testing
        TestModelDiscriminant::User
    }

    #[cfg(all(feature = "paxos", feature = "libp2p", not(target_arch = "wasm32")))]
    fn apply_to_store<S>(&self, _store: &mut S) -> Result<(), String>
    where
        S: libp2p::kad::store::RecordStore,
    {
        // Stub implementation for testing
        Ok(())
    }
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
pub enum TestModelKeys {
    UserId(u64),
    PostId(u64),
    CommentId(u64),
}

impl netabase_store::traits::definition::NetabaseDefinitionTraitKey for TestModelKeys {}

impl netabase_store::convert::ToIVec for TestModel {}

impl netabase_store::convert::ToIVec for TestModelKeys {}

#[test]
fn test_log_entry_create_variant() {
    let model = TestModel::User {
        id: 1,
        name: "Alice".to_string(),
    };

    let entry = LogEntry::Create(model.clone());

    // Test that we can pattern match
    match entry {
        LogEntry::Create(m) => assert_eq!(m, model),
        LogEntry::Delete(_) => panic!("Expected Create variant"),
    }
}

#[test]
fn test_log_entry_delete_variant() {
    let model = TestModel::Post {
        id: 42,
        content: "Test post".to_string(),
    };

    let entry = LogEntry::Delete(model.clone());

    match entry {
        LogEntry::Delete(m) => assert_eq!(m, model),
        LogEntry::Create(_) => panic!("Expected Delete variant"),
    }
}

#[test]
fn test_log_entry_id_generation() {
    use paxakos::LogEntry as PaxakosLogEntry;

    let model = TestModel::Comment {
        id: 1,
        post_id: 10,
        text: "Great post!".to_string(),
    };

    let entry1 = LogEntry::Create(model.clone());
    let entry2 = LogEntry::Create(model.clone());

    // Same log entry should produce same ID
    assert_eq!(entry1.id(), entry2.id());
}

#[test]
fn test_log_entry_different_ids() {
    use paxakos::LogEntry as PaxakosLogEntry;

    let model1 = TestModel::User {
        id: 1,
        name: "Alice".to_string(),
    };
    let model2 = TestModel::User {
        id: 2,
        name: "Bob".to_string(),
    };

    let entry1 = LogEntry::Create(model1);
    let entry2 = LogEntry::Create(model2);

    // Different models should produce different IDs
    assert_ne!(entry1.id(), entry2.id());
}

#[test]
fn test_log_entry_create_vs_delete_different_ids() {
    use paxakos::LogEntry as PaxakosLogEntry;

    let model = TestModel::Post {
        id: 100,
        content: "Content".to_string(),
    };

    let create_entry = LogEntry::Create(model.clone());
    let delete_entry = LogEntry::Delete(model);

    // Create and Delete of same model should have different IDs
    assert_ne!(create_entry.id(), delete_entry.id());
}

#[test]
fn test_log_entry_serialization() {
    let model = TestModel::User {
        id: 999,
        name: "Serializable".to_string(),
    };

    let entry = LogEntry::Create(model);

    let config = bincode::config::standard();
    let encoded = bincode::serde::encode_to_vec(&entry, config).expect("Should serialize");
    let decoded: LogEntry<TestModel> = bincode::serde::decode_from_slice(&encoded, config)
        .expect("Should deserialize")
        .0;

    match (&entry, &decoded) {
        (LogEntry::Create(m1), LogEntry::Create(m2)) => assert_eq!(m1, m2),
        _ => panic!("Serialization changed variant"),
    }
}

#[test]
fn test_frozen_state_creation() {
    let frozen: Frozen<TestModel> = Frozen::new(vec![], std::collections::HashMap::new());

    assert_eq!(frozen.entries.len(), 0);
    assert_eq!(frozen.metadata.len(), 0);
}

#[test]
fn test_frozen_state_with_entries() {
    let entry1 = (vec![1, 2, 3], vec![4, 5, 6]);
    let entry2 = (vec![7, 8, 9], vec![10, 11, 12]);

    let frozen: Frozen<TestModel> = Frozen::new(
        vec![entry1.clone(), entry2.clone()],
        std::collections::HashMap::new()
    );

    assert_eq!(frozen.entries.len(), 2);
    assert_eq!(frozen.entries[0], entry1);
    assert_eq!(frozen.entries[1], entry2);
}

#[test]
fn test_frozen_state_serialization() {
    let frozen: Frozen<TestModel> = Frozen::new(
        vec![(vec![1], vec![2])],
        std::collections::HashMap::new()
    );

    let config = bincode::config::standard();
    let encoded = bincode::serde::encode_to_vec(&frozen, config).expect("Should serialize");
    let decoded: Frozen<TestModel> = bincode::serde::decode_from_slice(&encoded, config)
        .expect("Should deserialize")
        .0;

    assert_eq!(frozen.entries, decoded.entries);
}

#[test]
fn test_round_and_coord_nums_together() {
    let round = RoundNum::new(10);
    let coord = CoordNum::new(5);

    // Test that they can be used together
    assert_eq!(round.value() / coord.value(), 2);
    assert!(round.value() > coord.value());
}

#[test]
fn test_node_id_as_state_identifier() {
    use libp2p::PeerId;

    let peer1 = PeerId::random();
    let peer2 = PeerId::random();
    let peer3 = PeerId::random();

    let node1 = NodeId::new(peer1);
    let node2 = NodeId::new(peer2);
    let node3 = NodeId::new(peer3);

    // Test that we can create a cluster of nodes
    let cluster = vec![node1, node2, node3];
    assert_eq!(cluster.len(), 3);

    // Test that we can look up nodes
    assert!(cluster.contains(&node1));
    assert!(cluster.contains(&node2));
}

#[cfg(test)]
mod integration_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_log_entry_id_consistency(
            id in 0u64..1000,
            name in "[a-z]{5,10}"
        ) {
            use paxakos::LogEntry as PaxakosLogEntry;

            let model = TestModel::User { id, name };
            let entry = LogEntry::Create(model);

            let id1 = entry.id();
            let id2 = entry.id();

            prop_assert_eq!(id1, id2);
        }

        #[test]
        fn prop_frozen_state_roundtrip(
            entries in prop::collection::vec(
                (prop::collection::vec(any::<u8>(), 0..10),
                 prop::collection::vec(any::<u8>(), 0..10)),
                0..5
            )
        ) {
            let frozen: Frozen<TestModel> = Frozen::new(
                entries,
                std::collections::HashMap::new()
            );

            let config = bincode::config::standard();
            let encoded = bincode::serde::encode_to_vec(&frozen, config).unwrap();
            let decoded: Frozen<TestModel> =
                bincode::serde::decode_from_slice(&encoded, config).unwrap().0;

            prop_assert_eq!(frozen.entries, decoded.entries);
        }
    }
}
