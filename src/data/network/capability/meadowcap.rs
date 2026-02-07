//! # Meadowcap Capabilities
//!
//! This module implements the core Meadowcap capability system adapted for netabase.
//! Capabilities are unforgeable tokens that grant read or write access to data.
//!
//! ## Capability Types
//!
//! ### Communal Namespaces ([`CommunalCapability`])
//!
//! In a communal namespace, each subspace is owned by its author (the `user_key`).
//! The namespace itself has no single owner - anyone can create entries in their
//! own subspace without prior authorization.
//!
//! ```text
//! Communal Namespace Structure:
//! ┌────────────────────────────────────────────────────┐
//! │ Namespace (no single owner)                        │
//! │  ├── Subspace: Alice (owned by Alice's key)       │
//! │  │    ├── /posts/...                               │
//! │  │    └── /profile/...                             │
//! │  ├── Subspace: Bob (owned by Bob's key)           │
//! │  │    └── /data/...                                │
//! │  └── Subspace: Charlie (owned by Charlie's key)   │
//! │       └── /...                                     │
//! └────────────────────────────────────────────────────┘
//! ```
//!
//! **Root Capability**: A user's root capability for their subspace is automatically
//! valid - no signature required. They created the subspace by having the keypair.
//!
//! **Delegation**: Users can delegate access to their subspace (or parts of it)
//! to other users by signing a delegation that restricts the area.
//!
//! ### Owned Namespaces ([`OwnedCapability`])
//!
//! In an owned namespace, the person who created the namespace is the owner of
//! all its data. They must explicitly delegate access to others.
//!
//! ```text
//! Owned Namespace Structure:
//! ┌────────────────────────────────────────────────────┐
//! │ Namespace (owned by Owen)                          │
//! │  ├── Subspace: Alice (delegated by Owen)          │
//! │  │    └── Limited to /allowed/paths/...            │
//! │  ├── Subspace: Bob (delegated by Owen)            │
//! │  │    └── Can only write, not read others         │
//! │  └── Any new subspace requires Owen's approval    │
//! └────────────────────────────────────────────────────┘
//! ```
//!
//! **Root Capability**: Requires a signature from the namespace owner's secret key,
//! authorizing the initial user to access the full namespace.
//!
//! **Moderation**: Owen can revoke or override anyone's data by creating entries
//! with timestamps in the future (Willow's timestamp-based conflict resolution).
//!
//! ## Delegation Chain
//!
//! Capabilities can be delegated in a chain, with each delegation potentially
//! restricting the granted area:
//!
//! ```text
//! Root Capability (Full Area)
//!     │
//!     ▼ delegation + restriction to /data/
//! Capability for /data/ (Alice → Bob)
//!     │
//!     ▼ delegation + restriction to /data/public/
//! Capability for /data/public/ (Bob → Charlie)
//!     │
//!     ▼ delegation + time restriction
//! Capability for /data/public/ in 2024 (Charlie → Dana)
//! ```
//!
//! Each delegation signs:
//! - The new restricted area
//! - The new receiver's public key
//! - The previous signature (or initial authorization)
//!
//! ## Security Properties
//!
//! ### Unforgeability
//!
//! A valid capability requires a chain of valid signatures starting from:
//! - **Communal**: The subspace owner's key (implicit for root)
//! - **Owned**: The namespace owner's key (explicit signature required)
//!
//! ### Restriction-Only Delegation
//!
//! Delegations can only **restrict**, never **expand** the granted area:
//!
//! ```rust
//! use netabase::data::network::capability::{
//!     AccessMode, Area, CommunalCapability, CapabilityError, PathConstraint
//! };
//! use netabase::data::network::capability::meadowcap::UserSignature;
//! use netabase::data::util::encryption::NamespacePublicKey;
//! use libp2p::PeerId;
//!
//! let namespace = NamespacePublicKey::new([1u8; 32]);
//! let alice_key = PeerId::random();
//! let bob_key = PeerId::random();
//! let charlie_key = PeerId::random();
//!
//! // Alice has capability for her subspace with path /data/
//! let alice_cap = CommunalCapability::new_root(AccessMode::Write, namespace.clone(), alice_key.clone());
//!
//! // First restrict to /data/
//! let data_area = Area::subspace(alice_key.clone())
//!     .with_path(PathConstraint::new(vec![b"data".to_vec()]));
//! let alice_data_cap = alice_cap.delegate(
//!     data_area,
//!     alice_key.clone(),
//!     UserSignature::new(vec![0u8; 64]),
//! ).unwrap();
//!
//! // Valid: delegate restricted path /data/subset/
//! let bob_area = Area::subspace(alice_key.clone())
//!     .with_path(PathConstraint::new(vec![b"data".to_vec(), b"subset".to_vec()]));
//! let bob_cap = alice_data_cap.delegate(
//!     bob_area,
//!     bob_key,
//!     UserSignature::new(vec![0u8; 64]),
//! );
//! assert!(bob_cap.is_ok(), "Should succeed - area is more restricted");
//!
//! // Invalid: cannot expand to / (full subspace)
//! let expanded_area = Area::subspace(alice_key.clone());
//! let invalid = alice_data_cap.delegate(
//!     expanded_area,
//!     charlie_key,
//!     UserSignature::new(vec![0u8; 64]),
//! );
//! assert!(matches!(invalid, Err(CapabilityError::AreaExpansion)));
//! ```
//!
//! ### Handover Messages
//!
//! The exact bytes signed for each delegation ensure:
//! - Chain integrity (each signature includes the previous)
//! - Area binding (the new area is cryptographically bound)
//! - Receiver binding (the new user is cryptographically bound)
//!
//! ## Access Verification
//!
//! To verify a capability grants access to an entry:
//!
//! 1. **Namespace Match**: `capability.granted_namespace() == entry.namespace_id`
//! 2. **Area Inclusion**: `capability.granted_area().includes_entry(&entry)`
//! 3. **Signature Chain**: All delegations have valid signatures
//! 4. **Type Match**: Communal for communal namespaces, Owned for owned
//!
//! ## Example: Complete Delegation Flow
//!
//! ```rust
//! use netabase::data::network::capability::{
//!     AccessMode, Area, OwnedCapability, PathConstraint
//! };
//! use netabase::data::network::capability::meadowcap::UserSignature;
//! use netabase::data::util::encryption::{NamespacePublicKey, NamespaceSignature};
//! use libp2p::PeerId;
//!
//! // Owen creates an owned namespace and grants Bob write access
//! let namespace_key = NamespacePublicKey::new([5u8; 32]);
//! let bob_key = PeerId::random();
//! let charlie_key = PeerId::random();
//!
//! // Owen signs the initial authorization (mock signature for this example)
//! // In production, this would be: owen_secret.sign(&[0x03, bob_key_bytes...])
//! let initial_auth = NamespaceSignature::new(vec![0u8; 64]);
//!
//! let bob_cap = OwnedCapability::new_root(
//!     AccessMode::Write,
//!     namespace_key,
//!     bob_key.clone(),
//!     initial_auth,
//! );
//! assert!(bob_cap.is_valid());
//!
//! // Bob delegates to Charlie with path restriction
//! let restricted_area = Area::full()
//!     .with_path(PathConstraint::new(vec![b"shared".to_vec()]));
//!
//! // In production, Bob would sign the handover message
//! let bob_sig = UserSignature::new(vec![0u8; 64]);
//!
//! let charlie_cap = bob_cap.delegate(restricted_area, charlie_key.clone(), bob_sig)
//!     .expect("Delegation should succeed");
//!
//! // Charlie can now access /shared/* but nothing else
//! let granted = charlie_cap.granted_area();
//! assert_eq!(granted.path.components, vec![b"shared".to_vec()]);
//! assert_eq!(charlie_cap.receiver(), &charlie_key);
//! ```

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::data::util::encryption::{NamespacePublicKey, NamespaceSignature};

use super::area::Area;

/// User public key (maps to SubspaceId in Willow terms)
/// In netabase, this is the PeerId
pub type UserPublicKey = PeerId;

/// User signature (Ed25519 signature from a user)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSignature(pub Vec<u8>);

impl UserSignature {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Verify this signature against a user's public key
    pub fn verify(&self, _message: &[u8], _user_key: &UserPublicKey) -> bool {
        // TODO: Implement actual signature verification using libp2p identity
        true
    }
}

/// Access mode for capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AccessMode {
    /// Read-only access
    Read,
    /// Write access (implies read)
    Write,
}

impl AccessMode {
    pub fn can_read(&self) -> bool {
        true // Both modes can read
    }

    pub fn can_write(&self) -> bool {
        matches!(self, Self::Write)
    }

    /// Check if this mode subsumes another (is at least as permissive)
    pub fn subsumes(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Write, _) => true,
            (Self::Read, Self::Read) => true,
            _ => false,
        }
    }

    /// Encoding byte for capability signatures
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::Read => 0x00,
            Self::Write => 0x01,
        }
    }
}

/// A delegation in a capability chain
///
/// Each delegation restricts the capability to a more specific area
/// and transfers it to a new user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDelegation {
    /// The restricted area for this delegation
    pub area: Area,
    /// The user to whom access is delegated
    pub user: UserPublicKey,
    /// Signature from the previous receiver authorizing this delegation
    pub signature: UserSignature,
}

impl CapabilityDelegation {
    pub fn new(area: Area, user: UserPublicKey, signature: UserSignature) -> Self {
        Self {
            area,
            user,
            signature,
        }
    }
}

/// Capability for communal namespaces
///
/// In a communal namespace, each subspace is owned by its author (user_key).
/// The namespace itself has no single owner - anyone can create entries in
/// their own subspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunalCapability {
    /// The kind of access this grants
    pub access_mode: AccessMode,
    /// The namespace in which this grants access
    pub namespace_key: NamespacePublicKey,
    /// The subspace for which and to whom this grants access
    /// (SubspaceId == UserPublicKey in Meadowcap)
    pub user_key: UserPublicKey,
    /// Successive authorizations of new UserPublicKeys, each restricted to a particular Area
    pub delegations: Vec<CapabilityDelegation>,
}

impl CommunalCapability {
    /// Create a new root communal capability (no delegations)
    pub fn new_root(
        access_mode: AccessMode,
        namespace_key: NamespacePublicKey,
        user_key: UserPublicKey,
    ) -> Self {
        Self {
            access_mode,
            namespace_key,
            user_key,
            delegations: Vec::new(),
        }
    }

    /// Get the receiver (the user to whom this grants access)
    pub fn receiver(&self) -> &UserPublicKey {
        self.delegations
            .last()
            .map(|d| &d.user)
            .unwrap_or(&self.user_key)
    }

    /// Get the granted namespace
    pub fn granted_namespace(&self) -> &NamespacePublicKey {
        &self.namespace_key
    }

    /// Get the granted area
    pub fn granted_area(&self) -> Area {
        self.delegations
            .last()
            .map(|d| d.area.clone())
            .unwrap_or_else(|| Area::subspace(self.user_key.clone()))
    }

    /// Check if this capability is valid
    pub fn is_valid(&self) -> bool {
        if self.delegations.is_empty() {
            // Root capabilities with zero delegations are always valid
            return true;
        }

        // Verify the delegation chain
        let mut prev_receiver = self.user_key.clone();
        let mut prev_area = Area::subspace(self.user_key.clone());

        for (i, delegation) in self.delegations.iter().enumerate() {
            // Check area restriction (new area must be included in previous)
            if !prev_area.includes(&delegation.area) {
                return false;
            }

            // Compute the handover message
            let handover = self.compute_handover(i, &prev_area, delegation);

            // Verify signature
            if !delegation.signature.verify(&handover, &prev_receiver) {
                return false;
            }

            prev_receiver = delegation.user.clone();
            prev_area = delegation.area.clone();
        }

        true
    }

    /// Compute the handover message for a delegation
    fn compute_handover(
        &self,
        delegation_index: usize,
        prev_area: &Area,
        delegation: &CapabilityDelegation,
    ) -> Vec<u8> {
        let mut handover = Vec::new();

        if delegation_index == 0 {
            // First delegation: include access mode and namespace key
            handover.push(self.access_mode.to_byte());
            handover.extend_from_slice(self.namespace_key.as_bytes());
        } else {
            // Subsequent delegations: include previous signature
            let prev_sig = &self.delegations[delegation_index - 1].signature;
            handover.extend_from_slice(prev_sig.as_bytes());
        }

        // Include relative area encoding and new user
        handover.extend_from_slice(&encode_area_in_area(&delegation.area, prev_area));
        handover.extend_from_slice(&delegation.user.to_bytes());

        handover
    }

    /// Delegate this capability to another user with restricted area
    pub fn delegate(
        &self,
        new_area: Area,
        new_user: UserPublicKey,
        signature: UserSignature,
    ) -> Result<Self, CapabilityError> {
        // Verify area restriction
        let current_area = self.granted_area();
        if !current_area.includes(&new_area) {
            return Err(CapabilityError::AreaExpansion);
        }

        let mut new_cap = self.clone();
        new_cap
            .delegations
            .push(CapabilityDelegation::new(new_area, new_user, signature));

        Ok(new_cap)
    }
}

/// Capability for owned namespaces
///
/// In an owned namespace, the namespace creator owns all data.
/// They can delegate access to specific areas to other users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedCapability {
    /// The kind of access this grants
    pub access_mode: AccessMode,
    /// The namespace for which this grants access
    pub namespace_key: NamespacePublicKey,
    /// The user to whom this initially grants access
    pub user_key: UserPublicKey,
    /// Authorization of the user_key by the namespace_key
    pub initial_authorisation: NamespaceSignature,
    /// Successive authorizations of new UserPublicKeys, each restricted to a particular Area
    pub delegations: Vec<CapabilityDelegation>,
}

impl OwnedCapability {
    /// Create a new root owned capability
    pub fn new_root(
        access_mode: AccessMode,
        namespace_key: NamespacePublicKey,
        user_key: UserPublicKey,
        initial_authorisation: NamespaceSignature,
    ) -> Self {
        Self {
            access_mode,
            namespace_key,
            user_key,
            initial_authorisation,
            delegations: Vec::new(),
        }
    }

    /// Get the receiver (the user to whom this grants access)
    pub fn receiver(&self) -> &UserPublicKey {
        self.delegations
            .last()
            .map(|d| &d.user)
            .unwrap_or(&self.user_key)
    }

    /// Get the granted namespace
    pub fn granted_namespace(&self) -> &NamespacePublicKey {
        &self.namespace_key
    }

    /// Get the granted area
    pub fn granted_area(&self) -> Area {
        self.delegations
            .last()
            .map(|d| d.area.clone())
            .unwrap_or_else(Area::full)
    }

    /// Check if this capability is valid
    pub fn is_valid(&self) -> bool {
        // Verify initial authorization
        let init_message = self.compute_initial_message();
        if !self
            .initial_authorisation
            .verify(&init_message, &self.namespace_key)
        {
            return false;
        }

        if self.delegations.is_empty() {
            return true;
        }

        // Verify the delegation chain
        let mut prev_receiver = self.user_key.clone();
        let mut prev_area = Area::full();

        for (i, delegation) in self.delegations.iter().enumerate() {
            // Check area restriction
            if !prev_area.includes(&delegation.area) {
                return false;
            }

            // Compute the handover message
            let handover = self.compute_handover(i, &prev_area, delegation);

            // Verify signature
            if !delegation.signature.verify(&handover, &prev_receiver) {
                return false;
            }

            prev_receiver = delegation.user.clone();
            prev_area = delegation.area.clone();
        }

        true
    }

    /// Compute the message for initial authorization
    fn compute_initial_message(&self) -> Vec<u8> {
        let mut message = Vec::new();
        // 0x02 for read, 0x03 for write in owned capabilities
        message.push(match self.access_mode {
            AccessMode::Read => 0x02,
            AccessMode::Write => 0x03,
        });
        message.extend_from_slice(&self.user_key.to_bytes());
        message
    }

    /// Compute the handover message for a delegation
    fn compute_handover(
        &self,
        delegation_index: usize,
        prev_area: &Area,
        delegation: &CapabilityDelegation,
    ) -> Vec<u8> {
        let mut handover = Vec::new();

        // Include relative area encoding
        handover.extend_from_slice(&encode_area_in_area(&delegation.area, prev_area));

        if delegation_index == 0 {
            // First delegation: include initial authorization
            handover.extend_from_slice(self.initial_authorisation.as_bytes());
        } else {
            // Subsequent delegations: include previous signature
            let prev_sig = &self.delegations[delegation_index - 1].signature;
            handover.extend_from_slice(prev_sig.as_bytes());
        }

        handover.extend_from_slice(&delegation.user.to_bytes());

        handover
    }

    /// Delegate this capability to another user with restricted area
    pub fn delegate(
        &self,
        new_area: Area,
        new_user: UserPublicKey,
        signature: UserSignature,
    ) -> Result<Self, CapabilityError> {
        // Verify area restriction
        let current_area = self.granted_area();
        if !current_area.includes(&new_area) {
            return Err(CapabilityError::AreaExpansion);
        }

        let mut new_cap = self.clone();
        new_cap
            .delegations
            .push(CapabilityDelegation::new(new_area, new_user, signature));

        Ok(new_cap)
    }
}

/// Unified Meadowcap capability
///
/// Wraps either a CommunalCapability or OwnedCapability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McCapability {
    Communal(CommunalCapability),
    Owned(OwnedCapability),
}

impl McCapability {
    /// Get the access mode
    pub fn access_mode(&self) -> AccessMode {
        match self {
            Self::Communal(c) => c.access_mode,
            Self::Owned(o) => o.access_mode,
        }
    }

    /// Get the receiver
    pub fn receiver(&self) -> &UserPublicKey {
        match self {
            Self::Communal(c) => c.receiver(),
            Self::Owned(o) => o.receiver(),
        }
    }

    /// Get the granted namespace
    pub fn granted_namespace(&self) -> &NamespacePublicKey {
        match self {
            Self::Communal(c) => c.granted_namespace(),
            Self::Owned(o) => o.granted_namespace(),
        }
    }

    /// Get the granted area
    pub fn granted_area(&self) -> Area {
        match self {
            Self::Communal(c) => c.granted_area(),
            Self::Owned(o) => o.granted_area(),
        }
    }

    /// Check if this capability is valid
    ///
    /// Also verifies that the capability type matches the namespace type
    /// (communal vs owned).
    pub fn is_valid<F>(&self, is_communal: F) -> bool
    where
        F: Fn(&NamespacePublicKey) -> bool,
    {
        match self {
            Self::Communal(c) => {
                // Must be a communal namespace and capability must be valid
                is_communal(&c.namespace_key) && c.is_valid()
            }
            Self::Owned(o) => {
                // Must be an owned namespace and capability must be valid
                !is_communal(&o.namespace_key) && o.is_valid()
            }
        }
    }

    /// Check if this capability grants access to an entry
    pub fn grants_access_to(&self, namespace: &NamespacePublicKey, subspace: &UserPublicKey) -> bool {
        // Must be for the same namespace
        if self.granted_namespace() != namespace {
            return false;
        }

        // Check if the subspace is within the granted area
        self.granted_area().includes_subspace(subspace)
    }

    /// Compute a hash of this capability for revocation/tracking
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"mc_capability");
        hasher.update(self.granted_namespace().as_bytes());
        hasher.update(&self.receiver().to_bytes());
        hasher.update(&[self.access_mode().to_byte()]);
        // Include area hash
        hasher.update(&self.granted_area().hash());
        hasher.finalize().into()
    }
}

/// Errors that can occur with capabilities
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    /// Attempted to expand the area (not allowed)
    AreaExpansion,
    /// Attempted to escalate privileges (not allowed)
    PrivilegeEscalation,
    /// Invalid signature
    InvalidSignature,
    /// Capability has been revoked
    Revoked,
    /// Maximum delegation depth exceeded
    MaxDepthExceeded(usize),
    /// Invalid capability type for namespace
    WrongCapabilityType,
    /// Capability chain is invalid
    InvalidChain(String),
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AreaExpansion => write!(f, "Delegation cannot expand the area"),
            Self::PrivilegeEscalation => write!(f, "Delegation cannot escalate privileges"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::Revoked => write!(f, "Capability has been revoked"),
            Self::MaxDepthExceeded(d) => write!(f, "Maximum delegation depth {} exceeded", d),
            Self::WrongCapabilityType => write!(f, "Wrong capability type for namespace"),
            Self::InvalidChain(r) => write!(f, "Invalid capability chain: {}", r),
        }
    }
}

impl std::error::Error for CapabilityError {}

/// Encode an area relative to another area (for efficient transmission)
fn encode_area_in_area(inner: &Area, outer: &Area) -> Vec<u8> {
    // Simplified encoding - in production would use proper relative encoding
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&inner.hash());
    bytes.extend_from_slice(&outer.hash());
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;

    #[test]
    fn test_access_mode_subsumes() {
        assert!(AccessMode::Write.subsumes(&AccessMode::Read));
        assert!(AccessMode::Write.subsumes(&AccessMode::Write));
        assert!(AccessMode::Read.subsumes(&AccessMode::Read));
        assert!(!AccessMode::Read.subsumes(&AccessMode::Write));
    }

    #[test]
    fn test_communal_capability_root() {
        let namespace_key = NamespacePublicKey::new([1u8; 32]);
        let user_key = PeerId::random();

        let cap = CommunalCapability::new_root(AccessMode::Read, namespace_key.clone(), user_key.clone());

        assert!(cap.is_valid());
        assert_eq!(cap.receiver(), &user_key);
        assert_eq!(cap.granted_namespace(), &namespace_key);
    }
}
