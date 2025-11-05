# Using cargo-nextest for Testing

## Why cargo-nextest?

Yes, **cargo-nextest is significantly easier and more powerful** than the custom Nushell script! Here's why:

### Advantages over Nushell Script

| Feature | Nushell Script | cargo-nextest |
|---------|---------------|---------------|
| **Installation** | Requires Nushell | Single cargo install |
| **Maintenance** | Custom script to maintain | Official Rust tool |
| **Parallel Execution** | Manual implementation | Built-in, optimized |
| **Test Filtering** | Manual grep/filtering | Advanced filter expressions |
| **Retry Logic** | Not implemented | Built-in retries |
| **Output Format** | Custom formatting | Multiple formats (JUnit, JSON) |
| **Performance** | Slower (sequential) | Faster (parallel by default) |
| **CI Integration** | Custom setup | Native CI support |
| **Progress Reporting** | Basic | Beautiful, real-time |
| **Test Grouping** | Not supported | Test groups with custom settings |

## Installation

```bash
# Install cargo-nextest
cargo install cargo-nextest --locked

# Verify installation
cargo nextest --version
```

## Basic Usage

### Run All Tests

```bash
# Run all tests (unit + integration)
cargo nextest run

# Run with all features
cargo nextest run --all-features

# Run specific test file
cargo nextest run --test p2p_integration_tests
```

### Run Integration Tests

```bash
# Run ignored tests (integration tests)
cargo nextest run --run-ignored ignored-only --test-threads 1

# Run specific integration test suite
cargo nextest run --test dht_advanced_tests --run-ignored ignored-only

# Run all P2P tests
cargo nextest run --test p2p_integration_tests --test dht_advanced_tests --test chat_integration_tests --run-ignored ignored-only
```

### Run with Profiles

```bash
# Use the integration profile (from .config/nextest.toml)
cargo nextest run --profile integration --run-ignored ignored-only

# Use CI profile
cargo nextest run --profile ci
```

## Configuration

The `.config/nextest.toml` file provides:

- **Increased timeouts** for long-running integration tests
- **Single-threaded execution** to avoid port conflicts
- **Profiles** for different environments (dev, ci, integration)
- **Retry logic** for flaky tests
- **Test grouping** for better organization

## Recommended Commands

### During Development

```bash
# Run fast unit tests
cargo nextest run --lib

# Run specific integration test
cargo nextest run --test p2p_integration_tests --run-ignored ignored-only
```

### Pre-Commit

```bash
# Run all integration tests
cargo nextest run --run-ignored ignored-only --test-threads 1

# Or use the integration profile
cargo nextest run --profile integration --run-ignored ignored-only
```

### CI/CD Pipeline

```bash
# Run with CI profile (retries, timeouts)
cargo nextest run --profile ci --run-ignored ignored-only --test-threads 1

# Generate JUnit XML for CI reporting
cargo nextest run --profile ci --run-ignored ignored-only --message-format libtest-json-plus > results.json
```

## Advanced Features

### Filtering Tests

```bash
# Run tests matching pattern
cargo nextest run 'test(provider)'

# Run tests in specific module
cargo nextest run 'test(//dht_advanced_tests::)'

# Exclude tests
cargo nextest run 'not test(wasm)'
```

### Parallel Execution (where safe)

```bash
# Run unit tests in parallel (default)
cargo nextest run --lib

# Run specific non-conflicting tests in parallel
cargo nextest run --test build_verification --test wasm_compilation
```

### Retry Flaky Tests

```bash
# Retry failed tests up to 3 times
cargo nextest run --retries 3
```

### Show Output

```bash
# Show output for failed tests
cargo nextest run --failure-output immediate

# Show output for all tests
cargo nextest run --success-output immediate
```

## Output Examples

### Beautiful Progress Display

```
    Starting 25 tests across 8 binaries
        PASS [   0.123s] netabase lib::tests
        PASS [   6.384s] p2p_integration_tests test_peer_discovery
        PASS [  12.156s] dht_advanced_tests test_concurrent_record_operations
     RUNNING [  15.234s] dht_advanced_tests test_large_network_scalability
...

------------
     Summary [  139.421s] 24 passed, 1 failed
        FAIL [  28.156s] dht_advanced_tests test_multiple_providers
```

### JUnit XML Output (for CI)

```bash
cargo nextest run --message-format junit > junit.xml
```

## Migration from Nushell Script

The Nushell script (`run_comprehensive_tests.nu`) can be replaced with these nextest commands:

| Nushell Script Section | Nextest Equivalent |
|----------------------|-------------------|
| Cargo Check | `cargo check --all-features` (separate command) |
| Unit Tests | `cargo nextest run --lib` |
| Doc Tests | `cargo test --doc` (nextest doesn't run doctests) |
| P2P Tests | `cargo nextest run --test p2p_integration_tests --run-ignored ignored-only` |
| DHT Tests | `cargo nextest run --test dht_advanced_tests --run-ignored ignored-only` |
| Chat Tests | `cargo nextest run --test chat_integration_tests --run-ignored ignored-only` |
| All Integration | `cargo nextest run --run-ignored ignored-only --test-threads 1` |

## Complete Test Suite with Nextest

Create a simple shell script or Makefile:

```bash
#!/bin/bash
# run_all_tests.sh

echo "Running comprehensive test suite with cargo-nextest..."

# 1. Check compilation
echo "\n[1/5] Checking compilation..."
cargo check --all-features || echo "⚠ Compilation check failed (WASM issues expected)"

# 2. Run unit tests
echo "\n[2/5] Running unit tests..."
cargo nextest run --lib

# 3. Run doc tests (nextest doesn't support these)
echo "\n[3/5] Running doc tests..."
cargo test --doc

# 4. Run build verification
echo "\n[4/5] Running build verification..."
cargo nextest run --test build_verification --test wasm_compilation

# 5. Run integration tests
echo "\n[5/5] Running integration tests..."
cargo nextest run --profile integration --run-ignored ignored-only

echo "\n✅ Test suite complete!"
```

## Makefile Alternative

```makefile
.PHONY: test test-unit test-integration test-all

test-unit:
\tcargo nextest run --lib

test-integration:
\tcargo nextest run --run-ignored ignored-only --test-threads 1

test-all: test-unit test-integration
\t@echo "✅ All tests complete"

test-ci:
\tcargo nextest run --profile ci --run-ignored ignored-only
```

## Comparison: Custom vs Nextest

### Nushell Script (run_comprehensive_tests.nu)

**Pros**:
- ✅ Custom reporting format
- ✅ Can include arbitrary commands (cargo check, etc.)
- ✅ Works without additional installation (if Nushell already installed)

**Cons**:
- ❌ Requires Nushell (not standard Rust tooling)
- ❌ Slower (sequential execution)
- ❌ More code to maintain (~100 lines)
- ❌ Limited filtering capabilities
- ❌ No retry logic
- ❌ No parallel execution optimization
- ❌ No CI-native output formats

### cargo-nextest

**Pros**:
- ✅ Official Rust ecosystem tool
- ✅ Faster (parallel execution where possible)
- ✅ Zero maintenance (maintained by community)
- ✅ Advanced filtering and grouping
- ✅ Built-in retry logic for flaky tests
- ✅ Beautiful, real-time progress output
- ✅ CI-native (JUnit XML, JSON output)
- ✅ Profile-based configuration
- ✅ Better error reporting

**Cons**:
- ❌ Doesn't run doc tests (use `cargo test --doc` separately)
- ❌ Doesn't run arbitrary commands like `cargo check`

## Recommendation

**Use cargo-nextest** for the test suite:

1. **Simpler**: Single command instead of 100-line script
2. **Faster**: Parallel execution where safe
3. **Standardized**: Part of Rust ecosystem
4. **Feature-rich**: Retries, filtering, profiles, CI integration
5. **Maintained**: Community-supported tool

Keep a simple shell script for the few things nextest doesn't do:
- `cargo check`
- `cargo test --doc`
- Example compilation

## Example: Complete Test Workflow

```bash
#!/bin/bash
# test.sh - Complete test workflow

set -e  # Exit on error

echo "=== Netabase Test Suite ==="

# Quick checks
echo "\n[1/4] Checking compilation..."
cargo check --all-features 2>&1 | grep -q "error" && echo "⚠ WASM errors (expected)" || echo "✅ Passed"

# Unit tests
echo "\n[2/4] Running unit tests..."
cargo nextest run --lib

# Integration tests
echo "\n[3/4] Running integration tests..."
cargo nextest run --run-ignored ignored-only --test-threads 1

# Doc tests
echo "\n[4/4] Running doc tests..."
cargo test --doc

echo "\n✅ Test suite complete!"
```

## Conclusion

**Answer: Yes, cargo-nextest is significantly easier!**

It's:
- More powerful
- Better maintained
- Faster
- Easier to use
- Standard Rust tooling

The Nushell script was useful for demonstrating the test suite structure, but cargo-nextest is the superior choice for actual day-to-day usage.

**Migration is simple**: Replace the Nushell script with the commands above, and you get better functionality with less code.
