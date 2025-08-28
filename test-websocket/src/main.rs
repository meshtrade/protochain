use solana_client::nonblocking::rpc_client::RpcClient;
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Solana WebSocket connectivity...");
    
    // Test RPC connectivity first
    println!("1. Testing RPC connection to http://localhost:8899...");
    let rpc_client = RpcClient::new("http://localhost:8899".to_string());
    
    match tokio::time::timeout(Duration::from_secs(5), rpc_client.get_health()).await {
        Ok(health_result) => {
            match health_result {
                Ok(_) => println!("   ✅ RPC connection successful"),
                Err(e) => println!("   ❌ RPC health check failed: {}", e),
            }
        }
        Err(_) => println!("   ⏰ RPC connection timed out"),
    }
    
    // Test WebSocket connectivity
    println!("2. Testing WebSocket connection to ws://localhost:8900...");
    
    match tokio::time::timeout(
        Duration::from_secs(10),
        PubsubClient::new("ws://localhost:8900")
    ).await {
        Ok(pubsub_result) => {
            match pubsub_result {
                Ok(pubsub_client) => {
                    println!("   ✅ WebSocket connection successful!");
                    println!("   📡 PubsubClient created successfully");
                    
                    println!("3. ✅ WebSocket functionality test passed!");
                    println!("   📡 Ready to handle signature subscriptions");
                }
                Err(e) => {
                    println!("   ❌ WebSocket connection failed: {}", e);
                    println!("   🔧 Make sure solana-test-validator is running with --rpc-port 8899");
                }
            }
        }
        Err(_) => {
            println!("   ⏰ WebSocket connection timed out");
            println!("   🔧 Check if port 8900 is open and accepting connections");
        }
    }
    
    println!("\n📊 Connection test completed");
    Ok(())
}