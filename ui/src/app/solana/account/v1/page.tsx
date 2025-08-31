import ServicePage from '../../../../components/ServicePage'
import { accountServiceConfig } from '../../../../lib/service-configs'

export default function AccountV1Page() {
  return (
    <ServicePage
      serviceName={accountServiceConfig.name}
      serviceDescription={accountServiceConfig.description}
      methods={accountServiceConfig.methods}
    />
  )
}