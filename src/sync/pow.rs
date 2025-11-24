//! Proof-of-Work based Sybil resistance
//!
//! Prevents Sybil attacks by requiring peers to solve computational puzzles
//! before they can participate in the network.
//!
//! ## Challenge-Response Protocol
//! 1. New peer requests to join
//! 2. Network issues a PoW challenge (random nonce + difficulty)
//! 3. Peer must find a solution (hash with required leading zeros)
//! 4. Solution is verified and peer is admitted if valid
//! 5. Verification expires after configured duration

use crate::network::config::SybilResistanceConfig;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Proof-of-Work manager
#[derive(Clone)]
pub struct ProofOfWork {
    config: SybilResistanceConfig,
}

impl ProofOfWork {
    /// Create a new PoW instance
    pub fn new(config: SybilResistanceConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn config(&self) -> &SybilResistanceConfig {
        &self.config
    }

    /// Generate a new challenge for a peer
    pub fn generate_challenge(&self) -> PowChallenge {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let mut challenge = [0u8; 32];
        rng.fill(&mut challenge);

        PowChallenge {
            challenge: challenge.to_vec(),
            difficulty: self.config.pow_difficulty,
            issued_at: Instant::now(),
            max_duration: self.config.challenge_duration,
        }
    }

    /// Verify a PoW solution
    pub fn verify_solution(&self, solution: &PowSolution) -> bool {
        // Check if solution is expired
        if solution.solved_at.elapsed() > self.config.verification_validity {
            return false;
        }

        // Reconstruct the hash input
        let mut input = solution.challenge.clone();
        input.extend_from_slice(&solution.nonce.to_le_bytes());

        // Compute hash
        let hash = blake3::hash(&input);
        let hash_bytes = hash.as_bytes();

        // Check if hash meets difficulty requirement
        count_leading_zeros(hash_bytes) >= self.config.pow_difficulty
    }

    /// Solve a challenge (blocking operation - should be run in background)
    pub fn solve_challenge(&self, challenge: &PowChallenge) -> Option<PowSolution> {
        let start = Instant::now();

        for nonce in 0..u64::MAX {
            // Check timeout
            if start.elapsed() > challenge.max_duration {
                return None;
            }

            // Try this nonce
            let mut input = challenge.challenge.clone();
            input.extend_from_slice(&nonce.to_le_bytes());

            let hash = blake3::hash(&input);
            let hash_bytes = hash.as_bytes();

            if count_leading_zeros(hash_bytes) >= challenge.difficulty {
                return Some(PowSolution {
                    challenge: challenge.challenge.clone(),
                    nonce,
                    hash: hash_bytes.to_vec(),
                    solved_at: Instant::now(),
                });
            }

            // Check every 10000 iterations
            if nonce % 10000 == 0 && start.elapsed() > challenge.max_duration {
                return None;
            }
        }

        None
    }

    /// Estimate time to solve challenge based on difficulty
    pub fn estimate_solve_time(&self) -> Duration {
        // Very rough estimate: 2^difficulty hashes on average
        // Assuming ~1M hashes/sec on a typical CPU
        let hashes_needed = 2u64.pow(self.config.pow_difficulty);
        let hashes_per_sec = 1_000_000u64;
        let seconds = hashes_needed / hashes_per_sec;

        Duration::from_secs(seconds)
    }
}

/// A PoW challenge issued to a peer
#[derive(Debug, Clone)]
pub struct PowChallenge {
    /// Random challenge bytes
    pub challenge: Vec<u8>,
    /// Required difficulty (number of leading zero bits)
    pub difficulty: u32,
    /// When the challenge was issued
    pub issued_at: Instant,
    /// Maximum time allowed to solve
    pub max_duration: Duration,
}

/// A solved PoW challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowSolution {
    /// The original challenge
    pub challenge: Vec<u8>,
    /// The nonce that solves the challenge
    pub nonce: u64,
    /// The resulting hash
    pub hash: Vec<u8>,
    /// When the solution was found (not serialized, set to now on deserialization)
    #[serde(skip, default = "std::time::Instant::now")]
    pub solved_at: Instant,
}

impl PowSolution {
    /// Verify this solution meets the difficulty requirement
    pub fn verify(&self, difficulty: u32) -> bool {
        count_leading_zeros(&self.hash) >= difficulty
    }
}

/// Count leading zero bits in a byte array
fn count_leading_zeros(bytes: &[u8]) -> u32 {
    let mut count = 0;

    for byte in bytes {
        if *byte == 0 {
            count += 8;
        } else {
            count += byte.leading_zeros();
            break;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_leading_zeros() {
        assert_eq!(count_leading_zeros(&[0x00, 0xFF]), 8);
        assert_eq!(count_leading_zeros(&[0x00, 0x00, 0xFF]), 16);
        assert_eq!(count_leading_zeros(&[0x80]), 0);
        assert_eq!(count_leading_zeros(&[0x01]), 7);
        assert_eq!(count_leading_zeros(&[0x00, 0x01]), 15);
    }

    #[test]
    fn test_pow_creation() {
        let config = SybilResistanceConfig::default();
        let pow = ProofOfWork::new(config);

        assert_eq!(pow.config().pow_difficulty, 16);
    }

    #[test]
    fn test_challenge_generation() {
        let config = SybilResistanceConfig::default();
        let pow = ProofOfWork::new(config);

        let challenge1 = pow.generate_challenge();
        let challenge2 = pow.generate_challenge();

        // Challenges should be different
        assert_ne!(challenge1.challenge, challenge2.challenge);
        assert_eq!(challenge1.difficulty, challenge2.difficulty);
    }

    #[test]
    fn test_easy_challenge_solve() {
        let config = SybilResistanceConfig {
            enabled: true,
            pow_difficulty: 8, // Easy difficulty for testing
            challenge_duration: Duration::from_secs(10),
            verification_validity: Duration::from_secs(60),
        };
        let pow = ProofOfWork::new(config);

        let challenge = pow.generate_challenge();
        let solution = pow.solve_challenge(&challenge);

        assert!(solution.is_some());

        let solution = solution.unwrap();
        assert!(pow.verify_solution(&solution));
        assert!(solution.verify(8));
    }

    #[test]
    fn test_solve_timeout() {
        let config = SybilResistanceConfig {
            enabled: true,
            pow_difficulty: 30, // Very hard
            challenge_duration: Duration::from_millis(10), // Very short timeout
            verification_validity: Duration::from_secs(60),
        };
        let pow = ProofOfWork::new(config);

        let challenge = pow.generate_challenge();
        let solution = pow.solve_challenge(&challenge);

        // Should timeout and return None
        assert!(solution.is_none());
    }

    #[test]
    fn test_estimate_solve_time() {
        let config = SybilResistanceConfig {
            enabled: true,
            pow_difficulty: 20, // Higher difficulty to get >1 second estimate
            challenge_duration: Duration::from_secs(60),
            verification_validity: Duration::from_secs(3600),
        };
        let pow = ProofOfWork::new(config);

        let estimate = pow.estimate_solve_time();
        // For difficulty 20, should be around 1 second
        assert!(estimate.as_millis() > 0);
        assert!(estimate.as_secs() >= 1);
    }
}
