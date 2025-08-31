// Server-side API route for account get
// Uses ProtoSol gRPC backend to fetch real account data

import { NextRequest, NextResponse } from 'next/server'
import { accountClient } from '../../../../lib/grpc-clients'
import type { CommitmentLevel } from '@protosol/api'

interface RequestBody {
  address: string
  commitmentLevel?: string
}

// Define the gRPC request interface matching the protobuf structure
interface GetAccountGrpcRequest {
  address: string
  commitmentLevel?: CommitmentLevel
}

export async function POST(request: NextRequest) {
  let requestBody: RequestBody | null = null;
  
  try {
    requestBody = await request.json()
    console.log('🔧 Server: GetAccount request:', requestBody)

    // Validate required address parameter
    if (!requestBody?.address || typeof requestBody.address !== 'string') {
      return NextResponse.json(
        { error: 'Address is required and must be a string' },
        { status: 400 }
      )
    }

    // Build gRPC request
    const grpcRequest: GetAccountGrpcRequest = {
      address: requestBody.address,
    }

    // Add commitment level if provided
    if (requestBody.commitmentLevel) {
      // Map string to protobuf enum value
      const commitmentMap: Record<string, CommitmentLevel> = {
        'processed': 0,
        'confirmed': 1, 
        'finalized': 2,
      }
      const commitmentLevel = commitmentMap[requestBody.commitmentLevel.toLowerCase()]
      if (commitmentLevel !== undefined) {
        grpcRequest.commitmentLevel = commitmentLevel
      }
    }

    // Call ProtoSol backend through gRPC
    const client = accountClient()
    const response = await client.getAccount(grpcRequest)

    // Return the account data
    return NextResponse.json({
      address: response.address,
      lamports: response.lamports,
      owner: response.owner,
      executable: response.executable,
      rentEpoch: response.rentEpoch,
      data: response.data, // Uint8Array will be JSON serialized as array
    })

  } catch (error) {
    console.error('gRPC error getting account:', error)
    
    // Handle specific gRPC errors
    if (error && typeof error === 'object' && 'code' in error) {
      const grpcError = error as any
      
      // Handle account not found
      if (grpcError.code === 'NOT_FOUND' || grpcError.message?.includes('not found')) {
        return NextResponse.json(
          { 
            error: 'Account not found', 
            details: `Account ${requestBody?.address || 'unknown'} does not exist`,
          },
          { status: 404 }
        )
      }

      // Handle other gRPC errors
      return NextResponse.json(
        { 
          error: 'Failed to get account', 
          details: grpcError.message || 'Unknown gRPC error',
          code: grpcError.code 
        },
        { status: 500 }
      )
    }

    // Handle general errors
    return NextResponse.json(
      { error: 'Failed to get account' },
      { status: 500 }
    )
  }
}
