#!/bin/bash

# Stop Solana Local Validator
# Usage: ./project/solana/scripts/stop-validator.sh (from root directory)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
PID_FILE="$PROJECT_ROOT/.solana-validator.pid"
LOG_FILE="$PROJECT_ROOT/.solana-validator.log"
LEDGER_DIR="$PROJECT_ROOT/.solana-validator-ledger"
CLI_CONFIG_BACKUP="$PROJECT_ROOT/.solana-cli-config.backup"

# Ensure we're in the project root
cd "$PROJECT_ROOT"

echo "🛑 Stopping Solana Local Validator..."

# Function to restore Solana CLI configuration
restore_cli_config() {
    if [[ -f "$CLI_CONFIG_BACKUP" ]]; then
        BACKUP_URL=$(cat "$CLI_CONFIG_BACKUP" 2>/dev/null || echo "")
        if [[ -n "$BACKUP_URL" && "$BACKUP_URL" != "localhost" && "$BACKUP_URL" != "http://localhost:8899" ]]; then
            echo "🔧 Restoring Solana CLI configuration to: $BACKUP_URL"
            solana config set --url "$BACKUP_URL" >/dev/null 2>&1 || true
        else
            echo "🔧 Restoring Solana CLI to mainnet-beta (default)"
            solana config set --url mainnet-beta >/dev/null 2>&1 || true
        fi
        rm -f "$CLI_CONFIG_BACKUP"
    else
        echo "🔧 Resetting Solana CLI to mainnet-beta (no backup found)"
        solana config set --url mainnet-beta >/dev/null 2>&1 || true
    fi
}

# Check if PID file exists
if [[ ! -f "$PID_FILE" ]]; then
    echo "⚠️  No validator PID file found"
    echo "   Validator may not be running or was not started with start-validator.sh"
    
    # Try to find any running solana-test-validator processes
    VALIDATOR_PID=$(pgrep -f "solana-test-validator" 2>/dev/null || true)
    if [[ -n "$VALIDATOR_PID" ]]; then
        echo "🔍 Found running solana-test-validator process (PID: $VALIDATOR_PID)"
        echo "   Attempting to stop it..."
        kill "$VALIDATOR_PID" 2>/dev/null || true
        sleep 2
        
        # Force kill if still running
        if kill -0 "$VALIDATOR_PID" 2>/dev/null; then
            echo "🔨 Force stopping validator process..."
            kill -9 "$VALIDATOR_PID" 2>/dev/null || true
        fi
        
        echo "✅ Stopped running solana-test-validator process"
    else
        echo "ℹ️  No running validator processes found"
    fi
    
    # Restore CLI configuration and clean up files
    restore_cli_config
    rm -f "$LOG_FILE"
    if [[ -d "$LEDGER_DIR" ]]; then
        echo "🧹 Cleaning up validator ledger..."
        rm -rf "$LEDGER_DIR"
    fi
    exit 0
fi

# Read PID from file
PID=$(cat "$PID_FILE")
echo "🔢 Found validator PID: $PID"

# Check if process is running
if ! kill -0 "$PID" 2>/dev/null; then
    echo "⚠️  Process $PID is not running (may have already stopped)"
    rm -f "$PID_FILE"
    restore_cli_config
    echo "✅ Cleaned up stale files"
    exit 0
fi

# Try graceful shutdown first
echo "🕊️  Attempting graceful shutdown..."
kill "$PID" 2>/dev/null || true

# Wait up to 10 seconds for graceful shutdown (validator takes longer than backend)
MAX_WAIT=10
WAIT_COUNT=0

while [[ $WAIT_COUNT -lt $MAX_WAIT ]] && kill -0 "$PID" 2>/dev/null; do
    sleep 1
    ((WAIT_COUNT++))
    echo "   Waiting for graceful shutdown... ($WAIT_COUNT/$MAX_WAIT)"
done

# Force kill if still running
if kill -0 "$PID" 2>/dev/null; then
    echo "🔨 Force stopping validator (PID: $PID)..."
    kill -9 "$PID" 2>/dev/null || true
    sleep 2
fi

# Verify process is stopped
if kill -0 "$PID" 2>/dev/null; then
    echo "❌ Failed to stop validator process (PID: $PID)"
    echo "   You may need to stop it manually: kill -9 $PID"
    exit 1
fi

# Restore Solana CLI configuration
restore_cli_config

# Clean up files and directories
echo "🧹 Cleaning up validator files..."
rm -f "$PID_FILE"

# Clean up ledger directory
if [[ -d "$LEDGER_DIR" ]]; then
    echo "🗂️  Removing validator ledger directory..."
    rm -rf "$LEDGER_DIR"
fi

# Optionally keep log file for debugging
if [[ -f "$LOG_FILE" ]]; then
    echo "📄 Log file preserved: ${LOG_FILE#$PROJECT_ROOT/}"
    echo "   (Delete manually if no longer needed)"
fi

echo "✅ Solana validator stopped successfully"
echo "🌐 Solana CLI configuration restored"