use solana_client::nonblocking::rpc_client::RpcClient;
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
use std::time::Duration;
use tokio;

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
                    
                    // Try to create a simple subscription to test functionality
                    println!("3. Testing signature subscription...");
                    let dummy_signature = "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYlCJjBRCN8FHXvVSs8h7oprNJfj6gJV26pEgJZNMAUh2tCgKHU9Sy"
                        .parse()
                        .expect("Valid signature");
                    
                    match pubsub_client.signature_subscribe(&dummy_signature, None).await {
                        Ok((mut stream, _unsubscribe)) => {
                            println!("   ✅ Signature subscription created successfully!");
                            
                            // Test receiving for a brief moment
                            let timeout = tokio::time::sleep(Duration::from_secs(2));
                            tokio::pin!(timeout);
                            
                            tokio::select! {
                                _ = &mut timeout => {
                                    println!("   ℹ️  No immediate updates (expected for dummy signature)");
                                }
                                notification = tokio_stream::StreamExt::next(&mut stream) => {
                                    if let Some(response) = notification {
                                        println!("   📨 Received notification: {:?}", response);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("   ❌ Failed to create signature subscription: {}", e);
                        }
                    }
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