#!/bin/bash

# Start Solana gRPC Backend Server in background
# Usage: ./scripts/tests/start-backend.sh (from root directory)
#
# For Docker-based development, use: ./scripts/tests/start-docker.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PID_FILE="$PROJECT_ROOT/.solana-backend.pid"
PORT_FILE="$PROJECT_ROOT/.solana-backend.port"
LOG_FILE="$PROJECT_ROOT/.solana-backend.log"

# Ensure we're in the project root
cd "$PROJECT_ROOT"

echo "🚀 Starting Solana gRPC Backend Server..."

# Check if server is already running
if [[ -f "$PID_FILE" ]]; then
    PID=$(cat "$PID_FILE")
    if kill -0 "$PID" 2>/dev/null; then
        echo "⚠️  Backend server is already running (PID: $PID)"
        echo "   Use ./scripts/tests/stop-backend.sh to stop it first"
        echo "   Or use ./scripts/tests/start-docker.sh for Docker-based development"
        exit 1
    else
        echo "🧹 Cleaning up stale PID file"
        rm -f "$PID_FILE" "$PORT_FILE"
    fi
fi

# Ensure Rust workspace is configured
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Error: Cargo.toml not found"
    echo "   Make sure you're running from the project root directory"
    exit 1
fi

if ! grep -q "app/solana/cmd/api" Cargo.toml; then
    echo "❌ Error: 'app/solana/cmd/api' workspace member not found in Cargo.toml"
    echo "   Make sure the Rust workspace is properly configured"
    exit 1
fi

echo "🦀 Starting Rust gRPC server (new structured app) in background..."
echo "📡 Server will be available at: localhost:50051"
echo "📄 Logs will be written to: ${LOG_FILE#$PROJECT_ROOT/}"
echo ""

# Start the new structured server in background and capture PID
cargo run -p protochain-solana-api > "$LOG_FILE" 2>&1 &
SERVER_PID=$!

# Save PID and port
echo "$SERVER_PID" > "$PID_FILE"
echo "50051" > "$PORT_FILE"

echo "✅ Backend server started in background"
echo "🔢 Process ID: $SERVER_PID"
echo "🗂️  PID file: ${PID_FILE#$PROJECT_ROOT/}"

# Wait for server to be ready
echo "⏳ Waiting for server to be ready..."
MAX_WAIT=10
WAIT_COUNT=0

while [[ $WAIT_COUNT -lt $MAX_WAIT ]]; do
    # Check if process is still running
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        echo "❌ Server process died unexpectedly"
        echo "📄 Last few lines of log file:"
        tail -5 "$LOG_FILE" 2>/dev/null || echo "   (no log file found)"
        rm -f "$PID_FILE" "$PORT_FILE"
        exit 1
    fi
    
    # Check if server is responding (using nc if available, otherwise just wait)
    if command -v nc >/dev/null 2>&1; then
        if nc -z localhost 50051 2>/dev/null; then
            echo "✅ Server is ready and accepting connections!"
            echo "🎯 Run integration tests: cd tests/go && go test -v"
            echo "🛑 Use './scripts/tests/stop-backend.sh' to stop server"
            exit 0
        fi
    fi
    
    sleep 1
    ((WAIT_COUNT++))
    echo "   Still waiting... ($WAIT_COUNT/$MAX_WAIT)"
done

# Final check if we couldn't use nc
if ! command -v nc >/dev/null 2>&1; then
    echo "✅ Server should be ready (nc not available for port check)"
    echo "🎯 Run integration tests: cd tests/go && go test -v"
    echo "🛑 Use './scripts/tests/stop-backend.sh' to stop server"
    exit 0
fi

echo "⚠️  Server started but may not be fully ready yet"
echo "🎯 You can still try running tests with: cd tests/go && go test -v"
echo "🛑 Use './scripts/tests/stop-backend.sh' to stop server"