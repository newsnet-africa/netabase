#!/bin/bash

# NetaBase Cross-Machine Writer Test Runner (5-Record Version)
# This script runs a writer node that stores exactly 5 records with unique keys
# Updated to use the new 5-record approach with unique key generation

set -e

# Default configuration
DEFAULT_WRITER_ADDR="0.0.0.0:9901"
DEFAULT_TEST_KEY="cross_machine_key"
DEFAULT_WRITER_TIMEOUT="0"  # 0 means run indefinitely

# Fixed 5 test records
readonly TEST_RECORDS=(
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
    echo -e "${CYAN}NetaBase Cross-Machine Writer Test Runner (5-Record Version)${NC}"
    echo ""
    echo "This script runs a writer node that stores exactly 5 records with unique keys:"
    for i in "${!TEST_RECORDS[@]}"; do
        echo "  Record $i: '${TEST_RECORDS[$i]}'"
    done
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -a, --addr ADDR      Listen address (default: $DEFAULT_WRITER_ADDR)"
    echo "  -k, --key KEY        Base test key for records (default: $DEFAULT_TEST_KEY)"
    echo "  -t, --timeout SECS   Timeout in seconds, 0 for indefinite (default: $DEFAULT_WRITER_TIMEOUT)"
    echo "  --verbose            Enable verbose logging"
    echo "  --dry-run            Show configuration without running the test"
    echo "  --validate-only      Only validate configuration and exit"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  NETABASE_WRITER_ADDR      Override listen address"
    echo "  NETABASE_TEST_KEY         Override base test key"
    echo "  NETABASE_WRITER_TIMEOUT   Override timeout (0 = indefinite)"
    echo ""
    echo "Examples:"
    echo "  # Basic usage with defaults (runs indefinitely)"
    echo "  $0"
    echo ""
    echo "  # Custom address and key"
    echo "  $0 --addr 0.0.0.0:8080 --key mykey"
    echo ""
    echo "  # Run for specific duration"
    echo "  $0 -a 192.168.1.100:9901 -t 300"
    echo ""
    echo "  # Verbose mode with dry-run"
    echo "  $0 --verbose --dry-run -a 0.0.0.0:9901"
    echo ""
    echo "  # Using environment variables"
    echo "  NETABASE_WRITER_ADDR=0.0.0.0:9901 \\"
    echo "  NETABASE_TEST_KEY=distributed_test \\"
    echo "  $0"
    echo ""
    echo "Key Generation:"
    echo "  Records are stored with unique keys based on the base key:"
    echo "  - {base_key}__0: 'Hello World'"
    echo "  - {base_key}__1: 'Test Record'"
    echo "  - {base_key}__2: 'Another Value'"
    echo "  - {base_key}__3: 'Fourth Record'"
    echo "  - {base_key}__4: 'Fifth Record'"
    echo ""
    echo "Network Setup:"
    echo "  1. Writer listens on specified address and stores 5 records"
    echo "  2. Reader machines connect to writer's IP address"
    echo "  3. Ensure firewall allows UDP traffic on the specified port"
    echo ""
    echo "The writer will run until stopped with Ctrl+C (or timeout if specified)"
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
    local writer_addr="$1"
    local timeout="$2"

    log_debug "Validating configuration..."

    # Validate address format
    if [[ ! "$writer_addr" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+:[0-9]+$ ]] && [[ ! "$writer_addr" =~ ^[a-zA-Z0-9.-]+:[0-9]+$ ]]; then
        log_error "Invalid address format: $writer_addr"
        log_error "Expected format: IP:PORT or HOSTNAME:PORT"
        return 1
    fi

    # Extract and validate port
    local port="${writer_addr##*:}"
    if ! [[ "$port" =~ ^[0-9]+$ ]] || [ "$port" -lt 1 ] || [ "$port" -gt 65535 ]; then
        log_error "Invalid port: $port (must be 1-65535)"
        return 1
    fi

    # Validate timeout
    if ! [[ "$timeout" =~ ^[0-9]+$ ]] || [ "$timeout" -lt 0 ]; then
        log_error "Invalid timeout: $timeout (must be non-negative integer, 0 for indefinite)"
        return 1
    fi

    # Check for potential issues
    local host="${writer_addr%:*}"
    if [[ "$host" != "0.0.0.0" ]] && [[ "$host" != "127.0.0.1" ]] && [[ "$host" != "localhost" ]]; then
        log_warning "Using specific IP address: $host"
        log_warning "Make sure this IP is actually bound to a network interface"
    fi

    if [ "$timeout" -gt 0 ] && [ "$timeout" -lt 30 ]; then
        log_warning "Timeout is very short ($timeout seconds). May not give readers enough time to connect."
    fi

    # Check for port conflicts
    if command -v netstat &> /dev/null; then
        if netstat -ln 2>/dev/null | grep -q ":$port "; then
            log_warning "Port $port appears to be in use. This may cause binding errors."
        fi
    fi

    log_debug "Configuration validation passed"
    return 0
}

get_local_ip() {
    local local_ip=""

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

get_all_local_ips() {
    local ips=""

    if command -v ip &> /dev/null; then
        ips=$(ip addr show | grep -oP 'inet \K[0-9.]+' | grep -v '127.0.0.1' | tr '\n' ' ')
    elif command -v ifconfig &> /dev/null; then
        ips=$(ifconfig | grep -Eo 'inet (addr:)?([0-9]*\.){3}[0-9]*' | grep -Eo '([0-9]*\.){3}[0-9]*' | grep -v '127.0.0.1' | tr '\n' ' ')
    fi

    echo "$ips"
}

display_configuration() {
    local writer_addr="$1"
    local test_key="$2"
    local timeout="$3"
    local local_ip
    local all_ips

    local_ip=$(get_local_ip)
    all_ips=$(get_all_local_ips)

    echo ""
    echo "========================== WRITER CONFIGURATION =========================="
    echo -e "${GREEN}Network Settings:${NC}"
    echo "  Listen Address: $writer_addr"
    echo "  Local IP(s): ${all_ips:-unknown}"

    if [ "$timeout" = "0" ]; then
        echo "  Runtime: Indefinite (until Ctrl+C)"
    else
        echo "  Runtime: ${timeout} seconds"
    fi

    echo ""
    echo -e "${GREEN}5-Record Test Settings:${NC}"
    echo "  Base Key: '$test_key'"
    echo "  Records to store:"
    for i in "${!TEST_RECORDS[@]}"; do
        echo "    ${test_key}__${i}: '${TEST_RECORDS[$i]}'"
    done
    echo ""

    echo -e "${BLUE}For Reader Machines:${NC}"
    if [[ "$writer_addr" == "0.0.0.0:"* ]]; then
        local port="${writer_addr#*:}"
        if [ "$local_ip" != "unknown" ]; then
            echo "  NETABASE_READER_CONNECT_ADDR=\"$local_ip:$port\""
        else
            echo "  NETABASE_READER_CONNECT_ADDR=\"<your-ip>:$port\""
        fi
    else
        echo "  NETABASE_READER_CONNECT_ADDR=\"$writer_addr\""
    fi
    echo "  NETABASE_TEST_KEY=\"$test_key\""
    echo ""
    echo "  Run reader with:"
    echo "    ./scripts/run_reader.sh --connect <writer-ip:port> --key $test_key"
    echo "  Or:"
    echo "    cargo test cross_machine_reader_5_records --ignored -- --nocapture"
    echo ""

    echo -e "${YELLOW}Firewall Configuration:${NC}"
    local port="${writer_addr##*:}"
    echo "  Port $port must be open for UDP traffic"
    echo "  Ubuntu/Debian: sudo ufw allow $port/udp"
    echo "  CentOS/RHEL:   sudo firewall-cmd --add-port=$port/udp --permanent"
    echo "  Windows:       netsh advfirewall firewall add rule name=\"NetaBase\" protocol=UDP dir=in localport=$port action=allow"
    echo ""

    echo -e "${CYAN}Network Testing:${NC}"
    echo "  Test connectivity from reader machine:"
    if [ "$local_ip" != "unknown" ]; then
        echo "    ping $local_ip"
        echo "    nc -u -z $local_ip $port"
    else
        echo "    ping <writer-ip>"
        echo "    nc -u -z <writer-ip> $port"
    fi
    echo "========================================================================"
    echo ""
}

run_rust_test() {
    local dry_run="$1"

    if [ "$dry_run" = "true" ]; then
        log_info "Dry run mode - configuration would be:"
        echo "  NETABASE_WRITER_ADDR=$WRITER_ADDR"
        echo "  NETABASE_TEST_KEY=$TEST_KEY"
        echo "  Test: cross_machine_writer_5_records"
        log_info "Would run: cargo test cross_machine_writer_5_records -- --nocapture --ignored"
        return 0
    fi

    log_info "Starting 5-record writer node..."
    if [ "$WRITER_TIMEOUT" = "0" ]; then
        log_warning "Writer will run indefinitely. Press Ctrl+C to stop."
    else
        log_info "Writer will run for $WRITER_TIMEOUT seconds"
    fi
    log_info "This may take a moment to compile and run..."
    echo ""

    # Export environment variables for the Rust test
    export NETABASE_WRITER_ADDR="$WRITER_ADDR"
    export NETABASE_TEST_KEY="$TEST_KEY"
    export NETABASE_WRITER_TIMEOUT="$WRITER_TIMEOUT"

    # Set Rust log level based on verbose flag
    if [ "$VERBOSE" = "true" ]; then
        export RUST_LOG="debug"
    else
        export RUST_LOG="info"
    fi

    # Run the 5-record test with proper error handling
    local exit_code=0
    if cargo test cross_machine_writer_5_records -- --nocapture --ignored; then
        echo ""
        log_success "5-record writer test completed successfully!"
        log_success "All 5 records have been stored with unique keys"
        if [ "$WRITER_TIMEOUT" = "0" ]; then
            log_info "Writer was stopped by user (Ctrl+C)"
        else
            log_info "Writer ran for the specified timeout duration"
        fi
    else
        exit_code=$?
        echo ""
        log_error "5-record writer test failed with exit code $exit_code"

        echo ""
        echo "Common issues and solutions:"
        echo "1. Port already in use:"
        echo "   → Check with: netstat -ln | grep :$port"
        echo "   → Kill conflicting process or use different port"
        echo "2. Permission denied:"
        echo "   → Use port > 1024 or run with appropriate privileges"
        echo "3. Address binding failed:"
        echo "   → Verify IP address is available on this machine"
        echo "   → Use 0.0.0.0:<port> to bind to all interfaces"
        echo "4. Compilation errors:"
        echo "   → Run: cargo build to check for dependency issues"
        echo "5. Network interface issues:"
        echo "   → Check available IPs with: ip addr show"

        return $exit_code
    fi
}

cleanup() {
    log_info "Writer shutdown initiated by user"
    log_info "Stopping 5-record writer node gracefully..."
    exit 0
}

validate_only_mode() {
    log_info "Configuration validation mode"

    # Test configuration parsing
    export NETABASE_WRITER_ADDR="$WRITER_ADDR"
    export NETABASE_TEST_KEY="$TEST_KEY"
    export NETABASE_WRITER_TIMEOUT="$WRITER_TIMEOUT"

    # Shell-based validation
    if validate_config "$WRITER_ADDR" "$WRITER_TIMEOUT"; then
        log_success "Configuration is valid"
        display_configuration "$WRITER_ADDR" "$TEST_KEY" "$WRITER_TIMEOUT"

        echo -e "${GREEN}Ready to run 5-record writer test with this configuration!${NC}"
        echo ""
        echo "Next steps:"
        echo "1. Run this script without --validate-only to start the writer"
        echo "2. On reader machines, use the NETABASE_READER_CONNECT_ADDR shown above"
        echo "3. Run the corresponding 5-record reader test"
        echo "4. Ensure firewall allows UDP traffic on the specified port"

        exit 0
    else
        log_error "Configuration validation failed"
        exit 1
    fi
}

# Parse command line arguments
WRITER_ADDR="$DEFAULT_WRITER_ADDR"
TEST_KEY="$DEFAULT_TEST_KEY"
WRITER_TIMEOUT="$DEFAULT_WRITER_TIMEOUT"
VERBOSE=false
DRY_RUN=false
VALIDATE_ONLY=false

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
        -t|--timeout)
            WRITER_TIMEOUT="$2"
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
WRITER_ADDR="${NETABASE_WRITER_ADDR:-$WRITER_ADDR}"
TEST_KEY="${NETABASE_TEST_KEY:-$TEST_KEY}"
WRITER_TIMEOUT="${NETABASE_WRITER_TIMEOUT:-$WRITER_TIMEOUT}"

# Main execution
main() {
    echo ""
    echo "====================================================================="
    echo "             NetaBase Cross-Machine Writer (5-Record Version)"
    echo "====================================================================="

    # Handle special modes first
    if [ "$VALIDATE_ONLY" = "true" ]; then
        validate_only_mode
        return
    fi

    check_prerequisites

    if ! validate_config "$WRITER_ADDR" "$WRITER_TIMEOUT"; then
        log_error "Configuration validation failed"
        exit 1
    fi

    display_configuration "$WRITER_ADDR" "$TEST_KEY" "$WRITER_TIMEOUT"

    # Set up trap for graceful shutdown
    trap cleanup SIGINT SIGTERM

    log_info "Configuration validated successfully"

    if [ "$VERBOSE" = "true" ]; then
        log_debug "Verbose mode enabled"
        log_debug "Writer will listen on: $WRITER_ADDR"
        log_debug "Storing 5 records under base key: $TEST_KEY"
        log_debug "Timeout: ${WRITER_TIMEOUT}s (0 = indefinite)"
    fi

    echo ""

    # Run the actual test
    if ! run_rust_test "$DRY_RUN"; then
        exit 1
    fi

    if [ "$DRY_RUN" != "true" ]; then
        echo ""
        log_success "5-record writer operation completed!"
        log_info "Check the logs above for detailed results"
    fi
}

# Run main function
main "$@"
