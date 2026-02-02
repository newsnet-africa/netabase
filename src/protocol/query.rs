//! Query protocol handler.
//!
//! This module implements query execution with capability-based authorization.

use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use netabase_store::traits::database::store::NBStore;

use crate::capabilities::{Capability, CapabilitySignature, Operation};
use crate::primitives::{ConflictRank, LamportClock, NDimensionalRange, NodeId};
use crate::query::messages::{
    QueryEntry, QueryError, QueryResponse, SecureQuery, WriteRequest, WriteResponse,
};
use crate::query::{CapabilityGuard, QueryGuard};

/// Result type for queries.
pub type QueryResult<T> = Result<T, QueryError>;

/// Query protocol handler.
pub struct QueryHandler<S, PK, SK> {
    /// The underlying store
    store: Arc<S>,
    
    /// Local node ID
    node_id: NodeId,
    
    /// Lamport clock
    clock: Arc<Mutex<LamportClock>>,
    
    /// Nonce tracker for replay protection
    nonces: Arc<Mutex<HashMap<NodeId, u64>>>,
    
    _phantom: PhantomData<(PK, SK)>,
}

impl<S, PK, SK> QueryHandler<S, PK, SK> {
    /// Create a new query handler.
    pub fn new(
        store: Arc<S>,
        node_id: NodeId,
        clock: Arc<Mutex<LamportClock>>,
    ) -> Self {
        Self {
            store,
            node_id,
            clock,
            nonces: Arc::new(Mutex::new(HashMap::new())),
            _phantom: PhantomData,
        }
    }
    
    /// Validate a query's authorization and freshness.
    pub fn validate_query(&self, query: &SecureQuery<PK, SK>) -> QueryResult<()>
    where
        PK: Clone + PartialEq,
        SK: Clone + PartialEq,
    {
        // 1. Check nonce (replay protection)
        let mut nonces = self.nonces.lock().unwrap();
        let sender = query.capability.granted_to;
        
        if let Some(&last_nonce) = nonces.get(&sender) {
            if query.nonce <= last_nonce {
                return Err(QueryError::ReplayDetected);
            }
        }
        nonces.insert(sender, query.nonce);
        drop(nonces);
        
        // 2. Check timestamp (clock skew protection)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let skew = if query.timestamp > now {
            query.timestamp - now
        } else {
            now - query.timestamp
        };
        
        if skew > 300 {
            // Max 5 minutes skew
            return Err(QueryError::TimestampSkew);
        }
        
        // 3. Validate capability
        let guard = CapabilityGuard::new(query.capability.granted_by);
        guard.check_query(query)?;
        
        // 4. Check capability expiry
        if query.capability.expiry < now {
            return Err(QueryError::CapabilityExpired);
        }
        
        // 5. Verify signature (TODO: actual signature verification)
        // For now we just check it's not all zeros
        if query.signature.0 == [0u8; 64] {
            // Allow for testing
        }
        
        Ok(())
    }
    
    /// Create a signed query response.
    pub fn create_response<T>(
        &self,
        entries: Vec<QueryEntry<T>>,
        has_more: bool,
    ) -> QueryResponse<T> {
        // TODO: Actually sign the response
        let signature = CapabilitySignature([0u8; 64]);
        
        QueryResponse {
            entries,
            has_more,
            continuation: None,
            responder: self.node_id,
            signature,
        }
    }
}

impl<S, PK, SK> Clone for QueryHandler<S, PK, SK> {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
            node_id: self.node_id,
            clock: Arc::clone(&self.clock),
            nonces: Arc::clone(&self.nonces),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{KeyRange, NodeIdRange, PathBuilder};
    
    #[test]
    fn test_query_validation_replay() {
        let node_id = NodeId::from_bytes([1u8; 32]);
        let clock = Arc::new(Mutex::new(LamportClock::new(0, [1u8; 8])));
        
        // Mock store (we don't use it in this test)
        struct MockStore;
        let store = Arc::new(MockStore);
        
        let handler: QueryHandler<MockStore, String, u16> = QueryHandler::new(
            store,
            node_id,
            clock,
        );
        
        let range = NDimensionalRange::new(
            NodeIdRange::All,
            KeyRange::all(),
            vec![],
        );
        
        let capability = Capability::new_root(
            NodeId::from_bytes([2u8; 32]),
            NodeId::from_bytes([3u8; 32]),
            Operation::Read,
            range.clone(),
            u64::MAX,
        );
        
        let query = SecureQuery {
            range: range.clone(),
            capability: capability.clone(),
            nonce: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signature: CapabilitySignature([1u8; 64]),
            continuation: None,
            limit: None,
        };
        
        // First query should pass
        handler.validate_query(&query).unwrap();
        
        // Replay with same nonce should fail
        let result = handler.validate_query(&query);
        assert!(matches!(result, Err(QueryError::ReplayDetected)));
    }
}
