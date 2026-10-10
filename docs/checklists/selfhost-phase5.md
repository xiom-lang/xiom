<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Selfhost Phase 5 checklist -- codegen bodies: scalars + control flow

Plan: `docs/SELFHOST_PLAN.md` (Phase 5: "Arithmetic, comparisons,
if/while/for/match, calls, returns, strings, floats. Port the Rust
emitter's exact statement order."). Worktree/branch: continue in
`selfhost-phase-4-codegen` (rebase onto current main first). Method:
repro-first under `tmp/sprintc/phase5/`; cargo sequential; `python
tools/ascii_guard.py check` before every commit; no push. Tracker:
`docs/SELFHOST_PROGRESS.md` ("Running the gates" + the gates table).

## Gate

```sh
# T1 stays green over the WHOLE corpus at every stage (stacked)
cargo test -p xiom-codegen --test full_diff_tests

# T2 normalized IR equality -- Phase 5 stages once scalar bodies land
$env:XIOM_SELFHOST_DIFF_TIER=2; cargo test -p xiom-codegen --test full_diff_tests

# T3 exact line-by-line IR equality on the SCALAR corpus = PHASE GATE
$env:XIOM_SELFHOST_DIFF_TIER=3; cargo test -p xiom-codegen --test full_diff_tests

# Skeleton build/run (must stay green; mirrors the harness)
target/debug/xiom.exe -o target/selfhost/xiomc-self.exe selfhost/src/main.xi
target/selfhost/xiomc-self.exe --selfcheck
target/selfhost/xiomc-self.exe examples/diff_test.xi
```

Phase 4's `diff_ir_headers` (T3 header equality over the corpus) must stay
green: Phase 5 only REPLACES stub bodies; header lines are frozen. T3 is
line-by-line over the SCALAR corpus subset (structs/tuples/generics/unsafe
bodies are Phase 6): the meter for this phase is the number of scalar
corpus files whose T3 IR matches byte-for-byte.

## Scalar corpus (S0 output)

Classify the current `corpus()` entries (see `full_diff_tests.rs`:
M37/CATFIX/PHASE1/EXTRA groups) into SCALAR vs PHASE6. Scalar = bodies
that only need literals, locals, arithmetic/comparisons, if/elif/else,
while/for, break/continue, match on scalars/Str, calls, returns, string
and float expressions. Anything needing struct/field lowering, tuple
naming, Option/Result payloads, generic monomorphisation or unsafe
trampolines is PHASE6 and may keep stub bodies (but must keep T1/headers
green). Record the manifest in the Evidence section below; add a corpus
filter only if it does not alter T1/T2/T3 semantics for the full corpus
(e.g. an env read in `corpus()`), otherwise track per-file counts by
hand.

## Staging

| Stage | Scope | State |
|-------|-------|-------|
| S0 | Scalar-corpus manifest + T3 baseline count (expected 0) + first scalar file T3-green end-to-end (proves the tier plumbing: `%tmpN` numbering, label names, ret lowering) | **DONE 2026-10-09** -- manifest below (19 scalar / 64 phase6); T3 baseline 0 -> **2/19** (`examples/diff_test.xi`, `examples/phase1_impl_trait.xi` byte-exact); module prologue + per-function prologue/epilogue + literal returns ported |
| S1 | Expression bodies: literals (`LitInt`/`LitStr`/floats incl. `{:.17e}` exactness), idents/locals, arithmetic, comparisons, unary, casts, tuple-free calls, returns | **IN PROGRESS 2026-10-09** -- int/bool/char literals, local loads/stores, i64/i128 binops + comparisons + div/rem guard, direct calls, `return` epilogue, module-level callable. Green so far: `phase1_impl_trait`, `phase1_modules`, `phase1_async`, `phase1_async_spawn`, `phase1_interface`, `m37_else_if`, `m37_labeled_loops` (+ `diff_test` from S0) = 8/19. Floats (`{:.17e}` formatter) pending; `m37_short_circuit` blocked (see Evidence) |
| S2 | Control flow: if/elif/else, while, for + break/continue, block merging, phi placement, EXACT label names + statement order vs the Rust emitter | **PARTIAL 2026-10-09** -- if/else (nested else-if), tail-if result slot, while + labeled break/continue, assignment statements, text-scan termination, `hoist_static_allocas` post-pass ported (8/19 green above). `for` loops pending |
| S3 | Calls: direct fn calls (declared order, arg coercion), string ops (literals/concat/strcmp), float ops (fcmp/literals), match on scalar/Str scrutinees | TODO |
| S4 | T3 green over ALL scalar-corpus files; T2 green over the same set; spot-check `examples/diff_test.xi` IR byte-equal | **DONE 2026-10-09** -- **T3 19/19** scalar files byte-exact (the whole SCALAR manifest is in `T3_FILES`; the T3 run asserts set equality); T2 runs first inside the same T3 pass over the same set |
| S5 | Tracker/bookkeeping: phase meter + next-phase pointer (Phase 6 -- structs/tuples/generics/unsafe whole-corpus T3) | **DONE 2026-10-09** -- SELFHOST_PROGRESS phase-5 status updated (S0-S4, T3 19/19, meter 55% = 6 of 11), next phase pointer set to Phase 6 |

## Rules of engagement

* Read the Rust emitter first (`crates/xiom-codegen/src/expr.rs`,
  `stmt.rs`, `coerce.rs`): the exact spelling/order of every body line is
  the contract; do not invent formatting, temp numbering or label names.
* Keep every corpus file emitting VALID IR at each stage (T1 runs the
  whole corpus); only replace a stub body when its file turns T3-green.
* One commit per stage (or per bounded sub-slice) with
  `docs/SELFHOST_PROGRESS.md` + this checklist's Evidence in the SAME
  commit; a stage is DONE only with the gate command + counts + commit.
* Findings (emitter/IR surprises, selfhost gaps) go to
  `docs/COMPILER_BUGS.md` with a minimal repro under `tmp/sprintc/phase5/`.
* Do not regress Phase 1-4 gates: tokens/AST dumps, checker parity and
  `diff_ir_headers` stay green on the rebased tree.
* Rebase onto current main before the first gate run and before each
  stage's final gate run.

## Evidence

### S3/S4 completion (2026-10-09, later same day)

**Meter: T3 19/19 -- the whole SCALAR corpus is byte-exact.** `T3_FILES`
now equals `SCALAR_FILES`; the T3 run asserts the set equality (phase gate)
and T2 normalized equality runs first over the same set inside the same
pass. Added since 13/19: `examples/demo_float.xi`,
`examples/stress_float_matrix.xi`, `tests/regression/m37_f128.xi`,
`tests/regression/m37_float_precision.xi`,
`tests/regression/m37_numeric_policy.xi`,
`tests/regression/m37_bug44_str_deref.xi`.

**Float exactness (resolves the S3 blocker)**: the selfhost now decodes
each float literal with its own correctly-rounded (round-to-nearest-even)
decimal->f64 conversion and emits the exact `{:.17e}` expansion:
- `cg_parse_dec_float`: decimal bignum (most-significant-first digit
  arrays) -> exact binary expansion; P>=0 via repeated /2 bit extraction;
  P<0 via fixed-point doubling of `D/10^q`; guard/sticky round-to-even. All
  intermediates are i64 (Int128 DIVISION AVs on this runtime, see below).
- `cg_fmt_double`: f64 -> integer via doubling, exact decimal via
  bignum x5, round-half-to-even to 18 significant digits.
- Validated against Rust on all 11 corpus literals
  (`tmp/sprintc/phase5/probe_float2.xi`): 11/11 byte-equal.
Float ops: `fadd/fsub/fmul/fdiv/frem`, `fcmp oeq/une/olt/ogt/ole/oge` +
zext, i64<->double/float/fp128 coercion, `as` float casts
(fpext/fptrunc/sitofp/fptosi), fp128 assignment stores (no align suffix,
stmt.rs parity).

**Strings (bug44)**: `NkLitStr` -> `@.strN` private constant globals
(collected during body emission, emitted after the functions in use order,
no dedup) + the `getelementptr [n x i8], ... i64 0, i64 0` handle;
Str `==`/`!=` via `@strcmp` + `icmp eq/ne i32` + zext; `&Str` ref-local
tracking (`NkExprRef` inferred as `&T`), `*p` on an i64-held `&T`
(inttoptr + load), and call-site ref coercion (pointee probe + address
inttoptr + slot-address lvalue path).

**Also in this slice**: unsigned/direct-call arg registry survives the new
float/str paths; `store` alignment parity in `Stmt::Assign`; `i8*`
arguments accepted; `NkLitStr`/`NkExprRef` XIM inference.

### S3 slice (2026-10-09, earlier)

**Meter: 13/19 scalar files T3 byte-exact.** Added since S0:
`examples/phase1_modules.xi`, `examples/phase1_async.xi`,
`examples/phase1_async_spawn.xi`, `examples/phase1_interface.xi`,
`examples/phase1_ownership.xi`, `examples/stress_body_parser.xi`,
`examples/stress_borrow_10level.xi`, `tests/regression/m37_else_if.xi`,
`tests/regression/m37_labeled_loops.xi`, `tests/regression/m37_short_circuit.xi`,
`tests/regression/m37_u128.xi` (plus S0's `diff_test`, `phase1_impl_trait`).

Ported in `selfhost/src/codegen.xi`: statement-level assignments
(`NkAssign`, value-first order), `var`/`let` bodies (value -> alloca ->
store -> local), int/bool/char literals (UNSIGNED u64 bit-pattern printing
via decimal-string arithmetic -- `UInt + ""`/`>`/`/`/`%` misbehave),
local loads (narrow widening), i64/i128 arithmetic/bitwise/shift/comparison
via the one Rust dispatch table, `Shr` -> lshr for unsigned operands
(tracked per-local XIOM type), `as` casts (trunc/sext/zext + the big-literal
-> i128 zext case + the temp-consumption of identity casts), unary
Neg/Not/BitNot, `&&`/`||` short-circuit (logic_rhs/logic_done_false/
logic_done + phi) inlined in the Binary arm, direct receiver-free calls
with registry-driven arg coercion (`&local` args pass the slot address,
ref args emit the pointee probe load), `&local` -> ptrtoint, `*p` deref,
`autoderef_ref_value` for bare `&T` operands of arithmetic, if/else
(nested `else if`, tail-if result slot, `unreachable` merges), while +
labeled `break`/`continue`, text-scan block termination, and the
`hoist_static_allocas` post-pass now feeding stdout.

**FIXED root cause from the S1/S2 blocker**: the "checker refuses to
register `cgb_logicx`" report was actually the PARSER silently DROPPING any
function containing `let not = ...` -- `not` is a reserved token, and the
drop produced `undefined variable` at every call site. With the local
renamed (`lnot`) the &&/|| port registers and `m37_short_circuit.xi` is
GREEN. Repro kept: `tmp/sprintc/phase5/probe_not.xi`
(`fn v() -> Int { let not = 1; return not; }` is dropped from `--dump-ast`
while `notx` compiles). COMPILER_BUGS entry corrected accordingly.

**Deferred (blocked)**: float literals need Rust `{:.17e}` exactness
(`2.50000000000000000e0`), i.e. a decimal->f64 parser + 17-digit scientific
formatter (demo_float, float_matrix, float_precision, f128, numeric_policy);
string literals/@.strN globals + strcmp comparison + i64-held ref-local
derefs: bug44.

**Float slice evidence (2026-10-09)**: exact `{:.17e}` formatter is built
and validated in `tmp/sprintc/phase5/probe_float_fmt.xi` (f64 -> integer
via doubling, exact decimal via bignum x5, round-half-to-even); it matches
10/11 corpus literals. BLOCKER: `xiom.core.to_float_from_str` (and a manual
accumulation parse) are 1 ULP OFF on `0.123456789` -- the selfhost would
print `1.23456789000000011e-1` while Rust prints the correctly-rounded
`1.23456788999999997e-1` (float_precision's headline literal). Needs a
correctly-rounded decimal->f64 parser (round-to-nearest-even) before the
formatter can be ported; then the float binop/cast lowering (expr.rs
1848-1982, 5022-5131: fadd/fsub/fmul/fdiv/frem, fcmp oeq/une/olt/ogt/ole/
oge, As i64<->float, fpext/fptrunc fp128 paths).

Findings filed this slice (COMPILER_BUGS 2026-10-09): `str_contains` AV,
Vec-by-value move leaves the caller's binding dangling (use-after-move not
diagnosed), `let not` silently drops the enclosing function (parser),
Bool/UInt `+ ""` string conversion AVs/garbage (Bool concat AV;
`UInt + ""` prints signed).

### S1/S2 slices (2026-10-09, same day as S0)

**Meter: 8/19 scalar files T3 byte-exact.** Added since S0:
`examples/phase1_modules.xi`, `examples/phase1_async.xi`,
`examples/phase1_async_spawn.xi`, `examples/phase1_interface.xi`,
`tests/regression/m37_else_if.xi`, `tests/regression/m37_labeled_loops.xi`
(plus S0's `diff_test`, `phase1_impl_trait`).

Ported in `selfhost/src/codegen.xi`: statement-level assignments
(`NkAssign`, value-first order), `var`/`let` bodies (value -> alloca ->
store -> local), int/bool/char literals, local loads (narrow widening),
i64/i128 arithmetic/bitwise/shift/comparison via the one Rust dispatch
table, `&&`-style div/rem zero trap with Rust's temp-allocation order
(operation temp BEFORE the guard temps), direct receiver-free calls via the
emission registry (key/symbol/ret recorded per emitted function),
if/else (nested `else if`, tail-if result slot, `unreachable` merges),
while + labeled `break`/`continue` (loop stack, dead `after_*` blocks),
text-scan block termination, and the `hoist_static_allocas` post-pass
(cyclic-label DFS + insert-after-entry) now feeding stdout.

**Known blocker (`m37_short_circuit.xi`)**: the `&&`/`||` lowering itself is
written (`cgb_logicx`, expr.rs 1410-1474 port: logic_rhs /
logic_done_false / logic_done + phi) but the checker REFUSES to register
the function -- `error[T001]: catalog body [selfhost_codegen] ...:
undefined variable 'cgb_logicx'` while every identically-shaped
`fn cgb_*` in the same module registers. The call site is reverted to the
binop fallback so the build stays green; the function stays in place,
unused, as the repro. Do not re-add the call until the registration bug is
fixed. Repro: `tmp/sprintc/phase5/` (short_circuit + cgb_logicx).

**Deferred (blocked)**: float literals need Rust `{:.17e}` exactness
(`2.50000000000000000e0`), i.e. a decimal->f64 parser + 17-digit scientific
formatter (demo_float, float_matrix, float_precision, f128, numeric_policy);
refs/borrows (`&Int` autoderef, ptrtoint/inttoptr, `*p`): ownership,
borrow_10level, bug44; i128 literal printing mismatch: u128; `for` loops
and string-expr bodies: body_parser.

Findings filed this slice (COMPILER_BUGS 2026-10-09): `str_contains` AV,
Vec-by-value move leaves the caller's binding dangling (use-after-move not
diagnosed), checker not registering `cgb_logicx`.

### S0 (2026-10-09)

**Gates** (`cargo` sequential in this worktree, branch rebased onto main
`08877f99` before the run):

* `cargo test -p xiom-codegen --test full_diff_tests diff_corpus` (T1,
  default tier): 1 passed, 0 failed -- 156.7 s; T1 over all 83 corpus files.
* `$env:XIOM_SELFHOST_DIFF_TIER=3; cargo test -p xiom-codegen --test
  full_diff_tests diff_corpus`: 1 passed, 0 failed -- 161.8 s;
  **meter 2/19 scalar files byte-exact** (T3 baseline at S0 start: 0).
  T3-GREEN: `examples/diff_test.xi`, `examples/phase1_impl_trait.xi`.
* `diff_ir_headers` (Phase 4) stays green on the same tree (full suite run
  recorded in the commit message / SELFHOST_PROGRESS).

**Harness scoping** (`crates/xiom-codegen/tests/full_diff_tests.rs`): T1 is
unchanged over the WHOLE corpus. The T2/T3 exact comparisons now run over
the PORTED set (`T3_FILES`, a declared subset of the `SCALAR_FILES`
manifest) so the T3 phase gate is "ported set == scalar corpus" while every
intermediate stage keeps the T3 command green. Unported scalar files report
as `T3-PENDING` in the meter line; a `T3_FILES !subset SCALAR_FILES` entry
is a hard assert. This preserves the checklist's T2/T3 definitions ("T3 is
line-by-line over the SCALAR corpus subset") without touching T1 semantics.

**SCALAR manifest** (19 files; `T3` = byte-exact as of this commit):

| File | T3 |
|------|----|
| `examples/demo_float.xi` | pending |
| `examples/diff_test.xi` | **GREEN** |
| `examples/phase1_async.xi` | pending |
| `examples/phase1_async_spawn.xi` | pending |
| `examples/phase1_impl_trait.xi` | **GREEN** |
| `examples/phase1_interface.xi` | pending |
| `examples/phase1_modules.xi` | pending |
| `examples/phase1_ownership.xi` | pending |
| `examples/stress_body_parser.xi` | pending |
| `examples/stress_borrow_10level.xi` | pending |
| `examples/stress_float_matrix.xi` | pending |
| `tests/regression/m37_bug44_str_deref.xi` | pending |
| `tests/regression/m37_else_if.xi` | pending |
| `tests/regression/m37_f128.xi` | pending |
| `tests/regression/m37_float_precision.xi` | pending |
| `tests/regression/m37_labeled_loops.xi` | pending |
| `tests/regression/m37_numeric_policy.xi` | pending |
| `tests/regression/m37_short_circuit.xi` | pending |
| `tests/regression/m37_u128.xi` | pending |

**PHASE6 manifest** (64 files; marker = the first classification reason):

* body struct/field/GEP/aggregate use (`struct`/`gep`/`agg`):
  `examples/catfix/b9mod.xi`, `examples/catfix/main.xi`,
  `examples/catfix/vecmod.xi`, `examples/phase1_contracts.xi`,
  `examples/phase1_derive.xi`, `examples/phase1_derive_enum.xi`,
  `examples/phase1_error.xi`, `examples/phase1_full.xi`,
  `examples/phase1_selfhost.xi`, `examples/phase1_stress.xi`,
  `examples/stress_derive_50field.xi`, `tests/regression/m33_z14.xi`,
  `tests/regression/m37_bug43_result_f64_payload.xi`,
  `tests/regression/m37_bug45_iface_method_generic.xi`,
  `tests/regression/m37_bug46_generic_struct_ref.xi`,
  `tests/regression/m37_bug47_ref_params_leak.xi`,
  `tests/regression/m37_bug48_associated_generic_vec.xi`,
  `tests/regression/m37_bug49_fn_param_impl_collision.xi`,
  `tests/regression/m37_bug50_ptr_container_name.xi`,
  `tests/regression/m37_bug51_option_struct_payload.xi`,
  `tests/regression/m37_bug52_map_enum_values.xi`,
  `tests/regression/m37_bug53_array_ref_param.xi`,
  `tests/regression/m37_bug55_payload_loop.xi`,
  `tests/regression/m37_bug55_unsafe_ptr_capture.xi`,
  `tests/regression/m37_bug56_ensure_expr_body.xi`,
  `tests/regression/m37_catalog_boundary.xi`,
  `tests/regression/m37_catmod.xi`,
  `tests/regression/m37_debug_intrinsics.xi`,
  `tests/regression/m37_from_bytes_fn.xi`,
  `tests/regression/m37_global_field_write.xi`,
  `tests/regression/m37_global_fn_init.xi`,
  `tests/regression/m37_gzip_roundtrip.xi`,
  `tests/regression/m37_index_arith.xi`,
  `tests/regression/m37_inline_call_concat.xi`,
  `tests/regression/m37_loop_capture.xi`,
  `tests/regression/m37_match_float_payload.xi`,
  `tests/regression/m37_nan_ieee.xi`,
  `tests/regression/m37_nested_vec.xi`,
  `tests/regression/m37_opt_payload_value.xi`,
  `tests/regression/m37_payload_ref.xi`,
  `tests/regression/m37_ptr_cast.xi`,
  `tests/regression/m37_ref_mut.xi`,
  `tests/regression/m37_round6_path_gzip.xi`,
  `tests/regression/m37_round7_vec_pop_slot.xi`,
  `tests/regression/m37_shr_builtin.xi`,
  `tests/regression/m37_simd_runtime.xi`,
  `tests/regression/m37_str_int_concat.xi`,
  `tests/regression/m37_structural_eq.xi`,
  `tests/regression/m37_tuple_struct.xi`,
  `tests/regression/m37_vec_f64.xi`,
  `tests/regression/m34_d01.xi`,
  `stdlib/tests/smoke/smoke_guard_fault.xi`,
  `examples/catfix/b9main.xi`
* generic monomorphisation (`genfn` + `name_Concrete` defines):
  `examples/benchmark_selfhost.xi`, `examples/phase1_generics.xi`,
  `examples/phase1_hardening.xi`, `examples/stress_generic_5chain.xi`,
  `tests/regression/m37_bug53_array_ref_write.xi`
* module-level struct/enum type defs / catalog-module emission:
  `examples/phase1_enum.xi` (TokenKind), `examples/catfix/circ_a.xi`,
  `examples/catfix/circ_b.xi`, `examples/catfix/circ_main.xi`
* contract machinery (`result`/`@pre` GEPs):
  `tests/regression/m37_contract_pass.xi`
* const-array GEPs: `tests/regression/m37_const_array.xi`

**Classification method** (reproducible; script kept in the ignored
`tmp/sprintc/phase5/classify_final.ps1`): scan every corpus file's Rust
`--emit-ir` output for body-level Phase 6 markers (`%struct.*`, non-string
`getelementptr`, `extractvalue`/`insertvalue`), calls outside the file plus
the scalar runtime allow-list, defines not owned by the primary AST
(`__unsafe_block*`, `__fnwrap*`, `__xiom_ginit*`, `Option__*`/`Result__*`
methods, monomorphised symbols, dotted imported-module symbols),
module-level `%struct.*` defs beyond the 7 builtins, and generic fn
declarations with bodies. Zero markers = SCALAR.

**Ported code** (S0):

* `selfhost/src/codegen.xi`: module prologue (blank ladder, target
  triple/datalayout, builtin struct defs merged sorted with tuples, final
  CLI newline) + per-function `compile_fn` prologue (entry label, entry-main
  `xiom_set_args`, recursion depth guard with exact `%tmpN`/label
  numbering), param allocas, return-literal lowering, depth-decrement
  epilogue, implicit return; unsupported constructs fall back to the legacy
  stub so T1 stays valid on every corpus file.
* `selfhost/src/codegen_declares.xi`: `@xiom_recursion_counter` global line
  + the trailing blank after the builtin declare table.
* `selfhost/src/runtime_ffi.xi`: `IrBuffer.raw_pre` (verbatim merge of the
  per-function scratch buffer).
* Findings filed in `docs/COMPILER_BUGS.md` (2026-10-09): top-level
  const-array `.len()` miscompile (`tmp/sprintc/phase5/probe_arr_len.xi`)
  and `&T`-param methods rejecting bare receiver-field access
  (`tmp/sprintc/phase5/probe_buf_field.xi`).

