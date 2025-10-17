# Multi-Process Kademlia Sled Database Tests

This directory contains comprehensive multi-process tests for the Netabase Kademlia DHT system using the Sled database backend. These tests verify that the distributed system works correctly across multiple processes with persistent storage.

## Overview

The multi-process tests simulate real-world scenarios where multiple Netabase instances run in separate processes, discover each other via mDNS, and exchange data through the Kademlia DHT while persisting everything to Sled databases.

## Test Components

### 1. Sender Process (`multiprocess_sender.rs`)
- **Purpose**: Discovers peers and sends test data to the network
- **Behavior**:
  - Starts a Netabase instance with unique Sled database
  - Waits for mDNS peer discovery
  - Automatically connects to discovered receivers
  - Sends multiple test messages to the DHT
  - Verifies data persistence in local database
- **Database Verification**: Uses `get()` method to verify messages are stored locally

### 2. Receiver Process (`multiprocess_receiver.rs`)
- **Purpose**: Receives and processes data from senders
- **Behavior**:
  - Starts a Netabase instance with unique Sled database
  - Waits for peer connections
  - Monitors for incoming data from the DHT
  - Prints received connections and data details
  - Verifies data persistence in local database
- **Database Verification**: Uses `get()` method to verify received messages are persisted

### 3. Late Joiner Process (`multiprocess_late_joiner.rs`)
- **Purpose**: Tests that new users can retrieve data after joining the network late
- **Behavior**:
  - Waits 20 seconds before starting (to join after data is established)
  - Connects to existing peers
  - Performs periodic data retrieval cycles
  - Tests DHT data availability for late-joining nodes
  - Verifies retrieved data persistence
- **Database Verification**: Uses `get()` method to verify retrieved historical data

## Running the Tests

### Automatic Parallel Execution (Recommended)

Use the provided script to run all tests simultaneously:

```bash
./run_multiprocess_sled_tests.sh all
```

This will:
- Start all three processes in parallel
- Coordinate timing automatically
- Provide colored output and progress monitoring
- Save logs to timestamped directories
- Show comprehensive results summary

### Individual Test Execution

Run individual components for debugging:

```bash
# Run sender only
./run_multiprocess_sled_tests.sh sender

# Run receiver only  
./run_multiprocess_sled_tests.sh receiver

# Run late joiner only
./run_multiprocess_sled_tests.sh late
```

### Sequential Execution

For debugging or when parallel execution isn't suitable:

```bash
./run_multiprocess_sled_tests.sh sequential
```

### Manual Execution

For direct cargo test execution:

```bash
# Must use --test-threads=1 to avoid Sled conflicts
cargo test --features native test_multiprocess_sender --test multiprocess_sender -- --nocapture --test-threads=1
cargo test --features native test_multiprocess_receiver --test multiprocess_receiver -- --nocapture --test-threads=1
cargo test --features native test_multiprocess_late_joiner --test multiprocess_late_joiner -- --nocapture --test-threads=1
```

## Test Architecture

### Database Isolation
- Each process uses a unique Sled database path in `/tmp`
- Prevents database conflicts between processes
- Allows concurrent execution while maintaining data integrity

### Network Configuration
- **Sender**: Port 9001
- **Receiver**: Port 9002  
- **Late Joiner**: Port 9003
- Uses mDNS for automatic peer discovery

### Data Schema
All processes use the same schema (`MultiProcessSchema`) containing:

#### `SenderData`
- Primary key: `message_id` (u64)
- Content: Message text, sender info, timestamps, sequence numbers
- Used for actual test data transmission

#### `NetworkEvent`
- Primary key: `event_id` (String)
- Tracks network events like peer discovery, connections
- Used for debugging and test verification

#### `ReceiverEvent` / `LateJoinerEvent`
- Process-specific event tracking
- Records reception cycles, message counts, timing info

### Database Verification Strategy

Each test performs multi-level verification:

1. **DHT Storage**: Messages stored via `netabase.put()`
2. **Local Retrieval**: Messages retrieved via `netabase.get()`
3. **Persistence Check**: Direct database access verification
4. **Content Validation**: Exact message content comparison

## Test Flow

### Timeline
```
T+0s:    Receiver starts, waits for connections
T+3s:    Sender starts, begins peer discovery
T+20s:   Late joiner starts (built-in delay)
T+25s:   Sender begins transmitting data
T+35s:   Receiver processes incoming data
T+45s:   Late joiner begins periodic retrieval
T+120s:  All tests complete
```

### Peer Discovery
- All processes use mDNS for automatic discovery
- No manual configuration required
- Simulates real-world network conditions

### Data Flow
1. Sender discovers receiver via mDNS
2. Sender transmits 5 test messages to DHT
3. Receiver monitors and processes incoming data
4. Late joiner connects and retrieves historical data
5. All processes verify database persistence

## Expected Outcomes

### Success Criteria
- ✅ All processes discover each other via mDNS
- ✅ Sender successfully transmits all test messages
- ✅ Receiver receives and verifies message content
- ✅ Late joiner retrieves existing data from network
- ✅ All data persists correctly in local Sled databases
- ✅ Database `get()` operations return expected content

### Typical Results
- **Sender**: Discovers 1-2 peers, sends 5 messages, 100% local verification
- **Receiver**: Receives 3-5 messages, 100% persistence verification  
- **Late Joiner**: Finds 1-2 existing peers, retrieves 2-5 historical messages

## Troubleshooting

### Common Issues

#### No Peers Discovered
- **Cause**: mDNS may be slow or blocked
- **Solution**: Tests continue anyway, data may still propagate via DHT
- **Check**: Firewall/network settings allowing mDNS

#### Sled Database Conflicts
- **Cause**: Multiple processes accessing same database
- **Solution**: Always use `--test-threads=1`
- **Check**: Unique database paths in logs

#### No Messages Retrieved
- **Cause**: Timing issues or network propagation delays
- **Solution**: Increase wait times, check process startup order
- **Check**: Test logs for actual message IDs and timing

### Debug Information

Each test produces extensive logging:
- Peer discovery events
- Message transmission/reception
- Database operations
- Timing information
- Error details

Logs are saved to `multiprocess_test_logs_YYYYMMDD_HHMMSS/` directories.

### Environment Requirements
- **OS**: Linux (mDNS support required)
- **Network**: Local network with mDNS enabled
- **Disk**: Write access to `/tmp` for Sled databases
- **Memory**: ~100MB per process for test data

## Integration with Main Test Suite

These tests are part of the larger Netabase test infrastructure:

- Use the same test runner patterns as other Sled tests
- Follow the established logging and verification conventions
- Integrate with existing cleanup and database management
- Compatible with the single-threaded execution requirements

## Future Enhancements

Potential improvements for these tests:
- Network partition simulation
- Stress testing with larger datasets
- Performance benchmarking
- Multi-node scaling tests
- Byzantine fault tolerance testing

## Related Documentation

- `../README.md` - General test suite overview
- `../../SLED_KADEMLIA_TEST_SUMMARY.md` - Sled test migration summary
- `../../run_sled_tests.sh` - Single-process Sled test runner
- `../../INTERPROCESS_TESTING.md` - Broader interprocess testing guide