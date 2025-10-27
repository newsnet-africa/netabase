//! Conflict-free Replicated Data Types (CRDTs)
//!
//! This module provides CRDT implementations for automatic conflict resolution.

// TODO: Phase 4 - Implement CRDTs
// - lww.rs: Last-Write-Wins Register
// - orset.rs: Observed-Remove Set
// - counter.rs: Grow-only Counter

pub mod lww;
pub mod orset;
pub mod counter;

use crate::sync::traits::CRDT;
use anyhow::Result;

/// CRDT wrapper for netabase records
#[derive(Clone, Debug)]
pub enum CrdtType {
    /// Last-Write-Wins register
    Lww(lww::LwwRegister),

    /// Observed-Remove Set
    OrSet(orset::OrSet),

    /// Grow-only counter
    Counter(counter::GCounter),
}

impl CrdtType {
    /// Merge this CRDT with another
    pub fn merge(&mut self, other: &Self) -> Result<()> {
        match (self, other) {
            (CrdtType::Lww(a), CrdtType::Lww(b)) => a.merge(b),
            (CrdtType::OrSet(a), CrdtType::OrSet(b)) => a.merge(b),
            (CrdtType::Counter(a), CrdtType::Counter(b)) => a.merge(b),
            _ => anyhow::bail!("Cannot merge different CRDT types"),
        }
    }

    /// Get the value
    pub fn value(&self) -> Vec<u8> {
        match self {
            CrdtType::Lww(lww) => lww.value(),
            CrdtType::OrSet(set) => set.value(),
            CrdtType::Counter(counter) => counter.value(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_type() {
        // Placeholder test
        assert!(true);
    }
}
