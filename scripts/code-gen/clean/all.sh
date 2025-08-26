#!/usr/bin/env bash
set -Eeuo pipefail

# Script to clean all generated protobuf files
# This script calls individual cleaning scripts for each language

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🧹 Cleaning all generated files"
echo "================================="
echo ""

# Make sure individual scripts are executable
chmod +x "${SCRIPT_DIR}/go.sh"
chmod +x "${SCRIPT_DIR}/rust.sh"
chmod +x "${SCRIPT_DIR}/typescript.sh"

# Run Go cleaning
"${SCRIPT_DIR}/go.sh"

echo ""

# Run Rust cleaning
"${SCRIPT_DIR}/rust.sh"

echo ""

# Run TypeScript cleaning
"${SCRIPT_DIR}/typescript.sh"

echo ""
echo "✅ All generated files cleaned successfully!"