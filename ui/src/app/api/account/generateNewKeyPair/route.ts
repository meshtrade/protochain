// Server-side API route for generate keypair
// Uses ProtoSol gRPC backend to generate real keypairs

import { NextRequest, NextResponse } from 'next/server'
import { accountClient } from '../../../../lib/grpc-clients'

// Define the request interface matching the protobuf structure
interface GenerateKeyPairRequest {
  seed?: string
}

export async function POST(request: NextRequest) {
  try {
    // Parse request body for optional seed parameter
    const requestData: GenerateKeyPairRequest = {}
    
    try {
      const body = await request.text()
      if (body) {
        const parsed = JSON.parse(body)
        if (parsed.seed && typeof parsed.seed === 'string') {
          requestData.seed = parsed.seed
        }
      }
    } catch {
      // If parsing fails, proceed with empty request (no seed)
      console.log('No seed provided or invalid JSON, generating random keypair')
    }

    // Call ProtoSol backend through gRPC
    const client = accountClient()
    const response = await client.generateNewKeyPair(requestData)

    // Return the generated keypair
    return NextResponse.json({
      keyPair: {
        publicKey: response.keyPair?.publicKey || '',
        privateKey: response.keyPair?.privateKey || '',
      }
    })

  } catch (error) {
    console.error('gRPC error generating keypair:', error)
    
    // Handle specific gRPC errors
    if (error && typeof error === 'object' && 'code' in error) {
      const grpcError = error as any
      return NextResponse.json(
        { 
          error: 'Failed to generate keypair', 
          details: grpcError.message || 'Unknown gRPC error',
          code: grpcError.code 
        },
        { status: 500 }
      )
    }

    // Handle general errors
    return NextResponse.json(
      { error: 'Failed to generate keypair' },
      { status: 500 }
    )
  }
}
