#!/bin/bash

# ProtoSol Go Linting Script
# Requires golangci-lint to be pre-installed

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}[Go Linting]${NC}"

cd "${PROJECT_ROOT}"

# Check if golangci-lint is available
if ! command -v golangci-lint &> /dev/null; then
    echo -e "${RED}✗ golangci-lint is required but not installed${NC}"
    echo -e "${RED}Please install golangci-lint:${NC}"
    echo -e "${RED}  go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest${NC}"
    echo -e "${RED}  export PATH=\"\$(go env GOPATH)/bin:\$PATH\"${NC}"
    exit 1
fi

# Check if goimports is available
if ! command -v goimports &> /dev/null; then
    echo -e "${RED}✗ goimports is required but not installed${NC}"
    echo -e "${RED}Please install goimports:${NC}"
    echo -e "${RED}  go install golang.org/x/tools/cmd/goimports@latest${NC}"
    exit 1
fi

# Run golangci-lint with auto-fix on specific directories
echo -e "${YELLOW}Running golangci-lint with auto-fix...${NC}"
if golangci-lint run --config .golangci.yml --fix tests/go/... tool/...; then
    echo -e "${GREEN}✓ golangci-lint passed${NC}"
else
    echo -e "${RED}✗ golangci-lint failed${NC}"
    exit 1
fi

# Run gofmt to check formatting (gofmt doesn't have --fix, but goimports does)
echo -e "${YELLOW}Running goimports for formatting...${NC}"
GOIMPORTS_FILES=$(find tests/go tool -type f -name '*.go' -not -name '*.pb.go' -not -name '*.passivgo.go')
if [ -n "$GOIMPORTS_FILES" ]; then
    # Apply goimports formatting
    echo "$GOIMPORTS_FILES" | xargs goimports -w
    
    # Check if gofmt is satisfied after goimports
    GOFMT_FILES=$(echo "$GOIMPORTS_FILES" | xargs gofmt -l)
    if [ -z "$GOFMT_FILES" ]; then
        echo -e "${GREEN}✓ Go formatting passed${NC}"
    else
        echo -e "${RED}✗ Go formatting failed. Files need formatting:${NC}"
        echo "$GOFMT_FILES"
        exit 1
    fi
else
    echo -e "${YELLOW}No Go files to format${NC}"
fi

echo -e "${GREEN}✓ All Go linting passed${NC}"