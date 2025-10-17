# NetabaseStore Benchmarks

This directory contains benchmarks comparing the NetabaseStore wrapper to raw sled operations.

## Available Benchmarks

### `wrapper_vs_raw_sled`

Compares the performance overhead of the NetabaseStore type-safe wrapper against raw sled database operations.

#### Benchmark Groups:

1. **insert_single** - Single insert operations
   - Measures the overhead of serializing typed models vs manual serialization

2. **get_single** - Single get operations
   - Measures the overhead of deserializing to typed models vs manual deserialization

3. **batch_insert** - Batch insert operations
   - Tests with 10, 100, and 1000 items
   - Measures throughput for bulk operations

4. **iteration** - Full tree iteration
   - Iterates over 1000 items
   - Measures deserialization overhead during iteration

5. **remove** - Deletion operations
   - Removes 1000 items sequentially
   - Measures cleanup performance

6. **mixed_workload** - Real-world simulation
   - 50% reads, 30% writes, 20% deletes
   - 1000 total operations on 500 initial items
   - Simulates typical application usage patterns

## Running Benchmarks

### Run all benchmarks:
```bash
cargo bench --bench wrapper_vs_raw_sled
```

### Run specific benchmark group:
```bash
cargo bench --bench wrapper_vs_raw_sled -- insert_single
cargo bench --bench wrapper_vs_raw_sled -- get_single
cargo bench --bench wrapper_vs_raw_sled -- mixed_workload
```

### Generate HTML reports:
```bash
cargo bench --bench wrapper_vs_raw_sled
# Reports will be in: target/criterion/
```

## Understanding Results

The benchmarks measure:
- **Time per operation**: Lower is better
- **Throughput**: Higher is better (operations/second)
- **Overhead percentage**: Difference between wrapper and raw sled

### Expected Overhead

The NetabaseStore wrapper adds minimal overhead:
- **Type safety**: Automatic discriminant-based tree routing
- **Serialization**: Bincode encoding/decoding with type validation
- **Key conversion**: Newtype wrapper conversion (zero-cost abstraction)

Typical overhead: **< 5%** for most operations

### Interpreting Results

Example output:
```
insert_single/wrapper   time:   [2.5 µs 2.6 µs 2.7 µs]
insert_single/raw_sled  time:   [2.4 µs 2.5 µs 2.6 µs]
```

This shows:
- Wrapper: ~2.6 µs per insert
- Raw sled: ~2.5 µs per insert
- Overhead: ~4% (acceptable for type safety benefits)

## Benchmark Details

### Test Data Model

```rust
struct Article {
    id: u64,
    title: String,
    content: String,  // ~540 bytes
    author: String,
    views: u64,
}
```

- Realistic data size (~600 bytes per article)
- Multiple field types (primitives and strings)
- Representative of typical domain models

### Comparison Methodology

**Wrapper Implementation:**
```rust
let store = SledStore::<TestDefinition>::new(path)?;
let tree = store.open_tree::<Article>();
tree.put(article)?;
```

**Raw Sled Implementation:**
```rust
let db = sled::open(path)?;
let tree = db.open_tree("articles")?;
let key = bincode::encode_to_vec(&id, config)?;
let value = bincode::encode_to_vec(&article, config)?;
tree.insert(key, value)?;
```

Both use:
- Same bincode configuration
- Same sled database
- Same serialization library
- Equivalent error handling

### What's Being Measured

**Wrapper adds:**
1. Type-safe model trait bounds
2. Discriminant-based tree selection
3. Newtype key wrapper conversion
4. Automatic serialization via NetabaseModel

**Raw sled requires:**
1. Manual serialization calls
2. Manual tree name management
3. Manual key/value byte vector creation
4. Manual deserialization with type annotations

## Performance Tips

If benchmarks show unacceptable overhead:

1. **Use batch operations** - Amortize per-operation costs
2. **Minimize allocations** - Reuse buffers where possible
3. **Profile hot paths** - Use `cargo flamegraph` to find bottlenecks
4. **Consider raw access** - Use `RawStore` trait for critical paths

## Continuous Benchmarking

To track performance regressions:

```bash
# Baseline
git checkout main
cargo bench --bench wrapper_vs_raw_sled -- --save-baseline main

# After changes
git checkout feature-branch
cargo bench --bench wrapper_vs_raw_sled -- --baseline main
```

This compares your changes against the main branch baseline.

## Contributing

When adding new benchmarks:

1. Follow existing naming conventions
2. Use realistic data sizes
3. Include both wrapper and raw sled implementations
4. Document what's being measured
5. Add to this README

## Hardware Notes

Benchmark results vary by hardware:
- **SSD vs HDD**: Significant difference in I/O operations
- **CPU**: Single-core performance matters most
- **RAM**: Affects caching behavior

Always run benchmarks on the same hardware for comparison.
