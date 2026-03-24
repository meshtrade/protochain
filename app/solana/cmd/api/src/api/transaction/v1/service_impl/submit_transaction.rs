use solana_sdk::{signature::Signature, transaction::Transaction as SolanaTransaction};
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};

use crate::api::transaction::v1::error_builder;
use crate::api::transaction::v1::validation::{
    validate_operation_allowed_for_state, validate_transaction_state_consistency,
};
use protochain_api::protochain::solana::transaction::v1::{
    SubmitTransactionRequest, SubmitTransactionResponse, TransactionState,
};

#[allow(clippy::result_large_err, clippy::unused_self)]
impl super::TransactionServiceImpl {
    /// Asynchronously submits a fully signed transaction to the Solana blockchain network
    ///
    /// State Transition: `FULLY_SIGNED` -> SUBMITTED (or FAILED)
    ///
    /// This method performs network submission and returns immediately after sending the
    /// transaction without waiting for confirmation. Clients should use `MonitorTransaction`
    /// to poll for confirmation status if they need to verify transaction execution.
    ///
    /// Submission Strategy:
    /// Uses `send_transaction_with_config()` with appropriate configuration but does NOT
    /// wait for confirmation. This provides a pure asynchronous submission interface.
    ///
    /// Benefits of Asynchronous Submission:
    /// 1. NON-BLOCKING: Returns immediately after sending, allowing parallel operations
    ///
    /// 2. CLIENT CONTROL: Clients decide whether to poll for confirmation using `MonitorTransaction`
    ///
    /// 3. PURE SDK WRAPPER: Maintains the protocol buffer wrapper philosophy without adding
    ///    business logic like automatic confirmation waiting
    ///
    /// 4. FLEXIBLE WORKFLOWS: Enables fire-and-forget patterns or custom confirmation strategies
    ///
    /// Error Classification:
    /// - Insufficient Funds: Account balance issues
    /// - Invalid Signature: Cryptographic validation failures
    /// - Network Error: Connectivity, timeout, or RPC issues
    /// - Validation Error: Transaction format or content problems
    ///
    /// NOTE: Successful submission only means the transaction was sent to the network,
    /// not that it was confirmed or executed. Use `MonitorTransaction` for confirmation.
    pub(super) fn handle_submit_transaction(
        &self,
        request: Request<SubmitTransactionRequest>,
    ) -> Result<Response<SubmitTransactionResponse>, Status> {
        let req = request.into_inner();
        let transaction = req
            .transaction
            .ok_or_else(|| Status::invalid_argument("Transaction is required"))?;

        // Validate current state allows submission
        let current_state = transaction.state();
        validate_operation_allowed_for_state(current_state, "submit")
            .map_err(Status::failed_precondition)?;

        // Validate transaction state consistency
        validate_transaction_state_consistency(&transaction)
            .map_err(|e| Status::invalid_argument(format!("Transaction validation failed: {e}")))?;

        // Ensure transaction is fully signed
        if current_state != TransactionState::FullySigned {
            return Err(Status::failed_precondition(
                "Transaction must be fully signed before submission",
            ));
        }

        // Deserialize the signed transaction data
        let transaction_data = bs58::decode(&transaction.data).into_vec().map_err(|e| {
            Status::invalid_argument(format!("Failed to decode transaction data: {e}"))
        })?;

        let solana_transaction: SolanaTransaction = bincode::deserialize(&transaction_data)
            .map_err(|e| {
                Status::invalid_argument(format!("Failed to deserialize transaction: {e}"))
            })?;

        // Verify transaction is properly signed
        if solana_transaction
            .signatures
            .iter()
            .any(|sig| *sig == Signature::default())
        {
            return Err(Status::failed_precondition("Transaction contains unsigned accounts"));
        }

        // Submit the transaction to the Solana network with explicit commitment level
        info!(
            fee_payer = %transaction.fee_payer,
            data_length = transaction.data.len(),
            "Submitting transaction to Solana network"
        );

        // Asynchronously submit transaction without waiting for confirmation
        //
        // Design philosophy:
        // 1. PURE WRAPPER: Maintains the protocol buffer wrapper philosophy - just send
        //    the transaction without adding business logic like confirmation waiting
        //
        // 2. CLIENT CONTROL: Clients decide whether to wait for confirmation using
        //    the separate MonitorTransaction streaming RPC
        //
        // 3. NON-BLOCKING: Returns immediately after network submission, enabling
        //    parallel operations and custom confirmation strategies
        //
        // 4. BACKEND APPROPRIATE: Uses send_transaction_with_config for proper
        //    configuration without any UI dependencies or confirmation polling
        let commitment = super::commitment_level_to_config(req.commitment_level);
        debug!(
            commitment_level = ?commitment,
            fee_payer = %transaction.fee_payer,
            "Transaction submission configured with commitment level"
        );

        // Submit the transaction with proper configuration
        let (signature_result, submission_result, structured_error) =
            match self.rpc_client.send_transaction_with_config(
                &solana_transaction,
                solana_client::rpc_config::RpcSendTransactionConfig {
                    skip_preflight: false,
                    preflight_commitment: Some(commitment.commitment),
                    encoding: Some(solana_transaction_status::UiTransactionEncoding::Base64),
                    max_retries: Some(3),
                    min_context_slot: None,
                },
            ) {
                Ok(signature) => {
                    info!(
                        signature = %signature,
                        fee_payer = %transaction.fee_payer,
                        commitment_level = ?commitment,
                        "Transaction submitted successfully (asynchronously)"
                    );

                    // Return immediately after submission without waiting for confirmation
                    // Clients can use MonitorTransaction to poll for confirmation if desired
                    (signature.to_string(), super::SubmissionResult::Submitted, None)
                }
                Err(e) => {
                    let classification = super::classify_submission_error(&e);

                    // Get current slot for blockhash resolution
                    let current_slot = self.rpc_client.get_slot().unwrap_or(0);

                    // Parse blockhash from transaction for resolution strategy
                    let transaction_blockhash = transaction
                        .recent_blockhash
                        .parse()
                        .unwrap_or_else(|_| solana_sdk::hash::Hash::default());

                    // Build comprehensive structured error
                    let structured_err = error_builder::build_structured_error(
                        &e,
                        classification,
                        &transaction_blockhash,
                        current_slot,
                    );

                    error!(
                        error = %e,
                        fee_payer = %transaction.fee_payer,
                        commitment_level = ?commitment,
                        classification = ?classification,
                        certainty = ?structured_err.certainty,
                        retryable = structured_err.retryable,
                        "Transaction submission failed"
                    );

                    (String::new(), classification, Some(structured_err))
                }
            };

        Ok(Response::new(SubmitTransactionResponse {
            signature: signature_result,
            submission_result: submission_result.into(),
            error_message: structured_error
                .as_ref()
                .map(|e| e.message.clone())
                .unwrap_or_default(),
            structured_error,
        }))
    }
}
