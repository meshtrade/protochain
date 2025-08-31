// Server-side API route for getting minimum balance for rent exemption
// Uses ProtoSol gRPC backend to calculate rent exemption amounts

import { NextRequest, NextResponse } from 'next/server'
import { rpcClient } from '../../../../lib/grpc-clients'

interface RequestBody {
  dataLength: string
}

// Define the gRPC request interface matching the protobuf structure
interface GetMinimumBalanceForRentExemptionGrpcRequest {
  dataLength: bigint
}

export async function POST(request: NextRequest) {
  let requestBody: RequestBody | null = null;
  
  try {
    requestBody = await request.json()
    console.log('🔧 Server: GetMinimumBalanceForRentExemption request:', requestBody)

    // Validate required dataLength parameter
    if (!requestBody?.dataLength || typeof requestBody.dataLength !== 'string') {
      return NextResponse.json(
        { error: 'dataLength is required and must be a string' },
        { status: 400 }
      )
    }

    // Validate dataLength is a valid number
    let dataLengthBigInt: bigint
    try {
      dataLengthBigInt = BigInt(requestBody.dataLength)
      if (dataLengthBigInt < 0) {
        return NextResponse.json(
          { error: 'dataLength must be a non-negative number' },
          { status: 400 }
        )
      }
    } catch (error) {
      return NextResponse.json(
        { error: 'dataLength must be a valid number' },
        { status: 400 }
      )
    }

    // Build gRPC request
    const grpcRequest: GetMinimumBalanceForRentExemptionGrpcRequest = {
      dataLength: dataLengthBigInt,
    }

    // Call ProtoSol backend through gRPC
    const client = rpcClient()
    const response = await client.getMinimumBalanceForRentExemption(grpcRequest)

    // Return the minimum balance in lamports
    return NextResponse.json({
      minimumBalance: response.balance.toString(), // Convert BigInt to string for JSON
      dataLength: requestBody.dataLength,
      message: `Minimum balance for ${requestBody.dataLength} bytes of data: ${response.balance} lamports`,
    })

  } catch (error) {
    console.error('gRPC error getting minimum balance for rent exemption:', error)
    
    // Handle specific gRPC errors
    if (error && typeof error === 'object' && 'code' in error) {
      const grpcError = error as any
      
      // Handle invalid argument
      if (grpcError.code === 'INVALID_ARGUMENT') {
        return NextResponse.json(
          { 
            error: 'Invalid request', 
            details: grpcError.message || 'Invalid data length',
          },
          { status: 400 }
        )
      }

      // Handle other gRPC errors
      return NextResponse.json(
        { 
          error: 'Failed to get minimum balance for rent exemption', 
          details: grpcError.message || 'Unknown gRPC error',
          code: grpcError.code 
        },
        { status: 500 }
      )
    }

    // Handle general errors
    return NextResponse.json(
      { error: 'Failed to get minimum balance for rent exemption' },
      { status: 500 }
    )
  }
}