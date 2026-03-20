#!/usr/bin/env bash
set -Eeuo pipefail

# Script to clean generated TypeScript protobuf files

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

echo "🧹 Cleaning TypeScript generated files"
echo "======================================"
echo ""

# Clean TypeScript generated files
echo "🧹 Cleaning TypeScript generated files in lib/ts-web/src..."
if [ -d "${PROJECT_ROOT}/lib/ts-web/src" ]; then
    # Count files before cleaning (exclude index.ts and other manual files)
    GENERATED_COUNT=$(find "${PROJECT_ROOT}/lib/ts-web/src" -name "*_pb.js" -o -name "*_pb.d.ts" -o -name "*_pb.ts" -o -name "*_protochaints.ts" | wc -l | tr -d ' ')

    echo "📊 Found ${GENERATED_COUNT} generated TypeScript files to clean"
    echo ""

    # Remove generated protobuf files
    find "${PROJECT_ROOT}/lib/ts-web/src" -name "*_pb.js" -type f -delete 2>/dev/null || true
    find "${PROJECT_ROOT}/lib/ts-web/src" -name "*_pb.d.ts" -type f -delete 2>/dev/null || true
    find "${PROJECT_ROOT}/lib/ts-web/src" -name "*_pb.ts" -type f -delete 2>/dev/null || true
    find "${PROJECT_ROOT}/lib/ts-web/src" -name "*_protochaints.ts" -type f -delete 2>/dev/null || true

    # Remove empty directories left after cleaning generated files
    find "${PROJECT_ROOT}/lib/ts-web/src" -type d -empty -delete 2>/dev/null || true

    echo "✅ TypeScript generated files cleaned"

    # Clean up build artifacts
    if [ -d "${PROJECT_ROOT}/lib/ts-web/dist" ]; then
        rm -rf "${PROJECT_ROOT}/lib/ts-web/dist"
        echo "✅ dist/ directory removed"
    fi
else
    echo "⚠️  Directory lib/ts-web/src not found, skipping TypeScript cleanup"
fi

echo ""
echo "✅ TypeScript cleanup complete!"