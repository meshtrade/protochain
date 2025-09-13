#!/bin/bash

# Start Solana Validator in Docker (for hybrid development)
# Usage: ./scripts/tests/start-validator-docker.sh (from root directory)
#
# This is the most common development pattern: validator in Docker, backend locally

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Ensure we're in the project root
cd "$PROJECT_ROOT"

echo "🐳 Starting Solana Validator in Docker..."

# Check if Docker is available
if ! command -v docker >/dev/null 2>&1; then
    echo "❌ Error: Docker is not installed or not in PATH"
    echo "   Please install Docker first"
    exit 1
fi

# Check if container is already running
if docker ps --filter name=solana-validator --format "table {{.Names}}" | grep -q solana-validator; then
    echo "⚠️  Solana validator container is already running"
    echo "   Current status:"
    docker ps --filter name=solana-validator --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
    echo ""
    echo "   Use 'docker stop solana-validator && docker rm solana-validator' to stop it"
    echo "   Or use './scripts/tests/stop-validator-docker.sh'"
    exit 1
fi

echo "🚀 Starting Solana test validator in Docker container..."
echo "📡 Validator will be available at: localhost:8899"
echo "🐳 Container name: solana-validator"
echo ""

# Start validator container
docker run -d \
    --name solana-validator \
    -p 8899:8899 \
    -p 8900:8900 \
    solanalabs/solana:v1.18.15 \
    solana-test-validator \
    --ledger /solana-ledger \
    --bind-address 0.0.0.0 \
    --rpc-bind-address 0.0.0.0 \
    --rpc-port 8899 \
    --faucet-sol 1000000 \
    --reset \
    --quiet

echo "✅ Validator container started"

# Wait for validator to be ready
echo "⏳ Waiting for validator to initialize and be ready..."
MAX_WAIT=30
WAIT_COUNT=0

while [[ $WAIT_COUNT -lt $MAX_WAIT ]]; do
    # Check if container is still running
    if ! docker ps --filter name=solana-validator --format "{{.Names}}" | grep -q solana-validator; then
        echo "❌ Validator container stopped unexpectedly"
        echo "📄 Container logs:"
        docker logs solana-validator 2>/dev/null || echo "   (no logs available)"
        exit 1
    fi

    # Check if validator is responding
    if curl -s -X POST -H "Content-Type: application/json" \
       -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
       http://localhost:8899 >/dev/null 2>&1; then
        echo "✅ Validator is ready and responding!"
        break
    fi

    sleep 1
    ((WAIT_COUNT++))
    if [[ $((WAIT_COUNT % 5)) -eq 0 ]]; then
        echo "   Still waiting... ($WAIT_COUNT/$MAX_WAIT seconds)"
    fi
done

if [[ $WAIT_COUNT -ge $MAX_WAIT ]]; then
    echo "⚠️  Validator started but may not be fully ready yet"
    echo "🧪 You can test with: curl -X POST -H 'Content-Type: application/json' -d '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getHealth\"}' http://localhost:8899"
else
    echo "🌐 Validator is ready at localhost:8899"
    echo ""
    echo "💰 To get test SOL: solana airdrop 100 --url http://localhost:8899"
    echo "🔍 To check status: curl -X POST -H 'Content-Type: application/json' -d '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getHealth\"}' http://localhost:8899"
fi

echo ""
echo "📋 Next Steps:"
echo "   1. Start backend locally: cargo run -p protochain-solana-api"
echo "   2. Run integration tests: cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v"
echo "   3. View validator logs: docker logs -f solana-validator"
echo "   4. Stop validator: ./scripts/tests/stop-validator-docker.sh"

echo ""
echo "🎯 Ready for hybrid development (Docker validator + local backend)!"