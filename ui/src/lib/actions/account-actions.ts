'use server'

import { accountClient } from '../grpc-clients'

// CommitmentLevel enum values (based on existing patterns in the codebase)
type CommitmentLevel = 1 | 2 | 3

// Helper function to map commitment level from string to enum
function mapCommitmentLevel(level?: string): CommitmentLevel {
  if (!level) return 2 // Default to CONFIRMED
  
  const commitmentMap: Record<string, CommitmentLevel> = {
    'processed': 1,
    'confirmed': 2,
    'finalized': 3,
  }
  
  const mappedLevel = commitmentMap[level.toLowerCase()]
  return mappedLevel !== undefined ? mappedLevel : 2
}

// Server action for getting account information
export async function getAccountAction(formData: FormData) {
  try {
    const address = formData.get('address') as string
    const commitmentLevel = mapCommitmentLevel(formData.get('commitmentLevel') as string)

    if (!address) {
      return { error: 'Address is required' }
    }

    // Build gRPC request (following existing pattern)
    const grpcRequest: any = {
      address,
      commitmentLevel
    }

    // Call Protochain backend through gRPC directly from server action
    const client = accountClient()
    const response = await client.getAccount(grpcRequest)

    // Return the account data
    return {
      success: true,
      address: response.address,
      lamports: response.lamports,
      owner: response.owner,
      executable: response.executable,
      rentEpoch: response.rentEpoch,
      data: response.data, // Uint8Array will be JSON serialized as array
    }

  } catch (error: any) {
    console.error('GetAccount server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Account lookup failed'
    }
  }
}

// Server action for generating new keypairs
export async function generateNewKeyPairAction() {
  try {
    // Build gRPC request (no parameters needed for basic generation)
    const grpcRequest: any = {}

    // Call Protochain backend through gRPC directly from server action
    const client = accountClient()
    const response = await client.generateNewKeyPair(grpcRequest)

    return {
      success: true,
      keyPair: {
        publicKey: response.keyPair?.publicKey,
        privateKey: response.keyPair?.privateKey
      }
    }

  } catch (error: any) {
    console.error('GenerateNewKeyPair server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Keypair generation failed'
    }
  }
}

// Server action for funding native accounts (airdrop)
export async function fundNativeAction(formData: FormData) {
  try {
    const address = formData.get('address') as string
    const amount = formData.get('amount') as string
    const commitmentLevel = mapCommitmentLevel(formData.get('commitmentLevel') as string)

    if (!address) {
      return { error: 'Address is required' }
    }

    if (!amount) {
      return { error: 'Amount is required' }
    }

    // Build gRPC request
    const grpcRequest: any = {
      address,
      amount,
      commitmentLevel
    }

    // Call Protochain backend through gRPC directly from server action
    const client = accountClient()
    const response = await client.fundNative(grpcRequest)

    return {
      success: true,
      signature: response.signature,
      message: 'Native funding completed successfully'
    }

  } catch (error: any) {
    console.error('FundNative server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Native funding failed'
    }
  }
}