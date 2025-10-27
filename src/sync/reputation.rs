//! Reputation system for Byzantine peer filtering

use crate::sync::traits::ReputationSystem;
use libp2p::PeerId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Default reputation score for new peers
const DEFAULT_REPUTATION: f64 = 0.5;

/// Minimum reputation score
const MIN_REPUTATION: f64 = 0.0;

/// Maximum reputation score
const MAX_REPUTATION: f64 = 1.0;

/// Simple reputation system based on peer interactions
#[derive(Clone)]
pub struct SimpleReputationSystem {
    /// Reputation scores for each peer
    scores: Arc<RwLock<HashMap<PeerId, f64>>>,
}

impl SimpleReputationSystem {
    /// Create a new reputation system
    pub fn new() -> Self {
        Self {
            scores: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the number of tracked peers
    pub fn peer_count(&self) -> usize {
        self.scores.read().unwrap().len()
    }
}

impl Default for SimpleReputationSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ReputationSystem for SimpleReputationSystem {
    fn reputation(&self, peer_id: &PeerId) -> f64 {
        self.scores
            .read()
            .unwrap()
            .get(peer_id)
            .copied()
            .unwrap_or(DEFAULT_REPUTATION)
    }

    fn reward(&mut self, peer_id: &PeerId, amount: f64) {
        let mut scores = self.scores.write().unwrap();
        let score = scores.entry(peer_id.clone()).or_insert(DEFAULT_REPUTATION);
        *score = (*score + amount).min(MAX_REPUTATION);
    }

    fn penalize(&mut self, peer_id: &PeerId, amount: f64) {
        let mut scores = self.scores.write().unwrap();
        let score = scores.entry(peer_id.clone()).or_insert(DEFAULT_REPUTATION);
        *score = (*score - amount).max(MIN_REPUTATION);
    }

    fn top_peers(&self, n: usize) -> Vec<PeerId> {
        let scores = self.scores.read().unwrap();
        let mut peers: Vec<_> = scores.iter().collect();

        // Sort by reputation (descending)
        peers.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        peers.into_iter().take(n).map(|(peer, _)| peer.clone()).collect()
    }

    fn remove_peer(&mut self, peer_id: &PeerId) {
        self.scores.write().unwrap().remove(peer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_system() {
        let mut system = SimpleReputationSystem::new();
        let peer = PeerId::random();

        // Default reputation
        assert_eq!(system.reputation(&peer), DEFAULT_REPUTATION);

        // Reward
        system.reward(&peer, 0.3);
        assert_eq!(system.reputation(&peer), 0.8);

        // Penalize
        system.penalize(&peer, 0.2);
        assert_eq!(system.reputation(&peer), 0.6);
    }

    #[test]
    fn test_reputation_bounds() {
        let mut system = SimpleReputationSystem::new();
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
        let mut system = SimpleReputationSystem::new();
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
}
