#!/bin/bash

# Start Solana Local Validator in background
# Usage: ./scripts/tests/start-validator.sh (from root directory)
#
# For Docker-based development, use: ./scripts/tests/start-docker.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
PID_FILE="$PROJECT_ROOT/.solana-validator.pid"
LOG_FILE="$PROJECT_ROOT/.solana-validator.log"
LEDGER_DIR="$PROJECT_ROOT/.solana-validator-ledger"
CLI_CONFIG_BACKUP="$PROJECT_ROOT/.solana-cli-config.backup"

# Ensure we're in the project root
cd "$PROJECT_ROOT"

echo "🌐 Starting Solana Local Validator..."

# Check if validator is already running
if [[ -f "$PID_FILE" ]]; then
    PID=$(cat "$PID_FILE")
    if kill -0 "$PID" 2>/dev/null; then
        echo "⚠️  Solana validator is already running (PID: $PID)"
        echo "   Use ./scripts/tests/stop-validator.sh to stop it first"
        echo "   Or use ./scripts/tests/start-docker.sh for Docker-based development"
        exit 1
    else
        echo "🧹 Cleaning up stale PID file"
        rm -f "$PID_FILE"
    fi
fi

# Check if solana-test-validator command is available
if ! command -v solana-test-validator >/dev/null 2>&1; then
    echo "❌ Error: solana-test-validator command not found"
    echo "   Please install Solana CLI tools first"
    echo "   Visit: https://docs.solana.com/cli/install-solana-cli-tools"
    exit 1
fi

# Check if solana command is available
if ! command -v solana >/dev/null 2>&1; then
    echo "❌ Error: solana command not found"
    echo "   Please install Solana CLI tools first"
    echo "   Visit: https://docs.solana.com/cli/install-solana-cli-tools"
    exit 1
fi

echo "🔧 Backing up current Solana CLI configuration..."
# Backup current solana config
CURRENT_URL=$(solana config get 2>/dev/null | grep "RPC URL:" | awk '{print $3}' || echo "")
echo "$CURRENT_URL" > "$CLI_CONFIG_BACKUP"
echo "   Backed up RPC URL: $CURRENT_URL"

echo "🦀 Starting Solana validator in background..."
echo "📡 Validator will be available at: localhost:8899"
echo "💰 Faucet will be available at: localhost:9900"
echo "📄 Logs will be written to: ${LOG_FILE#$PROJECT_ROOT/}"
echo "📁 Ledger will be stored in: ${LEDGER_DIR#$PROJECT_ROOT/}"
echo ""

# Create ledger directory
mkdir -p "$LEDGER_DIR"

# Start validator in background with comprehensive configuration
solana-test-validator \
    --reset \
    --quiet \
    --ledger "$LEDGER_DIR" \
    --faucet-sol 1000000 \
    --faucet-port 9900 \
    --rpc-port 8899 > "$LOG_FILE" 2>&1 &

VALIDATOR_PID=$!

# Save PID
echo "$VALIDATOR_PID" > "$PID_FILE"

echo "✅ Validator started in background"
echo "🔢 Process ID: $VALIDATOR_PID"
echo "🗂️  PID file: ${PID_FILE#$PROJECT_ROOT/}"

# Wait for validator to be ready
echo "⏳ Waiting for validator to initialize and be ready..."
MAX_WAIT=30
WAIT_COUNT=0

while [[ $WAIT_COUNT -lt $MAX_WAIT ]]; do
    # Check if process is still running
    if ! kill -0 "$VALIDATOR_PID" 2>/dev/null; then
        echo "❌ Validator process died unexpectedly"
        echo "📄 Last few lines of log file:"
        tail -10 "$LOG_FILE" 2>/dev/null || echo "   (no log file found)"
        rm -f "$PID_FILE"
        exit 1
    fi
    
    # Try configuring CLI and testing connection
    if [[ $WAIT_COUNT -eq 5 ]]; then
        echo "🔧 Configuring Solana CLI to use local validator..."
        solana config set --url localhost >/dev/null 2>&1 || true
    fi
    
    # Check if validator is responding (after CLI is configured)
    if [[ $WAIT_COUNT -gt 7 ]]; then
        if solana cluster-version >/dev/null 2>&1; then
            echo "✅ Validator is ready and responding!"
            echo "🌐 Configured Solana CLI to use localhost:8899"
            echo ""
            echo "💰 To get test SOL: solana airdrop 100"  
            echo "🔍 To check status: solana cluster-version"
            echo "🛑 To stop validator: ./scripts/tests/stop-validator.sh"
            echo ""
            echo "🎯 You can now run: ./scripts/tests/start-backend.sh"
            echo "🐳 Or try Docker: ./scripts/tests/start-docker.sh"
            exit 0
        fi
    fi
    
    sleep 1
    ((WAIT_COUNT++))
    if [[ $((WAIT_COUNT % 5)) -eq 0 ]]; then
        echo "   Still waiting... ($WAIT_COUNT/$MAX_WAIT seconds)"
    fi
done

# If we get here, validator didn't become ready in time
echo "⚠️  Validator started but may not be fully ready yet"
echo "🔧 Configuring Solana CLI anyway..."
solana config set --url localhost >/dev/null 2>&1 || true

echo "✅ Validator should be ready soon"
echo "🧪 You can test with: solana cluster-version"
echo "🛑 Use './scripts/tests/stop-validator.sh' to stop validator"