# Solana gRPC API Server

Rust gRPC backend for Solana blockchain operations (`protochain-solana-api`).

## Architecture

```
app/solana/cmd/api/
├── src/
│   ├── main.rs                    # gRPC server entry (port 50051)
│   ├── config.rs                  # Configuration management
│   ├── service_providers/         # Dependency injection container
│   │   ├── service_providers.rs   # Main service provider struct
│   │   └── solana_clients.rs      # Solana RPC client management
│   └── api/                       # Service implementations
│       ├── aggregator.rs          # API aggregator
│       ├── account/v1/            # Account service
│       ├── transaction/v1/        # Transaction state machine
│       ├── program/
│       │   ├── system/v1/         # System program wrappers
│       │   └── token/v1/          # SPL Token & Token-2022
│       └── rpc_client/v1/         # Direct RPC client operations
└── Cargo.toml
```

## Services

### Account Service v1
- **GetAccount** - Retrieve account data with configurable commitment levels
- **GenerateNewKeyPair** - Create deterministic or random keypairs
- **FundNative** - Airdrop SOL (devnet/testnet only)

### Transaction Service v1
- **CompileTransaction** - DRAFT -> COMPILED state transition
- **SignTransaction** - COMPILED -> PARTIALLY_SIGNED/FULLY_SIGNED
- **SubmitTransaction** - FULLY_SIGNED -> SUBMITTED
- **EstimateTransaction** - Fee calculation
- **SimulateTransaction** - Dry run
- **GetTransaction** - Fetch by signature
- **StreamTransactionStatuses** - Real-time gRPC streaming

### System Program Service v1
- Returns `SolanaInstruction` messages for transaction composition
- Create, Transfer, Allocate, Assign, and more

### Token Program Service v1
- SPL Token and Token-2022 operations
- Mint initialization with optional Metaplex metadata
- Token account creation and management

### RPC Client Service v1
- Direct Solana RPC method wrappers
- Rent calculations, slot queries

## Running

```bash
# From repository root
cargo run -p protochain-solana-api

# With specific network
SOLANA_RPC_URL="https://api.devnet.solana.com" cargo run -p protochain-solana-api
```

## Testing

```bash
# Rust unit tests
cargo test -p protochain-solana-api

# Go integration tests (requires running validator + backend)
cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v
```

## Docker

See [ci/api/README.md](../../ci/api/README.md) for containerization details.
