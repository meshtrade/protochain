#!/bin/sh
set -e

CONFIG_PATH="/etc/envoy/config.yaml"

# Apply environment variable overrides to envoy config placeholders
ENVOY_ADDRESS="${ENVOY_ADDRESS:-0.0.0.0}"
ENVOY_PORT="${ENVOY_PORT:-50064}"

sed -i "s/ADDRESS_PLACEHOLDER/${ENVOY_ADDRESS}/g" "$CONFIG_PATH"
sed -i "s/PORT_PLACEHOLDER/${ENVOY_PORT}/g" "$CONFIG_PATH"

echo "Starting protochain-solana-api on 127.0.0.1:50051..."
/usr/local/bin/protochain-solana-api &
API_PID=$!

# Wait briefly for the API to start
sleep 2

echo "Starting envoy on ${ENVOY_ADDRESS}:${ENVOY_PORT} -> 127.0.0.1:50051..."
exec envoy --config-path "$CONFIG_PATH" -l info
