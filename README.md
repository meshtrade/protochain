# Protochain

**Protocol Buffer Wrapper for Blockchain SDKs**

ProtoChain provides a language-agnostic gRPC API layer over blockchain operations. It wraps SDKs with Protocol Buffer service definitions, enabling automatic SDK generation for any language. Furthermore, protochain also provides the infrastructure scaffolding by providing an app per blockchain, allowing you to spin up your own protochain API server for your blockchain of choice.

[![Tests](https://img.shields.io/badge/Tests-All%20Passing-brightgreen.svg)](tests/)
[![Rust](https://img.shields.io/badge/Rust-30%2F30%20Unit%20Tests-brightgreen.svg)](api/)
[![Go](https://img.shields.io/badge/Go-Integration%20Tests-brightgreen.svg)](tests/go/)
[![Blockchain](https://img.shields.io/badge/Blockchain-Verified%20Integration-blue.svg)](#)

## 🎯 Mission

Addresses the challenge where your backend needs to be in one language, but your chosen blockchains have SDKs in another. ProtoChain provides:

- **Multi-Language SDK Generation**: Generate SDKs for Go, TypeScript, Rust, Python, etc.
- **Streaming Transaction Monitoring**: gRPC streaming for real-time transaction status updates
- **Protocol Buffer Definitions**: All APIs defined in Protocol Buffers for consistency

### Supported Blockchains

- Solana

## 🏗️ Architecture Overview

### Protocol-First Design
- **Source of Truth**: All APIs defined in `lib/proto/protochain/` using Protocol Buffers
- **Versioning**: Every service is versioned (v1) for backward compatibility
- **Standards**: Follows Google AIP resource-oriented design patterns
- **Namespace**: `protochain.[blockchain].[domain].v1` structure

### Composable Transaction Model
Implements a strict state machine for transaction lifecycle:
```
DRAFT → COMPILED → PARTIALLY_SIGNED → FULLY_SIGNED → SUBMITTED
```

### Multi-Language SDK Generation
- **Rust** (`lib/rust/`): Generated with tonic/prost for backend implementation
- **Go** (`lib/go/`): Generated with custom interfaces via protoc-gen-protochaingo
- **TypeScript** (`lib/ts-web/`): Generated with @bufbuild/protobuf for browser/Node.js ([`@protochain/ts-web`](https://www.npmjs.com/package/@protochain/ts-web))

## 📁 Repository Structure

```
protochain/
├── lib/proto/                     # 🔥 PROTOCOL DEFINITIONS (Source of Truth)
│   └── protochain/solana/
│       ├── account/v1/           # Account management services
│       ├── transaction/v1/       # Transaction lifecycle services
│       ├── program/
│       │   ├── system/v1/       # System program wrappers
│       │   └── token/v1/        # SPL Token & Token-2022 wrappers
│       ├── rpc_client/v1/       # Direct RPC client service
│       └── type/v1/              # Shared type definitions
│
├── app/                          # 🏗️ Multi-App Architecture
│   ├── solana/                  # Solana blockchain applications
│   │   └── cmd/
│   │       └── api/             # 🦀 Rust gRPC Backend
│   │           ├── src/main.rs  # gRPC server (port 50051)
│   │           └── src/api/     # Service implementations
│   │               ├── account/v1/      # Account service logic
│   │               ├── transaction/v1/  # Transaction state machine
│   │               ├── program/
│   │               │   ├── system/v1/  # System program conversions
│   │               │   └── token/v1/   # Token program operations
│   │               └── rpc_client/v1/  # Direct RPC operations
│   │
│   └── template/               # Template for new applications
│       └── cmd/
│           └── some-executable/ # 🐹 Go template app (template-some-executable)
│               ├── main.go     # Working Go executable
│               ├── go.mod      # Independent Go module
│               └── README.md   # Usage documentation
│
├── lib/                         # 📦 Generated Multi-Language SDKs
│   ├── rust/src/               # Generated Rust bindings
│   ├── go/protochain/           # Generated Go SDK + interfaces
│   └── ts-web/src/              # Generated TypeScript SDK (@protochain/ts-web)
│
├── tests/go/                   # 🧪 Integration Test Suite
│   ├── streaming_e2e_test.go  # Real blockchain integration tests
│   ├── token_program_e2e_test.go # Token program testing
│   └── rpc_client_e2e_test.go # RPC client validation
│
├── scripts/                    # 🔧 Development Automation
│   ├── code-gen/generate/all.sh # Generate all SDKs
│   ├── tests/start-backend.sh  # Start gRPC backend natively
│   └── lint/                   # Code quality scripts
│
└── CLAUDE.md                   # 📖 Comprehensive development guide
```

## 🏗️ Multi-App Architecture

ProtoChain features a **multi-app architecture** that allows multiple applications to coexist in the same repository:

### App Naming Convention
- **Pattern**: `{app-type}-{executable-name}`
- **Location**: `./app/{app-type}/cmd/{executable-name}/`
- **Example**: `template-some-executable` located at `./app/template/cmd/some-executable/`

### Current Applications

#### 🦀 **Solana API** (`solana-api`)
- **Location**: `./app/solana/cmd/api/`
- **Package**: `protochain-solana-api`
- **Description**: Complete Rust gRPC backend for Solana blockchain operations
- **Features**: All ProtoChain services (Account, Transaction, System Program, RPC Client)

#### 🐹 **Template App** (`template-some-executable`)
- **Location**: `./app/template/cmd/some-executable/`
- **Package**: `template-some-executable`
- **Description**: Template Go executable demonstrating app structure
- **Purpose**: Starting point for new applications

### Adding New Applications
1. Create directory: `./app/{type}/cmd/{name}/`
2. Follow naming convention: `{type}-{name}`
3. Implement according to application type (Go, Rust, etc.)
4. Add to workspace configuration if needed

## 🚀 Key Features & Services

### Solana Specific

#### Account Service (`protochain.solana.account.v1`)
- **Account Retrieval**: Fetch account data with configurable commitment levels
- **Keypair Generation**: Create deterministic or random keypairs
- **Native Funding**: Airdrop SOL for development (devnet/testnet)

#### Transaction Service (`protochain.solana.transaction.v1`)
- **Lifecycle Management**: Complete DRAFT→COMPILED→SIGNED→SUBMITTED flow
- **Fee Estimation**: Calculate transaction costs before submission
- **Simulation**: Dry-run transactions for validation
- **Real-time Monitoring**: Stream transaction status updates via gRPC

#### System Program Service (`protochain.solana.program.system.v1`)
- **Account Creation**: Create new accounts with proper rent calculations
- **SOL Transfers**: Transfer native SOL between accounts
- **Space Allocation**: Allocate account storage space
- **Owner Assignment**: Change account ownership

#### RPC Client Service (`protochain.solana.rpc_client.v1`)
- **Direct RPC Access**: Wrapper for raw Solana RPC methods
- **Rent Calculations**: Get minimum balance for rent exemption
- **Commitment Levels**: Support for processed/confirmed/finalized

## 🛠️ Quick Start

### Prerequisites
```bash
# Required tools
rustc --version    # Rust 1.70+
go version         # Go 1.21+
docker --version   # Docker (for surfpool validator)
buf --version      # Protocol buffer tools
```

### Running the Test Environment

**Option A: Full Stack (just want to run tests)**
```bash
# Start surfpool validator + envoy + API
docker compose up -d

# Run integration tests
cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v

# Stop everything
docker compose down
```

**Option B: Hybrid Development (iterating on the Rust backend)**
```bash
# Start only the surfpool validator
docker compose up surfpool -d

# Run backend locally (restart freely during development)
cargo run -p protochain-solana-api

# Run tests
cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v

# Stop surfpool when done
docker compose down
```

### Development Workflow

1. **Make Proto Changes**
```bash
vim lib/proto/protochain/solana/account/v1/service.proto
buf lint
./scripts/code-gen/generate/all.sh
```

2. **Implement & Test**
```bash
vim app/solana/cmd/api/src/api/account/v1/service_impl.rs
cargo test                    # Rust unit tests
cd tests/go && go test -v     # Go integration tests
```

3. **Quality Assurance**
```bash
# MANDATORY: Run linting after ANY code changes
./scripts/lint/all.sh         # All languages
./scripts/lint/rs.sh          # Rust only
./scripts/lint/go.sh          # Go only
```

## 🎯 Technical Design

- **Protocol-First**: Proto definitions drive all development
- **State Machine**: Enforces transaction lifecycle transitions
- **Testing**: Unit tests and integration tests with local blockchain
- **Multi-Language**: Generates SDKs for multiple programming languages
- **Streaming**: gRPC streaming for transaction monitoring
- **Error Handling**: Structured error responses via gRPC Status

## 📚 Documentation

- **[CLAUDE.md](CLAUDE.md)**: Comprehensive development guide with workflows, patterns, and troubleshooting
- **[Integration Tests](tests/go/)**: Live examples of API usage with blockchain integration
- **[Proto Definitions](lib/proto/)**: Complete API specification and data models

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)
