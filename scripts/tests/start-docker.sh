#!/bin/bash

# Start Solana Validator + Backend using Docker Compose
# Usage: ./scripts/tests/start-docker.sh (from root directory)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.yml"

# Ensure we're in the project root
cd "$PROJECT_ROOT"

echo "🐳 Starting Protochain Stack with Docker Compose..."

# Check if docker-compose.yml exists
if [[ ! -f "$COMPOSE_FILE" ]]; then
    echo "❌ Error: docker-compose.yml not found in project root"
    echo "   Make sure you're running from the correct directory"
    exit 1
fi

# Check if Docker is available
if ! command -v docker >/dev/null 2>&1; then
    echo "❌ Error: Docker is not installed or not in PATH"
    echo "   Please install Docker first"
    exit 1
fi

if ! command -v docker-compose >/dev/null 2>&1 && ! docker compose version >/dev/null 2>&1; then
    echo "❌ Error: Docker Compose is not available"
    echo "   Please install Docker Compose first"
    exit 1
fi

# Determine docker compose command (v1 vs v2)
COMPOSE_CMD="docker-compose"
if ! command -v docker-compose >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    COMPOSE_CMD="docker compose"
fi

echo "🔧 Using Docker Compose command: $COMPOSE_CMD"

# Check if services are already running
if $COMPOSE_CMD ps -q protochain-solana-api >/dev/null 2>&1; then
    if [[ -n $($COMPOSE_CMD ps -q protochain-solana-api 2>/dev/null) ]]; then
        echo "⚠️  Protochain services appear to already be running"
        echo "   Current status:"
        $COMPOSE_CMD ps
        echo ""
        echo "   Use './scripts/tests/stop-docker.sh' to stop services first"
        echo "   Or use '$COMPOSE_CMD restart' to restart services"
        exit 1
    fi
fi

echo "🏗️  Building and starting services..."
echo "   - solana-validator: Solana test validator (port 8899)"
echo "   - protochain-api: gRPC backend (port 50051)"
echo ""

# Start services
$COMPOSE_CMD up -d --build

echo "⏳ Waiting for services to be healthy..."

# Wait for services to be ready (with timeout)
MAX_WAIT=60
WAIT_COUNT=0
HEALTHY_COUNT=0

while [[ $WAIT_COUNT -lt $MAX_WAIT ]]; do
    # Check health status
    VALIDATOR_STATUS=$($COMPOSE_CMD ps -f "name=protochain-solana-validator" --format "{{.Health}}" 2>/dev/null || echo "unknown")
    API_STATUS=$($COMPOSE_CMD ps -f "name=protochain-solana-api" --format "{{.Health}}" 2>/dev/null || echo "unknown")

    if [[ "$VALIDATOR_STATUS" == "healthy" && "$API_STATUS" == "healthy" ]]; then
        ((HEALTHY_COUNT++))
        if [[ $HEALTHY_COUNT -ge 3 ]]; then  # Stable for 3 checks
            echo "✅ All services are healthy and ready!"
            break
        fi
    else
        HEALTHY_COUNT=0
    fi

    if [[ $((WAIT_COUNT % 10)) -eq 0 ]] || [[ $HEALTHY_COUNT -gt 0 ]]; then
        echo "   Validator: $VALIDATOR_STATUS, API: $API_STATUS ($WAIT_COUNT/$MAX_WAIT sec)"
    fi

    sleep 1
    ((WAIT_COUNT++))
done

if [[ $WAIT_COUNT -ge $MAX_WAIT ]]; then
    echo "⚠️  Services started but health check timed out"
    echo "   You can check status with: $COMPOSE_CMD ps"
    echo "   View logs with: $COMPOSE_CMD logs -f"
else
    echo ""
    echo "🎉 Protochain stack is ready!"
fi

# Show service status
echo ""
echo "📊 Service Status:"
$COMPOSE_CMD ps

echo ""
echo "🌐 Available Endpoints:"
echo "   - Solana RPC: http://localhost:8899"
echo "   - Protochain gRPC: localhost:50051"

echo ""
echo "📋 Useful Commands:"
echo "   - View all logs: $COMPOSE_CMD logs -f"
echo "   - View API logs: $COMPOSE_CMD logs -f protochain-api"
echo "   - View validator logs: $COMPOSE_CMD logs -f solana-validator"
echo "   - Run integration tests: cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v"
echo "   - Stop services: ./scripts/tests/stop-docker.sh"

# Test connectivity
echo ""
echo "🔍 Quick connectivity test:"

# Test Solana RPC
if curl -s -X POST -H "Content-Type: application/json" \
   -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
   http://localhost:8899 >/dev/null 2>&1; then
    echo "   ✅ Solana RPC responding at localhost:8899"
else
    echo "   ⚠️  Solana RPC not responding (may still be starting)"
fi

# Test gRPC API (using nc if available)
if command -v nc >/dev/null 2>&1; then
    if nc -z localhost 50051 2>/dev/null; then
        echo "   ✅ gRPC API responding at localhost:50051"
    else
        echo "   ⚠️  gRPC API not responding (may still be starting)"
    fi
else
    echo "   ℹ️  gRPC connectivity test skipped (nc not available)"
fi

echo ""
echo "✨ Ready for development and testing!"