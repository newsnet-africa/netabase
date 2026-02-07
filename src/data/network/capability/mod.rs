//! # Meadowcap Capability System
//!
//! This module implements the Meadowcap capability system from the Willow protocol,
//! adapted for netabase's table-level access control. Meadowcap provides:
//!
//! - **Unforgeable access tokens**: Capabilities that cannot be created without proper authorization
//! - **Delegable access**: Capability holders can delegate (restricted) access to others
//! - **Confidential discovery**: Private Area Intersection (PAI) reveals only mutual interests
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        Meadowcap Capability System                       │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │   ┌─────────────┐     ┌─────────────┐     ┌─────────────────────────┐  │
//! │   │ McCapability│────►│    Area     │────►│ SubspaceConstraint     │  │
//! │   │  (unified)  │     │ (granted    │     │ PathConstraint         │  │
//! │   └──────┬──────┘     │  region)    │     │ TimeRange              │  │
//! │          │            └─────────────┘     └─────────────────────────┘  │
//! │          │                                                              │
//! │          ├──────────────────────────────────┐                          │
//! │          ▼                                  ▼                          │
//! │   ┌─────────────────┐              ┌─────────────────┐                 │
//! │   │CommunalCapability│              │ OwnedCapability │                 │
//! │   │ (user-owned     │              │ (namespace-owned │                │
//! │   │  subspaces)     │              │  with delegation)│                │
//! │   └─────────────────┘              └─────────────────┘                 │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Core Types
//!
//! ### Capability Types
//!
//! - [`McCapability`]: Unified capability type wrapping either Communal or Owned
//! - [`CommunalCapability`]: For communal namespaces where each subspace is owned by its author
//! - [`OwnedCapability`]: For owned namespaces where the namespace creator owns all data
//! - [`McEnumerationCapability`]: For resolving awkward PAI cases
//!
//! ### Area Types
//!
//! - [`Area`]: Defines a region of entries (subspace + path prefix + time range)
//! - [`SubspaceConstraint`]: Either `Any` (all subspaces) or `Specific(PeerId)`
//! - [`PathConstraint`]: A path prefix that entries must match
//! - [`TimeRange`]: Optional time bounds for entries
//!
//! ### Private Area Intersection (PAI)
//!
//! - [`PrivateInterest`]: Confidential interest data (namespace, subspace, path)
//! - [`PaiState`]: State machine for the PAI protocol
//! - [`PaiFragment`]: Salted hash fragments exchanged during PAI
//! - [`OverlapAnnouncement`]: Message announcing detected overlap with authentication
//!
//! ### Encoding Types
//!
//! - [`EncodedCapability`]: Capability with sensitive data omitted for transmission
//! - [`MeadowcapAuthorisationToken`]: For authorizing entry writes
//!
//! ## Security Model
//!
//! ### Capability Properties
//!
//! 1. **Unforgeability**: Capabilities are signed by the namespace owner or a delegating user.
//!    Without the corresponding secret key, a valid capability cannot be created.
//!
//! 2. **Restriction Only**: Delegation can only restrict, never expand, the granted area.
//!    A capability for path `a/b` cannot be used to create one for path `a`.
//!
//! 3. **Verifiable Chain**: Each delegation includes a signature that can be verified
//!    back to the original root capability.
//!
//! ### Confidentiality Properties
//!
//! 1. **Interest Privacy**: PAI reveals only mutual interests through salted hashes.
//!    A peer cannot learn about interests they don't share.
//!
//! 2. **Announcement Authentication**: Overlap announcements include proof (salted hash)
//!    that the announcer knows the underlying interest.
//!
//! 3. **Encoded Transmission**: Capabilities are transmitted in encoded form that omits
//!    data already established from PAI context.
//!
//! ## Usage Examples
//!
//! ### Creating a Root Capability (Namespace Owner)
//!
//! ```rust
//! use netabase::data::network::capability::{
//!     AccessMode, CommunalCapability, OwnedCapability, McCapability
//! };
//! use netabase::data::util::encryption::{NamespacePublicKey, NamespaceSignature};
//! use libp2p::PeerId;
//!
//! // Setup: create keys for the example
//! let namespace_key = NamespacePublicKey::new([1u8; 32]);
//! let my_peer_id = PeerId::random();
//! let grantee_peer_id = PeerId::random();
//! let namespace_signature = NamespaceSignature::new(vec![0u8; 64]);
//!
//! // For a communal namespace (each user owns their subspace)
//! let communal_cap = CommunalCapability::new_root(
//!     AccessMode::Write,
//!     namespace_key.clone(),
//!     my_peer_id, // I own this subspace
//! );
//! assert!(communal_cap.is_valid());
//!
//! // For an owned namespace (namespace owner controls everything)
//! let owned_cap = OwnedCapability::new_root(
//!     AccessMode::Write,
//!     namespace_key,
//!     grantee_peer_id,
//!     namespace_signature, // Signed by namespace secret key
//! );
//! // Note: is_valid() checks the signature; with mock signature it returns true
//! assert!(owned_cap.is_valid());
//! ```
//!
//! ### Delegating a Capability
//!
//! ```rust
//! use netabase::data::network::capability::{
//!     AccessMode, Area, CommunalCapability, PathConstraint
//! };
//! use netabase::data::network::capability::meadowcap::UserSignature;
//! use netabase::data::util::encryption::NamespacePublicKey;
//! use libp2p::PeerId;
//!
//! let namespace_key = NamespacePublicKey::new([2u8; 32]);
//! let my_peer_id = PeerId::random();
//! let friend_peer_id = PeerId::random();
//!
//! // Create root capability
//! let my_capability = CommunalCapability::new_root(
//!     AccessMode::Write,
//!     namespace_key,
//!     my_peer_id.clone(),
//! );
//!
//! // Restrict to a specific path prefix
//! let restricted_area = Area::subspace(my_peer_id)
//!     .with_path(PathConstraint::new(vec![b"public".to_vec()]));
//!
//! // Create a mock signature (in production, sign with secret key)
//! let my_signature = UserSignature::new(vec![0u8; 64]);
//!
//! // Delegate the capability
//! let delegated = my_capability.delegate(
//!     restricted_area,
//!     friend_peer_id.clone(),
//!     my_signature,
//! ).expect("Delegation should succeed");
//!
//! assert_eq!(delegated.receiver(), &friend_peer_id);
//! ```
//!
//! ### Private Area Intersection
//!
//! ```rust
//! use netabase::data::network::capability::{PaiState, PrivateInterest};
//! use netabase::data::util::encryption::NamespacePublicKey;
//! use libp2p::PeerId;
//!
//! let namespace1 = NamespacePublicKey::new([3u8; 32]);
//! let namespace2 = NamespacePublicKey::new([4u8; 32]);
//! let my_peer_id = PeerId::random();
//!
//! // Create interests I want to sync
//! let interests = vec![
//!     PrivateInterest::namespace(namespace1),
//!     PrivateInterest::subspace(namespace2, my_peer_id),
//! ];
//!
//! // Random bytes from handshake (simulated)
//! let rnd = [0u8; 32];
//! let is_initiator = true;
//!
//! // Initialize PAI state (rnd comes from handshake)
//! let mut state = PaiState::new(interests, rnd, is_initiator);
//!
//! // Generate fragments to send to peer
//! let fragments = state.generate_fragments();
//! assert!(!fragments.is_empty());
//!
//! // In a real exchange, we'd receive fragments from peer and process them
//! // state.process_received(peer_fragments);
//!
//! // After processing, check for overlaps
//! // for overlap in state.overlaps() {
//! //     // Exchange capabilities for overlapping interests
//! // }
//! ```
//!
//! ## Netabase Extensions
//!
//! This implementation extends Meadowcap for netabase's use case:
//!
//! - **Table-level granularity**: Subscriptions map to tables, allowing fine-grained access
//! - **N-dimensional area constraints**: Areas can constrain by secondary keys
//! - **Typed capabilities**: Generic `Capability<D, M>` for compile-time model safety
//!
//! ## Protocol Flow
//!
//! ```text
//! 1. Handshake (Noise XX)
//!    ├── Exchange ephemeral keys
//!    ├── Exchange encrypted static keys  
//!    └── Derive session keys and PAI salt (rnd)
//!
//! 2. Private Area Intersection
//!    ├── Generate salted hash fragments for interests
//!    ├── Exchange fragments (concurrent)
//!    ├── Detect overlaps locally
//!    └── Send overlap announcements with authentication
//!
//! 3. Capability Exchange
//!    ├── For each overlap, send encoded capability
//!    ├── Verify received capabilities
//!    ├── Acknowledge accepted/rejected
//!    └── Establish sync areas
//!
//! 4. Data Synchronization
//!    └── Sync entries within granted areas
//! ```
//!
//! ## References
//!
//! - [Willow Protocol - Meadowcap](https://willowprotocol.org/specs/meadowcap/index.html)
//! - [Willow Protocol - Private Interest Overlap](https://willowprotocol.org/specs/pio/index.html)

// Core Meadowcap types
pub mod meadowcap;
pub mod area;
pub mod enumeration;
pub mod private_interest;
pub mod encoding;

// Re-export main types
pub use meadowcap::{
    AccessMode, CapabilityDelegation, CapabilityError, CommunalCapability, McCapability,
    OwnedCapability, UserPublicKey, UserSignature,
};

pub use area::{Area, PathConstraint, SubspaceConstraint, SubspaceId, TimeRange};

pub use enumeration::{EncodedEnumerationCapability, McEnumerationCapability};

pub use private_interest::{
    DetectedOverlap, OverlapAnnouncement, OverlapType, PaiFragment, PaiState, PrivateInterest,
};

pub use encoding::{
    EncodedCapability, EncodedDelegation, MeadowcapAuthorisationToken, RelativeArea,
};

