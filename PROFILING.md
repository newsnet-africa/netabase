# Netabase Performance Profiling Guide

This guide explains how to profile netabase benchmarks to identify performance bottlenecks in P2P networking operations.

## 🚨 CRITICAL: Must Use --profile-time Flag

**Flamegraphs will NOT be generated unless you use the `--profile-time` flag:**

```bash
# ❌ WRONG - No flamegraphs
cargo bench

# ✅ CORRECT - Generates flamegraphs
cargo bench -- --profile-time=5
```

**Why?** The profiler needs continuous execution (not short iterations) to collect meaningful CPU samples.

## Quick Start

Run benchmarks with profiling enabled:
```bash
# Profile for 5 seconds per benchmark
cargo bench -- --profile-time=5
```

**Important:** You MUST use the `--profile-time` flag to enable profiling and generate flamegraphs.

View flamegraphs:
```bash
# List all flamegraphs
ls target/criterion/*/profile/flamegraph.svg

# Open in browser (Linux)
xdg-open target/criterion/netabase_creation/profile/flamegraph.svg

# Open in browser (macOS)
open target/criterion/netabase_creation/profile/flamegraph.svg
```

## Profiling Specific Operations

### Network Operations

```bash
# Profile netabase instance creation
cargo bench -- --profile-time=5 netabase_creation

# Profile swarm lifecycle (start/stop)
cargo bench -- --profile-time=5 swarm_lifecycle

# Profile DHT operations
cargo bench -- --profile-time=5 dht_put_record
```

### Storage Operations

```bash
# Profile local record storage
cargo bench -- local_record_storage

# Profile local queries
cargo bench -- query_local_records
```

### Event System

```bash
# Profile event subscription overhead
cargo bench -- event_subscription
```

## Understanding Netabase Flamegraphs

### Common Components

**Netabase-specific functions to watch:**
- `Netabase::new_with_path` - initialization overhead
- `Netabase::start_swarm` - swarm startup time
- `Netabase::put_record_locally` - local storage operations
- `Netabase::query_local_records` - query performance
- `libp2p::swarm` - network stack overhead

### Typical Bottlenecks

1. **Database I/O** (wide bars in storage benchmarks)
   - Shows up as `sled::Tree::insert`, `redb::WriteTransaction`
   - Solution: Batch operations, use faster storage backend

2. **Serialization** (wide bars in record operations)
   - Shows up as `bincode::encode`, `serde::serialize`
   - Solution: Use smaller data types, custom serialization

3. **Network Overhead** (visible in DHT benchmarks)
   - Shows up as `libp2p::kad`, `tokio::runtime`
   - Expected: Network operations are inherently async and distributed

4. **Channel Contention** (visible in event benchmarks)
   - Shows up as `tokio::sync::broadcast`
   - Solution: Reduce broadcast frequency or use bounded channels

## Benchmark Organization

### bench_netabase_creation
Measures overhead of creating a new Netabase instance with temporary storage.

**Key metrics:**
- Memory allocation
- Database initialization
- Channel setup

**Expected profile:**
- `tempfile::TempDir::new` - temporary directory creation
- `Netabase::new_with_path` - instance initialization
- Minimal overhead (<1ms typical)

### bench_swarm_lifecycle
Measures start_swarm() and stop_swarm() overhead.

**Key metrics:**
- Thread spawning
- Network listener binding
- Graceful shutdown time

**Expected profile:**
- `tokio::spawn` - background task creation
- `libp2p::Swarm::new` - swarm initialization
- `libp2p::tcp::Transport::dial` - transport setup

### bench_local_record_storage
Measures put_record_locally() performance at different data sizes (100, 1000, 10000 bytes).

**Key metrics:**
- Serialization time
- Database write latency
- Scaling with data size

**Expected profile:**
- `bincode::encode_to_vec` - serialization
- `netabase_store::Tree::put` - storage operation
- Linear scaling with data size

### bench_query_local_records
Measures query_local_records() performance with varying record counts (10, 100, 1000).

**Key metrics:**
- Database scan time
- Deserialization overhead
- Memory allocation for results

**Expected profile:**
- `netabase_store::Tree::iter` - iteration
- `bincode::decode_from_slice` - deserialization
- Linear scaling with record count

### bench_dht_put_record
Measures DHT put_record() overhead (will timeout without peers).

**Key metrics:**
- Record preparation
- DHT query initiation
- Timeout handling

**Expected profile:**
- Most time in `tokio::time::timeout` (expected)
- `libp2p::kad::put_record` shows DHT overhead
- Network operations dominate time

### bench_event_subscription
Measures subscribe_to_broadcasts() and channel drop overhead.

**Key metrics:**
- Channel creation time
- Subscription cleanup
- Memory allocation

**Expected profile:**
- `tokio::sync::broadcast::channel` - channel setup
- Minimal overhead (<1μs typical)
- No network activity

## Profiling Best Practices

### 1. Run Multiple Iterations

Get stable measurements:
```bash
cargo bench -- --sample-size 100
```

### 2. Compare Baseline

Track performance over time:
```bash
# Establish baseline
cargo bench -- --save-baseline main

# After changes
cargo bench -- --baseline main
```

### 3. Isolate Bottlenecks

Profile one operation at a time:
```bash
# Just storage operations
cargo bench -- local_record_storage

# Just network operations
cargo bench -- dht_put_record
```

### 4. Consider Async Overhead

Netabase is async-heavy. Flamegraphs show:
- `tokio::runtime::block_on` - expected for sync benchmarks
- `tokio::task::spawn` - background task creation
- `futures::poll` - async state machine overhead

This is normal and necessary for async operations.

## Optimization Targets

### High-Impact Optimizations

1. **Reduce Serialization Overhead**
   - Use smaller data types
   - Consider custom serialization for hot paths
   - Profile: Look for wide `bincode::encode` bars

2. **Batch Database Operations**
   - Group multiple puts into transactions
   - Reduce fsync frequency
   - Profile: Look for many thin `Tree::put` calls

3. **Optimize Channel Usage**
   - Use bounded channels with appropriate capacity
   - Consider tokio's mpsc for single consumer
   - Profile: Look for contention in `broadcast::send`

### Low-Impact (Don't Optimize)

1. **Swarm Creation** - Happens once per instance
2. **Temporary Directory** - Benchmark-only overhead
3. **Async Runtime** - Required framework overhead

## Advanced: CPU Profiling

For CPU-level analysis on Linux:

```bash
# Record with hardware counters
perf record -F 997 -g --call-graph dwarf -- cargo bench -- --test

# View hotspots
perf report --stdio

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > perf.svg
```

## Continuous Monitoring

Integrate benchmarks into CI:

```bash
# Run benchmarks in CI
cargo bench --no-run  # Check compilation
cargo bench -- --test # Quick run without full sampling
```

Save criterion results to track performance over commits.

## Troubleshooting

### Benchmarks Take Too Long

Reduce iterations:
```bash
cargo bench -- --sample-size 10
```

### Inconsistent Results

Benchmarks involve I/O and networking which can vary. Solutions:
- Run multiple times and average
- Close background applications
- Use `--sample-size 100` for more data points
- Consider absolute times, not just relative comparisons

### Missing Flamegraphs

Check that files were generated:
```bash
ls target/criterion/*/profile/
```

If missing, check pprof installation:
```bash
cargo update -p pprof
```

## Interpreting Results

### Good Performance Indicators

- **Flat flamegraphs** - No obvious bottlenecks
- **Expected operations dominant** - Serialization for storage, networking for DHT
- **Predictable scaling** - Linear with data size/count

### Red Flags

- **Unexpected deep stacks** - Possible inefficiency
- **Lock contention** - Synchronization issues
- **Excessive allocation** - Memory management overhead
- **Non-linear scaling** - Algorithmic inefficiency

## Further Resources

- [Netabase Store Profiling Guide](../netabase_store/PROFILING.md)
- [Criterion.rs Book](https://bheisler.github.io/criterion.rs/book/)
- [Async Rust Performance](https://tokio.rs/tokio/topics/performance)
- [libp2p Performance](https://docs.libp2p.io/concepts/performance/)
