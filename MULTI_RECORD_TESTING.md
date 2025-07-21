# Multi-Record Cross-Machine Testing

This document explains how to use NetaBase's multi-record testing functionality to verify persistence across machines and sessions.

## Overview

The multi-record testing framework enables:

- Testing with **large numbers** of records (default: 50, configurable)
- **Deterministic path generation** for consistent testing across sessions
- **Comprehensive reporting** of success/failure for each record
- **Easy configuration** through environment variables or command-line options

This approach extends the standard 5-record cross-machine tests to validate storage and retrieval of many records, providing more rigorous verification of DHT functionality.

## Quick Start

### Machine 1 (Writer)
```bash
# Using the multi-record writer script
./scripts/run_multi_writer.sh \
  --addr 0.0.0.0:9901 \
  --key persistence_test \
  --count 100 \
  --seed 12345

# Or using environment variables
export NETABASE_WRITER_ADDR="0.0.0.0:9901"
export NETABASE_TEST_KEY="persistence_test"
export NETABASE_RECORD_COUNT="100"
export NETABASE_TEST_SEED="12345"
./scripts/run_multi_writer.sh
```

### Machine 2 (Reader)
```bash
# Using the multi-record reader script
./scripts/run_multi_reader.sh \
  --connect 192.168.1.100:9901 \
  --key persistence_test \
  --count 100 \
  --seed 12345 \
  --timeout 180 \
  --retries 5

# Or using environment variables
export NETABASE_READER_CONNECT_ADDR="192.168.1.100:9901"
export NETABASE_TEST_KEY="persistence_test"
export NETABASE_RECORD_COUNT="100"
export NETABASE_TEST_SEED="12345"
export NETABASE_TEST_TIMEOUT="180"
export NETABASE_READER_RETRIES="5"
./scripts/run_multi_reader.sh
```

## Persistence Testing with Deterministic Paths

The multi-record testing framework supports deterministic path generation, which is crucial for testing persistence across sessions and machines. When a seed value is provided, temporary directories are created with consistent names, allowing the database to persist between test runs.

### Key Features

- **Seed-based path generation**: Provide a seed value to generate consistent paths
- **Consistent across machines**: Use the same seed on different machines for cross-validation
- **Configurable record count**: Test with any number of records (default: 50)
- **Detailed results reporting**: See success/failure for each record

## Configuration Options

### Shell Scripts

Three scripts are provided for multi-record testing:

- `./scripts/run_multi_writer.sh` - Start a writer node with many records
- `./scripts/run_multi_reader.sh` - Start a reader node to find and verify records
- `./scripts/run_multi_local_test.sh` - Run local test with both nodes in sequence

### Writer Script Options

```bash
./scripts/run_multi_writer.sh [options]
```

| CLI Option | Environment Variable | Description | Default |
|------------|---------------------|-------------|---------|
| `-a, --addr` | `NETABASE_WRITER_ADDR` | IP address and port to listen on | `0.0.0.0:9901` |
| `-k, --key` | `NETABASE_TEST_KEY` | Base key for the records | `multi_record_test` |
| `-c, --count` | `NETABASE_RECORD_COUNT` | Number of records to create | `50` |
| `-s, --seed` | `NETABASE_TEST_SEED` | Seed for deterministic paths | Random |
| `-t, --timeout` | `NETABASE_WRITER_TIMEOUT` | Timeout in seconds (0 = indefinite) | `300` |
| `--verbose` | `NETABASE_VERBOSE` | Enable verbose logging | `false` |
| `--dry-run` | - | Show configuration without running | `false` |
| `--validate-only` | - | Only validate configuration | `false` |

### Reader Script Options

```bash
./scripts/run_multi_reader.sh [options]
```

| CLI Option | Environment Variable | Description | Default |
|------------|---------------------|-------------|---------|
| `-c, --connect` | `NETABASE_READER_CONNECT_ADDR` | Writer's IP:PORT to connect to | `127.0.0.1:9901` |
| `-k, --key` | `NETABASE_TEST_KEY` | Base key for the records | `multi_record_test` |
| `-n, --count` | `NETABASE_RECORD_COUNT` | Number of records to read | `50` |
| `-s, --seed` | `NETABASE_TEST_SEED` | Seed for deterministic paths | Random |
| `-t, --timeout` | `NETABASE_TEST_TIMEOUT` | Timeout in seconds | `120` |
| `-r, --retries` | `NETABASE_READER_RETRIES` | Number of retry attempts per record | `3` |
| `--verbose` | `NETABASE_VERBOSE` | Enable verbose logging | `false` |
| `--dry-run` | - | Show configuration without running | `false` |
| `--validate-only` | - | Only validate configuration | `false` |

### Local Test Script Options

```bash
./scripts/run_multi_local_test.sh [options]
```

| CLI Option | Environment Variable | Description | Default |
|------------|---------------------|-------------|---------|
| `-a, --addr` | `NETABASE_WRITER_ADDR` & `NETABASE_READER_CONNECT_ADDR` | Address for both nodes | `127.0.0.1:9901` |
| `-k, --key` | `NETABASE_TEST_KEY` | Base key for records | `multi_record_test` |
| `-n, --count` | `NETABASE_RECORD_COUNT` | Number of records | `50` |
| `-s, --seed` | `NETABASE_TEST_SEED` | Seed for deterministic paths | `42` |
| `-t, --timeout` | `NETABASE_TEST_TIMEOUT` | Reader timeout in seconds | `120` |
| `-r, --retries` | `NETABASE_READER_RETRIES` | Number of retry attempts per record | `3` |
| `--verbose` | `NETABASE_VERBOSE` | Enable verbose logging | `false` |
| `--dry-run` | - | Show configuration without running | `false` |
| `--validate-only` | - | Only validate configuration | `false` |

## Detailed Testing Instructions

### Testing Persistence Across Sessions

To verify that records persist across different sessions:

1. **Run a writer** with a specific seed:
   ```bash
   ./scripts/run_multi_writer.sh --seed 12345 --count 100 --addr 0.0.0.0:9901
   ```

2. **Terminate the writer** after records are stored

3. **Run a reader** with the same seed:
   ```bash
   ./scripts/run_multi_reader.sh --seed 12345 --count 100 --connect 127.0.0.1:9901
   ```

4. **Check the results** to verify that records were successfully retrieved

### Testing Persistence Across Machines

To verify persistence across different machines:

1. **Machine 1**: Run writer with specific seed:
   ```bash
   ./scripts/run_multi_writer.sh --seed 67890 --count 75 --addr 0.0.0.0:9901
   ```

2. **Machine 2**: Run reader with the same seed:
   ```bash
   ./scripts/run_multi_reader.sh --seed 67890 --count 75 --connect 192.168.1.100:9901
   ```

3. **Optional**: Restart Machine 1 with same seed to test persistence:
   ```bash
   ./scripts/run_multi_writer.sh --seed 67890 --count 75 --addr 0.0.0.0:9901
   ```

4. **Machine 2**: Run reader again to verify persistence:
   ```bash
   ./scripts/run_multi_reader.sh --seed 67890 --count 75 --connect 192.168.1.100:9901
   ```

## Record Generation

Records are generated with a predictable pattern:

- **Key format**: `{base_key}__{index}`
- **Value format**: `Multi-Record Test Value {index}`

For example, with a base key of "persistence_test" and 3 records:
- `persistence_test__0`: "Multi-Record Test Value 0"
- `persistence_test__1`: "Multi-Record Test Value 1"
- `persistence_test__2`: "Multi-Record Test Value 2"

This pattern makes it easy to verify that the correct values are being retrieved.

## Results Reporting

The reader test provides detailed results reporting:

- **Total records**: Number of records attempted
- **Successful records**: Number of records successfully retrieved
- **Failed records**: Number of records that could not be retrieved
- **Success percentage**: Percentage of records successfully retrieved
- **Detailed failure report**: For each failed record, information about the failure

Example output:
```
=== CROSS-MACHINE MULTI-RECORD READER RESULTS ===
Total records: 100
Successful: 98 (98%)
Failed: 2 (2%)
Test duration: 45.2s

--- Failed Records ---
Record 42: multi_record_test__42 - Not found
Record 87: multi_record_test__87 - Timeout
===========================================
```

## Common Issues and Troubleshooting

### Connection Issues
- Ensure firewall rules allow UDP traffic on the port you're using
- Verify the writer's IP address is correctly specified in the reader configuration
- Try increasing the timeout value for the reader

### Missing Records
- Verify that the seed value is the same on both machines
- Check that the record count matches between writer and reader
- Increase the number of retries per record
- Ensure the writer has completed storing all records before starting the reader

### Performance Considerations
- With large record counts (>100), increase the timeout values
- For machines with limited resources, reduce the record count
- If network connection is unstable, increase the retry count

## Integration with CI/CD

The multi-record testing can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
name: Cross-Machine Multi-Record Test

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run local multi-record test
        run: |
          cd netabase
          ./scripts/run_multi_local_test.sh --count 25 --seed 12345
```

## References

- [Cross-Machine Testing](CROSS_MACHINE_TESTING.md) - Basic cross-machine testing documentation
- [Scalability Testing](SCALABILITY_TESTING.md) - For high-volume testing scenarios