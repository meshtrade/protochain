##############################################################################
# Stage 1: Code generation with buf
##############################################################################
FROM bufbuild/buf:latest AS codegen

# Set working directory
WORKDIR /protochain

# Copy proto files and configuration
COPY lib/proto/ lib/proto/
COPY lib/_code_gen/ lib/_code_gen/
COPY buf.yaml .

# Run code generation for Rust only
RUN echo "🔍 Validating protobuf definitions..." && \
    buf lint lib/proto && \
    echo "✅ Protobuf definitions validated" && \
    echo "" && \
    echo "🏃 Generating Rust code from protobuf definitions..." && \
    buf generate --config lib/_code_gen/buf.gen.rust.yaml && \
    echo "✅ Rust code generation complete!"

##############################################################################
# Stage 2: Heavy Rust build environment
##############################################################################
FROM rust:latest AS builder

# Install system dependencies for Solana and crypto libraries
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libudev-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set working directory to match repo structure
WORKDIR /protochain

# Copy workspace configuration files first for better layer caching
COPY Cargo.toml .
COPY Cargo.lock .

# Copy the Rust SDK structure (Cargo.toml and other source files)
COPY lib/rust/ lib/rust/

# Copy generated lib directory from codegen stage (overlays generated src files)
COPY --from=codegen /protochain/lib/rust/src/ lib/rust/src/

# Copy the specific app directory we're building
COPY app/solana/cmd/api/ app/solana/cmd/api/

# Build the application in release mode
# Use specific package name to build just the Solana API
RUN cargo build --release --package protochain-solana-api

# Verify the binary was created where expected
RUN ls -la target/release/

##############################################################################
# Stage 3: Slim distroless runtime environment
##############################################################################
FROM gcr.io/distroless/cc-debian12:latest

# Create non-root user for security
# Note: distroless images come with a nonroot user (UID 65532)
USER 65532:65532

# Copy the compiled binary from builder stage
COPY --from=builder /protochain/target/release/protochain-solana-api /usr/local/bin/protochain-solana-api

# Expose the default gRPC port
EXPOSE 50051

# Environment variable configuration with sensible defaults
# Solana RPC Configuration
ENV SOLANA_RPC_URL=http://host.docker.internal:8899
ENV SOLANA_WEBSOCKET_URL=wss://host.docker.internal:8899
ENV SOLANA_TIMEOUT_SECONDS=30
ENV SOLANA_RETRY_ATTEMPTS=3
ENV SOLANA_HEALTH_CHECK_ON_STARTUP=true

# Server Configuration
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=50051

# Logging Configuration
ENV RUST_LOG=info,protochain_solana_api=info
ENV PROTOCHAIN_JSON_LOGS=true

# Health check to verify the service is running
# Note: distroless doesn't have curl, so we can't do HTTP health checks
# The application itself provides startup health checks via SOLANA_HEALTH_CHECK_ON_STARTUP

# Labels for better container management
LABEL org.opencontainers.image.title="Protochain Solana API"
LABEL org.opencontainers.image.description="Protocol Buffer wrapper for Solana blockchain operations via gRPC"
LABEL org.opencontainers.image.vendor="Protochain"
LABEL org.opencontainers.image.version="0.1.0"
LABEL org.opencontainers.image.source="https://github.com/meshtrade/protochain"

EXPOSE 8080

# Default command
ENTRYPOINT ["/usr/local/bin/protochain-solana-api"]

##############################################################################
# Usage Examples:
##############################################################################
#
# Local Development (connects to host validator):
# docker build -f app/solana/ci/Dockerfile -t protochain-solana-api .
# docker run -p 50051:50051 protochain-solana-api
#
# Devnet Configuration:
# docker run -p 50051:50051 \
#   -e SOLANA_RPC_URL=https://api.devnet.solana.com \
#   -e SOLANA_HEALTH_CHECK_ON_STARTUP=true \
#   protochain-solana-api
#
# Mainnet Beta Configuration:
# docker run -p 50051:50051 \
#   -e SOLANA_RPC_URL=https://api.mainnet-beta.solana.com \
#   -e SOLANA_TIMEOUT_SECONDS=60 \
#   -e SOLANA_RETRY_ATTEMPTS=5 \
#   protochain-solana-api
#
# Custom Configuration with Separate WebSocket URL:
# docker run -p 50051:50051 \
#   -e SOLANA_RPC_URL=https://your-custom-rpc.com \
#   -e SOLANA_WEBSOCKET_URL=wss://your-custom-ws.com \
#   -e SERVER_PORT=8080 \
#   -e RUST_LOG=debug \
#   -e PROTOCHAIN_JSON_LOGS=false \
#   protochain-solana-api
#
# Integration Test Compatibility:
# docker run -p 50051:50051 \
#   -e SOLANA_RPC_URL=http://host.docker.internal:8899 \
#   -e SOLANA_HEALTH_CHECK_ON_STARTUP=false \
#   --name protochain-api \
#   protochain-solana-api
#
##############################################################################