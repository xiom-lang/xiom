# XIOM Compiler — Production Roadmap

**Current:** v0.49.8 — **924/924 all tests** (673 compiler + 251 tooling), zero warnings
**Branch:** `feat/architect`
**Next:** Phase 9A — Pre-Release Infrastructure (registry, website, package distribution)

---

## 1. CURRENT STATE (2026-07-21 — v0.49.8, 924 tests)

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
| xiom | **4/10** | **28 process::exit**, only 3 tests |
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
| M3.1 xiom pipeline integration tests | ? 6 tests added | Done |
| M3.2 xiom::compile_with_diagnostics unit tests | 80% coverage target | 2d |
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

### 8B/M6 — Ecosystem Maturity (DELEGATED — separate agent)
| Item | Status |
|------|--------|
| xiom-redis fully implemented (hiredis bridge) | Delegated |
| Ecosystem test infrastructure (all 75 packages) | Delegated |
| Package registry backend (registry.xiom-lang.org) | Delegated |
| xiom pkg publish ? registry | Delegated |
| xiom pkg search working | Delegated |

### 8B/M7 — Stdlib Completion
| Item | Effort |
|------|--------|
| From/Into/TryFrom/TryInto traits | 2d |
| Deref/DerefMut + AsRef/AsMut | 1d |
| Cow<T>, PhantomData<T>, MaybeUninit<T> | 2d |
| Duration/Instant/SystemTime | ? Done |
| Path/PathBuf implementation | 2d |
| Iterator adapter parity (step_by, flat_map, etc.) | 2d |

### 8B/M8 — Polish & Ship (IN PROGRESS)
| Item | Effort | Status |
|------|--------|--------|
| Edition 2024 migration for all crates | 1d | ? Done |
| M9: `and`/`or`/`not` keywords | 0.5d | ? Done |
| M9: Compound assignment (`+=`, `-=`, `*=`, `/=`, `%=`) | 0.5d | ? Done |
| LSP: adopt lsp-types crate | 1d | Pending |
| MCP: audit 20 unsafe blocks | 1d | Pending |
| xiom-pkg: TLS support via ureq | 1d | Pending |
| xiom-doc: HTML output format | 1d | Pending |
| xiom-fmt: round-trip validation | 1d | Pending |

### 8B/M9 — Language Parity (IN PROGRESS)
Gap analysis completed: 11 missing features identified, 4 fixed so far.

| Gap | Status | Effort |
|-----|--------|--------|
| M9.1 `and`/`or`/`not` keywords | ? Done | 0.5d |
| M9.2 Compound assignment (`+=`, `-=`, `*=`, `/=`, `%=`) | ? Done | 0.5d |
| M9.3 Range syntax (`..` and `..=`) | Pending | 1d |
| M9.4 `if let` / `while let` expression | Pending | 2d |
| M9.5 `where` clauses on generics | Pending | 2d |
| M9.6 `impl Trait` return types | Pending | 3d |
| M9.7 Debug trait (stdlib) | Pending | 1d |
| M9.8 Labeled break/continue | Pending | 1d |
| M9.9 Tuple structs | Pending | 2d |
| M9.10 FromStr trait (stdlib) | Pending | 1d |
| M9.11 `defer` statement | Pending | 2d |

---

## 4. POST-10/10 — PHASE 9 — FIRST PUBLIC RELEASE

**Goal:** v0.50.0 or v1.0.0 — first stable public release with full ecosystem.

### 9A — Pre-Release Infrastructure (NOW — 1-2 weeks)

| Item | Status | Notes |
|------|--------|-------|
| **Package registry** (GitHub-based) | ?? Pending | `xiom pkg install <name>` downloads from GitHub Releases |
| **Website** (xiom-lang.org) | ?? Pending | Separate static repo, markdown?HTML via GH Actions |
| **Playground** (playground.xiom-lang.org) | ?? Pending | Separate repo, React + WASM worker |
| **Release automation** | ? Done | GitHub Actions builds all 9 tools for Win/Linux/macOS |
| **Installer verified** | ? Done | Windows install.bat, Linux/macOS install.sh working |
| **MCP configs** | ? Done | 12 IDE templates in release package |
| **Ecosystem packages** | ? Done | 72 packages in packages/ directory |

### 9B — Registry Architecture (GitHub-Based)

**No server needed.** Uses GitHub Releases API as the package registry:

```
registry.xiom-lang.org/
??? index.json          # Auto-generated manifest of all packages
?                        # { packages: [{ name:"xiom-vulkan", versions:["0.5.0","0.4.1"] }] }
?
Package storage: github.com/xiom-lang/packages/xiom-vulkan/releases/
??? v0.5.0.tar.gz
??? v0.4.1.tar.gz

xiom pkg install xiom-vulkan
  ? GET registry.xiom-lang.org/index.json        (discover latest version)
  ? GET github.com/xiom-lang/packages/xiom-vulkan/releases/download/v0.5.0/package.tar.gz
  ? extract to XIOM_HOME/packages/xiom-vulkan-0.5.0/
```

**Why GitHub-based:**
1. Zero server cost — GitHub provides free CDN
2. Automatic versioning — git tags = releases
3. No database — index.json regenerated from git
4. This is how Go modules + many Rust crates work
5. Later upgrade to dedicated registry when needed (Option C)

### 9C — Website (xiom-lang.org)

Separate static repo: `github.com/xiom-lang/xiom-lang.github.io`

```
xiom-lang.org/
??? index.html           # Landing page
??? docs/                # Generated from docs/language/*.md
??? install/             # Download page linking to GitHub Releases
??? playground ? redirect to playground.xiom-lang.org
??? packages/            # Package browser (generated from registry)
```

**Build pipeline:** GitHub Actions converts `docs/language/*.md` ? HTML via pandoc, deploys to GitHub Pages. No server needed.

### 9D — Final Pre-Release Checklist

| # | Item | Status |
|---|------|--------|
| 1 | 924 tests, zero warnings | ? |
| 2 | All 10 tools built + packaged | ? |
| 3 | Installer works on Windows + Linux + macOS | ? |
| 4 | Package registry index.json auto-generated | ?? |
| 5 | `xiom pkg install xiom-vulkan` works end-to-end | ?? |
| 6 | Website live at xiom-lang.org | ?? |
| 7 | Playground live at playground.xiom-lang.org | ?? |
| 8 | At least 5 ecosystem packages have passing tests | ?? |
| 9 | Documentation: language guide, stdlib reference, getting started | ?? |
| 10 | CI/CD green on all 3 platforms | ?? |

### 9E — Post-Release (after v1.0)

- Debugger Pro (commercial GUI, separate repo)
- Self-hosted compiler (XIOM compiled by XIOM)
- Dedicated package registry server (replace GitHub-based)

---

## 5. Release History

| Version | Date | Tests | Notes |
|---------|------|-------|-------|
| v0.49.8 | 2026-07-21 | 924 | 10/11 M9 gaps closed, ASCII installer, fuzz harnesses, HTML docs |
| v0.49.7 | 2026-07-21 | 910 | Phase 8B/M4-M9 complete, preflight audit |
| v0.49.5 | 2026-07-20 | 871 | Phase 7 complete, installer v2 |
| v0.48.9 | 2026-07-20 | 768 | Phase 5-6 complete |
