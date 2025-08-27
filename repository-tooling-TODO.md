# Repository Tooling Improvements TODO

*Generated from first integration journey - 2025-08-27*

This file tracks tooling improvements needed to enhance development experience and reliability in the protosol repository.

## 🚨 Critical Fixes Needed

### 1. Script Path Issues
**Problem**: Scripts contain hardcoded path calculations that are incorrect
- `scripts/tests/start-backend.sh` had wrong `PROJECT_ROOT` calculation (went up 3 levels instead of 2)
- Legacy references to `project/solana/cmd/api` instead of current `api` workspace member

**Fix**: ✅ FIXED - Updated path calculation and workspace member references

### 2. Legacy Code References  
**Problem**: Configuration code contains references to old project structure
- `tests/go/config/config.go` looked for `api-test` directory instead of `tests/go`
- Functions like `hasProtosolMarkers()` referenced non-existent files

**Fix**: ✅ FIXED - Updated to use current project structure markers

### 3. Port Conflict Management
**Problem**: Services can start on already-used ports without clear error handling
- Backend failed with "Address already in use" but continued silently
- No automatic port conflict detection

**Status**: Needs tooling improvement

## 🔧 Development Experience Improvements

### 1. **Unified Development Script**
**Need**: Single script to start entire stack in correct order
```bash
# Proposed: scripts/dev/start-all.sh
./scripts/dev/start-all.sh
# Should: stop existing services → start validator → start backend → verify health
```

### 2. **Health Check Tooling** 
**Need**: Scripts to verify entire stack health
```bash
# Proposed: scripts/dev/health-check.sh  
./scripts/dev/health-check.sh
# Should verify: validator responding, backend gRPC healthy, ports available
```

### 3. **Test Environment Reset**
**Need**: Clean slate testing capability
```bash
# Proposed: scripts/dev/reset-test-env.sh
./scripts/dev/reset-test-env.sh
# Should: stop all services, clean ledger, restart fresh, run smoke test
```

### 4. **Service Status Dashboard**
**Need**: Quick way to see what's running
```bash
# Proposed: scripts/dev/status.sh
./scripts/dev/status.sh
# Output:
# ✅ Solana Validator: Running (PID: 12345, Port: 8899)
# ✅ Backend Server: Running (PID: 67890, Port: 50051)  
# ❌ UI Server: Not running
```

## 🐛 Bug Fixes Completed

### 1. Start Backend Script - Path Calculation 
**Issue**: `PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"` went up 3 levels instead of 2
**Fix**: Changed to `PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"`
**Location**: `scripts/tests/start-backend.sh:9`

### 2. Start Backend Script - Workspace Member Check
**Issue**: `grep -q "project/solana/cmd/api" Cargo.toml` looked for non-existent workspace member  
**Fix**: Changed to `grep -q "api" Cargo.toml`
**Location**: `scripts/tests/start-backend.sh:33`

### 3. Go Test Config - Directory Structure
**Issue**: `findAPITestRoot()` looked for `api-test` directory that doesn't exist
**Fix**: Updated to look for `tests/go` directory structure
**Location**: `tests/go/config/config.go:62-90`

### 4. Go Test Config - Project Markers  
**Issue**: `hasProtosolMarkers()` looked for non-existent files like `CLAUDE.md`, `project/solana`
**Fix**: Updated to look for actual files: `claude.md`, `buf.yaml`, `lib/proto`, etc.
**Location**: `tests/go/config/config.go:126-142`

## 🚀 Performance & Reliability

### 1. **Parallel Service Startup**
**Current**: Services start sequentially, slow startup time
**Proposed**: Start validator and backend in parallel, with proper health checks

### 2. **Automatic Dependency Management**
**Need**: Services should auto-restart dependencies
- Backend should auto-restart if validator dies
- Tests should auto-verify services are ready

### 3. **Integrated Logging**
**Need**: Centralized log aggregation for debugging
```bash
# Proposed: scripts/dev/logs.sh
./scripts/dev/logs.sh --follow --service=all
# Should tail logs from validator, backend, tests in unified format
```

## 🧪 Testing Infrastructure

### 1. **Test Result Analysis**
**Current Issue**: Failed tests (3/9) need investigation:
- Transaction estimation returning 0 compute units
- Transaction signing flow failures  
- Complete composable flow dependencies

### 2. **Smoke Testing**
**Need**: Quick validation that basic functionality works
```bash
# Proposed: scripts/tests/smoke.sh
./scripts/tests/smoke.sh
# Should: verify basic account creation, simple transfer, service health
```

### 3. **Load Testing Setup**
**Future**: Performance testing infrastructure for gRPC services

## 📊 Monitoring & Observability  

### 1. **Metrics Collection**
**Need**: Basic metrics for service health
- gRPC request count/latency
- Transaction success/failure rates
- Validator sync status

### 2. **Error Aggregation**
**Need**: Common error patterns collection and reporting

## 🔒 Security & Validation

### 1. **Service Security**
**Current**: No authentication on gRPC services (OK for local dev)
**Future**: Add auth for production deployment guides

### 2. **Input Validation** 
**Need**: Enhanced validation for:
- Address format validation in CLI tools
- Amount/balance validation helpers
- Transaction parameter validation

## 📚 Documentation Tooling

### 1. **API Documentation Generation**
**Need**: Auto-generate API docs from proto files
**Proposed**: Use buf to generate HTML/markdown docs

### 2. **Command Reference**
**Need**: Auto-generated command reference for all scripts
**Format**: Help system for `scripts/dev/` commands

## 🎯 Priority Ranking

### High Priority (Should implement soon)
1. ✅ **Bug fixes** (COMPLETED)  
2. **Unified development script** - Major DX improvement
3. **Health check tooling** - Essential for debugging
4. **Service status dashboard** - Daily development aid

### Medium Priority  
1. **Test environment reset** - Useful for clean testing
2. **Integrated logging** - Helps with debugging complex issues
3. **Test result analysis** - Fix the 3 failing tests

### Low Priority (Future)
1. **Performance tooling** - After core functionality stable
2. **Monitoring infrastructure** - Production concerns
3. **Load testing** - Performance validation

---

*This file should be updated as new tooling needs are identified during development.*