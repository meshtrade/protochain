// Server-side API route for token initialise mint
// Note: This demonstrates the server-side call pattern
// In production, this would use the actual ProtoSol backend

import { NextRequest, NextResponse } from 'next/server'

interface InitialiseMintRequest {
  mintPubKey: string
  mintAuthorityPubKey: string
  freezeAuthorityPubKey: string
  decimals: number
}

export async function POST(request: NextRequest) {
  try {
    const body: InitialiseMintRequest = await request.json()
    console.log('🔧 Server: InitialiseMint request:', body)

    // Mock response - in production this would call actual ProtoSol backend
    const response = {
      instruction: {
        programId: 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
        accounts: [
          { pubkey: body.mintPubKey, isSigner: false, isWritable: true },
          { pubkey: body.mintAuthorityPubKey, isSigner: false, isWritable: false },
          { pubkey: body.freezeAuthorityPubKey, isSigner: false, isWritable: false }
        ],
        data: new Uint8Array([0, body.decimals])
      }
    }

    return NextResponse.json(response)
  } catch (error) {
    console.error('Server error initialising mint:', error)
    return NextResponse.json(
      { error: 'Failed to initialise mint' },
      { status: 500 }
    )
  }
}
