<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Selfhost Phase 0 -- Foundations (checklist)

**Gate:** T1 harness green on the corpus; `runtime_ffi.xi` behavior-tested
against the C helpers; `xiomc_v050.xi` archived.

**Status: LANDED 2026-09-29.** Evidence below. Corrections to the original
manifest are recorded inline (the plan predates the current checkout layout).

## Harness (crates/xiom-codegen/tests/full_diff_tests.rs)
- [x] Corpus manifest, hardcoded + sorted (no runtime globbing):
      `tests/regression/m33_z14`, `m34_d01`, `m37_*` (49 files),
      `examples/diff_test.xi`, `examples/catfix/*.xi` (7),
      `examples/phase1_*.xi` (16, the real sources behind the old
      `selfhost/_diff_phase1_*.xi` names), the v10-era extras
      (`demo_float`, `stress_*`, `benchmark_selfhost`, `stress_body_parser`),
      and `stdlib/tests/smoke/smoke_guard_fault.xi`.
      * Correction: the checklist's `examples/stdlib_smoke/` never existed;
        the fixture lives in the stdlib checkout.
      * Correction: `selfhost/_diff_phase1_*.xi` were stale v10 temp copies
        (9.5 KB each, same body as `xiomc_v10.xi`) -> deleted, not corpus.
- [x] T2 tier: normalized compare (strip `%tmp\d+` / `@.str\d+`).
- [x] T3 tier: exact line-by-line equality (`rust_ir == self_ir`).
- [x] Gate on T1 first; T2/T3 behind `XIOM_SELFHOST_DIFF_TIER=2|3`
      (default 1; tiers stack).
- [x] Deterministic corpus order (hardcoded lists, sorted by path).
- [x] Runner rewrite: `selfhost/src/main.xi` is compiled once per test
      process into `target/selfhost/xiomc-self.exe` and invoked as
      `xiomc-self <source.xi>` (IR on stdout). Replaces the v10
      temp-source-patch runner; no repo-root temp files.

## Module skeleton (selfhost/src/)
- [x] `main.xi` -- driver stub (args -> read file -> lexer -> parser ->
      checker -> codegen; `--selfcheck` mode).
- [x] `lexer.xi` -- Phase 1 stub (`lex_count`).
- [x] `parser.xi`, `checker.xi` -- stubs (`parse_count`, `check_count`).
- [x] `codegen.xi` -- emits the Phase 0 stub IR module (header + `@main`).
- [x] `selfcheck.xi` -- `--selfcheck` behavior assertions.
- [x] Skeleton compiles with `xiom.exe` and runs.

## runtime_ffi.xi (pure-XIOM ports of the v10 C helpers)
- [x] `rt_str_len` / `rt_char_at` / `rt_str_slice` (via stdlib string ops;
      `char_at` is a direct codepoint-decoder port including stray
      continuation / malformed-sequence behavior).
- [x] `intern` / `lookup` -- `SymbolTable`, 1-based IDs, content dedupe,
      0 = null (Vec scan; stdlib Map is itself a linear Vec scan).
- [x] `fn_table_*` -- `FnTable` (`add` / `count`).
- [x] `ir_*` -- `IrBuffer` (`rt_ir_open` / `raw` / `text` / `emit`).
- [ ] float formatting `{:.17e}` equivalent -- DEFERRED to the body-emitter
      phase (Phase 4/5), where its exact bytes are gated by T3. Documented
      in the runtime_ffi.xi header (not stubbed with wrong output).
- [x] Behavior tests: `xiomc-self --selfcheck` asserts the helpers against
      the C outputs (stdlib/runtime/xiom_runtime.c), run by the harness
      (`runtime_ffi_selfcheck`).
      * The v10 `batch_test.ps1` A/B runner is obsolete (it patched
        `xiomc_v10.axi` and the C helper path is being retired); the
        selfcheck encodes the C outputs directly.

## Naming constraint (new, 2026-09-29)
- [x] Every exported runtime_ffi free fn is `rt_`-prefixed. Evidence: a user
      module exporting a same-leaf fn as a stdlib fn (`char_at`) poisons
      catalog-body checking of UNRELATED stdlib modules (bogus T001s in
      `[xiom.num]`; the import alone triggers it). Repro:
      `tmp/sprintc/m162_sameleaf_catalog_poison/`. Compiler bug filed for
      the same-name-shadowing batch.

## Archive
- [x] `selfhost/xiomc_v050.xi` -> `selfhost/archive/xiomc_v050.xi`.
- [x] Deleted the 13 stale `selfhost/_diff_*_N.xi` temp artifacts.

## Phase 0 verification (2026-09-29)
- [x] `cargo test -p xiom-codegen --test full_diff_tests`:
      **2 passed** (`diff_corpus` T1 green over 84 files in 49.2 s,
      `runtime_ffi_selfcheck` ok); gate command exactly as planned.
- [x] Skeleton build: `xiom -o target/selfhost/xiomc-self.exe
      selfhost/src/main.xi` -> `compiled`, exit 0.
- [x] `xiomc-self --selfcheck` -> `SELFCHECK OK`, exit 0.
- [x] `xiomc-self <corpus file>` -> well-formed stub IR on stdout, exit 0.
- [ ] Workspace zero-warning claim (original checklist): the tree carries 2
      pre-existing `dead_code` warnings in `xiom-codegen` (not from this
      batch, no src/ touched).

## Findings filed during Phase 0 (compiler backlog)
- **m162**: same-leaf user-module fn poisons catalog-body checking
  (repro + characterization in `tmp/sprintc/m162_sameleaf_catalog_poison/`).
- **m163 FIXED (2026-09-29)**: `Vec[Str]` field element as a `+` operand
  inside a struct METHOD lowered to i64 add + inttoptr (silent wrong
  strings / AV). Fix: `record_receiver_field_vec_elems` registers field Vec
  element types in the method prologue (both branches); locks
  `m163_method_field_vec_concat` (e2e + CI line) + `regress_m163_field_vec_elem_concat`
  (IR). Repro + characterization kept in
  `tmp/sprintc/m163_method_str_accum/` and `tmp/sprintc/m163_single.xi`;
  write-up in `docs/COMPILER_BUGS.md` ("2026-09-29 -- m163 FIXED").
  `IrBuffer` keeps the single Str field for Phase 0 simplicity (restore the
  Vec-of-lines builder in O1).
