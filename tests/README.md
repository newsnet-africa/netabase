# Netabase Test Suite

This directory contains the comprehensive test suite for Netabase, organized into logical modules with proper isolation and single-threaded execution support to avoid race conditions, particularly with Sled database operations.

## 📁 Test Organization

```
tests/
├── README.md              # This file
├── lib.rs                 # Main test library with common utilities
├── tests.toml             # Test configuration file
├── common/                # Shared test utilities
│   ├── mod.rs             # Common utilities and helpers
│   └── test_runner.rs     # Single-threaded test runner
├── unit/                  # Unit tests for individual components
│   ├── mod.rs             # Unit test module declaration
│   └── handler_tests.rs   # Handler component tests
├── integration/           # Integration tests for multi-component functionality
│   ├── mod.rs             # Integration test module declaration
│   ├── integration_tests.rs  # Basic integration tests
│   └── multi_process_tests.rs # Multi-process communication tests
├── kademlia/              # Kademlia DHT specific tests
│   ├── mod.rs             # Kademlia test module declaration
│   ├── kademlia_memory_test.rs  # Memory-based Kademlia tests
│   └── kademlia_interprocess_messaging.rs # Interprocess messaging tests
└── sled/                  # Sled database persistence tests
    ├── mod.rs             # Sled test module declaration
    └── kademlia_sled_test.rs # Sled-based Kademlia persistence tests
```

## 🧪 Test Categories

### Unit Tests (`unit/`)
- **Purpose**: Test individual components in isolation
- **Execution**: Fast, parallel execution where possible
- **Focus**: Component functionality, method behavior, error handling

### Integration Tests (`integration/`)
- **Purpose**: Test component interactions and end-to-end workflows
- **Execution**: Sequential execution with shared resources
- **Focus**: Multi-component integration, API contracts, data flow

### Kademlia Tests (`kademlia/`)
- **Purpose**: Test Kademlia DHT functionality and network behavior
- **Execution**: Memory-based, can run in parallel
- **Focus**: Peer discovery, data sharing, provider functionality

### Sled Tests (`sled/`)
- **Purpose**: Test Sled database persistence and data integrity
- **Execution**: **MUST run single-threaded** due to Sled limitations
- **Focus**: Data persistence, database integrity, crash recovery

## 🚀 Running Tests

### Quick Test Script
Use the provided test runner for single-threaded execution:

```bash
# Run all Sled tests (recommended)
./run_sled_tests.sh --suite sled

# Run all test suites
./run_sled_tests.sh --suite all

# Run with verbose logging
./run_sled_tests.sh --suite sled --verbose

# Run specific test suite
./run_sled_tests.sh --suite integration
```

### Manual Cargo Commands

#### Sled Tests (Single-threaded required)
```bash
# Full Sled persistence test
cargo test --features native test_kademlia_sled_persistence -- --nocapture --test-threads=1

# Quick Sled validation
cargo test --features native test_minimal_two_node_sled -- --nocapture --test-threads=1

# All Sled tests
cargo test --features native sled -- --nocapture --test-threads=1
```

#### Kademlia Tests (Memory-based)
```bash
# Memory-based Kademlia tests
cargo test --features memory test_kademlia_memory_swarm -- --nocapture

# Interprocess messaging
cargo test --features memory test_interprocess_messaging -- --nocapture
```

#### Integration Tests
```bash
# Basic integration tests
cargo test --features native integration_tests -- --nocapture

# Multi-process tests
cargo test --features native multi_process_tests -- --nocapture
```

#### Unit Tests
```bash
# Handler tests
cargo test --features native handler_tests -- --nocapture

# All unit tests
cargo test --features native unit -- --nocapture
```

## 🔧 Test Configuration

### Environment Variables
- `RUST_LOG`: Set to `debug` for verbose logging, `info` for standard output
- `RUST_BACKTRACE`: Set to `1` for stack traces on panics
- `NETABASE_TEST_MODE`: Automatically set to `true` during test execution

### Features
- `native`: Full native functionality with TCP, mDNS, and Sled
- `memory`: Memory-only storage for fast testing
- `wasm`: WebAssembly target support

### Timeouts
- **Unit tests**: 30-60 seconds
- **Integration tests**: 5-10 minutes  
- **Sled tests**: 10-15 minutes
- **Kademlia tests**: 5-8 minutes

## 📊 Test Structure

### Common Utilities (`common/`)

#### `mod.rs`
- `init_test_logger()`: Initialize logging for tests
- `init_debug_logger()`: Initialize debug-level logging
- `create_temp_db_dir()`: Create temporary database directory
- `generate_test_id()`: Generate unique test identifiers
- `TestResults`: Track test execution results
- `TestNodeConfig`: Configuration for multi-node tests

#### `test_runner.rs`
- `SingleThreadTestRunner`: Enforces single-threaded execution
- `TestConfig`: Test execution configuration
- `TestResult`: Test execution results
- `run_quick_test()`: Run tests without full isolation

### Macros
The test library provides convenient macros for different test types:

```rust
// Single-threaded Sled test
sled_test!(my_sled_test, {
    // Test implementation
});

// Integration test
integration_test!(my_integration_test, {
    // Test implementation  
});

// Unit test
unit_test!(my_unit_test, {
    // Test implementation
});
```

## 🗃️ Sled Database Tests

The Sled tests are the primary focus of this test suite reorganization. They verify:

1. **Data Persistence**: Ensure data survives application restarts
2. **Cross-node Communication**: Verify DHT data sharing between nodes
3. **Database Integrity**: Check data consistency and checksums
4. **Provider Functionality**: Test data provision and retrieval
5. **Concurrent Access**: Verify single-threaded access patterns

### Key Features
- **Comprehensive Logging**: Detailed logs for debugging persistence issues
- **Database Verification**: Direct Sled database inspection
- **Checksum Validation**: Data integrity verification using MD5 and CRC32
- **Temporal Verification**: Timestamp-based data validation
- **Cleanup Management**: Automatic cleanup of test databases

### Test Data Models
```rust
// Test message with persistence markers
pub struct SledTestMessage {
    pub id: u64,
    pub content: String,
    pub sender_node: String,
    pub receiver_node: Option<String>,
    pub timestamp: u64,
    pub test_phase: String,
    pub message_size: usize,
    pub persistence_marker: String,
}

// Provider test with hash verification
pub struct SledProviderTest {
    pub provider_id: String,
    pub data: String,
    pub node_id: String,
    pub timestamp: u64,
    pub data_hash: String,
    pub verification_token: String,
}

// Persistence test with checksum
pub struct SledPersistenceTest {
    pub persistence_id: String,
    pub test_data: Vec<u8>,
    pub creation_time: u64,
    pub node_info: String,
    pub checksum: u32,
}
```

## 🔍 Debugging Test Failures

### Logging Levels
- `RUST_LOG=error`: Only show errors
- `RUST_LOG=warn`: Show warnings and errors
- `RUST_LOG=info`: Standard test output (default)
- `RUST_LOG=debug`: Detailed debugging information
- `RUST_LOG=trace`: Maximum verbosity

### Common Issues

#### Sled Database Conflicts
**Symptom**: Tests fail with database lock errors
**Solution**: Ensure single-threaded execution with `--test-threads=1`

#### Peer Discovery Timeouts
**Symptom**: mDNS peer discovery fails
**Solution**: Increase timeouts or run tests in isolated environment

#### Database Persistence Issues
**Symptom**: Data not found after storage
**Solution**: Check database paths and ensure proper cleanup between tests

#### Memory Issues
**Symptom**: Tests fail with out-of-memory errors
**Solution**: Reduce number of test nodes or increase system memory

### Test Data Inspection
The Sled tests include database verification functions that allow direct inspection of stored data:

```rust
// Verify data persistence in Sled database
verify_sled_persistence(&db_path, &expected_keys).await
```

## 📈 Performance Considerations

### Test Execution Times
- **Unit tests**: < 1 minute total
- **Integration tests**: 2-5 minutes total
- **Kademlia tests**: 5-10 minutes total
- **Sled tests**: 10-20 minutes total

### Resource Usage
- **Memory**: 500MB-2GB depending on test suite
- **Disk**: 100MB-1GB for temporary databases
- **Network**: Local networking only (mDNS, loopback)

### Optimization Tips
1. Run unit tests first for quick feedback
2. Use `--suite sled` for focused persistence testing
3. Enable verbose logging only when debugging
4. Clean up test databases regularly
5. Monitor system resources during long test runs

## 🔒 Test Isolation

### Database Isolation
- Each test uses a unique temporary directory
- Automatic cleanup after test completion
- No shared database instances between tests

### Network Isolation
- Tests use local networking (127.0.0.1, mDNS)
- Each test uses unique ports where applicable
- No external network dependencies

### Process Isolation
- Single-threaded execution prevents race conditions
- Proper cleanup between test executions
- Resource limits enforced per test

## 📚 Adding New Tests

### Creating a Unit Test
```rust
use crate::unit_test;

unit_test!(test_my_component, {
    // Your test implementation here
    assert!(true);
});
```

### Creating a Sled Test
```rust
use crate::sled_test;
use crate::{init_debug_logger, create_temp_db_dir};

sled_test!(test_my_sled_feature, {
    let temp_dir = create_temp_db_dir();
    let db_path = temp_dir.path().join("test_db");
    
    // Your Sled test implementation
    
    // Database will be automatically cleaned up
});
```

### Creating an Integration Test
```rust
use crate::integration_test;
use crate::{TEST_TIMEOUT, init_test_logger};

integration_test!(test_my_integration, {
    init_test_logger();
    
    // Your integration test implementation
    
    Ok(())
});
```

## 🎯 Best Practices

1. **Always use single-threaded execution for Sled tests**
2. **Include comprehensive logging for debugging**
3. **Verify data persistence, not just in-memory operations**
4. **Use appropriate timeouts for different operations**
5. **Clean up resources properly after tests**
6. **Test both success and failure scenarios**
7. **Include checksums and verification for data integrity**
8. **Use meaningful test data that aids in debugging**

## 📞 Support

If you encounter issues with the test suite:

1. Check the test logs for detailed error information
2. Verify you're using single-threaded execution for Sled tests
3. Ensure sufficient system resources are available
4. Review the test configuration in `tests.toml`
5. Run tests individually to isolate issues

For questions or improvements, please refer to the project documentation or submit an issue.