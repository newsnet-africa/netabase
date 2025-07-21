#!/bin/bash

# NetaBase Local Multi-Record Cross-Machine Test Runner
# This script runs both writer and reader nodes in sequence to test
# multi-record cross-machine functionality locally

set -e

# Default configuration
DEFAULT_ADDR="127.0.0.1:9901"
DEFAULT_HOST="127.0.0.1"
DEFAULT_PORT="9901"
DEFAULT_TEST_KEY="multi_record_test"
DEFAULT_TEST_TIMEOUT="120"    # 2 minutes default
DEFAULT_READER_RETRIES="3"    # Default retry attempts per record
DEFAULT_RECORD_COUNT="50"     # Default number of records
DEFAULT_TEST_SEED="42"        # Default to a consistent seed for local testing

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

show_usage() {
    echo -e "${CYAN}NetaBase Local Multi-Record Cross-Machine Test Runner${NC}"
    echo ""
    echo "This script runs both writer and reader nodes in sequence to test"
    echo "multi-record cross-machine functionality locally."
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -a, --addr ADDR      Address to use (default: $DEFAULT_ADDR)"
    echo "  -H, --host HOST      Host/IP address (default: $DEFAULT_HOST)"
    echo "  -p, --port PORT      Port number (default: $DEFAULT_PORT)"
    echo "  -k, --key KEY        Base test key for records (default: $DEFAULT_TEST_KEY)"
    echo "  -n, --count NUM      Number of records (default: $DEFAULT_RECORD_COUNT)"
    echo "  -s, --seed SEED      Seed for deterministic paths (default: $DEFAULT_TEST_SEED)"
    echo "  -t, --timeout SECS   Reader timeout in seconds (default: $DEFAULT_TEST_TIMEOUT)"
    echo "  -r, --retries NUM    Number of retry attempts per record (default: $DEFAULT_READER_RETRIES)"
    echo "  --verbose            Enable verbose logging"
    echo "  --dry-run            Show configuration without running the test"
    echo "  --validate-only      Only validate configuration and exit"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  NETABASE_WRITER_ADDR             Override listen address (writer)"
    echo "  NETABASE_READER_CONNECT_ADDR     Override connect address (reader)"
    echo "  NETABASE_TEST_KEY                Override base test key"
    echo "  NETABASE_RECORD_COUNT            Override number of records"
    echo "  NETABASE_TEST_SEED               Override deterministic seed"
    echo "  NETABASE_TEST_TIMEOUT            Override reader timeout"
    echo "  NETABASE_READER_RETRIES          Override reader retry attempts"
    echo ""
    echo "Examples:"
    echo "  # Basic usage with defaults"
    echo "  $0"
    echo ""
    echo "  # Custom key and 100 records"
    echo "  $0 --key mykey --count 100"
    echo ""
    echo "  # Specific seed and more retries"
    echo "  $0 --seed 12345 --retries 5"
    echo ""
    echo "Test Process:"
    echo "  1. Writer node starts and stores the specified number of records"
    echo "  2. After a brief pause, reader node attempts to retrieve all records"
    echo "  3. Results are reported with detailed success/failure statistics"
    echo ""
    echo "The test passes if all records are successfully retrieved and verified."
    echo ""
}

# Parse command line arguments
VERBOSE=false
DRY_RUN=false
VALIDATE_ONLY=false

while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        -a|--addr)
            export NETABASE_WRITER_ADDR="$2"
            export NETABASE_READER_CONNECT_ADDR="$2"
            shift 2
            ;;
        -H|--host)
            export NETABASE_HOST="$2"
            shift 2
            ;;
        -p|--port)
            export NETABASE_PORT="$2"
            shift 2
            ;;
        -k|--key)
            export NETABASE_TEST_KEY="$2"
            shift 2
            ;;
        -n|--count)
            export NETABASE_RECORD_COUNT="$2"
            shift 2
            ;;
        -s|--seed)
            export NETABASE_TEST_SEED="$2"
            shift 2
            ;;
        -t|--timeout)
            export NETABASE_TEST_TIMEOUT="$2"
            shift 2
            ;;
        -r|--retries)
            export NETABASE_READER_RETRIES="$2"
            shift 2
            ;;
        --verbose)
            export NETABASE_VERBOSE=true
            VERBOSE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --validate-only)
            VALIDATE_ONLY=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            show_usage
            exit 1
            ;;
    esac
done

# Apply defaults for address from separate host and port if provided
if [[ -n "$NETABASE_HOST" && -n "$NETABASE_PORT" ]]; then
    ADDR="${NETABASE_HOST}:${NETABASE_PORT}"
    export NETABASE_WRITER_ADDR="$ADDR"
    export NETABASE_READER_CONNECT_ADDR="$ADDR"
    if $VERBOSE; then
        echo "Combined host and port into address: $ADDR"
    fi
fi

# Use defaults if not set
: "${NETABASE_WRITER_ADDR:=$DEFAULT_ADDR}"
: "${NETABASE_READER_CONNECT_ADDR:=$DEFAULT_ADDR}"
: "${NETABASE_TEST_KEY:=$DEFAULT_TEST_KEY}"
: "${NETABASE_RECORD_COUNT:=$DEFAULT_RECORD_COUNT}"
: "${NETABASE_TEST_TIMEOUT:=$DEFAULT_TEST_TIMEOUT}"
: "${NETABASE_READER_RETRIES:=$DEFAULT_READER_RETRIES}"
: "${NETABASE_TEST_SEED:=$DEFAULT_TEST_SEED}"

# Validate record count is a positive integer
if ! [[ "$NETABASE_RECORD_COUNT" =~ ^[0-9]+$ ]]; then
    echo -e "${RED}Error: Record count must be a positive integer${NC}"
    exit 1
fi

if [[ "$NETABASE_RECORD_COUNT" -lt 1 ]]; then
    echo -e "${RED}Error: Record count must be at least 1${NC}"
    exit 1
fi

# Validate timeout is a positive integer
if ! [[ "$NETABASE_TEST_TIMEOUT" =~ ^[0-9]+$ ]]; then
    echo -e "${RED}Error: Timeout must be a positive integer${NC}"
    exit 1
fi

if [[ "$NETABASE_TEST_TIMEOUT" -lt 1 ]]; then
    echo -e "${RED}Error: Timeout must be at least 1 second${NC}"
    exit 1
fi

# Validate retries is a non-negative integer
if ! [[ "$NETABASE_READER_RETRIES" =~ ^[0-9]+$ ]]; then
    echo -e "${RED}Error: Retries must be a non-negative integer${NC}"
    exit 1
fi

# Validate seed is numeric if provided
if [[ -n "$NETABASE_TEST_SEED" && ! "$NETABASE_TEST_SEED" =~ ^[0-9]+$ ]]; then
    echo -e "${RED}Error: Seed must be a numeric value${NC}"
    exit 1
fi

# Print configuration
echo -e "${CYAN}NetaBase Local Multi-Record Test Configuration:${NC}"
echo "Writer Address: $NETABASE_WRITER_ADDR"
echo "Reader Connect Address: $NETABASE_READER_CONNECT_ADDR"
echo "Test Key: $NETABASE_TEST_KEY"
echo "Record Count: $NETABASE_RECORD_COUNT"
echo "Timeout: ${NETABASE_TEST_TIMEOUT} seconds"
echo "Retries: $NETABASE_READER_RETRIES per record"
echo "Test Seed: $NETABASE_TEST_SEED (deterministic paths)"
echo "Verbose Mode: $NETABASE_VERBOSE"

if $VALIDATE_ONLY; then
    echo -e "${GREEN}Configuration validated successfully${NC}"
    exit 0
fi

if $DRY_RUN; then
    echo -e "${YELLOW}Dry run completed, exiting without running the test${NC}"
    exit 0
fi

echo ""
echo -e "${CYAN}Starting Local Multi-Record Cross-Machine Test...${NC}"
echo -e "${CYAN}This test will run both writer and reader in sequence${NC}"

# Set a shorter timeout for the writer in the local test
export NETABASE_WRITER_TIMEOUT=30

# Run the local test in the Rust crate
cd "$(dirname "$0")/.."
RUST_BACKTRACE=1 cargo test --package netabase --test network_cross_machine_multi cross_machine_local_test_multi_records --no-run --message-format=json | jq -r 'select(.executable != null) | .executable' | xargs -I{} {} --exact

# Check exit code
EXIT_CODE=$?
if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}Local multi-record test completed successfully${NC}"
    echo -e "${GREEN}✓ All records were successfully stored and retrieved${NC}"
else
    echo -e "${RED}Local multi-record test failed with exit code $EXIT_CODE${NC}"
    echo -e "${RED}✗ Some records could not be verified${NC}"
fi

exit $EXIT_CODE
