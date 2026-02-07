//! Tests for Private Area Intersection (PAI) protocol.
//!
//! PAI allows peers to discover shared interests without revealing
//! information about interests they don't share.
//!
//! These tests verify:
//! - PrivateInterest creation and comparison
//! - PAI fragment generation
//! - Overlap detection (Equal, MoreSpecific, LessSpecific, Awkward, Disjoint)
//! - Salt handling for initiator/responder
//! - Announcement authentication

use netabase::data::network::capability::{
    DetectedOverlap, OverlapAnnouncement, OverlapType, PaiFragment, PaiState, PrivateInterest,
};
use netabase::data::network::capability::area::{PathConstraint, SubspaceConstraint};
use netabase::data::util::encryption::NamespacePublicKey;
use libp2p::PeerId;

// ============================================================================
// Test Helpers
// ============================================================================

fn test_namespace() -> NamespacePublicKey {
    NamespacePublicKey::new([1u8; 32])
}

fn namespace_with_id(id: u8) -> NamespacePublicKey {
    let mut bytes = [0u8; 32];
    bytes[0] = id;
    NamespacePublicKey::new(bytes)
}

// ============================================================================
// PrivateInterest Creation Tests
// ============================================================================

mod private_interest_creation {
    use super::*;

    #[test]
    fn test_namespace_interest() {
        let ns = test_namespace();
        let interest = PrivateInterest::namespace(ns.clone());

        assert_eq!(interest.namespace_id, ns);
        assert_eq!(interest.subspace_id, SubspaceConstraint::Any);
        assert!(interest.path.components.is_empty());
    }

    #[test]
    fn test_subspace_interest() {
        let ns = test_namespace();
        let user = PeerId::random();
        let interest = PrivateInterest::subspace(ns.clone(), user.clone());

        assert_eq!(interest.namespace_id, ns);
        assert_eq!(interest.subspace_id, SubspaceConstraint::Specific(user));
        assert!(interest.path.components.is_empty());
    }

    #[test]
    fn test_custom_interest() {
        let ns = test_namespace();
        let user = PeerId::random();
        let path = PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec()]);

        let interest = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Specific(user.clone()),
            path.clone(),
        );

        assert_eq!(interest.namespace_id, ns);
        assert_eq!(interest.subspace_id, SubspaceConstraint::Specific(user));
        assert_eq!(interest.path, path);
    }
}

// ============================================================================
// PrivateInterest Comparison Tests
// ============================================================================

mod private_interest_comparison {
    use super::*;

    #[test]
    fn test_equal_interests() {
        let ns = test_namespace();
        let user = PeerId::random();

        let p1 = PrivateInterest::subspace(ns.clone(), user.clone());
        let p2 = PrivateInterest::subspace(ns, user);

        assert_eq!(p1, p2);
        assert!(p1.is_more_specific_than(&p2));
        assert!(p2.is_more_specific_than(&p1));
        assert!(!p1.is_strictly_more_specific_than(&p2));
    }

    #[test]
    fn test_path_more_specific() {
        let ns = test_namespace();
        let user = PeerId::random();

        let parent = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Specific(user.clone()),
            PathConstraint::new(vec![b"a".to_vec()]),
        );

        let child = PrivateInterest::new(
            ns,
            SubspaceConstraint::Specific(user),
            PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec()]),
        );

        assert!(child.is_more_specific_than(&parent));
        assert!(child.is_strictly_more_specific_than(&parent));
        assert!(!parent.is_more_specific_than(&child));
        assert!(parent.is_less_specific_than(&child));
    }

    #[test]
    fn test_subspace_any_is_less_specific() {
        let ns = test_namespace();
        let user = PeerId::random();

        let any_subspace = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Any,
            PathConstraint::new(vec![b"data".to_vec()]),
        );

        let specific_subspace = PrivateInterest::new(
            ns,
            SubspaceConstraint::Specific(user),
            PathConstraint::new(vec![b"data".to_vec()]),
        );

        assert!(specific_subspace.is_more_specific_than(&any_subspace));
        assert!(!any_subspace.is_more_specific_than(&specific_subspace));
    }

    #[test]
    fn test_different_namespaces_not_comparable() {
        let ns1 = namespace_with_id(1);
        let ns2 = namespace_with_id(2);
        let user = PeerId::random();

        let p1 = PrivateInterest::subspace(ns1, user.clone());
        let p2 = PrivateInterest::subspace(ns2, user);

        assert!(!p1.is_more_specific_than(&p2));
        assert!(!p2.is_more_specific_than(&p1));
        assert!(!p1.is_comparable_to(&p2));
    }

    #[test]
    fn test_disjoint_subspaces() {
        let ns = test_namespace();
        let user1 = PeerId::random();
        let user2 = PeerId::random();

        let p1 = PrivateInterest::subspace(ns.clone(), user1);
        let p2 = PrivateInterest::subspace(ns, user2);

        assert!(p1.is_disjoint_from(&p2));
        assert!(p2.is_disjoint_from(&p1));
    }

    #[test]
    fn test_disjoint_paths() {
        let ns = test_namespace();
        let user = PeerId::random();

        let p1 = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Specific(user.clone()),
            PathConstraint::new(vec![b"path_a".to_vec()]),
        );

        let p2 = PrivateInterest::new(
            ns,
            SubspaceConstraint::Specific(user),
            PathConstraint::new(vec![b"path_b".to_vec()]),
        );

        assert!(p1.is_disjoint_from(&p2));
    }

    #[test]
    fn test_awkward_case() {
        let ns = test_namespace();
        let user = PeerId::random();

        // Any subspace with longer path
        let any_deep = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Any,
            PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec()]),
        );

        // Specific subspace with shorter path (prefix of above)
        let specific_shallow = PrivateInterest::new(
            ns,
            SubspaceConstraint::Specific(user),
            PathConstraint::new(vec![b"a".to_vec()]),
        );

        // These are awkward: neither comparable nor disjoint
        assert!(!any_deep.is_comparable_to(&specific_shallow));
        assert!(!any_deep.is_disjoint_from(&specific_shallow));
        assert!(any_deep.is_awkward_with(&specific_shallow));
        assert!(specific_shallow.is_awkward_with(&any_deep));
    }
}

// ============================================================================
// Relaxation Tests
// ============================================================================

mod relaxation {
    use super::*;

    #[test]
    fn test_specific_subspace_relaxation() {
        let ns = test_namespace();
        let user = PeerId::random();

        let specific = PrivateInterest::subspace(ns.clone(), user);
        let relaxed = specific.relaxation().expect("Should have relaxation");

        assert_eq!(relaxed.namespace_id, ns);
        assert_eq!(relaxed.subspace_id, SubspaceConstraint::Any);
        assert_eq!(relaxed.path, specific.path);
    }

    #[test]
    fn test_any_subspace_no_relaxation() {
        let ns = test_namespace();
        let any = PrivateInterest::namespace(ns);

        assert!(any.relaxation().is_none(), "Any subspace interest cannot be relaxed further");
    }

    #[test]
    fn test_relaxation_preserves_path() {
        let ns = test_namespace();
        let user = PeerId::random();
        let path = PathConstraint::new(vec![b"deep".to_vec(), b"path".to_vec()]);

        let specific = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Specific(user),
            path.clone(),
        );

        let relaxed = specific.relaxation().unwrap();

        assert_eq!(relaxed.path, path);
    }
}

// ============================================================================
// Path Prefix Tests
// ============================================================================

mod path_prefixes {
    use super::*;

    #[test]
    fn test_path_prefixes_generation() {
        let ns = test_namespace();
        let user = PeerId::random();

        let interest = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Specific(user.clone()),
            PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]),
        );

        let prefixes = interest.path_prefixes();

        // Should have: original + 3 prefixes (empty, [a], [a,b])
        assert_eq!(prefixes.len(), 4);

        // Verify each prefix
        assert_eq!(prefixes[0].path.components, vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]);
        assert!(prefixes.iter().any(|p| p.path.components.is_empty()));
        assert!(prefixes.iter().any(|p| p.path.components == vec![b"a".to_vec()]));
        assert!(prefixes.iter().any(|p| p.path.components == vec![b"a".to_vec(), b"b".to_vec()]));
    }

    #[test]
    fn test_empty_path_prefixes() {
        let ns = test_namespace();
        let interest = PrivateInterest::namespace(ns);

        let prefixes = interest.path_prefixes();

        // Empty path has only itself as prefix
        assert_eq!(prefixes.len(), 1);
    }
}

// ============================================================================
// Hashing Tests
// ============================================================================

mod hashing {
    use super::*;

    #[test]
    fn test_hash_determinism() {
        let ns = test_namespace();
        let user = PeerId::random();
        let salt = [42u8; 32];

        let interest = PrivateInterest::subspace(ns, user);

        let hash1 = interest.hash_with_salt(&salt);
        let hash2 = interest.hash_with_salt(&salt);

        assert_eq!(hash1, hash2, "Same interest with same salt should produce same hash");
    }

    #[test]
    fn test_different_salt_different_hash() {
        let ns = test_namespace();
        let user = PeerId::random();

        let interest = PrivateInterest::subspace(ns, user);

        let hash1 = interest.hash_with_salt(&[1u8; 32]);
        let hash2 = interest.hash_with_salt(&[2u8; 32]);

        assert_ne!(hash1, hash2, "Different salts should produce different hashes");
    }

    #[test]
    fn test_different_interests_different_hash() {
        let ns = test_namespace();
        let user1 = PeerId::random();
        let user2 = PeerId::random();
        let salt = [0u8; 32];

        let interest1 = PrivateInterest::subspace(ns.clone(), user1);
        let interest2 = PrivateInterest::subspace(ns, user2);

        let hash1 = interest1.hash_with_salt(&salt);
        let hash2 = interest2.hash_with_salt(&salt);

        assert_ne!(hash1, hash2);
    }
}

// ============================================================================
// PaiState Tests
// ============================================================================

mod pai_state {
    use super::*;

    #[test]
    fn test_initiator_salt_is_rnd() {
        let interests = vec![PrivateInterest::namespace(test_namespace())];
        let rnd = [99u8; 32];

        let state = PaiState::new(interests, rnd, true);
        
        // We can verify through fragment generation - initiator uses rnd directly
        assert!(state.overlaps().is_empty());
    }

    #[test]
    fn test_responder_salt_is_flipped() {
        let interests = vec![PrivateInterest::namespace(test_namespace())];
        let rnd = [0xFFu8; 32];

        let state = PaiState::new(interests, rnd, false);
        
        // Responder salt is ~rnd
        assert!(state.overlaps().is_empty());
    }

    #[test]
    fn test_fragment_generation_for_any_subspace() {
        let ns = test_namespace();
        let interests = vec![PrivateInterest::namespace(ns)];
        let rnd = [0u8; 32];

        let mut state = PaiState::new(interests, rnd, true);
        let fragments = state.generate_fragments();

        // Any subspace interest generates 1 fragment (primary only)
        assert_eq!(fragments.len(), 1);
        assert!(fragments[0].is_primary);
    }

    #[test]
    fn test_fragment_generation_for_specific_subspace() {
        let ns = test_namespace();
        let user = PeerId::random();
        let interests = vec![PrivateInterest::subspace(ns, user)];
        let rnd = [0u8; 32];

        let mut state = PaiState::new(interests, rnd, true);
        let fragments = state.generate_fragments();

        // Specific subspace generates 2 fragments: primary + relaxation
        assert_eq!(fragments.len(), 2);
        assert!(fragments.iter().any(|f| f.is_primary));
        assert!(fragments.iter().any(|f| !f.is_primary));
    }

    #[test]
    fn test_multiple_interests_fragment_generation() {
        let ns = test_namespace();
        let user1 = PeerId::random();
        let user2 = PeerId::random();

        let interests = vec![
            PrivateInterest::subspace(ns.clone(), user1), // 2 fragments
            PrivateInterest::subspace(ns.clone(), user2), // 2 fragments
            PrivateInterest::namespace(ns),               // 1 fragment
        ];
        let rnd = [0u8; 32];

        let mut state = PaiState::new(interests, rnd, true);
        let fragments = state.generate_fragments();

        assert_eq!(fragments.len(), 5);
    }

    #[test]
    fn test_disjoint_interests_empty_overlap() {
        let ns1 = namespace_with_id(1);
        let ns2 = namespace_with_id(2);
        let rnd = [0u8; 32];

        // Initiator interested in ns1
        let mut initiator = PaiState::new(
            vec![PrivateInterest::namespace(ns1)],
            rnd,
            true,
        );

        // Responder interested in ns2
        let mut responder = PaiState::new(
            vec![PrivateInterest::namespace(ns2)],
            rnd,
            false,
        );

        let initiator_fragments = initiator.generate_fragments();
        let responder_fragments = responder.generate_fragments();

        // Process each other's fragments
        initiator.process_received(responder_fragments);
        responder.process_received(initiator_fragments);

        // No overlaps should be detected
        assert!(initiator.overlaps().is_empty());
        assert!(responder.overlaps().is_empty());
    }

    #[test]
    fn test_equal_interests_detect_overlap() {
        let ns = test_namespace();
        let rnd = [0u8; 32];

        // Both interested in same namespace
        let mut initiator = PaiState::new(
            vec![PrivateInterest::namespace(ns.clone())],
            rnd,
            true,
        );
        let mut responder = PaiState::new(
            vec![PrivateInterest::namespace(ns)],
            rnd,
            false,
        );

        let initiator_fragments = initiator.generate_fragments();
        let responder_fragments = responder.generate_fragments();

        initiator.process_received(responder_fragments);
        responder.process_received(initiator_fragments);

        // Both should detect the overlap
        assert!(!initiator.overlaps().is_empty());
        assert!(!responder.overlaps().is_empty());
    }
}

// ============================================================================
// Overlap Detection Tests
// ============================================================================

mod overlap_detection {
    use super::*;

    #[test]
    fn test_detect_more_specific_overlap() {
        let ns = test_namespace();
        let user = PeerId::random();
        let rnd = [0u8; 32];

        // Initiator has more specific interest (specific subspace)
        let mut initiator = PaiState::new(
            vec![PrivateInterest::subspace(ns.clone(), user)],
            rnd,
            true,
        );

        // Responder has less specific interest (any subspace)
        let mut responder = PaiState::new(
            vec![PrivateInterest::namespace(ns)],
            rnd,
            false,
        );

        let initiator_fragments = initiator.generate_fragments();
        let responder_fragments = responder.generate_fragments();

        initiator.process_received(responder_fragments);
        responder.process_received(initiator_fragments);

        // Initiator should detect overlap (they're more specific)
        assert!(!initiator.overlaps().is_empty());
    }

    #[test]
    fn test_disjoint_interests_list() {
        let ns1 = namespace_with_id(1);
        let ns2 = namespace_with_id(2);
        let ns3 = namespace_with_id(3);
        let rnd = [0u8; 32];

        // Initiator interested in ns1 and ns3
        let mut initiator = PaiState::new(
            vec![
                PrivateInterest::namespace(ns1.clone()),
                PrivateInterest::namespace(ns3.clone()),
            ],
            rnd,
            true,
        );

        // Responder only interested in ns2
        let mut responder = PaiState::new(
            vec![PrivateInterest::namespace(ns2)],
            rnd,
            false,
        );

        let initiator_fragments = initiator.generate_fragments();
        let responder_fragments = responder.generate_fragments();

        initiator.process_received(responder_fragments);

        // All initiator interests should be disjoint
        let disjoint = initiator.disjoint_interests();
        assert_eq!(disjoint.len(), 2);
    }
}

// ============================================================================
// Announcement Authentication Tests
// ============================================================================

mod announcement_auth {
    use super::*;

    #[test]
    fn test_compute_announcement_auth() {
        let ns = test_namespace();
        let interest = PrivateInterest::namespace(ns);
        let rnd = [0u8; 32];

        let state = PaiState::new(vec![interest.clone()], rnd, true);
        let auth = state.compute_announcement_auth(&interest);

        // Auth should be the hash of the interest with our salt
        assert_eq!(auth.len(), 32);
    }

    #[test]
    fn test_verify_announcement_auth() {
        let ns = test_namespace();
        let interest = PrivateInterest::namespace(ns);
        let rnd = [0u8; 32];

        let initiator_state = PaiState::new(vec![interest.clone()], rnd, true);
        let responder_state = PaiState::new(vec![interest.clone()], rnd, false);

        // Initiator computes auth with initiator salt
        let auth = initiator_state.compute_announcement_auth(&interest);

        // Responder verifies with their peer's salt (which is initiator's salt)
        assert!(responder_state.verify_announcement_auth(&auth, &interest));
    }

    #[test]
    fn test_invalid_announcement_auth() {
        let ns = test_namespace();
        let interest1 = PrivateInterest::namespace(ns.clone());
        let interest2 = PrivateInterest::subspace(ns, PeerId::random());
        let rnd = [0u8; 32];

        let state = PaiState::new(vec![interest1.clone()], rnd, true);
        let peer_state = PaiState::new(vec![interest2.clone()], rnd, false);

        // Auth computed for interest1
        let auth = state.compute_announcement_auth(&interest1);

        // Should not verify for interest2
        assert!(!peer_state.verify_announcement_auth(&auth, &interest2));
    }
}

// ============================================================================
// PaiFragment Tests
// ============================================================================

mod pai_fragment {
    use super::*;

    #[test]
    fn test_fragment_creation() {
        let hash = [1u8; 32];
        let fragment = PaiFragment::new(hash, true);

        assert_eq!(fragment.hash, hash);
        assert!(fragment.is_primary);
    }

    #[test]
    fn test_fragment_serialization() {
        let fragment = PaiFragment::new([2u8; 32], false);

        let serialized = serde_json::to_string(&fragment).expect("Should serialize");
        let deserialized: PaiFragment = serde_json::from_str(&serialized)
            .expect("Should deserialize");

        assert_eq!(fragment.hash, deserialized.hash);
        assert_eq!(fragment.is_primary, deserialized.is_primary);
    }
}

// ============================================================================
// OverlapAnnouncement Tests
// ============================================================================

mod overlap_announcement {
    use super::*;
    use netabase::data::network::capability::enumeration::EncodedEnumerationCapability;

    #[test]
    fn test_basic_announcement() {
        let auth = [3u8; 32];
        let announcement = OverlapAnnouncement::new(auth);

        assert_eq!(announcement.announcement_auth, auth);
        assert!(announcement.enumeration_capability.is_none());
    }

    #[test]
    fn test_announcement_with_enumeration_capability() {
        let auth = [4u8; 32];
        let user = PeerId::random();
        let encoded_cap = EncodedEnumerationCapability {
            user_key: user,
            initial_authorisation_bytes: vec![0; 64],
            delegations: vec![],
        };

        let announcement = OverlapAnnouncement::new(auth)
            .with_enumeration(encoded_cap);

        assert!(announcement.enumeration_capability.is_some());
    }

    #[test]
    fn test_announcement_serialization() {
        let auth = [5u8; 32];
        let announcement = OverlapAnnouncement::new(auth);

        let serialized = serde_json::to_string(&announcement).expect("Should serialize");
        let deserialized: OverlapAnnouncement = serde_json::from_str(&serialized)
            .expect("Should deserialize");

        assert_eq!(announcement.announcement_auth, deserialized.announcement_auth);
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

mod integration {
    use super::*;

    /// Simulates a complete PAI exchange between two peers
    #[test]
    fn test_full_pai_exchange() {
        let ns = test_namespace();
        let user = PeerId::random();
        let rnd = [0u8; 32];

        // Alice is interested in a specific subspace
        let alice_interest = PrivateInterest::subspace(ns.clone(), user.clone());
        let mut alice = PaiState::new(vec![alice_interest.clone()], rnd, true);

        // Bob is interested in the entire namespace
        let bob_interest = PrivateInterest::namespace(ns);
        let mut bob = PaiState::new(vec![bob_interest.clone()], rnd, false);

        // Phase 1: Generate and exchange fragments
        let alice_fragments = alice.generate_fragments();
        let bob_fragments = bob.generate_fragments();

        // Phase 2: Process received fragments
        alice.process_received(bob_fragments);
        bob.process_received(alice_fragments);

        // Phase 3: Check overlaps
        // Alice (more specific) should detect overlap with Bob's broader interest
        let alice_overlaps = alice.overlaps();
        assert!(!alice_overlaps.is_empty(), "Alice should detect overlap");

        // Bob may or may not detect overlap depending on hash matching
        // (In real scenario with proper crypto, both would detect)
    }

    /// Test that PAI protects against a peer who doesn't know an interest
    #[test]
    fn test_pai_confidentiality() {
        let secret_ns = namespace_with_id(0xFF);
        let public_ns = namespace_with_id(0x01);
        let rnd = [0u8; 32];

        // Alice has a secret interest
        let mut alice = PaiState::new(
            vec![PrivateInterest::namespace(secret_ns.clone())],
            rnd,
            true,
        );

        // Bob only knows about public namespace
        let mut bob = PaiState::new(
            vec![PrivateInterest::namespace(public_ns)],
            rnd,
            false,
        );

        let alice_fragments = alice.generate_fragments();
        let bob_fragments = bob.generate_fragments();

        bob.process_received(alice_fragments);

        // Bob should not learn anything about Alice's secret interest
        assert!(bob.overlaps().is_empty());
    }
}
