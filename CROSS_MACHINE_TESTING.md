# Cross-Machine Network Testing

This document explains how to run NetaBase tests across two or more machines on the same network to verify the distributed hash table (DHT) functionality.

## Overview

The cross-machine tests allow you to:
- Run a **writer node** on one machine that stores records in the DHT
- Run a **reader node** on another machine that retrieves those records
- Verify that the DHT networking works correctly across network boundaries

## Quick Start

### Machine 1 (Writer)
```bash
# Using the writer script with CLI arguments
./scripts/run_writer.sh \
  --addr 0.0.0.0:9901 \
  --key my_test_key

# Or using environment variables
export NETABASE_WRITER_ADDR="0.0.0.0:9901"
export NETABASE_TEST_KEY="my_test_key"
./scripts/run_writer.sh

# Or run directly with cargo (legacy method)
cargo test cross_machine_writer -- --nocapture --ignored
```

### Machine 2 (Reader)
```bash
# Using the reader script with CLI arguments
./scripts/run_reader.sh \
  --connect 192.168.24.160:9901 \
  --key my_test_key \
  --timeout 60

# Or using environment variables
export NETABASE_READER_CONNECT_ADDR="192.168.1.100:9901"
export NETABASE_TEST_KEY="my_test_key"
export NETABASE_TEST_TIMEOUT="60"
./scripts/run_reader.sh

# Or run directly with cargo (legacy method)
cargo test cross_machine_reader_5_records -- --nocapture --ignored
```

## 5-Record Cross-Machine Tests

The project includes specialized tests for validating multi-record storage and retrieval across machines. These tests store exactly 5 records with unique keys and verify that all records can be retrieved correctly.

### Standard 5-Record Tests

NetaBase uses a standardized 5-record testing approach for all cross-machine validation:

```bash
# Run the writer test (stores 5 records)
cargo test cross_machine_writer_5_records --ignored -- --nocapture

# Or with shell script
./scripts/run_writer.sh --addr 0.0.0.0:9901 --key multi_record_test

# Run the reader test (retrieves 5 records)
NETABASE_READER_CONNECT_ADDR="192.168.1.100:9901" \
NETABASE_TEST_KEY="multi_record_test" \
cargo test cross_machine_reader_5_records --ignored -- --nocapture

# Or with shell script
./scripts/run_reader.sh --connect 192.168.1.100:9901 --key multi_record_test
```

The system stores and retrieves these 5 records with unique keys:
- `{base_key}__0`: "Hello World"
- `{base_key}__1`: "Test Record" 
- `{base_key}__2`: "Another Value"
- `{base_key}__3`: "Fourth Record"
- `{base_key}__4`: "Fifth Record"

### Key Benefits

- **Validates unique key generation**: Ensures records don't overwrite each other
- **Tests systematic retrieval**: Confirms the reader can find multiple records
- **Verifies data integrity**: Checks that all 5 records contain the correct values
- **Consistent test data**: Uses the same 5 test values across all environments
- **Eliminates configuration errors**: No need to manually specify values

## Configuration Options

### Shell Scripts (Recommended)

The easiest way to run cross-machine tests is using the provided shell scripts:

- `./scripts/run_writer.sh` - Start a writer node
- `./scripts/run_reader.sh` - Start a reader node
- `./scripts/run_local.sh` - Run local test with both nodes

### Writer Script Options

```bash
./scripts/run_writer.sh [options]
```

| CLI Option | Environment Variable | Description | Default |
|------------|---------------------|-------------|---------|
| `-a, --addr` | `NETABASE_WRITER_ADDR` | IP address and port to listen on | `0.0.0.0:9901` |
| `-k, --key` | `NETABASE_TEST_KEY` | Base key for the 5 records | `cross_machine_key` |
| `-t, --timeout` | `NETABASE_WRITER_TIMEOUT` | Timeout in seconds (0 = indefinite) | `0` |
| `--verbose` | - | Enable verbose logging | `false` |
| `--dry-run` | - | Show configuration without running | `false` |
| `--validate-only` | - | Only validate configuration | `false` |

**Note**: The 5 test values are now fixed and cannot be configured. The writer always stores:
- "Hello World", "Test Record", "Another Value", "Fourth Record", "Fifth Record"

### Reader Script Options

```bash
./scripts/run_reader.sh [options]
```

| CLI Option | Environment Variable | Description | Default |
|------------|---------------------|-------------|---------|
| `-c, --connect` | `NETABASE_READER_CONNECT_ADDR` | Writer's IP:PORT to connect to | `127.0.0.1:9901` |
| `-k, --key` | `NETABASE_TEST_KEY` | Base key for the 5 records | `cross_machine_key` |
| `-t, --timeout` | `NETABASE_TEST_TIMEOUT` | Timeout in seconds | `120` |
| `-r, --retries` | `NETABASE_READER_RETRIES` | Number of retry attempts per record | `3` |
| `--verbose` | - | Enable verbose logging | `false` |
| `--dry-run` | - | Show configuration without running | `false` |
| `--validate-only` | - | Only validate configuration | `false` |

**Note**: The 5 expected values are now fixed and cannot be configured. The reader always expects:
- "Hello World", "Test Record", "Another Value", "Fourth Record", "Fifth Record"

### Local Test Script Options

```bash
./scripts/run_local.sh [options]
```

| CLI Option | Environment Variable | Description | Default |
|------------|---------------------|-------------|---------|
| `-k, --key` | `NETABASE_TEST_KEY` | Base test key for 5 records | `cross_machine_key` |
| `-t, --timeout` | `NETABASE_TEST_TIMEOUT` | Timeout in seconds | `60` |
| `--verbose` | - | Enable verbose logging | `false` |
| `--dry-run` | - | Show configuration without running | `false` |
| `--validate-only` | - | Only validate configuration | `false` |

**Note**: Local tests also use the fixed 5-record approach for consistency.

## Detailed Instructions

### Step 1: Prepare Both Machines

1. **Install Rust and clone the repository** on both machines:
   ```bash
   git clone <repository_url>
   cd netabase
   cargo build
   ```

2. **Ensure network connectivity** between machines:
   ```bash
   # On Machine 1, find your IP address
   ip addr show
   # or
   ifconfig

   # Test connectivity from Machine 2
   ping <Machine1_IP>
   telnet <Machine1_IP> 9901  # Test if port is reachable
   ```

3. **Configure firewall** if needed:
   ```bash
   # Example for Ubuntu/Debian (on writer machine)
   sudo ufw allow 9901

   # Example for CentOS/RHEL
   sudo firewall-cmd --add-port=9901/udp --permanent
   sudo firewall-cmd --reload
   ```

### Step 2: Start Writer Node

On the writer machine:

```bash
# Using shell script with CLI arguments (recommended)
./scripts/run_writer.sh \
  --addr 0.0.0.0:9901 \
  --key distributed_test \
  --verbose

# Or using environment variables
export NETABASE_WRITER_ADDR="0.0.0.0:9901"
export NETABASE_TEST_KEY="distributed_test"
./scripts/run_writer.sh

# Or direct cargo method
cargo test cross_machine_writer_5_records -- --nocapture --ignored
```

#### Validation and Testing

Before starting the actual writer, you can validate your configuration:

```bash
# Validate configuration only
./scripts/run_writer.sh --validate-only \
  --addr 0.0.0.0:9901 \
  --key distributed_test

# Dry run to see what would be executed
./scripts/run_writer.sh --dry-run --verbose
```

**Expected output:**
```
===================================================================
             NetaBase Cross-Machine Writer (5-Record Version)
===================================================================

5-Record Test Settings:
  Base Key: 'distributed_test'
  Records to store:
    distributed_test__0: 'Hello World'
    distributed_test__1: 'Test Record'
    distributed_test__2: 'Another Value'
    distributed_test__3: 'Fourth Record'
    distributed_test__4: 'Fifth Record'

[INFO] Starting 5-record writer node...
[INFO] Writer node peer ID: 12D3KooW...
[INFO] Writer Put Record 0: QueryId(QueryId(0))
[INFO] Writer Put Record 1: QueryId(QueryId(1))
[INFO] Writer Put Record 2: QueryId(QueryId(2))
[INFO] Writer Put Record 3: QueryId(QueryId(3))
[INFO] Writer Put Record 4: QueryId(QueryId(4))
[INFO] Writer: Now listening for connections and serving requests...
```

### Step 3: Start Reader Node

On the reader machine (use the writer machine's actual IP address):

```bash
# Using shell script with CLI arguments (recommended)
./scripts/run_reader.sh \
  --connect 192.168.1.100:9901 \
  --key distributed_test \
  --timeout 60 \
  --retries 5 \
  --verbose

# Or using environment variables
export NETABASE_READER_CONNECT_ADDR="192.168.1.100:9901"
export NETABASE_TEST_KEY="distributed_test"
export NETABASE_TEST_TIMEOUT="60"
./scripts/run_reader.sh

# Or direct cargo method
cargo test cross_machine_reader_5_records -- --nocapture --ignored
```

#### Pre-flight Checks

Test your configuration before running:

```bash
# Validate configuration and test connectivity
./scripts/run_reader.sh --validate-only \
  --connect 192.168.1.100:9901 \
  --key distributed_test

# Dry run to see what would be executed
./scripts/run_reader.sh --dry-run --verbose
```

**Expected output:**
```
===================================================================
             NetaBase Cross-Machine Reader (5-Record Version)
===================================================================

5-Record Test Settings:
  Base Key: 'distributed_test'
  Expected Records:
    distributed_test__0: 'Hello World'
    distributed_test__1: 'Test Record'
    distributed_test__2: 'Another Value'
    distributed_test__3: 'Fourth Record'
    distributed_test__4: 'Fifth Record'

[INFO] Starting 5-record reader node...
[INFO] Reader node peer ID: 12D3KooW...
[INFO] Reader: Connected to peer: 12D3KooW...
[INFO] Reader: Attempting to get record 0 (attempt 1/5): QueryId(0)
[INFO] Reader: Found record 0: Hello World
[INFO] Reader: Attempting to get record 1 (attempt 1/5): QueryId(2)
[INFO] Reader: Found record 1: Test Record
[INFO] Reader: Found record 2: Another Value
[INFO] Reader: Found record 3: Fourth Record
[INFO] Reader: Found record 4: Fifth Record
[INFO] Reader: Finished searching, found 5 out of 5 expected records
[SUCCESS] 5-record reader test completed successfully!
[SUCCESS] All 5 records were retrieved and verified correctly
```

## Local Testing

Before running across machines, test the setup locally:

```bash
# Using the local test script (recommended)
./scripts/run_local.sh

# With custom configuration (5-record approach)
./scripts/run_local.sh \
  --key local_test \
  --timeout 30 \
  --verbose

# Validate configuration first
./scripts/run_local.sh --validate-only

# Or direct cargo method
cargo test cross_machine_local_test -- --nocapture --ignored
```

The local test script automatically:
- Starts a writer node on a random local port with 5 records
- Waits for the writer to be ready
- Starts a reader node to connect to the writer
- Verifies all 5 records are retrieved correctly with unique keys
- Shuts down both nodes automatically

**Note**: Local tests use the same fixed 5-record approach:
- "Hello World", "Test Record", "Another Value", "Fourth Record", "Fifth Record"

## Advanced Usage

### Multiple Readers

You can run multiple reader instances from different machines:

```bash
# Machine 2
./scripts/run_reader.sh --connect 192.168.1.100:9901 --key distributed_test

# Machine 3 (simultaneously)
./scripts/run_reader.sh --connect 192.168.1.100:9901 --key distributed_test --verbose

# Or with environment variables
export NETABASE_READER_CONNECT_ADDR="192.168.1.100:9901"
export NETABASE_TEST_KEY="distributed_test"
./scripts/run_reader.sh
```

### Custom Port

```bash
# Writer on custom port (stores 5 records)
./scripts/run_writer.sh --addr 0.0.0.0:8888 --key custom_test

# Reader connecting to custom port (retrieves 5 records)
./scripts/run_reader.sh --connect 192.168.1.100:8888 --key custom_test
```

### High-Throughput Test

```bash
# Test with multiple concurrent readers
for i in {1..5}; do
  ./scripts/run_reader.sh --connect 192.168.1.100:9901 --key throughput_test &
done
wait # Wait for all readers to complete
```

### Timed Tests

```bash
# Writer that runs for specific duration (5 records)
./scripts/run_writer.sh --timeout 300 --key timed_test

# Reader with custom timeout and retries (searches for 5 records)
./scripts/run_reader.sh --timeout 120 --retries 5 --key timed_test
```

## Troubleshooting

### Connection Issues

1. **"Connection refused" or "timeout"**
   - Check if writer is running and listening
   - Verify IP address is correct
   - Check firewall settings
   - Ensure port is not blocked
   - **Timing Issue**: Writer timeout may be too short (see Timing Recommendations below)

2. **"No route to host"**
   - Verify network connectivity: `ping <writer_ip>`
   - Check if machines are on same network/subnet
   - Verify routing tables

3. **"Writer shuts down before reader connects"**
   - Writer timeout is too short for cross-machine testing
   - Reader needs time to: connect (2-5s), stabilize (3s), retrieve records with retries
   - Minimum recommended writer timeout: 300 seconds (5 minutes)
   - Best practice: Use timeout=0 (indefinite) for cross-machine testing

### DHT Issues

1. **"Get record failed: NotFound"**
   - Wait longer for DHT to propagate records
   - Check if writer successfully stored records
   - Verify both nodes are using the same key

2. **"QuorumFailed" on writer**
   - This is expected behavior when there's only one node
   - Records may still be available for local retrieval
   - Run multiple writer nodes for true distributed testing

### Timing Recommendations

**Writer Timeout Settings:**
- **Indefinite (0 seconds)**: Recommended for cross-machine testing
  ```bash
  ./scripts/run_writer.sh -t 0  # Runs until Ctrl+C
  ```
- **300+ seconds**: Minimum for reliable cross-machine testing
  ```bash
  ./scripts/run_writer.sh -t 300  # 5 minutes
  ```
- **60-180 seconds**: May work for fast local networks only
- **<60 seconds**: Too short, will cause connection failures

**Reader Timing Factors:**
- Connection establishment: 2-5 seconds per attempt (up to 10 attempts)
- Connection stabilization: 3 seconds
- Record retrieval: 15 seconds timeout per record query
- Retry delays: Exponential backoff (2, 4, 8 seconds)
- Total time needed: ~60-120 seconds minimum for 5 records

**Environment Variables:**
```bash
# Writer - run indefinitely (recommended)
export NETABASE_WRITER_TIMEOUT=0

# Reader - sufficient timeout for cross-machine latency
export NETABASE_TEST_TIMEOUT=300
export NETABASE_READER_RETRIES=5
```

### Network Configuration

1. **Docker/Container environments**
   ```bash
   # Ensure proper port mapping
   docker run -p 9901:9901/udp <container>
   ```

2. **NAT/Router issues**
   - Configure port forwarding if needed
   - Use UPnP if available
   - Consider using relay servers for complex network topologies

### Performance Testing

```bash
# Monitor network traffic
sudo tcpdump -i any port 9901
```

## Scalability Testing

In addition to cross-machine functionality testing, NetaBase includes comprehensive scalability tests that measure how the P2P platform performs as the number of nodes increases. These tests are designed to identify bottlenecks, measure performance characteristics, and validate the system's ability to handle production-scale deployments.

### Overview of Scalability Tests

The scalability test suite includes:

- **Linear Node Scaling**: Tests performance as nodes are gradually added
- **Network Topology Performance**: Compares different connection patterns (mesh, ring, star, random)
- **High Throughput Operations**: Tests maximum DHT operation rates
- **Network Churn Resilience**: Tests stability as nodes join and leave dynamically
- **Bootstrap Performance**: Measures how quickly new nodes can join the network
- **Large Record Scalability**: Tests performance with large data payloads
- **Concurrent Operations**: Tests parallel DHT operations across multiple nodes
- **Network Partition Recovery**: Tests resilience to network splits and healing
- **Stress Testing**: Maximum scale testing with resource monitoring

### Running Scalability Tests

#### Quick Start

```bash
# Run all scalability tests with default settings (20 nodes, 5 minutes)
./scripts/run_scalability_tests.sh

# Quick smoke test for development (5 nodes, 1 minute)
./scripts/run_scalability_tests.sh --test-type smoke --max-nodes 5 --duration 60

# High-scale stress test (50 nodes, 10 minutes)
./scripts/run_scalability_tests.sh --test-type stress --max-nodes 50 --duration 600 --stress

# Test specific scenarios
./scripts/run_scalability_tests.sh --test-type topology --max-nodes 15 --verbose
./scripts/run_scalability_tests.sh --test-type bootstrap --max-nodes 25
./scripts/run_scalability_tests.sh --test-type large-records --record-size 65536
```

#### Configuration Options

| Option | Environment Variable | Description | Default |
|--------|---------------------|-------------|---------|
| `--max-nodes` | `NETABASE_SCALABILITY_MAX_NODES` | Maximum number of test nodes | 20 |
| `--duration` | `NETABASE_SCALABILITY_TEST_DURATION` | Test duration in seconds | 300 |
| `--interval` | `NETABASE_SCALABILITY_SPAWN_INTERVAL` | Node spawn interval in seconds | 2 |
| `--timeout` | `NETABASE_SCALABILITY_OP_TIMEOUT` | Operation timeout in seconds | 30 |
| `--records` | `NETABASE_SCALABILITY_RECORDS_PER_NODE` | Records per node | 10 |
| `--record-size` | `NETABASE_SCALABILITY_RECORD_SIZE` | Record size in bytes | 1024 |
| `--test-type` | - | Specific test to run | all |
| `--verbose` | `NETABASE_SCALABILITY_VERBOSE` | Enable detailed metrics | false |
| `--stress` | - | Enable stress testing mode | false |

#### Test Types

- **all**: Run complete test suite (default)
- **linear**: Linear node scaling test
- **topology**: Network topology performance comparison
- **throughput**: High throughput operations test
- **churn**: Network churn resilience test
- **bootstrap**: Bootstrap performance test
- **large-records**: Large record scalability test
- **concurrent**: Concurrent operations scaling test
- **partition**: Network partition recovery test
- **stress**: Maximum scale stress test
- **smoke**: Quick smoke test for CI/CD

### Performance Metrics

The scalability tests collect comprehensive performance metrics:

#### Connection Metrics
- **Connection Establishment Time**: Time to establish P2P connections
- **Active Connection Count**: Number of simultaneous connections per node
- **Connection Success Rate**: Percentage of successful connection attempts

#### DHT Operation Metrics
- **Operation Latency**: Average time for PUT/GET operations
- **Operations Per Second**: Throughput of DHT operations
- **Success/Failure Rates**: Reliability of DHT operations
- **Record Propagation Time**: Time for records to become available across nodes

#### Network Topology Metrics
- **Routing Table Size**: Number of peers in each node's routing table
- **Network Diameter**: Maximum hop count between any two nodes
- **Clustering Coefficient**: Measure of network connectivity density

#### Resource Usage Metrics
- **Memory Usage**: RAM consumption per node and total
- **File Descriptor Usage**: Number of open files/sockets
- **CPU Usage**: Processing overhead for P2P operations
- **Disk Usage**: Storage requirements for DHT data

#### Resilience Metrics
- **Churn Recovery Time**: Time to stabilize after nodes join/leave
- **Partition Recovery Time**: Time to heal network splits
- **Bootstrap Time**: Time for new nodes to fully join the network

### System Requirements

#### Minimum Requirements
- **RAM**: 4GB+ (50MB per node)
- **CPU**: 2+ cores
- **Disk**: 1GB+ free space
- **Network**: Unrestricted UDP ports

#### Recommended for High-Scale Testing
- **RAM**: 16GB+ for 50+ nodes
- **CPU**: 8+ cores for stress testing
- **Disk**: 10GB+ for large record tests
- **File Descriptors**: `ulimit -n 8192` or higher

#### Performance Tuning

```bash
# Increase file descriptor limit
ulimit -n 8192

# Monitor resources during testing
htop  # or top
netstat -an | grep ESTABLISHED | wc -l

# For high-scale testing
echo 'net.core.rmem_max = 16777216' | sudo tee -a /etc/sysctl.conf
echo 'net.core.wmem_max = 16777216' | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

### Interpreting Results

#### Key Performance Indicators

1. **Scalability Factor**: How performance degrades as nodes increase
   - Linear degradation is acceptable
   - Exponential degradation indicates bottlenecks

2. **Connection Density**: Average connections per node
   - Full mesh: N-1 connections per node
   - Efficient topologies: log(N) connections per node

3. **Operation Latency**: DHT operation response time
   - Local operations: <10ms
   - Network operations: <100ms typical
   - High latency may indicate network issues

4. **Throughput Scaling**: Operations per second vs node count
   - Should scale roughly linearly with nodes
   - Plateaus indicate saturation points

#### Example Output Analysis

```
=== Linear Node Scaling Test Report ===
Configuration:
  Max nodes: 20
  Test duration: 300s
  Records per node: 10
  Record value size: 1024 bytes

Network Overview:
  Total nodes: 20
  Total connections: 190  # Good - close to full mesh (20*19/2 = 190)
  Average connections per node: 9.5

Aggregate Performance Metrics:
  Total successful operations: 15420
  Total failed operations: 23
  Overall failure rate: 0.15%  # Excellent - very low failure rate
  Average latency: 45.2ms      # Good - reasonable for network operations
  Operations per second: 51.4  # Good scaling
```

### Production Usage Considerations

#### Features Useful for Production

The scalability tests include several features that are valuable for production P2P systems:

1. **Performance Metrics Collection**: 
   - Real-time metrics gathering
   - Aggregated performance reporting
   - Resource usage monitoring

2. **Network Topology Testing**:
   - Comparison of connection strategies
   - Bootstrap node optimization
   - Resilience pattern validation

3. **Load Testing Capabilities**:
   - Configurable record sizes and counts
   - Concurrent operation testing
   - System resource monitoring

4. **Failure Scenario Testing**:
   - Network partition simulation
   - Node churn handling
   - Recovery time measurement

#### Integration with Monitoring Systems

```rust
// Example: Integrating metrics collection in production
use netabase::network_scalability::PerformanceMetrics;

let metrics = PerformanceMetrics::new();
let metrics_receiver = setup_metrics_collection(&nodes, Duration::from_secs(10));

// Send metrics to your monitoring system
tokio::spawn(async move {
    while let Some(metrics_data) = metrics_receiver.recv().await {
        // Send to Prometheus, InfluxDB, etc.
        send_to_monitoring_system(metrics_data).await;
    }
});
```

### Continuous Integration

#### CI-Friendly Smoke Test

```bash
# Quick test suitable for CI/CD pipelines
./scripts/run_scalability_tests.sh --test-type smoke --max-nodes 3 --duration 30
```

#### GitHub Actions Example

```yaml
name: Scalability Tests
on: [push, pull_request]
jobs:
  scalability:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run scalability smoke test
        run: |
          ./scripts/run_scalability_tests.sh --test-type smoke --max-nodes 5 --duration 60
```

### Troubleshooting Scalability Tests

#### Common Issues

1. **Out of Memory Errors**
   ```bash
   # Solution: Reduce node count or increase system memory
   ./scripts/run_scalability_tests.sh --max-nodes 10  # Reduce from default 20
   ```

2. **File Descriptor Limits**
   ```bash
   # Solution: Increase ulimit
   ulimit -n 8192
   ./scripts/run_scalability_tests.sh
   ```

3. **Port Exhaustion**
   ```bash
   # Solution: Wait between test runs or use longer intervals
   ./scripts/run_scalability_tests.sh --interval 5  # Slower node spawning
   ```

4. **Test Timeouts**
   ```bash
   # Solution: Increase duration or reduce complexity
   ./scripts/run_scalability_tests.sh --duration 600 --timeout 60
   ```

#### Performance Analysis

If scalability tests show performance issues:

1. **Check System Resources**
   ```bash
   # During test execution
   htop                                    # Monitor CPU/RAM
   iostat -x 1                            # Monitor disk I/O
   netstat -an | grep ESTABLISHED | wc -l # Count connections
   ```

2. **Profile Network Usage**
   ```bash
   # Monitor network traffic
   iftop                    # Real-time network usage
   tcpdump -i any port 9901 # Capture P2P traffic
   ```

3. **Analyze Log Output**
   - Look for connection failures
   - Check for timeout patterns
   - Monitor DHT operation success rates

## What the Tests Validate

The 5-record cross-machine tests provide comprehensive validation of NetaBase's distributed functionality:

- ✅ **Network connectivity**: Machines can establish libp2p connections across networks
- ✅ **DHT functionality**: Records stored on one node are retrievable from another
- ✅ **Unique key generation**: Each record gets a distinct key (`base_key__0` through `base_key__4`)
- ✅ **Multi-record storage**: Writer successfully stores all 5 records without overwrites
- ✅ **Systematic retrieval**: Reader finds all 5 records using the same key generation logic
- ✅ **Protocol compatibility**: Different instances communicate correctly with consistent data formats
- ✅ **Data integrity**: All 5 retrieved records match their expected values exactly
- ✅ **Error handling**: Graceful handling of network issues, timeouts, and missing records
- ✅ **Retry mechanisms**: Reader retries failed record retrievals automatically
- ✅ **Scalability validation**: Multiple readers can simultaneously retrieve records
- ✅ **Consistency across environments**: Same 5 test values work across all setups

### Key Advantages of 5-Record Testing

- **Prevents overwrite bugs**: Ensures records don't replace each other
- **Tests distributed indexing**: Validates DHT can handle multiple keys efficiently  
- **Realistic workload**: Simulates applications that store multiple related records
- **Comprehensive coverage**: Tests both successful and failed record operations
- **Deterministic results**: Fixed test data eliminates configuration-related failures

## Next Steps

After successful cross-machine testing, you can:
- Deploy NetaBase nodes on production infrastructure
- Test with more complex network topologies
- Benchmark performance across different network conditions
- Implement application-specific logic on top of the DHT

## Notes

### Script Features
- **Configuration validation**: All scripts validate input before running
- **Dry-run mode**: Test configuration without actually running nodes
- **Verbose logging**: Detailed output for debugging
- **Connectivity testing**: Scripts test network connectivity before starting
- **Graceful shutdown**: Proper cleanup on Ctrl+C interruption

### Network Requirements
- Writer test runs indefinitely by default (use `--timeout` to limit)
- Reader test has configurable timeout and retries automatically
- Tests use UDP/QUIC for transport, ensure UDP traffic is allowed
- IPv6 addresses are supported by using IPv6 format in `--addr`
- Firewall must allow UDP traffic on specified ports

### Configuration Priority
1. Command-line arguments (highest priority)
2. Environment variables
3. Default values (lowest priority)

### Legacy Support
- Direct `cargo test` commands still work with new test names:
  - `cargo test cross_machine_writer_5_records --ignored -- --nocapture`
  - `cargo test cross_machine_reader_5_records --ignored -- --nocapture`
- Environment-only configuration is still supported (except test values are now fixed)
- Shell scripts provide better UX and validation on top of the core 5-record tests
- Legacy test names (`cross_machine_writer`, `cross_machine_reader`) still exist but use different logic
