// Main ProtoSol API client class
import { AccountService } from '../services/AccountService'
import { TokenProgramService } from '../services/TokenProgramService'
import { TransactionService } from '../services/TransactionService'
import { SystemProgramService } from '../services/SystemProgramService'

export class ProtoSolAPI {
  private endpoint: string
  private apiKey?: string

  constructor(endpoint: string, apiKey?: string) {
    this.endpoint = endpoint
    this.apiKey = apiKey
  }

  // Service instances
  get account(): { v1: { Service: AccountService } } {
    return {
      v1: {
        Service: new AccountService(this.endpoint, this.apiKey)
      }
    }
  }

  get token(): { program: { v1: { Service: TokenProgramService } } } {
    return {
      program: {
        v1: {
          Service: new TokenProgramService(this.endpoint, this.apiKey)
        }
      }
    }
  }

  get transaction(): { v1: { Service: TransactionService } } {
    return {
      v1: {
        Service: new TransactionService(this.endpoint, this.apiKey)
      }
    }
  }

  get system(): { program: { v1: { Service: SystemProgramService } } } {
    return {
      program: {
        v1: {
          Service: new SystemProgramService(this.endpoint, this.apiKey)
        }
      }
    }
  }
}
