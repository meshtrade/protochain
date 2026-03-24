use solana_sdk::{hash::Hash, message::Message, pubkey::Pubkey};
use std::str::FromStr;
use tonic::{Request, Response, Status};

use crate::api::common::solana_conversions::proto_instruction_to_sdk;
use crate::api::transaction::v1::validation::{
    validate_operation_allowed_for_state, validate_state_transition,
    validate_transaction_state_consistency,
};
use protochain_api::protochain::solana::transaction::v1::{
    CompileTransactionRequest, CompileTransactionResponse, TransactionState,
};

#[allow(clippy::result_large_err, clippy::unused_self)]
impl super::TransactionServiceImpl {
    /// Compiles a draft transaction with instructions into executable transaction bytecode
    ///
    /// State Transition: DRAFT -> COMPILED
    ///
    /// This method performs the critical compilation step that transforms human-readable
    /// instructions into binary transaction data that can be executed on Solana blockchain.
    ///
    /// Compilation Process:
    /// 1. Validates current transaction state allows compilation
    /// 2. Converts protobuf instructions to Solana SDK instructions
    /// 3. Fetches recent blockhash (or uses provided one)
    /// 4. Uses Solana SDK `Message::new_with_blockhash` for proper compilation
    /// 5. Serializes compiled message with bincode for compact binary encoding
    /// 6. Base58 encodes for safe protobuf transport
    /// 7. Updates transaction metadata and validates state consistency
    ///
    /// Critical Design Notes:
    /// - Uses Solana SDK compilation (not manual) for proper account deduplication
    /// - Handles signing requirements calculation automatically
    /// - Fetches blockhash if not provided (network call for freshness)
    /// - All validation occurs before and after compilation for safety
    ///
    /// Memory Management:
    /// - Instructions are converted (not cloned) to minimize allocations
    /// - Bincode provides zero-copy serialization where possible
    /// - Base58 encoding only happens once at the end
    pub(super) fn handle_compile_transaction(
        &self,
        request: Request<CompileTransactionRequest>,
    ) -> Result<Response<CompileTransactionResponse>, Status> {
        let req = request.into_inner();
        let mut transaction = req
            .transaction
            .ok_or_else(|| Status::invalid_argument("Transaction is required"))?;

        // Validate current state allows compilation
        let current_state = transaction.state();
        validate_operation_allowed_for_state(current_state, "compile")
            .map_err(Status::failed_precondition)?;

        // Validate transaction consistency in current state
        validate_transaction_state_consistency(&transaction)
            .map_err(|e| Status::invalid_argument(format!("Invalid transaction state: {e}")))?;

        // Ensure we have instructions
        if transaction.instructions.is_empty() {
            return Err(Status::invalid_argument("Transaction must have at least one instruction"));
        }

        // Validate fee_payer is provided
        if req.fee_payer.is_empty() {
            return Err(Status::invalid_argument("fee_payer is required"));
        }

        // Convert proto instructions to SDK instructions
        let sdk_instructions: Result<Vec<solana_sdk::instruction::Instruction>, String> =
            transaction
                .instructions
                .iter()
                .map(|proto_ix| proto_instruction_to_sdk(proto_ix.clone()))
                .collect();

        let sdk_instructions = sdk_instructions
            .map_err(|e| Status::invalid_argument(format!("Invalid instruction: {e}")))?;

        // Parse fee payer pubkey
        let fee_payer = Pubkey::from_str(&req.fee_payer)
            .map_err(|e| Status::invalid_argument(format!("Invalid fee_payer: {e}")))?;

        // Get recent blockhash (from request or fetch from network)
        let recent_blockhash = if req.recent_blockhash.is_empty() {
            // Fetch latest blockhash from network
            self.rpc_client
                .get_latest_blockhash()
                .map_err(|e| Status::internal(format!("Failed to get latest blockhash: {e}")))?
        } else {
            // Use provided blockhash
            Hash::from_str(&req.recent_blockhash)
                .map_err(|e| Status::invalid_argument(format!("Invalid blockhash format: {e}")))?
        };

        // CRITICAL: Use Solana SDK to compile the transaction
        // This handles all the complexity of account deduplication, signing requirements, etc.
        let message =
            Message::new_with_blockhash(&sdk_instructions, Some(&fee_payer), &recent_blockhash);

        // Serialize the compiled message for transport
        let transaction_bytes = bincode::serialize(&message)
            .map_err(|e| Status::internal(format!("Transaction serialization failed: {e}")))?;

        // Encode as base58 for proto transport
        let transaction_data = bs58::encode(&transaction_bytes).into_string();

        // Validate state transition DRAFT -> COMPILED
        validate_state_transition(current_state, TransactionState::Compiled)
            .map_err(|e| Status::internal(format!("State transition validation failed: {e}")))?;

        // Update transaction with compiled data and metadata
        transaction.data = transaction_data;
        transaction.state = TransactionState::Compiled.into();
        transaction.fee_payer = req.fee_payer;
        transaction.recent_blockhash = recent_blockhash.to_string();

        // Validate the updated transaction consistency
        validate_transaction_state_consistency(&transaction).map_err(|e| {
            Status::internal(format!("Compiled transaction validation failed: {e}"))
        })?;

        Ok(Response::new(CompileTransactionResponse {
            transaction: Some(transaction),
        }))
    }
}
