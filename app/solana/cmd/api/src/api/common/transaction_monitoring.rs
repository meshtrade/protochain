use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::TransactionError;
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tonic::Status;
use tracing::{error, info, warn};

/// Waits for transaction confirmation AND success validation
///
/// This function provides comprehensive transaction monitoring by:
/// 1. Polling for transaction confirmation at specified commitment level
/// 2. Validating that the transaction actually succeeded (no execution errors)
/// 3. Providing detailed error classification for failures
/// 4. Handling timeouts gracefully to prevent hanging requests
///
/// Unlike basic confirmation checking, this ensures the transaction:
/// - Was confirmed on the blockchain
/// - Executed successfully without errors  
/// - Did not fail due to insufficient funds, invalid instructions, etc.
pub async fn wait_for_transaction_success(
    rpc_client: Arc<RpcClient>,
    signature: &Signature,
    commitment: CommitmentConfig,
    timeout_seconds: Option<u64>,
) -> Result<(), Status> {
    let timeout_duration = Duration::from_secs(timeout_seconds.unwrap_or(60));
    let start_time = Instant::now();
    let poll_interval = Duration::from_millis(500); // Poll every 500ms

    info!(
        signature = %signature,
        timeout_seconds = timeout_seconds.unwrap_or(60),
        "🔍 Starting transaction success validation"
    );

    loop {
        // Check if we've exceeded the timeout
        if start_time.elapsed() > timeout_duration {
            error!(
                signature = %signature,
                timeout_seconds = timeout_seconds.unwrap_or(60),
                "Transaction confirmation timeout exceeded"
            );
            return Err(Status::deadline_exceeded(format!(
                "Transaction confirmation timeout after {} seconds. Transaction may still be processing.",
                timeout_seconds.unwrap_or(60)
            )));
        }

        // Query the transaction from the network
        match rpc_client.get_transaction_with_config(
            signature,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Json),
                commitment: Some(commitment),
                max_supported_transaction_version: Some(0),
            },
        ) {
            Ok(confirmed_transaction) => {
                info!(
                    signature = %signature,
                    elapsed_ms = start_time.elapsed().as_millis(),
                    "✅ Transaction confirmed, validating success"
                );

                // Check if transaction execution succeeded
                if let Some(meta) = confirmed_transaction.transaction.meta {
                    match meta.err {
                        None => {
                            info!(
                                signature = %signature,
                                elapsed_ms = start_time.elapsed().as_millis(),
                                "🎉 Transaction executed successfully"
                            );
                            return Ok(());
                        }
                        Some(transaction_error) => {
                            warn!(
                                signature = %signature,
                                elapsed_ms = start_time.elapsed().as_millis(),
                                error = ?transaction_error,
                                "❌ Transaction failed during execution"
                            );
                            return Err(classify_transaction_failure(&transaction_error));
                        }
                    }
                }
                // Transaction confirmed but meta is missing - this is unusual
                warn!(
                    signature = %signature,
                    "Transaction confirmed but metadata missing - treating as success"
                );
                return Ok(());
            }
            Err(e) => {
                // Transaction not yet confirmed - continue polling unless it's a permanent error
                let error_str = e.to_string().to_lowercase();
                if error_str.contains("not found") || error_str.contains("transaction not found") {
                    // Transaction not found yet - continue polling
                    info!(
                        signature = %signature,
                        elapsed_ms = start_time.elapsed().as_millis(),
                        "⏳ Transaction not found yet, continuing to poll"
                    );
                } else {
                    // Network or other error - this might be temporary, continue polling
                    warn!(
                        signature = %signature,
                        elapsed_ms = start_time.elapsed().as_millis(),
                        error = %e,
                        "🌐 Network error while checking transaction, continuing to poll"
                    );
                }
            }
        }

        // Wait before next poll
        sleep(poll_interval).await;
    }
}

/// Classifies transaction failure reasons for user-friendly error messages
///
/// This function maps `TransactionError` variants to appropriate gRPC Status codes
/// with detailed, actionable error messages for common failure scenarios.
///
/// Error Classification Strategy:
/// - `InsufficientFunds*` → `InvalidArgument` with funding guidance
/// - `Signature*` → Unauthenticated with re-signing guidance  
/// - Network capacity → Unavailable with retry guidance
/// - Validation errors → `InvalidArgument` with specific details
/// - Program errors → `FailedPrecondition` with execution details
pub fn classify_transaction_failure(transaction_error: &TransactionError) -> Status {
    match transaction_error {
        // Insufficient funds errors - provide specific guidance
        TransactionError::InsufficientFundsForFee => Status::invalid_argument(
            "Transaction failed: insufficient funds to pay transaction fee. \
                Account needs additional SOL to cover network fees.",
        ),
        TransactionError::InsufficientFundsForRent { account_index } => {
            Status::invalid_argument(format!(
                "Transaction failed: insufficient funds for rent exemption (account index: {account_index}). \
                Accounts must maintain minimum balance for rent exemption. \
                Consider funding with at least 1 SOL (1,000,000,000 lamports)."
            ))
        }

        // Signature and authorization errors
        TransactionError::SignatureFailure => Status::unauthenticated(
            "Transaction failed: invalid signature. \
                Please verify all required accounts have signed the transaction correctly.",
        ),
        TransactionError::MissingSignatureForFee => Status::unauthenticated(
            "Transaction failed: missing signature from fee payer. \
                The fee payer account must sign the transaction.",
        ),

        // Network capacity issues (potentially retryable)
        TransactionError::WouldExceedMaxBlockCostLimit
        | TransactionError::WouldExceedMaxAccountCostLimit
        | TransactionError::WouldExceedMaxVoteCostLimit
        | TransactionError::WouldExceedAccountDataBlockLimit
        | TransactionError::WouldExceedAccountDataTotalLimit
        | TransactionError::TooManyAccountLocks => Status::unavailable(
            "Transaction failed: network capacity exceeded. \
                This is typically temporary - please retry in a few seconds.",
        ),
        TransactionError::ClusterMaintenance => Status::unavailable(
            "Transaction failed: blockchain cluster is under maintenance. \
                Please retry after maintenance is complete.",
        ),

        // Transaction already processed successfully
        TransactionError::AlreadyProcessed => Status::already_exists(
            "Transaction was already processed successfully. \
                No further action needed.",
        ),

        // Account and validation errors
        TransactionError::AccountNotFound => Status::not_found(
            "Transaction failed: required account not found. \
                Verify all account addresses are correct and funded.",
        ),
        TransactionError::ProgramAccountNotFound => Status::not_found(
            "Transaction failed: program account not found. \
                Verify the program is deployed and the address is correct.",
        ),
        TransactionError::InvalidAccountForFee => Status::invalid_argument(
            "Transaction failed: fee payer account is invalid. \
                Fee payer must be a valid, funded account.",
        ),
        TransactionError::BlockhashNotFound => Status::invalid_argument(
            "Transaction failed: recent blockhash is too old. \
                Please refresh the transaction with a current blockhash.",
        ),

        // Program execution errors require more detailed analysis
        TransactionError::InstructionError(instruction_index, instruction_error) => {
            classify_instruction_failure(*instruction_index, instruction_error)
        }

        // Other validation errors
        TransactionError::AccountInUse
        | TransactionError::AccountLoadedTwice
        | TransactionError::AccountBorrowOutstanding => Status::resource_exhausted(
            "Transaction failed: account access conflict. \
                This account is currently being used by another transaction. Please retry.",
        ),

        TransactionError::SanitizeFailure => Status::invalid_argument(
            "Transaction failed: transaction format is invalid. \
                Please check transaction construction and try again.",
        ),

        TransactionError::UnsupportedVersion => Status::unimplemented(
            "Transaction failed: transaction version not supported. \
                Please use a supported transaction format.",
        ),

        // Generic fallback for any other transaction errors
        _ => {
            error!(
                transaction_error = ?transaction_error,
                "Unhandled transaction error type"
            );
            Status::failed_precondition(format!(
                "Transaction failed with error: {transaction_error:?}. \
                Please review transaction parameters and try again."
            ))
        }
    }
}

/// Classifies instruction-level errors from program execution
///
/// Instruction errors occur during program execution and provide detailed
/// information about failures within specific transaction instructions.
fn classify_instruction_failure(
    instruction_index: u8,
    instruction_error: &solana_sdk::instruction::InstructionError,
) -> Status {
    use solana_sdk::instruction::InstructionError;

    match instruction_error {
        // Program detected insufficient funds
        InstructionError::InsufficientFunds => Status::invalid_argument(format!(
            "Instruction {instruction_index} failed: insufficient funds for program operation. \
                Please ensure the account has adequate balance for the requested operation."
        )),

        // Missing required signatures for instruction
        InstructionError::MissingRequiredSignature => Status::unauthenticated(format!(
            "Instruction {instruction_index} failed: missing required signature. \
                Please ensure all required accounts have signed the transaction."
        )),

        // Compute budget exhausted
        InstructionError::ComputationalBudgetExceeded => Status::resource_exhausted(format!(
            "Instruction {instruction_index} failed: computational budget exceeded. \
                The instruction requires too much computation time. \
                Consider simplifying the operation or increasing compute budget."
        )),

        // Program-specific custom errors
        InstructionError::Custom(error_code) => Status::failed_precondition(format!(
            "Instruction {instruction_index} failed with program-specific error code {error_code}. \
                Please check the program documentation for error code details."
        )),

        // Program execution panics and failures
        InstructionError::ProgramFailedToComplete => Status::failed_precondition(format!(
            "Instruction {instruction_index} failed: program execution did not complete successfully. \
                This may indicate a bug in the program or invalid input data."
        )),

        InstructionError::ProgramFailedToCompile => Status::failed_precondition(format!(
            "Instruction {instruction_index} failed: program failed to compile. \
                The program bytecode may be corrupted or incompatible."
        )),

        // Account access and validation errors
        InstructionError::NotEnoughAccountKeys => Status::invalid_argument(format!(
            "Instruction {instruction_index} failed: not enough account keys provided. \
                The instruction requires more accounts than were specified."
        )),

        InstructionError::AccountDataSizeChanged => Status::failed_precondition(format!(
            "Instruction {instruction_index} failed: account data size changed unexpectedly. \
                This may indicate concurrent modification of the account."
        )),

        InstructionError::AccountNotExecutable => Status::invalid_argument(format!(
            "Instruction {instruction_index} failed: attempted to execute a non-executable account. \
                Only executable program accounts can be invoked."
        )),

        // Fallback for other instruction errors
        _ => {
            error!(
                instruction_index = instruction_index,
                instruction_error = ?instruction_error,
                "Unhandled instruction error type"
            );
            Status::failed_precondition(format!(
                "Instruction {instruction_index} failed with error: {instruction_error:?}. \
                Please review instruction parameters and try again."
            ))
        }
    }
}

/// Convenience function to parse signature string and call `wait_for_transaction_success`
///
/// This wrapper handles signature parsing and provides a simpler interface
/// for services that work with signature strings rather than parsed Signature objects.
pub async fn wait_for_transaction_success_by_string(
    rpc_client: Arc<RpcClient>,
    signature_str: &str,
    commitment: CommitmentConfig,
    timeout_seconds: Option<u64>,
) -> Result<(), Status> {
    // Parse the signature string
    let signature = Signature::from_str(signature_str)
        .map_err(|e| Status::invalid_argument(format!("Invalid signature format: {e}")))?;

    wait_for_transaction_success(rpc_client, &signature, commitment, timeout_seconds).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::instruction::InstructionError;
    use solana_sdk::transaction::TransactionError;

    #[test]
    fn test_classify_insufficient_rent_error() {
        let error = TransactionError::InsufficientFundsForRent { account_index: 1 };
        let status = classify_transaction_failure(&error);

        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status
            .message()
            .contains("insufficient funds for rent exemption"));
        assert!(status.message().contains("account index: 1"));
        assert!(status.message().contains("1,000,000,000 lamports"));
    }

    #[test]
    fn test_classify_insufficient_fee_error() {
        let error = TransactionError::InsufficientFundsForFee;
        let status = classify_transaction_failure(&error);

        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status
            .message()
            .contains("insufficient funds to pay transaction fee"));
    }

    #[test]
    fn test_classify_signature_failure() {
        let error = TransactionError::SignatureFailure;
        let status = classify_transaction_failure(&error);

        assert_eq!(status.code(), tonic::Code::Unauthenticated);
        assert!(status.message().contains("invalid signature"));
    }

    #[test]
    fn test_classify_network_capacity_error() {
        let error = TransactionError::WouldExceedMaxBlockCostLimit;
        let status = classify_transaction_failure(&error);

        assert_eq!(status.code(), tonic::Code::Unavailable);
        assert!(status.message().contains("network capacity exceeded"));
        assert!(status.message().contains("retry"));
    }

    #[test]
    fn test_classify_instruction_insufficient_funds() {
        let error = TransactionError::InstructionError(2, InstructionError::InsufficientFunds);
        let status = classify_transaction_failure(&error);

        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("Instruction 2 failed"));
        assert!(status.message().contains("insufficient funds"));
    }

    #[test]
    fn test_classify_custom_instruction_error() {
        let error = TransactionError::InstructionError(0, InstructionError::Custom(42));
        let status = classify_transaction_failure(&error);

        assert_eq!(status.code(), tonic::Code::FailedPrecondition);
        assert!(status.message().contains("Instruction 0 failed"));
        assert!(status.message().contains("error code 42"));
    }

    #[test]
    fn test_already_processed_error() {
        let error = TransactionError::AlreadyProcessed;
        let status = classify_transaction_failure(&error);

        assert_eq!(status.code(), tonic::Code::AlreadyExists);
        assert!(status.message().contains("already processed successfully"));
    }

    #[test]
    fn test_invalid_signature_string_parsing() {
        // This would be an async test in practice, but testing the sync parsing part
        let result = Signature::from_str("invalid_signature");
        assert!(result.is_err());
    }
}
