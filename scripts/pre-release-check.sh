#!/bin/bash
# Pre-release validation script
# Runs all automated checks from the release checklist

set -e  # Exit on any error

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Track overall status
CHECKS_PASSED=0
CHECKS_TOTAL=5

print_step() {
    echo -e "\n${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}!${NC} $1"
}

# Get project root (assuming script is in scripts/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Pipeline Kit - Pre-Release Check${NC}"
echo -e "${BLUE}========================================${NC}"

# 1. Code Formatting
print_step "Checking code formatting..."
cd pipeline-kit-rs
if cargo fmt --all --check; then
    print_success "Code formatting check passed"
else
    print_error "Code formatting check failed. Run 'cargo fmt --all' to fix."
    exit 1
fi
cd ..

# 2. Rust Tests
print_step "Running Rust tests..."
cd pipeline-kit-rs
if cargo test --workspace --quiet; then
    print_success "All Rust tests passed"
else
    print_error "Rust tests failed"
    exit 1
fi
cd ..

# 3. Clippy
print_step "Running Clippy linter..."
cd pipeline-kit-rs
if cargo clippy --all-targets --quiet -- -D warnings; then
    print_success "No Clippy warnings"
else
    print_error "Clippy found issues"
    exit 1
fi
cd ..

# 4. Release Build
print_step "Building release binary..."
cd pipeline-kit-rs
if cargo build --release --quiet; then
    print_success "Release build successful"
else
    print_error "Release build failed"
    exit 1
fi
cd ..

# 5. TypeScript Tests
print_step "Running TypeScript tests..."
cd pipeline-kit-cli
if [ -f "package.json" ]; then
    # Check if npm or pnpm is available
    if command -v pnpm &> /dev/null; then
        if pnpm test; then
            print_success "TypeScript tests passed"
        else
            print_error "TypeScript tests failed"
            exit 1
        fi
    elif command -v npm &> /dev/null; then
        if npm test; then
            print_success "TypeScript tests passed"
        else
            print_error "TypeScript tests failed"
            exit 1
        fi
    else
        print_warning "Neither npm nor pnpm found, skipping TypeScript tests"
        ((CHECKS_TOTAL--))
    fi
else
    print_warning "package.json not found, skipping TypeScript tests"
    ((CHECKS_TOTAL--))
fi
cd ..

# Summary
echo -e "\n${BLUE}========================================${NC}"
echo -e "${GREEN}Automated Checks: ${CHECKS_PASSED}/${CHECKS_TOTAL} passed${NC}"
echo -e "${BLUE}========================================${NC}"

# Manual checks reminder
echo -e "\n${YELLOW}Manual checks required:${NC}"
echo "  [ ] CHANGELOG.md updated with new version"
echo "  [ ] README.md version information verified"
echo ""

if [ $CHECKS_PASSED -eq $CHECKS_TOTAL ]; then
    echo -e "${GREEN}✓ All automated checks passed!${NC}"
    echo -e "Please verify the manual checks above before releasing."
    exit 0
else
    echo -e "${RED}✗ Some checks failed. Please fix the issues before releasing.${NC}"
    exit 1
fi
