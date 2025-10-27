//! Vector clock implementation for tracking causality in distributed systems

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::cmp::Ordering;

/// Logical timestamp for a single peer
pub type LogicalTimestamp = u64;

/// Vector clock for tracking causality across peers
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    /// Map from peer ID to logical timestamp
    #[serde(with = "crate::sync::serde_helper::peer_id_map")]
    clocks: HashMap<PeerId, LogicalTimestamp>,

    /// The local peer ID
    #[serde(serialize_with = "crate::sync::serde_helper::serialize_peer_id")]
    #[serde(deserialize_with = "crate::sync::serde_helper::deserialize_peer_id")]
    local_peer: PeerId,
}

impl VectorClock {
    /// Create a new vector clock for a peer
    pub fn new(local_peer: PeerId) -> Self {
        let mut clocks = HashMap::new();
        clocks.insert(local_peer.clone(), 0);

        Self {
            clocks,
            local_peer,
        }
    }

    /// Increment the local clock
    pub fn increment(&mut self) {
        let counter = self.clocks.entry(self.local_peer.clone()).or_insert(0);
        *counter += 1;
    }

    /// Get the local timestamp
    pub fn local_time(&self) -> LogicalTimestamp {
        *self.clocks.get(&self.local_peer).unwrap_or(&0)
    }

    /// Get timestamp for a specific peer
    pub fn get(&self, peer: &PeerId) -> LogicalTimestamp {
        *self.clocks.get(peer).unwrap_or(&0)
    }

    /// Set timestamp for a specific peer
    pub fn set(&mut self, peer: PeerId, time: LogicalTimestamp) {
        self.clocks.insert(peer, time);
    }

    /// Merge this clock with another (take max of each peer's timestamp)
    pub fn merge(&mut self, other: &VectorClock) {
        for (peer, &other_time) in &other.clocks {
            let local_time = self.clocks.entry(peer.clone()).or_insert(0);
            *local_time = (*local_time).max(other_time);
        }
    }

    /// Check if this clock happened before another
    /// Returns true if all timestamps in self are <= other and at least one is strictly less
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        let mut has_less = false;

        // Check all peers known to self
        for (peer, &self_time) in &self.clocks {
            let other_time = other.get(peer);

            if self_time > other_time {
                return false; // Found a timestamp that's greater, so not happened-before
            }

            if self_time < other_time {
                has_less = true;
            }
        }

        // Check peers only known to other
        for peer in other.clocks.keys() {
            if !self.clocks.contains_key(peer) {
                has_less = true;
            }
        }

        has_less
    }

    /// Check if this clock happened after another
    pub fn happened_after(&self, other: &VectorClock) -> bool {
        other.happened_before(self)
    }

    /// Check if this clock is concurrent with another (neither happened before the other)
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        !self.happened_before(other) && !other.happened_before(self)
    }

    /// Compare this clock with another
    pub fn compare(&self, other: &VectorClock) -> VectorClockOrdering {
        if self == other {
            VectorClockOrdering::Equal
        } else if self.happened_before(other) {
            VectorClockOrdering::Before
        } else if self.happened_after(other) {
            VectorClockOrdering::After
        } else {
            VectorClockOrdering::Concurrent
        }
    }

    /// Get all known peers
    pub fn peers(&self) -> Vec<PeerId> {
        self.clocks.keys().cloned().collect()
    }

    /// Get the number of peers in this clock
    pub fn peer_count(&self) -> usize {
        self.clocks.len()
    }

    /// Create a copy of this clock with incremented local time
    pub fn incremented(&self) -> Self {
        let mut new_clock = self.clone();
        new_clock.increment();
        new_clock
    }

    /// Update with a received event (merge and increment)
    pub fn update(&mut self, other: &VectorClock) {
        self.merge(other);
        self.increment();
    }

    /// Check if this clock is empty (only has local peer at 0)
    pub fn is_empty(&self) -> bool {
        self.clocks.len() == 1 && self.local_time() == 0
    }

    /// Convert to a compact string representation
    pub fn to_string(&self) -> String {
        let mut entries: Vec<_> = self.clocks.iter().collect();
        entries.sort_by_key(|(peer, _)| peer.to_string());

        let parts: Vec<String> = entries
            .iter()
            .map(|(peer, time)| format!("{}:{}", peer.to_string().chars().take(8).collect::<String>(), time))
            .collect();

        format!("[{}]", parts.join(", "))
    }
}

/// Ordering relationship between vector clocks
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VectorClockOrdering {
    /// Clocks are equal
    Equal,

    /// This clock happened before the other
    Before,

    /// This clock happened after the other
    After,

    /// Clocks are concurrent (neither happened before the other)
    Concurrent,
}

impl From<VectorClockOrdering> for Option<Ordering> {
    fn from(ordering: VectorClockOrdering) -> Self {
        match ordering {
            VectorClockOrdering::Equal => Some(Ordering::Equal),
            VectorClockOrdering::Before => Some(Ordering::Less),
            VectorClockOrdering::After => Some(Ordering::Greater),
            VectorClockOrdering::Concurrent => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_creation() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer.clone());

        assert_eq!(clock.local_time(), 0);
        assert_eq!(clock.peer_count(), 1);
    }

    #[test]
    fn test_increment() {
        let peer = PeerId::random();
        let mut clock = VectorClock::new(peer.clone());

        assert_eq!(clock.local_time(), 0);

        clock.increment();
        assert_eq!(clock.local_time(), 1);

        clock.increment();
        assert_eq!(clock.local_time(), 2);
    }

    #[test]
    fn test_merge() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let mut clock1 = VectorClock::new(peer1.clone());
        clock1.increment();
        clock1.increment(); // peer1: 2

        let mut clock2 = VectorClock::new(peer2.clone());
        clock2.increment(); // peer2: 1

        clock1.merge(&clock2);

        assert_eq!(clock1.get(&peer1), 2);
        assert_eq!(clock1.get(&peer2), 1);
    }

    #[test]
    fn test_happened_before() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let mut clock1 = VectorClock::new(peer1.clone());
        clock1.increment(); // peer1: 1

        let mut clock2 = VectorClock::new(peer1.clone());
        clock2.increment();
        clock2.increment(); // peer1: 2

        // clock1 happened before clock2
        assert!(clock1.happened_before(&clock2));
        assert!(!clock2.happened_before(&clock1));
    }

    #[test]
    fn test_concurrent_clocks() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let mut clock1 = VectorClock::new(peer1.clone());
        clock1.increment(); // peer1: 1, peer2: 0

        let mut clock2 = VectorClock::new(peer2.clone());
        clock2.increment(); // peer1: 0, peer2: 1

        // These clocks are concurrent
        assert!(clock1.is_concurrent(&clock2));
        assert!(clock2.is_concurrent(&clock1));
        assert!(!clock1.happened_before(&clock2));
        assert!(!clock2.happened_before(&clock1));
    }

    #[test]
    fn test_update() {
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        let mut clock1 = VectorClock::new(peer1.clone());
        clock1.increment(); // peer1: 1

        let mut clock2 = VectorClock::new(peer2.clone());
        clock2.increment();
        clock2.increment(); // peer2: 2

        // clock1 receives message from clock2
        clock1.update(&clock2);

        // After update: peer1: 2, peer2: 2 (merged then incremented peer1)
        assert_eq!(clock1.get(&peer1), 2);
        assert_eq!(clock1.get(&peer2), 2);
    }

    #[test]
    fn test_compare() {
        let peer = PeerId::random();

        let mut clock1 = VectorClock::new(peer.clone());
        let mut clock2 = VectorClock::new(peer.clone());

        // Initially equal
        assert_eq!(clock1.compare(&clock2), VectorClockOrdering::Equal);

        // clock2 advances
        clock2.increment();
        assert_eq!(clock1.compare(&clock2), VectorClockOrdering::Before);
        assert_eq!(clock2.compare(&clock1), VectorClockOrdering::After);
    }

    #[test]
    fn test_incremented() {
        let peer = PeerId::random();
        let clock1 = VectorClock::new(peer.clone());
        let clock2 = clock1.incremented();

        assert_eq!(clock1.local_time(), 0);
        assert_eq!(clock2.local_time(), 1);
    }

    #[test]
    fn test_complex_scenario() {
        // Simulate a distributed system with 3 peers
        let peer_a = PeerId::random();
        let peer_b = PeerId::random();
        let peer_c = PeerId::random();

        let mut clock_a = VectorClock::new(peer_a.clone());
        let mut clock_b = VectorClock::new(peer_b.clone());
        let mut clock_c = VectorClock::new(peer_c.clone());

        // A sends to B
        clock_a.increment();
        clock_b.update(&clock_a);

        // B sends to C
        clock_b.increment();
        clock_c.update(&clock_b);

        // Now clock_c happened after clock_a
        assert!(clock_a.happened_before(&clock_c));

        // A and B are concurrent (A sent before B updated)
        let clock_a_after_send = clock_a.clone();
        assert!(clock_a_after_send.is_concurrent(&clock_b));
    }
}
