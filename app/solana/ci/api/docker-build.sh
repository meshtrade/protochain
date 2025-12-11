#!/bin/bash
set -euo pipefail

# Protochain Solana API - Docker Build and Run Helper Script
# This script simplifies building and running the containerized Solana API

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../" && pwd)"

# Default configuration
DOCKER_TAG="${DOCKER_TAG:-protochain-solana-api}"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-}"
SOLANA_RPC_URL="${SOLANA_RPC_URL:-http://host.docker.internal:8899}"
SERVER_PORT="${SERVER_PORT:-50051}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

usage() {
    cat << EOF
Usage: $0 [COMMAND] [OPTIONS]

Commands:
    build       Build the Docker image
    run         Run the container (builds if needed)
    push        Push to registry (requires DOCKER_REGISTRY)
    clean       Remove local image and containers
    help        Show this help message

Configuration Options (environment variables):
    DOCKER_TAG               Docker image tag (default: protochain-solana-api)
    DOCKER_REGISTRY          Registry to push to (e.g., gcr.io/your-project)
    SOLANA_RPC_URL           Solana RPC endpoint (default: http://host.docker.internal:8899)
    SERVER_PORT              gRPC server port (default: 50051)

Network Presets:
    local       Uses host.docker.internal:8899 (for local validator)
    devnet      Uses https://api.devnet.solana.com
    mainnet     Uses https://api.mainnet-beta.solana.com

Examples:
    # Build for local development
    $0 build

    # Run with local validator
    $0 run local

    # Run with devnet
    $0 run devnet

    # Run with mainnet
    $0 run mainnet

    # Run with custom RPC
    SOLANA_RPC_URL=https://your-rpc.com $0 run

    # Build and push to registry
    DOCKER_REGISTRY=gcr.io/your-project $0 push

EOF
}

check_prerequisites() {
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed or not in PATH"
    fi

    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        error "Cannot find Cargo.toml. Please run this script from the project root."
    fi

    log "Prerequisites check passed"
}

build_image() {
    log "Building Docker image: $DOCKER_TAG"

    cd "$PROJECT_ROOT"

    # Check if dockerfile exists
    if [[ ! -f "app/solana/ci/Dockerfile" ]]; then
        error "Dockerfile not found at app/solana/ci/Dockerfile"
    fi

    # Build with proper context from project root
    docker build \
        -f app/solana/ci/Dockerfile \
        -t "$DOCKER_TAG" \
        --label "build-date=$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
        --label "git-commit=$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')" \
        .

    log "Docker image built successfully: $DOCKER_TAG"
}

run_container() {
    local network_preset="${1:-}"

    # Set RPC URL based on preset
    case "$network_preset" in
        "local")
            SOLANA_RPC_URL="http://host.docker.internal:8899"
            log "Using local validator configuration"
            ;;
        "devnet")
            SOLANA_RPC_URL="https://api.devnet.solana.com"
            log "Using Solana devnet configuration"
            ;;
        "mainnet")
            SOLANA_RPC_URL="https://api.mainnet-beta.solana.com"
            log "Using Solana mainnet-beta configuration"
            ;;
        "")
            log "Using configured RPC URL: $SOLANA_RPC_URL"
            ;;
        *)
            error "Unknown network preset: $network_preset. Use: local, devnet, mainnet"
            ;;
    esac

    # Check if image exists, build if it doesn't
    if ! docker image inspect "$DOCKER_TAG" &> /dev/null; then
        warn "Image $DOCKER_TAG not found, building..."
        build_image
    fi

    log "Starting container with:"
    log "  - RPC URL: $SOLANA_RPC_URL"
    log "  - Server port: $SERVER_PORT"
    log "  - Image: $DOCKER_TAG"

    # Remove existing container if it exists
    docker rm -f protochain-api &> /dev/null || true

    # Run the container
    docker run -d \
        --name protochain-api \
        -p "$SERVER_PORT:50051" \
        -e "SOLANA_RPC_URL=$SOLANA_RPC_URL" \
        -e "SERVER_HOST=0.0.0.0" \
        -e "SERVER_PORT=50051" \
        -e "RUST_LOG=info,protochain_solana_api=info" \
        -e "PROTOCHAIN_JSON_LOGS=true" \
        --restart unless-stopped \
        "$DOCKER_TAG"

    log "Container started successfully!"
    log "gRPC endpoint available at: localhost:$SERVER_PORT"

    # Show container status
    sleep 2
    docker ps --filter name=protochain-api --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

    log "View logs with: docker logs -f protochain-api"
    log "Stop container with: docker stop protochain-api"
}

push_image() {
    if [[ -z "$DOCKER_REGISTRY" ]]; then
        error "DOCKER_REGISTRY environment variable must be set for push command"
    fi

    local full_tag="$DOCKER_REGISTRY/$DOCKER_TAG"

    log "Tagging image for registry: $full_tag"
    docker tag "$DOCKER_TAG" "$full_tag"

    log "Pushing to registry: $full_tag"
    docker push "$full_tag"

    log "Push completed successfully"
}

clean() {
    log "Cleaning up Docker resources..."

    # Stop and remove container
    docker stop protochain-api &> /dev/null || true
    docker rm protochain-api &> /dev/null || true

    # Remove image
    docker rmi "$DOCKER_TAG" &> /dev/null || true

    # Remove registry-tagged image if it exists
    if [[ -n "$DOCKER_REGISTRY" ]]; then
        docker rmi "$DOCKER_REGISTRY/$DOCKER_TAG" &> /dev/null || true
    fi

    log "Cleanup completed"
}

main() {
    local command="${1:-help}"

    case "$command" in
        "build")
            check_prerequisites
            build_image
            ;;
        "run")
            check_prerequisites
            run_container "${2:-}"
            ;;
        "push")
            check_prerequisites
            build_image
            push_image
            ;;
        "clean")
            clean
            ;;
        "help"|"--help"|"-h")
            usage
            ;;
        *)
            error "Unknown command: $command. Use '$0 help' for usage information."
            ;;
    esac
}

# Run main function with all arguments
main "$@"