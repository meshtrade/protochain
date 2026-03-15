# AGENTS

Module: RPC client service API.

Proto sources
- `lib/proto/protochain/solana/rpc_client/v1/service.proto`

Implementation
- `v1/rpc_client_v1_api.rs` (API wrapper)
- `v1/service_impl.rs` (service logic)

Notes
- This module exposes low-level RPC access; keep behavior consistent with Solana RPC semantics.
