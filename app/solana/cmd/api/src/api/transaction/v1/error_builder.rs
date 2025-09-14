use protochain_api::protochain::solana::transaction::v1::{
    SubmissionResult, TransactionError, TransactionErrorCode, TransactionSubmissionCertainty,
};
use serde_json;
use solana_rpc_client_api::client_error::Error as ClientError;
use solana_rpc_client_api::{
    client_error::ErrorKind as ClientErrorKind,
    request::{RpcError, RpcResponseErrorData},
};
use solana_sdk::{
    clock::Slot, hash::Hash, instruction::InstructionError,
    transaction::TransactionError as SdkTransactionError,
};

/// Builds structured error responses for transaction submission failures
///
/// This module provides the core logic for mapping Solana RPC client errors to
/// comprehensive structured error responses that enable precise error handling
/// and resolution strategies for production systems.
///
/// Key Principles:
/// 1. **Re-signing Test**: If error requires new transaction with new signatures = PERMANENT
/// 2. **Certainty Classification**: Distinguish errors that occur before vs during/after submission
/// 3. **Blockhash Resolution**: Provide timing information for resolving indeterminate states
/// 4. **Backward Compatibility**: Maintain existing error classification while enhancing

/// Builds a structured `TransactionError` from Solana client error and context
///
/// This function performs comprehensive error analysis to provide:
/// - Specific error codes for programmatic handling
/// - Certainty indicators about transaction submission status
/// - Blockhash resolution timing for indeterminate states
/// - Retryability indicators based on error classification
/// - Rich context details as JSON for advanced debugging
pub fn build_structured_error(
    client_error: &ClientError,
    _submission_result: SubmissionResult,
    transaction_blockhash: &Hash,
    current_slot: Slot,
) -> TransactionError {
    let expiry_slot = current_slot + 150; // Standard blockhash validity window

    // Classify the error and determine certainty
    let (error_code, certainty) = classify_error_with_certainty(client_error);

    // Determine retryability based on error classification
    let retryable = determine_retryability(error_code);

    // Build comprehensive error details
    let details = extract_error_details(client_error);

    TransactionError {
        code: error_code.into(),
        message: format!("Transaction submission failed: {client_error}"),
        details,
        retryable,
        certainty: certainty.into(),
        blockhash: transaction_blockhash.to_string(),
        blockhash_expiry_slot: expiry_slot,
    }
}

/// Classifies client errors into specific error codes with certainty assessment
///
/// This function implements the core error classification logic based on the
/// "re-signing test" principle and certainty about transaction submission status.
///
/// Returns (`TransactionErrorCode`, `TransactionSubmissionCertainty`) tuple
fn classify_error_with_certainty(
    client_error: &ClientError,
) -> (TransactionErrorCode, TransactionSubmissionCertainty) {
    match &client_error.kind {
        // Preflight failures - CERTAIN transaction was NOT sent
        ClientErrorKind::RpcError(RpcError::RpcResponseError {
            data: RpcResponseErrorData::SendTransactionPreflightFailure(simulation_result),
            ..
        }) => {
            let certainty = TransactionSubmissionCertainty::NotSubmitted;

            if let Some(ref tx_error) = simulation_result.err {
                (classify_transaction_error(tx_error), certainty)
            } else {
                (TransactionErrorCode::InvalidTransaction, certainty)
            }
        }

        // Direct transaction errors - usually from preflight or validation
        ClientErrorKind::TransactionError(transaction_error) => {
            let certainty = TransactionSubmissionCertainty::NotSubmitted;
            (classify_transaction_error(transaction_error), certainty)
        }

        // Node health issues - INDETERMINATE (might have received it first)
        ClientErrorKind::RpcError(RpcError::RpcResponseError {
            data: RpcResponseErrorData::NodeUnhealthy { .. },
            ..
        }) => (
            TransactionErrorCode::NodeUnhealthy,
            TransactionSubmissionCertainty::UnknownResolvable,
        ),

        // Network transport errors - INDETERMINATE (could fail during/after sending)
        ClientErrorKind::Io(_) => (
            TransactionErrorCode::NetworkError,
            TransactionSubmissionCertainty::UnknownResolvable,
        ),

        ClientErrorKind::Reqwest(reqwest_error) => {
            if reqwest_error.is_timeout() {
                // Timeouts are especially dangerous - transaction might have been sent
                (TransactionErrorCode::Timeout, TransactionSubmissionCertainty::UnknownResolvable)
            } else if reqwest_error.is_connect() {
                (
                    TransactionErrorCode::ConnectionFailed,
                    TransactionSubmissionCertainty::UnknownResolvable,
                )
            } else {
                (
                    TransactionErrorCode::RequestFailed,
                    TransactionSubmissionCertainty::UnknownResolvable,
                )
            }
        }

        // Cryptographic signing errors - CERTAIN transaction was NOT sent
        ClientErrorKind::SigningError(_) => (
            TransactionErrorCode::InvalidSignature,
            TransactionSubmissionCertainty::NotSubmitted,
        ),

        // JSON serialization/parsing errors - CERTAIN transaction was NOT sent
        ClientErrorKind::SerdeJson(_) | ClientErrorKind::RpcError(RpcError::ParseError(_)) => (
            TransactionErrorCode::InvalidTransaction,
            TransactionSubmissionCertainty::NotSubmitted,
        ),

        // Generic RPC errors - usually INDETERMINATE
        ClientErrorKind::RpcError(_) => (
            TransactionErrorCode::RpcError,
            TransactionSubmissionCertainty::UnknownResolvable,
        ),

        // Custom errors - fallback to unknown with indeterminate certainty
        ClientErrorKind::Custom(_) => {
            (TransactionErrorCode::Unknown, TransactionSubmissionCertainty::Unknown)
        }
    }
}

/// Maps Solana SDK `TransactionError` to our `TransactionErrorCode` enum
///
/// This function implements the "re-signing test" principle:
/// - PERMANENT failures require rebuilding and re-signing the transaction
/// - TEMPORARY failures can be resolved with the same signed transaction
const fn classify_transaction_error(
    transaction_error: &SdkTransactionError,
) -> TransactionErrorCode {
    match transaction_error {
        // Account balance and fee-related errors (TEMPORARY - just add funds)
        SdkTransactionError::InsufficientFundsForFee
        | SdkTransactionError::InsufficientFundsForRent { .. } => {
            TransactionErrorCode::InsufficientFunds
        }

        // Signature and authorization errors (PERMANENT - need re-signing)
        SdkTransactionError::SignatureFailure | SdkTransactionError::MissingSignatureForFee => {
            TransactionErrorCode::SignatureVerificationFailed
        }

        // Network capacity issues (TEMPORARY - try next block)
        SdkTransactionError::WouldExceedMaxBlockCostLimit
        | SdkTransactionError::WouldExceedMaxAccountCostLimit
        | SdkTransactionError::WouldExceedMaxVoteCostLimit
        | SdkTransactionError::WouldExceedAccountDataBlockLimit
        | SdkTransactionError::WouldExceedAccountDataTotalLimit => {
            TransactionErrorCode::WouldExceedBlockLimit
        }

        // Account locking issues (TEMPORARY - wait for unlock)
        SdkTransactionError::TooManyAccountLocks => TransactionErrorCode::AccountInUse,

        // Network maintenance (TEMPORARY)
        SdkTransactionError::ClusterMaintenance => TransactionErrorCode::TransientSimulationFailure,

        // Account-related validation errors (PERMANENT)
        SdkTransactionError::AccountNotFound | SdkTransactionError::ProgramAccountNotFound => {
            TransactionErrorCode::AccountNotFound
        }

        SdkTransactionError::InvalidAccountForFee
        | SdkTransactionError::AccountLoadedTwice
        | SdkTransactionError::AccountBorrowOutstanding
        | SdkTransactionError::InvalidWritableAccount
        | SdkTransactionError::InvalidRentPayingAccount => TransactionErrorCode::InvalidAccount,

        // Blockhash errors (PERMANENT - requires re-signing with new blockhash)
        SdkTransactionError::BlockhashNotFound => TransactionErrorCode::BlockhashNotFound,

        // Transaction structure errors (PERMANENT)
        SdkTransactionError::SanitizeFailure
        | SdkTransactionError::UnsupportedVersion
        | SdkTransactionError::DuplicateInstruction(_)
        | SdkTransactionError::UnbalancedTransaction => TransactionErrorCode::InvalidTransaction,

        // Program execution and instruction errors
        SdkTransactionError::InstructionError(instruction_index, instruction_error) => {
            classify_instruction_error(*instruction_index, instruction_error)
        }

        // Address lookup table errors (PERMANENT)
        SdkTransactionError::AddressLookupTableNotFound
        | SdkTransactionError::InvalidAddressLookupTableOwner
        | SdkTransactionError::InvalidAddressLookupTableData
        | SdkTransactionError::InvalidAddressLookupTableIndex => {
            TransactionErrorCode::InvalidTransaction
        }

        // Other validation errors (PERMANENT)
        SdkTransactionError::CallChainTooDeep
        | SdkTransactionError::InvalidAccountIndex
        | SdkTransactionError::InvalidProgramForExecution
        | SdkTransactionError::MaxLoadedAccountsDataSizeExceeded
        | SdkTransactionError::InvalidLoadedAccountsDataSizeLimit
        | SdkTransactionError::ResanitizationNeeded
        | SdkTransactionError::ProgramExecutionTemporarilyRestricted { .. } => {
            TransactionErrorCode::InvalidTransaction
        }

        // Successfully processed (shouldn't be an error, but handle gracefully)
        SdkTransactionError::AlreadyProcessed => TransactionErrorCode::InvalidTransaction,

        // Account in use (TEMPORARY - wait for account to be available)
        SdkTransactionError::AccountInUse => TransactionErrorCode::AccountInUse,
    }
}

/// Classifies instruction-level errors
///
/// Provides detailed classification for errors that occur during program execution
const fn classify_instruction_error(
    _instruction_index: u8,
    instruction_error: &InstructionError,
) -> TransactionErrorCode {
    match instruction_error {
        // Program detected insufficient funds (TEMPORARY)
        InstructionError::InsufficientFunds => TransactionErrorCode::InsufficientFunds,

        // Missing required signatures (PERMANENT - need re-signing)
        InstructionError::MissingRequiredSignature => {
            TransactionErrorCode::SignatureVerificationFailed
        }

        // Compute budget exhausted (TEMPORARY - network capacity issue)
        InstructionError::ComputationalBudgetExceeded => {
            TransactionErrorCode::WouldExceedBlockLimit
        }

        // Program-specific custom error codes (PERMANENT - program logic issues)
        InstructionError::Custom(_error_code) => TransactionErrorCode::ProgramError,

        // All other instruction errors are program/instruction issues (PERMANENT)
        _ => TransactionErrorCode::InstructionError,
    }
}

/// Determines if an error type is retryable
///
/// Based on the error classification, determines whether retrying the same
/// transaction (or a similar one) might succeed
pub const fn determine_retryability(code: TransactionErrorCode) -> bool {
    match code {
        // TEMPORARY failures - same transaction might succeed later
        TransactionErrorCode::InsufficientFunds
        | TransactionErrorCode::AccountInUse
        | TransactionErrorCode::WouldExceedBlockLimit
        | TransactionErrorCode::TransientSimulationFailure => true,

        // INDETERMINATE failures - might be worth investigating/resolving
        TransactionErrorCode::NetworkError
        | TransactionErrorCode::Timeout
        | TransactionErrorCode::NodeUnhealthy
        | TransactionErrorCode::RateLimited
        | TransactionErrorCode::RpcError
        | TransactionErrorCode::ConnectionFailed
        | TransactionErrorCode::RequestFailed => true,

        // PERMANENT failures - will never succeed as-is
        _ => false,
    }
}

/// Extracts detailed error context as JSON string
///
/// Provides rich debugging information while maintaining structured format
/// for programmatic parsing if needed
pub fn extract_error_details(client_error: &ClientError) -> String {
    let details = match &client_error.kind {
        ClientErrorKind::RpcError(RpcError::RpcResponseError { data, .. }) => match data {
            RpcResponseErrorData::SendTransactionPreflightFailure(simulation_result) => {
                serde_json::json!({
                    "type": "preflight_failure",
                    "simulation_error": simulation_result.err,
                    "logs": simulation_result.logs,
                    "units_consumed": simulation_result.units_consumed,
                    "accounts": simulation_result.accounts
                })
            }
            RpcResponseErrorData::NodeUnhealthy { num_slots_behind } => {
                serde_json::json!({
                    "type": "node_unhealthy",
                    "slots_behind": num_slots_behind
                })
            }
            RpcResponseErrorData::Empty => {
                serde_json::json!({
                    "type": "empty_rpc_error",
                    "message": "RPC error with no additional data"
                })
            }
        },
        ClientErrorKind::Reqwest(reqwest_error) => {
            serde_json::json!({
                "type": "http_error",
                "is_timeout": reqwest_error.is_timeout(),
                "is_connect": reqwest_error.is_connect(),
                "is_request": reqwest_error.is_request(),
                "is_redirect": reqwest_error.is_redirect(),
                "status_code": reqwest_error.status().map(|s| s.as_u16())
            })
        }
        ClientErrorKind::Io(io_error) => {
            serde_json::json!({
                "type": "io_error",
                "kind": format!("{:?}", io_error.kind()),
                "os_error": io_error.raw_os_error()
            })
        }
        ClientErrorKind::TransactionError(tx_error) => {
            serde_json::json!({
                "type": "transaction_error",
                "error": format!("{:?}", tx_error)
            })
        }
        ClientErrorKind::SigningError(signing_error) => {
            serde_json::json!({
                "type": "signing_error",
                "error": signing_error.to_string()
            })
        }
        ClientErrorKind::SerdeJson(serde_error) => {
            serde_json::json!({
                "type": "serialization_error",
                "line": serde_error.line(),
                "column": serde_error.column()
            })
        }
        ClientErrorKind::RpcError(rpc_error) => {
            serde_json::json!({
                "type": "rpc_error",
                "error": format!("{:?}", rpc_error)
            })
        }
        ClientErrorKind::Custom(custom_error) => {
            serde_json::json!({
                "type": "custom_error",
                "error": custom_error
            })
        }
    };

    details.to_string()
}
