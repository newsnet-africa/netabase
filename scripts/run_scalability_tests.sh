#!/bin/bash

# NetaBase Scalability Test Runner
# This script runs comprehensive scalability tests for the NetaBase P2P DHT platform
# Updated to support various scalability testing scenarios with configurable parameters

set -e

# Default configuration
DEFAULT_MAX_NODES="20"
DEFAULT_TEST_DURATION="300"
DEFAULT_SPAWN_INTERVAL="2"
DEFAULT_OP_TIMEOUT="30"
DEFAULT_RECORDS_PER_NODE="10"
DEFAULT_RECORD_SIZE="1024"
DEFAULT_TEST_TYPE="all"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

show_usage() {
    echo -e "${CYAN}NetaBase Scalability Test Runner${NC}"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -n, --max-nodes NUM      Maximum number of nodes to test (default: $DEFAULT_MAX_NODES)"
    echo "  -d, --duration SECS      Test duration in seconds (default: $DEFAULT_TEST_DURATION)"
    echo "  -i, --interval SECS      Node spawn interval in seconds (default: $DEFAULT_SPAWN_INTERVAL)"
    echo "  -t, --timeout SECS       Operation timeout in seconds (default: $DEFAULT_OP_TIMEOUT)"
    echo "  -r, --records NUM        Records per node (default: $DEFAULT_RECORDS_PER_NODE)"
    echo "  -s, --record-size BYTES  Record size in bytes (default: $DEFAULT_RECORD_SIZE)"
    echo "  --test-type TYPE         Test type to run (default: $DEFAULT_TEST_TYPE)"
    echo "  --verbose                Enable verbose metrics logging"
    echo "  --no-partitions          Disable network partition tests"
    echo "  --stress                 Enable stress testing mode"
    echo "  --dry-run                Show configuration without running tests"
    echo "  --validate-only          Only validate configuration and exit"
    echo "  -h, --help               Show this help message"
    echo ""
    echo "Test Types:"
    echo "  all                      Run all scalability tests (default)"
    echo "  linear                   Linear node scaling test"
    echo "  topology                 Network topology performance test"
    echo "  throughput               High throughput operations test"
    echo "  churn                    Network churn resilience test"
    echo "  bootstrap                Bootstrap performance test"
    echo "  large-records            Large record scalability test"
    echo "  concurrent               Concurrent operations scaling test"
    echo "  partition                Network partition recovery test"
    echo "  stress                   Maximum scale stress test"
    echo "  smoke                    Quick smoke test for CI/CD"
    echo ""
    echo "Environment Variables:"
    echo "  NETABASE_SCALABILITY_MAX_NODES           Override maximum nodes"
    echo "  NETABASE_SCALABILITY_TEST_DURATION       Override test duration"
    echo "  NETABASE_SCALABILITY_SPAWN_INTERVAL      Override spawn interval"
    echo "  NETABASE_SCALABILITY_OP_TIMEOUT          Override operation timeout"
    echo "  NETABASE_SCALABILITY_RECORDS_PER_NODE    Override records per node"
    echo "  NETABASE_SCALABILITY_RECORD_SIZE         Override record size"
    echo "  NETABASE_SCALABILITY_VERBOSE             Enable verbose output"
    echo "  NETABASE_SCALABILITY_TEST_PARTITIONS     Enable/disable partition tests"
    echo "  NETABASE_STRESS_MAX_NODES                Max nodes for stress test"
    echo "  NETABASE_STRESS_DURATION                 Duration for stress test"
    echo ""
    echo "Examples:"
    echo "  # Run all tests with default settings"
    echo "  $0"
    echo ""
    echo "  # Quick smoke test for development"
    echo "  $0 --test-type smoke --max-nodes 5 --duration 60"
    echo ""
    echo "  # High-scale stress test"
    echo "  $0 --test-type stress --max-nodes 50 --duration 600 --stress"
    echo ""
    echo "  # Test specific topology scenarios"
    echo "  $0 --test-type topology --max-nodes 15 --verbose"
    echo ""
    echo "  # Bootstrap performance analysis"
    echo "  $0 --test-type bootstrap --max-nodes 25 --spawn-interval 1"
    echo ""
    echo "  # Large record handling test"
    echo "  $0 --test-type large-records --record-size 65536 --records 5"
    echo ""
    echo "  # Using environment variables"
    echo "  NETABASE_SCALABILITY_MAX_NODES=30 \\"
    echo "  NETABASE_SCALABILITY_VERBOSE=true \\"
    echo "  $0 --test-type concurrent"
    echo ""
    echo "Resource Requirements:"
    echo "  - Each node requires ~50MB RAM and temporary disk space"
    echo "  - Network ports: Dynamic allocation starting from 1024+"
    echo "  - CPU: Scales linearly with node count"
    echo "  - Recommend: 8GB+ RAM for 20+ nodes, 16GB+ for 50+ nodes"
    echo ""
    echo "Performance Tuning:"
    echo "  - Lower spawn interval = faster test but higher resource usage"
    echo "  - Higher timeout = more resilient but slower failure detection"
    echo "  - More records = better DHT coverage but longer test duration"
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
    log_info "Checking prerequisites for scalability testing..."

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

    # Check system resources
    local available_memory
    if command -v free &> /dev/null; then
        available_memory=$(free -m | grep '^Mem:' | awk '{print $7}')
        local required_memory=$((MAX_NODES * 50))

        if [ "$available_memory" -lt "$required_memory" ]; then
            log_warning "Available memory (${available_memory}MB) may be insufficient for ${MAX_NODES} nodes (~${required_memory}MB required)"
            log_warning "Consider reducing --max-nodes or increasing system memory"
        fi
    fi

    # Check for adequate disk space
    local available_space
    if command -v df &> /dev/null; then
        available_space=$(df . | tail -1 | awk '{print $4}')
        local required_space=$((MAX_NODES * 100)) # KB

        if [ "$available_space" -lt "$required_space" ]; then
            log_warning "Available disk space may be insufficient for temporary node storage"
        fi
    fi

    # Check ulimits for file descriptors (each node opens multiple FDs)
    local max_fds
    max_fds=$(ulimit -n)
    local required_fds=$((MAX_NODES * 50))

    if [ "$max_fds" -lt "$required_fds" ]; then
        log_warning "File descriptor limit ($max_fds) may be too low for $MAX_NODES nodes"
        log_warning "Consider running: ulimit -n $required_fds"
    fi

    log_success "Prerequisites check completed"
}

validate_config() {
    local max_nodes="$1"
    local test_duration="$2"
    local spawn_interval="$3"
    local op_timeout="$4"
    local records_per_node="$5"
    local record_size="$6"
    local test_type="$7"

    log_debug "Validating configuration..."

    # Validate numeric parameters
    if ! [[ "$max_nodes" =~ ^[0-9]+$ ]] || [ "$max_nodes" -le 0 ]; then
        log_error "Invalid max nodes: $max_nodes (must be positive integer)"
        return 1
    fi

    if ! [[ "$test_duration" =~ ^[0-9]+$ ]] || [ "$test_duration" -le 0 ]; then
        log_error "Invalid test duration: $test_duration (must be positive integer)"
        return 1
    fi

    if ! [[ "$spawn_interval" =~ ^[0-9]+$ ]] || [ "$spawn_interval" -lt 0 ]; then
        log_error "Invalid spawn interval: $spawn_interval (must be non-negative integer)"
        return 1
    fi

    if ! [[ "$op_timeout" =~ ^[0-9]+$ ]] || [ "$op_timeout" -le 0 ]; then
        log_error "Invalid operation timeout: $op_timeout (must be positive integer)"
        return 1
    fi

    if ! [[ "$records_per_node" =~ ^[0-9]+$ ]] || [ "$records_per_node" -le 0 ]; then
        log_error "Invalid records per node: $records_per_node (must be positive integer)"
        return 1
    fi

    if ! [[ "$record_size" =~ ^[0-9]+$ ]] || [ "$record_size" -le 0 ]; then
        log_error "Invalid record size: $record_size (must be positive integer)"
        return 1
    fi

    # Validate test type
    local valid_types="all linear topology throughput churn bootstrap large-records concurrent partition stress smoke"
    if ! echo "$valid_types" | grep -wq "$test_type"; then
        log_error "Invalid test type: $test_type"
        log_error "Valid types: $valid_types"
        return 1
    fi

    # Sanity checks
    if [ "$max_nodes" -gt 100 ] && [ "$test_type" != "stress" ]; then
        log_warning "Testing with >100 nodes requires significant system resources"
        log_warning "Consider using --test-type stress for high-scale testing"
    fi

    if [ "$record_size" -gt 1048576 ]; then # 1MB
        log_warning "Large record sizes (>1MB) may impact performance significantly"
    fi

    if [ "$spawn_interval" -eq 0 ] && [ "$max_nodes" -gt 10 ]; then
        log_warning "Zero spawn interval with many nodes may overwhelm the system"
    fi

    local total_data_mb=$(((max_nodes * records_per_node * record_size) / 1048576))
    if [ "$total_data_mb" -gt 1000 ]; then
        log_warning "Total test data size: ${total_data_mb}MB - ensure adequate resources"
    fi

    log_debug "Configuration validation passed"
    return 0
}

get_system_info() {
    echo ""
    echo "=== System Information ==="

    if command -v uname &> /dev/null; then
        echo "OS: $(uname -s) $(uname -r)"
    fi

    if command -v nproc &> /dev/null; then
        echo "CPU Cores: $(nproc)"
    fi

    if command -v free &> /dev/null; then
        echo "Memory: $(free -h | grep '^Mem:' | awk '{print $2 " total, " $7 " available"}')"
    fi

    if command -v df &> /dev/null; then
        echo "Disk Space: $(df -h . | tail -1 | awk '{print $4 " available"}')"
    fi

    echo "File Descriptor Limit: $(ulimit -n)"
    echo "=========================="
    echo ""
}

display_configuration() {
    local max_nodes="$1"
    local test_duration="$2"
    local spawn_interval="$3"
    local op_timeout="$4"
    local records_per_node="$5"
    local record_size="$6"
    local test_type="$7"

    echo ""
    echo "===================== SCALABILITY TEST CONFIGURATION ====================="
    echo -e "${GREEN}Test Parameters:${NC}"
    echo "  Test Type: $test_type"
    echo "  Maximum Nodes: $max_nodes"
    echo "  Test Duration: ${test_duration}s"
    echo "  Node Spawn Interval: ${spawn_interval}s"
    echo "  Operation Timeout: ${op_timeout}s"
    echo ""
    echo -e "${GREEN}Data Parameters:${NC}"
    echo "  Records per Node: $records_per_node"
    echo "  Record Size: $record_size bytes ($(echo "scale=1; $record_size/1024" | bc -l 2>/dev/null || echo "?")KB)"

    local total_records=$((max_nodes * records_per_node))
    local total_data_mb=$(((total_records * record_size) / 1048576))
    echo "  Total Records: $total_records"
    echo "  Total Data Volume: ~${total_data_mb}MB"
    echo ""
    echo -e "${GREEN}Runtime Settings:${NC}"
    echo "  Verbose Metrics: $VERBOSE"
    echo "  Test Partitions: $TEST_PARTITIONS"
    echo "  Stress Mode: $STRESS_MODE"
    echo ""

    get_system_info

    echo -e "${BLUE}Resource Estimates:${NC}"
    local est_memory=$((max_nodes * 50))
    local est_fds=$((max_nodes * 50))
    echo "  Estimated Memory Usage: ~${est_memory}MB"
    echo "  Estimated File Descriptors: ~${est_fds}"

    local est_duration_minutes=$((test_duration / 60))
    echo "  Estimated Test Duration: ~${est_duration_minutes} minutes"
    echo ""

    echo -e "${YELLOW}Performance Tips:${NC}"
    echo "  - Monitor system resources during testing"
    echo "  - Use 'htop' or 'top' to watch CPU/memory usage"
    echo "  - Check 'netstat -an' for network connections"
    echo "  - Logs will show detailed performance metrics"
    echo "========================================================================"
    echo ""
}

run_scalability_test() {
    local test_type="$1"
    local dry_run="$2"

    if [ "$dry_run" = "true" ]; then
        log_info "Dry run mode - would execute:"
        echo "  Test Type: $test_type"
        echo "  Environment Variables:"
        echo "    NETABASE_SCALABILITY_MAX_NODES=$MAX_NODES"
        echo "    NETABASE_SCALABILITY_TEST_DURATION=$TEST_DURATION"
        echo "    NETABASE_SCALABILITY_SPAWN_INTERVAL=$SPAWN_INTERVAL"
        echo "    NETABASE_SCALABILITY_OP_TIMEOUT=$OP_TIMEOUT"
        echo "    NETABASE_SCALABILITY_RECORDS_PER_NODE=$RECORDS_PER_NODE"
        echo "    NETABASE_SCALABILITY_RECORD_SIZE=$RECORD_SIZE"
        echo "    NETABASE_SCALABILITY_VERBOSE=$VERBOSE"
        echo "    NETABASE_SCALABILITY_TEST_PARTITIONS=$TEST_PARTITIONS"
        if [ "$STRESS_MODE" = "true" ]; then
            echo "    NETABASE_STRESS_MAX_NODES=$((MAX_NODES * 2))"
            echo "    NETABASE_STRESS_DURATION=$((TEST_DURATION * 2))"
        fi
        echo ""
        echo "  Cargo command: cargo test test_${test_type//-/_} -- --nocapture --ignored"
        return 0
    fi

    # Export environment variables for Rust tests
    export NETABASE_SCALABILITY_MAX_NODES="$MAX_NODES"
    export NETABASE_SCALABILITY_TEST_DURATION="$TEST_DURATION"
    export NETABASE_SCALABILITY_SPAWN_INTERVAL="$SPAWN_INTERVAL"
    export NETABASE_SCALABILITY_OP_TIMEOUT="$OP_TIMEOUT"
    export NETABASE_SCALABILITY_RECORDS_PER_NODE="$RECORDS_PER_NODE"
    export NETABASE_SCALABILITY_RECORD_SIZE="$RECORD_SIZE"
    export NETABASE_SCALABILITY_VERBOSE="$VERBOSE"
    export NETABASE_SCALABILITY_TEST_PARTITIONS="$TEST_PARTITIONS"

    if [ "$STRESS_MODE" = "true" ]; then
        export NETABASE_STRESS_MAX_NODES="$((MAX_NODES * 2))"
        export NETABASE_STRESS_DURATION="$((TEST_DURATION * 2))"
    fi

    # Set Rust log level
    if [ "$VERBOSE" = "true" ]; then
        export RUST_LOG="debug"
    else
        export RUST_LOG="info"
    fi

    # Map test types to actual test function names
    local rust_test_name
    case "$test_type" in
        "linear")
            rust_test_name="test_linear_node_scaling"
            ;;
        "topology")
            rust_test_name="test_network_topology_performance"
            ;;
        "throughput")
            rust_test_name="test_high_throughput_operations"
            ;;
        "churn")
            rust_test_name="test_network_churn_resilience"
            ;;
        "bootstrap")
            rust_test_name="test_bootstrap_performance"
            ;;
        "large-records")
            rust_test_name="test_large_record_scalability"
            ;;
        "concurrent")
            rust_test_name="test_concurrent_operations_scaling"
            ;;
        "partition")
            rust_test_name="test_network_partition_recovery"
            ;;
        "stress")
            rust_test_name="test_stress_test_maximum_scale"
            ;;
        "smoke")
            rust_test_name="quick_scalability_smoke_test"
            ;;
        *)
            log_error "Unknown test type: $test_type"
            return 1
            ;;
    esac

    log_info "Starting scalability test: $test_type"
    log_info "This may take several minutes to compile and run..."
    log_info "Monitor system resources and check logs for detailed progress"
    echo ""

    local start_time=$(date +%s)
    local exit_code=0

    # Run the specific test
    if cargo test "$rust_test_name" -- --nocapture --ignored; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo ""
        log_success "Scalability test '$test_type' completed successfully!"
        log_info "Test duration: ${duration} seconds"
    else
        exit_code=$?
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo ""
        log_error "Scalability test '$test_type' failed with exit code $exit_code"
        log_info "Test duration: ${duration} seconds"

        echo ""
        echo "Common issues and solutions:"
        echo "1. Out of memory:"
        echo "   → Reduce --max-nodes or add more system RAM"
        echo "2. File descriptor limit:"
        echo "   → Run: ulimit -n 4096"
        echo "3. Port exhaustion:"
        echo "   → Reduce concurrent nodes or wait between tests"
        echo "4. Compilation errors:"
        echo "   → Run: cargo clean && cargo build"
        echo "5. Test timeout:"
        echo "   → Increase --duration or reduce complexity"
        echo "6. Network issues:"
        echo "   → Check firewall settings and available ports"

        return $exit_code
    fi
}

run_all_tests() {
    local dry_run="$1"

    if [ "$dry_run" = "true" ]; then
        log_info "Dry run mode - would run all scalability tests in sequence"
        return 0
    fi

    log_info "Running comprehensive scalability test suite"
    log_info "This will take a significant amount of time and system resources"
    echo ""

    local tests=("smoke" "linear" "topology" "bootstrap" "throughput" "concurrent")

    # Add optional tests based on configuration
    if [ "$TEST_PARTITIONS" = "true" ]; then
        tests+=("partition")
    fi

    tests+=("churn" "large-records")

    if [ "$STRESS_MODE" = "true" ]; then
        tests+=("stress")
    fi

    local failed_tests=()
    local passed_tests=()

    for test in "${tests[@]}"; do
        log_info "=== Running test: $test ==="

        if run_scalability_test "$test" false; then
            passed_tests+=("$test")
            log_success "Test '$test' passed"
        else
            failed_tests+=("$test")
            log_error "Test '$test' failed"
        fi

        # Brief pause between tests to let system recover
        sleep 5
        echo ""
    done

    # Summary
    echo ""
    echo "=== TEST SUITE SUMMARY ==="
    echo "Passed tests: ${#passed_tests[@]}"
    for test in "${passed_tests[@]}"; do
        echo "  ✓ $test"
    done

    echo ""
    echo "Failed tests: ${#failed_tests[@]}"
    for test in "${failed_tests[@]}"; do
        echo "  ✗ $test"
    done
    echo "=========================="

    if [ ${#failed_tests[@]} -gt 0 ]; then
        return 1
    fi
}

cleanup() {
    log_info "Cleaning up scalability test resources..."

    # Kill any remaining test processes
    pkill -f "netabase.*scalability" 2>/dev/null || true

    # Clean up temporary directories
    find . -name "tmp*_scalability_*" -type d -exec rm -rf {} + 2>/dev/null || true

    log_info "Cleanup completed"
    exit 0
}

validate_only_mode() {
    log_info "Configuration validation mode"

    if validate_config "$MAX_NODES" "$TEST_DURATION" "$SPAWN_INTERVAL" "$OP_TIMEOUT" "$RECORDS_PER_NODE" "$RECORD_SIZE" "$TEST_TYPE"; then
        log_success "Configuration is valid"
        display_configuration "$MAX_NODES" "$TEST_DURATION" "$SPAWN_INTERVAL" "$OP_TIMEOUT" "$RECORDS_PER_NODE" "$RECORD_SIZE" "$TEST_TYPE"

        echo -e "${GREEN}Ready to run scalability tests with this configuration!${NC}"
        echo ""
        echo "Next steps:"
        echo "1. Run this script without --validate-only to start testing"
        echo "2. Monitor system resources during testing"
        echo "3. Check logs for detailed performance metrics"
        echo "4. Consider running individual tests first before 'all'"

        exit 0
    else
        log_error "Configuration validation failed"
        exit 1
    fi
}

# Parse command line arguments
MAX_NODES="$DEFAULT_MAX_NODES"
TEST_DURATION="$DEFAULT_TEST_DURATION"
SPAWN_INTERVAL="$DEFAULT_SPAWN_INTERVAL"
OP_TIMEOUT="$DEFAULT_OP_TIMEOUT"
RECORDS_PER_NODE="$DEFAULT_RECORDS_PER_NODE"
RECORD_SIZE="$DEFAULT_RECORD_SIZE"
TEST_TYPE="$DEFAULT_TEST_TYPE"
VERBOSE=false
TEST_PARTITIONS=true
STRESS_MODE=false
DRY_RUN=false
VALIDATE_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--max-nodes)
            MAX_NODES="$2"
            shift 2
            ;;
        -d|--duration)
            TEST_DURATION="$2"
            shift 2
            ;;
        -i|--interval)
            SPAWN_INTERVAL="$2"
            shift 2
            ;;
        -t|--timeout)
            OP_TIMEOUT="$2"
            shift 2
            ;;
        -r|--records)
            RECORDS_PER_NODE="$2"
            shift 2
            ;;
        -s|--record-size)
            RECORD_SIZE="$2"
            shift 2
            ;;
        --test-type)
            TEST_TYPE="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --no-partitions)
            TEST_PARTITIONS=false
            shift
            ;;
        --stress)
            STRESS_MODE=true
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

# Override with environment variables if set (CLI args have higher priority)
MAX_NODES="${NETABASE_SCALABILITY_MAX_NODES:-$MAX_NODES}"
TEST_DURATION="${NETABASE_SCALABILITY_TEST_DURATION:-$TEST_DURATION}"
SPAWN_INTERVAL="${NETABASE_SCALABILITY_SPAWN_INTERVAL:-$SPAWN_INTERVAL}"
OP_TIMEOUT="${NETABASE_SCALABILITY_OP_TIMEOUT:-$OP_TIMEOUT}"
RECORDS_PER_NODE="${NETABASE_SCALABILITY_RECORDS_PER_NODE:-$RECORDS_PER_NODE}"
RECORD_SIZE="${NETABASE_SCALABILITY_RECORD_SIZE:-$RECORD_SIZE}"

if [ "${NETABASE_SCALABILITY_VERBOSE:-false}" = "true" ]; then
    VERBOSE=true
fi

if [ "${NETABASE_SCALABILITY_TEST_PARTITIONS:-true}" = "false" ]; then
    TEST_PARTITIONS=false
fi

# Main execution
main() {
    echo ""
    echo "====================================================================="
    echo "                NetaBase P2P Scalability Test Suite"
    echo "====================================================================="

    # Handle special modes first
    if [ "$VALIDATE_ONLY" = "true" ]; then
        validate_only_mode
        return
    fi

    check_prerequisites

    if ! validate_config "$MAX_NODES" "$TEST_DURATION" "$SPAWN_INTERVAL" "$OP_TIMEOUT" "$RECORDS_PER_NODE" "$RECORD_SIZE" "$TEST_TYPE"; then
        log_error "Configuration validation failed"
        exit 1
    fi

    display_configuration "$MAX_NODES" "$TEST_DURATION" "$SPAWN_INTERVAL" "$OP_TIMEOUT" "$RECORDS_PER_NODE" "$RECORD_SIZE" "$TEST_TYPE"

    # Set up trap for graceful shutdown
    trap cleanup SIGINT SIGTERM

    log_info "Configuration validated successfully"

    if [ "$VERBOSE" = "true" ]; then
        log_debug "Verbose mode enabled - detailed metrics will be collected"
        log_debug "Test configuration: $MAX_NODES nodes, ${TEST_DURATION}s duration, $TEST_TYPE test"
    fi

    echo ""

    # Run the tests
    if [ "$TEST_TYPE" = "all" ]; then
        if ! run_all_tests "$DRY_RUN"; then
            exit 1
        fi
    else
        if ! run_scalability_test "$TEST_TYPE" "$DRY_RUN"; then
            exit 1
        fi
    fi

    if [ "$DRY_RUN" != "true" ]; then
        echo ""
        log_success "Scalability testing completed!"
        log_info "Check the logs above for detailed performance analysis"
        log_info "System resource usage and performance metrics have been reported"
    fi
}

# Run main function
main "$@"
