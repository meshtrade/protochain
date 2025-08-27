# 1. is submit transaction synchronous now?
in api/src/api/transaction/v1/service_impl.rs in the submit transaction method async fn get_transaction we use:
- self.rpc_client.send_transaction_with_config and then
- self.wait_for_confirmation_with_polling
that means we wait for the result after submission. This essentially makes the submissionn synchronous right?
This is NOT the flow we want. We should just return submission_result and signature result without waiting for a response.
The client can poll for the response using monitor_transaction IF THEY WANT to.