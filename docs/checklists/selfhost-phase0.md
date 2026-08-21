# Selfhost Phase 0 -- Foundations (checklist)

**Gate:** T1 harness green on the corpus; `runtime_ffi.xi` behavior-tested
against the C helpers; `xiomc_v050.xi` archived.

## Harness (crates/xiom-codegen/tests/full_diff_tests.rs)
- [ ] Add corpus manifest: `tests/regression/*.xi` (subset: m33_z14, m34_d01,
      m37_*), `examples/diff_test.xi`, `examples/catfix/*.xi`,
      `examples/stdlib_smoke/smoke_guard_fault.xi`, `selfhost/_diff_phase1_*.xi`.
- [ ] Add T2 tier: normalized compare (strip `%tmp\d+` and `@\.str\d+`).
- [ ] Add T3 tier: exact line-by-line equality (`rust_ir == self_ir`).
- [ ] Gate harness on T1 first; T2/T3 behind a flag until the target phase.
- [ ] Deterministic corpus order (sorted; no filesystem-order flakiness).

## Module skeleton (selfhost/src/)
- [ ] `selfhost/src/main.xi` -- driver stub (reads args, calls stages).
- [ ] `selfhost/src/lexer.xi` -- stub returning the token stream (Phase 1).
- [ ] `selfhost/src/parser.xi`, `checker.xi`, `codegen.xi` -- stubs.
- [ ] Compile the skeleton with `xiom.exe`; confirm it builds and runs.

## runtime_ffi.xi (pure-XIOM ports of the v10 C helpers)
- [ ] `str_len` / `char_at` / `str_slice` (via stdlib string ops).
- [ ] `intern` / `lookup` -- symbol table using stdlib `Map[Str, Int]`.
- [ ] `fn_table_*` -- fn registry using stdlib `Vec`/`Map`.
- [ ] `ir_*` (open/header/raw/close/emit_program) -- file I/O + string build.
- [ ] float formatting `{:.17e}` equivalent (matches BUG 10 emission).
- [ ] Behavior tests: port `selfhost/batch_test.ps1` to run each helper
      against known inputs; assert identical output to the C versions
      (temporarily keep the C versions for A/B in Phase 0 only).

## Archive
- [ ] Move `selfhost/xiomc_v050.xi` -> `selfhost/archive/` (keep for reference).
- [ ] Delete stale `_diff_*_N.xi` temp artifacts from previous runs.

## Phase 0 verification
- [ ] `cargo test -p xiom-codegen --test full_diff_tests` -> T1 green on corpus.
- [ ] `cargo build --workspace` and `cargo test --workspace --no-run` -> 0 warnings.
- [ ] Fast suite unchanged: 1112/1/1 (documented diff-test ignore only).
