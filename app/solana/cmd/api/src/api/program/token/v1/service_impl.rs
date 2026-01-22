use crate::api::program::token::v1::token_program::get_token_program_id;
use crate::api::{
    common::solana_conversions::sdk_instruction_to_proto,
    program::token::v1::token_program::sdk_token_program_to_proto,
};
use crate::api::{
    common::solana_conversions::solana_instruction_to_proto,
    program::system::v1::service_impl::SystemProgramServiceImpl,
};
use protochain_api::protochain::solana::program::system::v1::{
    service_server::Service as SystemProgramService, CreateRequest as SystemCreateRequest,
};
use protochain_api::protochain::solana::program::token::v1::{
    service_server::Service as TokenProgramService, CreateHoldingAccountRequest,
    CreateHoldingAccountResponse, CreateMintRequest, CreateMintResponse,
    GetCurrentMinRentForHoldingAccountRequest, GetCurrentMinRentForHoldingAccountResponse,
    GetCurrentMinRentForTokenAccountRequest, GetCurrentMinRentForTokenAccountResponse,
    InitialiseMintRequest, InitialiseMintResponse, MintInfo, MintRequest, MintResponse,
    ParseMintRequest, ParseMintResponse,
};
use protochain_api::protochain::solana::r#type::v1::TokenProgram;

use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use solana_address::Address;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey};
use solana_system_interface::instruction::create_account;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::{
    instruction::initialize_mint as initialize_mint_legacy, state::Mint, ID as LEGACY_PROGRAM_ID,
};
use spl_token_2022::{
    extension::ExtensionType,
    instruction::{initialize_mint2, mint_to_checked},
    state::{Account, Mint as LegacyMint},
    ID as TOKEN_2022_PROGRAM_ID,
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

#[allow(clippy::result_large_err)]
fn holding_account_space(require_memo: bool) -> Result<usize, Status> {
    if !require_memo {
        return Ok(Account::LEN);
    }

    let len = ExtensionType::try_calculate_account_len::<Account>(&[ExtensionType::MemoTransfer])
        .map_err(|e| {
        Status::internal(format!("failed to calculate memo-transfer account length: {e}"))
    })?;

    Ok(len)
}

#[allow(clippy::result_large_err)]
fn memo_rent_lamports(rpc: &RpcClient, require_memo: bool) -> Result<u64, Status> {
    let space = holding_account_space(require_memo)?;
    let space_usize = usize::try_from(space)
        .map_err(|_| Status::internal("memo-transfer account length overflow"))?;

    rpc.get_minimum_balance_for_rent_exemption(space_usize)
        .map_err(|e| Status::internal(format!("failed to fetch memo-aware rent: {e}")))
}

#[tonic::async_trait]
impl TokenProgramService for TokenProgramServiceImpl {
    /// Creates an `InitialiseMint` instruction for SPL or SPL 2022 token
    /// Creates an `InitialiseMint` instruction for SPL or SPL 2022 token
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

        // Determine which token program to use
        let token_program_enum = TokenProgram::try_from(req.token_program)
            .map_err(|_| Status::invalid_argument("Invalid token program value"))?;

        let decimals = u8::try_from(req.decimals)
            .map_err(|_| Status::invalid_argument("decimals must be between 0 and 255"))?;

        let instruction = match token_program_enum {
            TokenProgram::Legacy => initialize_mint_legacy(
                &LEGACY_PROGRAM_ID,
                &mint_pubkey,
                &mint_authority,
                freeze_authority.as_ref(),
                decimals,
            )
            .map_err(|e| {
                Status::internal(format!(
                    "could not create initialise mint legacy token instruction: {e}"
                ))
            }),
            TokenProgram::TokenProgram2022 => initialize_mint2(
                &TOKEN_2022_PROGRAM_ID,
                &mint_pubkey,
                &mint_authority,
                freeze_authority.as_ref(),
                decimals,
            )
            .map_err(|e| {
                Status::internal(format!(
                    "could not create initialise mint token 2022 instruction: {e}"
                ))
            }),
            TokenProgram::Unspecified => {
                Err(Status::invalid_argument("token_program must be specified"))
            }
        }?;

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

        // Determine token program from account owner
        let token_program = sdk_token_program_to_proto(&account.owner);

        let mint_info = match token_program {
            TokenProgram::Legacy => {
                // Unpack the mint account data
                let mint = LegacyMint::unpack(&account.data).map_err(|e| {
                    Status::invalid_argument(format!("Failed to parse mint account: {e}"))
                })?;

                // Convert to proto format
                Ok(MintInfo {
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
                })
            }
            TokenProgram::TokenProgram2022 => {
                // Unpack the mint account data
                let mint = Mint::unpack(&account.data).map_err(|e| {
                    Status::invalid_argument(format!("Failed to parse mint account: {e}"))
                })?;

                // Convert to proto format
                Ok(MintInfo {
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
                })
            }
            _ => Err("unknown token program"),
        }
        .map_err(|e| Status::invalid_argument(format!("Could not get mint info: {e}")))?;

        Ok(Response::new(ParseMintResponse {
            mint: Some(mint_info),
        }))
    }

    /// Gets current minimum rent for a token holding account
    async fn get_current_min_rent_for_holding_account(
        &self,
        request: Request<GetCurrentMinRentForHoldingAccountRequest>,
    ) -> Result<Response<GetCurrentMinRentForHoldingAccountResponse>, Status> {
        let req = request.into_inner();
        let require_memo = req
            .memo_transfer_config
            .as_ref()
            .is_some_and(|cfg| cfg.require_incoming_memo);

        let lamports = memo_rent_lamports(&self.rpc_client, require_memo)?;
        let response = GetCurrentMinRentForHoldingAccountResponse { lamports };
        Ok(Response::new(response))
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

        // Step 1: Get current rent for mint account
        let rent_response = self
            .get_current_min_rent_for_token_account(Request::new(
                GetCurrentMinRentForTokenAccountRequest {},
            ))
            .await?
            .into_inner();

        // Step 2: Create system account creation instruction
        let system_service = SystemProgramServiceImpl::new();

        let token_program_enum = TokenProgram::try_from(req.token_program)
            .map_err(|_| Status::invalid_argument("Invalid token program value"))?;

        // Get the program ID pubkey and convert to string for the system program
        let owner_pubkey = get_token_program_id(token_program_enum);

        let create_instruction = system_service
            .create(Request::new(SystemCreateRequest {
                payer: req.payer.clone(),
                new_account: req.mint_pub_key.clone(),
                owner: owner_pubkey.to_string(),
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
                token_program: req.token_program,
            }))
            .await?
            .into_inner();

        // Step 4: Compose response with both instructions
        let mut instructions = Vec::new();
        if let Some(instr) = create_instruction.instruction {
            instructions.push(instr);
        }
        if let Some(init_instruction) = init_response.instruction {
            instructions.push(init_instruction);
        }

        Ok(Response::new(CreateMintResponse { instructions }))
    }

    /// Creates holding account instructions with optional memo transfer support
    ///
    /// For Legacy token program: Uses simple ATA creation
    /// For Token 2022 with memo: Creates account with extended space, initializes as token account,
    /// then initializes the memo transfer extension
    async fn create_holding_account(
        &self,
        request: Request<CreateHoldingAccountRequest>,
    ) -> Result<Response<CreateHoldingAccountResponse>, Status> {
        let req = request.into_inner();

        // Validation
        if req.payer.is_empty() {
            return Err(Status::invalid_argument("Payer address is required"));
        }
        if req.owner_pub_key.is_empty() {
            return Err(Status::invalid_argument("Owner account address is required"));
        }
        // determine which token program to use (require token program be passed, so we can create owner and holding account within same transaction)
        let token_program_enum = TokenProgram::try_from(req.token_program)
            .map_err(|_| Status::invalid_argument("Invalid token program value"))?;
        if req.memo_transfer_config.is_some() && token_program_enum == TokenProgram::Legacy {
            return Err(Status::invalid_argument(
                "Memo transfer config can only be enabled for Token2022 program",
            ));
        }

        // parse public keys
        let payer_pubkey = Pubkey::from_str(&req.payer)
            .map_err(|e| Status::invalid_argument(format!("Invalid payer_pub_key: {e}")))?;
        let owner_pub_key = Pubkey::from_str(&req.owner_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid owner_pub_key: {e}")))?;
        let mint_pub_key = Pubkey::from_str(&req.mint_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;

        let require_memo = req
            .memo_transfer_config
            .as_ref()
            .is_some_and(|cfg| cfg.require_incoming_memo);

        // get program id from token_program_enum
        let token_program_id = match token_program_enum {
            TokenProgram::Legacy => Ok(LEGACY_PROGRAM_ID),
            TokenProgram::TokenProgram2022 => Ok(TOKEN_2022_PROGRAM_ID),
            _ => Err(format!("unexpected token program id: {token_program_enum:?}")),
        }
        .map_err(Status::internal)?;

        // prepare vector to hold instructions
        let mut instructions: Vec<protochain_api::SolanaInstruction> = Vec::new();

        // derive associated token account address
        let ata_address = get_associated_token_address_with_program_id(
            &owner_pub_key,    // wallet
            &mint_pub_key,     // mint
            &token_program_id, // token program id
        );

        // determine amount of account space to allocate
        let holding_account_allocation_space =
            holding_account_space(require_memo).map_err(|e| {
                Status::internal(format!(
                    "could not determine holding account allocation space: {e}"
                ))
            })?;

        // determine rent exemption amount
        let rent = self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(holding_account_allocation_space)
            .map_err(|e| {
                Status::internal(format!(
                    "could not get minimum rent balance for rent exemption: {e}"
                ))
            })?;

        // construct the create account instruction
        let create_account_instruction = create_account(
            &Address::from(payer_pubkey.to_bytes()),
            &Address::from(ata_address.to_bytes()),
            rent,
            holding_account_space as u64,
            &Address::from(token_program_id.to_bytes()),
        );
        instructions.push(solana_instruction_to_proto(create_account_instruction));

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

        // Get the mint account data
        let account = self
            .rpc_client
            .get_account_with_commitment(&mint_pubkey, CommitmentConfig::confirmed())
            .map_err(|e| Status::internal(format!("Failed to get account: {e}")))?
            .value
            .ok_or_else(|| Status::not_found("Account not found"))?;

        // Parse amount from string to handle large numbers
        let amount = req
            .amount
            .parse::<u64>()
            .map_err(|e| Status::invalid_argument(format!("Invalid amount: {e}")))?;

        // Validate decimals
        let decimals = u8::try_from(req.decimals)
            .map_err(|_| Status::invalid_argument("decimals must be between 0 and 255"))?;

        // Determine token program id from account owner
        let token_program_id = get_token_program_id(sdk_token_program_to_proto(&account.owner));

        // Create the MintToChecked instruction (no additional signers for single authority)
        let instruction = mint_to_checked(
            &token_program_id,
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
