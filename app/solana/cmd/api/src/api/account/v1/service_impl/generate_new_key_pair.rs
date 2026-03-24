//! Handler implementation for `GenerateNewKeyPair`.

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::account::v1::{
    GenerateNewKeyPairRequest, GenerateNewKeyPairResponse,
};
use protochain_api::protochain::solana::r#type::v1::KeyPair;

use solana_sdk::signature::{Keypair, SeedDerivable, Signer};

use super::AccountServiceImpl;

#[allow(clippy::result_large_err, clippy::unused_self)]
impl AccountServiceImpl {
    /// Generates a new Ed25519 keypair, either randomly or deterministically from a hex seed.
    pub(super) fn handle_generate_new_key_pair(
        &self,
        request: Request<GenerateNewKeyPairRequest>,
    ) -> Result<Response<GenerateNewKeyPairResponse>, Status> {
        println!("Received generate keypair request: {request:?}");

        let req = request.into_inner();

        // Generate keypair (random or from seed)
        let keypair = if req.seed.is_empty() {
            // Random generation
            Keypair::new()
        } else {
            // Deterministic generation from seed
            let seed_bytes = hex::decode(&req.seed)
                .map_err(|e| Status::invalid_argument(format!("Invalid hex seed: {e}")))?;

            if seed_bytes.len() != 32 {
                return Err(Status::invalid_argument("Seed must be exactly 32 bytes"));
            }

            let mut seed_array = [0u8; 32];
            seed_array.copy_from_slice(&seed_bytes);
            Keypair::from_seed(&seed_array).map_err(|e| {
                Status::internal(format!("Failed to generate keypair from seed: {e}"))
            })?
        };

        // Create protobuf KeyPair with proper field names
        let key_pair = KeyPair {
            public_key: keypair.pubkey().to_string(), // Base58 encoded
            private_key: bs58::encode(keypair.to_bytes()).into_string(), // Base58 encoded full keypair
        };

        println!("Generated keypair with public key: {}", key_pair.public_key);

        Ok(Response::new(GenerateNewKeyPairResponse {
            key_pair: Some(key_pair),
        }))
    }
}
