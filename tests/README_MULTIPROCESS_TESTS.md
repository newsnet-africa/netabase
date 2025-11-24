# Multi-Process Chat Example Tests

This directory contains multi-process integration tests for the chat example.

## Test Files

### 1. `chat_multiprocess_test.rs` - Rust Integration Tests
Programmatic tests that spawn multiple Netabase instances and verify:
- Single node message storage
- Message persistence across sessions
- Multiple users with separate databases
- Concurrent writes
- Peer discovery via mDNS

**Status**: ⚠️ Currently blocked by incomplete Sled RecordStore implementation

To run when Sled is fixed:
```bash
cargo test --test chat_multiprocess_test --all-features
```

### 2. `test_chat_example.sh` - Bash Test Script
Automated bash script that tests the interactive chat example by:
- Spawning multiple chat instances in parallel
- Piping commands via stdin
- Verifying message storage and retrieval
- Testing peer discovery
- Checking database persistence

To run:
```bash
chmod +x tests/test_chat_example.sh
./tests/test_chat_example.sh
```

### 3. `test_chat_example.nu` - Nushell Test Script
Same functionality as the bash script but written in Nushell for better structured output and error handling.

To run:
```bash
chmod +x tests/test_chat_example.nu
nu tests/test_chat_example.nu
```

## Current Limitation

**The tests are currently blocked** because the Sled RecordStore implementation in `netabase_store` is incomplete:

- `netabase_store/src/databases/sled_store.rs` exists but doesn't implement `libp2p::kad::store::RecordStore`
- The code in `src/network/store.rs` expects this implementation when the `sled` feature is enabled
- Without it, all `put` operations fail with `Error::MaxRecords`

### What Needs to be Done

To fix the tests, implement RecordStore for SledStore in `netabase_store`:

```rust
// In netabase_store/src/databases/sled_store.rs
#[cfg(feature = "libp2p")]
impl<D> libp2p::kad::store::RecordStore for SledStore<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec,
    // ... other bounds
{
    // Implement: get, put, remove, records, providers, etc.
}
```

## Workaround for Now

Until Sled RecordStore is implemented, you can test the sync and networking functionality using the existing tests:

```bash
# Run existing paxos tests
cargo test --features paxos --tests

# Run sync API tests
cargo test sync_api

# Test peer discovery (works without storage)
cargo test --test chat_multiprocess_test test_peer_discovery
```

## Test Features

When working, the tests demonstrate:

### Rust Tests (`chat_multiprocess_test.rs`)
- ✅ Programmatic control of multiple Netabase instances
- ✅ Async/await test patterns with tokio
- ✅ Temporary database creation and cleanup
- ✅ Message storage and retrieval verification
- ✅ Multi-user isolation testing
- ✅ Persistence across sessions

### Shell Scripts (`test_chat_example.sh`, `test_chat_example.nu`)
- ✅ Real executable testing (tests the actual binary)
- ✅ Interactive stdin/stdout automation
- ✅ Parallel process spawning
- ✅ Output verification with grep/pattern matching
- ✅ Colored terminal output for test results
- ✅ Automatic cleanup on exit

## Expected Test Output (When Fixed)

```bash
=== Chat Example Multi-Process Test ===

Building example...
✓ Build complete

Test 1: Single node message storage
✓ Messages stored successfully
✓ All 3 messages retrieved from history

Test 2: Multiple nodes with separate databases
✓ Bob has correct message count
✓ Charlie has correct message count

Test 3: Message persistence across sessions
✓ Messages persisted across sessions

Test 4: Peer discovery
✓ Peer discovery working

Test 5: Database directory structure
✓ Database directories created

=== All Tests Completed ===
```

## Contributing

When implementing the Sled RecordStore:

1. Add the RecordStore trait implementation to `netabase_store/src/databases/sled_store.rs`
2. Ensure feature gates match: `#[cfg(all(feature = "sled", feature = "libp2p"))]`
3. Run the tests to verify: `cargo test --test chat_multiprocess_test --all-features`
4. Update this README to remove the limitation notice
