//! Tests for deterministic path generation functionality.
//!
//! These tests verify that when a seed is provided via the NETABASE_TEST_SEED
//! environment variable, the same paths are consistently generated, which is
//! essential for persistence testing across sessions and machines.

use std::env;

#[test]
fn test_deterministic_temp_dir_with_seed() {
    // Set a specific seed
    env::set_var("NETABASE_TEST_SEED", "12345");

    // Generate paths with the same seed - should be identical
    let path1 = netabase::get_test_temp_dir(Some(1));
    let path2 = netabase::get_test_temp_dir(Some(1));

    assert_eq!(path1, path2, "Paths with the same seed should be identical");

    // Different test numbers should still produce different paths
    let path3 = netabase::get_test_temp_dir(Some(2));
    assert_ne!(
        path1, path3,
        "Paths with different test numbers should differ"
    );

    // Change the seed
    env::set_var("NETABASE_TEST_SEED", "67890");

    // Generate path with different seed - should be different
    let path4 = netabase::get_test_temp_dir(Some(1));
    assert_ne!(path1, path4, "Paths with different seeds should differ");
}

#[test]
fn test_deterministic_temp_dir_str_with_seed() {
    // Set a specific seed
    env::set_var("NETABASE_TEST_SEED", "54321");

    // Generate paths with the same seed - should be identical
    let path1 = netabase::get_test_temp_dir_str(Some("test_suffix"));
    let path2 = netabase::get_test_temp_dir_str(Some("test_suffix"));

    assert_eq!(
        path1, path2,
        "String-suffixed paths with the same seed should be identical"
    );

    // Different suffixes should still produce different paths
    let path3 = netabase::get_test_temp_dir_str(Some("different_suffix"));
    assert_ne!(path1, path3, "Paths with different suffixes should differ");

    // Change the seed
    env::set_var("NETABASE_TEST_SEED", "98765");

    // Generate path with different seed - should be different
    let path4 = netabase::get_test_temp_dir_str(Some("test_suffix"));
    assert_ne!(path1, path4, "Paths with different seeds should differ");
}

#[test]
fn test_deterministic_temp_dir_with_default_with_seed() {
    // Set a specific seed
    env::set_var("NETABASE_TEST_SEED", "24680");

    // Generate paths with the same seed - should be identical
    let path1 = netabase::get_test_temp_dir_with_default(Some(1));
    let path2 = netabase::get_test_temp_dir_with_default(Some(1));

    assert_eq!(
        path1, path2,
        "Default paths with the same seed should be identical"
    );

    // Different test numbers should still produce different paths
    let path3 = netabase::get_test_temp_dir_with_default(Some(2));
    assert_ne!(
        path1, path3,
        "Default paths with different test numbers should differ"
    );

    // Change the seed
    env::set_var("NETABASE_TEST_SEED", "13579");

    // Generate path with different seed - should be different
    let path4 = netabase::get_test_temp_dir_with_default(Some(1));
    assert_ne!(
        path1, path4,
        "Default paths with different seeds should differ"
    );
}

#[test]
fn test_seed_consistency_across_different_functions() {
    // Set a specific seed
    env::set_var("NETABASE_TEST_SEED", "11111");

    // Generate paths with different functions but same seed and parameters
    let path1 = netabase::get_test_temp_dir(Some(1));
    let path2 = netabase::get_test_temp_dir_with_default(Some(1));

    // Should be identical since they use the same seed and parameters
    assert_eq!(
        path1, path2,
        "Different functions with same seed and params should be identical"
    );
}

#[test]
fn test_random_suffix_without_seed() {
    // Clear any existing seed
    env::remove_var("NETABASE_TEST_SEED");

    // Generate paths without a seed - should be random and different
    let path1 = netabase::get_test_temp_dir(Some(1));
    let path2 = netabase::get_test_temp_dir(Some(1));

    // Without a seed, each call should generate a different random path
    assert_ne!(
        path1, path2,
        "Paths without a seed should be different (random)"
    );
}

#[test]
fn test_persistence_scenario() {
    // Simulate a test scenario across multiple "sessions"

    // Session 1: Writer creates paths with seed 42
    env::set_var("NETABASE_TEST_SEED", "42");
    let writer_path_session1 = netabase::get_test_temp_dir_str(Some("persistence_writer"));

    // Session 2: Reader uses the same seed to find the same paths
    let reader_path_session1 = netabase::get_test_temp_dir_str(Some("persistence_reader"));

    // Later session: Writer uses the same seed again
    let writer_path_session2 = netabase::get_test_temp_dir_str(Some("persistence_writer"));

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
