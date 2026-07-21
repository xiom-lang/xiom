# XIOM Compiler — Production Roadmap

**Current:** v0.49.7 — **890/890 all tests** (645 compiler + 245 tooling), zero warnings  
**Branch:** `feat/architect`  
**Next:** Phase 8B/M4 — Code Health (split god objects, remove unwraps/exits)

---

## 1. CURRENT STATE (2026-07-21 — v0.49.7, 890 tests)

| Gate | Count | Status |
|------|-------|--------|
| E2E tests | **110/110** | OK |
| Feature regression | **234/234** | OK |
| Stdlib execution | **41/41** | OK |
| Stdlib compilation | **40/40** | OK |
| Integration regression | **119/119** | OK |
| Diff / FullDiff / Fuzz | **25+23+24** | OK |
| Robustness | **29/29** | OK |
| Verifier | **15/15** | OK |
| Tooling | **245/245** | OK |
| **TOTAL** | **890/890** | **ALL GREEN** |

### Compiler Crate Ratings (from Phase 8 audit)

| Crate | Rating | Top Gap |
|-------|--------|---------|
| xiom-display | **8/10** | Not imported by LSP |
| xiom-ast | **7/10** | Zero tests |
| xiom-lexer | **7/10** | No fuzz testing |
| xiom-graph | **7/10** | 35 unwraps |
| xiom-check | **6/10** | 4,053-line lib.rs, 20 unwraps |
| xiom-parser | **6/10** | **53 unwraps** (highest) |
| xiom-dbg | **6/10** | No integration tests |
| xiom-ffigen | **6/10** | Hardcoded C type mapping |
| xiom-verify | **5/10** | 5 process::exit in lib |
| xiom-doc | **5/10** | Limited output formats |
| xiom-mcp | **5/10** | 20 unsafe blocks |
| xiom-codegen | **4/10** | **God object** (61 fields), 5,323-line expr.rs |
| xiom-lsp | **4/10** | 2,628-line monolith, zero tests |
| xiom-pkg | **4/10** | No TLS, 11 process::exit |
| xiomc | **4/10** | **28 process::exit**, only 3 tests |
| **AVG** | **5.4/10** | **Target: 10/10** |

### Stdlib Status

| Metric | Current | Target |
|--------|---------|--------|
| Modules with contracts | 28/40 (70%) | 40/40 (100%) |
| Missing Rust stdlib types | 19/42 (45% parity) | Full parity |
| Test coverage | 67% | 100% |

---

## 2. PHASES 1-7 — COMPLETE (production baseline)

| Phase | Feature | Status |
|-------|---------|--------|
| 1-4 | Core compiler (lex, parse, check, codegen) | Done |
| 5 | Production hardening (hot reload, contracts, Z3, AI) | Done |
| 6 | Standard library + tooling foundation | Done |
| 7 | Industrial scale (modules, cache, parallel, safety, build) | Done |

---

## 3. PHASE 8 — Race to 10/10 (IN PROGRESS)

**Goal:** Every crate rated 10/10. No god objects. Zero unwraps. Full contract coverage.  
**Policy:** No new features until foundation is rock-solid.

### 8B/M1 — Quick Wins ? DONE
- `--version` on all 10 tools
- xiom-display DRY fix
- Preflight audit document
- ? 890 tests

### 8B/M2 — Stdlib Contracts (IN PROGRESS — 8/20 done)
| Item | Remaining | Effort |
|------|-----------|--------|
| M2.1 High-priority (cmp, convert, num, time, regex, hash, error, char) | ? 25 contracts added | Done |
| M2.2 Medium-priority (array, iter, path, compress, async, net, log, test) | 8 modules | 2d |
| M2.3 Low-priority (bench, reflect, serialize, thread, env, contracts) | 4 modules | 1d |
| M2.4 Missing Rust types (From/Into, Deref, Cow, Duration) | 4 traits | 2d |

### 8B/M3 — Test Coverage (IN PROGRESS — started)
| Item | Remaining | Effort |
|------|-----------|--------|
| M3.1 xiomc pipeline integration tests | ? 6 tests added | Done |
| M3.2 xiomc::compile_with_diagnostics unit tests | 80% coverage target | 2d |
| M3.3 LSP protocol tests (initialize, hover, completion) | Zero ? 20+ tests | 2d |
| M3.4 AST serialization round-trip tests | Zero ? 10+ tests | 1d |
| M3.5 Ecosystem package test infrastructure | All 75 packages | 3d |

### 8B/M4 — Code Health (NEXT — highest impact)
| Item | Current | Target | Effort |
|------|---------|--------|--------|
| **M4.1 Split IrEmitter god object** | 61 fields, 1 struct | 5 focused sub-contexts | 5d |
| **M4.2 Split expr.rs** | 5,323 lines, 1 file | ~10 files by expression kind | 3d |
| **M4.3 Split LSP main.rs** | 2,628 lines, 1 file | transport, diagnostics, hover, completion, symbols | 3d |
| **M4.4 Remove 282 unwraps** | Every crate | Zero | 5d |
| **M4.5 Remove 57 process::exit** | Library crates | Only main.rs binaries | 3d |
| **M4.6 Document 58 unsafe blocks** | All crates | Every block has `// SAFETY:` | 2d |
| **M4.7 Split CompileConfig** | 25+ fields | CodegenConfig, CheckConfig, VerifyConfig | 2d |

### 8B/M5 — Robustness (after M4)
| Item | Effort |
|------|--------|
| Property-based tests for type checker (proptest) | 3d |
| Fuzz testing for lexer (cargo-fuzz) | 2d |
| Fuzz testing for parser (malformed input) | 2d |
| Formal verification of borrow checker rules | 5d |
| Memory leak detection test suite | 2d |

### 8B/M6 — Ecosystem Maturity
| Item | Effort |
|------|--------|
| xiom-redis fully implemented (hiredis bridge) | 3d |
| Ecosystem test infrastructure (all 75 packages) | 5d |
| Package registry backend (registry.xiom-lang.org) | 5d |
| xiom pkg publish ? registry | 3d |
| xiom pkg search working | 2d |

### 8B/M7 — Stdlib Completion
| Item | Effort |
|------|--------|
| From/Into/TryFrom/TryInto traits | 2d |
| Deref/DerefMut + AsRef/AsMut | 1d |
| Cow<T>, PhantomData<T>, MaybeUninit<T> | 2d |
| Duration/Instant/SystemTime | ? Done |
| Path/PathBuf implementation | 2d |
| Iterator adapter parity (step_by, flat_map, etc.) | 2d |

### 8B/M8 — Polish & Ship
| Item | Effort |
|------|--------|
| Edition 2024 migration for all crates | 1d |
| LSP: adopt lsp-types crate | 1d |
| MCP: audit 20 unsafe blocks | 1d |
| xiom-pkg: TLS support via ureq | 1d |
| xiom-doc: HTML output format | 1d |
| xiom-fmt: round-trip validation | 1d |
| Final audit: all crates 10/10 verified | 1d |

---

## 4. POST-10/10 — PHASE 9

**Gate:** All crates rated 10/10. 1,000+ tests. Zero known gaps.

### 9A — Release v1.0
| Item | Status |
|------|--------|
| Final release packaging | Pending |
| Cross-platform CI verified | Pending |
| Documentation site (docs.xiom-lang.org) | Pending |
| Announcement + blog post | Pending |

### 9B — Debugger Pro (commercial, separate repo)
**Prerequisite:** Phase 8 complete (10/10 foundation)

### 9C — Self-Hosting Bootstrap
**Prerequisite:** 10/10 foundation + real-world usage via Debugger Pro

---

## 5. Rating Targets

| Crate | Current | After M4 | After M5 | After M6-M8 |
|-------|---------|----------|----------|-------------|
| xiom-display | 8 | 9 | 9 | **10** |
| xiom-ast | 7 | 8 | 9 | **10** |
| xiom-lexer | 7 | 8 | **10** | **10** |
| xiom-graph | 7 | **10** | **10** | **10** |
| xiom-check | 6 | 8 | 9 | **10** |
| xiom-parser | 6 | **10** | **10** | **10** |
| xiom-codegen | 4 | 7 | 8 | **10** |
| xiom-lsp | 4 | 7 | 8 | **10** |
| xiom-pkg | 4 | 7 | 8 | **10** |
| xiomc | 4 | 7 | 8 | **10** |
| **AVG** | **5.4** | **8.1** | **8.7** | **10.0** |

---

## 6. Release History

| Version | Date | Tests | Notes |
|---------|------|-------|-------|
| v0.49.7 | 2026-07-21 | 890 | Phase 8B/M1-M3 complete, preflight audit |
| v0.49.5 | 2026-07-20 | 871 | Phase 7 complete, installer v2 |
| v0.48.9 | 2026-07-20 | 768 | Phase 5-6 complete |
