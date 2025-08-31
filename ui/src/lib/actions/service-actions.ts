'use server'

import { rpcClient, tokenProgramClient, systemProgramClient } from '../grpc-clients'

// Server action for RPC Client operations
export async function getMinimumBalanceForRentExemptionAction(formData: FormData) {
  try {
    const dataLength = formData.get('dataLength') as string

    if (!dataLength) {
      return { error: 'dataLength is required' }
    }

    // Convert to BigInt and validate
    let dataLengthBigInt: bigint
    try {
      dataLengthBigInt = BigInt(dataLength)
      if (dataLengthBigInt < 0) {
        return { error: 'dataLength must be a non-negative number' }
      }
    } catch {
      return { error: 'dataLength must be a valid number' }
    }

    // Build gRPC request
    const grpcRequest: any = {
      dataLength: dataLengthBigInt,
    }

    // Call ProtoSol backend through gRPC directly from server action
    const client = rpcClient()
    const response = await client.getMinimumBalanceForRentExemption(grpcRequest)

    return {
      success: true,
      minimumBalance: (response as any).minimumBalance?.toString() || '0',
      dataLength: dataLength
    }

  } catch (error: any) {
    console.error('GetMinimumBalanceForRentExemption server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'RPC client operation failed'
    }
  }
}

// Server action for Token Program operations
export async function initializeMintAction(formData: FormData) {
  try {
    const mintAddress = formData.get('mintAddress') as string
    const decimals = formData.get('decimals') as string
    const mintAuthority = formData.get('mintAuthority') as string

    if (!mintAddress) {
      return { error: 'mintAddress is required' }
    }

    if (!decimals) {
      return { error: 'decimals is required' }
    }

    if (!mintAuthority) {
      return { error: 'mintAuthority is required' }
    }

    // Convert decimals to number
    let decimalsNum: number
    try {
      decimalsNum = parseInt(decimals)
      if (decimalsNum < 0 || decimalsNum > 9) {
        return { error: 'decimals must be between 0 and 9' }
      }
    } catch {
      return { error: 'decimals must be a valid number' }
    }

    // Build gRPC request
    const grpcRequest: any = {
      mintAddress,
      decimals: decimalsNum,
      mintAuthority,
      freezeAuthority: formData.get('freezeAuthority') as string || undefined
    }

    // Call ProtoSol backend through gRPC directly from server action
    const client = tokenProgramClient()
    const response = await client.initialiseMint(grpcRequest)

    return {
      success: true,
      instruction: (response as any).instruction,
      message: 'Initialize mint instruction created successfully'
    }

  } catch (error: any) {
    console.error('InitializeMint server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'Token program operation failed'
    }
  }
}

// Server action for System Program Create Account operation
export async function systemCreateAccountAction(formData: FormData) {
  try {
    const fromPubkey = formData.get('fromPubkey') as string
    const newAccountPubkey = formData.get('newAccountPubkey') as string
    const lamports = formData.get('lamports') as string
    const space = formData.get('space') as string
    const owner = formData.get('owner') as string

    if (!fromPubkey) {
      return { error: 'fromPubkey is required' }
    }

    if (!newAccountPubkey) {
      return { error: 'newAccountPubkey is required' }
    }

    if (!lamports) {
      return { error: 'lamports is required' }
    }

    if (!space) {
      return { error: 'space is required' }
    }

    if (!owner) {
      return { error: 'owner is required' }
    }

    // Convert numeric fields
    let lamportsNum: bigint
    let spaceNum: bigint
    try {
      lamportsNum = BigInt(lamports)
      spaceNum = BigInt(space)
    } catch {
      return { error: 'lamports and space must be valid numbers' }
    }

    // Build gRPC request
    const grpcRequest: any = {
      fromPubkey,
      newAccountPubkey,
      lamports: lamportsNum,
      space: spaceNum,
      owner
    }

    // Call ProtoSol backend through gRPC directly from server action
    const client = systemProgramClient()
    const response = await client.create(grpcRequest)

    return {
      success: true,
      instruction: (response as any).instruction,
      message: 'Create account instruction created successfully'
    }

  } catch (error: any) {
    console.error('SystemCreateAccount server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program operation failed'
    }
  }
}

// Server action for System Program Transfer operation
export async function systemTransferAction(formData: FormData) {
  try {
    const fromPubkey = formData.get('fromPubkey') as string
    const toPubkey = formData.get('toPubkey') as string
    const lamports = formData.get('lamports') as string

    if (!fromPubkey) {
      return { error: 'fromPubkey is required' }
    }

    if (!toPubkey) {
      return { error: 'toPubkey is required' }
    }

    if (!lamports) {
      return { error: 'lamports is required' }
    }

    // Convert lamports to BigInt
    let lamportsNum: bigint
    try {
      lamportsNum = BigInt(lamports)
    } catch {
      return { error: 'lamports must be a valid number' }
    }

    // Build gRPC request
    const grpcRequest: any = {
      fromPubkey,
      toPubkey,
      lamports: lamportsNum
    }

    // Call ProtoSol backend through gRPC directly from server action
    const client = systemProgramClient()
    const response = await client.transfer(grpcRequest)

    return {
      success: true,
      instruction: (response as any).instruction,
      message: 'Transfer instruction created successfully'
    }

  } catch (error: any) {
    console.error('SystemTransfer server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program operation failed'
    }
  }
}