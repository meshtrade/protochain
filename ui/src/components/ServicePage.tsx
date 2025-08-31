'use client'

import { useState } from 'react'
import { ChevronDownIcon } from '@heroicons/react/24/outline'

// Types for service method definitions
export interface MethodParam {
  name: string
  type: 'string' | 'number' | 'boolean' | 'enum' | 'bigint'
  required: boolean
  description?: string
  enumOptions?: string[]
  placeholder?: string
}

export interface ServiceMethod {
  name: string
  displayName: string
  description: string
  params: MethodParam[]
  endpoint: string // API endpoint to call
}

export interface ServicePageProps {
  serviceName: string
  serviceDescription: string
  methods: ServiceMethod[]
  onMethodCall?: (method: string, params: Record<string, any>) => Promise<any>
}

interface FormData {
  [key: string]: string | number | boolean
}

interface ApiResponse {
  success: boolean
  data?: any
  error?: string
  details?: string
}

export default function ServicePage({
  serviceName,
  serviceDescription,
  methods,
  onMethodCall
}: ServicePageProps) {
  const [selectedMethod, setSelectedMethod] = useState<ServiceMethod | null>(null)
  const [formData, setFormData] = useState<FormData>({})
  const [loading, setLoading] = useState(false)
  const [response, setResponse] = useState<ApiResponse | null>(null)
  const [isMethodSelectorOpen, setIsMethodSelectorOpen] = useState(false)

  const handleMethodSelect = (method: ServiceMethod) => {
    setSelectedMethod(method)
    setFormData({})
    setResponse(null)
    setIsMethodSelectorOpen(false)
  }

  const handleInputChange = (paramName: string, value: string | number | boolean) => {
    setFormData(prev => ({
      ...prev,
      [paramName]: value
    }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!selectedMethod) return

    setLoading(true)
    setResponse(null)

    try {
      // Convert form data to proper types
      const processedData: Record<string, any> = {}
      
      for (const param of selectedMethod.params) {
        const value = formData[param.name]
        
        if (param.required && (value === undefined || value === '')) {
          throw new Error(`${param.name} is required`)
        }

        if (value !== undefined && value !== '') {
          switch (param.type) {
            case 'number':
              processedData[param.name] = Number(value)
              break
            case 'boolean':
              processedData[param.name] = Boolean(value)
              break
            case 'bigint':
              processedData[param.name] = BigInt(value as string)
              break
            default:
              processedData[param.name] = value
          }
        }
      }

      // Call the method either via prop callback or direct API call
      let result
      if (onMethodCall) {
        result = await onMethodCall(selectedMethod.name, processedData)
      } else {
        // Default API call
        const apiResponse = await fetch(selectedMethod.endpoint, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(processedData),
        })

        if (!apiResponse.ok) {
          const errorData = await apiResponse.json()
          throw new Error(errorData.error || `HTTP ${apiResponse.status}`)
        }

        result = await apiResponse.json()
      }

      setResponse({
        success: true,
        data: result
      })

    } catch (error) {
      setResponse({
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred',
        details: error instanceof Error ? error.stack : undefined
      })
    } finally {
      setLoading(false)
    }
  }

  const renderFormInput = (param: MethodParam) => {
    const value = formData[param.name] || ''

    switch (param.type) {
      case 'boolean':
        return (
          <div className="flex items-center">
            <input
              type="checkbox"
              id={param.name}
              checked={Boolean(value)}
              onChange={(e) => handleInputChange(param.name, e.target.checked)}
              className="rounded border-slate-300 text-blue-600 focus:ring-blue-500"
            />
            <label htmlFor={param.name} className="ml-2 text-sm text-slate-700">
              {param.description || param.name}
            </label>
          </div>
        )

      case 'enum':
        return (
          <select
            value={String(value)}
            onChange={(e) => handleInputChange(param.name, e.target.value)}
            className="mt-1 block w-full rounded-md border-slate-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            required={param.required}
          >
            <option value="">Select {param.name}</option>
            {param.enumOptions?.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        )

      case 'number':
      case 'bigint':
        return (
          <input
            type="number"
            value={String(value)}
            onChange={(e) => handleInputChange(param.name, e.target.value)}
            placeholder={param.placeholder}
            className="mt-1 block w-full rounded-md border-slate-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            required={param.required}
          />
        )

      default: // string
        return (
          <input
            type="text"
            value={String(value)}
            onChange={(e) => handleInputChange(param.name, e.target.value)}
            placeholder={param.placeholder}
            className="mt-1 block w-full rounded-md border-slate-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            required={param.required}
          />
        )
    }
  }

  return (
    <div className="space-y-6">
      {/* Service Header */}
      <div>
        <h1 className="text-2xl font-bold text-slate-900">{serviceName}</h1>
        <p className="mt-1 text-sm text-slate-600">{serviceDescription}</p>
      </div>

      {/* Method Selector */}
      <div className="bg-white shadow rounded-lg p-6">
        <h2 className="text-lg font-medium text-slate-900 mb-4">Select Method</h2>
        
        <div className="relative">
          <button
            type="button"
            onClick={() => setIsMethodSelectorOpen(!isMethodSelectorOpen)}
            className="relative w-full cursor-pointer rounded-md border border-slate-300 bg-white py-2 pl-3 pr-10 text-left shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          >
            <span className="block truncate">
              {selectedMethod ? selectedMethod.displayName : 'Choose a method...'}
            </span>
            <span className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
              <ChevronDownIcon
                className={`h-5 w-5 text-slate-400 transition-transform ${
                  isMethodSelectorOpen ? 'rotate-180' : ''
                }`}
              />
            </span>
          </button>

          {isMethodSelectorOpen && (
            <div className="absolute z-10 mt-1 w-full rounded-md bg-white shadow-lg">
              <div className="max-h-60 rounded-md py-1 text-base overflow-auto focus:outline-none">
                {methods.map((method) => (
                  <div
                    key={method.name}
                    onClick={() => handleMethodSelect(method)}
                    className="cursor-pointer select-none relative py-2 pl-3 pr-9 hover:bg-blue-50"
                  >
                    <div className="flex flex-col">
                      <span className="font-medium text-slate-900">{method.displayName}</span>
                      <span className="text-xs text-slate-500">{method.description}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Method Form */}
      {selectedMethod && (
        <div className="bg-white shadow rounded-lg p-6">
          <h2 className="text-lg font-medium text-slate-900 mb-4">
            {selectedMethod.displayName} Parameters
          </h2>
          
          <form onSubmit={handleSubmit} className="space-y-4">
            {selectedMethod.params.map((param) => (
              <div key={param.name}>
                <label htmlFor={param.name} className="block text-sm font-medium text-slate-700">
                  {param.name}
                  {param.required && <span className="text-red-500 ml-1">*</span>}
                </label>
                {param.description && param.type !== 'boolean' && (
                  <p className="text-xs text-slate-500 mt-1">{param.description}</p>
                )}
                {renderFormInput(param)}
              </div>
            ))}

            <div className="pt-4">
              <button
                type="submit"
                disabled={loading}
                className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loading ? 'Calling...' : `Call ${selectedMethod.displayName}`}
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Response Display */}
      {response && (
        <div className="bg-white shadow rounded-lg p-6">
          <h2 className="text-lg font-medium text-slate-900 mb-4">Response</h2>
          
          <div className={`rounded-md p-4 ${
            response.success 
              ? 'bg-green-50 border border-green-200' 
              : 'bg-red-50 border border-red-200'
          }`}>
            <div className="flex">
              <div className="flex-1">
                <h3 className={`text-sm font-medium ${
                  response.success ? 'text-green-800' : 'text-red-800'
                }`}>
                  {response.success ? 'Success' : 'Error'}
                </h3>
                
                {response.success && response.data && (
                  <div className="mt-2">
                    <pre className="text-xs text-slate-700 bg-white p-3 rounded border overflow-x-auto">
                      {JSON.stringify(response.data, null, 2)}
                    </pre>
                  </div>
                )}
                
                {!response.success && (
                  <div className="mt-2">
                    <p className="text-sm text-red-700">{response.error}</p>
                    {response.details && (
                      <details className="mt-2">
                        <summary className="text-xs text-red-600 cursor-pointer">Stack trace</summary>
                        <pre className="text-xs text-red-600 mt-1 bg-white p-2 rounded border overflow-x-auto">
                          {response.details}
                        </pre>
                      </details>
                    )}
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}