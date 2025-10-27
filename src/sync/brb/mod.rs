//! Byzantine Reliable Broadcast (BRB) implementation
//!
//! This module implements the Byzantine Reliable Broadcast protocol for
//! ensuring consistent message delivery even with Byzantine faults.
//!
//! ## Protocol Overview
//!
//! The BRB protocol operates in three phases:
//! 1. **INIT**: Sender broadcasts message to all peers
//! 2. **ECHO**: Peers echo the message after receiving INIT
//! 3. **READY**: Peers send READY after receiving echo threshold
//! 4. **DELIVER**: Message is delivered after receiving ready threshold
//!
//! ## Byzantine Fault Tolerance
//!
//! For n = 3f+1 nodes where f is the maximum Byzantine nodes:
//! - Echo threshold: (n+f)/2 + 1
//! - Ready threshold: 2f+1
//! - Delivery threshold: 2f+1 ready messages

pub mod quorum;
pub mod validator;

use crate::sync::types::{SyncMessage, Version};
use anyhow::{anyhow, Result};
use libp2p::PeerId;
use std::collections::HashMap;

pub use quorum::{QuorumConfig, QuorumTracker};
pub use validator::BrbValidator;

/// BRB message phases
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrbPhase {
    /// Initial broadcast
    Init,
    /// Echo phase
    Echo,
    /// Ready phase
    Ready,
    /// Delivered
    Delivered,
}

/// BRB message state tracking
#[derive(Clone, Debug)]
pub struct BrbMessageState {
    /// Message hash
    pub hash: [u8; 32],

    /// Original payload
    pub payload: Vec<u8>,

    /// Current phase
    pub phase: BrbPhase,

    /// Echo quorum tracker
    pub echo_quorum: QuorumTracker,

    /// Ready quorum tracker
    pub ready_quorum: QuorumTracker,

    /// Original sender
    pub sender: PeerId,

    /// Message version
    pub version: Version,

    /// Whether we've sent our echo
    pub sent_echo: bool,

    /// Whether we've sent our ready
    pub sent_ready: bool,
}

impl BrbMessageState {
    /// Create new message state
    pub fn new(
        hash: [u8; 32],
        payload: Vec<u8>,
        sender: PeerId,
        version: Version,
        total_peers: usize,
        max_faulty: usize,
    ) -> Self {
        Self {
            hash,
            payload,
            phase: BrbPhase::Init,
            echo_quorum: QuorumTracker::new(total_peers, max_faulty),
            ready_quorum: QuorumTracker::new(total_peers, max_faulty),
            sender,
            version,
            sent_echo: false,
            sent_ready: false,
        }
    }
}

/// BRB configuration
#[derive(Clone, Debug)]
pub struct BrbConfig {
    /// Total number of peers in the network
    pub total_peers: usize,

    /// Maximum Byzantine faults to tolerate
    pub max_faulty: usize,

    /// Require message signatures
    pub require_signatures: bool,
}

impl BrbConfig {
    /// Create a new BRB configuration
    pub fn new(total_peers: usize, max_faulty: usize) -> Self {
        Self {
            total_peers,
            max_faulty,
            require_signatures: false,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.total_peers < 3 * self.max_faulty + 1 {
            return Err(anyhow!(
                "Invalid BRB config: need at least 3f+1 nodes (have {}, need {})",
                self.total_peers,
                3 * self.max_faulty + 1
            ));
        }
        Ok(())
    }
}

/// BRB protocol manager
pub struct BrbManager {
    /// Configuration
    config: BrbConfig,

    /// Active BRB message states
    messages: HashMap<[u8; 32], BrbMessageState>,

    /// Message validator
    validator: BrbValidator,

    /// Local peer ID
    local_peer_id: PeerId,

    /// Statistics
    stats: BrbStats,
}

impl BrbManager {
    /// Create a new BRB manager
    pub fn new(config: BrbConfig, local_peer_id: PeerId) -> Result<Self> {
        config.validate()?;

        Ok(Self {
            validator: BrbValidator::new(config.require_signatures),
            config,
            messages: HashMap::new(),
            local_peer_id,
            stats: BrbStats::default(),
        })
    }

    /// Initiate a broadcast
    /// Returns the message hash and list of peers to send INIT to
    pub fn initiate_broadcast(
        &mut self,
        payload: Vec<u8>,
        version: Version,
    ) -> Result<([u8; 32], Vec<PeerId>)> {
        // Compute message hash
        let hash = blake3::hash(&payload);
        let message_hash: [u8; 32] = hash.into();

        // Create message state
        let state = BrbMessageState::new(
            message_hash,
            payload,
            self.local_peer_id,
            version,
            self.config.total_peers,
            self.config.max_faulty,
        );

        // Store state
        self.messages.insert(message_hash, state);
        self.stats.initiated += 1;

        // Return hash and empty peer list (will be populated by caller)
        Ok((message_hash, vec![]))
    }

    /// Handle incoming INIT message
    /// Returns actions to take: (send_echo, send_ready)
    pub fn handle_init(
        &mut self,
        from: &PeerId,
        message_hash: [u8; 32],
        payload: Vec<u8>,
        version: Version,
    ) -> Result<BrbAction> {
        // Verify hash
        let computed_hash: [u8; 32] = blake3::hash(&payload).into();
        if computed_hash != message_hash {
            return Err(anyhow!("Message hash mismatch"));
        }

        // Check if we already have this message
        if let Some(state) = self.messages.get_mut(&message_hash) {
            // Already processing this message
            return Ok(BrbAction::None);
        }

        // Create new message state
        let mut state = BrbMessageState::new(
            message_hash,
            payload,
            *from,
            version,
            self.config.total_peers,
            self.config.max_faulty,
        );

        // Mark that we've received INIT, send ECHO
        state.sent_echo = true;
        state.phase = BrbPhase::Echo;

        self.messages.insert(message_hash, state);
        self.stats.inits_received += 1;

        Ok(BrbAction::SendEcho(message_hash, *from))
    }

    /// Handle incoming ECHO message
    pub fn handle_echo(
        &mut self,
        from: &PeerId,
        message_hash: [u8; 32],
        original_sender: PeerId,
    ) -> Result<BrbAction> {
        // Get or create message state
        let state = self.messages.get_mut(&message_hash).ok_or_else(|| {
            anyhow!("Received ECHO for unknown message")
        })?;

        // Add echo
        if state.echo_quorum.add_responder(*from) {
            self.stats.echoes_received += 1;
        }

        // Check if we should transition to READY
        if state.phase == BrbPhase::Echo && state.echo_quorum.has_echo_threshold() {
            state.phase = BrbPhase::Ready;

            // Send READY if we haven't already
            if !state.sent_ready {
                state.sent_ready = true;
                return Ok(BrbAction::SendReady(message_hash, original_sender));
            }
        }

        Ok(BrbAction::None)
    }

    /// Handle incoming READY message
    pub fn handle_ready(
        &mut self,
        from: &PeerId,
        message_hash: [u8; 32],
        original_sender: PeerId,
    ) -> Result<BrbAction> {
        // Get message state
        let state = self.messages.get_mut(&message_hash).ok_or_else(|| {
            anyhow!("Received READY for unknown message")
        })?;

        // Add ready
        if state.ready_quorum.add_responder(*from) {
            self.stats.readies_received += 1;
        }

        // Amplification: If we receive f+1 READY messages, send READY ourselves
        // This ensures liveness even if some honest nodes don't send READY
        if !state.sent_ready
            && state.ready_quorum.responder_count() > self.config.max_faulty
        {
            state.sent_ready = true;
            state.phase = BrbPhase::Ready;
            return Ok(BrbAction::SendReady(message_hash, original_sender));
        }

        // Check if we should deliver
        if state.phase != BrbPhase::Delivered && state.ready_quorum.has_ready_threshold() {
            state.phase = BrbPhase::Delivered;
            self.stats.delivered += 1;
            return Ok(BrbAction::Deliver(
                message_hash,
                state.payload.clone(),
                state.version.clone(),
            ));
        }

        Ok(BrbAction::None)
    }

    /// Handle incoming BRB message
    pub fn handle_message(&mut self, from: &PeerId, message: &SyncMessage) -> Result<BrbAction> {
        // Validate message
        self.validator.validate_message(from, message)?;

        match message {
            SyncMessage::BrbInit {
                peer_id,
                payload,
                version,
                ..
            } => {
                let hash: [u8; 32] = blake3::hash(payload).into();
                self.handle_init(peer_id, hash, payload.clone(), version.clone())
            }
            SyncMessage::BrbEcho {
                message_hash,
                original_sender,
                ..
            } => self.handle_echo(from, *message_hash, *original_sender),
            SyncMessage::BrbReady {
                message_hash,
                original_sender,
                ..
            } => self.handle_ready(from, *message_hash, *original_sender),
            _ => Err(anyhow!("Not a BRB message")),
        }
    }

    /// Get message state
    pub fn get_message_state(&self, hash: &[u8; 32]) -> Option<&BrbMessageState> {
        self.messages.get(hash)
    }

    /// Get statistics
    pub fn stats(&self) -> &BrbStats {
        &self.stats
    }

    /// Get configuration
    pub fn config(&self) -> &BrbConfig {
        &self.config
    }

    /// Clean up old delivered messages
    pub fn cleanup_delivered(&mut self, max_age: usize) {
        self.messages
            .retain(|_, state| state.phase != BrbPhase::Delivered || max_age > 0);
    }
}

/// Actions to take after processing a BRB message
#[derive(Clone, Debug)]
pub enum BrbAction {
    /// No action needed
    None,

    /// Send ECHO message to all peers
    SendEcho([u8; 32], PeerId),

    /// Send READY message to all peers
    SendReady([u8; 32], PeerId),

    /// Deliver the message
    Deliver([u8; 32], Vec<u8>, Version),
}

/// BRB statistics
#[derive(Clone, Debug, Default)]
pub struct BrbStats {
    /// Number of broadcasts initiated
    pub initiated: usize,

    /// Number of INIT messages received
    pub inits_received: usize,

    /// Number of ECHO messages received
    pub echoes_received: usize,

    /// Number of READY messages received
    pub readies_received: usize,

    /// Number of messages delivered
    pub delivered: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::clock::VectorClock;

    #[test]
    fn test_brb_config_validation() {
        // Valid config: n=4, f=1 (4 >= 3*1+1)
        let config = BrbConfig::new(4, 1);
        assert!(config.validate().is_ok());

        // Invalid config: n=3, f=1 (3 < 3*1+1)
        let config = BrbConfig::new(3, 1);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_brb_manager_creation() {
        let peer = PeerId::random();
        let config = BrbConfig::new(4, 1);
        let manager = BrbManager::new(config, peer);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_invalid_brb_manager_creation() {
        let peer = PeerId::random();
        let config = BrbConfig::new(3, 1); // Invalid config
        let manager = BrbManager::new(config, peer);
        assert!(manager.is_err());
    }

    #[test]
    fn test_initiate_broadcast() {
        let peer = PeerId::random();
        let config = BrbConfig::new(4, 1);
        let mut manager = BrbManager::new(config, peer).unwrap();

        let payload = vec![1, 2, 3, 4];
        let clock = VectorClock::new(peer);
        let hash = blake3::hash(&payload);
        let version = Version::new(clock, hash.into());

        let result = manager.initiate_broadcast(payload.clone(), version);
        assert!(result.is_ok());

        let (message_hash, _) = result.unwrap();
        let expected_hash: [u8; 32] = blake3::hash(&payload).into();
        assert_eq!(message_hash, expected_hash);

        // Check state was created
        assert!(manager.get_message_state(&message_hash).is_some());
        assert_eq!(manager.stats().initiated, 1);
    }

    #[test]
    fn test_handle_init() {
        let peer = PeerId::random();
        let config = BrbConfig::new(4, 1);
        let mut manager = BrbManager::new(config, peer).unwrap();

        let sender = PeerId::random();
        let payload = vec![1, 2, 3, 4];
        let message_hash: [u8; 32] = blake3::hash(&payload).into();

        let clock = VectorClock::new(sender);
        let hash = blake3::hash(&payload);
        let version = Version::new(clock, hash.into());

        let action = manager
            .handle_init(&sender, message_hash, payload, version)
            .unwrap();

        // Should send ECHO
        match action {
            BrbAction::SendEcho(hash, from) => {
                assert_eq!(hash, message_hash);
                assert_eq!(from, sender);
            }
            _ => panic!("Expected SendEcho action"),
        }

        // Check state
        let state = manager.get_message_state(&message_hash).unwrap();
        assert_eq!(state.phase, BrbPhase::Echo);
        assert!(state.sent_echo);
        assert_eq!(manager.stats().inits_received, 1);
    }

    #[test]
    fn test_handle_echo_reaches_threshold() {
        let peer = PeerId::random();
        let config = BrbConfig::new(4, 1);
        let mut manager = BrbManager::new(config, peer).unwrap();

        let sender = PeerId::random();
        let payload = vec![1, 2, 3, 4];
        let message_hash: [u8; 32] = blake3::hash(&payload).into();

        let clock = VectorClock::new(sender);
        let hash = blake3::hash(&payload);
        let version = Version::new(clock, hash.into());

        // Initialize message
        manager
            .handle_init(&sender, message_hash, payload, version)
            .unwrap();

        // Send echoes from 3 peers (threshold is 3 for n=4, f=1)
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        manager.handle_echo(&peer1, message_hash, sender).unwrap();
        manager.handle_echo(&peer2, message_hash, sender).unwrap();

        let action = manager.handle_echo(&peer, message_hash, sender).unwrap();

        // Should transition to READY and send READY
        match action {
            BrbAction::SendReady(hash, from) => {
                assert_eq!(hash, message_hash);
                assert_eq!(from, sender);
            }
            _ => panic!("Expected SendReady action"),
        }

        let state = manager.get_message_state(&message_hash).unwrap();
        assert_eq!(state.phase, BrbPhase::Ready);
    }

    #[test]
    fn test_handle_ready_delivers() {
        let peer = PeerId::random();
        let config = BrbConfig::new(4, 1);
        let mut manager = BrbManager::new(config, peer).unwrap();

        let sender = PeerId::random();
        let payload = vec![1, 2, 3, 4];
        let message_hash: [u8; 32] = blake3::hash(&payload).into();

        let clock = VectorClock::new(sender);
        let hash = blake3::hash(&payload);
        let version = Version::new(clock.clone(), hash.into());

        // Initialize and move to READY phase
        manager
            .handle_init(&sender, message_hash, payload.clone(), version.clone())
            .unwrap();

        // Manually set phase to READY
        if let Some(state) = manager.messages.get_mut(&message_hash) {
            state.phase = BrbPhase::Ready;
        }

        // Send ready messages from 3 peers (threshold is 3 for n=4, f=1)
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        manager.handle_ready(&peer1, message_hash, sender).unwrap();
        manager.handle_ready(&peer2, message_hash, sender).unwrap();

        let action = manager
            .handle_ready(&peer3, message_hash, sender)
            .unwrap();

        // Should deliver
        match action {
            BrbAction::Deliver(hash, delivered_payload, delivered_version) => {
                assert_eq!(hash, message_hash);
                assert_eq!(delivered_payload, payload);
                assert_eq!(delivered_version.clock, clock);
            }
            _ => panic!("Expected Deliver action"),
        }

        let state = manager.get_message_state(&message_hash).unwrap();
        assert_eq!(state.phase, BrbPhase::Delivered);
        assert_eq!(manager.stats().delivered, 1);
    }

    #[test]
    fn test_ready_amplification() {
        let peer = PeerId::random();
        let config = BrbConfig::new(7, 2); // f=2
        let mut manager = BrbManager::new(config, peer).unwrap();

        let sender = PeerId::random();
        let payload = vec![1, 2, 3, 4];
        let message_hash: [u8; 32] = blake3::hash(&payload).into();

        let clock = VectorClock::new(sender);
        let hash = blake3::hash(&payload);
        let version = Version::new(clock, hash.into());

        // Initialize message
        manager
            .handle_init(&sender, message_hash, payload.clone(), version)
            .unwrap();

        // Send f+1 = 3 READY messages (less than delivery threshold of 2f+1 = 5)
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        manager.handle_ready(&peer1, message_hash, sender).unwrap();
        manager.handle_ready(&peer2, message_hash, sender).unwrap();

        let action = manager
            .handle_ready(&peer3, message_hash, sender)
            .unwrap();

        // Should send READY ourselves (amplification)
        match action {
            BrbAction::SendReady(hash, from) => {
                assert_eq!(hash, message_hash);
                assert_eq!(from, sender);
            }
            _ => panic!("Expected SendReady action (amplification)"),
        }

        let state = manager.get_message_state(&message_hash).unwrap();
        assert!(state.sent_ready);
    }

    #[test]
    fn test_duplicate_echo_ignored() {
        let peer = PeerId::random();
        let config = BrbConfig::new(4, 1);
        let mut manager = BrbManager::new(config, peer).unwrap();

        let sender = PeerId::random();
        let payload = vec![1, 2, 3, 4];
        let message_hash: [u8; 32] = blake3::hash(&payload).into();

        let clock = VectorClock::new(sender);
        let hash = blake3::hash(&payload);
        let version = Version::new(clock, hash.into());

        // Initialize message
        manager
            .handle_init(&sender, message_hash, payload, version)
            .unwrap();

        let peer1 = PeerId::random();

        // First echo
        manager.handle_echo(&peer1, message_hash, sender).unwrap();
        let count1 = manager.stats().echoes_received;

        // Duplicate echo
        manager.handle_echo(&peer1, message_hash, sender).unwrap();
        let count2 = manager.stats().echoes_received;

        // Count should only increase once
        assert_eq!(count1, count2);
    }

    #[test]
    fn test_message_state_creation() {
        let peer = PeerId::random();
        let clock = VectorClock::new(peer);
        let hash = blake3::hash(b"test");
        let version = Version::new(clock, hash.into());

        let state =
            BrbMessageState::new([1u8; 32], vec![1, 2, 3], peer, version, 4, 1);

        assert_eq!(state.phase, BrbPhase::Init);
        assert!(!state.sent_echo);
        assert!(!state.sent_ready);
        assert_eq!(state.echo_quorum.quorum_size(), 3); // 2*1+1
        assert_eq!(state.ready_quorum.quorum_size(), 3); // 2*1+1
    }

    #[test]
    fn test_cleanup_delivered() {
        let peer = PeerId::random();
        let config = BrbConfig::new(4, 1);
        let mut manager = BrbManager::new(config, peer).unwrap();

        let sender = PeerId::random();
        let payload = vec![1, 2, 3, 4];
        let message_hash: [u8; 32] = blake3::hash(&payload).into();

        let clock = VectorClock::new(sender);
        let hash = blake3::hash(&payload);
        let version = Version::new(clock, hash.into());

        // Initialize message
        manager
            .handle_init(&sender, message_hash, payload, version)
            .unwrap();

        // Set as delivered
        if let Some(state) = manager.messages.get_mut(&message_hash) {
            state.phase = BrbPhase::Delivered;
        }

        assert!(manager.get_message_state(&message_hash).is_some());

        // Cleanup
        manager.cleanup_delivered(0);

        // Should be removed
        assert!(manager.get_message_state(&message_hash).is_none());
    }
}
