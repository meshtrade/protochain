'use client'

import ServicePage from '../../../../components/ServicePage'
import { rpcClientServiceConfig } from '../../../../lib/service-configs'
import { 
  getMinimumBalanceForRentExemptionAction
} from '../../../../lib/actions/service-actions'

export default function RPCClientV1Page() {
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
      case 'getMinimumBalanceForRentExemption':
        return await getMinimumBalanceForRentExemptionAction(formData)
      
      default:
        throw new Error(`Unknown method: ${methodName}`)
    }
  }

  return (
    <ServicePage
      serviceName={rpcClientServiceConfig.name}
      serviceDescription={rpcClientServiceConfig.description}
      methods={rpcClientServiceConfig.methods}
      onMethodCall={handleMethodCall}
    />
  )
}