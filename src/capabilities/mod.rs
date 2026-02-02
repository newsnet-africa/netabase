//! Capability-based access control for Netabase networking.
//!
//! This module provides type-safe capability structures for the Netabase protocol.
//! Capabilities define what operations a peer can perform on specific data ranges,
//! with support for N-dimensional queries across primary and secondary keys.
//!
//! # Key Changes from Legacy
//!
//! - Uses `NodeId` instead of raw byte arrays
//! - Supports `NDimensionalRange` for multi-index queries
//! - Integrates `ConflictRank` for CRDT resolution
//! - No longer feature-gated (networking-only crate)

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::fmt;

use crate::primitives::{
    ConflictRank, KeyRange, NDimensionalRange, NodeId, NodeIdRange, PathRange,
    SecondaryKeyRange,
};

/// Capability expiration timestamp (Unix epoch seconds).
pub type CapabilityExpiration = u64;

/// Operations that can be performed with a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Operation {
    /// Read access to data
    Read,
    
    /// Write access (create, update, delete)
    Write,
    
    /// Permission to host/replicate data
    Store,
    
    /// Permission to delegate capabilities to others
    Mint,
}

impl Operation {
    /// Check if this operation includes another operation.
    ///
    /// Operation hierarchy: Mint ⊃ Write ⊃ Store ⊃ Read
    pub fn includes(&self, other: &Operation) -> bool {
        match (self, other) {
            (Operation::Mint, _) => true,
            (Operation::Write, Operation::Write | Operation::Store | Operation::Read) => true,
            (Operation::Store, Operation::Store | Operation::Read) => true,
            (Operation::Read, Operation::Read) => true,
            _ => false,
        }
    }
}

/// A cryptographic signature over capability data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySignature(
    #[serde(with = "serde_big_array::BigArray")] 
    pub [u8; 64]
);

impl AsRef<[u8]> for CapabilitySignature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// A capability granting access to a specific range of data.
///
/// Capabilities are cryptographically signed and can be delegated.
/// They specify:
/// - Who granted the capability
/// - Who it was granted to
/// - What operations are allowed
/// - What data range is covered
/// - When it expires
/// - Optional delegation chain
///
/// # Type Parameters
///
/// - `PK`: Primary key type
/// - `SK`: Secondary key discriminant type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability<PK, SK> {
    /// Node that granted this capability
    pub granted_by: NodeId,
    
    /// Node that received this capability
    pub granted_to: NodeId,
    
    /// The operation permitted
    pub operation: Operation,
    
    /// The N-dimensional range this capability covers
    pub range: NDimensionalRange<PK, SK>,
    
    /// Unix timestamp when this capability expires
    pub expiry: CapabilityExpiration,
    
    /// Signature by the grantor
    pub signature: CapabilitySignature,
    
    /// Optional parent capability (for delegation chains)
    pub parent: Option<Box<Capability<PK, SK>>>,
}

impl<PK, SK> Capability<PK, SK>
where
    PK: Clone,
    SK: Clone + PartialEq,
{
    /// Create a new root capability (no parent).
    pub fn new_root(
        owner: NodeId,
        granted_to: NodeId,
        operation: Operation,
        range: NDimensionalRange<PK, SK>,
        expiry: CapabilityExpiration,
    ) -> Self {
        Self {
            granted_by: owner,
            granted_to,
            operation,
            range,
            expiry,
            signature: CapabilitySignature([0u8; 64]), // To be signed
            parent: None,
        }
    }

    /// Create a delegated capability from a parent.
    pub fn delegate(
        parent: Capability<PK, SK>,
        granted_to: NodeId,
        operation: Operation,
        range: NDimensionalRange<PK, SK>,
        expiry: CapabilityExpiration,
    ) -> Result<Self, CapabilityError> {
        // Verify parent allows this delegation
        if !parent.operation.includes(&operation) {
            return Err(CapabilityError::OperationNotSubset);
        }

        if !range.is_subset_of(&parent.range) {
            return Err(CapabilityError::RangeNotSubset);
        }

        if expiry > parent.expiry {
            return Err(CapabilityError::ExpiryTooLate);
        }

        Ok(Self {
            granted_by: parent.granted_to,
            granted_to,
            operation,
            range,
            expiry,
            signature: CapabilitySignature([0u8; 64]), // To be signed
            parent: Some(Box::new(parent)),
        })
    }

    /// Check if this capability has expired.
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.expiry < now
    }

    /// Verify the entire capability chain.
    ///
    /// Checks:
    /// 1. Not expired at any level
    /// 2. Chain integrity (each step is valid)
    /// 3. Permission narrowing (child ⊆ parent)
    /// 4. Signatures (TODO: actual crypto verification)
    pub fn verify_chain(&self, root_owner: &NodeId) -> Result<(), CapabilityError> {
        // Check expiration
        if self.is_expired() {
            return Err(CapabilityError::Expired);
        }

        // If there's a parent, verify it recursively
        if let Some(parent) = &self.parent {
            // Verify parent chain
            parent.verify_chain(root_owner)?;

            // Verify link: parent was issued to our grantor
            if parent.granted_to != self.granted_by {
                return Err(CapabilityError::ChainBroken);
            }

            // Verify permission narrowing
            if !parent.operation.includes(&self.operation) {
                return Err(CapabilityError::OperationNotSubset);
            }

            // Verify range narrowing
            if !self.range.is_subset_of(&parent.range) {
                return Err(CapabilityError::RangeNotSubset);
            }

            // Verify expiry
            if self.expiry > parent.expiry {
                return Err(CapabilityError::ExpiryTooLate);
            }

            // TODO: Verify signature with granted_by's public key
            // self.granted_by.verify(message, &self.signature)?;
        } else {
            // Root capability: verify it was granted by the owner
            if &self.granted_by != root_owner {
                return Err(CapabilityError::InvalidRoot);
            }

            // TODO: Verify signature with root_owner's key
        }

        Ok(())
    }

    /// Get the effective depth of the delegation chain.
    pub fn chain_depth(&self) -> usize {
        match &self.parent {
            Some(parent) => 1 + parent.chain_depth(),
            None => 0,
        }
    }

    /// Check if this capability authorizes a specific operation on a range.
    pub fn authorizes(
        &self,
        operation: &Operation,
        range: &NDimensionalRange<PK, SK>,
    ) -> bool {
        // Check operation
        if !self.operation.includes(operation) {
            return false;
        }

        // Check range
        if !range.is_subset_of(&self.range) {
            return false;
        }

        // Check expiration
        if self.is_expired() {
            return false;
        }

        true
    }
}

/// Errors that can occur during capability operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    /// Capability has expired
    Expired,
    
    /// Operation is not a subset of parent's operation
    OperationNotSubset,
    
    /// Range is not a subset of parent's range
    RangeNotSubset,
    
    /// Expiry is later than parent's expiry
    ExpiryTooLate,
    
    /// Chain is broken (parent wasn't issued to grantor)
    ChainBroken,
    
    /// Root capability not granted by owner
    InvalidRoot,
    
    /// Signature verification failed
    InvalidSignature,
    
    /// Malformed capability data
    Malformed,
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expired => write!(f, "Capability has expired"),
            Self::OperationNotSubset => write!(f, "Operation is not a subset of parent"),
            Self::RangeNotSubset => write!(f, "Range is not a subset of parent"),
            Self::ExpiryTooLate => write!(f, "Expiry is later than parent's expiry"),
            Self::ChainBroken => write!(f, "Delegation chain is broken"),
            Self::InvalidRoot => write!(f, "Root capability not granted by owner"),
            Self::InvalidSignature => write!(f, "Signature verification failed"),
            Self::Malformed => write!(f, "Malformed capability data"),
        }
    }
}

impl std::error::Error for CapabilityError {}

/// Authorization token proving write access to an entry.
///
/// This is attached to write operations to prove the writer has permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationToken<PK, SK> {
    /// The capability granting write access
    pub capability: Capability<PK, SK>,
    
    /// Signature by the capability holder over the entry
    pub entry_signature: CapabilitySignature,
}

impl<PK, SK> AuthorizationToken<PK, SK>
where
    PK: Clone,
    SK: Clone + PartialEq,
{
    /// Verify this token authorizes a write operation.
    pub fn verify_write(
        &self,
        root_owner: &NodeId,
        range: &NDimensionalRange<PK, SK>,
    ) -> Result<(), CapabilityError> {
        // Verify capability chain
        self.capability.verify_chain(root_owner)?;

        // Verify write operation is allowed
        if !self.capability.operation.includes(&Operation::Write) {
            return Err(CapabilityError::OperationNotSubset);
        }

        // Verify range is covered
        if !range.is_subset_of(&self.capability.range) {
            return Err(CapabilityError::RangeNotSubset);
        }

        // TODO: Verify entry_signature matches entry content
        // self.capability.granted_to.verify(entry_bytes, &self.entry_signature)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{Path, PathBuilder};

    fn test_node_id(byte: u8) -> NodeId {
        NodeId::from_bytes([byte; 32])
    }

    #[test]
    fn test_operation_hierarchy() {
        assert!(Operation::Mint.includes(&Operation::Write));
        assert!(Operation::Mint.includes(&Operation::Store));
        assert!(Operation::Mint.includes(&Operation::Read));
        assert!(Operation::Write.includes(&Operation::Store));
        assert!(Operation::Write.includes(&Operation::Read));
        assert!(Operation::Store.includes(&Operation::Read));
        assert!(!Operation::Read.includes(&Operation::Write));
    }

    #[test]
    fn test_capability_creation() {
        let owner = test_node_id(1);
        let user = test_node_id(2);
        let range: NDimensionalRange<String, u16> = NDimensionalRange::all();

        let cap = Capability::new_root(
            owner,
            user,
            Operation::Read,
            range,
            u64::MAX,
        );

        assert_eq!(cap.granted_by, owner);
        assert_eq!(cap.granted_to, user);
        assert_eq!(cap.operation, Operation::Read);
        assert!(!cap.is_expired());
    }

    #[test]
    fn test_capability_delegation() {
        let owner = test_node_id(1);
        let user1 = test_node_id(2);
        let user2 = test_node_id(3);

        let range: NDimensionalRange<String, u16> = NDimensionalRange::new(
            NodeIdRange::All,
            KeyRange::all(),
            vec![],
        );

        // Root capability: owner → user1 (Write)
        let root = Capability::new_root(
            owner,
            user1,
            Operation::Write,
            range.clone(),
            u64::MAX,
        );

        // Delegate: user1 → user2 (Read, narrower)
        let delegated = Capability::delegate(
            root,
            user2,
            Operation::Read,
            range.clone(),
            u64::MAX,
        )
        .unwrap();

        assert_eq!(delegated.granted_by, user1);
        assert_eq!(delegated.granted_to, user2);
        assert_eq!(delegated.operation, Operation::Read);
        assert_eq!(delegated.chain_depth(), 1);
    }

    #[test]
    fn test_delegation_validation() {
        let owner = test_node_id(1);
        let user1 = test_node_id(2);
        let user2 = test_node_id(3);

        let range: NDimensionalRange<String, u16> = NDimensionalRange::all();

        let root = Capability::new_root(
            owner,
            user1,
            Operation::Read,
            range.clone(),
            u64::MAX,
        );

        // Try to delegate Write from Read - should fail
        let result = Capability::delegate(
            root,
            user2,
            Operation::Write,
            range,
            u64::MAX,
        );

        assert!(matches!(result, Err(CapabilityError::OperationNotSubset)));
    }

    #[test]
    fn test_capability_authorization() {
        let owner = test_node_id(1);
        let user = test_node_id(2);

        let range: NDimensionalRange<String, u16> = NDimensionalRange::new(
            NodeIdRange::All,
            KeyRange::prefix(PathBuilder::new().key("users").build()),
            vec![],
        );

        let cap = Capability::new_root(
            owner,
            user,
            Operation::Write,
            range,
            u64::MAX,
        );

        // Should authorize write to subset range
        let query_range: NDimensionalRange<String, u16> = NDimensionalRange::new(
            NodeIdRange::All,
            KeyRange::prefix(
                PathBuilder::new().key("users").key("alice").build(),
            ),
            vec![],
        );

        assert!(cap.authorizes(&Operation::Write, &query_range));
        assert!(cap.authorizes(&Operation::Read, &query_range));
    }

    #[test]
    fn test_expired_capability() {
        let owner = test_node_id(1);
        let user = test_node_id(2);
        let range: NDimensionalRange<String, u16> = NDimensionalRange::all();

        let cap = Capability::new_root(
            owner,
            user,
            Operation::Read,
            range.clone(),
            0, // Already expired
        );

        assert!(cap.is_expired());
        assert!(!cap.authorizes(&Operation::Read, &range));
    }
}
