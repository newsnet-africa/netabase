//! Synchronization manager that integrates with existing libp2p NetworkBehaviour
//!
//! This module provides sync capabilities (gossip, BRB, challenges) that work
//! with the existing NetabaseBehaviour's Kademlia DHT for peer discovery.

use crate::sync::brb::{BrbAction, BrbConfig, BrbManager};
use crate::sync::gossip::{GossipConfig, GossipManager};
use crate::sync::proof::{ChallengeSystem, ProofOfWork, ProofOfWorkConfig};
use crate::sync::traits::SyncableStore;
use crate::sync::types::SyncMessage;
use anyhow::Result;
use libp2p::PeerId;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

/// Synchronization manager configuration
#[derive(Clone, Debug)]
pub struct SyncManagerConfig {
    /// Gossip configuration
    pub gossip: GossipConfig,

    /// BRB configuration
    pub brb: BrbConfig,

    /// PoW configuration
    pub pow: ProofOfWorkConfig,

    /// Challenge duration
    pub challenge_duration: Duration,

    /// Verification duration
    pub verification_duration: Duration,
}

impl Default for SyncManagerConfig {
    fn default() -> Self {
        Self {
            gossip: GossipConfig::default(),
            brb: BrbConfig::new(7, 2), // n=7, f=2
            pow: ProofOfWorkConfig::default(),
            challenge_duration: Duration::from_secs(60),
            verification_duration: Duration::from_secs(3600),
        }
    }
}

/// Synchronization manager that works with existing NetabaseBehaviour
/// Uses the NetabaseBehaviour's Kademlia DHT for peer discovery
pub struct SyncManager {
    /// Local peer ID
    local_peer_id: PeerId,

    /// Configuration
    config: SyncManagerConfig,

    /// Gossip manager
    gossip: GossipManager,

    /// BRB manager
    brb: BrbManager,

    /// Challenge system for Sybil resistance
    challenges: ChallengeSystem,

    /// Pending events to emit
    events: VecDeque<SyncEvent>,

    /// Pending messages to send
    pending_messages: HashMap<PeerId, Vec<SyncMessage>>,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new(local_peer_id: PeerId, config: SyncManagerConfig) -> Result<Self> {
        let gossip = GossipManager::new(config.gossip.clone(), local_peer_id);
        let brb = BrbManager::new(config.brb.clone(), local_peer_id)?;
        let challenges = ChallengeSystem::new(
            config.pow.clone(),
            config.challenge_duration,
            config.verification_duration,
        );

        Ok(Self {
            local_peer_id,
            config,
            gossip,
            brb,
            challenges,
            events: VecDeque::new(),
            pending_messages: HashMap::new(),
        })
    }

    /// Add a peer to the gossip network
    /// Note: Peer discovery is handled by NetabaseBehaviour's Kademlia
    pub fn add_peer(&mut self, peer_id: PeerId) {
        self.gossip.add_peer(peer_id);
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.gossip.remove_peer(peer_id);
    }

    /// Issue a challenge to a peer
    pub fn issue_challenge(&mut self, peer_id: PeerId) -> Vec<u8> {
        self.challenges.issue_challenge(peer_id)
    }

    /// Verify a challenge response
    pub fn verify_challenge(&mut self, peer_id: &PeerId, proof: &ProofOfWork) -> Result<()> {
        self.challenges.verify_response(peer_id, proof)
    }

    /// Check if a peer is verified
    pub fn is_peer_verified(&self, peer_id: &PeerId) -> bool {
        self.challenges.is_verified(peer_id)
    }

    /// Initiate a gossip round (requires store reference)
    pub async fn initiate_gossip<S: SyncableStore>(&mut self, store: &S) -> Result<()> {
        let messages = self.gossip.initiate_gossip_round(store).await?;

        for (peer, message) in messages {
            self.pending_messages
                .entry(peer)
                .or_insert_with(Vec::new)
                .push(message);
        }

        Ok(())
    }

    /// Handle incoming sync message (requires store reference)
    pub async fn handle_sync_message<S: SyncableStore>(
        &mut self,
        store: &S,
        from: &PeerId,
        message: SyncMessage,
    ) -> Result<()> {
        // Check if peer is verified (for non-BRB messages)
        if !matches!(message, SyncMessage::BrbInit { .. } | SyncMessage::BrbEcho { .. } | SyncMessage::BrbReady { .. }) {
            if !self.is_peer_verified(from) {
                self.events.push_back(SyncEvent::PeerNeedsVerification(*from));
                return Ok(());
            }
        }

        // Handle based on message type
        match &message {
            SyncMessage::BrbInit { .. } | SyncMessage::BrbEcho { .. } | SyncMessage::BrbReady { .. } => {
                // Handle BRB message
                let action = self.brb.handle_message(from, &message)?;
                self.handle_brb_action(action)?;
            }
            _ => {
                // Handle gossip message
                if let Some(response) = self.gossip.handle_message(store, from, message).await? {
                    self.pending_messages
                        .entry(*from)
                        .or_insert_with(Vec::new)
                        .push(response);
                }
            }
        }

        Ok(())
    }

    /// Handle BRB action
    fn handle_brb_action(&mut self, action: BrbAction) -> Result<()> {
        match action {
            BrbAction::SendEcho(hash, original_sender) => {
                let message = SyncMessage::BrbEcho {
                    peer_id: self.local_peer_id,
                    message_hash: hash,
                    original_sender,
                };
                self.broadcast_message(message);
            }
            BrbAction::SendReady(hash, original_sender) => {
                let message = SyncMessage::BrbReady {
                    peer_id: self.local_peer_id,
                    message_hash: hash,
                    original_sender,
                };
                self.broadcast_message(message);
            }
            BrbAction::Deliver(hash, payload, version) => {
                self.events.push_back(SyncEvent::MessageDelivered {
                    hash,
                    payload,
                    version,
                });
            }
            BrbAction::None => {}
        }
        Ok(())
    }

    /// Broadcast a message to all peers
    fn broadcast_message(&mut self, _message: SyncMessage) {
        // TODO: Implement broadcast using peer list from gossip
        // In production, this would iterate over all known peers and queue messages
    }

    /// Cleanup expired challenges and verifications
    pub fn cleanup(&mut self) {
        self.challenges.cleanup_expired();
    }

    /// Get peer count from gossip network
    pub fn peer_count(&self) -> usize {
        self.gossip.peer_count()
    }
}

/// Events emitted by SyncManager
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// A peer needs verification (challenge should be issued)
    PeerNeedsVerification(PeerId),

    /// A message was delivered through BRB
    MessageDelivered {
        hash: [u8; 32],
        payload: Vec<u8>,
        version: crate::sync::types::Version,
    },

    /// Kademlia event
    Kademlia(Box<KademliaEvent>),
}

// Note: NetworkBehaviour implementation would go here
// This requires implementing the NetworkBehaviour trait which is complex
// and depends on the libp2p version. The full implementation would look like:
//
// impl NetworkBehaviour for SyncBehavior {
//     type ConnectionHandler = ...;
//     type OutEvent = SyncEvent;
//
//     fn handle_established_inbound_connection(...) -> Result<THandler<Self>, ConnectionDenied> { ... }
//     fn handle_established_outbound_connection(...) -> Result<THandler<Self>, ConnectionDenied> { ... }
//     fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) { ... }
//     fn on_connection_handler_event(...) { ... }
//     fn poll(&mut self, cx: &mut Context<'_>, params: &mut impl PollParameters) -> Poll<ToSwarm<Self::OutEvent, THandlerInEvent<Self>>> { ... }
// }
//
// This is left as a TODO due to the complexity and version-specific nature of libp2p's NetworkBehaviour trait

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::clock::VectorClock;

    #[test]
    fn test_sync_behavior_creation() {
        let peer = PeerId::random();
        let config = SyncBehaviorConfig::default();
        let behavior = SyncBehavior::new(peer, config);
        assert!(behavior.is_ok());
    }

    #[test]
    fn test_add_remove_peer() {
        let peer = PeerId::random();
        let config = SyncBehaviorConfig::default();
        let mut behavior = SyncBehavior::new(peer, config).unwrap();

        let other_peer = PeerId::random();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/1234".parse().unwrap();

        behavior.add_peer(other_peer, addr);
        assert_eq!(behavior.peer_count(), 1);

        behavior.remove_peer(&other_peer);
        assert_eq!(behavior.peer_count(), 0);
    }

    #[test]
    fn test_challenge_system_integration() {
        let peer = PeerId::random();
        let config = SyncBehaviorConfig::default();
        let mut behavior = SyncBehavior::new(peer, config).unwrap();

        let other_peer = PeerId::random();
        let challenge = behavior.issue_challenge(other_peer);
        assert!(!challenge.is_empty());
        assert!(!behavior.is_peer_verified(&other_peer));
    }
}
