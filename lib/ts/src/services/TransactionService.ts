// Transaction service client
import type {
  CompileTransactionRequest,
  CompileTransactionResponse,
  SignTransactionRequest,
  SignTransactionResponse,
  SubmitTransactionRequest,
  SubmitTransactionResponse,
  GetTransactionRequest,
  GetTransactionResponse,
  Transaction
} from '../types'
import { SubmissionResult } from '../types'

export class TransactionService {
  private endpoint: string
  private apiKey?: string

  constructor(endpoint: string, apiKey?: string) {
    this.endpoint = endpoint
    this.apiKey = apiKey
  }

  async CompileTransaction(request: CompileTransactionRequest): Promise<CompileTransactionResponse> {
    // TODO: Replace with actual gRPC call once backend is available
    console.log('🟡 TransactionService.CompileTransaction called:', request)

    // Mock compilation response for development
    const compiledTransaction: Transaction = {
      ...request.transaction,
      recentBlockhash: request.recentBlockhash || 'ABCDEFGH1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890ABCD',
      feePayer: request.feePayer,
      state: 2 // TRANSACTION_STATE_COMPILED
    }

    const mockResponse: CompileTransactionResponse = {
      transaction: compiledTransaction
    }

    return Promise.resolve(mockResponse)
  }

  async SignTransaction(request: SignTransactionRequest): Promise<SignTransactionResponse> {
    // TODO: Replace with actual gRPC call once backend is available
    console.log('🟡 TransactionService.SignTransaction called:', request)

    // Mock signing response for development
    const signedTransaction: Transaction = {
      ...request.transaction,
      signatures: request.transaction.signatures || ['mock_signature_would_be_64_bytes'],
      state: 3 // TRANSACTION_STATE_SIGNED
    }

    const mockResponse: SignTransactionResponse = {
      transaction: signedTransaction
    }

    return Promise.resolve(mockResponse)
  }

  async SubmitTransaction(request: SubmitTransactionRequest): Promise<SubmitTransactionResponse> {
    // TODO: Replace with actual gRPC call once backend is available
    console.log('🟡 TransactionService.SubmitTransaction called:', request)

    // Mock submission response for development
    const mockResponse: SubmitTransactionResponse = {
      signature: 'ABCDEFGH1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890ABCDEFGH',
      submissionResult: SubmissionResult.SUBMISSION_RESULT_SUBMITTED
    }

    return Promise.resolve(mockResponse)
  }

  async GetTransaction(request: GetTransactionRequest): Promise<GetTransactionResponse> {
    // TODO: Replace with actual gRPC call once backend is available
    console.log('🟡 TransactionService.GetTransaction called:', request)

    // Mock get transaction response
    const mockTransaction: Transaction = {
      instructions: [],
      signatures: [request.signature],
      recentBlockhash: 'ABCDEFGH1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890ABCD',
      feePayer: '11111111111111111111111111111112',
      state: 4 // TRANSACTION_STATE_SUBMITTED
    }

    const mockResponse: GetTransactionResponse = {
      transaction: mockTransaction
    }

    return Promise.resolve(mockResponse)
  }
}
