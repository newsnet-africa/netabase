#!/bin/bash

# NetaBase Local Cross-Machine Test Runner (5-Record Version)
# This script runs both writer and reader nodes locally with exactly 5 records
# Updated to use the new 5-record approach with unique key generation

set -e

# Default configuration
DEFAULT_TEST_KEY="cross_machine_key"
DEFAULT_TEST_TIMEOUT="60"

# Fixed 5 test records (must match writer and reader)
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
    echo -e "${CYAN}NetaBase Local Cross-Machine Test Runner (5-Record Version)${NC}"
    echo ""
    echo "This script runs a local test with exactly 5 records using unique keys:"
    for i in "${!TEST_RECORDS[@]}"; do
        echo "  Record $i: '${TEST_RECORDS[$i]}'"
    done
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -k, --key KEY        Base test key for the 5 records (default: $DEFAULT_TEST_KEY)"
    echo "  -t, --timeout SECS   Timeout in seconds (default: $DEFAULT_TEST_TIMEOUT)"
    echo "  --verbose            Enable verbose logging"
    echo "  --dry-run            Show configuration without running the test"
    echo "  --validate-only      Only validate configuration and exit"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  NETABASE_TEST_KEY      Override base test key"
    echo "  NETABASE_TEST_TIMEOUT  Override timeout"
    echo ""
    echo "Examples:"
    echo "  # Basic usage with defaults"
    echo "  $0"
    echo ""
    echo "  # Custom key"
    echo "  $0 --key mytest"
    echo ""
    echo "  # Quick test with short timeout"
    echo "  $0 -t 30 -k quicktest"
    echo ""
    echo "  # Verbose mode with dry-run"
    echo "  $0 --verbose --dry-run"
    echo ""
    echo "  # Using environment variables"
    echo "  NETABASE_TEST_KEY=local_test \\"
    echo "  NETABASE_TEST_TIMEOUT=90 \\"
    echo "  $0"
    echo ""
    echo "Key Generation:"
    echo "  Records are stored and retrieved with unique keys based on the base key:"
    echo "  - {base_key}__0: 'Hello World'"
    echo "  - {base_key}__1: 'Test Record'"
    echo "  - {base_key}__2: 'Another Value'"
    echo "  - {base_key}__3: 'Fourth Record'"
    echo "  - {base_key}__4: 'Fifth Record'"
    echo ""
    echo "Description:"
    echo "  This script runs a local test that starts both a writer and reader"
    echo "  on the same machine using different ports. It's useful for:"
    echo "  - Testing the 5-record approach without needing multiple machines"
    echo "  - Validating configuration before deploying to multiple machines"
    echo "  - Development and debugging of DHT functionality"
    echo "  - CI/CD pipeline testing with consistent 5-record data"
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

    # Check available ports (basic check)
    if command -v netstat &> /dev/null; then
        local used_ports
        used_ports=$(netstat -ln 2>/dev/null | grep ":9901\|:9902" | wc -l)
        if [ "$used_ports" -gt 0 ]; then
            log_warning "Some test ports (9901-9902) appear to be in use."
            log_warning "This may cause port binding issues during the test."
        fi
    fi

    log_success "Prerequisites check passed"
}

validate_config() {
    local test_key="$1"
    local timeout="$2"

    log_debug "Validating configuration..."

    # Validate test key
    if [ -z "$test_key" ] || [ "${#test_key}" -lt 1 ]; then
        log_error "Test key cannot be empty"
        return 1
    fi

    if [ "${#test_key}" -gt 100 ]; then
        log_warning "Test key is very long (${#test_key} characters). This may cause issues."
    fi

    # Validate timeout
    if ! [[ "$timeout" =~ ^[0-9]+$ ]] || [ "$timeout" -le 0 ]; then
        log_error "Invalid timeout: $timeout (must be positive integer)"
        return 1
    fi

    # Check reasonable ranges
    if [ "$timeout" -lt 15 ]; then
        log_warning "Timeout is very short ($timeout seconds). 5-record local tests may need more time."
    elif [ "$timeout" -gt 300 ]; then
        log_warning "Timeout is very long ($timeout seconds). Local tests usually complete faster."
    fi

    log_debug "Configuration validation passed"
    return 0
}

display_configuration() {
    local test_key="$1"
    local timeout="$2"

    echo ""
    echo "======================== LOCAL 5-RECORD TEST CONFIGURATION ========================"
    echo -e "${GREEN}Test Settings:${NC}"
    echo "  Base Key: '$test_key'"
    echo "  Records to test:"
    for i in "${!TEST_RECORDS[@]}"; do
        echo "    ${test_key}__${i}: '${TEST_RECORDS[$i]}'"
    done
    echo "  Timeout: ${timeout}s"
    echo ""
    echo -e "${BLUE}Test Process:${NC}"
    echo "  1. Start writer node on local port (automatically assigned)"
    echo "  2. Writer stores all 5 records with unique keys"
    echo "  3. Start reader node to connect to writer"
    echo "  4. Reader systematically retrieves all 5 records"
    echo "  5. Verify all records match expected values"
    echo "  6. Both nodes shut down automatically"
    echo ""
    echo -e "${CYAN}Network Usage:${NC}"
    echo "  - Uses local loopback interface (127.0.0.1)"
    echo "  - Automatically assigns available ports"
    echo "  - No firewall configuration needed"
    echo "  - No external network connectivity required"
    echo ""
    echo -e "${YELLOW}What This Tests:${NC}"
    echo "  ✓ DHT 5-record storage with unique keys"
    echo "  ✓ Systematic multi-record retrieval"
    echo "  ✓ Network protocol functionality"
    echo "  ✓ Configuration system"
    echo "  ✓ Cross-node communication"
    echo "  ✓ Data integrity verification"
    echo "  ✓ No record overwrite validation"
    echo "================================================================================"
    echo ""
}

run_rust_test() {
    local dry_run="$1"

    if [ "$dry_run" = "true" ]; then
        log_info "Dry run mode - configuration would be:"
        echo "  NETABASE_TEST_KEY=$TEST_KEY"
        echo "  Test: cross_machine_local_test (5-record version)"
        log_info "Would run: cargo test cross_machine_local_test -- --nocapture --ignored"
        return 0
    fi

    log_info "Starting local 5-record cross-machine test..."
    log_info "This will run both writer and reader nodes locally with 5 records"
    log_info "Compilation and test execution may take a moment..."
    echo ""

    # Export environment variables for the Rust test
    export NETABASE_TEST_KEY="$TEST_KEY"
    export NETABASE_TEST_TIMEOUT="$TEST_TIMEOUT"

    # Set Rust log level based on verbose flag
    if [ "$VERBOSE" = "true" ]; then
        export RUST_LOG="debug"
    else
        export RUST_LOG="info"
    fi

    # Run the test with proper error handling
    local exit_code=0
    if cargo test cross_machine_local_test -- --nocapture --ignored; then
        echo ""
        log_success "Local 5-record test completed successfully!"
        log_success "Both writer and reader nodes functioned correctly"
        log_success "All 5 records were stored and retrieved with unique keys"
        log_info "✓ Writer stored: Hello World, Test Record, Another Value, Fourth Record, Fifth Record"
        log_info "✓ Reader found all 5 records correctly"
    else
        exit_code=$?
        echo ""
        log_error "Local 5-record test failed with exit code $exit_code"

        echo ""
        echo "Common issues and solutions:"
        echo "1. Compilation errors:"
        echo "   → Run 'cargo build' to check for dependency issues"
        echo "   → Ensure you're in the correct directory (netabase project root)"
        echo "2. Port binding issues:"
        echo "   → Close other applications using ports in the 9900-9999 range"
        echo "   → Check with: netstat -ln | grep :99"
        echo "3. Test timeout:"
        echo "   → Increase timeout with --timeout <seconds> (try 120 for 5 records)"
        echo "   → Check system performance and available resources"
        echo "4. Record retrieval issues:"
        echo "   → Ensure all 5 records are being stored and found correctly"
        echo "   → Check logs for specific record retrieval failures"
        echo "5. Configuration errors:"
        echo "   → Validate configuration with --validate-only"
        echo "   → Check test key format (no special characters recommended)"
        echo "6. Resource issues:"
        echo "   → Ensure sufficient memory and CPU available"
        echo "   → Close unnecessary applications"

        return $exit_code
    fi
}

cleanup() {
    log_info "Local 5-record test interrupted by user"
    exit 1
}

validate_only_mode() {
    log_info "Configuration validation mode"

    # Test configuration parsing
    export NETABASE_TEST_KEY="$TEST_KEY"
    export NETABASE_TEST_TIMEOUT="$TEST_TIMEOUT"

    # Shell-based validation
    if validate_config "$TEST_KEY" "$TEST_TIMEOUT"; then
        log_success "Configuration is valid"
        display_configuration "$TEST_KEY" "$TEST_TIMEOUT"

        echo -e "${GREEN}Ready to run local 5-record test with this configuration!${NC}"
        echo ""
        echo "Next steps:"
        echo "1. Run this script without --validate-only to start the test"
        echo "2. The test will run automatically and show results"
        echo "3. All 5 records will be tested with unique keys"
        echo "4. No additional setup or configuration required"

        exit 0
    else
        log_error "Configuration validation failed"
        exit 1
    fi
}

# Parse command line arguments
TEST_KEY="$DEFAULT_TEST_KEY"
TEST_TIMEOUT="$DEFAULT_TEST_TIMEOUT"
VERBOSE=false
DRY_RUN=false
VALIDATE_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -k|--key)
            TEST_KEY="$2"
            shift 2
            ;;
        -t|--timeout)
            TEST_TIMEOUT="$2"
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
TEST_KEY="${NETABASE_TEST_KEY:-$TEST_KEY}"
TEST_TIMEOUT="${NETABASE_TEST_TIMEOUT:-$TEST_TIMEOUT}"

# Main execution
main() {
    echo ""
    echo "====================================================================="
    echo "            NetaBase Local Cross-Machine Test (5-Record Version)"
    echo "====================================================================="

    # Handle special modes first
    if [ "$VALIDATE_ONLY" = "true" ]; then
        validate_only_mode
        return
    fi

    check_prerequisites

    if ! validate_config "$TEST_KEY" "$TEST_TIMEOUT"; then
        log_error "Configuration validation failed"
        exit 1
    fi

    display_configuration "$TEST_KEY" "$TEST_TIMEOUT"

    # Set up trap for graceful shutdown
    trap cleanup SIGINT SIGTERM

    log_info "Configuration validated successfully"

    if [ "$VERBOSE" = "true" ]; then
        log_debug "Verbose mode enabled"
        log_debug "Test key: $TEST_KEY"
        log_debug "Testing 5 records with unique keys"
        log_debug "Timeout: ${TEST_TIMEOUT}s"
    fi

    echo ""

    # Run the actual test
    if ! run_rust_test "$DRY_RUN"; then
        exit 1
    fi

    if [ "$DRY_RUN" != "true" ]; then
        echo ""
        log_success "Local 5-record test operation completed!"
        log_info "The 5-record cross-machine setup has been validated locally"
        log_info "You can now deploy to multiple machines with confidence"
        log_info "Use the same base key on all machines for consistency"
    fi
}

# Run main function
main "$@"
