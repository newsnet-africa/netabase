//! Gossip protocol implementation for epidemic-style data propagation
//!
//! The gossip protocol provides eventual consistency by periodically
//! exchanging data with random peers. Each node:
//! 1. Periodically selects `fanout` random peers
//! 2. Sends them a digest of local data
//! 3. Receives digests from peers and requests missing data
//! 4. Converges to consistent state across the network

use crate::network::config::GossipConfig;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Gossip protocol manager
#[derive(Clone)]
pub struct GossipProtocol {
    config: GossipConfig,
    last_gossip: Option<Instant>,
    peers: HashSet<PeerId>,
    message_cache: HashMap<Vec<u8>, Instant>,
}

impl GossipProtocol {
    /// Create a new gossip protocol with the given configuration
    pub fn new(config: GossipConfig) -> Self {
        Self {
            config,
            last_gossip: None,
            peers: HashSet::new(),
            message_cache: HashMap::new(),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &GossipConfig {
        &self.config
    }

    /// Add a peer to the gossip network
    pub fn add_peer(&mut self, peer: PeerId) {
        self.peers.insert(peer);
    }

    /// Remove a peer from the gossip network
    pub fn remove_peer(&mut self, peer: &PeerId) {
        self.peers.remove(peer);
    }

    /// Get all known peers
    pub fn peers(&self) -> impl Iterator<Item = &PeerId> {
        self.peers.iter()
    }

    /// Get the number of peers
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Check if it's time to gossip
    pub fn should_gossip(&self) -> bool {
        if !self.config.enabled {
            return false;
        }

        match self.last_gossip {
            None => true,
            Some(last) => last.elapsed() >= self.config.interval,
        }
    }

    /// Select random peers for this gossip round
    pub fn select_peers(&mut self) -> Vec<PeerId> {
        use rand::seq::SliceRandom;

        self.last_gossip = Some(Instant::now());

        let mut peers: Vec<_> = self.peers.iter().cloned().collect();
        let mut rng = rand::thread_rng();
        peers.shuffle(&mut rng);

        peers.into_iter().take(self.config.fanout).collect()
    }

    /// Cache a message to prevent rebroadcasting
    pub fn cache_message(&mut self, message_id: Vec<u8>) {
        self.message_cache.insert(message_id, Instant::now());
        self.cleanup_old_messages();
    }

    /// Check if a message has been seen before
    pub fn has_seen_message(&self, message_id: &[u8]) -> bool {
        self.message_cache.contains_key(message_id)
    }

    /// Clean up old messages from cache (messages older than 10 minutes)
    fn cleanup_old_messages(&mut self) {
        let cutoff = Duration::from_secs(600);
        self.message_cache.retain(|_, timestamp| {
            timestamp.elapsed() < cutoff
        });
    }

    /// Get interval duration
    pub fn interval(&self) -> Duration {
        self.config.interval
    }

    /// Get fanout
    pub fn fanout(&self) -> usize {
        self.config.fanout
    }
}

/// Message types for the gossip protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    /// Digest of local data (hash -> timestamp)
    Digest {
        digests: Vec<(Vec<u8>, i64)>,
    },

    /// Request for specific data items
    Request {
        keys: Vec<Vec<u8>>,
    },

    /// Response with requested data
    Response {
        data: Vec<(Vec<u8>, Vec<u8>)>,
    },

    /// Direct data propagation
    Data {
        key: Vec<u8>,
        value: Vec<u8>,
        timestamp: i64,
    },
}

impl GossipMessage {
    /// Create a new digest message
    pub fn new_digest(digests: Vec<(Vec<u8>, i64)>) -> Self {
        Self::Digest { digests }
    }

    /// Create a new request message
    pub fn new_request(keys: Vec<Vec<u8>>) -> Self {
        Self::Request { keys }
    }

    /// Create a new response message
    pub fn new_response(data: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        Self::Response { data }
    }

    /// Create a new data message
    pub fn new_data(key: Vec<u8>, value: Vec<u8>, timestamp: i64) -> Self {
        Self::Data { key, value, timestamp }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gossip_protocol_creation() {
        let config = GossipConfig {
            enabled: true,
            interval: Duration::from_secs(10),
            fanout: 3,
        };

        let gossip = GossipProtocol::new(config);
        assert_eq!(gossip.peer_count(), 0);
        assert!(gossip.should_gossip());
    }

    #[test]
    fn test_peer_management() {
        let config = GossipConfig::default();
        let mut gossip = GossipProtocol::new(config);

        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        gossip.add_peer(peer1);
        gossip.add_peer(peer2);
        assert_eq!(gossip.peer_count(), 2);

        gossip.remove_peer(&peer1);
        assert_eq!(gossip.peer_count(), 1);
    }

    #[test]
    fn test_message_cache() {
        let config = GossipConfig::default();
        let mut gossip = GossipProtocol::new(config);

        let msg_id = vec![1, 2, 3, 4];
        assert!(!gossip.has_seen_message(&msg_id));

        gossip.cache_message(msg_id.clone());
        assert!(gossip.has_seen_message(&msg_id));
    }

    #[test]
    fn test_peer_selection() {
        let config = GossipConfig {
            enabled: true,
            interval: Duration::from_secs(10),
            fanout: 2,
        };
        let mut gossip = GossipProtocol::new(config);

        for _ in 0..5 {
            gossip.add_peer(PeerId::random());
        }

        let selected = gossip.select_peers();
        assert_eq!(selected.len(), 2);
    }
}
