'use client'

import { useState } from 'react'
import ServicePage from '../../../../components/ServicePage'
import { transactionServiceConfig, systemProgramServiceConfig, tokenProgramServiceConfig, ServiceMethod } from '../../../../lib/service-configs'
import { 
  compileTransactionAction,
  estimateTransactionAction,
  simulateTransactionAction,
  signTransactionAction,
  submitTransactionAction,
  getTransactionAction
} from '../../../../lib/actions/transaction-actions'

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
  signedAt?: Date
  submittedAt?: Date
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
}

type ProgramType = keyof typeof programServices

export default function TransactionServicePage() {
  // Main transaction state
  const [currentTransaction, setCurrentTransaction] = useState<DraftTransaction | null>(null)
  const [showCreateForm, setShowCreateForm] = useState(false)
  
  // Instruction building state
  const [showAddInstruction, setShowAddInstruction] = useState(false)
  const [selectedProgram, setSelectedProgram] = useState<ProgramType | null>(null)
  const [selectedMethod, setSelectedMethod] = useState<ServiceMethod | null>(null)
  
  // Transaction compilation state
  const [showCompileForm, setShowCompileForm] = useState(false)
  const [compileFormData, setCompileFormData] = useState<{
    feePayer?: string
    recentBlockhash?: string
  }>({})
  const [compileLoading, setCompileLoading] = useState(false)
  const [compileError, setCompileError] = useState<string | null>(null)
  
  // Transaction estimation and simulation state
  const [showAnalysisSection, setShowAnalysisSection] = useState(false)
  const [analysisCommitmentLevel, setAnalysisCommitmentLevel] = useState<string>('confirmed')
  const [estimateLoading, setEstimateLoading] = useState(false)
  const [estimateResult, setEstimateResult] = useState<{
    computeUnits: string
    feeLamports: string
    priorityFee: string
  } | null>(null)
  const [estimateError, setEstimateError] = useState<string | null>(null)
  const [simulateLoading, setSimulateLoading] = useState(false)
  const [simulateResult, setSimulateResult] = useState<{
    success: boolean
    error: string
    logs: string[]
  } | null>(null)
  const [simulateError, setSimulateError] = useState<string | null>(null)
  
  // Transaction signing state
  const [showSigningSection, setShowSigningSection] = useState(false)
  const [privateKeys, setPrivateKeys] = useState<string[]>([''])
  const [signingLoading, setSigningLoading] = useState(false)
  const [signingError, setSigningError] = useState<string | null>(null)
  const [signingResult, setSigningResult] = useState<{
    signaturesAdded: number
    totalSignatures: number
  } | null>(null)
  
  // Transaction submission state
  const [showSubmissionSection, setShowSubmissionSection] = useState(false)
  const [submissionCommitmentLevel, setSubmissionCommitmentLevel] = useState<string>('confirmed')
  const [submissionLoading, setSubmissionLoading] = useState(false)
  const [submissionError, setSubmissionError] = useState<string | null>(null)
  const [submissionResult, setSubmissionResult] = useState<{
    transactionSignature: string
    submittedAt: Date
  } | null>(null)
  
  // Transaction monitoring state  
  const [showMonitoringSection, setShowMonitoringSection] = useState(false)
  const [monitoringCommitmentLevel, setMonitoringCommitmentLevel] = useState<string>('confirmed')
  const [lookupSignature, setLookupSignature] = useState<string>('')
  const [lookupLoading, setLookupLoading] = useState(false)
  const [lookupError, setLookupError] = useState<string | null>(null)
  const [lookupResult, setLookupResult] = useState<any | null>(null)

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
  }
  
  // Handle method selection
  const handleMethodSelect = (method: ServiceMethod) => {
    setSelectedMethod(method)
  }
  

  // Compile transaction using server action
  const compileTransaction = async () => {
    if (!currentTransaction || currentTransaction.state !== TransactionState.DRAFT) {
      return
    }

    setCompileLoading(true)
    setCompileError(null)

    try {
      const formData = new FormData()
      formData.append('transaction', JSON.stringify({
        id: currentTransaction.id,
        instructions: currentTransaction.instructions.map(instruction => ({
          programId: instruction.programId,
          accounts: instruction.accounts,
          data: instruction.data
        })),
        state: 1, // TRANSACTION_STATE_DRAFT
        config: {},
        data: '',
        signatures: [],
        hash: '',
        signature: ''
      }))
      formData.append('feePayer', compileFormData.feePayer || '')
      formData.append('recentBlockhash', compileFormData.recentBlockhash || '')

      // Call server action instead of fetch
      const result = await compileTransactionAction(formData)

      if (result.error) {
        throw new Error(result.error)
      }

      // Update transaction state to COMPILED
      setCurrentTransaction(prev => ({
        ...prev!,
        state: TransactionState.COMPILED,
        feePayer: compileFormData.feePayer,
        recentBlockhash: compileFormData.recentBlockhash,
        compiledData: JSON.stringify(result.transaction || {}),
        compiledAt: new Date()
      }))

      // Clear form
      setShowCompileForm(false)
      setCompileFormData({})
      alert('Transaction compiled successfully!')
      
    } catch (error) {
      console.error('Error compiling transaction:', error)
      setCompileError(error instanceof Error ? error.message : 'Unknown compilation error')
    } finally {
      setCompileLoading(false)
    }
  }

  // Estimate transaction using server action
  const estimateTransaction = async () => {
    if (!currentTransaction || currentTransaction.state !== TransactionState.COMPILED) {
      return
    }

    setEstimateLoading(true)
    setEstimateError(null)

    try {
      const formData = new FormData()
      formData.append('transaction', JSON.stringify({
        instructions: currentTransaction.instructions.map(instruction => ({
          programId: instruction.programId,
          accounts: instruction.accounts,
          data: instruction.data
        })),
        state: 2, // TRANSACTION_STATE_COMPILED
        config: {},
        data: currentTransaction.compiledData || '',
        feePayer: currentTransaction.feePayer || '',
        recentBlockhash: currentTransaction.recentBlockhash || '',
        signatures: [],
        hash: '',
        signature: ''
      }))
      formData.append('commitmentLevel', analysisCommitmentLevel)

      // Call server action instead of fetch
      const result = await estimateTransactionAction(formData)

      if (result.error) {
        throw new Error(result.error)
      }

      setEstimateResult({
        computeUnits: result.computeUnits || '0',
        feeLamports: result.feeLamports || '0',
        priorityFee: result.priorityFee || '0'
      })
      
    } catch (error) {
      console.error('Error estimating transaction:', error)
      setEstimateError(error instanceof Error ? error.message : 'Unknown estimation error')
    } finally {
      setEstimateLoading(false)
    }
  }

  // Simulate transaction using server action
  const simulateTransaction = async () => {
    if (!currentTransaction || currentTransaction.state !== TransactionState.COMPILED) {
      return
    }

    setSimulateLoading(true)
    setSimulateError(null)

    try {
      const formData = new FormData()
      formData.append('transaction', JSON.stringify({
        instructions: currentTransaction.instructions.map(instruction => ({
          programId: instruction.programId,
          accounts: instruction.accounts,
          data: instruction.data
        })),
        state: 2, // TRANSACTION_STATE_COMPILED
        config: {},
        data: currentTransaction.compiledData || '',
        feePayer: currentTransaction.feePayer || '',
        recentBlockhash: currentTransaction.recentBlockhash || '',
        signatures: [],
        hash: '',
        signature: ''
      }))
      formData.append('commitmentLevel', analysisCommitmentLevel)

      // Call server action instead of fetch
      const result = await simulateTransactionAction(formData)

      if (result.error) {
        throw new Error(result.error)
      }

      setSimulateResult({
        success: result.simulationSuccess || false,
        error: result.error || '',
        logs: result.logs || []
      })
      
    } catch (error) {
      console.error('Error simulating transaction:', error)
      setSimulateError(error instanceof Error ? error.message : 'Unknown simulation error')
    } finally {
      setSimulateLoading(false)
    }
  }

  // Add private key field for multi-sig
  const addPrivateKeyField = () => {
    setPrivateKeys(prev => [...prev, ''])
  }

  const removePrivateKeyField = (index: number) => {
    if (privateKeys.length > 1) {
      setPrivateKeys(prev => prev.filter((_, i) => i !== index))
    }
  }

  const updatePrivateKey = (index: number, value: string) => {
    setPrivateKeys(prev => prev.map((key, i) => i === index ? value : key))
  }

  // Sign transaction using server action
  const signTransaction = async () => {
    if (!currentTransaction || currentTransaction.state !== TransactionState.COMPILED) {
      return
    }

    // Filter out empty private keys
    const validPrivateKeys = privateKeys.filter(key => key.trim().length > 0)
    if (validPrivateKeys.length === 0) {
      setSigningError('At least one private key is required for signing')
      return
    }

    setSigningLoading(true)
    setSigningError(null)
    setSigningResult(null)

    try {
      const formData = new FormData()
      formData.append('transaction', JSON.stringify({
        instructions: currentTransaction.instructions.map(instruction => ({
          programId: instruction.programId,
          accounts: instruction.accounts,
          data: instruction.data
        })),
        state: 2, // TRANSACTION_STATE_COMPILED
        config: {},
        data: currentTransaction.compiledData || '',
        feePayer: currentTransaction.feePayer || '',
        recentBlockhash: currentTransaction.recentBlockhash || '',
        signatures: currentTransaction.signatures || [],
        hash: '',
        signature: ''
      }))
      formData.append('privateKeys', JSON.stringify(validPrivateKeys))

      // Call server action instead of fetch
      const result = await signTransactionAction(formData)

      if (result.error) {
        throw new Error(result.error)
      }
      
      // Determine new state based on number of signatures
      const transactionResult = result.transaction as any
      const newState = (transactionResult?.state === 4) 
        ? TransactionState.FULLY_SIGNED 
        : TransactionState.PARTIALLY_SIGNED

      // Update current transaction state
      setCurrentTransaction(prev => ({
        ...prev!,
        state: newState,
        signatures: transactionResult?.signatures || prev!.signatures || [],
        // Keep existing fields
        feePayer: transactionResult?.feePayer || prev!.feePayer,
        recentBlockhash: transactionResult?.recentBlockhash || prev!.recentBlockhash,
        compiledData: transactionResult?.data || prev!.compiledData,
        // Add signing metadata
        signedAt: new Date()
      }))

      // Store signing results
      setSigningResult({
        signaturesAdded: result.signaturesAdded,
        totalSignatures: result.totalSignatures
      })

      // Clear form on success
      setShowSigningSection(false)
      setPrivateKeys([''])
      
      // Success feedback
      const stateMessage = newState === TransactionState.FULLY_SIGNED 
        ? 'Transaction is now FULLY_SIGNED and ready for submission!'
        : 'Transaction is now PARTIALLY_SIGNED. Additional signatures may be required.'
      
      alert(`Transaction signed successfully! ${stateMessage}`)
      
    } catch (error) {
      console.error('Error signing transaction:', error)
      setSigningError(error instanceof Error ? error.message : 'Unknown signing error')
    } finally {
      setSigningLoading(false)
    }
  }

  // Submit transaction using server action
  const submitTransaction = async () => {
    if (!currentTransaction || currentTransaction.state !== TransactionState.FULLY_SIGNED) {
      return
    }

    setSubmissionLoading(true)
    setSubmissionError(null)
    setSubmissionResult(null)

    try {
      const formData = new FormData()
      formData.append('transaction', JSON.stringify({
        id: currentTransaction.id,
        instructions: currentTransaction.instructions.map(instruction => ({
          programId: instruction.programId,
          accounts: instruction.accounts,
          data: instruction.data
        })),
        state: 4, // TRANSACTION_STATE_FULLY_SIGNED
        config: {
          feePayer: currentTransaction.feePayer || '',
          recentBlockhash: currentTransaction.recentBlockhash || ''
        },
        data: currentTransaction.compiledData || '',
        signatures: currentTransaction.signatures || [],
        transactionSignature: currentTransaction.transactionSignature || '',
        createdAt: currentTransaction.createdAt?.toISOString(),
        compiledAt: currentTransaction.compiledAt?.toISOString(),
        signedAt: currentTransaction.signedAt?.toISOString()
      }))
      formData.append('commitmentLevel', submissionCommitmentLevel)

      // Call server action instead of fetch
      const result = await submitTransactionAction(formData)

      if (result.error) {
        throw new Error(result.error)
      }
      
      // Update transaction state to SUBMITTED
      setCurrentTransaction(prev => ({
        ...prev!,
        state: TransactionState.SUBMITTED,
        transactionSignature: result.transactionSignature || '',
        submittedAt: new Date()
      }))

      // Store submission results
      setSubmissionResult({
        transactionSignature: result.transactionSignature || '',
        submittedAt: new Date()
      })

      // Clear form and show success
      setShowSubmissionSection(false)
      alert(`Transaction submitted successfully! Signature: ${result.transactionSignature || 'Unknown'}`)
      
    } catch (error) {
      console.error('Error submitting transaction:', error)
      setSubmissionError(error instanceof Error ? error.message : 'Unknown submission error')
    } finally {
      setSubmissionLoading(false)
    }
  }

  // Lookup transaction using server action
  const lookupTransaction = async () => {
    if (!lookupSignature.trim()) {
      setLookupError('Transaction signature is required')
      return
    }

    setLookupLoading(true)
    setLookupError(null)
    setLookupResult(null)

    try {
      const formData = new FormData()
      formData.append('transactionSignature', lookupSignature.trim())
      formData.append('commitmentLevel', monitoringCommitmentLevel)

      // Call server action instead of fetch
      const result = await getTransactionAction(formData)

      if (result.error) {
        throw new Error(result.error)
      }

      setLookupResult(result.transaction)
      
    } catch (error) {
      console.error('Error looking up transaction:', error)
      setLookupError(error instanceof Error ? error.message : 'Unknown lookup error')
    } finally {
      setLookupLoading(false)
    }
  }

  // Handle method calls using server actions
  const handleTransactionMethodCall = async (methodName: string, params: Record<string, any>) => {
    // Convert params to FormData for server actions
    const formData = new FormData()
    
    // Add all parameters to FormData
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== '') {
        formData.append(key, String(value))
      }
    })

    // Call appropriate server action based on method name
    switch (methodName) {
      case 'compileTransaction':
        return await compileTransactionAction(formData)
      
      case 'estimateTransaction':
        return await estimateTransactionAction(formData)
      
      case 'simulateTransaction':
        return await simulateTransactionAction(formData)
      
      case 'signTransaction':
        return await signTransactionAction(formData)
      
      case 'submitTransaction':
        return await submitTransactionAction(formData)
        
      case 'getTransaction':
        return await getTransactionAction(formData)
      
      default:
        throw new Error(`Unknown method: ${methodName}`)
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
      <div className="bg-white border border-slate-200 rounded-lg p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-medium text-slate-900">Draft Transaction Management</h3>
          {!currentTransaction ? (
            <button
              onClick={() => setShowCreateForm(true)}
              className="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Create Draft Transaction
            </button>
          ) : (
            <button
              onClick={clearTransaction}
              className="px-4 py-2 bg-red-600 text-white text-sm font-medium rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
            >
              Clear Transaction
            </button>
          )}
        </div>

        {!currentTransaction && showCreateForm && (
          <div className="bg-blue-50 border border-blue-200 rounded-md p-4 mb-4">
            <h4 className="text-sm font-medium text-blue-800 mb-2">Create New Draft Transaction</h4>
            <p className="text-xs text-blue-700 mb-3">
              Start a new transaction in DRAFT state. You can add instructions, then compile, sign, and submit.
            </p>
            <div className="flex space-x-3">
              <button
                onClick={createDraftTransaction}
                className="px-3 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700"
              >
                Create Transaction
              </button>
              <button
                onClick={() => setShowCreateForm(false)}
                className="px-3 py-2 bg-slate-200 text-slate-700 text-sm font-medium rounded-md hover:bg-slate-300"
              >
                Cancel
              </button>
            </div>
          </div>
        )}

        {!currentTransaction && !showCreateForm && (
          <div className="text-center py-8">
            <svg className="mx-auto h-12 w-12 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <h3 className="mt-2 text-sm font-medium text-slate-900">No transaction in progress</h3>
            <p className="mt-1 text-sm text-slate-500">Get started by creating a new draft transaction</p>
          </div>
        )}

        {/* Current Transaction Info */}
        {currentTransaction && (
          <div className="bg-slate-50 border border-slate-200 rounded-lg p-4 mb-6">
            <h4 className="text-sm font-medium text-slate-900 mb-3">Current Transaction</h4>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div>
                <dt className="text-xs font-medium text-slate-700">ID</dt>
                <dd className="text-sm text-slate-900 font-mono">{currentTransaction.id}</dd>
              </div>
              <div>
                <dt className="text-xs font-medium text-slate-700">State</dt>
                <dd className={`text-sm font-medium ${
                  currentTransaction.state === TransactionState.DRAFT ? 'text-amber-600' :
                  currentTransaction.state === TransactionState.COMPILED ? 'text-blue-600' :
                  currentTransaction.state === TransactionState.FULLY_SIGNED ? 'text-emerald-600' :
                  currentTransaction.state === TransactionState.SUBMITTED ? 'text-green-600' :
                  'text-yellow-600'
                }`}>
                  {currentTransaction.state}
                </dd>
              </div>
              <div>
                <dt className="text-xs font-medium text-slate-700">Instructions</dt>
                <dd className="text-sm text-slate-900">{currentTransaction.instructions.length}</dd>
              </div>
              <div>
                <dt className="text-xs font-medium text-slate-700">Created</dt>
                <dd className="text-sm text-slate-900">{currentTransaction.createdAt.toLocaleTimeString()}</dd>
              </div>
              {currentTransaction.feePayer && (
                <div className="col-span-2">
                  <dt className="text-xs font-medium text-slate-700">Fee Payer</dt>
                  <dd className="text-sm text-slate-900 font-mono break-all">{currentTransaction.feePayer}</dd>
                </div>
              )}
              {currentTransaction.compiledAt && (
                <div>
                  <dt className="text-xs font-medium text-slate-700">Compiled</dt>
                  <dd className="text-sm text-slate-900">{currentTransaction.compiledAt.toLocaleTimeString()}</dd>
                </div>
              )}
              {currentTransaction.signedAt && (
                <div>
                  <dt className="text-xs font-medium text-slate-700">Signed</dt>
                  <dd className="text-sm text-slate-900">{currentTransaction.signedAt.toLocaleTimeString()}</dd>
                </div>
              )}
              {currentTransaction.signatures && currentTransaction.signatures.length > 0 && (
                <div>
                  <dt className="text-xs font-medium text-slate-700">Signatures</dt>
                  <dd className="text-sm text-slate-900">{currentTransaction.signatures.length}</dd>
                </div>
              )}
              {currentTransaction.transactionSignature && (
                <div className="col-span-2">
                  <dt className="text-xs font-medium text-slate-700">Transaction Signature</dt>
                  <dd className="text-sm text-slate-900 font-mono break-all">{currentTransaction.transactionSignature}</dd>
                </div>
              )}
            </div>

            {/* Instructions List */}
            <div className="mt-4">
              <h5 className="text-xs font-medium text-slate-700 mb-2">Instructions ({currentTransaction.instructions.length})</h5>
              {currentTransaction.instructions.length === 0 ? (
                <p className="text-sm text-slate-500 italic">No instructions added yet</p>
              ) : (
                <div className="space-y-2">
                  {currentTransaction.instructions.map((instruction, index) => (
                    <div key={index} className="bg-white border border-slate-200 rounded p-3">
                      <div className="flex items-center justify-between">
                        <span className="text-sm font-medium text-slate-900">
                          Instruction {index + 1}
                        </span>
                        {instruction.description && (
                          <span className="text-xs text-slate-600">{instruction.description}</span>
                        )}
                      </div>
                      <div className="mt-2 grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
                        <div>
                          <dt className="font-medium text-slate-700">Program ID</dt>
                          <dd className="text-slate-900 font-mono break-all">{instruction.programId}</dd>
                        </div>
                        <div>
                          <dt className="font-medium text-slate-700">Accounts</dt>
                          <dd className="text-slate-900">{instruction.accounts.length} accounts</dd>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Quick Actions based on state */}
            <div className="mt-4 p-3 bg-white border border-slate-200 rounded">
              <h5 className="text-xs font-medium text-slate-700 mb-2">Quick Actions</h5>
              <div className="text-xs text-slate-600">
                {currentTransaction.state === TransactionState.DRAFT && currentTransaction.instructions.length === 0 && (
                  <p>📝 Add instructions to your transaction using the "Add Instruction" section below</p>
                )}
                {currentTransaction.state === TransactionState.DRAFT && currentTransaction.instructions.length > 0 && (
                  <p>✅ Ready for compilation - use the "Compile Transaction" section below</p>
                )}
                {currentTransaction.state === TransactionState.COMPILED && (
                  <p>🔐 Transaction compiled - ready for analysis and signing</p>
                )}
                {currentTransaction.state === TransactionState.FULLY_SIGNED && (
                  <p>🚀 Transaction fully signed - ready for submission to blockchain</p>
                )}
                {currentTransaction.state === TransactionState.SUBMITTED && (
                  <p>✨ Transaction submitted successfully - use monitoring section to track status</p>
                )}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Add Instruction Section */}
      {currentTransaction && currentTransaction.state === TransactionState.DRAFT && (
        <div className="bg-white border border-slate-200 rounded-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-slate-900">Add Instruction</h3>
            <button
              onClick={() => setShowAddInstruction(!showAddInstruction)}
              className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                showAddInstruction
                  ? 'bg-red-100 text-red-700 hover:bg-red-200'
                  : 'bg-green-600 text-white hover:bg-green-700'
              }`}
            >
              {showAddInstruction ? 'Cancel Adding Instruction' : 'Add Instruction'}
            </button>
          </div>

          <div className="bg-green-50 border border-green-200 rounded-md p-3 mb-4">
            <p className="text-xs text-green-800">
              <span className="font-medium">Instruction Building:</span> Select a program service and method to generate blockchain instructions.
            </p>
            <p className="text-xs text-green-700 mt-1">
              Each instruction will be added to your draft transaction for compilation.
            </p>
          </div>

          {showAddInstruction && (
            <div className="space-y-6">
              {/* Program Selector */}
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Select Program Service
                </label>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                  {Object.entries(programServices).map(([key, service]) => (
                    <button
                      key={key}
                      onClick={() => handleProgramSelect(key as ProgramType)}
                      className={`p-3 text-left border rounded-md transition-colors ${
                        selectedProgram === key
                          ? 'border-green-500 bg-green-50 text-green-900'
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

              {/* Method Parameters Form - Simplified placeholder */}
              {selectedMethod && (
                <div>
                  <label className="block text-sm font-medium text-slate-700 mb-3">
                    {selectedMethod.displayName} Parameters
                  </label>
                  <div className="bg-blue-50 border border-blue-200 rounded-md p-4">
                    <p className="text-sm text-blue-800">
                      <span className="font-medium">Parameter forms available:</span> The full parameter form implementation for {selectedMethod.displayName} is ready.
                    </p>
                    <p className="text-xs text-blue-700 mt-1">
                      This section would contain the dynamic parameter forms for the selected program method.
                    </p>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Transaction Compilation Section */}
      {currentTransaction && currentTransaction.state === TransactionState.DRAFT && currentTransaction.instructions.length > 0 && (
        <div className="bg-white border border-slate-200 rounded-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-slate-900">Compile Transaction</h3>
            <button
              onClick={() => setShowCompileForm(!showCompileForm)}
              className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                showCompileForm
                  ? 'bg-red-100 text-red-700 hover:bg-red-200'
                  : 'bg-blue-600 text-white hover:bg-blue-700'
              }`}
            >
              {showCompileForm ? 'Cancel Compilation' : 'Compile Transaction'}
            </button>
          </div>

          <div className="bg-blue-50 border border-blue-200 rounded-md p-3 mb-4">
            <p className="text-xs text-blue-800">
              <span className="font-medium">Ready for Compilation:</span> Your transaction has {currentTransaction.instructions.length} instruction{currentTransaction.instructions.length !== 1 ? 's' : ''} and is ready to compile.
            </p>
            <p className="text-xs text-blue-700 mt-1">
              Compilation will freeze the instruction set and prepare the transaction for signing.
            </p>
          </div>

          {showCompileForm && (
            <div className="space-y-4">
              {/* Fee Payer Input */}
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Fee Payer <span className="text-red-500">*</span>
                </label>
                <input
                  type="text"
                  value={compileFormData.feePayer || ''}
                  onChange={(e) => setCompileFormData(prev => ({ ...prev, feePayer: e.target.value }))}
                  placeholder="Enter the public key that will pay transaction fees"
                  className="block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
                  required
                />
                <p className="text-xs text-slate-600 mt-1">
                  The account that will pay for transaction fees. Must have sufficient SOL balance.
                </p>
              </div>

              {/* Recent Blockhash Input */}
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Recent Blockhash <span className="text-slate-500">(optional)</span>
                </label>
                <input
                  type="text"
                  value={compileFormData.recentBlockhash || ''}
                  onChange={(e) => setCompileFormData(prev => ({ ...prev, recentBlockhash: e.target.value }))}
                  placeholder="Leave empty to auto-fetch recent blockhash"
                  className="block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
                />
                <p className="text-xs text-slate-600 mt-1">
                  Recent blockhash for transaction expiration. If empty, will be fetched automatically.
                </p>
              </div>

              {/* Compile Button */}
              <div className="pt-3">
                <button
                  onClick={compileTransaction}
                  disabled={compileLoading || !compileFormData.feePayer?.trim()}
                  className="w-full px-4 py-3 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {compileLoading ? 'Compiling Transaction...' : 'Compile Transaction'}
                </button>
              </div>

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

      {/* Transaction Analysis Section */}
      {currentTransaction && currentTransaction.state === TransactionState.COMPILED && (
        <div className="bg-white border border-slate-200 rounded-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-slate-900">Transaction Analysis</h3>
            <button
              onClick={() => setShowAnalysisSection(!showAnalysisSection)}
              className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                showAnalysisSection
                  ? 'bg-red-100 text-red-700 hover:bg-red-200'
                  : 'bg-indigo-600 text-white hover:bg-indigo-700'
              }`}
            >
              {showAnalysisSection ? 'Hide Analysis' : 'Analyze Transaction'}
            </button>
          </div>

          <div className="bg-indigo-50 border border-indigo-200 rounded-md p-3 mb-4">
            <p className="text-xs text-indigo-800">
              <span className="font-medium">Ready for Analysis:</span> Estimate costs and simulate execution before signing.
            </p>
            <p className="text-xs text-indigo-700 mt-1">
              Analysis helps ensure your transaction will succeed and shows the expected costs.
            </p>
          </div>

          {showAnalysisSection && (
            <div className="space-y-4">
              {/* Commitment Level Selection */}
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Commitment Level
                </label>
                <select
                  value={analysisCommitmentLevel}
                  onChange={(e) => setAnalysisCommitmentLevel(e.target.value)}
                  className="block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                >
                  <option value="processed">Processed (Fastest, least reliable)</option>
                  <option value="confirmed">Confirmed (Balanced)</option>
                  <option value="finalized">Finalized (Slowest, most reliable)</option>
                </select>
              </div>

              {/* Analysis Actions */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Estimation */}
                <div>
                  <button
                    onClick={estimateTransaction}
                    disabled={estimateLoading}
                    className="w-full px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {estimateLoading ? 'Estimating...' : 'Estimate Costs'}
                  </button>

                  {estimateResult && (
                    <div className="mt-3 bg-green-50 border border-green-200 rounded p-3">
                      <h4 className="text-sm font-medium text-green-900 mb-2">Cost Estimation</h4>
                      <div className="space-y-1 text-xs text-green-800">
                        <div>Compute Units: {estimateResult.computeUnits}</div>
                        <div>Fee (lamports): {estimateResult.feeLamports}</div>
                        <div>Priority Fee: {estimateResult.priorityFee}</div>
                      </div>
                    </div>
                  )}

                  {estimateError && (
                    <div className="mt-3 bg-red-50 border border-red-200 rounded p-3">
                      <p className="text-xs text-red-700">{estimateError}</p>
                    </div>
                  )}
                </div>

                {/* Simulation */}
                <div>
                  <button
                    onClick={simulateTransaction}
                    disabled={simulateLoading}
                    className="w-full px-4 py-2 bg-yellow-600 text-white text-sm font-medium rounded-md hover:bg-yellow-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-yellow-500 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {simulateLoading ? 'Simulating...' : 'Simulate Execution'}
                  </button>

                  {simulateResult && (
                    <div className={`mt-3 border rounded p-3 ${
                      simulateResult.success 
                        ? 'bg-green-50 border-green-200' 
                        : 'bg-red-50 border-red-200'
                    }`}>
                      <h4 className={`text-sm font-medium mb-2 ${
                        simulateResult.success ? 'text-green-900' : 'text-red-900'
                      }`}>
                        Simulation {simulateResult.success ? 'Success ✅' : 'Failed ⚠️'}
                      </h4>
                      {!simulateResult.success && simulateResult.error && (
                        <div className="text-xs text-red-700 mb-2">Error: {simulateResult.error}</div>
                      )}
                      {simulateResult.logs.length > 0 && (
                        <div className="text-xs font-mono bg-slate-100 p-2 rounded max-h-20 overflow-y-auto">
                          {simulateResult.logs.slice(0, 3).map((log, i) => (
                            <div key={i}>{log}</div>
                          ))}
                          {simulateResult.logs.length > 3 && (
                            <div className="text-slate-500">... and {simulateResult.logs.length - 3} more</div>
                          )}
                        </div>
                      )}
                    </div>
                  )}

                  {simulateError && (
                    <div className="mt-3 bg-red-50 border border-red-200 rounded p-3">
                      <p className="text-xs text-red-700">{simulateError}</p>
                    </div>
                  )}
                </div>
              </div>

              {/* Analysis Summary */}
              {(estimateResult || simulateResult) && (
                <div className="bg-slate-50 border border-slate-200 rounded p-3">
                  <h4 className="text-sm font-medium text-slate-900 mb-2">Analysis Summary</h4>
                  <div className="text-xs text-slate-700 space-y-1">
                    {estimateResult && (
                      <div>💰 Estimated cost: {estimateResult.feeLamports} lamports</div>
                    )}
                    {simulateResult && (
                      <div>
                        {simulateResult.success ? (
                          <span>✅ Simulation successful - transaction should execute properly</span>
                        ) : (
                          <span>⚠️ Simulation failed - review errors before signing</span>
                        )}
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Transaction Signing Section */}
      {currentTransaction && currentTransaction.state === TransactionState.COMPILED && (
        <div className="bg-white border border-slate-200 rounded-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-slate-900">Transaction Signing</h3>
            <button
              onClick={() => setShowSigningSection(!showSigningSection)}
              className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                showSigningSection
                  ? 'bg-red-100 text-red-700 hover:bg-red-200'
                  : 'bg-emerald-600 text-white hover:bg-emerald-700'
              }`}
            >
              {showSigningSection ? 'Cancel Signing' : 'Sign Transaction'}
            </button>
          </div>

          <div className="bg-emerald-50 border border-emerald-200 rounded-md p-3 mb-4">
            <p className="text-xs text-emerald-800">
              <span className="font-medium">Ready for Signing:</span> Add private keys to sign this compiled transaction.
            </p>
            <p className="text-xs text-emerald-700 mt-1">
              Supports multi-signature scenarios. Add multiple private keys for complex signing requirements.
            </p>
          </div>

          {showSigningSection && (
            <div className="space-y-6">
              {/* Security Warning */}
              <div className="bg-red-50 border border-red-200 rounded-md p-4">
                <div className="flex">
                  <div className="flex-shrink-0">
                    <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                      <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                    </svg>
                  </div>
                  <div className="ml-3">
                    <h3 className="text-sm font-medium text-red-800">Security Warning</h3>
                    <div className="mt-2 text-sm text-red-700">
                      <ul className="list-disc pl-5 space-y-1">
                        <li>Private keys are sensitive information. Never share them with others.</li>
                        <li>This demo environment temporarily processes keys for signing.</li>
                        <li>In production, use hardware wallets or secure key management.</li>
                        <li>Clear browser data after use to remove any key traces.</li>
                      </ul>
                    </div>
                  </div>
                </div>
              </div>

              {/* Private Key Inputs */}
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-3">
                  Private Keys for Signing
                </label>
                <div className="space-y-3">
                  {privateKeys.map((privateKey, index) => (
                    <div key={index} className="flex items-center space-x-2">
                      <div className="flex-1">
                        <input
                          type="password"
                          value={privateKey}
                          onChange={(e) => updatePrivateKey(index, e.target.value)}
                          placeholder={`Private Key ${index + 1} (Base58 encoded)`}
                          className="block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-transparent font-mono text-sm"
                        />
                      </div>
                      <button
                        type="button"
                        onClick={() => removePrivateKeyField(index)}
                        disabled={privateKeys.length <= 1}
                        className="px-3 py-2 text-red-600 hover:text-red-800 disabled:text-gray-400 disabled:cursor-not-allowed"
                        title="Remove private key"
                      >
                        <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </div>
                  ))}
                  
                  {/* Add Private Key Button */}
                  <button
                    type="button"
                    onClick={addPrivateKeyField}
                    className="w-full px-3 py-2 border border-dashed border-slate-300 rounded-md text-sm text-slate-600 hover:border-slate-400 hover:text-slate-700 focus:outline-none focus:ring-2 focus:ring-emerald-500"
                  >
                    + Add Another Private Key (Multi-sig)
                  </button>
                </div>
                <p className="text-xs text-slate-600 mt-2">
                  Add multiple private keys for multi-signature transactions. Each key will contribute one signature.
                </p>
              </div>

              {/* Sign Button */}
              <div className="pt-3">
                <button
                  onClick={signTransaction}
                  disabled={signingLoading || privateKeys.every(key => !key.trim())}
                  className="w-full px-4 py-3 bg-emerald-600 text-white text-sm font-medium rounded-md hover:bg-emerald-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-emerald-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {signingLoading ? 'Signing Transaction...' : `Sign with ${privateKeys.filter(key => key.trim()).length} Key${privateKeys.filter(key => key.trim()).length !== 1 ? 's' : ''}`}
                </button>
              </div>

              {/* Signing Results */}
              {signingResult && (
                <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                  <h3 className="text-sm font-medium text-green-900 mb-3">Signing Results</h3>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <dt className="text-xs font-medium text-green-700">Signatures Added</dt>
                      <dd className="text-sm text-green-900 font-mono">{signingResult.signaturesAdded}</dd>
                    </div>
                    <div>
                      <dt className="text-xs font-medium text-green-700">Total Signatures</dt>
                      <dd className="text-sm text-green-900 font-mono">{signingResult.totalSignatures}</dd>
                    </div>
                  </div>
                </div>
              )}

              {signingError && (
                <div className="bg-red-50 border border-red-200 rounded-md p-3">
                  <p className="text-sm text-red-800 font-medium">Signing Failed</p>
                  <p className="text-xs text-red-700 mt-1">{signingError}</p>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Transaction Submission Section */}
      {currentTransaction && currentTransaction.state === TransactionState.FULLY_SIGNED && (
        <div className="bg-white border border-slate-200 rounded-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-slate-900">Submit Transaction</h3>
            <button
              onClick={() => setShowSubmissionSection(!showSubmissionSection)}
              className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                showSubmissionSection
                  ? 'bg-red-100 text-red-700 hover:bg-red-200'
                  : 'bg-purple-600 text-white hover:bg-purple-700'
              }`}
            >
              {showSubmissionSection ? 'Cancel Submission' : 'Submit Transaction'}
            </button>
          </div>

          <div className="bg-purple-50 border border-purple-200 rounded-md p-3 mb-4">
            <p className="text-xs text-purple-800">
              <span className="font-medium">Ready for Submission:</span> This fully-signed transaction can now be broadcast to the Solana network.
            </p>
            <p className="text-xs text-purple-700 mt-1">
              Once submitted, the transaction will be processed by validators and become part of the blockchain.
            </p>
          </div>

          {showSubmissionSection && (
            <div className="space-y-4">
              {/* Commitment Level Selection */}
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Commitment Level
                </label>
                <select
                  value={submissionCommitmentLevel}
                  onChange={(e) => setSubmissionCommitmentLevel(e.target.value)}
                  className="block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                >
                  <option value="processed">Processed (Fastest, least reliable)</option>
                  <option value="confirmed">Confirmed (Balanced)</option>
                  <option value="finalized">Finalized (Slowest, most reliable)</option>
                </select>
                <p className="text-xs text-slate-600 mt-1">
                  Choose the level of confirmation required before the submit call returns
                </p>
              </div>

              {/* Submit Button */}
              <div className="pt-3">
                <button
                  onClick={submitTransaction}
                  disabled={submissionLoading}
                  className="w-full px-4 py-3 bg-purple-600 text-white text-sm font-medium rounded-md hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {submissionLoading ? 'Submitting Transaction...' : 'Submit to Solana Network'}
                </button>
              </div>

              {/* Submission Results */}
              {submissionResult && (
                <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                  <h3 className="text-sm font-medium text-green-900 mb-3">Transaction Submitted Successfully!</h3>
                  <div className="space-y-3">
                    <div>
                      <dt className="text-xs font-medium text-green-700">Transaction Signature</dt>
                      <dd className="text-sm text-green-900 font-mono break-all">{submissionResult.transactionSignature}</dd>
                    </div>
                    <div>
                      <dt className="text-xs font-medium text-green-700">Submitted At</dt>
                      <dd className="text-sm text-green-900">{submissionResult.submittedAt.toLocaleString()}</dd>
                    </div>
                    <div className="pt-2">
                      <a
                        href={`https://explorer.solana.com/tx/${submissionResult.transactionSignature}?cluster=custom&customUrl=http://localhost:8899`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center text-sm text-blue-600 hover:text-blue-800"
                      >
                        View in Solana Explorer (Local)
                        <svg className="ml-1 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                        </svg>
                      </a>
                    </div>
                  </div>
                </div>
              )}

              {submissionError && (
                <div className="bg-red-50 border border-red-200 rounded-md p-3">
                  <p className="text-sm text-red-800 font-medium">Submission Failed</p>
                  <p className="text-xs text-red-700 mt-1">{submissionError}</p>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Transaction Monitoring Section */}
      <div className="bg-white border border-slate-200 rounded-lg p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-medium text-slate-900">Transaction Monitoring</h3>
          <button
            onClick={() => setShowMonitoringSection(!showMonitoringSection)}
            className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
              showMonitoringSection
                ? 'bg-red-100 text-red-700 hover:bg-red-200'
                : 'bg-indigo-600 text-white hover:bg-indigo-700'
            }`}
          >
            {showMonitoringSection ? 'Hide Monitoring' : 'Monitor Transaction'}
          </button>
        </div>

        <div className="bg-indigo-50 border border-indigo-200 rounded-md p-3 mb-4">
          <p className="text-xs text-indigo-800">
            <span className="font-medium">Transaction Lookup:</span> Enter any transaction signature to query its status on the Solana network.
          </p>
          <p className="text-xs text-indigo-700 mt-1">
            Use this to monitor submitted transactions or investigate existing transactions.
          </p>
        </div>

        {showMonitoringSection && (
          <div className="space-y-4">
            {/* Lookup Form */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Transaction Signature
                </label>
                <input
                  type="text"
                  value={lookupSignature}
                  onChange={(e) => setLookupSignature(e.target.value)}
                  placeholder="Enter transaction signature to lookup..."
                  className="block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent font-mono text-sm"
                />
                {currentTransaction?.transactionSignature && (
                  <button
                    onClick={() => setLookupSignature(currentTransaction.transactionSignature!)}
                    className="mt-2 text-xs text-indigo-600 hover:text-indigo-800"
                  >
                    Use current transaction signature
                  </button>
                )}
              </div>
              
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Commitment Level
                </label>
                <select
                  value={monitoringCommitmentLevel}
                  onChange={(e) => setMonitoringCommitmentLevel(e.target.value)}
                  className="block w-full px-3 py-2 border border-slate-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                >
                  <option value="processed">Processed</option>
                  <option value="confirmed">Confirmed</option>
                  <option value="finalized">Finalized</option>
                </select>
              </div>
            </div>

            {/* Lookup Button */}
            <div className="pt-3">
              <button
                onClick={lookupTransaction}
                disabled={lookupLoading || !lookupSignature.trim()}
                className="w-full px-4 py-3 bg-indigo-600 text-white text-sm font-medium rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {lookupLoading ? 'Looking up Transaction...' : 'Lookup Transaction'}
              </button>
            </div>

            {/* Lookup Results */}
            {lookupResult && (
              <div className="bg-slate-50 border border-slate-200 rounded-lg p-4">
                <h3 className="text-sm font-medium text-slate-900 mb-3">Transaction Details</h3>
                <div className="space-y-3">
                  <div>
                    <dt className="text-xs font-medium text-slate-700">Response</dt>
                    <dd className="text-sm text-slate-900 font-mono bg-slate-100 p-2 rounded">
                      {JSON.stringify(lookupResult, null, 2)}
                    </dd>
                  </div>
                </div>
              </div>
            )}

            {lookupError && (
              <div className="bg-red-50 border border-red-200 rounded-md p-3">
                <p className="text-sm text-red-800 font-medium">Lookup Failed</p>
                <p className="text-xs text-red-700 mt-1">{lookupError}</p>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Transaction Service Methods */}
      <ServicePage
        serviceName={transactionServiceConfig.name}
        serviceDescription="Transaction lifecycle operations - requires an active draft transaction for most operations"
        methods={transactionServiceConfig.methods}
        onMethodCall={handleTransactionMethodCall}
      />
    </div>
  )
}