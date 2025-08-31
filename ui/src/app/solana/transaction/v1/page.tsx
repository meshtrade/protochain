'use client'

import { useState } from 'react'
import ServicePage from '../../../../components/ServicePage'
import { transactionServiceConfig, systemProgramServiceConfig, tokenProgramServiceConfig, ServiceMethod } from '../../../../lib/service-configs'

// Transaction states based on ProtoSol state machine
enum TransactionState {
  DRAFT = 'DRAFT',
  COMPILED = 'COMPILED', 
  PARTIALLY_SIGNED = 'PARTIALLY_SIGNED',
  FULLY_SIGNED = 'FULLY_SIGNED',
  SUBMITTED = 'SUBMITTED'
}

// Draft transaction interface
interface DraftTransaction {
  id: string
  state: TransactionState
  instructions: SolanaInstruction[]
  feePayer?: string
  recentBlockhash?: string
  signatures?: string[]
  transactionSignature?: string
  createdAt: Date
  compiledData?: string
  compiledAt?: Date
}

// Placeholder instruction interface (will be replaced by actual ProtoSol types)
interface SolanaInstruction {
  programId: string
  accounts: Array<{
    publicKey: string
    isSigner: boolean
    isWritable: boolean
  }>
  data: string
  description?: string
}

// Program service configurations map
const programServices = {
  'system': {
    name: 'System Program',
    config: systemProgramServiceConfig
  },
  'token': {
    name: 'Token Program',
    config: tokenProgramServiceConfig
  }
} as const

type ProgramType = keyof typeof programServices

export default function TransactionV1Page() {
  // Draft transaction state management
  const [currentTransaction, setCurrentTransaction] = useState<DraftTransaction | null>(null)
  const [showCreateForm, setShowCreateForm] = useState(false)
  
  // Instruction building state
  const [showAddInstruction, setShowAddInstruction] = useState(false)
  const [selectedProgram, setSelectedProgram] = useState<ProgramType | null>(null)
  const [selectedMethod, setSelectedMethod] = useState<ServiceMethod | null>(null)
  const [instructionFormData, setInstructionFormData] = useState<Record<string, any>>({})
  
  // Transaction compilation state
  const [showCompileForm, setShowCompileForm] = useState(false)
  const [compileFormData, setCompileFormData] = useState<{
    feePayer?: string
    recentBlockhash?: string
  }>({})
  const [compileLoading, setCompileLoading] = useState(false)
  const [compileError, setCompileError] = useState<string | null>(null)
  
  // Create new draft transaction
  const createDraftTransaction = () => {
    const newTransaction: DraftTransaction = {
      id: `tx_${Date.now()}`,
      state: TransactionState.DRAFT,
      instructions: [],
      createdAt: new Date()
    }
    setCurrentTransaction(newTransaction)
    setShowCreateForm(false)
  }
  
  // Clear current transaction
  const clearTransaction = () => {
    setCurrentTransaction(null)
    setShowCreateForm(false)
  }
  
  // Handle program selection
  const handleProgramSelect = (program: ProgramType) => {
    setSelectedProgram(program)
    setSelectedMethod(null)
    setInstructionFormData({})
  }
  
  // Handle method selection
  const handleMethodSelect = (method: ServiceMethod) => {
    setSelectedMethod(method)
    setInstructionFormData({})
  }
  
  // Handle instruction form input changes
  const handleInstructionInputChange = (paramName: string, value: any) => {
    setInstructionFormData(prev => ({
      ...prev,
      [paramName]: value
    }))
  }
  
  // Add instruction to draft transaction
  const addInstructionToTransaction = async () => {
    if (!currentTransaction || !selectedMethod) return
    
    try {
      // Convert form data to proper types for API call
      const processedData: Record<string, any> = {}
      
      for (const param of selectedMethod.params) {
        const value = instructionFormData[param.name]
        
        if (param.required && (value === undefined || value === '')) {
          throw new Error(`${param.name} is required`)
        }

        if (value !== undefined && value !== '') {
          switch (param.type) {
            case 'number':
              processedData[param.name] = Number(value)
              break
            case 'bigint':
              processedData[param.name] = BigInt(value as string).toString()
              break
            default:
              processedData[param.name] = value
          }
        }
      }
      
      // Call the program service API
      const response = await fetch(selectedMethod.endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(processedData),
      })
      
      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.error || `HTTP ${response.status}`)
      }
      
      const result = await response.json()
      
      // Create a SolanaInstruction from the API response
      const newInstruction: SolanaInstruction = {
        programId: result.instruction?.programId || 'Unknown',
        accounts: result.instruction?.accounts || [],
        data: result.instruction?.data || '',
        description: `${selectedMethod.displayName}: ${selectedProgram} program operation`
      }
      
      // Add instruction to current transaction
      setCurrentTransaction(prev => ({
        ...prev!,
        instructions: [...prev!.instructions, newInstruction]
      }))
      
      // Reset instruction building form
      setShowAddInstruction(false)
      setSelectedProgram(null)
      setSelectedMethod(null)
      setInstructionFormData({})
      
    } catch (error) {
      console.error('Error adding instruction:', error)
      alert(`Failed to add instruction: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }

  // Compile transaction function
  const compileTransaction = async () => {
    if (!currentTransaction || !compileFormData.feePayer) {
      return
    }

    setCompileLoading(true)
    setCompileError(null)

    try {
      // Convert local transaction to ProtoSol Transaction format
      const protoTransaction = {
        instructions: currentTransaction.instructions.map(instruction => ({
          programId: instruction.programId,
          accounts: instruction.accounts,
          data: instruction.data
        })),
        state: 1, // TRANSACTION_STATE_DRAFT
        config: {},
        data: '',
        feePayer: '',
        recentBlockhash: '',
        signatures: [],
        hash: '',
        signature: ''
      }

      const response = await fetch('/api/transaction/compile', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          transaction: protoTransaction,
          feePayer: compileFormData.feePayer,
          recentBlockhash: compileFormData.recentBlockhash || ''
        }),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.error || 'Failed to compile transaction')
      }

      const result = await response.json()
      
      // Update current transaction state to COMPILED
      setCurrentTransaction(prev => ({
        ...prev!,
        state: TransactionState.COMPILED,
        feePayer: result.transaction.feePayer,
        recentBlockhash: result.transaction.recentBlockhash,
        signatures: result.transaction.signatures,
        // Add compiled transaction data
        compiledData: result.transaction.data,
        compiledAt: new Date()
      }))

      // Clear form
      setShowCompileForm(false)
      setCompileFormData({})
      
      // Success feedback
      alert('Transaction compiled successfully! It is now in COMPILED state.')
      
    } catch (error) {
      console.error('Error compiling transaction:', error)
      setCompileError(error instanceof Error ? error.message : 'Unknown compilation error')
    } finally {
      setCompileLoading(false)
    }
  }

  return (
    <div className="space-y-6">
      {/* Service Header */}
      <div>
        <h1 className="text-2xl font-bold text-slate-900">Transaction Service v1</h1>
        <p className="mt-1 text-sm text-slate-600">
          Complete transaction lifecycle management: compile → sign → submit
        </p>
      </div>

      {/* Transaction State Machine Overview */}
      <div className="bg-amber-50 border border-amber-200 rounded-lg p-4">
        <h2 className="text-sm font-medium text-amber-800 mb-2">Transaction State Machine</h2>
        <div className="flex items-center space-x-2 text-xs text-amber-700">
          <span className="px-2 py-1 bg-amber-100 rounded">DRAFT</span>
          <span>→</span>
          <span className="px-2 py-1 bg-amber-100 rounded">COMPILED</span>
          <span>→</span>
          <span className="px-2 py-1 bg-amber-100 rounded">SIGNED</span>
          <span>→</span>
          <span className="px-2 py-1 bg-amber-100 rounded">SUBMITTED</span>
        </div>
      </div>

      {/* Draft Transaction Management */}
      <div className="bg-white shadow rounded-lg p-6">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg font-medium text-slate-900">Draft Transaction</h2>
          <div className="space-x-2">
            {!currentTransaction && (
              <button
                onClick={() => setShowCreateForm(!showCreateForm)}
                className="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              >
                {showCreateForm ? 'Cancel' : 'Create Draft Transaction'}
              </button>
            )}
            {currentTransaction && (
              <button
                onClick={clearTransaction}
                className="px-4 py-2 bg-red-600 text-white text-sm font-medium rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
              >
                Clear Transaction
              </button>
            )}
          </div>
        </div>

        {/* Create Draft Form */}
        {showCreateForm && !currentTransaction && (
          <div className="border border-slate-200 rounded-md p-4 mb-4">
            <h3 className="text-sm font-medium text-slate-900 mb-3">Create New Draft Transaction</h3>
            <p className="text-xs text-slate-600 mb-4">
              A draft transaction allows you to add instructions from program services before compilation.
            </p>
            <button
              onClick={createDraftTransaction}
              className="px-3 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
            >
              Create Draft Transaction
            </button>
          </div>
        )}

        {/* Current Transaction Display */}
        {currentTransaction ? (
          <div className="space-y-4">
            {/* Transaction Info */}
            <div className="grid grid-cols-2 gap-4 p-4 bg-slate-50 rounded-md">
              <div>
                <dt className="text-xs font-medium text-slate-500">Transaction ID</dt>
                <dd className="text-sm text-slate-900 font-mono">{currentTransaction.id}</dd>
              </div>
              <div>
                <dt className="text-xs font-medium text-slate-500">State</dt>
                <dd className="text-sm">
                  <span className={`inline-flex px-2 py-1 text-xs font-medium rounded-full ${
                    currentTransaction.state === TransactionState.DRAFT 
                      ? 'bg-blue-100 text-blue-800'
                      : currentTransaction.state === TransactionState.COMPILED
                      ? 'bg-yellow-100 text-yellow-800'
                      : currentTransaction.state === TransactionState.FULLY_SIGNED
                      ? 'bg-green-100 text-green-800'
                      : 'bg-purple-100 text-purple-800'
                  }`}>
                    {currentTransaction.state}
                  </span>
                </dd>
              </div>
              <div>
                <dt className="text-xs font-medium text-slate-500">Instructions</dt>
                <dd className="text-sm text-slate-900">{currentTransaction.instructions.length}</dd>
              </div>
              <div>
                <dt className="text-xs font-medium text-slate-500">Created</dt>
                <dd className="text-sm text-slate-900">{currentTransaction.createdAt.toLocaleTimeString()}</dd>
              </div>
              {/* Compiled Transaction Details */}
              {currentTransaction.state === TransactionState.COMPILED && (
                <>
                  <div>
                    <dt className="text-xs font-medium text-slate-500">Fee Payer</dt>
                    <dd className="text-sm text-slate-900 font-mono">
                      {currentTransaction.feePayer ? `${currentTransaction.feePayer.slice(0, 8)}...${currentTransaction.feePayer.slice(-8)}` : 'N/A'}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-xs font-medium text-slate-500">Recent Blockhash</dt>
                    <dd className="text-sm text-slate-900 font-mono">
                      {currentTransaction.recentBlockhash ? `${currentTransaction.recentBlockhash.slice(0, 8)}...${currentTransaction.recentBlockhash.slice(-8)}` : 'N/A'}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-xs font-medium text-slate-500">Compiled Data</dt>
                    <dd className="text-sm text-slate-900 font-mono">
                      {currentTransaction.compiledData ? `${currentTransaction.compiledData.slice(0, 8)}... (${currentTransaction.compiledData.length} bytes)` : 'N/A'}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-xs font-medium text-slate-500">Compiled At</dt>
                    <dd className="text-sm text-slate-900">
                      {currentTransaction.compiledAt ? currentTransaction.compiledAt.toLocaleTimeString() : 'N/A'}
                    </dd>
                  </div>
                </>
              )}
            </div>

            {/* Instructions List */}
            <div>
              <h3 className="text-sm font-medium text-slate-900 mb-3">Instructions</h3>
              {currentTransaction.instructions.length === 0 ? (
                <div className="text-center py-6 border-2 border-dashed border-slate-300 rounded-md">
                  <p className="text-sm text-slate-500">No instructions added yet</p>
                  <p className="text-xs text-slate-400 mt-1">
                    Use Program Services to add instructions to this transaction
                  </p>
                </div>
              ) : (
                <div className="space-y-2">
                  {currentTransaction.instructions.map((instruction, index) => (
                    <div key={index} className="border border-slate-200 rounded-md p-3">
                      <div className="flex justify-between items-start">
                        <div>
                          <h4 className="text-sm font-medium text-slate-900">
                            Instruction {index + 1}
                          </h4>
                          {instruction.description && (
                            <p className="text-xs text-slate-600 mt-1">{instruction.description}</p>
                          )}
                        </div>
                        <span className="text-xs text-slate-400 font-mono">
                          {instruction.programId.slice(0, 8)}...
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Quick Actions */}
            {currentTransaction.instructions.length > 0 && currentTransaction.state === TransactionState.DRAFT && (
              <div className="bg-blue-50 border border-blue-200 rounded-md p-3">
                <p className="text-xs text-blue-800 mb-2">
                  <span className="font-medium">Next Steps:</span> Your transaction has instructions and is ready for compilation.
                </p>
                <p className="text-xs text-blue-600">
                  Use the &quot;Compile Transaction&quot; method below to proceed to the COMPILED state.
                </p>
              </div>
            )}
          </div>
        ) : (
          <div className="text-center py-6">
            <p className="text-sm text-slate-500">No draft transaction</p>
            <p className="text-xs text-slate-400 mt-1">
              Create a draft transaction to begin the transaction lifecycle
            </p>
          </div>
        )}
      </div>

      {/* Add Instruction Section */}
      {currentTransaction && currentTransaction.state === TransactionState.DRAFT && (
        <div className="bg-white shadow rounded-lg p-6">
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-lg font-medium text-slate-900">Add Instruction</h2>
            <button
              onClick={() => setShowAddInstruction(!showAddInstruction)}
              className="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
            >
              {showAddInstruction ? 'Cancel' : 'Add Instruction'}
            </button>
          </div>

          {showAddInstruction && (
            <div className="space-y-4 border border-slate-200 rounded-md p-4">
              {/* Program Selector */}
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Select Program
                </label>
                <div className="grid grid-cols-2 gap-2">
                  {Object.entries(programServices).map(([key, service]) => (
                    <button
                      key={key}
                      onClick={() => handleProgramSelect(key as ProgramType)}
                      className={`p-3 text-left border rounded-md transition-colors ${
                        selectedProgram === key
                          ? 'border-blue-500 bg-blue-50 text-blue-900'
                          : 'border-slate-300 hover:border-slate-400'
                      }`}
                    >
                      <div className="font-medium text-sm">{service.name}</div>
                      <div className="text-xs text-slate-600 mt-1">
                        {service.config.methods.length} method{service.config.methods.length !== 1 ? 's' : ''}
                      </div>
                    </button>
                  ))}
                </div>
              </div>

              {/* Method Selector */}
              {selectedProgram && (
                <div>
                  <label className="block text-sm font-medium text-slate-700 mb-2">
                    Select Method
                  </label>
                  <div className="space-y-2">
                    {programServices[selectedProgram].config.methods.map((method) => (
                      <button
                        key={method.name}
                        onClick={() => handleMethodSelect(method)}
                        className={`w-full p-3 text-left border rounded-md transition-colors ${
                          selectedMethod?.name === method.name
                            ? 'border-blue-500 bg-blue-50 text-blue-900'
                            : 'border-slate-300 hover:border-slate-400'
                        }`}
                      >
                        <div className="font-medium text-sm">{method.displayName}</div>
                        <div className="text-xs text-slate-600 mt-1">{method.description}</div>
                      </button>
                    ))}
                  </div>
                </div>
              )}

              {/* Method Parameters Form */}
              {selectedMethod && (
                <div>
                  <label className="block text-sm font-medium text-slate-700 mb-3">
                    {selectedMethod.displayName} Parameters
                  </label>
                  <div className="space-y-3">
                    {selectedMethod.params.map((param: any) => (
                      <div key={param.name}>
                        <label className="block text-xs font-medium text-slate-700 mb-1">
                          {param.name}
                          {param.required && <span className="text-red-500 ml-1">*</span>}
                        </label>
                        {param.description && (
                          <p className="text-xs text-slate-500 mb-2">{param.description}</p>
                        )}
                        
                        {param.type === 'enum' ? (
                          <select
                            value={instructionFormData[param.name] || ''}
                            onChange={(e) => handleInstructionInputChange(param.name, e.target.value)}
                            className="block w-full px-3 py-1 text-sm border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            required={param.required}
                          >
                            <option value="">Select {param.name}</option>
                            {param.enumOptions?.map((option: string) => (
                              <option key={option} value={option}>
                                {option}
                              </option>
                            ))}
                          </select>
                        ) : param.type === 'boolean' ? (
                          <div className="flex items-center">
                            <input
                              type="checkbox"
                              checked={Boolean(instructionFormData[param.name])}
                              onChange={(e) => handleInstructionInputChange(param.name, e.target.checked)}
                              className="rounded border-slate-300 text-blue-600 focus:ring-blue-500"
                            />
                            <span className="ml-2 text-xs text-slate-600">{param.description}</span>
                          </div>
                        ) : param.type === 'number' || param.type === 'bigint' ? (
                          <input
                            type="number"
                            value={instructionFormData[param.name] || ''}
                            onChange={(e) => handleInstructionInputChange(param.name, e.target.value)}
                            placeholder={param.placeholder}
                            className="block w-full px-3 py-1 text-sm border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            required={param.required}
                          />
                        ) : (
                          <input
                            type="text"
                            value={instructionFormData[param.name] || ''}
                            onChange={(e) => handleInstructionInputChange(param.name, e.target.value)}
                            placeholder={param.placeholder}
                            className="block w-full px-3 py-1 text-sm border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            required={param.required}
                          />
                        )}
                      </div>
                    ))}
                    
                    <div className="pt-3">
                      <button
                        onClick={addInstructionToTransaction}
                        className="w-full px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                      >
                        Add to Transaction
                      </button>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Transaction Compilation Section */}
      {currentTransaction && currentTransaction.state === TransactionState.DRAFT && currentTransaction.instructions.length > 0 && (
        <div className="bg-white shadow rounded-lg p-6">
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-lg font-medium text-slate-900">Compile Transaction</h2>
            <button
              onClick={() => setShowCompileForm(!showCompileForm)}
              className="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              {showCompileForm ? 'Cancel' : 'Compile Transaction'}
            </button>
          </div>

          {showCompileForm && (
            <div className="space-y-4 border border-slate-200 rounded-md p-4">
              <div className="bg-amber-50 border border-amber-200 rounded-md p-3 mb-4">
                <p className="text-xs text-amber-800">
                  <span className="font-medium">Ready for Compilation:</span> Your transaction has {currentTransaction.instructions.length} instruction{currentTransaction.instructions.length !== 1 ? 's' : ''} and is ready to be compiled.
                </p>
                <p className="text-xs text-amber-700 mt-1">
                  Compilation will transition the transaction from DRAFT → COMPILED state.
                </p>
              </div>

              {/* Fee Payer Input */}
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Fee Payer *
                </label>
                <input
                  type="text"
                  value={compileFormData.feePayer || ''}
                  onChange={(e) => setCompileFormData(prev => ({ ...prev, feePayer: e.target.value }))}
                  placeholder="Base58-encoded public key of the fee payer"
                  className="block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  required
                />
                <p className="text-xs text-slate-600 mt-1">
                  The account that will pay for transaction fees and rent costs
                </p>
              </div>

              {/* Recent Blockhash Input (Optional) */}
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Recent Blockhash (Optional)
                </label>
                <input
                  type="text"
                  value={compileFormData.recentBlockhash || ''}
                  onChange={(e) => setCompileFormData(prev => ({ ...prev, recentBlockhash: e.target.value }))}
                  placeholder="Leave empty to fetch automatically"
                  className="block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
                <p className="text-xs text-slate-600 mt-1">
                  Recent blockhash for transaction validation. Will be fetched automatically if not provided.
                </p>
              </div>

              {/* Compile Button */}
              <div className="pt-3">
                <button
                  onClick={compileTransaction}
                  disabled={!compileFormData.feePayer || compileLoading}
                  className="w-full px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {compileLoading ? 'Compiling...' : 'Compile Transaction'}
                </button>
              </div>

              {/* Compilation Results */}
              {compileError && (
                <div className="bg-red-50 border border-red-200 rounded-md p-3">
                  <p className="text-sm text-red-800 font-medium">Compilation Failed</p>
                  <p className="text-xs text-red-700 mt-1">{compileError}</p>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Transaction Service Methods */}
      <ServicePage
        serviceName={transactionServiceConfig.name}
        serviceDescription="Transaction lifecycle operations - requires an active draft transaction for most operations"
        methods={transactionServiceConfig.methods}
      />
    </div>
  )
}