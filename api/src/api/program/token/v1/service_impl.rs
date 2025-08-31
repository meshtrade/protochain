use std::sync::Arc;
use tonic::{Request, Response, Status};

use protosol_api::protosol::solana::program::token::v1::{
    service_server::Service as TokenProgramService, CreateHoldingAccountRequest,
    CreateHoldingAccountResponse, CreateMintRequest, CreateMintResponse,
    GetCurrentMinRentForHoldingAccountRequest, GetCurrentMinRentForHoldingAccountResponse,
    GetCurrentMinRentForTokenAccountRequest, GetCurrentMinRentForTokenAccountResponse,
    InitialiseHoldingAccountRequest, InitialiseHoldingAccountResponse, InitialiseMintRequest,
    InitialiseMintResponse, MintInfo, MintRequest, MintResponse, ParseMintRequest,
    ParseMintResponse,
};

use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey};
use spl_token_2022::{
    instruction::{initialize_account, initialize_mint2, mint_to_checked},
    state::{Account, Mint},
    ID as TOKEN_2022_PROGRAM_ID,
};
use std::str::FromStr;

use crate::api::common::solana_conversions::sdk_instruction_to_proto;
use crate::api::program::system::v1::service_impl::SystemProgramServiceImpl;
use protosol_api::protosol::solana::program::system::v1::{
    service_server::Service as SystemProgramService, CreateRequest as SystemCreateRequest,
};

/// Token Program service implementation for Token 2022 operations
#[derive(Clone)]
pub struct TokenProgramServiceImpl {
    /// Solana RPC client for blockchain interactions
    rpc_client: Arc<RpcClient>,
}

impl TokenProgramServiceImpl {
    /// Creates a new `TokenProgramServiceImpl` instance with the provided RPC client
    pub const fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self { rpc_client }
    }
}

#[tonic::async_trait]
impl TokenProgramService for TokenProgramServiceImpl {
    /// Creates an `InitialiseMint` instruction for Token 2022 program
    async fn initialise_mint(
        &self,
        request: Request<InitialiseMintRequest>,
    ) -> Result<Response<InitialiseMintResponse>, Status> {
        let req = request.into_inner();

        // Parse public keys
        let mint_pubkey = Pubkey::from_str(&req.mint_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;
        let mint_authority = Pubkey::from_str(&req.mint_authority_pub_key).map_err(|e| {
            Status::invalid_argument(format!("Invalid mint_authority_pub_key: {e}"))
        })?;

        // Parse optional freeze authority
        let freeze_authority = if req.freeze_authority_pub_key.is_empty() {
            None
        } else {
            Some(Pubkey::from_str(&req.freeze_authority_pub_key).map_err(|e| {
                Status::invalid_argument(format!("Invalid freeze_authority_pub_key: {e}"))
            })?)
        };

        // Create the InitialiseMint instruction
        let instruction = initialize_mint2(
            &TOKEN_2022_PROGRAM_ID,
            &mint_pubkey,
            &mint_authority,
            freeze_authority.as_ref(),
            u8::try_from(req.decimals)
                .map_err(|_| Status::invalid_argument("decimals must be between 0 and 255"))?,
        )
        .map_err(|e| {
            Status::invalid_argument(format!("Failed to create InitialiseMint instruction: {e}"))
        })?;

        // Convert to proto and return
        let proto_instruction = sdk_instruction_to_proto(instruction);
        Ok(Response::new(InitialiseMintResponse {
            instruction: Some(proto_instruction),
        }))
    }

    /// Gets current minimum rent for a token account (mint size)
    async fn get_current_min_rent_for_token_account(
        &self,
        _request: Request<GetCurrentMinRentForTokenAccountRequest>,
    ) -> Result<Response<GetCurrentMinRentForTokenAccountResponse>, Status> {
        // Get minimum balance for rent exemption using Mint::LEN
        match self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(Mint::LEN)
        {
            Ok(lamports) => {
                let response = GetCurrentMinRentForTokenAccountResponse { lamports };
                Ok(Response::new(response))
            }
            Err(e) => Err(Status::internal(format!(
                "Failed to get minimum balance for token account: {e}"
            ))),
        }
    }

    /// Parses mint account data into structured format
    async fn parse_mint(
        &self,
        request: Request<ParseMintRequest>,
    ) -> Result<Response<ParseMintResponse>, Status> {
        let req = request.into_inner();

        // Parse the account address
        let account_pubkey = Pubkey::from_str(&req.account_address)
            .map_err(|e| Status::invalid_argument(format!("Invalid account_address: {e}")))?;

        // Get the account data
        let account = self
            .rpc_client
            .get_account_with_commitment(&account_pubkey, CommitmentConfig::confirmed())
            .map_err(|e| Status::internal(format!("Failed to get account: {e}")))?
            .value
            .ok_or_else(|| Status::not_found("Account not found"))?;

        // Verify the account is owned by the Token 2022 program
        if account.owner != TOKEN_2022_PROGRAM_ID {
            return Err(Status::invalid_argument("Account is not owned by Token 2022 program"));
        }

        // Unpack the mint account data
        let mint = Mint::unpack(&account.data)
            .map_err(|e| Status::invalid_argument(format!("Failed to parse mint account: {e}")))?;

        // Convert to proto format
        let mint_info = MintInfo {
            mint_authority_pub_key: mint
                .mint_authority
                .map(|key| key.to_string())
                .unwrap_or_default(),
            freeze_authority_pub_key: mint
                .freeze_authority
                .map(|key| key.to_string())
                .unwrap_or_default(),
            decimals: u32::from(mint.decimals),
            supply: mint.supply.to_string(),
            is_initialized: mint.is_initialized,
        };

        Ok(Response::new(ParseMintResponse {
            mint: Some(mint_info),
        }))
    }

    /// Creates an `InitialiseHoldingAccount` instruction for Token 2022 program
    async fn initialise_holding_account(
        &self,
        request: Request<InitialiseHoldingAccountRequest>,
    ) -> Result<Response<InitialiseHoldingAccountResponse>, Status> {
        let req = request.into_inner();

        // Parse public keys
        let account_pubkey = Pubkey::from_str(&req.account_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid account_pub_key: {e}")))?;
        let mint_pubkey = Pubkey::from_str(&req.mint_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;
        let owner_pubkey = Pubkey::from_str(&req.owner_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid owner_pub_key: {e}")))?;

        // Create the InitializeAccount instruction
        let instruction = initialize_account(
            &TOKEN_2022_PROGRAM_ID,
            &account_pubkey,
            &mint_pubkey,
            &owner_pubkey,
        )
        .map_err(|e| {
            Status::invalid_argument(format!(
                "Failed to create InitialiseHoldingAccount instruction: {e}"
            ))
        })?;

        // Convert to proto and return
        let proto_instruction = sdk_instruction_to_proto(instruction);
        Ok(Response::new(InitialiseHoldingAccountResponse {
            instruction: Some(proto_instruction),
        }))
    }

    /// Gets current minimum rent for a token holding account
    async fn get_current_min_rent_for_holding_account(
        &self,
        _request: Request<GetCurrentMinRentForHoldingAccountRequest>,
    ) -> Result<Response<GetCurrentMinRentForHoldingAccountResponse>, Status> {
        // Get minimum balance for rent exemption using Account::LEN
        match self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(Account::LEN)
        {
            Ok(lamports) => {
                let response = GetCurrentMinRentForHoldingAccountResponse { lamports };
                Ok(Response::new(response))
            }
            Err(e) => Err(Status::internal(format!(
                "Failed to get minimum balance for holding account: {e}"
            ))),
        }
    }

    /// Creates both system account creation and mint initialization instructions
    async fn create_mint(
        &self,
        request: Request<CreateMintRequest>,
    ) -> Result<Response<CreateMintResponse>, Status> {
        let req = request.into_inner();

        // Validation
        if req.payer.is_empty() {
            return Err(Status::invalid_argument("Payer address is required"));
        }
        if req.new_account.is_empty() {
            return Err(Status::invalid_argument("New account address is required"));
        }
        if req.mint_pub_key != req.new_account {
            return Err(Status::invalid_argument("mint_pub_key must match new_account"));
        }

        // Step 1: Get current rent for mint account
        let rent_response = self
            .get_current_min_rent_for_token_account(Request::new(
                GetCurrentMinRentForTokenAccountRequest {},
            ))
            .await?
            .into_inner();

        // Step 2: Create system account creation instruction
        let system_service = SystemProgramServiceImpl::new();
        let create_instruction = system_service
            .create(Request::new(SystemCreateRequest {
                payer: req.payer.clone(),
                new_account: req.new_account.clone(),
                owner: TOKEN_2022_PROGRAM_ID.to_string(),
                lamports: rent_response.lamports,
                space: Mint::LEN as u64,
            }))
            .await?
            .into_inner();

        // Step 3: Create mint initialization instruction
        let init_response = self
            .initialise_mint(Request::new(InitialiseMintRequest {
                mint_pub_key: req.mint_pub_key,
                mint_authority_pub_key: req.mint_authority_pub_key,
                freeze_authority_pub_key: req.freeze_authority_pub_key,
                decimals: req.decimals,
            }))
            .await?
            .into_inner();

        // Step 4: Compose response with both instructions
        let mut instructions = Vec::new();
        instructions.push(create_instruction);
        if let Some(init_instruction) = init_response.instruction {
            instructions.push(init_instruction);
        }

        Ok(Response::new(CreateMintResponse { instructions }))
    }

    /// Creates both system account creation and holding account initialization instructions
    async fn create_holding_account(
        &self,
        request: Request<CreateHoldingAccountRequest>,
    ) -> Result<Response<CreateHoldingAccountResponse>, Status> {
        let req = request.into_inner();

        // Validation
        if req.payer.is_empty() {
            return Err(Status::invalid_argument("Payer address is required"));
        }
        if req.new_account.is_empty() {
            return Err(Status::invalid_argument("New account address is required"));
        }
        if req.holding_account_pub_key != req.new_account {
            return Err(Status::invalid_argument("holding_account_pub_key must match new_account"));
        }

        // Step 1: Get current rent for holding account
        let rent_response = self
            .get_current_min_rent_for_holding_account(Request::new(
                GetCurrentMinRentForHoldingAccountRequest {},
            ))
            .await?
            .into_inner();

        // Step 2: Create system account creation instruction
        let system_service = SystemProgramServiceImpl::new();
        let create_instruction = system_service
            .create(Request::new(SystemCreateRequest {
                payer: req.payer.clone(),
                new_account: req.new_account.clone(),
                owner: TOKEN_2022_PROGRAM_ID.to_string(),
                lamports: rent_response.lamports,
                space: Account::LEN as u64,
            }))
            .await?
            .into_inner();

        // Step 3: Create holding account initialization instruction
        let init_response = self
            .initialise_holding_account(Request::new(InitialiseHoldingAccountRequest {
                account_pub_key: req.holding_account_pub_key,
                mint_pub_key: req.mint_pub_key,
                owner_pub_key: req.owner_pub_key,
            }))
            .await?
            .into_inner();

        // Step 4: Compose response with both instructions
        let mut instructions = Vec::new();
        instructions.push(create_instruction);
        if let Some(init_instruction) = init_response.instruction {
            instructions.push(init_instruction);
        }

        Ok(Response::new(CreateHoldingAccountResponse { instructions }))
    }

    /// Creates a `MintToChecked` instruction for Token 2022 program
    async fn mint(&self, request: Request<MintRequest>) -> Result<Response<MintResponse>, Status> {
        let req = request.into_inner();

        // Parse public keys
        let mint_pubkey = Pubkey::from_str(&req.mint_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;
        let destination_account_pubkey = Pubkey::from_str(&req.destination_account_pub_key)
            .map_err(|e| {
                Status::invalid_argument(format!("Invalid destination_account_pub_key: {e}"))
            })?;
        let mint_authority_pubkey = Pubkey::from_str(&req.mint_authority_pub_key).map_err(|e| {
            Status::invalid_argument(format!("Invalid mint_authority_pub_key: {e}"))
        })?;

        // Parse amount from string to handle large numbers
        let amount = req
            .amount
            .parse::<u64>()
            .map_err(|e| Status::invalid_argument(format!("Invalid amount: {e}")))?;

        // Validate decimals
        let decimals = u8::try_from(req.decimals)
            .map_err(|_| Status::invalid_argument("decimals must be between 0 and 255"))?;

        // Create the MintToChecked instruction (no additional signers for single authority)
        let instruction = mint_to_checked(
            &TOKEN_2022_PROGRAM_ID,
            &mint_pubkey,
            &destination_account_pubkey,
            &mint_authority_pubkey,
            &[], // Empty signer array for single authority
            amount,
            decimals,
        )
        .map_err(|e| {
            Status::invalid_argument(format!("Failed to create MintToChecked instruction: {e}"))
        })?;

        // Convert to proto and return
        let proto_instruction = sdk_instruction_to_proto(instruction);
        Ok(Response::new(MintResponse {
            instruction: Some(proto_instruction),
        }))
    }
}
