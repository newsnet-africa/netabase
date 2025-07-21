#!/bin/bash

# NetaBase Cross-Machine Multi-Record Reader Test Runner
# This script runs a reader node that retrieves multiple records with unique keys
# for testing persistence across machines and sessions

set -e

# Default configuration
DEFAULT_READER_CONNECT_ADDR="127.0.0.1:9901"
DEFAULT_READER_HOST="127.0.0.1"
DEFAULT_READER_PORT="9901"
DEFAULT_TEST_KEY="multi_record_test"
DEFAULT_TEST_TIMEOUT="120"    # 2 minutes default
DEFAULT_READER_RETRIES="3"    # Default retry attempts per record
DEFAULT_RECORD_COUNT="50"     # Default number of records to read
DEFAULT_TEST_SEED=""          # Default to random (empty string)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

show_usage() {
    echo -e "${CYAN}NetaBase Cross-Machine Multi-Record Reader Test Runner${NC}"
    echo ""
    echo "This script runs a reader node that retrieves multiple records with unique keys"
    echo "for testing persistence across machines and sessions."
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -c, --connect ADDR   Writer address to connect to (default: $DEFAULT_READER_CONNECT_ADDR)"
    echo "  -H, --host HOST      Writer host/IP address (default: $DEFAULT_READER_HOST)"
    echo "  -p, --port PORT      Writer port number (default: $DEFAULT_READER_PORT)"
    echo "  -k, --key KEY        Base test key for records (default: $DEFAULT_TEST_KEY)"
    echo "  -n, --count NUM      Number of records to read (default: $DEFAULT_RECORD_COUNT)"
    echo "  -s, --seed SEED      Seed for deterministic paths (default: random)"
    echo "  -t, --timeout SECS   Timeout in seconds (default: $DEFAULT_TEST_TIMEOUT)"
    echo "  -r, --retries NUM    Number of retry attempts per record (default: $DEFAULT_READER_RETRIES)"
    echo "  --verbose            Enable verbose logging"
    echo "  --dry-run            Show configuration without running the test"
    echo "  --validate-only      Only validate configuration and exit"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  NETABASE_READER_CONNECT_ADDR  Override writer address"
    echo "  NETABASE_READER_HOST          Override writer host"
    echo "  NETABASE_READER_PORT          Override writer port"
    echo "  NETABASE_TEST_KEY             Override base test key"
    echo "  NETABASE_RECORD_COUNT         Override number of records"
    echo "  NETABASE_TEST_SEED            Override deterministic seed"
    echo "  NETABASE_TEST_TIMEOUT         Override timeout"
    echo "  NETABASE_READER_RETRIES       Override retry attempts"
    echo ""
    echo "Examples:"
    echo "  # Basic usage with defaults"
    echo "  $0"
    echo ""
    echo "  # Connect to specific writer with custom key and record count"
    echo "  $0 --connect 192.168.1.100:9901 --key mykey --count 100"
    echo ""
    echo "  # Use separate host and port specification"
    echo "  $0 --host 192.168.1.100 --port 9901 --key mykey --count 25"
    echo ""
    echo "  # Short timeout with more retries"
    echo "  $0 -c 10.0.0.5:8080 -t 30 -r 5"
    echo ""
    echo "  # Use deterministic paths with a specific seed (must match writer seed)"
    echo "  $0 --seed 12345 --count 50"
    echo ""
    echo "  # Using environment variables"
    echo "  NETABASE_READER_CONNECT_ADDR=192.168.1.100:9901 \\"
    echo "  NETABASE_TEST_KEY=distributed_test \\"
    echo "  NETABASE_RECORD_COUNT=75 \\"
    echo "  $0"
    echo ""
    echo "Key Matching:"
    echo "  The reader looks for records with unique keys based on the base key:"
    echo "  - {base_key}__0: Expected 'Multi-Record Test Value 0'"
    echo "  - {base_key}__1: Expected 'Multi-Record Test Value 1'"
    echo "  - ... and so on"
    echo ""
    echo "The reader will use mDNS to discover writer nodes and retrieve all records"
    echo "with their unique keys and verify they match the expected values."
    echo ""
    echo "Deterministic Testing:"
    echo "  Using the --seed option ensures that the same temporary directories"
    echo "  are used across runs, allowing for testing persistence across sessions."
    echo "  When testing across machines, use the same seed value on all machines."
    echo ""
    echo "Results Reporting:"
    echo "  The script will report detailed results for each record attempt,"
    echo "  including success rate, failures, and reasons for any failures."
    echo ""
}

# Parse command line arguments
VERBOSE=false
DRY_RUN=false
VALIDATE_ONLY=false

while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        -c|--connect)
            export NETABASE_READER_CONNECT_ADDR="$2"
            shift 2
            ;;
        -H|--host)
            export NETABASE_READER_HOST="$2"
            shift 2
            ;;
        -p|--port)
            export NETABASE_READER_PORT="$2"
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
if [[ -n "$NETABASE_READER_HOST" && -n "$NETABASE_READER_PORT" && -z "$NETABASE_READER_CONNECT_ADDR" ]]; then
    export NETABASE_READER_CONNECT_ADDR="${NETABASE_READER_HOST}:${NETABASE_READER_PORT}"
    if $VERBOSE; then
        echo "Combined host and port into address: $NETABASE_READER_CONNECT_ADDR"
    fi
fi

# Use defaults if not set
: "${NETABASE_READER_CONNECT_ADDR:=$DEFAULT_READER_CONNECT_ADDR}"
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

# Print configuration
echo -e "${CYAN}NetaBase Multi-Record Reader Configuration:${NC}"
echo "Connect Address: $NETABASE_READER_CONNECT_ADDR"
echo "Test Key: $NETABASE_TEST_KEY"
echo "Record Count: $NETABASE_RECORD_COUNT"
echo "Timeout: ${NETABASE_TEST_TIMEOUT} seconds"
echo "Retries: $NETABASE_READER_RETRIES per record"
if [[ -n "$NETABASE_TEST_SEED" ]]; then
    echo "Test Seed: $NETABASE_TEST_SEED (deterministic paths)"
else
    echo "Test Seed: Random (non-deterministic paths)"
fi
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
echo -e "${CYAN}Starting Multi-Record Reader...${NC}"

# Run the reader test in the Rust crate
cd "$(dirname "$0")/.."
RUST_BACKTRACE=1 cargo test --package netabase --test network_cross_machine_multi cross_machine_reader_multi_records --no-run --message-format=json | jq -r 'select(.executable != null) | .executable' | xargs -I{} {} --exact

# Check exit code
EXIT_CODE=$?
if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}Reader test completed successfully${NC}"
else
    echo -e "${RED}Reader test failed with exit code $EXIT_CODE${NC}"
fi

exit $EXIT_CODE
