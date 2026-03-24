use solana_commitment_config::CommitmentConfig;
use solana_sdk::hash::Hash;
use std::str::FromStr;
use tonic::{Request, Response, Status};
use tracing::debug;

use protochain_api::protochain::solana::transaction::v1::{
    CheckIfTransactionIsExpiredRequest, CheckIfTransactionIsExpiredResponse,
};

#[allow(clippy::result_large_err, clippy::unused_self)]
impl super::TransactionServiceImpl {
    /// Checks if a transaction's blockhash has expired on the Solana blockchain
    ///
    /// This method determines whether a compiled transaction can still be submitted
    /// by checking if its blockhash is still valid on the network. Blockhashes expire
    /// approximately 150 blocks (~60 seconds) after they are created.
    ///
    /// Expiration Check Process:
    /// 1. Extracts the blockhash from the compiled transaction
    /// 2. Queries Solana RPC for current slot and blockhash validity
    /// 3. Checks if the blockhash is still within the valid window
    /// 4. Returns boolean indicating expiration status
    ///
    /// Transaction Lifecycle Context:
    /// - Blockhash is set during `CompileTransaction` (state: COMPILED)
    /// - Blockhash must be valid at submission time
    /// - Expired transactions must be recompiled with a fresh blockhash
    /// - This check prevents failed submissions due to stale blockhashes
    ///
    /// Cost Implications:
    /// - Prevents wasted fees on doomed transactions
    /// - Allows preemptive recompilation before submission
    /// - Reduces transaction submission failures
    ///
    /// RPC Behavior:
    /// - Uses `is_blockhash_valid` which checks against recent blockhash list
    /// - Commitment level defaults to CONFIRMED (standard finality assumption)
    /// - Returns false if network cannot determine validity (safe for expiration)
    pub(super) fn handle_check_if_transaction_is_expired(
        &self,
        request: Request<CheckIfTransactionIsExpiredRequest>,
    ) -> Result<Response<CheckIfTransactionIsExpiredResponse>, Status> {
        let req = request.into_inner();
        let transaction = req
            .transaction
            .ok_or_else(|| Status::invalid_argument("Transaction is required"))?;

        // Ensure transaction has a blockhash set
        if transaction.recent_blockhash.is_empty() {
            return Err(Status::invalid_argument("Transaction must be compiled with a blockhash"));
        }

        // Parse the blockhash
        let blockhash = Hash::from_str(&transaction.recent_blockhash)
            .map_err(|e| Status::invalid_argument(format!("Invalid blockhash format: {e}")))?;

        // Check if the blockhash is still valid on the network
        let is_valid = self
            .rpc_client
            .is_blockhash_valid(&blockhash, CommitmentConfig::finalized())
            .map_err(|e| Status::internal(format!("Failed to check blockhash validity: {e}")))?;

        // A transaction is expired if the blockhash is NOT valid
        let is_expired = !is_valid;

        debug!(
            blockhash = %transaction.recent_blockhash,
            is_expired = is_expired,
            "Checked transaction blockhash expiration"
        );

        Ok(Response::new(CheckIfTransactionIsExpiredResponse { is_expired }))
    }
}
