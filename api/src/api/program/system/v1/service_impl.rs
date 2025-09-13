use solana_sdk::{pubkey::Pubkey, system_instruction, system_program};
use std::str::FromStr;
use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::program::system::v1::{
    service_server::Service as SystemProgramService, AdvanceNonceAccountRequest, AllocateRequest,
    AllocateWithSeedRequest, AssignRequest, AssignWithSeedRequest, AuthorizeNonceAccountRequest,
    CreateRequest, CreateWithSeedRequest, InitializeNonceAccountRequest, TransferRequest,
    TransferWithSeedRequest, UpgradeNonceAccountRequest, WithdrawNonceAccountRequest,
};
use protochain_api::protochain::solana::transaction::v1::SolanaInstruction;

use crate::api::common::solana_conversions::sdk_instruction_to_proto;

/// Pure instruction-based System Program service implementation.
///
/// All methods return composable `SolanaInstruction` objects for transaction building.
/// This is a pure SDK wrapper - no RPC client or transaction compilation here.
#[derive(Clone)]
pub struct SystemProgramServiceImpl {
    // No RPC client needed - we only build instructions
}

impl Default for SystemProgramServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemProgramServiceImpl {
    /// Creates a new instance of the System Program service.
    pub const fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl SystemProgramService for SystemProgramServiceImpl {
    /// Creates a new account instruction.
    async fn create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        // Validation
        if req.payer.is_empty() {
            return Err(Status::invalid_argument("Payer address is required"));
        }
        if req.new_account.is_empty() {
            return Err(Status::invalid_argument("New account address is required"));
        }

        let payer = Pubkey::from_str(&req.payer)
            .map_err(|e| Status::invalid_argument(format!("Invalid payer address: {e}")))?;

        let new_account = Pubkey::from_str(&req.new_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid new account address: {e}")))?;

        // Parse owner program (default to system program if empty)
        let owner = if req.owner.is_empty() {
            system_program::id()
        } else {
            Pubkey::from_str(&req.owner).map_err(|e| {
                Status::invalid_argument(format!("Invalid owner program address: {e}"))
            })?
        };

        // Build instruction using SDK
        let instruction = system_instruction::create_account(
            &payer,
            &new_account,
            req.lamports,
            req.space,
            &owner,
        );

        // Convert to proto format
        let mut proto_instruction = sdk_instruction_to_proto(instruction);

        // Add descriptive information for composable transactions
        let owner_display = if req.owner.is_empty() {
            "system program (default)".to_string()
        } else {
            req.owner.clone()
        };
        proto_instruction.description = format!(
            "Create account: {} (payer: {}, owner: {}, lamports: {}, space: {})",
            req.new_account, req.payer, owner_display, req.lamports, req.space
        );

        Ok(Response::new(proto_instruction))
    }

    /// Creates a transfer instruction.
    async fn transfer(
        &self,
        request: Request<TransferRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        if req.from.is_empty() {
            return Err(Status::invalid_argument("From address is required"));
        }
        if req.to.is_empty() {
            return Err(Status::invalid_argument("To address is required"));
        }

        let from = Pubkey::from_str(&req.from)
            .map_err(|e| Status::invalid_argument(format!("Invalid from address: {e}")))?;

        let to = Pubkey::from_str(&req.to)
            .map_err(|e| Status::invalid_argument(format!("Invalid to address: {e}")))?;

        let instruction = system_instruction::transfer(&from, &to, req.lamports);

        // Convert to proto format and add description
        let mut proto_instruction = sdk_instruction_to_proto(instruction);
        proto_instruction.description =
            format!("Transfer {} lamports from {} to {}", req.lamports, req.from, req.to);

        Ok(Response::new(proto_instruction))
    }

    /// Creates an allocate instruction.
    async fn allocate(
        &self,
        request: Request<AllocateRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        if req.account.is_empty() {
            return Err(Status::invalid_argument("Account address is required"));
        }

        let account = Pubkey::from_str(&req.account)
            .map_err(|e| Status::invalid_argument(format!("Invalid account address: {e}")))?;

        let instruction = system_instruction::allocate(&account, req.space);
        Ok(Response::new(sdk_instruction_to_proto(instruction)))
    }

    /// Creates an assign instruction.
    async fn assign(
        &self,
        request: Request<AssignRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        if req.account.is_empty() {
            return Err(Status::invalid_argument("Account address is required"));
        }
        if req.owner_program.is_empty() {
            return Err(Status::invalid_argument("Owner program is required"));
        }

        let account = Pubkey::from_str(&req.account)
            .map_err(|e| Status::invalid_argument(format!("Invalid account address: {e}")))?;

        let owner_program = Pubkey::from_str(&req.owner_program)
            .map_err(|e| Status::invalid_argument(format!("Invalid owner program: {e}")))?;

        let instruction = system_instruction::assign(&account, &owner_program);
        Ok(Response::new(sdk_instruction_to_proto(instruction)))
    }

    /// Creates a create-with-seed instruction.
    async fn create_with_seed(
        &self,
        request: Request<CreateWithSeedRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        if req.payer.is_empty() {
            return Err(Status::invalid_argument("Payer address is required"));
        }
        if req.new_account.is_empty() {
            return Err(Status::invalid_argument("New account address is required"));
        }
        if req.base.is_empty() {
            return Err(Status::invalid_argument("Base address is required"));
        }
        if req.seed.is_empty() {
            return Err(Status::invalid_argument("Seed is required"));
        }

        let payer = Pubkey::from_str(&req.payer)
            .map_err(|e| Status::invalid_argument(format!("Invalid payer address: {e}")))?;

        let new_account = Pubkey::from_str(&req.new_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid new account address: {e}")))?;

        let base = Pubkey::from_str(&req.base)
            .map_err(|e| Status::invalid_argument(format!("Invalid base address: {e}")))?;

        let instruction = system_instruction::create_account_with_seed(
            &payer,
            &new_account,
            &base,
            &req.seed,
            req.lamports,
            req.space,
            &system_program::id(),
        );

        Ok(Response::new(sdk_instruction_to_proto(instruction)))
    }

    /// Creates an allocate-with-seed instruction.
    async fn allocate_with_seed(
        &self,
        request: Request<AllocateWithSeedRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        if req.account.is_empty() {
            return Err(Status::invalid_argument("Account address is required"));
        }
        if req.base.is_empty() {
            return Err(Status::invalid_argument("Base address is required"));
        }
        if req.seed.is_empty() {
            return Err(Status::invalid_argument("Seed is required"));
        }

        let account = Pubkey::from_str(&req.account)
            .map_err(|e| Status::invalid_argument(format!("Invalid account address: {e}")))?;

        let base = Pubkey::from_str(&req.base)
            .map_err(|e| Status::invalid_argument(format!("Invalid base address: {e}")))?;

        let instruction = system_instruction::allocate_with_seed(
            &account,
            &base,
            &req.seed,
            req.space,
            &system_program::id(),
        );

        Ok(Response::new(sdk_instruction_to_proto(instruction)))
    }

    /// Creates an assign-with-seed instruction.
    async fn assign_with_seed(
        &self,
        request: Request<AssignWithSeedRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        if req.account.is_empty() {
            return Err(Status::invalid_argument("Account address is required"));
        }
        if req.base.is_empty() {
            return Err(Status::invalid_argument("Base address is required"));
        }
        if req.seed.is_empty() {
            return Err(Status::invalid_argument("Seed is required"));
        }
        if req.owner_program.is_empty() {
            return Err(Status::invalid_argument("Owner program is required"));
        }

        let account = Pubkey::from_str(&req.account)
            .map_err(|e| Status::invalid_argument(format!("Invalid account address: {e}")))?;

        let base = Pubkey::from_str(&req.base)
            .map_err(|e| Status::invalid_argument(format!("Invalid base address: {e}")))?;

        let owner_program = Pubkey::from_str(&req.owner_program)
            .map_err(|e| Status::invalid_argument(format!("Invalid owner program: {e}")))?;

        let instruction =
            system_instruction::assign_with_seed(&account, &base, &req.seed, &owner_program);

        Ok(Response::new(sdk_instruction_to_proto(instruction)))
    }

    /// Creates a transfer-with-seed instruction.
    async fn transfer_with_seed(
        &self,
        request: Request<TransferWithSeedRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        if req.from.is_empty() {
            return Err(Status::invalid_argument("From address is required"));
        }
        if req.from_base.is_empty() {
            return Err(Status::invalid_argument("From base address is required"));
        }
        if req.from_seed.is_empty() {
            return Err(Status::invalid_argument("From seed is required"));
        }
        if req.to.is_empty() {
            return Err(Status::invalid_argument("To address is required"));
        }

        let from = Pubkey::from_str(&req.from)
            .map_err(|e| Status::invalid_argument(format!("Invalid from address: {e}")))?;

        let from_base = Pubkey::from_str(&req.from_base)
            .map_err(|e| Status::invalid_argument(format!("Invalid from base address: {e}")))?;

        let to = Pubkey::from_str(&req.to)
            .map_err(|e| Status::invalid_argument(format!("Invalid to address: {e}")))?;

        let instruction = system_instruction::transfer_with_seed(
            &from,
            &from_base,
            req.from_seed.clone(),
            &system_program::id(),
            &to,
            req.lamports,
        );

        Ok(Response::new(sdk_instruction_to_proto(instruction)))
    }

    /// Creates an initialize-nonce-account instruction.
    async fn initialize_nonce_account(
        &self,
        request: Request<InitializeNonceAccountRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        if req.nonce_account.is_empty() {
            return Err(Status::invalid_argument("Nonce account address is required"));
        }
        if req.authority.is_empty() {
            return Err(Status::invalid_argument("Authority address is required"));
        }

        let nonce_account = Pubkey::from_str(&req.nonce_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid nonce account address: {e}")))?;

        let authority = Pubkey::from_str(&req.authority)
            .map_err(|e| Status::invalid_argument(format!("Invalid authority address: {e}")))?;

        // Note: initialize_nonce_account might not be available in this solana-sdk version
        // Using create_nonce_account which returns Vec<Instruction>, take the second one (initialize)
        let instructions = system_instruction::create_nonce_account(
            &authority,     // payer
            &nonce_account, // nonce account
            &authority,     // authority
            1_000_000,      // minimum balance for nonce account
        );
        // Take the initialize instruction (second one) - first is create_account
        let instruction = instructions
            .into_iter()
            .nth(1)
            .ok_or_else(|| Status::internal("Failed to create initialize nonce instruction"))?;

        Ok(Response::new(sdk_instruction_to_proto(instruction)))
    }

    /// Creates an authorize-nonce-account instruction.
    async fn authorize_nonce_account(
        &self,
        request: Request<AuthorizeNonceAccountRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        if req.nonce_account.is_empty() {
            return Err(Status::invalid_argument("Nonce account address is required"));
        }
        if req.current_authority.is_empty() {
            return Err(Status::invalid_argument("Current authority address is required"));
        }
        if req.new_authority.is_empty() {
            return Err(Status::invalid_argument("New authority address is required"));
        }

        let nonce_account = Pubkey::from_str(&req.nonce_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid nonce account address: {e}")))?;

        let current_authority = Pubkey::from_str(&req.current_authority).map_err(|e| {
            Status::invalid_argument(format!("Invalid current authority address: {e}"))
        })?;

        let new_authority = Pubkey::from_str(&req.new_authority)
            .map_err(|e| Status::invalid_argument(format!("Invalid new authority address: {e}")))?;

        let instruction = system_instruction::authorize_nonce_account(
            &nonce_account,
            &current_authority,
            &new_authority,
        );

        Ok(Response::new(sdk_instruction_to_proto(instruction)))
    }

    /// Creates a withdraw-nonce-account instruction.
    async fn withdraw_nonce_account(
        &self,
        request: Request<WithdrawNonceAccountRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        if req.nonce_account.is_empty() {
            return Err(Status::invalid_argument("Nonce account address is required"));
        }
        if req.authority.is_empty() {
            return Err(Status::invalid_argument("Authority address is required"));
        }
        if req.to.is_empty() {
            return Err(Status::invalid_argument("To address is required"));
        }

        let nonce_account = Pubkey::from_str(&req.nonce_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid nonce account address: {e}")))?;

        let authority = Pubkey::from_str(&req.authority)
            .map_err(|e| Status::invalid_argument(format!("Invalid authority address: {e}")))?;

        let to = Pubkey::from_str(&req.to)
            .map_err(|e| Status::invalid_argument(format!("Invalid to address: {e}")))?;

        let instruction = system_instruction::withdraw_nonce_account(
            &nonce_account,
            &authority,
            &to,
            req.lamports,
        );

        Ok(Response::new(sdk_instruction_to_proto(instruction)))
    }

    /// Creates an advance-nonce-account instruction.
    async fn advance_nonce_account(
        &self,
        request: Request<AdvanceNonceAccountRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        if req.nonce_account.is_empty() {
            return Err(Status::invalid_argument("Nonce account address is required"));
        }
        if req.authority.is_empty() {
            return Err(Status::invalid_argument("Authority address is required"));
        }

        let nonce_account = Pubkey::from_str(&req.nonce_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid nonce account address: {e}")))?;

        let authority = Pubkey::from_str(&req.authority)
            .map_err(|e| Status::invalid_argument(format!("Invalid authority address: {e}")))?;

        let instruction = system_instruction::advance_nonce_account(&nonce_account, &authority);

        Ok(Response::new(sdk_instruction_to_proto(instruction)))
    }

    /// Creates an upgrade-nonce-account instruction.
    async fn upgrade_nonce_account(
        &self,
        request: Request<UpgradeNonceAccountRequest>,
    ) -> Result<Response<SolanaInstruction>, Status> {
        let req = request.into_inner();

        if req.nonce_account.is_empty() {
            return Err(Status::invalid_argument("Nonce account address is required"));
        }

        let nonce_account = Pubkey::from_str(&req.nonce_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid nonce account address: {e}")))?;

        let instruction = system_instruction::upgrade_nonce_account(nonce_account);

        Ok(Response::new(sdk_instruction_to_proto(instruction)))
    }
}

#[cfg(test)]
mod tests;
