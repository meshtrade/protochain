#!/usr/bin/env bash
set -Eeuo pipefail

# Script to clean generated Rust protobuf files

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

echo "🧹 Cleaning Rust generated files"
echo "================================="
echo ""

# Clean Rust generated files
echo "🧹 Cleaning Rust generated files in lib/rust/src..."
if [ -d "${PROJECT_ROOT}/lib/rust/src" ]; then
    # Count files before cleaning (exclude lib.rs)
    GENERATED_COUNT=$(find "${PROJECT_ROOT}/lib/rust/src" -name "*.rs" -type f ! -name "lib.rs" | wc -l | tr -d ' ')
    
    echo "📊 Found ${GENERATED_COUNT} generated .rs files to clean (excluding lib.rs)"
    echo ""
    
    # Remove all .rs files except lib.rs (which is hand-written)
    find "${PROJECT_ROOT}/lib/rust/src" -name "*.rs" -type f ! -name "lib.rs" -delete 2>/dev/null || true
    
    echo "✅ Rust generated files cleaned"
    
    # Also clean up Cargo.lock and target directory if they exist
    if [ -f "${PROJECT_ROOT}/lib/rust/Cargo.lock" ]; then
        rm -f "${PROJECT_ROOT}/lib/rust/Cargo.lock"
        echo "✅ Cargo.lock removed"
    fi
    
    if [ -d "${PROJECT_ROOT}/lib/rust/target" ]; then
        rm -rf "${PROJECT_ROOT}/lib/rust/target"
        echo "✅ target/ directory removed"
    fi
else
    echo "⚠️  Directory lib/rust/src not found, skipping Rust cleanup"
fi

echo ""
echo "✅ Rust cleanup complete!"