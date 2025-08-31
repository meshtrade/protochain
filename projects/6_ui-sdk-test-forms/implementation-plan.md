# ProtoSol UI Dashboard Implementation Plan

## Overview
This plan implements a comprehensive Next.js dashboard for testing and interacting with all ProtoSol gRPC APIs. The implementation follows small, incremental steps building on each other to achieve a production-ready UI for transaction construction, compilation, signing, and submission.

## Goals
- Create a sidebar-based dashboard for all ProtoSol services
- Enable instruction construction via program services
- Support full transaction lifecycle (draft → compile → sign → submit)  
- Provide server-side gRPC client integration using connect-es
- Maintain type safety with generated TypeScript bindings

## CRITICAL: Context Management, Progress and Quality
Some critical information for the implemenation agent.

### Important Notice to Implementation Agent on Step Comletion

**You DO NOT need to complete all steps in one session.** There is NO requirement to fit everything in a single context window. This implementation can and should be done methodically, with breaks and context resets as needed.

### On Progress Tracking

1. **Create Progress File**: Before starting implementation, create/update your progress task tracking file .claude-task in the same directory as this plan
2. **Update Progress**: After completing each step or substep, update the progress file with:
   - Timestamp
   - Step completed
   - Any important findings or deviations
   - Next step to tackle
3. **Context Reset Protocol**: If you need to clear context or feel overwhelmed:
   - Update progress file with current state
   - Note any pending work
   - When resuming, use the magic phrase: "carry on with this task implementation-plan and take a look at the progress.md to see where you got up to"
4. **Eventually Consistent**: The progress file is eventually consistent - you may have progressed further than the last entry. Always verify the actual state before continuing (only relevant on RESTARTING TASK).

### On Quality Over Speed

**NEVER** simulate or fake implementation. If you find yourself writing comments like "simulating token mint creation" instead of actual code, STOP immediately, update the progress file, and request a context reset.

---

## Phase 1: Foundation Setup

### Step 1.1: Create TypeScript SDK Index and Exports
**Objective**: Enable clean imports of all generated types and services

**Actions**:
1. Create `lib/ts/src/index.ts` to export all generated services and types
2. Export account, transaction, program services and request/response types
3. Test imports work from UI project
4. Rebuild TypeScript SDK and verify exports

**Validation**: Can import `import { Service as AccountService } from "@protosol/api"`

---

### Step 1.2: Setup gRPC Client Infrastructure  
**Objective**: Create centralized gRPC client management for server-side functions

**Actions**:
1. Create `ui/src/lib/grpc-clients.ts` with transport configuration
2. Implement client factory pattern with createGrpcTransport from connect-node
3. Create typed client interface for all services (account, transaction, program.system, program.token, rpc_client)
4. Add error handling and connection configuration
5. Export ready-to-use client instances

**Validation**: Server-side functions can import and use `grpcClients.account.generateNewKeyPair()`

---

### Step 1.3: Update Existing API Routes to Use Real gRPC Clients
**Objective**: Replace mock implementations with actual gRPC calls

**Actions**:
1. Update `ui/src/app/api/account/generateNewKeyPair/route.ts` to use real gRPC client
2. Import grpc clients and call `grpcClients.account.generateNewKeyPair()`
3. Handle gRPC errors and convert to HTTP responses
4. Test existing dashboard functionality works with real backend
5. Add proper request/response type validation

**Validation**: Existing dashboard generates real keypairs when backend is running

---

## Phase 2: UI Foundation Architecture

### Step 2.1: Create Layout with Sidebar Navigation
**Objective**: Implement the main dashboard structure with sidebar

**Actions**:
1. Create `ui/src/components/Sidebar.tsx` with navigation tree
2. Implement navigation structure matching proto services hierarchy:
   ```
   └── solana
       ├── account/v1
       ├── transaction/v1  
       ├── program
       │   ├── system/v1
       │   └── token/v1
       └── rpc_client/v1
   ```
3. Update `ui/src/app/layout.tsx` to include sidebar
4. Add routing for each service section
5. Style with Tailwind for clean dashboard appearance

**Validation**: Navigation works, sidebar shows service structure

---

### Step 2.2: Create Base Page Components Structure
**Objective**: Setup reusable page components for service interactions

**Actions**:
1. Create `ui/src/components/ServicePage.tsx` base component
2. Implement method selector dropdown (for choosing RPC methods)
3. Add request form builder (dynamic based on selected method)
4. Create response display area
5. Add loading states and error handling
6. Make components reusable across different services

**Validation**: Can render a page with method selection and form framework

---

## Phase 2.5: Backend Connectivity Validation (Confidence Building)

### Step 2.5: Create TypeScript Test Playground for gRPC Client Validation
**Objective**: Build confidence in gRPC client connectivity before proceeding with more UI features

**Actions**:
1. Create new yarn workspace member at `./tests/ts/`
2. Set up `package.json` with dependencies: `@protosol/api`, `@connectrpc/connect`, `@connectrpc/connect-node`
3. Create TypeScript configuration matching the UI project setup
4. Import ProtoSol API types the same way the UI does: `import { AccountService, createClient, createGrpcTransport } from "@protosol/api"`
5. Create test scripts to validate each service:
   - `test-account.ts`: Test AccountService methods (GetAccount, GenerateNewKeyPair)
   - `test-token.ts`: Test TokenProgramService methods (InitializeMint)
   - `test-system.ts`: Test SystemProgramService methods (Create, Transfer)
   - `test-rpc.ts`: Test RPCClientService methods (GetMinimumBalanceForRentExemption)
   - `test-transaction.ts`: Test basic transaction operations
6. Create `run-tests.ts` main script that calls all services systematically
7. Add npm scripts for easy execution: `yarn test-connectivity`, `yarn test-account`, etc.
8. Implement proper error handling and success/failure reporting
9. Add connection health check utility
10. Document test scenarios and expected results

**Key Requirements**:
- Use the exact same import patterns as the UI (`import from "@protosol/api"`)
- Use `createGrpcTransport` from `@connectrpc/connect-node` (same as UI backend)
- Implement similar error handling patterns to what we'll use in UI
- Test both success and failure scenarios
- Provide clear console output showing connectivity status

**Validation**: 
- All gRPC service calls work correctly from TypeScript
- Error scenarios are handled gracefully  
- Connection status can be verified programmatically
- Ready to proceed with full UI implementation with confidence

---

## Phase 3: Account Service Implementation

### Step 3.1: Implement Account Service Page
**Objective**: Complete account service UI with GetAccount method focus

**Actions**:
1. Create `ui/src/app/solana/account/v1/page.tsx`
2. Implement method selector for account service methods
3. Focus on GetAccount method (as per scope limiter)
4. Create form for GetAccount (address input, commitment level dropdown)
5. Create server action to call account service
6. Display account data results with proper formatting
7. Add error handling for account not found cases

**Validation**: Can successfully query account data from UI

---

### Step 3.2: Add GenerateNewKeyPair and FundNative to Account Page  
**Objective**: Complete account service functionality

**Actions**:
1. Add GenerateNewKeyPair method form (optional seed input)
2. Add FundNative method form (address, amount, commitment level)
3. Update server actions to handle all account methods
4. Add proper form validation and user feedback
5. Display generated keypairs with copy functionality
6. Show funding transaction signatures

**Validation**: All account service methods work from UI

---

## Phase 4: RPC Client Service Implementation

### Step 4.1: Implement RPC Client Service Page
**Objective**: Add basic RPC client functionality with minimal viable call

**Actions**:
1. Create `ui/src/app/solana/rpc_client/v1/page.tsx`
2. Implement method selector for RPC client service
3. Focus on one minimal RPC method as specified in scope limiter
4. Create appropriate form inputs based on the selected method
5. Create server action for RPC client calls
6. Display RPC results with proper formatting

**Validation**: Can make RPC calls through the UI interface

---

## Phase 5: Transaction Service Core Implementation

### Step 5.1: Create Transaction Draft Management
**Objective**: Enable draft transaction creation and instruction management

**Actions**:
1. Create `ui/src/app/solana/transaction/v1/page.tsx`
2. Implement method selector with all transaction service methods
3. Create draft transaction state management (React state or context)
4. Add "Create Draft Transaction" form (basic transaction setup)
5. Display current draft transaction state
6. Add instruction list component (empty initially)

**Validation**: Can create and display a draft transaction

---

### Step 5.2: Integrate Program Service Instruction Building
**Objective**: Connect program services to transaction instruction building

**Actions**:
1. Add "Add Instruction" section to transaction page
2. Create program selector dropdown (system, token)
3. Create method selector dropdown for selected program
4. Build dynamic forms for program method parameters
5. Create server actions to call program services (system.create, system.transfer, etc.)
6. Add returned SolanaInstruction to draft transaction
7. Display instruction list in transaction draft

**Validation**: Can add instructions from program services to draft transaction

---

### Step 5.3: Implement Transaction Compilation
**Objective**: Enable compilation of draft transactions

**Actions**:
1. Add CompileTransaction method form to transaction page
2. Implement fee payer input and recent blockhash handling
3. Create server action for transaction compilation
4. Update transaction state to COMPILED when successful
5. Display compiled transaction details
6. Show compilation errors with helpful messages
7. Auto-populate fee payer from previous operations when possible

**Validation**: Can compile draft transactions with instructions

---

## Phase 6: Transaction Lifecycle Completion

### Step 6.1: Implement Transaction Estimation and Simulation  
**Objective**: Add transaction analysis before signing

**Actions**:
1. Add EstimateTransaction form (commitment level selection)
2. Add SimulateTransaction form with same parameters
3. Create server actions for estimation and simulation
4. Display compute units, fee estimates, and priority fees
5. Display simulation results (success/failure, logs, errors)
6. Add warnings for failed simulations
7. Enable/disable signing based on simulation results

**Validation**: Can estimate costs and simulate transactions before signing

---

### Step 6.2: Implement Transaction Signing
**Objective**: Enable transaction signing with private keys

**Actions**:
1. Add SignTransaction form with private key inputs
2. Support multiple private keys for multi-sig scenarios
3. Create server action for transaction signing
4. Update transaction state to PARTIALLY_SIGNED or FULLY_SIGNED
5. Display signature information and signing status
6. Add private key validation and security warnings
7. Auto-populate signing flow from previous transaction state

**Validation**: Can sign compiled transactions and see signature status

---

### Step 6.3: Implement Transaction Submission and Monitoring
**Objective**: Complete transaction lifecycle with submission and monitoring

**Actions**:
1. Add SubmitTransaction form (commitment level selection)
2. Create server action for transaction submission
3. Display submission results and transaction signature
4. Add GetTransaction method for transaction lookup
5. Implement basic MonitorTransaction display (non-streaming initially)
6. Show transaction status progression
7. Add links to transaction explorers

**Validation**: Can submit signed transactions and monitor their status

---

## Phase 7: System Program Service Implementation

### Step 7.1: Create System Program Service Page
**Objective**: Dedicated interface for system program operations

**Actions**:
1. Create `ui/src/app/solana/program/system/v1/page.tsx`
2. Implement method selector for all system program methods
3. Focus on Create and Transfer methods initially
4. Build dynamic forms for system program parameters (payer, new_account, lamports, etc.)
5. Create server actions for system program calls
6. Display returned SolanaInstruction data
7. Add "Copy to Transaction" functionality to move instructions to transaction page

**Validation**: Can generate system program instructions and copy them to transaction workflow

---

### Step 7.2: Complete System Program Methods
**Objective**: Support all system program operations

**Actions**:
1. Add forms for remaining system program methods (Allocate, Assign, CreateWithSeed, etc.)
2. Implement nonce account operations (Initialize, Authorize, Withdraw, Advance, Upgrade)
3. Add seed-based operations with proper validation
4. Create specialized forms for each operation type
5. Add help text and examples for complex operations
6. Test all system program methods return valid instructions

**Validation**: All system program methods can be called and generate valid instructions

---

## Phase 8: Token Program Service Implementation  

### Step 8.1: Implement Token Program Service Page
**Objective**: Complete token program functionality

**Actions**:
1. Create `ui/src/app/solana/program/token/v1/page.tsx`
2. Implement method selector for token program methods
3. Add forms for token program operations (InitialiseMint, ParseMint, etc.)
4. Create server actions for token program calls
5. Display token program instruction results
6. Add token program specific validation and help text
7. Connect to transaction workflow for instruction copying

**Validation**: Can generate token program instructions and use them in transactions

---

## Phase 9: Enhanced User Experience

### Step 9.1: Add Cross-Page State Management
**Objective**: Enable seamless data flow between pages

**Actions**:
1. Implement React Context for shared state (accounts, keypairs, transactions)
2. Add "Use Previous Result" buttons across pages
3. Enable copying public keys from GenerateNewKeyPair to other forms
4. Auto-populate fee payer addresses from known accounts
5. Add "Quick Actions" for common workflows
6. Implement localStorage persistence for form data

**Validation**: Can use generated accounts and data across different pages

---

### Step 9.2: Add Advanced Form Features
**Objective**: Improve form usability and validation

**Actions**:
1. Add real-time form validation for addresses, amounts, etc.
2. Implement address validation (Base58, length checks)
3. Add amount formatting helpers (SOL ⟷ lamports conversion)
4. Create reusable input components for common types
5. Add copy/paste functionality for addresses and keys
6. Implement form state persistence and restoration
7. Add keyboard shortcuts for common operations

**Validation**: Forms provide immediate feedback and helpful validation

---

### Step 9.3: Enhanced Error Handling and Feedback
**Objective**: Provide clear error messages and status feedback

**Actions**:
1. Create comprehensive error boundary components
2. Add specific error messages for common gRPC failures
3. Implement retry logic for network errors
4. Add progress indicators for long-running operations
5. Create toast notifications for success/error feedback
6. Add connection status indicators for backend services
7. Implement graceful degradation when backend is unavailable

**Validation**: Users receive clear feedback on all operations and errors

---

## Phase 10: Testing and Polish

### Step 10.1: Add Integration Testing Support
**Objective**: Ensure reliability across the full stack

**Actions**:
1. Create test utilities for gRPC client mocking
2. Add component testing for key user flows  
3. Implement E2E tests for transaction lifecycle
4. Add performance testing for form interactions
5. Create test data fixtures for consistent testing
6. Add automated testing for all service integrations

**Validation**: Comprehensive test coverage with reliable test suite

---

### Step 10.2: Performance Optimization and Final Polish
**Objective**: Optimize performance and complete user experience

**Actions**:
1. Implement code splitting for service pages
2. Add React.memo for expensive components
3. Optimize bundle size and loading performance
4. Add proper meta tags and SEO (if applicable)
5. Implement dark mode support (optional)
6. Add keyboard navigation support
7. Polish visual design and responsive behavior
8. Add comprehensive documentation and help system

**Validation**: Fast, polished application with excellent user experience

---

## Technical Dependencies and Build Order

### Critical Path Dependencies:
1. **TypeScript SDK** must build cleanly before UI development
2. **gRPC client setup** must be complete before any server actions
3. **Base UI components** must exist before service-specific pages
4. **Transaction draft management** must work before instruction building
5. **Instruction building** must work before compilation/signing/submission

### Build Validation Points:
- After each step: `yarn workspace @protosol/api build` succeeds
- After each step: `yarn workspace protosol-ui typecheck` succeeds  
- After each step: `yarn workspace protosol-ui build` succeeds
- Integration testing: Backend running on port 50051, UI connects successfully
- E2E validation: Can complete full transaction lifecycle through UI

### Resource Management:
- Clean up form state between method selections
- Manage gRPC connection pooling in client factory
- Handle memory cleanup for large transaction/log displays
- Implement proper cleanup in useEffect hooks

### Security Considerations:
- Never log private keys in console or server logs
- Implement secure private key input handling
- Add warnings for mainnet operations (if applicable)
- Validate all inputs before sending to gRPC services

---

This implementation plan provides a systematic approach to building the complete ProtoSol UI dashboard with small, testable incremental steps that build upon each other to achieve the goal of a comprehensive transaction construction and submission interface.