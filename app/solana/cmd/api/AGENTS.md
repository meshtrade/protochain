# AGENTS

Scope: `app/solana/cmd/api` (Solana gRPC backend service).

Key entry points
- `src/main.rs` (server bootstrap)
- `src/lib.rs` (library exports)
- `src/config.rs` (env/config loading)
- `src/service_providers/` (dependency injection and RPC clients)
- `src/api/aggregator.rs` (service wiring)
- `src/websocket/` (subscription manager)

API layout
- `src/api/<domain>/v1/{*_api.rs, service_impl.rs}` for each versioned service
- Shared helpers live in `src/api/common/`

Proto sources
- `lib/proto/protochain/solana/**/v1/*.proto`

Typical workflow
- Add or change RPCs: update proto, run `./scripts/code-gen/generate/all.sh`, then update `service_impl.rs`.
- Run server: `cargo run -p protochain-solana-api`.

Tests and tooling
- Integration tests: `tests/go/composable_e2e_test.go`
- Validator/back-end helpers: `scripts/tests/*`

Notes
- Transaction lifecycle rules live in `src/api/transaction/v1/validation.rs`.
