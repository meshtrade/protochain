#!/bin/bash

# ProtoSol TypeScript Linting Script
# Uses yarn exclusively and configures linters to skip generated files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TS_DIR="${PROJECT_ROOT}/lib/ts"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}[TypeScript Linting]${NC}"

# Check if TypeScript workspace exists
if [ ! -d "${TS_DIR}" ]; then
    echo -e "${YELLOW}TypeScript workspace not found, skipping...${NC}"
    exit 0
fi

cd "${TS_DIR}"

# Ensure yarn is available
if ! command -v yarn &> /dev/null; then
    echo -e "${RED}✗ yarn is required but not installed${NC}"
    echo -e "${RED}This is a yarn project. Please install yarn:${NC}"
    echo -e "${RED}  npm install -g yarn${NC}"
    exit 1
fi

# Ensure dependencies are installed
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}Installing TypeScript dependencies...${NC}"
    yarn install
fi

# Run ESLint with auto-fix (configured to skip generated files)
echo -e "${YELLOW}Running ESLint with auto-fix...${NC}"
# Check if there are any non-generated .ts/.tsx files to lint
if find src -name "*.ts" -o -name "*.tsx" | grep -v "src/protosol/" | head -1 > /dev/null 2>&1; then
    if yarn lint:fix; then
        echo -e "${GREEN}✓ ESLint passed${NC}"
    else
        echo -e "${RED}✗ ESLint failed${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}No non-generated TypeScript files found, skipping ESLint${NC}"
fi

# Run Prettier with auto-fix
echo -e "${YELLOW}Running Prettier with auto-fix...${NC}"
if yarn format; then
    echo -e "${GREEN}✓ Prettier passed${NC}"
else
    echo -e "${RED}✗ Prettier failed${NC}"
    exit 1
fi

# Run TypeScript type checking
echo -e "${YELLOW}Running TypeScript type checking...${NC}"
if yarn typecheck; then
    echo -e "${GREEN}✓ TypeScript type checking passed${NC}"
else
    echo -e "${RED}✗ TypeScript type checking failed${NC}"
    exit 1
fi

echo -e "${GREEN}✓ All TypeScript linting passed${NC}"