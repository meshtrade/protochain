# AGENTS

Module: Transaction service API.

Proto sources
- `lib/proto/protochain/solana/transaction/v1/service.proto`
- `lib/proto/protochain/solana/transaction/v1/transaction.proto`
- `lib/proto/protochain/solana/transaction/v1/instruction.proto`
- `lib/proto/protochain/solana/transaction/v1/error.proto`

Implementation
- `v1/transaction_v1_api.rs` (API wrapper)
- `v1/service_impl.rs` (service logic)
- `v1/validation.rs` (state machine checks)
- `v1/error_builder.rs` (error mapping)

Notes
- Respect the transaction state machine; validation lives in `v1/validation.rs`.
