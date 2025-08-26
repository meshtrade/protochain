//! Protosol API Rust SDK
//! 
//! This crate provides Rust bindings for the Protosol API protocol buffers.
//! All code is auto-generated from protobuf definitions.

pub mod protosol {
    pub mod solana {
        pub mod transaction {
            pub mod v1 {
                include!("protosol.solana.transaction.v1.rs");
            }
        }
        pub mod account {
            pub mod v1 {
                include!("protosol.solana.account.v1.rs");
            }
        }
        pub mod program {
            pub mod system {
                pub mod v1 {
                    include!("protosol.solana.program.system.v1.rs");
                }
            }
        }
        pub mod r#type {
            pub mod v1 {
                include!("protosol.solana.type.v1.rs");
            }
        }
    }
}

// Re-export commonly used types at the crate root for convenience
pub use protosol::solana::transaction::v1::*;