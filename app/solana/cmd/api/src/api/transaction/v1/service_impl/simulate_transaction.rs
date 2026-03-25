use solana_sdk::{message::Message, transaction::Transaction as SolanaTransaction};
use tonic::{Request, Response, Status};

use crate::api::transaction::v1::validation::{
    validate_operation_allowed_for_state, validate_transaction_state_consistency,
};
use protochain_api::protochain::solana::transaction::v1::{
    SimulateTransactionRequest, SimulateTransactionResponse,
};

#[allow(clippy::result_large_err)]
impl super::TransactionServiceImpl {
    /// Simulates a compiled transaction execution without blockchain submission
    ///
    /// This method provides a "dry run" execution of the transaction to predict
    /// outcomes, catch errors early, and analyze execution logs before submission.
    ///
    /// Simulation Benefits:
    /// 1. Error Detection: Catches failures before expensive submission
    /// 2. Log Analysis: Provides execution logs for debugging
    /// 3. State Validation: Confirms transaction will succeed given current blockchain state
    /// 4. Cost Prevention: Avoids wasted transaction fees on failing operations
    ///
    /// Simulation Configuration:
    /// - `sig_verify`: false (bypasses signature validation for simulation)
    /// - `replace_recent_blockhash`: false (uses transaction's blockhash)
    /// - commitment: configurable (matches user's desired confirmation level)
    /// - `inner_instructions`: false (reduces simulation overhead)
    ///
    /// Response Format:
    /// - success: boolean indicating if transaction would succeed
    /// - error: detailed error message if simulation fails
    /// - logs: program execution logs for analysis and debugging
    ///
    /// Note: Simulation uses unsigned transaction since signatures aren't validated.
    /// This allows simulation of partially signed transactions during development.
    pub(super) async fn handle_simulate_transaction(
        &self,
        request: Request<SimulateTransactionRequest>,
    ) -> Result<Response<SimulateTransactionResponse>, Status> {
        let req = request.into_inner();
        let transaction = req
            .transaction
            .ok_or_else(|| Status::invalid_argument("Transaction is required"))?;

        // Validate current state allows simulation
        let current_state = transaction.state();
        validate_operation_allowed_for_state(current_state, "simulate")
            .map_err(Status::failed_precondition)?;

        // Validate transaction state consistency
        validate_transaction_state_consistency(&transaction)
            .map_err(|e| Status::invalid_argument(format!("Transaction validation failed: {e}")))?;

        // Ensure transaction has compiled data
        if transaction.data.is_empty() {
            return Err(Status::invalid_argument("Transaction must be compiled before simulation"));
        }

        // Deserialize the compiled transaction data
        let transaction_data = bs58::decode(&transaction.data).into_vec().map_err(|e| {
            Status::invalid_argument(format!("Failed to decode transaction data: {e}"))
        })?;

        let message: Message = bincode::deserialize(&transaction_data).map_err(|e| {
            Status::invalid_argument(format!("Failed to deserialize transaction: {e}"))
        })?;

        // Create an unsigned transaction for simulation
        let solana_transaction = SolanaTransaction::new_unsigned(message);

        // Get commitment level for simulation
        let commitment = super::commitment_level_to_config(req.commitment_level);

        // Simulate the transaction using RPC with configurable commitment level
        match self
            .rpc_client
            .simulate_transaction_with_config(
                &solana_transaction,
                solana_client::rpc_config::RpcSimulateTransactionConfig {
                    sig_verify: false,
                    replace_recent_blockhash: false,
                    commitment: Some(commitment),
                    encoding: None,
                    accounts: None,
                    min_context_slot: None,
                    inner_instructions: false,
                },
            )
            .await
        {
            Ok(simulation_result) => {
                let success = simulation_result.value.err.is_none();
                let error = simulation_result
                    .value
                    .err
                    .map(|err| format!("{err:?}"))
                    .unwrap_or_default();
                let logs = simulation_result.value.logs.unwrap_or_default();

                Ok(Response::new(SimulateTransactionResponse {
                    success,
                    error,
                    logs,
                }))
            }
            Err(e) => {
                // Simulation failed - this could be due to network issues or invalid transaction
                Ok(Response::new(SimulateTransactionResponse {
                    success: false,
                    error: format!("Simulation failed: {e}"),
                    logs: vec![],
                }))
            }
        }
    }
}
