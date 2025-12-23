#!/bin/sh
# Benchmark suite comparing NLP vs traditional commands
# Target: NLP should be <20% slower than traditional commands

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

BENCH_CATEGORY="nlp_bench"

echo "========================================"
echo "NLP vs Traditional Command Benchmarks"
echo "========================================"
echo

# Check if hyperfine is installed
if ! command -v hyperfine >/dev/null 2>&1; then
    echo "Error: hyperfine is not installed"
    echo "Install it with: cargo install hyperfine"
    exit 1
fi

# Cleanup function
cleanup() {
    # Clean up benchmark tasks/records
    for i in $(seq 1 100); do
        echo "$i" | tascli delete "$i" >/dev/null 2>&1 || true
    done
}

# Initial cleanup
cleanup

echo "Benchmark 1: Task Creation"
echo "----------------------------"

# Traditional command
hyperfine -r 50 --warmup 3 \
    'tascli task -c nlp_bench "traditional task benchmark"' \
    -n "Traditional: Task Creation"

# NLP command (with API call)
hyperfine -r 50 --warmup 3 \
    'tascli nlp "add task nlp benchmark"' \
    -n "NLP: Task Creation (with API)"

# NLP command with cache hit (second run)
hyperfine -r 50 --warmup 3 \
    'tascli nlp "add task nlp benchmark"' \
    -n "NLP: Task Creation (cached)"

echo
echo "Benchmark 2: Record Creation"
echo "----------------------------"

hyperfine -r 50 --warmup 3 \
    'tascli record -c nlp_bench "traditional record benchmark"' \
    -n "Traditional: Record Creation"

hyperfine -r 50 --warmup 3 \
    'tascli nlp "add record nlp benchmark"' \
    -n "NLP: Record Creation (with API)"

hyperfine -r 50 --warmup 3 \
    'tascli nlp "add record nlp benchmark"' \
    -n "NLP: Record Creation (cached)"

echo
echo "Benchmark 3: List Tasks"
echo "----------------------------"

# Create some tasks for listing
for i in $(seq 1 10); do
    tascli task -c nlp_bench "list benchmark task $i" >/dev/null 2>&1 || true
done

hyperfine -r 50 --warmup 3 \
    'tascli list task -c nlp_bench' \
    -n "Traditional: List Tasks"

hyperfine -r 50 --warmup 3 \
    'tascli nlp "show tasks in nlp_bench"' \
    -n "NLP: List Tasks (with API)"

hyperfine -r 50 --warmup 3 \
    'tascli nlp "show tasks in nlp_bench"' \
    -n "NLP: List Tasks (cached)"

echo
echo "Benchmark 4: Task Completion"
echo "----------------------------"

hyperfine -r 50 --warmup 3 \
    'tascli done 1' \
    -n "Traditional: Complete Task"

hyperfine -r 50 --warmup 3 \
    'tascli nlp "complete task 1"' \
    -n "NLP: Complete Task (with API)"

hyperfine -r 50 --warmup 3 \
    'tascli nlp "complete task 1"' \
    -n "NLP: Complete Task (cached)"

echo
echo "Benchmark 5: Task Deletion"
echo "----------------------------"

hyperfine -r 50 --warmup 3 \
    'echo "yes" | tascli delete 2' \
    -n "Traditional: Delete Task"

hyperfine -r 50 --warmup 3 \
    'echo "yes" | tascli nlp "delete task 2"' \
    -n "NLP: Delete Task (with API)"

hyperfine -r 50 --warmup 3 \
    'echo "yes" | tascli nlp "delete task 2"' \
    -n "NLP: Delete Task (cached)"

echo
echo "Benchmark 6: Complex Queries"
echo "----------------------------"

# List with filters
hyperfine -r 30 --warmup 3 \
    'tascli list task -c nlp_bench --limit 5' \
    -n "Traditional: List with limit"

hyperfine -r 30 --warmup 3 \
    'tascli nlp "show 5 tasks in nlp_bench"' \
    -n "NLP: List with limit (with API)"

hyperfine -r 30 --warmup 3 \
    'tascli nlp "show 5 tasks in nlp_bench"' \
    -n "NLP: List with limit (cached)"

echo
echo "========================================"
echo "Benchmark Complete"
echo "========================================"
echo
echo "To analyze performance impact:"
echo "1. Compare 'Traditional' vs 'NLP (cached)' results"
echo "2. NLP with cache should be <20% slower than traditional"
echo "3. NLP with API will be slower (network overhead expected)"
echo

# Cleanup
cleanup
