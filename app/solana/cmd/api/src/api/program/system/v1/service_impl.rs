use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::program::system::v1::{
    service_server::Service as SystemProgramService, AdvanceNonceAccountRequest,
    AdvanceNonceAccountResponse, AllocateRequest, AllocateResponse, AllocateWithSeedRequest,
    AllocateWithSeedResponse, AssignRequest, AssignResponse, AssignWithSeedRequest,
    AssignWithSeedResponse, AuthorizeNonceAccountRequest, AuthorizeNonceAccountResponse,
    BuildMemoRequest, BuildMemoResponse, CreateRequest, CreateResponse, CreateWithSeedRequest,
    CreateWithSeedResponse, InitializeNonceAccountRequest, InitializeNonceAccountResponse,
    TransferRequest, TransferResponse, TransferWithSeedRequest, TransferWithSeedResponse,
    UpgradeNonceAccountRequest, UpgradeNonceAccountResponse, WithdrawNonceAccountRequest,
    WithdrawNonceAccountResponse,
};

mod allocate;
mod assign;
mod create;
mod memo;
mod nonce;
mod transfer;

#[cfg(test)]
mod tests;

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
    ) -> Result<Response<CreateResponse>, Status> {
        self.handle_create(request)
    }

    /// Creates a transfer instruction.
    async fn transfer(
        &self,
        request: Request<TransferRequest>,
    ) -> Result<Response<TransferResponse>, Status> {
        self.handle_transfer(request)
    }

    /// Creates an allocate instruction.
    async fn allocate(
        &self,
        request: Request<AllocateRequest>,
    ) -> Result<Response<AllocateResponse>, Status> {
        self.handle_allocate(request)
    }

    /// Creates an assign instruction.
    async fn assign(
        &self,
        request: Request<AssignRequest>,
    ) -> Result<Response<AssignResponse>, Status> {
        self.handle_assign(request)
    }

    /// Creates a create-with-seed instruction.
    async fn create_with_seed(
        &self,
        request: Request<CreateWithSeedRequest>,
    ) -> Result<Response<CreateWithSeedResponse>, Status> {
        self.handle_create_with_seed(request)
    }

    /// Creates an allocate-with-seed instruction.
    async fn allocate_with_seed(
        &self,
        request: Request<AllocateWithSeedRequest>,
    ) -> Result<Response<AllocateWithSeedResponse>, Status> {
        self.handle_allocate_with_seed(request)
    }

    /// Creates an assign-with-seed instruction.
    async fn assign_with_seed(
        &self,
        request: Request<AssignWithSeedRequest>,
    ) -> Result<Response<AssignWithSeedResponse>, Status> {
        self.handle_assign_with_seed(request)
    }

    /// Creates a transfer-with-seed instruction.
    async fn transfer_with_seed(
        &self,
        request: Request<TransferWithSeedRequest>,
    ) -> Result<Response<TransferWithSeedResponse>, Status> {
        self.handle_transfer_with_seed(request)
    }

    /// Creates an initialize-nonce-account instruction.
    async fn initialize_nonce_account(
        &self,
        request: Request<InitializeNonceAccountRequest>,
    ) -> Result<Response<InitializeNonceAccountResponse>, Status> {
        self.handle_initialize_nonce_account(request)
    }

    /// Creates an authorize-nonce-account instruction.
    async fn authorize_nonce_account(
        &self,
        request: Request<AuthorizeNonceAccountRequest>,
    ) -> Result<Response<AuthorizeNonceAccountResponse>, Status> {
        self.handle_authorize_nonce_account(request)
    }

    /// Creates a withdraw-nonce-account instruction.
    async fn withdraw_nonce_account(
        &self,
        request: Request<WithdrawNonceAccountRequest>,
    ) -> Result<Response<WithdrawNonceAccountResponse>, Status> {
        self.handle_withdraw_nonce_account(request)
    }

    /// Creates an advance-nonce-account instruction.
    async fn advance_nonce_account(
        &self,
        request: Request<AdvanceNonceAccountRequest>,
    ) -> Result<Response<AdvanceNonceAccountResponse>, Status> {
        self.handle_advance_nonce_account(request)
    }

    /// Creates an upgrade-nonce-account instruction.
    async fn upgrade_nonce_account(
        &self,
        request: Request<UpgradeNonceAccountRequest>,
    ) -> Result<Response<UpgradeNonceAccountResponse>, Status> {
        self.handle_upgrade_nonce_account(request)
    }

    /// Builds a memo instruction.
    async fn build_memo(
        &self,
        request: Request<BuildMemoRequest>,
    ) -> Result<Response<BuildMemoResponse>, Status> {
        self.handle_build_memo(request)
    }
}
