# XIOM Compiler — Production Roadmap

**Current:** v0.49.9 — **934/934 all tests** (680 compiler + 254 tooling)
**Branch:** `feat/architect`
**Next:** Phase 9A — Pre-Release Infrastructure (website, playground deployment, package distribution)

---

## 1. CURRENT STATE (2026-07-24 — v0.49.9, 934 tests)

| Gate | Count | Status |
|------|-------|--------|
| E2E tests | **111/111** | OK |
| Feature regression | **268/268** | OK |
| Stdlib execution | **41/41** | OK |
| Stdlib compilation | **40/40** | OK |
| Integration regression | **119/119** | OK |
| Diff / FullDiff / Fuzz | **25+23+24** | OK |
| Robustness | **29/29** | OK |
| Verifier | **15/15** | OK |
| Tooling | **254/254** | OK |
| **TOTAL** | **934/934** | **ALL GREEN** |

### Compiler Crate Ratings (post Phase 8 hardening)

| Crate | Rating | Status |
|-------|--------|--------|
| xiom-display | **8/10** | Clean type rendering |
| xiom-ast | **7/10** | 11/11 M9 language features |
| xiom-lexer | **7/10** | Fuzz harness (500+23 edge cases) |
| xiom-graph | **7/10** | Dependency graph, topo sort |
| xiom-check | **7/10** | impl Trait support, compatibility fixes |
| xiom-parser | **7/10** | Fuzz harness (200+37 edge cases) |
| xiom-codegen | **7/10** | God object decomposed (5 sub-contexts) |
| xiom-lsp | **7/10** | Monolith split (10 modules) |
| xiom-dbg | **6/10** | SAFETY-documented |
| xiom-ffigen | **6/10** | 18 tests |
| xiom-verify | **5/10** | 15 tests, Z3 integration |
| xiom-doc | **5/10** | Limited output formats |
| xiom-mcp | **5/10** | 17 tests |
| xiom-pkg | **5/10** | TLS (ureq), registry integration |
| xiom | **5/10** | CLI, compile_with_diagnostics |
| **AVG** | **6.3/10** | **Up from 5.4** |

### M9 Language Parity: 11/11 CLOSED

| Feature | Status |
|---------|--------|
| `and`/`or`/`not` keywords | Done |
| Compound assignment | Done |
| Range syntax (`..`/`..=`) | Done |
| `if let` / `while let` | Done |
| `where` clauses | Done |
| `impl Trait` return types | Done |
| Debug trait (stdlib) | Done |
| Labeled break/continue | Done |
| Tuple structs | Done |
| FromStr trait (stdlib) | Done |
| Fuzz harnesses | Done |

---

## 2. PHASES 1-7 — COMPLETE (production baseline)

| Phase | Feature | Status |
|-------|---------|--------|
| 1-4 | Core compiler (lex, parse, check, codegen) | Done |
| 5 | Production hardening (hot reload, contracts, Z3, AI) | Done |
| 6 | Standard library + tooling foundation | Done |
| 7 | Industrial scale (modules, cache, parallel, safety, build) | Done |

---

## 3. PHASE 8 — Production Hardening (DONE — 2026-07-24)

**Goal:** Every critical crate rated 7/10+. No god objects. M9 gaps closed. Registry live.

### 8B/M1 — Quick Wins DONE
- `--version` on all 10 tools
- xiom-display DRY fix
- Preflight audit document
- 890 tests

### 8B/M2 — Stdlib Contracts (IN PROGRESS — 8/20 done)
| Item | Remaining | Effort |
|------|-----------|--------|
| M2.1 High-priority | 25 contracts added | Done |
| M2.2 Medium-priority (array, iter, path, compress, async, net, log, test) | 8 modules | 2d |
| M2.3 Low-priority (bench, reflect, serialize, thread, env, contracts) | 4 modules | 1d |
| M2.4 Missing Rust types (From/Into, Deref, Cow, Duration) | 4 traits | 2d |

### 8B/M3 — Test Coverage (IN PROGRESS)
| Item | Remaining | Effort |
|------|-----------|--------|
| M3.1 xiom pipeline integration tests | 6 tests added | Done |
| M3.2 xiom::compile_with_diagnostics unit tests | 80% coverage target | 2d |
| M3.3 LSP protocol tests (11 tests exist) | Add rename/codeAction tests | 1d |
| M3.4 AST serialization round-trip tests | 10+ tests | 1d |
| M3.5 Ecosystem package test infrastructure | All 75 packages | 3d |

### 8B/M4 — Code Health
| Item | Status | Details |
|------|--------|---------|
| M4.1 Split IrEmitter god object | **DONE** | 86 fields ? 5 sub-contexts (CodegenConfig, TypeContext, FunctionContext, MonoContext, LocalContext) |
| M4.2 Split expr.rs | Pending | 5,323 lines ? ~10 files by expression kind |
| M4.3 Split LSP main.rs | **DONE** | 2,850 lines ? 10 modules (backend, transport, uri, diagnostics, resolver, symbols, semantic_tokens, text_edit, ai, handlers) |
| M4.4 Remove 282 unwraps | Pending | Every crate |
| M4.5 Remove 57 process::exit | Pending | Library crates |
| M4.6 Document unsafe blocks | **DONE** | 2 blocks (not 58 — original count included XIOM test strings). Both have `// SAFETY:` |
| M4.7 Split CompileConfig | **DONE** | CodegenConfig extracted as part of M4.1 sub-contexts |

### 8B/M5 — Robustness (deferred)
| Item | Effort |
|------|--------|
| Property-based tests for type checker (proptest) | 3d |
| Fuzz testing for lexer (cargo-fuzz) | Done |
| Fuzz testing for parser (malformed input) | Done |
| Formal verification of borrow checker rules | 5d |
| Memory leak detection test suite | 2d |

### 8B/M6 — Ecosystem Maturity
| Item | Status |
|------|--------|
| Package registry backend | **DONE** — Node.js/Express API, Docker/Portainer deploy, health/search/publish/sync endpoints |
| xiom-redis fully implemented | Delegated |
| Ecosystem test infrastructure (all 75 packages) | Delegated |
| xiom pkg publish ? registry | Pending |
| xiom pkg search working | Done (local) |

### 8B/M7 — Stdlib Completion
| Item | Effort |
|------|--------|
| From/Into/TryFrom/TryInto traits | 2d |
| Deref/DerefMut + AsRef/AsMut | 1d |
| Cow<T>, PhantomData<T>, MaybeUninit<T> | 2d |
| Duration/Instant/SystemTime | Done |
| Path/PathBuf implementation | 2d |
| Iterator adapter parity (step_by, flat_map, etc.) | 2d |

### 8B/M8 — Polish & Ship
| Item | Status |
|------|--------|
| Edition 2024 migration for all crates | Done |
| All 11 M9 language gaps closed | **DONE** |
| Playground: 372 lessons, fresh index.json, 0 stale refs | **DONE** |
| LSP: adopt lsp-types crate | Pending |
| xiom-pkg: TLS support via ureq | Done |
| xiom-doc: HTML output format | Pending |
| xiom-fmt: round-trip validation | Pending |

### 8B/M9 — Language Parity (11/11 DONE)
All 11 language gaps closed. `impl Trait` was the final one (M9.6), completed 2026-07-24 with parser, AST, checker, and codegen support. 3 parser tests + 1 E2E test verify correct behavior.

---

## 4. PHASE 9 — FIRST PUBLIC RELEASE (v0.50.0)

**Goal:** v0.50.0 — first stable public release with full ecosystem.

### 9A — Pre-Release Infrastructure (1-2 weeks)

| Item | Status | Notes |
|------|--------|-------|
| **Package registry** | DONE | Node.js/Express, Docker deployable, `docker-compose up -d` |
| **Website** (xiom-lang.org) | Pending | Static site, GH Pages |
| **Playground** (playground.xiom-lang.org) | Pending | 372 lessons, 3-mode SPA |
| **Release automation** | Done | GitHub Actions builds all tools for Win/Linux/macOS |
| **Installer verified** | Done | Windows install.bat, Linux/macOS install.sh |
| **MCP configs** | Done | 12 IDE templates in release package |
| **Ecosystem packages** | Done | 72 packages in packages/ directory |

### 9B — Registry Architecture

Two-tier: Node.js registry server (`registry.xiom-lang.org`) for metadata + GitHub Releases for tarballs.

```
registry.xiom-lang.org/
  GET  /                        ? health check
  GET  /index.json              ? full package catalog (70+ packages)
  GET  /packages/:name/:version/download ? tarball (hosted on registry server)
  POST /publish                 ? publish new package (authenticated)
  GET  /search?q=<query>        ? search packages
```

Deploy with: `cd registry && docker-compose up -d`

### 9C — Final Pre-Release Checklist

| # | Item | Status |
|---|------|--------|
| 1 | 934 tests, zero failures | Done |
| 2 | All 10 tools built + packaged | Done |
| 3 | Installer works on Windows + Linux + macOS | Done |
| 4 | Package registry live at registry.xiom-lang.org | Pending (deploy to Contabo VPS) |
| 5 | `xiom pkg install xiom-vulkan` works end-to-end | Pending |
| 6 | Website live at xiom-lang.org | Pending |
| 7 | Playground live at playground.xiom-lang.org | Pending |
| 8 | 11/11 M9 language gaps closed | Done |
| 9 | Documentation: language guide, stdlib reference, getting started | Pending |
| 10 | CI/CD green on all 3 platforms | Pending |

### 9D — Post-Release (after v1.0)

- Dedicated package registry server with storage backend
- Ecosystem package publishing workflow
- Community contribution guidelines

---

## 5. Release History

| Version | Date | Tests | Notes |
|---------|------|-------|-------|
| v0.49.9 | 2026-07-24 | **934** | M4.1 (IrEmitter split), M4.3 (LSP split), M4.6 (unsafe docs), playground fixes (372 lessons), M9.6 (impl Trait), Node.js registry backend. 11/11 M9 closed. |
| v0.49.8 | 2026-07-21 | 924 | 10/11 M9 gaps closed, ASCII installer, fuzz harnesses, HTML docs |
| v0.49.7 | 2026-07-21 | 910 | Phase 8B/M4-M9 complete, preflight audit |
| v0.49.5 | 2026-07-20 | 871 | Phase 7 complete, installer v2 |
| v0.48.9 | 2026-07-20 | 768 | Phase 5-6 complete |
