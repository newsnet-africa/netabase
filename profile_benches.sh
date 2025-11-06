#!/bin/bash
# Profiling script for netabase benchmarks
# This script runs benchmarks with profiling enabled and opens the flamegraphs

set -e

echo "==================== Netabase Profiling ===================="
echo ""
echo "Running benchmarks with profiling enabled..."
echo ""

# Run benchmarks with profiling
cargo bench -- --profile-time=5

echo ""
echo "==================== Profiling Complete ===================="
echo ""
echo "Flamegraphs have been generated in:"
echo ""

# Find and list all flamegraphs
find target/criterion -name "flamegraph.svg" -type f | while read -r file; do
    size=$(du -h "$file" | cut -f1)
    echo "  ✓ $file ($size)"
done

echo ""
echo "To view a flamegraph:"
echo "  xdg-open target/criterion/<benchmark>/profile/flamegraph.svg"
echo ""
echo "Available flamegraphs by benchmark:"

# Group by benchmark
find target/criterion -name "flamegraph.svg" -type f | sed 's|target/criterion/||; s|/profile/flamegraph.svg||' | sort | uniq | while read -r bench; do
    echo "  • $bench"
done

echo ""
echo "Example commands to view specific flamegraphs:"
echo "  xdg-open target/criterion/netabase_creation/profile/flamegraph.svg"
echo "  xdg-open target/criterion/swarm_lifecycle/profile/flamegraph.svg"
echo "  xdg-open target/criterion/local_record_storage/100/profile/flamegraph.svg"
echo ""
