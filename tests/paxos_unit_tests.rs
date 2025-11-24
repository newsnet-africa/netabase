#![cfg(feature = "paxos")]

use netabase::network::behaviour::sync_behaviour::paxos::{
    CoordNum, LogEntryId, NodeId, RoundNum,
};
use libp2p::PeerId;

#[test]
fn test_node_id_creation_and_conversion() {
    let peer_id = PeerId::random();
    let node_id = NodeId::new(peer_id);

    // Test that we can retrieve the peer_id
    assert_eq!(node_id.peer_id(), peer_id);

    // Test From conversion
    let node_id2 = NodeId::from(peer_id);
    assert_eq!(node_id, node_id2);

    // Test PeerId conversion
    let peer_id_back: PeerId = node_id.into();
    assert_eq!(peer_id_back, peer_id);
}

#[test]
fn test_node_id_ordering() {
    let peer1 = PeerId::random();
    let peer2 = PeerId::random();

    let node1 = NodeId::new(peer1);
    let node2 = NodeId::new(peer2);

    // Test that ordering is consistent
    if peer1 < peer2 {
        assert!(node1 < node2);
    } else if peer1 > peer2 {
        assert!(node1 > node2);
    } else {
        assert_eq!(node1, node2);
    }
}

#[test]
fn test_node_id_hash_consistency() {
    use std::collections::HashMap;

    let peer_id = PeerId::random();
    let node_id = NodeId::new(peer_id);

    let mut map = HashMap::new();
    map.insert(node_id, "test");

    assert_eq!(map.get(&node_id), Some(&"test"));
}

#[test]
fn test_node_id_serialization() {
    let peer_id = PeerId::random();
    let node_id = NodeId::new(peer_id);

    // Test serialization with bincode
    let config = bincode::config::standard();
    let encoded = bincode::serde::encode_to_vec(&node_id, config).expect("Should serialize");
    let decoded: NodeId = bincode::serde::decode_from_slice(&encoded, config)
        .expect("Should deserialize")
        .0;

    assert_eq!(node_id, decoded);
}

#[test]
fn test_round_num_creation() {
    let round = RoundNum::new(42);
    assert_eq!(round.value(), 42);
}

#[test]
fn test_round_num_arithmetic() {
    let r1 = RoundNum::new(10);
    let r2 = RoundNum::new(5);

    // Test addition
    assert_eq!(r1 + r2, RoundNum::new(15));

    // Test subtraction
    assert_eq!(r1 - r2, RoundNum::new(5));

    // Test comparison
    assert!(r1 > r2);
    assert!(r2 < r1);
    assert_eq!(r1, RoundNum::new(10));
}

#[test]
fn test_round_num_serialization() {
    let round = RoundNum::new(100);

    let config = bincode::config::standard();
    let encoded = bincode::serde::encode_to_vec(&round, config).expect("Should serialize");
    let decoded: RoundNum = bincode::serde::decode_from_slice(&encoded, config)
        .expect("Should deserialize")
        .0;

    assert_eq!(round, decoded);
}

#[test]
fn test_coord_num_creation() {
    let coord = CoordNum::new(99);
    assert_eq!(coord.value(), 99);
}

#[test]
fn test_coord_num_arithmetic() {
    let c1 = CoordNum::new(20);
    let c2 = CoordNum::new(7);

    // Test basic arithmetic
    assert_eq!(c1 + c2, CoordNum::new(27));
    assert_eq!(c1 - c2, CoordNum::new(13));
    assert_eq!(c1 * c2, CoordNum::new(140));
    assert_eq!(c1 / c2, CoordNum::new(2));
    assert_eq!(c1 % c2, CoordNum::new(6));

    // Test comparison
    assert!(c1 > c2);
    assert_eq!(c1, CoordNum::new(20));
}

#[test]
fn test_log_entry_id_creation() {
    let data = b"test data";
    let hash = blake3::hash(data);
    let entry_id = LogEntryId::new(hash);

    // Test from_bytes
    let entry_id2 = LogEntryId::from_bytes(data);
    assert_eq!(entry_id, entry_id2);
}

#[test]
fn test_log_entry_id_determinism() {
    let data = b"consistent data";
    let id1 = LogEntryId::from_bytes(data);
    let id2 = LogEntryId::from_bytes(data);

    // Same input should produce same ID
    assert_eq!(id1, id2);
}

#[test]
fn test_log_entry_id_uniqueness() {
    let id1 = LogEntryId::from_bytes(b"data1");
    let id2 = LogEntryId::from_bytes(b"data2");

    // Different input should produce different IDs
    assert_ne!(id1, id2);
}

#[test]
fn test_log_entry_id_serialization() {
    let data = b"serialize me";
    let entry_id = LogEntryId::from_bytes(data);

    let config = bincode::config::standard();
    let encoded = bincode::serde::encode_to_vec(&entry_id, config).expect("Should serialize");
    let decoded: LogEntryId = bincode::serde::decode_from_slice(&encoded, config)
        .expect("Should deserialize")
        .0;

    assert_eq!(entry_id, decoded);
}

#[test]
fn test_log_entry_id_ordering() {
    let id1 = LogEntryId::from_bytes(b"aaa");
    let id2 = LogEntryId::from_bytes(b"bbb");

    // IDs should be orderable (for use in btreemaps, etc.)
    let _ = id1 < id2 || id1 > id2 || id1 == id2;
}

#[test]
fn test_log_entry_id_display() {
    let data = b"display test";
    let entry_id = LogEntryId::from_bytes(data);

    let displayed = format!("{}", entry_id);
    // Should produce a hex string
    assert!(!displayed.is_empty());
    assert!(displayed.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_numeric_types_zero() {
    // Test that zero values can be created
    assert_eq!(RoundNum::new(0).value(), 0);
    assert_eq!(CoordNum::new(0).value(), 0);
}

#[test]
fn test_numeric_types_max() {
    let max_round = RoundNum::new(u64::MAX);
    let max_coord = CoordNum::new(u64::MAX);

    assert_eq!(max_round.value(), u64::MAX);
    assert_eq!(max_coord.value(), u64::MAX);
}

#[test]
fn test_numeric_types_clone_copy() {
    let round = RoundNum::new(42);
    let round_clone = round;  // Should be Copy
    assert_eq!(round, round_clone);

    let coord = CoordNum::new(99);
    let coord_clone = coord;  // Should be Copy
    assert_eq!(coord, coord_clone);
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_round_num_addition_commutative(a in 0u64..1000, b in 0u64..1000) {
            let r1 = RoundNum::new(a);
            let r2 = RoundNum::new(b);
            prop_assert_eq!(r1 + r2, r2 + r1);
        }

        #[test]
        fn prop_coord_num_multiplication_commutative(a in 0u64..100, b in 0u64..100) {
            let c1 = CoordNum::new(a);
            let c2 = CoordNum::new(b);
            prop_assert_eq!(c1 * c2, c2 * c1);
        }

        #[test]
        fn prop_log_entry_id_stable(data in prop::collection::vec(any::<u8>(), 0..1000)) {
            let id1 = LogEntryId::from_bytes(&data);
            let id2 = LogEntryId::from_bytes(&data);
            prop_assert_eq!(id1, id2);
        }

        #[test]
        fn prop_node_id_roundtrip_serialization(_seed in any::<u64>()) {
            let peer_id = PeerId::random();
            let node_id = NodeId::new(peer_id);

            let config = bincode::config::standard();
            let encoded = bincode::serde::encode_to_vec(&node_id, config).unwrap();
            let decoded: NodeId = bincode::serde::decode_from_slice(&encoded, config).unwrap().0;

            prop_assert_eq!(node_id, decoded);
        }
    }
}
