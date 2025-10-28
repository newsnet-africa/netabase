//! Integration tests for the sync module

use netabase::sync::{
    SyncBehaviorManager, SyncManagerConfigBuilder, SyncPresets,
    SimpleReputationSystem, ReputationSystem, ChallengeSystem, ProofOfWorkConfig,
    PaxosInstance, PaxosConfig, PaxosMessage,
};
use libp2p::PeerId;
use std::time::Duration;

#[test]
fn test_sync_manager_creation() {
    let peer_id = PeerId::random();
    let config = SyncManagerConfigBuilder::new().build();

    let manager = SyncBehaviorManager::new(peer_id, config);
    assert!(manager.is_ok());
}

#[test]
fn test_sync_manager_peer_management() {
    let peer_id = PeerId::random();
    let config = SyncManagerConfigBuilder::new().build();

    let mut manager = SyncBehaviorManager::new(peer_id, config).unwrap();

    // Initially no peers
    assert_eq!(manager.peer_count(), 0);

    // Add peers
    let peer1 = PeerId::random();
    let peer2 = PeerId::random();
    manager.add_peer(peer1);
    manager.add_peer(peer2);
    assert_eq!(manager.peer_count(), 2);

    // Remove peer
    manager.remove_peer(&peer1);
    assert_eq!(manager.peer_count(), 1);
}

#[test]
fn test_challenge_system_workflow() {
    let peer_id = PeerId::random();
    let config = SyncManagerConfigBuilder::new()
        .pow_difficulty(4)
        .pow_enabled(true)
        .build();

    let mut manager = SyncBehaviorManager::new(peer_id, config).unwrap();

    let target_peer = PeerId::random();

    // Issue challenge
    let challenge = manager.issue_challenge(target_peer);
    assert!(!challenge.is_empty());

    // Peer not verified initially
    assert!(!manager.is_peer_verified(&target_peer));

    // Note: In real scenario, peer would solve challenge and submit proof
    // For testing, we verify the challenge was issued
}

#[test]
fn test_reputation_system_integration() {
    let mut reputation = SimpleReputationSystem::new();

    let peer1 = PeerId::random();
    let peer2 = PeerId::random();
    let peer3 = PeerId::random();

    // Record interactions
    for _ in 0..5 {
        reputation.record_success(&peer1);
    }
    for _ in 0..2 {
        reputation.record_success(&peer2);
    }
    reputation.record_failure(&peer3);

    // Check reputations
    assert!(reputation.reputation(&peer1) > 0.5);
    assert!(reputation.reputation(&peer2) > 0.5);
    assert!(reputation.reputation(&peer3) < 0.5);

    // Top peers
    let top = reputation.top_peers(2);
    assert_eq!(top.len(), 2);
    assert_eq!(top[0], peer1); // Most successful interactions
}

#[test]
fn test_reputation_with_decay_disabled() {
    let mut reputation = SimpleReputationSystem::with_decay(false);

    let peer = PeerId::random();

    reputation.reward(&peer, 0.3);
    let initial_rep = reputation.reputation(&peer);

    // Apply decay (should not change anything)
    reputation.apply_decay_all();

    let after_decay = reputation.reputation(&peer);
    assert_eq!(initial_rep, after_decay);
}

#[test]
fn test_paxos_basic_workflow() {
    // Setup: 3 nodes, tolerate 1 failure
    let proposer_id = PeerId::random();
    let acceptor1_id = PeerId::random();
    let acceptor2_id = PeerId::random();

    let config = PaxosConfig::new(3, 1);

    let mut proposer = PaxosInstance::new(proposer_id, config.clone());
    let mut acceptor1 = PaxosInstance::new(acceptor1_id, config.clone());
    let mut acceptor2 = PaxosInstance::new(acceptor2_id, config);

    // Proposer proposes a value
    let value = b"test value".to_vec();
    let proposal = proposer.propose(value.clone());

    // Acceptors handle prepare
    let promise1 = acceptor1.handle_prepare(proposal).unwrap();
    let promise2 = acceptor2.handle_prepare(proposal).unwrap();

    // Proposer handles promises
    if let PaxosMessage::Promise {
        proposal_number,
        accepted_proposal,
        accepted_value,
    } = promise1
    {
        let result = proposer.handle_promise(
            acceptor1_id,
            proposal_number,
            accepted_proposal,
            accepted_value,
        );
        assert!(result.is_ok());
    }

    if let PaxosMessage::Promise {
        proposal_number,
        accepted_proposal,
        accepted_value,
    } = promise2
    {
        let result = proposer.handle_promise(
            acceptor2_id,
            proposal_number,
            accepted_proposal,
            accepted_value,
        );
        assert!(result.is_ok());
        // After quorum, should get Accept message
        assert!(result.unwrap().is_some());
    }
}

#[test]
fn test_paxos_reject_lower_proposals() {
    let peer = PeerId::random();
    let config = PaxosConfig::new(3, 1);

    let mut acceptor = PaxosInstance::new(peer, config);

    // Accept higher proposal first
    let high_proposal = netabase::sync::ProposalNumber::new(10, peer);
    acceptor.handle_prepare(high_proposal).unwrap();

    // Try lower proposal - should fail
    let low_proposal = netabase::sync::ProposalNumber::new(5, peer);
    let result = acceptor.handle_prepare(low_proposal);

    assert!(result.is_err());
}

#[test]
fn test_config_builder_custom() {
    let config = SyncManagerConfigBuilder::new()
        .gossip_interval(Duration::from_secs(5))
        .gossip_fanout(5)
        .brb_config(9, 3)
        .pow_difficulty(8)
        .challenge_duration(Duration::from_secs(120))
        .build();

    assert_eq!(config.gossip.interval, Duration::from_secs(5));
    assert_eq!(config.gossip.fanout, 5);
    assert_eq!(config.pow.difficulty, 8);
    // BRB with n=9, f=3 requires 2f+1 = 7 for ready quorum
    assert_eq!(config.brb.max_faulty, 3);
}

#[test]
fn test_config_presets() {
    // Development preset
    let dev_config = SyncPresets::development();
    assert!(!dev_config.signature_required);
    assert_eq!(dev_config.gossip_interval, Duration::from_secs(5));

    // Private network preset
    let private_config = SyncPresets::private_network();
    assert!(private_config.signature_required);

    // Public small preset
    let public_config = SyncPresets::public_small();
    assert!(public_config.byzantine_tolerance.enable_brb);

    // High throughput preset
    let throughput_config = SyncPresets::high_throughput();
    assert_eq!(throughput_config.max_sync_batch_size, 200);
}

#[test]
fn test_reputation_diminishing_returns() {
    let mut reputation = SimpleReputationSystem::with_decay(false);
    let peer = PeerId::random();

    // First few successes give significant boosts
    reputation.record_success(&peer);
    let rep_after_1 = reputation.reputation(&peer);

    reputation.record_success(&peer);
    let rep_after_2 = reputation.reputation(&peer);

    let initial_boost = rep_after_2 - rep_after_1;

    // Many more successes
    for _ in 0..98 {
        reputation.record_success(&peer);
    }
    let rep_after_100 = reputation.reputation(&peer);

    // Later successes give smaller boosts (diminishing returns)
    // The 100th success should give much less boost than the 2nd
    let later_boost = (rep_after_100 - rep_after_2) / 98.0;
    assert!(later_boost < initial_boost / 2.0);
}

#[test]
fn test_reputation_failure_penalty() {
    let mut reputation = SimpleReputationSystem::with_decay(false);
    let peer = PeerId::random();

    let initial = reputation.reputation(&peer);

    // Record a failure
    reputation.record_failure(&peer);
    let after_failure = reputation.reputation(&peer);

    // Failure should decrease reputation significantly
    assert!(initial - after_failure >= 0.2);
}

#[test]
fn test_multiple_sync_managers() {
    // Simulate multiple nodes
    let peer1 = PeerId::random();
    let peer2 = PeerId::random();
    let peer3 = PeerId::random();

    let config = SyncManagerConfigBuilder::new()
        .brb_config(4, 1)  // Need at least 3f+1 = 4 nodes for f=1
        .build();

    let manager1 = SyncBehaviorManager::new(peer1, config.clone()).unwrap();
    let manager2 = SyncBehaviorManager::new(peer2, config.clone()).unwrap();
    let manager3 = SyncBehaviorManager::new(peer3, config).unwrap();

    // All managers created successfully
    assert_eq!(manager1.peer_count(), 0);
    assert_eq!(manager2.peer_count(), 0);
    assert_eq!(manager3.peer_count(), 0);
}

#[test]
fn test_pow_system_verification() {
    use netabase::sync::PoWSystem;

    let config = ProofOfWorkConfig {
        difficulty: 4,
        enabled: true,
    };

    let pow_system = PoWSystem::new(config);
    let message = b"test message";

    // Generate proof
    let proof = pow_system.generate(message);

    // Verify proof
    assert!(pow_system.verify(message, &proof));

    // Wrong message should fail
    let wrong_message = b"wrong message";
    assert!(!pow_system.verify(wrong_message, &proof));
}

#[test]
fn test_challenge_system_expiry() {
    let config = ProofOfWorkConfig {
        difficulty: 4,
        enabled: true,
    };

    let mut challenges = ChallengeSystem::new(
        config,
        Duration::from_millis(100), // Very short duration for testing
        Duration::from_secs(3600),
    );

    let peer = PeerId::random();
    challenges.issue_challenge(peer);

    // Initially has 1 challenge
    assert_eq!(challenges.challenge_count(), 1);

    // Wait for expiry
    std::thread::sleep(Duration::from_millis(150));

    // Cleanup expired
    challenges.cleanup_expired();

    // Challenge should be expired
    assert_eq!(challenges.challenge_count(), 0);
}
