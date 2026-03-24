# app/protochain

This directory contains browser-ready Docker images for protochain APIs. See [README.md](README.md) for full architecture, configuration, and usage details.

## Key Context

- Each image bundles **Envoy** (Connect-to-gRPC bridge) + a **Rust gRPC backend**
- Envoy is exposed on port `50064`; the Rust backend runs internally on `127.0.0.1:50051`
- Dockerfiles live in `ci/`, runtime config and entrypoints in `cmd/`

## Build & Run

```bash
# Build from repo root (needs access to lib/, app/solana/, buf.yaml)
docker build -f ci/solana-api/Dockerfile -t protochain-solana-api .

# Run locally against a host validator
docker run -p 50064:50064 \
  -e SOLANA_RPC_URL=http://host.docker.internal:8899 \
  protochain-solana-api
```

## Adding a New Chain API

Follow the `cmd/solana-api` pattern:
1. Create `cmd/<chain>-api/` with `entrypoint.sh` and `config.yaml`
2. Create `ci/<chain>-api/Dockerfile`
3. Add the entry to the table in `README.md`
