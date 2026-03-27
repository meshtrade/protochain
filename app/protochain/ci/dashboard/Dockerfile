# ==============================================================================
# Protochain Dashboard — Multi-stage Docker build
#
# Build context: repository root (required for yarn workspace resolution)
#
# PREREQUISITE: The @protochain/ts-web generated source files (_pb.ts,
# _protochaints.ts) must exist before running this build. In CI, the workflow
# runs code generation first. For local builds:
#
#   ./scripts/code-gen/generate/all.sh   # generates the _pb.ts files
#   docker build -f app/protochain/ci/dashboard/Dockerfile -t protochain-dashboard .
#
# Run:
#   docker run -p 3000:3000 protochain-dashboard
#
# Environment variables:
#   PORT      — Server port (default: 3000)
#   HOSTNAME  — Bind address (default: 0.0.0.0)
# ==============================================================================

# ---------------------------------------------------------------------------
# Stage 1: Install dependencies
# ---------------------------------------------------------------------------
FROM node:20-alpine AS deps

RUN apk add --no-cache libc6-compat

WORKDIR /app

# Copy workspace root files needed for dependency resolution
COPY package.json yarn.lock ./

# Copy workspace packages (only package.json for layer caching)
COPY lib/ts-web/package.json lib/ts-web/
COPY app/protochain/cmd/dashboard/package.json app/protochain/cmd/dashboard/

# Install all dependencies (frozen lockfile for reproducibility)
RUN yarn install --frozen-lockfile --network-timeout 300000

# ---------------------------------------------------------------------------
# Stage 2: Build the Next.js application
# ---------------------------------------------------------------------------
FROM node:20-alpine AS builder

WORKDIR /app

# Copy installed node_modules from deps stage
COPY --from=deps /app/node_modules ./node_modules
COPY --from=deps /app/lib/ts-web/node_modules ./lib/ts-web/node_modules
COPY --from=deps /app/app/protochain/cmd/dashboard/node_modules ./app/protochain/cmd/dashboard/node_modules

# Copy workspace root config
COPY package.json yarn.lock ./

# Copy the ts-web library source (includes generated _pb.ts files from CI)
COPY lib/ts-web/ lib/ts-web/

# Build ts-web (compile TS source → dist/)
RUN yarn workspace @protochain/ts-web build

# Copy the dashboard source
COPY app/protochain/cmd/dashboard/ app/protochain/cmd/dashboard/

# Build the dashboard (standalone output mode)
RUN yarn workspace @protochain/dashboard build

# ---------------------------------------------------------------------------
# Stage 3: Production runtime (minimal image)
# ---------------------------------------------------------------------------
FROM node:20-alpine AS runner

WORKDIR /app

ENV NODE_ENV=production
ENV PORT=3000
ENV HOSTNAME=0.0.0.0

RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 nextjs

# Copy public assets
COPY --from=builder /app/app/protochain/cmd/dashboard/public ./public

# Copy the standalone server (includes only traced dependencies)
COPY --from=builder --chown=nextjs:nodejs \
    /app/app/protochain/cmd/dashboard/.next/standalone ./

# Copy static assets (JS/CSS bundles)
COPY --from=builder --chown=nextjs:nodejs \
    /app/app/protochain/cmd/dashboard/.next/static \
    ./app/protochain/cmd/dashboard/.next/static

USER nextjs

EXPOSE 3000

# The standalone server.js is output at the workspace package path
CMD ["node", "app/protochain/cmd/dashboard/server.js"]
