//! Message validation for Byzantine Reliable Broadcast

use crate::sync::types::{MessageSignature, SyncMessage};
use anyhow::{Result, anyhow};
use blake3::Hasher;
use libp2p::PeerId;
use std::collections::HashMap;

/// BRB message validator for signature verification
pub struct BrbValidator {
    /// Known peer public keys (for signature verification)
    /// In production, this would be populated from a PKI or trust system
    peer_keys: HashMap<PeerId, Vec<u8>>,

    /// Require signatures for all messages
    require_signatures: bool,
}

impl BrbValidator {
    /// Create a new BRB validator
    pub fn new(require_signatures: bool) -> Self {
        Self {
            peer_keys: HashMap::new(),
            require_signatures,
        }
    }

    /// Register a peer's public key
    pub fn register_peer_key(&mut self, peer_id: PeerId, public_key: Vec<u8>) {
        self.peer_keys.insert(peer_id, public_key);
    }

    /// Validate a BRB message
    /// Note: Signature validation is TODO - currently validates sender only
    pub fn validate_message(&self, from: &PeerId, message: &SyncMessage) -> Result<()> {
        match message {
            SyncMessage::BrbInit {
                peer_id,
                payload,
                version: _,
            } => {
                // Verify sender matches claimed peer_id
                if from != peer_id {
                    return Err(anyhow!(
                        "Sender mismatch: from={}, claimed={}",
                        from,
                        peer_id
                    ));
                }

                // TODO: Add signature verification when MessageSignature is added to BrbInit
                // For now, we only validate sender identity
                if self.require_signatures {
                    // In production, verify signature here
                    // return Err(anyhow!("Signature verification not yet implemented"));
                }

                // Basic payload validation
                if payload.is_empty() {
                    return Err(anyhow!("Empty payload"));
                }

                Ok(())
            }
            SyncMessage::BrbEcho {
                peer_id,
                message_hash,
                original_sender: _,
            } => {
                if from != peer_id {
                    return Err(anyhow!("Sender mismatch"));
                }

                // Validate hash is not all zeros
                if *message_hash == [0u8; 32] {
                    return Err(anyhow!("Invalid message hash"));
                }

                Ok(())
            }
            SyncMessage::BrbReady {
                peer_id,
                message_hash,
                original_sender: _,
            } => {
                if from != peer_id {
                    return Err(anyhow!("Sender mismatch"));
                }

                // Validate hash is not all zeros
                if *message_hash == [0u8; 32] {
                    return Err(anyhow!("Invalid message hash"));
                }

                Ok(())
            }
            _ => {
                // Not a BRB message
                Err(anyhow!("Not a BRB message"))
            }
        }
    }

    /// Verify INIT message signature
    fn verify_init_signature(
        &self,
        peer_id: &PeerId,
        payload: &[u8],
        timestamp: u64,
        signature: &MessageSignature,
    ) -> Result<()> {
        // Compute message hash
        let mut hasher = Hasher::new();
        hasher.update(b"BRB-INIT");
        hasher.update(payload);
        hasher.update(&timestamp.to_le_bytes());
        let message_hash = hasher.finalize();

        // Verify signature
        self.verify_signature(peer_id, message_hash.as_bytes(), signature)
    }

    /// Verify ECHO message signature
    fn verify_echo_signature(
        &self,
        peer_id: &PeerId,
        message_hash: &[u8; 32],
        original_sender: &PeerId,
        signature: &MessageSignature,
    ) -> Result<()> {
        let mut hasher = Hasher::new();
        hasher.update(b"BRB-ECHO");
        hasher.update(message_hash);
        hasher.update(&original_sender.to_bytes());
        let hash = hasher.finalize();

        self.verify_signature(peer_id, hash.as_bytes(), signature)
    }

    /// Verify READY message signature
    fn verify_ready_signature(
        &self,
        peer_id: &PeerId,
        message_hash: &[u8; 32],
        original_sender: &PeerId,
        signature: &MessageSignature,
    ) -> Result<()> {
        let mut hasher = Hasher::new();
        hasher.update(b"BRB-READY");
        hasher.update(message_hash);
        hasher.update(&original_sender.to_bytes());
        let hash = hasher.finalize();

        self.verify_signature(peer_id, hash.as_bytes(), signature)
    }

    /// Verify DELIVER message signature
    fn verify_deliver_signature(
        &self,
        peer_id: &PeerId,
        message_hash: &[u8; 32],
        original_sender: &PeerId,
        signature: &MessageSignature,
    ) -> Result<()> {
        let mut hasher = Hasher::new();
        hasher.update(b"BRB-DELIVER");
        hasher.update(message_hash);
        hasher.update(&original_sender.to_bytes());
        let hash = hasher.finalize();

        self.verify_signature(peer_id, hash.as_bytes(), signature)
    }

    /// Verify a signature using peer's public key
    #[allow(unused_variables)]
    fn verify_signature(
        &self,
        peer_id: &PeerId,
        message_hash: &[u8],
        signature: &MessageSignature,
    ) -> Result<()> {
        // TODO: Implement actual signature verification with ed25519 or similar
        // For now, we'll do basic validation

        // Check if we have the peer's public key
        if let Some(_public_key) = self.peer_keys.get(peer_id) {
            // In production, verify signature using:
            // - ed25519_dalek for Ed25519
            // - secp256k1 for ECDSA
            // - or whatever signing algorithm is used

            // Placeholder: Accept if signature is non-empty and matches expected length
            if signature.signature.is_empty() {
                return Err(anyhow!("Empty signature"));
            }

            // Expected signature length for ed25519 is 64 bytes
            if signature.signature.len() != 64 {
                return Err(anyhow!(
                    "Invalid signature length: expected 64, got {}",
                    signature.signature.len()
                ));
            }

            Ok(())
        } else {
            // No public key registered - in permissionless mode, we might accept
            // In production, this should fail or use web-of-trust
            if self.require_signatures {
                Err(anyhow!("No public key registered for peer {}", peer_id))
            } else {
                Ok(())
            }
        }
    }

    /// Check if validator requires signatures
    pub fn requires_signatures(&self) -> bool {
        self.require_signatures
    }

    /// Get registered peer count
    pub fn registered_peer_count(&self) -> usize {
        self.peer_keys.len()
    }
}

impl Default for BrbValidator {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::clock::VectorClock;
    use crate::sync::types::Version;

    #[test]
    fn test_validator_creation() {
        let validator = BrbValidator::new(true);
        assert!(validator.requires_signatures());
        assert_eq!(validator.registered_peer_count(), 0);
    }

    #[test]
    fn test_register_peer_key() {
        let mut validator = BrbValidator::new(true);
        let peer = PeerId::random();
        let key = vec![1, 2, 3, 4];

        validator.register_peer_key(peer, key);
        assert_eq!(validator.registered_peer_count(), 1);
    }

    #[test]
    fn test_validate_init() {
        let validator = BrbValidator::new(false);
        let peer = PeerId::random();

        let clock = VectorClock::new(peer);
        let hash = blake3::hash(b"test");
        let version = Version::new(clock, hash.into());

        let message = SyncMessage::BrbInit {
            peer_id: peer,
            payload: vec![1, 2, 3],
            version,
        };

        // Should pass with valid payload
        assert!(validator.validate_message(&peer, &message).is_ok());
    }

    #[test]
    fn test_validate_init_empty_payload() {
        let validator = BrbValidator::new(false);
        let peer = PeerId::random();

        let clock = VectorClock::new(peer);
        let hash = blake3::hash(b"test");
        let version = Version::new(clock, hash.into());

        let message = SyncMessage::BrbInit {
            peer_id: peer,
            payload: vec![], // Empty payload
            version,
        };

        // Should fail with empty payload
        assert!(validator.validate_message(&peer, &message).is_err());
    }

    #[test]
    fn test_sender_mismatch() {
        let validator = BrbValidator::new(false);
        let sender = PeerId::random();
        let claimed = PeerId::random();

        let clock = VectorClock::new(claimed);
        let hash = blake3::hash(b"test");
        let version = Version::new(clock, hash.into());

        let message = SyncMessage::BrbInit {
            peer_id: claimed,
            payload: vec![1, 2, 3],
            version,
        };

        // Should fail when sender != claimed peer_id
        let result = validator.validate_message(&sender, &message);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Sender mismatch"));
    }

    #[test]
    fn test_validate_echo_message() {
        let validator = BrbValidator::new(false);
        let peer = PeerId::random();
        let original = PeerId::random();

        let message = SyncMessage::BrbEcho {
            peer_id: peer,
            message_hash: [1u8; 32], // Non-zero hash
            original_sender: original,
        };

        assert!(validator.validate_message(&peer, &message).is_ok());
    }

    #[test]
    fn test_validate_echo_invalid_hash() {
        let validator = BrbValidator::new(false);
        let peer = PeerId::random();
        let original = PeerId::random();

        let message = SyncMessage::BrbEcho {
            peer_id: peer,
            message_hash: [0u8; 32], // All zeros - invalid
            original_sender: original,
        };

        assert!(validator.validate_message(&peer, &message).is_err());
    }

    #[test]
    fn test_validate_ready_message() {
        let validator = BrbValidator::new(false);
        let peer = PeerId::random();
        let original = PeerId::random();

        let message = SyncMessage::BrbReady {
            peer_id: peer,
            message_hash: [1u8; 32], // Non-zero hash
            original_sender: original,
        };

        assert!(validator.validate_message(&peer, &message).is_ok());
    }

    #[test]
    fn test_non_brb_message() {
        let validator = BrbValidator::new(false);
        let peer = PeerId::random();

        let clock = VectorClock::new(peer);
        let digest = crate::sync::types::StateDigest::new([0u8; 32], 0, clock);

        let message = SyncMessage::SyncRequest {
            peer_id: peer,
            state_digest: digest,
        };

        // Should fail for non-BRB messages
        let result = validator.validate_message(&peer, &message);
        assert!(result.is_err());
    }
}
