import ServicePage from '../../../../../components/ServicePage'
import { tokenProgramServiceConfig } from '../../../../../lib/service-configs'

export default function TokenProgramV1Page() {
  return (
    <ServicePage
      serviceName={tokenProgramServiceConfig.name}
      serviceDescription={tokenProgramServiceConfig.description}
      methods={tokenProgramServiceConfig.methods}
    />
  )
}