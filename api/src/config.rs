use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;

/// Main application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub solana: SolanaConfig,
    pub server: ServerConfig,
}

/// Solana RPC client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub health_check_on_startup: bool,
}

/// gRPC server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            solana: SolanaConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

impl Default for SolanaConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8899".to_string(), // Local validator default
            timeout_seconds: 30,
            retry_attempts: 3,
            health_check_on_startup: true,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 50051,
        }
    }
}

/// Loads configuration with the following precedence:
/// 1. Start with defaults
/// 2. Load from config.json file (or --config specified file)
/// 3. Override with environment variables
pub fn load_config() -> Result<Config, String> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut config_path: Option<PathBuf> = None;
    
    // Look for --config flag
    for i in 0..args.len() {
        if args[i] == "--config" && i + 1 < args.len() {
            config_path = Some(PathBuf::from(&args[i + 1]));
            break;
        }
    }
    
    // Configuration loading precedence:
    // 1. Start with defaults
    let mut config = Config::default();
    
    // 2. Try default location if no --config flag
    let config_file_path = config_path.unwrap_or_else(|| {
        PathBuf::from("./config.json") // Default location
    });
    
    // 3. Load from config file if it exists
    if config_file_path.exists() {
        let config_content = std::fs::read_to_string(&config_file_path)
            .map_err(|e| format!("Failed to read config file {:?}: {}", config_file_path, e))?;
        
        config = serde_json::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file {:?}: {}", config_file_path, e))?;
            
        println!("✅ Loaded configuration from: {:?}", config_file_path);
    } else {
        println!("ℹ️  No config file found at {:?}, using defaults", config_file_path);
    }
    
    // 4. Override with environment variables if present
    if let Ok(rpc_url) = std::env::var("SOLANA_RPC_URL") {
        config.solana.rpc_url = rpc_url;
        println!("ℹ️  Override: SOLANA_RPC_URL = {}", config.solana.rpc_url);
    }
    
    if let Ok(port) = std::env::var("SERVER_PORT") {
        config.server.port = port.parse()
            .map_err(|e| format!("Invalid SERVER_PORT environment variable: {}", e))?;
        println!("ℹ️  Override: SERVER_PORT = {}", config.server.port);
    }
    
    if let Ok(timeout) = std::env::var("SOLANA_TIMEOUT_SECONDS") {
        config.solana.timeout_seconds = timeout.parse()
            .map_err(|e| format!("Invalid SOLANA_TIMEOUT_SECONDS environment variable: {}", e))?;
        println!("ℹ️  Override: SOLANA_TIMEOUT_SECONDS = {}", config.solana.timeout_seconds);
    }
    
    if let Ok(retry) = std::env::var("SOLANA_RETRY_ATTEMPTS") {
        config.solana.retry_attempts = retry.parse()
            .map_err(|e| format!("Invalid SOLANA_RETRY_ATTEMPTS environment variable: {}", e))?;
        println!("ℹ️  Override: SOLANA_RETRY_ATTEMPTS = {}", config.solana.retry_attempts);
    }
    
    if let Ok(health_check) = std::env::var("SOLANA_HEALTH_CHECK_ON_STARTUP") {
        config.solana.health_check_on_startup = health_check.to_lowercase() == "true";
        println!("ℹ️  Override: SOLANA_HEALTH_CHECK_ON_STARTUP = {}", config.solana.health_check_on_startup);
    }
    
    Ok(config)
}

/// Validates the Solana RPC connection by performing a health check
pub async fn validate_solana_connection(rpc_url: &str) -> Result<(), String> {
    println!("🔍 Health check: Testing connection to Solana RPC at {}", rpc_url);
    
    let client = RpcClient::new(rpc_url.to_string());
    
    // Perform health check
    match client.get_version() {
        Ok(version) => {
            println!("✅ Solana RPC connection successful!");
            println!("   - RPC URL: {}", rpc_url);
            println!("   - Solana version: {}", version.solana_core);
            Ok(())
        }
        Err(e) => {
            Err(format!("❌ Solana RPC health check failed at {}: {}", rpc_url, e))
        }
    }
}

/// Creates a sample configuration file for reference
pub fn create_sample_config() -> Result<(), String> {
    let sample_config = Config::default();
    
    let config_json = serde_json::to_string_pretty(&sample_config)
        .map_err(|e| format!("Failed to serialize sample config: {}", e))?;
    
    std::fs::write("config.sample.json", config_json)
        .map_err(|e| format!("Failed to write sample config: {}", e))?;
    
    println!("✅ Created config.sample.json");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        
        assert_eq!(config.solana.rpc_url, "http://localhost:8899");
        assert_eq!(config.solana.timeout_seconds, 30);
        assert_eq!(config.solana.retry_attempts, 3);
        assert!(config.solana.health_check_on_startup);
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 50051);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.solana.rpc_url, deserialized.solana.rpc_url);
        assert_eq!(config.server.port, deserialized.server.port);
    }
    
    #[test]
    fn test_environment_variable_parsing() {
        // Test port parsing
        env::set_var("SERVER_PORT", "8080");
        let mut config = Config::default();
        
        if let Ok(port) = env::var("SERVER_PORT") {
            config.server.port = port.parse().unwrap();
        }
        
        assert_eq!(config.server.port, 8080);
        env::remove_var("SERVER_PORT");
    }
    
    #[test]
    fn test_config_file_not_found() {
        // This should work without error when no config file exists
        let result = load_config();
        assert!(result.is_ok());
    }
}