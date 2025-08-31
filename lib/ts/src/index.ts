// ProtoSol TypeScript SDK
export const VERSION = '1.0.0'
export const SDK_NAME = 'ProtoSol TypeScript SDK'

// Import all services and types
export { ProtoSolAPI } from './api/ProtoSolAPI'
export { ProtoSolAPIProvider } from './api/ProtoSolAPIContext'
export { useAPIContext } from './api/useAPIContext'

// Service clients
export { AccountService } from './services/AccountService'
export { TokenProgramService } from './services/TokenProgramService'
export { TransactionService } from './services/TransactionService'
export { SystemProgramService } from './services/SystemProgramService'

// Generated types (would normally be from protobuf generation)
export * from './types'
