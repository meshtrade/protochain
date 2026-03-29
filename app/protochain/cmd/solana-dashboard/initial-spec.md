# Protochain Dashboard — Initial Specification

## Purpose

A Next.js 16 web application that provides a developer-facing UI for exercising every RPC method exposed by the Protochain Solana gRPC API. Think of it as a purpose-built Postman/gRPCurl replacement where the sidebar mirrors the proto service tree, each leaf is a form for one RPC method, and the response is rendered inline.

An agent will build this from scratch using this spec.

---

## Tech Stack

| Concern | Choice | Notes |
|---|---|---|
| Framework | Next.js 16 (App Router) | Already bootstrapped. See `AGENTS.md` — read `node_modules/next/dist/docs/` before writing code. |
| Language | TypeScript (strict) | tsconfig already configured |
| Styling | Tailwind CSS v4 | Already installed via `@tailwindcss/postcss` |
| Component library | shadcn/ui | Install via `npx shadcn@latest init`. Use the "new-york" style, zinc base color. |
| API client | `@protochain/ts-web` | Workspace sibling — add as dependency: `"@protochain/ts-web": "*"`. Also add `@bufbuild/protobuf` as a dependency (peer dep of ts-web). |
| State management | React Context | For the API context (protochain URL + constructed clients) |
| Forms | React Hook Form + zod | For request forms with validation |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│ Root Layout (src/app/layout.tsx)                        │
│ ┌─────────────────────┬───────────────────────────────┐ │
│ │ Sidebar             │ Main Content Area             │ │
│ │                     │ ┌───────────────────────────┐ │ │
│ │ [Protochain URL]    │ │ Breadcrumb                │ │ │
│ │                     │ ├───────────────────────────┤ │ │
│ │ ▾ Account V1        │ │ Request Form              │ │ │
│ │   ├ Get Account     │ │  [fields...]  [Go]        │ │ │
│ │   ├ Generate Key..  │ ├───────────────────────────┤ │ │
│ │   ├ Fund Native     │ │ Response                  │ │ │
│ │   ├ Get Token Bal.. │ │  (rendered JSON / table)  │ │ │
│ │   └ Get Assoc. ..   │ └───────────────────────────┘ │ │
│ │ ▾ Program           │                               │ │
│ │   ▾ System V1       │                               │ │
│ │     ├ Create        │                               │ │
│ │     ├ Transfer      │                               │ │
│ │     ├ Allocate      │                               │ │
│ │     └ ...           │                               │ │
│ │   ▾ Token V1        │                               │ │
│ │     ├ Create 2022.. │                               │ │
│ │     ├ Create SPL..  │                               │ │
│ │     ├ Parse Mint    │                               │ │
│ │     └ ...           │                               │ │
│ │ ▾ Transaction V1    │                               │ │
│ │   ├ Compile         │                               │ │
│ │   ├ Estimate        │                               │ │
│ │   └ ...             │                               │ │
│ │ ▾ RPC Client V1     │                               │ │
│ │   └ Get Min Bal..   │                               │ │
│ └─────────────────────┴───────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

---

## Core Concepts

### 1. API Context (`ProtochainProvider`)

A React context that holds:
- `serverUrl: string` — the Protochain backend URL (default: `http://localhost:50064`)
- `setServerUrl(url: string): void` — update the URL; reconstructs all clients
- One instance of each service client from `@protochain/ts-web`:
  - `accountService: AccountServiceWeb`
  - `transactionService: TransactionServiceWeb`
  - `programSystemService: ProgramSystemServiceWeb`
  - `programTokenService: ProgramTokenServiceWeb`
  - `rpcClientService: RpcClientServiceWeb`

When the user changes the URL in the sidebar input, all clients are reconstructed with the new URL via `WithServerUrl(newUrl)`. Clients are memoised — only reconstructed when the URL actually changes.

The provider wraps the entire app in the root layout. Because the clients use `@connectrpc/connect-web` (browser API), the provider must be a client component (`"use client"`).

### 2. Sidebar Navigation

The sidebar is a **collapsible tree** that mirrors the proto service structure. It is always visible in the left panel.

**Tree structure:**

```
Account V1
├── Get Account
├── Generate New Key Pair
├── Fund Native
├── Get Token Account Balance
└── Get Associated Token Address

Program
├── System V1
│   ├── Create
│   ├── Transfer
│   ├── Allocate
│   ├── Assign
│   ├── Create With Seed
│   ├── Allocate With Seed
│   ├── Assign With Seed
│   ├── Transfer With Seed
│   ├── Initialize Nonce Account
│   ├── Authorize Nonce Account
│   ├── Withdraw Nonce Account
│   ├── Advance Nonce Account
│   └── Upgrade Nonce Account
└── Token V1
    ├── Create Token 2022 Mint
    ├── Create SPL Token Mint
    ├── Parse Mint
    ├── Create Token 2022 Holding Account
    ├── Create SPL Token Holding Account
    └── Mint

Transaction V1
├── Compile Transaction
├── Estimate Transaction
├── Simulate Transaction
├── Sign Transaction
├── Check If Transaction Is Expired
├── Submit Transaction
├── Get Transaction
└── Monitor Transaction

RPC Client V1
└── Get Minimum Balance For Rent Exemption
```

**Behavior:**
- The entire sidebar is collapsible (toggle button to hide/show it, freeing the full viewport for the main content)
- Category nodes (Account V1, Program, System V1, etc.) are collapsible — click toggles open/closed
- Leaf nodes (Get Account, Transfer, etc.) are links that navigate to the method page
- **Active page indicator**: The currently active leaf (matching `usePathname()`) must be visually distinct — bold text + accent background highlight + left border indicator. The parent group(s) of the active leaf should auto-expand when the page loads.
- The sidebar should be resizable or have a sensible fixed width (~280px)

**Protochain URL selector** (above the tree):

Use a combobox / select-with-custom-input pattern (shadcn `Popover` + `Command` or similar). The user can pick from preset URLs or type a custom one. Presets:

| Label | URL |
|---|---|
| Local (Docker Compose) | `http://localhost:50064` |
| Production | `https://protochain.mesh.trade` |
| Test | `https://protochain-test.mesh.trade` |

Default selection: **Local (Docker Compose)** — this is the most common development flow (the agent will test with Playwright against the docker-compose stack).

When the user selects a preset, the URL is set. When the user types a custom URL, it replaces the selection. The selected/typed URL is what gets passed to `WithServerUrl()` in the `ProtochainProvider`.

### 3. Folder-Based Routing

Routes map 1:1 to proto service paths. The version is merged with the resource name (no extra version folder). Kebab-case throughout.

```
src/app/
├── layout.tsx                          # Root layout: sidebar + ProtochainProvider
├── page.tsx                            # Home / landing (brief intro + links)
├── error.tsx                           # Root error boundary
│
├── account-v1/
│   ├── layout.tsx                      # Service-level layout (breadcrumb prefix)
│   ├── error.tsx                       # Service-level error boundary
│   ├── get-account/
│   │   └── page.tsx
│   ├── generate-new-key-pair/
│   │   └── page.tsx
│   ├── fund-native/
│   │   └── page.tsx
│   ├── get-token-account-balance/
│   │   └── page.tsx
│   └── get-associated-token-address/
│       └── page.tsx
│
├── program/
│   ├── layout.tsx                      # Program group layout
│   ├── system-v1/
│   │   ├── layout.tsx
│   │   ├── error.tsx
│   │   ├── create/
│   │   │   └── page.tsx
│   │   ├── transfer/
│   │   │   └── page.tsx
│   │   ├── allocate/
│   │   │   └── page.tsx
│   │   ├── assign/
│   │   │   └── page.tsx
│   │   ├── create-with-seed/
│   │   │   └── page.tsx
│   │   ├── allocate-with-seed/
│   │   │   └── page.tsx
│   │   ├── assign-with-seed/
│   │   │   └── page.tsx
│   │   ├── transfer-with-seed/
│   │   │   └── page.tsx
│   │   ├── initialize-nonce-account/
│   │   │   └── page.tsx
│   │   ├── authorize-nonce-account/
│   │   │   └── page.tsx
│   │   ├── withdraw-nonce-account/
│   │   │   └── page.tsx
│   │   ├── advance-nonce-account/
│   │   │   └── page.tsx
│   │   └── upgrade-nonce-account/
│   │       └── page.tsx
│   └── token-v1/
│       ├── layout.tsx
│       ├── error.tsx
│       ├── create-token-2022-mint/
│       │   └── page.tsx
│       ├── create-spl-token-mint/
│       │   └── page.tsx
│       ├── parse-mint/
│       │   └── page.tsx
│       ├── create-token-2022-holding-account/
│       │   └── page.tsx
│       ├── create-spl-token-holding-account/
│       │   └── page.tsx
│       └── mint/
│           └── page.tsx
│
├── transaction-v1/
│   ├── layout.tsx
│   ├── error.tsx
│   ├── compile-transaction/
│   │   └── page.tsx
│   ├── estimate-transaction/
│   │   └── page.tsx
│   ├── simulate-transaction/
│   │   └── page.tsx
│   ├── sign-transaction/
│   │   └── page.tsx
│   ├── check-if-transaction-is-expired/
│   │   └── page.tsx
│   ├── submit-transaction/
│   │   └── page.tsx
│   ├── get-transaction/
│   │   └── page.tsx
│   └── monitor-transaction/
│       └── page.tsx
│
└── rpc-client-v1/
    ├── layout.tsx
    ├── error.tsx
    └── get-minimum-balance-for-rent-exemption/
        └── page.tsx
```

### 4. URL Query Parameters

Some methods benefit from pre-populated fields via URL query params. This enables cross-linking between pages (e.g. after generating a key pair, link to Fund Native with the address pre-filled).

**Next.js 16 note:** `searchParams` is async in page components — must be awaited.

Methods that support query params:

| Route | Query Params | Pre-populates |
|---|---|---|
| `/account-v1/get-account` | `?address=` | Address field |
| `/account-v1/fund-native` | `?address=&amount=` | Address, Amount fields |
| `/account-v1/get-token-account-balance` | `?address=` | Address field |
| `/account-v1/get-associated-token-address` | `?owner=&mint=` | Owner, Mint fields |
| `/program/token-v1/parse-mint` | `?address=` | Account Address field |
| `/program/system-v1/transfer` | `?from=&to=&lamports=` | From, To, Lamports fields |
| `/transaction-v1/get-transaction` | `?signature=` | Signature field |
| `/transaction-v1/monitor-transaction` | `?signature=` | Signature field |

Any method page MAY support query params for its fields — the above are the priority ones. The pattern is: read `searchParams`, use values as `defaultValues` in the form.

---

## Component Structure

```
src/
├── app/                         # Route pages (see above)
├── components/
│   ├── ui/                      # shadcn/ui primitives (auto-generated by shadcn CLI)
│   │   ├── button.tsx
│   │   ├── input.tsx
│   │   ├── select.tsx
│   │   ├── card.tsx
│   │   ├── collapsible.tsx
│   │   ├── badge.tsx
│   │   ├── separator.tsx
│   │   ├── scroll-area.tsx
│   │   ├── sidebar.tsx          # shadcn sidebar component
│   │   └── ...
│   ├── sidebar/
│   │   ├── app-sidebar.tsx      # Main sidebar: URL input + navigation tree
│   │   └── nav-tree.tsx         # Recursive tree node component
│   ├── method/
│   │   ├── request-form.tsx     # Generic wrapper: form card with "Go" button
│   │   ├── response-panel.tsx   # Response display: loading / success JSON / error
│   │   ├── transaction-monitor.tsx # Reusable streaming transaction monitor panel
│   │   ├── field-input.tsx      # Text input field (string, uint64, bytes)
│   │   ├── field-select.tsx     # Enum select field (CommitmentLevel, TokenProgram, etc.)
│   │   ├── field-toggle.tsx     # Boolean toggle field
│   │   ├── field-repeated.tsx   # Repeated field: add/remove items
│   │   ├── field-message.tsx    # Nested message field: renders sub-fields in a bordered group
│   │   └── field-oneof.tsx      # Oneof selector: radio/select to pick variant, then render sub-fields
│   ├── breadcrumb.tsx           # Breadcrumb built from route segments
│   └── error-boundary.tsx       # Reusable error boundary component
├── providers/
│   └── protochain-provider.tsx  # ProtochainProvider context (client component)
└── lib/
    ├── navigation.ts            # Sidebar tree data structure (services → methods → routes)
    └── utils.ts                 # shadcn cn() utility
```

---

## shadcn/ui Components Needed

Install these via `npx shadcn@latest add <name>`:

- `button` — Go button, sidebar toggles
- `input` — text/number fields
- `select` — enum dropdowns (CommitmentLevel, TokenProgram, etc.)
- `card` — Request and Response panels
- `collapsible` — sidebar tree sections
- `badge` — status indicators (loading, success, error)
- `separator` — visual dividers
- `scroll-area` — sidebar scroll
- `sidebar` — shadcn sidebar primitive (layout + toggle)
- `breadcrumb` — page breadcrumbs
- `label` — form labels
- `switch` — boolean fields
- `textarea` — large text fields (transaction data, JSON)
- `alert` — error display
- `skeleton` — loading states
- `command` — combobox for URL selector (used with `popover`)
- `popover` — combobox container for URL selector

---

## Page Component Pattern

Every method page follows the same structure. Here is the canonical pattern:

```tsx
// src/app/account-v1/generate-new-key-pair/page.tsx
"use client";

import { useState } from "react";
import { useProtochain } from "@/providers/protochain-provider";
import { RequestForm } from "@/components/method/request-form";
import { ResponsePanel } from "@/components/method/response-panel";
import { FieldInput } from "@/components/method/field-input";

export default function GenerateNewKeyPairPage() {
  const { accountService } = useProtochain();
  const [response, setResponse] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  // Form state
  const [seed, setSeed] = useState("");

  async function handleSubmit() {
    setLoading(true);
    setError(null);
    try {
      const res = await accountService.generateNewKeyPair({ seed });
      setResponse(res);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <>
      <RequestForm onSubmit={handleSubmit} loading={loading}>
        <FieldInput label="Seed" value={seed} onChange={setSeed} placeholder="Optional hex-encoded seed" />
      </RequestForm>
      <ResponsePanel data={response} error={error} loading={loading} />
    </>
  );
}
```

**Key rules:**
- Every page is `"use client"` (needs browser APIs for gRPC-Web transport)
- Gets its service client from `useProtochain()` context
- Uses a `RequestForm` wrapper that provides the "Go" button and submit handling
- Uses `ResponsePanel` to display the result as formatted JSON
- For pages with query param support: read from `useSearchParams()` and set as initial form state

---

## Field Type Mapping

| Proto Type | Component | Notes |
|---|---|---|
| `string` | `FieldInput` (text) | Base58 addresses, seeds, signatures |
| `uint32` / `uint64` | `FieldInput` (text) | Rendered as text, validated as numeric. uint64 uses string representation. |
| `bool` | `FieldToggle` (switch) | |
| `bytes` | `FieldInput` (textarea) | Base64 encoded |
| `enum` (CommitmentLevel, TokenProgram, etc.) | `FieldSelect` | Dropdown with human-readable labels |
| `message` (nested) | `FieldMessage` | Bordered group containing the message's sub-fields |
| `repeated string` | `FieldRepeated` | Add/remove list of text inputs |
| `repeated message` | `FieldRepeated` + `FieldMessage` | Add/remove list of nested message groups |
| `oneof` | `FieldOneof` | Radio/select to pick variant, then render the chosen variant's sub-fields |
| `map<string, string>` | Key-value editor | Add/remove rows of key + value inputs |

---

## Response Rendering

The `ResponsePanel` component handles three states:

1. **Idle** — empty state, light grey background, "Response will appear here"
2. **Loading** — skeleton / spinner
3. **Success** — formatted JSON tree with syntax highlighting. For well-known fields:
   - Addresses (base58 strings) — monospace font
   - Lamport amounts — show both lamports and SOL conversion
   - Transaction state enums — colored badge
4. **Error** — red alert with error message, gRPC status code if available

---

## Error Boundaries

Error boundaries are placed at each service-level layout:
- `src/app/error.tsx` — root fallback
- `src/app/account-v1/error.tsx`
- `src/app/program/system-v1/error.tsx`
- `src/app/program/token-v1/error.tsx`
- `src/app/transaction-v1/error.tsx`
- `src/app/rpc-client-v1/error.tsx`

Each error boundary shows the error message with a "Try Again" button that resets the boundary. These catch rendering errors; RPC call errors are handled inline by `ResponsePanel`.

---

## Streaming RPC: Monitor Transaction

`MonitorTransaction` is the only server-streaming RPC. Its page differs from the standard pattern:

- The response panel shows a **live log** of `MonitorTransactionResponse` messages as they arrive
- Each message is appended to a scrollable list showing: status badge, slot, commitment level, logs (if requested)
- A "Stop" button cancels the stream via `AbortController`
- The form has: signature (string), commitment level (enum select), include logs (toggle), timeout seconds (number)

---

## FundNative Flow Detail

The FundNative page is more than a simple request/response — it chains into transaction monitoring to show the user the full lifecycle of their airdrop.

**Step 1 — Request form:**
- Fields: `address` (string), `amount` (string, lamports), `commitment_level` (enum: CommitmentLevel)
- User clicks "Go"

**Step 2 — Call `accountService.fundNative()`:**
- On success, the response contains a `signature` (string) — this is the transaction signature
- Display the signature in the response panel with a copy-to-clipboard button

**Step 3 — Automatically start monitoring:**
- Immediately after receiving the signature, call `transactionService.monitorTransaction()` with:
  - `signature`: the signature from step 2
  - `commitment_level`: same commitment level the user selected (or CONFIRMED as default)
  - `include_logs`: true
  - `timeout_seconds`: 60
- This is a server-streaming RPC — messages arrive over time

**Step 4 — Live monitoring panel:**
- Below the initial response, show a "Transaction Monitor" panel
- As `MonitorTransactionResponse` messages stream in, render each one showing:
  - Status badge (color-coded: processing=yellow, confirmed=blue, finalized=green, failed=red)
  - Slot number
  - Current commitment level achieved
  - Program logs (if present)
  - Compute units consumed
- The panel auto-scrolls to the latest message
- A "Stop" button allows the user to cancel monitoring via `AbortController`
- When the stream completes (transaction reaches target commitment or times out), show a final status summary

**Implementation notes:**
- This page needs both `accountService` and `transactionService` from the `ProtochainProvider`
- The monitoring panel should be a reusable component (`src/components/method/transaction-monitor.tsx`) since it will be reused later when we build other transaction-related pages
- The streaming logic uses async iteration over the Connect-ES streaming response

---

## Navigation Data Structure

Define the sidebar tree as a static data structure in `src/lib/navigation.ts`:

```typescript
export interface NavLeaf {
  label: string;       // Display name: "Generate New Key Pair"
  href: string;        // Route path: "/account-v1/generate-new-key-pair"
}

export interface NavGroup {
  label: string;       // Display name: "Account V1"
  children: (NavGroup | NavLeaf)[];
}

export const navigationTree: NavGroup[] = [
  {
    label: "Account V1",
    children: [
      { label: "Get Account", href: "/account-v1/get-account" },
      { label: "Generate New Key Pair", href: "/account-v1/generate-new-key-pair" },
      { label: "Fund Native", href: "/account-v1/fund-native" },
      { label: "Get Token Account Balance", href: "/account-v1/get-token-account-balance" },
      { label: "Get Associated Token Address", href: "/account-v1/get-associated-token-address" },
    ],
  },
  {
    label: "Program",
    children: [
      {
        label: "System V1",
        children: [
          { label: "Create", href: "/program/system-v1/create" },
          { label: "Transfer", href: "/program/system-v1/transfer" },
          { label: "Allocate", href: "/program/system-v1/allocate" },
          { label: "Assign", href: "/program/system-v1/assign" },
          { label: "Create With Seed", href: "/program/system-v1/create-with-seed" },
          { label: "Allocate With Seed", href: "/program/system-v1/allocate-with-seed" },
          { label: "Assign With Seed", href: "/program/system-v1/assign-with-seed" },
          { label: "Transfer With Seed", href: "/program/system-v1/transfer-with-seed" },
          { label: "Initialize Nonce Account", href: "/program/system-v1/initialize-nonce-account" },
          { label: "Authorize Nonce Account", href: "/program/system-v1/authorize-nonce-account" },
          { label: "Withdraw Nonce Account", href: "/program/system-v1/withdraw-nonce-account" },
          { label: "Advance Nonce Account", href: "/program/system-v1/advance-nonce-account" },
          { label: "Upgrade Nonce Account", href: "/program/system-v1/upgrade-nonce-account" },
        ],
      },
      {
        label: "Token V1",
        children: [
          { label: "Create Token 2022 Mint", href: "/program/token-v1/create-token-2022-mint" },
          { label: "Create SPL Token Mint", href: "/program/token-v1/create-spl-token-mint" },
          { label: "Parse Mint", href: "/program/token-v1/parse-mint" },
          { label: "Create Token 2022 Holding Account", href: "/program/token-v1/create-token-2022-holding-account" },
          { label: "Create SPL Token Holding Account", href: "/program/token-v1/create-spl-token-holding-account" },
          { label: "Mint", href: "/program/token-v1/mint" },
        ],
      },
    ],
  },
  {
    label: "Transaction V1",
    children: [
      { label: "Compile Transaction", href: "/transaction-v1/compile-transaction" },
      { label: "Estimate Transaction", href: "/transaction-v1/estimate-transaction" },
      { label: "Simulate Transaction", href: "/transaction-v1/simulate-transaction" },
      { label: "Sign Transaction", href: "/transaction-v1/sign-transaction" },
      { label: "Check If Transaction Is Expired", href: "/transaction-v1/check-if-transaction-is-expired" },
      { label: "Submit Transaction", href: "/transaction-v1/submit-transaction" },
      { label: "Get Transaction", href: "/transaction-v1/get-transaction" },
      { label: "Monitor Transaction", href: "/transaction-v1/monitor-transaction" },
    ],
  },
  {
    label: "RPC Client V1",
    children: [
      { label: "Get Minimum Balance For Rent Exemption", href: "/rpc-client-v1/get-minimum-balance-for-rent-exemption" },
    ],
  },
];
```

---

## `@protochain/ts-web` Integration Details

### Imports

```typescript
// Service clients
import { AccountServiceWeb } from "@protochain/ts-web/solana/account/v1";
import { TransactionServiceWeb } from "@protochain/ts-web/solana/transaction/v1";
import { ProgramSystemServiceWeb } from "@protochain/ts-web/solana/program/system/v1";
import { ProgramTokenServiceWeb } from "@protochain/ts-web/solana/program/token/v1";
import { RpcClientServiceWeb } from "@protochain/ts-web/solana/rpc_client/v1";

// Configuration
import { WithServerUrl, WithLogging } from "@protochain/ts-web/config";

// Request/response types (import from same subpath as client)
import { GenerateNewKeyPairRequest } from "@protochain/ts-web/solana/account/v1";

// Shared types
import { CommitmentLevel } from "@protochain/ts-web/solana/type/v1";
import { TokenProgram } from "@protochain/ts-web/solana/program/token/v1";
```

### Client Construction (in ProtochainProvider)

```typescript
const clients = useMemo(() => ({
  accountService: new AccountServiceWeb(WithServerUrl(serverUrl), WithLogging()),
  transactionService: new TransactionServiceWeb(WithServerUrl(serverUrl), WithLogging()),
  programSystemService: new ProgramSystemServiceWeb(WithServerUrl(serverUrl), WithLogging()),
  programTokenService: new ProgramTokenServiceWeb(WithServerUrl(serverUrl), WithLogging()),
  rpcClientService: new RpcClientServiceWeb(WithServerUrl(serverUrl), WithLogging()),
}), [serverUrl]);
```

---

## Method Page Status Tracker

See [method-page-status-tracker.md](./method-page-status-tracker.md) — maintained separately so the agent can update it as pages are completed without modifying this spec.

---

## Next.js 16 Considerations

These are breaking changes in Next.js 16 that the agent MUST handle:

1. **Async `searchParams`**: In page components, `searchParams` is a Promise and must be awaited. Use the async page function signature or `use()`.
2. **`proxy.ts` not `middleware.ts`**: If middleware is needed later, use the new naming convention.
3. **No `next lint`**: Use `eslint` CLI directly. The project already has `eslint.config.mjs` with flat config.
4. **React 19.2**: Use React 19 patterns. `use()` hook is available.
5. **Read `node_modules/next/dist/docs/`**: Before writing any code, consult the docs for current API signatures.

---

## Implementation Order

For the agent building this, follow this order:

### Phase 1 — Skeleton
1. Install dependencies: `@protochain/ts-web`, `@bufbuild/protobuf`, `react-hook-form`, `zod`, `@hookform/resolvers`
2. Initialize shadcn/ui (`npx shadcn@latest init`) and install needed components
3. Create `src/lib/utils.ts` (shadcn utility) and `src/lib/navigation.ts`
4. Create `src/providers/protochain-provider.tsx`
5. Create the root layout with sidebar + provider
6. Create the home page

### Phase 2 — Shared Components
7. Build `src/components/sidebar/app-sidebar.tsx` and `nav-tree.tsx`
8. Build `src/components/method/request-form.tsx`
9. Build `src/components/method/response-panel.tsx`
10. Build field components: `field-input.tsx`, `field-select.tsx`, `field-toggle.tsx`, `field-repeated.tsx`, `field-message.tsx`, `field-oneof.tsx`
11. Build `src/components/breadcrumb.tsx`
12. Build error boundary components

### Phase 3 — Under Construction Pages + Initial 3 Method Pages

All method routes get created in this phase, but every page starts as an "Under Construction" splash. Then, implement the following 3 methods as fully functional pages:

13. Create the shared "Under Construction" component (`src/components/under-construction.tsx`) — centered layout, construction icon, method name, "Coming Soon" text
14. Create all route folders and `page.tsx` files for every method listed in the sidebar tree — each one renders the Under Construction component with the method name
15. `account-v1/generate-new-key-pair` — simplest form (one optional string field). Query param support: none needed.
16. `account-v1/fund-native` — string + string + enum (CommitmentLevel). Query param support: `?address=&amount=`. **See "FundNative Flow Detail" section below for the full UX.**
17. `program/token-v1/parse-mint` — single string field, complex nested response (MintInfo, extensions, metaplex metadata). Query param support: `?address=`

### Phase 4 — Polish
18. Query parameter support on the 3 implemented routes (as listed above)
19. Cross-linking: GenerateNewKeyPair response should link to FundNative with the generated address pre-filled
20. Copy-to-clipboard buttons on useful response fields (addresses, signatures, public keys)
21. Verify all error boundaries work
22. Test against a live backend
23. Verify dark/light mode toggle works correctly

---

## Resolved Decisions

1. **Dark mode**: Yes. Support light/dark/system toggle. Use `next-themes` (standard shadcn approach). Persist the user's choice in localStorage. Default to system preference.
2. **Response history**: No. No history of any kind.
3. **Transaction builder**: Deferred. Individual method pages are sufficient for v1. We will revisit when we implement the transaction service pages.
4. **Copy-to-clipboard**: Yes. Any response field that is useful to copy (addresses, signatures, public keys, transaction data) should have a copy-to-clipboard button.
5. **Port**: Default 3000 is fine. No custom port configuration needed.
