//! Session management for peer connections.
//!
//! This module tracks established sessions with peers, including
//! their capabilities, shared areas of interest, and connection state.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::capabilities::Capability;
use crate::primitives::{LamportClock, NDimensionalRange, NodeId};
use crate::protocol::handshake::HandshakeState;

/// A session with a peer.
#[derive(Debug, Clone)]
pub struct PeerSession<PK, SK> {
    /// The peer's node ID
    pub peer_id: NodeId,
    
    /// Handshake state
    pub handshake_state: HandshakeState,
    
    /// Protocol version negotiated
    pub protocol_version: u32,
    
    /// Features supported by peer
    pub peer_features: u64,
    
    /// Capabilities we've granted to this peer
    pub granted_capabilities: Vec<Capability<PK, SK>>,
    
    /// Capabilities the peer has granted us
    pub received_capabilities: Vec<Capability<PK, SK>>,
    
    /// Areas of interest we're syncing with this peer
    pub sync_areas: Vec<NDimensionalRange<PK, SK>>,
    
    /// Last message timestamp (for timeout detection)
    pub last_message_time: u64,
    
    /// Lamport clock for this session
    pub clock: LamportClock,
}

impl<PK, SK> PeerSession<PK, SK> {
    /// Create a new peer session.
    pub fn new(peer_id: NodeId, protocol_version: u32, peer_features: u64) -> Self {
        Self {
            peer_id,
            handshake_state: HandshakeState::Init,
            protocol_version,
            peer_features,
            granted_capabilities: Vec::new(),
            received_capabilities: Vec::new(),
            sync_areas: Vec::new(),
            last_message_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            clock: LamportClock::new(0, peer_id.to_bytes()[0..8].try_into().unwrap()),
        }
    }
    
    /// Update last message time.
    pub fn touch(&mut self) {
        self.last_message_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    /// Check if session has timed out.
    pub fn is_timed_out(&self, timeout_secs: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now - self.last_message_time > timeout_secs
    }
    
    /// Grant a capability to this peer.
    pub fn grant_capability(&mut self, cap: Capability<PK, SK>)
    where
        PK: Clone,
        SK: Clone,
    {
        self.granted_capabilities.push(cap);
    }
    
    /// Receive a capability from this peer.
    pub fn receive_capability(&mut self, cap: Capability<PK, SK>)
    where
        PK: Clone,
        SK: Clone,
    {
        self.received_capabilities.push(cap);
    }
    
    /// Add a sync area.
    pub fn add_sync_area(&mut self, area: NDimensionalRange<PK, SK>)
    where
        PK: Clone,
        SK: Clone,
    {
        self.sync_areas.push(area);
    }
}

/// Manager for all peer sessions.
pub struct SessionManager<PK, SK> {
    sessions: Arc<Mutex<HashMap<NodeId, PeerSession<PK, SK>>>>,
    timeout_secs: u64,
}

impl<PK, SK> SessionManager<PK, SK> {
    /// Create a new session manager.
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            timeout_secs,
        }
    }
    
    /// Get a session by peer ID.
    pub fn get(&self, peer_id: &NodeId) -> Option<PeerSession<PK, SK>>
    where
        PK: Clone,
        SK: Clone,
    {
        self.sessions.lock().unwrap().get(peer_id).cloned()
    }
    
    /// Create or update a session.
    pub fn upsert(&self, session: PeerSession<PK, SK>)
    where
        PK: Clone,
        SK: Clone,
    {
        self.sessions.lock().unwrap().insert(session.peer_id, session);
    }
    
    /// Remove a session.
    pub fn remove(&self, peer_id: &NodeId) {
        self.sessions.lock().unwrap().remove(peer_id);
    }
    
    /// Get all active sessions.
    pub fn all(&self) -> Vec<PeerSession<PK, SK>>
    where
        PK: Clone,
        SK: Clone,
    {
        self.sessions.lock().unwrap().values().cloned().collect()
    }
    
    /// Remove timed out sessions.
    pub fn remove_timed_out(&self) -> Vec<NodeId> {
        let mut sessions = self.sessions.lock().unwrap();
        let timed_out: Vec<NodeId> = sessions
            .iter()
            .filter(|(_, s)| s.is_timed_out(self.timeout_secs))
            .map(|(id, _)| *id)
            .collect();
        
        for id in &timed_out {
            sessions.remove(id);
        }
        
        timed_out
    }
    
    /// Update a session with a function.
    pub fn update<F>(&self, peer_id: &NodeId, f: F)
    where
        F: FnOnce(&mut PeerSession<PK, SK>),
        PK: Clone,
        SK: Clone,
    {
        if let Some(session) = self.sessions.lock().unwrap().get_mut(peer_id) {
            f(session);
        }
    }
}

impl<PK, SK> Clone for SessionManager<PK, SK> {
    fn clone(&self) -> Self {
        Self {
            sessions: Arc::clone(&self.sessions),
            timeout_secs: self.timeout_secs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_creation() {
        let peer_id = NodeId::from_bytes([1u8; 32]);
        let session: PeerSession<String, u16> = PeerSession::new(peer_id, 1, 0);
        
        assert_eq!(session.peer_id, peer_id);
        assert_eq!(session.protocol_version, 1);
        assert!(!session.is_timed_out(60));
    }
    
    #[test]
    fn test_session_manager() {
        let manager: SessionManager<String, u16> = SessionManager::new(60);
        let peer_id = NodeId::from_bytes([1u8; 32]);
        
        let session = PeerSession::new(peer_id, 1, 0);
        manager.upsert(session.clone());
        
        let retrieved = manager.get(&peer_id).unwrap();
        assert_eq!(retrieved.peer_id, peer_id);
        
        manager.remove(&peer_id);
        assert!(manager.get(&peer_id).is_none());
    }
}
