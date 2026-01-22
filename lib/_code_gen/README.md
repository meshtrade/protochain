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
- `lib/ts/src/` - TypeScript SDK (es module)

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

### `buf.gen.ts.yaml` (TypeScript Only)
Generates only the **TypeScript SDK** using es modules.

```bash
buf generate lib/proto --template lib/_code_gen/buf.gen.ts.yaml
```

**Output:**
- `lib/ts/src/` - TypeScript SDK

**Use Case:** Frontend development, Browser clients

## Usage in Scripts

To use language-specific generation in scripts:

```bash
#!/bin/bash
# For Rust only (faster)
buf generate lib/proto --template lib/_code_gen/buf.gen.rust.yaml

# For Go only (testing)
buf generate lib/proto --template lib/_code_gen/buf.gen.go.yaml

# For TypeScript only (frontend)
buf generate lib/proto --template lib/_code_gen/buf.gen.ts.yaml

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
- Proto source files are in `lib/proto/protosol/solana/`
- Generated files should never be manually edited (regenerate instead)
- Each language has its own output directory to avoid conflicts
