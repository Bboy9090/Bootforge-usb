#!/bin/bash
# BootForge Smoke Test Script
# Runs quick sanity checks on builds and entrypoints

set -o pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "BootForge Smoke Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Change to project root
cd "$PROJECT_ROOT"

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((TESTS_PASSED++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((TESTS_FAILED++))
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Test 1: Workspace builds
echo "1. Building workspace (debug)..."
if cargo build; then
    pass "Workspace builds successfully"
else
    fail "Workspace build failed"
fi

# Test 2: Release build
echo ""
echo "2. Building workspace (release)..."
if cargo build --release; then
    pass "Release build successful"
else
    fail "Release build failed"
fi

# Test 3: libbootforge library
echo ""
echo "3. Testing libbootforge library..."
if cargo build -p libbootforge --lib; then
    pass "libbootforge library builds"
else
    fail "libbootforge library build failed"
fi

# Test 4: bootforge-cli binary
echo ""
echo "4. Testing bootforge-cli binary..."
if cargo build --bin bootforge-cli; then
    pass "bootforge-cli binary builds"
else
    fail "bootforge-cli binary build failed"
fi

# Test 5: Examples compile
echo ""
echo "5. Testing examples..."
EXAMPLES_PASSED=0
EXAMPLES_TOTAL=0

for example in libbootforge/examples/*.rs; do
    if [ -f "$example" ]; then
        EXAMPLE_NAME=$(basename "$example" .rs)
        ((EXAMPLES_TOTAL++))

        if cargo build --example "$EXAMPLE_NAME"; then
            ((EXAMPLES_PASSED++))
        else
            fail "Example '$EXAMPLE_NAME' failed to build"
        fi
    fi
done

if [ $EXAMPLES_TOTAL -eq 0 ]; then
    warn "No examples found"
elif [ $EXAMPLES_PASSED -eq $EXAMPLES_TOTAL ]; then
    pass "All $EXAMPLES_TOTAL examples build successfully"
else
    fail "Only $EXAMPLES_PASSED/$EXAMPLES_TOTAL examples built"
fi

# Test 6: Unit tests
echo ""
echo "6. Running unit tests..."
if cargo test --lib --workspace --quiet; then
    pass "Unit tests pass"
else
    fail "Unit tests failed"
fi

# Test 7: Services build
echo ""
echo "7. Testing ForgeWorks services..."
SERVICES=("device-analysis" "ownership-verification" "legal-classification" "audit-logging" "authority-routing" "auth" "metrics")
SERVICES_PASSED=0

for service in "${SERVICES[@]}"; do
    if cargo build -p "$service" --quiet; then
        ((SERVICES_PASSED++))
    else
        fail "Service '$service' build failed"
    fi
done

if [ $SERVICES_PASSED -eq ${#SERVICES[@]} ]; then
    pass "All ${#SERVICES[@]} services build successfully"
else
    fail "Only $SERVICES_PASSED/${#SERVICES[@]} services built"
fi

# Test 8: CLI entrypoint
echo ""
echo "8. Testing CLI entrypoint..."
CLI_OUTPUT=$(cargo run --bin bootforge-cli -- --help 2>&1)
CLI_STATUS=$?
if [ $CLI_STATUS -eq 0 ] && echo "$CLI_OUTPUT" | grep -qiE "(bootforge|usage|options|--help)"; then
    pass "CLI entrypoint works (--help flag)"
else
    fail "CLI entrypoint failed"
fi

# Test 9: Code formatting
echo ""
echo "9. Checking code formatting..."
if cargo fmt --check; then
    pass "Code is properly formatted"
else
    fail "Code formatting check failed (run: cargo fmt)"
fi

# Test 10: Clippy lints (warnings allowed for smoke test)
echo ""
echo "10. Running clippy lints..."
if cargo clippy --workspace --all-targets --all-features -- -D warnings; then
    pass "Clippy checks pass with warnings denied"
else
    fail "Clippy found errors"
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Smoke Test Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All smoke tests passed!${NC}"
    echo ""
    echo "BootForge is ready for deployment."
    echo ""
    echo "Next steps:"
    echo "  - Run: cargo run --bin bootforge-cli"
    echo "  - Review: docs/RELEASE_CHECKLIST.md"
    exit 0
else
    echo -e "${RED}✗ Some smoke tests failed.${NC}"
    echo ""
    echo "Please fix the issues before deploying."
    exit 1
fi
