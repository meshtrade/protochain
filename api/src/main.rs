use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Server;
use anyhow::Result;

// Import the generated protobuf services
use protosol_api::protosol::solana::transaction::v1::service_server::ServiceServer as TransactionServiceServer;
use protosol_api::protosol::solana::account::v1::service_server::ServiceServer as AccountServiceServer;
use protosol_api::protosol::solana::program::system::v1::service_server::ServiceServer as SystemProgramServiceServer;

// Import our application modules
mod service_providers;
mod api;
mod config;
mod websocket;

use service_providers::ServiceProviders;
use api::API;
use config::{load_config, validate_solana_connection};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Solana gRPC Application Server");
    
    // Load configuration with precedence (CLI args, file, env vars)
    let config = load_config().unwrap_or_else(|e| {
        eprintln!("❌ Configuration loading failed: {}", e);
        std::process::exit(1);
    });
    
    println!("📋 Configuration loaded:");
    println!("   - Solana RPC URL: {}", config.solana.rpc_url);
    println!("   - Server: {}:{}", config.server.host, config.server.port);
    println!("   - Timeout: {}s, Retry: {} attempts", 
             config.solana.timeout_seconds, config.solana.retry_attempts);
    
    // Perform Solana RPC health check if enabled
    if config.solana.health_check_on_startup {
        if let Err(e) = validate_solana_connection(&config.solana.rpc_url).await {
            eprintln!("{}", e);
            eprintln!("💡 Tip: Set SOLANA_HEALTH_CHECK_ON_STARTUP=false to skip health check");
            std::process::exit(1);
        }
    } else {
        println!("⚠️  Skipping Solana RPC health check (disabled in config)");
    }
    
    // Initialize service providers with configuration
    let service_providers = Arc::new(ServiceProviders::new_with_config(config.clone()).await?);
    
    println!("🌐 Network Configuration: {}", service_providers.get_network_info());
    
    // Initialize application API layer
    let api = Arc::new(API::new(Arc::clone(&service_providers)));
    
    // Configure server address from config
    let addr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    println!("🌟 Starting Solana gRPC server on {}", addr);
    println!("📡 Services: Transaction v1, Account v1, System Program v1");
    println!("📋 Ready to accept connections!");
    
    // Start periodic cleanup task for WebSocket subscriptions
    let websocket_manager_cleanup = service_providers.websocket_manager.clone();
    let cleanup_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            websocket_manager_cleanup.cleanup_expired_subscriptions().await;
        }
    });
    
    // Build and start the gRPC server with our service implementations
    // Clone the services from the Arc containers
    let transaction_service = (*api.transaction_v1.transaction_service).clone();
    let account_service = (*api.account_v1.account_service).clone();
    let system_program_service = (*api.program.system.v1.system_program_service).clone();
    
    // Clone service providers for graceful shutdown
    let service_providers_shutdown = Arc::clone(&service_providers);
    
    // Set up graceful shutdown
    let server = Server::builder()
        .add_service(TransactionServiceServer::new(transaction_service))
        .add_service(AccountServiceServer::new(account_service))
        .add_service(SystemProgramServiceServer::new(system_program_service))
        .serve(addr);
    
    // Wait for server or shutdown signal
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                eprintln!("❌ Server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\n🛑 Shutdown signal received");
            println!("🧹 Cleaning up resources...");
            
            // Abort cleanup task
            cleanup_task.abort();
            
            // Shutdown WebSocket manager
            if let Err(e) = service_providers_shutdown.websocket_manager.shutdown().await {
                eprintln!("⚠️ WebSocket shutdown error: {}", e);
            }
            
            println!("✅ Graceful shutdown complete");
        }
    }

    Ok(())
}