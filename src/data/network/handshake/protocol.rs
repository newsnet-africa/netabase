//! Handshake protocol messages and state transitions
//!
//! Implements the two-phase handshake:
//! 1. Encrypted tunnel setup (Noise XX-style key exchange)
//! 2. Private Area Intersection (PAI) + capability exchange

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::data::network::capability::{EncodedCapability, OverlapAnnouncement, PaiFragment};

use super::{
    keys::{EphemeralPublicKey, StaticPublicKey},
    session::SessionKey,
    state::{HandshakePhase, HandshakeState, InitiatorHandshakeState, ResponderHandshakeState},
};

/// Handshake protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandshakeMessage {
    /// Phase 1: Initiator sends ephemeral public key
    Hello { ephemeral_key: EphemeralPublicKey },

    /// Phase 1: Responder sends ephemeral key + encrypted static key
    HelloResponse {
        ephemeral_key: EphemeralPublicKey,
        encrypted_static_key: Vec<u8>,
    },

    /// Phase 1: Initiator sends encrypted static key
    KeyExchangeComplete { encrypted_static_key: Vec<u8> },

    /// Phase 2: PAI fragments (salted hashes of interests)
    PaiFragments { fragments: Vec<PaiFragment> },

    /// Phase 2: Overlap announcement
    OverlapAnnounce { announcement: OverlapAnnouncement },

    /// Phase 3: Read capability for detected overlap
    ReadCapability {
        overlap_index: u32,
        capability: EncodedCapability,
    },

    /// Phase 3: Write capability proof
    WriteCapability {
        overlap_index: u32,
        capability: EncodedCapability,
        entry_signature: Vec<u8>,
    },

    /// Phase 3: Acknowledge received capabilities
    CapabilityAck {
        accepted: Vec<u32>,
        rejected: Vec<(u32, String)>,
    },

    /// Handshake complete
    Complete,

    /// Abort handshake
    Abort { reason: String },
}

/// Handshake errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeError {
    UnexpectedMessage(String),
    InvalidSignature,
    InvalidChallenge,
    ChallengeExpired,
    KeyExchangeFailed,
    DecryptionFailed,
    NoMutualInterests,
    CapabilityVerificationFailed(String),
    InvalidOverlapAnnouncement,
    ProtocolViolation(String),
    Timeout,
    PeerAborted(String),
}

impl fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedMessage(msg) => write!(f, "Unexpected message: {}", msg),
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::InvalidChallenge => write!(f, "Invalid challenge"),
            Self::ChallengeExpired => write!(f, "Challenge expired"),
            Self::KeyExchangeFailed => write!(f, "Key exchange failed"),
            Self::DecryptionFailed => write!(f, "Decryption failed"),
            Self::NoMutualInterests => write!(f, "No mutual interests found"),
            Self::CapabilityVerificationFailed(reason) => {
                write!(f, "Capability verification failed: {}", reason)
            }
            Self::InvalidOverlapAnnouncement => write!(f, "Invalid overlap announcement"),
            Self::ProtocolViolation(reason) => write!(f, "Protocol violation: {}", reason),
            Self::Timeout => write!(f, "Handshake timeout"),
            Self::PeerAborted(reason) => write!(f, "Peer aborted: {}", reason),
        }
    }
}

impl std::error::Error for HandshakeError {}

/// Process an incoming handshake message (initiator side)
pub fn process_initiator_message(
    state: &mut InitiatorHandshakeState,
    message: HandshakeMessage,
) -> Result<Option<HandshakeMessage>, HandshakeError> {
    match (&state.phase, message) {
        (
            HandshakePhase::KeyExchange,
            HandshakeMessage::HelloResponse {
                ephemeral_key,
                encrypted_static_key,
            },
        ) => {
            state.peer_ephemeral = Some(ephemeral_key.clone());
            let _ee_shared = state.ephemeral_keys.diffie_hellman(&ephemeral_key);

            let peer_static_bytes: [u8; 32] = encrypted_static_key
                .try_into()
                .map_err(|_| HandshakeError::DecryptionFailed)?;
            state.peer_static = Some(StaticPublicKey::new(peer_static_bytes));

            let our_static_encrypted = state.static_keys.public_key().as_bytes().to_vec();
            state.phase = HandshakePhase::PaiExchange;

            Ok(Some(HandshakeMessage::KeyExchangeComplete {
                encrypted_static_key: our_static_encrypted,
            }))
        }

        (HandshakePhase::PaiExchange, HandshakeMessage::PaiFragments { .. }) => {
            state.phase = HandshakePhase::CapabilityExchange;
            Ok(None)
        }

        (HandshakePhase::CapabilityExchange, HandshakeMessage::ReadCapability { .. }) => Ok(None),

        (
            HandshakePhase::CapabilityExchange,
            HandshakeMessage::CapabilityAck { accepted, rejected },
        ) => {
            if accepted.is_empty() && !rejected.is_empty() {
                return Err(HandshakeError::CapabilityVerificationFailed(format!(
                    "All capabilities rejected: {:?}",
                    rejected
                )));
            }
            state.phase = HandshakePhase::Complete;
            Ok(Some(HandshakeMessage::Complete))
        }

        (_, HandshakeMessage::Abort { reason }) => {
            state.phase = HandshakePhase::Failed;
            Err(HandshakeError::PeerAborted(reason))
        }

        (phase, msg) => Err(HandshakeError::UnexpectedMessage(format!(
            "Got {:?} in phase {:?}",
            std::mem::discriminant(&msg),
            phase
        ))),
    }
}

/// Process an incoming handshake message (responder side)
pub fn process_responder_message(
    state: &mut ResponderHandshakeState,
    message: HandshakeMessage,
) -> Result<Option<HandshakeMessage>, HandshakeError> {
    match (&state.phase, message) {
        (HandshakePhase::Start, HandshakeMessage::Hello { ephemeral_key }) => {
            state.peer_ephemeral = Some(ephemeral_key.clone());
            let _ee_shared = state.ephemeral_keys.diffie_hellman(&ephemeral_key);
            let our_static_encrypted = state.static_keys.public_key().as_bytes().to_vec();

            state.phase = HandshakePhase::KeyExchange;

            Ok(Some(HandshakeMessage::HelloResponse {
                ephemeral_key: state.ephemeral_keys.public_key().clone(),
                encrypted_static_key: our_static_encrypted,
            }))
        }

        (
            HandshakePhase::KeyExchange,
            HandshakeMessage::KeyExchangeComplete {
                encrypted_static_key,
            },
        ) => {
            let peer_static_bytes: [u8; 32] = encrypted_static_key
                .try_into()
                .map_err(|_| HandshakeError::DecryptionFailed)?;
            state.peer_static = Some(StaticPublicKey::new(peer_static_bytes));

            state.phase = HandshakePhase::PaiExchange;
            Ok(None)
        }

        (HandshakePhase::PaiExchange, HandshakeMessage::PaiFragments { .. }) => {
            state.phase = HandshakePhase::CapabilityExchange;
            Ok(None)
        }

        (HandshakePhase::CapabilityExchange, HandshakeMessage::ReadCapability { .. }) => Ok(None),

        (
            HandshakePhase::CapabilityExchange,
            HandshakeMessage::CapabilityAck { accepted, rejected },
        ) => {
            if accepted.is_empty() && !rejected.is_empty() {
                return Err(HandshakeError::CapabilityVerificationFailed(format!(
                    "All capabilities rejected: {:?}",
                    rejected
                )));
            }
            state.phase = HandshakePhase::Complete;
            Ok(Some(HandshakeMessage::Complete))
        }

        (_, HandshakeMessage::Abort { reason }) => {
            state.phase = HandshakePhase::Failed;
            Err(HandshakeError::PeerAborted(reason))
        }

        (phase, msg) => Err(HandshakeError::UnexpectedMessage(format!(
            "Got {:?} in phase {:?}",
            std::mem::discriminant(&msg),
            phase
        ))),
    }
}

/// Generate the initial Hello message (initiator)
pub fn generate_hello(state: &mut InitiatorHandshakeState) -> HandshakeMessage {
    state.phase = HandshakePhase::KeyExchange;
    HandshakeMessage::Hello {
        ephemeral_key: state.ephemeral_keys.public_key().clone(),
    }
}

/// Generate PAI fragments message
pub fn generate_pai_fragments(fragments: Vec<PaiFragment>) -> HandshakeMessage {
    HandshakeMessage::PaiFragments { fragments }
}

/// Derive session keys after successful key exchange
pub fn derive_session_keys(state: &HandshakeState) -> Result<SessionKey, HandshakeError> {
    match state {
        HandshakeState::Initiator(s) => {
            let peer_ephemeral = s
                .peer_ephemeral
                .as_ref()
                .ok_or(HandshakeError::KeyExchangeFailed)?;

            let shared = s.ephemeral_keys.diffie_hellman(peer_ephemeral);
            Ok(SessionKey::derive(&shared, None, true))
        }
        HandshakeState::Responder(s) => {
            let peer_ephemeral = s
                .peer_ephemeral
                .as_ref()
                .ok_or(HandshakeError::KeyExchangeFailed)?;

            let shared = s.ephemeral_keys.diffie_hellman(peer_ephemeral);
            Ok(SessionKey::derive(&shared, None, false))
        }
    }
}

/// Derive the random bytestring for PAI salt
pub fn derive_pai_rnd(state: &HandshakeState) -> Result<[u8; 32], HandshakeError> {
    match state {
        HandshakeState::Initiator(s) => {
            let peer_ephemeral = s
                .peer_ephemeral
                .as_ref()
                .ok_or(HandshakeError::KeyExchangeFailed)?;
            let shared = s.ephemeral_keys.diffie_hellman(peer_ephemeral);

            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(b"pai_rnd");
            hasher.update(shared.as_bytes());
            Ok(hasher.finalize().into())
        }
        HandshakeState::Responder(s) => {
            let peer_ephemeral = s
                .peer_ephemeral
                .as_ref()
                .ok_or(HandshakeError::KeyExchangeFailed)?;
            let shared = s.ephemeral_keys.diffie_hellman(peer_ephemeral);

            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(b"pai_rnd");
            hasher.update(shared.as_bytes());
            Ok(hasher.finalize().into())
        }
    }
}
