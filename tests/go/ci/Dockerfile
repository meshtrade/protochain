##############################################################################
# Multi-stage Dockerfile for Protochain Go Integration Tests
# Runs comprehensive integration tests against the Protochain Solana API
##############################################################################

##############################################################################
# Stage 1: Code generation with buf
##############################################################################
FROM ubuntu:latest AS codegen

WORKDIR /protochain

# Install curl
RUN apt-get update && \
    apt-get install -y \
    curl

# Install Go
RUN curl -L https://go.dev/dl/go1.24.4.linux-amd64.tar.gz -o golang.tar.gz
RUN tar -C /usr/local -xzf golang.tar.gz && \
    rm golang.tar.gz
ENV GOROOT=/usr/local/go
ENV PATH=$GOROOT/bin:$PATH

# Install Buf
RUN BIN="/usr/local/bin" && \
    VERSION="1.55.1" && \
    curl -sSL \
    "https://github.com/bufbuild/buf/releases/download/v${VERSION}/buf-$(uname -s)-$(uname -m)" \
    -o "${BIN}/buf" && \
    chmod +x "${BIN}/buf"

# Copy proto files and configuration
COPY lib/proto/ lib/proto/
COPY lib/_code_gen/ lib/_code_gen/
COPY buf.yaml .

# Generate only Go code from protobuf definitions
# Note: Using buf.gen.go.docker.yaml which only uses remote plugins (no local custom plugin)
# The custom protochaingo plugin is not needed for Docker builds - the standard protoc-gen-go
# and grpc-go plugins provide all the functionality needed for integration tests
RUN echo "🔍 Validating protobuf definitions..." && \
    buf lint lib/proto && \
    echo "✅ Protobuf definitions validated" && \
    echo "" && \
    echo "🏃 Generating Go code only from protobuf definitions..." && \
    buf generate lib/proto --template lib/_code_gen/buf.gen.go.docker.yaml && \
    echo "✅ Go code generation complete!"

##############################################################################
# Stage 2: Go test environment with all dependencies
##############################################################################
FROM golang:1.24-alpine AS tester

WORKDIR /protochain

# Copy workspace files for module resolution
COPY go.work .
COPY go.work.sum .

# Copy lib/go directory structure
COPY lib/go/ lib/go/

# Copy generated Go SDK from codegen stage
COPY --from=codegen /protochain/lib/go/ lib/go/

# Copy tests directory
COPY tests/ tests/

# Set working directory to tests/go for running tests
WORKDIR /protochain/tests/go

# Download Go module dependencies
RUN go mod download

# Default environment variables (can be overridden)
ENV RUN_INTEGRATION_TESTS=1

# Default command runs all integration tests
CMD ["go", "test", "-v", "-timeout", "5m"]
