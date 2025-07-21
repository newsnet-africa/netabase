#!/bin/bash

# NetaBase Cross-Machine Multi-Record Writer Test Runner
# This script runs a writer node that stores multiple records with unique keys
# for testing persistence across machines and sessions

set -e

# Default configuration
DEFAULT_WRITER_ADDR="0.0.0.0:9901"
DEFAULT_WRITER_HOST="0.0.0.0"
DEFAULT_WRITER_PORT="9901"
DEFAULT_TEST_KEY="multi_record_test"
DEFAULT_WRITER_TIMEOUT="300"  # 5 minutes default, long enough for readers to connect
DEFAULT_RECORD_COUNT="50"     # Default number of records to create
DEFAULT_TEST_SEED=""          # Default to random (empty string)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

show_usage() {
    echo -e "${CYAN}NetaBase Cross-Machine Multi-Record Writer Test Runner${NC}"
    echo ""
    echo "This script runs a writer node that stores multiple records with unique keys"
    echo "for testing persistence across machines and sessions."
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -a, --addr ADDR      Listen address (default: $DEFAULT_WRITER_ADDR)"
    echo "  -H, --host HOST      Listen host/IP address (default: $DEFAULT_WRITER_HOST)"
    echo "  -p, --port PORT      Listen port number (default: $DEFAULT_WRITER_PORT)"
    echo "  -k, --key KEY        Base test key for records (default: $DEFAULT_TEST_KEY)"
    echo "  -c, --count NUM      Number of records to create (default: $DEFAULT_RECORD_COUNT)"
    echo "  -s, --seed SEED      Seed for deterministic paths (default: random)"
    echo "  -t, --timeout SECS   Timeout in seconds, 0 for indefinite (default: $DEFAULT_WRITER_TIMEOUT)"
    echo "  --verbose            Enable verbose logging"
    echo "  --dry-run            Show configuration without running the test"
    echo "  --validate-only      Only validate configuration and exit"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  NETABASE_WRITER_ADDR             Override listen address"
    echo "  NETABASE_WRITER_HOST             Override listen host"
    echo "  NETABASE_WRITER_PORT             Override listen port"
    echo "  NETABASE_TEST_KEY                Override base test key"
    echo "  NETABASE_RECORD_COUNT            Override number of records"
    echo "  NETABASE_TEST_SEED               Override deterministic seed"
    echo "  NETABASE_WRITER_TIMEOUT          Override timeout"
    echo ""
    echo "Examples:"
    echo "  # Basic usage with defaults"
    echo "  $0"
    echo ""
    echo "  # Listen on specific address with custom key and 100 records"
    echo "  $0 --addr 0.0.0.0:9902 --key mykey --count 100"
    echo ""
    echo "  # Use separate host and port specification"
    echo "  $0 --host 0.0.0.0 --port 9902 --key mykey --count 25"
    echo ""
    echo "  # Run indefinitely with verbose output"
    echo "  $0 -a 192.168.1.100:8080 -t 0 --verbose"
    echo ""
    echo "  # Use deterministic paths with a specific seed"
    echo "  $0 --seed 12345 --count 50"
    echo ""
    echo "  # Using environment variables"
    echo "  NETABASE_WRITER_ADDR=0.0.0.0:9902 \\"
    echo "  NETABASE_TEST_KEY=distributed_test \\"
    echo "  NETABASE_RECORD_COUNT=75 \\"
    echo "  $0"
    echo ""
    echo "Key Generation:"
    echo "  The writer stores records with unique keys based on the base key:"
    echo "  - {base_key}__0: Contains 'Multi-Record Test Value 0'"
    echo "  - {base_key}__1: Contains 'Multi-Record Test Value 1'"
    echo "  - ... and so on"
    echo ""
    echo "The writer will listen for connections and store all records,"
    echo "making them available for reader nodes to retrieve via DHT queries."
    echo ""
    echo "Network Setup:"
    echo "  1. Writer listens on specified address and stores records"
    echo "  2. Reader machines connect to writer's IP address"
    echo "  3. Ensure firewall allows UDP traffic on the specified port"
    echo ""
    echo "Deterministic Testing:"
    echo "  Using the --seed option ensures that the same temporary directories"
    echo "  are used across runs, allowing for testing persistence across sessions."
    echo "  When testing across machines, use the same seed value on all machines."
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
            shift 2
            ;;
        -H|--host)
            export NETABASE_WRITER_HOST="$2"
            shift 2
            ;;
        -p|--port)
            export NETABASE_WRITER_PORT="$2"
            shift 2
            ;;
        -k|--key)
            export NETABASE_TEST_KEY="$2"
            shift 2
            ;;
        -c|--count)
            export NETABASE_RECORD_COUNT="$2"
            shift 2
            ;;
        -s|--seed)
            export NETABASE_TEST_SEED="$2"
            shift 2
            ;;
        -t|--timeout)
            export NETABASE_WRITER_TIMEOUT="$2"
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
if [[ -n "$NETABASE_WRITER_HOST" && -n "$NETABASE_WRITER_PORT" && -z "$NETABASE_WRITER_ADDR" ]]; then
    export NETABASE_WRITER_ADDR="${NETABASE_WRITER_HOST}:${NETABASE_WRITER_PORT}"
    if $VERBOSE; then
        echo "Combined host and port into address: $NETABASE_WRITER_ADDR"
    fi
fi

# Use defaults if not set
: "${NETABASE_WRITER_ADDR:=$DEFAULT_WRITER_ADDR}"
: "${NETABASE_TEST_KEY:=$DEFAULT_TEST_KEY}"
: "${NETABASE_RECORD_COUNT:=$DEFAULT_RECORD_COUNT}"
: "${NETABASE_WRITER_TIMEOUT:=$DEFAULT_WRITER_TIMEOUT}"
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

# Validate timeout is a non-negative integer
if ! [[ "$NETABASE_WRITER_TIMEOUT" =~ ^[0-9]+$ ]]; then
    echo -e "${RED}Error: Timeout must be a non-negative integer${NC}"
    exit 1
fi

# Print configuration
echo -e "${CYAN}NetaBase Multi-Record Writer Configuration:${NC}"
echo "Writer Address: $NETABASE_WRITER_ADDR"
echo "Test Key: $NETABASE_TEST_KEY"
echo "Record Count: $NETABASE_RECORD_COUNT"
echo "Timeout: ${NETABASE_WRITER_TIMEOUT} seconds (0 = indefinite)"
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
echo -e "${CYAN}Starting Multi-Record Writer...${NC}"

# Run the writer test in the Rust crate
cd "$(dirname "$0")/.."
RUST_BACKTRACE=1 cargo test --package netabase --test network_cross_machine_multi cross_machine_writer_multi_records --no-run --message-format=json | jq -r 'select(.executable != null) | .executable' | xargs -I{} {} --exact

echo -e "${GREEN}Writer test completed${NC}"
