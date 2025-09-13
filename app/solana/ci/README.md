# Protochain Solana API - Containerization

This directory contains Docker containerization assets for the Protochain Solana API.

## 🐳 Files Overview

- **`Dockerfile`** - Multi-stage build for production-ready container
- **`docker-build.sh`** - Helper script for building and running containers
- **`../../docker-compose.yml`** - Full stack orchestration (validator + API)
- **`../../.dockerignore`** - Optimizes build context

## 🚀 Quick Start

### Option 1: Docker Compose (Recommended for Development)

```bash
# Start the full stack (validator + API)
docker-compose up -d

# View logs
docker-compose logs -f protochain-api

# Run integration tests
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v

# Stop everything
docker-compose down
```

### Option 2: Helper Script

```bash
# Build the image
./app/solana/ci/docker-build.sh build

# Run with local validator
./app/solana/ci/docker-build.sh run local

# Run with devnet
./app/solana/ci/docker-build.sh run devnet

# Run with mainnet
./app/solana/ci/docker-build.sh run mainnet
```

### Option 3: Manual Docker Commands

```bash
# Build
docker build -f app/solana/ci/Dockerfile -t protochain-solana-api .

# Run with local validator
docker run -p 50051:50051 \
  -e SOLANA_RPC_URL=http://host.docker.internal:8899 \
  protochain-solana-api

# Run with devnet
docker run -p 50051:50051 \
  -e SOLANA_RPC_URL=https://api.devnet.solana.com \
  protochain-solana-api
```

## 🌐 Network Configurations

### Local Development
```bash
SOLANA_RPC_URL=http://host.docker.internal:8899  # For Docker Desktop
SOLANA_RPC_URL=http://172.17.0.1:8899           # For Linux Docker
```

### Public Networks
```bash
# Devnet
SOLANA_RPC_URL=https://api.devnet.solana.com

# Mainnet Beta
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com

# Custom RPC
SOLANA_RPC_URL=https://your-custom-rpc.com
```

## ⚙️ Environment Variables

### Solana Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `SOLANA_RPC_URL` | `http://host.docker.internal:8899` | Solana RPC endpoint |
| `SOLANA_TIMEOUT_SECONDS` | `30` | Request timeout |
| `SOLANA_RETRY_ATTEMPTS` | `3` | Retry attempts for failed requests |
| `SOLANA_HEALTH_CHECK_ON_STARTUP` | `true` | Perform RPC health check on start |

### Server Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `0.0.0.0` | gRPC server bind address |
| `SERVER_PORT` | `50051` | gRPC server port |

### Logging Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info,protochain_solana_api=info` | Log level filtering |
| `PROTOCHAIN_JSON_LOGS` | `true` | Enable JSON structured logging |

## 🏗️ Multi-Stage Build Details

### Stage 1: Builder (rust:1.80-bullseye)
- **Size**: ~2GB (includes full Rust toolchain)
- **Purpose**: Compiles the Rust application
- **Dependencies**: `pkg-config`, `libssl-dev`, `libudev-dev`, `build-essential`
- **Output**: Optimized release binary

### Stage 2: Runtime (gcr.io/distroless/cc-debian12)
- **Size**: ~50MB (minimal runtime)
- **Purpose**: Runs the compiled binary
- **Security**: Runs as non-root user (UID 65532)
- **Contents**: Only the binary and essential shared libraries

## 🔍 Health Checks & Monitoring

### Application Health
The application performs automatic health checks:
- **Startup Health Check**: Validates Solana RPC connection on startup
- **Configurable**: Set `SOLANA_HEALTH_CHECK_ON_STARTUP=false` to disable

### Container Health Check
Docker Compose includes health checks:
```yaml
healthcheck:
  test: ["CMD-SHELL", "timeout 5 bash -c '</dev/tcp/localhost/50051' || exit 1"]
  interval: 10s
  timeout: 5s
  retries: 5
```

## 🧪 Integration Test Compatibility

The container is designed to work with the existing integration test suite:

```bash
# Start the stack
docker-compose up -d

# Wait for services to be healthy
docker-compose ps

# Run tests
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v

# The tests will automatically discover the running services
```

## 📊 Resource Requirements

### Minimum Requirements
- **CPU**: 1 core
- **Memory**: 512MB
- **Disk**: 100MB (for image)

### Recommended for Production
- **CPU**: 2+ cores
- **Memory**: 1GB+
- **Disk**: 1GB+ (includes logging)

## 🛠️ Troubleshooting

### Common Issues

#### "No space left on device" during build
```bash
# Clean up Docker resources
docker system prune -f
docker volume prune -f
```

#### Container can't connect to local validator
```bash
# On macOS/Windows (Docker Desktop)
SOLANA_RPC_URL=http://host.docker.internal:8899

# On Linux
SOLANA_RPC_URL=http://172.17.0.1:8899
```

#### gRPC connection refused
```bash
# Check if container is running
docker ps --filter name=protochain-api

# Check logs
docker logs protochain-api

# Verify port binding
netstat -tulpn | grep 50051
```

### Debugging Commands

```bash
# View container logs
docker logs -f protochain-api

# Execute shell in running container (note: distroless has no shell)
# For debugging, temporarily change FROM gcr.io/distroless/cc-debian12
# to FROM debian:12-slim in Dockerfile

# Inspect container configuration
docker inspect protochain-api

# Test gRPC connectivity
grpc_health_probe -addr localhost:50051

# Test Solana RPC connectivity from container
docker exec protochain-api curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
  $SOLANA_RPC_URL
```

## 🚢 Production Deployment

### Registry Push
```bash
# Set registry
export DOCKER_REGISTRY=gcr.io/your-project

# Build and push
./app/solana/ci/docker-build.sh push
```

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: protochain-solana-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: protochain-solana-api
  template:
    metadata:
      labels:
        app: protochain-solana-api
    spec:
      containers:
      - name: protochain-solana-api
        image: gcr.io/your-project/protochain-solana-api:latest
        ports:
        - containerPort: 50051
        env:
        - name: SOLANA_RPC_URL
          value: "https://api.mainnet-beta.solana.com"
        - name: RUST_LOG
          value: "info,protochain_solana_api=info"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
```

## 📝 Configuration File Support

While the container is designed for environment variable configuration, you can also use configuration files:

```bash
# Create config.json
echo '{
  "solana": {
    "rpc_url": "https://api.devnet.solana.com",
    "timeout_seconds": 60,
    "retry_attempts": 5,
    "health_check_on_startup": true
  },
  "server": {
    "host": "0.0.0.0",
    "port": 50051
  }
}' > config.json

# Mount config file
docker run -p 50051:50051 \
  -v $(pwd)/config.json:/protochain/config.json \
  protochain-solana-api
```

## 🔗 Related Documentation

- **[Main README](../../../README.md)** - Project overview and architecture
- **[CLAUDE.md](../../../CLAUDE.md)** - Comprehensive development guide
- **[Integration Tests](../../../tests/go/)** - API usage examples