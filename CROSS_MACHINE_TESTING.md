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
  --key my_test_key \
  --values "Hello,World,Distributed,Hash,Table"

# Or using environment variables
export NETABASE_WRITER_ADDR="0.0.0.0:9901"
export NETABASE_TEST_KEY="my_test_key"
export NETABASE_TEST_VALUES="Hello,World,Distributed,Hash,Table"
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
  --values "Hello,World,Distributed,Hash,Table" \
  --timeout 60

# Or using environment variables
export NETABASE_READER_CONNECT_ADDR="192.168.1.100:9901"
export NETABASE_TEST_KEY="my_test_key"
export NETABASE_TEST_VALUES="Hello,World,Distributed,Hash,Table"
export NETABASE_TEST_TIMEOUT="60"
./scripts/run_reader.sh

# Or run directly with cargo (legacy method)
cargo test cross_machine_reader -- --nocapture --ignored
```

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
| `-a, --addr` | `NETABASE_WRITER_ADDR` | IP:PORT to listen on | `0.0.0.0:9901` |
| `-k, --key` | `NETABASE_TEST_KEY` | Key to store records under | `cross_machine_key` |
| `-v, --values` | `NETABASE_TEST_VALUES` | Comma-separated values to store | `Value1,Value2,Value3,HelloWorld` |
| `-t, --timeout` | `NETABASE_WRITER_TIMEOUT` | Timeout in seconds (0=indefinite) | `0` |
| `--verbose` | - | Enable verbose logging | `false` |
| `--dry-run` | - | Show configuration without running | `false` |
| `--validate-only` | - | Only validate configuration | `false` |

### Reader Script Options

```bash
./scripts/run_reader.sh [options]
```

| CLI Option | Environment Variable | Description | Default |
|------------|---------------------|-------------|---------|
| `-c, --connect` | `NETABASE_READER_CONNECT_ADDR` | Writer's IP:PORT to connect to | `127.0.0.1:9901` |
| `-k, --key` | `NETABASE_TEST_KEY` | Key to retrieve records from | `cross_machine_key` |
| `-v, --values` | `NETABASE_TEST_VALUES` | Expected comma-separated values | `Value1,Value2,Value3,HelloWorld` |
| `-t, --timeout` | `NETABASE_TEST_TIMEOUT` | Timeout in seconds | `120` |
| `-r, --retries` | `NETABASE_READER_RETRIES` | Number of retry attempts | `3` |
| `--verbose` | - | Enable verbose logging | `false` |
| `--dry-run` | - | Show configuration without running | `false` |
| `--validate-only` | - | Only validate configuration | `false` |

### Local Test Script Options

```bash
./scripts/run_local.sh [options]
```

| CLI Option | Environment Variable | Description | Default |
|------------|---------------------|-------------|---------|
| `-k, --key` | `NETABASE_TEST_KEY` | Test key to use | `cross_machine_key` |
| `-v, --values` | `NETABASE_TEST_VALUES` | Comma-separated test values | `Value1,Value2,Value3,HelloWorld` |
| `-t, --timeout` | `NETABASE_TEST_TIMEOUT` | Timeout in seconds | `60` |
| `--verbose` | - | Enable verbose logging | `false` |
| `--dry-run` | - | Show configuration without running | `false` |
| `--validate-only` | - | Only validate configuration | `false` |

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
  --values "Message1,Message2,Message3,Hello from Writer" \
  --verbose

# Or using environment variables
export NETABASE_WRITER_ADDR="0.0.0.0:9901"
export NETABASE_TEST_KEY="distributed_test"
export NETABASE_TEST_VALUES="Message1,Message2,Message3,Hello from Writer"
./scripts/run_writer.sh

# Or legacy method with cargo
cargo test cross_machine_writer -- --nocapture --ignored
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
=== CROSS-MACHINE WRITER TEST ===
Writer address: 0.0.0.0:9901
Test key: Key(b"distributed_test")
Test values: ["Message1", "Message2", "Message3", "Hello from Writer"]
=====================================
[INFO] Starting writer node on address: 0.0.0.0:9901
[INFO] Writer node peer ID: 12D3KooW...
[INFO] Writer: Stored record 1/4: 'Message1' (QueryId: QueryId(0))
[INFO] Writer: Stored record 2/4: 'Message2' (QueryId: QueryId(1))
[INFO] Writer: Stored record 3/4: 'Message3' (QueryId: QueryId(2))
[INFO] Writer: Stored record 4/4: 'Hello from Writer' (QueryId: QueryId(3))
[INFO] Writer: Listening on /ip4/0.0.0.0/udp/9901/quic-v1
[INFO] Writer: Now listening for connections and serving requests...
[INFO] Writer: Press Ctrl+C to stop the writer node
```

### Step 3: Start Reader Node

On the reader machine (use the writer machine's actual IP address):

```bash
# Using shell script with CLI arguments (recommended)
./scripts/run_reader.sh \
  --connect 192.168.1.100:9901 \
  --key distributed_test \
  --values "Message1,Message2,Message3,Hello from Writer" \
  --timeout 60 \
  --retries 5 \
  --verbose

# Or using environment variables
export NETABASE_READER_CONNECT_ADDR="192.168.1.100:9901"
export NETABASE_TEST_KEY="distributed_test"
export NETABASE_TEST_VALUES="Message1,Message2,Message3,Hello from Writer"
export NETABASE_TEST_TIMEOUT="60"
./scripts/run_reader.sh

# Or legacy method with cargo
cargo test cross_machine_reader -- --nocapture --ignored
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
=== CROSS-MACHINE READER TEST ===
Connecting to writer at: 192.168.1.100:9901
Test key: Key(b"distributed_test")
Expected values: ["Message1", "Message2", "Message3", "Hello from Writer"]
Timeout: 60 seconds
===================================
[INFO] Starting reader node, connecting to writer at: 192.168.1.100:9901
[INFO] Reader node peer ID: 12D3KooW...
[INFO] Reader: Attempting to dial writer at: /ip4/192.168.1.100/udp/9901/quic-v1
[INFO] Reader: Connected to peer: 12D3KooW...
[INFO] Reader: Attempting to get record with key: Key(b"distributed_test")
[INFO] Reader: Found record: 'Message1'
[INFO] Reader: Found record: 'Message2'
[INFO] Reader: Found record: 'Hello from Writer'
[INFO] ✓ Found expected value: 'Message1'
[INFO] ✓ Found expected value: 'Message2'
[INFO] ✗ Missing expected value: 'Message3'
[INFO] ✓ Found expected value: 'Hello from Writer'
[INFO] Cross-machine reader test completed successfully!
```

## Local Testing

Before running across machines, test the setup locally:

```bash
# Using the local test script (recommended)
./scripts/run_local.sh

# With custom configuration
./scripts/run_local.sh \
  --key local_test \
  --values "Local1,Local2,Local3" \
  --timeout 30 \
  --verbose

# Validate configuration first
./scripts/run_local.sh --validate-only

# Or legacy method with cargo
cargo test cross_machine_local_test -- --nocapture --ignored
```

The local test script automatically:
- Starts a writer node on a random local port
- Waits for the writer to be ready
- Starts a reader node to connect to the writer
- Verifies all records are retrieved correctly
- Shuts down both nodes automatically

## Advanced Usage

### Multiple Readers

You can run multiple reader instances from different machines:

```bash
# Machine 2
./scripts/run_reader.sh --connect 192.168.1.100:9901

# Machine 3 (simultaneously)
./scripts/run_reader.sh --connect 192.168.1.100:9901 --verbose

# Or with environment variables
export NETABASE_READER_CONNECT_ADDR="192.168.1.100:9901"
./scripts/run_reader.sh
```

### Custom Port

```bash
# Writer on custom port
./scripts/run_writer.sh --addr 0.0.0.0:8888 --key custom_test

# Reader connecting to custom port
./scripts/run_reader.sh --connect 192.168.1.100:8888 --key custom_test
```

### Large Data Test

```bash
# Test with larger payload
LARGE_VALUES="$(python3 -c 'print(",".join([f"Data{i}" for i in range(100)]))')"
./scripts/run_writer.sh --values "$LARGE_VALUES" --key large_test
./scripts/run_reader.sh --values "$LARGE_VALUES" --key large_test
```

### Timed Tests

```bash
# Writer that runs for specific duration
./scripts/run_writer.sh --timeout 300 --key timed_test

# Reader with custom timeout and retries
./scripts/run_reader.sh --timeout 60 --retries 10 --key timed_test
```

## Troubleshooting

### Connection Issues

1. **"Connection refused" or "timeout"**
   - Check if writer is running and listening
   - Verify IP address is correct
   - Check firewall settings
   - Ensure port is not blocked

2. **"No route to host"**
   - Verify network connectivity: `ping <writer_ip>`
   - Check if machines are on same network/subnet
   - Verify routing tables

### DHT Issues

1. **"Get record failed: NotFound"**
   - Wait longer for DHT to propagate records
   - Check if writer successfully stored records
   - Verify both nodes are using the same key

2. **"QuorumFailed" on writer**
   - This is expected behavior when there's only one node
   - Records may still be available for local retrieval
   - Run multiple writer nodes for true distributed testing

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
# Test with timestamp to measure propagation delay
export NETABASE_TEST_VALUES="$(date +%s),Test1,Test2"

# Monitor network traffic
sudo tcpdump -i any port 9901
```

## What the Tests Validate

- ✅ **Network connectivity**: Machines can establish libp2p connections
- ✅ **DHT functionality**: Records stored on one node are retrievable from another
- ✅ **Protocol compatibility**: Different instances can communicate correctly
- ✅ **Data integrity**: Retrieved data matches stored data
- ✅ **Error handling**: Graceful handling of network issues and timeouts

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
- Direct `cargo test` commands still work for backward compatibility
- Environment-only configuration is still supported
- New scripts provide better UX and validation on top of existing tests
