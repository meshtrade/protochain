# Protochain

**Protocol Buffer Wrapper for Solana SDKs**

ProtoChain provides a language-agnostic gRPC API layer over Solana blockchain operations. It wraps the best-in-class Solana SDKs (primarily Rust) with Protocol Buffer service definitions, enabling automatic SDK generation for any language.

[![Tests](https://img.shields.io/badge/Tests-All%20Passing-brightgreen.svg)](tests/)
[![Rust](https://img.shields.io/badge/Rust-30%2F30%20Unit%20Tests-brightgreen.svg)](api/)
[![Go](https://img.shields.io/badge/Go-Integration%20Tests-brightgreen.svg)](tests/go/)
[![Blockchain](https://img.shields.io/badge/Blockchain-Verified%20Integration-blue.svg)](#)

## 🎯 Mission

Addresses the challenge where your backend needs to be in one language, but the most mature Solana SDK is in Rust. ProtoChain provides:

- **Multi-Language SDK Generation**: Generate SDKs for Go, TypeScript, Rust, Python, etc.
- **Rust SDK Access**: Access Rust's Solana ecosystem via gRPC from any language
- **Streaming Transaction Monitoring**: gRPC streaming for real-time transaction status updates
- **Protocol Buffer Definitions**: All APIs defined in Protocol Buffers for consistency

## 🏗️ Architecture Overview

### Protocol-First Design
- **Source of Truth**: All APIs defined in `lib/proto/protochain/solana/` using Protocol Buffers
- **Versioning**: Every service is versioned (v1) for backward compatibility
- **Standards**: Follows Google AIP resource-oriented design patterns
- **Namespace**: `protochain.solana.[domain].v1` structure

### Composable Transaction Model
Implements a strict state machine for transaction lifecycle:
```
DRAFT → COMPILED → PARTIALLY_SIGNED → FULLY_SIGNED → SUBMITTED
```

### Multi-Language SDK Generation
- **Rust** (`lib/rust/`): Generated with tonic/prost for backend implementation
- **Go** (`lib/go/`): Generated with custom interfaces via protoc-gen-protochaingo
- **TypeScript** (`lib/ts/`): Generated with @bufbuild/protobuf for browser/Node.js

## 📁 Repository Structure

```
protochain/
├── lib/proto/                     # 🔥 PROTOCOL DEFINITIONS (Source of Truth)
│   └── protochain/solana/
│       ├── account/v1/           # Account management services
│       ├── transaction/v1/       # Transaction lifecycle services
│       ├── program/system/v1/    # System program wrappers
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
│   │               └── program/system/v1/ # System program conversions
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

### Account Service (`protochain.solana.account.v1`)
- **Account Retrieval**: Fetch account data with configurable commitment levels
- **Keypair Generation**: Create deterministic or random keypairs
- **Native Funding**: Airdrop SOL for development (devnet/testnet)

### Transaction Service (`protochain.solana.transaction.v1`)
- **Lifecycle Management**: Complete DRAFT→COMPILED→SIGNED→SUBMITTED flow
- **Fee Estimation**: Calculate transaction costs before submission
- **Simulation**: Dry-run transactions for validation
- **Real-time Monitoring**: Stream transaction status updates via gRPC

### System Program Service (`protochain.solana.program.system.v1`)
- **Account Creation**: Create new accounts with proper rent calculations
- **SOL Transfers**: Transfer native SOL between accounts
- **Space Allocation**: Allocate account storage space
- **Owner Assignment**: Change account ownership

### RPC Client Service (`protochain.solana.rpc_client.v1`)
- **Direct RPC Access**: Wrapper for raw Solana RPC methods
- **Rent Calculations**: Get minimum balance for rent exemption
- **Commitment Levels**: Support for processed/confirmed/finalized

## ✅ Test Coverage

Test suite includes unit tests and integration tests with local blockchain validation

### 🦀 Rust Unit Tests (30/30 ✅)
- Service implementations and business logic
- Transaction state machine validation
- Error handling and edge cases
- Protocol buffer conversions

### 🐹 Go Integration Tests
- **Local Blockchain Testing**: Creates accounts and submits transactions to local validator
- **Streaming Implementation**: Tests gRPC streaming transaction status updates
- **Multi-instruction Transactions**: Tests atomic transaction execution
- **Service Integration**: End-to-end API functionality testing

**Test Implementation:**
- Creates test accounts and verifies balances on local validator
- Submits transactions and monitors status via streaming APIs
- Tests transaction state machine transitions
- Validates system and token program functionality

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

#### Option 1: Docker Compose (Recommended)
```bash
# Start full stack (validator + API)
./scripts/tests/start-docker.sh

# Stop full stack
./scripts/tests/stop-docker.sh
```

#### Option 2: Hybrid Development (Most Common)
```bash
# Start only Solana validator in Docker
./scripts/tests/start-validator-docker.sh

# Start backend locally for development (restart as needed)
cargo run -p protochain-solana-api

# Stop validator when done
./scripts/tests/stop-validator-docker.sh
```

#### Option 3: Native Development
```bash
# Terminal 1: Start Solana validator
./scripts/tests/start-validator.sh

# Terminal 2: Start gRPC backend
./scripts/tests/start-backend.sh
```

2. **Make Proto Changes**
```bash
# Edit proto files in lib/proto/protochain/solana/
vim lib/proto/protochain/solana/account/v1/service.proto

# Validate and generate code
buf lint
./scripts/code-gen/generate/all.sh
```

3. **Implement & Test**
```bash
# Update Rust implementation
vim app/solana/cmd/api/src/api/account/v1/service_impl.rs

# Run tests
cargo test                    # Rust unit tests
cd tests/go && go test -v     # Go integration tests (auto-detects services)
```

4. **Try Template App**
```bash
# Run the template app to understand the structure
go run ./app/template/cmd/some-executable/main.go

# Test with arguments
go run ./app/template/cmd/some-executable/main.go test arg
```

5. **Quality Assurance**
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

1. Read `CLAUDE.md` for comprehensive development guidelines
2. Follow the protocol-first development workflow
3. Ensure all tests pass before committing
4. Run mandatory linting: `./scripts/lint/all.sh`
5. Verify blockchain integration with integration tests

## 🏆 Current Status

- All unit tests passing (30/30)
- Integration tests passing with local Solana validator
- gRPC streaming implementation functional
- Multi-language SDK generation working
- Transaction state machine implemented and tested
