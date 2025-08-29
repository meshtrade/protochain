use super::SystemProgramServiceImpl;
use protosol_api::protosol::solana::program::system::v1::{
    service_server::Service as SystemProgramService,
    CreateRequest,
    TransferRequest,
    AllocateRequest,
    AssignRequest,
    CreateWithSeedRequest,
};
use tonic::{Request, Status};

/// Creates a test service instance
/// Note: Tests focus on validation logic - RPC calls will fail in test environment
fn create_test_service() -> SystemProgramServiceImpl {
    SystemProgramServiceImpl::new()
}

/// Helper to check if error is a validation error (vs RPC error)
fn is_validation_error(status: &Status) -> bool {
    matches!(status.code(), tonic::Code::InvalidArgument)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_request_validation() {
    let service = create_test_service();
    
    // Valid test pubkey constants (actual Solana public keys)
    const VALID_PUBKEY: &str = "11111111111111111111111111111112"; // System Program
    const ANOTHER_VALID_PUBKEY: &str = "SysvarS1otHashes111111111111111111111111111"; // Slot Hashes Sysvar
    const INVALID_PUBKEY: &str = "invalid_not_base58!!!";
    
    struct TestCase {
        name: &'static str,
        payer: &'static str,
        new_account: &'static str,
        lamports: u64,
        space: u64,
        expect_validation_error: bool,
        error_contains: &'static str,
    }
    
    let test_cases = vec![
        TestCase {
            name: "valid request - will fail on RPC but pass validation",
            payer: VALID_PUBKEY,
            new_account: ANOTHER_VALID_PUBKEY,
            lamports: 1000000,
            space: 100,
            expect_validation_error: false,
            error_contains: "",
        },
        TestCase {
            name: "empty payer",
            payer: "",
            new_account: VALID_PUBKEY,
            lamports: 1000000,
            space: 100,
            expect_validation_error: true,
            error_contains: "Payer address is required",
        },
        TestCase {
            name: "empty new_account",
            payer: VALID_PUBKEY,
            new_account: "",
            lamports: 1000000,
            space: 100,
            expect_validation_error: true,
            error_contains: "New account address is required",
        },
        TestCase {
            name: "invalid payer pubkey",
            payer: INVALID_PUBKEY,
            new_account: VALID_PUBKEY,
            lamports: 1000000,
            space: 100,
            expect_validation_error: true,
            error_contains: "Invalid payer address",
        },
        TestCase {
            name: "invalid new_account pubkey",
            payer: VALID_PUBKEY,
            new_account: INVALID_PUBKEY,
            lamports: 1000000,
            space: 100,
            expect_validation_error: true,
            error_contains: "Invalid new account address",
        },
        TestCase {
            name: "zero lamports allowed",
            payer: VALID_PUBKEY,
            new_account: ANOTHER_VALID_PUBKEY,
            lamports: 0,
            space: 100,
            expect_validation_error: false,
            error_contains: "",
        },
        TestCase {
            name: "zero space allowed",
            payer: VALID_PUBKEY,
            new_account: ANOTHER_VALID_PUBKEY,
            lamports: 1000000,
            space: 0,
            expect_validation_error: false,
            error_contains: "",
        },
    ];
    
    for test_case in test_cases {
        let request = Request::new(CreateRequest {
            payer: test_case.payer.to_string(),
            new_account: test_case.new_account.to_string(),
            lamports: test_case.lamports,
            space: test_case.space,
        });
        
        let result = service.create(request).await;
        
        if test_case.expect_validation_error {
            // Should fail with validation error
            assert!(result.is_err(), "Test '{}' expected validation error but got success", test_case.name);
            let error = result.unwrap_err();
            assert!(
                is_validation_error(&error),
                "Test '{}' expected validation error but got different error type: {:?}",
                test_case.name, error.code()
            );
            assert!(
                error.message().contains(test_case.error_contains),
                "Test '{}' expected error containing '{}' but got '{}'",
                test_case.name,
                test_case.error_contains,
                error.message()
            );
        } else {
            // Should pass validation but may fail on RPC (which is expected in test environment)
            if result.is_err() {
                let error = result.unwrap_err();
                assert!(
                    !is_validation_error(&error),
                    "Test '{}' should pass validation but got validation error: {}",
                    test_case.name, error.message()
                );
                // RPC errors are expected in test environment - that's fine
            }
            // If it succeeds, that's also fine (though unlikely without running validator)
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_transfer_request_validation() {
    let service = create_test_service();
    
    const VALID_PUBKEY: &str = "11111111111111111111111111111112"; // System Program
    const ANOTHER_VALID_PUBKEY: &str = "SysvarS1otHashes111111111111111111111111111"; // Slot Hashes Sysvar
    const INVALID_PUBKEY: &str = "invalid_not_base58!!!";
    
    struct TestCase {
        name: &'static str,
        from: &'static str,
        to: &'static str,
        lamports: u64,
        expect_validation_error: bool,
        error_contains: &'static str,
    }
    
    let test_cases = vec![
        TestCase {
            name: "valid request - will fail on RPC but pass validation",
            from: VALID_PUBKEY,
            to: ANOTHER_VALID_PUBKEY,
            lamports: 1000000,
            expect_validation_error: false,
            error_contains: "",
        },
        TestCase {
            name: "empty from",
            from: "",
            to: VALID_PUBKEY,
            lamports: 1000000,
            expect_validation_error: true,
            error_contains: "From address is required",
        },
        TestCase {
            name: "empty to",
            from: VALID_PUBKEY,
            to: "",
            lamports: 1000000,
            expect_validation_error: true,
            error_contains: "To address is required",
        },
        TestCase {
            name: "invalid from pubkey",
            from: INVALID_PUBKEY,
            to: VALID_PUBKEY,
            lamports: 1000000,
            expect_validation_error: true,
            error_contains: "Invalid from address",
        },
        TestCase {
            name: "invalid to pubkey",
            from: VALID_PUBKEY,
            to: INVALID_PUBKEY,
            lamports: 1000000,
            expect_validation_error: true,
            error_contains: "Invalid to address",
        },
        TestCase {
            name: "zero lamports allowed",
            from: VALID_PUBKEY,
            to: ANOTHER_VALID_PUBKEY,
            lamports: 0,
            expect_validation_error: false,
            error_contains: "",
        },
        TestCase {
            name: "same from and to allowed",
            from: VALID_PUBKEY,
            to: VALID_PUBKEY,
            lamports: 1000000,
            expect_validation_error: false,
            error_contains: "",
        },
    ];
    
    for test_case in test_cases {
        let request = Request::new(TransferRequest {
            from: test_case.from.to_string(),
            to: test_case.to.to_string(),
            lamports: test_case.lamports,
        });
        
        let result = service.transfer(request).await;
        
        if test_case.expect_validation_error {
            // Should fail with validation error
            assert!(result.is_err(), "Test '{}' expected validation error but got success", test_case.name);
            let error = result.unwrap_err();
            assert!(
                is_validation_error(&error),
                "Test '{}' expected validation error but got different error type: {:?}",
                test_case.name, error.code()
            );
            assert!(
                error.message().contains(test_case.error_contains),
                "Test '{}' expected error containing '{}' but got '{}'",
                test_case.name,
                test_case.error_contains,
                error.message()
            );
        } else {
            // Should pass validation but may fail on RPC (which is expected in test environment)
            if result.is_err() {
                let error = result.unwrap_err();
                assert!(
                    !is_validation_error(&error),
                    "Test '{}' should pass validation but got validation error: {}",
                    test_case.name, error.message()
                );
                // RPC errors are expected in test environment - that's fine
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_allocate_request_validation() {
    let service = create_test_service();
    
    const VALID_PUBKEY: &str = "11111111111111111111111111111112"; // System Program
    const INVALID_PUBKEY: &str = "invalid_not_base58!!!";
    
    struct TestCase {
        name: &'static str,
        account: &'static str,
        space: u64,
        expect_validation_error: bool,
        error_contains: &'static str,
    }
    
    let test_cases = vec![
        TestCase {
            name: "valid request - will fail on RPC but pass validation",
            account: VALID_PUBKEY,
            space: 100,
            expect_validation_error: false,
            error_contains: "",
        },
        TestCase {
            name: "empty account",
            account: "",
            space: 100,
            expect_validation_error: true,
            error_contains: "Account address is required",
        },
        TestCase {
            name: "invalid account pubkey",
            account: INVALID_PUBKEY,
            space: 100,
            expect_validation_error: true,
            error_contains: "Invalid account address",
        },
        TestCase {
            name: "zero space allowed",
            account: VALID_PUBKEY,
            space: 0,
            expect_validation_error: false,
            error_contains: "",
        },
        TestCase {
            name: "large space allowed",
            account: VALID_PUBKEY,
            space: 1_000_000,
            expect_validation_error: false,
            error_contains: "",
        },
    ];
    
    for test_case in test_cases {
        let request = Request::new(AllocateRequest {
            account: test_case.account.to_string(),
            space: test_case.space,
        });
        
        let result = service.allocate(request).await;
        
        if test_case.expect_validation_error {
            // Should fail with validation error
            assert!(result.is_err(), "Test '{}' expected validation error but got success", test_case.name);
            let error = result.unwrap_err();
            assert!(
                is_validation_error(&error),
                "Test '{}' expected validation error but got different error type: {:?}",
                test_case.name, error.code()
            );
            assert!(
                error.message().contains(test_case.error_contains),
                "Test '{}' expected error containing '{}' but got '{}'",
                test_case.name,
                test_case.error_contains,
                error.message()
            );
        } else {
            // Should pass validation but may fail on RPC (which is expected in test environment)
            if result.is_err() {
                let error = result.unwrap_err();
                assert!(
                    !is_validation_error(&error),
                    "Test '{}' should pass validation but got validation error: {}",
                    test_case.name, error.message()
                );
                // RPC errors are expected in test environment - that's fine
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_assign_request_validation() {
    let service = create_test_service();
    
    const VALID_PUBKEY: &str = "11111111111111111111111111111112"; // System Program
    const ANOTHER_VALID_PUBKEY: &str = "SysvarS1otHashes111111111111111111111111111"; // Slot Hashes Sysvar
    const INVALID_PUBKEY: &str = "invalid_not_base58!!!";
    
    struct TestCase {
        name: &'static str,
        account: &'static str,
        owner_program: &'static str,
        expect_validation_error: bool,
        error_contains: &'static str,
    }
    
    let test_cases = vec![
        TestCase {
            name: "valid request - will fail on RPC but pass validation",
            account: VALID_PUBKEY,
            owner_program: ANOTHER_VALID_PUBKEY,
            expect_validation_error: false,
            error_contains: "",
        },
        TestCase {
            name: "empty account",
            account: "",
            owner_program: VALID_PUBKEY,
            expect_validation_error: true,
            error_contains: "Account address is required",
        },
        TestCase {
            name: "empty owner_program",
            account: VALID_PUBKEY,
            owner_program: "",
            expect_validation_error: true,
            error_contains: "Owner program is required",
        },
        TestCase {
            name: "invalid account pubkey",
            account: INVALID_PUBKEY,
            owner_program: VALID_PUBKEY,
            expect_validation_error: true,
            error_contains: "Invalid account address",
        },
        TestCase {
            name: "invalid owner_program pubkey",
            account: VALID_PUBKEY,
            owner_program: INVALID_PUBKEY,
            expect_validation_error: true,
            error_contains: "Invalid owner program address",
        },
        TestCase {
            name: "same account and owner allowed",
            account: VALID_PUBKEY,
            owner_program: VALID_PUBKEY,
            expect_validation_error: false,
            error_contains: "",
        },
    ];
    
    for test_case in test_cases {
        let request = Request::new(AssignRequest {
            account: test_case.account.to_string(),
            owner_program: test_case.owner_program.to_string(),
        });
        
        let result = service.assign(request).await;
        
        if test_case.expect_validation_error {
            // Should fail with validation error
            assert!(result.is_err(), "Test '{}' expected validation error but got success", test_case.name);
            let error = result.unwrap_err();
            assert!(
                is_validation_error(&error),
                "Test '{}' expected validation error but got different error type: {:?}",
                test_case.name, error.code()
            );
            assert!(
                error.message().contains(test_case.error_contains),
                "Test '{}' expected error containing '{}' but got '{}'",
                test_case.name,
                test_case.error_contains,
                error.message()
            );
        } else {
            // Should pass validation but may fail on RPC (which is expected in test environment)
            if result.is_err() {
                let error = result.unwrap_err();
                assert!(
                    !is_validation_error(&error),
                    "Test '{}' should pass validation but got validation error: {}",
                    test_case.name, error.message()
                );
                // RPC errors are expected in test environment - that's fine
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_with_seed_request_validation() {
    let service = create_test_service();
    
    const VALID_PUBKEY: &str = "11111111111111111111111111111112"; // System Program
    const ANOTHER_VALID_PUBKEY: &str = "SysvarS1otHashes111111111111111111111111111"; // Slot Hashes Sysvar
    const THIRD_VALID_PUBKEY: &str = "SysvarC1ock11111111111111111111111111111111"; // Clock Sysvar
    const INVALID_PUBKEY: &str = "invalid_not_base58!!!";
    
    struct TestCase {
        name: &'static str,
        payer: &'static str,
        new_account: &'static str,
        base: &'static str,
        seed: &'static str,
        lamports: u64,
        space: u64,
        expect_validation_error: bool,
        error_contains: &'static str,
    }
    
    let test_cases = vec![
        TestCase {
            name: "valid request - will fail on RPC but pass validation",
            payer: VALID_PUBKEY,
            new_account: ANOTHER_VALID_PUBKEY,
            base: THIRD_VALID_PUBKEY,
            seed: "my-seed",
            lamports: 1000000,
            space: 100,
            expect_validation_error: false,
            error_contains: "",
        },
        TestCase {
            name: "empty payer",
            payer: "",
            new_account: VALID_PUBKEY,
            base: ANOTHER_VALID_PUBKEY,
            seed: "my-seed",
            lamports: 1000000,
            space: 100,
            expect_validation_error: true,
            error_contains: "Payer address is required",
        },
        TestCase {
            name: "empty new_account",
            payer: VALID_PUBKEY,
            new_account: "",
            base: ANOTHER_VALID_PUBKEY,
            seed: "my-seed",
            lamports: 1000000,
            space: 100,
            expect_validation_error: true,
            error_contains: "New account address is required",
        },
        TestCase {
            name: "empty base",
            payer: VALID_PUBKEY,
            new_account: ANOTHER_VALID_PUBKEY,
            base: "",
            seed: "my-seed",
            lamports: 1000000,
            space: 100,
            expect_validation_error: true,
            error_contains: "Base address is required",
        },
        TestCase {
            name: "empty seed",
            payer: VALID_PUBKEY,
            new_account: ANOTHER_VALID_PUBKEY,
            base: THIRD_VALID_PUBKEY,
            seed: "",
            lamports: 1000000,
            space: 100,
            expect_validation_error: true,
            error_contains: "Seed is required",
        },
        TestCase {
            name: "invalid payer pubkey",
            payer: INVALID_PUBKEY,
            new_account: VALID_PUBKEY,
            base: ANOTHER_VALID_PUBKEY,
            seed: "my-seed",
            lamports: 1000000,
            space: 100,
            expect_validation_error: true,
            error_contains: "Invalid payer address",
        },
        TestCase {
            name: "invalid new_account pubkey",
            payer: VALID_PUBKEY,
            new_account: INVALID_PUBKEY,
            base: ANOTHER_VALID_PUBKEY,
            seed: "my-seed",
            lamports: 1000000,
            space: 100,
            expect_validation_error: true,
            error_contains: "Invalid new account address",
        },
        TestCase {
            name: "invalid base pubkey",
            payer: VALID_PUBKEY,
            new_account: ANOTHER_VALID_PUBKEY,
            base: INVALID_PUBKEY,
            seed: "my-seed",
            lamports: 1000000,
            space: 100,
            expect_validation_error: true,
            error_contains: "Invalid base address",
        },
        TestCase {
            name: "zero lamports allowed",
            payer: VALID_PUBKEY,
            new_account: ANOTHER_VALID_PUBKEY,
            base: THIRD_VALID_PUBKEY,
            seed: "my-seed",
            lamports: 0,
            space: 100,
            expect_validation_error: false,
            error_contains: "",
        },
        TestCase {
            name: "zero space allowed",
            payer: VALID_PUBKEY,
            new_account: ANOTHER_VALID_PUBKEY,
            base: THIRD_VALID_PUBKEY,
            seed: "my-seed",
            lamports: 1000000,
            space: 0,
            expect_validation_error: false,
            error_contains: "",
        },
        TestCase {
            name: "long seed allowed",
            payer: VALID_PUBKEY,
            new_account: ANOTHER_VALID_PUBKEY,
            base: THIRD_VALID_PUBKEY,
            seed: "this-is-a-very-long-seed-string-that-should-still-be-valid",
            lamports: 1000000,
            space: 100,
            expect_validation_error: false,
            error_contains: "",
        },
    ];
    
    for test_case in test_cases {
        let request = Request::new(CreateWithSeedRequest {
            payer: test_case.payer.to_string(),
            new_account: test_case.new_account.to_string(),
            base: test_case.base.to_string(),
            seed: test_case.seed.to_string(),
            lamports: test_case.lamports,
            space: test_case.space,
        });
        
        let result = service.create_with_seed(request).await;
        
        if test_case.expect_validation_error {
            // Should fail with validation error
            assert!(result.is_err(), "Test '{}' expected validation error but got success", test_case.name);
            let error = result.unwrap_err();
            assert!(
                is_validation_error(&error),
                "Test '{}' expected validation error but got different error type: {:?}",
                test_case.name, error.code()
            );
            assert!(
                error.message().contains(test_case.error_contains),
                "Test '{}' expected error containing '{}' but got '{}'",
                test_case.name,
                test_case.error_contains,
                error.message()
            );
        } else {
            // Should pass validation but may fail on RPC (which is expected in test environment)
            if result.is_err() {
                let error = result.unwrap_err();
                assert!(
                    !is_validation_error(&error),
                    "Test '{}' should pass validation but got validation error: {}",
                    test_case.name, error.message()
                );
                // RPC errors are expected in test environment - that's fine
            }
        }
    }
}