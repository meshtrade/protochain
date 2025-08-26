#!/usr/bin/env bash
set -Eeuo pipefail

# Script to clean generated Go protobuf files

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

echo "🧹 Cleaning Go generated files"
echo "==============================="
echo ""

# Clean Go generated files
echo "🧹 Cleaning Go generated files in api/go..."
if [ -d "${PROJECT_ROOT}/lib/go" ]; then
    # Count files before cleaning
    PB_COUNT=$(find "${PROJECT_ROOT}/lib/go" -name "*.pb.go" -type f | wc -l | tr -d ' ')
    GRPC_COUNT=$(find "${PROJECT_ROOT}/lib/go" -name "*_grpc.pb.go" -type f | wc -l | tr -d ' ')
    PASSIV_COUNT=$(find "${PROJECT_ROOT}/lib/go" -name "*.passivgo.go" -type f | wc -l | tr -d ' ')
    
    echo "📊 Found files to clean:"
    echo "   • *.pb.go files: ${PB_COUNT}"
    echo "   • *_grpc.pb.go files: ${GRPC_COUNT}"
    echo "   • *.passivgo.go files: ${PASSIV_COUNT}"
    echo ""
    
    # Remove all .pb.go files (protobuf generated)
    find "${PROJECT_ROOT}/lib/go" -name "*.pb.go" -type f -delete 2>/dev/null || true
    
    # Remove all _grpc.pb.go files (gRPC generated)
    find "${PROJECT_ROOT}/lib/go" -name "*_grpc.pb.go" -type f -delete 2>/dev/null || true
    
    # Remove all .passivgo.go files (custom generator)
    find "${PROJECT_ROOT}/lib/go" -name "*.passivgo.go" -type f -delete 2>/dev/null || true
    
    echo "✅ Go generated files cleaned"
    
    # Clean up empty directories
    find "${PROJECT_ROOT}/lib/go" -type d -empty -delete 2>/dev/null || true
    echo "✅ Empty directories removed"
else
    echo "⚠️  Directory lib/go not found, skipping Go cleanup"
fi

echo ""
echo "✅ Go cleanup complete!"