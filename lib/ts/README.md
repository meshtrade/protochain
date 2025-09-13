# TypeScript API Bindings

This directory contains auto-generated TypeScript bindings for the Protochain gRPC APIs, generated using the [@bufbuild/es](https://buf.build/bufbuild/es) plugin.

## Installation

This package is part of a Yarn workspace. Install all dependencies from the repository root:

```bash
# From repository root
yarn install
```

## Building

Compile the TypeScript code using Yarn workspace commands:

```bash
# Build this workspace only
yarn workspace @protochain/api-ts build

# Or build all workspaces from root
yarn build
```

## Type Checking

Run TypeScript type checking without building:

```bash
# Type check this workspace only  
yarn workspace @protochain/api-ts typecheck

# Or type check all workspaces from root
yarn typecheck
```

## Usage

Import the generated types and services:

```typescript
import { 
  Account, 
  AccountServiceClient 
} from '@protochain/api-ts';

// Use the generated types and clients in your application
```

## Generated Files

The following files are auto-generated from protobuf definitions:
- `src/protochain/solana/account/v1/` - Account management types and services
- `src/protochain/solana/program/system/v1/` - System program types and services
- `src/protochain/solana/transaction/v1/` - Transaction types and services
- `src/protochain/solana/type/v1/` - Common types (commitment levels, keypairs)

## Development

### Generation

To regenerate the TypeScript bindings from protobuf definitions (run from repository root):

```bash
# Generate all language bindings (including TypeScript)
yarn generate

# Generate only TypeScript bindings  
yarn generate:ts

# Or use the direct scripts
./dev/generate/all.sh
./dev/generate/typescript.sh
```

### Cleaning

To clean generated files (run from repository root):

```bash
# Clean all generated files (all languages)
yarn clean:generated

# Clean only TypeScript generated files
yarn clean:ts

# Or use the direct scripts
./dev/clean/all.sh
./dev/clean/typescript.sh
```

## Dependencies

This package uses the following key dependencies:
- `@bufbuild/protobuf` - Protobuf runtime for TypeScript
- `@connectrpc/connect` - Connect RPC client for TypeScript
- `@connectrpc/connect-node` - Node.js support for Connect RPC
- `@connectrpc/connect-web` - Web browser support for Connect RPC