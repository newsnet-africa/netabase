#!/bin/bash

# NetaBase Port Configuration and Testing Utility
# This script helps test and diagnose port-related issues for NetaBase networking

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default configuration
DEFAULT_TEST_HOST="127.0.0.1"
DEFAULT_TEST_PORTS="9900,9901,9902"
DEFAULT_REMOTE_HOST=""
DEFAULT_TIMEOUT="5"

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

show_usage() {
    echo -e "${CYAN}NetaBase Port Configuration and Testing Utility${NC}"
    echo ""
    echo "This script helps test and diagnose port-related issues for NetaBase networking."
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -H, --host HOST          Host to test (default: $DEFAULT_TEST_HOST)"
    echo "  -p, --ports PORTS        Comma-separated list of ports (default: $DEFAULT_TEST_PORTS)"
    echo "  -r, --remote HOST        Remote host to test connectivity to"
    echo "  -t, --timeout SECS       Connection timeout (default: $DEFAULT_TIMEOUT)"
    echo "  --test-local             Test local port availability"
    echo "  --test-remote            Test remote connectivity"
    echo "  --test-udp               Test UDP connectivity (default for NetaBase)"
    echo "  --test-tcp               Test TCP connectivity"
    echo "  --test-both              Test both UDP and TCP"
    echo "  --scan-range START:END   Scan port range (e.g., 9900:9910)"
    echo "  --netabase-defaults      Use NetaBase default ports and settings"
    echo "  --show-firewall          Show firewall configuration suggestions"
    echo "  --verbose                Enable verbose output"
    echo "  -h, --help               Show this help message"
    echo ""
    echo "Examples:"
    echo "  # Test default NetaBase ports locally"
    echo "  $0 --test-local --netabase-defaults"
    echo ""
    echo "  # Test connectivity to remote NetaBase writer"
    echo "  $0 --test-remote -r 192.168.1.100 -p 9900 --test-udp"
    echo ""
    echo "  # Scan for available ports in NetaBase range"
    echo "  $0 --scan-range 9900:9910 --test-local"
    echo ""
    echo "  # Test both UDP and TCP on multiple ports"
    echo "  $0 --test-both -H 0.0.0.0 -p 9900,9901,9902"
    echo ""
    echo "  # Test remote connectivity with custom timeout"
    echo "  $0 --test-remote -r 192.168.24.160 -p 9900 -t 10"
    echo ""
    echo "NetaBase Port Information:"
    echo "  - Writer nodes typically listen on ports 9900-9902"
    echo "  - Reader nodes connect to writer ports"
    echo "  - UDP is the primary protocol (QUIC over UDP)"
    echo "  - mDNS uses port 5353 (multicast)"
    echo "  - Firewall must allow UDP traffic on chosen ports"
}

get_system_info() {
    echo ""
    echo "========================== SYSTEM INFORMATION =========================="
    echo "Hostname: $(hostname)"
    echo "Operating System: $(uname -s)"

    if command -v ip &> /dev/null; then
        echo "Network Interfaces:"
        ip addr show | grep -E "^[0-9]+:|inet " | while read line; do
            echo "  $line"
        done
    elif command -v ifconfig &> /dev/null; then
        echo "Network Interfaces:"
        ifconfig | grep -E "^[a-z]|inet " | while read line; do
            echo "  $line"
        done
    fi

    echo "======================================================================"
}

test_port_availability() {
    local host="$1"
    local port="$2"
    local protocol="$3"

    log_debug "Testing port availability: $protocol://$host:$port"

    if [ "$protocol" = "udp" ]; then
        # For UDP, we try to bind to see if port is available
        if command -v nc &> /dev/null; then
            if nc -u -l -p "$port" -s "$host" 2>/dev/null &
            then
                local nc_pid=$!
                sleep 0.1
                kill "$nc_pid" 2>/dev/null || true
                wait "$nc_pid" 2>/dev/null || true
                log_success "UDP port $port on $host is available"
                return 0
            else
                log_warning "UDP port $port on $host appears to be in use"
                return 1
            fi
        else
            log_warning "netcat not available - cannot test UDP port availability"
            return 2
        fi
    else
        # For TCP, we can use various methods
        if command -v nc &> /dev/null; then
            if nc -l -p "$port" -s "$host" 2>/dev/null &
            then
                local nc_pid=$!
                sleep 0.1
                kill "$nc_pid" 2>/dev/null || true
                wait "$nc_pid" 2>/dev/null || true
                log_success "TCP port $port on $host is available"
                return 0
            else
                log_warning "TCP port $port on $host appears to be in use"
                return 1
            fi
        elif command -v telnet &> /dev/null; then
            if timeout 1 bash -c "echo >/dev/tcp/$host/$port" 2>/dev/null; then
                log_warning "TCP port $port on $host appears to be in use"
                return 1
            else
                log_success "TCP port $port on $host is available"
                return 0
            fi
        else
            log_warning "No tools available to test TCP port availability"
            return 2
        fi
    fi
}

test_remote_connectivity() {
    local host="$1"
    local port="$2"
    local protocol="$3"
    local timeout="$4"

    log_debug "Testing remote connectivity: $protocol://$host:$port (timeout: ${timeout}s)"

    if [ "$protocol" = "udp" ]; then
        if command -v nc &> /dev/null; then
            # Test UDP connectivity by trying to send data
            if echo "test" | timeout "$timeout" nc -u -w1 "$host" "$port" 2>/dev/null; then
                log_success "UDP connection to $host:$port successful"
                return 0
            else
                log_error "UDP connection to $host:$port failed"
                return 1
            fi
        else
            log_warning "netcat not available - cannot test UDP connectivity"
            return 2
        fi
    else
        # Test TCP connectivity
        if command -v nc &> /dev/null; then
            if timeout "$timeout" nc -z "$host" "$port" 2>/dev/null; then
                log_success "TCP connection to $host:$port successful"
                return 0
            else
                log_error "TCP connection to $host:$port failed"
                return 1
            fi
        elif command -v telnet &> /dev/null; then
            if timeout "$timeout" bash -c "echo >/dev/tcp/$host/$port" 2>/dev/null; then
                log_success "TCP connection to $host:$port successful"
                return 0
            else
                log_error "TCP connection to $host:$port failed"
                return 1
            fi
        else
            log_warning "No tools available to test TCP connectivity"
            return 2
        fi
    fi
}

test_ping_connectivity() {
    local host="$1"
    local timeout="$2"

    log_info "Testing basic network connectivity to $host..."

    if command -v ping &> /dev/null; then
        if timeout "$timeout" ping -c 1 "$host" &>/dev/null; then
            log_success "Host $host is reachable via ping"
            return 0
        else
            log_warning "Host $host is not responding to ping"
            return 1
        fi
    else
        log_warning "ping command not available"
        return 2
    fi
}

scan_port_range() {
    local host="$1"
    local start_port="$2"
    local end_port="$3"
    local protocol="$4"
    local test_type="$5"

    log_info "Scanning ports $start_port-$end_port on $host ($protocol, $test_type)"

    local available_ports=()
    local used_ports=()
    local unreachable_ports=()

    for port in $(seq "$start_port" "$end_port"); do
        echo -n "."
        if [ "$test_type" = "local" ]; then
            if test_port_availability "$host" "$port" "$protocol" >/dev/null 2>&1; then
                available_ports+=("$port")
            else
                used_ports+=("$port")
            fi
        else
            if test_remote_connectivity "$host" "$port" "$protocol" 1 >/dev/null 2>&1; then
                available_ports+=("$port")
            else
                unreachable_ports+=("$port")
            fi
        fi
    done
    echo ""

    if [ ${#available_ports[@]} -gt 0 ]; then
        if [ "$test_type" = "local" ]; then
            log_success "Available ports: ${available_ports[*]}"
        else
            log_success "Reachable ports: ${available_ports[*]}"
        fi
    fi

    if [ ${#used_ports[@]} -gt 0 ]; then
        log_warning "Ports in use: ${used_ports[*]}"
    fi

    if [ ${#unreachable_ports[@]} -gt 0 ]; then
        log_error "Unreachable ports: ${unreachable_ports[*]}"
    fi
}

show_firewall_help() {
    local ports="$1"

    echo ""
    echo "========================== FIREWALL CONFIGURATION =========================="
    echo ""
    log_info "NetaBase requires UDP traffic on the following ports: $ports"
    echo ""
    echo "Ubuntu/Debian (ufw):"
    for port in ${ports//,/ }; do
        echo "  sudo ufw allow $port/udp"
    done
    echo ""
    echo "CentOS/RHEL/Fedora (firewalld):"
    for port in ${ports//,/ }; do
        echo "  sudo firewall-cmd --add-port=$port/udp --permanent"
    done
    echo "  sudo firewall-cmd --reload"
    echo ""
    echo "Windows (netsh):"
    for port in ${ports//,/ }; do
        echo "  netsh advfirewall firewall add rule name=\"NetaBase-$port\" protocol=UDP dir=in localport=$port action=allow"
    done
    echo ""
    echo "macOS (pfctl) - add to /etc/pf.conf:"
    for port in ${ports//,/ }; do
        echo "  pass in proto udp from any to any port $port"
    done
    echo ""
    echo "Docker containers:"
    for port in ${ports//,/ }; do
        echo "  docker run -p $port:$port/udp ..."
    done
    echo ""
    echo "mDNS (if using automatic discovery):"
    echo "  sudo ufw allow 5353/udp          # Ubuntu/Debian"
    echo "  sudo firewall-cmd --add-port=5353/udp --permanent  # CentOS/RHEL"
    echo ""
    echo "Testing firewall rules:"
    for port in ${ports//,/ }; do
        echo "  # From remote machine:"
        echo "  nc -u -v <target-ip> $port"
    done
    echo "======================================================================"
}

check_netabase_processes() {
    log_info "Checking for running NetaBase processes..."

    if command -v ps &> /dev/null; then
        local netabase_processes
        netabase_processes=$(ps aux | grep -E "(netabase|cargo test)" | grep -v grep || true)

        if [ -n "$netabase_processes" ]; then
            log_warning "Found running NetaBase-related processes:"
            echo "$netabase_processes" | while read line; do
                echo "  $line"
            done
        else
            log_info "No NetaBase processes currently running"
        fi
    fi

    if command -v lsof &> /dev/null; then
        log_info "Checking port usage (lsof)..."
        local port_usage
        port_usage=$(lsof -i :9900-9902 2>/dev/null || true)

        if [ -n "$port_usage" ]; then
            log_warning "Ports 9900-9902 usage:"
            echo "$port_usage"
        else
            log_info "NetaBase ports (9900-9902) are not in use"
        fi
    elif command -v netstat &> /dev/null; then
        log_info "Checking port usage (netstat)..."
        local port_usage
        port_usage=$(netstat -ulnp 2>/dev/null | grep -E ":990[0-2]" || true)

        if [ -n "$port_usage" ]; then
            log_warning "NetaBase ports usage:"
            echo "$port_usage"
        else
            log_info "NetaBase ports (9900-9902) are not in use"
        fi
    fi
}

# Parse command line arguments
TEST_HOST="$DEFAULT_TEST_HOST"
TEST_PORTS="$DEFAULT_TEST_PORTS"
REMOTE_HOST="$DEFAULT_REMOTE_HOST"
TIMEOUT="$DEFAULT_TIMEOUT"
TEST_LOCAL=false
TEST_REMOTE=false
TEST_UDP=false
TEST_TCP=false
SCAN_RANGE=""
NETABASE_DEFAULTS=false
SHOW_FIREWALL=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -H|--host)
            TEST_HOST="$2"
            shift 2
            ;;
        -p|--ports)
            TEST_PORTS="$2"
            shift 2
            ;;
        -r|--remote)
            REMOTE_HOST="$2"
            shift 2
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --test-local)
            TEST_LOCAL=true
            shift
            ;;
        --test-remote)
            TEST_REMOTE=true
            shift
            ;;
        --test-udp)
            TEST_UDP=true
            shift
            ;;
        --test-tcp)
            TEST_TCP=true
            shift
            ;;
        --test-both)
            TEST_UDP=true
            TEST_TCP=true
            shift
            ;;
        --scan-range)
            SCAN_RANGE="$2"
            shift 2
            ;;
        --netabase-defaults)
            NETABASE_DEFAULTS=true
            shift
            ;;
        --show-firewall)
            SHOW_FIREWALL=true
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

# Apply NetaBase defaults if requested
if [ "$NETABASE_DEFAULTS" = "true" ]; then
    TEST_PORTS="9900,9901,9902"
    TEST_UDP=true
    if [ "$TEST_LOCAL" = "false" ] && [ "$TEST_REMOTE" = "false" ]; then
        TEST_LOCAL=true
    fi
fi

# Default to UDP if no protocol specified
if [ "$TEST_UDP" = "false" ] && [ "$TEST_TCP" = "false" ]; then
    TEST_UDP=true
fi

# Main execution
main() {
    echo ""
    echo "====================================================================="
    echo "              NetaBase Port Configuration and Testing"
    echo "====================================================================="

    log_info "Configuration:"
    echo "  Test Host: $TEST_HOST"
    echo "  Test Ports: $TEST_PORTS"
    echo "  Remote Host: ${REMOTE_HOST:-none}"
    echo "  Timeout: ${TIMEOUT}s"
    echo "  Test Local: $TEST_LOCAL"
    echo "  Test Remote: $TEST_REMOTE"
    echo "  Test UDP: $TEST_UDP"
    echo "  Test TCP: $TEST_TCP"
    echo "  Verbose: $VERBOSE"

    if [ "$SHOW_FIREWALL" = "true" ]; then
        show_firewall_help "$TEST_PORTS"
    fi

    get_system_info
    check_netabase_processes

    # Handle port range scanning
    if [ -n "$SCAN_RANGE" ]; then
        local start_port="${SCAN_RANGE%:*}"
        local end_port="${SCAN_RANGE#*:}"

        echo ""
        log_info "=== PORT RANGE SCANNING ==="

        if [ "$TEST_LOCAL" = "true" ]; then
            if [ "$TEST_UDP" = "true" ]; then
                scan_port_range "$TEST_HOST" "$start_port" "$end_port" "udp" "local"
            fi
            if [ "$TEST_TCP" = "true" ]; then
                scan_port_range "$TEST_HOST" "$start_port" "$end_port" "tcp" "local"
            fi
        fi

        if [ "$TEST_REMOTE" = "true" ] && [ -n "$REMOTE_HOST" ]; then
            if [ "$TEST_UDP" = "true" ]; then
                scan_port_range "$REMOTE_HOST" "$start_port" "$end_port" "udp" "remote"
            fi
            if [ "$TEST_TCP" = "true" ]; then
                scan_port_range "$REMOTE_HOST" "$start_port" "$end_port" "tcp" "remote"
            fi
        fi
    fi

    # Handle individual port testing
    if [ "$TEST_LOCAL" = "true" ] || [ "$TEST_REMOTE" = "true" ]; then
        echo ""
        log_info "=== PORT TESTING ==="

        IFS=',' read -ra PORTS <<< "$TEST_PORTS"
        for port in "${PORTS[@]}"; do
            port=$(echo "$port" | tr -d ' ')  # Remove whitespace

            if [ "$TEST_LOCAL" = "true" ]; then
                if [ "$TEST_UDP" = "true" ]; then
                    test_port_availability "$TEST_HOST" "$port" "udp"
                fi
                if [ "$TEST_TCP" = "true" ]; then
                    test_port_availability "$TEST_HOST" "$port" "tcp"
                fi
            fi

            if [ "$TEST_REMOTE" = "true" ] && [ -n "$REMOTE_HOST" ]; then
                test_ping_connectivity "$REMOTE_HOST" "$TIMEOUT"
                if [ "$TEST_UDP" = "true" ]; then
                    test_remote_connectivity "$REMOTE_HOST" "$port" "udp" "$TIMEOUT"
                fi
                if [ "$TEST_TCP" = "true" ]; then
                    test_remote_connectivity "$REMOTE_HOST" "$port" "tcp" "$TIMEOUT"
                fi
            fi
        done
    fi

    echo ""
    log_info "Port testing completed"

    if [ "$TEST_REMOTE" = "true" ] && [ -z "$REMOTE_HOST" ]; then
        log_warning "Remote testing requested but no remote host specified (use -r/--remote)"
    fi
}

# Run main function
main "$@"
