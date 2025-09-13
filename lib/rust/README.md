# Protosol API Rust SDK

This crate provides Rust bindings for the Protosol API protocol buffers.

## Generated Code

All code in `src/` (except `lib.rs`) is auto-generated from the protobuf definitions in `api/proto/`.

To regenerate the code:
```bash
./dev/tool.sh generate --project=api
```

## Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
protochain-api = { path = "../path/to/api/rust" }
```

Then use in your code:
```rust
use protochain_api::{Transaction, SubmitTransactionRequest};
use protochain_api::service_client::ServiceClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ServiceClient::connect("http://[::1]:50051").await?;
    
    let request = tonic::Request::new(SubmitTransactionRequest {
        transaction: Some(Transaction {
            hash: "example_hash".to_string(),
            data: "example_data".to_string(),
        }),
    });
    
    let response = client.submit_transaction(request).await?;
    println!("Response: {:?}", response);
    
    Ok(())
}
```