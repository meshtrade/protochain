# Protochain Rust SDK

This crate (`protochain-api`) provides generated Rust bindings (prost + tonic) for the Protochain gRPC API.

## Generated Code

All code in `src/` (except `lib.rs`) is auto-generated from protobuf definitions in `lib/proto/protochain/solana/`.

To regenerate:
```bash
# From repository root
./scripts/code-gen/generate/all.sh
```

## Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
protochain-api = { path = "../../lib/rust" }
```

This crate is used by the Solana API backend at `app/solana/cmd/api/`.
