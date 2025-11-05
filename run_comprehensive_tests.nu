#!/usr/bin/env nu
# Comprehensive test runner for netabase
# This script runs all tests in a systematic order

print "═══════════════════════════════════════════════════════════"
print "  Netabase Comprehensive Test Suite"
print "═══════════════════════════════════════════════════════════\n"

# Track results
mut results = []

# Helper function to run a test and record result
def run-test [name: string, command: closure] {
    print $"Running: ($name)"
    print "─────────────────────────────────────────────────────────\n"

    let start_time = (date now)
    let result = (do $command | complete)
    let end_time = (date now)
    let duration = (($end_time - $start_time) / 1sec)

    if $result.exit_code == 0 {
        print $"✓ PASSED: ($name) \(($duration)s\)\n"
        {name: $name, status: "PASSED", duration: $duration}
    } else {
        print $"✗ FAILED: ($name) \(($duration)s\)"
        print $"  Error: ($result.stderr)\n"
        {name: $name, status: "FAILED", duration: $duration}
    }
}

# 1. Build Verification Tests
print "\n[1/8] Build Verification"
print "─────────────────────────────────────────────────────────"
$results = ($results | append (run-test "Cargo Check" { cargo check --all-features }))
$results = ($results | append (run-test "Build Verification Tests" { cargo test --test build_verification }))

# 2. Unit Tests (if any exist)
print "\n[2/8] Unit Tests"
print "─────────────────────────────────────────────────────────"
$results = ($results | append (run-test "Unit Tests" { cargo test --lib --all-features }))

# 3. Documentation Tests
print "\n[3/8] Documentation Tests"
print "─────────────────────────────────────────────────────────"
$results = ($results | append (run-test "Doc Tests" { cargo test --doc --all-features }))

# 4. Basic Integration Tests
print "\n[4/8] Basic Integration Tests"
print "─────────────────────────────────────────────────────────"
$results = ($results | append (run-test "P2P Integration Tests" {
    cargo test --test p2p_integration_tests -- --test-threads=1
}))

# 5. Advanced DHT Tests
print "\n[5/8] Advanced DHT Tests"
print "─────────────────────────────────────────────────────────"
$results = ($results | append (run-test "DHT Advanced Tests" {
    cargo test --test dht_advanced_tests -- --test-threads=1
}))

# 6. Chat Integration Tests
print "\n[6/8] Chat Integration Tests"
print "─────────────────────────────────────────────────────────"
$results = ($results | append (run-test "Chat Integration Tests" {
    cargo test --test chat_integration_tests -- --test-threads=1
}))

# 7. WASM Compilation Tests
print "\n[7/8] WASM Compilation Tests"
print "─────────────────────────────────────────────────────────"
$results = ($results | append (run-test "WASM Compilation" {
    cargo test --test wasm_compilation
}))

# 8. Examples Verification
print "\n[8/8] Examples Verification"
print "─────────────────────────────────────────────────────────"
$results = ($results | append (run-test "Examples Build" {
    cargo build --examples --all-features
}))

# Summary
print "\n═══════════════════════════════════════════════════════════"
print "  Test Summary"
print "═══════════════════════════════════════════════════════════\n"

let passed = ($results | where status == "PASSED" | length)
let failed = ($results | where status == "FAILED" | length)
let total = ($results | length)

print $"Total: ($total) | Passed: ($passed) | Failed: ($failed)\n"

if $failed > 0 {
    print "Failed tests:"
    $results | where status == "FAILED" | each {|test|
        print $"  - ($test.name)"
    }
    print ""
}

print "Detailed Results:"
$results | select name status duration | table

# Exit with error if any tests failed
if $failed > 0 {
    exit 1
} else {
    print "\n✓ All tests passed!"
    exit 0
}
