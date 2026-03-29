<!-- BEGIN:nextjs-agent-rules -->
# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` before writing any code. Heed deprecation notices.
<!-- END:nextjs-agent-rules -->

---

# Protochain Dashboard — Agent Rules

These rules govern how an agent should work on this codebase. Read `initial-spec.md` for the full design; this file distills the non-negotiable constraints.

## Source of Truth

- **Proto definitions** (`lib/proto/protochain/solana/`) are the source of truth for all request/response types, field names, and field types. Do NOT hardcode field lists — read the `.proto` files.
- **`@protochain/ts-web`** (`lib/ts-web/`) is the API client. Read the generated `service_web_protochaints.ts` files for method signatures. Read `src/protochain/config/index.ts` for the functional options pattern (`WithServerUrl`, `WithLogging`).
- **`initial-spec.md`** is the design spec. Follow it for architecture, component structure, routing, and UX.
- **`method-page-status-tracker.md`** tracks which method pages are complete. Update it when you finish a page.

## Next.js 16 Breaking Changes (MUST follow)

1. `searchParams` and `params` in page/layout components are **Promises** — must be awaited or consumed with `use()`.
2. Middleware uses `proxy.ts` / `proxy` export, NOT `middleware.ts`.
3. `next lint` does not exist — use `eslint` directly.
4. React 19.2 — use `use()` hook where appropriate.
5. **Always consult `node_modules/next/dist/docs/`** before writing framework-level code (layouts, pages, route handlers, config).

## Architecture Rules

### Routing
- Routes map 1:1 to proto service paths. Version is merged with resource name: `account-v1/`, `program/system-v1/`, `program/token-v1/`, `transaction-v1/`, `rpc-client-v1/`.
- Kebab-case for all route segments.
- Every method gets its own `page.tsx` inside a folder named after the method in kebab-case.

### Under Construction Default
- Every method page that is NOT yet implemented MUST render the shared `UnderConstruction` component (`src/components/under-construction.tsx`).
- Do NOT build out a method page unless it is explicitly requested or listed in the current build phase.

### Client Components
- All method pages are `"use client"` — they need browser APIs for gRPC-Web transport.
- The `ProtochainProvider` is a client component.
- Layouts and error boundaries may be server or client components as appropriate.

### API Context
- All service clients come from `useProtochain()` context — never construct clients directly in page components.
- Clients are memoised on `serverUrl` — only reconstructed when the URL changes.
- Default URL: `http://localhost:50064`.

### Component Organization
- shadcn/ui primitives go in `src/components/ui/` (managed by the shadcn CLI).
- Custom shared components go in `src/components/` organized by concern: `sidebar/`, `method/`.
- Providers go in `src/providers/`.
- Data structures and utilities go in `src/lib/`.
- Do NOT put components inside `src/app/` route folders — keep route folders minimal (only `page.tsx`, `layout.tsx`, `error.tsx`).

### Error Handling
- Error boundaries (`error.tsx`) at each service-level layout: root, account-v1, program/system-v1, program/token-v1, transaction-v1, rpc-client-v1.
- RPC call errors are handled inline by `ResponsePanel`, NOT by error boundaries.
- Error boundaries catch rendering/runtime errors only.

### Forms & Fields
- Use the shared field components (`FieldInput`, `FieldSelect`, `FieldToggle`, etc.) for consistency.
- Use `RequestForm` wrapper for every method form — it provides the "Go" button and submit handling.
- Use `ResponsePanel` for every method response — it handles idle, loading, success (formatted JSON), and error states.

### Copy-to-Clipboard
- Any response field that is useful to copy (addresses, signatures, public keys, transaction data) MUST have a copy-to-clipboard button.

### Dark Mode
- Support light/dark/system toggle via `next-themes`. Default to system preference. Persist in localStorage.

### Query Parameters
- Pages that support query params read from `useSearchParams()` and use values as initial form state.
- Query params are for pre-populating forms, not for triggering submissions.

## Styling Rules

- Use Tailwind CSS v4 for all styling.
- Use shadcn/ui "new-york" style with zinc base color.
- Do NOT write custom CSS files — use Tailwind utilities.
- Respect dark mode — always use semantic color tokens from shadcn (e.g. `bg-background`, `text-foreground`), never hardcoded colors.

## Testing

- The standard testing flow is: `docker compose up -d` then `yarn workspace @protochain/solana-dashboard dev` and test with Playwright MCP against `http://localhost:3000`.
- The docker-compose stack exposes the API on `http://localhost:50064` behind Envoy.

### Screenshots

- **All Playwright screenshots MUST be saved to `.screenshots/`** inside the dashboard directory (`app/protochain/cmd/solana-dashboard/.screenshots/`).
- When calling `browser_take_screenshot`, ALWAYS set `filename` to `.screenshots/<descriptive-name>.png` (e.g. `.screenshots/fund-native-dark.png`).
- The `.screenshots/` directory is gitignored — files there will never be committed.
- Do NOT save screenshots to the repo root, `src/`, or any other tracked directory.
- The `.playwright-mcp/` directory is also gitignored — no cleanup needed.

## What NOT to Do

- Do NOT modify files in `lib/ts-web/` — that is generated code.
- Do NOT modify proto files — those are outside this app's scope.
- Do NOT install alternative UI libraries — use shadcn/ui.
- Do NOT add a backend/API route layer — the dashboard talks directly to the Protochain gRPC API via `@protochain/ts-web` in the browser.
- Do NOT build method pages that aren't in the current build phase — use the Under Construction component instead.
- Do NOT store request/response history — this was explicitly decided against.
