// Service configuration definitions for all Protochain services
import type { ServiceMethod } from '../components/ServicePage'

// Re-export ServiceMethod type for use in other components
export type { ServiceMethod }

// Account Service Configuration
export const accountServiceConfig = {
  name: 'Account Service v1',
  description: 'Account operations including balance queries, keypair generation, and funding',
  methods: [
    {
      name: 'getAccount',
      displayName: 'Get Account',
      description: 'Fetch account data with commitment level support',
      params: [
        {
          name: 'address',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded account address to fetch from Solana network',
          placeholder: 'e.g. 11111111111111111111111111111112'
        },
        {
          name: 'commitmentLevel',
          type: 'enum' as const,
          required: false,
          description: 'Confirmation level for the account query',
          enumOptions: ['processed', 'confirmed', 'finalized']
        }
      ]
    },
    {
      name: 'generateNewKeyPair',
      displayName: 'Generate New Key Pair',
      description: 'Generate new keypair (deterministic or random)',
      params: [
        {
          name: 'seed',
          type: 'string' as const,
          required: false,
          description: 'Optional seed for deterministic keypair generation',
          placeholder: 'Leave empty for random generation'
        }
      ]
    },
    {
      name: 'fundNative',
      displayName: 'Fund Native (Airdrop)',
      description: 'Fund account with SOL using airdrop (devnet/testnet only)',
      params: [
        {
          name: 'address',
          type: 'string' as const,
          required: true,
          description: 'Target address for funding (Base58)',
          placeholder: 'e.g. 11111111111111111111111111111112'
        },
        {
          name: 'amount',
          type: 'string' as const,
          required: true,
          description: 'Amount in lamports as string (1 SOL = 1,000,000,000 lamports)',
          placeholder: 'e.g. 1000000000 (1 SOL)'
        },
        {
          name: 'commitmentLevel',
          type: 'enum' as const,
          required: false,
          description: 'Confirmation level for funding confirmation',
          enumOptions: ['processed', 'confirmed', 'finalized']
        }
      ]
    }
  ] as ServiceMethod[]
}

// Transaction Service Configuration
export const transactionServiceConfig = {
  name: 'Transaction Service v1',
  description: 'Complete transaction lifecycle management: compile → sign → submit',
  methods: [
    {
      name: 'compileTransaction',
      displayName: 'Compile Transaction',
      description: 'Transform DRAFT transaction to COMPILED (DRAFT → COMPILED)',
      params: [
        {
          name: 'feePayer',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the fee payer account',
          placeholder: 'e.g. 11111111111111111111111111111112'
        },
        {
          name: 'commitmentLevel',
          type: 'enum' as const,
          required: false,
          description: 'Commitment level for blockhash retrieval',
          enumOptions: ['processed', 'confirmed', 'finalized']
        }
      ]
    },
    {
      name: 'estimateTransaction',
      displayName: 'Estimate Transaction',
      description: 'Calculate transaction fees and compute units',
      params: [
        {
          name: 'commitmentLevel',
          type: 'enum' as const,
          required: false,
          description: 'Commitment level for fee calculation',
          enumOptions: ['processed', 'confirmed', 'finalized']
        }
      ]
    },
    {
      name: 'simulateTransaction',
      displayName: 'Simulate Transaction',
      description: 'Dry run the transaction to check for errors',
      params: [
        {
          name: 'commitmentLevel',
          type: 'enum' as const,
          required: false,
          description: 'Commitment level for simulation',
          enumOptions: ['processed', 'confirmed', 'finalized']
        }
      ]
    },
    {
      name: 'signTransaction',
      displayName: 'Sign Transaction', 
      description: 'Add signatures to compiled transaction (COMPILED → SIGNED)',
      params: [
        {
          name: 'privateKeys',
          type: 'string' as const,
          required: true,
          description: 'Comma-separated list of Base58-encoded private keys for signing',
          placeholder: 'e.g. privateKey1,privateKey2'
        }
      ]
    },
    {
      name: 'submitTransaction',
      displayName: 'Submit Transaction',
      description: 'Submit signed transaction to blockchain (SIGNED → SUBMITTED)',
      params: [
        {
          name: 'commitmentLevel',
          type: 'enum' as const,
          required: false,
          description: 'Commitment level for submission confirmation',
          enumOptions: ['processed', 'confirmed', 'finalized']
        }
      ]
    },
    {
      name: 'getTransaction',
      displayName: 'Get Transaction',
      description: 'Fetch transaction details by signature',
      params: [
        {
          name: 'signature',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded transaction signature',
          placeholder: 'e.g. 5VERv8NMvQMB8QC...'
        },
        {
          name: 'commitmentLevel',
          type: 'enum' as const,
          required: false,
          description: 'Commitment level for transaction lookup',
          enumOptions: ['processed', 'confirmed', 'finalized']
        }
      ]
    }
  ] as ServiceMethod[]
}

// System Program Service Configuration
export const systemProgramServiceConfig = {
  name: 'System Program Service v1',
  description: 'Core Solana system program operations - all return composable SolanaInstruction',
  methods: [
    {
      name: 'create',
      displayName: 'Create Account',
      description: 'Create a new Solana account',
      params: [
        {
          name: 'payer',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the account paying for the creation',
          placeholder: 'Payer public key'
        },
        {
          name: 'newAccount',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the account to be created',
          placeholder: 'New account public key'
        },
        {
          name: 'lamports',
          type: 'bigint' as const,
          required: true,
          description: 'Amount of lamports to fund the new account with',
          placeholder: '1000000000'
        },
        {
          name: 'space',
          type: 'bigint' as const,
          required: true,
          description: 'Number of bytes of space to allocate for the account data',
          placeholder: '0'
        },
        {
          name: 'owner',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the program that will own the new account',
          placeholder: '11111111111111111111111111111112'
        }
      ]
    },
    {
      name: 'transfer',
      displayName: 'Transfer SOL',
      description: 'Transfer SOL from one account to another',
      params: [
        {
          name: 'from',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the source account',
          placeholder: 'Source account public key'
        },
        {
          name: 'to',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the destination account',
          placeholder: 'Destination account public key'
        },
        {
          name: 'lamports',
          type: 'bigint' as const,
          required: true,
          description: 'Amount of lamports to transfer',
          placeholder: '1000000000'
        }
      ]
    },
    {
      name: 'allocate',
      displayName: 'Allocate Space',
      description: 'Allocate space for an account',
      params: [
        {
          name: 'account',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of account to allocate space for (must be signer)',
          placeholder: 'Account public key'
        },
        {
          name: 'space',
          type: 'bigint' as const,
          required: true,
          description: 'Number of bytes of memory to allocate',
          placeholder: '0'
        }
      ]
    },
    {
      name: 'assign',
      displayName: 'Assign Owner',
      description: 'Change the owner of an account',
      params: [
        {
          name: 'account',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of account to assign new owner (must be signer)',
          placeholder: 'Account public key'
        },
        {
          name: 'ownerProgram',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the new owner program',
          placeholder: 'New owner program public key'
        }
      ]
    },
    {
      name: 'createWithSeed',
      displayName: 'Create Account With Seed',
      description: 'Create a new account with a seed',
      params: [
        {
          name: 'payer',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the account paying for creation (must be signer)',
          placeholder: 'Payer public key'
        },
        {
          name: 'newAccount',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the new account to be created',
          placeholder: 'New account public key'
        },
        {
          name: 'base',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded base public key used to derive the new account address',
          placeholder: 'Base public key'
        },
        {
          name: 'seed',
          type: 'string' as const,
          required: true,
          description: 'Seed string used to derive the new account address',
          placeholder: 'Account seed'
        },
        {
          name: 'lamports',
          type: 'bigint' as const,
          required: true,
          description: 'Amount of lamports to transfer to the new account',
          placeholder: '1000000000'
        },
        {
          name: 'space',
          type: 'bigint' as const,
          required: true,
          description: 'Number of bytes of memory to allocate for the account',
          placeholder: '0'
        }
      ]
    },
    {
      name: 'allocateWithSeed',
      displayName: 'Allocate Space With Seed',
      description: 'Allocate space for an account using seed derivation',
      params: [
        {
          name: 'account',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of account to allocate space for',
          placeholder: 'Account public key'
        },
        {
          name: 'base',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded base public key used for derivation',
          placeholder: 'Base public key'
        },
        {
          name: 'seed',
          type: 'string' as const,
          required: true,
          description: 'Seed string used for account derivation',
          placeholder: 'Account seed'
        },
        {
          name: 'space',
          type: 'bigint' as const,
          required: true,
          description: 'Number of bytes of memory to allocate',
          placeholder: '0'
        }
      ]
    },
    {
      name: 'assignWithSeed',
      displayName: 'Assign Owner With Seed',
      description: 'Change account owner using seed derivation',
      params: [
        {
          name: 'account',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of account to assign new owner',
          placeholder: 'Account public key'
        },
        {
          name: 'base',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded base public key used for derivation',
          placeholder: 'Base public key'
        },
        {
          name: 'seed',
          type: 'string' as const,
          required: true,
          description: 'Seed string used for account derivation',
          placeholder: 'Account seed'
        },
        {
          name: 'ownerProgram',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the new owner program',
          placeholder: 'New owner program public key'
        }
      ]
    },
    {
      name: 'transferWithSeed',
      displayName: 'Transfer SOL With Seed',
      description: 'Transfer SOL from seed-derived account',
      params: [
        {
          name: 'from',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the source account',
          placeholder: 'Source account public key'
        },
        {
          name: 'fromBase',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded base public key for source account derivation',
          placeholder: 'Source base public key'
        },
        {
          name: 'fromSeed',
          type: 'string' as const,
          required: true,
          description: 'Seed string for source account derivation',
          placeholder: 'Source account seed'
        },
        {
          name: 'to',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the destination account',
          placeholder: 'Destination account public key'
        },
        {
          name: 'lamports',
          type: 'bigint' as const,
          required: true,
          description: 'Amount of lamports to transfer',
          placeholder: '1000000000'
        }
      ]
    },
    {
      name: 'initializeNonceAccount',
      displayName: 'Initialize Nonce Account',
      description: 'Initialize a nonce account for durable transactions',
      params: [
        {
          name: 'nonceAccount',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of nonce account to initialize',
          placeholder: 'Nonce account public key'
        },
        {
          name: 'authority',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of nonce authority',
          placeholder: 'Authority public key'
        }
      ]
    },
    {
      name: 'authorizeNonceAccount',
      displayName: 'Authorize Nonce Account',
      description: 'Change the authority of a nonce account',
      params: [
        {
          name: 'nonceAccount',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of nonce account',
          placeholder: 'Nonce account public key'
        },
        {
          name: 'currentAuthority',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of current authority (must be signer)',
          placeholder: 'Current authority public key'
        },
        {
          name: 'newAuthority',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of new authority',
          placeholder: 'New authority public key'
        }
      ]
    },
    {
      name: 'withdrawNonceAccount',
      displayName: 'Withdraw from Nonce Account',
      description: 'Withdraw lamports from a nonce account',
      params: [
        {
          name: 'nonceAccount',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of nonce account',
          placeholder: 'Nonce account public key'
        },
        {
          name: 'authority',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of nonce authority (must be signer)',
          placeholder: 'Authority public key'
        },
        {
          name: 'to',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of destination account',
          placeholder: 'Destination account public key'
        },
        {
          name: 'lamports',
          type: 'bigint' as const,
          required: true,
          description: 'Amount of lamports to withdraw',
          placeholder: '1000000000'
        }
      ]
    },
    {
      name: 'advanceNonceAccount',
      displayName: 'Advance Nonce Account',
      description: 'Advance the nonce value in a nonce account',
      params: [
        {
          name: 'nonceAccount',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of nonce account',
          placeholder: 'Nonce account public key'
        },
        {
          name: 'authority',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of nonce authority (must be signer)',
          placeholder: 'Authority public key'
        }
      ]
    },
    {
      name: 'upgradeNonceAccount',
      displayName: 'Upgrade Nonce Account',
      description: 'Upgrade a nonce account to a newer version',
      params: [
        {
          name: 'nonceAccount',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of nonce account to upgrade',
          placeholder: 'Nonce account public key'
        }
      ]
    }
  ] as ServiceMethod[]
}

// Token Program Service Configuration
export const tokenProgramServiceConfig = {
  name: 'Token Program Service v1',
  description: 'SPL Token 2022 program operations for mint and account management',
  methods: [
    {
      name: 'initialiseMint',
      displayName: 'Initialize Mint',
      description: 'Creates an InitializeMint instruction for Token 2022 program',
      params: [
        {
          name: 'mintPubKey',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the mint account to initialize',
          placeholder: 'Mint account public key'
        },
        {
          name: 'mintAuthorityPubKey',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded public key of the mint authority',
          placeholder: 'Mint authority public key'
        },
        {
          name: 'freezeAuthorityPubKey',
          type: 'string' as const,
          required: false,
          description: 'Optional base58-encoded public key of the freeze authority',
          placeholder: 'Freeze authority public key (optional)'
        },
        {
          name: 'decimals',
          type: 'number' as const,
          required: true,
          description: 'Number of decimal places for the token (0-9)',
          placeholder: '9'
        }
      ]
    },
    {
      name: 'getCurrentMinRentForTokenAccount',
      displayName: 'Get Min Rent for Token Account',
      description: 'Calculate minimum rent required for a token account (mint size)',
      params: []
    },
    {
      name: 'parseMint',
      displayName: 'Parse Mint Account',
      description: 'Parse mint account data into structured format',
      params: [
        {
          name: 'accountAddress',
          type: 'string' as const,
          required: true,
          description: 'Base58-encoded address of mint account to parse',
          placeholder: 'Mint account address'
        }
      ]
    },
  ] as ServiceMethod[]
}

// RPC Client Service Configuration
export const rpcClientServiceConfig = {
  name: 'RPC Client Service v1',
  description: 'Direct Solana RPC client calls with minimal viable operations',
  methods: [
    {
      name: 'getMinimumBalanceForRentExemption',
      displayName: 'Get Minimum Balance for Rent Exemption',
      description: 'Calculate minimum balance required for rent exemption based on data length',
      params: [
        {
          name: 'dataLength',
          type: 'bigint' as const,
          required: true,
          description: 'Length of data that will be stored in the account',
          placeholder: '0'
        }
      ]
    }
  ] as ServiceMethod[]
}