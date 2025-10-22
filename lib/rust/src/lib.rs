//! Protochain API Rust SDK
//!
//! This crate provides Rust bindings for the Protochain API protocol buffers.
//! All code is auto-generated from protobuf definitions using tonic-build.

// Generated modules from build.rs
pub mod protochain {
    pub mod solana {
        pub mod account {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/protochain.solana.account.v1.rs"));
            }
        }
        pub mod transaction {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/protochain.solana.transaction.v1.rs"));
            }
        }
        pub mod program {
            pub mod system {
                pub mod v1 {
                    include!(concat!(env!("OUT_DIR"), "/protochain.solana.program.system.v1.rs"));
                }
            }
            pub mod token {
                pub mod v1 {
                    include!(concat!(env!("OUT_DIR"), "/protochain.solana.program.token.v1.rs"));
                }
            }
        }
        pub mod r#type {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/protochain.solana.r#type.v1.rs"));
            }
        }
        pub mod rpc_client {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/protochain.solana.rpc_client.v1.rs"));
            }
        }
    }
}

// Re-export commonly used types at the crate root for convenience
pub use protochain::solana::transaction::v1::*;
