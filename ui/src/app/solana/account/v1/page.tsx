'use client'

import ServicePage from '../../../../components/ServicePage'
import { accountServiceConfig } from '../../../../lib/service-configs'
import { 
  getAccountAction,
  generateNewKeyPairAction,
  fundNativeAction
} from '../../../../lib/actions/account-actions'

export default function AccountV1Page() {
  // Handle method calls using server actions
  const handleMethodCall = async (methodName: string, params: Record<string, any>) => {
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
      case 'getAccount':
        return await getAccountAction(formData)
      
      case 'generateNewKeyPair':
        return await generateNewKeyPairAction()
      
      case 'fundNative':
        return await fundNativeAction(formData)
      
      default:
        throw new Error(`Unknown method: ${methodName}`)
    }
  }

  return (
    <ServicePage
      serviceName={accountServiceConfig.name}
      serviceDescription={accountServiceConfig.description}
      methods={accountServiceConfig.methods}
      onMethodCall={handleMethodCall}
    />
  )
}