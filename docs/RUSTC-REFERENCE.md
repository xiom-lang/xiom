# Rustc Reference Index

Purpose: the Rust compiler is our primary pattern source. Before implementing
checker, codegen, layout, ABI, or diagnostics work, consult the corresponding
rustc implementation in the clone below -- do not write from memory. Cite the
path (and, where useful, the line) in commit messages and bus notes.

## Clone

- Location: `E:\repos\rust`
- Commit: `7211ea594cbf137ef6118b1752b959dea7cb9088` (2026-10-11, "Auto merge
  of #164132"), cloned with `--depth 1 --filter=blob:none --sparse`.
- Materialized: `compiler/` -- all 80 `rustc_*` crates (~34 MB of source) --
  plus `src/bootstrap` and `src/tools/compiletest` (for the selfhost/infra
  study).
- Extend the sparse set on demand (blobs fetch lazily over the network):
  - `git -C E:\repos\rust sparse-checkout add tests/ui` (golden diagnostic tests)
  - `git -C E:\repos\rust sparse-checkout add src/tools/compiletest`
  - `git -C E:\repos\rust sparse-checkout add library`
  - full tree: `git -C E:\repos\rust sparse-checkout disable`
- Refresh: `git -C E:\repos\rust pull --depth 1` (shallow), or re-clone.

## Rules absorbed from rustc (adopt these in XIOM)

1. **No cascading diagnostics after an error.** Every dependent check is gated
   behind `ErrorGuaranteed`, a token that can only be minted by the diagnostic
   emitter. Our bogus W004 unreachable-arm warnings during unresolved-module
   recovery (REL-20261010-1900; also seen in the C-ORBIT-06 probes) are
   exactly what this rule prevents.
2. **Derive ABI/layout from types, never from names.** rustc computes `FnAbi`,
   `ArgAbi`, and `LayoutS` from `Ty`; nothing round-trips through a printed
   type name. Our tuple-element namer family (m250/m254/m256) and the M19
   erased enum slots are name/string-derived; every fix so far has been a
   patch to a namer.
3. **TLS-ness is a declaration property set once.** A thread-local global gets
   `set_thread_local_mode` at declaration; every access uses the same model.
   (REL-20261010-1854's `movq 0xf8` absolute loads are the mixed-TLS-ness
   symptom.)
4. **Emit only reachable items.** Monomorphization collects reachable,
   instantiated items; unreachable functions are never codegen'd.
5. **Verify IR before handing it to an external tool.** The LLVM backend has
   `-Z verify-llvm-ir`; the cranelift backend verifies every function it
   builds.

## Verified areas -- paths and notes

### 1. Diagnostics and error recovery
- `compiler/rustc_span/src/lib.rs:2858` -- `pub struct ErrorGuaranteed(())`
  ("type cannot be constructed outside of this crate"; "Don't use this outside
  of `DiagCtxtInner::emit_diagnostic`!").
- `compiler/rustc_errors/` -- `DiagCtxt`, delayed bugs; emitting a diagnostic
  returns the token that downstream code requires.
- Adopt for: REL-20261010-1900 (suppress W004 when the scrutinee/type failed
  to resolve), C-ORBIT-06 phantom-import recovery diagnostics.

### 2. Name resolution / unresolved imports
- `compiler/rustc_resolve/src/imports.rs` -- import resolution and
  `ResolutionError` reporting; also `late.rs`, `build_reduced_graph.rs`.
- Adopt for: phantom `use` handling (C-ORBIT-06, adjacent to m255), so an
  unresolved import produces exactly one clear error and no follow-on noise.

### 3. Match lowering and bindings
- `compiler/rustc_mir_build/src/builder/matches/{mod.rs,match_pair.rs,
  buckets.rs,test.rs,user_ty.rs}` -- arm bindings are typed from the
  scrutinee's `Ty`/`Place` projections (`MatchPair`); field identity is
  variant-relative index, never a global field-name lookup.
- `compiler/rustc_mir_build/src/thir/pattern/check_match.rs` -- arm
  reachability/exhaustiveness (the analysis our W004 mirrors).
- Adopt for: the m256 root cause (repeated field names across variants) and
  the M19 field-erasure convention; the correctness anchor is the
  variant-relative field type, which rustc takes from layout, not a name table.

### 4. ABI, fn-pointer calls, float returns
- `compiler/rustc_target/src/callconv/mod.rs:394` --
  `pub struct ArgAbi { layout, mode: PassMode }`;
  `:660` -- `pub struct FnAbi { args: Box<[ArgAbi]>, ret: ArgAbi }`
  ("The layout, type, and the way a value is returned from this function.").
- `compiler/rustc_abi/src/{callconv.rs,extern_abi.rs,canon_abi.rs}` -- ABI
  derivation from types.
- Adopt for: REL-20261010-1904 -- a cast fn-pointer call must take its return
  mode (register, direct, or by-pointer) from the fn-ptr's declared signature;
  the current codegen yields 0/garbage for `Float64` returns.

### 5. Layout of enums and tuples
- `compiler/rustc_abi/src/lib.rs:2032` -- `pub enum Variants<FieldIdx,
  VariantIdx>` (Empty / Single / Multiple { tag, tag_encoding, tag_field, ... }).
- `compiler/rustc_abi/src/layout/ty.rs:76` (`TyAndLayout`) /
  `rustc_abi/src/lib.rs:2223` (`LayoutData`) -- layout is computed, fields
  are indexed per variant. (The historical `LayoutS` name does not exist in
  the 2026-10-11 clone; study 05 verified the successors.)
- `compiler/rustc_public/src/abi.rs` -- stable `LayoutShape`/`VariantsShape`
  shapes (a useful model for any published metadata).
- Adopt for: replacing name-string typing (`Tuple__Int__Vec`) with computed
  layout/type ground truth where feasible; the m250/m254/m256 regression family
  exists because names disagree with types.

### 6. TLS codegen
- `compiler/rustc_codegen_llvm/src/context.rs:99` -- `tls_model:
  llvm::ThreadLocalMode`; `:174` -- `to_llvm_tls_model` (GeneralDynamic,
  LocalDynamic, InitialExec, LocalExec).
- `compiler/rustc_codegen_llvm/src/consts.rs:413,542` --
  `set_thread_local_mode(g, self.tls_model)` with the comment: "complains that
  one object file has a thread local version of the symbol and another one
  doesn't."
- `compiler/rustc_codegen_llvm/src/llvm/mod.rs:224` --
  `set_thread_local_mode`.
- Adopt for: REL-20261010-1854 -- exactly the mixed-TLS-ness failure class;
  set the mode once per declaration and audit every access (our two
  `movq 0xf8` offsets).

### 7. Monomorphization and reachability
- `compiler/rustc_monomorphize/src/collector.rs`, `partitioning.rs` --
  reachable/instantiated items only; unreachable definitions never reach
  codegen.
- Adopt for: REL-20261010-1854 -- an UNUSED function grows our binary 5.6x
  (341 KB -> 1.9 MB) and the extra emission is where the malformed TLS
  accesses appear; rustc's reachability discipline is the fix direction.

### 8. IR verification
- `compiler/rustc_codegen_ssa/src/back/write.rs:102` -- `pub verify_llvm_ir:
  bool` (the `-Z verify-llvm-ir` switch); `compiler/rustc_codegen_llvm/src/
  back/write.rs:62` -- threaded to the LLVM writer.
- `compiler/rustc_codegen_cranelift/src/base.rs:271` --
  `cranelift_codegen::verify_function(func, &flags)` after each function,
  reporting via `tcx.dcx().err(...)`.
- Adopt for: catching m256-class invalid IR before clang does; give XIOM an
  always-on IR verification step (or clang `-fsyntax-only` equivalents) so
  broken IR fails the build with a clear diagnostic.

### 9. Incremental / query architecture (future)
- `compiler/rustc_query_system/`, `compiler/rustc_incremental/`.

### 10. Golden diagnostic tests (fetch on demand)
- `tests/ui/**/*.stderr` -- rustc's diagnostic tests are exact stderr
  snapshots. Add with `git sparse-checkout add tests/ui`, then read examples.
- `src/tools/compiletest` -- the harness.

## Active XIOM items -> reference map

| XIOM item / bug family | Rustc reference | Pattern to adopt |
|---|---|---|
| REL-20261010-1900 W004 cascades | S1 ErrorGuaranteed; S3 check_match | suppress reachability warn when resolution failed |
| REL-20261010-1904 fp cast fn-ptr returns | S4 FnAbi/ArgAbi | return mode from the fn-ptr signature |
| REL-20261010-1854 TLS offset-as-address + bloat | S6 TLS; S7 mono reachability | one TLS mode per decl; emit reachable items only |
| m250/m254/m256 tuple-element typing | S5 layout; S3 matches | type from layout/variant index, not names |
| M19 repeated-field erasure | S5 Variants | variant-relative field identity |
| C-ORBIT-06 phantom imports / m255 | S2 resolve; S1 recovery | one clear error, no cascades |
| Invalid-IR classes (m256, ...) | S8 verify | verify IR before clang |
