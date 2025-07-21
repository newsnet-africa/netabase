#!/bin/bash

# NetaBase Scalability Test Validation Script
# This script validates that the scalability testing environment is properly set up
# and can run basic P2P operations without issues.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

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

log_header() {
    echo ""
    echo -e "${CYAN}=== $1 ===${NC}"
    echo ""
}

check_rust_installation() {
    log_header "Checking Rust Installation"

    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust from https://rustup.rs/"
        return 1
    fi

    local rust_version=$(rustc --version)
    log_success "Rust installed: $rust_version"

    # Check for nightly features
    if rustc --version | grep -q "nightly"; then
        log_success "Using nightly Rust (required for some features)"
    else
        log_warning "Consider using nightly Rust for full feature support"
        log_info "Install with: rustup install nightly && rustup default nightly"
    fi

    return 0
}

check_project_structure() {
    log_header "Checking Project Structure"

    if [ ! -f "Cargo.toml" ]; then
        log_error "Cargo.toml not found. Run this script from the netabase project root."
        return 1
    fi

    if ! grep -q "netabase" Cargo.toml; then
        log_error "This doesn't appear to be the netabase project."
        return 1
    fi

    log_success "Found netabase project structure"

    # Check for scalability test file
    if [ -f "tests/network_scalability.rs" ]; then
        log_success "Scalability test file found"
    else
        log_error "Scalability test file not found at tests/network_scalability.rs"
        return 1
    fi

    # Check for scripts
    if [ -f "scripts/run_scalability_tests.sh" ]; then
        log_success "Scalability test runner script found"
    else
        log_warning "Scalability test runner script not found"
    fi

    return 0
}

check_system_resources() {
    log_header "Checking System Resources"

    local warnings=0

    # Check available memory
    if command -v free &> /dev/null; then
        local available_memory=$(free -m | grep '^Mem:' | awk '{print $7}')
        local total_memory=$(free -m | grep '^Mem:' | awk '{print $2}')

        log_info "Available memory: ${available_memory}MB / ${total_memory}MB"

        if [ "$available_memory" -lt 2048 ]; then
            log_warning "Low available memory. Scalability tests may be limited."
            log_info "Consider closing other applications or reducing test scale."
            warnings=$((warnings + 1))
        else
            log_success "Sufficient memory available for scalability testing"
        fi
    else
        log_warning "Cannot check memory usage on this system"
        warnings=$((warnings + 1))
    fi

    # Check disk space
    if command -v df &> /dev/null; then
        local available_space_kb=$(df . | tail -1 | awk '{print $4}')
        local available_space_mb=$((available_space_kb / 1024))

        log_info "Available disk space: ${available_space_mb}MB"

        if [ "$available_space_mb" -lt 1024 ]; then
            log_warning "Low disk space. Tests may fail due to insufficient storage."
            warnings=$((warnings + 1))
        else
            log_success "Sufficient disk space available"
        fi
    fi

    # Check file descriptor limit
    local max_fds=$(ulimit -n)
    log_info "File descriptor limit: $max_fds"

    if [ "$max_fds" -lt 1024 ]; then
        log_warning "Low file descriptor limit. Consider running: ulimit -n 4096"
        warnings=$((warnings + 1))
    else
        log_success "File descriptor limit is adequate"
    fi

    # Check CPU cores
    if command -v nproc &> /dev/null; then
        local cpu_cores=$(nproc)
        log_info "CPU cores: $cpu_cores"

        if [ "$cpu_cores" -lt 2 ]; then
            log_warning "Few CPU cores available. Tests may run slowly."
            warnings=$((warnings + 1))
        else
            log_success "Multiple CPU cores available for parallel testing"
        fi
    fi

    if [ $warnings -gt 0 ]; then
        log_warning "System resource checks completed with $warnings warnings"
    else
        log_success "System resources look good for scalability testing"
    fi

    return 0
}

check_dependencies() {
    log_header "Checking Dependencies"

    log_info "Checking Rust dependencies..."

    if cargo check --quiet 2>/dev/null; then
        log_success "All dependencies are available and compile correctly"
    else
        log_error "Dependency check failed. Running cargo build to get more details..."
        echo ""
        cargo build
        return 1
    fi

    return 0
}

validate_basic_compilation() {
    log_header "Validating Basic Compilation"

    log_info "Compiling scalability tests..."

    if cargo test --test network_scalability --no-run --quiet 2>/dev/null; then
        log_success "Scalability tests compile successfully"
    else
        log_error "Scalability test compilation failed"
        echo ""
        log_info "Running full compilation to see errors:"
        cargo test --test network_scalability --no-run
        return 1
    fi

    return 0
}

run_minimal_test() {
    log_header "Running Minimal Scalability Test"

    log_info "Running configuration validation test..."

    # Set minimal test environment
    export NETABASE_SCALABILITY_MAX_NODES=3
    export NETABASE_SCALABILITY_TEST_DURATION=10
    export NETABASE_SCALABILITY_VERBOSE=false
    export RUST_LOG=error  # Suppress most logs for cleaner output

    if timeout 60s cargo test test_scalability_config_from_env --test network_scalability -- --nocapture --quiet; then
        log_success "Configuration test passed"
    else
        log_error "Configuration test failed"
        return 1
    fi

    log_info "Testing basic node creation (this may take a moment)..."

    # Create a simple validation test
    cat > ./test_temp/tmp/netabase_validation_test.rs << 'EOF'
use std::time::Duration;
use tokio::time::timeout;
use netabase::network::swarm::generate_swarm;

#[tokio::test]
async fn validate_node_creation() {
    let temp_dir = "./test_tmp/tmp_validation_test";

    // Clean up any existing test directory
    if std::path::Path::new(temp_dir).exists() {
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    // Test node creation
    let result = timeout(Duration::from_secs(10), async {
        generate_swarm(temp_dir)
    }).await;

    match result {
        Ok(Ok(_swarm)) => {
            println!("✓ Node creation successful");
            // Cleanup
            if std::path::Path::new(temp_dir).exists() {
                let _ = std::fs::remove_dir_all(temp_dir);
            }
        },
        Ok(Err(e)) => {
            panic!("Node creation failed: {}", e);
        },
        Err(_) => {
            panic!("Node creation timed out");
        }
    }
}
EOF

    # Copy the validation test to the tests directory
    cp ./test_tmp/tmp/netabase_validation_test.rs tests/validation_test.rs

    if timeout 30s cargo test validate_node_creation --test validation_test -- --nocapture --quiet; then
        log_success "Basic node creation test passed"
    else
        log_error "Basic node creation test failed"
        # Cleanup
        rm -f tests/validation_test.rs
        return 1
    fi

    # Cleanup
    rm -f tests/validation_test.rs
    rm -f ./test_temp/tmp/netabase_validation_test.rs

    return 0
}

check_network_ports() {
    log_header "Checking Network Configuration"

    log_info "Testing UDP port availability..."

    # Test if we can bind to a UDP port
    if command -v nc &> /dev/null; then
        if timeout 2s nc -u -l -p 0 2>/dev/null & then
            local nc_pid=$!
            sleep 1
            kill $nc_pid 2>/dev/null || true
            wait $nc_pid 2>/dev/null || true
            log_success "UDP port binding works"
        else
            log_warning "UDP port binding test inconclusive"
        fi
    else
        log_info "netcat not available, skipping port binding test"
    fi

    # Check if common ports are available
    local test_ports=(9901 8080 7777)
    for port in "${test_ports[@]}"; do
        if command -v netstat &> /dev/null; then
            if netstat -ln 2>/dev/null | grep -q ":$port "; then
                log_warning "Port $port is already in use"
            else
                log_success "Port $port is available"
            fi
        fi
    done

    return 0
}

generate_recommendations() {
    log_header "Performance Recommendations"

    echo "Based on your system, here are some recommendations:"
    echo ""

    # Memory recommendations
    local available_memory=0
    if command -v free &> /dev/null; then
        available_memory=$(free -m | grep '^Mem:' | awk '{print $7}')
    fi

    if [ "$available_memory" -gt 0 ]; then
        local recommended_nodes=$((available_memory / 50))
        if [ "$recommended_nodes" -gt 50 ]; then
            recommended_nodes=50
        fi

        echo "💡 Memory-based recommendations:"
        echo "   - For basic testing: --max-nodes 5-10"
        echo "   - For your system: --max-nodes $recommended_nodes (based on ${available_memory}MB available)"
        echo "   - For stress testing: --max-nodes 20+ (monitor memory usage)"
        echo ""
    fi

    echo "🚀 Getting started:"
    echo "   1. Quick smoke test:"
    echo "      ./scripts/run_scalability_tests.sh --test-type smoke --max-nodes 3 --duration 30"
    echo ""
    echo "   2. Basic scalability test:"
    echo "      ./scripts/run_scalability_tests.sh --test-type linear --max-nodes 10 --duration 120"
    echo ""
    echo "   3. Full test suite (takes longer):"
    echo "      ./scripts/run_scalability_tests.sh --max-nodes 15 --duration 300"
    echo ""

    echo "⚠️  Important notes:"
    echo "   - Tests create temporary directories (cleaned up automatically)"
    echo "   - Each node uses ~50MB RAM and several file descriptors"
    echo "   - Higher node counts require more system resources"
    echo "   - Use Ctrl+C to stop tests early if needed"
    echo ""

    echo "🔧 Troubleshooting:"
    echo "   - If tests fail: Check system resources and reduce --max-nodes"
    echo "   - If compilation fails: Run 'cargo clean && cargo build'"
    echo "   - If ports are in use: Wait a moment and try again"
    echo "   - For help: ./scripts/run_scalability_tests.sh --help"
    echo ""
}

cleanup() {
    log_info "Cleaning up validation artifacts..."
    rm -f ./test_tmp/tests/validation_test.rs 2>/dev/null || true
    rm -rf ./test_tmp/tmp_validation_test 2>/dev/null || true
    rm -f ./test_tmp/tmp/netabase_validation_test.rs 2>/dev/null || true
}

main() {
    echo ""
    echo -e "${CYAN}NetaBase Scalability Test Validation${NC}"
    echo "======================================"
    echo ""
    echo "This script validates your environment for running NetaBase scalability tests."
    echo "It checks system requirements, dependencies, and runs basic functionality tests."
    echo ""

    local checks_passed=0
    local total_checks=0

    # Run all validation checks
    local checks=(
        "check_rust_installation"
        "check_project_structure"
        "check_system_resources"
        "check_dependencies"
        "validate_basic_compilation"
        "check_network_ports"
        "run_minimal_test"
    )

    for check in "${checks[@]}"; do
        total_checks=$((total_checks + 1))
        if $check; then
            checks_passed=$((checks_passed + 1))
        else
            log_error "Check failed: $check"
        fi
    done

    echo ""
    log_header "Validation Summary"

    if [ $checks_passed -eq $total_checks ]; then
        log_success "All validation checks passed! ($checks_passed/$total_checks)"
        log_success "Your system is ready for NetaBase scalability testing."

        generate_recommendations

        echo -e "${GREEN}✓ Validation completed successfully${NC}"
        echo ""
        echo "You can now run scalability tests with confidence!"
    else
        log_warning "Some validation checks had issues ($checks_passed/$total_checks passed)"

        if [ $checks_passed -ge 5 ]; then
            log_info "Basic functionality appears to work, but there may be limitations."
            log_info "Consider addressing the warnings above for optimal performance."

            generate_recommendations
        else
            log_error "Several critical checks failed. Please resolve these issues before running tests."
            echo ""
            echo "Common solutions:"
            echo "1. Ensure Rust is properly installed: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
            echo "2. Run from the correct directory (netabase project root)"
            echo "3. Install dependencies: cargo build"
            echo "4. Check system resources and close other applications"
        fi
    fi

    cleanup

    if [ $checks_passed -eq $total_checks ]; then
        exit 0
    else
        exit 1
    fi
}

# Set up trap for cleanup
trap cleanup EXIT INT TERM

# Run main function
main "$@"
