# AXIOM — Requirements: Multi-File Module Catalog

**Date:** 2026-07-03
**Branch:** `feat/ecosystem`
**Status:** In Progress (Red baseline established)

## Background

The session document (`SESSION.md`) describes a `ModuleCatalog` architecture with `CachedModule`,
`register_all_types_into`, `collect_external_decls`, and `CheckedType::to_ast_type` — none of
these exist in the codebase (verified via grep of `crates/`). The actual module resolution in
`crates/axiom-check/src/lib.rs` uses only `load_external_module` (single-segment, looks up
`{source_dir}/{name}.ax`) and the nested-path variant `load_external_module_path` is dead code
(Rust `#[warn(dead_code)]` fires at lib.rs:715).

As a result, three multi-file example programs fail:

| Example | Failure | Root Cause |
|---------|---------|-----------|
| `examples/test_mod/math.ax` | `undefined variable 'make_result'` | External `main.ax` function not registered; codegen emits `@make_result` undefined |
| `examples/benchmark/bench_math.ax` | `unknown type 'BenchResult'` at 937:10 + `getelementptr i64, i64*` invalid | Type from external `main.ax` not visible to codegen |
| `examples/benchmark/main.ax` | 51 type errors — all 24 imported submodules undefined | None of the `use benchmark.<sub>;` declarations resolve |

Secondary defect: `build_module_map_inner` (lib.rs ~743) only registers an external file's
functions if they already exist in `self.functions`, which is impossible for a freshly loaded
file. External functions are silently dropped.

## Goals

1. Implement `ModuleCatalog` + `CachedModule` in `axiom-check` with lazy loading keyed on
   `source_dirs`, supporting multi-segment dotted paths (`benchmark.main` resolves to
   `{source_dir}/benchmark/main.ax`).
2. Register external-module functions unconditionally from the parsed external AST (not gated on
   `self.functions`).
3. Provide `collect_external_decls()` + `CheckedType::to_ast_type()` to inject
   `TopDecl::Type`/`TopDecl::Fn`/`TopDecl::Enum` AST stubs for imported-but-missing decls before
   codegen runs.
4. In `axiomc/main.rs`, automatically add the project `examples` root plus the primary file's
   parent directory to `checker.source_dirs`.
5. Codegen must emit `%struct.BenchResult`, `@make_result`, and module-call thunks
   (`@benchmark_math_run_all`, etc.) for the injected stubs.
6. Fix the `getelementptr i64, i64* … i32 0, i32 0` invalid-indices error that occurs when
   struct types from external modules are treated as `i64`.

## Non-Goals

- Tuple-return codegen — separate pre-existing issue (benchmark_stress `partition()`).
- Removing pre-existing `use of moved value` borrow warnings (146 non-fatal warnings).
- Full-function-body injection — stubs only for types and function declarations to enable
  type-checking and codegen linking. (Bodies come from multi-file merging when all files are
  passed explicitly.)

## Acceptance Criteria

1. `cargo run -p axiomc -- --run examples\test_mod\math.ax` compiles and exits 34.
2. `cargo run -p axiomc -- --run examples\benchmark\bench_math.ax` compiles and exits cleanly.
3. `cargo run -p axiomc -- --run examples\benchmark\main.ax` compiles and runs (aggregates all
   24 submodule scores).
4. `cargo test -p axiom-check` stays at 44/44 passing.
5. `cargo test -p axiom-codegen` stays green (diff 25 + e2e 61 + full_diff 23 + integration 30,
   except resolved v10 flake).
6. No new compiler warnings introduced.

## Verification Commands

```powershell
cd E:\Projects\AXIOM

# Checker tests — must stay 44/44
cargo test -p axiom-check

# Codegen tests — must stay green
cargo test -p axiom-codegen

# Full suite
cargo test

# Multi-file examples — all must compile and run
cargo run -p axiomc -- --run examples\test_mod\math.ax
cargo run -p axiomc -- --run examples\benchmark\bench_math.ax
cargo run -p axiomc -- --run examples\benchmark\main.ax
```
