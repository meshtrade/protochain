//! Token-2022 **Default Account State** extension: extraction and instruction building.

use tonic::Status;

use solana_sdk::pubkey::Pubkey;
use spl_token_2022::{
    extension::{
        default_account_state::{
            instruction::initialize_default_account_state, DefaultAccountState,
        },
        BaseStateWithExtensions, StateWithExtensions,
    },
    state::{AccountState, Mint},
};

use protochain_api::protochain::solana::program::token::v1::{
    self as token_proto, token2022_extension, Token2022Extension,
    Token2022ExtensionDefaultAccountState,
};

/// Converts a proto `AccountState` enum to the SDK `AccountState`.
#[allow(clippy::result_large_err)]
fn proto_account_state_to_sdk(state: token_proto::AccountState) -> Result<AccountState, Status> {
    match state {
        token_proto::AccountState::Unspecified => Err(Status::invalid_argument(
            "default_account_state must be INITIALIZED or FROZEN, not UNSPECIFIED",
        )),
        token_proto::AccountState::Initialized => Ok(AccountState::Initialized),
        token_proto::AccountState::Frozen => Ok(AccountState::Frozen),
    }
}

/// Converts an SDK `AccountState` to the proto `AccountState` enum.
const fn sdk_account_state_to_proto(state: u8) -> token_proto::AccountState {
    match state {
        1 => token_proto::AccountState::Initialized,
        2 => token_proto::AccountState::Frozen,
        _ => token_proto::AccountState::Unspecified,
    }
}

/// Extracts the Default Account State extension from a parsed Token-2022 mint
/// account and returns it as a `Token2022Extension` proto, or `None` if the
/// extension is not present.
pub(crate) fn extract_default_account_state_extension(
    state: &StateWithExtensions<'_, Mint>,
) -> Option<Token2022Extension> {
    let ext = state.get_extension::<DefaultAccountState>().ok()?;

    Some(Token2022Extension {
        extension: Some(token2022_extension::Extension::DefaultAccountState(
            Token2022ExtensionDefaultAccountState {
                state: sdk_account_state_to_proto(ext.state).into(),
            },
        )),
    })
}

/// Builds the pre-init instruction for the Default Account State extension.
///
/// Returns `(pre_init, post_init)` — only pre-init is populated.
///
/// - **pre-init**: `initialize_default_account_state` (must precede `initialize_mint`)
#[allow(clippy::result_large_err)]
pub(crate) fn build_default_account_state_instructions(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    config: Token2022ExtensionDefaultAccountState,
) -> Result<
    (
        Vec<solana_sdk::instruction::Instruction>,
        Vec<solana_sdk::instruction::Instruction>,
    ),
    Status,
> {
    let account_state = proto_account_state_to_sdk(config.state())?;

    let ix = initialize_default_account_state(token_program_id, mint_pubkey, &account_state)
        .map_err(|e| {
            Status::internal(format!(
                "could not create initialize_default_account_state instruction: {e}"
            ))
        })?;

    Ok((vec![ix], Vec::new()))
}
