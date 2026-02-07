//! Comprehensive tests for the Meadowcap capability system.
//!
//! These tests verify the complete capability lifecycle:
//! - Root capability creation
//! - Capability delegation
//! - Area restriction
//! - Validity verification
//! - Encoding/decoding roundtrips
//! - Security properties

use netabase::data::network::capability::{
    AccessMode, Area, CapabilityDelegation, CapabilityError, CommunalCapability, EncodedCapability,
    McCapability, McEnumerationCapability, OwnedCapability, PathConstraint, SubspaceConstraint,
    TimeRange,
};
use netabase::data::network::capability::meadowcap::UserSignature;
use netabase::data::util::encryption::{NamespacePublicKey, NamespaceSignature};
use libp2p::PeerId;

// ============================================================================
// Test Helpers
// ============================================================================

fn random_namespace() -> NamespacePublicKey {
    NamespacePublicKey::new(rand_bytes())
}

fn rand_bytes() -> [u8; 32] {
    // Simple deterministic bytes for testing
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        let mut bytes = [0u8; 32];
        bytes[0..4].copy_from_slice(&COUNTER.to_le_bytes());
        bytes
    }
}

fn mock_namespace_signature() -> NamespaceSignature {
    NamespaceSignature::new(vec![0; 64])
}

fn mock_user_signature() -> UserSignature {
    UserSignature::new(vec![0; 64])
}

// ============================================================================
// CommunalCapability Tests
// ============================================================================

mod communal_capability {
    use super::*;

    #[test]
    fn test_root_capability_creation() {
        let namespace = random_namespace();
        let user = PeerId::random();

        let cap = CommunalCapability::new_root(AccessMode::Read, namespace.clone(), user.clone());

        assert_eq!(cap.access_mode, AccessMode::Read);
        assert_eq!(cap.namespace_key, namespace);
        assert_eq!(cap.user_key, user);
        assert!(cap.delegations.is_empty());
    }

    #[test]
    fn test_root_capability_always_valid() {
        let namespace = random_namespace();
        let user = PeerId::random();

        let read_cap = CommunalCapability::new_root(AccessMode::Read, namespace.clone(), user.clone());
        let write_cap = CommunalCapability::new_root(AccessMode::Write, namespace, user);

        assert!(read_cap.is_valid(), "Root read capability should be valid");
        assert!(write_cap.is_valid(), "Root write capability should be valid");
    }

    #[test]
    fn test_receiver_is_user_key_for_root() {
        let namespace = random_namespace();
        let user = PeerId::random();

        let cap = CommunalCapability::new_root(AccessMode::Read, namespace, user.clone());

        assert_eq!(cap.receiver(), &user);
    }

    #[test]
    fn test_granted_area_is_subspace_for_root() {
        let namespace = random_namespace();
        let user = PeerId::random();

        let cap = CommunalCapability::new_root(AccessMode::Read, namespace, user.clone());
        let area = cap.granted_area();

        // Root capability grants access to the user's subspace
        assert!(area.includes_subspace(&user));
    }

    #[test]
    fn test_delegation_changes_receiver() {
        let namespace = random_namespace();
        let owner = PeerId::random();
        let delegate = PeerId::random();

        let root_cap = CommunalCapability::new_root(AccessMode::Read, namespace, owner.clone());
        
        let restricted_area = Area::subspace(owner.clone())
            .with_path(PathConstraint::new(vec![b"data".to_vec()]));

        let delegated = root_cap
            .delegate(restricted_area, delegate.clone(), mock_user_signature())
            .expect("Delegation should succeed");

        assert_eq!(delegated.receiver(), &delegate);
        assert_eq!(delegated.delegations.len(), 1);
    }

    #[test]
    fn test_delegation_restricts_area() {
        let namespace = random_namespace();
        let owner = PeerId::random();
        let delegate = PeerId::random();

        let root_cap = CommunalCapability::new_root(AccessMode::Read, namespace, owner.clone());
        
        let restricted_area = Area::subspace(owner.clone())
            .with_path(PathConstraint::new(vec![b"restricted".to_vec()]));

        let delegated = root_cap
            .delegate(restricted_area.clone(), delegate, mock_user_signature())
            .expect("Delegation should succeed");

        let granted = delegated.granted_area();
        
        // The granted area should be the restricted area
        assert_eq!(granted.path.components, restricted_area.path.components);
    }

    #[test]
    fn test_cannot_expand_area_on_delegation() {
        let namespace = random_namespace();
        let owner = PeerId::random();
        let delegate1 = PeerId::random();
        let delegate2 = PeerId::random();

        let root_cap = CommunalCapability::new_root(AccessMode::Read, namespace, owner.clone());
        
        // First delegation restricts to a specific path
        let restricted_area = Area::subspace(owner.clone())
            .with_path(PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec()]));

        let delegated1 = root_cap
            .delegate(restricted_area, delegate1.clone(), mock_user_signature())
            .expect("First delegation should succeed");

        // Second delegation tries to expand the path (remove restriction)
        let expanded_area = Area::subspace(owner)
            .with_path(PathConstraint::new(vec![b"a".to_vec()]));

        let result = delegated1.delegate(expanded_area, delegate2, mock_user_signature());

        assert!(matches!(result, Err(CapabilityError::AreaExpansion)));
    }

    #[test]
    fn test_delegation_chain_preserves_validity() {
        let namespace = random_namespace();
        let owner = PeerId::random();
        let delegate1 = PeerId::random();
        let delegate2 = PeerId::random();
        let delegate3 = PeerId::random();

        let root = CommunalCapability::new_root(AccessMode::Write, namespace, owner.clone());

        // Create a chain of delegations
        let cap1 = root
            .delegate(
                Area::subspace(owner.clone()).with_path(PathConstraint::new(vec![b"a".to_vec()])),
                delegate1.clone(),
                mock_user_signature(),
            )
            .unwrap();

        let cap2 = cap1
            .delegate(
                Area::subspace(owner.clone()).with_path(PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec()])),
                delegate2.clone(),
                mock_user_signature(),
            )
            .unwrap();

        let cap3 = cap2
            .delegate(
                Area::subspace(owner).with_path(PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()])),
                delegate3.clone(),
                mock_user_signature(),
            )
            .unwrap();

        // All capabilities in the chain should be valid (with placeholder signature verification)
        assert!(root.is_valid());
        assert!(cap1.is_valid());
        assert!(cap2.is_valid());
        assert!(cap3.is_valid());

        // Final receiver should be delegate3
        assert_eq!(cap3.receiver(), &delegate3);
        assert_eq!(cap3.delegations.len(), 3);
    }
}

// ============================================================================
// OwnedCapability Tests
// ============================================================================

mod owned_capability {
    use super::*;

    #[test]
    fn test_root_owned_capability_creation() {
        let namespace = random_namespace();
        let user = PeerId::random();
        let auth = mock_namespace_signature();

        let cap = OwnedCapability::new_root(AccessMode::Write, namespace.clone(), user.clone(), auth);

        assert_eq!(cap.access_mode, AccessMode::Write);
        assert_eq!(cap.namespace_key, namespace);
        assert_eq!(cap.user_key, user);
        assert!(cap.delegations.is_empty());
    }

    #[test]
    fn test_owned_root_grants_full_area() {
        let namespace = random_namespace();
        let user = PeerId::random();
        let auth = mock_namespace_signature();

        let cap = OwnedCapability::new_root(AccessMode::Read, namespace, user, auth);
        let area = cap.granted_area();

        // Full area should include any subspace
        let random_user = PeerId::random();
        assert!(area.includes_subspace(&random_user));
    }

    #[test]
    fn test_owned_delegation_to_subspace() {
        let namespace = random_namespace();
        let owner = PeerId::random();
        let user_in_subspace = PeerId::random();
        let auth = mock_namespace_signature();

        let root = OwnedCapability::new_root(AccessMode::Write, namespace, owner, auth);

        // Delegate to a specific subspace
        let subspace_area = Area::subspace(user_in_subspace.clone());
        let delegated = root
            .delegate(subspace_area, user_in_subspace.clone(), mock_user_signature())
            .expect("Delegation should succeed");

        // The delegated capability should only cover the specific subspace
        let granted = delegated.granted_area();
        assert!(granted.includes_subspace(&user_in_subspace));
        assert!(!granted.includes_subspace(&PeerId::random()));
    }

    #[test]
    fn test_owned_delegation_with_time_restriction() {
        let namespace = random_namespace();
        let owner = PeerId::random();
        let delegate = PeerId::random();
        let auth = mock_namespace_signature();

        let root = OwnedCapability::new_root(AccessMode::Write, namespace, owner, auth);

        // Delegate with time restriction (e.g., one week)
        let start_time = 1000000;
        let end_time = start_time + (7 * 24 * 60 * 60); // One week in seconds
        
        let time_restricted_area = Area::full()
            .with_times(TimeRange::new(Some(start_time), Some(end_time)));

        let delegated = root
            .delegate(time_restricted_area, delegate, mock_user_signature())
            .expect("Delegation should succeed");

        let granted = delegated.granted_area();
        assert!(granted.times.contains(start_time + 1000));
        assert!(!granted.times.contains(start_time - 1));
        assert!(!granted.times.contains(end_time + 1));
    }
}

// ============================================================================
// McCapability (Unified) Tests
// ============================================================================

mod mc_capability {
    use super::*;

    #[test]
    fn test_mc_capability_communal_wrapper() {
        let namespace = random_namespace();
        let user = PeerId::random();

        let communal = CommunalCapability::new_root(AccessMode::Read, namespace.clone(), user.clone());
        let mc = McCapability::Communal(communal);

        assert_eq!(mc.access_mode(), AccessMode::Read);
        assert_eq!(mc.receiver(), &user);
        assert_eq!(mc.granted_namespace(), &namespace);
    }

    #[test]
    fn test_mc_capability_owned_wrapper() {
        let namespace = random_namespace();
        let user = PeerId::random();
        let auth = mock_namespace_signature();

        let owned = OwnedCapability::new_root(AccessMode::Write, namespace.clone(), user.clone(), auth);
        let mc = McCapability::Owned(owned);

        assert_eq!(mc.access_mode(), AccessMode::Write);
        assert_eq!(mc.receiver(), &user);
        assert_eq!(mc.granted_namespace(), &namespace);
    }

    #[test]
    fn test_mc_capability_validity_with_communal_check() {
        let namespace = random_namespace();
        let user = PeerId::random();

        let communal = CommunalCapability::new_root(AccessMode::Read, namespace.clone(), user);
        let mc = McCapability::Communal(communal);

        // With a function that says this namespace is communal
        let is_valid = mc.is_valid(|ns| ns == &namespace);
        assert!(is_valid);

        // With a function that says this namespace is owned
        let is_invalid = mc.is_valid(|_| false);
        assert!(!is_invalid);
    }

    #[test]
    fn test_mc_capability_hash_determinism() {
        let namespace = random_namespace();
        let user = PeerId::random();

        let cap1 = McCapability::Communal(CommunalCapability::new_root(
            AccessMode::Read,
            namespace.clone(),
            user.clone(),
        ));
        let cap2 = McCapability::Communal(CommunalCapability::new_root(
            AccessMode::Read,
            namespace,
            user,
        ));

        assert_eq!(cap1.hash(), cap2.hash(), "Same capability should produce same hash");
    }

    #[test]
    fn test_different_capabilities_have_different_hashes() {
        let namespace = random_namespace();
        let user1 = PeerId::random();
        let user2 = PeerId::random();

        let cap1 = McCapability::Communal(CommunalCapability::new_root(
            AccessMode::Read,
            namespace.clone(),
            user1,
        ));
        let cap2 = McCapability::Communal(CommunalCapability::new_root(
            AccessMode::Read,
            namespace,
            user2,
        ));

        assert_ne!(cap1.hash(), cap2.hash(), "Different capabilities should have different hashes");
    }
}

// ============================================================================
// AccessMode Tests
// ============================================================================

mod access_mode {
    use super::*;

    #[test]
    fn test_read_mode_permissions() {
        let read = AccessMode::Read;
        assert!(read.can_read());
        assert!(!read.can_write());
    }

    #[test]
    fn test_write_mode_permissions() {
        let write = AccessMode::Write;
        assert!(write.can_read(), "Write mode should also allow reading");
        assert!(write.can_write());
    }

    #[test]
    fn test_subsumes_relationships() {
        // Write subsumes everything
        assert!(AccessMode::Write.subsumes(&AccessMode::Read));
        assert!(AccessMode::Write.subsumes(&AccessMode::Write));

        // Read only subsumes itself
        assert!(AccessMode::Read.subsumes(&AccessMode::Read));
        assert!(!AccessMode::Read.subsumes(&AccessMode::Write));
    }

    #[test]
    fn test_access_mode_bytes() {
        assert_eq!(AccessMode::Read.to_byte(), 0x00);
        assert_eq!(AccessMode::Write.to_byte(), 0x01);
    }
}

// ============================================================================
// Area Tests
// ============================================================================

mod area {
    use super::*;

    #[test]
    fn test_full_area_includes_everything() {
        let full = Area::full();
        let user = PeerId::random();

        assert!(full.includes_subspace(&user));
        assert!(full.includes(&Area::subspace(user)));
    }

    #[test]
    fn test_subspace_area_only_includes_specific_user() {
        let user1 = PeerId::random();
        let user2 = PeerId::random();

        let subspace = Area::subspace(user1.clone());

        assert!(subspace.includes_subspace(&user1));
        assert!(!subspace.includes_subspace(&user2));
    }

    #[test]
    fn test_path_prefix_inclusion() {
        let user = PeerId::random();

        let parent = Area::subspace(user.clone())
            .with_path(PathConstraint::new(vec![b"a".to_vec()]));
        
        let child = Area::subspace(user)
            .with_path(PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec()]));

        assert!(parent.includes(&child), "Parent path should include child");
        assert!(!child.includes(&parent), "Child path should not include parent");
    }

    #[test]
    fn test_time_range_inclusion() {
        let full_time = Area::full();
        let restricted_time = Area::full()
            .with_times(TimeRange::new(Some(100), Some(200)));

        assert!(full_time.includes(&restricted_time));
        assert!(!restricted_time.includes(&full_time));
    }

    #[test]
    fn test_area_intersection() {
        let user = PeerId::random();

        let area1 = Area::subspace(user.clone())
            .with_path(PathConstraint::new(vec![b"a".to_vec()]))
            .with_times(TimeRange::new(Some(0), Some(200)));

        let area2 = Area::subspace(user.clone())
            .with_path(PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec()]))
            .with_times(TimeRange::new(Some(100), Some(300)));

        let intersection = area1.intersection(&area2).expect("Should have intersection");

        // Path should be the more specific one
        assert_eq!(intersection.path.components, vec![b"a".to_vec(), b"b".to_vec()]);
        
        // Time should be the overlap
        assert_eq!(intersection.times.start, Some(100));
        assert_eq!(intersection.times.end, Some(200));
    }

    #[test]
    fn test_disjoint_areas_no_intersection() {
        let user1 = PeerId::random();
        let user2 = PeerId::random();

        let area1 = Area::subspace(user1);
        let area2 = Area::subspace(user2);

        assert!(area1.intersection(&area2).is_none());
    }
}

// ============================================================================
// EncodedCapability Tests
// ============================================================================

mod encoded_capability {
    use super::*;

    #[test]
    fn test_communal_encode_decode_roundtrip() {
        let namespace = random_namespace();
        let user = PeerId::random();

        let cap = CommunalCapability::new_root(AccessMode::Read, namespace.clone(), user.clone());
        let mc = McCapability::Communal(cap);
        let context = Area::full();

        let encoded = EncodedCapability::encode(&mc, &context);
        let decoded = encoded.decode(namespace);

        assert_eq!(decoded.access_mode(), AccessMode::Read);
        assert_eq!(decoded.receiver(), &user);
    }

    #[test]
    fn test_owned_encode_decode_roundtrip() {
        let namespace = random_namespace();
        let user = PeerId::random();
        let auth = mock_namespace_signature();

        let cap = OwnedCapability::new_root(AccessMode::Write, namespace.clone(), user.clone(), auth);
        let mc = McCapability::Owned(cap);
        let context = Area::full();

        let encoded = EncodedCapability::encode(&mc, &context);
        let decoded = encoded.decode(namespace);

        assert_eq!(decoded.access_mode(), AccessMode::Write);
        assert_eq!(decoded.receiver(), &user);
    }

    #[test]
    fn test_delegated_capability_encode_decode() {
        let namespace = random_namespace();
        let owner = PeerId::random();
        let delegate = PeerId::random();

        let root = CommunalCapability::new_root(AccessMode::Write, namespace.clone(), owner.clone());
        let delegated = root
            .delegate(
                Area::subspace(owner).with_path(PathConstraint::new(vec![b"x".to_vec()])),
                delegate.clone(),
                mock_user_signature(),
            )
            .unwrap();

        let mc = McCapability::Communal(delegated);
        let context = Area::full();

        let encoded = EncodedCapability::encode(&mc, &context);
        assert_eq!(encoded.delegations.len(), 1);

        let decoded = encoded.decode(namespace);
        assert_eq!(decoded.receiver(), &delegate);
    }
}

// ============================================================================
// McEnumerationCapability Tests
// ============================================================================

mod enumeration_capability {
    use super::*;
    use netabase::data::network::capability::enumeration::EncodedEnumerationCapability;

    #[test]
    fn test_enumeration_capability_creation() {
        let namespace = random_namespace();
        let user = PeerId::random();
        let auth = mock_namespace_signature();

        let cap = McEnumerationCapability::new_root(namespace.clone(), user.clone(), auth);

        assert_eq!(cap.receiver(), &user);
        assert_eq!(cap.granted_namespace(), &namespace);
    }

    #[test]
    fn test_enumeration_capability_delegation() {
        let namespace = random_namespace();
        let user1 = PeerId::random();
        let user2 = PeerId::random();
        let auth = mock_namespace_signature();

        let root = McEnumerationCapability::new_root(namespace.clone(), user1.clone(), auth);
        let delegated = root.delegate(user2.clone(), mock_user_signature());

        assert_eq!(delegated.receiver(), &user2);
        assert_eq!(delegated.delegations.len(), 1);
    }

    #[test]
    fn test_enumeration_encode_decode() {
        let namespace = random_namespace();
        let user = PeerId::random();
        let auth = mock_namespace_signature();

        let cap = McEnumerationCapability::new_root(namespace.clone(), user.clone(), auth);
        
        let encoded = EncodedEnumerationCapability::encode(&cap);
        let decoded = encoded.decode(namespace.clone());

        assert_eq!(decoded.receiver(), &user);
        assert_eq!(decoded.granted_namespace(), &namespace);
    }
}

// ============================================================================
// Security Property Tests
// ============================================================================

mod security {
    use super::*;

    #[test]
    fn test_cannot_forge_capability_for_different_namespace() {
        let namespace1 = random_namespace();
        let namespace2 = random_namespace();
        let user = PeerId::random();

        let cap = CommunalCapability::new_root(AccessMode::Read, namespace1.clone(), user);
        let mc = McCapability::Communal(cap);

        // Capability should not grant access to a different namespace
        assert!(!mc.grants_access_to(&namespace2, &PeerId::random()));
    }

    #[test]
    fn test_delegation_preserves_namespace() {
        let namespace = random_namespace();
        let owner = PeerId::random();
        let delegate = PeerId::random();

        let root = CommunalCapability::new_root(AccessMode::Write, namespace.clone(), owner.clone());
        let delegated = root
            .delegate(
                Area::subspace(owner),
                delegate.clone(),
                mock_user_signature(),
            )
            .unwrap();

        assert_eq!(delegated.granted_namespace(), &namespace);
    }

    #[test]
    fn test_capability_hash_includes_all_relevant_data() {
        let namespace = random_namespace();
        let user = PeerId::random();

        // Same user, same namespace, different access mode = different hash
        let read_cap = McCapability::Communal(CommunalCapability::new_root(
            AccessMode::Read,
            namespace.clone(),
            user.clone(),
        ));
        let write_cap = McCapability::Communal(CommunalCapability::new_root(
            AccessMode::Write,
            namespace,
            user,
        ));

        assert_ne!(read_cap.hash(), write_cap.hash());
    }

    #[test]
    fn test_time_expired_capability_area() {
        let past_start = 0;
        let past_end = 1000;
        let current_time = 2000;

        let time_range = TimeRange::new(Some(past_start), Some(past_end));
        
        // Time range doesn't contain current time
        assert!(!time_range.contains(current_time));
        assert!(time_range.contains(500)); // Within the valid range
    }
}
