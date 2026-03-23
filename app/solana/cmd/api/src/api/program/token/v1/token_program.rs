//! Token program conversion utilities
//!
//! This module provides conversion utilities for mapping between protobuf token program
//! definitions and Solana SDK token program IDs.

use protochain_api::protochain::solana::r#type::v1::TokenProgram;
use solana_sdk::pubkey::Pubkey;
use spl_token::id as spl_token_id;
use spl_token_2022::id as spl_token_2022_id;

/// Converts a Solana SDK token program ID to the corresponding protobuf `TokenProgram`
///
/// Maps a Solana token program ID back to its protobuf representation.
///
/// # Arguments
/// * `program_id` - The Solana token program ID
///
/// # Returns
/// `TokenProgram::Unspecified` if the program ID doesn't match any known token programs
pub fn sdk_token_program_to_proto(program_id: &Pubkey) -> TokenProgram {
    if program_id == &spl_token_id() {
        TokenProgram::Legacy
    } else if program_id == &spl_token_2022_id() {
        TokenProgram::TokenProgram2022
    } else {
        TokenProgram::Unspecified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Converts a protobuf `TokenProgram` enum to the corresponding Solana SDK program ID.
    fn proto_token_program_to_sdk(token_program: TokenProgram) -> Result<Pubkey, String> {
        match token_program {
            TokenProgram::Unspecified => {
                Err("TokenProgram must be specified (cannot be UNSPECIFIED)".to_string())
            }
            TokenProgram::Legacy => Ok(spl_token_id()),
            TokenProgram::TokenProgram2022 => Ok(spl_token_2022_id()),
        }
    }

    #[test]
    fn test_proto_to_sdk_legacy() {
        let result = proto_token_program_to_sdk(TokenProgram::Legacy);
        assert!(result.is_ok());
        let Ok(v) = result else {
            unreachable!("Already asserted Ok")
        };
        assert_eq!(v, spl_token_id());
    }

    #[test]
    fn test_proto_to_sdk_token_2022() {
        let result = proto_token_program_to_sdk(TokenProgram::TokenProgram2022);
        assert!(result.is_ok());
        let Ok(v) = result else {
            unreachable!("Already asserted Ok")
        };
        assert_eq!(v, spl_token_2022_id());
    }

    #[test]
    fn test_proto_to_sdk_unspecified() {
        let result = proto_token_program_to_sdk(TokenProgram::Unspecified);
        assert!(result.is_err());
        let Err(err) = result else {
            unreachable!("Already asserted Err")
        };
        assert!(err.contains("TokenProgram must be specified"));
    }

    #[test]
    fn test_sdk_to_proto_legacy() {
        let result = sdk_token_program_to_proto(&spl_token_id());
        assert_eq!(result, TokenProgram::Legacy);
    }

    #[test]
    fn test_sdk_to_proto_token_2022() {
        let result = sdk_token_program_to_proto(&spl_token_2022_id());
        assert_eq!(result, TokenProgram::TokenProgram2022);
    }

    #[test]
    fn test_sdk_to_proto_unknown() {
        let program_id = Pubkey::new_unique();
        let result = sdk_token_program_to_proto(&program_id);
        assert_eq!(result, TokenProgram::Unspecified);
    }

    #[test]
    fn test_roundtrip_legacy() {
        let original = TokenProgram::Legacy;
        let Ok(program_id) = proto_token_program_to_sdk(original) else {
            unreachable!("Legacy should convert successfully")
        };
        let converted = sdk_token_program_to_proto(&program_id);
        assert_eq!(original, converted);
    }

    #[test]
    fn test_roundtrip_token_2022() {
        let original = TokenProgram::TokenProgram2022;
        let Ok(program_id) = proto_token_program_to_sdk(original) else {
            unreachable!("TokenProgram2022 should convert successfully")
        };
        let converted = sdk_token_program_to_proto(&program_id);
        assert_eq!(original, converted);
    }
}
