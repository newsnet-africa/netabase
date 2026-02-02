//! Range primitives for N-dimensional queries.
//!
//! This module provides type-safe range types for querying across
//! multiple dimensions: primary keys, secondary keys, and subspaces.

use super::{NodeId, Path};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// A range of paths for prefix queries.
///
/// PathRange supports both exact matches and prefix queries,
/// enabling efficient range scans in the key-value store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathRange {
    /// Exact path match
    Exact(Path),
    
    /// Prefix match (all paths starting with this prefix)
    Prefix(Path),
    
    /// Range between two paths (inclusive start, exclusive end)
    Range { start: Path, end: Path },
    
    /// All paths
    All,
}

impl PathRange {
    /// Check if a path is included in this range.
    pub fn contains(&self, path: &Path) -> bool {
        match self {
            PathRange::Exact(p) => path == p,
            PathRange::Prefix(prefix) => {
                path.nodes().starts_with(prefix.nodes())
            }
            PathRange::Range { start, end } => {
                path >= start && path < end
            }
            PathRange::All => true,
        }
    }

    /// Check if this range is a subset of another range.
    pub fn is_subset_of(&self, other: &PathRange) -> bool {
        match (self, other) {
            (_, PathRange::All) => true,
            (PathRange::All, _) => false,
            (PathRange::Exact(p), PathRange::Exact(o)) => p == o,
            (PathRange::Exact(p), PathRange::Prefix(prefix)) => {
                p.nodes().starts_with(prefix.nodes())
            }
            (PathRange::Exact(p), PathRange::Range { start, end }) => {
                p >= start && p < end
            }
            (PathRange::Prefix(p), PathRange::Prefix(o)) => {
                p.nodes().starts_with(o.nodes())
            }
            (PathRange::Range { start: s1, end: e1 }, PathRange::Range { start: s2, end: e2 }) => {
                s1 >= s2 && e1 <= e2
            }
            _ => false,
        }
    }

    /// Compute the intersection of two path ranges.
    pub fn intersect(&self, other: &PathRange) -> Option<PathRange> {
        match (self, other) {
            (PathRange::All, other) | (other, PathRange::All) => Some(other.clone()),
            (PathRange::Exact(p1), PathRange::Exact(p2)) => {
                if p1 == p2 {
                    Some(PathRange::Exact(p1.clone()))
                } else {
                    None
                }
            }
            (PathRange::Exact(p), PathRange::Prefix(prefix))
            | (PathRange::Prefix(prefix), PathRange::Exact(p)) => {
                if p.nodes().starts_with(prefix.nodes()) {
                    Some(PathRange::Exact(p.clone()))
                } else {
                    None
                }
            }
            (PathRange::Prefix(p1), PathRange::Prefix(p2)) => {
                // Intersection is the longer prefix (more specific)
                if p1.nodes().starts_with(p2.nodes()) {
                    Some(PathRange::Prefix(p1.clone()))
                } else if p2.nodes().starts_with(p1.nodes()) {
                    Some(PathRange::Prefix(p2.clone()))
                } else {
                    None
                }
            }
            (PathRange::Range { start: s1, end: e1 }, PathRange::Range { start: s2, end: e2 }) => {
                let start = s1.max(s2).clone();
                let end = e1.min(e2).clone();
                if start < end {
                    Some(PathRange::Range { start, end })
                } else {
                    None
                }
            }
            _ => {
                // Other combinations require more complex logic
                None
            }
        }
    }
}

/// A range of node IDs (subspaces).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeIdRange {
    /// Single node
    Exact(NodeId),
    
    /// Range between two node IDs
    Range { start: NodeId, end: NodeId },
    
    /// All nodes
    All,
}

impl NodeIdRange {
    /// Check if a node ID is included in this range.
    pub fn contains(&self, node_id: &NodeId) -> bool {
        match self {
            NodeIdRange::Exact(n) => node_id == n,
            NodeIdRange::Range { start, end } => {
                node_id >= start && node_id < end
            }
            NodeIdRange::All => true,
        }
    }

    /// Check if this range is a subset of another range.
    pub fn is_subset_of(&self, other: &NodeIdRange) -> bool {
        match (self, other) {
            (_, NodeIdRange::All) => true,
            (NodeIdRange::All, _) => false,
            (NodeIdRange::Exact(n), NodeIdRange::Exact(o)) => n == o,
            (NodeIdRange::Exact(n), NodeIdRange::Range { start, end }) => {
                n >= start && n < end
            }
            (NodeIdRange::Range { start: s1, end: e1 }, NodeIdRange::Range { start: s2, end: e2 }) => {
                s1 >= s2 && e1 <= e2
            }
            _ => false,
        }
    }
}

/// A type-safe range for a specific model key.
///
/// KeyRange is generic over the model's key type, ensuring
/// compile-time verification that ranges match the model schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyRange<K> {
    /// The underlying path range
    pub range: PathRange,
    
    /// Phantom data to bind the key type
    _phantom: PhantomData<K>,
}

impl<K> KeyRange<K> {
    /// Create a new key range.
    pub fn new(range: PathRange) -> Self {
        Self {
            range,
            _phantom: PhantomData,
        }
    }

    /// Create an exact key range.
    pub fn exact(path: Path) -> Self {
        Self::new(PathRange::Exact(path))
    }

    /// Create a prefix key range.
    pub fn prefix(path: Path) -> Self {
        Self::new(PathRange::Prefix(path))
    }

    /// Create a range between two paths.
    pub fn between(start: Path, end: Path) -> Self {
        Self::new(PathRange::Range { start, end })
    }

    /// Create a range covering all keys.
    pub fn all() -> Self {
        Self::new(PathRange::All)
    }

    /// Check if this range contains a path.
    pub fn contains(&self, path: &Path) -> bool {
        self.range.contains(path)
    }

    /// Check if this range is a subset of another.
    pub fn is_subset_of(&self, other: &KeyRange<K>) -> bool {
        self.range.is_subset_of(&other.range)
    }

    /// Intersect with another key range.
    pub fn intersect(&self, other: &KeyRange<K>) -> Option<KeyRange<K>> {
        self.range.intersect(&other.range).map(|range| KeyRange {
            range,
            _phantom: PhantomData,
        })
    }
}

/// A range for secondary key queries.
///
/// SecondaryKeyRange includes the discriminant to identify which
/// secondary key is being queried, enabling multi-dimensional queries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecondaryKeyRange<D> {
    /// The secondary key discriminant (identifies which index)
    pub discriminant: D,
    
    /// The range within that secondary key
    pub range: PathRange,
}

impl<D> SecondaryKeyRange<D> {
    /// Create a new secondary key range.
    pub fn new(discriminant: D, range: PathRange) -> Self {
        Self { discriminant, range }
    }

    /// Check if this range contains a path.
    pub fn contains(&self, path: &Path) -> bool {
        self.range.contains(path)
    }
}

/// Combined N-dimensional range for queries.
///
/// This allows querying across:
/// - Subspace (owner/author)
/// - Primary key
/// - Multiple secondary keys
///
/// All dimensions can be restricted simultaneously for fine-grained access control.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NDimensionalRange<PK, SK> {
    /// Range of node IDs (subspaces)
    pub subspace: NodeIdRange,
    
    /// Range of primary keys
    pub primary_key: KeyRange<PK>,
    
    /// Optional ranges for secondary keys
    pub secondary_keys: Vec<SecondaryKeyRange<SK>>,
}

impl<PK, SK> NDimensionalRange<PK, SK>
where
    SK: PartialEq,
{
    /// Create a new N-dimensional range.
    pub fn new(
        subspace: NodeIdRange,
        primary_key: KeyRange<PK>,
        secondary_keys: Vec<SecondaryKeyRange<SK>>,
    ) -> Self {
        Self {
            subspace,
            primary_key,
            secondary_keys,
        }
    }

    /// Create a range covering all dimensions.
    pub fn all() -> Self {
        Self {
            subspace: NodeIdRange::All,
            primary_key: KeyRange::all(),
            secondary_keys: Vec::new(),
        }
    }

    /// Check if this range is a subset of another.
    pub fn is_subset_of(&self, other: &NDimensionalRange<PK, SK>) -> bool {
        // Subspace must be subset
        if !self.subspace.is_subset_of(&other.subspace) {
            return false;
        }

        // Primary key must be subset
        if !self.primary_key.is_subset_of(&other.primary_key) {
            return false;
        }

        // All our secondary keys must be covered by other's secondary keys
        for our_sk in &self.secondary_keys {
            let covered = other.secondary_keys.iter().any(|other_sk| {
                other_sk.discriminant == our_sk.discriminant
                    && our_sk.range.is_subset_of(&other_sk.range)
            });
            if !covered {
                return false;
            }
        }

        true
    }

    /// Get the secondary key range for a specific discriminant.
    pub fn get_secondary_range(&self, discriminant: &SK) -> Option<&PathRange> {
        self.secondary_keys
            .iter()
            .find(|sk| &sk.discriminant == discriminant)
            .map(|sk| &sk.range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::PathBuilder;

    #[test]
    fn test_path_range_contains() {
        let path = PathBuilder::new().key("users").key("alice").build();
        let prefix = PathBuilder::new().key("users").build();
        let range = PathRange::Prefix(prefix);

        assert!(range.contains(&path));
    }

    #[test]
    fn test_path_range_intersection() {
        let prefix1 = PathRange::Prefix(PathBuilder::new().key("users").build());
        let prefix2 = PathRange::Prefix(PathBuilder::new().key("users").key("a").build());

        let intersection = prefix1.intersect(&prefix2).unwrap();
        assert_eq!(intersection, prefix2);
    }

    #[test]
    fn test_key_range_subset() {
        let range1: KeyRange<String> = KeyRange::prefix(PathBuilder::new().key("users").build());
        let range2: KeyRange<String> = KeyRange::all();

        assert!(range1.is_subset_of(&range2));
        assert!(!range2.is_subset_of(&range1));
    }

    #[test]
    fn test_node_id_range() {
        let node1 = NodeId::from_bytes([1u8; 32]);
        let node2 = NodeId::from_bytes([2u8; 32]);
        let range = NodeIdRange::Range {
            start: node1,
            end: node2,
        };

        assert!(range.contains(&node1));
        assert!(!range.contains(&node2));
    }
}
