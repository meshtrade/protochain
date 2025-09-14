#!/bin/bash

# Network Manipulation Orchestration Script for Error Categories Testing
#
# This script provides functions to manipulate network conditions for testing
# indeterminate transaction submission states in the Solana API error categories test suite.

set -euo pipefail

# Configuration
SOLANA_VALIDATOR_CONTAINER="solana-validator"
PROTOCHAIN_API_CONTAINER="protochain-api"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check if Docker is available
check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not available. This script requires Docker for network manipulation."
        exit 1
    fi
}

# Stop RPC node (Solana validator) for network error simulation
stop_rpc_node() {
    log_info "Stopping Solana validator to simulate network failure"

    # Try Docker container first
    if docker ps --format "table {{.Names}}" | grep -q "$SOLANA_VALIDATOR_CONTAINER"; then
        docker stop "$SOLANA_VALIDATOR_CONTAINER" 2>/dev/null || true
        log_success "Solana validator container stopped"
        return 0
    fi

    # Try Docker Compose
    if docker-compose ps | grep -q solana; then
        docker-compose stop solana-validator 2>/dev/null || true
        log_success "Solana validator stopped via docker-compose"
        return 0
    fi

    # Try native process
    if pkill -f "solana-test-validator" 2>/dev/null; then
        log_success "Native solana-test-validator process killed"
        return 0
    fi

    log_warn "No running Solana validator found to stop"
}

# Start RPC node (Solana validator)
start_rpc_node() {
    log_info "Starting Solana validator"

    # Check if we have a docker-compose setup
    if [ -f "docker-compose.yml" ]; then
        docker-compose up -d solana-validator
        log_success "Solana validator started via docker-compose"
        return 0
    fi

    # Try to start the validator container directly if it exists
    if docker ps -a --format "table {{.Names}}" | grep -q "$SOLANA_VALIDATOR_CONTAINER"; then
        docker start "$SOLANA_VALIDATOR_CONTAINER"
        log_success "Solana validator container started"
        return 0
    fi

    # Fall back to script if available
    if [ -f "./scripts/tests/start-validator-docker.sh" ]; then
        ./scripts/tests/start-validator-docker.sh
        log_success "Solana validator started via script"
        return 0
    fi

    log_error "Could not determine how to start Solana validator"
    log_info "Please start the validator manually using one of:"
    log_info "  - ./scripts/tests/start-validator-docker.sh"
    log_info "  - docker-compose up -d solana-validator"
    log_info "  - solana-test-validator"
    return 1
}

# Stop backend API for network error simulation
stop_backend() {
    log_info "Stopping ProtoChain API backend to simulate backend unavailability"

    # Try Docker container first
    if docker ps --format "table {{.Names}}" | grep -q "$PROTOCHAIN_API_CONTAINER"; then
        docker stop "$PROTOCHAIN_API_CONTAINER" 2>/dev/null || true
        log_success "ProtoChain API container stopped"
        return 0
    fi

    # Try Docker Compose
    if docker-compose ps | grep -q protochain; then
        docker-compose stop protochain-api 2>/dev/null || true
        log_success "ProtoChain API stopped via docker-compose"
        return 0
    fi

    # Try to kill by port
    if lsof -i :50051 2>/dev/null | grep LISTEN; then
        local pid=$(lsof -ti :50051)
        if [ -n "$pid" ]; then
            kill -9 "$pid" 2>/dev/null || true
            log_success "Process on port 50051 killed"
        fi
        return 0
    fi

    log_warn "No running ProtoChain API backend found to stop"
}

# Start backend API
start_backend() {
    log_info "Starting ProtoChain API backend"

    # Check if we have a docker-compose setup
    if [ -f "docker-compose.yml" ]; then
        docker-compose up -d protochain-api
        log_success "ProtoChain API started via docker-compose"
        return 0
    fi

    # Try to start the container directly if it exists
    if docker ps -a --format "table {{.Names}}" | grep -q "$PROTOCHAIN_API_CONTAINER"; then
        docker start "$PROTOCHAIN_API_CONTAINER"
        log_success "ProtoChain API container started"
        return 0
    fi

    # Fall back to script if available
    if [ -f "./scripts/tests/start-backend.sh" ]; then
        ./scripts/tests/start-backend.sh &
        log_success "ProtoChain API started via script"
        return 0
    fi

    log_error "Could not determine how to start ProtoChain API backend"
    log_info "Please start the backend manually using one of:"
    log_info "  - ./scripts/tests/start-backend.sh"
    log_info "  - docker-compose up -d protochain-api"
    log_info "  - cargo run -p protochain-solana-api"
    return 1
}

# Simulate network latency using tc (traffic control)
simulate_network_latency() {
    local delay_ms="${1:-1000}"
    log_info "Simulating network latency: ${delay_ms}ms delay"

    # Check if tc is available
    if ! command -v tc &> /dev/null; then
        log_warn "tc (traffic control) not available. Network latency simulation skipped."
        log_info "On macOS: This requires root privileges and advanced networking setup"
        log_info "On Linux: sudo tc qdisc add dev lo root netem delay ${delay_ms}ms"
        return 1
    fi

    # Add network delay (requires root)
    if ! sudo tc qdisc add dev lo root netem delay "${delay_ms}ms" 2>/dev/null; then
        log_warn "Could not add network delay. May require root privileges."
        return 1
    fi

    log_success "Network latency of ${delay_ms}ms added to loopback interface"
}

# Remove network latency simulation
remove_network_latency() {
    log_info "Removing network latency simulation"

    if command -v tc &> /dev/null; then
        sudo tc qdisc del dev lo root 2>/dev/null || true
        log_success "Network latency simulation removed"
    else
        log_info "tc not available, no latency to remove"
    fi
}

# Wait for service to become available
wait_for_service() {
    local service_name="$1"
    local host="$2"
    local port="$3"
    local timeout="${4:-30}"

    log_info "Waiting for $service_name to become available on $host:$port (timeout: ${timeout}s)"

    local count=0
    while [ $count -lt $timeout ]; do
        if nc -z "$host" "$port" 2>/dev/null; then
            log_success "$service_name is available"
            return 0
        fi
        sleep 1
        ((count++))
    done

    log_error "$service_name failed to become available within ${timeout} seconds"
    return 1
}

# Test network connectivity
test_connectivity() {
    log_info "Testing network connectivity"

    # Test Solana RPC
    if curl -s -X POST -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
        http://localhost:8899 > /dev/null 2>&1; then
        log_success "Solana RPC (port 8899) is accessible"
    else
        log_error "Solana RPC (port 8899) is not accessible"
    fi

    # Test ProtoChain API
    if nc -z localhost 50051 2>/dev/null; then
        log_success "ProtoChain API (port 50051) is accessible"
    else
        log_error "ProtoChain API (port 50051) is not accessible"
    fi
}

# Display usage information
usage() {
    echo "Network Manipulation Script for ProtoChain Error Categories Testing"
    echo ""
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  stop-rpc              Stop Solana validator (simulate RPC unavailability)"
    echo "  start-rpc             Start Solana validator"
    echo "  stop-backend          Stop ProtoChain API backend"
    echo "  start-backend         Start ProtoChain API backend"
    echo "  simulate-latency [ms] Add network latency (default: 1000ms)"
    echo "  remove-latency        Remove network latency simulation"
    echo "  test-connectivity     Test connectivity to services"
    echo "  full-restart          Stop and restart all services"
    echo "  help                  Display this help message"
    echo ""
    echo "Examples:"
    echo "  $0 stop-rpc                    # Stop Solana validator"
    echo "  $0 simulate-latency 2000       # Add 2 second network delay"
    echo "  $0 full-restart                # Restart all services"
}

# Full restart of all services
full_restart() {
    log_info "Performing full restart of all services"

    stop_backend
    stop_rpc_node
    sleep 2

    start_rpc_node
    wait_for_service "Solana RPC" "localhost" "8899" 30

    start_backend
    wait_for_service "ProtoChain API" "localhost" "50051" 30

    test_connectivity
    log_success "Full restart completed"
}

# Main command processing
main() {
    check_docker

    case "${1:-help}" in
        "stop-rpc")
            stop_rpc_node
            ;;
        "start-rpc")
            start_rpc_node
            ;;
        "stop-backend")
            stop_backend
            ;;
        "start-backend")
            start_backend
            ;;
        "simulate-latency")
            simulate_network_latency "${2:-1000}"
            ;;
        "remove-latency")
            remove_network_latency
            ;;
        "test-connectivity")
            test_connectivity
            ;;
        "full-restart")
            full_restart
            ;;
        "help"|"--help"|"-h")
            usage
            ;;
        *)
            log_error "Unknown command: $1"
            echo ""
            usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"