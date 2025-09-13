#!/bin/bash

# Stop Solana Validator Docker container
# Usage: ./scripts/tests/stop-validator-docker.sh (from root directory)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Ensure we're in the project root
cd "$PROJECT_ROOT"

echo "🐳 Stopping Solana Validator Docker container..."

# Check if Docker is available
if ! command -v docker >/dev/null 2>&1; then
    echo "❌ Error: Docker is not installed or not in PATH"
    exit 1
fi

# Check if container exists
if ! docker ps -a --filter name=solana-validator --format "{{.Names}}" | grep -q solana-validator; then
    echo "ℹ️  No solana-validator container found"
    exit 0
fi

# Check if container is running
if docker ps --filter name=solana-validator --format "{{.Names}}" | grep -q solana-validator; then
    echo "🛑 Stopping solana-validator container..."
    docker stop solana-validator
    echo "✅ Container stopped"
else
    echo "ℹ️  Container already stopped"
fi

# Remove the container
echo "🗑️  Removing solana-validator container..."
docker rm solana-validator >/dev/null 2>&1

echo "✅ Solana validator Docker container stopped and removed"

# Check if there are any other Solana containers running
OTHER_SOLANA=$(docker ps --format "{{.Names}}" | grep -i solana | grep -v solana-validator || true)
if [[ -n "$OTHER_SOLANA" ]]; then
    echo ""
    echo "ℹ️  Other Solana containers still running:"
    docker ps --filter name=solana --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
fi

echo ""
echo "📋 To restart validator:"
echo "   ./scripts/tests/start-validator-docker.sh"