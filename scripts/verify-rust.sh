#!/bin/bash
set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Running Rust verification checks..."
echo ""

cd pipeline-kit-rs

# 1. Code Formatting
echo -e "${YELLOW}1/5 Checking code formatting...${NC}"
if cargo fmt --all --check; then
    echo -e "${GREEN}✅ Code formatting check passed${NC}"
else
    echo -e "${RED}❌ Code formatting check failed${NC}"
    echo "Run 'cargo fmt --all' to fix formatting issues"
    exit 1
fi
echo ""

# 2. Static Analysis (Clippy)
echo -e "${YELLOW}2/5 Running static analysis (clippy)...${NC}"
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo -e "${GREEN}✅ Clippy analysis passed (0 warnings)${NC}"
else
    echo -e "${RED}❌ Clippy analysis failed${NC}"
    exit 1
fi
echo ""

# 3. Build Verification
echo -e "${YELLOW}3/5 Building all targets...${NC}"
if cargo build --all-targets --all-features; then
    echo -e "${GREEN}✅ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
echo ""

# 4. Test Execution
echo -e "${YELLOW}4/5 Running tests...${NC}"
if command -v cargo-nextest &> /dev/null; then
    if cargo nextest run --all-features; then
        echo -e "${GREEN}✅ All tests passed (cargo nextest)${NC}"
    else
        echo -e "${RED}❌ Tests failed${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  cargo-nextest not found, falling back to cargo test${NC}"
    if cargo test --all-targets --all-features; then
        echo -e "${GREEN}✅ All tests passed (cargo test)${NC}"
    else
        echo -e "${RED}❌ Tests failed${NC}"
        exit 1
    fi
fi
echo ""

# 5. Documentation Check
echo -e "${YELLOW}5/5 Building documentation...${NC}"
if cargo doc --no-deps --all-features --quiet; then
    echo -e "${GREEN}✅ Documentation built without errors${NC}"
else
    echo -e "${RED}❌ Documentation build failed${NC}"
    exit 1
fi
echo ""

echo -e "${GREEN}🎉 All Rust verification checks passed!${NC}"
