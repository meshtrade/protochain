'use server'

import { tokenProgramClient } from '../grpc-clients'

// Server action for Token Program InitialiseMint operation (existing)
export async function initialiseMintAction(formData: FormData) {
  try {
    const mintPubKey = formData.get('mintPubKey') as string
    const mintAuthorityPubKey = formData.get('mintAuthorityPubKey') as string
    const freezeAuthorityPubKey = formData.get('freezeAuthorityPubKey') as string
    const decimals = formData.get('decimals') as string

    if (!mintPubKey) return { error: 'mintPubKey is required' }
    if (!mintAuthorityPubKey) return { error: 'mintAuthorityPubKey is required' }
    if (!decimals) return { error: 'decimals is required' }

    let decimalsNum: number
    try {
      decimalsNum = parseInt(decimals)
      if (decimalsNum < 0 || decimalsNum > 9) {
        return { error: 'decimals must be between 0 and 9' }
      }
    } catch {
      return { error: 'decimals must be a valid number' }
    }

    const grpcRequest = {
      mintPubKey,
      mintAuthorityPubKey,
      freezeAuthorityPubKey: freezeAuthorityPubKey || undefined,
      decimals: decimalsNum,
    }

    const client = tokenProgramClient()
    const response = await client.initialiseMint(grpcRequest)

    return {
      success: true,
      instruction: response.instruction,
      operation: 'initialiseMint'
    }

  } catch (error: any) {
    console.error('InitialiseMint server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Token program initialiseMint operation failed'
    }
  }
}

// Server action for Token Program GetCurrentMinRentForTokenAccount operation
export async function getCurrentMinRentForTokenAccountAction() {
  try {
    const grpcRequest = {}

    const client = tokenProgramClient()
    const response = await client.getCurrentMinRentForTokenAccount(grpcRequest)

    return {
      success: true,
      lamports: response.lamports?.toString() || '0',
      operation: 'getCurrentMinRentForTokenAccount'
    }

  } catch (error: any) {
    console.error('GetCurrentMinRentForTokenAccount server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Token program getCurrentMinRentForTokenAccount operation failed'
    }
  }
}

// Server action for Token Program ParseMint operation
export async function parseMintAction(formData: FormData) {
  try {
    const accountAddress = formData.get('accountAddress') as string

    if (!accountAddress) return { error: 'accountAddress is required' }

    const grpcRequest = {
      accountAddress,
    }

    const client = tokenProgramClient()
    const response = await client.parseMint(grpcRequest)

    return {
      success: true,
      mint: response.mint,
      operation: 'parseMint'
    }

  } catch (error: any) {
    console.error('ParseMint server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Token program parseMint operation failed'
    }
  }
}

// NOTE: Additional token program methods from proto are not yet fully implemented in backend
// For now, focusing on the core working methods. Additional methods will be added as backend support is completed.