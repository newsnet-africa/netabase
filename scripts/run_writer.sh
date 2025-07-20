#!/bin/bash

# NetaBase Cross-Machine Writer Test Runner
# This script runs a writer node that stores records in the distributed hash table

set -e

# Default configuration
DEFAULT_WRITER_ADDR="0.0.0.0:9901"
DEFAULT_TEST_KEY="cross_machine_key"
DEFAULT_TEST_VALUES="Value1,Value2,Value3,HelloWorld"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

show_usage() {
    echo "NetaBase Cross-Machine Writer Test Runner"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -a, --addr ADDR      Listen address (default: $DEFAULT_WRITER_ADDR)"
    echo "  -k, --key KEY        Test key to store records under (default: $DEFAULT_TEST_KEY)"
    echo "  -v, --values VALUES  Comma-separated values to store (default: $DEFAULT_TEST_VALUES)"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  NETABASE_WRITER_ADDR     Override listen address"
    echo "  NETABASE_TEST_KEY        Override test key"
    echo "  NETABASE_TEST_VALUES     Override test values"
    echo ""
    echo "Examples:"
    echo "  $0"
    echo "  $0 --addr 0.0.0.0:8080 --key mykey --values 'Hello,World,Test'"
    echo "  $0 -a 192.168.1.100:9901 -v 'Data1,Data2,Data3'"
    echo ""
    echo "The writer will run indefinitely until stopped with Ctrl+C"
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

get_local_ip() {
    # Try to get the local IP address
    local_ip=""

    # Try different methods to get local IP
    if command -v ip &> /dev/null; then
        local_ip=$(ip route get 1.1.1.1 2>/dev/null | grep -oP 'src \K\S+' | head -1)
    elif command -v ifconfig &> /dev/null; then
        local_ip=$(ifconfig | grep -Eo 'inet (addr:)?([0-9]*\.){3}[0-9]*' | grep -Eo '([0-9]*\.){3}[0-9]*' | grep -v '127.0.0.1' | head -1)
    fi

    if [ -n "$local_ip" ]; then
        echo "$local_ip"
    else
        echo "unknown"
    fi
}

display_network_info() {
    local writer_addr="$1"
    local local_ip
    local_ip=$(get_local_ip)

    echo ""
    echo "========================== NETWORK INFO =========================="
    echo -e "${GREEN}Writer Configuration:${NC}"
    echo "  Listen Address: $writer_addr"
    echo "  Local IP: $local_ip"
    echo ""
    echo -e "${BLUE}For Reader Machine:${NC}"
    if [[ "$writer_addr" == "0.0.0.0:"* ]]; then
        local port="${writer_addr#*:}"
        echo "  Set NETABASE_READER_CONNECT_ADDR=\"$local_ip:$port\""
    else
        echo "  Set NETABASE_READER_CONNECT_ADDR=\"$writer_addr\""
    fi
    echo ""
    echo -e "${YELLOW}Firewall:${NC}"
    local port="${writer_addr##*:}"
    echo "  Make sure port $port is open for UDP traffic"
    echo "  Ubuntu/Debian: sudo ufw allow $port/udp"
    echo "  CentOS/RHEL:   sudo firewall-cmd --add-port=$port/udp --permanent"
    echo "==============================================================="
    echo ""
}

cleanup() {
    log_info "Shutting down writer node..."
    exit 0
}

# Parse command line arguments
WRITER_ADDR="$DEFAULT_WRITER_ADDR"
TEST_KEY="$DEFAULT_TEST_KEY"
TEST_VALUES="$DEFAULT_TEST_VALUES"

while [[ $# -gt 0 ]]; do
    case $1 in
        -a|--addr)
            WRITER_ADDR="$2"
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
WRITER_ADDR="${NETABASE_WRITER_ADDR:-$WRITER_ADDR}"
TEST_KEY="${NETABASE_TEST_KEY:-$TEST_KEY}"
TEST_VALUES="${NETABASE_TEST_VALUES:-$TEST_VALUES}"

# Main execution
main() {
    echo ""
    echo "====================================================================="
    echo "                  NetaBase Cross-Machine Writer"
    echo "====================================================================="

    check_prerequisites
    display_network_info "$WRITER_ADDR"

    # Set up trap for graceful shutdown
    trap cleanup SIGINT SIGTERM

    log_info "Starting writer node..."
    log_info "Listen Address: $WRITER_ADDR"
    log_info "Test Key: $TEST_KEY"
    log_info "Test Values: $TEST_VALUES"
    log_info ""
    log_warning "Writer will run indefinitely. Press Ctrl+C to stop."
    echo ""

    # Export environment variables
    export NETABASE_WRITER_ADDR="$WRITER_ADDR"
    export NETABASE_TEST_KEY="$TEST_KEY"
    export NETABASE_TEST_VALUES="$TEST_VALUES"

    # Run the test
    cargo test cross_machine_writer -- --nocapture --ignored
}

# Run main function
main "$@"
