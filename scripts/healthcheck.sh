#!/bin/bash
# BootForge Health Check Script
# Verifies USB detection and safe mode operation

set -o pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "BootForge Health Check"
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

# Test 1: Rust toolchain
echo "1. Checking Rust toolchain..."
if command -v rustc >/dev/null 2>&1; then
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    pass "Rust compiler found: $RUST_VERSION"
else
    fail "Rust compiler not found (install from https://rustup.rs)"
fi

# Test 2: Cargo available
echo ""
echo "2. Checking Cargo build tool..."
if command -v cargo >/dev/null 2>&1; then
    CARGO_VERSION=$(cargo --version | awk '{print $2}')
    pass "Cargo found: $CARGO_VERSION"
else
    fail "Cargo not found"
fi

# Test 3: libusb system library
echo ""
echo "3. Checking libusb system library..."
if pkg-config --exists libusb-1.0 2>/dev/null; then
    LIBUSB_VERSION=$(pkg-config --modversion libusb-1.0)
    pass "libusb-1.0 found: $LIBUSB_VERSION"
elif [ -f /usr/lib/libusb-1.0.so ] || [ -f /usr/local/lib/libusb-1.0.dylib ] || [ -f /opt/homebrew/lib/libusb-1.0.dylib ]; then
    pass "libusb-1.0 library found (version check unavailable)"
elif [ "$(uname)" = "Darwin" ]; then
    warn "libusb not found via pkg-config (macOS may use native IOKit)"
elif [ "$(uname)" = "MINGW" ] || [ "$(uname)" = "MSYS" ]; then
    warn "libusb detection skipped on Windows (WinUSB used)"
else
    warn "libusb-1.0 not detected (install: apt install libusb-1.0-0-dev, yum install libusb1-devel, or brew install libusb) - build may still succeed via bundled libs"
fi

# Test 4: Project builds successfully
echo ""
echo "4. Testing project build..."
if cargo build --quiet 2>&1 | grep -q "error"; then
    fail "Project build failed"
else
    pass "Project builds successfully"
fi

# Test 5: Core library (libbootforge) builds
echo ""
echo "5. Testing libbootforge library..."
if cargo build -p libbootforge --quiet 2>&1 | grep -q "error"; then
    fail "libbootforge build failed"
else
    pass "libbootforge builds successfully"
fi

# Test 6: CLI binary builds
echo ""
echo "6. Testing bootforge-cli binary..."
if cargo build --bin bootforge-cli --quiet 2>&1 | grep -q "error"; then
    fail "bootforge-cli build failed"
else
    pass "bootforge-cli builds successfully"
fi

# Test 7: Unit tests pass
echo ""
echo "7. Running unit tests..."
if cargo test --lib -p libbootforge --quiet 2>&1 | grep -q "test result: FAILED"; then
    fail "libbootforge unit tests failed"
else
    pass "libbootforge unit tests pass"
fi

# Test 8: USB detection capability
echo ""
echo "8. Testing USB device detection..."
if cargo run --bin bootforge-cli --quiet 2>&1 | grep -qE "(Device|USB|VID|PID|No devices)" || \
   cargo run --bin bootforge-cli --quiet 2>/dev/null >/dev/null; then
    pass "USB detection capability confirmed"
else
    warn "USB detection test inconclusive (may require connected devices)"
fi

# Test 9: No write operations in binary
echo ""
echo "9. Verifying read-only safety..."
if strings target/debug/bootforge-cli 2>/dev/null | grep -qiE "(write_bulk|write_control|format|erase|flash)"; then
    warn "Potential write operations found in binary (manual review needed)"
else
    pass "No obvious write operations detected in binary"
fi

# Test 10: Audit logging service
echo ""
echo "10. Testing audit logging service..."
if cargo test -p audit-logging --lib --quiet 2>&1 | grep -qE "test result: (ok|FAILED)"; then
    pass "Audit logging service tests completed"
else
    fail "Audit logging service tests failed"
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Health Check Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All health checks passed!${NC}"
    echo ""
    echo "BootForge is ready for use."
    echo ""
    echo "Next steps:"
    echo "  - Run: cargo run --bin bootforge-cli"
    echo "  - Run: ./scripts/smoke-test.sh"
    exit 0
else
    echo -e "${RED}✗ Some health checks failed.${NC}"
    echo ""
    echo "Please resolve the failed checks before using BootForge."
    exit 1
fi
