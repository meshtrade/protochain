# **Agent Task: Generate Implementation Plan for Solana RPC Client Wrapper**

## **Persona**

You are a senior software engineer expert in:

* Rust asynchronous programming

* Solana blockchain development

* Protobuf and gRPC end-to-end service implementation

* Test-driven development (TDD)

## **Objective**

Generate a step-by-step, purely technical implementation plan to wrap the Solana RPC `get_minimum_balance_for_rent_exemption` method within the `protosol` gRPC service.

This plan will be consumed by a subsequent execution agent. Therefore, the plan **must contain the complete and final code** required for the implementation, making the execution step a direct and verifiable application of this plan.

## **Execution Context**

* **Repository Root:** All file paths are relative to the `protosol` project root.

* **Reference Pattern:** The implementation **MUST** follow the existing pattern established by the `system program` wrapper:

  * **Proto Definition:** `lib/proto/protosol/solana/program/system/v1/service.proto`

  * **Rust Implementation:** `api/src/api/program/system/v1/service_impl.rs`

  * **Go E2E Test:** `tests/go/streaming_e2e_test.go`

## **Architecture References**

* **Solana RPC Method Documentation:** `https://docs.rs/solana-client/latest/solana_client/rpc_client/struct.RpcClient.html#method.get_minimum_balance_for_rent_exemption`
* **Example of Existing RPC Client Usage:** `api/src/api/account/v1/service_impl.rs`

## **Primary Deliverable: `projects/add-rpc-client-wrapper/implementation-plan.md`**

Generate the content for the markdown file specified above. The content **MUST** contain the following sections, formatted exactly as described.

### **Implementation Plan: RPC Client Wrapper**

#### **Step 1: Define the gRPC Service and Messages**

1. **Action:** `CREATE`

2. **File Path:** `lib/proto/protosol/solana/rpc_client/v1/service.proto`

3. **Required Code to Implement:** Generate the complete Protobuf definition for a new `RpcClientService`.

   * The service must contain one RPC method: `GetMinimumBalanceForRentExemption`.

   * Define a `GetMinimumBalanceForRentExemptionRequest` message that includes a `uint64 data_length` field.

   * Define a `GetMinimumBalanceForRentExemptionResponse` message that includes a `uint64 balance` field.

4. **Validation:**

   * Run `buf build` to ensure the new protobuf definition compiles without errors.

   * Verify that generated code stubs are created for Rust and Go.

#### **Step 2: Implement the Rust gRPC Service**

1. **Action:** `CREATE`

2. **File Path:** `api/src/api/rpc_client/v1/service_impl.rs`

3. **Required Code to Implement:** Generate the complete Rust implementation of the `RpcClientService` trait within a `RpcClientServiceImpl` struct.

   * The struct should hold a dependency to the `solana_client::rpc_client::RpcClient`, likely via `std::sync::Arc`.

   * The `get_minimum_balance_for_rent_exemption` async method must:

     * Extract the `data_length` from the incoming `tonic::Request`.

     * Call the underlying `rpc_client.get_minimum_balance_for_rent_exemption()` method with the `data_length`.

     * On success, wrap the resulting balance in the `GetMinimumBalanceForRentExemptionResponse` and return `Ok(Response::new(...))`.

     * On error, return an appropriate `tonic::Status::internal` error.

4. **Integration:**

   * Wire the `RpcClientServiceImpl` into the main gRPC server in `api/src/main.rs`.

5. **Validation:**

   * The Rust project (`/api`) must compile successfully with `cargo build`.

   * All existing unit and integration tests must pass.

#### **Step 3: Implement and Pass the End-to-End Test**

1. **Action:** `CREATE` or `MODIFY` (depending on test suite structure)

2. **File Path:** `tests/go/e2e-rpc-client_test.go`

3. **Target Implementation (Provided):** Add the following test case to the Go E2E test suite. This is the goal state.

   ```
   // get minimum balance for rent exemption with random data length
   resp, err := suite.rpcClientService.GetMinimumBalanceForRentExemption(suite.ctx, &rpc_client_v1.GetMinimumBalanceForRentExemptionRequest{
       DataLength: 100,
   })
   suite.Require().NoError(err, "should succeed in getting minimum balance for rent exemption")
   suite.Require().NotZero(resp.Balance, "minimum balance for rent exemption should not be zero")
   
   ```

4. **Validation:**

   * Run the Go E2E test suite.

   * The new `GetMinimumBalanceForRentExemption` test **MUST** pass.

   * All other existing E2E tests **MUST** continue to pass.

#### **Step 4: Finalize Implementation**

1. **Action:** `REVIEW & REFINE`

2. **File Paths:**

   * `lib/proto/protosol/solana/rpc_client/v1/service.proto`

   * `api/src/api/rpc_client/v1/service_impl.rs`

   * `tests/go/e2e-rpc-client_test.go`

3. **Code Review Instructions:**

   * Conduct a thorough self-review of all newly created and modified code.

   * Improve code for clarity, efficiency, and adherence to existing project patterns.

   * Remove any temporary or debugging statements.

4. **Documentation Instructions:**

   * Add comprehensive documentation (comments, docstrings) to all new code.

   * Ensure documentation is clean and describes the final functionality (e.g., "This function does X").

   * **Forbidden Documentation Content:** Do not include session-specific notes like "new code:", "updated to fix bug", or "TODO".

5. **Validation:**

   * Re-run all tests (`cargo build`, Go E2E tests) one final time to ensure no regressions were introduced during review and documentation.

### **Success Criteria**

1. All four steps above are completed in order.

2. The `e2e-rpc-client_test.go` test passes successfully against a running service.

3. The implementation is merged without breaking any existing functionality.

4. The final code is self-reviewed and documented according to the criteria in Step 4.

## **Forbidden Content**

* Implementation timelines, schedules, or time allocations.

* Human workflow recommendations or project management advice.

* References to daily/weekly/monthly patterns.

* Business justifications or value propositions.

* Performance targets with specific durations.

## **Enforcement Mechanism**

Before generating the implementation plan, verify it contains **ZERO** instances of the forbidden content. The output must be a pure technical specification suitable for automated agent consumption.
