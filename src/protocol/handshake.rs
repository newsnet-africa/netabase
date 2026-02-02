//! Handshake protocol state machine.
//!
//! This module implements the handshake phase of the Netabase protocol,
//! which establishes a connection between two peers, verifies compatibility,
//! and sets up a secure session.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::capabilities::CapabilitySignature;
use crate::network::protocol::{HandshakeRequest, HandshakeResponse, features};
use crate::primitives::{LamportClock, NodeId};

/// State of a handshake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeState {
    /// Initial state - not started
    Init,
    
    /// Waiting for response to our request
    RequestSent {
        nonce: u64,
        timestamp: u64,
    },
    
    /// Received a request, preparing response
    RequestReceived {
        request: HandshakeRequest,
    },
    
    /// Handshake complete - connection established
    Complete {
        peer_id: NodeId,
        protocol_version: u32,
        peer_features: u64,
    },
    
    /// Handshake failed
    Failed {
        reason: String,
    },
}

/// Handshake state machine.
pub struct HandshakeStateMachine {
    state: HandshakeState,
    local_id: NodeId,
    protocol_version: u32,
    features: u64,
    schema_hash: [u8; 32],
}

impl HandshakeStateMachine {
    /// Create a new handshake state machine.
    pub fn new(
        local_id: NodeId,
        protocol_version: u32,
        features: u64,
        schema_hash: [u8; 32],
    ) -> Self {
        Self {
            state: HandshakeState::Init,
            local_id,
            protocol_version,
            features,
            schema_hash,
        }
    }
    
    /// Get current state.
    pub fn state(&self) -> &HandshakeState {
        &self.state
    }
    
    /// Is handshake complete?
    pub fn is_complete(&self) -> bool {
        matches!(self.state, HandshakeState::Complete { .. })
    }
    
    /// Is handshake failed?
    pub fn is_failed(&self) -> bool {
        matches!(self.state, HandshakeState::Failed { .. })
    }
    
    /// Initiate handshake by creating a request.
    pub fn initiate(&mut self, clock: &mut LamportClock) -> HandshakeRequest {
        clock.tick();
        let nonce = clock.counter;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.state = HandshakeState::RequestSent { nonce, timestamp };
        
        HandshakeRequest {
            from: self.local_id,
            protocol_version: self.protocol_version,
            features: self.features,
            schema_hash: self.schema_hash,
            nonce,
            timestamp,
        }
    }
    
    /// Handle receiving a handshake request.
    pub fn handle_request(&mut self, request: HandshakeRequest) -> HandshakeResponse {
        // Validate protocol version
        let accepted = request.protocol_version == self.protocol_version
            && request.schema_hash == self.schema_hash;
        
        let reason = if request.protocol_version != self.protocol_version {
            Some("Incompatible protocol version".to_string())
        } else if request.schema_hash != self.schema_hash {
            Some("Incompatible schema".to_string())
        } else {
            None
        };
        
        if accepted {
            self.state = HandshakeState::Complete {
                peer_id: request.from,
                protocol_version: request.protocol_version,
                peer_features: request.features,
            };
        } else {
            self.state = HandshakeState::Failed {
                reason: reason.clone().unwrap_or_else(|| "Unknown reason".to_string()),
            };
        }
        
        // TODO: Actually sign the response
        let signature = CapabilitySignature([0u8; 64]);
        
        HandshakeResponse {
            from: self.local_id,
            protocol_version: self.protocol_version,
            accepted,
            reason,
            signature,
        }
    }
    
    /// Handle receiving a handshake response.
    pub fn handle_response(&mut self, response: HandshakeResponse) -> Result<(), String> {
        match &self.state {
            HandshakeState::RequestSent { .. } => {
                if response.accepted {
                    self.state = HandshakeState::Complete {
                        peer_id: response.from,
                        protocol_version: response.protocol_version,
                        peer_features: 0, // We don't get peer features in response
                    };
                    Ok(())
                } else {
                    let reason = response.reason.unwrap_or_else(|| "Rejected".to_string());
                    self.state = HandshakeState::Failed { reason: reason.clone() };
                    Err(reason)
                }
            }
            _ => Err("Invalid state for handling response".to_string()),
        }
    }
    
    /// Get the peer ID if handshake is complete.
    pub fn peer_id(&self) -> Option<NodeId> {
        match &self.state {
            HandshakeState::Complete { peer_id, .. } => Some(*peer_id),
            _ => None,
        }
    }
    
    /// Get peer features if handshake is complete.
    pub fn peer_features(&self) -> Option<u64> {
        match &self.state {
            HandshakeState::Complete { peer_features, .. } => Some(*peer_features),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_handshake_initiate() {
        let node_id = NodeId::from_bytes([1u8; 32]);
        let mut clock = LamportClock::new(0, [1u8; 8]);
        let mut hs = HandshakeStateMachine::new(node_id, 1, features::SYNC, [0u8; 32]);
        
        let request = hs.initiate(&mut clock);
        
        assert_eq!(request.from, node_id);
        assert_eq!(request.protocol_version, 1);
        assert!(matches!(hs.state(), HandshakeState::RequestSent { .. }));
    }
    
    #[test]
    fn test_handshake_accept() {
        let node1 = NodeId::from_bytes([1u8; 32]);
        let node2 = NodeId::from_bytes([2u8; 32]);
        
        let mut hs1 = HandshakeStateMachine::new(node1, 1, features::SYNC, [0u8; 32]);
        let mut hs2 = HandshakeStateMachine::new(node2, 1, features::SYNC, [0u8; 32]);
        
        let mut clock = LamportClock::new(0, [1u8; 8]);
        let request = hs1.initiate(&mut clock);
        
        let response = hs2.handle_request(request);
        assert!(response.accepted);
        assert!(hs2.is_complete());
        
        hs1.handle_response(response).unwrap();
        assert!(hs1.is_complete());
    }
    
    #[test]
    fn test_handshake_reject_version() {
        let node1 = NodeId::from_bytes([1u8; 32]);
        let node2 = NodeId::from_bytes([2u8; 32]);
        
        let mut hs1 = HandshakeStateMachine::new(node1, 1, features::SYNC, [0u8; 32]);
        let mut hs2 = HandshakeStateMachine::new(node2, 2, features::SYNC, [0u8; 32]);
        
        let mut clock = LamportClock::new(0, [1u8; 8]);
        let request = hs1.initiate(&mut clock);
        
        let response = hs2.handle_request(request);
        assert!(!response.accepted);
        assert!(hs2.is_failed());
    }
}
