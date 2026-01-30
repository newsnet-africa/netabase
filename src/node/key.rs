use serde::{Serialize, Deserialize};

/// Logical Representation of a Semantic Key
/// 
/// As per Netabase Networking Specification (v3.0):
/// [Subspace (32B)] ++ [Path (Variable)] ++ [Rank (8B)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NetabaseKey {
    pub subspace: [u8; 32],  // Author PubKey
    pub path: Vec<u8>,       // Tuple-Encoded Semantic Path
    pub rank: u64,           // Priority/Lamport Clock (Big Endian)
}

impl NetabaseKey {
    pub fn new(subspace: [u8; 32], path: Vec<u8>, rank: u64) -> Self {
        Self { subspace, path, rank }
    }

    /// Encode to bytes preserving order (Tuple Encoding)
    /// Layout: [Subspace (32B)] ++ [Path (Variable)] ++ [Rank (8B Big Endian)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32 + self.path.len() + 8);
        bytes.extend_from_slice(&self.subspace);
        bytes.extend_from_slice(&self.path);
        bytes.extend_from_slice(&self.rank.to_be_bytes());
        bytes
    }

    /// Decode from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 40 { // 32 + 0 + 8
            return None;
        }
        let subspace: [u8; 32] = bytes[0..32].try_into().ok()?;
        let rank_start = bytes.len() - 8;
        let path = bytes[32..rank_start].to_vec();
        let rank = u64::from_be_bytes(bytes[rank_start..].try_into().ok()?);
        
        Some(Self { subspace, path, rank })
    }
}

/// Trait for types that can be converted to a semantic path
pub trait NetabasePath {
    fn to_path(&self) -> Vec<u8>;
}
