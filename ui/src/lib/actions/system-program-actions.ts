'use server'

import { systemProgramClient } from '../grpc-clients'

// Server action for System Program Create operation
export async function createAccountAction(formData: FormData) {
  try {
    const payer = formData.get('payer') as string
    const newAccount = formData.get('newAccount') as string
    const lamports = formData.get('lamports') as string
    const space = formData.get('space') as string
    const owner = formData.get('owner') as string

    if (!payer) return { error: 'payer is required' }
    if (!newAccount) return { error: 'newAccount is required' }
    if (!lamports) return { error: 'lamports is required' }
    if (!space) return { error: 'space is required' }
    if (!owner) return { error: 'owner is required' }

    // Convert numeric values
    let lamportsBigInt: bigint
    let spaceBigInt: bigint
    try {
      lamportsBigInt = BigInt(lamports)
      spaceBigInt = BigInt(space)
      if (lamportsBigInt < 0 || spaceBigInt < 0) {
        return { error: 'lamports and space must be non-negative' }
      }
    } catch {
      return { error: 'lamports and space must be valid numbers' }
    }

    const grpcRequest = {
      payer,
      newAccount,
      owner,
      lamports: lamportsBigInt,
      space: spaceBigInt,
    }

    const client = systemProgramClient()
    const response = await client.create(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'create'
    }

  } catch (error: any) {
    console.error('Create account server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program create operation failed'
    }
  }
}

// Server action for System Program Transfer operation
export async function transferAction(formData: FormData) {
  try {
    const from = formData.get('from') as string
    const to = formData.get('to') as string
    const lamports = formData.get('lamports') as string

    if (!from) return { error: 'from is required' }
    if (!to) return { error: 'to is required' }
    if (!lamports) return { error: 'lamports is required' }

    let lamportsBigInt: bigint
    try {
      lamportsBigInt = BigInt(lamports)
      if (lamportsBigInt < 0) {
        return { error: 'lamports must be non-negative' }
      }
    } catch {
      return { error: 'lamports must be a valid number' }
    }

    const grpcRequest = {
      from,
      to,
      lamports: lamportsBigInt,
    }

    const client = systemProgramClient()
    const response = await client.transfer(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'transfer'
    }

  } catch (error: any) {
    console.error('Transfer server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program transfer operation failed'
    }
  }
}

// Server action for System Program Allocate operation
export async function allocateAction(formData: FormData) {
  try {
    const account = formData.get('account') as string
    const space = formData.get('space') as string

    if (!account) return { error: 'account is required' }
    if (!space) return { error: 'space is required' }

    let spaceBigInt: bigint
    try {
      spaceBigInt = BigInt(space)
      if (spaceBigInt < 0) {
        return { error: 'space must be non-negative' }
      }
    } catch {
      return { error: 'space must be a valid number' }
    }

    const grpcRequest = {
      account,
      space: spaceBigInt,
    }

    const client = systemProgramClient()
    const response = await client.allocate(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'allocate'
    }

  } catch (error: any) {
    console.error('Allocate server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program allocate operation failed'
    }
  }
}

// Server action for System Program Assign operation
export async function assignAction(formData: FormData) {
  try {
    const account = formData.get('account') as string
    const ownerProgram = formData.get('ownerProgram') as string

    if (!account) return { error: 'account is required' }
    if (!ownerProgram) return { error: 'ownerProgram is required' }

    const grpcRequest = {
      account,
      ownerProgram,
    }

    const client = systemProgramClient()
    const response = await client.assign(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'assign'
    }

  } catch (error: any) {
    console.error('Assign server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program assign operation failed'
    }
  }
}

// Server action for System Program CreateWithSeed operation
export async function createWithSeedAction(formData: FormData) {
  try {
    const payer = formData.get('payer') as string
    const newAccount = formData.get('newAccount') as string
    const base = formData.get('base') as string
    const seed = formData.get('seed') as string
    const lamports = formData.get('lamports') as string
    const space = formData.get('space') as string

    if (!payer) return { error: 'payer is required' }
    if (!newAccount) return { error: 'newAccount is required' }
    if (!base) return { error: 'base is required' }
    if (!seed) return { error: 'seed is required' }
    if (!lamports) return { error: 'lamports is required' }
    if (!space) return { error: 'space is required' }

    let lamportsBigInt: bigint
    let spaceBigInt: bigint
    try {
      lamportsBigInt = BigInt(lamports)
      spaceBigInt = BigInt(space)
      if (lamportsBigInt < 0 || spaceBigInt < 0) {
        return { error: 'lamports and space must be non-negative' }
      }
    } catch {
      return { error: 'lamports and space must be valid numbers' }
    }

    const grpcRequest = {
      payer,
      newAccount,
      base,
      seed,
      lamports: lamportsBigInt,
      space: spaceBigInt,
    }

    const client = systemProgramClient()
    const response = await client.createWithSeed(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'createWithSeed'
    }

  } catch (error: any) {
    console.error('CreateWithSeed server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program createWithSeed operation failed'
    }
  }
}

// Server action for System Program AllocateWithSeed operation
export async function allocateWithSeedAction(formData: FormData) {
  try {
    const account = formData.get('account') as string
    const base = formData.get('base') as string
    const seed = formData.get('seed') as string
    const space = formData.get('space') as string

    if (!account) return { error: 'account is required' }
    if (!base) return { error: 'base is required' }
    if (!seed) return { error: 'seed is required' }
    if (!space) return { error: 'space is required' }

    let spaceBigInt: bigint
    try {
      spaceBigInt = BigInt(space)
      if (spaceBigInt < 0) {
        return { error: 'space must be non-negative' }
      }
    } catch {
      return { error: 'space must be a valid number' }
    }

    const grpcRequest = {
      account,
      base,
      seed,
      space: spaceBigInt,
    }

    const client = systemProgramClient()
    const response = await client.allocateWithSeed(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'allocateWithSeed'
    }

  } catch (error: any) {
    console.error('AllocateWithSeed server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program allocateWithSeed operation failed'
    }
  }
}

// Server action for System Program AssignWithSeed operation
export async function assignWithSeedAction(formData: FormData) {
  try {
    const account = formData.get('account') as string
    const base = formData.get('base') as string
    const seed = formData.get('seed') as string
    const ownerProgram = formData.get('ownerProgram') as string

    if (!account) return { error: 'account is required' }
    if (!base) return { error: 'base is required' }
    if (!seed) return { error: 'seed is required' }
    if (!ownerProgram) return { error: 'ownerProgram is required' }

    const grpcRequest = {
      account,
      base,
      seed,
      ownerProgram,
    }

    const client = systemProgramClient()
    const response = await client.assignWithSeed(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'assignWithSeed'
    }

  } catch (error: any) {
    console.error('AssignWithSeed server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program assignWithSeed operation failed'
    }
  }
}

// Server action for System Program TransferWithSeed operation
export async function transferWithSeedAction(formData: FormData) {
  try {
    const from = formData.get('from') as string
    const fromBase = formData.get('fromBase') as string
    const fromSeed = formData.get('fromSeed') as string
    const to = formData.get('to') as string
    const lamports = formData.get('lamports') as string

    if (!from) return { error: 'from is required' }
    if (!fromBase) return { error: 'fromBase is required' }
    if (!fromSeed) return { error: 'fromSeed is required' }
    if (!to) return { error: 'to is required' }
    if (!lamports) return { error: 'lamports is required' }

    let lamportsBigInt: bigint
    try {
      lamportsBigInt = BigInt(lamports)
      if (lamportsBigInt < 0) {
        return { error: 'lamports must be non-negative' }
      }
    } catch {
      return { error: 'lamports must be a valid number' }
    }

    const grpcRequest = {
      from,
      fromBase,
      fromSeed,
      to,
      lamports: lamportsBigInt,
    }

    const client = systemProgramClient()
    const response = await client.transferWithSeed(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'transferWithSeed'
    }

  } catch (error: any) {
    console.error('TransferWithSeed server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program transferWithSeed operation failed'
    }
  }
}

// Server action for System Program InitializeNonceAccount operation
export async function initializeNonceAccountAction(formData: FormData) {
  try {
    const nonceAccount = formData.get('nonceAccount') as string
    const authority = formData.get('authority') as string

    if (!nonceAccount) return { error: 'nonceAccount is required' }
    if (!authority) return { error: 'authority is required' }

    const grpcRequest = {
      nonceAccount,
      authority,
    }

    const client = systemProgramClient()
    const response = await client.initializeNonceAccount(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'initializeNonceAccount'
    }

  } catch (error: any) {
    console.error('InitializeNonceAccount server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program initializeNonceAccount operation failed'
    }
  }
}

// Server action for System Program AuthorizeNonceAccount operation
export async function authorizeNonceAccountAction(formData: FormData) {
  try {
    const nonceAccount = formData.get('nonceAccount') as string
    const currentAuthority = formData.get('currentAuthority') as string
    const newAuthority = formData.get('newAuthority') as string

    if (!nonceAccount) return { error: 'nonceAccount is required' }
    if (!currentAuthority) return { error: 'currentAuthority is required' }
    if (!newAuthority) return { error: 'newAuthority is required' }

    const grpcRequest = {
      nonceAccount,
      currentAuthority,
      newAuthority,
    }

    const client = systemProgramClient()
    const response = await client.authorizeNonceAccount(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'authorizeNonceAccount'
    }

  } catch (error: any) {
    console.error('AuthorizeNonceAccount server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program authorizeNonceAccount operation failed'
    }
  }
}

// Server action for System Program WithdrawNonceAccount operation
export async function withdrawNonceAccountAction(formData: FormData) {
  try {
    const nonceAccount = formData.get('nonceAccount') as string
    const authority = formData.get('authority') as string
    const to = formData.get('to') as string
    const lamports = formData.get('lamports') as string

    if (!nonceAccount) return { error: 'nonceAccount is required' }
    if (!authority) return { error: 'authority is required' }
    if (!to) return { error: 'to is required' }
    if (!lamports) return { error: 'lamports is required' }

    let lamportsBigInt: bigint
    try {
      lamportsBigInt = BigInt(lamports)
      if (lamportsBigInt < 0) {
        return { error: 'lamports must be non-negative' }
      }
    } catch {
      return { error: 'lamports must be a valid number' }
    }

    const grpcRequest = {
      nonceAccount,
      authority,
      to,
      lamports: lamportsBigInt,
    }

    const client = systemProgramClient()
    const response = await client.withdrawNonceAccount(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'withdrawNonceAccount'
    }

  } catch (error: any) {
    console.error('WithdrawNonceAccount server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program withdrawNonceAccount operation failed'
    }
  }
}

// Server action for System Program AdvanceNonceAccount operation
export async function advanceNonceAccountAction(formData: FormData) {
  try {
    const nonceAccount = formData.get('nonceAccount') as string
    const authority = formData.get('authority') as string

    if (!nonceAccount) return { error: 'nonceAccount is required' }
    if (!authority) return { error: 'authority is required' }

    const grpcRequest = {
      nonceAccount,
      authority,
    }

    const client = systemProgramClient()
    const response = await client.advanceNonceAccount(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'advanceNonceAccount'
    }

  } catch (error: any) {
    console.error('AdvanceNonceAccount server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program advanceNonceAccount operation failed'
    }
  }
}

// Server action for System Program UpgradeNonceAccount operation
export async function upgradeNonceAccountAction(formData: FormData) {
  try {
    const nonceAccount = formData.get('nonceAccount') as string

    if (!nonceAccount) return { error: 'nonceAccount is required' }

    const grpcRequest = {
      nonceAccount,
    }

    const client = systemProgramClient()
    const response = await client.upgradeNonceAccount(grpcRequest)

    return {
      success: true,
      instruction: response,
      operation: 'upgradeNonceAccount'
    }

  } catch (error: any) {
    console.error('UpgradeNonceAccount server action error:', error)
    return {
      error: `gRPC Error: ${error.message}`,
      details: 'System program upgradeNonceAccount operation failed'
    }
  }
}