//! Sync protocol handler.
//!
//! This module implements the synchronization protocol for reconciling
//! data between peers using fingerprints and range-based queries.

use std::collections::HashMap;

use crate::network::protocol::{
    Fingerprint, RangeFingerprint, SyncRequest, SyncResponse, SyncStrategy,
};
use crate::primitives::{LamportClock, NDimensionalRange, NodeId};
use crate::query::messages::QueryEntry;

/// State of a sync operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncState {
    /// Initial state
    Init,
    
    /// Comparing fingerprints
    ComparingFingerprints,
    
    /// Requesting subranges
    RequestingSubranges {
        ranges: Vec<RangeFingerprint>,
    },
    
    /// Syncing data
    SyncingData {
        remaining_ranges: usize,
    },
    
    /// Sync complete
    Complete {
        entries_synced: usize,
    },
    
    /// Sync failed
    Failed {
        reason: String,
    },
}

/// Sync protocol handler.
pub struct SyncHandler<PK, SK> {
    local_id: NodeId,
    state: SyncState,
    _phantom: std::marker::PhantomData<(PK, SK)>,
}

impl<PK, SK> SyncHandler<PK, SK> {
    /// Create a new sync handler.
    pub fn new(local_id: NodeId) -> Self {
        Self {
            local_id,
            state: SyncState::Init,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Get current state.
    pub fn state(&self) -> &SyncState {
        &self.state
    }
    
    /// Is sync complete?
    pub fn is_complete(&self) -> bool {
        matches!(self.state, SyncState::Complete { .. })
    }
    
    /// Calculate fingerprint for a range.
    pub fn calculate_fingerprint<T>(
        &self,
        entries: &[QueryEntry<T>],
    ) -> Fingerprint {
        use blake3::Hasher;
        
        let mut hasher = Hasher::new();
        let mut max_clock = 0u64;
        
        for entry in entries {
            hasher.update(&entry.data_hash);
            max_clock = max_clock.max(entry.lamport.counter);
        }
        
        Fingerprint {
            hash: *hasher.finalize().as_bytes(),
            count: entries.len() as u64,
            max_clock,
        }
    }
    
    /// Handle a sync request.
    pub fn handle_sync_request<T>(
        &mut self,
        request: &SyncRequest<PK, SK>,
        local_entries: Vec<QueryEntry<T>>,
    ) -> SyncResponse<T>
    where
        T: Clone,
    {
        // Calculate local fingerprint
        let local_fp = self.calculate_fingerprint(&local_entries);
        
        // Compare with remote fingerprint
        if local_fp.hash == request.local_fingerprint.hash {
            // Fingerprints match - no sync needed
            self.state = SyncState::Complete { entries_synced: 0 };
            
            return SyncResponse {
                strategy: SyncStrategy::NoOp,
                entries: vec![],
                fingerprints: vec![],
                has_more: false,
            };
        }
        
        // If small dataset, just send everything
        if local_entries.len() < 100 {
            self.state = SyncState::Complete {
                entries_synced: local_entries.len(),
            };
            
            return SyncResponse {
                strategy: SyncStrategy::Full,
                entries: local_entries,
                fingerprints: vec![],
                has_more: false,
            };
        }
        
        // For larger datasets, use incremental sync
        // Split range into subranges and send fingerprints
        self.state = SyncState::ComparingFingerprints;
        
        // TODO: Actually split the range intelligently
        // For now, just send full data
        SyncResponse {
            strategy: SyncStrategy::Full,
            entries: local_entries,
            fingerprints: vec![],
            has_more: false,
        }
    }
    
    /// Process a sync response.
    pub fn process_sync_response<T>(
        &mut self,
        response: &SyncResponse<T>,
    ) -> Vec<QueryEntry<T>>
    where
        T: Clone,
    {
        match response.strategy {
            SyncStrategy::NoOp => {
                self.state = SyncState::Complete { entries_synced: 0 };
                vec![]
            }
            SyncStrategy::Full => {
                self.state = SyncState::Complete {
                    entries_synced: response.entries.len(),
                };
                response.entries.clone()
            }
            SyncStrategy::Incremental => {
                // TODO: Handle incremental sync
                self.state = SyncState::RequestingSubranges {
                    ranges: response.fingerprints.clone(),
                };
                vec![]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::ConflictRank;
    
    #[test]
    fn test_fingerprint_calculation() {
        let node_id = NodeId::from_bytes([1u8; 32]);
        let handler: SyncHandler<String, u16> = SyncHandler::new(node_id);
        
        let entries = vec![
            QueryEntry {
                author: node_id,
                rank: ConflictRank::basic(1),
                lamport: LamportClock::new(10, [1u8; 8]),
                data: "test1".to_string(),
                data_hash: [1u8; 32],
            },
            QueryEntry {
                author: node_id,
                rank: ConflictRank::basic(2),
                lamport: LamportClock::new(20, [1u8; 8]),
                data: "test2".to_string(),
                data_hash: [2u8; 32],
            },
        ];
        
        let fp = handler.calculate_fingerprint(&entries);
        assert_eq!(fp.count, 2);
        assert_eq!(fp.max_clock, 20);
    }
}
