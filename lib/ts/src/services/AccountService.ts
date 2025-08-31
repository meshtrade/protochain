// Account service client (would normally use generated protobuf client)
import type {
  GetAccountRequest,
  Account,
  GenerateNewKeyPairRequest,
  GenerateNewKeyPairResponse,
  FundNativeRequest,
  FundNativeResponse
} from '../types'

export class AccountService {
  private endpoint: string
  private apiKey?: string

  constructor(endpoint: string, apiKey?: string) {
    this.endpoint = endpoint
    this.apiKey = apiKey
  }

  async GetAccount(request: GetAccountRequest): Promise<Account> {
    // TODO: Replace with actual gRPC call once backend is available
    console.log('🟡 AccountService.GetAccount called:', request)

    // Mock response for development
    const mockResponse: Account = {
      address: request.address,
      lamports: '1000000000', // 1 SOL
      owner: '11111111111111111111111111111112', // System Program
      executable: false,
      rentEpoch: 0,
      data: new Uint8Array([0, 0, 0, 0]) // No data
    }

    return Promise.resolve(mockResponse)
  }

  async GenerateNewKeyPair(request?: GenerateNewKeyPairRequest): Promise<GenerateNewKeyPairResponse> {
    // TODO: Replace with actual gRPC call once backend is available
    console.log('🟡 AccountService.GenerateNewKeyPair called:', request)

    // Mock keypair response
    const mockResponse: GenerateNewKeyPairResponse = {
      keyPair: {
        publicKey: '11111111111111111111111111111112', // Mock public key
        privateKey: 'mock_private_key_would_be_64_bytes'
      }
    }

    return Promise.resolve(mockResponse)
  }

  async FundNative(request: FundNativeRequest): Promise<FundNativeResponse> {
    // TODO: Replace with actual gRPC call once backend is available
    console.log('🟡 AccountService.FundNative called:', request)

    // Mock funding response
    const mockResponse: FundNativeResponse = {
      signature: 'mock_transaction_signature_funding'
    }

    return Promise.resolve(mockResponse)
  }
}
