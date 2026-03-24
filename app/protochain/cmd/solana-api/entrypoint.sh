#!/bin/sh
set -e

CONFIG_PATH="/etc/envoy/config.yaml"

# Apply environment variable overrides to envoy config placeholders
ENVOY_ADDRESS="${ENVOY_ADDRESS:-0.0.0.0}"
ENVOY_PORT="${ENVOY_PORT:-50064}"
SERVER_HOST="${SERVER_HOST:-127.0.0.1}"
SERVER_PORT="${SERVER_PORT:-50051}"

sed -i "s/ENVOY_ADDRESS_PLACEHOLDER/${ENVOY_ADDRESS}/g" "$CONFIG_PATH"
sed -i "s/ENVOY_PORT_PLACEHOLDER/${ENVOY_PORT}/g" "$CONFIG_PATH"
sed -i "s/SOLANA_API_ADDRESS_PLACEHOLDER/${SERVER_HOST}/g" "$CONFIG_PATH"
sed -i "s/SOLANA_API_PORT_PLACEHOLDER/${SERVER_PORT}/g" "$CONFIG_PATH"

echo "Starting protochain-solana-api on ${SERVER_HOST}:${SERVER_PORT}..."
/usr/local/bin/protochain-solana-api &
API_PID=$!

# Wait briefly for the API to start
sleep 2

echo "Starting envoy on ${ENVOY_ADDRESS}:${ENVOY_PORT} -> ${SERVER_HOST}:${SERVER_PORT}..."
exec envoy --config-path "$CONFIG_PATH" -l info
