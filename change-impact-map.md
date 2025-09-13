# ProtoChain Repository Restructuring - Change Impact Map

## Summary of Analysis Results

### Namespace Confirmation ✅
- **Actual namespace used**: `protochain.solana.x.v1` (NOT protosol)
- **Package name**: `protochain-solana-api`
- **Go package paths**: `github.com/BRBussy/protochain/lib/go/protochain/solana/`

### Critical Dependencies Found

## 1. Current Structure → Target Structure Mapping

### File/Directory Movements
| Current Location | Target Location | Change Type |
|------------------|-----------------|-------------|
| `./api/` (entire directory) | `./app/solana/cmd/api/` | **DIRECTORY MOVE** |
| `./api/Cargo.toml` | `./app/solana/cmd/api/Cargo.toml` | File move + package name update |
| `./api/src/` | `./app/solana/cmd/api/src/` | Directory move |
| `./api/README.md` | `./app/solana/cmd/api/README.md` | File move + path updates |

### Configuration Files Requiring Updates
| File | Current Reference | New Reference | Update Type |
|------|------------------|---------------|-------------|
| `./Cargo.toml` | `members = ["api"]` | `members = ["app/solana/cmd/api"]` | **WORKSPACE MEMBER** |
| `./app/solana/cmd/api/Cargo.toml` | `name = "protochain-solana-api"` | `name = "protochain-solana-api"` | **KEEP SAME** |
| `./scripts/tests/start-backend.sh` | `cargo run -p protochain-solana-api` | `cargo run -p protochain-solana-api` | **NO CHANGE** |
| `./scripts/tests/stop-backend.sh` | `pgrep -f "protochain-solana-api"` | `pgrep -f "protochain-solana-api"` | **NO CHANGE** |

### Script Dependencies
| Script | Line | Current Reference | New Reference | Critical? |
|--------|------|------------------|---------------|-----------|
| `scripts/tests/start-backend.sh` | 39 | `grep -q "api" Cargo.toml` | `grep -q "app/solana/cmd/api" Cargo.toml` | **YES** |
| `scripts/tests/start-backend.sh` | 51 | `cargo run -p protochain-solana-api` | `cargo run -p protochain-solana-api` | NO |
| `scripts/tests/stop-backend.sh` | 25,27,38 | `pgrep -f "protochain-solana-api"` | `pgrep -f "protochain-solana-api"` | NO |

### Test Configuration Dependencies
| File | Line | Current Reference | New Reference | Critical? |
|------|------|------------------|---------------|-----------|
| `tests/go/config/config.go` | 32 | `// Find api-test root by walking up` | Update comment | NO |
| `tests/go/config/config.go` | 133 | `"api/Cargo.toml"` | `"app/solana/cmd/api/Cargo.toml"` | **YES** |

### IDE Configuration Dependencies
| File | Line | Current Reference | New Reference | Critical? |
|------|------|------------------|---------------|-----------|
| `.vscode/launch.json` | - | `"program": "${workspaceFolder}/target/debug/protochain-solana-api"` | **NO CHANGE** | NO |
| `.vscode/launch.json` | - | `"preLaunchTask": "cargo: build --bin protochain-solana-api"` | **NO CHANGE** | NO |

## 2. Implementation Strategy

### Safe Approach - Package Name Preservation
- **KEEP**: Package name `protochain-solana-api` unchanged
- **KEEP**: Binary name `protochain-solana-api` unchanged
- **CHANGE ONLY**: Directory paths and workspace member references

### Critical Update Sequence
1. **Update Cargo.toml workspace member**: `"api"` → `"app/solana/cmd/api"`
2. **Move directory**: `./api` → `./app/solana/cmd/api`
3. **Update script workspace check**: `start-backend.sh` line 39
4. **Update test config path**: `config.go` line 133

### Low-Risk Items (Package Name Dependencies)
These reference the package name, which we're keeping the same:
- All `cargo run -p protochain-solana-api` commands ✅
- All `pgrep -f "protochain-solana-api"` commands ✅
- Binary path references in target/debug/ ✅
- VS Code launch configurations ✅

## 3. Template App Structure to Create

### New Directories to Create
```
./app/
├── solana/                    # [MOVE api content here]
│   └── cmd/
│       └── api/              # [./api content moved here]
└── template/                 # [NEW empty template]
    └── cmd/
        └── some-executable/  # [NEW Go template]
            └── main.go       # [NEW simple Go main]
```

## 4. Risk Assessment

### HIGH RISK ⚠️
1. **Workspace member path change** in Cargo.toml
2. **Test configuration path** in tests/go/config/config.go
3. **Script workspace detection** in start-backend.sh

### MEDIUM RISK ⚡
1. **Directory move operation** (ensure atomic move)
2. **Path references in README files**

### LOW RISK ✅
1. Package name references (keeping same name)
2. Binary execution (cargo run still works)
3. IDE configurations (package name unchanged)

## 5. Validation Strategy

### Pre-Move Validation ✅ COMPLETED
- [x] All tests passing
- [x] Current package name documented
- [x] Backup files created

### Post-Move Validation Required
- [ ] `cargo check --workspace` succeeds
- [ ] `cargo build --workspace` succeeds
- [ ] Start backend script works
- [ ] Integration tests pass
- [ ] Stop backend script works

## 6. Recovery Plan

### If Issues Occur
1. **Quick Recovery**: Use `.backup-before-restructure/` files
2. **Workspace Issues**: Restore Cargo.toml from backup
3. **Test Issues**: Restore config.go from backup
4. **Script Issues**: Restore start-backend.sh from backup

## 7. Next Steps Summary

**CRITICAL CHANGES REQUIRED (3 files):**
1. `Cargo.toml`: Update workspace member path
2. `tests/go/config/config.go`: Update Cargo.toml path reference
3. `scripts/tests/start-backend.sh`: Update workspace validation check

**MINIMAL IMPACT**: Package name preservation means most references continue to work unchanged.