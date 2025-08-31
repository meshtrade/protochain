'use client'

import { useEffect, useState } from 'react'

// Temporary mock for ProtoSol API until workspace is resolved
const mockProtoSolApi = {
  VERSION: '1.0.0',
  SDK_NAME: 'ProtoSol SDK (Mock)'
}

interface DashboardState {
  sdkVersion: string
  sdkName: string
  currentTime: string
  connectionStatus: 'connecting' | 'connected' | 'disconnected'
}

export function ProtoSolDashboard() {
  const [state, setState] = useState<DashboardState>({
    sdkVersion: 'Loading...',
    sdkName: 'Loading...',
    currentTime: '--:--:--',
    connectionStatus: 'connecting'
  })

  const [keypair, setKeypair] = useState<{ publicKey: string; privateKey: string } | null>(null)

  useEffect(() => {
    // Initialize SDK information
    const timerId = setTimeout(() => {
      setState(prev => ({
        ...prev,
        sdkVersion: mockProtoSolApi.VERSION,
        sdkName: mockProtoSolApi.SDK_NAME,
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

  const generateKeypair = () => {
    // This would use the ProtoSol API to generate a keypair
    // For now, we'll simulate it
    setKeypair({
      publicKey: '11111111111111111111111111111112', // Example Solana pubkey
      privateKey: 'secret-private-key-example'
    })
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
          ProtoSol SDK Dashboard
        </h2>
        <p className="text-slate-600">
          Running the latest ProtoSol SDK with Next.js 15
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
          Interactive Demo
        </h3>
        <div className="space-y-4">
          <div className="flex gap-4 items-center">
            <button
              onClick={generateKeypair}
              className="px-6 py-3 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-medium rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
            >
              Generate Keypair
            </button>
            <div className="text-sm text-slate-600">
              Demo ProtoSol SDK integration
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
        </div>
      </div>

      <div className="border-t border-slate-200 pt-8 mt-8">
        <div className="text-center text-slate-500">
          <p className="text-sm mb-2">
            This is a modern Next.js 15 App Router application using the ProtoSol TypeScript SDK.
          </p>
          <p className="text-xs">
            Built with TypeScript, Tailwind CSS, and the latest web technologies.
          </p>
        </div>
      </div>
    </div>
  )
}
