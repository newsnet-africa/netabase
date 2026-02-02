//! Node implementation that integrates store and network.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use netabase_store::databases::redb::RedbStore;
use netabase_store::prelude::NetabaseDefinition;
use netabase_store::traits::database::store::NBStore;
use netabase_store::traits::registry::definition::redb_definition::RedbDefinition;

use crate::capabilities::{Capability, CapabilitySignature, Operation};
use crate::network::protocol::*;
use crate::primitives::{LamportClock, NDimensionalRange, NodeId};
use crate::query::{CapabilityGuard, QueryError, SecureQuery};
use crate::query::executor::QueryGuard;

/// Configuration for a node.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// Maximum queries per second
    pub max_queries_per_second: u64,
    
    /// Maximum concurrent connections
    pub max_connections: usize,
    
    /// Query timeout in seconds
    pub query_timeout_secs: u64,
    
    /// Enable sync protocol
    pub enable_sync: bool,
    
    /// Enable PAI
    pub enable_pai: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            max_queries_per_second: 100,
            max_connections: 1000,
            query_timeout_secs: 30,
            enable_sync: true,
            enable_pai: false,
        }
    }
}

/// A Netabase node with integrated store and network capabilities.
pub struct Node<D>
where
    D: NetabaseDefinition + RedbDefinition + Clone + 'static,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + Send + Sync + 'static,
{
    /// This node's ID
    pub id: NodeId,
    
    /// Configuration
    pub config: NodeConfig,
    
    /// The underlying database store
    pub store: Arc<RedbStore<D>>,
    
    /// Lamport clock for causality tracking
    clock: Arc<Mutex<LamportClock>>,
    
    /// Capabilities granted by this node
    granted_caps: Arc<Mutex<Vec<Capability<String, u16>>>>,
    
    /// Nonce tracker for replay protection
    nonces: Arc<Mutex<HashMap<NodeId, u64>>>,
}

impl<D> Node<D>
where
    D: NetabaseDefinition + RedbDefinition + Clone + 'static,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + Send + Sync + 'static,
{
    /// Create a new node with the given store.
    pub fn new(id: NodeId, store: RedbStore<D>, config: NodeConfig) -> Self {
        Self {
            id,
            config,
            store: Arc::new(store),
            clock: Arc::new(Mutex::new(LamportClock::new(
                0,
                id.to_bytes()[0..8].try_into().unwrap(),
            ))),
            granted_caps: Arc::new(Mutex::new(Vec::new())),
            nonces: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Grant a capability to a peer.
    pub fn grant_capability(
        &self,
        to: NodeId,
        operation: Operation,
        range: NDimensionalRange<String, u16>,
        expiry: u64,
    ) -> Capability<String, u16> {
        let cap = Capability::new_root(self.id, to, operation, range, expiry);
        self.granted_caps.lock().unwrap().push(cap.clone());
        cap
    }
    
    /// Get the node's current Lamport clock value.
    pub fn clock_value(&self) -> u64 {
        self.clock.lock().unwrap().counter
    }
    
    /// Tick the Lamport clock.
    pub fn tick_clock(&self) {
        self.clock.lock().unwrap().tick();
    }
    
    /// Merge a remote Lamport clock.
    pub fn merge_clock(&self, remote: &LamportClock) {
        self.clock.lock().unwrap().merge(remote);
    }
    
    /// Get current timestamp.
    pub fn timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    
    /// Create a handshake request.
    pub fn create_handshake_request(&self) -> HandshakeRequest {
        let nonce = {
            let mut clock = self.clock.lock().unwrap();
            clock.tick();
            clock.counter
        };
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        HandshakeRequest {
            from: self.id,
            protocol_version: 1,
            features: features::SYNC | features::SUBSCRIPTIONS,
            schema_hash: [0u8; 32], // TODO: Actual schema hash
            nonce,
            timestamp,
        }
    }
    
    /// Handle a handshake request.
    pub fn handle_handshake_request(&self, req: HandshakeRequest) -> HandshakeResponse {
        // Check protocol version
        let accepted = req.protocol_version == 1;
        
        let reason = if !accepted {
            Some("Incompatible protocol version".to_string())
        } else {
            None
        };
        
        HandshakeResponse {
            from: self.id,
            protocol_version: 1,
            accepted,
            reason,
            signature: CapabilitySignature([0u8; 64]), // TODO: Actual signature
        }
    }
    
    /// Validate a query capability.
    pub fn validate_query<PK, SK>(
        &self,
        query: &SecureQuery<PK, SK>,
    ) -> Result<(), QueryError>
    where
        PK: Clone + PartialEq,
        SK: Clone + PartialEq,
    {
        // Check nonce
        let mut nonces = self.nonces.lock().unwrap();
        if let Some(&last_nonce) = nonces.get(&query.capability.granted_by) {
            if query.nonce <= last_nonce {
                return Err(QueryError::ReplayDetected);
            }
        }
        nonces.insert(query.capability.granted_by, query.nonce);
        
        // Check timestamp
        let now = self.timestamp();
        let skew = if query.timestamp > now {
            query.timestamp - now
        } else {
            now - query.timestamp
        };
        
        if skew > 300 {
            // 5 minute max skew
            return Err(QueryError::TimestampSkew);
        }
        
        // Validate capability
        let guard = CapabilityGuard::new(query.capability.granted_by);
        guard.check_query(query)?;
        
        Ok(())
    }
    
    /// Get feature flags for this node.
    pub fn features(&self) -> u64 {
        let mut features = features::SUBSCRIPTIONS;
        
        if self.config.enable_sync {
            features |= features::SYNC;
        }
        
        if self.config.enable_pai {
            features |= features::PAI;
        }
        
        features
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_config_default() {
        let config = NodeConfig::default();
        assert_eq!(config.max_queries_per_second, 100);
        assert_eq!(config.max_connections, 1000);
        assert!(config.enable_sync);
    }
    
    // Note: Store integration tests moved to integration tests
    // where netabase_macros is available
}
