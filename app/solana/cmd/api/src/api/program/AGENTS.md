# AGENTS

Module: Solana program services (system, token).

Proto sources
- `lib/proto/protochain/solana/program/system/v1/service.proto`
- `lib/proto/protochain/solana/program/token/v1/service.proto`

Implementation
- `system/v1/` (System program service)
- `token/v1/` (Token program service)
- `manager.rs` (program API wiring)
- `mod.rs` (module exports)

Notes
- System program conversions live in `system/v1/conversion.rs`.
