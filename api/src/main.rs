//! # `ProtoSol` Solana API Backend
//!
//! This is the main gRPC server for the `ProtoSol` Solana API.
//! It provides a Protocol Buffer-based API over Solana blockchain operations.
//!
//! The server provides services for:
//! - Account management (creation, funding, querying)
//! - Transaction lifecycle management (compilation, signing, submission)
//! - System program operations (transfers, account creation)
//! - Real-time transaction monitoring via WebSocket

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Server;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// Import the generated protobuf services
use protosol_api::protosol::solana::account::v1::service_server::ServiceServer as AccountServiceServer;
use protosol_api::protosol::solana::program::system::v1::service_server::ServiceServer as SystemProgramServiceServer;
use protosol_api::protosol::solana::program::token::v1::service_server::ServiceServer as TokenProgramServiceServer;
use protosol_api::protosol::solana::rpc_client::v1::service_server::ServiceServer as RpcClientServiceServer;
use protosol_api::protosol::solana::transaction::v1::service_server::ServiceServer as TransactionServiceServer;

// Import our application modules
mod api;
mod config;
mod service_providers;
mod websocket;

use api::Api;
use config::{load_config, validate_solana_connection};
use service_providers::ServiceProviders;

/// Initialize structured logging with appropriate formatting and filtering
///
/// Logging Configuration:
/// - Uses environment variable `RUST_LOG` for level filtering (default: "info")
/// - JSON format for production environments (when `PROTOSOL_JSON_LOGS=true`)
/// - Human-readable format for development (default)
/// - Supports log levels: trace, debug, info, warn, error
/// - Includes source code locations for debug builds
///
/// Environment Variables:
/// - `RUST_LOG`: Controls log level filtering (e.g., "debug", "`protosol_solana_api=trace`")
/// - `PROTOSOL_JSON_LOGS`: Set to "true" for JSON structured output
///
/// Examples:
/// - Development: `RUST_LOG=debug` cargo run
/// - Production: `RUST_LOG=info` `PROTOSOL_JSON_LOGS=true` ./protosol-solana-api
/// - Service-specific: `RUST_LOG=protosol_solana_api=trace,websocket=debug` cargo run
fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,protosol_solana_api=info"));

    let use_json = std::env::var("PROTOSOL_JSON_LOGS").unwrap_or_default() == "true";

    if use_json {
        // JSON structured logging for production
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(false)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_thread_names(true),
            )
            .with(env_filter)
            .init();
    } else {
        // Human-readable logging for development
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .with_file(cfg!(debug_assertions))
                    .with_line_number(cfg!(debug_assertions))
                    .compact(),
            )
            .with(env_filter)
            .init();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    init_logging();

    info!("🚀 Starting Solana gRPC Application Server");

    // Load configuration with precedence (CLI args, file, env vars)
    let config = load_config().unwrap_or_else(|e| {
        error!(error = %e, "Configuration loading failed");
        std::process::exit(1);
    });

    info!(
        rpc_url = %config.solana.rpc_url,
        server_host = %config.server.host,
        server_port = config.server.port,
        timeout_seconds = config.solana.timeout_seconds,
        retry_attempts = config.solana.retry_attempts,
        "📋 Configuration loaded successfully"
    );

    // Perform Solana RPC health check if enabled
    if config.solana.health_check_on_startup {
        debug!(rpc_url = %config.solana.rpc_url, "Performing Solana RPC health check");
        if let Err(e) = validate_solana_connection(&config.solana.rpc_url) {
            error!(
                error = %e,
                rpc_url = %config.solana.rpc_url,
                "Solana RPC health check failed"
            );
            error!("💡 Tip: Set SOLANA_HEALTH_CHECK_ON_STARTUP=false to skip health check");
            std::process::exit(1);
        }
        info!(rpc_url = %config.solana.rpc_url, "✅ Solana RPC health check passed");
    } else {
        warn!("Skipping Solana RPC health check (disabled in config)");
    }

    // Initialize service providers with configuration
    let service_providers = Arc::new(ServiceProviders::new_with_config(config.clone()).await?);

    info!(
        network_info = %service_providers.get_network_info(),
        "🌐 Network configuration initialized"
    );

    // Initialize application API layer
    let api = Arc::new(Api::new(Arc::clone(&service_providers)));

    // Configure server address from config
    let addr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    info!(
        address = %addr,
        "🌟 Starting Solana gRPC server"
    );
    info!("📡 Services: Transaction v1, Account v1, System Program v1, Token Program v1, RPC Client v1");
    info!("📋 Ready to accept connections!");

    // Start periodic cleanup task for WebSocket subscriptions
    let websocket_manager_cleanup = service_providers.websocket_manager.clone();
    let cleanup_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        debug!("Started WebSocket subscription cleanup task with 60s interval");
        loop {
            interval.tick().await;
            websocket_manager_cleanup.cleanup_expired_subscriptions();
        }
    });

    // Build and start the gRPC server with our service implementations
    // Clone the services from the Arc containers
    let transaction_service = (*api.transaction_v1.transaction_service).clone();
    let account_service = (*api.account_v1.account_service).clone();
    let system_program_service = (*api.program.system.v1.system_program_service).clone();
    let token_program_service = (*api.program.token.token_program_service).clone();
    let rpc_client_service = (*api.rpc_client_v1.rpc_client_service).clone();

    // Clone service providers for graceful shutdown
    let service_providers_shutdown = Arc::clone(&service_providers);

    // Set up graceful shutdown
    let server = Server::builder()
        .add_service(TransactionServiceServer::new(transaction_service))
        .add_service(AccountServiceServer::new(account_service))
        .add_service(SystemProgramServiceServer::new(system_program_service))
        .add_service(TokenProgramServiceServer::new(token_program_service))
        .add_service(RpcClientServiceServer::new(rpc_client_service))
        .serve(addr);

    // Wait for server or shutdown signal
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                error!(error = %e, "❌ Server error occurred");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("🛑 Shutdown signal received");
            info!("🧹 Cleaning up resources...");

            // Abort cleanup task
            cleanup_task.abort();
            debug!("WebSocket cleanup task aborted");

            // Shutdown WebSocket manager
            service_providers_shutdown.websocket_manager.shutdown();

            info!("✅ Graceful shutdown complete");
        }
    }

    Ok(())
}
