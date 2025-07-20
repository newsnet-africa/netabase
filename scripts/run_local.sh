#!/bin/bash

# NetaBase Local Cross-Machine Test Runner
# This script runs both writer and reader nodes on the same machine for testing
# Updated to use the new configuration system with improved argument handling

set -e

# Default configuration
DEFAULT_TEST_KEY="cross_machine_key"
DEFAULT_TEST_VALUES="Value1,Value2,Value3,HelloWorld"
DEFAULT_TEST_TIMEOUT="60"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

show_usage() {
    echo -e "${CYAN}NetaBase Local Cross-Machine Test Runner${NC}"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -k, --key KEY        Test key to use for the test (default: $DEFAULT_TEST_KEY)"
    echo "  -v, --values VALUES  Comma-separated values to test (default: $DEFAULT_TEST_VALUES)"
    echo "  -t, --timeout SECS   Timeout in seconds (default: $DEFAULT_TEST_TIMEOUT)"
    echo "  --verbose            Enable verbose logging"
    echo "  --dry-run            Show configuration without running the test"
    echo "  --validate-only      Only validate configuration and exit"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  NETABASE_TEST_KEY      Override test key"
    echo "  NETABASE_TEST_VALUES   Override test values"
    echo "  NETABASE_TEST_TIMEOUT  Override timeout"
    echo ""
    echo "Examples:"
    echo "  # Basic usage with defaults"
    echo "  $0"
    echo ""
    echo "  # Custom key and values"
    echo "  $0 --key mytest --values 'Hello,World,Local,Test'"
    echo ""
    echo "  # Quick test with short timeout"
    echo "  $0 -t 30 -k quicktest"
    echo ""
    echo "  # Verbose mode with dry-run"
    echo "  $0 --verbose --dry-run"
    echo ""
    echo "  # Using environment variables"
    echo "  NETABASE_TEST_KEY=local_test \\"
    echo "  NETABASE_TEST_VALUES='Data1,Data2,Data3' \\"
    echo "  NETABASE_TEST_TIMEOUT=90 \\"
    echo "  $0"
    echo ""
    echo "Description:"
    echo "  This script runs a local test that starts both a writer and reader"
    echo "  on the same machine using different ports. It's useful for:"
    echo "  - Testing the cross-machine setup without needing multiple machines"
    echo "  - Validating configuration before deploying to multiple machines"
    echo "  - Development and debugging of DHT functionality"
    echo "  - CI/CD pipeline testing"
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

    # Check for required dependencies
    if ! grep -q "clap.*=" Cargo.toml && ! grep -q "envy.*=" Cargo.toml; then
        log_warning "Configuration dependencies may not be installed."
        log_info "Run 'cargo build' to ensure all dependencies are available."
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
    if [ "$timeout" -lt 10 ]; then
        log_warning "Timeout is very short ($timeout seconds). Local tests may need more time."
    elif [ "$timeout" -gt 300 ]; then
        log_warning "Timeout is very long ($timeout seconds). Local tests usually complete faster."
    fi

    log_debug "Configuration validation passed"
    return 0
}

display_configuration() {
    local test_key="$1"
    local test_values="$2"
    local timeout="$3"

    echo ""
    echo "========================== LOCAL TEST CONFIGURATION =========================="
    echo -e "${GREEN}Test Settings:${NC}"
    echo "  Test Key: '$test_key'"
    echo "  Test Values: $test_values"
    echo "  Values Count: $(echo "$test_values" | tr ',' '\n' | wc -l)"
    echo "  Timeout: ${timeout}s"
    echo ""
    echo -e "${BLUE}Test Process:${NC}"
    echo "  1. Start writer node on local port (automatically assigned)"
    echo "  2. Wait for writer to be ready and store records"
    echo "  3. Start reader node to connect to writer"
    echo "  4. Reader retrieves and verifies all records"
    echo "  5. Both nodes shut down automatically"
    echo ""
    echo -e "${CYAN}Network Usage:${NC}"
    echo "  - Uses local loopback interface (127.0.0.1)"
    echo "  - Automatically assigns available ports"
    echo "  - No firewall configuration needed"
    echo "  - No external network connectivity required"
    echo ""
    echo -e "${YELLOW}What This Tests:${NC}"
    echo "  ✓ DHT record storage and retrieval"
    echo "  ✓ Network protocol functionality"
    echo "  ✓ Configuration system"
    echo "  ✓ Cross-node communication"
    echo "  ✓ Data integrity verification"
    echo "========================================================================"
    echo ""
}

run_rust_test() {
    local dry_run="$1"

    if [ "$dry_run" = "true" ]; then
        log_info "Dry run mode - configuration would be:"
        echo "  NETABASE_TEST_KEY=$TEST_KEY"
        echo "  NETABASE_TEST_VALUES=$TEST_VALUES"
        echo "  NETABASE_TEST_TIMEOUT=$TEST_TIMEOUT"
        log_info "Would run: cargo test cross_machine_local_test -- --nocapture --ignored"
        return 0
    fi

    log_info "Starting local cross-machine test..."
    log_info "This will run both writer and reader nodes locally"
    log_info "Compilation and test execution may take a moment..."
    echo ""

    # Export environment variables for the Rust test
    export NETABASE_TEST_KEY="$TEST_KEY"
    export NETABASE_TEST_VALUES="$TEST_VALUES"
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
        log_success "Local test completed successfully!"
        log_info "Both writer and reader nodes functioned correctly"
        log_info "All records were stored and retrieved successfully"
    else
        exit_code=$?
        echo ""
        log_error "Local test failed with exit code $exit_code"

        echo ""
        echo "Common issues and solutions:"
        echo "1. Compilation errors:"
        echo "   → Run 'cargo build' to check for dependency issues"
        echo "   → Ensure you're in the correct directory (netabase project root)"
        echo "2. Port binding issues:"
        echo "   → Close other applications using ports in the 9900-9999 range"
        echo "   → Check with: netstat -ln | grep :99"
        echo "3. Test timeout:"
        echo "   → Increase timeout with --timeout <seconds>"
        echo "   → Check system performance and available resources"
        echo "4. Configuration errors:"
        echo "   → Validate configuration with --validate-only"
        echo "   → Check test values format (comma-separated, no spaces)"
        echo "5. Resource issues:"
        echo "   → Ensure sufficient memory and CPU available"
        echo "   → Close unnecessary applications"

        return $exit_code
    fi
}

cleanup() {
    log_info "Local test interrupted by user"
    exit 1
}

validate_only_mode() {
    log_info "Configuration validation mode"

    # Test configuration parsing
    export NETABASE_TEST_KEY="$TEST_KEY"
    export NETABASE_TEST_VALUES="$TEST_VALUES"
    export NETABASE_TEST_TIMEOUT="$TEST_TIMEOUT"

    # Shell-based validation
    if validate_config "$TEST_KEY" "$TEST_TIMEOUT"; then
        log_success "Configuration is valid"
        display_configuration "$TEST_KEY" "$TEST_VALUES" "$TEST_TIMEOUT"

        echo -e "${GREEN}Ready to run local test with this configuration!${NC}"
        echo ""
        echo "Next steps:"
        echo "1. Run this script without --validate-only to start the test"
        echo "2. The test will run automatically and show results"
        echo "3. No additional setup or configuration required"

        exit 0
    else
        log_error "Configuration validation failed"
        exit 1
    fi
}

# Parse command line arguments
TEST_KEY="$DEFAULT_TEST_KEY"
TEST_VALUES="$DEFAULT_TEST_VALUES"
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
        -v|--values)
            TEST_VALUES="$2"
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
TEST_VALUES="${NETABASE_TEST_VALUES:-$TEST_VALUES}"
TEST_TIMEOUT="${NETABASE_TEST_TIMEOUT:-$TEST_TIMEOUT}"

# Main execution
main() {
    echo ""
    echo "====================================================================="
    echo "                NetaBase Local Cross-Machine Test"
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

    display_configuration "$TEST_KEY" "$TEST_VALUES" "$TEST_TIMEOUT"

    # Set up trap for graceful shutdown
    trap cleanup SIGINT SIGTERM

    log_info "Configuration validated successfully"

    if [ "$VERBOSE" = "true" ]; then
        log_debug "Verbose mode enabled"
        log_debug "Test key: $TEST_KEY"
        log_debug "Test values: $TEST_VALUES"
        log_debug "Timeout: ${TEST_TIMEOUT}s"
    fi

    echo ""

    # Run the actual test
    if ! run_rust_test "$DRY_RUN"; then
        exit 1
    fi

    if [ "$DRY_RUN" != "true" ]; then
        echo ""
        log_success "Local test operation completed!"
        log_info "The cross-machine setup has been validated locally"
        log_info "You can now deploy to multiple machines with confidence"
    fi
}

# Run main function
main "$@"
