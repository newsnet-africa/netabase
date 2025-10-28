//! Reputation system for Byzantine peer filtering

use crate::sync::traits::ReputationSystem;
use libp2p::PeerId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Default reputation score for new peers
const DEFAULT_REPUTATION: f64 = 0.5;

/// Minimum reputation score
const MIN_REPUTATION: f64 = 0.0;

/// Maximum reputation score
const MAX_REPUTATION: f64 = 1.0;

/// Decay rate per hour (reputation decays toward default)
const DECAY_RATE: f64 = 0.1;

/// Peer reputation data
#[derive(Clone, Debug)]
struct PeerReputation {
    /// Current reputation score
    score: f64,

    /// Last update timestamp
    last_updated: Instant,

    /// Total successful interactions
    successful_interactions: u64,

    /// Total failed interactions
    failed_interactions: u64,
}

impl PeerReputation {
    fn new() -> Self {
        Self {
            score: DEFAULT_REPUTATION,
            last_updated: Instant::now(),
            successful_interactions: 0,
            failed_interactions: 0,
        }
    }

    /// Apply time-based decay toward default reputation
    fn apply_decay(&mut self) {
        let elapsed = self.last_updated.elapsed();
        let hours = elapsed.as_secs_f64() / 3600.0;
        let decay_amount = hours * DECAY_RATE;

        // Decay toward DEFAULT_REPUTATION
        if self.score > DEFAULT_REPUTATION {
            self.score = (self.score - decay_amount).max(DEFAULT_REPUTATION);
        } else if self.score < DEFAULT_REPUTATION {
            self.score = (self.score + decay_amount).min(DEFAULT_REPUTATION);
        }

        self.last_updated = Instant::now();
    }
}

/// Enhanced reputation system based on peer interactions
#[derive(Clone)]
pub struct SimpleReputationSystem {
    /// Reputation data for each peer
    reputations: Arc<RwLock<HashMap<PeerId, PeerReputation>>>,

    /// Decay configuration
    decay_enabled: bool,
}

impl SimpleReputationSystem {
    /// Create a new reputation system with decay enabled
    pub fn new() -> Self {
        Self::with_decay(true)
    }

    /// Create a new reputation system with configurable decay
    pub fn with_decay(decay_enabled: bool) -> Self {
        Self {
            reputations: Arc::new(RwLock::new(HashMap::new())),
            decay_enabled,
        }
    }

    /// Get the number of tracked peers
    pub fn peer_count(&self) -> usize {
        self.reputations.read().unwrap().len()
    }

    /// Record a successful interaction with a peer
    pub fn record_success(&mut self, peer_id: &PeerId) {
        let mut reputations = self.reputations.write().unwrap();
        let rep = reputations.entry(peer_id.clone()).or_insert_with(PeerReputation::new);

        if self.decay_enabled {
            rep.apply_decay();
        }

        rep.successful_interactions += 1;
        // Smaller rewards for continued good behavior (diminishing returns)
        let reward = 0.1 / (1.0 + (rep.successful_interactions as f64 * 0.01));
        rep.score = (rep.score + reward).min(MAX_REPUTATION);
        rep.last_updated = Instant::now();
    }

    /// Record a failed interaction with a peer
    pub fn record_failure(&mut self, peer_id: &PeerId) {
        let mut reputations = self.reputations.write().unwrap();
        let rep = reputations.entry(peer_id.clone()).or_insert_with(PeerReputation::new);

        if self.decay_enabled {
            rep.apply_decay();
        }

        rep.failed_interactions += 1;
        // Larger penalties for failures
        let penalty = 0.2;
        rep.score = (rep.score - penalty).max(MIN_REPUTATION);
        rep.last_updated = Instant::now();
    }

    /// Get interaction statistics for a peer
    pub fn get_stats(&self, peer_id: &PeerId) -> Option<(u64, u64)> {
        let reputations = self.reputations.read().unwrap();
        reputations.get(peer_id).map(|rep| {
            (rep.successful_interactions, rep.failed_interactions)
        })
    }

    /// Apply decay to all peer reputations
    pub fn apply_decay_all(&mut self) {
        if !self.decay_enabled {
            return;
        }

        let mut reputations = self.reputations.write().unwrap();
        for rep in reputations.values_mut() {
            rep.apply_decay();
        }
    }
}

impl Default for SimpleReputationSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ReputationSystem for SimpleReputationSystem {
    fn reputation(&self, peer_id: &PeerId) -> f64 {
        let mut reputations = self.reputations.write().unwrap();

        if let Some(rep) = reputations.get_mut(peer_id) {
            if self.decay_enabled {
                rep.apply_decay();
            }
            rep.score
        } else {
            DEFAULT_REPUTATION
        }
    }

    fn reward(&mut self, peer_id: &PeerId, amount: f64) {
        let mut reputations = self.reputations.write().unwrap();
        let rep = reputations.entry(peer_id.clone()).or_insert_with(PeerReputation::new);

        if self.decay_enabled {
            rep.apply_decay();
        }

        rep.score = (rep.score + amount).min(MAX_REPUTATION);
        rep.last_updated = Instant::now();
    }

    fn penalize(&mut self, peer_id: &PeerId, amount: f64) {
        let mut reputations = self.reputations.write().unwrap();
        let rep = reputations.entry(peer_id.clone()).or_insert_with(PeerReputation::new);

        if self.decay_enabled {
            rep.apply_decay();
        }

        rep.score = (rep.score - amount).max(MIN_REPUTATION);
        rep.last_updated = Instant::now();
    }

    fn top_peers(&self, n: usize) -> Vec<PeerId> {
        let reputations = self.reputations.read().unwrap();
        let mut peers: Vec<_> = reputations.iter()
            .map(|(peer, rep)| (peer.clone(), rep.score))
            .collect();

        // Sort by reputation (descending)
        peers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        peers.into_iter().take(n).map(|(peer, _)| peer).collect()
    }

    fn remove_peer(&mut self, peer_id: &PeerId) {
        self.reputations.write().unwrap().remove(peer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_system() {
        let mut system = SimpleReputationSystem::with_decay(false);
        let peer = PeerId::random();

        // Default reputation
        assert_eq!(system.reputation(&peer), DEFAULT_REPUTATION);

        // Reward
        system.reward(&peer, 0.3);
        assert!((system.reputation(&peer) - 0.8).abs() < 0.001);

        // Penalize
        system.penalize(&peer, 0.2);
        assert!((system.reputation(&peer) - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_reputation_bounds() {
        let mut system = SimpleReputationSystem::with_decay(false);
        let peer = PeerId::random();

        // Max bound
        system.reward(&peer, 1.0);
        assert_eq!(system.reputation(&peer), MAX_REPUTATION);

        // Min bound
        system.penalize(&peer, 2.0);
        assert_eq!(system.reputation(&peer), MIN_REPUTATION);
    }

    #[test]
    fn test_top_peers() {
        let mut system = SimpleReputationSystem::with_decay(false);
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        system.reward(&peer1, 0.1); // 0.6
        system.reward(&peer2, 0.3); // 0.8
        system.penalize(&peer3, 0.2); // 0.3

        let top = system.top_peers(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0], peer2); // Highest reputation
    }

    #[test]
    fn test_record_success_failure() {
        let mut system = SimpleReputationSystem::with_decay(false);
        let peer = PeerId::random();

        // Record successes
        system.record_success(&peer);
        system.record_success(&peer);

        let stats = system.get_stats(&peer).unwrap();
        assert_eq!(stats.0, 2); // 2 successful interactions
        assert_eq!(stats.1, 0); // 0 failed interactions

        // Record failures
        system.record_failure(&peer);

        let stats = system.get_stats(&peer).unwrap();
        assert_eq!(stats.0, 2);
        assert_eq!(stats.1, 1);
    }

    #[test]
    fn test_diminishing_returns() {
        let mut system = SimpleReputationSystem::with_decay(false);
        let peer = PeerId::random();

        let initial_rep = system.reputation(&peer);

        // First success gives bigger boost
        system.record_success(&peer);
        let rep_after_first = system.reputation(&peer);
        let first_boost = rep_after_first - initial_rep;

        // Many more successes
        for _ in 0..100 {
            system.record_success(&peer);
        }
        let rep_after_many = system.reputation(&peer);
        let last_boost = rep_after_many - rep_after_first;

        // Average boost per interaction should be smaller after many interactions
        assert!(last_boost / 100.0 < first_boost);
    }

    #[test]
    fn test_decay_disabled() {
        let mut system = SimpleReputationSystem::with_decay(false);
        let peer = PeerId::random();

        system.reward(&peer, 0.3);
        let rep_before = system.reputation(&peer);

        // Sleep would be tested in integration tests
        // Here we just verify decay doesn't happen when disabled
        system.apply_decay_all();

        let rep_after = system.reputation(&peer);
        assert_eq!(rep_before, rep_after);
    }
}
