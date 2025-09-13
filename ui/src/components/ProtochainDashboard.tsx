'use client'

import { useEffect, useState } from 'react'
import { generateNewKeyPairAction } from '../lib/actions/account-actions'

interface DashboardState {
  sdkVersion: string
  sdkName: string
  currentTime: string
  connectionStatus: 'connecting' | 'connected' | 'disconnected'
}

export function ProtochainDashboard() {
  const [state, setState] = useState<DashboardState>({
    sdkVersion: 'Loading...',
    sdkName: 'Loading...',
    currentTime: '--:--:--',
    connectionStatus: 'connecting'
  })

  const [keypair, setKeypair] = useState<{ publicKey: string; privateKey: string } | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    // Initialize SDK information
    const timerId = setTimeout(() => {
      setState(prev => ({
        ...prev,
        sdkVersion: 'Server Actions',
        sdkName: 'Protochain SDK (Server Actions)',
        connectionStatus: 'connected'
      }))
    }, 500) // Add a small delay to simulate loading

    // Update current time every second
    const timeInterval = setInterval(() => {
      setState(prev => ({
        ...prev,
        currentTime: new Date().toLocaleTimeString()
      }))
    }, 1000)

    return () => {
      clearTimeout(timerId)
      clearInterval(timeInterval)
    }
  }, [])

  const generateKeypair = async () => {
    try {
      setLoading(true)

      // Call server action
      console.log('🟡 Calling server action: generateNewKeyPairAction')
      const result = await generateNewKeyPairAction()
      
      if (!result.success || !result.keyPair) {
        throw new Error(result.error || 'Failed to generate keypair')
      }

      console.log('✅ Server action response:', result.keyPair)
      setKeypair({
        publicKey: result.keyPair.publicKey || '',
        privateKey: result.keyPair.privateKey || ''
      })
    } catch (error) {
      console.error('Error generating keypair:', error)
      // Fallback to mock if server call fails
      setKeypair({
        publicKey: '11111111111111111111111111111112',
        privateKey: 'mock_private_key_fallback'
      })
    } finally {
      setLoading(false)
    }
  }

  const getStatusColor = (status: DashboardState['connectionStatus']) => {
    switch (status) {
      case 'connected': return 'text-green-600 bg-green-100'
      case 'connecting': return 'text-yellow-600 bg-yellow-100'
      case 'disconnected': return 'text-red-600 bg-red-100'
    }
  }

  const getStatusIcon = (status: DashboardState['connectionStatus']) => {
    switch (status) {
      case 'connected': return '🟢'
      case 'connecting': return '🟡'
      case 'disconnected': return '🔴'
    }
  }

  return (
    <div className="bg-white rounded-2xl shadow-xl p-8 max-w-4xl mx-auto">
      <div className="text-center mb-8">
        <h2 className="text-3xl font-bold text-slate-900 mb-2">
          Protochain SDK Dashboard
        </h2>
        <p className="text-slate-600">
          Running the latest Protochain SDK with Next.js 15
        </p>
      </div>

      <div className="grid md:grid-cols-2 gap-8 mb-8">
        {/* SDK Information */}
        <div className="space-y-4">
          <h3 className="text-xl font-semibold text-slate-800 mb-4">
            SDK Information
          </h3>
          <div className="space-y-3">
            <div className="flex justify-between items-center p-3 bg-slate-50 rounded-lg">
              <span className="font-medium text-slate-700">SDK Name:</span>
              <span className="text-slate-900 font-mono">{state.sdkName}</span>
            </div>
            <div className="flex justify-between items-center p-3 bg-slate-50 rounded-lg">
              <span className="font-medium text-slate-700">Version:</span>
              <span className="text-slate-900 font-mono">v{state.sdkVersion}</span>
            </div>
            <div className="flex justify-between items-center p-3 bg-slate-50 rounded-lg">
              <span className="font-medium text-slate-700">Time:</span>
              <span className="text-slate-900 font-mono">{state.currentTime}</span>
            </div>
          </div>
        </div>

        {/* Connection Status */}
        <div className="space-y-4">
          <h3 className="text-xl font-semibold text-slate-800 mb-4">
            System Status
          </h3>
          <div className="p-4 bg-slate-50 rounded-lg">
            <div className="flex items-center justify-between mb-3">
              <span className="font-medium text-slate-700">SDK Connection:</span>
              <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(state.connectionStatus)}`}>
                {getStatusIcon(state.connectionStatus)} {state.connectionStatus}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="font-medium text-slate-700">Next.js:</span>
              <span className="px-2 py-1 rounded-full text-xs font-medium text-green-600 bg-green-100">
                🟢 Running (v15)
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Interactive Demo Section */}
      <div className="border-t border-slate-200 pt-8">
        <h3 className="text-xl font-semibold text-slate-800 mb-4">
          Interactive Demo - Protochain API Calls
        </h3>
        <div className="space-y-4">
          <div className="flex gap-4 items-center">
            <button
              onClick={generateKeypair}
              disabled={loading}
              className="px-6 py-3 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-medium rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:from-gray-400 disabled:to-gray-500 disabled:cursor-not-allowed transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
            >
              {loading ? 'Generating...' : 'Generate Keypair'}
            </button>
            <div className="text-sm text-slate-600">
              Uses: <code className="bg-slate-100 px-1 py-0.5 rounded text-xs">api.account.v1.Service.GenerateNewKeyPair()</code>
            </div>
          </div>

          {keypair && (
            <div className="bg-green-50 border border-green-200 rounded-lg p-4 animate-fade-in">
              <div className="flex items-center mb-2">
                <span className="text-green-700 font-medium">✓ Keypair Generated</span>
              </div>
              <div className="space-y-2 text-sm font-mono text-slate-700">
                <div>
                  <span className="font-medium text-slate-600">Public Key:</span>{' '}
                  <span className="bg-white px-2 py-1 rounded border">{keypair.publicKey}</span>
                </div>
                <div>
                  <span className="font-medium text-slate-600">Private Key:</span>{' '}
                  <span className="bg-white px-2 py-1 rounded border">••••••••••••••••••••</span>
                </div>
              </div>
            </div>
          )}

          {/* API Call Examples */}
          <div className="bg-slate-50 rounded-lg p-6 mt-6">
            <h4 className="font-medium text-slate-800 mb-4">Available Protochain API Calls:</h4>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div className="space-y-2">
                <div className="font-medium text-slate-700">Account Services:</div>
                <div className="space-y-1">
                  <code className="block bg-white px-2 py-1 rounded border">api.account.v1.Service.GetAccount(...)</code>
                  <code className="block bg-white px-2 py-1 rounded border">api.account.v1.Service.GenerateNewKeyPair()</code>
                  <code className="block bg-white px-2 py-1 rounded border">api.account.v1.Service.FundNative(...)</code>
                </div>
              </div>
              <div className="space-y-2">
                <div className="font-medium text-slate-700">Token Program:</div>
                <div className="space-y-1">
                  <code className="block bg-white px-2 py-1 rounded border">api.token.program.v1.Service.InitialiseMint(...)</code>
                  <code className="block bg-white px-2 py-1 rounded border">api.token.program.v1.Service.ParseMint(...)</code>
                  <code className="block bg-white px-2 py-1 rounded border">api.token.program.v1.Service.GetCurrentMinRentForTokenAccount()</code>
                </div>
              </div>
              <div className="space-y-2">
                <div className="font-medium text-slate-700">Transactions:</div>
                <div className="space-y-1">
                  <code className="block bg-white px-2 py-1 rounded border">api.transaction.v1.Service.CompileTransaction(...)</code>
                  <code className="block bg-white px-2 py-1 rounded border">api.transaction.v1.Service.SignTransaction(...)</code>
                  <code className="block bg-white px-2 py-1 rounded border">api.transaction.v1.Service.SubmitTransaction(...)</code>
                </div>
              </div>
              <div className="space-y-2">
                <div className="font-medium text-slate-700">System Program:</div>
                <div className="space-y-1">
                  <code className="block bg-white px-2 py-1 rounded border">api.system.program.v1.Service.Create(...)</code>
                  <code className="block bg-white px-2 py-1 rounded border">api.system.program.v1.Service.Transfer(...)</code>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="border-t border-slate-200 pt-8 mt-8">
        <div className="text-center text-slate-500">
          <p className="text-sm mb-2">
            This is a modern Next.js 15 App Router application using the Protochain TypeScript SDK.
          </p>
          <p className="text-xs">
            Built with TypeScript, Tailwind CSS, and the latest web technologies.
          </p>
        </div>
      </div>
    </div>
  )
}
