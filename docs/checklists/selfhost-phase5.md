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
| S0 | Scalar-corpus manifest + T3 baseline count (expected 0) + first scalar file T3-green end-to-end (proves the tier plumbing: `%tmpN` numbering, label names, ret lowering) | TODO |
| S1 | Expression bodies: literals (`LitInt`/`LitStr`/floats incl. `{:.17e}` exactness), idents/locals, arithmetic, comparisons, unary, casts, tuple-free calls, returns | TODO |
| S2 | Control flow: if/elif/else, while, for + break/continue, block merging, phi placement, EXACT label names + statement order vs the Rust emitter | TODO |
| S3 | Calls: direct fn calls (declared order, arg coercion), string ops (literals/concat/strcmp), float ops (fcmp/literals), match on scalar/Str scrutinees | TODO |
| S4 | T3 green over ALL scalar-corpus files; T2 green over the same set; spot-check `examples/diff_test.xi` IR byte-equal | TODO |
| S5 | Tracker/bookkeeping: phase meter + next-phase pointer (Phase 6 -- structs/tuples/generics/unsafe whole-corpus T3) | TODO |

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

(none yet -- fill after S0: gate command, T3 scalar count, commit)
