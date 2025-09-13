#!/bin/bash

# Stop Solana Validator + Backend Docker Compose stack
# Usage: ./scripts/tests/stop-docker.sh (from root directory)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.yml"

# Ensure we're in the project root
cd "$PROJECT_ROOT"

echo "🐳 Stopping Protochain Docker Compose Stack..."

# Check if docker-compose.yml exists
if [[ ! -f "$COMPOSE_FILE" ]]; then
    echo "❌ Error: docker-compose.yml not found in project root"
    echo "   Make sure you're running from the correct directory"
    exit 1
fi

# Check if Docker is available
if ! command -v docker >/dev/null 2>&1; then
    echo "❌ Error: Docker is not installed or not in PATH"
    exit 1
fi

# Determine docker compose command (v1 vs v2)
COMPOSE_CMD="docker-compose"
if ! command -v docker-compose >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    COMPOSE_CMD="docker compose"
fi

echo "🔧 Using Docker Compose command: $COMPOSE_CMD"

# Check if any services are running
RUNNING_SERVICES=$($COMPOSE_CMD ps -q 2>/dev/null | wc -l | tr -d ' ')

if [[ "$RUNNING_SERVICES" -eq 0 ]]; then
    echo "ℹ️  No Protochain services are currently running"
    exit 0
fi

echo "🛑 Stopping services..."
$COMPOSE_CMD stop

echo "🗑️  Removing containers..."
$COMPOSE_CMD down

# Optional: Clean up volumes (commented out by default to preserve data)
# echo "🧹 Removing volumes..."
# $COMPOSE_CMD down -v

echo "✅ Protochain Docker stack stopped successfully"

# Show any remaining containers (shouldn't be any)
REMAINING=$($COMPOSE_CMD ps -q 2>/dev/null | wc -l | tr -d ' ')
if [[ "$REMAINING" -gt 0 ]]; then
    echo "⚠️  Some containers may still be running:"
    $COMPOSE_CMD ps
else
    echo "🎯 All containers stopped and removed"
fi

echo ""
echo "📋 To restart the stack:"
echo "   ./scripts/tests/start-docker.sh"
echo ""
echo "📋 To clean up everything including volumes:"
echo "   $COMPOSE_CMD down -v --remove-orphans"