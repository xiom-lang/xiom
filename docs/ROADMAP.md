# XIOM Compiler — Production Roadmap

**Current:** v0.49.7 — **881/881 all tests** (636 compiler + 245 tooling), zero warnings  
**Branch:** `feat/architect`  
**Next:** Debugger JSON API hardening

---

## 1. CURRENT STATE (2026-07-21 — v0.49.7, 881 tests)

| Gate | Count | Status |
|------|-------|--------|
| E2E tests | **110/110** | OK |
| Feature regression | **225/225** | OK |
| Stdlib execution | **41/41** | OK |
| Stdlib compilation | **40/40** | OK |
| Integration regression | **119/119** | OK |
| Diff / FullDiff / Fuzz | **25+23+24** | OK |
| Robustness | **29/29** | OK |
| Verifier | **15/15** | OK |
| Tooling | **245/245** | OK |
| **TOTAL** | **881/881** | **ALL GREEN** |

### Key Crates (16 total)

| Crate | Rating | Notes |
|-------|--------|-------|
| xiom-graph | 8.0 | Dependency graph, xiom.toml parser, topo sort |
| xiom-codegen | 4.9 | God object (55 fields), raw string IR |
| xiom-check | 6.3 | Type alias resolution, newtype auto-conversion |
| xiom-lsp | 3.1 | 2800+ line monolith, needs splitting |
| xiom-display | 8.0 | Shared crate — type_to_string, op_to_str |
| xiom-pkg | 3.4 | Local ecosystem resolver added (7F) |

### All P0/P1 Gaps: CLOSED

All 11 P0 gaps and 11 P1 gaps resolved. Zero known compiler limitations.
All 10 legacy bugs (BUG-001 through BUG-010) resolved.

---

## 2. PHASE 5 — Production Hardening (COMPLETE)

| Phase | Feature | Status |
|-------|---------|--------|
| 5a | Core MCP integration | Done |
| 5b | Lexer + parser hardening | Done |
| 5c.29-30 | 6 production bugs + 5 stdlib gaps | Done |
| 5d | Standard library coverage (41+ modules) | Done |
| 5e.1 | C struct field access via pointer | Done |
| 5e.2 | C callback lowering + fn-ptr cast | Done |
| 5e.3 | Walk-up project root detection | Done |
| 5e.5 | Hot reload: thunks, state migration, DLL lifecycle | Done |
| 5e.6 | Contract verification + Z3 | Done |
| 5f | Z3 verification stage 2: body encoding | Done |
| 5f.3 | AI/MCP hardening: structured JSON, LSP hover, batch mode, Z3 injection | Done |

---

## 3. PHASE 7 — Industrial Scale (COMPLETE)

| Phase | Feature | Tests Added | Key Deliverable |
|-------|---------|-------------|-----------------|
| **7A** | Module system + dependency graph | +11 | xiom-graph crate, xiom.toml manifest, topo sort |
| **7B** | Industrial incremental compilation | +11 | CacheDb (Arc<RwLock>), SHA-256 hashing, transitive invalidation |
| **7C** | Parallel compilation | +7 | rayon-based parallel lex+parse, --parallel/--jobs |
| **7D** | Hot reload safety at scale | +8 | Process-wide pointer table, state versioning, contract-on-reload |
| **7E** | Runtime safety guarantees | +11 | Sanitizer flags, stack protector, runtime contracts |
| **7F** | Build system & IDE integration | +7 | --graph viz, build daemon, batch AI, Z3 counterexamples |
| **Fix** | Newtype auto-conversion + Float32 narrowing | +16 | Type alias resolution, tuple coercion |
| **Fix** | Enum derive deep Eq/Hash/Ord/Display | +10 | Vec.eq/Option.eq delegation, content-based Hash |

---

## 4. PHASE 8 — Tooling & Ecosystem (IN PROGRESS)

### 8A — Release Infrastructure (DONE)

| Item | Status |
|------|--------|
| install.bat (Windows) | Done — ASCII art, AI config, MCP setup, PATH registration |
| install.sh (Linux/macOS) | Done — cross-platform, shell RC integration |
| MCP templates (12 IDEs) | Done — Kilo, Cursor, Claude, Windsurf, Continue, Cline, Copilot, Aider, Codex, Antigravity, Trae |
| package.ps1 | Done — bundles MCP, ecosystem skipped (xiom pkg install) |
| GitHub Actions CI/CD | Done — 3-platform build matrix, test suite, release artifacts |
| Ecosystem package resolver | Done — xiom pkg install from local ecosystem/ fallback |

### 8B — Debugger Foundation (IN PROGRESS)

| Item | Effort | Status |
|------|--------|--------|
| xiom-dbg --json API engine | 3-5 days | Next |
| Type-aware variable display | 2 days | Planned |
| Memory inspection + registers | 2 days | Planned |
| DWARF source-level debugging | 3 days | Planned |

### 8C — Debugger Pro (separate repo)

See `docs/DEBUGGER_PRO_ROADMAP.md` for full plan.

| Feature | License |
|---------|---------|
| egui-based GUI debugger | Commercial / source-available |
| Breakpoint manager, watch panel | Pro |
| Theme engine, project launcher | Pro |
| Hot-reload debug integration | Pro |

### 8D — Package Registry (P3)

| Item | Status |
|------|--------|
| Remote registry (registry.xiom-lang.org) | P3 — needs hosting |
| xiom pkg publish | Framework exists, needs registry backend |
| xiom pkg search | Framework exists, needs registry data |

### 8E — Self-Hosting Bootstrap (P4)

| Item | Status |
|------|--------|
| Compile xiomc with xiomc | Requires Phase 7D complete |
| Full bootstrap chain | Long-term research project |

---

## 5. P2/P3 Deferred Items

| Item | Priority | Effort |
|------|----------|--------|
| Platform debug API (WinDbg/lldb) | P3 | 3-5 days |
| Remote dependency registry backend | P3 | 5-7 days |
| Digital code signing | P3 | 2-3 days |
| GitHub Actions release automation | P2 | Done |
| Derive macros for enum heap fields | P2 | Done |
| Z3 counterexamples in AI prompts | P2 | Done |
| Batch AI mode | P2 | Done |

---

## 6. Release History

| Version | Date | Tests | Notes |
|---------|------|-------|-------|
| v0.49.7 | 2026-07-21 | 881 | Debugger API, ecosystem packages, CI/CD |
| v0.49.5 | 2026-07-20 | 871 | Phase 7 complete, installer v2 |
| v0.48.9 | 2026-07-20 | 768 | Phase 5-6 complete |
| v0.46.0 | 2026-07-19 | — | First release with zero known compiler gaps |
