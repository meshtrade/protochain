#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

echo "🏃 Generating All Language Bindings"
echo "==================================="
echo ""

# Check if buf is installed
if ! command -v buf &> /dev/null; then
    echo "❌ Error: buf is not installed"
    echo "Please install buf from https://docs.buf.build/installation"
    exit 1
fi

# Check if the buf.gen.yaml exists
if [ ! -f "${PROJECT_ROOT}/lib/_code_gen/buf.gen.yaml" ]; then
    echo "❌ Error: buf.gen.yaml not found at ${PROJECT_ROOT}/lib/_code_gen/buf.gen.yaml"
    exit 1
fi

# Check if proto directory exists
if [ ! -d "${PROJECT_ROOT}/lib/proto" ]; then
    echo "❌ Error: Proto directory not found at ${PROJECT_ROOT}/lib/proto"
    exit 1
fi

echo "🔍 Validating protobuf definitions..."
if ! buf lint "${PROJECT_ROOT}/lib/proto"; then
    echo "❌ Protobuf linting failed"
    exit 1
fi
echo "✅ Protobuf definitions validated"
echo ""

echo "🏃 Building TypeScript protoc plugin..."
cd "${PROJECT_ROOT}/tool/protoc-gen/cmd/protochain_ts_web"
if [ ! -d "node_modules" ]; then
    yarn install --frozen-lockfile 2>/dev/null || yarn install
fi
yarn build
cd "${PROJECT_ROOT}"
echo "✅ TypeScript protoc plugin built"
echo ""

echo "🏃 Generating code from protobuf definitions..."
# Change to project root to ensure relative paths in buf.gen.yaml work correctly
cd "${PROJECT_ROOT}"
if ! buf generate "lib/proto" --template "lib/_code_gen/buf.gen.yaml"; then
    echo "❌ Code generation failed"
    exit 1
fi

echo ""
echo "🏃 Generating TypeScript index files..."
node "${PROJECT_ROOT}/tool/ts-index-generation/generate-index-files.js"
echo "✅ TypeScript index files generated"

echo ""
echo "✅ All code generation complete!"
echo ""
echo "Generated files:"
echo "  • Go:         lib/go/protochain/"
echo "  • Rust:       lib/rust/src/"
echo "  • TypeScript: lib/ts-web/src/"
echo ""