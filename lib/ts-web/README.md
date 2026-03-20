# @protochain/ts-web

TypeScript SDK for Protochain gRPC APIs, optimized for browser and Node.js usage via Connect-Web.

[![npm](https://img.shields.io/npm/v/@protochain/ts-web)](https://www.npmjs.com/package/@protochain/ts-web)

## Installation

```bash
npm install @protochain/ts-web
# or
yarn add @protochain/ts-web
```

## Usage

```typescript
import {
  AccountServiceWebClient,
} from '@protochain/ts-web/solana/account/v1';

import {
  TransactionServiceWebClient,
} from '@protochain/ts-web/solana/transaction/v1';
```

## Generated Files

Auto-generated from protobuf definitions in `lib/proto/protochain/solana/`:

- `src/protochain/solana/account/v1/` - Account management types and services
- `src/protochain/solana/transaction/v1/` - Transaction lifecycle types and services
- `src/protochain/solana/program/system/v1/` - System program types and services
- `src/protochain/solana/program/token/v1/` - SPL Token & Token-2022 types and services
- `src/protochain/solana/rpc_client/v1/` - Direct RPC client types and services
- `src/protochain/solana/type/v1/` - Common types (commitment levels, keypairs, token programs)

## Development

### Regenerating from protobuf

```bash
# From repository root - generates only TypeScript bindings
./scripts/code-gen/generate/ts-web.sh

# Or generate all languages (Go, Rust, TypeScript)
./scripts/code-gen/generate/all.sh
```

### Building

```bash
cd lib/ts-web
yarn build
```

### Linting & Type Checking

```bash
yarn lint
yarn typecheck
```

## Dependencies

- `@bufbuild/protobuf` - Protobuf runtime for TypeScript
- `@connectrpc/connect` - Connect RPC client
- `@connectrpc/connect-web` - Web browser transport for Connect RPC

## Publishing

Published automatically via GitHub Actions when a `ts-web/v*.*.*` tag is pushed. Uses npm OIDC Trusted Publishing (no long-lived tokens).

```bash
# To publish a new version:
git tag ts-web/v1.0.0
git push origin ts-web/v1.0.0
```
