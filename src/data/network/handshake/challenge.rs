//! Challenge-response authentication
//!
//! Mutual authentication using signed challenges to prevent
//! man-in-the-middle attacks and replay attacks.

use serde::{Deserialize, Serialize};

use super::keys::{StaticPublicKey, StaticSignature};

/// Cryptographic challenge for authentication
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Challenge {
    /// Random nonce
    pub nonce: [u8; 32],
    /// Timestamp to prevent replay attacks
    pub timestamp: u64,
    /// Commitment to the sender's interests (hashed)
    pub interest_commitment: [u8; 32],
}

impl Challenge {
    /// Generate a new random challenge
    pub fn generate(interest_commitment: [u8; 32]) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // TODO: Use proper RNG
        let nonce = [0u8; 32];

        Self {
            nonce,
            timestamp,
            interest_commitment,
        }
    }

    /// Get the message to sign for this challenge
    pub fn to_sign_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(72);
        bytes.extend_from_slice(&self.nonce);
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.interest_commitment);
        bytes
    }

    /// Check if the challenge is fresh (not expired)
    pub fn is_fresh(&self, max_age_secs: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now.saturating_sub(self.timestamp) <= max_age_secs
    }
}

/// Response to a challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    /// The original challenge
    pub challenge: Challenge,
    /// Signature over the challenge
    pub signature: StaticSignature,
    /// The responder's static public key
    pub responder_key: StaticPublicKey,
    /// The responder's revealed interests (after verification)
    pub revealed_interests: Option<Vec<InterestReveal>>,
}

impl ChallengeResponse {
    /// Create a new challenge response
    pub fn new(
        challenge: Challenge,
        signature: StaticSignature,
        responder_key: StaticPublicKey,
    ) -> Self {
        Self {
            challenge,
            signature,
            responder_key,
            revealed_interests: None,
        }
    }

    /// Verify the signature on this response
    pub fn verify(&self) -> bool {
        let message = self.challenge.to_sign_bytes();
        self.responder_key.verify(&message, &self.signature)
    }

    /// Add revealed interests after mutual verification
    pub fn with_interests(mut self, interests: Vec<InterestReveal>) -> Self {
        self.revealed_interests = Some(interests);
        self
    }
}

/// Revealed interest after private area intersection succeeds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestReveal {
    /// Hash of the subscription this interest is for
    pub subscription_hash: [u8; 32],
    /// Tables within the subscription we're interested in
    pub table_hashes: Vec<[u8; 32]>,
    /// Optional capability proof for write access
    pub capability_proof: Option<Vec<u8>>,
}
