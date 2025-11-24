//! Integration tests for the high-level Sync API
//!
//! These tests verify that the sync configuration and high-level APIs
//! (trigger_sync, peer_count, etc.) work correctly with Netabase.

use netabase::Netabase;
use netabase::network::config::{NetabaseConfig, SyncConfig, GossipConfig, BrbConfig, SybilResistanceConfig};
use netabase_store::traits::definition::{NetabaseDefinitionTrait, NetabaseDefinitionTraitKey};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::time::Duration;
use strum::EnumDiscriminants;
use tokio::time::sleep;

// Test model definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, EnumDiscriminants)]
#[strum_discriminants(derive(
    strum::EnumIter,
    strum::Display,
    strum::AsRefStr,
    strum::EnumString,
    Hash,
    Encode,
    Decode
))]
#[strum_discriminants(name(TestDefinition))]
enum TestModel {
    User {
        id: u64,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Encode, Decode, EnumDiscriminants)]
#[strum_discriminants(derive(
    strum::EnumIter,
    strum::Display,
    strum::AsRefStr,
    strum::EnumString,
    Hash,
    Encode,
    Decode
))]
#[strum_discriminants(name(TestKeysDiscriminant))]
enum TestKeys {
    Id(u64),
}

impl NetabaseDefinitionTraitKey for TestKeys {}

impl NetabaseDefinitionTrait for TestModel {
    type Keys = TestKeys;
    type Tables = TestDefinition;

    fn tables() -> Self::Tables {
        TestDefinition::User
    }

    #[cfg(all(feature = "paxos", feature = "libp2p", not(target_arch = "wasm32")))]
    fn apply_to_store<S>(&self, _store: &mut S) -> Result<(), String>
    where
        S: libp2p::kad::store::RecordStore,
    {
        Ok(())
    }
}

impl netabase_store::convert::ToIVec for TestModel {}
impl netabase_store::convert::ToIVec for TestKeys {}

// ============================================================================
// CONFIGURATION TESTS
// ============================================================================

#[tokio::test]
async fn test_netabase_with_sync_config() {
    let sync_config = SyncConfig::development();
    let mut config = NetabaseConfig::default();
    config.sync = Some(sync_config);

    let netabase = Netabase::<TestModel>::new_with_path_and_config(
        "./test_data/sync_config_test",
        config,
    );

    assert!(netabase.is_ok(), "Should create Netabase with sync config");

    // Cleanup
    let _ = std::fs::remove_dir_all("./test_data/sync_config_test");
}

#[tokio::test]
async fn test_netabase_without_sync_config() {
    let config = NetabaseConfig::default();
    // No sync config

    let netabase = Netabase::<TestModel>::new_with_path_and_config(
        "./test_data/no_sync_test",
        config,
    );

    assert!(netabase.is_ok(), "Should create Netabase without sync config");

    if let Ok(nb) = netabase {
        assert!(!nb.is_sync_enabled(), "Sync should be disabled");
        assert!(nb.sync_config().is_none(), "Sync config should be None");
    }

    // Cleanup
    let _ = std::fs::remove_dir_all("./test_data/no_sync_test");
}

#[tokio::test]
async fn test_sync_config_presets() {
    // Test development preset
    let dev_config = SyncConfig::development();
    assert!(dev_config.enabled);
    assert_eq!(dev_config.sync_interval, Duration::from_secs(10));
    assert_eq!(dev_config.gossip.interval, Duration::from_secs(5));

    // Test production preset
    let prod_config = SyncConfig::production();
    assert!(prod_config.enabled);
    assert_eq!(prod_config.sync_interval, Duration::from_secs(30));
    assert_eq!(prod_config.brb.total_peers, 7);

    // Test high-security preset
    let secure_config = SyncConfig::high_security();
    assert!(secure_config.enabled);
    assert_eq!(secure_config.sybil_resistance.pow_difficulty, 20);

    // Test low-latency preset
    let fast_config = SyncConfig::low_latency();
    assert!(fast_config.enabled);
    assert_eq!(fast_config.sync_interval, Duration::from_secs(5));
}

// ============================================================================
// API TESTS
// ============================================================================

#[tokio::test]
async fn test_is_sync_enabled_api() {
    // With sync
    let sync_config = SyncConfig::development();
    let mut config = NetabaseConfig::default();
    config.sync = Some(sync_config);

    let netabase = Netabase::<TestModel>::new_with_path_and_config(
        "./test_data/sync_enabled_test",
        config,
    )
    .expect("Failed to create Netabase");

    assert!(netabase.is_sync_enabled(), "Sync should be enabled");

    // Cleanup
    let _ = std::fs::remove_dir_all("./test_data/sync_enabled_test");
}

#[tokio::test]
async fn test_sync_config_api() {
    let sync_config = SyncConfig::development();
    let mut config = NetabaseConfig::default();
    config.sync = Some(sync_config.clone());

    let netabase = Netabase::<TestModel>::new_with_path_and_config(
        "./test_data/sync_config_api_test",
        config,
    )
    .expect("Failed to create Netabase");

    let retrieved_config = netabase.sync_config();
    assert!(retrieved_config.is_some(), "Should have sync config");

    if let Some(cfg) = retrieved_config {
        assert_eq!(cfg.enabled, sync_config.enabled);
        assert_eq!(cfg.sync_interval, sync_config.sync_interval);
        assert_eq!(cfg.gossip.interval, sync_config.gossip.interval);
    }

    // Cleanup
    let _ = std::fs::remove_dir_all("./test_data/sync_config_api_test");
}

#[tokio::test]
async fn test_peer_count_api() {
    let sync_config = SyncConfig::development();
    let mut config = NetabaseConfig::default();
    config.sync = Some(sync_config);

    let mut netabase = Netabase::<TestModel>::new_with_path_and_config(
        "./test_data/peer_count_test",
        config,
    )
    .expect("Failed to create Netabase");

    // Start swarm
    netabase
        .start_swarm()
        .await
        .expect("Failed to start swarm");

    // Give network time to initialize
    sleep(Duration::from_millis(500)).await;

    // Get peer count (mDNS may discover other test instances, so we just verify the API works)
    let count = netabase.peer_count().await;
    assert!(count.is_ok(), "peer_count() should succeed");
    // Note: count may be > 0 due to mDNS discovering other test instances running in parallel

    // Cleanup
    netabase.stop_swarm().await.expect("Failed to stop swarm");
    let _ = std::fs::remove_dir_all("./test_data/peer_count_test");
}

#[tokio::test]
async fn test_trigger_sync_without_swarm() {
    let sync_config = SyncConfig::development();
    let mut config = NetabaseConfig::default();
    config.sync = Some(sync_config);

    let netabase = Netabase::<TestModel>::new_with_path_and_config(
        "./test_data/trigger_sync_no_swarm",
        config,
    )
    .expect("Failed to create Netabase");

    // Try to trigger sync without starting swarm
    let result = netabase.trigger_sync().await;
    // This should fail because swarm isn't running
    assert!(result.is_err(), "Should fail without swarm");

    // Cleanup
    let _ = std::fs::remove_dir_all("./test_data/trigger_sync_no_swarm");
}

#[tokio::test]
async fn test_trigger_sync_with_swarm() {
    let sync_config = SyncConfig::development();
    let mut config = NetabaseConfig::default();
    config.sync = Some(sync_config);

    let mut netabase = Netabase::<TestModel>::new_with_path_and_config(
        "./test_data/trigger_sync_with_swarm",
        config,
    )
    .expect("Failed to create Netabase");

    // Start swarm
    netabase
        .start_swarm()
        .await
        .expect("Failed to start swarm");

    sleep(Duration::from_millis(500)).await;

    // Trigger sync (may fail due to no peers, but should not panic)
    let result = netabase.trigger_sync().await;
    // With no peers, this will fail but shouldn't crash
    if result.is_err() {
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("No peers") || err_msg.contains("failed"),
            "Error should mention no peers"
        );
    }

    // Cleanup
    netabase.stop_swarm().await.expect("Failed to stop swarm");
    let _ = std::fs::remove_dir_all("./test_data/trigger_sync_with_swarm");
}

#[tokio::test]
async fn test_trigger_sync_without_sync_enabled() {
    let config = NetabaseConfig::default();
    // No sync config

    let mut netabase = Netabase::<TestModel>::new_with_path_and_config(
        "./test_data/no_sync_trigger",
        config,
    )
    .expect("Failed to create Netabase");

    netabase
        .start_swarm()
        .await
        .expect("Failed to start swarm");

    sleep(Duration::from_millis(500)).await;

    // Try to trigger sync without sync enabled
    let result = netabase.trigger_sync().await;
    assert!(result.is_err(), "Should fail when sync is disabled");

    let err_msg = result.err().unwrap().to_string();
    assert!(
        err_msg.contains("not enabled") || err_msg.contains("not found"),
        "Error should mention sync not enabled"
    );

    // Cleanup
    netabase.stop_swarm().await.expect("Failed to stop swarm");
    let _ = std::fs::remove_dir_all("./test_data/no_sync_trigger");
}

// ============================================================================
// BRB CONFIG VALIDATION TESTS
// ============================================================================

#[test]
fn test_brb_config_validation() {
    // Valid config
    let valid_config = BrbConfig {
        enabled: true,
        total_peers: 7,
        max_faulty: 2,  // 7 >= 2*2+1 = 5 ✓
    };
    assert!(valid_config.validate().is_ok());

    // Invalid config
    let invalid_config = BrbConfig {
        enabled: true,
        total_peers: 4,
        max_faulty: 2,  // 4 < 2*2+1 = 5 ✗
    };
    assert!(invalid_config.validate().is_err());

    // Disabled config (should pass even with invalid numbers)
    let disabled_config = BrbConfig {
        enabled: false,
        total_peers: 1,
        max_faulty: 10,
    };
    assert!(disabled_config.validate().is_ok());
}

#[test]
fn test_brb_quorum_size() {
    let config = BrbConfig {
        enabled: true,
        total_peers: 10,
        max_faulty: 3,
    };

    assert_eq!(config.quorum_size(), 7);  // 2*3+1 = 7

    let config2 = BrbConfig {
        enabled: true,
        total_peers: 7,
        max_faulty: 2,
    };

    assert_eq!(config2.quorum_size(), 5);  // 2*2+1 = 5
}

// ============================================================================
// LIFECYCLE TESTS
// ============================================================================

#[tokio::test]
async fn test_netabase_lifecycle_with_sync() {
    let sync_config = SyncConfig::development();
    let mut config = NetabaseConfig::default();
    config.sync = Some(sync_config);

    let mut netabase = Netabase::<TestModel>::new_with_path_and_config(
        "./test_data/lifecycle_test",
        config,
    )
    .expect("Failed to create Netabase");

    // Check initial state
    assert!(netabase.is_sync_enabled());
    assert!(netabase.sync_config().is_some());

    // Start swarm
    assert!(netabase.start_swarm().await.is_ok());
    sleep(Duration::from_millis(500)).await;

    // Check peer count
    let count = netabase.peer_count().await;
    assert!(count.is_ok());

    // Stop swarm
    assert!(netabase.stop_swarm().await.is_ok());

    // Cleanup
    let _ = std::fs::remove_dir_all("./test_data/lifecycle_test");
}
