#!/bin/bash
set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Verifying TypeScript type generation..."
echo ""

# 1. Generate TypeScript types from Rust
echo -e "${YELLOW}1/2 Generating TypeScript types from Rust...${NC}"
cd pipeline-kit-rs
if cargo test --package pk-protocol --lib -- --nocapture 2>&1 | grep -q "TypeScript bindings generated"; then
    echo -e "${GREEN}✅ TypeScript types generated successfully${NC}"
else
    # Run the test and check if it passes (it generates types as a side effect)
    if cargo test --package pk-protocol --lib -- --nocapture > /dev/null 2>&1; then
        echo -e "${GREEN}✅ TypeScript types generated successfully${NC}"
    else
        echo -e "${RED}❌ TypeScript type generation failed${NC}"
        exit 1
    fi
fi
echo ""

# 2. Verify TypeScript type checking
echo -e "${YELLOW}2/2 Running TypeScript type check...${NC}"
cd ../pipeline-kit-cli
if pnpm type-check; then
    echo -e "${GREEN}✅ TypeScript type check passed${NC}"
else
    echo -e "${RED}❌ TypeScript type check failed${NC}"
    exit 1
fi
echo ""

echo -e "${GREEN}🎉 TypeScript type verification passed!${NC}"
