import ServicePage from '../../../../components/ServicePage'
import { rpcClientServiceConfig } from '../../../../lib/service-configs'

export default function RPCClientV1Page() {
  return (
    <ServicePage
      serviceName={rpcClientServiceConfig.name}
      serviceDescription={rpcClientServiceConfig.description}
      methods={rpcClientServiceConfig.methods}
    />
  )
}