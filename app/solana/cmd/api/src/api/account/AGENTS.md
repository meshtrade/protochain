# AGENTS

Module: Account service API.

Proto sources
- `lib/proto/protochain/solana/account/v1/service.proto`
- `lib/proto/protochain/solana/account/v1/account.proto`

Implementation
- `v1/account_v1_api.rs` (API wrapper)
- `v1/service_impl.rs` (service logic)

Notes
- Keep RPC behavior aligned with the proto contract and error semantics.
