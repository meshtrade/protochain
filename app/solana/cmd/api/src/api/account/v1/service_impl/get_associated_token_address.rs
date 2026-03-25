//! Handler implementation for `GetAssociatedTokenAddress`.

use std::str::FromStr;

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::account::v1::{
    GetAssociatedTokenAddressRequest, GetAssociatedTokenAddressResponse,
};
use protochain_api::protochain::solana::r#type::v1::TokenProgram;

use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::ID as LEGACY_PROGRAM_ID;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use super::AccountServiceImpl;

#[allow(clippy::result_large_err, clippy::unused_self)]
impl AccountServiceImpl {
    /// Derives the associated token address for a given owner, mint, and token program.
    pub(super) fn handle_get_associated_token_address(
        &self,
        request: Request<GetAssociatedTokenAddressRequest>,
    ) -> Result<Response<GetAssociatedTokenAddressResponse>, Status> {
        let req = request.into_inner();

        let owner_pub_key = Pubkey::from_str(req.owner_address.as_str())
            .map_err(|e| Status::invalid_argument(format!("Invalid address format: {e}")))?;

        let mint_pub_key = Pubkey::from_str(req.mint_address.as_str())
            .map_err(|e| Status::invalid_argument(format!("Invalid address format: {e}")))?;

        let token_program_enum = TokenProgram::try_from(req.token_program)
            .map_err(|_| Status::invalid_argument("Invalid token program value"))?;

        // get program id from token_program_enum
        let token_program = match token_program_enum {
            TokenProgram::Legacy => Ok(LEGACY_PROGRAM_ID),
            TokenProgram::TokenProgram2022 => Ok(TOKEN_2022_PROGRAM_ID),
            TokenProgram::Unspecified => {
                Err(format!("unexpected token program id: {token_program_enum:?}"))
            }
        }
        .map_err(Status::internal)?;

        let address = get_associated_token_address_with_program_id(
            &owner_pub_key,
            &mint_pub_key,
            &token_program,
        );

        Ok(Response::new(GetAssociatedTokenAddressResponse {
            address: address.to_string(),
        }))
    }
}
