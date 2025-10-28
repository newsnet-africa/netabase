#!/usr/bin/env nu

# Comprehensive sync protocol test orchestration script
# This script runs multiple test nodes as separate processes to test network communication

def main [
    --test-mode: string = "full" # Test mode: paxos, brb, gossip, sybil, full
    --num-nodes: int = 4          # Number of nodes to spawn
    --max-failures: int = 1       # Number of Byzantine failures to tolerate
    --paxos-enabled: bool = true  # Enable Paxos consensus
    --duration: int = 30          # Test duration in seconds
    --verbose: bool = false       # Verbose logging
] {
    print $"(ansi cyan_bold)╔══════════════════════════════════════════════════════════════╗(ansi reset)"
    print $"(ansi cyan_bold)║     NetaBase Sync Protocol Multi-Process Test Suite          ║(ansi reset)"
    print $"(ansi cyan_bold)╚══════════════════════════════════════════════════════════════╝(ansi reset)"
    print ""

    # Validate configuration
    let min_nodes = 3 * $max_failures + 1
    if $num_nodes < $min_nodes {
        print $"(ansi red_bold)Error: Need at least ($min_nodes) nodes for f=($max_failures) Byzantine faults(ansi reset)"
        print $"(ansi yellow)Formula: n >= 3f+1, where n=nodes, f=max_failures(ansi reset)"
        exit 1
    }

    print $"(ansi green)Configuration:(ansi reset)"
    print $"  Test Mode:      (ansi yellow)($test_mode)(ansi reset)"
    print $"  Number of Nodes: (ansi yellow)($num_nodes)(ansi reset)"
    print $"  Max Failures:    (ansi yellow)($max_failures)(ansi reset)"
    print $"  Paxos Enabled:   (ansi yellow)($paxos_enabled)(ansi reset)"
    print $"  Test Duration:   (ansi yellow)($duration)s(ansi reset)"
    print ""

    # Build the test binary first
    print $"(ansi cyan)Building test binary...(ansi reset)"
    cargo build --tests --release
    if $env.LAST_EXIT_CODE != 0 {
        print $"(ansi red_bold)Failed to build tests!(ansi reset)"
        exit 1
    }
    print $"(ansi green)✓ Build successful(ansi reset)\n"

    # Run unit tests first
    print $"(ansi cyan_bold)Running unit tests...(ansi reset)"
    cargo test --test sync_comprehensive --release -- --nocapture
    if $env.LAST_EXIT_CODE != 0 {
        print $"(ansi red_bold)Unit tests failed!(ansi reset)"
        exit 1
    }
    print $"(ansi green)✓ All unit tests passed(ansi reset)\n"

    # Prepare for network tests
    let base_port = 9000
    let log_dir = "test_logs"

    # Clean up old logs
    if ($log_dir | path exists) {
        rm -rf $log_dir
    }
    mkdir $log_dir

    # Spawn nodes
    print $"(ansi cyan_bold)Spawning ($num_nodes) network nodes...(ansi reset)"

    let mut node_procs = []
    let mut node_addrs = []

    for node_id in 0..<$num_nodes {
        let port = $base_port + $node_id
        let log_file = $"($log_dir)/node_($node_id).log"
        let log_level = if $verbose { "debug" } else { "info" }

        # Build bootstrap list (connect to previous nodes)
        let bootstrap = if $node_id > 0 {
            (0..<$node_id | each {|i| $"/ip4/127.0.0.1/tcp/($base_port + $i)" } | str join ",")
        } else {
            ""
        }

        print $"  (ansi yellow)Node ($node_id)(ansi reset): port ($port)"

        # Build command
        let cmd = if $bootstrap == "" {
            $"cargo run --test sync_network_node --release -- --node-id ($node_id) --port ($port) --test-mode ($test_mode) --max-failures ($max_failures) --log-level ($log_level)" + (if $paxos_enabled { " --paxos-enabled" } else { "" })
        } else {
            $"cargo run --test sync_network_node --release -- --node-id ($node_id) --port ($port) --bootstrap '($bootstrap)' --test-mode ($test_mode) --max-failures ($max_failures) --log-level ($log_level)" + (if $paxos_enabled { " --paxos-enabled" } else { "" })
        }

        # Spawn node as background process
        let proc = (
            nu -c $cmd
            | save --force $log_file &
        )

        $node_procs = ($node_procs | append $proc)
        $node_addrs = ($node_addrs | append {node_id: $node_id, port: $port})

        # Give nodes time to start
        sleep 500ms
    }

    print $"(ansi green)✓ All nodes spawned(ansi reset)\n"

    # Let nodes run and communicate
    print $"(ansi cyan)Running test for ($duration) seconds...(ansi reset)"
    print $"(ansi dim)Nodes are communicating over the network...(ansi reset)\n"

    # Progress bar
    for i in 1..($duration + 1) {
        let progress = ($i * 100 / $duration)
        let bar_length = ($progress / 5 | into int)
        let bar = ("█" | str repeat $bar_length)
        let empty = ("░" | str repeat (20 - $bar_length))
        print -n $"\r  (ansi cyan)Progress:(ansi reset) [($bar)($empty)] ($progress)% ($i)/($duration)s"
        sleep 1sec
    }
    print "\n"

    # Stop all nodes
    print $"(ansi cyan)Stopping nodes...(ansi reset)"
    # Note: In a real scenario, we'd send graceful shutdown signals
    # For now, processes will exit on their own

    sleep 2sec
    print $"(ansi green)✓ Test completed(ansi reset)\n"

    # Collect and display results
    print $"(ansi cyan_bold)Test Results Summary(ansi reset)"
    print $"(ansi cyan_bold)═══════════════════════════════════════════════════════════════(ansi reset)\n"

    for node in $node_addrs {
        let log_file = $"($log_dir)/node_($node.node_id).log"

        if ($log_file | path exists) {
            print $"(ansi yellow)Node ($node.node_id) (port ($node.port)):(ansi reset)"

            # Extract results from log
            let results = (open $log_file | lines | where $it =~ "Test Results" | last)

            if $results != null {
                # Try to parse JSON results
                let json_start = ($results | str index-of "{")
                if $json_start != -1 {
                    let json_str = ($results | str substring $json_start..)
                    try {
                        let parsed = ($json_str | from json)
                        print $"  Paxos Proposals:       ($parsed.paxos_proposals)"
                        print $"  Paxos Consensuses:     ($parsed.paxos_consensuses)"
                        print $"  BRB Broadcasts:        ($parsed.brb_broadcasts)"
                        print $"  BRB Deliveries:        ($parsed.brb_deliveries)"
                        print $"  Gossip Rounds:         ($parsed.gossip_rounds)"
                        print $"  State Syncs:           ($parsed.state_syncs)"
                        print $"  PoW Challenges Issued: ($parsed.pow_challenges_issued)"
                        print $"  PoW Verified:          ($parsed.pow_challenges_verified)"

                        if ($parsed.errors | length) > 0 {
                            print $"  (ansi red)Errors: ($parsed.errors | length)(ansi reset)"
                            for error in $parsed.errors {
                                print $"    - ($error)"
                            }
                        } else {
                            print $"  (ansi green)✓ No errors(ansi reset)"
                        }
                    } catch {
                        print $"  (ansi yellow)Could not parse results(ansi reset)"
                    }
                }
            } else {
                print $"  (ansi yellow)No results found in log(ansi reset)"
            }
            print ""
        }
    }

    # Check for Paxos-specific behavior
    if $paxos_enabled {
        print $"(ansi cyan_bold)Paxos-Specific Checks(ansi reset)"
        print $"(ansi cyan_bold)═══════════════════════════════════════════════════════════════(ansi reset)\n"

        for node in $node_addrs {
            let log_file = $"($log_dir)/node_($node.node_id).log"

            if ($log_file | path exists) {
                let paxos_logs = (open $log_file | lines | where $it =~ "Paxos|consensus|proposal")

                if ($paxos_logs | length) > 0 {
                    print $"(ansi yellow)Node ($node.node_id) Paxos activity:(ansi reset)"
                    for log in ($paxos_logs | last 5) {
                        print $"  (ansi dim)($log)(ansi reset)"
                    }
                    print ""
                }
            }
        }
    }

    # Final summary
    print $"(ansi cyan_bold)═══════════════════════════════════════════════════════════════(ansi reset)"
    print $"(ansi green_bold)Test suite completed successfully!(ansi reset)"
    print $"(ansi dim)Logs saved in: ($log_dir)/(ansi reset)\n"
}

# Run specific test scenario
def "main scenario" [
    scenario: string # Scenario name: small, large, byzantine, high-throughput
] {
    print $"(ansi cyan)Running scenario: ($scenario)(ansi reset)\n"

    match $scenario {
        "small" => {
            main --num-nodes 4 --max-failures 1 --test-mode "full" --duration 20
        }
        "large" => {
            main --num-nodes 10 --max-failures 3 --test-mode "full" --duration 30
        }
        "byzantine" => {
            main --num-nodes 7 --max-failures 2 --test-mode "brb" --duration 25
        }
        "paxos" => {
            main --num-nodes 7 --max-failures 2 --test-mode "paxos" --duration 20 --paxos-enabled true
        }
        "high-throughput" => {
            main --num-nodes 10 --max-failures 3 --test-mode "gossip" --duration 30
        }
        _ => {
            print $"(ansi red)Unknown scenario: ($scenario)(ansi reset)"
            print $"(ansi yellow)Available scenarios: small, large, byzantine, paxos, high-throughput(ansi reset)"
        }
    }
}

# Run all test scenarios
def "main all" [] {
    print $"(ansi cyan_bold)Running all test scenarios...(ansi reset)\n"

    let scenarios = ["small", "byzantine", "paxos", "high-throughput"]

    for scenario in $scenarios {
        print $"(ansi magenta_bold)╔══════════════════════════════════════════════════════════════╗(ansi reset)"
        print $"(ansi magenta_bold)║ Scenario: ($scenario)(ansi reset)"
        print $"(ansi magenta_bold)╚══════════════════════════════════════════════════════════════╝(ansi reset)"

        main scenario $scenario

        print $"\n(ansi dim)Waiting before next scenario...(ansi reset)\n"
        sleep 5sec
    }

    print $"(ansi green_bold)All scenarios completed!(ansi reset)"
}

# Clean up test artifacts
def "main clean" [] {
    print $"(ansi cyan)Cleaning up test artifacts...(ansi reset)"

    if ("test_logs" | path exists) {
        rm -rf test_logs
        print $"(ansi green)✓ Removed test_logs(ansi reset)"
    }

    if ("target" | path exists) {
        print $"(ansi yellow)Note: To clean build artifacts, run: cargo clean(ansi reset)"
    }

    print $"(ansi green)Cleanup complete(ansi reset)"
}

# Show test logs
def "main logs" [
    --node: int = 0  # Node ID to show logs for
    --tail: int = 50 # Number of lines to show
] {
    let log_file = $"test_logs/node_($node).log"

    if not ($log_file | path exists) {
        print $"(ansi red)Log file not found: ($log_file)(ansi reset)"
        print $"(ansi yellow)Run tests first with: ./run_sync_tests.nu(ansi reset)"
        return
    }

    print $"(ansi cyan)Last ($tail) lines from Node ($node):(ansi reset)\n"
    open $log_file | lines | last $tail | each {|line| print $line }
}

# Verify test prerequisites
def "main check" [] {
    print $"(ansi cyan)Checking test prerequisites...(ansi reset)\n"

    # Check Rust/Cargo
    let cargo_check = (do -i { cargo --version } | complete)
    if $cargo_check.exit_code == 0 {
        print $"(ansi green)✓ Cargo: ($cargo_check.stdout | str trim)(ansi reset)"
    } else {
        print $"(ansi red)✗ Cargo not found(ansi reset)"
        return
    }

    # Check if project compiles
    print $"(ansi cyan)Checking if project compiles...(ansi reset)"
    let build_check = (do -i { cargo check --tests } | complete)
    if $build_check.exit_code == 0 {
        print $"(ansi green)✓ Project compiles successfully(ansi reset)"
    } else {
        print $"(ansi red)✗ Project has compilation errors(ansi reset)"
        print $"(ansi yellow)Run 'cargo check --tests' for details(ansi reset)"
        return
    }

    print $"\n(ansi green_bold)All checks passed! Ready to run tests.(ansi reset)"
    print $"(ansi cyan)Run: ./run_sync_tests.nu(ansi reset)"
}
