// Server-side API route for token initialise mint
// Uses ProtoSol gRPC backend to generate real mint initialization instructions

import { NextRequest, NextResponse } from 'next/server'
import { tokenProgramClient } from '../../../../lib/grpc-clients'

interface RequestBody {
  mintPubKey: string
  mintAuthorityPubKey: string
  freezeAuthorityPubKey?: string // Optional freeze authority
  decimals: number
}

// Define the gRPC request interface matching the protobuf structure
interface InitialiseMintGrpcRequest {
  mintPubKey: string
  mintAuthorityPubKey: string
  freezeAuthorityPubKey?: string
  decimals: number
}

export async function POST(request: NextRequest) {
  try {
    const body: RequestBody = await request.json()
    console.log('🔧 Server: InitialiseMint request:', body)

    // Validate required parameters
    if (!body.mintPubKey || typeof body.mintPubKey !== 'string') {
      return NextResponse.json(
        { error: 'mintPubKey is required and must be a string' },
        { status: 400 }
      )
    }

    if (!body.mintAuthorityPubKey || typeof body.mintAuthorityPubKey !== 'string') {
      return NextResponse.json(
        { error: 'mintAuthorityPubKey is required and must be a string' },
        { status: 400 }
      )
    }

    if (typeof body.decimals !== 'number' || body.decimals < 0 || body.decimals > 9) {
      return NextResponse.json(
        { error: 'decimals must be a number between 0 and 9' },
        { status: 400 }
      )
    }

    // Build gRPC request
    const grpcRequest: InitialiseMintGrpcRequest = {
      mintPubKey: body.mintPubKey,
      mintAuthorityPubKey: body.mintAuthorityPubKey,
      decimals: body.decimals,
    }

    // Add freeze authority if provided
    if (body.freezeAuthorityPubKey) {
      grpcRequest.freezeAuthorityPubKey = body.freezeAuthorityPubKey
    }

    // Call ProtoSol backend through gRPC
    const client = tokenProgramClient()
    const response = await client.initialiseMint(grpcRequest)

    // Return the instruction response
    return NextResponse.json({
      instruction: {
        programId: response.instruction?.programId || '',
        accounts: response.instruction?.accounts?.map((account: any) => ({
          pubkey: account.pubkey,
          isSigner: account.isSigner,
          isWritable: account.isWritable,
        })) || [],
        data: response.instruction?.data || new Uint8Array(),
      }
    })

  } catch (error) {
    console.error('gRPC error initialising mint:', error)
    
    // Handle specific gRPC errors
    if (error && typeof error === 'object' && 'code' in error) {
      const grpcError = error as any
      
      // Handle validation errors
      if (grpcError.code === 'INVALID_ARGUMENT') {
        return NextResponse.json(
          { 
            error: 'Invalid parameters', 
            details: grpcError.message || 'Invalid mint initialization parameters',
          },
          { status: 400 }
        )
      }

      // Handle other gRPC errors
      return NextResponse.json(
        { 
          error: 'Failed to initialize mint', 
          details: grpcError.message || 'Unknown gRPC error',
          code: grpcError.code 
        },
        { status: 500 }
      )
    }

    // Handle general errors
    return NextResponse.json(
      { error: 'Failed to initialize mint' },
      { status: 500 }
    )
  }
}
