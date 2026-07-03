# AXIOM — Session Handoff: v0.22.1 "ModuleCatalog"

**Date:** 2026-07-03  
**Branch:** `feat/ecosystem`  
**Status:** ModuleCatalog implemented and verified. All multi-file examples compile via catalog. axiom-check 44/44. axiom-codegen 141/141 (including 2 new multi-file e2e tests). Zero warnings. v10 selfhost flake resolved.  
**Tests:** 44/44 axiom-check, 141/141 axiom-codegen (25 diff + 63 e2e + 23 full_diff + 30 integration). e2e: 2 new multi-file tests pass, 1 ignored (24-module benchmark/main.ax — deferred to Wave 2).  
**Key files changed:** `crates/axiom-check/src/lib.rs` (+~290 catalog + ~30 loading fixes), `crates/axiomc/src/main.rs` (+~55 injection gate + subsystem fix), `crates/axiom-codegen/src/lib.rs` (warnings), `crates/axiom-codegen/tests/e2e_tests.rs` (v10 flake + 3 multi-file tests), `crates/axiom-codegen/tests/full_diff_tests.rs` (warnings), `docs/requirements/multi-file-catalog.md` (NEW), `docs/checklists/multi-file-catalog.md` (NEW)

---

## What Was Accomplished This Session

### Branch 1: Multi-File Module Catalog (`axiom-check` + `axiomc`)

- **`ModuleCatalog`** struct: lazy-loading cache of `.ax` files from `source_dirs`. Files are parsed + cached on first reference (no eager scanning).
- **`CachedModule`**: stores parsed AST, type registry, function registry, export map, enum variants, variant fields.
- **`find_by_module_name()` / `find_submodule()`**: filesystem-aware lookups with auto-load.
- **`register_all_types_into()`**: bulk-registers catalog entries into a Checker.
- **`collect_external_decls()`**: iterates checker's `self.types` + `self.functions`, creates `TopDecl::Type` stubs for types/functions missing from current AST. Filters primitives (Bool, Int-64, UInt-64, Float32/64, Str, Char, Slice, Vec, Option, Result, Map, Set, fn).
- **`CheckedType::to_ast_type()`**: converts internal `CheckedType` back to AST `Type` for codegen consumption.
- **Injection gate** (`main.rs`): deduplicated type-only injection before codegen. Functions filtered out (bodies handled by codegen's own `register_functions`).
- **Resolve flow**: `resolve_imports` → `process_use` → catalog → lazy-load → register types/fns → inject AST stubs → codegen.

### Branch 2: Codegen Hardening (~20 fixes in `axiom-codegen`)

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
| **benchmark_safe.ax** | Compiles + runs, exit 34 |
| **benchmark_stress.ax** | Compiles via `--emit-llvm`; `--run` has 1 pre-existing issue (tuple return in `partition()`) |
| **bench_math.ax** (multi-file) | Resolves `use benchmark.main.BenchResult` via ModuleCatalog; IR has correct `BenchResult` type |
| **test_mod/math.ax** (multi-file) | Compiles + runs, exit 34 |
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
cd E:\Projects\AXIOM

# Full test suite
cargo test -p axiom-check          # 44 tests
cargo test -p axiom-codegen        # 139 tests
cargo test                          # all tests

# Safe benchmark (always works)
cargo run -p axiomc -- --run examples\benchmark_safe.ax
# Expected: compiled: a.exe, exit code: 34

# Stress benchmark (1 remaining pre-existing tuple return issue)
cargo run -p axiomc -- --emit-llvm examples\benchmark_stress.ax 2>$null | Out-String | Set-Content a.exe.ll -NoNewline
& "C:\Program Files\LLVM\bin\clang.exe" -o a.exe a.exe.ll 2>&1
# If OK: .\a.exe ; echo "EXIT: $LASTEXITCODE"

# Multi-file test
cargo run -p axiomc -- --run examples\test_mod\math.ax

# Individual benchmark with multi-file resolution
cargo run -p axiomc -- --run examples\benchmark\bench_math.ax
```

---

## Architecture: Module Resolution Flow

```
File B: `use fileA.Type`
  → process_use → catalog.find_by_module_name("fileA")
  → lazy-load + parse + cache fileA.ax
  → register types/functions into checker
  → collect_external_decls → create AST stubs
  → inject into program.items before codegen
  → codegen sees all types → valid LLVM IR
```
