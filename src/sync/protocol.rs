//! Sync protocol messages for libp2p request_response
//!
//! This module defines the message types and protocol for synchronization
//! using libp2p's request_response behavior.

use crate::sync::{
    VectorClock, ProofOfWork, PaxosMessage, StateDigest,
};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};

/// Request types for the sync protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncRequest {
    /// Request state digest from peer
    GetStateDigest,

    /// Request specific records by keys
    GetRecords {
        /// Collection name
        collection: String,
        /// Keys to fetch
        keys: Vec<Vec<u8>>,
    },

    /// Request all records since a version
    GetRecordsSince {
        /// Collection name
        collection: String,
        /// Vector clock of last known state
        since: VectorClock,
    },

    /// Request PoW challenge
    GetChallenge,

    /// Submit PoW proof
    SubmitProof {
        proof: ProofOfWork,
    },

    /// Byzantine Reliable Broadcast - Echo phase
    BrbEcho {
        /// Message ID
        message_id: Vec<u8>,
        /// Payload hash
        payload_hash: Vec<u8>,
        /// Sender's signature
        signature: Vec<u8>,
    },

    /// Byzantine Reliable Broadcast - Ready phase
    BrbReady {
        /// Message ID
        message_id: Vec<u8>,
        /// Payload hash
        payload_hash: Vec<u8>,
        /// Sender's signature
        signature: Vec<u8>,
    },

    /// Paxos message
    Paxos {
        message: PaxosMessage,
    },
}

/// Response types for the sync protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncResponse {
    /// State digest response
    StateDigest {
        digest: StateDigest,
        vector_clock: VectorClock,
    },

    /// Records response
    Records {
        /// Collection name
        collection: String,
        /// Records as serialized bytes
        records: Vec<SyncRecord>,
    },

    /// PoW challenge response
    Challenge {
        /// Challenge data
        challenge: Vec<u8>,
        /// Challenge ID for tracking
        challenge_id: Vec<u8>,
    },

    /// PoW proof verification result
    ProofVerified {
        /// Whether proof is valid
        valid: bool,
        /// Verification duration remaining
        duration_secs: u64,
    },

    /// BRB Echo acknowledgment
    BrbEchoAck {
        message_id: Vec<u8>,
    },

    /// BRB Ready acknowledgment
    BrbReadyAck {
        message_id: Vec<u8>,
    },

    /// Paxos response
    Paxos {
        message: PaxosMessage,
    },

    /// Error response
    Error {
        message: String,
    },
}

/// A synchronized record with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    /// Record key
    pub key: Vec<u8>,
    /// Record value (serialized model)
    pub value: Vec<u8>,
    /// Version/timestamp
    pub version: u64,
    /// Peer that created/updated this record
    pub peer_id: Vec<u8>,
    /// Vector clock at time of update
    pub vector_clock: VectorClock,
}

impl SyncRecord {
    /// Create a new sync record
    pub fn new(
        key: Vec<u8>,
        value: Vec<u8>,
        version: u64,
        peer_id: PeerId,
        vector_clock: VectorClock,
    ) -> Self {
        Self {
            key,
            value,
            version,
            peer_id: peer_id.to_bytes(),
            vector_clock,
        }
    }

    /// Get the peer ID
    pub fn peer_id(&self) -> Result<PeerId, String> {
        PeerId::from_bytes(&self.peer_id)
            .map_err(|e| format!("Invalid peer ID: {}", e))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_record_creation() {
        let peer_id = PeerId::random();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();
        let version = 42;
        let clock = VectorClock::new(peer_id);

        let record = SyncRecord::new(
            key.clone(),
            value.clone(),
            version,
            peer_id,
            clock.clone(),
        );

        assert_eq!(record.key, key);
        assert_eq!(record.value, value);
        assert_eq!(record.version, version);
        assert_eq!(record.peer_id().unwrap(), peer_id);
        assert_eq!(record.vector_clock, clock);
    }

    #[test]
    fn test_request_serialization() {
        let request = SyncRequest::GetStateDigest;
        let serialized = serde_json::to_vec(&request).unwrap();
        let deserialized: SyncRequest = serde_json::from_slice(&serialized).unwrap();

        match deserialized {
            SyncRequest::GetStateDigest => {},
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let response = SyncResponse::Error {
            message: "test error".to_string(),
        };
        let serialized = serde_json::to_vec(&response).unwrap();
        let deserialized: SyncResponse = serde_json::from_slice(&serialized).unwrap();

        match deserialized {
            SyncResponse::Error { message } => {
                assert_eq!(message, "test error");
            },
            _ => panic!("Wrong variant"),
        }
    }
}
