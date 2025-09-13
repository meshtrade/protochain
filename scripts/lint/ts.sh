#!/bin/bash

# Protochain TypeScript Linting Script
# Uses yarn exclusively and configures linters to skip generated files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Define TypeScript workspaces to check
TS_WORKSPACES=(
    "${PROJECT_ROOT}/lib/ts"
    "${PROJECT_ROOT}/tests/ts"
)

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}[TypeScript Linting]${NC}"

# Ensure yarn is available
if ! command -v yarn &> /dev/null; then
    echo -e "${RED}✗ yarn is required but not installed${NC}"
    echo -e "${RED}This is a yarn project. Please install yarn:${NC}"
    echo -e "${RED}  npm install -g yarn${NC}"
    exit 1
fi

# Overall tracking
OVERALL_SUCCESS=true

# Process each TypeScript workspace
for TS_DIR in "${TS_WORKSPACES[@]}"; do
    WORKSPACE_NAME=$(basename "${TS_DIR}")
    echo -e "${BLUE}[TypeScript - ${WORKSPACE_NAME}]${NC}"
    
    # Check if workspace exists
    if [ ! -d "${TS_DIR}" ]; then
        echo -e "${YELLOW}${WORKSPACE_NAME} workspace not found, skipping...${NC}"
        continue
    fi

    cd "${TS_DIR}"

    # Ensure dependencies are installed
    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}Installing ${WORKSPACE_NAME} dependencies...${NC}"
        yarn install
    fi

    # Run ESLint with auto-fix (skip generated files for lib/ts only)
    echo -e "${YELLOW}Running ESLint with auto-fix on ${WORKSPACE_NAME}...${NC}"
    
    if [ "${WORKSPACE_NAME}" = "ts" ]; then
        # lib/ts - skip generated protochain files
        if find src -name "*.ts" -o -name "*.tsx" | grep -v "src/protochain/" | head -1 > /dev/null 2>&1; then
            if yarn lint:fix; then
                echo -e "${GREEN}✓ ESLint passed for ${WORKSPACE_NAME}${NC}"
            else
                echo -e "${RED}✗ ESLint failed for ${WORKSPACE_NAME}${NC}"
                OVERALL_SUCCESS=false
            fi
        else
            echo -e "${YELLOW}No non-generated TypeScript files found in ${WORKSPACE_NAME}, skipping ESLint${NC}"
        fi
    else
        # tests/ts and other workspaces - lint all files 
        if find . -name "*.ts" -o -name "*.tsx" | grep -v "node_modules" | head -1 > /dev/null 2>&1; then
            if yarn lint 2>/dev/null || yarn typecheck; then
                echo -e "${GREEN}✓ TypeScript check passed for ${WORKSPACE_NAME}${NC}"
            else
                echo -e "${RED}✗ TypeScript check failed for ${WORKSPACE_NAME}${NC}"
                OVERALL_SUCCESS=false
            fi
        else
            echo -e "${YELLOW}No TypeScript files found in ${WORKSPACE_NAME}, skipping${NC}"
        fi
    fi

    # Run Prettier with auto-fix (only for lib/ts which has the script)
    if [ "${WORKSPACE_NAME}" = "ts" ]; then
        echo -e "${YELLOW}Running Prettier with auto-fix on ${WORKSPACE_NAME}...${NC}"
        if yarn format; then
            echo -e "${GREEN}✓ Prettier passed for ${WORKSPACE_NAME}${NC}"
        else
            echo -e "${RED}✗ Prettier failed for ${WORKSPACE_NAME}${NC}"
            OVERALL_SUCCESS=false
        fi
    fi

    # Run TypeScript type checking
    echo -e "${YELLOW}Running TypeScript type checking on ${WORKSPACE_NAME}...${NC}"
    if yarn typecheck; then
        echo -e "${GREEN}✓ TypeScript type checking passed for ${WORKSPACE_NAME}${NC}"
    else
        echo -e "${RED}✗ TypeScript type checking failed for ${WORKSPACE_NAME}${NC}"
        OVERALL_SUCCESS=false
    fi

    echo ""
done

if [ "$OVERALL_SUCCESS" = true ]; then
    echo -e "${GREEN}✓ All TypeScript linting passed${NC}"
else
    echo -e "${RED}✗ Some TypeScript linting checks failed${NC}"
    exit 1
fi