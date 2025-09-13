# Solana gRPC Application Server

This is the structured Solana gRPC API backend service.

## Architecture

The application follows a layered architecture with dependency injection:

```
project/solana/cmd/api/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── lib.rs                  # Library exports
│   ├── service_providers/      # Dependency injection container
│   │   ├── mod.rs              # Module exports
│   │   ├── service_providers.rs # Main service provider struct
│   │   └── solana_clients.rs   # Solana RPC client management
│   └── api/                    # API layer organization
│       ├── mod.rs              # API module exports
│       ├── api.rs              # Root API struct
│       ├── transaction/        # Transaction service
│       │   └── v1/             # Version 1 implementation
│       │       ├── mod.rs
│       │       ├── transaction_v1_api.rs
│       │       └── service_impl.rs
│       └── account/            # Account service
│           └── v1/             # Version 1 implementation
│               ├── mod.rs
│               ├── account_v1_api.rs
│               └── service_impl.rs
└── Cargo.toml                  # Package configuration
```

## Key Features

### Dependency Injection
- **ServiceProviders**: Main dependency container managing all service dependencies
- **SolanaClientsServiceProviders**: Manages Solana RPC client instances
- **Specific Dependencies**: Service implementations only hold dependencies they actually need (e.g., RPC client)
- **Thread-safe Arc<RpcClient>**: Shared safely across service implementations

### API Organization
- **Clean Layer Structure**: Direct API → service pattern without unnecessary nesting
- **Versioned Services**: Transaction v1 and Account v1 services with room for evolution
- **Service Implementations**: Hold only required dependencies, not entire service provider structs
- **Proper Separation**: API layers extract dependencies and pass them to service implementations

### Network Configuration
- **Environment-based Configuration**: Uses SOLANA_RPC_URL environment variable
- **Mainnet/Devnet Support**: Easy network switching via environment variables
- **Safe Defaults**: Defaults to devnet if no environment variable is set

## Usage

### Development

Start the server using the development scripts:
```bash
# From project root
./project/solana/scripts/dev.sh start
```

### Direct Usage

Run the application directly:
```bash
# From project root
cargo run -p protochain-solana-api

# With specific network
SOLANA_RPC_URL="https://api.mainnet-beta.solana.com" cargo run -p protochain-solana-api
```

### Testing

The structured app is fully compatible with existing integration tests:
```bash
# Run integration tests against the structured app
RUN_INTEGRATION_TESTS=1 go test -v ./project/solana/cmd/api-test
```

## Services

### Transaction Service v1
- **SubmitTransaction**: Process and submit transactions to Solana network
- **GetTransaction**: Retrieve transaction data from Solana network by signature

### Account Service v1
- **GetAccount**: Retrieve account data from Solana network by address

Both services include:
- Real Solana network integration
- Comprehensive error handling
- Input validation
- Request/response logging

## Architecture Benefits

1. **Maintainability**: Clear separation of concerns with dependency injection
2. **Testability**: Mockable dependencies through the service provider pattern
3. **Scalability**: Easy to add new services and versions
4. **Professional Structure**: Follows enterprise patterns used in production systems
5. **Network Flexibility**: Easy switching between Solana networks (mainnet/devnet/testnet)
