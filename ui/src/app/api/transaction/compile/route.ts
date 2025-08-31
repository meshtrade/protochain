import { NextRequest, NextResponse } from 'next/server'
import { transactionClient } from '../../../../lib/grpc-clients'

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const { transaction, feePayer, recentBlockhash } = body

    if (!transaction) {
      return NextResponse.json(
        { error: 'Transaction is required' },
        { status: 400 }
      )
    }

    if (!feePayer) {
      return NextResponse.json(
        { error: 'Fee payer is required' },
        { status: 400 }
      )
    }

    // Get gRPC client
    const client = transactionClient()

    // Call gRPC service with transaction, fee_payer, and optional recent_blockhash
    const response = await client.compileTransaction({
      transaction,
      feePayer,
      recentBlockhash: recentBlockhash || ''
    })

    return NextResponse.json({
      transaction: {
        instructions: response.transaction?.instructions || [],
        state: response.transaction?.state || 0,
        config: response.transaction?.config || {},
        data: response.transaction?.data || '',
        feePayer: response.transaction?.feePayer || '',
        recentBlockhash: response.transaction?.recentBlockhash || '',
        signatures: response.transaction?.signatures || [],
        hash: response.transaction?.hash || '',
        signature: response.transaction?.signature || ''
      }
    })
  } catch (error) {
    console.error('Transaction compilation failed:', error)
    
    // Enhanced error handling for gRPC errors
    if (error && typeof error === 'object' && 'code' in error) {
      const grpcError = error as { code: string; message: string; details?: string }
      return NextResponse.json(
        { 
          error: `gRPC Error (${grpcError.code}): ${grpcError.message}`,
          details: grpcError.details 
        },
        { status: 500 }
      )
    }

    return NextResponse.json(
      { 
        error: 'Transaction compilation failed',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    )
  }
}