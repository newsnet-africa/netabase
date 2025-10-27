//! Proof mechanisms for Sybil resistance
//!
//! This module provides Sybil attack resistance through:
//! - Proof-of-Work (computational puzzles)
//! - Challenge-response systems
//! - Reputation-based admission control
//! - Time-limited challenges with automatic expiry

use crate::sync::traits::SybilResistance;
use anyhow::{anyhow, Result};
use libp2p::PeerId;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Proof-of-Work configuration
#[derive(Clone, Debug)]
pub struct ProofOfWorkConfig {
    /// Difficulty (number of leading zero bits required)
    pub difficulty: u32,

    /// Enable PoW verification
    pub enabled: bool,
}

impl Default for ProofOfWorkConfig {
    fn default() -> Self {
        Self {
            difficulty: 4,
            enabled: false,
        }
    }
}

/// Proof-of-Work result
#[derive(Clone, Debug)]
pub struct ProofOfWork {
    /// Nonce that satisfies difficulty
    pub nonce: u64,

    /// Message hash
    pub hash: [u8; 32],

    /// Difficulty used
    pub difficulty: u32,
}

/// Proof-of-Work system
pub struct PoWSystem {
    config: ProofOfWorkConfig,
}

impl PoWSystem {
    /// Create a new PoW system
    pub fn new(config: ProofOfWorkConfig) -> Self {
        Self { config }
    }

    /// Generate proof-of-work for a message
    pub fn generate(&self, message: &[u8]) -> ProofOfWork {
        let mut nonce = 0u64;

        loop {
            let mut hasher = blake3::Hasher::new();
            hasher.update(message);
            hasher.update(&nonce.to_le_bytes());
            let hash: [u8; 32] = hasher.finalize().into();

            if self.verify_hash(&hash) {
                return ProofOfWork {
                    nonce,
                    hash,
                    difficulty: self.config.difficulty,
                };
            }

            nonce += 1;
        }
    }

    /// Verify a proof-of-work
    pub fn verify(&self, message: &[u8], proof: &ProofOfWork) -> bool {
        if proof.difficulty != self.config.difficulty {
            return false;
        }

        let mut hasher = blake3::Hasher::new();
        hasher.update(message);
        hasher.update(&proof.nonce.to_le_bytes());
        let hash: [u8; 32] = hasher.finalize().into();

        hash == proof.hash && self.verify_hash(&hash)
    }

    fn verify_hash(&self, hash: &[u8; 32]) -> bool {
        let leading_zeros = hash.iter().take_while(|&&b| b == 0).count() * 8;
        leading_zeros >= self.config.difficulty as usize
    }
}

impl SybilResistance for PoWSystem {
    fn verify_peer(&self, _peer_id: &PeerId) -> bool {
        // PoW verification is done per-message, not per-peer
        true
    }

    fn has_sufficient_stake(&self, _peer_id: &PeerId, _minimum: u64) -> bool {
        // Not applicable for PoW
        false
    }
}

/// Simple stake-based system (placeholder)
pub struct StakeSystem {
    /// Minimum stake required
    minimum_stake: u64,
}

impl StakeSystem {
    /// Create a new stake system
    pub fn new(minimum_stake: u64) -> Self {
        Self { minimum_stake }
    }
}

impl SybilResistance for StakeSystem {
    fn verify_peer(&self, peer_id: &PeerId) -> bool {
        // TODO: Implement actual stake verification
        // For now, accept all peers
        true
    }

    fn has_sufficient_stake(&self, _peer_id: &PeerId, minimum: u64) -> bool {
        // TODO: Check actual stake
        minimum <= self.minimum_stake
    }
}

/// Challenge-response system for Sybil resistance
pub struct ChallengeSystem {
    /// PoW system for challenges
    pow: PoWSystem,

    /// Active challenges: peer_id -> (challenge, expiry)
    challenges: HashMap<PeerId, (Vec<u8>, Instant)>,

    /// Challenge validity duration
    challenge_duration: Duration,

    /// Verified peers (passed challenge)
    verified_peers: HashMap<PeerId, Instant>,

    /// Verification validity duration
    verification_duration: Duration,
}

impl ChallengeSystem {
    /// Create a new challenge system
    pub fn new(config: ProofOfWorkConfig, challenge_duration: Duration, verification_duration: Duration) -> Self {
        Self {
            pow: PoWSystem::new(config),
            challenges: HashMap::new(),
            challenge_duration,
            verified_peers: HashMap::new(),
            verification_duration,
        }
    }

    /// Issue a challenge to a peer
    pub fn issue_challenge(&mut self, peer_id: PeerId) -> Vec<u8> {
        let challenge = self.generate_challenge(&peer_id);
        let expiry = Instant::now() + self.challenge_duration;
        self.challenges.insert(peer_id, (challenge.clone(), expiry));
        challenge
    }

    /// Verify a challenge response
    pub fn verify_response(&mut self, peer_id: &PeerId, proof: &ProofOfWork) -> Result<()> {
        // Get challenge
        let (challenge, expiry) = self.challenges.get(peer_id)
            .ok_or_else(|| anyhow!("No challenge found for peer"))?;

        // Check expiry
        if Instant::now() > *expiry {
            self.challenges.remove(peer_id);
            return Err(anyhow!("Challenge expired"));
        }

        // Verify PoW
        if !self.pow.verify(challenge, proof) {
            return Err(anyhow!("Invalid proof of work"));
        }

        // Mark peer as verified
        let verification_expiry = Instant::now() + self.verification_duration;
        self.verified_peers.insert(*peer_id, verification_expiry);
        self.challenges.remove(peer_id);

        Ok(())
    }

    /// Check if a peer is verified
    pub fn is_verified(&self, peer_id: &PeerId) -> bool {
        if let Some(expiry) = self.verified_peers.get(peer_id) {
            Instant::now() <= *expiry
        } else {
            false
        }
    }

    /// Clean up expired challenges and verifications
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();

        // Remove expired challenges
        self.challenges.retain(|_, (_, expiry)| now <= *expiry);

        // Remove expired verifications
        self.verified_peers.retain(|_, expiry| now <= *expiry);
    }

    /// Generate a unique challenge for a peer
    fn generate_challenge(&self, peer_id: &PeerId) -> Vec<u8> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&peer_id.to_bytes());
        hasher.update(&std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_le_bytes());
        hasher.finalize().as_bytes().to_vec()
    }

    /// Get number of verified peers
    pub fn verified_count(&self) -> usize {
        self.verified_peers.len()
    }

    /// Get number of active challenges
    pub fn challenge_count(&self) -> usize {
        self.challenges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow_generation_and_verification() {
        let config = ProofOfWorkConfig {
            difficulty: 4,
            enabled: true,
        };
        let pow_system = PoWSystem::new(config);

        let message = b"test message";
        let proof = pow_system.generate(message);

        assert!(pow_system.verify(message, &proof));
    }

    #[test]
    fn test_pow_wrong_message() {
        let config = ProofOfWorkConfig {
            difficulty: 4,
            enabled: true,
        };
        let pow_system = PoWSystem::new(config);

        let message = b"test message";
        let proof = pow_system.generate(message);

        let wrong_message = b"wrong message";
        assert!(!pow_system.verify(wrong_message, &proof));
    }

    #[test]
    fn test_pow_wrong_difficulty() {
        let config = ProofOfWorkConfig {
            difficulty: 4,
            enabled: true,
        };
        let pow_system = PoWSystem::new(config);

        let message = b"test message";
        let mut proof = pow_system.generate(message);

        // Change difficulty
        proof.difficulty = 8;

        assert!(!pow_system.verify(message, &proof));
    }

    #[test]
    fn test_pow_difficulty_levels() {
        let message = b"test message";

        // Test different difficulty levels
        for difficulty in [4, 8, 12] {
            let config = ProofOfWorkConfig {
                difficulty,
                enabled: true,
            };
            let pow_system = PoWSystem::new(config);
            let proof = pow_system.generate(message);

            assert_eq!(proof.difficulty, difficulty);
            assert!(pow_system.verify(message, &proof));
        }
    }

    #[test]
    fn test_challenge_system_issue_and_verify() {
        let config = ProofOfWorkConfig {
            difficulty: 4,
            enabled: true,
        };
        let mut challenge_system = ChallengeSystem::new(
            config,
            Duration::from_secs(60),
            Duration::from_secs(3600),
        );

        let peer = PeerId::random();

        // Issue challenge
        let challenge = challenge_system.issue_challenge(peer);
        assert!(!challenge.is_empty());
        assert_eq!(challenge_system.challenge_count(), 1);

        // Generate proof
        let pow_system = PoWSystem::new(ProofOfWorkConfig {
            difficulty: 4,
            enabled: true,
        });
        let proof = pow_system.generate(&challenge);

        // Verify response
        let result = challenge_system.verify_response(&peer, &proof);
        assert!(result.is_ok());
        assert!(challenge_system.is_verified(&peer));
        assert_eq!(challenge_system.verified_count(), 1);
        assert_eq!(challenge_system.challenge_count(), 0);
    }

    #[test]
    fn test_challenge_system_no_challenge() {
        let config = ProofOfWorkConfig {
            difficulty: 4,
            enabled: true,
        };
        let mut challenge_system = ChallengeSystem::new(
            config,
            Duration::from_secs(60),
            Duration::from_secs(3600),
        );

        let peer = PeerId::random();
        let proof = ProofOfWork {
            nonce: 0,
            hash: [0u8; 32],
            difficulty: 4,
        };

        // Try to verify without issuing challenge
        let result = challenge_system.verify_response(&peer, &proof);
        assert!(result.is_err());
    }

    #[test]
    fn test_challenge_system_wrong_proof() {
        let config = ProofOfWorkConfig {
            difficulty: 4,
            enabled: true,
        };
        let mut challenge_system = ChallengeSystem::new(
            config,
            Duration::from_secs(60),
            Duration::from_secs(3600),
        );

        let peer = PeerId::random();
        let _challenge = challenge_system.issue_challenge(peer);

        // Submit wrong proof
        let wrong_proof = ProofOfWork {
            nonce: 0,
            hash: [0u8; 32],
            difficulty: 4,
        };

        let result = challenge_system.verify_response(&peer, &wrong_proof);
        assert!(result.is_err());
        assert!(!challenge_system.is_verified(&peer));
    }

    #[test]
    fn test_challenge_system_expiry() {
        let config = ProofOfWorkConfig {
            difficulty: 4,
            enabled: true,
        };
        let mut challenge_system = ChallengeSystem::new(
            config,
            Duration::from_millis(100), // Short duration for testing
            Duration::from_secs(3600),
        );

        let peer = PeerId::random();
        challenge_system.issue_challenge(peer);
        assert_eq!(challenge_system.challenge_count(), 1);

        // Wait for expiry
        std::thread::sleep(Duration::from_millis(150));

        // Cleanup
        challenge_system.cleanup_expired();
        assert_eq!(challenge_system.challenge_count(), 0);
    }

    #[test]
    fn test_verification_expiry() {
        let config = ProofOfWorkConfig {
            difficulty: 4,
            enabled: true,
        };
        let mut challenge_system = ChallengeSystem::new(
            config,
            Duration::from_secs(60),
            Duration::from_millis(100), // Short verification duration
        );

        let peer = PeerId::random();
        let challenge = challenge_system.issue_challenge(peer);

        let pow_system = PoWSystem::new(ProofOfWorkConfig {
            difficulty: 4,
            enabled: true,
        });
        let proof = pow_system.generate(&challenge);

        challenge_system.verify_response(&peer, &proof).unwrap();
        assert!(challenge_system.is_verified(&peer));

        // Wait for verification to expire
        std::thread::sleep(Duration::from_millis(150));
        assert!(!challenge_system.is_verified(&peer));
    }

    #[test]
    fn test_multiple_peers() {
        let config = ProofOfWorkConfig {
            difficulty: 4,
            enabled: true,
        };
        let mut challenge_system = ChallengeSystem::new(
            config,
            Duration::from_secs(60),
            Duration::from_secs(3600),
        );

        let pow_system = PoWSystem::new(ProofOfWorkConfig {
            difficulty: 4,
            enabled: true,
        });

        // Issue challenges to multiple peers
        let peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
        for peer in &peers {
            challenge_system.issue_challenge(*peer);
        }
        assert_eq!(challenge_system.challenge_count(), 5);

        // Verify some peers
        for peer in &peers[0..3] {
            let challenge = challenge_system.challenges.get(peer).unwrap().0.clone();
            let proof = pow_system.generate(&challenge);
            challenge_system.verify_response(peer, &proof).unwrap();
        }

        assert_eq!(challenge_system.verified_count(), 3);
        assert_eq!(challenge_system.challenge_count(), 2);
    }
}
