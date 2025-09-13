// @generated Protochain TypeScript SDK Index
// This file exports all generated services and types for clean imports

// Export SDK metadata
export const VERSION = '1.0.0';
export const SDK_NAME = 'Protochain TypeScript SDK';

// =============================================================================
// SERVICES
// =============================================================================

// Account Service
export { Service as AccountService } from './protochain/solana/account/v1/service_pb';
export type {
  GetAccountRequest,
  GenerateNewKeyPairRequest,
  GenerateNewKeyPairResponse,
  FundNativeRequest,
  FundNativeResponse,
} from './protochain/solana/account/v1/service_pb';

// Transaction Service
export { Service as TransactionService } from './protochain/solana/transaction/v1/service_pb';
export type {
  CompileTransactionRequest,
  CompileTransactionResponse,
  EstimateTransactionRequest,
  EstimateTransactionResponse,
  SimulateTransactionRequest,
  SimulateTransactionResponse,
  SignTransactionRequest,
  SignTransactionResponse,
  SubmitTransactionRequest,
  SubmitTransactionResponse,
  GetTransactionRequest,
  GetTransactionResponse,
  MonitorTransactionRequest,
  MonitorTransactionResponse,
} from './protochain/solana/transaction/v1/service_pb';

// RPC Client Service
export { Service as RPCClientService } from './protochain/solana/rpc_client/v1/service_pb';
export type {
  GetMinimumBalanceForRentExemptionRequest,
  GetMinimumBalanceForRentExemptionResponse,
} from './protochain/solana/rpc_client/v1/service_pb';

// System Program Service (returns SolanaInstruction for all methods)
export { Service as SystemProgramService } from './protochain/solana/program/system/v1/service_pb';
export type {
  CreateRequest,
  TransferRequest,
  AllocateRequest,
  AssignRequest,
  CreateWithSeedRequest,
  AllocateWithSeedRequest,
  AssignWithSeedRequest,
  TransferWithSeedRequest,
  AdvanceNonceAccountRequest,
  WithdrawNonceAccountRequest,
  InitializeNonceAccountRequest,
  AuthorizeNonceAccountRequest,
  UpgradeNonceAccountRequest,
} from './protochain/solana/program/system/v1/service_pb';

// Token Program Service
export { Service as TokenProgramService } from './protochain/solana/program/token/v1/service_pb';
export type {
  InitialiseMintRequest,
  InitialiseMintResponse,
  GetCurrentMinRentForTokenAccountRequest,
  GetCurrentMinRentForTokenAccountResponse,
  ParseMintRequest,
  ParseMintResponse,
} from './protochain/solana/program/token/v1/service_pb';

// =============================================================================
// CORE TYPES
// =============================================================================

// Account types
export type { Account as AccountSchema } from './protochain/solana/account/v1/account_pb';

// Transaction types
export type {
  TransactionConfig,
  Transaction,
} from './protochain/solana/transaction/v1/transaction_pb';

// Instruction types
export type {
  SolanaInstruction,
  SolanaAccountMeta,
} from './protochain/solana/transaction/v1/instruction_pb';

// Common types
export type { KeyPair } from './protochain/solana/type/v1/keypair_pb';

export type { CommitmentLevel } from './protochain/solana/type/v1/commitment_level_pb';

// =============================================================================
// RE-EXPORTS FOR CONNECT USAGE
// =============================================================================

// Re-export connect types that consumers will need
export { createClient } from '@connectrpc/connect';
export { createGrpcTransport } from '@connectrpc/connect-node';
export { createGrpcWebTransport } from '@connectrpc/connect-web';
export type { Transport } from '@connectrpc/connect';
