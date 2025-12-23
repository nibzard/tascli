#!/bin/sh
# Benchmark startup time impact

echo "========================================"
echo "Startup Time Benchmarks"
echo "========================================"
echo

# Check if hyperfine is installed
if ! command -v hyperfine >/dev/null 2>&1; then
    echo "Error: hyperfine is not installed"
    echo "Install it with: cargo install hyperfine"
    exit 1
fi

echo "Benchmark 1: Cold Start - Traditional Command"
echo "----------------------------------------------"
hyperfine -r 100 --warmup 5 \
    'tascli --no-nlp task "startup test" || true' \
    -n "Cold Start: Traditional"

echo
echo "Benchmark 2: Cold Start - NLP Command"
echo "----------------------------------------------"
hyperfine -r 100 --warmup 5 \
    'tascli nlp "add task startup test" || true' \
    -n "Cold Start: NLP"

echo
echo "Benchmark 3: Argument Parsing Only"
echo "----------------------------------------------"
hyperfine -r 100 --warmup 5 \
    'tascli task --help >/dev/null' \
    -n "Parse Traditional Args"

hyperfine -r 100 --warmup 5 \
    'tascli nlp --help >/dev/null' \
    -n "Parse NLP Args"

echo
echo "Benchmark 4: No-op Invocation"
echo "----------------------------------------------"
hyperfine -r 100 --warmup 5 \
    'tascli --no-nlp' \
    -n "No-op: Traditional mode"

hyperfine -r 100 --warmup 5 \
    'tascli' \
    -n "No-op: NLP mode"

echo
echo "========================================"
echo "Startup Benchmark Complete"
echo "========================================"
