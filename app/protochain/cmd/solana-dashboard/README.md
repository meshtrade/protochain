# Solana Dashboard

Browser UI for the Protochain Solana gRPC API. Routes mirror the proto service definitions — each RPC method gets its own page with a request form and response panel.

Built with Next.js, React, shadcn/ui, and `@protochain/ts-web` (gRPC-Web client).

## Development

```bash
# Start the backend stack
docker compose up surfpool -d
cargo run -p protochain-solana-api

# Run the dashboard
yarn workspace @protochain/solana-dashboard dev
```

Open http://localhost:3000. The dashboard connects to the API at `http://localhost:50064` by default (configurable via the UI).

## Build

```bash
yarn workspace @protochain/solana-dashboard build
```

## Docker

The image is published to GHCR via CI on `protochain-solana-dashboard/v*.*.*` tags. To run the full stack locally:

```bash
docker compose up -d
```
