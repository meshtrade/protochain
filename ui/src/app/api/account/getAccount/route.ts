// Server-side API route for account get
// Note: This demonstrates the server-side call pattern
// In production, this would call the actual ProtoSol backend

import { NextRequest, NextResponse } from 'next/server'

interface GetAccountRequest {
  address: string
  commitmentLevel?: string
}

export async function POST(request: NextRequest) {
  try {
    const body: GetAccountRequest = await request.json()
    console.log('🔧 Server: GetAccount request:', body)

    // Mock response - in production this would call actual ProtoSol backend
    const account = {
      address: body.address,
      lamports: '1000000000', // 1 SOL
      owner: '11111111111111111111111111111112', // System Program
      executable: false,
      rentEpoch: 0,
      data: new Uint8Array([0, 0, 0, 0]) // No data
    }

    return NextResponse.json(account)
  } catch (error) {
    console.error('Server error getting account:', error)
    return NextResponse.json(
      { error: 'Failed to get account' },
      { status: 500 }
    )
  }
}
