#!/bin/bash

# ProtoSol Unified Linting Script
# Runs all linters across TypeScript, Go, and Rust workspaces

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   ProtoSol Unified Linting Check${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Track if any linting failed
LINT_FAILED=0

# Function to run a command and check its status
run_lint() {
    local name=$1
    local command=$2
    
    echo -e "${YELLOW}Running ${name} linting...${NC}"
    if eval "${command}"; then
        echo -e "${GREEN}✓ ${name} linting passed${NC}"
    else
        echo -e "${RED}✗ ${name} linting failed${NC}"
        LINT_FAILED=1
    fi
    echo ""
}

# TypeScript Linting
echo -e "${BLUE}[TypeScript Linting]${NC}"
if [ -d "${PROJECT_ROOT}/lib/ts" ]; then
    cd "${PROJECT_ROOT}/lib/ts"
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        echo "Installing TypeScript dependencies..."
        if command -v yarn &> /dev/null; then
            yarn install
        else
            npm install
        fi
    fi
    
    # Check if there are non-generated TypeScript files to lint
    TS_FILES=$(find src -name '*.ts' -o -name '*.tsx' | grep -v 'src/protosol' | head -1)
    if [ -n "$TS_FILES" ]; then
        # Run ESLint
        run_lint "ESLint" "npm run lint 2>&1"
    else
        echo -e "${YELLOW}No non-generated TypeScript files found, skipping ESLint${NC}"
    fi
    
    # Run Prettier check
    run_lint "Prettier" "npm run format:check 2>&1"
    
    # Run TypeScript type checking
    run_lint "TypeScript" "npm run typecheck 2>&1"
else
    echo -e "${YELLOW}TypeScript workspace not found, skipping...${NC}"
fi
echo ""

# Go Linting
echo -e "${BLUE}[Go Linting]${NC}"
cd "${PROJECT_ROOT}"

# Install golangci-lint if not present
if ! command -v golangci-lint &> /dev/null; then
    echo "Installing golangci-lint..."
    go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
    
    # Add GOPATH/bin to PATH if not already there
    export PATH="$(go env GOPATH)/bin:$PATH"
fi

# Install goimports if not present
if ! command -v goimports &> /dev/null; then
    echo "Installing goimports..."
    go install golang.org/x/tools/cmd/goimports@latest
fi

# Run golangci-lint on specific directories that have Go files
run_lint "golangci-lint" "golangci-lint run --config .golangci.yml tests/go/... tool/... 2>&1"

# Check Go formatting
echo -e "${YELLOW}Checking Go formatting...${NC}"
GOFMT_FILES=$(find tests/go tool -type f -name '*.go' -not -name '*.pb.go' -not -name '*.passivgo.go' -exec gofmt -l {} \; 2>/dev/null)
if [ -z "$GOFMT_FILES" ]; then
    echo -e "${GREEN}✓ Go formatting check passed${NC}"
else
    echo -e "${RED}✗ Go formatting check failed. Files need formatting:${NC}"
    echo "$GOFMT_FILES"
    LINT_FAILED=1
fi
echo ""

# Rust Linting
echo -e "${BLUE}[Rust Linting]${NC}"
cd "${PROJECT_ROOT}"

# Check if cargo is available
if command -v cargo &> /dev/null; then
    # Run rustfmt check
    echo -e "${YELLOW}Running rustfmt check...${NC}"
    if cargo fmt --all -- --check 2>&1; then
        echo -e "${GREEN}✓ Rust formatting check passed${NC}"
    else
        echo -e "${RED}✗ Rust formatting check failed${NC}"
        echo "Run 'cargo fmt --all' to fix formatting issues"
        LINT_FAILED=1
    fi
    echo ""
    
    # Run clippy
    echo -e "${YELLOW}Running clippy...${NC}"
    if cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1; then
        echo -e "${GREEN}✓ Clippy check passed${NC}"
    else
        echo -e "${RED}✗ Clippy check failed${NC}"
        LINT_FAILED=1
    fi
else
    echo -e "${YELLOW}Cargo not found, skipping Rust linting...${NC}"
fi
echo ""

# Final report
echo -e "${BLUE}========================================${NC}"
if [ $LINT_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All linting checks passed!${NC}"
    echo -e "${BLUE}========================================${NC}"
    exit 0
else
    echo -e "${RED}✗ Some linting checks failed!${NC}"
    echo -e "${BLUE}========================================${NC}"
    exit 1
fi