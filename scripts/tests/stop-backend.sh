#!/bin/bash

# Stop Solana gRPC Backend Server
# Usage: ./project/solana/scripts/stop-backend.sh (from root directory)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
PID_FILE="$PROJECT_ROOT/.solana-backend.pid"
PORT_FILE="$PROJECT_ROOT/.solana-backend.port"
LOG_FILE="$PROJECT_ROOT/.solana-backend.log"

# Ensure we're in the project root
cd "$PROJECT_ROOT"

echo "🛑 Stopping Solana gRPC Backend Server..."

# Check if PID file exists
if [[ ! -f "$PID_FILE" ]]; then
    echo "⚠️  No backend server PID file found"
    echo "   Server may not be running or was not started with start-backend.sh"
    
    # Try to find any running cargo processes for our project
    CARGO_PID=$(pgrep -f "protochain-solana-api" 2>/dev/null || true)
    if [[ -n "$CARGO_PID" ]]; then
        echo "🔍 Found running protochain-solana-api process (PID: $CARGO_PID)"
        echo "   Attempting to stop it..."
        kill "$CARGO_PID" 2>/dev/null || true
        sleep 2
        
        # Force kill if still running
        if kill -0 "$CARGO_PID" 2>/dev/null; then
            echo "🔨 Force stopping process..."
            kill -9 "$CARGO_PID" 2>/dev/null || true
        fi
        
        echo "✅ Stopped running protochain-solana-api process"
    else
        echo "ℹ️  No running backend processes found"
    fi
    
    # Clean up any leftover files
    rm -f "$PORT_FILE" "$LOG_FILE"
    exit 0
fi

# Read PID from file
PID=$(cat "$PID_FILE")
echo "🔢 Found PID: $PID"

# Check if process is running
if ! kill -0 "$PID" 2>/dev/null; then
    echo "⚠️  Process $PID is not running (may have already stopped)"
    rm -f "$PID_FILE" "$PORT_FILE"
    echo "✅ Cleaned up stale files"
    exit 0
fi

# Try graceful shutdown first
echo "🕊️  Attempting graceful shutdown..."
kill "$PID" 2>/dev/null || true

# Wait up to 5 seconds for graceful shutdown
MAX_WAIT=5
WAIT_COUNT=0

while [[ $WAIT_COUNT -lt $MAX_WAIT ]] && kill -0 "$PID" 2>/dev/null; do
    sleep 1
    ((WAIT_COUNT++))
    echo "   Waiting for graceful shutdown... ($WAIT_COUNT/$MAX_WAIT)"
done

# Force kill if still running
if kill -0 "$PID" 2>/dev/null; then
    echo "🔨 Force stopping server (PID: $PID)..."
    kill -9 "$PID" 2>/dev/null || true
    sleep 1
fi

# Verify process is stopped
if kill -0 "$PID" 2>/dev/null; then
    echo "❌ Failed to stop server process (PID: $PID)"
    echo "   You may need to stop it manually: kill -9 $PID"
    exit 1
fi

# Clean up files
echo "🧹 Cleaning up files..."
rm -f "$PID_FILE" "$PORT_FILE"

# Optionally keep log file for debugging
if [[ -f "$LOG_FILE" ]]; then
    echo "📄 Log file preserved: ${LOG_FILE#$PROJECT_ROOT/}"
    echo "   (Delete manually if no longer needed)"
fi

echo "✅ Backend server stopped successfully"