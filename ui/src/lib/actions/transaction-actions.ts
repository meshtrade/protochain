'use server'

import { transactionClient } from '../grpc-clients'

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

// Server action for compiling transactions
export async function compileTransactionAction(formData: FormData) {
  try {
    const transaction = JSON.parse(formData.get('transaction') as string)
    const feePayer = formData.get('feePayer') as string
    const recentBlockhash = formData.get('recentBlockhash') as string

    if (!transaction) {
      return { error: 'Transaction is required' }
    }

    if (!feePayer) {
      return { error: 'Fee payer is required' }
    }

    // Build gRPC request (following existing pattern)
    const grpcRequest: any = {
      transaction,
      feePayer,
      recentBlockhash: recentBlockhash || undefined
    }

    // Call ProtoSol backend through gRPC directly from server action
    const client = transactionClient()
    const response = await client.compileTransaction(grpcRequest)

    return {
      success: true,
      transaction: response.transaction || {},
      note: 'Transaction compiled successfully'
    }

  } catch (error: any) {
    console.error('CompileTransaction server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Transaction compilation failed'
    }
  }
}

// Server action for estimating transactions
export async function estimateTransactionAction(formData: FormData) {
  try {
    const transaction = JSON.parse(formData.get('transaction') as string)
    const commitmentLevel = mapCommitmentLevel(formData.get('commitmentLevel') as string)

    if (!transaction) {
      return { error: 'Transaction is required' }
    }

    // Build gRPC request
    const grpcRequest: any = {
      transaction,
      commitmentLevel
    }

    // Call ProtoSol backend through gRPC directly from server action
    const client = transactionClient()
    const response = await client.estimateTransaction(grpcRequest)

    return {
      success: true,
      computeUnits: response.computeUnits?.toString(),
      feeLamports: response.feeLamports?.toString(),
      priorityFee: response.priorityFee?.toString()
    }

  } catch (error: any) {
    console.error('EstimateTransaction server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Transaction estimation failed'
    }
  }
}

// Server action for simulating transactions
export async function simulateTransactionAction(formData: FormData) {
  try {
    const transaction = JSON.parse(formData.get('transaction') as string)
    const commitmentLevel = mapCommitmentLevel(formData.get('commitmentLevel') as string)

    if (!transaction) {
      return { error: 'Transaction is required' }
    }

    // Build gRPC request
    const grpcRequest: any = {
      transaction,
      commitmentLevel
    }

    // Call ProtoSol backend through gRPC directly from server action
    const client = transactionClient()
    const response = await client.simulateTransaction(grpcRequest)

    return {
      success: true,
      simulationSuccess: response.success,
      error: response.error,
      logs: response.logs || []
    }

  } catch (error: any) {
    console.error('SimulateTransaction server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Transaction simulation failed'
    }
  }
}

// Server action for signing transactions
export async function signTransactionAction(formData: FormData) {
  try {
    const transaction = JSON.parse(formData.get('transaction') as string)
    const privateKeysInput = formData.get('privateKeys') as string
    
    if (!transaction) {
      return { error: 'Transaction is required' }
    }

    if (!privateKeysInput) {
      return { error: 'At least one private key is required for signing' }
    }

    // Parse private keys (could be array or comma-separated string)
    let privateKeys: string[]
    try {
      privateKeys = JSON.parse(privateKeysInput)
    } catch {
      privateKeys = privateKeysInput.split(',').map(key => key.trim())
    }

    // Filter out empty private keys
    const validPrivateKeys = privateKeys.filter(key => key.trim().length > 0)
    if (validPrivateKeys.length === 0) {
      return { error: 'At least one valid private key is required for signing' }
    }

    // Build gRPC request
    const grpcRequest: any = {
      transaction,
      privateKeys: validPrivateKeys
    }

    // Call ProtoSol backend through gRPC directly from server action
    const client = transactionClient()
    const response = await client.signTransaction(grpcRequest)

    return {
      success: true,
      transaction: response.transaction || {},
      signaturesAdded: (response as any).signaturesAdded,
      totalSignatures: (response as any).totalSignatures
    }

  } catch (error: any) {
    console.error('SignTransaction server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Transaction signing failed'
    }
  }
}

// Server action for submitting transactions
export async function submitTransactionAction(formData: FormData) {
  try {
    const transaction = JSON.parse(formData.get('transaction') as string)
    const commitmentLevel = mapCommitmentLevel(formData.get('commitmentLevel') as string)

    if (!transaction) {
      return { error: 'Transaction is required' }
    }

    // Build gRPC request
    const grpcRequest: any = {
      transaction,
      commitmentLevel
    }

    // Call ProtoSol backend through gRPC directly from server action
    const client = transactionClient()
    const response = await client.submitTransaction(grpcRequest)

    return {
      success: true,
      transactionSignature: response.signature,
      submissionResult: response.submissionResult,
      note: 'Transaction submitted to blockchain network'
    }

  } catch (error: any) {
    console.error('SubmitTransaction server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Transaction submission failed'
    }
  }
}

// Server action for getting/monitoring transactions
export async function getTransactionAction(formData: FormData) {
  try {
    const transactionSignature = formData.get('transactionSignature') as string
    const commitmentLevel = mapCommitmentLevel(formData.get('commitmentLevel') as string)

    if (!transactionSignature) {
      return { error: 'Transaction signature is required' }
    }

    // Build gRPC request
    const grpcRequest: any = {
      signature: transactionSignature,
      commitmentLevel
    }

    // Call ProtoSol backend through gRPC directly from server action
    const client = transactionClient()
    const response = await client.getTransaction(grpcRequest)

    return {
      success: true,
      transaction: response.transaction || {},
      signature: transactionSignature,
      note: 'GetTransaction service response structure may vary based on backend implementation'
    }

  } catch (error: any) {
    console.error('GetTransaction server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Transaction lookup failed'
    }
  }
}