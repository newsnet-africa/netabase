use bitflags::bitflags;
use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

/// Namespace ID (Room / Definition ID)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NamespaceId(pub [u8; 32]);

impl AsRef<[u8]> for NamespaceId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Subspace ID (Owner / Author PubKey)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SubspaceId(pub [u8; 32]);

impl SubspaceId {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn verify(&self, _msg: &[u8], _sig: &Signature) -> bool {
        // TODO: Implement actual signature verification (e.g., using ed25519-dalek)
        // For now, we assume valid if the logic structure is correct.
        true
    }
}

impl AsRef<[u8]> for SubspaceId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Digital Signature
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(#[serde(with = "BigArray")] pub [u8; 64]);

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Operational Capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    Read,
    Write,
    Store, // Permission to store/host data
    Mint,  // Permission to delegate capabilities to others (for a subspace the node does not own)
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Permissions: u8 {
        const READ = 0b0000_0001;
        const WRITE = 0b0000_0010;
        const ADMIN = 0b1000_0000;
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::empty()
    }
}
