# Protochain

Browser-ready Docker images for protochain APIs. Each image bundles an [Envoy](https://www.envoyproxy.io/) reverse proxy with the chain-specific gRPC backend, so clients can connect using either:

- **Connect Web** (browser / `lib/ts-web`) — Envoy's `connect_grpc_bridge` filter upgrades Connect-protocol requests to native gRPC.
- **gRPC** (server-side / `lib/go`) — forwarded through as-is, no upgrade needed.

```
Browser (Connect Web) ──┐
                        ├──▶ Envoy ──▶ gRPC backend
Go / gRPC client ───────┘
```

## Available APIs

| Directory | Chain | Status |
|-----------|-------|--------|
| `cmd/solana-api` | Solana | Available |

More chains (e.g. `cmd/sui-api`) may be added in the future following the same pattern.

## Solana API

### Build

```bash
docker build -f ci/solana-api/Dockerfile -t protochain-solana-api .
```

Run from the repository root — the build context needs access to `lib/`, `app/solana/`, and `buf.yaml`.

### Run

```bash
# Local development (validator on host)
docker run -p 50064:50064 \
  -e SOLANA_RPC_URL=http://host.docker.internal:8899 \
  protochain-solana-api
```

### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SOLANA_RPC_URL` | `http://host.docker.internal:8899` | Solana RPC endpoint |
| `SOLANA_WEBSOCKET_URL` | `wss://host.docker.internal:8899` | Solana WebSocket endpoint |
| `SOLANA_TIMEOUT_SECONDS` | `30` | RPC request timeout |
| `SOLANA_RETRY_ATTEMPTS` | `3` | RPC retry attempts |
| `SOLANA_HEALTH_CHECK_ON_STARTUP` | `true` | Verify RPC connectivity on start |
| `ENVOY_ADDRESS` | `0.0.0.0` | Envoy listen address |
| `ENVOY_PORT` | `50064` | Envoy listen port (exposed to clients) |

### How it works

The container runs two processes via `entrypoint.sh`:

1. **Rust gRPC backend** (`protochain-solana-api`) on `127.0.0.1:50051` (internal only)
2. **Envoy proxy** on `0.0.0.0:50064` (exposed), applying the Connect-to-gRPC bridge

The static Envoy config lives at `cmd/solana-api/config.yaml`. Placeholder values (`ADDRESS_PLACEHOLDER`, `PORT_PLACEHOLDER`) are replaced at startup from environment variables.
