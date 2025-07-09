#!/bin/bash

# Performance benchmarking script for hsipc
# 
# This script runs comprehensive benchmarks to validate the performance claims:
# - 750k+ messages/second throughput
# - Sub-millisecond latency for small messages
# - Zero-copy for large payloads
# - Efficient topic-based routing

set -e

echo "🚀 Starting hsipc performance benchmarks..."
echo "📊 This will validate the performance claims made in README.md"
echo ""

# Create benchmark results directory
RESULTS_DIR="benchmark_results"
mkdir -p "$RESULTS_DIR"

# Function to run a benchmark suite
run_benchmark() {
    local bench_name="$1"
    local description="$2"
    
    echo "🧪 Running $description..."
    echo "📂 Results will be saved to: $RESULTS_DIR/${bench_name}_results.html"
    
    cd hsipc
    cargo bench --bench "$bench_name"
    
    # Move HTML report to results directory
    if [ -f "target/criterion/reports/index.html" ]; then
        cp "target/criterion/reports/index.html" "../$RESULTS_DIR/${bench_name}_results.html"
    fi
    
    cd ..
    echo "✅ $description completed"
    echo ""
}

# Run all benchmark suites
echo "🏁 Running benchmark suites..."
echo ""

# 1. Core performance benchmarks
run_benchmark "simple_benchmarks" "Core hsipc performance benchmarks"

echo "🎉 All benchmarks completed!"
echo ""
echo "📈 Performance Results Summary:"
echo "- Core hsipc performance: Check $RESULTS_DIR/simple_benchmarks_results.html"
echo ""
echo "🎯 Performance Claims Validation:"
echo "- Message throughput: Check simple_benchmarks 'message_throughput'"
echo "- Hub operations: Check simple_benchmarks 'hub_operations'"
echo "- High-frequency operations: Check simple_benchmarks 'high_frequency'"
echo "- Concurrent operations: Check simple_benchmarks 'concurrent_operations'"
echo "- Latency measurements: Check simple_benchmarks 'latency'"
echo ""
echo "📋 To view detailed results, open the HTML files in your browser:"
echo "  firefox $RESULTS_DIR/simple_benchmarks_results.html"