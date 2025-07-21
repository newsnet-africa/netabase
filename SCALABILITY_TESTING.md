# NetaBase P2P Platform Scalability Testing

This document provides comprehensive guidance for running, understanding, and interpreting scalability tests for the NetaBase peer-to-peer distributed hash table (DHT) platform.

## Overview

NetaBase scalability tests are designed to validate how the P2P platform performs as the number of participating nodes increases. These tests help identify performance bottlenecks, measure system limits, and validate the platform's readiness for production deployments.

### What We Test

The scalability test suite evaluates several critical aspects of P2P performance:

- **Network Connectivity**: Connection establishment time and success rates
- **DHT Operations**: PUT/GET operation latency and throughput
- **Node Discovery**: Bootstrap performance and peer routing efficiency  
- **Resilience**: Network churn and partition recovery capabilities
- **Resource Usage**: Memory, CPU, and file descriptor consumption
- **Topology Performance**: Different network connection patterns
- **Data Handling**: Performance with varying record sizes and counts

## Quick Start

### Prerequisites

Before running scalability tests, ensure your system meets the basic requirements:

```bash
# Validate your environment (recommended first step)
./scripts/validate_scalability_setup.sh
```

**System Requirements:**
- **RAM**: 4GB+ minimum, 16GB+ recommended for large-scale tests
- **CPU**: 2+ cores minimum, 8+ cores recommended
- **Disk**: 1GB+ free space for temporary node data
- **Network**: Unrestricted UDP port access
- **File Descriptors**: `ulimit -n 4096` or higher

### Running Your First Test

---

## Configurable Tests Using Clap

You can also create and run highly configurable tests using the [Clap](https://docs.rs/clap/latest/clap/) command-line argument parser in Rust. This allows you to parameterize your tests directly from the command line or scripts.

### Example: Integration Test with Clap

You can add a test like this to `netabase/tests/configurable_test.rs`:

```netabase/tests/configurable_test.rs#L1-80
use clap::Parser;
use std::process;
use tokio;

/// A configurable test that can accept command-line arguments
#[derive(Parser, Debug)]
#[command(name = "configurable_test")]
#[command(about = "A test that accepts various configuration parameters")]
struct Args {
    /// Number of iterations to run
    #[arg(short, long, default_value = "10")]
    iterations: u32,

    /// Timeout in seconds
    #[arg(short, long, default_value = "30")]
    timeout: u64,

    /// Test mode to run
    #[arg(short, long, default_value = "basic")]
    mode: String,

    /// Enable verbose output
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    /// Custom message to use in test
    #[arg(short = 'M', long)]
    message: Option<String>,

    /// Port number to use
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::test]
async fn test_with_arguments() {
    // Parse command line arguments
    let args = Args::parse();

    if args.verbose {
        println!("Running configurable test with parameters:");
        println!("  Iterations: {}", args.iterations);
        println!("  Timeout: {}s", args.timeout);
        println!("  Mode: {}", args.mode);
        println!("  Port: {}", args.port);
        if let Some(ref msg) = args.message {
            println!("  Message: {}", msg);
        }
    }

    // Run test based on mode
    match args.mode.as_str() {
        "basic" => run_basic_test(&args).await,
        "network" => run_network_test(&args).await,
        "performance" => run_performance_test(&args).await,
        _ => {
            eprintln!("Unknown test mode: {}", args.mode);
            process::exit(1);
        }
    }
}
```

#### Running the Integration Test

You can run this test and pass arguments using `cargo test`:

```bash
cargo test test_with_arguments -- --iterations 5 --verbose
cargo test test_with_arguments -- --mode network --port 9090 --timeout 20
cargo test test_with_arguments -- --mode performance --iterations 100
```

---

### Example: Standalone Test Runner Binary

You can also create a binary in `src/bin/test_runner.rs` for even more flexible CLI testing:

```netabase/src/bin/test_runner.rs#L1-80
use clap::Parser;
use std::process;
use tokio;

/// A configurable test runner that can accept command-line arguments
#[derive(Parser, Debug)]
#[command(name = "test_runner")]
#[command(about = "A standalone test runner with configurable parameters")]
#[command(version = "1.0")]
struct Args {
    /// Number of test iterations to run
    #[arg(short, long, default_value = "5")]
    iterations: u32,

    /// Timeout in seconds for each test
    #[arg(short, long, default_value = "10")]
    timeout: u64,

    /// Test suite to run
    #[arg(short = 's', long, default_value = "all")]
    suite: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Custom configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Test data directory
    #[arg(short, long, default_value = "./test_data")]
    data_dir: String,

    /// Number of concurrent workers
    #[arg(short, long, default_value = "1")]
    workers: u32,

    /// Skip cleanup after tests
    #[arg(long)]
    no_cleanup: bool,

    /// Run only specific test by name
    #[arg(long)]
    test_name: Option<String>,
}
```

#### Running the Test Runner

```bash
cargo build --bin test_runner
./target/debug/test_runner --verbose --suite all --iterations 3
./target/debug/test_runner --suite network --timeout 30 --workers 4
./target/debug/test_runner --suite performance --data-dir ./custom_test_data --no-cleanup
./target/debug/test_runner --test-name "integration_test_2" --verbose
./target/debug/test_runner --config ./test_config.toml --suite integration
```

---

**Key Features:**
- Command-line arguments for test configuration
- Default values and type safety
- Support for different test modes and suites
- Timeout and concurrency control
- Verbose output and error handling

This approach gives you flexible, configurable tests that can be easily parameterized for different testing scenarios!


```bash
# Quick smoke test (3 nodes, 30 seconds) - great for development
./scripts/run_scalability_tests.sh --test-type smoke --max-nodes 3 --duration 30

# Basic scalability test (10 nodes, 2 minutes) - good for validation
./scripts/run_scalability_tests.sh --test-type linear --max-nodes 10 --duration 120

# Comprehensive test suite (20 nodes, 5 minutes) - full evaluation
./scripts/run_scalability_tests.sh
```

## Test Types and Scenarios

### 1. Linear Node Scaling (`--test-type linear`)
Tests performance degradation as nodes are gradually added to the network.

**What it measures:**
- Connection establishment time vs. network size
- DHT operation latency scaling
- Resource usage growth patterns

**Use when:** Evaluating how the platform scales from small to large deployments.

```bash
# Example: Test scaling from 1 to 25 nodes
./scripts/run_scalability_tests.sh --test-type linear --max-nodes 25 --duration 180
```

### 2. Network Topology Performance (`--test-type topology`)
Compares different network connection patterns to find optimal configurations.

**Topologies tested:**
- **Full Mesh**: Every node connects to every other node
- **Ring**: Nodes form a circular connection pattern
- **Star**: Central hub with spoke connections
- **Random**: Probabilistic connections between nodes
- **Bootstrap**: Dedicated bootstrap nodes for joining

**Use when:** Optimizing network topology for specific deployment scenarios.

### 3. High Throughput Operations (`--test-type throughput`)
Tests maximum DHT operation rates with intensive concurrent operations.

**What it measures:**
- Operations per second across all nodes
- Latency under high load
- System stability under stress

**Use when:** Planning for high-traffic production deployments.

### 4. Network Churn Resilience (`--test-type churn`)
Simulates nodes dynamically joining and leaving the network.

**What it measures:**
- Network stability during membership changes
- Recovery time after node departures
- New node integration performance

**Use when:** Validating behavior in dynamic environments with frequent node changes.

### 5. Bootstrap Performance (`--test-type bootstrap`)
Measures how quickly new nodes can join and synchronize with an existing network.

**What it measures:**
- Time from connection to full network participation
- Bootstrap node efficiency
- Network discovery performance

**Use when:** Optimizing new node onboarding processes.

### 6. Large Record Scalability (`--test-type large-records`)
Tests performance with large data payloads.

**Default configuration:**
- Record size: 64KB (configurable with `--record-size`)
- Fewer records per node due to size
- Memory usage monitoring

**Use when:** Planning to store large data objects in the DHT.

### 7. Concurrent Operations (`--test-type concurrent`)
Tests parallel DHT operations across multiple nodes simultaneously.

**What it measures:**
- Race condition handling
- Lock contention under load
- Parallel operation throughput

**Use when:** Validating high-concurrency scenarios.

### 8. Network Partition Recovery (`--test-type partition`)
Simulates network splits and measures healing capabilities.

**What it measures:**
- Behavior during network isolation
- Recovery time after partition healing
- Data consistency after network reunification

**Use when:** Validating resilience in unreliable network environments.

### 9. Stress Testing (`--test-type stress`)
Maximum scale testing with resource monitoring.

**Features:**
- Uses 2x normal node count (configurable via environment)
- Extended test duration
- Comprehensive resource monitoring
- Failure rate tracking

**Use when:** Finding system limits and breaking points.

### 10. Smoke Test (`--test-type smoke`)
Quick validation test suitable for CI/CD pipelines.

**Characteristics:**
- Fast execution (30-60 seconds)
- Small node count (3-5 nodes)
- Basic functionality validation
- Minimal resource usage

**Use when:** Automated testing, development validation, or quick health checks.

## Configuration Options

### Command Line Arguments

```bash
./scripts/run_scalability_tests.sh [OPTIONS]

# Core configuration
--max-nodes N          # Maximum number of test nodes (default: 20)
--duration SECS        # Test duration in seconds (default: 300)
--test-type TYPE       # Specific test to run (default: all)

# Performance tuning
--interval SECS        # Node spawn interval in seconds (default: 2)
--timeout SECS         # Operation timeout (default: 30)
--records N            # Records per node (default: 10)
--record-size BYTES    # Record size in bytes (default: 1024)

# Behavior modifiers
--verbose              # Enable detailed metrics logging
--stress               # Enable stress testing mode
--no-partitions        # Disable network partition tests
--dry-run              # Show configuration without running
--validate-only        # Validate configuration and exit
```

### Environment Variables

```bash
# Test scale configuration
export NETABASE_SCALABILITY_MAX_NODES=30
export NETABASE_SCALABILITY_TEST_DURATION=600
export NETABASE_SCALABILITY_SPAWN_INTERVAL=1

# Performance tuning
export NETABASE_SCALABILITY_RECORDS_PER_NODE=20
export NETABASE_SCALABILITY_RECORD_SIZE=2048

# Behavior control
export NETABASE_SCALABILITY_VERBOSE=true
export NETABASE_SCALABILITY_TEST_PARTITIONS=false

# Stress testing
export NETABASE_STRESS_MAX_NODES=100
export NETABASE_STRESS_DURATION=1200
```

## Understanding Test Results

### Performance Metrics

#### Connection Metrics
```
Network Overview:
  Total nodes: 20
  Total connections: 190           # Close to theoretical max (20*19/2=190)
  Average connections per node: 9.5  # Good connectivity
```

**Interpretation:**
- **High connection count**: Good for redundancy, higher resource usage
- **Low connection count**: Resource efficient, potential single points of failure
- **Full mesh**: N*(N-1)/2 total connections, highest redundancy
- **Efficient topologies**: ~log(N) connections per node

#### Operation Performance
```
Aggregate Performance Metrics:
  Total successful operations: 15420
  Total failed operations: 23
  Overall failure rate: 0.15%        # Excellent - very low
  Average latency: 45.2ms            # Good for network operations
  Operations per second: 51.4        # Scales with network size
```

**Benchmarks:**
- **Failure rate**: <1% excellent, <5% acceptable, >10% concerning
- **Latency**: <50ms good, <100ms acceptable, >200ms investigate
- **Throughput**: Should scale roughly linearly with node count

#### Resource Usage
```
Resource Estimates:
  Estimated Memory Usage: ~1000MB    # 50MB per node
  Estimated File Descriptors: ~1000  # 50 FDs per node
  Test duration: ~5 minutes
```

**Monitoring:**
- Memory usage should be predictable (~50MB per node)
- File descriptor usage increases with connections
- CPU usage should be moderate during steady state

### Performance Analysis Examples

#### Scaling Efficiency
```bash
# Good scaling pattern
Node Count | Avg Latency | Ops/Sec | Connections/Node
    5      |    35ms     |   25.3  |      4.0
   10      |    42ms     |   48.7  |      9.0  
   20      |    45ms     |   89.2  |     19.0

# Concerning scaling pattern (investigate bottlenecks)
Node Count | Avg Latency | Ops/Sec | Connections/Node
    5      |    35ms     |   25.3  |      4.0
   10      |    89ms     |   31.2  |      6.5  # Latency spike, low connectivity
   20      |   156ms     |   42.1  |      8.2  # Performance degrading
```

#### Topology Comparison
```bash
# Typical results for different topologies
Topology     | Avg Latency | Throughput | Memory Usage | Recommended Use
-------------|-------------|------------|--------------|----------------
Full Mesh    |    35ms     |  High      | High         | Small networks
Ring         |    65ms     |  Medium    | Low          | Resource constrained
Star         |    45ms     |  Medium    | Medium       | Centralized scenarios
Random (30%) |    52ms     |  Medium    | Medium       | General purpose
Bootstrap    |    48ms     |  Good      | Medium       | Large networks
```

## Production Considerations

### Features Useful for Production

The scalability tests include several components that provide value for production P2P systems:

#### 1. Performance Metrics Collection
```rust
// Example integration in production code
use netabase::network_scalability::{PerformanceMetrics, setup_metrics_collection};

let metrics = PerformanceMetrics::new();
let metrics_receiver = setup_metrics_collection(&nodes, Duration::from_secs(10));

// Integration with monitoring systems
tokio::spawn(async move {
    while let Some(metrics_data) = metrics_receiver.recv().await {
        // Send to Prometheus, InfluxDB, DataDog, etc.
        monitoring_client.send_metrics(metrics_data).await;
    }
});
```

#### 2. Network Topology Optimization
Based on test results, choose optimal connection strategies:

```rust
// Production topology recommendations based on test results
match deployment_scenario {
    SmallNetwork => NetworkTopology::FullMesh,
    ResourceConstrained => NetworkTopology::Ring,
    HighAvailability => NetworkTopology::Random { connection_ratio: 0.3 },
    LargeScale => NetworkTopology::Bootstrap { bootstrap_count: 5 },
}
```

#### 3. Adaptive Configuration
```rust
// Dynamic configuration based on network conditions
let recommended_config = match network_size {
    1..=10 => Config { connections_per_node: network_size - 1, ..default },
    11..=50 => Config { connections_per_node: 15, ..default },
    51..=200 => Config { connections_per_node: 25, ..default },
    _ => Config { connections_per_node: 50, ..default },
};
```

#### 4. Health Monitoring
```rust
// Production health checks derived from test metrics
pub struct NetworkHealth {
    pub average_latency_ms: f64,
    pub failure_rate_percent: f64,
    pub connection_count: usize,
    pub operations_per_second: f64,
}

impl NetworkHealth {
    pub fn is_healthy(&self) -> bool {
        self.failure_rate_percent < 5.0 
            && self.average_latency_ms < 100.0
            && self.connection_count > 0
    }
    
    pub fn needs_scaling(&self) -> bool {
        self.average_latency_ms > 200.0 || self.failure_rate_percent > 10.0
    }
}
```

### Deployment Recommendations

#### Small Networks (2-10 nodes)
```bash
# Recommended test
./scripts/run_scalability_tests.sh --test-type topology --max-nodes 10

# Suggested configuration
- Topology: Full mesh or star
- Connection maintenance: Active keep-alive
- Bootstrap nodes: 1-2 dedicated nodes
```

#### Medium Networks (10-50 nodes)
```bash
# Recommended test
./scripts/run_scalability_tests.sh --test-type linear --max-nodes 50 --duration 600

# Suggested configuration  
- Topology: Random with 30-40% connection ratio
- Bootstrap nodes: 3-5 well-connected nodes
- Resource limits: 2GB RAM, 2000 file descriptors
```

#### Large Networks (50+ nodes)
```bash
# Recommended test
./scripts/run_scalability_tests.sh --test-type stress --max-nodes 100 --stress

# Suggested configuration
- Topology: Bootstrap-based with dedicated infrastructure nodes
- Connection limits: 20-50 connections per node
- Monitoring: Real-time metrics collection
- Redundancy: Multiple bootstrap nodes across regions
```

## Continuous Integration

### GitHub Actions Integration

```yaml
name: Scalability Tests
on: 
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
    
jobs:
  smoke-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Validate scalability setup
        run: ./scripts/validate_scalability_setup.sh
        
      - name: Run smoke test
        run: ./scripts/run_scalability_tests.sh --test-type smoke --max-nodes 3 --duration 30
        
  nightly-scalability:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run comprehensive tests
        run: ./scripts/run_scalability_tests.sh --max-nodes 15 --duration 300
        
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: scalability-results
          path: test-results/
```

### Jenkins Pipeline

```groovy
pipeline {
    agent any
    
    stages {
        stage('Validate Environment') {
            steps {
                sh './scripts/validate_scalability_setup.sh'
            }
        }
        
        stage('Smoke Test') {
            steps {
                sh './scripts/run_scalability_tests.sh --test-type smoke --max-nodes 5 --duration 60'
            }
        }
        
        stage('Scalability Suite') {
            when { branch 'main' }
            steps {
                sh './scripts/run_scalability_tests.sh --max-nodes 20 --duration 300'
            }
            post {
                always {
                    archiveArtifacts artifacts: 'test-results/**', allowEmptyArchive: true
                }
            }
        }
    }
}
```

## Troubleshooting

### Common Issues and Solutions

#### Out of Memory Errors
```bash
# Symptoms
- Test crashes with "out of memory" errors
- System becomes unresponsive
- Swap usage increases dramatically

# Solutions
./scripts/run_scalability_tests.sh --max-nodes 10  # Reduce node count
# Or increase system memory
# Or use stress testing mode with monitoring
```

#### File Descriptor Limits
```bash
# Symptoms
- "Too many open files" errors
- Connection failures after initial success

# Solutions
ulimit -n 8192  # Increase limit
./scripts/run_scalability_tests.sh  # Retry

# Permanent fix (add to ~/.bashrc or ~/.profile)
echo 'ulimit -n 8192' >> ~/.bashrc
```

#### Port Exhaustion
```bash
# Symptoms
- "Address already in use" errors
- Cannot bind to UDP ports
- Tests fail to start nodes

# Solutions
# Wait for port cleanup (30-60 seconds)
# Or increase spawn interval
./scripts/run_scalability_tests.sh --interval 5

# Check for stuck processes
ps aux | grep netabase
kill -9 <stuck_process_pids>
```

#### Compilation Errors
```bash
# Symptoms
- Rust compilation failures
- Missing dependencies
- Version conflicts

# Solutions
cargo clean
cargo build  # Check for specific errors
rustup update  # Update Rust toolchain

# If using nightly features
rustup install nightly
rustup default nightly
```

#### Network Connectivity Issues
```bash
# Symptoms
- Connection timeouts
- High failure rates
- Nodes cannot discover each other

# Solutions
# Check firewall settings
sudo ufw status
sudo ufw allow 9901/udp  # Or your test ports

# Test network connectivity
nc -u -l -p 9901 &  # In one terminal
echo "test" | nc -u localhost 9901  # In another

# Reduce network demands
./scripts/run_scalability_tests.sh --timeout 60 --interval 3
```

#### Performance Issues
```bash
# Symptoms
- Very high latency (>500ms)
- Low throughput
- Test timeouts

# Root causes and solutions:
# 1. System overload
htop  # Check CPU/memory usage
./scripts/run_scalability_tests.sh --max-nodes 5  # Scale down

# 2. Network congestion  
iftop  # Monitor network usage
./scripts/run_scalability_tests.sh --record-size 256  # Smaller records

# 3. Disk I/O bottlenecks
iostat -x 1  # Monitor disk usage
# Use faster storage or reduce record counts
```

### Performance Analysis Tools

#### During Test Execution
```bash
# Terminal 1: Run the test
./scripts/run_scalability_tests.sh --verbose

# Terminal 2: Monitor resources
htop  # CPU and memory
iftop  # Network usage  
iostat -x 1  # Disk I/O
netstat -an | grep ESTABLISHED | wc -l  # Connection count
```

#### Log Analysis
```bash
# Enable verbose logging
export RUST_LOG=debug
./scripts/run_scalability_tests.sh --verbose

# Filter important metrics
grep -E "(average latency|failure rate|operations per second)" test.log

# Look for error patterns
grep -E "(ERROR|WARN)" test.log | sort | uniq -c | sort -nr
```

### Getting Help

If you encounter issues not covered in this troubleshooting guide:

1. **Check system requirements**: Run `./scripts/validate_scalability_setup.sh`
2. **Review logs**: Enable verbose mode and check for specific error messages
3. **Start small**: Try smoke test first, then gradually increase scale
4. **Monitor resources**: Watch memory, CPU, and network usage during tests
5. **Environment variables**: Verify configuration with `--dry-run` flag

## Advanced Usage

### Custom Test Scenarios

#### Testing Specific Network Conditions
```bash
# High-latency network simulation
./scripts/run_scalability_tests.sh --timeout 120 --interval 10

# High-churn environment
./scripts/run_scalability_tests.sh --test-type churn --duration 600

# Large data scenario
./scripts/run_scalability_tests.sh --test-type large-records --record-size 1048576
```

#### Load Testing for Production Planning
```bash
# Simulate production load
export NETABASE_SCALABILITY_MAX_NODES=50
export NETABASE_SCALABILITY_RECORDS_PER_NODE=100
export NETABASE_SCALABILITY_TEST_DURATION=1800  # 30 minutes
export NETABASE_SCALABILITY_VERBOSE=true

./scripts/run_scalability_tests.sh --test-type stress
```

#### Automated Performance Regression Testing
```bash
#!/bin/bash
# performance_regression.sh

BASELINE_LATENCY=50  # ms
BASELINE_THROUGHPUT=40  # ops/sec

# Run test and extract metrics
RESULTS=$(./scripts/run_scalability_tests.sh --test-type linear --max-nodes 10 | grep "Average latency")

# Parse results and compare
# (Implementation would parse actual metrics)
echo "Performance regression check completed"
```

### Integration with Monitoring Systems

#### Prometheus Integration Example
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'netabase-scalability'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 5s
    metrics_path: '/metrics'
```

#### Custom Metrics Export
```bash
# Export test results to monitoring format
./scripts/run_scalability_tests.sh --verbose 2>&1 | \
  grep -E "(latency|throughput|failure)" | \
  awk '{print "netabase_" $1 " " $2 " " systime()}' > metrics.prom
```

This comprehensive scalability testing framework provides the tools and insights needed to validate NetaBase performance at scale, optimize for production deployments, and maintain reliability as your P2P network grows.