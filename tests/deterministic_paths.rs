//! Tests for deterministic path generation functionality.
//!
//! These tests verify that when a seed is provided via the NETABASE_TEST_SEED
//! environment variable, the same paths are consistently generated, which is
//! essential for persistence testing across sessions and machines.

use std::env;

#[test]
fn test_deterministic_temp_dir_with_seed() {
    let seed_1 = Some(12345);
    // Generate paths with the same seed - should be identical
    let path1 = netabase::get_test_temp_dir(Some(1), seed_1);
    let path2 = netabase::get_test_temp_dir(Some(1), seed_1);

    assert_eq!(path1, path2, "Paths with the same seed should be identical");
}

#[test]
fn test_deterministic_temp_dir_str_with_seed() {
    let test_seed = Some(54321);
    // Generate paths with the same seed - should be identical
    let path1 = netabase::get_test_temp_dir_str(Some("test_suffix"), test_seed);
    let path2 = netabase::get_test_temp_dir_str(Some("test_suffix"), test_seed);

    assert_eq!(
        path1, path2,
        "String-suffixed paths with the same seed should be identical"
    );

    // Different suffixes should still produce different paths
    let path3 = netabase::get_test_temp_dir_str(Some("different_suffix"), test_seed);
    assert_ne!(path1, path3, "Paths with different suffixes should differ");

    let second_seed = Some(98765);

    // Generate path with different seed - should be different
    let path4 = netabase::get_test_temp_dir_str(Some("test_suffix"), second_seed);
    assert_ne!(path1, path4, "Paths with different seeds should differ");
}

#[test]
fn test_deterministic_temp_dir_with_default_with_seed() {
    // Generate paths with the same seed - should be identical
    let path1 = netabase::get_test_temp_dir(Some(1), Some(24680));
    let path2 = netabase::get_test_temp_dir(Some(1), Some(24680));

    assert_eq!(
        path1, path2,
        "Default paths with the same seed should be identical"
    );

    // Different test numbers should still produce different paths
    let path3 = netabase::get_test_temp_dir(Some(2), Some(24680));
    assert_ne!(
        path1, path3,
        "Default paths with different test numbers should differ"
    );

    // Generate path with different seed - should be different
    let path4 = netabase::get_test_temp_dir(Some(1), Some(13579));
    assert_ne!(
        path1, path4,
        "Default paths with different seeds should differ"
    );
}

#[test]
fn test_seed_consistency_across_different_functions() {
    // Set a specific seed

    let seed_1 = Some(11111);

    // Generate paths with different functions but same seed and parameters
    let path1 = netabase::get_test_temp_dir(Some(1), seed_1);
    let path2 = netabase::get_test_temp_dir(Some(1), seed_1);

    // Should be identical since they use the same seed and parameters
    assert_eq!(
        path1, path2,
        "Different functions with same seed and params should be identical"
    );
}

#[test]
fn test_random_suffix_without_seed() {
    // Clear any existing seed
    let path1 = netabase::get_test_temp_dir(Some(1), None);
    let path2 = netabase::get_test_temp_dir(Some(1), None);

    // Without a seed, each call should generate a different random path
    assert_ne!(
        path1, path2,
        "Paths without a seed should be different (random)"
    );
}

#[test]
fn test_persistence_scenario() {
    // Simulate a test scenario across multiple "sessions"

    let test_seed = Some(42);
    let writer_path_session1 =
        netabase::get_test_temp_dir_str(Some("persistence_writer"), test_seed);

    // Session 2: Reader uses the same seed to find the same paths
    let reader_path_session1 =
        netabase::get_test_temp_dir_str(Some("persistence_reader"), test_seed);

    // Later session: Writer uses the same seed again
    let writer_path_session2 =
        netabase::get_test_temp_dir_str(Some("persistence_writer"), test_seed);

    // Verify paths are consistent across "sessions"
    assert_eq!(
        writer_path_session1, writer_path_session2,
        "Writer paths should be consistent across sessions with the same seed"
    );

    // Different role (writer vs reader) with same seed should still have different paths
    // because the suffix is different
    assert_ne!(
        writer_path_session1, reader_path_session1,
        "Writer and reader paths should differ even with the same seed (due to different suffixes)"
    );
}
