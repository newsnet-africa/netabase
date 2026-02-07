//! Handshake state machine
//!
//! Tracks the state of the handshake through its phases.

use libp2p::PeerId;

use super::{
    challenge::{Challenge, InterestReveal},
    keys::{EphemeralKeyPair, EphemeralPublicKey, StaticKeyPair, StaticPublicKey},
    session::{EncryptedChannel, SessionKey},
};

/// Connection state after handshake
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    /// Handshake not started
    Disconnected,
    /// Handshake in progress
    Handshaking,
    /// Handshake complete, channel established
    Connected,
    /// Connection closed
    Closed,
    /// Error during handshake
    Failed(String),
}

/// Handshake state machine (Initiator side)
#[derive(Clone)]
pub struct InitiatorHandshakeState {
    /// Our static key pair
    pub static_keys: StaticKeyPair,
    /// Our ephemeral key pair for this handshake
    pub ephemeral_keys: EphemeralKeyPair,
    /// Challenge we sent
    pub sent_challenge: Option<Challenge>,
    /// Peer's ephemeral public key
    pub peer_ephemeral: Option<EphemeralPublicKey>,
    /// Peer's static public key
    pub peer_static: Option<StaticPublicKey>,
    /// Current phase
    pub phase: HandshakePhase,
    /// Our interests (subscription hashes)
    pub our_interests: Vec<[u8; 32]>,
    /// Revealed mutual interests
    pub mutual_interests: Vec<InterestReveal>,
}

/// Handshake state machine (Responder side)
#[derive(Clone)]
pub struct ResponderHandshakeState {
    /// Our static key pair
    pub static_keys: StaticKeyPair,
    /// Our ephemeral key pair for this handshake
    pub ephemeral_keys: EphemeralKeyPair,
    /// Received challenge from initiator
    pub received_challenge: Option<Challenge>,
    /// Peer's ephemeral public key
    pub peer_ephemeral: Option<EphemeralPublicKey>,
    /// Peer's static public key
    pub peer_static: Option<StaticPublicKey>,
    /// Current phase
    pub phase: HandshakePhase,
    /// Our interests (subscription hashes)
    pub our_interests: Vec<[u8; 32]>,
    /// Revealed mutual interests
    pub mutual_interests: Vec<InterestReveal>,
}

/// Phases of the handshake
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakePhase {
    /// Initial state
    Start,
    /// Phase 1: Key exchange (Noise XX)
    KeyExchange,
    /// Phase 2: Private Area Intersection
    PaiExchange,
    /// Phase 3: Capability exchange
    CapabilityExchange,
    /// Handshake complete
    Complete,
    /// Handshake failed
    Failed,
}

/// Unified handshake state
#[derive(Clone)]
pub enum HandshakeState {
    Initiator(InitiatorHandshakeState),
    Responder(ResponderHandshakeState),
}

impl HandshakeState {
    /// Create a new initiator handshake
    pub fn new_initiator(static_keys: StaticKeyPair, our_interests: Vec<[u8; 32]>) -> Self {
        Self::Initiator(InitiatorHandshakeState {
            static_keys,
            ephemeral_keys: EphemeralKeyPair::generate(),
            sent_challenge: None,
            peer_ephemeral: None,
            peer_static: None,
            phase: HandshakePhase::Start,
            our_interests,
            mutual_interests: Vec::new(),
        })
    }

    /// Create a new responder handshake
    pub fn new_responder(static_keys: StaticKeyPair, our_interests: Vec<[u8; 32]>) -> Self {
        Self::Responder(ResponderHandshakeState {
            static_keys,
            ephemeral_keys: EphemeralKeyPair::generate(),
            received_challenge: None,
            peer_ephemeral: None,
            peer_static: None,
            phase: HandshakePhase::Start,
            our_interests,
            mutual_interests: Vec::new(),
        })
    }

    /// Get current phase
    pub fn phase(&self) -> HandshakePhase {
        match self {
            Self::Initiator(s) => s.phase,
            Self::Responder(s) => s.phase,
        }
    }

    /// Check if handshake is complete
    pub fn is_complete(&self) -> bool {
        self.phase() == HandshakePhase::Complete
    }

    /// Check if handshake failed
    pub fn is_failed(&self) -> bool {
        self.phase() == HandshakePhase::Failed
    }

    /// Get mutual interests after successful handshake
    pub fn mutual_interests(&self) -> &[InterestReveal] {
        match self {
            Self::Initiator(s) => &s.mutual_interests,
            Self::Responder(s) => &s.mutual_interests,
        }
    }
}

/// Established connection state after successful handshake
pub struct EstablishedConnection {
    /// Peer's identity
    pub peer_id: PeerId,
    /// Peer's static public key
    pub peer_static_key: StaticPublicKey,
    /// Encrypted channel for communication
    pub channel: EncryptedChannel,
    /// Mutual interests discovered during handshake
    pub mutual_interests: Vec<InterestReveal>,
    /// Connection timestamp
    pub established_at: u64,
}

impl EstablishedConnection {
    /// Create from completed handshake
    pub fn from_handshake(
        peer_id: PeerId,
        peer_static_key: StaticPublicKey,
        session_key: SessionKey,
        mutual_interests: Vec<InterestReveal>,
    ) -> Self {
        let established_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            peer_id,
            peer_static_key,
            channel: EncryptedChannel::new(session_key),
            mutual_interests,
            established_at,
        }
    }

    /// Check if peer has interest in a subscription
    pub fn has_interest_in(&self, subscription_hash: &[u8; 32]) -> bool {
        self.mutual_interests
            .iter()
            .any(|i| &i.subscription_hash == subscription_hash)
    }

    /// Get tables peer is interested in for a subscription
    pub fn interested_tables(&self, subscription_hash: &[u8; 32]) -> Option<&[[u8; 32]]> {
        self.mutual_interests
            .iter()
            .find(|i| &i.subscription_hash == subscription_hash)
            .map(|i| i.table_hashes.as_slice())
    }
}
