# ProtoChain (ProtoSol)

**Protocol Buffer Wrapper for Solana SDKs**

ProtoChain provides a language-agnostic gRPC API layer over Solana blockchain operations. It wraps the best-in-class Solana SDKs (primarily Rust) with Protocol Buffer service definitions, enabling automatic SDK generation for any language.

[![Tests](https://img.shields.io/badge/Tests-All%20Passing-brightgreen.svg)](tests/)
[![Rust](https://img.shields.io/badge/Rust-30%2F30%20Unit%20Tests-brightgreen.svg)](api/)
[![Go](https://img.shields.io/badge/Go-Integration%20Tests-brightgreen.svg)](tests/go/)
[![Blockchain](https://img.shields.io/badge/Blockchain-Verified%20Integration-blue.svg)](#)

## 🎯 Mission

Solve the fundamental challenge where your backend needs to be in one language, but the best Solana SDK is in another. ProtoChain enables you to:

- **Build in Any Language**: Generate SDKs for Go, TypeScript, Rust, Python, etc.
- **Use Best-in-Class SDKs**: Leverage Rust's mature Solana ecosystem via gRPC
- **Scale Production Systems**: Battle-tested streaming architecture with comprehensive monitoring
- **Develop Protocol-First**: All APIs defined in Protocol Buffers for consistency

## 🏗️ Architecture Overview

### Protocol-First Design
- **Source of Truth**: All APIs defined in `lib/proto/protosol/solana/` using Protocol Buffers
- **Versioning**: Every service is versioned (v1) for backward compatibility
- **Standards**: Follows Google AIP resource-oriented design patterns
- **Namespace**: `protosol.solana.[domain].v1` structure

### Composable Transaction Model
Implements a strict state machine for transaction lifecycle:
```
DRAFT → COMPILED → PARTIALLY_SIGNED → FULLY_SIGNED → SUBMITTED
```

### Multi-Language SDK Generation
- **Rust** (`lib/rust/`): Generated with tonic/prost for backend implementation
- **Go** (`lib/go/`): Generated with custom interfaces via protoc-gen-protosolgo
- **TypeScript** (`lib/ts/`): Generated with @bufbuild/protobuf for browser/Node.js

## 📁 Repository Structure

```
protochain/
├── lib/proto/                     # 🔥 PROTOCOL DEFINITIONS (Source of Truth)
│   └── protosol/solana/
│       ├── account/v1/           # Account management services
│       ├── transaction/v1/       # Transaction lifecycle services
│       ├── program/system/v1/    # System program wrappers
│       └── type/v1/              # Shared type definitions
│
├── api/                          # 🦀 Rust gRPC Backend Implementation
│   └── src/
│       ├── main.rs              # gRPC server (port 50051)
│       └── api/                 # Service implementations
│           ├── account/v1/      # Account service logic
│           ├── transaction/v1/  # Transaction state machine
│           └── program/system/v1/ # System program conversions
│
├── lib/                         # 📦 Generated Multi-Language SDKs
│   ├── rust/src/               # Generated Rust bindings
│   ├── go/protosol/           # Generated Go SDK + interfaces
│   └── ts/src/               # Generated TypeScript SDK
│
├── tests/go/                   # 🧪 Integration Test Suite
│   ├── streaming_e2e_test.go  # Real blockchain integration tests
│   ├── token_program_e2e_test.go # Token program testing
│   └── rpc_client_e2e_test.go # RPC client validation
│
├── scripts/                    # 🔧 Development Automation
│   ├── code-gen/generate/all.sh # Generate all SDKs
│   ├── tests/start-validator.sh # Local Solana validator
│   ├── tests/start-backend.sh  # Start gRPC backend
│   └── lint/                   # Code quality scripts
│
└── CLAUDE.md                   # 📖 Comprehensive development guide
```

## 🚀 Key Features & Services

### Account Service (`protosol.solana.account.v1`)
- **Account Retrieval**: Fetch account data with configurable commitment levels
- **Keypair Generation**: Create deterministic or random keypairs
- **Native Funding**: Airdrop SOL for development (devnet/testnet)

### Transaction Service (`protosol.solana.transaction.v1`)
- **Lifecycle Management**: Complete DRAFT→COMPILED→SIGNED→SUBMITTED flow
- **Fee Estimation**: Calculate transaction costs before submission
- **Simulation**: Dry-run transactions for validation
- **Real-time Monitoring**: Stream transaction status updates via gRPC

### System Program Service (`protosol.solana.program.system.v1`)
- **Account Creation**: Create new accounts with proper rent calculations
- **SOL Transfers**: Transfer native SOL between accounts
- **Space Allocation**: Allocate account storage space
- **Owner Assignment**: Change account ownership

### RPC Client Service (`protosol.solana.rpc_client.v1`)
- **Direct RPC Access**: Wrapper for raw Solana RPC methods
- **Rent Calculations**: Get minimum balance for rent exemption
- **Commitment Levels**: Support for processed/confirmed/finalized

## ✅ Production-Ready Testing

**All Tests Passing**: Comprehensive test coverage with real blockchain integration

### 🦀 Rust Unit Tests (30/30 ✅)
- Service implementations and business logic
- Transaction state machine validation
- Error handling and edge cases
- Protocol buffer conversions

### 🐹 Go Integration Tests (All Suites ✅)
- **Real Blockchain Testing**: Creates actual accounts, submits real transactions
- **Streaming Validation**: Tests real-time transaction monitoring
- **Multi-instruction Atomic Transactions**: Verifies Solana's atomic execution
- **Complete Workflows**: End-to-end user journey testing

**Recent Test Run Results:**
- 4 accounts created with verified balances
- 4 transactions submitted and finalized on blockchain
- All streaming notifications working correctly
- Token program functionality fully validated

## 🛠️ Quick Start

### Prerequisites
```bash
# Required tools
rustc --version    # Rust 1.70+
go version         # Go 1.21+
solana --version   # Solana CLI tools
buf --version      # Protocol buffer tools
```

### Development Workflow

1. **Start Local Environment**
```bash
# Terminal 1: Start Solana validator
./scripts/tests/start-validator.sh

# Terminal 2: Start gRPC backend
./scripts/tests/start-backend.sh
```

2. **Make Proto Changes**
```bash
# Edit proto files in lib/proto/protosol/solana/
vim lib/proto/protosol/solana/account/v1/service.proto

# Validate and generate code
buf lint
./scripts/code-gen/generate/all.sh
```

3. **Implement & Test**
```bash
# Update Rust implementation
vim api/src/api/account/v1/service_impl.rs

# Run tests
cargo test                    # Rust unit tests
cd tests/go && go test -v     # Go integration tests (auto-detects services)
```

4. **Quality Assurance**
```bash
# MANDATORY: Run linting after ANY code changes
./scripts/lint/all.sh         # All languages
./scripts/lint/rs.sh          # Rust only
./scripts/lint/go.sh          # Go only
```

## 🎯 Key Design Principles

- **Protocol-First**: Proto definitions drive all development
- **State Machine Integrity**: Strict transaction lifecycle enforcement
- **Production Quality**: Comprehensive testing with real blockchain integration
- **Multi-Language**: Generate clean, idiomatic SDKs for any language
- **Streaming Architecture**: Real-time transaction monitoring capabilities
- **Error Resilience**: Robust error handling and graceful degradation

## 📚 Documentation

- **[CLAUDE.md](CLAUDE.md)**: Comprehensive development guide with workflows, patterns, and troubleshooting
- **[Integration Tests](tests/go/)**: Live examples of API usage with blockchain integration
- **[Proto Definitions](lib/proto/)**: Complete API specification and data models

## 🤝 Contributing

1. Read `CLAUDE.md` for comprehensive development guidelines
2. Follow the protocol-first development workflow
3. Ensure all tests pass before committing
4. Run mandatory linting: `./scripts/lint/all.sh`
5. Verify blockchain integration with integration tests

## 🏆 Status

**Production Ready** - All tests passing, comprehensive blockchain integration verified, streaming architecture functional, multi-language SDKs generated and tested.
