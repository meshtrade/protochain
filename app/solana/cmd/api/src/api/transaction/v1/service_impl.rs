mod check_if_transaction_is_expired;
mod compile_transaction;
mod estimate_transaction;
mod get_transaction;
mod monitor_transaction;
mod sign_transaction;
mod simulate_transaction;
mod submit_transaction;

use crate::websocket::WebSocketManager;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_rpc_client_api::{
    client_error::{Error as ClientError, ErrorKind as ClientErrorKind},
    request::{RpcError, RpcResponseErrorData},
};
use solana_sdk::{instruction::InstructionError, transaction::TransactionError};
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::r#type::v1::CommitmentLevel;
use protochain_api::protochain::solana::transaction::v1::{
    service_server::Service as TransactionService, CheckIfTransactionIsExpiredRequest,
    CheckIfTransactionIsExpiredResponse, CompileTransactionRequest, CompileTransactionResponse,
    EstimateTransactionRequest, EstimateTransactionResponse, GetTransactionRequest,
    GetTransactionResponse, MonitorTransactionRequest, MonitorTransactionResponse,
    SignTransactionRequest, SignTransactionResponse, SimulateTransactionRequest,
    SimulateTransactionResponse, SubmissionResult, SubmitTransactionRequest,
    SubmitTransactionResponse,
};

/// Composable Transaction Service Implementation
///
/// This service implements the full transaction lifecycle for Solana blockchain operations:
/// - DRAFT -> COMPILED: Converts instructions into executable transaction bytecode
/// - COMPILED -> SIGNED: Applies cryptographic signatures for authorization
/// - SIGNED -> SUBMITTED: Broadcasts to network with commitment level handling
///
/// Key Architecture Decisions:
/// - Uses Arc<RpcClient> for thread-safe shared access to Solana RPC
/// - Integrates Arc<WebSocketManager> for real-time transaction monitoring
/// - All state transitions are validated to ensure transaction integrity
/// - Supports configurable commitment levels (processed/confirmed/finalized)
/// - Implements robust error classification for submission failures
///
/// Memory Management:
/// - Clone-based sharing for service instances across async contexts
/// - Arc-wrapped clients prevent use-after-free in concurrent operations
/// - Bincode serialization provides compact binary encoding for network transport
#[derive(Clone)]
pub struct TransactionServiceImpl {
    rpc_client: Arc<RpcClient>,
    websocket_manager: Arc<WebSocketManager>,
}

impl TransactionServiceImpl {
    /// Creates a new `TransactionServiceImpl` with the provided RPC client and WebSocket manager
    pub const fn new(rpc_client: Arc<RpcClient>, websocket_manager: Arc<WebSocketManager>) -> Self {
        Self {
            rpc_client,
            websocket_manager,
        }
    }
}

/// Classifies Solana RPC client errors into appropriate `SubmissionResult` categories
///
/// DEPRECATED: This function provides backward compatibility for the legacy enum classification.
/// New code should use `error_builder::build_structured_error` for comprehensive error analysis.
///
/// This function performs type-safe error analysis using Solana's structured error types
/// instead of fragile string pattern matching. It provides reliable classification based
/// on the actual error enums from the Solana codebase.
///
/// Type-Safe Classification Strategy:
/// 1. Direct `TransactionError` variants (most reliable)
/// 2. RPC preflight failure errors with embedded `TransactionError`
/// 3. Network/transport errors (Io, Reqwest)
/// 4. Signing errors from cryptographic operations
/// 5. Node health issues
/// 6. Fallback to string analysis for unstructured errors
///
/// Reference: Solana Agave codebase at https://github.com/anza-xyz/agave
/// - rpc-client-api/src/client_error.rs: Main `ClientError` structure
/// - rpc-client-types/src/request.rs: RPC error types and response data
/// - transaction-status/src/lib.rs: `TransactionError` enum variants
///
/// This approach provides reliable error classification that won't break with message
/// format changes and enables precise automated retry logic.
pub(super) fn classify_submission_error(error: &ClientError) -> SubmissionResult {
    match &*error.kind {
        // Direct transaction errors - most reliable classification path
        ClientErrorKind::TransactionError(transaction_error) => {
            classify_transaction_error(transaction_error)
        }

        // RPC response errors with embedded transaction simulation results
        // This occurs when send_transaction fails during preflight checks
        ClientErrorKind::RpcError(RpcError::RpcResponseError {
            data: RpcResponseErrorData::SendTransactionPreflightFailure(simulation_result),
            ..
        }) => simulation_result
            .err
            .as_ref()
            .map_or(SubmissionResult::FailedValidation, |e| {
                classify_transaction_error(&TransactionError::from(e.clone()))
            }),

        // Node health issues - network problems at the validator level
        ClientErrorKind::RpcError(RpcError::RpcResponseError {
            data: RpcResponseErrorData::NodeUnhealthy { .. },
            ..
        }) => SubmissionResult::FailedNetworkError,

        // Network transport errors - connectivity, timeouts, HTTP issues (INDETERMINATE)
        ClientErrorKind::Io(_) => SubmissionResult::Indeterminate,

        ClientErrorKind::Reqwest(_) => {
            // Connection/request/timeout failures - also indeterminate
            SubmissionResult::Indeterminate
        }

        // Cryptographic signing errors
        ClientErrorKind::SigningError(_) => SubmissionResult::FailedInvalidSignature,

        // JSON serialization/parsing errors - usually validation issues
        ClientErrorKind::SerdeJson(_) | ClientErrorKind::RpcError(RpcError::ParseError(_)) => {
            SubmissionResult::FailedValidation
        }

        // Fallback for unstructured errors - use string analysis as last resort
        ClientErrorKind::RpcError(_) => {
            // Generic RPC errors are typically indeterminate
            SubmissionResult::Indeterminate
        }

        ClientErrorKind::Custom(_) | ClientErrorKind::Middleware(_) => {
            // Only use string matching for truly unstructured error types
            classify_by_message(&error.to_string())
        }
    }
}

/// Classifies `TransactionError` variants into `SubmissionResult` categories
///
/// This function maps specific Solana transaction errors to actionable response categories
/// based on the transaction error variants defined in the Solana SDK.
///
/// Error Categories:
/// - `InsufficientFunds`: Account balance or fee issues requiring funding
/// - `InvalidSignature`: Cryptographic signature problems requiring re-signing
/// - `NetworkError`: Network capacity, maintenance, or timeout issues (retryable)
/// - Validation: Transaction format, account, or instruction issues (not retryable)
/// - Submitted: Transaction already processed (actually successful)
///
/// Reference: Solana transaction error definitions in transaction-status crate
const fn classify_transaction_error(transaction_error: &TransactionError) -> SubmissionResult {
    match transaction_error {
        // Account balance and fee-related errors
        TransactionError::InsufficientFundsForFee
        | TransactionError::InsufficientFundsForRent { .. } => {
            SubmissionResult::FailedInsufficientFunds
        }

        // Signature and authorization errors
        TransactionError::SignatureFailure | TransactionError::MissingSignatureForFee => {
            SubmissionResult::FailedInvalidSignature
        }

        // Network capacity and node availability issues (potentially retryable)
        TransactionError::WouldExceedMaxBlockCostLimit
        | TransactionError::WouldExceedMaxAccountCostLimit
        | TransactionError::WouldExceedMaxVoteCostLimit
        | TransactionError::WouldExceedAccountDataBlockLimit
        | TransactionError::WouldExceedAccountDataTotalLimit
        | TransactionError::TooManyAccountLocks
        | TransactionError::ClusterMaintenance => SubmissionResult::FailedNetworkError,

        // Transaction already successfully processed
        TransactionError::AlreadyProcessed => SubmissionResult::Submitted,

        // Account and validation errors (transaction format issues)
        TransactionError::AccountNotFound
        | TransactionError::ProgramAccountNotFound
        | TransactionError::InvalidAccountForFee
        | TransactionError::AccountInUse
        | TransactionError::AccountLoadedTwice
        | TransactionError::AccountBorrowOutstanding
        | TransactionError::BlockhashNotFound
        | TransactionError::CallChainTooDeep
        | TransactionError::InvalidAccountIndex
        | TransactionError::InvalidProgramForExecution
        | TransactionError::SanitizeFailure
        | TransactionError::UnsupportedVersion
        | TransactionError::InvalidWritableAccount
        | TransactionError::AddressLookupTableNotFound
        | TransactionError::InvalidAddressLookupTableOwner
        | TransactionError::InvalidAddressLookupTableData
        | TransactionError::InvalidAddressLookupTableIndex
        | TransactionError::InvalidRentPayingAccount
        | TransactionError::DuplicateInstruction(_)
        | TransactionError::MaxLoadedAccountsDataSizeExceeded
        | TransactionError::InvalidLoadedAccountsDataSizeLimit
        | TransactionError::ResanitizationNeeded
        | TransactionError::ProgramExecutionTemporarilyRestricted { .. }
        | TransactionError::UnbalancedTransaction
        | TransactionError::ProgramCacheHitMaxLimit
        | TransactionError::CommitCancelled => SubmissionResult::FailedValidation,

        // Instruction-level errors require detailed analysis
        TransactionError::InstructionError(instruction_index, instruction_error) => {
            classify_instruction_error(*instruction_index, instruction_error)
        }
    }
}

/// Classifies instruction-level errors that occur during program execution
///
/// Instruction errors provide detailed information about failures within specific
/// transaction instructions, enabling precise error handling for program-specific issues.
///
/// Reference: solana-sdk instruction error definitions
const fn classify_instruction_error(
    _instruction_index: u8,
    instruction_error: &InstructionError,
) -> SubmissionResult {
    match instruction_error {
        // Program detected insufficient funds (e.g., token transfer, program fee)
        InstructionError::InsufficientFunds => SubmissionResult::FailedInsufficientFunds,

        // Missing required signatures for instruction execution
        InstructionError::MissingRequiredSignature => SubmissionResult::FailedInvalidSignature,

        // Compute budget exhausted during execution
        InstructionError::ComputationalBudgetExceeded => SubmissionResult::FailedNetworkError,

        // Most instruction errors are validation issues - handled by wildcard below

        // Program-specific custom error codes
        InstructionError::Custom(_error_code) => {
            // Custom error codes are program-specific and could indicate various issues
            // Without context about the specific program, treat as validation error
            SubmissionResult::FailedValidation
        }

        // Any new instruction error variants default to validation
        _ => SubmissionResult::FailedValidation,
    }
}

/// Fallback error classification using string pattern matching
///
/// This function is used only when structured error information is not available,
/// serving as a compatibility layer for unstructured error messages.
///
/// This approach is intentionally limited and should only be reached for:
/// - Custom error messages that don't fit standard patterns
/// - Legacy error formats
/// - Middleware or proxy errors
///
/// The type-safe classification above should handle 95%+ of real-world cases.
fn classify_by_message(error_message: &str) -> SubmissionResult {
    let error_str = error_message.to_lowercase();

    if error_str.contains("insufficient")
        && (error_str.contains("fund") || error_str.contains("balance"))
    {
        SubmissionResult::FailedInsufficientFunds
    } else if error_str.contains("invalid") && error_str.contains("signature") {
        SubmissionResult::FailedInvalidSignature
    } else if error_str.contains("timeout") { 
        SubmissionResult::Indeterminate // don't know if the transaction was actually received
    } else if error_str.contains("network")
        || error_str.contains("connection")
    {
        SubmissionResult::FailedNetworkError
    } else {
        // Default to validation error for unknown unstructured errors
        SubmissionResult::FailedValidation
    }
}

/// Converts protobuf `CommitmentLevel` enum to Solana SDK `CommitmentConfig`
///
/// This function handles the impedance mismatch between protobuf enums and Rust enums,
/// providing safe conversion with fallback behavior for invalid or unspecified values.
///
/// Default Behavior:
/// - Uses CONFIRMED commitment as default (balances speed vs. reliability)
/// - Matches the account service default to maintain API consistency
/// - Invalid enum values fallback to CONFIRMED for predictable behavior
///
/// Commitment Levels Explained:
/// - PROCESSED: Fastest, least reliable (single validator confirmation)
/// - CONFIRMED: Balanced (supermajority of validators, ~400ms typical)
/// - FINALIZED: Slowest, most reliable (irreversible, ~13s typical)
///
/// The confirmed default prevents timing issues while maintaining reasonable performance.
pub(super) fn commitment_level_to_config(commitment_level: i32) -> CommitmentConfig {
    match CommitmentLevel::try_from(commitment_level) {
        Ok(CommitmentLevel::Processed) => CommitmentConfig::processed(),
        Ok(CommitmentLevel::Confirmed) => CommitmentConfig::confirmed(),
        Ok(CommitmentLevel::Finalized) => CommitmentConfig::finalized(),
        Ok(CommitmentLevel::Unspecified) | Err(_) => {
            // Default to confirmed for reliability - matches account service default
            CommitmentConfig::confirmed()
        }
    }
}

#[tonic::async_trait]
impl TransactionService for TransactionServiceImpl {
    type MonitorTransactionStream = ReceiverStream<Result<MonitorTransactionResponse, Status>>;

    async fn compile_transaction(
        &self,
        request: Request<CompileTransactionRequest>,
    ) -> Result<Response<CompileTransactionResponse>, Status> {
        self.handle_compile_transaction(request).await
    }

    async fn estimate_transaction(
        &self,
        request: Request<EstimateTransactionRequest>,
    ) -> Result<Response<EstimateTransactionResponse>, Status> {
        self.handle_estimate_transaction(request).await
    }

    async fn simulate_transaction(
        &self,
        request: Request<SimulateTransactionRequest>,
    ) -> Result<Response<SimulateTransactionResponse>, Status> {
        self.handle_simulate_transaction(request).await
    }

    async fn sign_transaction(
        &self,
        request: Request<SignTransactionRequest>,
    ) -> Result<Response<SignTransactionResponse>, Status> {
        self.handle_sign_transaction(request)
    }

    async fn check_if_transaction_is_expired(
        &self,
        request: Request<CheckIfTransactionIsExpiredRequest>,
    ) -> Result<Response<CheckIfTransactionIsExpiredResponse>, Status> {
        self.handle_check_if_transaction_is_expired(request).await
    }

    async fn submit_transaction(
        &self,
        request: Request<SubmitTransactionRequest>,
    ) -> Result<Response<SubmitTransactionResponse>, Status> {
        self.handle_submit_transaction(request).await
    }

    async fn get_transaction(
        &self,
        request: Request<GetTransactionRequest>,
    ) -> Result<Response<GetTransactionResponse>, Status> {
        self.handle_get_transaction(request).await
    }

    async fn monitor_transaction(
        &self,
        request: Request<MonitorTransactionRequest>,
    ) -> Result<Response<Self::MonitorTransactionStream>, Status> {
        self.handle_monitor_transaction(request)
    }
}
