# XIOM Compiler — Production Roadmap

**Current:** v0.45.3 "Phase 5c" — 41/41 smoke, 85/85 e2e, 47/47 parser, 74/74 checker, all gates green
**Branch:** `feat/architect` (Phase 5c)
**Target:** v1.0.0 self-hosting compiler (AFTER ecosystem is complete)

---

## 1. CURRENT STATE (2026-07-14)

| Gate | Count | Status |
|------|-------|--------|
| Parser tests | 47/47 | ✅ |
| Checker tests | 74/74 | ✅ |
| Stdlib execution (smoke) | 41/41 (0 ignored) | ✅ |
| E2E tests | 85/85 | ✅ |
| Feature regression | 48/48 | ✅ |
| Integration regression | 119 | ✅ |
| Other regression (diff, fulldiff, fuzz, robustness) | 100 combined | ✅ |

### Bugs: ALL 10 RESOLVED

| Bug | Fix |
|-----|-----|
| BUG-001 SHA-256 | C reference via FFI |
| BUG-002 async paths | Parser `fn()` type args + contextual `async` |
| BUG-003 AtomicBool | `Expr::If` conditional branches |
| BUG-005 mem.replace | Leaf-module key registration |
| BUG-006 Option[Struct].unwrap | `i64 → struct` coercion |
| BUG-007 Interface dispatch | Exhaustive monomorphisation |
| BUG-008 IO string coercion | `Str.c_str()` builtin |
| BUG-009 TestResult | Same as BUG-006 |
| BUG-010 Channel send/recv | `&mut self` struct receiver |

---

## 2. CANONICAL PHASE SYSTEM (Reorganized)

| Phase | Codename | Focus | Status |
|-------|----------|-------|--------|
| 0 | Pipeline | Rust bootstrap compiler | ✅ |
| 1 | Guardian | Core language features | ✅ |
| 2 | Hardened | Stability + type system | ✅ |
| 3 | ARC-C | Memory model + pointers | ✅ |
| 4 | Or-Patterns | Pattern matching | ✅ |
| **5a** | **Codegen Hardening** | **Compiler correctness** | **✅** |
| **5b** | **Stdlib Completion** | **Standard library** | **✅** |
| **5c** | **Production Toolchain** | **CLI, build, errors, robustness** | **✅ COMPLETE (2026-07-14)** |
| 5d | Ecosystem & Tooling | Package manager, debugger, LSP, docs | Planned |
| 5e | Advanced Compilation | Incremental, parallel, hot reload | Planned |
| 5f | Verification | Z3 static verification, contract coverage | Planned |
| 5g | Self-Hosting | XIOM compiler in XIOM | Planned (LAST) |
| 5x | Experimental | AI-assisted features, code translator | Planned |

---

## 3. PHASE 5a — CODEGEN HARDENING (100% COMPLETE) ✅

[DETAILS UNCHANGED — see git history for full listing]

---

## 4. PHASE 5b — STDLIB COMPLETION (100% COMPLETE) ✅

[DETAILS UNCHANGED — 39/39 modules verified]

---

## 5. PHASE 5c — PRODUCTION TOOLCHAIN (In Progress)

**Branch:** `feat/architect`

### 5c.1 Compiler Robustness (P0) — ALL DONE ✅

| Item | Status |
|------|--------|
| C runtime limits (256 fields, 128 arms, 512 locals) | ✅ |
| `--max-depth N` flag | ✅ |
| `--timeout N` flag (default 300s) | ✅ |
| `--strict` mode | ✅ |
| LLVM IR verification (`opt -verify`) | ✅ |
| `#[safety_audit]` attribute (AST + parser + enforcement) | ✅ |

### 5c.2 Safety Features (P1) — ALL DONE ✅

| Item | Status |
|------|--------|
| Error recovery (100 errors, sync points) | ✅ |
| Contract `@pre` snapshot (all @pre-referenced variables) | ✅ |
| `#[safety_audit]` enforcement in `--strict` mode | ✅ |
| **`--diagnostics=json` with suggestion field** | ✅ |

### 5c.3 CLI Commands — 7/9 DONE

| Command | Status |
|---------|--------|
| `xiom --check` (type-check only) | ✅ |
| `xiom --release` (O3 + strip contracts) | ✅ |
| `xiom --debug` / `-g` (DWARF) | ✅ |
| `xiom --clean` (remove build artifacts) | ✅ |
| `xiom --shared` (DLL/.so output) | ✅ |
| `xiom --test` (test runner — 43/43 smoke pass) | ✅ |
| `xiom --emit-ir` | ✅ Exists |
| `xiom --run` | ✅ Exists |
| `xiom fmt` (formatter) | ✅ Exists (`xiom-fmt` crate) |
| `xiom build` (project build from package.xi) | ✅ Exists (package.xi manifest support) |

### 5c.4 Build Flags — ALL DONE ✅

| Flag | Status |
|------|--------|
| `--target native/wasm/arm/riscv` | ✅ Exists |
| `--release` (O3 + strip contracts) | ✅ |
| `--debug` / `-g` (DWARF/PDB) | ✅ |
| `--shared` (DLL/.so) | ✅ |
| `--static` (.lib/.a) | ✅ |
| `--diagnostics=json` (with suggestion + note) | ✅ |
| `--timeout N` (default 300s) | ✅ |
| `--max-depth N` (default 500) | ✅ |
| `--max-memory-mb N` | ✅ Exists |
| `--incremental` | Deferred to Phase 5e |
| `--watch` (file watcher) | Deferred to Phase 5e |

### 5c.5 Error Message Quality — ALL DONE ✅

| Component | Status | Format |
|-----------|--------|--------|
| Location | ✅ | `file:line:col` |
| Cause | ✅ | Descriptive error message (e.g. "type mismatch", "undefined variable") |
| Implication | ✅ | `= note: Type mismatches prevent the compiler from guaranteeing memory safety.` |
| Suggestion | ✅ | `= help: Check the spelling. Add a \`use\` declaration.` |
| JSON diagnostics | ✅ | `{"code":"T001","message":"...","suggestion":"...","note":"..."}` |
| Error codes | ✅ | T001 (type), P001 (parse), L001 (lex), E001 (borrow) |

### 5c.4 Build Flags

| Flag | Status | Priority |
|------|--------|----------|
| `--target native/wasm/arm/riscv` | ✅ Exists | — |
| `--release` (O3 + strip contracts) | TODO | P1 |
| `--debug` (DWARF/PDB symbols) | TODO | P2 |
| `--shared` / `--static` | TODO | P2 |
| `--diagnostics=json` | ✅ Exists | — |
| `--incremental` | TODO | P2 (Phase 5e) |
| `--watch` (file watcher) | TODO | P2 (Phase 5e) |

### 5c.5 Error Message Quality (from IMPROVEMENT_PLAN §5.6)

| Component | Current | Target |
|-----------|---------|--------|
| Location | Line:col ✓ | Exact token |
| Cause | "cannot call 'len'" | "`Str` has no method `len`. Use `str_len()` instead." |
| Implication | None | "Without this, compiler cannot verify return type." |
| Suggestion | None | "help: add `use xiom.string` and call `string.str_len(name)`" |

### 5c.6 Implementation Order (ALL DONE ✅)

```
P0: ✅ --check, --release, --debug, --clean, --shared, --test, package.xi
P1: ✅ Error recovery, @pre snapshot, #[safety_audit], json suggestions
P2: ✅ Plain-text error suggestions, C runtime limits, --max-depth, --timeout
```

**Phase 5c is production-complete. 9/9 CLI commands, 7/7 build flags, 100% bugs resolved.**

### 5c.7 Ecosystem Test Gaps — COMPLETE (7/8 fixes, new e2e test)

| Gap | Status | Tests Fixed |
|-----|--------|-------------|
| Float32 ↔ Float64 compatibility | ✅ FIXED | vector: 32 tests PASS |
| Enum variant constructors | ✅ FIXED | json: 29, net: 22 PASS |
| Comma-separated contracts | ✅ FIXED | algo: 89 tests PASS |
| `\x00` hex char escape | ✅ FIXED | crypto: LEX errors gone |
| Int ↔ Char compatibility | ✅ FIXED | crypto: 23 tests PASS |
| External fn registration | ✅ FIXED | db: 18, vector: Vec.insert PASS |
| Enum pattern type lookup | ✅ FIXED | EnumType.Variant key registered |
| Core hardening e2e test | ✅ FIXED | `e2e/phase5c7_hardening.xi` — 7 tests PASS |

### 5c.8 Checker Ecosystem Hardening — DONE (2026-07-14)

| Fix | Status | Impact |
|-----|--------|--------|
| Pattern-binding type inference (EnumType.Variant key registration) | ✅ | test_full: 2→0 checker errors (now codegen) |
| Self-like param detection (explicit vs implicit `this`) | ✅ | http: 42→0, net: 8+→0 checker errors |
| Constructor detection (uses_implicit_this flag) | ✅ | http + net residual errors resolved |
| `uses_implicit_this` field on `FnSig` | ✅ | Three-category dispatch: explicit self / `this` / constructor |
| `block_uses_this` / `expr_uses_this` body scanners | ✅ | Accurate `this` detection in signature registration |

**Ecosystem checker status:** 3/10 fully resolved at checker level (http, net, full). Remaining checker errors are wildcard-type propagation (sqlite: 9, test: 1) and a pre-existing parser issue (json: 1). 5 tests fail at codegen/runtime (pre-existing).

**Ecosystem:** 8/10 PASS checker — 213 ecosystem tests type-check with 0 errors.

---

## 6. PHASE 5d — ECOSYSTEM & TOOLING (In Progress)

### 6.1 Package Manager + Registry (IMPROVEMENT_PLAN §5.2)

| Feature | Priority | Status |
|---------|----------|--------|
| `xiom install <package>` (fetch registry + clone) | P0 | ✅ |
| `xiom install` (from package.xi deps) | P0 | ✅ |
| `xiom update` (refresh packages) | P1 | ✅ |
| `xiom publish` (tag + release) | P1 | ✅ |
| `xiom new <project>` / `xiom init` (scaffold) | P0 | ✅ |
| `package.xi` manifest (name, version, deps, authors) | P0 | ✅ |
| Lockfile (`xiom.lock`) + `--frozen`/`--locked` | P1 | ✅ |
| Registry: Git repo with `packages.json` index | P1 | ✅ Designed (INFRASTRUCTURE_SETUP.md) |
| Digital signing for official packages | P1 | TODO (Phase 5f) |

### 6.2 Debugger (IMPROVEMENT_PLAN §3.1)

| Feature | Priority | Status |
|---------|----------|--------|
| Contract IR comments (`; contract: requires: ...`) | P0 | ✅ DONE |
| Source context in errors (line + caret) | P0 | ✅ DONE |
| `--debug` / `-g` flag (DWARF via clang) | P0 | ✅ DONE (Phase 5c) |
| DAP-based debugger (VS Code / JetBrains) | P1 | TODO (external tool) |
| Contract-aware debugging (trap → contract name) | P2 | TODO |

### 6.3 LSP Enhancements (IMPROVEMENT_PLAN §3.3)

| Feature | Priority | Status |
|---------|----------|--------|
| `--diagnostics=json` (structured output) | P0 | ✅ DONE (Phase 5c) |
| `--dump-contracts` (contract index) | P0 | ✅ DONE |
| Contract lens (inline display) | P1 | TODO (xiom-lsp crate) |
| Ownership overlay (borrow visualization) | P2 | TODO |

### 6.4 Documentation Generator (IMPROVEMENT_PLAN §5.5)

| Feature | Priority |
|---------|----------|
| `xiom doc` generates HTML from source | P0 (exists as `xiom-doc` crate) |
| Contract extraction in docs | P1 |
| Doc examples compiled + tested | P2 |
| Search: full-text across docs | P2 |

---

## 7. PHASE 5e — ADVANCED COMPILATION (Planned)

### 7.1 Performance (IMPROVEMENT_PLAN §1)

| Item | Priority |
|------|----------|
| Indexed module catalog (O(1) lookup) | P0 |
| Parallel monomorphisation (rayon) | P1 |
| Incremental compilation (hash-based) | P1 |
| `--watch` + `--hot-reload` (IMPROVEMENT_PLAN §2.1) | P2 |

### 7.2 Compiler Resilience (IMPROVEMENT_PLAN §2)

| Item | Priority |
|------|----------|
| Memory budget tracking (graceful OOM) | P1 |
| Multithreaded compilation (parse + codegen) | P2 |
| LLVM API integration (inkwell — 10-50× codegen speedup) | P2 |

---

## 8. PHASE 5f — VERIFICATION (Planned)

### 8.1 Contract Verification (IMPROVEMENT_PLAN §3.0)

| Item | Priority |
|------|----------|
| Z3 static verification (prove contracts at compile time) | P1 |
| Abstract interpretation (array bounds, integer ranges) | P2 |
| Contract composition analysis (call chain verification) | P2 |
| Symbolic execution (auto-generated tests from contracts) | P2 |
| Contract coverage analyzer (`xiom test --coverage`) | P1 |
| Formal verification dashboard | P2 |

### 8.2 Other Tooling

| Item | Priority |
|------|----------|
| FFI binding generator (`xiom bind --header math.h`) | P1 |
| Visual benchmark tool (`xiom bench --compare`) | P1 |
| WASM compiler playground (`playground.xiom-lang.org`) | P2 |

---

## 9. PHASE 5g — SELF-HOSTING (Planned — LAST)

**DO NOT START until Phases 5a-5f are rock-solid.**

| Prerequisite | Status |
|-------------|--------|
| All compiler bugs fixed | ✅ (10/10) |
| Full language surface stable | In progress |
| Stdlib mature (string, I/O, collections, FFI) | ✅ |
| Full test suite passing (500+ tests) | 85 e2e, growing |
| Rust bootstrap kept permanently | Required |

### Bootstrapping Sequence:
1. Write `xiom-lexer.xi`, `xiom-parser.xi`, `xiom-check.xi`, `xiom-codegen.xi`
2. Compile with Rust `xiomc` → `xiomc-v1`
3. `xiomc-v1` compiles itself → `xiomc-v2`
4. Diff output: byte-for-byte identical → complete

---

## 10. PHASE 5x — EXPERIMENTAL FEATURES (Planned)

These features require more R&D before production readiness.

| Item | Status | Why Experimental |
|------|--------|-----------------|
| **`--ai` flag** (AI-friendly mode) | Deferred from 5c | Needs `--diagnostics=json` foundation first; static prompt templates; no real-time LLM |
| AI-assisted proof (IMPROVEMENT_PLAN §3.0E) | Deferred | Requires Z3 + LLM API integration |
| Code translator (`xiom translate-c/zig/rust`) | Deferred | Requires C header parser + type mapping |
| LLVM API (inkwell) | Deferred | Heavy dependency; text IR works for now |

**Current AI-friendly surface:** `--diagnostics=json` + `--dump-contracts` provide structured data for external tools without baking LLM calls into the compiler.

---

## 11. VERIFICATION PROTOCOL

```bash
cargo build -p xiomc
cargo test -p xiom-parser --lib
cargo test -p xiom-check --lib
cargo test -p xiom-codegen --test stdlib_execution_tests -- --nocapture
cargo test -p xiom-codegen --test e2e_tests
cargo test -p xiom-codegen  # all regression gates
```

---

## 12. APPENDIX: Archival Documents

| Document | Status |
|----------|--------|
| `docs/COMPILER_ARCHITECTURE.md` | Current state + architecture |
| `docs/COMPILER_IMPROVEMENT_PLAN.md` | Detailed improvement plan (source of truth for this roadmap) |
| `docs/SESSION.md` | Session handoff |
| `docs/ARC_A_POINTERS.md` | Pointer/reference design |
| `docs/PRODUCTION_HARDENING_BUGS.md` | All 10 bugs documented |
