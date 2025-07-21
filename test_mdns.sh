#!/bin/bash

# NetaBase mDNS Discovery Test Script
# This script tests if mDNS peer discovery is working correctly between writer and reader nodes

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default configuration
DEFAULT_TEST_KEY="mdns_test"
DEFAULT_TIMEOUT="30"

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

show_usage() {
    echo -e "${CYAN}NetaBase mDNS Discovery Test Script${NC}"
    echo ""
    echo "This script tests mDNS peer discovery between writer and reader nodes."
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -k, --key KEY        Test key for records (default: $DEFAULT_TEST_KEY)"
    echo "  -t, --timeout SECS   Test timeout in seconds (default: $DEFAULT_TIMEOUT)"
    echo "  --test-system        Test system mDNS functionality"
    echo "  --test-writer        Test writer mDNS advertisement"
    echo "  --test-reader        Test reader mDNS discovery"
    echo "  --test-full          Run full writer/reader mDNS test"
    echo "  --verbose            Enable verbose logging"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  # Test system mDNS"
    echo "  $0 --test-system"
    echo ""
    echo "  # Test writer advertising"
    echo "  $0 --test-writer --verbose"
    echo ""
    echo "  # Test reader discovery"
    echo "  $0 --test-reader --verbose"
    echo ""
    echo "  # Full integration test"
    echo "  $0 --test-full -k test_key -t 60"
}

test_system_mdns() {
    log_info "Testing system mDNS functionality..."

    echo ""
    echo "1. Checking if mDNS is enabled on system:"

    # Check for Avahi (Linux)
    if command -v avahi-daemon &> /dev/null; then
        if systemctl is-active --quiet avahi-daemon 2>/dev/null; then
            log_success "Avahi daemon is running"
        else
            log_warning "Avahi daemon is not running - may need to start it"
            echo "  Try: sudo systemctl start avahi-daemon"
        fi
    elif command -v avahi-browse &> /dev/null; then
        log_info "Avahi tools are available"
        echo "  Testing mDNS browsing:"
        timeout 5 avahi-browse -t _services._dns-sd._udp.local 2>/dev/null || log_warning "No mDNS services found"
    else
        log_warning "Avahi not found - mDNS may not be available"
    fi

    # Check for Bonjour (macOS)
    if command -v dns-sd &> /dev/null; then
        log_info "Bonjour (dns-sd) is available"
        echo "  Testing service discovery:"
        timeout 3 dns-sd -B _services._dns-sd._udp.local. 2>/dev/null || log_warning "No services discovered via Bonjour"
    fi

    echo ""
    echo "2. Network interface check:"
    if command -v ip &> /dev/null; then
        echo "Active network interfaces:"
        ip addr show | grep -E "^[0-9]+:|inet " | head -10
    elif command -v ifconfig &> /dev/null; then
        echo "Active network interfaces:"
        ifconfig | grep -E "^[a-z]|inet " | head -10
    fi

    echo ""
    echo "3. Multicast support check:"
    if ip route show | grep -q "224.0.0.0/4"; then
        log_success "Multicast routing appears to be configured"
    else
        log_warning "Multicast routing may not be properly configured"
    fi
}

test_writer_mdns() {
    local test_key="$1"
    local timeout="$2"
    local verbose="$3"

    log_info "Testing writer mDNS advertisement..."
    log_info "Key: $test_key, Timeout: ${timeout}s"

    export NETABASE_TEST_KEY="$test_key"
    export NETABASE_TEST_TIMEOUT="$timeout"

    if [ "$verbose" = "true" ]; then
        export RUST_LOG="debug"
    else
        export RUST_LOG="info"
    fi

    log_info "Starting writer node with mDNS..."
    log_info "Watch for 'Writer: Discovered peer via mDNS' messages"

    # Run writer in background and capture logs
    cargo test cross_machine_writer_5_records --ignored -- --nocapture &
    local writer_pid=$!

    log_info "Writer started with PID: $writer_pid"
    log_info "Let it run for ${timeout} seconds to test mDNS advertising..."

    sleep "$timeout"

    log_info "Stopping writer..."
    kill "$writer_pid" 2>/dev/null || true
    wait "$writer_pid" 2>/dev/null || true

    log_success "Writer mDNS test completed"
}

test_reader_mdns() {
    local test_key="$1"
    local timeout="$2"
    local verbose="$3"

    log_info "Testing reader mDNS discovery..."
    log_info "Key: $test_key, Timeout: ${timeout}s"

    export NETABASE_READER_CONNECT_ADDR="127.0.0.1:9901"  # Not used with mDNS but required
    export NETABASE_TEST_KEY="$test_key"
    export NETABASE_TEST_TIMEOUT="$timeout"

    if [ "$verbose" = "true" ]; then
        export RUST_LOG="debug"
    else
        export RUST_LOG="info"
    fi

    log_info "Starting reader node with mDNS discovery..."
    log_info "Watch for 'Reader: Discovered peer via mDNS' messages"

    if cargo test cross_machine_reader_5_records --ignored -- --nocapture; then
        log_success "Reader mDNS discovery test completed successfully"
    else
        log_warning "Reader test completed with issues (expected if no writer running)"
    fi
}

test_full_mdns() {
    local test_key="$1"
    local timeout="$2"
    local verbose="$3"

    log_info "Running full mDNS integration test..."
    log_info "This will start a writer, then a reader to test mDNS discovery"

    # Start writer in background
    log_info "=== PHASE 1: Starting Writer ==="
    export NETABASE_TEST_KEY="$test_key"
    export NETABASE_TEST_TIMEOUT="300"  # Writer runs longer

    if [ "$verbose" = "true" ]; then
        export RUST_LOG="debug"
    else
        export RUST_LOG="info"
    fi

    cargo test cross_machine_writer_5_records --ignored -- --nocapture &
    local writer_pid=$!

    log_info "Writer started with PID: $writer_pid"
    log_info "Waiting for writer to start and advertise via mDNS..."
    sleep 10

    # Start reader
    log_info "=== PHASE 2: Starting Reader ==="
    export NETABASE_READER_CONNECT_ADDR="127.0.0.1:9901"  # Not used with mDNS
    export NETABASE_TEST_TIMEOUT="$timeout"

    local reader_success=false
    if cargo test cross_machine_reader_5_records --ignored -- --nocapture; then
        log_success "Reader successfully discovered writer via mDNS and retrieved records!"
        reader_success=true
    else
        log_error "Reader failed to discover writer or retrieve records via mDNS"
    fi

    # Clean up writer
    log_info "=== CLEANUP ==="
    log_info "Stopping writer..."
    kill "$writer_pid" 2>/dev/null || true
    wait "$writer_pid" 2>/dev/null || true

    if [ "$reader_success" = "true" ]; then
        log_success "Full mDNS integration test PASSED!"
        return 0
    else
        log_error "Full mDNS integration test FAILED!"
        return 1
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

# Parse command line arguments
TEST_KEY="$DEFAULT_TEST_KEY"
TIMEOUT="$DEFAULT_TIMEOUT"
VERBOSE=false
TEST_SYSTEM=false
TEST_WRITER=false
TEST_READER=false
TEST_FULL=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -k|--key)
            TEST_KEY="$2"
            shift 2
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --test-system)
            TEST_SYSTEM=true
            shift
            ;;
        --test-writer)
            TEST_WRITER=true
            shift
            ;;
        --test-reader)
            TEST_READER=true
            shift
            ;;
        --test-full)
            TEST_FULL=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
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

# Main execution
main() {
    echo ""
    echo "====================================================================="
    echo "                NetaBase mDNS Discovery Test Tool"
    echo "====================================================================="
    echo ""

    log_info "Configuration:"
    echo "  Test Key: $TEST_KEY"
    echo "  Timeout: ${TIMEOUT}s"
    echo "  Verbose: $VERBOSE"
    echo ""

    check_prerequisites

    local any_test_run=false

    if [ "$TEST_SYSTEM" = "true" ]; then
        any_test_run=true
        echo ""
        log_info "=== SYSTEM mDNS TEST ==="
        test_system_mdns
    fi

    if [ "$TEST_WRITER" = "true" ]; then
        any_test_run=true
        echo ""
        log_info "=== WRITER mDNS TEST ==="
        test_writer_mdns "$TEST_KEY" "$TIMEOUT" "$VERBOSE"
    fi

    if [ "$TEST_READER" = "true" ]; then
        any_test_run=true
        echo ""
        log_info "=== READER mDNS TEST ==="
        test_reader_mdns "$TEST_KEY" "$TIMEOUT" "$VERBOSE"
    fi

    if [ "$TEST_FULL" = "true" ]; then
        any_test_run=true
        echo ""
        log_info "=== FULL mDNS INTEGRATION TEST ==="
        if test_full_mdns "$TEST_KEY" "$TIMEOUT" "$VERBOSE"; then
            echo ""
            log_success "All mDNS tests PASSED!"
        else
            echo ""
            log_error "mDNS integration test FAILED!"
            exit 1
        fi
    fi

    if [ "$any_test_run" = "false" ]; then
        log_warning "No tests specified. Use --help for options."
        echo ""
        log_info "Quick system check:"
        test_system_mdns
        echo ""
        log_info "Available tests:"
        echo "  $0 --test-system    # Test system mDNS support"
        echo "  $0 --test-writer    # Test writer mDNS advertising"
        echo "  $0 --test-reader    # Test reader mDNS discovery"
        echo "  $0 --test-full      # Full integration test"
    fi

    echo ""
    log_info "mDNS test session completed"
}

# Run main function
main "$@"
