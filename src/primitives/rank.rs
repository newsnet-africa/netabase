//! Conflict resolution and ranking primitives.
//!
//! This module provides a flexible ranking system for conflict resolution
//! that allows models to define custom business logic while maintaining
//! deterministic ordering.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// A rank value used for conflict resolution.
///
/// ConflictRank combines a user-defined rank with a Lamport clock
/// for deterministic tie-breaking. This allows models to implement
/// custom conflict resolution logic while guaranteeing consistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictRank {
    /// User-defined rank value (higher wins)
    pub rank: u64,
    
    /// Lamport clock for causality tracking
    pub lamport: LamportClock,
}

impl ConflictRank {
    /// Create a new conflict rank.
    pub const fn new(rank: u64, lamport: LamportClock) -> Self {
        Self { rank, lamport }
    }

    /// Create a rank with just a Lamport clock (rank = 0).
    pub const fn from_lamport(lamport: LamportClock) -> Self {
        Self { rank: 0, lamport }
    }

    /// Create a basic rank without causality tracking.
    pub const fn basic(rank: u64) -> Self {
        Self {
            rank,
            lamport: LamportClock::ZERO,
        }
    }
}

impl PartialOrd for ConflictRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ConflictRank {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare rank values
        match self.rank.cmp(&other.rank) {
            Ordering::Equal => {
                // Tie-break with Lamport clock
                self.lamport.cmp(&other.lamport)
            }
            other => other,
        }
    }
}

/// Lamport clock for causality-preserving ordering.
///
/// Lamport clocks provide a partial ordering of events in a distributed system.
/// The combination of counter and node_id ensures deterministic tie-breaking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LamportClock {
    /// Logical clock counter
    pub counter: u64,
    
    /// Node identifier for tie-breaking (first 8 bytes of NodeId hash)
    pub node_id: [u8; 8],
}

impl LamportClock {
    /// Zero clock (initial state).
    pub const ZERO: Self = Self {
        counter: 0,
        node_id: [0u8; 8],
    };

    /// Create a new Lamport clock.
    pub const fn new(counter: u64, node_id: [u8; 8]) -> Self {
        Self { counter, node_id }
    }

    /// Increment the clock (local event).
    pub fn tick(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    /// Merge with another clock (receive event).
    ///
    /// Sets this clock to max(local, remote) + 1.
    pub fn merge(&mut self, other: &Self) {
        self.counter = self.counter.max(other.counter).saturating_add(1);
    }

    /// Get the maximum of two clocks without incrementing.
    pub fn max_with(&self, other: &Self) -> Self {
        if self.counter > other.counter {
            *self
        } else if other.counter > self.counter {
            *other
        } else {
            // Equal counters - use node_id as tie-breaker
            if self.node_id >= other.node_id {
                *self
            } else {
                *other
            }
        }
    }
}

impl PartialOrd for LamportClock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LamportClock {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.counter.cmp(&other.counter) {
            Ordering::Equal => self.node_id.cmp(&other.node_id),
            other => other,
        }
    }
}

impl Default for LamportClock {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for LamportClock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.counter, hex::encode(&self.node_id[..4]))
    }
}

/// Trait for models that define custom ranking strategies.
///
/// Models can implement this trait to provide business-logic-aware
/// conflict resolution. The rank can be based on any model fields.
///
/// # Examples
///
/// ```ignore
/// #[derive(NetabaseModel)]
/// struct Document {
///     #[primary_key]
///     id: String,
///     version: u64,
///     lamport: LamportClock,
/// }
///
/// impl RankStrategy for Document {
///     fn conflict_rank(&self) -> ConflictRank {
///         ConflictRank::new(self.version, self.lamport)
///     }
///
///     fn merge(&self, other: &Self) -> Option<Self> {
///         // Custom merge logic
///         if self.version > other.version {
///             Some(self.clone())
///         } else {
///             Some(other.clone())
///         }
///     }
/// }
/// ```
pub trait RankStrategy {
    /// Compute the conflict rank for this instance.
    ///
    /// This method should be deterministic and based only on
    /// the model's fields (not external state).
    fn conflict_rank(&self) -> ConflictRank;

    /// Attempt to merge this instance with another.
    ///
    /// Returns `Some(merged)` if merging is possible, or `None`
    /// to fall back to rank-based resolution.
    ///
    /// Default implementation returns `None` (no merging).
    fn merge(&self, _other: &Self) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }

    /// Check if this instance should supersede another based on rank.
    fn supersedes(&self, other: &Self) -> bool {
        self.conflict_rank() > other.conflict_rank()
    }
}

/// Pre-defined ranking strategies for common use cases.
pub mod strategies {
    use super::*;

    /// Last-Write-Wins based on Lamport clock only.
    #[derive(Debug, Clone, Copy)]
    pub struct LamportLWW;

    impl LamportLWW {
        pub fn rank(lamport: LamportClock) -> ConflictRank {
            ConflictRank::from_lamport(lamport)
        }
    }

    /// Version-based ranking (higher version wins).
    #[derive(Debug, Clone, Copy)]
    pub struct VersionRank;

    impl VersionRank {
        pub fn rank(version: u64, lamport: LamportClock) -> ConflictRank {
            ConflictRank::new(version, lamport)
        }
    }

    /// Timestamp-based ranking (newer wins).
    #[derive(Debug, Clone, Copy)]
    pub struct TimestampRank;

    impl TimestampRank {
        pub fn rank(timestamp_ms: u64, lamport: LamportClock) -> ConflictRank {
            ConflictRank::new(timestamp_ms, lamport)
        }
    }

    /// Counter-based ranking (higher count wins) - useful for CRDTs.
    #[derive(Debug, Clone, Copy)]
    pub struct CounterRank;

    impl CounterRank {
        pub fn rank(count: u64, lamport: LamportClock) -> ConflictRank {
            ConflictRank::new(count, lamport)
        }
    }
}

/// Conflict resolution strategy enum.
///
/// Determines how conflicts are resolved when sync encounters
/// entries with the same key but different content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStrategy {
    /// Use rank-based resolution (default)
    Rank,
    
    /// Attempt merge, fallback to rank
    Merge,
    
    /// Keep both entries with disambiguated paths
    KeepBoth,
    
    /// Always reject remote (local authority)
    LocalWins,
    
    /// Always accept remote (remote authority)
    RemoteWins,
}

impl Default for ConflictStrategy {
    fn default() -> Self {
        Self::Rank
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lamport_clock_ordering() {
        let clock1 = LamportClock::new(10, [1u8; 8]);
        let clock2 = LamportClock::new(20, [1u8; 8]);
        assert!(clock1 < clock2);
    }

    #[test]
    fn test_lamport_clock_tie_break() {
        let clock1 = LamportClock::new(10, [1u8; 8]);
        let clock2 = LamportClock::new(10, [2u8; 8]);
        assert!(clock1 < clock2);
    }

    #[test]
    fn test_lamport_clock_merge() {
        let mut clock1 = LamportClock::new(10, [1u8; 8]);
        let clock2 = LamportClock::new(20, [2u8; 8]);
        clock1.merge(&clock2);
        assert_eq!(clock1.counter, 21);
    }

    #[test]
    fn test_conflict_rank_ordering() {
        let rank1 = ConflictRank::new(10, LamportClock::new(5, [1u8; 8]));
        let rank2 = ConflictRank::new(20, LamportClock::new(5, [1u8; 8]));
        assert!(rank1 < rank2);
    }

    #[test]
    fn test_conflict_rank_lamport_tiebreak() {
        let rank1 = ConflictRank::new(10, LamportClock::new(5, [1u8; 8]));
        let rank2 = ConflictRank::new(10, LamportClock::new(10, [1u8; 8]));
        assert!(rank1 < rank2);
    }
}
