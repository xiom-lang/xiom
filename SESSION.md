# XIOM — Session Handoff: v0.22.1 "Hardened"

**Date:** 2026-07-03  
**Branch:** `feat/ecosystem`  
**Status:** Ready to merge to main. All 186 tests green. 30-module benchmark suite compiles. Architecture and improvement plan documented.

---

## Next Session: Rebranding XIOM → XIOM

### Why

XIOM is a trademarked name and `.xi` is used by other tools. Full rebranding is needed before public release.

### Target Names

| Old | New |
|-----|-----|
| Language: **XIOM** | **XIOM** |
| Source files: `.xi` | `.xi` |
| Bytecode: `.xibc` | `.xibc` |
| Compiler: `xiomc` | `xiomc` |
| Crate prefix: `xiom-*` | `xiom-*` |
| Config: `kilo.jsonc` → unchanged | Project name doesn't need renaming |

### Rebranding Difficulty Assessment (Updated — Full Scope)

**Scale:** ~250 files across the entire project. Mechanical but extensive.

| Layer | Files | Effort | Risk |
|-------|-------|--------|------|
| **Source files** (.xi → .xi) | ~35 example/benchmark/spec files | Low — batch rename | Low |
| **Module declarations** (in .xi files) | `module a.b.c` stays the same — only file extension changes | None | None |
| **Crate names** (Cargo.toml) | 7 `Cargo.toml` files | Low — string replace | Medium |
| **Rust source** (`xiom_*` → `xiom_*`) | ~15 .rs files | Medium — crate refs, use statements, strings | Medium |
| **Docs** (COMPILER_VERSIONS.md, etc.) | ~15 .md files | Low — find/replace | Low |
| **Specs** (XIOM_*.md → XIOM_*.md) | 9 files | Low — rename + content | Low |
| **Website** (all HTML/CSS/JS) | **12 HTML + 1 CSS + images + playground** | Medium — every page has XIOM in title, nav, headers, code samples, footer | Medium |
| **Grammar** (xiom.tmLanguage.json → xiom.tmLanguage.json) | 1 file + scopeName | Low | Low |
| **C runtime** (xiom_runtime.c) | 1 file | Low | Low |
| **Build/install scripts** | 5 scripts (install.ps1, install.sh, install_deps.ps1, install_deps.sh, install.bat) | Low — PATH references, binary names | Low |
| **Website content** | index.html, xiom-landing.html, spec.html, docs.html, download.html, ecosystem.html, versions.html, AI_CONTEXT.html, style.css, playground/ | Medium — ~150 "XIOM" references across all pages | Low |
| **VS Code extension** | grammar, snippets, config | Low | Low |
| **kilo.jsonc** | No changes needed | None | None |
| **Total** | **~250 files** | **3-4 hour session** | **Low-Medium** |

### Website Files to Update

| File | XIOM Refs | What Changes |
|------|-----------|--------------|
| `index.html` | 25+ | Title, logo text, nav, hero, pillars, code samples, footer |
| `xiom-landing.html` | 30+ | Title, nav, hero, proof panel filename, pillars, footer |
| `spec.html` | 40+ | Title, nav, type names in code samples, comparison table |
| `docs.html` | 5+ | Redirect, title, nav |
| `download.html` | 20+ | Title, nav, CLI commands, binary names, paths |
| `ecosystem.html` | 35+ | Title, nav, package names (xiom-http → xiom-http, etc.) |
| `versions.html` | 15+ | Title, nav, version rows |
| `AI_CONTEXT.html` | 50+ | Every code sample, type reference, keyword |
| `style.css` | 2 | Title in CSS comment |
| `playground/` | ~10 | Server config, HTML templates |
| `docs/` subdirectory | ~15 .md files | Internal references |

### Distribution / Installer Scope

| File | What Changes |
|------|-------------|
| `install.ps1` | Binary name `xiomc.exe` → `xiomc.exe`, PATH additions, icon references |
| `install.sh` | Same — Unix paths, binary names |
| `install_deps.ps1` | Tool references (likely unchanged — installs Rust/LLVM) |
| `install_deps.sh` | Same |
| `release/xiom-v0.20.0/install.bat` | Binary name, PATH |
| `package.ps1` | Archive names, binary references |
| `Cargo.toml` (root) | Workspace member names |
| `.vscode/` | Extension config, task names |

### Recommended Approach

1. **Atomic commits** — one commit per layer (source files, then crates, then docs, then tests)
2. **Compile after each commit** — `cargo build` to catch missed references immediately
3. **Keep `feat/ecosystem` branch** — do rebranding on `feat/rebrand` branch, merge to main
4. **Run full test suite** after rebranding — 186 tests must stay green
5. **Update SESSION.md with rebranded names** after the session

### Phase Reordering

**Decision: Self-hosting comes after Phase 3 (Z3, toolchain, debugger), not before.**

The v0.9.x–v0.11.x self-hosting MVP worked — concept proven. But pursuing self-hosting while the Rust compiler was unstable caused benchmark breakage. The revised order:

1. **Phase 2 (now):** Harden Rust compiler — performance, warnings, benchmarks, multi-file, hot reload
2. **Phase 3 (next):** Z3 static verification, debugger (DAP), LSP, CLI toolchain, visual benchmarks
3. **Phase 4 (final):** Self-hosting — bootstrap XIOM compiler in XIOM, byte-for-byte verified
4. **Ecosystem (after):** Showcase projects (AxiomDB, AxiomVDB), package registry

The Rust compiler is the PERMANENT bootstrap fallback — never deleted.

See `docs/COMPILER_IMPROVEMENT_PLAN.md` for the detailed roadmap, `docs/XIOM_DISTRIBUTION_SPEC.md` for the distribution + installer specification, and `specs/XIOM_Build_Strategy.md` for the decision log.

---

## What Was Accomplished This Session

### Branch 1: Multi-File Module Catalog (`xiom-check` + `xiomc`)

- **`ModuleCatalog`** struct: lazy-loading cache of `.xi` files from `source_dirs`. Files are parsed + cached on first reference (no eager scanning).
- **`CachedModule`**: stores parsed AST, type registry, function registry, export map, enum variants, variant fields.
- **`find_by_module_name()` / `find_submodule()`**: filesystem-aware lookups with auto-load.
- **`register_all_types_into()`**: bulk-registers catalog entries into a Checker.
- **`collect_external_decls()`**: iterates checker's `self.types` + `self.functions`, creates `TopDecl::Type` stubs for types/functions missing from current AST. Filters primitives (Bool, Int-64, UInt-64, Float32/64, Str, Char, Slice, Vec, Option, Result, Map, Set, fn).
- **`CheckedType::to_ast_type()`**: converts internal `CheckedType` back to AST `Type` for codegen consumption.
- **Injection gate** (`main.rs`): deduplicated type-only injection before codegen. Functions filtered out (bodies handled by codegen's own `register_functions`).
- **Resolve flow**: `resolve_imports` → `process_use` → catalog → lazy-load → register types/fns → inject AST stubs → codegen.

### Branch 2: Codegen Hardening (~20 fixes in `xiom-codegen`)

**Structural fixes:**
| # | Fix | Error resolved |
|---|------|----------------|
| 1 | Mixed-type binary op coercion (`sitofp i64→double`) | `sin_taylor` fmul type mismatch |
| 2 | Module-qualified receiver calls (skip i64 0 for type/module names) | `TrafficLight.new()` spurious receiver arg |
| 3 | Pointer type comparisons (`i8*` icmp + `inttoptr` null) | String comparison `i64 vs ptr` |
| 4 | Module-scoped type registry + `current_module` tracking | `types.Person` vs `derive.Person` collision |
| 5 | Derived methods registered in `self.functions` | `eq`/`clone` return type inference |
| 6 | `infer_llvm_type` for `Expr::Field`/`Ref`/`MutRef` | Struct field types defaulting to `i64` |
| 7 | Return type coercion (`sitofp` in return stmt) | `ret double %i64_val` mismatch |
| 8 | Struct equality via `.eq()` with `emitted_fns` check | `@Option.eq` / `@Vec.eq` undefined |
| 9 | Generic field type fallback (use value's `infer_llvm_type`) | `Range[Float64]` fields as `i64` |
| 10 | Deterministic `infer_struct_type_name` (`current_module` first) | Module cross-reference non-determinism |
| 11 | Generic call receiver handling (self param + monomorphised fn sig) | `Counter.set_Int` receiver arg missing/doubled |
| 12 | `zeroinitializer` for struct-type stores (Let/Var/Assign/Destructure/match) | `store %struct.X 0` invalid LLVM |
| 13 | `compile_eq_impl` nested struct eq guard (`emitted_fns` check) | Derived eq calling non-existent `Vec.eq` |
| 14 | Builtin `%struct.Vec` emitted before user struct types | `%struct.Stack { %struct.Vec }` ordering |
| 15 | `fn_key` module-scoped for receiver types | `describe_error` param type resolution |
| 16 | `Pattern::Ident` as enum variant in match checks | `List.sum()` Nil arm stores struct as `i64` |
| 17 | `compile_generic_monomorphisations` field locals | `Stack.push()` bare-name field access (`items`) |
| 18 | `llvm_type_for` resolves variant → parent enum | `Image(...)` / `DivByZero` type as `i64` |
| 19 | `Expr::Struct` variant field index mapping (lookup by name) | `Add(left:, right:)` field offsets wrong |
| 20 | `infer_llvm_type` variant → enum struct type | `describe_error(DivByZero)` arg type |
| 21 | Function pointer return type inference (`current_return_type`) | `predicate(v[i])` returns Bool |
| 22 | Vec builtins guarded — no inline for non-Vec types | `Stack.push()` triggering Vec GEP on Stack struct |

### Current State

| Metric | Value |
|--------|-------|
| **Axiom-check tests** | **44/44 pass** |
| **Axiom-codegen tests** | **139/139 pass** (diff + e2e + full_diff + integration) |
| **benchmark_safe.xi** | Compiles + runs, exit 34 |
| **benchmark_stress.xi** | Compiles via `--emit-llvm`; `--run` has 1 pre-existing issue (tuple return in `partition()`) |
| **bench_math.xi** (multi-file) | Resolves `use benchmark.main.BenchResult` via ModuleCatalog; IR has correct `BenchResult` type |
| **test_mod/math.xi** (multi-file) | Compiles + runs, exit 34 |
| **Multi-file resolution** | ModuleCatalog works for lazy file loading and `collect_external_decls` injection |

### Remaining: 1 Pre-Existing Codegen Issue

```
partition() returns (Vec[Int], Vec[Int]) — tuple return type
not yet supported by the codegen.
```

Tuples are parsed and type-checked but codegen lacks tuple struct type emission and destructuring.

---

## Testing Commands

```powershell
cd E:\Projects\XIOM

# Full test suite
cargo test -p xiom-check          # 44 tests
cargo test -p xiom-codegen        # 139 tests
cargo test                          # all tests

# Safe benchmark (always works)
cargo run -p xiomc -- --run examples\benchmark_safe.xi
# Expected: compiled: a.exe, exit code: 34

# Stress benchmark (1 remaining pre-existing tuple return issue)
cargo run -p xiomc -- --emit-llvm examples\benchmark_stress.xi 2>$null | Out-String | Set-Content a.exe.ll -NoNewline
& "C:\Program Files\LLVM\bin\clang.exe" -o a.exe a.exe.ll 2>&1
# If OK: .\a.exe ; echo "EXIT: $LASTEXITCODE"

# Multi-file test
cargo run -p xiomc -- --run examples\test_mod\math.xi

# Individual benchmark with multi-file resolution
cargo run -p xiomc -- --run examples\benchmark\bench_math.xi
```

---

## Architecture: Module Resolution Flow

```
File B: `use fileA.Type`
  → process_use → catalog.find_by_module_name("fileA")
  → lazy-load + parse + cache fileA.xi
  → register types/functions into checker
  → collect_external_decls → create AST stubs
  → inject into program.items before codegen
  → codegen sees all types → valid LLVM IR
```
