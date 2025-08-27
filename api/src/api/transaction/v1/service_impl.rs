use std::sync::Arc;
use std::str::FromStr;
use tonic::{Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use solana_sdk::{
    message::Message, 
    hash::Hash, 
    pubkey::Pubkey,
    instruction::Instruction,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction as SolanaTransaction,
};
use solana_client::rpc_client::RpcClient;
use solana_client::client_error::ClientError;
use solana_transaction_status::{UiTransactionEncoding, EncodedTransaction};
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use crate::websocket::WebSocketManager;

use crate::api::program::system::v1::conversion::proto_instruction_to_sdk;
use crate::api::transaction::v1::validation::{
    validate_state_transition, 
    validate_transaction_state_consistency,
    validate_operation_allowed_for_state,
};
use protosol_api::protosol::solana::transaction::v1::{
    service_server::Service as TransactionService,
    *,
};
use protosol_api::protosol::solana::r#type::v1::CommitmentLevel;

/// Composable Transaction Service Implementation
/// Provides methods to compile, estimate, simulate, sign, and submit composable transactions
#[derive(Clone)]
pub struct TransactionServiceImpl {
    rpc_client: Arc<RpcClient>,
    websocket_manager: Arc<WebSocketManager>,
}

impl TransactionServiceImpl {
    /// Creates a new TransactionServiceImpl with the provided RPC client and WebSocket manager
    pub fn new(rpc_client: Arc<RpcClient>, websocket_manager: Arc<WebSocketManager>) -> Self {
        Self { 
            rpc_client,
            websocket_manager,
        }
    }
}

/// Classifies submission errors into appropriate SubmissionResult categories
fn classify_submission_error(error: &ClientError) -> SubmissionResult {
    let error_str = error.to_string().to_lowercase();
    
    // Check for specific error patterns in the error message
    if error_str.contains("insufficient") && error_str.contains("fund") {
        SubmissionResult::FailedInsufficientFunds
    } else if error_str.contains("invalid") && error_str.contains("signature") {
        SubmissionResult::FailedInvalidSignature
    } else if error_str.contains("network") || error_str.contains("connection") || error_str.contains("timeout") {
        SubmissionResult::FailedNetworkError
    } else if error_str.contains("validation") || error_str.contains("invalid") {
        SubmissionResult::FailedValidation
    } else {
        // Default to network error for unknown errors
        SubmissionResult::FailedNetworkError
    }
}

/// Helper function to convert proto CommitmentLevel to Solana CommitmentConfig
/// Provides sensible defaults when commitment level is not specified
fn commitment_level_to_config(commitment_level: Option<i32>) -> CommitmentConfig {
    match commitment_level {
        Some(level) => {
            match CommitmentLevel::try_from(level) {
                Ok(CommitmentLevel::Processed) => CommitmentConfig::processed(),
                Ok(CommitmentLevel::Confirmed) => CommitmentConfig::confirmed(),
                Ok(CommitmentLevel::Finalized) => CommitmentConfig::finalized(),
                Ok(CommitmentLevel::Unspecified) | Err(_) => {
                    // Default to confirmed for reliability - matches account service default
                    CommitmentConfig::confirmed()
                }
            }
        }
        None => {
            // Default to confirmed when not specified - maintains consistency with account service
            CommitmentConfig::confirmed()
        }
    }
}

#[tonic::async_trait]
impl TransactionService for TransactionServiceImpl {
    type MonitorTransactionStream = ReceiverStream<Result<MonitorTransactionResponse, Status>>;
    /// Compiles a draft transaction with instructions into an executable transaction
    async fn compile_transaction(
        &self,
        request: Request<CompileTransactionRequest>,
    ) -> Result<Response<CompileTransactionResponse>, Status> {
        let req = request.into_inner();
        let mut transaction = req.transaction
            .ok_or_else(|| Status::invalid_argument("Transaction is required"))?;
        
        // Validate current state allows compilation
        let current_state = transaction.state();
        validate_operation_allowed_for_state(current_state, "compile")
            .map_err(|e| Status::failed_precondition(e))?;
        
        // Validate transaction consistency in current state
        validate_transaction_state_consistency(&transaction)
            .map_err(|e| Status::invalid_argument(format!("Invalid transaction state: {}", e)))?;
        
        // Ensure we have instructions
        if transaction.instructions.is_empty() {
            return Err(Status::invalid_argument(
                "Transaction must have at least one instruction"
            ));
        }
        
        // Validate fee_payer is provided
        if req.fee_payer.is_empty() {
            return Err(Status::invalid_argument("fee_payer is required"));
        }
        
        // Convert proto instructions to SDK instructions
        let sdk_instructions: Result<Vec<Instruction>, String> = transaction.instructions
            .iter()
            .map(|proto_ix| proto_instruction_to_sdk(proto_ix.clone()))
            .collect();
        
        let sdk_instructions = sdk_instructions
            .map_err(|e| Status::invalid_argument(format!("Invalid instruction: {}", e)))?;
        
        // Parse fee payer pubkey
        let fee_payer = Pubkey::from_str(&req.fee_payer)
            .map_err(|e| Status::invalid_argument(format!("Invalid fee_payer: {}", e)))?;
        
        // Get recent blockhash (from request or fetch from network)
        let recent_blockhash = if req.recent_blockhash.is_empty() {
            // Fetch latest blockhash from network
            self.rpc_client.get_latest_blockhash()
                .map_err(|e| Status::internal(format!("Failed to get latest blockhash: {}", e)))?
        } else {
            // Use provided blockhash
            Hash::from_str(&req.recent_blockhash)
                .map_err(|e| Status::invalid_argument(format!("Invalid blockhash format: {}", e)))?
        };
        
        // CRITICAL: Use Solana SDK to compile the transaction
        // This handles all the complexity of account deduplication, signing requirements, etc.
        let message = Message::new_with_blockhash(
            &sdk_instructions,
            Some(&fee_payer),
            &recent_blockhash,
        );
        
        // Serialize the compiled message for transport
        let transaction_bytes = bincode::serialize(&message)
            .map_err(|e| Status::internal(format!("Transaction serialization failed: {}", e)))?;
        
        // Encode as base58 for proto transport
        let transaction_data = bs58::encode(&transaction_bytes).into_string();
        
        // Validate state transition DRAFT -> COMPILED
        validate_state_transition(current_state, TransactionState::Compiled)
            .map_err(|e| Status::internal(format!("State transition validation failed: {}", e)))?;
        
        // Update transaction with compiled data and metadata
        transaction.data = transaction_data;
        transaction.state = TransactionState::Compiled.into();
        transaction.fee_payer = req.fee_payer;
        transaction.recent_blockhash = recent_blockhash.to_string();
        
        // Validate the updated transaction consistency
        validate_transaction_state_consistency(&transaction)
            .map_err(|e| Status::internal(format!("Compiled transaction validation failed: {}", e)))?;
        
        Ok(Response::new(CompileTransactionResponse {
            transaction: Some(transaction),
        }))
    }
    
    /// Estimates compute units and fees for a compiled transaction
    async fn estimate_transaction(
        &self,
        request: Request<EstimateTransactionRequest>,
    ) -> Result<Response<EstimateTransactionResponse>, Status> {
        let req = request.into_inner();
        let transaction = req.transaction
            .ok_or_else(|| Status::invalid_argument("Transaction is required"))?;
        
        // Validate current state allows estimation
        let current_state = transaction.state();
        validate_operation_allowed_for_state(current_state, "estimate")
            .map_err(|e| Status::failed_precondition(e))?;
        
        // Validate transaction state consistency
        validate_transaction_state_consistency(&transaction)
            .map_err(|e| Status::invalid_argument(format!("Transaction validation failed: {}", e)))?;
        
        // Ensure transaction has compiled data
        if transaction.data.is_empty() {
            return Err(Status::invalid_argument("Transaction must be compiled before estimation"));
        }
        
        // Deserialize the compiled transaction data 
        let transaction_data = bs58::decode(&transaction.data)
            .into_vec()
            .map_err(|e| Status::invalid_argument(format!("Failed to decode transaction data: {}", e)))?;
        
        let message: Message = bincode::deserialize(&transaction_data)
            .map_err(|e| Status::invalid_argument(format!("Failed to deserialize transaction: {}", e)))?;
        
        // Create an unsigned transaction for simulation  
        let solana_transaction = SolanaTransaction::new_unsigned(message);
        
        // Get commitment level for estimation simulation
        let commitment = commitment_level_to_config(req.commitment_level);
        
        // Use simulation to get accurate compute unit estimation with configurable commitment level
        let (compute_units, _logs) = match self.rpc_client.simulate_transaction_with_config(&solana_transaction, solana_client::rpc_config::RpcSimulateTransactionConfig {
            sig_verify: false,
            replace_recent_blockhash: false,
            commitment: Some(commitment),
            encoding: None,
            accounts: None,
            min_context_slot: None,
            inner_instructions: false,
        }) {
            Ok(simulation_result) => {
                // Handle both None and 0 cases by providing reasonable fallback
                let compute_units = match simulation_result.value.units_consumed {
                    Some(units) if units > 0 => units,
                    _ => {
                        // Fallback estimation based on instruction count
                        let instruction_count = transaction.instructions.len() as u64;
                        (instruction_count * 50_000).max(200_000).min(1_400_000)
                    }
                };
                let logs = simulation_result.value.logs.unwrap_or_default();
                (compute_units, logs)
            }
            Err(_) => {
                // Fallback to basic estimation if simulation fails
                let instruction_count = transaction.instructions.len() as u64;
                let estimated_compute_units = (instruction_count * 50_000).max(200_000).min(1_400_000);
                (estimated_compute_units, vec![])
            }
        };
        
        // Calculate fee estimation
        let base_fee_lamports = 5_000; // Base transaction fee
        let compute_unit_price = transaction.config
            .as_ref()
            .map(|config| config.compute_unit_price)
            .unwrap_or(0);
        
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
    
    /// Simulates a compiled transaction without submitting it
    async fn simulate_transaction(
        &self,
        request: Request<SimulateTransactionRequest>,
    ) -> Result<Response<SimulateTransactionResponse>, Status> {
        let req = request.into_inner();
        let transaction = req.transaction
            .ok_or_else(|| Status::invalid_argument("Transaction is required"))?;
        
        // Validate current state allows simulation
        let current_state = transaction.state();
        validate_operation_allowed_for_state(current_state, "simulate")
            .map_err(|e| Status::failed_precondition(e))?;
        
        // Validate transaction state consistency
        validate_transaction_state_consistency(&transaction)
            .map_err(|e| Status::invalid_argument(format!("Transaction validation failed: {}", e)))?;
        
        // Ensure transaction has compiled data
        if transaction.data.is_empty() {
            return Err(Status::invalid_argument("Transaction must be compiled before simulation"));
        }
        
        // Deserialize the compiled transaction data 
        let transaction_data = bs58::decode(&transaction.data)
            .into_vec()
            .map_err(|e| Status::invalid_argument(format!("Failed to decode transaction data: {}", e)))?;
        
        let message: Message = bincode::deserialize(&transaction_data)
            .map_err(|e| Status::invalid_argument(format!("Failed to deserialize transaction: {}", e)))?;
        
        // Create an unsigned transaction for simulation
        let solana_transaction = SolanaTransaction::new_unsigned(message);
        
        // Get commitment level for simulation
        let commitment = commitment_level_to_config(req.commitment_level);
        
        // Simulate the transaction using RPC with configurable commitment level
        match self.rpc_client.simulate_transaction_with_config(&solana_transaction, solana_client::rpc_config::RpcSimulateTransactionConfig {
            sig_verify: false,
            replace_recent_blockhash: false,
            commitment: Some(commitment),
            encoding: None,
            accounts: None,
            min_context_slot: None,
            inner_instructions: false,
        }) {
            Ok(simulation_result) => {
                let success = simulation_result.value.err.is_none();
                let error = simulation_result.value.err
                    .map(|err| format!("{:?}", err))
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
                    error: format!("Simulation failed: {}", e),
                    logs: vec![],
                }))
            }
        }
    }
    
    /// Signs a compiled transaction with provided signing methods
    async fn sign_transaction(
        &self,
        request: Request<SignTransactionRequest>,
    ) -> Result<Response<SignTransactionResponse>, Status> {
        let req = request.into_inner();
        let mut transaction = req.transaction
            .ok_or_else(|| Status::invalid_argument("Transaction is required"))?;
        
        // Validate current state allows signing
        let current_state = transaction.state();
        validate_operation_allowed_for_state(current_state, "sign")
            .map_err(|e| Status::failed_precondition(e))?;
        
        // Validate transaction state consistency
        validate_transaction_state_consistency(&transaction)
            .map_err(|e| Status::invalid_argument(format!("Transaction validation failed: {}", e)))?;
        
        // Ensure transaction has compiled data
        if transaction.data.is_empty() {
            return Err(Status::invalid_argument("Transaction must be compiled before signing"));
        }
        
        // Deserialize the compiled transaction data
        let transaction_data = bs58::decode(&transaction.data)
            .into_vec()
            .map_err(|e| Status::invalid_argument(format!("Failed to decode transaction data: {}", e)))?;
        
        let message: Message = bincode::deserialize(&transaction_data)
            .map_err(|e| Status::invalid_argument(format!("Failed to deserialize transaction: {}", e)))?;
        
        // Process signing method and apply signatures
        let keypairs = match req.signing_method {
            Some(signing_method) => match signing_method {
                sign_transaction_request::SigningMethod::PrivateKeys(private_keys_method) => {
                    // Parse private keys into keypairs
                    let mut keypairs = Vec::new();
                    for private_key_str in &private_keys_method.private_keys {
                        let private_key_bytes = bs58::decode(private_key_str)
                            .into_vec()
                            .map_err(|e| Status::invalid_argument(format!("Invalid private key format: {}", e)))?;
                        
                        if private_key_bytes.len() != 64 {
                            return Err(Status::invalid_argument("Private key must be 64 bytes"));
                        }
                        
                        let keypair = Keypair::from_bytes(&private_key_bytes)
                            .map_err(|e| Status::invalid_argument(format!("Invalid private key: {}", e)))?;
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
            if let Some(account_index) = solana_transaction.message.account_keys.iter()
                .position(|key| key == &keypair.pubkey()) {
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
        transaction.signatures = solana_transaction.signatures.iter()
            .filter(|sig| **sig != Signature::default())
            .map(|sig| sig.to_string())
            .collect();
        
        // Determine new state based on signature completeness
        let required_signatures = solana_transaction.message.header.num_required_signatures as usize;
        let provided_signatures = transaction.signatures.len();
        
        let new_state = if provided_signatures >= required_signatures {
            TransactionState::FullySigned
        } else {
            TransactionState::PartiallySigned
        };
        
        // Validate state transition
        validate_state_transition(current_state, new_state)
            .map_err(|e| Status::internal(format!("State transition validation failed: {}", e)))?;
        
        transaction.state = new_state.into();
        
        // Update the transaction data with signed transaction
        let signed_transaction_bytes = bincode::serialize(&solana_transaction)
            .map_err(|e| Status::internal(format!("Failed to serialize signed transaction: {}", e)))?;
        transaction.data = bs58::encode(&signed_transaction_bytes).into_string();
        
        Ok(Response::new(SignTransactionResponse {
            transaction: Some(transaction),
        }))
    }
    
    /// Submits a fully signed transaction to the network
    async fn submit_transaction(
        &self,
        request: Request<SubmitTransactionRequest>,
    ) -> Result<Response<SubmitTransactionResponse>, Status> {
        let req = request.into_inner();
        let transaction = req.transaction
            .ok_or_else(|| Status::invalid_argument("Transaction is required"))?;
        
        // Validate current state allows submission
        let current_state = transaction.state();
        validate_operation_allowed_for_state(current_state, "submit")
            .map_err(|e| Status::failed_precondition(e))?;
        
        // Validate transaction state consistency
        validate_transaction_state_consistency(&transaction)
            .map_err(|e| Status::invalid_argument(format!("Transaction validation failed: {}", e)))?;
        
        // Ensure transaction is fully signed
        if current_state != TransactionState::FullySigned {
            return Err(Status::failed_precondition("Transaction must be fully signed before submission"));
        }
        
        // Deserialize the signed transaction data
        let transaction_data = bs58::decode(&transaction.data)
            .into_vec()
            .map_err(|e| Status::invalid_argument(format!("Failed to decode transaction data: {}", e)))?;
        
        let solana_transaction: SolanaTransaction = bincode::deserialize(&transaction_data)
            .map_err(|e| Status::invalid_argument(format!("Failed to deserialize transaction: {}", e)))?;
        
        // Verify transaction is properly signed
        if solana_transaction.signatures.iter().any(|sig| *sig == Signature::default()) {
            return Err(Status::failed_precondition("Transaction contains unsigned accounts"));
        }
        
        // Submit the transaction to the Solana network with explicit commitment level
        println!("🚀 Submitting transaction to Solana network...");
        
        // CRITICAL: Use send_and_confirm_transaction_with_spinner_and_commitment instead of send_transaction
        // 
        // Reasoning for this design choice:
        // 1. CONSISTENCY: The GetAccount API uses CommitmentConfig::confirmed() to handle timing issues
        //    where newly funded accounts aren't immediately visible. Transaction submission needs 
        //    the same commitment level for consistency.
        //
        // 2. SIMULATION RELIABILITY: send_transaction() uses default commitment (often 'processed')
        //    which can cause "Attempt to debit an account but found no record of a prior credit" 
        //    errors when the transaction simulation can't see recently funded accounts.
        //
        // 3. ATOMIC TRANSACTION SUPPORT: For multi-instruction atomic transactions that create 
        //    accounts and immediately transfer from them, we need confirmed commitment to ensure
        //    the validator sees all prior transactions that funded the fee payer.
        //
        // 4. LOCAL VALIDATOR COMPATIBILITY: Local test validators can have different timing 
        //    characteristics than mainnet. Confirmed commitment provides reliable behavior
        //    across different network conditions.
        //
        // Alternative approaches considered:
        // - send_transaction(): Fast but unreliable due to commitment timing
        // - send_and_confirm_transaction(): Better but uses default commitment 
        // - send_and_confirm_transaction_with_spinner_and_commitment(): Chosen for explicit control
        let commitment = commitment_level_to_config(req.commitment_level);
        println!("Submitting transaction with commitment level: {:?}", commitment);
        let (signature_result, submission_result, error_message) = match self.rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(
            &solana_transaction, 
            commitment
        ) {
            Ok(signature) => {
                (signature.to_string(), SubmissionResult::Submitted, None)
            }
            Err(e) => {
                let classification = classify_submission_error(&e);
                let error_msg = format!("Transaction submission failed: {}", e);
                (String::new(), classification, Some(error_msg))
            }
        };
        
        Ok(Response::new(SubmitTransactionResponse {
            signature: signature_result,
            submission_result: submission_result.into(),
            error_message,
        }))
    }
    
    /// Retrieves a transaction by its signature
    async fn get_transaction(
        &self,
        request: Request<GetTransactionRequest>,
    ) -> Result<Response<GetTransactionResponse>, Status> {
        let req = request.into_inner();
        
        if req.signature.is_empty() {
            return Err(Status::invalid_argument("Transaction signature is required"));
        }
        
        // Parse the signature
        let signature = Signature::from_str(&req.signature)
            .map_err(|e| Status::invalid_argument(format!("Invalid signature format: {}", e)))?;
        
        // Get commitment level for transaction retrieval
        let commitment = commitment_level_to_config(req.commitment_level);
        
        // Query the transaction from the network with configurable commitment level
        match self.rpc_client.get_transaction_with_config(&signature, RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Base64),
            commitment: Some(commitment),
            max_supported_transaction_version: Some(0),
        }) {
            Ok(confirmed_transaction) => {
                // Extract transaction data
                let transaction_data = match confirmed_transaction.transaction.transaction {
                    EncodedTransaction::Binary(data, _) => {
                        bs58::decode(&data).into_vec()
                            .map_err(|e| Status::internal(format!("Failed to decode transaction data: {}", e)))?
                    }
                    _ => {
                        return Err(Status::internal("Unsupported transaction encoding"));
                    }
                };
                
                // Deserialize the transaction
                let solana_transaction: SolanaTransaction = bincode::deserialize(&transaction_data)
                    .map_err(|e| Status::internal(format!("Failed to deserialize transaction: {}", e)))?;
                
                // Convert to our proto format
                let proto_transaction = Transaction {
                    instructions: vec![], // Instructions are not preserved in network storage
                    state: TransactionState::FullySigned.into(), // Network transactions are fully signed
                    config: None, // Config is not preserved in network storage  
                    data: bs58::encode(&transaction_data).into_string(),
                    fee_payer: solana_transaction.message.account_keys.first()
                        .map(|key| key.to_string())
                        .unwrap_or_default(),
                    recent_blockhash: solana_transaction.message.recent_blockhash.to_string(),
                    signatures: solana_transaction.signatures.iter()
                        .map(|sig| sig.to_string())
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
                Err(Status::not_found(format!("Transaction not found: {}", e)))
            }
        }
    }
    
    /// Monitors a transaction for status changes until target commitment level
    async fn monitor_transaction(
        &self,
        request: Request<MonitorTransactionRequest>,
    ) -> Result<Response<Self::MonitorTransactionStream>, Status> {
        let req = request.into_inner();
        
        // Validate signature format
        if req.signature.is_empty() {
            return Err(Status::invalid_argument("Transaction signature is required"));
        }
        
        // Parse signature to validate format
        req.signature.parse::<solana_sdk::signature::Signature>()
            .map_err(|_| Status::invalid_argument("Invalid signature format"))?;
        
        // Validate commitment level
        let commitment_level = CommitmentLevel::try_from(req.commitment_level)
            .map_err(|_| Status::invalid_argument("Invalid commitment level"))?;
        
        // Validate timeout (if provided)
        let timeout_seconds = req.timeout_seconds.unwrap_or(60);
        if timeout_seconds < 5 || timeout_seconds > 300 {
            return Err(Status::invalid_argument("Timeout must be between 5 and 300 seconds"));
        }
        
        println!("🔍 Starting transaction monitoring for signature: {}", req.signature);
        
        // Create response stream channel
        let (tx, rx) = mpsc::channel(100);
        
        // Subscribe to signature updates via WebSocket manager
        let websocket_rx = match self.websocket_manager.subscribe_to_signature(
            req.signature.clone(),
            commitment_level,
            req.include_logs,
            Some(timeout_seconds),
        ).await {
            Ok(rx) => rx,
            Err(e) => {
                return Err(e);
            }
        };
        
        // Spawn task to bridge WebSocket updates to gRPC stream
        let signature_for_task = req.signature.clone();
        tokio::spawn(async move {
            bridge_websocket_to_grpc_stream(signature_for_task, websocket_rx, tx).await;
        });
        
        println!("✅ Transaction monitoring stream established for: {}", req.signature);
        
        Ok(Response::new(ReceiverStream::new(rx)))
    }
    
}

/// Bridges WebSocket subscription updates to gRPC streaming response
async fn bridge_websocket_to_grpc_stream(
        signature: String,
        mut websocket_rx: tokio::sync::mpsc::UnboundedReceiver<MonitorTransactionResponse>,
        grpc_tx: mpsc::Sender<Result<MonitorTransactionResponse, Status>>,
    ) {
        println!("🌉 Starting stream bridge for signature: {}", signature);
        
        while let Some(response) = websocket_rx.recv().await {
            println!("📨 Received WebSocket update for {}: status={:?}", signature, response.status());
            
            // Check if client is still connected
            if grpc_tx.send(Ok(response.clone())).await.is_err() {
                println!("🔌 Client disconnected for signature: {}", signature);
                break;
            }
            
            // Check if this is a terminal status
            let is_terminal = matches!(
                response.status(),
                TransactionStatus::Confirmed |
                TransactionStatus::Finalized |
                TransactionStatus::Failed |
                TransactionStatus::Dropped |
                TransactionStatus::Timeout
            );
            
            if is_terminal {
                println!("🏁 Terminal status reached for signature: {} - status={:?}", signature, response.status());
                break;
            }
        }
        
        println!("🌉 Stream bridge completed for signature: {}", signature);
    }