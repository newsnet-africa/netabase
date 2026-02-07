//! # Private Interest and Private Area Intersection (PAI)
//!
//! This module implements the confidential sync mechanism from the Willow protocol.
//! PAI allows peers to discover shared interests without revealing information
//! about interests they don't share.
//!
//! ## The Problem
//!
//! When syncing data, peers need to determine what they have in common:
//! - Which namespaces do we both care about?
//! - Which subspaces within those namespaces?
//! - Which path prefixes?
//!
//! Naively exchanging this information leaks sensitive data:
//! - A malicious peer learns what namespaces exist
//! - Path names could reveal private information
//! - Even learning "they have namespace X" is information
//!
//! ## The Solution: Salted Hashes
//!
//! Instead of exchanging interests directly, peers exchange **salted hashes**:
//!
//! ```text
//! Alice's Interest: (namespace: N, subspace: S, path: /a/b)
//!         │
//!         ▼ hash with Alice's salt
//! Hash Fragment: 0xabc123...
//!
//! Bob receives 0xabc123... but cannot reverse it without knowing:
//!   - The exact namespace N
//!   - The exact subspace S
//!   - The exact path /a/b
//! ```
//!
//! ## Salt Derivation
//!
//! Salts are derived from the handshake to prevent replay attacks:
//!
//! ```text
//! Handshake produces random bytestring `rnd`
//!
//! Initiator salt = rnd
//! Responder salt = ~rnd (bitwise NOT)
//!
//! This prevents peers from mirroring hashes back
//! ```
//!
//! ## Key Concepts
//!
//! ### Private Interest
//!
//! A [`PrivateInterest`] is a triple `(namespace_id, subspace_id, path)` that
//! represents data a peer wants to sync:
//!
//! ```text
//! PrivateInterest {
//!     namespace_id: [32 bytes]    // Which subscription/namespace
//!     subspace_id: Any | Specific // Whose data (any user or specific)
//!     path: [components]          // Path prefix filter
//! }
//! ```
//!
//! ### Specificity Relationships
//!
//! Interests have specificity relationships:
//!
//! ```text
//! More Specific ◄──────────────────────────────── Less Specific
//!
//! (ns=X, sub=Alice, path=/a/b/c)
//!     is more specific than
//! (ns=X, sub=Alice, path=/a/b)
//!     is more specific than
//! (ns=X, sub=Alice, path=/a)
//!     is more specific than
//! (ns=X, sub=Any, path=/a)
//!     is more specific than
//! (ns=X, sub=Any, path=/)
//! ```
//!
//! ### Relaxation
//!
//! A [`PrivateInterest`] with a specific subspace can be "relaxed" to `Any`:
//!
//! ```text
//! Original:  (ns=X, sub=Alice, path=/data)
//! Relaxed:   (ns=X, sub=Any,   path=/data)
//! ```
//!
//! Relaxed interests are sent as additional fragments (with `is_primary=false`)
//! to enable detection of overlaps where one peer has `Any` and the other
//! has a specific subspace.
//!
//! ### Awkward Interests
//!
//! Two interests are "awkward" if neither is more specific than the other,
//! yet they're not disjoint:
//!
//! ```text
//! Interest A: (ns=X, sub=Any,   path=/a/b)
//! Interest B: (ns=X, sub=Alice, path=/a)
//!
//! Neither includes the other:
//! - A has Any subspace but longer path
//! - B has specific subspace but shorter path
//!
//! Yet they overlap: Alice's entries at /a/b/... match both
//! ```
//!
//! Awkward cases require [`McEnumerationCapability`](super::McEnumerationCapability)
//! to resolve without leaking information.
//!
//! ## Protocol Flow
//!
//! ### Phase 1: Fragment Generation
//!
//! Each peer generates fragments for their interests:
//!
//! ```text
//! For each interest:
//!   if subspace == Any:
//!     send: hash(salt, interest), is_primary=true
//!   else (subspace == Specific):
//!     send: hash(salt, interest), is_primary=true
//!     send: hash(salt, relaxed_interest), is_primary=false
//! ```
//!
//! ### Phase 2: Fragment Exchange
//!
//! Peers exchange fragments (can be concurrent):
//!
//! ```text
//! Initiator ──────[fragments]──────► Responder
//! Initiator ◄─────[fragments]─────── Responder
//! ```
//!
//! ### Phase 3: Local Comparison
//!
//! Each peer computes local comparison hashes using the **peer's salt**:
//!
//! ```text
//! For each of my interests:
//!   For each path prefix (including empty):
//!     compute hash(peer_salt, interest_with_prefix)
//!     if hash matches any received fragment:
//!       detect overlap
//! ```
//!
//! ### Phase 4: Overlap Announcement
//!
//! When overlap is detected, the detecting peer announces it:
//!
//! ```text
//! OverlapAnnouncement {
//!     // Proves we know the interest (hashed with our salt)
//!     announcement_auth: hash(my_salt, interest)
//!     
//!     // For awkward cases, include enumeration capability
//!     enumeration_capability: Option<EncodedEnumerationCapability>
//! }
//! ```
//!
//! ### Phase 5: Capability Exchange
//!
//! After announcements, peers exchange capabilities for overlapping interests.
//!
//! ## Security Properties
//!
//! ### What's Protected
//!
//! - **NamespaceIds**: Attacker cannot learn namespace IDs they don't already know
//! - **SubspaceIds**: Attacker cannot learn who participates in a namespace
//! - **Paths**: Attacker cannot learn path structure
//!
//! ### What's NOT Protected
//!
//! - **Timestamps**: Easily guessable, not hidden
//! - **Guess Confirmation**: Attacker can confirm guesses about data
//!
//! This is why NamespaceIds, SubspaceIds, and Paths should be:
//! - Sufficiently long and random-looking
//! - Encrypted per-subspace if needed
//!
//! ## Example: Full PAI Exchange
//!
//! ```rust
//! use netabase::data::network::capability::{PaiState, PrivateInterest, OverlapAnnouncement};
//! use netabase::data::util::encryption::NamespacePublicKey;
//! use libp2p::PeerId;
//!
//! // Create a shared namespace
//! let namespace_x = NamespacePublicKey::new([10u8; 32]);
//! let alice_id = PeerId::random();
//!
//! // Alice wants to sync namespace X, subspace her_id
//! let alice_interests = vec![
//!     PrivateInterest::subspace(namespace_x.clone(), alice_id.clone()),
//! ];
//!
//! // Bob wants to sync entire namespace X (all subspaces)
//! let bob_interests = vec![
//!     PrivateInterest::namespace(namespace_x.clone()),
//! ];
//!
//! // rnd from handshake (simulated here)
//! let rnd = [42u8; 32];
//!
//! // Alice is initiator, Bob is responder
//! let mut alice_state = PaiState::new(alice_interests.clone(), rnd, true);
//! let mut bob_state = PaiState::new(bob_interests, rnd, false);
//!
//! // Generate and exchange fragments
//! let alice_frags = alice_state.generate_fragments();
//! let bob_frags = bob_state.generate_fragments();
//!
//! // Each peer processes the other's fragments
//! alice_state.process_received(bob_frags);
//! bob_state.process_received(alice_frags);
//!
//! // Alice detects overlap (her interest is more specific than Bob's Any-subspace)
//! // Note: The exact overlap detection depends on the hash matching logic
//! // In this example, Alice's specific subspace interest will match Bob's
//! // namespace-wide interest.
//!
//! // Alice can send an overlap announcement with authentication
//! let auth = alice_state.compute_announcement_auth(&alice_interests[0]);
//! let announcement = OverlapAnnouncement::new(auth);
//!
//! // The announcement contains the salted hash as proof
//! assert_eq!(announcement.announcement_auth, auth);
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::data::util::encryption::NamespacePublicKey;

use super::area::{PathConstraint, SubspaceConstraint, SubspaceId};

/// A private interest represents data a peer is interested in synchronizing.
///
/// This is the confidential data that the PAI protocol protects:
/// - NamespaceId (subscription identifier)
/// - SubspaceId (who owns the data, or Any)
/// - Path (where in the hierarchy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateInterest {
    /// The namespace (subscription) this interest is for
    pub namespace_id: NamespacePublicKey,
    /// The subspace constraint (Any or Specific user)
    pub subspace_id: SubspaceConstraint,
    /// The path prefix we're interested in
    pub path: PathConstraint,
}

impl PrivateInterest {
    /// Create a new private interest
    pub fn new(
        namespace_id: NamespacePublicKey,
        subspace_id: SubspaceConstraint,
        path: PathConstraint,
    ) -> Self {
        Self {
            namespace_id,
            subspace_id,
            path,
        }
    }

    /// Create an interest for an entire namespace
    pub fn namespace(namespace_id: NamespacePublicKey) -> Self {
        Self {
            namespace_id,
            subspace_id: SubspaceConstraint::Any,
            path: PathConstraint::empty(),
        }
    }

    /// Create an interest for a specific subspace in a namespace
    pub fn subspace(namespace_id: NamespacePublicKey, subspace_id: SubspaceId) -> Self {
        Self {
            namespace_id,
            subspace_id: SubspaceConstraint::Specific(subspace_id),
            path: PathConstraint::empty(),
        }
    }

    /// Check if this interest is more specific than another
    ///
    /// p1 is more specific than p2 if:
    /// - Same namespace_id
    /// - p2.subspace_id == Any OR p1.subspace_id == p2.subspace_id
    /// - p1.path is an extension of p2.path
    pub fn is_more_specific_than(&self, other: &Self) -> bool {
        // Must be same namespace
        if self.namespace_id != other.namespace_id {
            return false;
        }

        // Subspace check
        let subspace_ok = match &other.subspace_id {
            SubspaceConstraint::Any => true,
            SubspaceConstraint::Specific(s) => {
                matches!(&self.subspace_id, SubspaceConstraint::Specific(s2) if s == s2)
            }
        };

        if !subspace_ok {
            return false;
        }

        // Path check: self.path must be an extension of other.path
        other.path.is_prefix_of(&self.path)
    }

    /// Check if this interest is strictly more specific than another
    pub fn is_strictly_more_specific_than(&self, other: &Self) -> bool {
        self.is_more_specific_than(other) && self != other
    }

    /// Check if this interest is less specific than another
    pub fn is_less_specific_than(&self, other: &Self) -> bool {
        other.is_more_specific_than(self)
    }

    /// Check if two interests are comparable (one is more specific than the other)
    pub fn is_comparable_to(&self, other: &Self) -> bool {
        self.is_more_specific_than(other) || other.is_more_specific_than(self)
    }

    /// Check if two interests are disjoint (no entry can be in both)
    pub fn is_disjoint_from(&self, other: &Self) -> bool {
        // Different namespaces are always disjoint
        if self.namespace_id != other.namespace_id {
            return true;
        }

        // Check subspace
        match (&self.subspace_id, &other.subspace_id) {
            (SubspaceConstraint::Specific(a), SubspaceConstraint::Specific(b)) if a != b => {
                return true;
            }
            _ => {}
        }

        // Check paths - disjoint if neither is a prefix of the other
        !self.path.is_prefix_of(&other.path) && !other.path.is_prefix_of(&self.path)
    }

    /// Check if two interests are "awkward"
    ///
    /// Awkward means neither comparable nor disjoint. This happens when one
    /// has subspace_id Any with path P, and the other has a specific subspace
    /// with a path that is a strict prefix of P.
    pub fn is_awkward_with(&self, other: &Self) -> bool {
        !self.is_comparable_to(other) && !self.is_disjoint_from(other)
    }

    /// Get the relaxation of this interest (replace specific subspace with Any)
    pub fn relaxation(&self) -> Option<Self> {
        match &self.subspace_id {
            SubspaceConstraint::Any => None, // Already relaxed
            SubspaceConstraint::Specific(_) => Some(Self {
                namespace_id: self.namespace_id.clone(),
                subspace_id: SubspaceConstraint::Any,
                path: self.path.clone(),
            }),
        }
    }

    /// Compute hash with a salt
    pub fn hash_with_salt(&self, salt: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(salt);
        hasher.update(self.namespace_id.as_bytes());

        match &self.subspace_id {
            SubspaceConstraint::Any => hasher.update(&[0x00]),
            SubspaceConstraint::Specific(s) => {
                hasher.update(&[0x01]);
                hasher.update(&s.to_bytes());
            }
        }

        hasher.update(&self.path.hash());
        hasher.finalize().into()
    }

    /// Get all path prefixes (for local comparison)
    pub fn path_prefixes(&self) -> Vec<Self> {
        let mut prefixes = Vec::new();

        // Include self
        prefixes.push(self.clone());

        // Add each prefix by removing components from the end
        for i in 0..self.path.components.len() {
            prefixes.push(Self {
                namespace_id: self.namespace_id.clone(),
                subspace_id: self.subspace_id.clone(),
                path: PathConstraint::new(self.path.components[..i].to_vec()),
            });
        }

        prefixes
    }
}

impl PartialEq for PrivateInterest {
    fn eq(&self, other: &Self) -> bool {
        self.namespace_id == other.namespace_id
            && self.subspace_id == other.subspace_id
            && self.path == other.path
    }
}

impl Eq for PrivateInterest {}

/// A PAI fragment sent during the protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaiFragment {
    /// The salted hash of the interest
    pub hash: [u8; 32],
    /// True if this is a primary interest, false if it's just a relaxation
    pub is_primary: bool,
}

impl PaiFragment {
    pub fn new(hash: [u8; 32], is_primary: bool) -> Self {
        Self { hash, is_primary }
    }
}

/// Type of overlap detected between interests
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlapType {
    /// Both interests are equal
    Equal,
    /// We have the more specific interest
    WeMoreSpecific,
    /// Peer has the more specific interest
    PeerMoreSpecific,
    /// Awkward case: requires enumeration capability
    Awkward,
}

/// A detected overlap during PAI
#[derive(Debug, Clone)]
pub struct DetectedOverlap {
    /// Our interest that overlapped
    pub our_interest: PrivateInterest,
    /// Type of overlap
    pub overlap_type: OverlapType,
    /// Hash that matched (for verification)
    pub matched_hash: [u8; 32],
}

/// State machine for the PAI protocol
pub struct PaiState {
    /// Our interests
    our_interests: Vec<PrivateInterest>,
    /// Our salt (initiator uses rnd, responder uses ~rnd)
    our_salt: [u8; 32],
    /// Peer's salt
    peer_salt: [u8; 32],
    /// Fragments we've sent
    sent_fragments: Vec<PaiFragment>,
    /// Fragments we've received
    received_fragments: Vec<PaiFragment>,
    /// Detected overlaps
    detected_overlaps: Vec<DetectedOverlap>,
    /// Whether we are the initiator
    is_initiator: bool,
}

impl PaiState {
    /// Create a new PAI state
    ///
    /// The `rnd` is the random bytestring from the handshake.
    /// Initiator uses `rnd` as their salt, responder flips bits.
    pub fn new(
        interests: Vec<PrivateInterest>,
        rnd: [u8; 32],
        is_initiator: bool,
    ) -> Self {
        let (our_salt, peer_salt) = if is_initiator {
            let flipped: [u8; 32] = rnd.map(|b| !b);
            (rnd, flipped)
        } else {
            let flipped: [u8; 32] = rnd.map(|b| !b);
            (flipped, rnd)
        };

        Self {
            our_interests: interests,
            our_salt,
            peer_salt,
            sent_fragments: Vec::new(),
            received_fragments: Vec::new(),
            detected_overlaps: Vec::new(),
            is_initiator,
        }
    }

    /// Generate fragments to send
    pub fn generate_fragments(&mut self) -> Vec<PaiFragment> {
        let mut fragments = Vec::new();

        for interest in &self.our_interests {
            match &interest.subspace_id {
                SubspaceConstraint::Any => {
                    // For Any subspace, send primary hash
                    let hash = interest.hash_with_salt(&self.our_salt);
                    fragments.push(PaiFragment::new(hash, true));
                }
                SubspaceConstraint::Specific(_) => {
                    // For specific subspace, send both primary and relaxation
                    let hash = interest.hash_with_salt(&self.our_salt);
                    fragments.push(PaiFragment::new(hash, true));

                    if let Some(relaxed) = interest.relaxation() {
                        let relaxed_hash = relaxed.hash_with_salt(&self.our_salt);
                        fragments.push(PaiFragment::new(relaxed_hash, false));
                    }
                }
            }
        }

        self.sent_fragments = fragments.clone();
        fragments
    }

    /// Compute local hashes for comparison (using peer's salt)
    fn compute_local_hashes(&self) -> Vec<([u8; 32], PrivateInterest, bool)> {
        let mut hashes = Vec::new();

        for interest in &self.our_interests {
            // Compute hashes for the interest and all path prefixes
            for prefix_interest in interest.path_prefixes() {
                match &prefix_interest.subspace_id {
                    SubspaceConstraint::Any => {
                        let hash = prefix_interest.hash_with_salt(&self.peer_salt);
                        hashes.push((hash, prefix_interest, true));
                    }
                    SubspaceConstraint::Specific(_) => {
                        let hash = prefix_interest.hash_with_salt(&self.peer_salt);
                        hashes.push((hash, prefix_interest.clone(), true));

                        if let Some(relaxed) = prefix_interest.relaxation() {
                            let relaxed_hash = relaxed.hash_with_salt(&self.peer_salt);
                            hashes.push((relaxed_hash, relaxed, false));
                        }
                    }
                }
            }
        }

        hashes
    }

    /// Process received fragments and detect overlaps
    pub fn process_received(&mut self, fragments: Vec<PaiFragment>) {
        self.received_fragments.extend(fragments.iter().cloned());

        let local_hashes = self.compute_local_hashes();

        for fragment in &fragments {
            // Find matching local hash
            for (local_hash, interest, is_local_primary) in &local_hashes {
                if local_hash == &fragment.hash {
                    // At least one must be primary for this to count
                    if !fragment.is_primary && !is_local_primary {
                        continue;
                    }

                    // Determine overlap type
                    let overlap_type = self.determine_overlap_type(interest, &fragment);

                    // Avoid duplicates
                    let already_detected = self.detected_overlaps.iter().any(|o| {
                        o.our_interest == *interest && o.overlap_type == overlap_type
                    });

                    if !already_detected {
                        self.detected_overlaps.push(DetectedOverlap {
                            our_interest: interest.clone(),
                            overlap_type,
                            matched_hash: *local_hash,
                        });
                    }
                }
            }
        }
    }

    /// Determine the overlap type for a matched interest
    fn determine_overlap_type(&self, our_interest: &PrivateInterest, fragment: &PaiFragment) -> OverlapType {
        // If both are primary and hashes match exactly for the same interest, it's Equal
        if fragment.is_primary {
            // Check if our actual interests (not prefixes) match
            for interest in &self.our_interests {
                if interest.hash_with_salt(&self.peer_salt) == fragment.hash {
                    return OverlapType::Equal;
                }
            }
        }

        // If we matched on a prefix, we have the more specific interest
        for interest in &self.our_interests {
            if interest.is_strictly_more_specific_than(our_interest) {
                return OverlapType::WeMoreSpecific;
            }
        }

        // Check for awkward case
        for interest in &self.our_interests {
            if interest.is_awkward_with(our_interest) {
                return OverlapType::Awkward;
            }
        }

        OverlapType::PeerMoreSpecific
    }

    /// Get detected overlaps
    pub fn overlaps(&self) -> &[DetectedOverlap] {
        &self.detected_overlaps
    }

    /// Get interests that had no overlap (disjoint from all peer interests)
    pub fn disjoint_interests(&self) -> Vec<&PrivateInterest> {
        self.our_interests
            .iter()
            .filter(|interest| {
                !self.detected_overlaps.iter().any(|o| &o.our_interest == *interest)
            })
            .collect()
    }

    /// Compute announcement authentication hash
    ///
    /// Used to prove we know the interest when announcing overlap.
    pub fn compute_announcement_auth(&self, interest: &PrivateInterest) -> [u8; 32] {
        interest.hash_with_salt(&self.our_salt)
    }

    /// Verify an announcement authentication from peer
    pub fn verify_announcement_auth(&self, auth: &[u8; 32], interest: &PrivateInterest) -> bool {
        let expected = interest.hash_with_salt(&self.peer_salt);
        auth == &expected
    }
}

/// Overlap announcement message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlapAnnouncement {
    /// Hash that proves we know the interest (salted with announcer's salt)
    pub announcement_auth: [u8; 32],
    /// For awkward cases, include enumeration capability
    pub enumeration_capability: Option<super::enumeration::EncodedEnumerationCapability>,
}

impl OverlapAnnouncement {
    pub fn new(announcement_auth: [u8; 32]) -> Self {
        Self {
            announcement_auth,
            enumeration_capability: None,
        }
    }

    pub fn with_enumeration(
        mut self,
        cap: super::enumeration::EncodedEnumerationCapability,
    ) -> Self {
        self.enumeration_capability = Some(cap);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;

    fn test_namespace() -> NamespacePublicKey {
        NamespacePublicKey::new([1u8; 32])
    }

    #[test]
    fn test_private_interest_more_specific() {
        let ns = test_namespace();
        let user = PeerId::random();

        let p1 = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Specific(user.clone()),
            PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec()]),
        );

        let p2 = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Specific(user.clone()),
            PathConstraint::new(vec![b"a".to_vec()]),
        );

        assert!(p1.is_more_specific_than(&p2));
        assert!(!p2.is_more_specific_than(&p1));
    }

    #[test]
    fn test_private_interest_any_subspace() {
        let ns = test_namespace();
        let user = PeerId::random();

        let p_any = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Any,
            PathConstraint::new(vec![b"a".to_vec()]),
        );

        let p_specific = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Specific(user),
            PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec()]),
        );

        assert!(p_specific.is_more_specific_than(&p_any));
        assert!(!p_any.is_more_specific_than(&p_specific));
    }

    #[test]
    fn test_private_interest_disjoint() {
        let ns = test_namespace();
        let user1 = PeerId::random();
        let user2 = PeerId::random();

        let p1 = PrivateInterest::subspace(ns.clone(), user1);
        let p2 = PrivateInterest::subspace(ns.clone(), user2);

        assert!(p1.is_disjoint_from(&p2));
    }

    #[test]
    fn test_private_interest_awkward() {
        let ns = test_namespace();
        let user = PeerId::random();

        // Any subspace with longer path
        let p1 = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Any,
            PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec()]),
        );

        // Specific subspace with shorter path (prefix of p1's path)
        let p2 = PrivateInterest::new(
            ns.clone(),
            SubspaceConstraint::Specific(user),
            PathConstraint::new(vec![b"a".to_vec()]),
        );

        assert!(p1.is_awkward_with(&p2));
        assert!(p2.is_awkward_with(&p1));
    }

    #[test]
    fn test_pai_state_generate_fragments() {
        let ns = test_namespace();
        let user = PeerId::random();
        let rnd = [0u8; 32];

        let interests = vec![
            PrivateInterest::subspace(ns.clone(), user.clone()),
        ];

        let mut state = PaiState::new(interests, rnd, true);
        let fragments = state.generate_fragments();

        // Should have 2 fragments: primary and relaxation
        assert_eq!(fragments.len(), 2);
        assert!(fragments.iter().any(|f| f.is_primary));
        assert!(fragments.iter().any(|f| !f.is_primary));
    }

    #[test]
    fn test_relaxation() {
        let ns = test_namespace();
        let user = PeerId::random();

        let specific = PrivateInterest::subspace(ns.clone(), user);
        let relaxed = specific.relaxation().unwrap();

        assert_eq!(relaxed.subspace_id, SubspaceConstraint::Any);
        assert_eq!(relaxed.namespace_id, specific.namespace_id);

        let any = PrivateInterest::namespace(ns);
        assert!(any.relaxation().is_none());
    }
}
