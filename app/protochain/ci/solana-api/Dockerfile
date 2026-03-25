##############################################################################
# Stage 1: Code generation with buf
##############################################################################
FROM bufbuild/buf:latest AS codegen

WORKDIR /protochain

COPY lib/proto/ lib/proto/
COPY lib/_code_gen/ lib/_code_gen/
COPY buf.yaml .

RUN echo "Validating protobuf definitions..." && \
    buf lint lib/proto && \
    echo "Protobuf definitions validated" && \
    echo "Generating Rust code from protobuf definitions..." && \
    buf generate lib/proto --template lib/_code_gen/buf.gen.rust.yaml && \
    echo "Rust code generation complete!"

##############################################################################
# Stage 2: Rust build
##############################################################################
FROM rust:latest AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libudev-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /protochain

COPY Cargo.toml .
COPY Cargo.lock .
COPY lib/rust/ lib/rust/
COPY --from=codegen /protochain/lib/rust/src/ lib/rust/src/
COPY app/solana/cmd/api/ app/solana/cmd/api/

# Work around spl-token-2022/spl-associated-token-account duplicate `entrypoint`
# symbol conflict. macOS linker tolerates this; Linux GNU ld does not.
ENV RUSTFLAGS="-C link-args=-Wl,--allow-multiple-definition"
RUN cargo build --release --package protochain-solana-api

##############################################################################
# Stage 3: Runtime with envoy
##############################################################################
FROM envoyproxy/envoy:v1.32-latest

# Copy the compiled Rust binary
COPY --from=builder /protochain/target/release/protochain-solana-api /usr/local/bin/protochain-solana-api

# Copy envoy config and entrypoint
COPY app/protochain/cmd/solana-api/config.yaml /etc/envoy/config.yaml
COPY app/protochain/cmd/solana-api/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Solana RPC configuration
ENV SOLANA_RPC_URL=http://host.docker.internal:8899
ENV SOLANA_WEBSOCKET_URL=wss://host.docker.internal:8899
ENV SOLANA_TIMEOUT_SECONDS=30
ENV SOLANA_RETRY_ATTEMPTS=3
ENV SOLANA_HEALTH_CHECK_ON_STARTUP=true

# Rust API listens internally on 50051
ENV SERVER_HOST=127.0.0.1
ENV SERVER_PORT=50051

# Envoy listens externally on 50064 (matches Go test defaults)
ENV ENVOY_ADDRESS=0.0.0.0
ENV ENVOY_PORT=50064

# Logging
ENV RUST_LOG=info,protochain_solana_api=info
ENV PROTOCHAIN_JSON_LOGS=true

EXPOSE 50064

ENTRYPOINT ["/entrypoint.sh"]
