//! Byzantine message filtering for gossip protocol

use crate::sync::reputation::SimpleReputationSystem;
use crate::sync::traits::ReputationSystem;
use crate::sync::types::{MessageSignature, SyncMessage, SyncRecord};
use libp2p::PeerId;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Byzantine filter configuration
#[derive(Clone, Debug)]
pub struct ByzantineFilterConfig {
    /// Minimum reputation threshold for accepting messages
    pub min_reputation: f64,

    /// Enable signature verification
    pub verify_signatures: bool,

    /// Enable rate limiting
    pub enable_rate_limit: bool,

    /// Maximum messages per peer per window
    pub max_messages_per_window: usize,

    /// Rate limit window duration
    pub rate_limit_window: Duration,

    /// Penalty for failed signature verification
    pub signature_failure_penalty: f64,

    /// Penalty for rate limit violation
    pub rate_limit_penalty: f64,

    /// Reward for successful message
    pub success_reward: f64,
}

impl Default for ByzantineFilterConfig {
    fn default() -> Self {
        Self {
            min_reputation: 0.3,
            verify_signatures: true,
            enable_rate_limit: true,
            max_messages_per_window: 100,
            rate_limit_window: Duration::from_secs(60),
            signature_failure_penalty: 0.3,
            rate_limit_penalty: 0.1,
            success_reward: 0.05,
        }
    }
}

/// Byzantine filter for validating and filtering gossip messages
pub struct ByzantineFilter {
    /// Configuration
    config: ByzantineFilterConfig,

    /// Reputation system
    reputation: SimpleReputationSystem,

    /// Rate limiter: peer -> (message count, window start)
    rate_limiter: HashMap<PeerId, (usize, Instant)>,

    /// Statistics
    stats: FilterStats,
}

impl ByzantineFilter {
    /// Create a new Byzantine filter
    pub fn new(config: ByzantineFilterConfig) -> Self {
        Self {
            config,
            reputation: SimpleReputationSystem::new(),
            rate_limiter: HashMap::new(),
            stats: FilterStats::default(),
        }
    }

    /// Filter a gossip message
    pub fn filter_message(&mut self, from: &PeerId, message: &SyncMessage) -> FilterResult {
        self.stats.total_messages += 1;

        // Check reputation
        if !self.check_reputation(from) {
            self.stats.rejected_reputation += 1;
            return FilterResult::Rejected(FilterReason::LowReputation);
        }

        // Check rate limit
        if self.config.enable_rate_limit && !self.check_rate_limit(from) {
            self.stats.rejected_rate_limit += 1;
            self.reputation
                .penalize(from, self.config.rate_limit_penalty);
            return FilterResult::Rejected(FilterReason::RateLimitExceeded);
        }

        // Verify signature if required
        if self.config.verify_signatures
            && let Some(reason) = self.verify_message_signature(message)
        {
            self.stats.rejected_signature += 1;
            self.reputation
                .penalize(from, self.config.signature_failure_penalty);
            return FilterResult::Rejected(reason);
        }

        // Message passed all filters
        self.stats.accepted += 1;
        self.reputation.reward(from, self.config.success_reward);

        FilterResult::Accepted
    }

    /// Filter sync records
    pub fn filter_records(&mut self, from: &PeerId, records: &[SyncRecord]) -> Vec<SyncRecord> {
        let mut filtered = Vec::new();

        for record in records {
            // Verify record signature if present
            if self.config.verify_signatures {
                if let Some(sig) = &record.signature {
                    if !self.verify_record_signature(record, sig) {
                        self.reputation
                            .penalize(from, self.config.signature_failure_penalty);
                        continue;
                    }
                }
            }

            // Check for duplicate or malformed records
            if self.validate_record(record) {
                filtered.push(record.clone());
            } else {
                self.reputation.penalize(from, 0.05);
            }
        }

        filtered
    }

    /// Check if peer has sufficient reputation
    fn check_reputation(&self, peer: &PeerId) -> bool {
        self.reputation.reputation(peer) >= self.config.min_reputation
    }

    /// Check rate limit for peer
    fn check_rate_limit(&mut self, peer: &PeerId) -> bool {
        let now = Instant::now();

        let (count, window_start) = self.rate_limiter.entry(peer.clone()).or_insert((0, now));

        // Reset window if expired
        if now.duration_since(*window_start) > self.config.rate_limit_window {
            *count = 0;
            *window_start = now;
        }

        // Check limit
        if *count >= self.config.max_messages_per_window {
            return false;
        }

        // Increment counter
        *count += 1;
        true
    }

    /// Verify message signature
    fn verify_message_signature(&self, message: &SyncMessage) -> Option<FilterReason> {
        // TODO: Implement actual signature verification
        // For now, accept all messages
        // In production, this should verify digital signatures
        match message {
            SyncMessage::SyncRequest { .. } => None,
            SyncMessage::SyncResponse { .. } => None,
            SyncMessage::DataTransfer { .. } => None,
            SyncMessage::GossipUpdate { .. } => None,
            _ => None,
        }
    }

    /// Verify record signature
    fn verify_record_signature(&self, _record: &SyncRecord, _signature: &MessageSignature) -> bool {
        // TODO: Implement actual signature verification
        // For now, accept all records
        true
    }

    /// Validate record structure and content
    fn validate_record(&self, record: &SyncRecord) -> bool {
        // Basic validation
        if record.key.is_empty() {
            return false;
        }

        if record.value.is_empty() {
            return false;
        }

        // Check timestamp is not in future
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if record.version.timestamp > now + 300 {
            // Allow 5 minutes clock skew
            return false;
        }

        true
    }

    /// Get reputation for a peer
    pub fn get_reputation(&self, peer: &PeerId) -> f64 {
        self.reputation.reputation(peer)
    }

    /// Get filter statistics
    pub fn stats(&self) -> &FilterStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = FilterStats::default();
    }

    /// Clean up old rate limiter entries
    pub fn cleanup_rate_limiter(&mut self) {
        let now = Instant::now();
        let window = self.config.rate_limit_window;

        self.rate_limiter.retain(|_, (_, start)| {
            now.duration_since(*start) <= window * 2 // Keep entries for 2 windows
        });
    }
}

impl Default for ByzantineFilter {
    fn default() -> Self {
        Self::new(ByzantineFilterConfig::default())
    }
}

/// Result of filtering a message
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterResult {
    /// Message accepted
    Accepted,

    /// Message rejected
    Rejected(FilterReason),
}

impl FilterResult {
    /// Check if message was accepted
    pub fn is_accepted(&self) -> bool {
        matches!(self, FilterResult::Accepted)
    }

    /// Check if message was rejected
    pub fn is_rejected(&self) -> bool {
        matches!(self, FilterResult::Rejected(_))
    }
}

/// Reason for rejecting a message
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterReason {
    /// Peer reputation too low
    LowReputation,

    /// Rate limit exceeded
    RateLimitExceeded,

    /// Invalid signature
    InvalidSignature,

    /// Malformed message
    MalformedMessage,

    /// Other reason
    Other(String),
}

/// Filter statistics
#[derive(Clone, Debug, Default)]
pub struct FilterStats {
    /// Total messages processed
    pub total_messages: usize,

    /// Messages accepted
    pub accepted: usize,

    /// Rejected due to low reputation
    pub rejected_reputation: usize,

    /// Rejected due to rate limit
    pub rejected_rate_limit: usize,

    /// Rejected due to invalid signature
    pub rejected_signature: usize,
}

impl FilterStats {
    /// Get acceptance rate (0.0 to 1.0)
    pub fn acceptance_rate(&self) -> f64 {
        if self.total_messages == 0 {
            return 1.0;
        }
        self.accepted as f64 / self.total_messages as f64
    }

    /// Get rejection rate (0.0 to 1.0)
    pub fn rejection_rate(&self) -> f64 {
        1.0 - self.acceptance_rate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::clock::VectorClock;
    use crate::sync::types::{StateDigest, Version};

    #[test]
    fn test_filter_creation() {
        let filter = ByzantineFilter::default();
        assert_eq!(filter.stats().total_messages, 0);
    }

    #[test]
    fn test_reputation_check() {
        let mut filter = ByzantineFilter::default();
        let peer = PeerId::random();

        // New peers have default reputation (0.5), which is above threshold (0.3)
        assert!(filter.check_reputation(&peer));

        // Penalize heavily
        filter.reputation.penalize(&peer, 0.5);
        assert!(!filter.check_reputation(&peer));
    }

    #[test]
    fn test_rate_limiting() {
        let config = ByzantineFilterConfig {
            max_messages_per_window: 2,
            ..Default::default()
        };
        let mut filter = ByzantineFilter::new(config);
        let peer = PeerId::random();

        // First two messages should pass
        assert!(filter.check_rate_limit(&peer));
        assert!(filter.check_rate_limit(&peer));

        // Third message should fail
        assert!(!filter.check_rate_limit(&peer));
    }

    #[test]
    fn test_filter_message() {
        let mut filter = ByzantineFilter::default();
        let peer = PeerId::random();

        let peer_id2 = PeerId::random();
        let clock = VectorClock::new(peer_id2);
        let digest = StateDigest::new([0u8; 32], 0, clock);

        let message = SyncMessage::SyncRequest {
            peer_id: peer,
            state_digest: digest,
        };

        let result = filter.filter_message(&peer, &message);
        assert!(result.is_accepted());
        assert_eq!(filter.stats().accepted, 1);
    }

    #[test]
    fn test_filter_records() {
        let mut filter = ByzantineFilter::default();
        let peer = PeerId::random();

        let peer_id2 = PeerId::random();
        let clock = VectorClock::new(peer_id2);
        let hash = blake3::hash(b"test");
        let version = Version::new(clock, hash.into());

        let records = vec![
            SyncRecord::new(vec![1], vec![10], version.clone()),
            SyncRecord::new(vec![2], vec![20], version),
        ];

        let filtered = filter.filter_records(&peer, &records);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_invalid_record_filtering() {
        let mut filter = ByzantineFilter::default();
        let peer = PeerId::random();

        let peer_id2 = PeerId::random();
        let clock = VectorClock::new(peer_id2);
        let hash = blake3::hash(b"test");
        let version = Version::new(clock, hash.into());

        let records = vec![
            SyncRecord::new(vec![], vec![10], version.clone()), // Empty key - invalid
            SyncRecord::new(vec![1], vec![], version),          // Empty value - invalid
        ];

        let filtered = filter.filter_records(&peer, &records);
        assert_eq!(filtered.len(), 0); // All invalid
    }

    #[test]
    fn test_stats() {
        let filter = ByzantineFilter::default();
        let stats = filter.stats();

        assert_eq!(stats.acceptance_rate(), 1.0);
        assert_eq!(stats.rejection_rate(), 0.0);
    }
}
