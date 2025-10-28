//! Configuration builders and presets for the sync module

use crate::sync::{
    SyncConfig, ByzantineTolerance, SybilResistanceMode,
    behavior::SyncManagerConfig,
    gossip::GossipConfig,
    brb::BrbConfig,
    proof::ProofOfWorkConfig,
};
use std::time::Duration;

/// Configuration builder for sync module
pub struct SyncConfigBuilder {
    config: SyncConfig,
}

impl SyncConfigBuilder {
    /// Create a new configuration builder with defaults
    pub fn new() -> Self {
        Self {
            config: SyncConfig::default(),
        }
    }

    /// Enable or disable synchronization
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set gossip interval
    pub fn gossip_interval(mut self, interval: Duration) -> Self {
        self.config.gossip_interval = interval;
        self
    }

    /// Set number of peers to gossip with per round
    pub fn gossip_fanout(mut self, fanout: usize) -> Self {
        self.config.gossip_fanout = fanout;
        self
    }

    /// Set maximum number of Byzantine faulty nodes to tolerate
    pub fn max_faulty_nodes(mut self, max_faulty: usize) -> Self {
        self.config.byzantine_tolerance.max_faulty_nodes = max_faulty;
        self
    }

    /// Enable or disable Byzantine Reliable Broadcast
    pub fn enable_brb(mut self, enabled: bool) -> Self {
        self.config.byzantine_tolerance.enable_brb = enabled;
        self
    }

    /// Require signature verification on all messages
    pub fn verify_signatures(mut self, verify: bool) -> Self {
        self.config.byzantine_tolerance.verify_signatures = verify;
        self
    }

    /// Require quorum for state acceptance
    pub fn require_quorum(mut self, require: bool) -> Self {
        self.config.byzantine_tolerance.require_quorum = require;
        self
    }

    /// Set Sybil resistance mode
    pub fn sybil_resistance(mut self, mode: SybilResistanceMode) -> Self {
        self.config.sybil_resistance = mode;
        self
    }

    /// Set maximum sync batch size
    pub fn max_sync_batch_size(mut self, size: usize) -> Self {
        self.config.max_sync_batch_size = size;
        self
    }

    /// Require signatures on all messages
    pub fn signature_required(mut self, required: bool) -> Self {
        self.config.signature_required = required;
        self
    }

    /// Set maximum concurrent sync operations
    pub fn max_concurrent_syncs(mut self, max: usize) -> Self {
        self.config.max_concurrent_syncs = max;
        self
    }

    /// Set timeout for sync operations
    pub fn sync_timeout(mut self, timeout: Duration) -> Self {
        self.config.sync_timeout = timeout;
        self
    }

    /// Build the configuration
    pub fn build(self) -> SyncConfig {
        self.config
    }
}

impl Default for SyncConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration builder for SyncManager
pub struct SyncManagerConfigBuilder {
    gossip: GossipConfig,
    brb: BrbConfig,
    pow: ProofOfWorkConfig,
    challenge_duration: Duration,
    verification_duration: Duration,
}

impl SyncManagerConfigBuilder {
    /// Create a new SyncManager configuration builder with defaults
    pub fn new() -> Self {
        Self {
            gossip: GossipConfig::default(),
            brb: BrbConfig::new(7, 2).expect("Invalid BRB config"),
            pow: ProofOfWorkConfig::default(),
            challenge_duration: Duration::from_secs(60),
            verification_duration: Duration::from_secs(3600),
        }
    }

    /// Set gossip interval
    pub fn gossip_interval(mut self, interval: Duration) -> Self {
        self.gossip.interval = interval;
        self
    }

    /// Set gossip fanout (number of peers per round)
    pub fn gossip_fanout(mut self, fanout: usize) -> Self {
        self.gossip.fanout = fanout;
        self
    }

    /// Set BRB configuration (total nodes, max Byzantine faults)
    pub fn brb_config(mut self, total_nodes: usize, max_faults: usize) -> Self {
        self.brb = BrbConfig::new(total_nodes, max_faults).expect("Invalid BRB config");
        self
    }

    /// Set PoW difficulty
    pub fn pow_difficulty(mut self, difficulty: u32) -> Self {
        self.pow.difficulty = difficulty;
        self
    }

    /// Enable or disable PoW
    pub fn pow_enabled(mut self, enabled: bool) -> Self {
        self.pow.enabled = enabled;
        self
    }

    /// Set challenge duration
    pub fn challenge_duration(mut self, duration: Duration) -> Self {
        self.challenge_duration = duration;
        self
    }

    /// Set verification validity duration
    pub fn verification_duration(mut self, duration: Duration) -> Self {
        self.verification_duration = duration;
        self
    }

    /// Build the configuration
    pub fn build(self) -> SyncManagerConfig {
        SyncManagerConfig {
            gossip: self.gossip,
            brb: self.brb,
            pow: self.pow,
            challenge_duration: self.challenge_duration,
            verification_duration: self.verification_duration,
        }
    }
}

impl Default for SyncManagerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Preset configurations for common use cases
pub struct SyncPresets;

impl SyncPresets {
    /// Configuration for development/testing (relaxed security)
    pub fn development() -> SyncConfig {
        SyncConfigBuilder::new()
            .gossip_interval(Duration::from_secs(5))
            .gossip_fanout(2)
            .max_faulty_nodes(1)
            .sybil_resistance(SybilResistanceMode::None)
            .signature_required(false)
            .build()
    }

    /// Configuration for small private networks (trusted peers)
    pub fn private_network() -> SyncConfig {
        SyncConfigBuilder::new()
            .gossip_interval(Duration::from_secs(10))
            .gossip_fanout(3)
            .max_faulty_nodes(1)
            .sybil_resistance(SybilResistanceMode::None)
            .signature_required(true)
            .verify_signatures(true)
            .build()
    }

    /// Configuration for small public networks (moderate security)
    pub fn public_small() -> SyncConfig {
        SyncConfigBuilder::new()
            .gossip_interval(Duration::from_secs(10))
            .gossip_fanout(3)
            .max_faulty_nodes(1)
            .enable_brb(true)
            .sybil_resistance(SybilResistanceMode::Reputation)
            .signature_required(true)
            .verify_signatures(true)
            .require_quorum(true)
            .build()
    }

    /// Configuration for large public networks (high security)
    pub fn public_large() -> SyncConfig {
        SyncConfigBuilder::new()
            .gossip_interval(Duration::from_secs(15))
            .gossip_fanout(5)
            .max_faulty_nodes(2)
            .enable_brb(true)
            .sybil_resistance(SybilResistanceMode::ProofOfWork { difficulty: 16 })
            .signature_required(true)
            .verify_signatures(true)
            .require_quorum(true)
            .max_sync_batch_size(50)
            .build()
    }

    /// Configuration for high-throughput scenarios
    pub fn high_throughput() -> SyncConfig {
        SyncConfigBuilder::new()
            .gossip_interval(Duration::from_secs(5))
            .gossip_fanout(5)
            .max_faulty_nodes(1)
            .enable_brb(false) // Disable BRB for lower overhead
            .sybil_resistance(SybilResistanceMode::Reputation)
            .max_sync_batch_size(200)
            .max_concurrent_syncs(10)
            .build()
    }
}

/// SyncManager preset configurations
pub struct SyncManagerPresets;

impl SyncManagerPresets {
    /// Development configuration (fast, minimal security)
    pub fn development() -> SyncManagerConfig {
        SyncManagerConfigBuilder::new()
            .gossip_interval(Duration::from_secs(5))
            .gossip_fanout(2)
            .brb_config(3, 1)
            .pow_difficulty(4)
            .pow_enabled(false)
            .challenge_duration(Duration::from_secs(30))
            .verification_duration(Duration::from_secs(1800))
            .build()
    }

    /// Production configuration (balanced)
    pub fn production() -> SyncManagerConfig {
        SyncManagerConfigBuilder::new()
            .gossip_interval(Duration::from_secs(10))
            .gossip_fanout(3)
            .brb_config(7, 2)
            .pow_difficulty(16)
            .pow_enabled(true)
            .challenge_duration(Duration::from_secs(60))
            .verification_duration(Duration::from_secs(3600))
            .build()
    }

    /// High security configuration
    pub fn high_security() -> SyncManagerConfig {
        SyncManagerConfigBuilder::new()
            .gossip_interval(Duration::from_secs(15))
            .gossip_fanout(5)
            .brb_config(9, 3)
            .pow_difficulty(20)
            .pow_enabled(true)
            .challenge_duration(Duration::from_secs(120))
            .verification_duration(Duration::from_secs(7200))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_builder() {
        let config = SyncConfigBuilder::new()
            .gossip_interval(Duration::from_secs(5))
            .gossip_fanout(4)
            .max_faulty_nodes(2)
            .build();

        assert_eq!(config.gossip_interval, Duration::from_secs(5));
        assert_eq!(config.gossip_fanout, 4);
        assert_eq!(config.byzantine_tolerance.max_faulty_nodes, 2);
    }

    #[test]
    fn test_presets() {
        let dev = SyncPresets::development();
        assert!(!dev.signature_required);

        let private = SyncPresets::private_network();
        assert!(private.signature_required);
        assert_eq!(private.sybil_resistance, SybilResistanceMode::None);

        let public = SyncPresets::public_small();
        assert!(public.byzantine_tolerance.enable_brb);
        assert_eq!(public.sybil_resistance, SybilResistanceMode::Reputation);

        let large = SyncPresets::public_large();
        assert_eq!(large.gossip_fanout, 5);
        assert_eq!(large.byzantine_tolerance.max_faulty_nodes, 2);
    }

    #[test]
    fn test_sync_manager_builder() {
        let config = SyncManagerConfigBuilder::new()
            .gossip_interval(Duration::from_secs(5))
            .pow_difficulty(8)
            .brb_config(5, 1)
            .build();

        assert_eq!(config.gossip.interval, Duration::from_secs(5));
        assert_eq!(config.pow.difficulty, 8);
        assert_eq!(config.brb.max_faulty, 1); // Configured for 1 Byzantine fault
    }
}
