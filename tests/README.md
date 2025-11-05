# Netabase Integration Tests

This directory contains integration tests for netabase's peer-to-peer functionality.

## Overview

The P2P integration tests verify that multiple netabase nodes can communicate and synchronize data across a network. These tests use a helper binary (`test_node`) that can be controlled via JSON commands over stdin/stdout to simulate real inter-process communication.

## Test Structure

### `test_node.rs`
A standalone test binary that acts as a controllable netabase node. It:
- Accepts JSON commands via stdin
- Sends JSON responses via stdout
- Manages its own netabase instance and event loop
- Can be spawned as a subprocess for integration testing

**Commands supported:**
- `StartSwarm` - Initialize the P2P swarm
- `WaitForPeers` - Wait for peer discovery
- `PutRecord` - Store a record in the DHT
- `GetRecord` - Retrieve a record
- `QueryLocal` - Query locally stored records
- `StartProviding` - Advertise as a provider for a key
- `GetProviders` - Find providers for a key
- `Bootstrap` - Join the DHT network
- `GetPeers` - Get connected peers
- `GetPeerId` - Get local peer ID
- `Shutdown` - Clean shutdown

### `p2p_integration_tests.rs`
Integration tests that spawn multiple `test_node` processes and verify P2P functionality:

1. **`test_mdns_peer_discovery`** - Verifies that two nodes can discover each other via mDNS
2. **`test_local_record_storage`** - Tests local record storage and retrieval
3. **`test_distributed_record_storage`** - Tests DHT record storage across peers
4. **`test_provider_records`** - Tests provider record functionality
5. **`test_bootstrap`** - Tests DHT bootstrap functionality

## Running the Tests

### Quick Start (Using nushell script)

The easiest way to run the P2P tests is using the provided nushell script:

```bash
cd netabase
nu run_p2p_tests.nu
```

Options:
```bash
# Run with verbose logging
nu run_p2p_tests.nu --verbose

# Run a specific test
nu run_p2p_tests.nu --test test_mdns_peer_discovery
```

### Manual Execution

#### 1. Build the test binaries:
```bash
# Build test_node
cargo test --test test_node --no-run

# Build integration tests
cargo test --test p2p_integration_tests --no-run
```

#### 2. Run the integration tests:
```bash
# Run all P2P integration tests
cargo test --test p2p_integration_tests -- --ignored --nocapture

# Run a specific test
cargo test --test p2p_integration_tests -- --ignored --nocapture test_mdns_peer_discovery

# Run with debug logging
RUST_LOG=debug cargo test --test p2p_integration_tests -- --ignored --nocapture
```

**Note:** These tests are marked with `#[ignore]` because they:
- Spawn multiple processes
- Require network access
- Take longer to run (30+ seconds)
- May be flaky in CI environments without proper network setup

### Running test_node Standalone

You can also run `test_node` directly for manual testing:

```bash
# Run test_node with a name
cargo test --test test_node -- node1

# Then send JSON commands via stdin:
{"StartSwarm":""}
{"PutRecord":{"id":"test1","data":"hello"}}
{"GetRecord":{"id":"test1"}}
{"Shutdown":""}
```

## Test Architecture

```
┌─────────────────────────────────────┐
│  p2p_integration_tests.rs           │
│  (Test Coordinator)                 │
└────────┬────────────────────────┬───┘
         │                        │
         │ spawns                 │ spawns
         ▼                        ▼
┌─────────────────┐      ┌─────────────────┐
│  test_node      │      │  test_node      │
│  (Process 1)    │◄────►│  (Process 2)    │
│                 │      │                 │
│  ┌───────────┐  │ P2P  │  ┌───────────┐  │
│  │ Netabase  │  │◄────►│  │ Netabase  │  │
│  │ Instance  │  │      │  │ Instance  │  │
│  └───────────┘  │      │  └───────────┘  │
│                 │      │                 │
│  stdin/stdout   │      │  stdin/stdout   │
└─────────────────┘      └─────────────────┘
         ▲                        ▲
         │                        │
         └────────────┬───────────┘
                      │
              JSON commands/responses
```

## What Each Test Verifies

### 1. mDNS Peer Discovery
- **Goal:** Verify automatic peer discovery on local networks
- **Process:**
  - Spawn two nodes
  - Start their swarms
  - Wait up to 30 seconds for mDNS discovery
- **Success:** Both nodes discover each other

### 2. Local Record Storage
- **Goal:** Verify basic record storage and retrieval
- **Process:**
  - Spawn single node
  - Store a record
  - Query local storage
  - Retrieve specific record
- **Success:** Record is stored and retrieved correctly

### 3. Distributed Record Storage
- **Goal:** Verify DHT record propagation
- **Process:**
  - Spawn two connected nodes
  - Node 1 stores a record
  - Verify DHT behavior (records not auto-replicated)
- **Success:** Demonstrates DHT storage semantics

### 4. Provider Records
- **Goal:** Verify provider advertisement and discovery
- **Process:**
  - Spawn two connected nodes
  - Node 1 advertises as provider
  - Node 2 searches for providers
- **Success:** Provider records propagate through DHT

### 5. Bootstrap
- **Goal:** Verify DHT bootstrap functionality
- **Process:**
  - Spawn single node
  - Trigger bootstrap
- **Success:** Bootstrap completes without error

## Troubleshooting

### Tests Timeout
- **Cause:** mDNS may be slow or blocked on some networks
- **Solution:** Increase timeout in test code or run on a local network

### Port Conflicts
- **Cause:** Multiple test runs without cleanup
- **Solution:**
  ```bash
  # Clean up test data
  rm -rf ./test_data

  # Kill any stray processes
  pkill -f test_node
  ```

### Compilation Errors
- **Cause:** Missing dependencies
- **Solution:** Ensure `serde_json` and `env_logger` are in dev-dependencies

### Process Spawn Failures
- **Cause:** test_node binary not built
- **Solution:** Run `cargo test --test test_node --no-run` first

## Future Improvements

- [ ] Add tests for record retrieval from DHT (not just local storage)
- [ ] Add tests for network partitions and recovery
- [ ] Add tests for large numbers of nodes (stress testing)
- [ ] Add tests for malicious peer handling
- [ ] Add benchmarks for DHT performance
- [ ] Add tests for cross-platform compatibility (Linux, macOS, Windows)
- [ ] Add tests for WASM compatibility (when supported)

## Contributing

When adding new P2P tests:

1. Document what the test verifies
2. Add appropriate timeouts (P2P operations can be slow)
3. Clean up resources in test teardown
4. Mark test with `#[ignore]` if it requires network access
5. Update this README with the new test description

## Related Documentation

- [Netabase Architecture](../ARCHITECTURE.md) - Design overview
- [Getting Started](../GETTING_STARTED.md) - Basic usage guide
- [Examples Guide](../../EXAMPLES_GUIDE.md) - Example applications
