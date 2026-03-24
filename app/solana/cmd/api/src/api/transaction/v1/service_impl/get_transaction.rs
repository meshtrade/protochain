use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::{signature::Signature, transaction::Transaction as SolanaTransaction};
use solana_transaction_status::{
    option_serializer::OptionSerializer, EncodedTransaction, UiTransactionEncoding,
};
use std::str::FromStr;
use tonic::{Request, Response, Status};
use tracing::error;

use protochain_api::protochain::solana::transaction::v1::{
    GetTransactionRequest, GetTransactionResponse, Transaction, TransactionState,
};

#[allow(clippy::result_large_err, clippy::unused_self)]
impl super::TransactionServiceImpl {
    /// Retrieves a previously submitted transaction from the blockchain by signature
    ///
    /// This method queries the Solana blockchain for a transaction that was previously
    /// submitted and confirmed, providing access to historical transaction data.
    ///
    /// Query Process:
    /// 1. Validates signature format (prevents malformed queries)
    /// 2. Converts to Solana SDK Signature type for type safety
    /// 3. Queries blockchain with configurable commitment level
    /// 4. Handles different transaction encoding formats
    /// 5. Deserializes blockchain data back to protobuf format
    /// 6. Reconstructs transaction metadata for API consistency
    ///
    /// Data Reconstruction:
    /// Since blockchain storage is optimized and doesn't preserve all original metadata:
    /// - instructions: Empty (not stored on-chain after execution)
    /// - state: `FULLY_SIGNED` (network transactions are always fully signed)
    /// - config: None (execution config not preserved)
    /// - signatures: Reconstructed from on-chain data
    /// - `fee_payer`: First account key (Solana convention)
    /// - data: Raw transaction bytes (preserved exactly)
    ///
    /// Commitment Level Impact:
    /// - PROCESSED: May return transactions not yet finalized
    /// - CONFIRMED: Returns transactions confirmed by supermajority
    /// - FINALIZED: Only returns irreversibly confirmed transactions
    ///
    /// Use Cases:
    /// - Transaction status checking after submission
    /// - Historical transaction analysis
    /// - Audit trail reconstruction
    /// - Debugging failed or successful transactions
    pub(super) fn handle_get_transaction(
        &self,
        request: Request<GetTransactionRequest>,
    ) -> Result<Response<GetTransactionResponse>, Status> {
        let req = request.into_inner();

        if req.signature.is_empty() {
            error!("GetTransaction called with empty signature");
            return Err(Status::invalid_argument("Transaction signature is required"));
        }

        // Parse the signature
        let signature = Signature::from_str(&req.signature)
            .map_err(|e| Status::invalid_argument(format!("Invalid signature format: {e}")))?;

        // Get commitment level for transaction retrieval
        let commitment = super::commitment_level_to_config(req.commitment_level);

        // Query the transaction from the network with configurable commitment level
        match self.rpc_client.get_transaction_with_config(
            &signature,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Base58),
                commitment: Some(commitment),
                max_supported_transaction_version: Some(0),
            },
        ) {
            Ok(confirmed_transaction) => {
                // Extract transaction data
                let transaction_data = match confirmed_transaction.transaction.transaction {
                    EncodedTransaction::Binary(data, _) => {
                        bs58::decode(&data).into_vec().map_err(|e| {
                            Status::internal(format!("Failed to decode transaction data: {e}"))
                        })?
                    }
                    _ => {
                        return Err(Status::internal("Unsupported transaction encoding"));
                    }
                };

                // Deserialize the transaction
                let solana_transaction: SolanaTransaction = bincode::deserialize(&transaction_data)
                    .map_err(|e| {
                        Status::internal(format!("Failed to deserialize transaction: {e}"))
                    })?;

                // Check for program logs
                let logs = confirmed_transaction.transaction.meta.as_ref().map_or_else(
                    String::new,
                    |meta| {
                        if let OptionSerializer::Some(logs) = &meta.log_messages {
                            logs.iter().cloned().collect()
                        } else {
                            String::new()
                        }
                    },
                );

                // Check for program error
                let program_err = confirmed_transaction
                    .transaction
                    .meta
                    .map_or_else(String::new, |meta| {
                        meta.err.map_or_else(String::new, |err| format!("{err:?}"))
                    });

                // Convert to our proto format
                let proto_transaction = Transaction {
                    instructions: vec![], // Instructions are not preserved in network storage
                    state: TransactionState::FullySigned.into(), // Network transactions are fully signed
                    config: None, // Config is not preserved in network storage
                    data: bs58::encode(&transaction_data).into_string(),
                    fee_payer: solana_transaction
                        .message
                        .account_keys
                        .first()
                        .map(std::string::ToString::to_string)
                        .unwrap_or_default(),
                    recent_blockhash: solana_transaction.message.recent_blockhash.to_string(),
                    signatures: solana_transaction
                        .signatures
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect(),
                    hash: signature.to_string(), // Use signature as hash for compatibility
                    signature: req.signature,
                    slot: confirmed_transaction.slot,
                    meta_error_message: program_err,
                    meta_logs: logs,
                };

                Ok(Response::new(GetTransactionResponse {
                    transaction: Some(proto_transaction),
                }))
            }
            Err(e) => {
                // Transaction not found or other error
                Err(Status::not_found(format!("Error getting transaction with config: {e}")))
            }
        }
    }
}
