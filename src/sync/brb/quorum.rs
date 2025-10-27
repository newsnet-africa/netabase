//! Quorum tracking for BRB echo/ready phases

use libp2p::PeerId;
use std::collections::HashSet;

/// Quorum tracker for Byzantine Reliable Broadcast
///
/// For n = 3f+1 nodes where f is the maximum number of Byzantine nodes:
/// - Echo threshold: (n+f)/2 + 1 (majority of honest + some Byzantine)
/// - Ready threshold: 2f+1 (ensures at least one honest node in intersection)
/// - Delivery threshold: 2f+1 ready messages
#[derive(Clone, Debug)]
pub struct QuorumTracker {
    /// Total number of peers in the system
    total_peers: usize,

    /// Maximum faulty nodes (f in 3f+1)
    max_faulty: usize,

    /// Peers who have responded (sent echo/ready)
    responders: HashSet<PeerId>,
}

impl QuorumTracker {
    /// Create a new quorum tracker
    ///
    /// # Arguments
    /// * `total_peers` - Total number of nodes (should be >= 3f+1)
    /// * `max_faulty` - Maximum Byzantine nodes to tolerate (f)
    pub fn new(total_peers: usize, max_faulty: usize) -> Self {
        // Validate configuration
        assert!(
            total_peers >= 3 * max_faulty + 1,
            "Need at least 3f+1 nodes to tolerate f Byzantine faults"
        );

        Self {
            total_peers,
            max_faulty,
            responders: HashSet::new(),
        }
    }

    /// Add a responder to the tracker
    /// Returns true if this is a new responder
    pub fn add_responder(&mut self, peer: PeerId) -> bool {
        self.responders.insert(peer)
    }

    /// Remove a responder (e.g., if peer disconnects)
    pub fn remove_responder(&mut self, peer: &PeerId) -> bool {
        self.responders.remove(peer)
    }

    /// Check if general quorum is reached (2f+1 responses)
    /// This is the standard quorum for BRB delivery
    pub fn has_quorum(&self) -> bool {
        let required = self.quorum_size();
        self.responders.len() >= required
    }

    /// Get the required quorum size (2f+1)
    pub fn quorum_size(&self) -> usize {
        2 * self.max_faulty + 1
    }

    /// Check if echo threshold is reached ((n+f)/2 + 1)
    /// This is the threshold for transitioning from ECHO to READY phase
    pub fn has_echo_threshold(&self) -> bool {
        let required = self.echo_threshold();
        self.responders.len() >= required
    }

    /// Get the echo threshold size ((n+f)/2 + 1)
    pub fn echo_threshold(&self) -> usize {
        (self.total_peers + self.max_faulty) / 2 + 1
    }

    /// Check if ready threshold is reached (2f+1)
    /// This is the threshold for delivering the message
    pub fn has_ready_threshold(&self) -> bool {
        let required = self.ready_threshold();
        self.responders.len() >= required
    }

    /// Get the ready threshold size (2f+1)
    pub fn ready_threshold(&self) -> usize {
        2 * self.max_faulty + 1
    }

    /// Get the number of responders
    pub fn responder_count(&self) -> usize {
        self.responders.len()
    }

    /// Get all responders
    pub fn responders(&self) -> &HashSet<PeerId> {
        &self.responders
    }

    /// Check if a peer has responded
    pub fn has_responded(&self, peer: &PeerId) -> bool {
        self.responders.contains(peer)
    }

    /// Clear all responders
    pub fn clear(&mut self) {
        self.responders.clear();
    }

    /// Update total peers (e.g., when network size changes)
    pub fn update_total_peers(&mut self, total_peers: usize) {
        assert!(
            total_peers >= 3 * self.max_faulty + 1,
            "Need at least 3f+1 nodes to tolerate f Byzantine faults"
        );
        self.total_peers = total_peers;
    }

    /// Get configuration info
    pub fn config(&self) -> QuorumConfig {
        QuorumConfig {
            total_peers: self.total_peers,
            max_faulty: self.max_faulty,
            quorum_size: self.quorum_size(),
            echo_threshold: self.echo_threshold(),
            ready_threshold: self.ready_threshold(),
        }
    }
}

/// Quorum configuration information
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuorumConfig {
    /// Total number of peers
    pub total_peers: usize,

    /// Maximum Byzantine faults tolerated
    pub max_faulty: usize,

    /// Quorum size (2f+1)
    pub quorum_size: usize,

    /// Echo threshold ((n+f)/2 + 1)
    pub echo_threshold: usize,

    /// Ready threshold (2f+1)
    pub ready_threshold: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quorum_tracker_creation() {
        // With f=1, need n=4
        let tracker = QuorumTracker::new(4, 1);
        assert_eq!(tracker.quorum_size(), 3); // 2f+1 = 3
        assert_eq!(tracker.echo_threshold(), 3); // (4+1)/2 + 1 = 3
        assert_eq!(tracker.ready_threshold(), 3); // 2f+1 = 3
    }

    #[test]
    #[should_panic(expected = "Need at least 3f+1 nodes")]
    fn test_invalid_configuration() {
        // With f=1, need at least 4 nodes, but only providing 3
        QuorumTracker::new(3, 1);
    }

    #[test]
    fn test_quorum_with_f1() {
        // f=1, n=4: need 2f+1 = 3 responses
        let mut tracker = QuorumTracker::new(4, 1);

        assert!(!tracker.has_quorum());

        tracker.add_responder(PeerId::random());
        assert!(!tracker.has_quorum()); // 1/3

        tracker.add_responder(PeerId::random());
        assert!(!tracker.has_quorum()); // 2/3

        tracker.add_responder(PeerId::random());
        assert!(tracker.has_quorum()); // 3/3 ✓
    }

    #[test]
    fn test_quorum_with_f2() {
        // f=2, n=7: need 2f+1 = 5 responses
        let mut tracker = QuorumTracker::new(7, 2);

        assert_eq!(tracker.quorum_size(), 5);

        for _ in 0..4 {
            tracker.add_responder(PeerId::random());
        }
        assert!(!tracker.has_quorum()); // 4/5

        tracker.add_responder(PeerId::random());
        assert!(tracker.has_quorum()); // 5/5 ✓
    }

    #[test]
    fn test_echo_threshold() {
        // f=1, n=4: echo threshold = (4+1)/2 + 1 = 3
        let mut tracker = QuorumTracker::new(4, 1);

        assert_eq!(tracker.echo_threshold(), 3);
        assert!(!tracker.has_echo_threshold());

        tracker.add_responder(PeerId::random());
        tracker.add_responder(PeerId::random());
        assert!(!tracker.has_echo_threshold()); // 2/3

        tracker.add_responder(PeerId::random());
        assert!(tracker.has_echo_threshold()); // 3/3 ✓
    }

    #[test]
    fn test_ready_threshold() {
        // f=2, n=7: ready threshold = 2f+1 = 5
        let mut tracker = QuorumTracker::new(7, 2);

        assert_eq!(tracker.ready_threshold(), 5);

        for _ in 0..4 {
            tracker.add_responder(PeerId::random());
        }
        assert!(!tracker.has_ready_threshold()); // 4/5

        tracker.add_responder(PeerId::random());
        assert!(tracker.has_ready_threshold()); // 5/5 ✓
    }

    #[test]
    fn test_duplicate_responders() {
        let mut tracker = QuorumTracker::new(4, 1);
        let peer = PeerId::random();

        assert!(tracker.add_responder(peer.clone())); // First time: true
        assert!(!tracker.add_responder(peer)); // Duplicate: false

        assert_eq!(tracker.responder_count(), 1);
    }

    #[test]
    fn test_remove_responder() {
        let mut tracker = QuorumTracker::new(4, 1);
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        tracker.add_responder(peer1.clone());
        tracker.add_responder(peer2.clone());
        assert_eq!(tracker.responder_count(), 2);

        assert!(tracker.remove_responder(&peer1));
        assert_eq!(tracker.responder_count(), 1);

        assert!(!tracker.remove_responder(&peer1)); // Already removed
    }

    #[test]
    fn test_has_responded() {
        let mut tracker = QuorumTracker::new(4, 1);
        let peer = PeerId::random();

        assert!(!tracker.has_responded(&peer));

        tracker.add_responder(peer.clone());
        assert!(tracker.has_responded(&peer));
    }

    #[test]
    fn test_clear() {
        let mut tracker = QuorumTracker::new(4, 1);

        tracker.add_responder(PeerId::random());
        tracker.add_responder(PeerId::random());
        assert_eq!(tracker.responder_count(), 2);

        tracker.clear();
        assert_eq!(tracker.responder_count(), 0);
    }

    #[test]
    fn test_config() {
        let tracker = QuorumTracker::new(7, 2);
        let config = tracker.config();

        assert_eq!(config.total_peers, 7);
        assert_eq!(config.max_faulty, 2);
        assert_eq!(config.quorum_size, 5);
        assert_eq!(config.echo_threshold, 5);
        assert_eq!(config.ready_threshold, 5);
    }

    #[test]
    fn test_update_total_peers() {
        let mut tracker = QuorumTracker::new(4, 1);
        assert_eq!(tracker.echo_threshold(), 3); // (4+1)/2 + 1 = 3

        tracker.update_total_peers(7);
        assert_eq!(tracker.echo_threshold(), 5); // (7+1)/2 + 1 = 5
    }
}
