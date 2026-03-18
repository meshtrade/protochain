use crate::api::program::system::v1::service_impl::SystemProgramServiceImpl;
use crate::api::program::token::v1::token_program::get_token_program_id;
use crate::api::{
    common::solana_conversions::sdk_instruction_to_proto,
    program::token::v1::token_program::sdk_token_program_to_proto,
};
use protochain_api::protochain::solana::program::system::v1::{
    service_server::Service as SystemProgramService, CreateRequest as SystemCreateRequest,
};
use protochain_api::protochain::solana::program::token::v1::{
    service_server::Service as TokenProgramService, token2022_extension,
    CreateHoldingAccountRequest, CreateHoldingAccountResponse, CreateMintRequest,
    CreateMintResponse, GetCurrentMinRentForHoldingAccountRequest,
    GetCurrentMinRentForHoldingAccountResponse, GetCurrentMinRentForMintAccountRequest,
    GetCurrentMinRentForMintAccountResponse, InitialiseMintRequest, InitialiseMintResponse,
    MintInfo, MintRequest, MintResponse, ParseMintRequest, ParseMintResponse, Token2022Extension,
};
use protochain_api::protochain::solana::r#type::v1::TokenProgram;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token_2022::extension::memo_transfer::instruction::enable_required_transfer_memos;

use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_metadata_interface::{
    instruction::{initialize as initialize_token_metadata, update_field},
    state::{Field, TokenMetadata},
};

use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::ID as LEGACY_PROGRAM_ID;
use spl_token_2022::{
    extension::{
        metadata_pointer::instruction::initialize as initialize_metadata_pointer, ExtensionType,
        StateWithExtensions,
    },
    instruction::{initialize_mint2, mint_to_checked, reallocate},
    state::{Account, Mint},
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

/// Validates that the given extension list contains no duplicates.
#[allow(clippy::result_large_err)]
fn validate_no_duplicate_extensions(extensions: &[Token2022Extension]) -> Result<(), Status> {
    let mut seen: HashSet<&str> = HashSet::new();
    for ext in extensions {
        let key = match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_extension::Extension::Metadata(_) => "Metadata",
        };
        if !seen.insert(key) {
            return Err(Status::invalid_argument(format!("Duplicate extension: {key}")));
        }
    }
    Ok(())
}

/// Calculates the space (in bytes) to allocate when creating a mint account via
/// `System::CreateAccount`.
///
/// This includes the base mint layout and fixed-size extension type pods
/// (e.g. `MetadataPointer`), but **not** variable-length content like
/// `TokenMetadata` which the Token-2022 program allocates internally via
/// `realloc` when `initialize_token_metadata` is called.
///
/// Returns `Mint::LEN` when no extensions are provided.
#[allow(clippy::result_large_err)]
fn mint_create_account_space(extensions: &[Token2022Extension]) -> Result<usize, Status> {
    if extensions.is_empty() {
        return Ok(Mint::LEN);
    }

    let mut sdk_extension_types: Vec<ExtensionType> = Vec::new();
    for ext in extensions {
        match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_extension::Extension::Metadata(_) => {
                sdk_extension_types.push(ExtensionType::MetadataPointer);
            }
        }
    }

    ExtensionType::try_calculate_account_len::<Mint>(&sdk_extension_types).map_err(|e| {
        Status::internal(format!("failed to calculate mint account length for extensions: {e}"))
    })
}

/// Calculates the total space a mint account will occupy after **all** extensions
/// — including variable-length metadata content — have been fully initialised.
///
/// This is used to determine the rent-exempt lamport deposit at account creation.
/// The Token-2022 program resizes the account via `realloc` when metadata is
/// written, so the account must be pre-funded with enough lamports for the final
/// size even though `mint_create_account_space` returns a smaller allocation.
///
/// Returns `Mint::LEN` when no extensions are provided.
#[allow(clippy::result_large_err)]
fn mint_total_space_for_rent(extensions: &[Token2022Extension]) -> Result<usize, Status> {
    if extensions.is_empty() {
        return Ok(Mint::LEN);
    }

    let mut sdk_extension_types: Vec<ExtensionType> = Vec::new();
    let mut extra_variable_len: usize = 0;

    for ext in extensions {
        match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_extension::Extension::Metadata(meta) => {
                sdk_extension_types.push(ExtensionType::MetadataPointer);

                // `TokenMetadata` has two different pubkey field types by design:
                //   - `mint: Pubkey` — always required; every mint must have an address.
                //   - `update_authority: OptionalNonZeroPubkey` — optional; immutable mints
                //     have no update authority.  `OptionalNonZeroPubkey` encodes
                //     `Option<Pubkey>` in a fixed 32 bytes on-chain by treating the all-zeros
                //     pubkey as the "None" sentinel, avoiding the extra discriminant byte that
                //     a standard `Option<Pubkey>` would require.
                //
                // Both fields are always exactly 32 bytes on-chain, so neither value affects
                // the TLV size calculation — placeholder defaults are sufficient here.
                let token_metadata = TokenMetadata {
                    update_authority: OptionalNonZeroPubkey::default(),
                    mint: Pubkey::default(),
                    name: meta.name.clone(),
                    symbol: meta.symbol.clone(),
                    uri: meta.uri.clone(),
                    additional_metadata: meta
                        .additional_metadata
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                };

                extra_variable_len += token_metadata.tlv_size_of().map_err(|e| {
                    Status::internal(format!("failed to calculate metadata TLV size: {e}"))
                })?;
            }
        }
    }

    let base_space = ExtensionType::try_calculate_account_len::<Mint>(&sdk_extension_types)
        .map_err(|e| {
            Status::internal(format!("failed to calculate mint account length for extensions: {e}"))
        })?;

    Ok(base_space + extra_variable_len)
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

    rpc.get_minimum_balance_for_rent_exemption(space)
        .map_err(|e| Status::internal(format!("failed to fetch memo-aware rent: {e}")))
}

/// Builds the ordered list of SDK instructions needed to initialise a Token-2022
/// mint with the requested extensions.
///
/// The instruction sequence for a mint with the Metadata extension is:
///   1. `initialize_metadata_pointer`  – must precede `initialize_mint`
///   2. `initialize_mint`
///   3. `initialize_token_metadata`    – must follow `initialize_mint`
///   4. `update_field` × N             – one per additional-metadata entry
///
/// For a plain mint (no extensions) only step 2 is emitted.
///
/// New extension types can be supported by adding arms to the pre/post match
/// blocks and collecting the relevant instructions.
#[allow(clippy::result_large_err)]
fn build_token2022_mint_instructions(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
    extensions: &[Token2022Extension],
) -> Result<Vec<solana_sdk::instruction::Instruction>, Status> {
    // --- Phase 1: instructions that MUST run before initialize_mint ---
    let mut pre_init_instructions = Vec::new();
    // --- Phase 3: instructions that MUST run after initialize_mint ---
    let mut post_init_instructions = Vec::new();

    for ext in extensions {
        match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_extension::Extension::Metadata(meta) => {
                // Resolve metadata_address: default to mint itself (self-referencing metadata)
                let metadata_address = if meta.metadata_address.is_empty() {
                    *mint_pubkey
                } else {
                    Pubkey::from_str(&meta.metadata_address).map_err(|e| {
                        Status::invalid_argument(format!("Invalid metadata_address: {e}"))
                    })?
                };

                // Resolve update_authority: default to mint_authority
                let update_authority = if meta.update_authority_pub_key.is_empty() {
                    *mint_authority
                } else {
                    Pubkey::from_str(&meta.update_authority_pub_key).map_err(|e| {
                        Status::invalid_argument(format!(
                            "Invalid metadata update_authority_pub_key: {e}"
                        ))
                    })?
                };

                // Pre-init: metadata pointer must be initialised before the mint
                pre_init_instructions.push(
                    initialize_metadata_pointer(
                        token_program_id,
                        mint_pubkey,
                        Some(update_authority),
                        Some(metadata_address),
                    )
                    .map_err(|e| {
                        Status::internal(format!(
                            "could not create initialize_metadata_pointer instruction: {e}"
                        ))
                    })?,
                );

                // Post-init: token metadata must be initialised after the mint
                post_init_instructions.push(initialize_token_metadata(
                    token_program_id,
                    &metadata_address,
                    &update_authority,
                    mint_pubkey,
                    mint_authority,
                    meta.name.clone(),
                    meta.symbol.clone(),
                    meta.uri.clone(),
                ));

                // Post-init: additional metadata fields
                for (key, value) in &meta.additional_metadata {
                    post_init_instructions.push(update_field(
                        token_program_id,
                        &metadata_address,
                        &update_authority,
                        Field::Key(key.clone()),
                        value.clone(),
                    ));
                }
            }
        }
    }

    // --- Phase 2: initialize_mint itself ---
    let init_mint_instruction =
        initialize_mint2(token_program_id, mint_pubkey, mint_authority, freeze_authority, decimals)
            .map_err(|e| {
                Status::internal(format!("could not create initialise mint token instruction: {e}"))
            })?;

    // Assemble: pre-init → initialize_mint → post-init
    let mut instructions =
        Vec::with_capacity(pre_init_instructions.len() + 1 + post_init_instructions.len());
    instructions.append(&mut pre_init_instructions);
    instructions.push(init_mint_instruction);
    instructions.append(&mut post_init_instructions);

    Ok(instructions)
}

#[tonic::async_trait]
impl TokenProgramService for TokenProgramServiceImpl {
    /// Creates initialisation instructions for an SPL Legacy or Token-2022 mint.
    ///
    /// **SPL Legacy** (`TOKEN_PROGRAM_LEGACY`): returns a single `initialize_mint`
    /// instruction. Extensions must be empty.
    ///
    /// **Token-2022** (`TOKEN_PROGRAM_2022`): returns one or more instructions
    /// depending on the requested extensions.  With the Metadata extension the
    /// order is: metadata-pointer init → `initialize_mint` → token-metadata init
    /// → `update_field` × N.
    async fn initialise_mint(
        &self,
        request: Request<InitialiseMintRequest>,
    ) -> Result<Response<InitialiseMintResponse>, Status> {
        let req = request.into_inner();

        // parse public keys
        let mint_pubkey = Pubkey::from_str(&req.mint_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;
        let mint_authority = Pubkey::from_str(&req.mint_authority_pub_key).map_err(|e| {
            Status::invalid_argument(format!("Invalid mint_authority_pub_key: {e}"))
        })?;

        // parse optional freeze authority
        let freeze_authority = if req.freeze_authority_pub_key.is_empty() {
            None
        } else {
            Some(Pubkey::from_str(&req.freeze_authority_pub_key).map_err(|e| {
                Status::invalid_argument(format!("Invalid freeze_authority_pub_key: {e}"))
            })?)
        };

        // determine which token program to use
        let token_program_enum = TokenProgram::try_from(req.token_program)
            .map_err(|_| Status::invalid_argument("Invalid token program value"))?;

        let token_program_id = get_token_program_id(token_program_enum)
            .map_err(|e| Status::invalid_argument(format!("Invalid token program: {e}")))?;

        let decimals = u8::try_from(req.decimals)
            .map_err(|_| Status::invalid_argument("decimals must be between 0 and 255"))?;

        let sdk_instructions = match token_program_enum {
            TokenProgram::Legacy => {
                // SPL Legacy does not support extensions
                if !req.extensions.is_empty() {
                    return Err(Status::invalid_argument(
                        "Extensions are not supported for the Legacy SPL Token program",
                    ));
                }
                let instruction = initialize_mint2(
                    &token_program_id,
                    &mint_pubkey,
                    &mint_authority,
                    freeze_authority.as_ref(),
                    decimals,
                )
                .map_err(|e| {
                    Status::internal(format!(
                        "could not create initialise mint token instruction: {e}"
                    ))
                })?;
                vec![instruction]
            }
            TokenProgram::TokenProgram2022 => {
                validate_no_duplicate_extensions(&req.extensions)?;
                build_token2022_mint_instructions(
                    &token_program_id,
                    &mint_pubkey,
                    &mint_authority,
                    freeze_authority.as_ref(),
                    decimals,
                    &req.extensions,
                )?
            }
            TokenProgram::Unspecified => {
                return Err(Status::invalid_argument(
                    "token_program must be specified (cannot be UNSPECIFIED)",
                ));
            }
        };

        let instructions = sdk_instructions
            .into_iter()
            .map(sdk_instruction_to_proto)
            .collect();

        Ok(Response::new(InitialiseMintResponse { instructions }))
    }

    /// Gets the minimum rent-exempt balance and allocation space for a mint account.
    ///
    /// With no extensions both values are based on `Mint::LEN` (82 bytes).
    ///
    /// With extensions the returned `space` covers only the base mint layout and
    /// fixed-size extension pods (e.g. `MetadataPointer`).  The returned
    /// `lamports` covers the **full** final account size — including
    /// variable-length metadata content that the Token-2022 program allocates
    /// via `realloc` during `initialize_token_metadata`.  This means `lamports`
    /// may exceed `rent_exempt(space)` when metadata extensions are present; the
    /// excess ensures the account remains rent-exempt after Token-2022 resizes it.
    async fn get_current_min_rent_for_mint_account(
        &self,
        request: Request<GetCurrentMinRentForMintAccountRequest>,
    ) -> Result<Response<GetCurrentMinRentForMintAccountResponse>, Status> {
        let req = request.into_inner();

        validate_no_duplicate_extensions(&req.extensions)?;

        // Space for System::CreateAccount — base extension types only.
        let space = mint_create_account_space(&req.extensions)?;

        // Rent for the full final size including variable-length metadata content
        // that Token-2022 will allocate via realloc.
        let rent_space = mint_total_space_for_rent(&req.extensions)?;

        let lamports = self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(rent_space)
            .map_err(|e| {
                Status::internal(format!("failed to get minimum balance for mint account: {e}"))
            })?;

        Ok(Response::new(GetCurrentMinRentForMintAccountResponse {
            lamports,
            space: space as u64,
        }))
    }

    /// Parses mint account data into structured format.
    ///
    /// Supports both Legacy SPL Token and Token-2022 mints. The account owner is
    /// checked to ensure it belongs to a known token program, and the appropriate
    /// unpacking strategy is used (Token-2022 mints may contain extension data
    /// beyond the base 82-byte Mint layout).
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

        // Validate account is owned by a known token program
        if account.owner != LEGACY_PROGRAM_ID && account.owner != TOKEN_2022_PROGRAM_ID {
            return Err(Status::invalid_argument(format!(
                "Account owner {} is not a known token program",
                account.owner,
            )));
        }

        // Unpack the mint account data.
        // Use StateWithExtensions for Token-2022 accounts which may have extension
        // data beyond the base 82-byte Mint layout; use Mint::unpack for legacy
        // SPL accounts which are always exactly 82 bytes.
        let mint = if account.owner == TOKEN_2022_PROGRAM_ID {
            StateWithExtensions::<Mint>::unpack(&account.data)
                .map(|state| state.base)
                .map_err(|e| {
                    Status::invalid_argument(format!("Failed to parse Token-2022 mint: {e}"))
                })?
        } else {
            Mint::unpack(&account.data).map_err(|e| {
                Status::invalid_argument(format!("Failed to parse mint account: {e}"))
            })?
        };

        Ok(Response::new(ParseMintResponse {
            mint: Some(MintInfo {
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
            }),
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
            .get_current_min_rent_for_mint_account(Request::new(
                GetCurrentMinRentForMintAccountRequest { extensions: vec![] },
            ))
            .await?
            .into_inner();

        // Step 2: Create system account creation instruction
        let system_service = SystemProgramServiceImpl::new();

        let token_program_enum = TokenProgram::try_from(req.token_program)
            .map_err(|_| Status::invalid_argument("Invalid token program value"))?;

        // Get the program ID pubkey and convert to string for the system program
        let owner_pubkey = get_token_program_id(token_program_enum)
            .map_err(|e| Status::invalid_argument(format!("Invalid token program: {e}")))?;

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

        // Step 3: Create mint initialization instructions (extensions not yet handled)
        let init_response = self
            .initialise_mint(Request::new(InitialiseMintRequest {
                mint_pub_key: req.mint_pub_key,
                mint_authority_pub_key: req.mint_authority_pub_key,
                freeze_authority_pub_key: req.freeze_authority_pub_key,
                decimals: req.decimals,
                token_program: req.token_program,
                extensions: vec![],
            }))
            .await?
            .into_inner();

        // Step 4: Compose response with create account + all init instructions
        let mut instructions = Vec::new();
        if let Some(instr) = create_instruction.instruction {
            instructions.push(instr);
        }
        instructions.extend(init_response.instructions);

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

        // check if memo extension should be added
        let require_memo = req
            .memo_transfer_config
            .as_ref()
            .is_some_and(|cfg| cfg.require_incoming_memo);

        // get program id from token_program_enum
        let token_program_id = match token_program_enum {
            TokenProgram::Legacy => Ok(LEGACY_PROGRAM_ID),
            TokenProgram::TokenProgram2022 => Ok(TOKEN_2022_PROGRAM_ID),
            TokenProgram::Unspecified => {
                Err(format!("unexpected token program id: {token_program_enum:?}"))
            }
        }
        .map_err(Status::internal)?;

        // prepare vector to hold instructions
        let mut instructions: Vec<protochain_api::SolanaInstruction> = Vec::new();

        let create_account_instruction = create_associated_token_account(
            &payer_pubkey,
            &owner_pub_key,
            &mint_pub_key,
            &token_program_id,
        );
        instructions.push(sdk_instruction_to_proto(create_account_instruction));

        // derive associated token account address
        let ata_address = get_associated_token_address_with_program_id(
            &owner_pub_key,    // wallet
            &mint_pub_key,     // mint
            &token_program_id, // token program id
        );

        // add enable memo instruction if required
        if require_memo {
            let reallocate_instruction = reallocate(
                &token_program_id,
                &ata_address,
                &payer_pubkey,
                &owner_pub_key,
                &[&owner_pub_key],
                &[ExtensionType::MemoTransfer],
            )
            .map_err(|e| {
                Status::internal(format!(
                    "could not create reallocation instruction to allow for memo extension: {e}"
                ))
            })?;
            instructions.push(sdk_instruction_to_proto(reallocate_instruction));

            let enable_required_transfer_memos_instruction = enable_required_transfer_memos(
                &token_program_id,
                &ata_address,
                &owner_pub_key,
                &[&payer_pubkey],
            )
            .map_err(|e| {
                Status::internal(format!(
                    "could not create required transfer memos instruction: {e}"
                ))
            })?;
            instructions.push(sdk_instruction_to_proto(enable_required_transfer_memos_instruction));
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
        let token_program_id = get_token_program_id(sdk_token_program_to_proto(&account.owner))
            .map_err(|e| Status::internal(format!("Failed to determine token program: {e}")))?;

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
