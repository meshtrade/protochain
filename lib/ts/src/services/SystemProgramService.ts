// System Program service client
import type {
  CreateRequest,
  TransferRequest,
  SolanaInstruction
} from '../types'

export class SystemProgramService {
  private endpoint: string
  private apiKey?: string

  constructor(endpoint: string, apiKey?: string) {
    this.endpoint = endpoint
    this.apiKey = apiKey
  }

  async Create(request: CreateRequest): Promise<SolanaInstruction> {
    // TODO: Replace with actual gRPC call once backend is available
    console.log('🟡 SystemProgramService.Create called:', request)

    // Mock create instruction response for development
    const mockInstruction: SolanaInstruction = {
      programId: '11111111111111111111111111111112', // System Program
      accounts: [
        { pubkey: request.payer, isSigner: true, isWritable: true },
        { pubkey: request.newAccount, isSigner: true, isWritable: true }
      ],
      data: new Uint8Array([
        0, // Create instruction index
        ...Array.from(new Uint8Array(4)), // Mock lamports (4 bytes)
        ...Array.from(new Uint8Array(4)), // Mock space (4 bytes)
        // Mock owner public key bytes (32 bytes for Solana pubkey)
        ...Array.from(new Uint8Array(32)) // Would be actual pubkey bytes from base58 decode
      ])
    }

    return Promise.resolve(mockInstruction)
  }

  async Transfer(request: TransferRequest): Promise<SolanaInstruction> {
    // TODO: Replace with actual gRPC call once backend is available
    console.log('🟡 SystemProgramService.Transfer called:', request)

    // Mock transfer instruction response for development
    const mockInstruction: SolanaInstruction = {
      programId: '11111111111111111111111111111112', // System Program
      accounts: [
        { pubkey: request.from, isSigner: true, isWritable: true },
        { pubkey: request.to, isSigner: false, isWritable: true }
      ],
      data: new Uint8Array([
        2, // Transfer instruction index
        ...new Uint8Array(new Uint32Array([request.lamports]).buffer)
      ])
    }

    return Promise.resolve(mockInstruction)
  }
}
