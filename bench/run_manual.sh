#!/bin/sh
# Manual benchmark suite for NLP vs Traditional performance
# Uses shell built-in time command instead of hyperfine

set -e

BENCH_CATEGORY="nlp_bench"

echo "========================================"
echo "NLP Performance Benchmarks (Manual)"
echo "========================================"
echo

# Cleanup function
cleanup() {
    for i in $(seq 1 100); do
        echo "$i" | tascli delete "$i" >/dev/null 2>&1 || true
    done
}

cleanup

echo "1. Task Creation (50 iterations)"
echo "---------------------------------"
echo -n "Traditional: "
START=$(date +%s%N)
for i in $(seq 1 50); do
    tascli task -c nlp_bench "traditional bench $i" >/dev/null 2>&1 || true
done
END=$(date +%s%N)
TRADITIONAL_MS=$(( (END - START) / 1000000 ))
echo "${TRADITIONAL_MS}ms total, $((TRADITIONAL_MS / 50))ms avg"

cleanup

echo -n "NLP (with API): "
START=$(date +%s%N)
for i in $(seq 1 10); do
    tascli nlp "add task nlp bench $i" >/dev/null 2>&1 || true
done
END=$(date +%s%N)
NLP_API_MS=$(( (END - START) / 1000000 ))
echo "${NLP_API_MS}ms total, $((NLP_API_MS / 10))ms avg"

echo -n "NLP (cached): "
START=$(date +%s%N)
for i in $(seq 1 50); do
    tascli nlp "add task nlp bench $i" >/dev/null 2>&1 || true
done
END=$(date +%s%N)
NLP_CACHED_MS=$(( (END - START) / 1000000 ))
echo "${NLP_CACHED_MS}ms total, $((NLP_CACHED_MS / 50))ms avg"

echo
echo "2. List Tasks (50 iterations)"
echo "---------------------------------"

# Create test tasks
for i in $(seq 1 10); do
    tascli task -c nlp_bench "list test $i" >/dev/null 2>&1 || true
done

echo -n "Traditional: "
START=$(date +%s%N)
for i in $(seq 1 50); do
    tascli list task -c nlp_bench >/dev/null 2>&1 || true
done
END=$(date +%s%N)
TRAD_LIST_MS=$(( (END - START) / 1000000 ))
echo "${TRAD_LIST_MS}ms total, $((TRAD_LIST_MS / 50))ms avg"

echo -n "NLP (cached): "
START=$(date +%s%N)
for i in $(seq 1 50); do
    tascli nlp "show tasks in nlp_bench" >/dev/null 2>&1 || true
done
END=$(date +%s%N)
NLP_LIST_MS=$(( (END - START) / 1000000 ))
echo "${NLP_LIST_MS}ms total, $((NLP_LIST_MS / 50))ms avg"

echo
echo "3. Command Completion (20 iterations)"
echo "---------------------------------"

echo -n "Traditional: "
START=$(date +%s%N)
for i in $(seq 1 20); do
    echo "yes" | tascli done 1 >/dev/null 2>&1 || true
done
END=$(date +%s%N)
TRAD_DONE_MS=$(( (END - START) / 1000000 ))
echo "${TRAD_DONE_MS}ms total, $((TRAD_DONE_MS / 20))ms avg"

echo -n "NLP (cached): "
START=$(date +%s%N)
for i in $(seq 1 20); do
    echo "yes" | tascli nlp "complete task 1" >/dev/null 2>&1 || true
done
END=$(date +%s%N)
NLP_DONE_MS=$(( (END - START) / 1000000 ))
echo "${NLP_DONE_MS}ms total, $((NLP_DONE_MS / 20))ms avg"

echo
echo "4. Cold Start Latency (100 iterations)"
echo "---------------------------------"

echo -n "Traditional cold start: "
START=$(date +%s%N)
for i in $(seq 1 100); do
    tascli --no-nlp --version >/dev/null 2>&1 || true
done
END=$(date +%s%N)
COLD_TRAD_MS=$(( (END - START) / 1000000 ))
echo "${COLD_TRAD_MS}ms total, $((COLD_TRAD_MS / 100))ms avg"

echo -n "NLP cold start: "
START=$(date +%s%N)
for i in $(seq 1 100); do
    tascli --version >/dev/null 2>&1 || true
done
END=$(date +%s%N)
COLD_NLP_MS=$(( (END - START) / 1000000 ))
echo "${COLD_NLP_MS}ms total, $((COLD_NLP_MS / 100))ms avg"

echo
echo "========================================"
echo "Performance Summary"
echo "========================================"
echo

# Calculate performance impact
if [ $TRADITIONAL_MS -gt 0 ]; then
    TASK_IMPACT=$(( (NLP_CACHED_MS - TRADITIONAL_MS) * 100 / TRADITIONAL_MS ))
    echo "Task Creation Impact: ${TASK_IMPACT}% (NLP cached vs Traditional)"
fi

if [ $TRAD_LIST_MS -gt 0 ]; then
    LIST_IMPACT=$(( (NLP_LIST_MS - TRAD_LIST_MS) * 100 / TRAD_LIST_MS ))
    echo "List Tasks Impact: ${LIST_IMPACT}% (NLP cached vs Traditional)"
fi

if [ $TRAD_DONE_MS -gt 0 ]; then
    DONE_IMPACT=$(( (NLP_DONE_MS - TRAD_DONE_MS) * 100 / TRAD_DONE_MS ))
    echo "Task Completion Impact: ${DONE_IMPACT}% (NLP cached vs Traditional)"
fi

if [ $COLD_TRAD_MS -gt 0 ]; then
    STARTUP_IMPACT=$(( (COLD_NLP_MS - COLD_TRAD_MS) * 100 / COLD_TRAD_MS ))
    echo "Startup Overhead: ${STARTUP_IMPACT}% (NLP vs Traditional)"
fi

echo
echo "Target: <20% performance impact for cached NLP commands"
echo

cleanup
