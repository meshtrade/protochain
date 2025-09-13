#!/bin/bash

# Protochain Rust Linting Script
# Uses cargo fmt and clippy with auto-fix

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}[Rust Linting]${NC}"

cd "${PROJECT_ROOT}"

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ cargo is required but not installed${NC}"
    echo -e "${RED}Please install Rust and cargo:${NC}"
    echo -e "${RED}  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    exit 1
fi

# Auto-fix formatting with rustfmt
echo -e "${YELLOW}Running rustfmt auto-fix...${NC}"
if cargo fmt --all; then
    echo -e "${GREEN}✓ Rust formatting applied${NC}"
else
    echo -e "${RED}✗ Rust formatting failed${NC}"
    exit 1
fi

# Run clippy with auto-fix for safe fixes
echo -e "${YELLOW}Running clippy with auto-fix...${NC}"
if cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged -- -D warnings; then
    echo -e "${GREEN}✓ Clippy passed${NC}"
else
    echo -e "${RED}✗ Clippy failed${NC}"
    exit 1
fi

echo -e "${GREEN}✓ All Rust linting passed${NC}"