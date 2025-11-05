#!/usr/bin/env nu
#
# Script to run P2P integration tests for netabase
#
# These tests verify that multiple netabase nodes can:
# - Discover each other via mDNS
# - Store and retrieve records via Kademlia DHT
# - Use provider records
# - Communicate across the network
#
# Usage:
#   nu run_p2p_tests.nu [--verbose] [--test <test_name>]
#

def main [
    --verbose (-v)        # Enable verbose output
    --test (-t): string   # Run a specific test (e.g., "test_mdns_peer_discovery")
] {
    print "╔════════════════════════════════════════════════════════╗"
    print "║        Netabase P2P Integration Tests                 ║"
    print "╚════════════════════════════════════════════════════════╝"
    print ""

    # Clean up any leftover test data
    print "🧹 Cleaning up previous test data..."
    if ("./test_data" | path exists) {
        rm -rf ./test_data
    }

    # Build the test node binary
    print "🔨 Building test node binary..."
    let build_result = (
        cargo test --test test_node --no-run
        | complete
    )

    if $build_result.exit_code != 0 {
        print $"❌ Failed to build test_node binary"
        print $build_result.stderr
        exit 1
    }

    print "✅ Test node binary built successfully"
    print ""

    # Build the integration tests
    print "🔨 Building integration tests..."
    let test_build_result = (
        cargo test --test p2p_integration_tests --no-run
        | complete
    )

    if $test_build_result.exit_code != 0 {
        print $"❌ Failed to build integration tests"
        print $test_build_result.stderr
        exit 1
    }

    print "✅ Integration tests built successfully"
    print ""

    # Run the tests
    print "🧪 Running P2P integration tests..."
    print "   (Note: These tests are marked as #[ignore], running with --ignored)"
    print ""

    let test_args = if $test != null {
        ["--test", "p2p_integration_tests", "--", "--ignored", "--nocapture", $test]
    } else {
        ["--test", "p2p_integration_tests", "--", "--ignored", "--nocapture"]
    }

    let verbose_env = if $verbose { {RUST_LOG: "debug"} } else { {} }

    let test_result = (
        cargo test ...$test_args
        | complete
    )

    print ""
    if $test_result.exit_code == 0 {
        print "✅ All P2P integration tests passed!"
    } else {
        print "❌ Some tests failed"
        print $test_result.stderr
        exit 1
    }

    # Clean up test data
    print ""
    print "🧹 Cleaning up test data..."
    if ("./test_data" | path exists) {
        rm -rf ./test_data
    }

    print "✨ Done!"
}
