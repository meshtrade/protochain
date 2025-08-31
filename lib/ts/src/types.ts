// ProtoSol Generated Types (would normally be generated from protobuf)
// Based on the proto definitions found in lib/proto/

export enum CommitmentLevel {
  COMMITMENT_LEVEL_UNSPECIFIED = 0,
  COMMITMENT_LEVEL_PROCESSED = 1,
  COMMITMENT_LEVEL_CONFIRMED = 2,
  COMMITMENT_LEVEL_FINALIZED = 3,
}

export enum TransactionState {
  TRANSACTION_STATE_UNSPECIFIED = 0,
  TRANSACTION_STATE_DRAFT = 1,
  TRANSACTION_STATE_COMPILED = 2,
  TRANSACTION_STATE_SIGNED = 3,
  TRANSACTION_STATE_SUBMITTED = 4,
}

export enum TransactionStatus {
  TRANSACTION_STATUS_UNSPECIFIED = 0,
  TRANSACTION_STATUS_RECEIVED = 1,
  TRANSACTION_STATUS_PROCESSED = 2,
  TRANSACTION_STATUS_CONFIRMED = 3,
  TRANSACTION_STATUS_FINALIZED = 4,
  TRANSACTION_STATUS_FAILED = 5,
  TRANSACTION_STATUS_DROPPED = 6,
  TRANSACTION_STATUS_TIMEOUT = 7,
}

export interface KeyPair {
  publicKey: string
  privateKey: string
}

export interface SolanaInstruction {
  programId: string
  accounts: AccountMeta[]
  data: Uint8Array
}

export interface AccountMeta {
  pubkey: string
  isSigner: boolean
  isWritable: boolean
}

export interface Transaction {
  instructions: SolanaInstruction[]
  signatures: string[]
  recentBlockhash: string
  feePayer: string
  state: TransactionState
}

export interface Account {
  address: string
  lamports: string
  owner: string
  executable: boolean
  rentEpoch: number
  data: Uint8Array
}

export interface MintInfo {
  mintAuthorityPubKey: string
  freezeAuthorityPubKey: string
  decimals: number
  supply: string
  isInitialized: boolean
}

// Request/Response types
export interface GetAccountRequest {
  address: string
  commitmentLevel?: CommitmentLevel
}

export interface GenerateNewKeyPairRequest {
  seed?: string
}

export interface GenerateNewKeyPairResponse {
  keyPair: KeyPair
}

export interface FundNativeRequest {
  address: string
  amount: string
  commitmentLevel?: CommitmentLevel
}

export interface FundNativeResponse {
  signature: string
}

export interface CreateRequest {
  payer: string
  newAccount: string
  owner: string
  lamports: number
  space: number
}

export interface TransferRequest {
  from: string
  to: string
  lamports: number
}

export interface InitialiseMintRequest {
  mintPubKey: string
  mintAuthorityPubKey: string
  freezeAuthorityPubKey: string
  decimals: number
}

export interface InitialiseMintResponse {
  instruction: SolanaInstruction
}

export interface GetCurrentMinRentForTokenAccountRequest {}

export interface GetCurrentMinRentForTokenAccountResponse {
  lamports: number
}

export interface ParseMintRequest {
  accountAddress: string
}

export interface ParseMintResponse {
  mint: MintInfo
}

export interface CompileTransactionRequest {
  transaction: Transaction
  feePayer: string
  recentBlockhash?: string
}

export interface CompileTransactionResponse {
  transaction: Transaction
}

export interface SignTransactionRequest {
  transaction: Transaction
  signingMethod: SignWithPrivateKeys | SignWithSeeds
}

export interface SignWithPrivateKeys {
  privateKeys: string[]
}

export interface SignWithSeeds {
  seeds: KeySeed[]
}

export interface KeySeed {
  seed: string
  passphrase?: string
}

export interface SignTransactionResponse {
  transaction: Transaction
}

export interface SubmitTransactionRequest {
  transaction: Transaction
  commitmentLevel?: CommitmentLevel
}

export enum SubmissionResult {
  SUBMISSION_RESULT_UNSPECIFIED = 0,
  SUBMISSION_RESULT_SUBMITTED = 1,
  SUBMISSION_RESULT_FAILED_VALIDATION = 2,
  SUBMISSION_RESULT_FAILED_NETWORK_ERROR = 3,
  SUBMISSION_RESULT_FAILED_INSUFFICIENT_FUNDS = 4,
  SUBMISSION_RESULT_FAILED_INVALID_SIGNATURE = 5,
}

export interface SubmitTransactionResponse {
  signature: string
  submissionResult: SubmissionResult
  errorMessage?: string
}

export interface GetTransactionRequest {
  signature: string
  commitmentLevel?: CommitmentLevel
}

export interface GetTransactionResponse {
  transaction: Transaction
}
