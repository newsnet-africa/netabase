#!/usr/bin/env nu
#
# Script to run chat integration tests for netabase
#
# These tests verify that the simple_mdns_chat example works correctly:
# - Multiple nodes can discover each other
# - Messages are sent and propagated
# - Commands work correctly
# - Tests scale from 2 to 10+ nodes
#
# Usage:
#   nu run_chat_tests.nu [--verbose] [--test <test_name>] [--scale <small|medium|large>]
#

def main [
    --verbose (-v)              # Enable verbose output
    --test (-t): string         # Run a specific test
    --scale (-s): string        # Run tests by scale: small (2-3 nodes), medium (5 nodes), large (10 nodes)
] {
    print "╔══════════════════════════════════════════════════════════╗"
    print "║    Netabase Chat Integration Tests                      ║"
    print "╚══════════════════════════════════════════════════════════╝"
    print ""

    # Clean up any leftover test data
    print "🧹 Cleaning up previous test data..."
    if ("./test_chat_data" | path exists) {
        rm -rf ./test_chat_data
    }

    # Build the chat_test_node binary
    print "🔨 Building chat_test_node binary..."
    let build_result = (
        cargo test --test chat_test_node --no-run
        | complete
    )

    if $build_result.exit_code != 0 {
        print $"❌ Failed to build chat_test_node binary"
        print $build_result.stderr
        exit 1
    }

    print "✅ chat_test_node binary built successfully"
    print ""

    # Build the integration tests
    print "🔨 Building chat integration tests..."
    let test_build_result = (
        cargo test --test chat_integration_tests --no-run
        | complete
    )

    if $test_build_result.exit_code != 0 {
        print $"❌ Failed to build chat integration tests"
        print $test_build_result.stderr
        exit 1
    }

    print "✅ Chat integration tests built successfully"
    print ""

    # Run the tests
    print "🧪 Running chat integration tests..."
    print ""

    # Determine which tests to run
    let test_filter = if $test != null {
        $test
    } else if $scale != null {
        match $scale {
            "small" => "two_node|small_network"
            "medium" => "medium_network"
            "large" => "large_network"
            _ => {
                print $"❌ Unknown scale: ($scale). Use: small, medium, or large"
                exit 1
            }
        }
    } else {
        null
    }

    let test_args = if $test_filter != null {
        ["--test", "chat_integration_tests", "--", "--ignored", "--nocapture", $test_filter]
    } else {
        ["--test", "chat_integration_tests", "--", "--ignored", "--nocapture"]
    }

    let verbose_env = if $verbose { {RUST_LOG: "debug"} } else { {} }

    let test_result = (
        cargo test ...$test_args
        | complete
    )

    print ""
    if $test_result.exit_code == 0 {
        print "✅ All chat integration tests passed!"
    } else {
        print "❌ Some tests failed"
        print $test_result.stderr
        exit 1
    }

    # Clean up test data
    print ""
    print "🧹 Cleaning up test data..."
    if ("./test_chat_data" | path exists) {
        rm -rf ./test_chat_data
    }

    print "✨ Done!"
}
