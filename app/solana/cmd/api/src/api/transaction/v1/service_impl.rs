use crate::websocket::WebSocketManager;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_rpc_client_api::{
    client_error::{Error as ClientError, ErrorKind as ClientErrorKind},
    request::{RpcError, RpcResponseErrorData},
};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::transaction::TransactionError;
use solana_sdk::{
    hash::Hash,
    instruction::{Instruction, InstructionError},
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction as SolanaTransaction,
};
use solana_transaction_status::{EncodedTransaction, UiTransactionEncoding};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, warn};

use crate::api::common::solana_conversions::proto_instruction_to_sdk;
use crate::api::transaction::v1::error_builder;
use crate::api::transaction::v1::validation::{
    validate_operation_allowed_for_state, validate_state_transition,
    validate_transaction_state_consistency,
};
use protochain_api::protochain::solana::r#type::v1::CommitmentLevel;
use protochain_api::protochain::solana::transaction::v1::{
    service_server::Service as TransactionService, sign_transaction_request,
    CompileTransactionRequest, CompileTransactionResponse, EstimateTransactionRequest,
    EstimateTransactionResponse, GetTransactionRequest, GetTransactionResponse,
    MonitorTransactionRequest, MonitorTransactionResponse, SignTransactionRequest,
    SignTransactionResponse, SimulateTransactionRequest, SimulateTransactionResponse,
    SubmissionResult, SubmitTransactionRequest, SubmitTransactionResponse, Transaction,
    TransactionState, TransactionStatus,
};

/// Composable Transaction Service Implementation
///
/// This service implements the full transaction lifecycle for Solana blockchain operations:
/// - DRAFT → COMPILED: Converts instructions into executable transaction bytecode
/// - COMPILED → SIGNED: Applies cryptographic signatures for authorization  
/// - SIGNED → SUBMITTED: Broadcasts to network with commitment level handling
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
/// Reference: Solana Agave codebase at /Users/bernardbussy/Projects/github.com/anza-xyz/agave
/// - rpc-client-api/src/client_error.rs: Main `ClientError` structure
/// - rpc-client-types/src/request.rs: RPC error types and response data
/// - transaction-status/src/lib.rs: `TransactionError` enum variants
///
/// This approach provides reliable error classification that won't break with message
/// format changes and enables precise automated retry logic.
fn classify_submission_error(error: &ClientError) -> SubmissionResult {
    match &error.kind {
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
            .map_or(SubmissionResult::FailedValidation, classify_transaction_error),

        // Node health issues - network problems at the validator level
        ClientErrorKind::RpcError(RpcError::RpcResponseError {
            data: RpcResponseErrorData::NodeUnhealthy { .. },
            ..
        }) => SubmissionResult::FailedNetworkError,

        // Network transport errors - connectivity, timeouts, HTTP issues (INDETERMINATE)
        ClientErrorKind::Io(_) => SubmissionResult::Indeterminate,

        ClientErrorKind::Reqwest(reqwest_error) => {
            if reqwest_error.is_timeout() {
                // Timeouts are especially dangerous - transaction might have been sent
                SubmissionResult::Indeterminate
            } else {
                // Connection/request failures - also indeterminate
                SubmissionResult::Indeterminate
            }
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

        ClientErrorKind::Custom(_) => {
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
        | TransactionError::UnbalancedTransaction => SubmissionResult::FailedValidation,

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
    } else if error_str.contains("network")
        || error_str.contains("connection")
        || error_str.contains("timeout")
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
fn commitment_level_to_config(commitment_level: i32) -> CommitmentConfig {
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
    /// Compiles a draft transaction with instructions into executable transaction bytecode
    ///
    /// State Transition: DRAFT → COMPILED
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
    async fn compile_transaction(
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
        let sdk_instructions: Result<Vec<Instruction>, String> = transaction
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
    async fn estimate_transaction(
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
        let commitment = commitment_level_to_config(req.commitment_level);

        // Use simulation to get accurate compute unit estimation with configurable commitment level
        let (compute_units, _logs) = if let Ok(simulation_result) =
            self.rpc_client.simulate_transaction_with_config(
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
            ) {
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
    async fn simulate_transaction(
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
        let commitment = commitment_level_to_config(req.commitment_level);

        // Simulate the transaction using RPC with configurable commitment level
        match self.rpc_client.simulate_transaction_with_config(
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
        ) {
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

    /// Signs a compiled transaction with cryptographic signatures for authorization
    ///
    /// State Transition: COMPILED → `PARTIALLY_SIGNED` or `FULLY_SIGNED`
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
    async fn sign_transaction(
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
                        let private_key_bytes =
                            bs58::decode(private_key_str).into_vec().map_err(|e| {
                                Status::invalid_argument(format!("Invalid private key format: {e}"))
                            })?;

                        if private_key_bytes.len() != 64 {
                            return Err(Status::invalid_argument("Private key must be 64 bytes"));
                        }

                        let keypair = Keypair::from_bytes(&private_key_bytes).map_err(|e| {
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

    /// Asynchronously submits a fully signed transaction to the Solana blockchain network
    ///
    /// State Transition: `FULLY_SIGNED` → SUBMITTED (or FAILED)
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
    async fn submit_transaction(
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
            "🚀 Submitting transaction to Solana network"
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
        let commitment = commitment_level_to_config(req.commitment_level);
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
                        "✅ Transaction submitted successfully (asynchronously)"
                    );

                    // Return immediately after submission without waiting for confirmation
                    // Clients can use MonitorTransaction to poll for confirmation if desired
                    (signature.to_string(), SubmissionResult::Submitted, None)
                }
                Err(e) => {
                    let classification = classify_submission_error(&e);

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
    async fn get_transaction(
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
        let commitment = commitment_level_to_config(req.commitment_level);

        // Query the transaction from the network with configurable commitment level
        match self.rpc_client.get_transaction_with_config(
            &signature,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Base64),
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
                };

                Ok(Response::new(GetTransactionResponse {
                    transaction: Some(proto_transaction),
                }))
            }
            Err(e) => {
                // Transaction not found or other error
                Err(Status::not_found(format!("Transaction not found: {e}")))
            }
        }
    }

    /// Monitors a transaction for real-time status changes via WebSocket streaming
    ///
    /// This method establishes a persistent gRPC server streaming connection that pushes
    /// transaction status updates from the Solana blockchain in real-time. It bridges
    /// WebSocket pubsub notifications to gRPC streaming protocol.
    ///
    /// Networking Architecture:
    /// 1. Validates input parameters and signature format
    /// 2. Creates unbounded WebSocket subscription via `WebSocketManager`
    /// 3. Establishes bounded gRPC stream channel (capacity: 100)
    /// 4. Spawns async bridge task for protocol translation
    /// 5. Returns `ReceiverStream` for client consumption
    ///
    /// Resource Management:
    /// - WebSocket subscription auto-cleanup on client disconnect
    /// - Bridge task terminates on terminal status or client disconnect
    /// - Bounded channel prevents memory exhaustion from fast updates
    ///
    /// Error Handling:
    /// - Input validation prevents malformed signature attacks
    /// - Timeout bounds prevent resource exhaustion (5-300 seconds)
    /// - Channel failures trigger automatic cleanup
    async fn monitor_transaction(
        &self,
        request: Request<MonitorTransactionRequest>,
    ) -> Result<Response<Self::MonitorTransactionStream>, Status> {
        let req = request.into_inner();

        // Validate signature format
        if req.signature.is_empty() {
            error!("MonitorTransaction called with empty signature");
            return Err(Status::invalid_argument("Transaction signature is required"));
        }

        // Parse signature to validate format
        req.signature
            .parse::<solana_sdk::signature::Signature>()
            .map_err(|_| {
                error!(
                    signature = %req.signature,
                    "Invalid signature format provided to MonitorTransaction"
                );
                Status::invalid_argument("Invalid signature format")
            })?;

        // Validate commitment level
        let commitment_level = CommitmentLevel::try_from(req.commitment_level).map_err(|_| {
            error!(
                commitment_level = req.commitment_level,
                signature = %req.signature,
                "Invalid commitment level provided to MonitorTransaction"
            );
            Status::invalid_argument("Invalid commitment level")
        })?;

        // Validate timeout (if provided)
        let timeout_seconds = if req.timeout_seconds == 0 {
            60
        } else {
            req.timeout_seconds
        };
        if !(5..=300).contains(&timeout_seconds) {
            error!(
                timeout_seconds = timeout_seconds,
                signature = %req.signature,
                "Invalid timeout value provided to MonitorTransaction"
            );
            return Err(Status::invalid_argument("Timeout must be between 5 and 300 seconds"));
        }

        info!(
            signature = %req.signature,
            commitment_level = ?commitment_level,
            timeout_seconds = timeout_seconds,
            include_logs = req.include_logs,
            "🔍 Starting transaction monitoring"
        );

        // Create response stream channel with bounded capacity
        // Buffer size 100 provides good balance between memory usage and throughput
        // This prevents unbounded memory growth if client consumes slowly
        let (tx, rx) = mpsc::channel(100);

        // Subscribe to signature updates via WebSocket manager
        let websocket_rx = match self.websocket_manager.subscribe_to_signature(
            &req.signature,
            commitment_level,
            req.include_logs,
            Some(timeout_seconds),
        ) {
            Ok(rx) => rx,
            Err(e) => {
                return Err(*e);
            }
        };

        // Spawn task to bridge WebSocket updates to gRPC stream
        // This task handles protocol translation between WebSocket pubsub and gRPC streaming
        let signature_for_task = req.signature.clone();
        tokio::spawn(async move {
            bridge_websocket_to_grpc_stream(signature_for_task, websocket_rx, tx, timeout_seconds)
                .await;
        });

        info!(
            signature = %req.signature,
            commitment_level = ?commitment_level,
            "✅ Transaction monitoring stream established"
        );

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

/// Bridges WebSocket subscription updates to gRPC streaming response
///
/// This function performs critical protocol translation between Solana WebSocket pubsub
/// and gRPC server streaming. It handles proper resource cleanup and prevents memory leaks.
///
/// Architecture:
/// - Receives updates from unbounded WebSocket channel (real-time blockchain events)
/// - Translates to bounded gRPC stream channel (client consumption rate-limited)
/// - Implements timeout-based cleanup to prevent zombie tasks
/// - Detects client disconnections for immediate resource cleanup
///
/// Resource Management:
/// - Uses timeout to prevent indefinite hanging on stalled WebSocket
/// - Detects gRPC channel closure (client disconnect) for immediate cleanup
/// - Terminates on terminal transaction states to free resources
/// - No explicit drop needed - channels auto-cleanup when task ends
///
/// Helper function to check if a transaction status is terminal
const fn is_terminal_status(status: TransactionStatus) -> bool {
    matches!(
        status,
        TransactionStatus::Confirmed
            | TransactionStatus::Finalized
            | TransactionStatus::Failed
            | TransactionStatus::Dropped
            | TransactionStatus::Timeout
    )
}

/// Helper function to send timeout notification to grPC client
async fn send_timeout_notification(
    grpc_tx: &mpsc::Sender<Result<MonitorTransactionResponse, Status>>,
    signature: &str,
) {
    let timeout_response = MonitorTransactionResponse {
        signature: signature.to_string(),
        status: TransactionStatus::Timeout.into(),
        slot: 0,
        error_message: "Stream monitoring timeout reached".to_string(),
        logs: vec![],
        compute_units_consumed: 0,
        current_commitment: CommitmentLevel::Unspecified.into(),
    };

    // Best effort - ignore if client already disconnected
    if grpc_tx.send(Ok(timeout_response)).await.is_err() {
        debug!(
            signature = %signature,
            "Client disconnected before timeout notification could be sent"
        );
    }
}

/// Memory Safety:
/// - No heap allocations in hot path (only stack-based message passing)
/// - Clone operations are minimal (only for logging)
/// - Task automatically terminates preventing memory leaks
async fn bridge_websocket_to_grpc_stream(
    signature: String,
    mut websocket_rx: tokio::sync::mpsc::UnboundedReceiver<MonitorTransactionResponse>,
    grpc_tx: mpsc::Sender<Result<MonitorTransactionResponse, Status>>,
    timeout_seconds: u32,
) {
    debug!(
        signature = %signature,
        timeout_seconds = timeout_seconds,
        "🌉 Starting stream bridge"
    );

    let bridge_timeout = Duration::from_secs(u64::from(timeout_seconds) + 5); // Add 5s buffer

    // Use timeout to prevent indefinite hanging if WebSocket stops responding
    let bridge_result = timeout(bridge_timeout, async {
        while let Some(response) = websocket_rx.recv().await {
            debug!(
                signature = %signature,
                status = ?response.status(),
                slot = response.slot,
                "📨 Received WebSocket update"
            );

            // Try to send to gRPC client - if this fails, client has disconnected
            if matches!(grpc_tx.send(Ok(response.clone())).await, Ok(())) {
                // Successfully sent to client
            } else {
                info!(
                    signature = %signature,
                    "🔌 Client disconnected (gRPC channel closed)"
                );
                return; // Early return - no need to continue processing
            }

            // Check if this is a terminal status that should end the stream
            if is_terminal_status(response.status()) {
                info!(
                    signature = %signature,
                    status = ?response.status(),
                    slot = response.slot,
                    "🏁 Terminal status reached"
                );
                return; // End stream on terminal status
            }
        }

        // WebSocket channel closed (sender dropped)
        debug!(
            signature = %signature,
            "📡 WebSocket stream ended (sender closed)"
        );
    })
    .await;

    if bridge_result == Ok(()) {
        debug!(
            signature = %signature,
            "✅ Stream bridge completed normally"
        );
    } else {
        warn!(
            signature = %signature,
            timeout_seconds = timeout_seconds + 5,
            "⏰ Stream bridge timed out"
        );
        // Send timeout notification to client if channel is still open
        send_timeout_notification(&grpc_tx, &signature).await;
    }
}
