//! Type-safe path primitives.
//!
//! Paths in Netabase are structured as vectors of PathNodes, where each node
//! can represent different semantic concepts (keys, versions, prefixes, etc.).
//! This allows for type-safe path construction and validation at compile time.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A semantic path segment in the Netabase key structure.
///
/// PathNodes provide type information about what each segment represents,
/// enabling compile-time verification and runtime introspection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PathNode {
    /// A string key segment
    Key(String),
    
    /// A numeric index segment
    Index(u64),
    
    /// A UUID segment
    Uuid([u8; 16]),
    
    /// A timestamp segment (milliseconds since Unix epoch)
    Timestamp(u64),
    
    /// A version identifier at a fork point
    /// 
    /// When a path represents a forking point (e.g., multiple versions of the same entry),
    /// the version must be specified. This educates peers about available versions.
    Version(u64),
    
    /// A content-addressed segment (hash)
    ContentAddress([u8; 32]),
    
    /// An arbitrary byte segment (for compatibility)
    Bytes(Vec<u8>),
    
    /// A typed discriminant for model-specific paths
    Discriminant(u16),
}

impl PathNode {
    /// Check if this node represents a forking point.
    ///
    /// Forking points require subsequent path segments to disambiguate.
    pub fn is_fork(&self) -> bool {
        matches!(self, PathNode::Version(_))
    }

    /// Encode this path node to bytes for storage/transmission.
    ///
    /// Uses tuple encoding to preserve ordering.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            PathNode::Key(s) => {
                bytes.push(0x01); // Tag
                bytes.extend_from_slice(s.as_bytes());
                bytes.push(0x00); // Null terminator for ordering
            }
            PathNode::Index(n) => {
                bytes.push(0x02);
                bytes.extend_from_slice(&n.to_be_bytes());
            }
            PathNode::Uuid(u) => {
                bytes.push(0x03);
                bytes.extend_from_slice(u);
            }
            PathNode::Timestamp(t) => {
                bytes.push(0x04);
                bytes.extend_from_slice(&t.to_be_bytes());
            }
            PathNode::Version(v) => {
                bytes.push(0x05);
                bytes.extend_from_slice(&v.to_be_bytes());
            }
            PathNode::ContentAddress(h) => {
                bytes.push(0x06);
                bytes.extend_from_slice(h);
            }
            PathNode::Bytes(b) => {
                bytes.push(0x07);
                bytes.extend_from_slice(b);
                bytes.push(0x00);
            }
            PathNode::Discriminant(d) => {
                bytes.push(0x08);
                bytes.extend_from_slice(&d.to_be_bytes());
            }
        }
        bytes
    }

    /// Decode a path node from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.is_empty() {
            return None;
        }

        let tag = bytes[0];
        let rest = &bytes[1..];

        match tag {
            0x01 => {
                // Key - read until null terminator
                let end = rest.iter().position(|&b| b == 0x00)?;
                let s = String::from_utf8(rest[..end].to_vec()).ok()?;
                Some((PathNode::Key(s), end + 2)) // +1 for tag, +1 for terminator
            }
            0x02 => {
                // Index
                if rest.len() < 8 { return None; }
                let n = u64::from_be_bytes(rest[..8].try_into().ok()?);
                Some((PathNode::Index(n), 9))
            }
            0x03 => {
                // UUID
                if rest.len() < 16 { return None; }
                let u: [u8; 16] = rest[..16].try_into().ok()?;
                Some((PathNode::Uuid(u), 17))
            }
            0x04 => {
                // Timestamp
                if rest.len() < 8 { return None; }
                let t = u64::from_be_bytes(rest[..8].try_into().ok()?);
                Some((PathNode::Timestamp(t), 9))
            }
            0x05 => {
                // Version
                if rest.len() < 8 { return None; }
                let v = u64::from_be_bytes(rest[..8].try_into().ok()?);
                Some((PathNode::Version(v), 9))
            }
            0x06 => {
                // ContentAddress
                if rest.len() < 32 { return None; }
                let h: [u8; 32] = rest[..32].try_into().ok()?;
                Some((PathNode::ContentAddress(h), 33))
            }
            0x07 => {
                // Bytes
                let end = rest.iter().position(|&b| b == 0x00)?;
                let b = rest[..end].to_vec();
                Some((PathNode::Bytes(b), end + 2))
            }
            0x08 => {
                // Discriminant
                if rest.len() < 2 { return None; }
                let d = u16::from_be_bytes(rest[..2].try_into().ok()?);
                Some((PathNode::Discriminant(d), 3))
            }
            _ => None,
        }
    }
}

impl fmt::Display for PathNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathNode::Key(s) => write!(f, "{}", s),
            PathNode::Index(n) => write!(f, "[{}]", n),
            PathNode::Uuid(u) => write!(f, "uuid({})", hex::encode(u)),
            PathNode::Timestamp(t) => write!(f, "ts({})", t),
            PathNode::Version(v) => write!(f, "v{}", v),
            PathNode::ContentAddress(h) => write!(f, "hash({})", hex::encode(&h[..4])),
            PathNode::Bytes(b) => write!(f, "bytes({})", hex::encode(&b[..b.len().min(4)])),
            PathNode::Discriminant(d) => write!(f, "disc({})", d),
        }
    }
}

/// A structured path consisting of multiple path segments.
///
/// Paths maintain semantic information about each segment, enabling
/// type-safe operations and validation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Path {
    nodes: Vec<PathNode>,
}

impl Path {
    /// Create a new empty path.
    pub const fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Create a path from a vector of nodes.
    pub fn from_nodes(nodes: Vec<PathNode>) -> Self {
        Self { nodes }
    }

    /// Get the path nodes.
    pub fn nodes(&self) -> &[PathNode] {
        &self.nodes
    }

    /// Append a node to the path.
    pub fn push(&mut self, node: PathNode) {
        self.nodes.push(node);
    }

    /// Validate that the path is well-formed.
    ///
    /// Rules:
    /// - If a node is a fork point, it must not be the last node
    /// - Version nodes must have subsequent nodes
    pub fn validate(&self) -> Result<(), PathValidationError> {
        for (i, node) in self.nodes.iter().enumerate() {
            if node.is_fork() && i == self.nodes.len() - 1 {
                return Err(PathValidationError::ForkWithoutVersion(i));
            }
        }
        Ok(())
    }

    /// Encode the entire path to bytes.
    ///
    /// Uses tuple encoding to preserve ordering.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for node in &self.nodes {
            bytes.extend_from_slice(&node.to_bytes());
        }
        bytes
    }

    /// Decode a path from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut nodes = Vec::new();
        let mut offset = 0;

        while offset < bytes.len() {
            let (node, consumed) = PathNode::from_bytes(&bytes[offset..])?;
            nodes.push(node);
            offset += consumed;
        }

        Some(Self { nodes })
    }

    /// Get the length of the path in nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Create a path segment for prefix queries.
    pub fn prefix(&self, len: usize) -> Self {
        Self {
            nodes: self.nodes[..len.min(self.nodes.len())].to_vec(),
        }
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.nodes.is_empty() {
            return write!(f, "/");
        }
        for (i, node) in self.nodes.iter().enumerate() {
            if i > 0 {
                write!(f, "/")?;
            }
            write!(f, "{}", node)?;
        }
        Ok(())
    }
}

/// Path validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathValidationError {
    /// A fork point node appears without subsequent version specification
    ForkWithoutVersion(usize),
}

impl fmt::Display for PathValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathValidationError::ForkWithoutVersion(i) => {
                write!(f, "Fork point at index {} requires subsequent version node", i)
            }
        }
    }
}

impl std::error::Error for PathValidationError {}

/// Convenience type for path segments in user code.
pub type PathSegment = PathNode;

/// Builder for constructing paths ergonomically.
#[derive(Debug, Default)]
pub struct PathBuilder {
    nodes: Vec<PathNode>,
}

impl PathBuilder {
    /// Create a new path builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a key segment.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.nodes.push(PathNode::Key(key.into()));
        self
    }

    /// Add an index segment.
    pub fn index(mut self, idx: u64) -> Self {
        self.nodes.push(PathNode::Index(idx));
        self
    }

    /// Add a version segment.
    pub fn version(mut self, ver: u64) -> Self {
        self.nodes.push(PathNode::Version(ver));
        self
    }

    /// Add a timestamp segment.
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.nodes.push(PathNode::Timestamp(ts));
        self
    }

    /// Build the path.
    pub fn build(self) -> Path {
        Path::from_nodes(self.nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_builder() {
        let path = PathBuilder::new()
            .key("users")
            .key("alice")
            .version(1)
            .build();

        assert_eq!(path.len(), 3);
        assert_eq!(path.to_string(), "users/alice/v1");
    }

    #[test]
    fn test_path_validation_fork() {
        let path = Path::from_nodes(vec![
            PathNode::Key("users".into()),
            PathNode::Version(1),
        ]);

        assert!(path.validate().is_err());
    }

    #[test]
    fn test_path_encoding_roundtrip() {
        let path = PathBuilder::new()
            .key("test")
            .index(42)
            .build();

        let bytes = path.to_bytes();
        let decoded = Path::from_bytes(&bytes).unwrap();

        assert_eq!(path, decoded);
    }

    #[test]
    fn test_path_prefix() {
        let path = PathBuilder::new()
            .key("a")
            .key("b")
            .key("c")
            .build();

        let prefix = path.prefix(2);
        assert_eq!(prefix.len(), 2);
        assert_eq!(prefix.to_string(), "a/b");
    }
}
