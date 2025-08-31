'use client'

import ServicePage from '../../../../../components/ServicePage'
import { systemProgramServiceConfig } from '../../../../../lib/service-configs'
import { 
  systemCreateAccountAction,
  systemTransferAction
} from '../../../../../lib/actions/service-actions'

export default function SystemProgramV1Page() {
  // Handle method calls using server actions
  const handleMethodCall = async (methodName: string, params: Record<string, any>) => {
    // Convert params to FormData for server actions
    const formData = new FormData()
    
    // Add all parameters to FormData
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== '') {
        // Map parameter names to match server action expectations
        let mappedKey = key
        if (methodName === 'create') {
          // Map service config names to server action parameter names
          if (key === 'payer') mappedKey = 'fromPubkey'
          if (key === 'newAccount') mappedKey = 'newAccountPubkey'
          if (key === 'owner') mappedKey = 'owner'
        } else if (methodName === 'transfer') {
          // Map service config names to server action parameter names
          if (key === 'from') mappedKey = 'fromPubkey'
          if (key === 'to') mappedKey = 'toPubkey'
        }
        
        formData.append(mappedKey, String(value))
      }
    })

    // Call appropriate server action based on method name
    switch (methodName) {
      case 'create':
        return await systemCreateAccountAction(formData)
      
      case 'transfer':
        return await systemTransferAction(formData)
      
      default:
        throw new Error(`Unknown method: ${methodName}`)
    }
  }

  return (
    <ServicePage
      serviceName={systemProgramServiceConfig.name}
      serviceDescription={systemProgramServiceConfig.description}
      methods={systemProgramServiceConfig.methods}
      onMethodCall={handleMethodCall}
    />
  )
}