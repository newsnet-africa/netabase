#!/bin/bash

# NetaBase Cross-Machine Reader Test Runner (5-Record Version)
# This script runs a reader node that retrieves exactly 5 records with unique keys
# Updated to use the new 5-record approach with unique key generation

set -e

# Default configuration
DEFAULT_READER_CONNECT_ADDR="127.0.0.1:9901"
DEFAULT_TEST_KEY="cross_machine_key"
DEFAULT_TEST_TIMEOUT="120"
DEFAULT_READER_RETRIES="3"

# Fixed 5 expected test records (must match writer)
readonly EXPECTED_RECORDS=(
    "Hello World"
    "Test Record"
    "Another Value"
    "Fourth Record"
    "Fifth Record"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

show_usage() {
    echo -e "${CYAN}NetaBase Cross-Machine Reader Test Runner (5-Record Version)${NC}"
    echo ""
    echo "This script runs a reader node that retrieves exactly 5 records with unique keys:"
    for i in "${!EXPECTED_RECORDS[@]}"; do
        echo "  Record $i: '${EXPECTED_RECORDS[$i]}'"
    done
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -c, --connect ADDR   Writer address to connect to (default: $DEFAULT_READER_CONNECT_ADDR)"
    echo "  -k, --key KEY        Base test key for records (default: $DEFAULT_TEST_KEY)"
    echo "  -t, --timeout SECS   Timeout in seconds (default: $DEFAULT_TEST_TIMEOUT)"
    echo "  -r, --retries NUM    Number of retry attempts per record (default: $DEFAULT_READER_RETRIES)"
    echo "  --verbose            Enable verbose logging"
    echo "  --dry-run            Show configuration without running the test"
    echo "  --validate-only      Only validate configuration and exit"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  NETABASE_READER_CONNECT_ADDR  Override writer address"
    echo "  NETABASE_TEST_KEY             Override base test key"
    echo "  NETABASE_TEST_TIMEOUT         Override timeout"
    echo "  NETABASE_READER_RETRIES       Override retry attempts"
    echo ""
    echo "Examples:"
    echo "  # Basic usage with defaults"
    echo "  $0"
    echo ""
    echo "  # Connect to specific writer with custom key"
    echo "  $0 --connect 192.168.1.100:9901 --key mykey"
    echo ""
    echo "  # Short timeout for quick testing"
    echo "  $0 -c 10.0.0.5:8080 -t 30 -r 5"
    echo ""
    echo "  # Verbose mode with dry-run"
    echo "  $0 --verbose --dry-run -c 192.168.1.50:9901"
    echo ""
    echo "  # Using environment variables"
    echo "  NETABASE_READER_CONNECT_ADDR=192.168.1.100:9901 \\"
    echo "  NETABASE_TEST_KEY=distributed_test \\"
    echo "  $0"
    echo ""
    echo "Key Generation:"
    echo "  The reader looks for records with unique keys based on the base key:"
    echo "  - {base_key}__0: Expected 'Hello World'"
    echo "  - {base_key}__1: Expected 'Test Record'"
    echo "  - {base_key}__2: Expected 'Another Value'"
    echo "  - {base_key}__3: Expected 'Fourth Record'"
    echo "  - {base_key}__4: Expected 'Fifth Record'"
    echo ""
    echo "The reader will attempt to connect to the writer and retrieve all 5 records"
    echo "with their unique keys and verify they match the expected values."
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    if [ "$VERBOSE" = "true" ]; then
        echo -e "${CYAN}[DEBUG]${NC} $1"
    fi
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi

    if [ ! -f "Cargo.toml" ]; then
        log_error "Cargo.toml not found. Please run this script from the netabase project root."
        exit 1
    fi

    if ! grep -q "netabase" Cargo.toml; then
        log_error "Not in the netabase project directory. Please run from project root."
        exit 1
    fi

    log_success "Prerequisites check passed"
}

validate_config() {
    local connect_addr="$1"
    local timeout="$2"
    local retries="$3"

    log_debug "Validating configuration..."

    # Validate address format
    if [[ ! "$connect_addr" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+:[0-9]+$ ]] && [[ ! "$connect_addr" =~ ^[a-zA-Z0-9.-]+:[0-9]+$ ]]; then
        log_error "Invalid address format: $connect_addr"
        log_error "Expected format: IP:PORT or HOSTNAME:PORT"
        return 1
    fi

    # Extract and validate port
    local port="${connect_addr##*:}"
    if ! [[ "$port" =~ ^[0-9]+$ ]] || [ "$port" -lt 1 ] || [ "$port" -gt 65535 ]; then
        log_error "Invalid port: $port (must be 1-65535)"
        return 1
    fi

    # Validate timeout
    if ! [[ "$timeout" =~ ^[0-9]+$ ]] || [ "$timeout" -le 0 ]; then
        log_error "Invalid timeout: $timeout (must be positive integer)"
        return 1
    fi

    # Validate retries
    if ! [[ "$retries" =~ ^[0-9]+$ ]] || [ "$retries" -lt 0 ]; then
        log_error "Invalid retries: $retries (must be non-negative integer)"
        return 1
    fi

    # Check reasonable ranges
    if [ "$timeout" -lt 10 ]; then
        log_warning "Timeout is very short ($timeout seconds). 5-record test may need more time."
    elif [ "$timeout" -gt 600 ]; then
        log_warning "Timeout is very long ($timeout seconds). This may not be necessary."
    fi

    if [ "$retries" -gt 10 ]; then
        log_warning "High retry count ($retries). Consider reducing if network is reliable."
    fi

    log_debug "Configuration validation passed"
    return 0
}

test_connectivity() {
    local connect_addr="$1"
    local host="${connect_addr%:*}"
    local port="${connect_addr##*:}"

    log_info "Testing connectivity to writer at $connect_addr..."

    # Test basic network connectivity
    if command -v ping &> /dev/null; then
        log_debug "Testing host reachability with ping..."
        if timeout 3 ping -c 1 "$host" &> /dev/null; then
            log_success "Host $host is reachable"
        else
            log_warning "Host $host is not responding to ping (may be firewalled)"
        fi
    fi

    # Test port connectivity
    local port_reachable=false
    if command -v nc &> /dev/null; then
        log_debug "Testing port connectivity with netcat..."
        if nc -z -u -w 3 "$host" "$port" &> /dev/null; then
            log_success "UDP port $port on $host appears reachable"
            port_reachable=true
        fi
    elif command -v telnet &> /dev/null; then
        log_debug "Testing connectivity with telnet..."
        if timeout 3 bash -c "echo >/dev/tcp/$host/$port" &> /dev/null; then
            log_success "Host $host port $port appears reachable"
            port_reachable=true
        fi
    fi

    if [ "$port_reachable" = false ]; then
        log_warning "Cannot verify port connectivity (writer may not be running yet)"
    fi
}

display_configuration() {
    local connect_addr="$1"
    local test_key="$2"
    local timeout="$3"
    local retries="$4"

    echo ""
    echo "========================== READER CONFIGURATION =========================="
    echo -e "${GREEN}Connection Settings:${NC}"
    echo "  Writer Address: $connect_addr"
    echo "  Connection Timeout: ${timeout}s"
    echo "  Retry Attempts per Record: $retries"
    echo ""
    echo -e "${GREEN}5-Record Test Settings:${NC}"
    echo "  Base Key: '$test_key'"
    echo "  Expected Records:"
    for i in "${!EXPECTED_RECORDS[@]}"; do
        echo "    ${test_key}__${i}: '${EXPECTED_RECORDS[$i]}'"
    done
    echo ""
    echo -e "${BLUE}Prerequisites:${NC}"
    echo "  1. Writer node must be running on $connect_addr"
    echo "  2. Writer must have stored 5 records under base key '$test_key'"
    echo "  3. Network connectivity must exist between machines"
    echo "  4. Firewall must allow UDP traffic on port ${connect_addr##*:}"
    echo ""
    echo -e "${CYAN}Writer Commands:${NC}"
    echo "  Start writer with:"
    echo "    ./scripts/run_writer.sh --addr 0.0.0.0:${connect_addr##*:} --key $test_key"
    echo "  Or:"
    echo "    cargo test cross_machine_writer_5_records --ignored -- --nocapture"
    echo ""
    echo -e "${YELLOW}Troubleshooting Tips:${NC}"
    echo "  - Connection fails: Check writer is running and firewall allows UDP"
    echo "  - Records not found: Verify key matches and writer stored 5 records"
    echo "  - Timeout issues: Increase timeout or check network latency"
    echo "  - Partial results: Check if writer has finished storing all 5 records"
    echo "========================================================================"
    echo ""
}

run_rust_test() {
    local dry_run="$1"

    if [ "$dry_run" = "true" ]; then
        log_info "Dry run mode - configuration would be:"
        echo "  NETABASE_READER_CONNECT_ADDR=$READER_CONNECT_ADDR"
        echo "  NETABASE_TEST_KEY=$TEST_KEY"
        echo "  Test: cross_machine_reader_5_records"
        log_info "Would run: cargo test cross_machine_reader_5_records -- --nocapture --ignored"
        return 0
    fi

    log_info "Starting 5-record reader node..."
    log_info "This may take a moment to compile and run..."
    echo ""

    # Export environment variables for the Rust test
    export NETABASE_READER_CONNECT_ADDR="$READER_CONNECT_ADDR"
    export NETABASE_TEST_KEY="$TEST_KEY"
    export NETABASE_TEST_TIMEOUT="$TEST_TIMEOUT"
    export NETABASE_READER_RETRIES="$READER_RETRIES"

    # Set Rust log level based on verbose flag
    if [ "$VERBOSE" = "true" ]; then
        export RUST_LOG="debug"
    else
        export RUST_LOG="info"
    fi

    # Run the 5-record test with proper error handling
    local exit_code=0
    if cargo test cross_machine_reader_5_records -- --nocapture --ignored; then
        echo ""
        log_success "5-record reader test completed successfully!"
        log_success "All 5 records were retrieved and verified correctly"
    else
        exit_code=$?
        echo ""
        log_error "5-record reader test failed with exit code $exit_code"

        echo ""
        echo "Common issues and solutions:"
        echo "1. Writer not running:"
        echo "   → Start the 5-record writer node first using run_writer.sh"
        echo "2. Wrong address:"
        echo "   → Verify the writer's IP address and port with --connect"
        echo "3. Firewall blocking:"
        echo "   → Ensure UDP port is open on writer machine"
        echo "4. Network connectivity:"
        echo "   → Test with: ping <writer-ip>"
        echo "5. Key mismatch:"
        echo "   → Ensure both writer and reader use the same --key"
        echo "6. No records stored:"
        echo "   → Check if writer successfully stored all 5 records"
        echo "7. Timeout too short:"
        echo "   → Increase with --timeout <seconds> (5 records need more time)"
        echo "8. Partial records found:"
        echo "   → Wait for writer to complete, then retry"

        return $exit_code
    fi
}

cleanup() {
    log_info "5-record reader test interrupted by user"
    exit 1
}

validate_only_mode() {
    log_info "Configuration validation mode"

    # Test configuration parsing
    export NETABASE_READER_CONNECT_ADDR="$READER_CONNECT_ADDR"
    export NETABASE_TEST_KEY="$TEST_KEY"
    export NETABASE_TEST_TIMEOUT="$TEST_TIMEOUT"
    export NETABASE_READER_RETRIES="$READER_RETRIES"

    # Shell-based validation
    if validate_config "$READER_CONNECT_ADDR" "$TEST_TIMEOUT" "$READER_RETRIES"; then
        log_success "Configuration is valid"
        display_configuration "$READER_CONNECT_ADDR" "$TEST_KEY" "$TEST_TIMEOUT" "$READER_RETRIES"

        echo -e "${GREEN}Ready to run 5-record reader test with this configuration!${NC}"
        echo ""
        echo "Next steps:"
        echo "1. Ensure the writer node is running on $READER_CONNECT_ADDR"
        echo "2. Run this script without --validate-only to start the reader"
        echo "3. The reader will systematically search for all 5 records"

        exit 0
    else
        log_error "Configuration validation failed"
        exit 1
    fi
}

# Parse command line arguments
READER_CONNECT_ADDR="$DEFAULT_READER_CONNECT_ADDR"
TEST_KEY="$DEFAULT_TEST_KEY"
TEST_TIMEOUT="$DEFAULT_TEST_TIMEOUT"
READER_RETRIES="$DEFAULT_READER_RETRIES"
VERBOSE=false
DRY_RUN=false
VALIDATE_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--connect)
            READER_CONNECT_ADDR="$2"
            shift 2
            ;;
        -k|--key)
            TEST_KEY="$2"
            shift 2
            ;;
        -t|--timeout)
            TEST_TIMEOUT="$2"
            shift 2
            ;;
        -r|--retries)
            READER_RETRIES="$2"
            shift 2
            ;;
        --verbose)
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
            log_error "Unknown option: $1"
            echo ""
            show_usage
            exit 1
            ;;
    esac
done

# Override with environment variables if set (env vars have lower priority than CLI args)
READER_CONNECT_ADDR="${NETABASE_READER_CONNECT_ADDR:-$READER_CONNECT_ADDR}"
TEST_KEY="${NETABASE_TEST_KEY:-$TEST_KEY}"
TEST_TIMEOUT="${NETABASE_TEST_TIMEOUT:-$TEST_TIMEOUT}"
READER_RETRIES="${NETABASE_READER_RETRIES:-$READER_RETRIES}"

# Main execution
main() {
    echo ""
    echo "====================================================================="
    echo "             NetaBase Cross-Machine Reader (5-Record Version)"
    echo "====================================================================="

    # Handle special modes first
    if [ "$VALIDATE_ONLY" = "true" ]; then
        validate_only_mode
        return
    fi

    check_prerequisites

    if ! validate_config "$READER_CONNECT_ADDR" "$TEST_TIMEOUT" "$READER_RETRIES"; then
        log_error "Configuration validation failed"
        exit 1
    fi

    display_configuration "$READER_CONNECT_ADDR" "$TEST_KEY" "$TEST_TIMEOUT" "$READER_RETRIES"

    # Test connectivity (non-blocking)
    if [ "$DRY_RUN" != "true" ]; then
        test_connectivity "$READER_CONNECT_ADDR"
    fi

    # Set up trap for graceful shutdown
    trap cleanup SIGINT SIGTERM

    log_info "Configuration validated successfully"

    if [ "$VERBOSE" = "true" ]; then
        log_debug "Verbose mode enabled"
        log_debug "Reader will connect to: $READER_CONNECT_ADDR"
        log_debug "Looking for 5 records under base key: $TEST_KEY"
        log_debug "Timeout: ${TEST_TIMEOUT}s, Retries per record: $READER_RETRIES"
    fi

    echo ""

    # Run the actual test
    if ! run_rust_test "$DRY_RUN"; then
        exit 1
    fi

    if [ "$DRY_RUN" != "true" ]; then
        echo ""
        log_success "5-record reader operation completed!"
        log_info "Check the logs above for detailed results"
    fi
}

# Run main function
main "$@"
