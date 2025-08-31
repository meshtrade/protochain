// Server-side API route for account funding (airdrop)
// Uses ProtoSol gRPC backend to fund accounts with SOL

import { NextRequest, NextResponse } from 'next/server'
import { accountClient } from '../../../../lib/grpc-clients'
import type { CommitmentLevel } from '@protosol/api'

interface RequestBody {
  address: string
  amount: string
  commitmentLevel?: string
}

// Define the gRPC request interface matching the protobuf structure
interface FundNativeGrpcRequest {
  address: string
  amount: string
  commitmentLevel?: CommitmentLevel
}

export async function POST(request: NextRequest) {
  let requestBody: RequestBody | null = null;
  
  try {
    requestBody = await request.json()
    console.log('🔧 Server: FundNative request:', requestBody)

    // Validate required parameters
    if (!requestBody?.address || typeof requestBody.address !== 'string') {
      return NextResponse.json(
        { error: 'Address is required and must be a string' },
        { status: 400 }
      )
    }

    if (!requestBody?.amount || typeof requestBody.amount !== 'string') {
      return NextResponse.json(
        { error: 'Amount is required and must be a string' },
        { status: 400 }
      )
    }

    // Validate amount is a valid number
    const amountNum = BigInt(requestBody.amount)
    if (amountNum <= 0) {
      return NextResponse.json(
        { error: 'Amount must be a positive number' },
        { status: 400 }
      )
    }

    // Build gRPC request
    const grpcRequest: FundNativeGrpcRequest = {
      address: requestBody.address,
      amount: requestBody.amount,
    }

    // Add commitment level if provided
    if (requestBody.commitmentLevel) {
      // Map string to protobuf enum value
      const commitmentMap: Record<string, CommitmentLevel> = {
        'processed': 1,
        'confirmed': 2, 
        'finalized': 3,
      }
      const commitmentLevel = commitmentMap[requestBody.commitmentLevel.toLowerCase()]
      if (commitmentLevel !== undefined) {
        grpcRequest.commitmentLevel = commitmentLevel
      }
    }

    // Call ProtoSol backend through gRPC
    const client = accountClient()
    const response = await client.fundNative(grpcRequest)

    // Return the transaction signature
    return NextResponse.json({
      signature: response.signature,
      message: `Successfully funded ${requestBody.address} with ${requestBody.amount} lamports`,
    })

  } catch (error) {
    console.error('gRPC error funding account:', error)
    
    // Handle specific gRPC errors
    if (error && typeof error === 'object' && 'code' in error) {
      const grpcError = error as any
      
      // Handle insufficient funds or airdrop limits
      if (grpcError.code === 'RESOURCE_EXHAUSTED' || grpcError.message?.includes('airdrop')) {
        return NextResponse.json(
          { 
            error: 'Airdrop failed', 
            details: `Failed to airdrop to ${requestBody?.address || 'unknown'}. This may be due to airdrop limits or network issues.`,
          },
          { status: 429 }
        )
      }

      // Handle invalid address
      if (grpcError.code === 'INVALID_ARGUMENT' || grpcError.message?.includes('invalid')) {
        return NextResponse.json(
          { 
            error: 'Invalid request', 
            details: grpcError.message || 'Invalid address or amount',
          },
          { status: 400 }
        )
      }

      // Handle other gRPC errors
      return NextResponse.json(
        { 
          error: 'Failed to fund account', 
          details: grpcError.message || 'Unknown gRPC error',
          code: grpcError.code 
        },
        { status: 500 }
      )
    }

    // Handle general errors
    return NextResponse.json(
      { error: 'Failed to fund account' },
      { status: 500 }
    )
  }
}