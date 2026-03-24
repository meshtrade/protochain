use solana_sdk::{
    message::Message,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction as SolanaTransaction,
};
use tonic::{Request, Response, Status};
use tracing::info;

use crate::api::transaction::v1::validation::{
    validate_operation_allowed_for_state, validate_state_transition,
    validate_transaction_state_consistency,
};
use protochain_api::protochain::solana::transaction::v1::{
    sign_transaction_request, SignTransactionRequest, SignTransactionResponse, TransactionState,
};

#[allow(clippy::result_large_err, clippy::unused_self)]
impl super::TransactionServiceImpl {
    /// Signs a compiled transaction with cryptographic signatures for authorization
    ///
    /// State Transition: COMPILED -> `PARTIALLY_SIGNED` or `FULLY_SIGNED`
    ///
    /// This method applies cryptographic signatures to authorize transaction execution.
    /// It supports multiple signing methods and automatically determines completion state.
    ///
    /// Signing Process:
    /// 1. Validates transaction state allows signing (must be COMPILED or `PARTIALLY_SIGNED`)
    /// 2. Deserializes compiled transaction data back to Solana SDK format
    /// 3. Processes signing method (currently supports private key signing)
    /// 4. Matches provided keys with transaction's required signers
    /// 5. Applies signatures for matching accounts only
    /// 6. Determines final state based on signature completeness
    /// 7. Re-serializes signed transaction for storage
    ///
    /// State Determination Logic:
    /// - `FULLY_SIGNED`: All required signatures present (ready for submission)
    /// - `PARTIALLY_SIGNED`: Some signatures present, more needed
    ///
    /// Security Features:
    /// - Only signs for accounts present in transaction (prevents signature reuse)
    /// - Validates private key format (64 bytes, Base58 encoded)
    /// - Signature verification through Solana SDK cryptographic functions
    /// - No signature storage of private keys (used and discarded)
    ///
    /// Signing Methods:
    /// - `PrivateKeys`: Direct private key signing (current implementation)
    /// - Seeds: Deterministic key derivation (not yet implemented)
    ///
    /// The multi-step signing support enables complex workflows like multi-signature
    /// transactions and hardware wallet integration.
    pub(super) fn handle_sign_transaction(
        &self,
        request: Request<SignTransactionRequest>,
    ) -> Result<Response<SignTransactionResponse>, Status> {
        let req = request.into_inner();
        let mut transaction = req
            .transaction
            .ok_or_else(|| Status::invalid_argument("Transaction is required"))?;

        // Validate current state allows signing
        let current_state = transaction.state();
        validate_operation_allowed_for_state(current_state, "sign")
            .map_err(Status::failed_precondition)?;

        // Validate transaction state consistency
        validate_transaction_state_consistency(&transaction)
            .map_err(|e| Status::invalid_argument(format!("Transaction validation failed: {e}")))?;

        // Ensure transaction has compiled data
        if transaction.data.is_empty() {
            return Err(Status::invalid_argument("Transaction must be compiled before signing"));
        }

        // Deserialize the compiled transaction data
        let transaction_data = bs58::decode(&transaction.data).into_vec().map_err(|e| {
            Status::invalid_argument(format!("Failed to decode transaction data: {e}"))
        })?;

        let message: Message = bincode::deserialize(&transaction_data).map_err(|e| {
            Status::invalid_argument(format!("Failed to deserialize transaction: {e}"))
        })?;

        // Process signing method and apply signatures
        let keypairs = match req.signing_method {
            Some(signing_method) => match signing_method {
                sign_transaction_request::SigningMethod::PrivateKeys(private_keys_method) => {
                    // Parse private keys into keypairs
                    let mut keypairs = Vec::new();
                    for private_key_str in &private_keys_method.private_keys {
                        let keypair =
                            Keypair::try_from_base58_string(private_key_str).map_err(|e| {
                                Status::invalid_argument(format!("Invalid private key: {e}"))
                            })?;
                        keypairs.push(keypair);
                    }
                    keypairs
                }
                sign_transaction_request::SigningMethod::Seeds(_seed_method) => {
                    // Seed-based signing not implemented in current version
                    return Err(Status::unimplemented("Seed-based signing not available"));
                }
            },
            None => return Err(Status::invalid_argument("Signing method is required")),
        };

        // Create Solana transaction with message and apply signatures
        let mut solana_transaction = SolanaTransaction::new_unsigned(message);

        // Sign with each keypair that has a matching account in the transaction
        let mut signatures_applied = 0;
        for keypair in &keypairs {
            if let Some(account_index) = solana_transaction
                .message
                .account_keys
                .iter()
                .position(|key| key == &keypair.pubkey())
            {
                // Apply signature for this account
                let signature = keypair.sign_message(&solana_transaction.message_data());
                solana_transaction.signatures[account_index] = signature;
                signatures_applied += 1;
            }
        }

        if signatures_applied == 0 {
            return Err(Status::invalid_argument("No matching accounts found for provided keys"));
        }

        // Update transaction with signatures
        transaction.signatures = solana_transaction
            .signatures
            .iter()
            .filter(|sig| **sig != Signature::default())
            .map(std::string::ToString::to_string)
            .collect();

        // Determine new state based on signature completeness
        let required_signatures =
            solana_transaction.message.header.num_required_signatures as usize;
        let provided_signatures = transaction.signatures.len();

        info!("required signatures: {:?}", required_signatures);
        info!("provided signatures: {:?}", provided_signatures);
        let new_state = if provided_signatures >= required_signatures {
            TransactionState::FullySigned
        } else {
            TransactionState::PartiallySigned
        };

        // Validate state transition
        validate_state_transition(current_state, new_state)
            .map_err(|e| Status::internal(format!("State transition validation failed: {e}")))?;

        transaction.state = new_state.into();

        // Update the transaction data with signed transaction
        let signed_transaction_bytes = bincode::serialize(&solana_transaction).map_err(|e| {
            Status::internal(format!("Failed to serialize signed transaction: {e}"))
        })?;
        transaction.data = bs58::encode(&signed_transaction_bytes).into_string();

        Ok(Response::new(SignTransactionResponse {
            transaction: Some(transaction),
        }))
    }
}
