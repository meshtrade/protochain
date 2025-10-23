##############################################################################
# Multi-stage Dockerfile for Protochain Go Integration Tests
# Runs comprehensive integration tests against the Protochain Solana API
##############################################################################

##############################################################################
# Stage 1: Code generation with buf
##############################################################################
FROM bufbuild/buf:latest AS codegen

WORKDIR /protochain

# Copy proto files and configuration
COPY lib/proto/ lib/proto/
COPY lib/_code_gen/ lib/_code_gen/
COPY buf.yaml .

# Generate only Go code from protobuf definitions
RUN echo "🔍 Validating protobuf definitions..." && \
    buf lint lib/proto && \
    echo "✅ Protobuf definitions validated" && \
    echo "" && \
    echo "🏃 Generating Go code only from protobuf definitions..." && \
    buf generate lib/proto --template lib/_code_gen/buf.gen.go.yaml && \
    echo "✅ Go code generation complete!"

##############################################################################
# Stage 2: Go test environment with all dependencies
##############################################################################
FROM golang:1.24-alpine AS tester

# Install required packages for Go testing
RUN apk add --no-cache \
    git \
    make \
    bash \
    curl \
    ca-certificates

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
ENV SOLANA_RPC_URL=http://solana-validator:8899
ENV BackendGRPCEndpoint=protochain-api
ENV BackendGRPCPort=50051
ENV BackendGRPCTLS=false
ENV RUN_INTEGRATION_TESTS=1

# Default command runs all integration tests
CMD ["go", "test", "-v", "-timeout", "5m"]

##############################################################################
# Usage Examples:
##############################################################################
#
# Build the test image:
# docker build -f tests/ci/Dockerfile -t protochain-tests .
#
# Run tests against full Docker Compose stack:
# docker run --rm \
#   --network protochain \
#   -e SOLANA_RPC_URL=http://solana-validator:8899 \
#   -e BackendGRPCEndpoint=protochain-api \
#   -e BackendGRPCPort=50051 \
#   protochain-tests
#
# Run specific test:
# docker run --rm \
#   --network protochain \
#   protochain-tests \
#   go test -v -run TestRpcClientE2ESuite/Test_01
#
# Run with local development setup:
# docker run --rm \
#   -e SOLANA_RPC_URL=http://host.docker.internal:8899 \
#   -e BackendGRPCEndpoint=host.docker.internal \
#   -e BackendGRPCPort=50051 \
#   protochain-tests
#
##############################################################################
