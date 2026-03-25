# Code Generation Configurations

This directory contains buf configuration files for generating language-specific SDKs from Protocol Buffer definitions.

## Available Configurations

### `buf.gen.yaml` (All Languages)
Generates code for **all supported languages**: Rust, Go, and TypeScript.

```bash
buf generate lib/proto --template lib/_code_gen/buf.gen.yaml
```

**Output:**
- `lib/rust/src/` - Rust SDK (prost + tonic)
- `lib/go/` - Go SDK (protobuf + gRPC + custom interfaces)
- `lib/ts-web/src/` - TypeScript SDK (es module + ServiceWeb client wrappers)

### `buf.gen.rust.yaml` (Rust Only)
Generates only the **Rust SDK** using prost and tonic.

```bash
buf generate lib/proto --template lib/_code_gen/buf.gen.rust.yaml
```

**Output:**
- `lib/rust/src/` - Rust SDK

**Use Case:** Docker builds for the Rust backend (faster, no external dependencies)

### `buf.gen.go.yaml` (Go Only)
Generates only the **Go SDK** with protobuf, gRPC, and custom interfaces.

```bash
buf generate lib/proto --template lib/_code_gen/buf.gen.go.yaml
```

**Output:**
- `lib/go/` - Go SDK with clean interfaces

**Use Case:** Integration testing, Go client development

### `buf.gen.ts-web.yaml` (TypeScript Only)
Generates only the **TypeScript SDK** with ES modules and ServiceWeb client wrappers.

```bash
buf generate lib/proto --template lib/_code_gen/buf.gen.ts-web.yaml
```

**Output:**
- `lib/ts-web/src/` - TypeScript SDK (`@protochain/ts-web`)

**Use Case:** CI/CD npm publishing (avoids generating Go/Rust in the deploy pipeline)

## Usage in Scripts

To use language-specific generation in scripts:

```bash
#!/bin/bash
# For Rust only (faster)
buf generate lib/proto --template lib/_code_gen/buf.gen.rust.yaml

# For Go only (testing)
buf generate lib/proto --template lib/_code_gen/buf.gen.go.yaml

# For all languages (complete)
buf generate lib/proto --template lib/_code_gen/buf.gen.yaml
```

## Docker Builds

For containerized API builds, use Rust-only generation:

```dockerfile
RUN buf generate lib/proto --template lib/_code_gen/buf.gen.rust.yaml
```

This is already configured in `app/solana/ci/api/Dockerfile`.

## Adding New Languages

To add a new language:

1. Add the plugin to `buf.gen.yaml` under the appropriate section
2. Create a new `buf.gen.[language].yaml` file with just that language's plugins
3. Update this README with usage instructions
4. Optionally create a corresponding script in `scripts/code-gen/`

## Notes

- All configurations validate proto files with `buf lint` before generation
- Proto source files are in `lib/proto/protochain/solana/`
- Generated files should never be manually edited (regenerate instead)
- Each language has its own output directory to avoid conflicts
