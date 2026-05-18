//! Validation helpers for Token-2022 extensions.

use std::collections::HashSet;

use tonic::Status;

use protochain_api::protochain::solana::program::token::v1::{
    token2022_extension, token2022_holding_account_extension, Token2022Extension,
    Token2022HoldingAccountExtension,
};
use solana_sdk::program_pack::Pack;
use spl_token_2022::{extension::ExtensionType, state::Account};

/// Validates that the given extension list contains no duplicates.
#[allow(clippy::result_large_err)]
pub(crate) fn validate_no_duplicate_extensions(
    extensions: &[Token2022Extension],
) -> Result<(), Status> {
    let mut seen: HashSet<&str> = HashSet::new();
    for ext in extensions {
        let key = match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_extension::Extension::Metadata(_) => "Metadata",
            token2022_extension::Extension::MintCloseAuthority(_) => "MintCloseAuthority",
            token2022_extension::Extension::TransferFee(_) => "TransferFee",
            token2022_extension::Extension::DefaultAccountState(_) => "DefaultAccountState",
            token2022_extension::Extension::PermanentDelegate(_) => "PermanentDelegate",
            token2022_extension::Extension::Pausable(_) => "Pausable",
        };
        if !seen.insert(key) {
            return Err(Status::invalid_argument(format!("Duplicate extension: {key}")));
        }
    }
    Ok(())
}

/// Validates that the given holding account extension list contains no duplicates.
#[allow(clippy::result_large_err)]
pub(crate) fn validate_no_duplicate_holding_account_extensions(
    extensions: &[Token2022HoldingAccountExtension],
) -> Result<(), Status> {
    let mut seen: HashSet<&str> = HashSet::new();
    for ext in extensions {
        let key = match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_holding_account_extension::Extension::MemoTransfer(_) => "MemoTransfer",
            token2022_holding_account_extension::Extension::ImmutableOwner(_) => "ImmutableOwner",
        };
        if !seen.insert(key) {
            return Err(Status::invalid_argument(format!(
                "Duplicate holding account extension: {key}"
            )));
        }
    }
    Ok(())
}

/// Collects the SDK `ExtensionType` variants requested by the holding account
/// extensions, used for calculating account size and building reallocate
/// instructions.
#[allow(clippy::result_large_err)]
pub(crate) fn holding_account_extension_types(
    extensions: &[Token2022HoldingAccountExtension],
) -> Result<Vec<ExtensionType>, Status> {
    let mut types = Vec::with_capacity(extensions.len());
    for ext in extensions {
        match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_holding_account_extension::Extension::MemoTransfer(_) => {
                types.push(ExtensionType::MemoTransfer);
            }
            token2022_holding_account_extension::Extension::ImmutableOwner(_) => {
                types.push(ExtensionType::ImmutableOwner);
            }
        }
    }
    Ok(types)
}

/// Calculates the total account size for a Token-2022 holding account with the
/// given extensions.
///
/// Returns `Account::LEN` when no extensions are provided.
#[allow(clippy::result_large_err)]
pub(crate) fn holding_account_total_space(
    extensions: &[Token2022HoldingAccountExtension],
) -> Result<usize, Status> {
    if extensions.is_empty() {
        return Ok(Account::LEN);
    }

    let sdk_types = holding_account_extension_types(extensions)?;
    ExtensionType::try_calculate_account_len::<Account>(&sdk_types).map_err(|e| {
        Status::internal(format!("failed to calculate holding account length for extensions: {e}"))
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use protochain_api::protochain::solana::program::token::v1::Token2022ExtensionMetadata;

    fn make_metadata_extension() -> Token2022Extension {
        Token2022Extension {
            extension: Some(token2022_extension::Extension::Metadata(Token2022ExtensionMetadata {
                name: "Test".to_string(),
                symbol: "TST".to_string(),
                uri: "https://example.com".to_string(),
                additional_metadata: HashMap::default(),
                metadata_address: String::new(),
                update_authority_pub_key: String::new(),
            })),
        }
    }

    #[test]
    fn test_no_duplicates_empty() {
        assert!(validate_no_duplicate_extensions(&[]).is_ok());
    }

    #[test]
    fn test_no_duplicates_single() {
        assert!(validate_no_duplicate_extensions(&[make_metadata_extension()]).is_ok());
    }

    #[test]
    fn test_duplicates_rejected() {
        let result = validate_no_duplicate_extensions(&[
            make_metadata_extension(),
            make_metadata_extension(),
        ]);
        assert!(result.is_err());
        let Err(status) = result else {
            unreachable!("Already asserted Err");
        };
        assert!(status.message().contains("Duplicate extension: Metadata"));
    }

    #[test]
    fn test_missing_extension_type() {
        let ext = Token2022Extension { extension: None };
        let result = validate_no_duplicate_extensions(&[ext]);
        assert!(result.is_err());
        let Err(status) = result else {
            unreachable!("Already asserted Err");
        };
        assert!(status.message().contains("Extension must have a type set"));
    }
}
