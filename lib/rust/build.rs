use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get workspace root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let workspace_root = PathBuf::from(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let proto_dir = workspace_root.join("lib/proto");

    // Tell cargo to recompile if proto files change
    println!("cargo:rerun-if-changed={}", proto_dir.display());

    // All proto files needed for compilation
    let proto_files = vec![
        "protochain/solana/account/v1/account.proto",
        "protochain/solana/account/v1/service.proto",
        "protochain/solana/transaction/v1/instruction.proto",
        "protochain/solana/transaction/v1/transaction.proto",
        "protochain/solana/transaction/v1/service.proto",
        "protochain/solana/program/system/v1/service.proto",
        "protochain/solana/program/token/v1/service.proto",
        "protochain/solana/rpc_client/v1/service.proto",
        "protochain/solana/type/v1/commitment_level.proto",
        "protochain/solana/type/v1/keypair.proto",
    ];

    // Use tonic_prost_build to compile protos with tonic 0.14 support
    tonic_prost_build::configure().compile_protos(&proto_files, &[proto_dir.to_str().unwrap()])?;

    Ok(())
}
