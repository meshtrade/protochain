// Server-side API route for generate keypair
// Note: This demonstrates the server-side call pattern
// In production, this would call the actual ProtoSol backend

import { NextResponse } from 'next/server'

export async function POST() {
  try {
    // Mock keypair response - in production this would call actual ProtoSol backend
    const keyPairResponse = {
      keyPair: {
        publicKey: '11111111111111111111111111111112',
        privateKey: 'mock_private_key_would_be_64_bytes'
      }
    }

    return NextResponse.json(keyPairResponse)
  } catch (error) {
    console.error('Server error generating keypair:', error)
    return NextResponse.json(
      { error: 'Failed to generate keypair' },
      { status: 500 }
    )
  }
}
