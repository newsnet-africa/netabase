#!/bin/bash

# NetaBase Cross-Machine Reader Test Runner
# This script runs a reader node that retrieves records from the distributed hash table

set -e

# Default configuration
DEFAULT_READER_CONNECT_ADDR="127.0.0.1:9901"
DEFAULT_TEST_KEY="cross_machine_key"
DEFAULT_TEST_VALUES="Value1,Value2,Value3,HelloWorld"
DEFAULT_TEST_TIMEOUT="120"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

show_usage() {
    echo "NetaBase Cross-Machine Reader Test Runner"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -c, --connect ADDR   Writer address to connect to (default: $DEFAULT_READER_CONNECT_ADDR)"
    echo "  -k, --key KEY        Test key to retrieve records from (default: $DEFAULT_TEST_KEY)"
    echo "  -v, --values VALUES  Comma-separated expected values (default: $DEFAULT_TEST_VALUES)"
    echo "  -t, --timeout SECS   Timeout in seconds (default: $DEFAULT_TEST_TIMEOUT)"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  NETABASE_READER_CONNECT_ADDR  Override writer address"
    echo "  NETABASE_TEST_KEY             Override test key"
    echo "  NETABASE_TEST_VALUES          Override expected values"
    echo "  NETABASE_TEST_TIMEOUT         Override timeout"
    echo ""
    echo "Examples:"
    echo "  $0"
    echo "  $0 --connect 192.168.1.100:9901 --key mykey --values 'Hello,World,Test'"
    echo "  $0 -c 10.0.0.5:8080 -t 60"
    echo ""
    echo "The reader will attempt to connect to the writer and retrieve records"
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

test_connectivity() {
    local connect_addr="$1"
    local host="${connect_addr%:*}"
    local port="${connect_addr##*:}"

    log_info "Testing connectivity to writer at $connect_addr..."

    # Test ping first
    if command -v ping &> /dev/null; then
        if ping -c 1 -W 3 "$host" &> /dev/null; then
            log_success "Host $host is reachable"
        else
            log_warning "Host $host is not responding to ping (this may be normal)"
        fi
    fi

    # Test port connectivity if netcat is available
    if command -v nc &> /dev/null; then
        if nc -z -u -w 3 "$host" "$port" &> /dev/null; then
            log_success "Port $port on $host is reachable"
        else
            log_warning "Port $port on $host is not reachable (writer may not be running yet)"
        fi
    elif command -v telnet &> /dev/null; then
        # Telnet test (less reliable for UDP but better than nothing)
        if timeout 3 telnet "$host" "$port" &> /dev/null; then
            log_success "Host $host port $port appears to be reachable"
        else
            log_warning "Could not connect to $host:$port (writer may not be running yet)"
        fi
    else
        log_info "No network testing tools available (nc/telnet). Proceeding anyway."
    fi
}

display_network_info() {
    local connect_addr="$1"
    local test_key="$2"
    local expected_values="$3"
    local timeout="$4"

    echo ""
    echo "========================== READER INFO ==========================="
    echo -e "${GREEN}Reader Configuration:${NC}"
    echo "  Writer Address: $connect_addr"
    echo "  Test Key: $test_key"
    echo "  Expected Values: $expected_values"
    echo "  Timeout: ${timeout}s"
    echo ""
    echo -e "${BLUE}Prerequisites:${NC}"
    echo "  1. Writer node must be running on $connect_addr"
    echo "  2. Writer must have stored records under key '$test_key'"
    echo "  3. Network connectivity must exist between machines"
    echo ""
    echo -e "${YELLOW}Troubleshooting:${NC}"
    echo "  - If connection fails: Check writer is running and firewall allows UDP"
    echo "  - If records not found: Verify key matches and writer stored data"
    echo "  - If timeout: Increase timeout or check network latency"
    echo "==============================================================="
    echo ""
}

validate_config() {
    local connect_addr="$1"
    local timeout="$2"

    # Validate address format
    if [[ ! "$connect_addr" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+:[0-9]+$ ]] && [[ ! "$connect_addr" =~ ^[a-zA-Z0-9.-]+:[0-9]+$ ]]; then
        log_error "Invalid address format: $connect_addr"
        log_error "Expected format: IP:PORT or HOSTNAME:PORT"
        exit 1
    fi

    # Validate timeout
    if ! [[ "$timeout" =~ ^[0-9]+$ ]] || [ "$timeout" -le 0 ]; then
        log_error "Invalid timeout: $timeout (must be positive integer)"
        exit 1
    fi

    # Check if timeout is reasonable
    if [ "$timeout" -lt 10 ]; then
        log_warning "Timeout is very short ($timeout seconds). Consider increasing it."
    elif [ "$timeout" -gt 600 ]; then
        log_warning "Timeout is very long ($timeout seconds). This may not be necessary."
    fi
}

cleanup() {
    log_info "Reader test interrupted"
    exit 1
}

# Parse command line arguments
READER_CONNECT_ADDR="$DEFAULT_READER_CONNECT_ADDR"
TEST_KEY="$DEFAULT_TEST_KEY"
TEST_VALUES="$DEFAULT_TEST_VALUES"
TEST_TIMEOUT="$DEFAULT_TEST_TIMEOUT"

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
        -v|--values)
            TEST_VALUES="$2"
            shift 2
            ;;
        -t|--timeout)
            TEST_TIMEOUT="$2"
            shift 2
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Override with environment variables if set
READER_CONNECT_ADDR="${NETABASE_READER_CONNECT_ADDR:-$READER_CONNECT_ADDR}"
TEST_KEY="${NETABASE_TEST_KEY:-$TEST_KEY}"
TEST_VALUES="${NETABASE_TEST_VALUES:-$TEST_VALUES}"
TEST_TIMEOUT="${NETABASE_TEST_TIMEOUT:-$TEST_TIMEOUT}"

# Main execution
main() {
    echo ""
    echo "====================================================================="
    echo "                  NetaBase Cross-Machine Reader"
    echo "====================================================================="

    check_prerequisites
    validate_config "$READER_CONNECT_ADDR" "$TEST_TIMEOUT"
    display_network_info "$READER_CONNECT_ADDR" "$TEST_KEY" "$TEST_VALUES" "$TEST_TIMEOUT"

    # Test connectivity (non-blocking)
    test_connectivity "$READER_CONNECT_ADDR"

    # Set up trap for graceful shutdown
    trap cleanup SIGINT SIGTERM

    log_info "Starting reader node..."
    log_info "Connecting to writer at: $READER_CONNECT_ADDR"
    log_info "Looking for key: $TEST_KEY"
    log_info "Expected values: $TEST_VALUES"
    log_info "Timeout: ${TEST_TIMEOUT}s"
    echo ""
    log_info "Reader will attempt to retrieve records and then exit"
    echo ""

    # Export environment variables
    export NETABASE_READER_CONNECT_ADDR="$READER_CONNECT_ADDR"
    export NETABASE_TEST_KEY="$TEST_KEY"
    export NETABASE_TEST_VALUES="$TEST_VALUES"
    export NETABASE_TEST_TIMEOUT="$TEST_TIMEOUT"

    # Run the test
    if cargo test cross_machine_reader -- --nocapture --ignored; then
        echo ""
        log_success "Reader test completed successfully!"
        log_info "Records were successfully retrieved from the writer node"
    else
        echo ""
        log_error "Reader test failed!"
        log_error "Check the logs above for details"

        echo ""
        echo "Common issues and solutions:"
        echo "1. Writer not running: Start the writer node first"
        echo "2. Wrong address: Verify the writer's IP address and port"
        echo "3. Firewall blocking: Ensure UDP port is open on writer machine"
        echo "4. Network connectivity: Test ping and basic connectivity"
        echo "5. Key mismatch: Ensure both writer and reader use the same key"
        echo "6. No records stored: Check if writer successfully stored records"

        exit 1
    fi
}

# Run main function
main "$@"
