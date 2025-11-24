//! Byzantine Reliable Broadcast implementation
//!
//! BRB ensures that all correct nodes deliver the same set of messages
//! even in the presence of Byzantine (malicious) nodes.
//!
//! ## Properties
//! - **Validity**: If a correct node broadcasts m, then all correct nodes deliver m
//! - **Agreement**: If a correct node delivers m, then all correct nodes deliver m
//! - **Integrity**: Every correct node delivers m at most once, and only if m was broadcast
//!
//! ## Protocol
//! 1. Sender broadcasts ECHO messages to all nodes
//! 2. Upon receiving ECHO, node broadcasts READY
//! 3. Upon receiving (f+1) READY, broadcast READY if not already done
//! 4. Upon receiving (2f+1) READY, deliver the message
//!
//! This tolerates up to f Byzantine faults in a system of n >= 2f+1 nodes.

use crate::network::config::BrbConfig;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Byzantine Reliable Broadcast manager
#[derive(Clone)]
pub struct ByzantineReliableBroadcast {
    config: BrbConfig,
    /// Messages we've broadcast ECHO for
    echoed: HashSet<Vec<u8>>,
    /// Messages we've broadcast READY for
    readied: HashSet<Vec<u8>>,
    /// Messages we've delivered
    delivered: HashSet<Vec<u8>>,
    /// State tracking for each message
    message_states: HashMap<Vec<u8>, BrbState>,
}

impl ByzantineReliableBroadcast {
    /// Create a new BRB instance
    pub fn new(config: BrbConfig) -> Self {
        Self {
            config,
            echoed: HashSet::new(),
            readied: HashSet::new(),
            delivered: HashSet::new(),
            message_states: HashMap::new(),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &BrbConfig {
        &self.config
    }

    /// Initiate broadcast of a message
    /// Returns ECHO message to be sent to all peers
    pub fn broadcast(&mut self, message_id: Vec<u8>, content: Vec<u8>) -> Option<BrbMessage> {
        if self.echoed.contains(&message_id) {
            return None;
        }

        self.echoed.insert(message_id.clone());
        self.get_or_create_state(&message_id);

        Some(BrbMessage::Echo {
            message_id,
            content,
        })
    }

    /// Handle incoming BRB message
    /// Returns messages that should be broadcast as a result
    pub fn handle_message(
        &mut self,
        from: PeerId,
        message: BrbMessage,
    ) -> Vec<BrbMessage> {
        match message {
            BrbMessage::Echo { message_id, content } => {
                self.handle_echo(from, message_id, content)
            }
            BrbMessage::Ready { message_id, content_hash } => {
                self.handle_ready(from, message_id, content_hash)
            }
        }
    }

    /// Handle ECHO message
    fn handle_echo(
        &mut self,
        from: PeerId,
        message_id: Vec<u8>,
        content: Vec<u8>,
    ) -> Vec<BrbMessage> {
        let mut result = Vec::new();

        // Update state
        {
            let state = self.get_or_create_state(&message_id);
            state.echo_senders.insert(from);
            state.content = Some(content.clone());
        }

        // Check if we should send READY
        let echo_count = self.message_states.get(&message_id)
            .map(|s| s.echo_senders.len())
            .unwrap_or(0);

        if !self.readied.contains(&message_id) && echo_count >= self.config.max_faulty + 1 {
            self.readied.insert(message_id.clone());

            let content_hash = blake3::hash(&content).as_bytes().to_vec();
            result.push(BrbMessage::Ready {
                message_id: message_id.clone(),
                content_hash,
            });
        }

        // Check if we should deliver
        if !self.delivered.contains(&message_id) {
            if self.should_deliver(&message_id) {
                self.delivered.insert(message_id);
            }
        }

        result
    }

    /// Handle READY message
    fn handle_ready(
        &mut self,
        from: PeerId,
        message_id: Vec<u8>,
        content_hash: Vec<u8>,
    ) -> Vec<BrbMessage> {
        let mut result = Vec::new();

        // Update state
        {
            let state = self.get_or_create_state(&message_id);
            state.ready_senders.entry(content_hash.clone())
                .or_insert_with(HashSet::new)
                .insert(from);
        }

        // Check if we should send READY
        let ready_count = self.message_states.get(&message_id)
            .and_then(|s| s.ready_senders.get(&content_hash))
            .map(|s| s.len())
            .unwrap_or(0);

        if !self.readied.contains(&message_id) && ready_count >= self.config.max_faulty + 1 {
            self.readied.insert(message_id.clone());
            result.push(BrbMessage::Ready {
                message_id: message_id.clone(),
                content_hash,
            });
        }

        // Check if we should deliver
        if !self.delivered.contains(&message_id) {
            if self.should_deliver(&message_id) {
                self.delivered.insert(message_id);
            }
        }

        result
    }

    /// Check if a message should be delivered
    fn should_deliver(&self, message_id: &[u8]) -> bool {
        if let Some(state) = self.message_states.get(message_id) {
            // Deliver if we have 2f+1 READY messages for the same content hash
            for ready_count in state.ready_senders.values() {
                if ready_count.len() >= self.config.quorum_size() {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a message has been delivered
    pub fn is_delivered(&self, message_id: &[u8]) -> bool {
        self.delivered.contains(message_id)
    }

    /// Get or create state for a message
    fn get_or_create_state(&mut self, message_id: &[u8]) -> &mut BrbState {
        self.message_states
            .entry(message_id.to_vec())
            .or_insert_with(BrbState::new)
    }

    /// Get the state of a message
    pub fn get_state(&self, message_id: &[u8]) -> Option<&BrbState> {
        self.message_states.get(message_id)
    }
}

/// State tracking for a single message in the BRB protocol
#[derive(Debug, Clone)]
pub struct BrbState {
    /// Peers who sent ECHO for this message
    pub echo_senders: HashSet<PeerId>,
    /// Peers who sent READY for each content hash
    pub ready_senders: HashMap<Vec<u8>, HashSet<PeerId>>,
    /// The message content (if we've received it)
    pub content: Option<Vec<u8>>,
}

impl BrbState {
    pub fn new() -> Self {
        Self {
            echo_senders: HashSet::new(),
            ready_senders: HashMap::new(),
            content: None,
        }
    }
}

/// BRB protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrbMessage {
    /// ECHO phase: initial broadcast
    Echo {
        message_id: Vec<u8>,
        content: Vec<u8>,
    },

    /// READY phase: acknowledgment
    Ready {
        message_id: Vec<u8>,
        content_hash: Vec<u8>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brb_creation() {
        let config = BrbConfig {
            enabled: true,
            total_peers: 7,
            max_faulty: 2,
        };

        let brb = ByzantineReliableBroadcast::new(config);
        assert_eq!(brb.config.quorum_size(), 5);
    }

    #[test]
    fn test_brb_broadcast() {
        let config = BrbConfig::default();
        let mut brb = ByzantineReliableBroadcast::new(config);

        let msg_id = vec![1, 2, 3];
        let content = vec![4, 5, 6];

        let echo = brb.broadcast(msg_id.clone(), content.clone());
        assert!(echo.is_some());

        // Second broadcast of same message should return None
        let echo2 = brb.broadcast(msg_id, content);
        assert!(echo2.is_none());
    }

    #[test]
    fn test_brb_echo_ready_flow() {
        let config = BrbConfig {
            enabled: true,
            total_peers: 7,
            max_faulty: 2,
        };
        let mut brb = ByzantineReliableBroadcast::new(config);

        let msg_id = vec![1, 2, 3];
        let content = vec![4, 5, 6];

        // Receive ECHOs from f+1 = 3 peers
        for i in 0..3 {
            let peer = PeerId::random();
            let responses = brb.handle_message(
                peer,
                BrbMessage::Echo {
                    message_id: msg_id.clone(),
                    content: content.clone(),
                },
            );

            if i == 2 {
                // After third ECHO, should send READY
                assert_eq!(responses.len(), 1);
                assert!(matches!(responses[0], BrbMessage::Ready { .. }));
            }
        }
    }

    #[test]
    fn test_brb_delivery() {
        let config = BrbConfig {
            enabled: true,
            total_peers: 7,
            max_faulty: 2,
        };
        let mut brb = ByzantineReliableBroadcast::new(config);

        let msg_id = vec![1, 2, 3];
        let content = vec![4, 5, 6];
        let content_hash = blake3::hash(&content).as_bytes().to_vec();

        // Receive 2f+1 = 5 READY messages
        for _ in 0..5 {
            let peer = PeerId::random();
            brb.handle_message(
                peer,
                BrbMessage::Ready {
                    message_id: msg_id.clone(),
                    content_hash: content_hash.clone(),
                },
            );
        }

        // Message should be delivered
        assert!(brb.is_delivered(&msg_id));
    }
}
