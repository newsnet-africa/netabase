#!/bin/bash

# NetaBase Record Debugging Script
# This script helps debug why records aren't being found between writer and reader nodes

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default configuration
DEFAULT_CONNECT_ADDR="192.168.24.160:9900"
DEFAULT_CONNECT_HOST="192.168.24.160"
DEFAULT_CONNECT_PORT="9900"
DEFAULT_TEST_KEY="multi_record_test"
DEFAULT_TIMEOUT="60"

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
    echo -e "${CYAN}NetaBase Record Debugging Script${NC}"
    echo ""
    echo "This script helps debug record retrieval issues between writer and reader nodes."
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -c, --connect ADDR   Writer address to connect to (default: $DEFAULT_CONNECT_ADDR)"
    echo "  -H, --host HOST      Writer host/IP address (default: $DEFAULT_CONNECT_HOST)"
    echo "  -p, --port PORT      Writer port number (default: $DEFAULT_CONNECT_PORT)"
    echo "  -k, --key KEY        Base test key for records (default: $DEFAULT_TEST_KEY)"
    echo "  -t, --timeout SECS   Timeout in seconds (default: $DEFAULT_TIMEOUT)"
    echo "  --test-single        Test single record retrieval"
    echo "  --test-multi         Test multi-record retrieval"
    echo "  --test-connection    Test basic connectivity"
    echo "  --verbose            Enable verbose logging"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  # Test basic connectivity"
    echo "  $0 --test-connection"
    echo ""
    echo "  # Test single record retrieval"
    echo "  $0 --test-single -c 192.168.24.160:9900 -k multi_record_test"
    echo ""
    echo "  # Use separate host and port specification"
    echo "  $0 --test-single --host 192.168.24.160 --port 9900 -k multi_record_test"
    echo ""
    echo "  # Test all 5 records with verbose output"
    echo "  $0 --test-multi --verbose"
}

test_basic_connectivity() {
    local connect_addr="$1"
    local host="${connect_addr%:*}"
    local port="${connect_addr##*:}"

    log_info "Testing basic connectivity to $connect_addr..."

    # Test ping
    if command -v ping &> /dev/null; then
        log_info "Testing host reachability..."
        if timeout 3 ping -c 1 "$host" &> /dev/null; then
            log_success "Host $host is reachable via ping"
        else
            log_warning "Host $host is not responding to ping (may be firewalled)"
        fi
    fi

    # Test UDP port connectivity
    if command -v nc &> /dev/null; then
        log_info "Testing UDP port connectivity..."
        if nc -z -u -w 3 "$host" "$port" &> /dev/null; then
            log_success "UDP port $port appears to be open"
        else
            log_warning "UDP port $port may not be accessible"
        fi
    fi

    # Test if we can reach the QUIC endpoint
    log_info "Testing QUIC connectivity would require libp2p client..."
}

test_single_record() {
    local connect_addr="$1"
    local test_key="$2"
    local timeout="$3"
    local verbose="$4"

    log_info "Testing single record retrieval..."
    log_info "Connect address: $connect_addr"
    log_info "Test key: $test_key"
    log_info "Looking for key: ${test_key}__0 (expecting: 'Hello World')"

    export NETABASE_READER_CONNECT_ADDR="$connect_addr"
    export NETABASE_TEST_KEY="$test_key"
    export NETABASE_TEST_TIMEOUT="$timeout"
    export NETABASE_TEST_VALUES="Hello World"

    if [ "$verbose" = "true" ]; then
        export RUST_LOG="debug"
    else
        export RUST_LOG="info"
    fi

    log_info "Running single record test..."
    if cargo test cross_machine_reader -- --nocapture --ignored; then
        log_success "Single record test passed!"
    else
        log_error "Single record test failed!"
        return 1
    fi
}

test_multi_record() {
    local connect_addr="$1"
    local test_key="$2"
    local timeout="$3"
    local verbose="$4"

    log_info "Testing multi-record retrieval..."
    log_info "Connect address: $connect_addr"
    log_info "Test key: $test_key"
    log_info "Looking for 5 records: ${test_key}__0 through ${test_key}__4"

    export NETABASE_READER_CONNECT_ADDR="$connect_addr"
    export NETABASE_TEST_KEY="$test_key"
    export NETABASE_TEST_TIMEOUT="$timeout"

    if [ "$verbose" = "true" ]; then
        export RUST_LOG="debug"
    else
        export RUST_LOG="info"
    fi

    log_info "Running 5-record test..."
    if cargo test cross_machine_reader_5_records -- --nocapture --ignored; then
        log_success "Multi-record test passed!"
    else
        log_error "Multi-record test failed!"
        return 1
    fi
}

debug_network_state() {
    local connect_addr="$1"

    log_info "Debugging network state..."

    echo ""
    echo "========================== NETWORK DEBUG INFO =========================="
    echo "1. Network interfaces:"
    ip addr show 2>/dev/null || ifconfig 2>/dev/null || echo "Cannot get network info"

    echo ""
    echo "2. Routing table:"
    ip route show 2>/dev/null || route -n 2>/dev/null || echo "Cannot get routing info"

    echo ""
    echo "3. Active connections:"
    ss -ulpn 2>/dev/null | grep -E ":9900|:9901" || netstat -ulpn 2>/dev/null | grep -E ":9900|:9901" || echo "No active connections on ports 9900/9901"

    echo ""
    echo "4. Testing writer connectivity:"
    test_basic_connectivity "$connect_addr"

    echo "======================================================================"
    echo ""
}

manual_record_check() {
    local connect_addr="$1"
    local test_key="$2"
    local verbose="$3"

    log_info "Manual record existence check..."

    cat > ./test_tmp/tmp/debug_record_check.rs << 'EOF'
use netabase::network::swarm::generate_swarm;
use libp2p::{kad::RecordKey, Multiaddr};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let connect_addr = std::env::var("CONNECT_ADDR")?;
    let test_key = std::env::var("TEST_KEY")?;

    println!("Checking for records under key: {}", test_key);

    let temp_dir = "./test_tmp/tmp_debug_reader";
    let mut swarm = generate_swarm(temp_dir)?;

    let dial_addr: Multiaddr = format!("/ip4/{}/udp/{}/quic-v1",
        connect_addr.split(':').next().unwrap(),
        connect_addr.split(':').nth(1).unwrap())
        .parse()?;

    println!("Dialing: {}", dial_addr);
    swarm.dial(dial_addr)?;

    // Wait for connection
    loop {
        let event = swarm.select_next_some().await;
        if let libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } = event {
            println!("Connected to: {}", peer_id);
            break;
        }
    }

    // Check each record
    for i in 0..5 {
        let key = RecordKey::new(&format!("{}__{}", test_key, i));
        println!("Checking key: {:?}", std::str::from_utf8(key.as_ref()));

        let query_id = swarm.behaviour_mut().kad.get_record(key);
        println!("Query ID: {:?}", query_id);

        // Wait for result
        let result = timeout(Duration::from_secs(10), async {
            loop {
                let event = swarm.select_next_some().await;
                // Process events...
                break;
            }
        }).await;

        if result.is_err() {
            println!("Timeout for record {}", i);
        }
    }

    Ok(())
}
EOF

    log_info "Compiling manual check tool..."
    # This would need to be compiled and run separately
    log_warning "Manual check requires separate compilation - skipping for now"
}

# Parse command line arguments
CONNECT_ADDR="$DEFAULT_CONNECT_ADDR"
CONNECT_HOST="$DEFAULT_CONNECT_HOST"
CONNECT_PORT="$DEFAULT_CONNECT_PORT"
TEST_KEY="$DEFAULT_TEST_KEY"
TIMEOUT="$DEFAULT_TIMEOUT"
VERBOSE=false
TEST_CONNECTION=false
TEST_SINGLE=false
TEST_MULTI=false
CUSTOM_HOST_PORT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--connect)
            CONNECT_ADDR="$2"
            shift 2
            ;;
        -H|--host)
            CONNECT_HOST="$2"
            CUSTOM_HOST_PORT=true
            shift 2
            ;;
        -p|--port)
            CONNECT_PORT="$2"
            CUSTOM_HOST_PORT=true
            shift 2
            ;;
        -k|--key)
            TEST_KEY="$2"
            shift 2
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --test-connection)
            TEST_CONNECTION=true
            shift
            ;;
        --test-single)
            TEST_SINGLE=true
            shift
            ;;
        --test-multi)
            TEST_MULTI=true
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
    echo "                NetaBase Record Debugging Tool"
    echo "====================================================================="
    echo ""

    if [ ! -f "Cargo.toml" ]; then
        log_error "Please run this script from the netabase project root"
        exit 1
    fi

    # If custom host/port specified, build the connect address
    if [ "$CUSTOM_HOST_PORT" = "true" ]; then
        CONNECT_ADDR="${CONNECT_HOST}:${CONNECT_PORT}"
    fi

    log_info "Configuration:"
    echo "  Connect Address: $CONNECT_ADDR"
    echo "  Connect Host: ${CONNECT_ADDR%:*}"
    echo "  Connect Port: ${CONNECT_ADDR##*:}"
    echo "  Test Key: $TEST_KEY"
    echo "  Timeout: ${TIMEOUT}s"
    echo "  Verbose: $VERBOSE"
    echo ""

    # Run requested tests
    local any_test_run=false

    if [ "$TEST_CONNECTION" = "true" ]; then
        any_test_run=true
        log_info "=== CONNECTIVITY TEST ==="
        debug_network_state "$CONNECT_ADDR"
    fi

    if [ "$TEST_SINGLE" = "true" ]; then
        any_test_run=true
        echo ""
        log_info "=== SINGLE RECORD TEST ==="
        if test_single_record "$CONNECT_ADDR" "$TEST_KEY" "$TIMEOUT" "$VERBOSE"; then
            log_success "Single record test completed successfully"
        else
            log_error "Single record test failed"
        fi
    fi

    if [ "$TEST_MULTI" = "true" ]; then
        any_test_run=true
        echo ""
        log_info "=== MULTI RECORD TEST ==="
        if test_multi_record "$CONNECT_ADDR" "$TEST_KEY" "$TIMEOUT" "$VERBOSE"; then
            log_success "Multi-record test completed successfully"
        else
            log_error "Multi-record test failed"
        fi
    fi

    if [ "$any_test_run" = "false" ]; then
        log_warning "No tests specified. Use --help for options."
        echo ""
        log_info "Quick diagnostic run:"
        debug_network_state "$CONNECT_ADDR"

        echo ""
        log_info "Suggested next steps:"
        echo "1. Test basic connectivity: $0 --test-connection"
        echo "2. Test single record: $0 --test-single --verbose"
        echo "3. Test all records: $0 --test-multi --verbose"
        echo ""
        echo "Common issues:"
        echo "- Writer not running or not fully started"
        echo "- Network connectivity problems"
        echo "- Records not yet propagated in DHT"
        echo "- Firewall blocking UDP traffic"
        echo "- Writer and reader using different keys"
    fi

    echo ""
    log_info "Debug session completed"
}

# Run main function
main "$@"
