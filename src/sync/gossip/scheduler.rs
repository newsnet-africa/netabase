//! Gossip scheduler for periodic state exchange

use libp2p::PeerId;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tokio::time::interval;

/// Gossip round scheduler
pub struct GossipScheduler {
    /// Gossip interval
    interval: Duration,

    /// Number of peers to gossip with per round
    fanout: usize,

    /// Last gossip time
    last_gossip: Option<Instant>,

    /// Peers recently gossiped with (to avoid immediate re-gossip)
    recent_peers: HashSet<PeerId>,

    /// Maximum size of recent peers cache
    max_recent: usize,
}

impl GossipScheduler {
    /// Create a new gossip scheduler
    pub fn new(interval: Duration, fanout: usize) -> Self {
        Self {
            interval,
            fanout,
            last_gossip: None,
            recent_peers: HashSet::new(),
            max_recent: fanout * 3, // Keep last 3 rounds of peers
        }
    }

    /// Check if it's time for a gossip round
    pub fn should_gossip(&self) -> bool {
        match self.last_gossip {
            None => true,
            Some(last) => last.elapsed() >= self.interval,
        }
    }

    /// Select peers for gossip, excluding recently gossiped peers
    pub fn select_peers(&mut self, available_peers: &[PeerId]) -> Vec<PeerId> {
        // Filter out recently gossiped peers
        let candidates: Vec<_> = available_peers
            .iter()
            .filter(|peer| !self.recent_peers.contains(peer))
            .copied()
            .collect();

        // If we don't have enough candidates, allow re-gossip
        let peers_to_select = if candidates.len() < self.fanout {
            available_peers
        } else {
            &candidates
        };

        // Random selection
        let selected = self.random_selection(peers_to_select, self.fanout);

        // Update recent peers
        for peer in &selected {
            self.recent_peers.insert(*peer);
        }

        // Trim recent peers cache
        if self.recent_peers.len() > self.max_recent {
            // Remove oldest entries (randomly, since HashSet doesn't preserve order)
            let to_remove = self.recent_peers.len() - self.max_recent;
            let peers_to_remove: Vec<_> =
                self.recent_peers.iter().take(to_remove).cloned().collect();
            for peer in peers_to_remove {
                self.recent_peers.remove(&peer);
            }
        }

        self.last_gossip = Some(Instant::now());
        selected
    }

    /// Random peer selection using Fisher-Yates shuffle
    fn random_selection(&self, peers: &[PeerId], count: usize) -> Vec<PeerId> {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut rng = thread_rng();
        let mut selected = peers.to_vec();
        selected.shuffle(&mut rng);
        selected.into_iter().take(count).collect()
    }

    /// Reset the scheduler
    pub fn reset(&mut self) {
        self.last_gossip = None;
        self.recent_peers.clear();
    }

    /// Get the gossip interval
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Set the gossip interval
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    /// Get the fanout
    pub fn fanout(&self) -> usize {
        self.fanout
    }

    /// Set the fanout
    pub fn set_fanout(&mut self, fanout: usize) {
        self.fanout = fanout;
        self.max_recent = fanout * 3;
    }

    /// Create a tokio interval for the gossip rounds
    pub fn create_interval(&self) -> tokio::time::Interval {
        interval(self.interval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = GossipScheduler::new(Duration::from_secs(10), 3);
        assert_eq!(scheduler.interval(), Duration::from_secs(10));
        assert_eq!(scheduler.fanout(), 3);
        assert!(scheduler.should_gossip());
    }

    #[test]
    fn test_peer_selection() {
        let mut scheduler = GossipScheduler::new(Duration::from_secs(10), 2);
        let peers = vec![
            PeerId::random(),
            PeerId::random(),
            PeerId::random(),
            PeerId::random(),
        ];

        let selected = scheduler.select_peers(&peers);
        assert_eq!(selected.len(), 2);

        // Selected peers should be from available peers
        for peer in &selected {
            assert!(peers.contains(peer));
        }
    }

    #[test]
    fn test_recent_peers_tracking() {
        let mut scheduler = GossipScheduler::new(Duration::from_secs(10), 2);
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        let peers = vec![peer1, peer2, peer3];

        // First selection
        let _selected1 = scheduler.select_peers(&peers);
        assert!(!scheduler.recent_peers.is_empty());

        // Recent peers should be tracked
        assert!(scheduler.recent_peers.len() <= 2);
    }

    #[tokio::test]
    async fn test_should_gossip_timing() {
        let mut scheduler = GossipScheduler::new(Duration::from_millis(100), 2);

        assert!(scheduler.should_gossip());

        let peers = vec![PeerId::random(), PeerId::random()];
        scheduler.select_peers(&peers);

        // Immediately after, should not gossip
        assert!(!scheduler.should_gossip());

        // After interval, should gossip
        tokio::time::sleep(Duration::from_millis(110)).await;
        assert!(scheduler.should_gossip());
    }
}
