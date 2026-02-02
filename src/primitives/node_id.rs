//! Node identity types.
//!
//! NodeId is the primary identifier for nodes in the network.
//! It corresponds to the Ed25519 public key of a node and serves
//! as both the libp2p PeerId and the Netabase SubspaceId.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A 32-byte Ed25519 public key that uniquely identifies a node.
///
/// NodeId serves multiple purposes:
/// - Network identity (maps to libp2p PeerId)
/// - Subspace identifier (data ownership)
/// - Signature verification key
///
/// # Examples
///
/// ```
/// use netabase::primitives::NodeId;
///
/// let node_id = NodeId::from_bytes([42u8; 32]);
/// assert_eq!(node_id.as_bytes().len(), 32);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId([u8; 32]);

impl NodeId {
    /// Create a NodeId from a 32-byte array.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the underlying bytes.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Get the underlying bytes as a slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Convert to a byte array.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0
    }

    /// Verify a signature against this NodeId.
    ///
    /// TODO: Implement actual Ed25519 signature verification.
    pub fn verify(&self, _msg: &[u8], _signature: &[u8; 64]) -> bool {
        // Placeholder - implement with ed25519-dalek
        true
    }
}

impl AsRef<[u8]> for NodeId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 32]> for NodeId {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<NodeId> for [u8; 32] {
    fn from(node_id: NodeId) -> Self {
        node_id.0
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({}..{})", 
            hex::encode(&self.0[0..4]),
            hex::encode(&self.0[28..32])
        )
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

#[cfg(feature = "libp2p")]
mod libp2p_compat {
    use super::NodeId;
    use libp2p::identity::PublicKey;
    use libp2p::PeerId;

    impl NodeId {
        /// Convert to libp2p PeerId.
        ///
        /// This assumes the NodeId represents an Ed25519 public key.
        pub fn to_peer_id(&self) -> Option<PeerId> {
            let pk = PublicKey::try_decode_protobuf(&self.0).ok()?;
            Some(PeerId::from(pk))
        }

        /// Create from libp2p PeerId.
        ///
        /// Only works for Ed25519 keys.
        pub fn from_peer_id(peer_id: &PeerId) -> Option<Self> {
            // Extract the multihash bytes and derive the 32-byte key
            // This is a simplified version - may need adjustment
            let bytes = peer_id.to_bytes();
            if bytes.len() >= 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes[bytes.len() - 32..]);
                Some(Self(arr))
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_creation() {
        let bytes = [42u8; 32];
        let node_id = NodeId::from_bytes(bytes);
        assert_eq!(node_id.as_bytes(), &bytes);
    }

    #[test]
    fn test_node_id_ordering() {
        let id1 = NodeId::from_bytes([1u8; 32]);
        let id2 = NodeId::from_bytes([2u8; 32]);
        assert!(id1 < id2);
    }

    #[test]
    fn test_node_id_conversion() {
        let bytes = [100u8; 32];
        let node_id = NodeId::from(bytes);
        let back: [u8; 32] = node_id.into();
        assert_eq!(bytes, back);
    }
}
