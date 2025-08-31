// Server-side service factories without React dependencies
import { AccountService } from './services/AccountService'
import { TokenProgramService } from './services/TokenProgramService'

// Factory functions to create service instances on the server-side
export function createAccountService(endpoint: string): AccountService {
  return new AccountService(endpoint)
}

export function createTokenProgramService(endpoint: string): TokenProgramService {
  return new TokenProgramService(endpoint)
}

// Export the original types without React dependencies
export type {
  GenerateNewKeyPairRequest,
  GetAccountRequest
} from './types'

export type {
  InitialiseMintRequest
} from './types'
