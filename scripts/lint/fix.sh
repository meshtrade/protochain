#!/bin/bash

# ProtoSol Auto-Fix Linting Script
# Automatically fixes linting issues across TypeScript, Go, and Rust workspaces

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
echo -e "${BLUE}   ProtoSol Auto-Fix Linting${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# TypeScript Auto-Fix
echo -e "${BLUE}[TypeScript Auto-Fix]${NC}"
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
    
    # Run ESLint fix
    echo -e "${YELLOW}Running ESLint auto-fix...${NC}"
    npm run lint:fix
    echo -e "${GREEN}✓ ESLint auto-fix complete${NC}"
    
    # Run Prettier format
    echo -e "${YELLOW}Running Prettier format...${NC}"
    npm run format
    echo -e "${GREEN}✓ Prettier format complete${NC}"
else
    echo -e "${YELLOW}TypeScript workspace not found, skipping...${NC}"
fi
echo ""

# Go Auto-Fix
echo -e "${BLUE}[Go Auto-Fix]${NC}"
cd "${PROJECT_ROOT}"

# Install tools if not present
if ! command -v golangci-lint &> /dev/null; then
    echo "Installing golangci-lint..."
    go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
    export PATH="$(go env GOPATH)/bin:$PATH"
fi

if ! command -v goimports &> /dev/null; then
    echo "Installing goimports..."
    go install golang.org/x/tools/cmd/goimports@latest
fi

# Run gofmt
echo -e "${YELLOW}Running gofmt...${NC}"
find . -type f -name '*.go' \
    -not -path './lib/go/protosol/*' \
    -not -path './vendor/*' \
    -not -name '*.pb.go' \
    -not -name '*.passivgo.go' \
    -exec gofmt -w {} \;
echo -e "${GREEN}✓ Go formatting complete${NC}"

# Run goimports
echo -e "${YELLOW}Running goimports...${NC}"
find . -type f -name '*.go' \
    -not -path './lib/go/protosol/*' \
    -not -path './vendor/*' \
    -not -name '*.pb.go' \
    -not -name '*.passivgo.go' \
    -exec goimports -w -local github.com/BRBussy/protosol {} \;
echo -e "${GREEN}✓ Go imports organized${NC}"

# Run golangci-lint with fix
echo -e "${YELLOW}Running golangci-lint auto-fix...${NC}"
golangci-lint run --fix --config .golangci.yml ./... 2>&1 || true
echo -e "${GREEN}✓ golangci-lint auto-fix complete${NC}"
echo ""

# Rust Auto-Fix
echo -e "${BLUE}[Rust Auto-Fix]${NC}"
cd "${PROJECT_ROOT}"

if command -v cargo &> /dev/null; then
    # Run rustfmt
    echo -e "${YELLOW}Running rustfmt...${NC}"
    cargo fmt --all
    echo -e "${GREEN}✓ Rust formatting complete${NC}"
    
    # Run clippy fix (where possible)
    echo -e "${YELLOW}Running clippy auto-fix...${NC}"
    cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged 2>&1 || true
    echo -e "${GREEN}✓ Clippy auto-fix complete${NC}"
else
    echo -e "${YELLOW}Cargo not found, skipping Rust auto-fix...${NC}"
fi
echo ""

# Final message
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}✓ Auto-fix complete!${NC}"
echo -e "${YELLOW}Note: Some issues may require manual intervention.${NC}"
echo -e "${YELLOW}Run './scripts/lint/all.sh' to check remaining issues.${NC}"
echo -e "${BLUE}========================================${NC}"