<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Release gate: v0.65.6 (PENDING OWNER GO)

**Status:** gate defined; version bump, stdlib tag/pin, and tag are NOT cut.
No commit/push/tag has been made for this set.

## What v0.65.6 carries

- **P0 orbitdb**: Vec overwrite-drop reads the current header (`1df9bd5af`);
  `p_vec_rebind_grow` exits 0, `vec_rebind_growth` gate PASS.
- **P1 t3**: POSIX confinement trampoline uses `sigsetjmp(..., 0)` and the
  handler restores the delivered signal mask. Scored t3: 27 ms -> 5 ms
  internal (Rust 6 ms). Locks: `e2e_t3_trampoline_floor_budget`,
  `e2e_confined_repeat_fault_contained`, fixtures under `tests/regression/`.
- **P2 JSON 1704**: R1-R4 reclaim the parse tree and builder map. Peak:
  4.6/5.0/5.5 MB at 1k/10k/20k (~0.05 KB/iter), provenance-strict.
- **Stdlib blocker REL-20261103-1626 FIXED**: a `#[owns_return]` Result bound
  in a nested block now drops via an entry-hoisted pointer slot, so the exit
  GEPs dominate the binding alloca. The nested smoke compiles/runs;
  `stdlib_exec_serialize_json_nested_runs` green; 1704 remains flat at
  5.5 MB (10k/20k); m375 26/26; memory gates 22/22. Compiler-side readiness
  is complete; the stdlib 10/10 pin waits on the v0.65.6 archive.
- **P2 2117**: unique-owned aggregate field/whole overwrite reclaim
  (5 MB at 20k); trace-ring/alias negatives stay leak-only.
- **P4**: tier-3 lint is OPT-IN (`--lint-strict`), corpus noise 47 -> 0;
  default-on is a separate owner decision.
- **t8/t9**: `--sanitize=address` now tags generated IR with
  `sanitize_address` and bypasses the guard arena; t9 LSI 45 -> 50 when the
  benchmark lane wires `getSanitizedCompileCmd`.
- **Checker**: `let` immutability (init-once: a deferred `let x: T;` may be
  initialized once; initialized `let`/`const`/field stores are rejected),
  plus the spec-gap locks in `checker_spec_gaps` (all seven enforced,
  0 ignored; see Gap tracker).
- **Relay closed**: 1904/1055/1121 already-green; 1553 fixed; B-05 fixed;
  macOS typed memset fixed + IR lock; C-PULSE runtime half fixed and the
  loop-body drop-glue half reclaimed; server parse buffers freed;
  website-8 (a)(c) fixed.
- **Selfhost**: T6 37/64 (non-phase-8 work complete). Not a release blocker;
  no bootstrap binary ships.

## Preconditions (must be true BEFORE the tag)

- [ ] Owner GO for the tag (hard stop).
- [ ] stdlib tag for this set (t3 runtime, JSON `#[owns_return]` /
      `#[consumes_agg]` / `#[borrows_str]` / `#[owns_str]`, `sb_to_str`
      `#[borrows_vec]`, server parse-buffer free) and `STDLIB_VERSION` bump.
- [ ] workspace `Cargo.toml` version = 0.65.6; `selfhost` `SELFHOST_VERSION`
      parity if the selfhost binary is in scope (it is not).
- [ ] `release-notes/v0.65.6.md` + `.json` via convert/verify with the
      pinned stdlib checkout.
- [ ] `git status` clean apart from intended release files; no stray probe
      dirs in the commit.

## Gate A -- compiler suites

Run at the release commit; expected counts are the current green baseline.

- [ ] `cargo test -p xiom-check` (219+)
- [ ] `cargo test -p xiom --test checker_locks` (60+)
- [ ] `cargo test -p xiom --test checker_spec_gaps` (26 active, 0 ignored,
      0 failed) and `... -- --ignored` (0 tests) -- see Gap tracker
- [ ] `cargo test -p xiom-codegen --test e2e_tests ctfe` (4/4; the three
      CTFE fixtures + comprehensive)`
- [ ] `cargo test -p xiom --test borrow_e001`, `doctor_cli`,
      `run_script_cli`, `cxx_standard_cli`, `ffi_fnptr_abi`,
      `ffi_ptr_arith`, `ffi_ptr_slot_readback`, `ffi_alloc_guard_free`
- [ ] `cargo test -p xiom-codegen --test feature_regression_tests`
      (582/582, `RUST_MIN_STACK=64MB`)
- [ ] e2e suites: drop/container/loop/nested_struct/R3/R4/2117 families,
      plus the full e2e run (pre-existing `examples/benchmark` T001
      failures are the only known exceptions)
- [ ] `tests/soak/run_memory_gates.ps1` 22/22 GREEN
- [ ] t3 probes: `e2e_t3_trampoline_floor_budget`,
      `e2e_confined_repeat_fault_contained`, `run_t3` scored median
- [ ] t9 ASan: `--sanitize=address --no-contracts --overflow-checks`
      with `ASAN_OPTIONS=detect_leaks=0`; UAF probe rejects, `json_escape("")`
      does not bad-free
- [ ] selfhost: `diff_ast` 83/83, `diff_corpus` T3 19/19, T6 >= 37, full
      tier-3 suite green

## Gap tracker (must stay visible in CI) -- ENFORCED, 0 IGNORED (2026-11-03)

`crates/xiom/tests/checker_spec_gaps.rs` contains **0 `#[ignore]`**. All seven
2026-11-03 audit gaps are hard diagnostics, each with a positive control that
pins the still-legal shape. Evidence:

```text
cargo test -p xiom --test checker_spec_gaps
  -> 26 passed; 0 failed; 0 ignored
cargo test -p xiom --test checker_spec_gaps -- --ignored
  -> 0 tests
cargo test -p xiom --test checker_locks
  -> 57 passed; 0 failed
cargo test -p xiom-check
  -> 219 passed; 0 failed
```

| # | Rule (hard error) | Implementation (`crates/xiom-check/src/lib.rs`) | Locks |
|---|-------------------|--------------------------------------------------|-------|
| 1 | `break`/`continue` outside a loop; unknown loop label | `check_loop_control` :7894; `While`/`For` depth+labels :8206/:8233; `Break`/`Continue` :8347; borrow-pass mirror :12695/:12839/:12853/:12886 | negative `break_outside_loop`, `continue_outside_loop`, `unknown_loop_label`; positive `loop_control_inside_loops_is_accepted` |
| 2 | duplicate function parameter names (receiver `self` included) | `check_fn_decl` :6682 (also receiver-first collision) | negative `duplicate_parameter_name_is_rejected`, `duplicate_receiver_name_is_rejected`; positive `var_params_and_for_bindings_stay_mutable` |
| 3 | duplicate struct-literal field names (spread base excluded) | `Expr::Struct` :11105 | negative `duplicate_struct_literal_field_is_rejected`; positive `in_range_literals_and_explicit_casts_are_accepted` (struct field stores) |
| 4 | duplicate pattern bindings; `|` alternatives may repeat names only with an identical bound set | `check_pattern_duplicate_bindings` :1837, `collect_pattern_bindings` :1857 | negative `duplicate_pattern_binding_is_rejected`, `or_pattern_with_different_bindings_is_rejected`; positive `legal_or_patterns_are_accepted` |
| 5 | `const` initializer must be **CTFE-evaluable** (SPEC 2.6): literals, const refs, unary/binary, aggregates, `if`/`match`, user-function and builtin calls with const args; rejects extern/FFI, method calls, runtime I/O, `unsafe`, mutable-global reads | `is_const_expr` :2827, `is_const_expr_with_names` :2834, `is_const_block_with_names` :2954, `collect_const_match_bindings` :2988, `is_ctfe_callable` :3016, diagnostic :3888 | negative `non_constant_const_initializer_is_rejected` (extern/FFI), `non_constant_const_identifier_is_rejected` (mutable global); positive `constant_expression_consts_are_accepted`, `ctfe_call_if_match_and_builtin_consts_are_accepted` |
| 6 | out-of-range fixed-width integer literal (let/var/assign/arg/field/return/const) | `fixed_width_int_bounds` :8485, `check_int_literal_range` :8513 (call sites at the let/var, assign, return, struct-field, call-arg arms) | negative `out_of_range_{int_literal,negative,unsigned,call_argument,struct_field,return,const_initializer}_...`; positive `in_range_literals_and_explicit_casts_are_accepted` |
| 7 | `let`/`const`/field immutability init-once | pre-existing m396 enforcement (unchanged) | `deferred_let_init_once_stays_legal` + `checker_locks` m396 group |

Selfhost parity: the same rules live in `selfhost/src/check_state.xi`
(loop state + `ck_check_loop_control`), `check_expr.xi` (loop enter/exit,
pattern duplicates, struct-field duplicates, `ce_is_const_expr`,
`ce_check_int_literal_range`), and `check_core.xi` (parameter duplicates,
const pre-check). The `selfhost_spec_gap_locks_fail_hard` harness test proves
every invalid fixture exits 4 with the same message family and every accept
fixture exits 0; `diff_check` is byte-exact over the 83-file corpus + 101
manifest cases.

CTFE parity (RESOLVED 2026-11-03): the const predicate mirrors
`IrEmitter::evaluate_const_init`. The three previously-blocked fixtures are
green in both drivers:

```text
cargo test -p xiom-codegen --test e2e_tests ctfe
  -> 4 passed (phase_a, phase_b, builtins_struct, comprehensive)
xiom --check tests/regression/ctfe_phase_a.xi        -> PASSED
xiom --check tests/regression/ctfe_phase_b.xi        -> PASSED
xiom --check tests/regression/ctfe_builtins_struct.xi -> PASSED
selfhost --dump-check on all three                   -> CHECK-OK
```

The negative const lock is now a genuinely non-CTFE initializer (an
`unsafe { extern_fn() }` call); a mutable-global read is the second lock. The
worktree's `m266_unresolved_import_one_clear_error` failure is a worktree-only
baseline gap: the worktree still carries the pre-3fbf32500 fixture (main
updated it in `test(checker): point m266 at a permanently missing module`).
Main's `checker_locks` is 60/60; the worktree is 36/37 with the stale fixture.

Spec-silent decisions (deliberate, no corpus waiver):

- Bare `Int`/`UInt` are not range-checked; only the width-qualified
  `Int8/16/32/64` and `UInt8/16/32/64` spellings are. Explicit `as` casts are
  the wrapping form and stay legal (`300 as Int8` is accepted; the pre-existing
  W000 cast-truncation warning may still fire).
- Qualified `module.CONST` reads are not accepted as constant expressions
  (only bare `const` names, literals, and `Enum.Variant`); no corpus or stdlib
  const initializer uses a qualified const, so this is a strictness decision,
  not a burn-down waiver.
- Selfhost stores 64-bit integer literals in an `i64`; unsigned hex/binary
  `UInt64` bit patterns with the top bit set are accepted (Rust keeps them as
  `u64`), while explicit unary-minus forms are range-checked.

Defined-by-current-semantics, NOT gaps: uninitialized `let`/`var` reads
return 0, and a non-unit function that falls through returns the zero value.

## Post-release verification

- orbitdb: `scripts/hybrid_crash_test.ps1` on the new toolchain (REL-1716).
- benchmark: t3 re-measure (REL-1554), t9 sanitized wiring
  (REL-20261102-1929), t8 raw unchanged.
- engine: 120/300/600 soak on the pinned toolchain (C-PULSE + 2117).
- relay: close fixed/verified items; re-run the stale-acceptance threads.
