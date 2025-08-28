use protosol_api::protosol::solana::transaction::v1::{SolanaAccountMeta, SolanaInstruction};
use solana_sdk::{instruction::AccountMeta, instruction::Instruction};
use std::str::FromStr;

pub fn sdk_instruction_to_proto(instruction: Instruction) -> SolanaInstruction {
    SolanaInstruction {
        program_id: instruction.program_id.to_string(),
        accounts: instruction
            .accounts
            .iter()
            .map(sdk_account_meta_to_proto)
            .collect(),
        data: instruction.data,
        description: String::new(),
    }
}

pub fn sdk_account_meta_to_proto(account_meta: &AccountMeta) -> SolanaAccountMeta {
    SolanaAccountMeta {
        pubkey: account_meta.pubkey.to_string(),
        is_signer: account_meta.is_signer,
        is_writable: account_meta.is_writable,
    }
}

pub fn proto_instruction_to_sdk(instruction: SolanaInstruction) -> Result<Instruction, String> {
    let program_id = solana_sdk::pubkey::Pubkey::from_str(&instruction.program_id)
        .map_err(|e| format!("Invalid program_id: {e}"))?;

    let accounts: Result<Vec<AccountMeta>, String> = instruction
        .accounts
        .iter()
        .map(proto_account_meta_to_sdk)
        .collect();

    Ok(Instruction {
        program_id,
        accounts: accounts?,
        data: instruction.data,
    })
}

/// Converts a protobuf `SolanaAccountMeta` to a Solana SDK `AccountMeta`.
///
/// This function transforms account metadata from the protobuf representation
/// used in gRPC APIs to the native Solana SDK format used for transactions.
///
/// # Arguments
/// * `account_meta` - The protobuf account metadata to convert
///
/// # Returns
/// * `Ok(AccountMeta)` - Successfully converted account metadata
/// * `Err(String)` - Error message if the pubkey string is invalid
pub fn proto_account_meta_to_sdk(account_meta: &SolanaAccountMeta) -> Result<AccountMeta, String> {
    let pubkey = solana_sdk::pubkey::Pubkey::from_str(&account_meta.pubkey)
        .map_err(|e| format!("Invalid pubkey: {e}"))?;

    Ok(AccountMeta {
        pubkey,
        is_signer: account_meta.is_signer,
        is_writable: account_meta.is_writable,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // unwrap is acceptable in tests for cleaner assertions
mod tests {
    use super::*;
    use solana_sdk::{pubkey::Pubkey, system_instruction, system_program};

    #[test]
    fn test_instruction_conversion_roundtrip() {
        let original = system_instruction::create_account(
            &Pubkey::new_unique(),
            &Pubkey::new_unique(),
            1_000_000,
            0,
            &system_program::id(),
        );

        let proto = sdk_instruction_to_proto(original.clone());
        let converted = proto_instruction_to_sdk(proto).unwrap();

        assert_eq!(original.program_id, converted.program_id);
        assert_eq!(original.data, converted.data);
        assert_eq!(original.accounts.len(), converted.accounts.len());

        for (orig, conv) in original.accounts.iter().zip(converted.accounts.iter()) {
            assert_eq!(orig.pubkey, conv.pubkey);
            assert_eq!(orig.is_signer, conv.is_signer);
            assert_eq!(orig.is_writable, conv.is_writable);
        }
    }

    #[test]
    fn test_account_meta_conversion_roundtrip() {
        let original = AccountMeta::new(Pubkey::new_unique(), true);

        let proto = sdk_account_meta_to_proto(&original);
        let converted = proto_account_meta_to_sdk(&proto).unwrap();

        assert_eq!(original.pubkey, converted.pubkey);
        assert_eq!(original.is_signer, converted.is_signer);
        assert_eq!(original.is_writable, converted.is_writable);
    }

    #[test]
    fn test_account_meta_readonly() {
        let original = AccountMeta::new_readonly(Pubkey::new_unique(), false);

        let proto = sdk_account_meta_to_proto(&original);
        let converted = proto_account_meta_to_sdk(&proto).unwrap();

        assert_eq!(original.pubkey, converted.pubkey);
        assert_eq!(original.is_signer, converted.is_signer);
        assert_eq!(original.is_writable, converted.is_writable);
        assert!(!converted.is_writable);
        assert!(!converted.is_signer);
    }

    #[test]
    fn test_invalid_pubkey_error() {
        let invalid_proto = SolanaAccountMeta {
            pubkey: "invalid_pubkey".to_string(),
            is_signer: false,
            is_writable: false,
        };

        let result = proto_account_meta_to_sdk(&invalid_proto);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid pubkey"));
    }

    #[test]
    fn test_invalid_program_id_error() {
        let invalid_proto = SolanaInstruction {
            program_id: "invalid_program_id".to_string(),
            accounts: vec![],
            data: vec![],
            description: String::new(),
        };

        let result = proto_instruction_to_sdk(invalid_proto);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid program_id"));
    }
}
