#!/bin/bash
# benchmarks.sh - Mesure les performances de compilation et tests

set -e

echo "🔥 otter Benchmarks"
echo "===================="
echo ""

echo "Compilation time..."
TIME_START=$(date +%s)
cargo build --workspace --release > /dev/null 2>&1
TIME_END=$(date +%s)
COMPILE_TIME=$((TIME_END - TIME_START))
echo " Compilation: ${COMPILE_TIME}s"
echo ""

echo " Binary sizes..."
du -sh target/release/otter 2>/dev/null || echo "No binary yet"
echo ""

echo "🧪 Test execution time..."
TIME_START=$(date +%s)
cargo test --workspace > /dev/null 2>&1
TIME_END=$(date +%s)
TEST_TIME=$((TIME_END - TIME_START))
echo " Tests: ${TEST_TIME}s"
echo ""

echo "📈 Test coverage..."
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --workspace --out Stdout --output-dir coverage/ 2>/dev/null | grep "coverage:"
else
    echo "cargo-tarpaulin not installed (optional)"
fi
echo ""

echo "===================="
echo " Summary:"
echo "  - Compile: ${COMPILE_TIME}s"
echo "  - Tests: ${TEST_TIME}s"
echo "  - Total: $((COMPILE_TIME + TEST_TIME))s"
echo ""
