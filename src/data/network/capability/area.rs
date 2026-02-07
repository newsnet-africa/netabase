//! Meadowcap Area type
//!
//! An Area defines a region of data within a namespace, constraining:
//! - Subspace (who owns the data)
//! - Path prefix (where in the hierarchy)
//! - Time range (when it was created)

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// User public key (SubspaceId in Willow terms)
pub type SubspaceId = PeerId;

/// An Area defines a region of entries within a namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    /// Subspace constraint
    pub subspace: SubspaceConstraint,
    /// Path prefix constraint
    pub path: PathConstraint,
    /// Time range constraint
    pub times: TimeRange,
}

/// Constraint on which subspace(s) an area covers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubspaceConstraint {
    /// Any subspace
    Any,
    /// A specific subspace
    Specific(SubspaceId),
}

impl SubspaceConstraint {
    /// Check if this constraint includes a specific subspace
    pub fn includes(&self, subspace: &SubspaceId) -> bool {
        match self {
            Self::Any => true,
            Self::Specific(s) => s == subspace,
        }
    }

    /// Check if this constraint includes another constraint
    pub fn includes_constraint(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Any, _) => true,
            (Self::Specific(a), Self::Specific(b)) => a == b,
            (Self::Specific(_), Self::Any) => false,
        }
    }
}

/// Constraint on path prefix
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathConstraint {
    /// Path components that must be a prefix
    pub components: Vec<Vec<u8>>,
}

impl PathConstraint {
    /// Empty path (matches everything)
    pub fn empty() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    /// Create from components
    pub fn new(components: Vec<Vec<u8>>) -> Self {
        Self { components }
    }

    /// Check if this path is a prefix of another
    pub fn is_prefix_of(&self, other: &Self) -> bool {
        if self.components.len() > other.components.len() {
            return false;
        }

        self.components
            .iter()
            .zip(other.components.iter())
            .all(|(a, b)| a == b)
    }

    /// Check if this constraint includes another
    pub fn includes(&self, other: &Self) -> bool {
        self.is_prefix_of(other)
    }

    /// Compute hash
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for component in &self.components {
            hasher.update(&(component.len() as u32).to_le_bytes());
            hasher.update(component);
        }
        hasher.finalize().into()
    }
}

/// Time range constraint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (inclusive), None means from the beginning
    pub start: Option<u64>,
    /// End time (exclusive), None means until the end
    pub end: Option<u64>,
}

impl TimeRange {
    /// Open time range (all times)
    pub fn open() -> Self {
        Self {
            start: None,
            end: None,
        }
    }

    /// Create a range from start to end
    pub fn new(start: Option<u64>, end: Option<u64>) -> Self {
        Self { start, end }
    }

    /// Check if a timestamp is within this range
    pub fn contains(&self, timestamp: u64) -> bool {
        let after_start = self.start.map(|s| timestamp >= s).unwrap_or(true);
        let before_end = self.end.map(|e| timestamp < e).unwrap_or(true);
        after_start && before_end
    }

    /// Check if this range includes another range
    pub fn includes(&self, other: &Self) -> bool {
        // Our start must be <= their start (or we have no start)
        let start_ok = match (self.start, other.start) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(a), Some(b)) => a <= b,
        };

        // Our end must be >= their end (or we have no end)
        let end_ok = match (self.end, other.end) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(a), Some(b)) => a >= b,
        };

        start_ok && end_ok
    }
}

impl Area {
    /// Create a full area (entire namespace)
    pub fn full() -> Self {
        Self {
            subspace: SubspaceConstraint::Any,
            path: PathConstraint::empty(),
            times: TimeRange::open(),
        }
    }

    /// Create an area for a specific subspace
    pub fn subspace(subspace_id: SubspaceId) -> Self {
        Self {
            subspace: SubspaceConstraint::Specific(subspace_id),
            path: PathConstraint::empty(),
            times: TimeRange::open(),
        }
    }

    /// Create an area with a path prefix
    pub fn with_path(mut self, path: PathConstraint) -> Self {
        self.path = path;
        self
    }

    /// Create an area with a time range
    pub fn with_times(mut self, times: TimeRange) -> Self {
        self.times = times;
        self
    }

    /// Check if this area includes another area
    pub fn includes(&self, other: &Area) -> bool {
        self.subspace.includes_constraint(&other.subspace)
            && self.path.includes(&other.path)
            && self.times.includes(&other.times)
    }

    /// Check if this area includes a specific subspace
    pub fn includes_subspace(&self, subspace: &SubspaceId) -> bool {
        self.subspace.includes(subspace)
    }

    /// Check if this area includes a specific entry
    pub fn includes_entry(
        &self,
        subspace: &SubspaceId,
        path: &PathConstraint,
        timestamp: u64,
    ) -> bool {
        self.subspace.includes(subspace)
            && self.path.is_prefix_of(path)
            && self.times.contains(timestamp)
    }

    /// Compute hash of this area
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"area");

        match &self.subspace {
            SubspaceConstraint::Any => hasher.update(&[0x00]),
            SubspaceConstraint::Specific(s) => {
                hasher.update(&[0x01]);
                hasher.update(&s.to_bytes());
            }
        }

        hasher.update(&self.path.hash());

        if let Some(start) = self.times.start {
            hasher.update(&start.to_le_bytes());
        }
        if let Some(end) = self.times.end {
            hasher.update(&end.to_le_bytes());
        }

        hasher.finalize().into()
    }

    /// Intersection of two areas (returns None if disjoint)
    pub fn intersection(&self, other: &Area) -> Option<Area> {
        // Subspace intersection
        let subspace = match (&self.subspace, &other.subspace) {
            (SubspaceConstraint::Any, s) | (s, SubspaceConstraint::Any) => s.clone(),
            (SubspaceConstraint::Specific(a), SubspaceConstraint::Specific(b)) => {
                if a == b {
                    SubspaceConstraint::Specific(a.clone())
                } else {
                    return None; // Disjoint
                }
            }
        };

        // Path intersection (take the more specific one)
        let path = if self.path.is_prefix_of(&other.path) {
            other.path.clone()
        } else if other.path.is_prefix_of(&self.path) {
            self.path.clone()
        } else {
            return None; // Disjoint paths
        };

        // Time intersection
        let start = match (self.times.start, other.times.start) {
            (None, s) | (s, None) => s,
            (Some(a), Some(b)) => Some(a.max(b)),
        };
        let end = match (self.times.end, other.times.end) {
            (None, e) | (e, None) => e,
            (Some(a), Some(b)) => Some(a.min(b)),
        };

        // Check if time range is valid
        if let (Some(s), Some(e)) = (start, end) {
            if s >= e {
                return None; // Empty time range
            }
        }

        Some(Area {
            subspace,
            path,
            times: TimeRange { start, end },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;

    #[test]
    fn test_subspace_constraint_includes() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        assert!(SubspaceConstraint::Any.includes(&peer1));
        assert!(SubspaceConstraint::Specific(peer1.clone()).includes(&peer1));
        assert!(!SubspaceConstraint::Specific(peer1).includes(&peer2));
    }

    #[test]
    fn test_time_range_contains() {
        let range = TimeRange::new(Some(100), Some(200));
        assert!(range.contains(100));
        assert!(range.contains(150));
        assert!(!range.contains(99));
        assert!(!range.contains(200));

        let open = TimeRange::open();
        assert!(open.contains(0));
        assert!(open.contains(u64::MAX));
    }

    #[test]
    fn test_area_includes() {
        let full = Area::full();
        let subspace = Area::subspace(PeerId::random());

        assert!(full.includes(&subspace));
        assert!(!subspace.includes(&full));
    }

    #[test]
    fn test_path_prefix() {
        let p1 = PathConstraint::new(vec![b"a".to_vec()]);
        let p2 = PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec()]);

        assert!(p1.is_prefix_of(&p2));
        assert!(!p2.is_prefix_of(&p1));
        assert!(p1.is_prefix_of(&p1));
    }
}
