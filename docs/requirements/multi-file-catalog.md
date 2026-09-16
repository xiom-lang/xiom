<!-- Copyright (c) 2026 Eleftherios Notas and XIOM Foundation -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM -- Requirements: Multi-File Module Catalog

**Date:** 2026-07-03
**Branch:** `feat/ecosystem`
**Status:** In Progress (Red baseline established)

## Background

The session document (`SESSION.md`) describes a `ModuleCatalog` architecture with `CachedModule`,
`register_all_types_into`, `collect_external_decls`, and `CheckedType::to_ast_type` -- none of
these exist in the codebase (verified via grep of `crates/`). The actual module resolution in
`crates/xiom-check/src/lib.rs` uses only `load_external_module` (single-segment, looks up
`{source_dir}/{name}.xi`) and the nested-path variant `load_external_module_path` is dead code
(Rust `#[warn(dead_code)]` fires at lib.rs:715).

As a result, three multi-file example programs fail:

| Example | Failure | Root Cause |
|---------|---------|-----------|
| `examples/test_mod/math.xi` | `undefined variable 'make_result'` | External `main.xi` function not registered; codegen emits `@make_result` undefined |
| `examples/benchmark/bench_math.xi` | `unknown type 'BenchResult'` at 937:10 + `getelementptr i64, i64*` invalid | Type from external `main.xi` not visible to codegen |
| `examples/benchmark/main.xi` | 51 type errors -- all 24 imported submodules undefined | None of the `use benchmark.<sub>;` declarations resolve |

Secondary defect: `build_module_map_inner` (lib.rs ~743) only registers an external file's
functions if they already exist in `self.functions`, which is impossible for a freshly loaded
file. External functions are silently dropped.

## Goals

1. Implement `ModuleCatalog` + `CachedModule` in `xiom-check` with lazy loading keyed on
   `source_dirs`, supporting multi-segment dotted paths (`benchmark.main` resolves to
   `{source_dir}/benchmark/main.xi`).
2. Register external-module functions unconditionally from the parsed external AST (not gated on
   `self.functions`).
3. Provide `collect_external_decls()` + `CheckedType::to_ast_type()` to inject
   `TopDecl::Type`/`TopDecl::Fn`/`TopDecl::Enum` AST stubs for imported-but-missing decls before
   codegen runs.
4. In `xiom/main.rs`, automatically add the project `examples` root plus the primary file's
   parent directory to `checker.source_dirs`.
5. Codegen must emit `%struct.BenchResult`, `@make_result`, and module-call thunks
   (`@benchmark_math_run_all`, etc.) for the injected stubs.
6. Fix the `getelementptr i64, i64* ... i32 0, i32 0` invalid-indices error that occurs when
   struct types from external modules are treated as `i64`.

## Non-Goals

- Tuple-return codegen -- separate pre-existing issue (benchmark_stress `partition()`).
- Removing pre-existing `use of moved value` borrow warnings (146 non-fatal warnings).
- Full-function-body injection -- stubs only for types and function declarations to enable
  type-checking and codegen linking. (Bodies come from multi-file merging when all files are
  passed explicitly.)

## Acceptance Criteria

1. `cargo run -p xiom -- --run examples\test_mod\math.xi` compiles and exits 34.
2. `cargo run -p xiom -- --run examples\benchmark\bench_math.xi` compiles and exits cleanly.
3. `cargo run -p xiom -- --run examples\benchmark\main.xi` compiles and runs (aggregates all
   24 submodule scores).
4. `cargo test -p xiom-check` stays at 44/44 passing.
5. `cargo test -p xiom-codegen` stays green (diff 25 + e2e 61 + full_diff 23 + integration 30,
   except resolved v10 flake).
6. No new compiler warnings introduced.

## Verification Commands

```powershell
cd E:\Projects\XIOM

# Checker tests -- must stay 44/44
cargo test -p xiom-check

# Codegen tests -- must stay green
cargo test -p xiom-codegen

# Full suite
cargo test

# Multi-file examples -- all must compile and run
cargo run -p xiom -- --run examples\test_mod\math.xi
cargo run -p xiom -- --run examples\benchmark\bench_math.xi
cargo run -p xiom -- --run examples\benchmark\main.xi
```
