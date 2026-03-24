#!/bin/sh
set -e

CONFIG_PATH="/etc/envoy/config.yaml"

# Apply environment variable overrides to envoy config placeholders
ENVOY_ADDRESS="${ENVOY_ADDRESS:-0.0.0.0}"
ENVOY_PORT="${ENVOY_PORT:-50064}"
SOLANA_API_ADDRESS="${SERVER_HOST:-127.0.0.1}"
SOLANA_API_PORT="${SERVER_PORT:-50051}"

sed -i "s/ENVOY_ADDRESS_PLACEHOLDER/${ENVOY_ADDRESS}/g" "$CONFIG_PATH"
sed -i "s/ENVOY_PORT_PLACEHOLDER/${ENVOY_PORT}/g" "$CONFIG_PATH"
sed -i "s/SOLANA_API_ADDRESS_PLACEHOLDER/${SOLANA_API_ADDRESS}/g" "$CONFIG_PATH"
sed -i "s/SOLANA_API_PORT_PLACEHOLDER/${SOLANA_API_PORT}/g" "$CONFIG_PATH"

echo "Starting protochain-solana-api on ${SOLANA_API_ADDRESS}:${SOLANA_API_PORT}..."
/usr/local/bin/protochain-solana-api &
API_PID=$!

# Wait briefly for the API to start
sleep 2

echo "Starting envoy on ${ENVOY_ADDRESS}:${ENVOY_PORT} -> ${SOLANA_API_ADDRESS}:${SOLANA_API_PORT}..."
exec envoy --config-path "$CONFIG_PATH" -l info
