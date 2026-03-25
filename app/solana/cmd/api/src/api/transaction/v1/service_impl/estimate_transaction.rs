use solana_sdk::{message::Message, transaction::Transaction as SolanaTransaction};
use tonic::{Request, Response, Status};

use crate::api::transaction::v1::validation::{
    validate_operation_allowed_for_state, validate_transaction_state_consistency,
};
use protochain_api::protochain::solana::transaction::v1::{
    EstimateTransactionRequest, EstimateTransactionResponse,
};

#[allow(clippy::result_large_err)]
impl super::TransactionServiceImpl {
    /// Estimates compute units and transaction fees for a compiled transaction
    ///
    /// This method provides accurate resource consumption estimates by simulating
    /// transaction execution without actually submitting to the blockchain.
    ///
    /// Estimation Strategy:
    /// 1. Primary: Uses RPC `simulate_transaction_with_config` for real execution analysis
    /// 2. Fallback: Instruction-count-based heuristics if simulation fails
    /// 3. Handles both None and 0 compute units with reasonable defaults
    ///
    /// Compute Unit Estimation:
    /// - Real simulation: Uses actual execution consumption when available
    /// - Fallback formula: instructions * 50,000 CU (realistic per-instruction average)
    /// - Bounds: minimum 200,000 CU, maximum 1,400,000 CU (network limits)
    ///
    /// Fee Calculation:
    /// - Base fee: 5,000 lamports (standard transaction fee)
    /// - Priority fee: `compute_units` * `compute_unit_price` (from transaction config)
    /// - Caps priority fee at 1,000,000 lamports to prevent excessive costs
    /// - Fallback priority fee: 1,000 lamports for network prioritization
    ///
    /// The estimation accuracy helps users avoid transaction failures due to
    /// insufficient fees or compute budget exhaustion.
    pub(super) async fn handle_estimate_transaction(
        &self,
        request: Request<EstimateTransactionRequest>,
    ) -> Result<Response<EstimateTransactionResponse>, Status> {
        let req = request.into_inner();
        let transaction = req
            .transaction
            .ok_or_else(|| Status::invalid_argument("Transaction is required"))?;

        // Validate current state allows estimation
        let current_state = transaction.state();
        validate_operation_allowed_for_state(current_state, "estimate")
            .map_err(Status::failed_precondition)?;

        // Validate transaction state consistency
        validate_transaction_state_consistency(&transaction)
            .map_err(|e| Status::invalid_argument(format!("Transaction validation failed: {e}")))?;

        // Ensure transaction has compiled data
        if transaction.data.is_empty() {
            return Err(Status::invalid_argument("Transaction must be compiled before estimation"));
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

        // Get commitment level for estimation simulation
        let commitment = super::commitment_level_to_config(req.commitment_level);

        // Use simulation to get accurate compute unit estimation with configurable commitment level
        let (compute_units, _logs) = if let Ok(simulation_result) = self
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
            // Handle both None and 0 cases by providing reasonable fallback
            let compute_units = match simulation_result.value.units_consumed {
                Some(units) if units > 0 => units,
                _ => {
                    // Fallback estimation based on instruction count
                    let instruction_count = transaction.instructions.len() as u64;
                    (instruction_count * 50_000).clamp(200_000, 1_400_000)
                }
            };
            let logs = simulation_result.value.logs.unwrap_or_default();
            (compute_units, logs)
        } else {
            // Fallback to basic estimation if simulation fails
            let instruction_count = transaction.instructions.len() as u64;
            let estimated_compute_units = (instruction_count * 50_000).clamp(200_000, 1_400_000);
            (estimated_compute_units, vec![])
        };

        // Calculate fee estimation
        let base_fee_lamports = 5_000; // Base transaction fee
        let compute_unit_price = transaction
            .config
            .as_ref()
            .map_or(0, |config| config.compute_unit_price);

        // Priority fee calculation based on compute units and price
        let priority_fee = if compute_unit_price > 0 {
            (compute_units * compute_unit_price).min(1_000_000) // Cap priority fee
        } else {
            // Default priority fee estimation based on network conditions
            1_000
        };

        let total_fee = base_fee_lamports + priority_fee;

        Ok(Response::new(EstimateTransactionResponse {
            compute_units,
            fee_lamports: total_fee,
            priority_fee,
        }))
    }
}
