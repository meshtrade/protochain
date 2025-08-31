// Token Program service client
import type {
  InitialiseMintRequest,
  InitialiseMintResponse,
  GetCurrentMinRentForTokenAccountRequest,
  GetCurrentMinRentForTokenAccountResponse,
  ParseMintRequest,
  ParseMintResponse,
  SolanaInstruction
} from '../types'

export class TokenProgramService {
  private endpoint: string
  private apiKey?: string

  constructor(endpoint: string, apiKey?: string) {
    this.endpoint = endpoint
    this.apiKey = apiKey
  }

  async InitialiseMint(request: InitialiseMintRequest): Promise<InitialiseMintResponse> {
    // TODO: Replace with actual gRPC call once backend is available
    console.log('🟡 TokenProgramService.InitialiseMint called:', request)

    // Mock instruction response for development
    const mockInstruction: SolanaInstruction = {
      programId: 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA', // SPL Token Program
      accounts: [
        { pubkey: request.mintPubKey, isSigner: false, isWritable: true },
        { pubkey: request.mintAuthorityPubKey, isSigner: false, isWritable: false },
        { pubkey: request.freezeAuthorityPubKey, isSigner: false, isWritable: false }
      ],
      data: new Uint8Array([0, request.decimals]) // Mock instruction data
    }

    const mockResponse: InitialiseMintResponse = {
      instruction: mockInstruction
    }

    return Promise.resolve(mockResponse)
  }

  async GetCurrentMinRentForTokenAccount(
    request?: GetCurrentMinRentForTokenAccountRequest
  ): Promise<GetCurrentMinRentForTokenAccountResponse> {
    // TODO: Replace with actual gRPC call once backend is available
    console.log('🟡 TokenProgramService.GetCurrentMinRentForTokenAccount called:', request)

    // Mock rent response for development
    const mockResponse: GetCurrentMinRentForTokenAccountResponse = {
      lamports: 1461600 // ~0.00146 SOL for mint account
    }

    return Promise.resolve(mockResponse)
  }

  async ParseMint(request: ParseMintRequest): Promise<ParseMintResponse> {
    // TODO: Replace with actual gRPC call once backend is available
    console.log('🟡 TokenProgramService.ParseMint called:', request)

    // Mock mint parsing response
    const mockResponse: ParseMintResponse = {
      mint: {
        mintAuthorityPubKey: '11111111111111111111111111111112', // Mock authority
        freezeAuthorityPubKey: '11111111111111111111111111111112', // Mock authority
        decimals: 2,
        supply: '0',
        isInitialized: false
      }
    }

    return Promise.resolve(mockResponse)
  }
}
