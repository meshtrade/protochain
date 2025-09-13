#!/bin/bash

# Protochain Unified Linting Script
# Runs all linters with auto-fix across TypeScript, Go, and Rust workspaces

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   Protochain Unified Linting Check${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Track if any linting failed
LINT_FAILED=0

# Function to run a linting script and check its status
run_lint_script() {
    local script_name=$1
    local script_path="${SCRIPT_DIR}/${script_name}"
    
    if [ -f "${script_path}" ]; then
        if bash "${script_path}"; then
            echo ""  # Add spacing after successful script
        else
            LINT_FAILED=1
            echo ""  # Add spacing after failed script
        fi
    else
        echo -e "${RED}✗ Linting script not found: ${script_path}${NC}"
        LINT_FAILED=1
    fi
}

# Run individual linting scripts
run_lint_script "ts.sh"
run_lint_script "go.sh" 
run_lint_script "rs.sh"

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