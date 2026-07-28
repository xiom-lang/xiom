# XIOM Compiler — Production Roadmap

**Current:** v0.52.0 — **1060/1060 all tests** (692 compiler + 368 tooling)
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

### 8B/M2 — Stdlib Contracts (IN PROGRESS — 5/20 done)
| Item | Remaining | Effort |
|------|-----------|--------|
| M2.1 High-priority (stats, array, mem, fmt, runner) | 63 contracts added | DONE |
| M2.2 Medium-priority (iter, path, compress, async, net, log, test) | 7 modules | 2d |
| M2.3 Low-priority (bench, reflect, serialize, thread, env, contracts) | 4 modules | 1d |
| M2.4 Missing Rust types (From/Into, Deref, Cow, Duration) | 4 traits | 2d |

### 8B/M3 — Test Coverage (IN PROGRESS)
| Item | Status |
|------|--------|
| M3.2 compile_with_diagnostics tests | DONE (8 tests) |
| M3.3 LSP protocol tests (11 tests exist) | Pending (rename/codeAction) |
| M3.4 AST serialization round-trip tests | DONE (25 tests) |
| M3.5 Ecosystem package test infrastructure | Pending (75 packages) |

### 8B/M4 — Code Health
| Item | Status |
|------|--------|
| M4.1 Split IrEmitter | **DONE** (86 fields ? 5 sub-contexts) |
| M4.2 Split expr.rs | **DONE** (5,362?1,961 lines; Call?call.rs + Stmt?stmt.rs) |
| M4.3 Split LSP | **DONE** (2,850?10 modules) |
| M4.4 Remove unwraps | **DONE** (all production unwraps ? 0 across all crates) |
| M4.5 Remove process::exit | **DONE** (library: 14 exits ? 0; compile() returns Result) |
| M4.6 Document unsafe | **DONE** (2 blocks with SAFETY:) |
| M4.7 Split CompileConfig | **DONE** (as part of M4.1) |

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

### 8B/M10 — Scripting / JIT Mode (DONE — 2026-07-24)

| Phase | Items | Status |
|-------|-------|--------|
| M10.1 Foundation | Shebang lexer, implicit main wrapping, `xiom run` CLI, libloading JIT | **DONE** |
| M10.2 Standalone | `xiom --standalone` script-to-binary, `--scaffold` | **DONE** |
| M10.3 Self-host diff | 15 differential tests (AOT vs scripting IR), AOT parity verified | **DONE** |
| M10.4 REPL | `xiom repl` with state persistence (`:vars`, `:reset`) | **DONE** |
| M10.5 Watch | `xiom run --watch` with 200ms debouncing | **DONE** |

**Delivered: True JIT via libloading (DLL?load?call main() verified), 34 scripting tests, 15 diff tests, declaration support, shebang, cache eviction, cross-OS paths.**

### 8B/M11 — Final Hardening (DONE — 2026-07-24)

| Item | Status |
|------|--------|
| M11.1 Cross-OS CI | GitHub Actions workflow (Win/Linux/macOS) — DONE |
| M11.2 Runtime packaging | XIOM_RUNTIME_DIR override — DONE |
| M11.3 Cache hardening | 100MB LRU eviction, `xiom clean --cache` — DONE |
| M11.4 MCP scripting | W_SCRIPT workflow guide + test — DONE |
| M11.5 JIT diff | 15 language features diff-tested — DONE |
| M11.6 Script tests | 34 tests (flaky fixed with unique IDs) — DONE |
| M11.7 Release | v0.50.0 bumped, RELEASE_PROCESS updated — DONE |

**1040/1040 ALL TESTS PASS. Production ready.**

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

## 4.5 PHASE M12 — Scripting Ergonomics & Compiler Gaps (v0.51.0)

Gaps discovered during scripting-mode testing and real-world usage.

### M12.1 — Scripting Ergonomics (from md_to_html.xi testing)

| Gap | Issue | Fix |
|-----|-------|-----|
| `io.read_line()` returns Result | Every script needs `match Ok/Err` for stdin reading | Add `io.read_line_or_panic()` or auto-unwrap in scripting mode |
| String `+` returns Result | `"a" + "b"` doesn't work inline | Make `+` on Str infallible (it always succeeds) |
| No `line[2:]` slicing | Need `string.str_slice(line, 2, len)` — verbose | Add `Str.slice(start)` and `Str.slice(start, end)` methods |
| `--check` no implicit main | Can't test script compilation without `xiom run` | Apply implicit main in `--check` mode when no `fn main` found |
| No `Str.starts_with()` method | Must use `string.str_starts_with(s, prefix)` | Add `Str.starts_with(prefix)` and `Str.ends_with(suffix)` methods |

### M12.2 — Compiler Robustness

| Gap | Issue | Fix |
|-----|-------|-----|
| Declarations in implicit main | type/enum/interface inside fn body = error | Parser-level detection: move declarations to program level |
| No inline regex | String splitting/parsing requires manual loops | Add `string.str_split(s, delim)` ergonomic wrapper |
| No `io.print` vs `io.println` clarity | Both exist but `println` adds newline — confusing | Document difference clearly in AI_CONTEXT.md |

**M12 effort: 3d. Target v0.51.0.**

---

## 4.6 PHASE M13 — LSP 10/10 & IDE Experience (v0.51.0)

Goal: Production-grade LSP with stdlib completion, `lsp-types` adoption, formatting,
and full IDE integration across VS Code and the playground.

### M13.1 — Stdlib Completion Catalog (1d)

Build a `stdlib_completions.json` catalog by parsing all stdlib modules.

**Script:** `tools/build_completions.ps1` — runs the xiom parser on every stdlib
file, extracts `pub fn` declarations with signatures, and writes a JSON catalog.

```
stdlib_completions.json:
{
  "xiom.io": {
    "functions": [
      {"name": "println", "sig": "fn println(msg: Str)", "detail": "Print line to stdout"},
      {"name": "read_file", "sig": "fn read_file(path: Str) -> Result[Str, IOError]"},
      ...
    ]
  },
  "xiom.math": { ... },
  ...
}
```

**Integration:** LSP loads catalog at startup. `handle_completion` checks if dot-target
matches a known module name, returns catalog entries.

**Tests:** stdlib_completions.json validation (all 40 modules present, valid JSON,
signature format correct). LSP completion test: `io.` returns `println` in top position.

### M13.2 — Adopt lsp-types Crate (1.5d)

Replace hand-rolled JSON-RPC `serde_json::Value` with typed `lsp-types` structs.

| Before | After |
|--------|-------|
| `serde_json::json!({"contents": {"kind": "markdown", "value": ...}})` | `HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value: ... })` |
| `serde_json::json!({"range": ..., "severity": ...})` | `Diagnostic { range: Range { ... }, severity: Some(DiagnosticSeverity::ERROR), ... }` |
| Hand-parsed `params["position"]["line"]` | `let pos: Position = params.position;` |

**Benefits:**
- Type-safe — compiler catches missing fields at build time
- Spec-compliant — `lsp-types` follows the LSP specification exactly
- Future-proof — new LSP features are just new struct variants

**Tests:** Existing 11 LSP tests must pass with identical behavior. Add type-safety
test: deserialize a valid LSP message, verify typed fields.

### M13.3 — textDocument/formatting + rangeFormatting (0.5d)

Wire `xiom-fmt` as the LSP formatter.

```rust
"textDocument/formatting" => {
    let formatted = xiom_fmt::format(&text);
    Ok(Some(vec![TextEdit { range: full_doc_range, new_text: formatted }]))
}
"textDocument/rangeFormatting" => {
    let formatted = xiom_fmt::format_range(&text, range);
    Ok(Some(vec![TextEdit { range, new_text: formatted }]))
}
```

**Tests:** Format a file, verify output is valid XIOM. Round-trip: format ? parse ? format produces identical output.

### M13.4 — Rename/CodeAction Tests (0.5d) — closes M3.3

| Test | What it verifies |
|------|-----------------|
| `test_rename_local` | Rename a local variable, verify all occurrences updated |
| `test_rename_function` | Rename a function, verify call sites updated |
| `test_rename_cross_file` | Rename across multiple files in workspace |
| `test_codeaction_quickfix` | Code action suggests fix for type mismatch |

### M13.5 — Playground WASM Completion (1d)

Bundle `stdlib_completions.json` in the WASM module. Register Monaco
`CompletionItemProvider` that queries the catalog.

```javascript
monaco.languages.registerCompletionItemProvider('xiom', {
    provideCompletionItems: (model, position) => {
        const word = model.getWordUntilPosition(position);
        // Check if preceded by 'module.'
        const line = model.getLineContent(position.lineNumber);
        const before = line.substring(0, position.column - 1);
        const dotPos = before.lastIndexOf('.');
        if (dotPos > 0) {
            const moduleRef = before.substring(0, dotPos).split(/\s+/).pop();
            return fetchStdlibCompletions(moduleRef);
        }
        // ... default completions
    }
});
```

### M13.6 — REPL :list Command (0.5d)

```bash
xiom> :list io
  io.println(msg: Str)        Print line to stdout
  io.print(msg: Str)          Print without newline
  io.read_line() -> Str       Read line from stdin
  io.read_file(path: Str) -> Result[Str, IOError]

xiom> :list math
  math.sqrt(x: Float64) -> Float64
  math.pow(base: Float64, exp: Float64) -> Float64
  ...
```

### M13.7 — Diagnostics Quickfix + Document Links (0.5d)

Enhance existing diagnostic capabilities:
- Add "Did you mean?" suggestions for undefined variables
- Add document links for `use` statements (clickable to open imported file)
- Add folding range support for `{ ... }` blocks

### M13 Schedule

| Phase | Items | Effort | Depends on |
|-------|-------|--------|-----------|
| M13.1 | Stdlib completion catalog | 1d | -- |
| M13.2 | lsp-types adoption | 1.5d | -- |
| M13.3 | Formatting support | 0.5d | M13.2 |
| M13.4 | Rename/codeAction tests | 0.5d | M13.2 |
| M13.5 | Playground completion | 1d | M13.1 |
| M13.6 | REPL :list | 0.5d | M13.1 |
| M13.7 | Quickfix + links | 0.5d | M13.2 |
| M13.8 | Workspace index | 1d | M13.1 |
| M13.9 | Package index | 0.5d | M13.1, M13.8 |
| M13.10 | Cross-file use resolution | 0.5d | M13.8 |

**Total M13 effort: 7.5d. Target v0.51.0.**

### M13.8 — Workspace Index (1d) — custom modules

Parse all `.xi` files in the workspace at LSP startup. Extract every `pub fn`
declaration with signatures. Re-index on file save. Enables `use ./utils; utils.`
completion for project-local modules.

### M13.9 — Package Index (0.5d) — ecosystem packages

Scan `~/.xiom/packages/` for installed packages. Extract `pub fn` exports.
Refreshed on `xiom pkg install`. Enables `use mypkg; mypkg.` completion.

### M13.10 — Cross-File `use` Resolution (0.5d)

Follow `use` chains transitively. `use ./a;` where `./a.xi` has `use ./b;`
resolves functions from `b` through `a`.

---

## 4.7 PHASE M14 — Production Cleanup & Quality Gates (v0.51.0)

Full audit of all 17 compiler crates completed 2026-07-25. This phase addresses
the gaps found — oversized files, giant functions, dead code, missing docs,
duplication, and bare unwraps.

### M14.1 — Split Oversized Files (3d)

| File | Current Lines | Target | Split into |
|------|-------------|--------|-----------|
| `xiom-check/src/lib.rs` | **4,415** | 800 | `expr.rs`, `stmt.rs`, `borrow.rs`, `tests/` |
| `xiom-codegen/src/lib.rs` | **3,766** | 800 | `compile_struct.rs`, `compile_impls.rs`, `emit_helpers.rs`, `type_builtins.rs` |
| `xiom-codegen/src/call.rs` | **2,278** | 800 | `builtins.rs`, `user_call.rs`, `method_dispatch.rs` |
| `xiom-codegen/src/expr.rs` | **1,948** | 800 | `binary_ops.rs`, `literals.rs`, `control_flow.rs` |
| `xiom-codegen/src/types.rs` | **1,152** | 800 | `type_resolve.rs`, `type_inference.rs` |
| `xiom-codegen/src/stmt.rs` | **1,134** | 800 | `let_var.rs`, `match_compile.rs`, `loop.rs`, `return.rs` |
| `xiom-codegen/src/decl.rs` | **1,066** | 800 | `fn_decl.rs`, `struct_decl.rs`, `enum_decl.rs` |
| `xiom/src/lib.rs` | **2,030** | 800 | `compile_pipeline.rs`, `contracts.rs`, `incremental.rs`, `linker.rs` |
| `xiom-parser/src/lib.rs` | **1,851** | 800 | Extract tests to `tests/`, split `parse_postfix_expr` (178 lines) |
| `xiom-fmt/src/lib.rs` | **1,123** | 800 | `format_expr.rs`, `format_stmt.rs` |
| `xiom-pkg/src/main.rs` | **1,102** | 800 | `registry.rs`, `install.rs`, `manifest.rs` |
| `xiom-mcp/src/main.rs` | **1,049** | 800 | `tools/` subdirectory per tool |
| `xiom-dbg/src/main.rs` | **1,027** | 800 | `gdb.rs`, `cdb.rs`, `dap.rs`, `json_api.rs` |
| `xiom-verify/src/lib.rs` | 863 | 800 | `smt_gen.rs`, `z3_runner.rs` |

**Total: 14 files over 800 lines ? split into ~40 modules. 3d effort.**

### M14.2 — Split Giant Functions (2d)

Most oversized functions are in `xiom-codegen` and `xiom-check`. Each must be decomposed
into focused sub-functions with clear boundaries.

| Function | File | Lines | Split plan |
|----------|------|-------|-----------|
| `compile_eq_impl` | `lib.rs:2063` | **1,133** | `compile_struct_eq`, `compile_enum_eq`, `compile_interface_eq` |
| `check_expr` | `xiom-check/lib.rs:2087` | **778** | Extract per-variant handlers |
| `compile()` | `xiom/lib.rs:537` | **529** | `compile_to_ir()`, `emit_binary()`, `link_with_clang()` |
| `compile_option_impls` | `lib.rs:3198` | **450** | `compile_option_methods`, `compile_result_methods` |
| `compile_with_diagnostics` | `xiom/lib.rs:223` | **313** | Merge shared logic with `compile()` |
| `next_token` | `xiom-lexer/lib.rs:155` | **291** | `lex_number()`, `lex_string()`, `lex_char()`, `lex_operator()` |
| `handle_request` | `xiom-dbg/main.rs:777` | **167** | Dispatch table pattern |
| `run_json_mode` | `xiom-dbg/main.rs:558` | **178** | Dispatch table pattern |

### M14.3 — Fix Bare Unwraps + SAFETY Gaps (0.5d)

| Location | Count | Fix |
|----------|-------|-----|
| `xiom-check/lib.rs` | **9** remains | `.unwrap()` ? `.expect("invariant message")` |
| `xiom-ffigen/main.rs:263` | 1 | `serde_json::to_string_pretty` ? `.unwrap_or_default()` |
| `xiom-mcp/main.rs:242,260,647` | 3 | `.to_str().unwrap()` ? proper error handling |
| `xiom/main.rs:551` | 1 | `.parent().unwrap()` ? `.expect()` |
| `xiom/src/lib.rs` | **2 new** | `unsafe { set_var("XIOM_STDLIB") }` needs SAFETY comment |
| `xiom/src/jit.rs:37` | 1 | `unsafe { libloading }` needs SAFETY comment |

### M14.4 — Remove Dead Code & Deduplication (1d)

| Item | File | Action |
|------|------|--------|
| `struct_type_from_expr` duplicated | `lib.rs` + `types.rs` | Remove private copy, use public one |
| `const_promote_to_float` | `lib.rs:454` | Remove — never called |
| `try_i64_field_access` | `lib.rs:1215` | Remove — never called |
| `get_concrete_option_type` | `decl.rs:996` | Remove — never called |
| `get_concrete_result_type` | `decl.rs:1024` | Remove — never called |
| `release_borrows_for` | `xiom-check/lib.rs:3603` | Use or remove |
| `debug_test.rs` | `xiom-parser/src/` | Move to `tests/` or add `#[cfg(test)]` |
| `recover_to_sync`, `expect` | `xiom-parser/lib.rs` | Use or remove `#[allow(dead_code)]` |
| Duplicate type_to_string | `xiom/src/lib.rs` | Delegate to `xiom-display` crate |

### M14.5 — Document Public API (1.5d, LANGUAGE DOCS DONE — 2026-07-25)

**Language docs (website-ready):**
| Deliverable | Status |
|-------------|--------|
| `docs/language/index.md` | Updated to v0.51.0, 1041 tests, added Scripting Mode + Pattern Matching links |
| `docs/language/reference.md` | **NEW** — complete language reference (all syntax, types, patterns in one doc) |
| `docs/language/pattern-matching.md` | **NEW** — match, if let, while let, ? operator, exhaustiveness |
| 39 stdlib module docs | Existing — covers all modules |

**Rust API docs (remaining):**
| Crate | Undocumented pub items | Priority |
|-------|----------------------|----------|
| `xiom-check` | **25** (Checker, check_program, CheckedType, BorrowChecker, ...) | HIGH |
| `xiom-codegen/types.rs` | **17** (zero_val_for, type_from_ast, llvm_type_for, ...) | HIGH |
| `xiom-codegen/sandbox.rs` | **7** (SafetyAuditor, to_json, to_text, ...) | MEDIUM |
| `xiom-ast` | Partial variant docs on Type, Pattern, Stmt | MEDIUM |
| `xiom-lexer` | `Lexer`, `Token`, `tokenize()` | MEDIUM |
| `xiom-wasm` | `WasmDiagnostic`, `CompileResult` | LOW |
| `xiom-display` | `type_to_string`, `format_fn_signature` | LOW |

### M14.6 — Quality Fixes (1d)

| Item | Detail |
|------|--------|
| **5 unreachable!() without messages** | `call.rs:156,196,306`, `expr.rs:500`, `stmt.rs:825` — add diagnostic strings |
| **M13.8 fmt round-trip** | **DONE** — extern/unsafe blocks now round-trippable (M8, 2026-07-25) |
| **`type_to_string` dedup** | `xiom/src/lib.rs` copies from `xiom-display` — delegate instead |
| **AST variant docs** | Add doc comments to `Type`, `Pattern`, `Stmt` variants in `xiom-ast` |

### M14 Schedule

| Phase | Items | Effort | Status |
|-------|-------|--------|--------|
| M14.1 | Split 14 oversized files | 3d | Pending |
| M14.2 | Split 8 giant functions | 2d | Pending |
| M14.3 | Fix bare unwraps + SAFETY gaps | 0.5d | **DONE** |
| M14.4 | Remove dead code + deduplication | 1d | **DONE** |
| M14.5 | Document public API | 1.5d | **DONE** |
| M14.6 | Quality fixes (unreachable, round-trip, dedup) | 1d | **DONE** |
| M14.7 | LLVM constants extraction | 1d | **DONE** |

**Total M14 effort: 10d. Target v0.51.0.**
**M14.1 (file splits) and M14.2 (function splits) DEFERRED — cosmetic, do not affect self-hosting correctness.**

---

### M15 — Self-Hosting Readiness (Target: 10/10, ~4d)

**Current rating: 8/10.** Three codegen bugs prevent certain patterns from working. Fixing these enables the compiler to handle all valid XIOM programs including potentially self-compilation.

| Bug | Symptom | Root Cause | Blocks | Effort |
|-----|---------|-----------|--------|--------|
| **B-001** | `Result[T, struct E]` truncates error to 8 bytes | `%struct.Result = {i64,i64,i64}` hardcoded. Needs monomorphization: `Result__JsonValue__SerializeError` with actual field types | `parse_json`, `Err(Struct)`, any `Result` with struct payload | 2d |
| **B-002** | `&mut self` methods crash (ACCESS_VIOLATION) | LLVM IR for mutable receiver stores wrong pointer | `PathBuf.push`, `Vec.push`, mutable state patterns | 1d |
| **B-003** | `Option<Str>` from method returns crash (ILLEGAL_INSTRUCTION) | Method return on struct types produces wrong LLVM type for Option discriminant | `file_name`, `extension`, `file_stem`, any method returning `Option<T>` | 1d |

### M15 Approach (Different from failed attempts)

All previous attempts tried to fix the LLVM type *after* the function was compiled. The correct approach:
1. Pre-register concrete types during `register_type_layout_impl` (when the struct is first seen)
2. Change `type_from_ast` to return concrete names for `Result[T, E]` with struct args
3. The existing emission loop already handles both base and concrete types (already split into two passes)
4. `compile_fn` already pre-resolves the return type — just need to make sure the pre-registration runs

**Key insight from failed attempts:** The AST uses `Type::Named("Result", args)` not `Type::Result(ok, err)`. The pre-registration must handle BOTH forms.

### M15 Verification

After each fix:
- `json_parse("[1,2,3]")` returns `Ok(...)` (not empty Err)
- `PathBuf.push("foo")` actually modifies the buffer
- `Path.file_name()` returns correct `Some("file.txt")`
- Original smoke tests restored (removing workarounds)
- Full 1049 test suite passes

### M15 Schedule

| Phase | Items | Effort |
|-------|-------|--------|
| M15.1 | Fix B-001: Result struct truncation | 2d |
| M15.2 | Fix B-002: &mut self methods | 1d |
| M15.3 | Fix B-003: Option<Str> method returns | 1d |
| M15.4 | Restore original smoke tests + verify | 0.5d |

**Total M15 effort: 4d. Target v0.52.0 "Self-Host Ready".**

### What M15 does NOT include (deferred cosmetic work)

| Item | Why deferred |
|------|-------------|
| M14.1 file splits (11 remaining) | Cosmetic — file length doesn't affect correctness |
| M14.2 function splits (8 functions) | Cosmetic — function length doesn't affect correctness |
| LSP lsp-types adoption | Not needed for self-hosting |
| Formal verification (M5 deferred) | Nice-to-have, not blocking |

---

## M16 — Compiler Bug Fixing Phase (ACTIVE — 2026-07-28)

**Context:** 41 agents generated 1,015 E2E tests in 5 batches (M32-M36), discovering
15+ compiler bugs. Total test baseline: ~3,136. All bugs are caused by correct
XIOM syntax that the compiler mishandles. No test simplification — compiler
must be hardened to pass all tests.

### M16 Critical Bugs (P0 — LLVM Type Mismatch / Crash)

| Bug ID | Symptom | Root Cause | Trigger | Effort |
|--------|---------|------------|---------|--------|
| **B-004** | Int8/Int16/Int32 ops generate i8/i32 LLVM types mixed with i64 | Codegen promotes small ints to i64 but types inconsistently applied at operand boundaries | `var x: Int8 = 5; return x + 1;` | 3h |
| **B-005** | `~` NOT on Int8/Int16/Int32 crashes | Same as B-004 — trunc/zext missing around NOT operand | `var x: Int32 = 0; return ~x;` | 2h |
| **B-006** | `-1 as Int8` generates i8 in i64 subtract mismatch | Unary negation on casted value: `sub i64 0, %tmp7` where %tmp7 is i8 | `var x: Int8 = -128;` | 2h |
| **B-007** | Returning closures ? ACCESS_VIOLATION (0xC0000005) | Closure return type not properly lowered; fn ptr return value mismatch | `fn make_adder(n: Int) -> fn(Int) -> Int { return fn(x) { x + n }; }` | 4h |
| **B-008** | `&Int` deref crashes at runtime | Pointer deref of stack int produces misaligned load | `var x: Int = 42; var p = &x; return p;` | 2h |

### M16 High Bugs (P1 — Incorrect Codegen / Semantic Bugs)

| Bug ID | Symptom | Root Cause | Trigger | Effort |
|--------|---------|------------|---------|--------|
| **B-009** | `derive[Ord]` on structs ? broken LLVM IR (`icmp slt` on struct type) | Ord derive emits icmp on aggregate type instead of field-wise comparison | `type Pair = { x: Int; y: Int; } derive[Ord]` | 3h |
| **B-010** | Str-derived `Eq` compares pointers, not content | derive[Eq] for structs with Str fields emits ptr compare instead of strcmp | `type Named = { name: Str; } derive[Eq]` | 2h |
| **B-011** | Display `derive` returns empty string | Auto-derived fmt() generates empty format string | `type T = {} derive[Display]; t.to_str()` | 1h |
| **B-012** | Nested `Option[Result[...]]` ? getelementptr crash | Nested generic types not properly monomorphised — type layout mismatch | `fn f() -> Option[Result[Int, Str]] { ... }` | 3h |
| **B-013** | Option-struct field access returns wrong data | Value extracted from `Option[Struct]` via match has incorrect LLVM type for field access | `match opt { Some(s) => s.field, ... }` | 2h |
| **B-014** | Custom generic enum multi-field payload ? ACCESS_VIOLATION | Enum variant with struct-like payload in generic enum generates wrong GEP indices | `enum Container[T] { Pair(a: T, b: T) }` | 3h |
| **B-015** | Struct field address-of `&s.field` crashes at runtime | Address-of on struct field through GEP produces invalid pointer | `var p = &point.x;` | 2h |

### M16 Medium Bugs (P2 — Parser/Checker Gaps)

| Bug ID | Symptom | Root Cause | Trigger | Effort |
|--------|---------|------------|---------|--------|
| **B-016** | `Bool as Int` / `Int as Bool` rejected by checker (T001) | Checker type system doesn't support Bool?Int casts | `var x: Int = true as Int;` | 1h |
| **B-017** | `Vec![]` macro not parsed | Parser expects `Vec[T].new()` instead of macro form | `var v = Vec![1, 2, 3];` | 30m |
| **B-018** | `else if` not supported | Parser only accepts `elif` as chained condition keyword | `if a { } else if b { }` | 30m |
| **B-019** | `impl Type { }` (without `for Trait`) rejected | Parser expected `impl Trait for Type` but XIOM has `impl Type` + method defs | `impl Counter { fn inc(c: Counter) -> Counter { ... } }` | 1h |
| **B-020** | Standalone `{ }` blocks not valid at statement level | Parser requires `if/while/etc.` wrapping; bare blocks aren't statements | `{ var x = 5; }` | 30m |
| **B-021** | Const folding not implemented | `40+2` generates add instruction instead of literal 42 — optimization gap | `return 40 + 2;` | 2h |
| **B-022** | Negative float literal `-1.5` not parsed | Unary minus on float literal treated as separate token | `var f: Float64 = -1.5;` | 30m |

### M16 Schedule

| Phase | Items | Effort |
|-------|-------|--------|
| M16.1 P0 | B-004, B-005, B-006 (Int type mismatch fixes) | 7h |
| M16.2 P1 | B-007, B-008 (Closure + pointer crash fixes) | 6h |
| M16.3 P1 | B-009, B-010, B-011 (Derive fixes) | 6h |
| M16.4 P1 | B-012, B-013, B-014, B-015 (Nested type + enum fixes) | 10h |
| M16.5 P2 | B-016 through B-022 (Parser/checker gaps) | 5h |
| M16.6 | Verify all 30 failing M32 tests now pass | 2h |

**Total M16 effort: ~36h. Target v0.53.0 "Hardened Compiler".**

---

## M16.1 STATUS — Partially Complete (2026-07-28)

| Bug | Status |
|-----|--------|
| B-004 Binary op widening | ? FIXED (+12 tests, zero regressions) |
| B-005 BitNot widening | ? FIXED |
| B-006 Negation after cast | ? FIXED |
| B-008 Int8 store truncation | ?? DEFERRED — 3 approaches tried, all caused regressions in i64-first ABI |
| B-007/009-022 | ?? DEFERRED — blocked on narrow-int refactor |

**Key finding:** The i64-first ABI design (all integers stored as i64 in LLVM)
is the root cause of all remaining narrow-int failures. Point fixes cannot
resolve this without broader refactoring. See v0.53.0 plan below.

---

## v0.53.0 "Narrow-Int Foundation" — Production Plan

**Target:** First-class LLVM integer types. Every XIOM integer width maps to
its native LLVM width with correct truncation/extension at boundaries.

**Why this approach, not patching:**
- 3 point-fix attempts (B-008) all caused regressions
- The i64-first design is fundamentally incompatible with correct Int8/Int16/Int32 semantics
- Every other systems language (Rust, Zig, C, Ada) uses native LLVM widths
- The current design works for 96% of tests only because most code uses `Int` (i64)

### M17 — Narrow-Int Refactor (Option A: First-Class LLVM Types)

**Architecture change:**
```
BEFORE (i64-first):          AFTER (native widths):
  Int   ? i64                  Int   ? i64
  Int8  ? i64 (lossy!)        Int8  ? i8  + sext/zext on load, trunc on store
  Int16 ? i64 (lossy!)        Int16 ? i16 + sext on load, trunc on store
  Int32 ? i64 (lossy!)        Int32 ? i32 + sext on load, trunc on store
  UInt8 ? i64 (lossy!)        UInt8 ? i8  + zext on load, trunc on store
  Char  ? i64                 Char  ? i32 (Unicode 32-bit)
```

**Implementation steps:**

| Step | Task | Effort |
|------|------|--------|
| M17.1 | Update `llvm_type_for` to return native widths for all types | 2h |
| M17.2 | Update `widen_to_i64` to handle all widths with correct sign extension (sext for signed, zext for unsigned, zext for Char) | 2h |
| M17.3 | Update alloca/store paths in stmt.rs to use declared type widths | 3h |
| M17.4 | Update function param/return type lowering to native widths | 3h |
| M17.5 | Update `coerce_value` for all width conversions | 3h |
| M17.6 | Update struct field type resolution | 2h |
| M17.7 | Update enum discriminant/payload storage | 3h |
| M17.8 | Update Vec element storage (vec_abi.rs truncation paths) | 2h |
| M17.9 | Update derive codegen (Eq/Ord/Hash/Display) for native widths | 3h |
| M17.10 | Run full 3000+ test suite, fix regressions | 8h |
| M17.11 | Verify M32 18 remaining integer tests pass | 2h |

**Total M17 effort: ~33h (4-5 days)**

**M17 verification checklist:**
- [ ] All 240 M32 integer stress tests pass (currently 222/240)
- [ ] All 1303 E2E tests pass (currently 1282/1303)
- [ ] `Int8(-128) - Int8(1) == Int8(127)` (wrapping semantics)
- [ ] `UInt8(255) + UInt8(1) == UInt8(0)` (wrapping semantics)
- [ ] `Int32(-1)` stored and loaded correctly
- [ ] All struct fields with narrow int types preserve values
- [ ] All enum payloads with narrow int types preserve values
- [ ] Generic functions with Int8/Int16/Int32 type params compile correctly
- [ ] Compound assignment on narrow int types works
- [ ] LLVM IR passes `opt --verify` without type errors

---

## v0.53.0 — Spec Review Improvements

The following improvements were identified through external language review and
are agreed to strengthen XIOM's position as a production systems language.

### M18 — Pattern Guards

| ID | Task | Effort |
|----|------|--------|
| M18.1 | Add `if` guard to match arm syntax: `pattern if condition => expr` | 3h |
| M18.2 | Add guard variables to match arm binding scope | 2h |
| M18.3 | Codegen: guard as condition before arm body | 2h |
| M18.4 | Checker: guard expression must be `Bool` | 1h |
| M18.5 | Parser: `pattern => if cond { body }` desugars to guarded arm | 1h |

**Syntax:**
```xiom
match value {
  Some(v) if v > 10 => process(v),
  Some(v)           => default_handler(v),
  None              => {},
}
```

**Total M18 effort: ~9h**

### M19 — Default Interface Implementations

| ID | Task | Effort |
|----|------|--------|
| M19.1 | Allow method bodies in `interface` declarations | 2h |
| M19.2 | Default method dispatch: use default if type doesn't provide override | 3h |
| M19.3 | `Self` type resolution in default method bodies | 2h |
| M19.4 | Checker: verify default bodies compile against `Self` | 2h |
| M19.5 | Parser: allow `{ ... }` body after interface method sig | 1h |

**Syntax:**
```xiom
interface Comparable {
  fn compare(other: &Self) -> Int;  // no default — must implement
  fn lt(other: &Self) -> Bool { return self.compare(other) < 0; }  // default
}
```

**Total M19 effort: ~10h**

### M20 — Error Conventions (Soft Convention)

| ID | Task | Effort |
|----|------|--------|
| M20.1 | Document recommended `Error` interface in spec (not enforced) | 1h |
| M20.2 | Add `message() -> Str` method to stdlib `error::Error` interface | 1h |
| M20.3 | Update code patterns section with canonical error handling examples | 1h |
| M20.4 | Add `AppError` and `DomainError` standard patterns to docs | 1h |

**Total M20 effort: ~4h (documentation + minor stdlib)**

### M21 — Borrow Checker Activation (Partial)

| ID | Task | Effort |
|----|------|--------|
| M21.1 | Fix spec: "Borrow returned from function" ? "Borrow returned from function to stack-local data" | 1h |
| M21.2 | Activate borrow checker for `&mut` exclusivity (write borrow while read active) | 3h |
| M21.3 | Activate borrow checker for use-after-move detection | 3h |
| M21.4 | Allow borrow returns for heap/caller-owned data (stdlib functions like `array.first()`) | 2h |
| M21.5 | Add borrow-checker test suite: 100+ edge cases | 8h |

**Total M21 effort: ~17h**

---

## v0.53.0 Test Plan

After M17-M21, the following NEW tests will be added:

| Area | Tests | Target |
|------|-------|--------|
| Narrow-int exhaustive: all width combinations for all ops | 200 | All pass |
| Pattern guard combinatorics: every guard shape | 80 | All pass |
| Default interface impl dispatch: static/dynamic/override | 60 | All pass |
| Borrow checker activation: move, borrow, mutate edges | 100 | Most pass, some defer |
| Error convention patterns: real-world error handling | 40 | All pass |
| Self-host lint: compile the compiler's own source patterns | 50 | All pass |
| **TOTAL new tests** | **~530** | **3500+ test baseline** |

---

## Honest Self-Hosting Readiness Assessment

**Can XIOM self-host after v0.53.0? — PARTIALLY YES, fully NO.**

### What WILL work (the compiler can compile itself):

| Feature | Status | Compiler needs this? |
|---------|--------|---------------------|
| Int arithmetic (narrow-int refactored) | ? After M17 | Yes — heavily |
| Enums with payloads (AST nodes) | ? Working | Yes — AST is enum |
| Structs with nested fields | ? Working | Yes — types/diagnostics |
| Generics and monomorphisation | ? Working | Yes — type system |
| Match expressions | ? Working | Yes — parser/codegen |
| Functions/closures | ? Working | Yes — lexer/parser |
| Modules and visibility | ? Working | Yes — crate system |
| String operations | ? Working | Yes — lexer |
| Option/Result/simple contracts | ? Working | Yes — error handling |
| File I/O | ?? Needs stdlib | Yes — read source files |
| Derive Eq/Ord/Clone | ?? After fix | Yes — AST comparison |

### What will NOT work yet:

| Feature | Status | Blocked by |
|---------|--------|------------|
| Borrow checker (full) | ? Not active | Safety — the compiled compiler would have memory bugs |
| Thread safety | ? Not active | The compiler is single-threaded, OK for now |
| Full stdlib (40 modules) | ? ~30% | Compiler doesn't need most modules |
| Async/await | ? Not active | Not needed |

### The honest answer:

**v0.53.0 can produce a compiler binary that compiles correct XIOM source.**
The binary won't be safe (no borrow checker) but it will be correct (codegen
produces right answers). This is sufficient for a "self-host preview" —
compile the compiler with itself and verify the output compiles the same
programs identically.

**Full self-hosting (replace Rust bootstrap) requires v0.54.0:**
- Active borrow checker ? safety
- Working `derive` ? Eq/Ord for AST comparison
- File I/O stdlib ? read source files on all platforms
- Full differential testing: Rust-bootstrap-compiled vs self-compiled IR must match

**Recommendation:** Target v0.53.0 for "self-host preview" (correctness, not safety).
Target v0.54.0 for "self-host production" (correctness + safety).

---

## v0.53.0 Schedule Summary

| Phase | Contents | Effort |
|-------|----------|--------|
| M17 | Narrow-int refactor (Option A: first-class LLVM types) | 33h |
| M18 | Pattern guards | 9h |
| M19 | Default interface implementations | 10h |
| M20 | Error conventions (docs + minor stdlib) | 4h |
| M21 | Borrow checker activation (partial) | 17h |
| M22 | Test suite expansion (+530 tests) | 16h |
| M23 | Fix remaining M32 integer failures from refactor | 4h |
| M24 | Self-host preview differential testing | 8h |
| **Total** | **v0.53.0 "Narrow-Int Foundation + Spec Review"** | **~101h (2-3 weeks)** |

### v0.53.0 Target Metrics

| Metric | Current | v0.53.0 Target |
|--------|---------|----------------|
| Total test baseline | ~2986 | ~3500 |
| E2E pass rate | 1282/1303 (98.4%) | 100% |
| Narrow-int correctness | ? (18 failures) | ? All 240 M32 tests pass |
| Pattern guards | ? Not implemented | ? Production |
| Default interface impls | ? Not implemented | ? Production |
| Borrow checker (partial) | ? Not active | ? Move/exclusivity active |
| Self-host preview | ? | ? Compiles self, IR matches bootstrap |

| Version | Date | Tests | Notes |
|---------|------|-------|-------|
| **v0.52.0** | 2026-07-26 | **~1055** | M15 complete, B-001/B-002/B-003 fixed, self-host ready 10/10 |
| **v0.51.0** | 2026-07-25 | **1049** | M1-M12 complete, M14.3-M14.7 done, P0+P1 closed, release-ready |
| **v0.50.0** | 2026-07-25 | **1041** | M10-M12 complete, scripting/JIT, libloading, cache, CI, 34 script + 15 diff tests. M4/M9 all done. XIOM v0.50.0 LLVM IR header. Auto stdlib discovery. |
| v0.49.9 | 2026-07-24 | 934 | M4.1 (IrEmitter split), M4.3 (LSP split), M4.6 (unsafe docs), playground fixes (372 lessons), M9.6 (impl Trait), Node.js registry backend. 11/11 M9 closed. |
| v0.49.8 | 2026-07-21 | 924 | 10/11 M9 gaps closed, ASCII installer, fuzz harnesses, HTML docs |
| v0.49.7 | 2026-07-21 | 910 | Phase 8B/M4-M9 complete, preflight audit |
| v0.49.5 | 2026-07-20 | 871 | Phase 7 complete, installer v2 |
| v0.48.9 | 2026-07-20 | 768 | Phase 5-6 complete |
