// React Context for ProtoSol API
import React, { createContext, useContext } from 'react'
import { ProtoSolAPI } from './ProtoSolAPI'

// Create the context
const APIContext = createContext<ProtoSolAPI | null>(null)

// Provider component
interface ProtoSolAPIProviderProps {
  children: React.ReactNode
  endpoint?: string
  apiKey?: string
}

export const ProtoSolAPIProvider: React.FC<ProtoSolAPIProviderProps> = ({
  children,
  endpoint = 'http://localhost:50051', // Default gRPC endpoint
  apiKey
}) => {
  const api = new ProtoSolAPI(endpoint, apiKey)

  return (
    <APIContext.Provider value={api}>
      {children}
    </APIContext.Provider>
  )
}

// Hook to use the API context
export const useAPIContext = (): ProtoSolAPI => {
  const context = useContext(APIContext)

  if (!context) {
    throw new Error('useAPIContext must be used within a ProtoSolAPIProvider')
  }

  return context
}
