# XIOM -- Checklist: Multi-File Module Catalog

**Branch:** `feat/ecosystem`
**Date:** 2026-07-03

> Step-by-step execution checklist. Mark `- [x]` when a step is verified green.

## Wave 1 -- xiom-check catalog

### 1.1 Add to_ast_type on CheckedType
- [x] Add `CheckedType::to_ast_type(&self) -> Type` method in `crates/xiom-check/src/lib.rs`
  that converts back to AST `Type` for codegen consumption. Map primitives to `Type::Named`,
  `Named` to `Type::Named`, `Generic` to `Type::Named`, `Fn` to `Type::Fn`, `Unit` to `Type::Named("()")`.

### 1.2 Add ModuleExport::Enum variant
- [x] Extend `ModuleExport` enum with `Enum { is_pub: bool, decl: Option<EnumDecl> }` variant.

### 1.3 Add ModuleCatalog + CachedModule structs
- [x] Define `CachedModule` storing: `Program`, `HashMap<String, CheckedType>`, `HashMap<String, FnSig>`,
  `HashMap<String, HashMap<String, CheckedType>>` (fields), `HashMap<String, Vec<(String, CheckedType)>>` (variant fields).
- [x] Define `ModuleCatalog` holding: `source_dirs: Vec<String>`, `cache: HashMap<String, CachedModule>`
  (keyed by dotted module path like `"benchmark.main"`).
- [x] Implement `ModuleCatalog::new(source_dirs)`, `add_source_dir`, `find_owned(&mut self, dotted_path: &[String]) -> Option<CachedModule>`.

### 1.4 Multi-segment path loading
- [x] `find_owned` walks `source_dirs` for `<dir>/<p0>/<p1>/.../<pn>.xi` (path-based). Falls back to
  scanning `source_dirs` for any `.xi` file whose declared module matches the dotted path (scan-based
  for files like `test_mod/main.xi` that declare `module benchmark.main` but aren't nested in a
  `benchmark/` directory). Parses once, caches.

### 1.5 Wire catalog into Checker
- [x] Add `catalog: ModuleCatalog` field to `Checker`.
- [x] Add `Checker::add_source_dir(dir)` method that pushes to both `self.source_dirs` and `self.catalog.source_dirs`.
- [x] Add `Checker::register_external_module(cached: &CachedModule)` that:
  - Walks `cached.program.items` calling `self.register_type_decl()` and `self.register_fn_signature()`.
  - Builds the module map via `self.build_module_map(&cached.program.items)`.
  - Calls `self.flatten_submodules(&cached.program.items)`.
  - Inserts the exports into `self.modules`.

### 1.6 Rewrite resolve_imports
- [x] In `resolve_imports`, after building in-program module hierarchy, for each use decl:
  - Walk each path prefix `ud.path[..i]` for `i in 1..=len`.
  - If the last segment isn't in `self.modules`, call `self.catalog.find_owned(&dotted)`.
  - If found, call `self.register_external_module(&cached)`.
  - Dedup via a `self.cached_loaded: HashSet<String>` tracking which dotted paths have been loaded.
- [x] Fix `build_module_map_inner` to register external function signatures from the parsed FnDecl
  AST directly (build FnSig from params/return_type, NOT gated on `self.functions`).

### 1.7 Add collect_external_decls
- [x] Add `Checker::collect_external_decls(program: &Program) -> Vec<TopDecl>`:
  - Walk `self.catalog` cache + `self.imported_items`.
  - For each pub type/function/enum from external modules not already present in `program.items`
    (dedup by name), emit a `TopDecl::Type`/`TopDecl::Fn`/`TopDecl::Enum` stub.
  - Filter primitives: Bool, Int, Int8-Int64, UInt, UInt8-UInt64, Float32/64, Char, Str, Unit, Never,
    Slice, Option, Result, Vec, Map, Set, Ptr, Array, Tuple, Fn.
  - Functions get empty bodies (stubs for codegen to resolve symbols).

### 1.8 Verify checker
- [ ] `cargo test -p xiom-check` -- 44/44.
- [ ] `cargo test -p xiom-codegen` -- 139/139 (post v10 flake fix).

## Wave 1 -- xiom wiring

### 1.9 Auto-add source directories
- [x] Replace direct `checker.source_dirs.push()` in `crates/xiom/src/main.rs` with
  `checker.add_source_dir()` calls for:
  - Parent directory of the primary source file.
  - Project `examples/` root (derived from `CARGO_MANIFEST_DIR` -> parent -> parent -> join("examples")).

### 1.10 Inject external decls before codegen
- [x] After `checker.check_program()`, call `checker.collect_external_decls(&program)`.
- [x] Append returned stubs to `program.items` before `emitter.compile_program(&program)`.
- [x] Keep the `is_multi_file` soft-error gate logic unchanged.

### 1.11 Verify examples
- [ ] `cargo run -p xiom -- --run examples\test_mod\math.xi` -> exit 34.
- [ ] `cargo run -p xiom -- --run examples\benchmark\bench_math.xi` -> compiles and runs.
- [ ] `cargo run -p xiom -- --run examples\benchmark\main.xi` -> compiles and runs.

## Wave 2 -- Codegen visibility

### 2.1 Codegen honors injected stubs
- [ ] Ensure `IrEmitter::compile_program` processes injected `TopDecl::Type` stubs to emit
  `%struct.BenchResult` with correct field types.
- [ ] Ensure injected `TopDecl::Fn` stubs become `declare` entries in LLVM IR so cross-file calls
  (e.g., `@make_result`, `@benchmark_math_run_all`) resolve at link time.
- [ ] Fix the `getelementptr i64, i64* ... i32 0, i32 0` error by ensuring struct types from
  external modules have proper LLVM type tags (not defaulted to `i64`).

### 2.2 Verify codegen
- [ ] `cargo test -p xiom-codegen` -- all 139 pass.
- [ ] `cargo run -p xiom -- --run examples\test_mod\math.xi` -- exit 34.

## Wave 3 -- Stability + cleanup

### 3.1 Isolate v10 selfhost tests
- [x] Unique output filenames: `e2e_v10_self_compile.exe` vs `e2e_v10_self_bootstrap_src.exe`.
  File: `crates/xiom-codegen/tests/e2e_tests.rs`.

### 3.2 Clean compiler warnings (9 total)
- [ ] `xiom-codegen/src/lib.rs`: prefix `fields` (lines 1097, 1299) and `cond`/`elifs` (line 3551) with `_`.
- [ ] `xiom-check/src/lib.rs`: prefix `name` (line 324) and `method_key` (line 590) with `_`.
- [ ] `xiom-check/src/lib.rs`: remove unreachable `_ => return` arms at lines 384 and 392
  (all `DeriveTrait` variants are covered).
- [ ] `xiom-check/src/lib.rs`: wire `load_external_module_path` (remove `#[warn(dead_code)]`),
  OR delete it if unused after Wave 1.
- [ ] `full_diff_tests.rs` line 163: fix `rust_fns >= $min_fns` / `sh_fns >= $min_fns` useless
  comparison warnings (caused by `min_fns = 0` in `diff_error` macro expansion).

### 3.3 Final verification
- [ ] `cargo test` (full workspace) -- green except pre-known tuple-return issue.
- [ ] No warnings from `cargo build` on xiom-check and xiom-codegen.

## Wave 4 -- Regression + docs

### 4.1 E2E regression tests
- [ ] Add test `e2e_testmod_math_runs` in `crates/xiom-codegen/tests/e2e_tests.rs`: compiles
  and runs `examples/test_mod/math.xi`, asserts exit code 34.
- [ ] Add test `e2e_benchmark_bench_math_compiles`: compiles `examples/benchmark/bench_math.xi`
  (assert success, no run requirement yet until linking is solid).
- [ ] Add test `e2e_benchmark_main_compiles` (ignore by default until all 24 submodules link):
  compiles `examples/benchmark/main.xi`.

### 4.2 Update SESSION.md
- [ ] Replace the inaccurate "multi-file catalog built" claim with accurate status reflecting
  what was actually implemented in this session.
- [ ] Update test counts: 44 checker + 139 codegen = 183, with the v10 flake now resolved so
  183/0 instead of the logged 60 passed/1 failed.

### 4.3 Final check
- [ ] `cargo test` -- all green.
