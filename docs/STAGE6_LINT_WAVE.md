# Stage 6: control-flow lint wave

Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
SPDX-License-Identifier: MIT OR Apache-2.0

**Status:** LANDED -- tier 1 (W002/W003/W004) and tier 2 (W006/W007/W008);
tier 3 is **opt-in** (`--lint-strict`) per the 2026-11-02 owner decision, with
a measured noise report before any default-on change. Default builds stay
warning-clean.
**Owner probe that queued it:** 2026-09-25, playground -- two mutually
recursive functions that call each other before printing are reported by the
runner as "Program ran with no output". Diagnosis: not a compiler bug; the
program cannot terminate (no base case), the process dies via the runtime's
fault trap (exit `0xC000001D`; the runtime maps stack overflow and hardware
faults onto one deliberate trap), stdout stays empty. The runner surfacing
crash exits is a website-lane fix (see SESSION.md cross-lane status).

## Motivation

The compiler is deliberately conservative about warnings: it has hard errors
(`L001` lex, `P001` parse, `T001` type, `C001` codegen, `E001` borrow under
`--strict`) and advisory warnings (`E001` borrow by default, `W001` module
collision, `W000` catalog/checker). A small family of PROVABLE control-flow
defects currently compiles silently and fails at runtime -- exactly the class
that wastes a beginner's afternoon. They are cheap to detect syntactically
and belong in a warning-only wave.

Rules for the whole wave:

- **Warning-only, always.** No new warning becomes an error under `--strict`
  or any mode. A heuristic must never block a build.
- **User program only.** Lint the compilation unit's own functions and
  statements, never catalog/stdlib bodies, so the corpus stays warning-free
  by construction (the `catalog_corpus_is_clean` + zero-warning e2e gates
  must stay green).
- **One code per lint**, printed as `warning[WNNN]`, wired into the JSON
  diagnostics (`kind` value added to the v1 schema doc; v1 explicitly allows
  new `kind`/`code` values).
- **Two locks per lint**: a positive fixture that warns (binary integration
  test asserts the code in stderr AND exit 0) and a negative fixture that
  must stay silent (typ catchable false positive). Plus the corpus/e2e gates.
- Direct calls by name only in the call-graph lints. Method dispatch, trait
  resolution, fn pointers and generics are deferred; the first wave must not
  guess.

## Tier 1 -- sound syntactic checks (highest value, lowest risk)

### W002: unconditional recursive cycle

Call graph over the user program's free functions (direct calls by name).
Find SCCs with more than one node or a self-loop. Warn when **every function
in the cycle unconditionally reaches a call back into the cycle before any
`return`/`break`/`continue` that exits it** -- i.e. the recursive call
dominates the function exit. That is the owner's probe:

```
fn a() { b(); io.println("A"); }   // b() dominates the exit -> cycle a<->b
fn b() { a(); io.println("B"); }   // can never terminate
```

Message: `warning[W002]: 'a' is part of an unconditional recursive cycle
a -> b -> a; this call chain can never terminate`.

Not reported: recursion with a guard before the recursive call
(`if n <= 0 { return; } f(n - 1);`), recursion behind a condition, mutual
recursion reached through a branch, fn-pointer recursion.

### W003: unreachable statement after a diverging statement

Statements in a block that follow an unconditional diverger with no label or
branch target between them: `return`, `break`, `continue`, `while true`
without a reachable `break` belonging to it, or an `if`/`match` whose EVERY
path diverges. Message: `warning[W003]: unreachable statement (the previous
statement always exits)`.

Not reported: statements after a `return` inside an `if` branch that the
current block continues from; anything after a loop with a reachable `break`;
`debugger` (not a diverger).

### W004: unreachable match arm

An arm that can never match because an earlier arm is a catch-all (`_` or a
plain binding) or an exact duplicate literal/enum variant. Message:
`warning[W004]: unreachable match arm (an earlier arm already matches these
values)`. Complements `--strict-exhaustive`, which covers MISSING arms.

## Tier 2 -- sound with literal/type facts

**Numbering note (2026-09-29):** `W005` is already taken by the m142
codegen known-gap advisory (`warning[W005]: unresolved call ...`;
stderr-only, not a checker lint -- see `docs/JSON_DIAGNOSTICS_V1.md`),
so the div/rem lint ships as `W008`. `W006`/`W007` keep their planned
codes.

- **W008 literal division/remainder by zero** (`1 / 0`, `x % 0`) for integer
  types. Float `/ 0.0` is inf and allowed. Runtime today: the trap fires
  (`lint_div0` probe -> exit `0xC000001D`).
- **W006 shift amount out of range** for integer literals (`1 << 64` on i64;
  the probe ran with a garbage result instead of trapping -- worse than a
  crash). Type-aware (Int8/16/32/64, UInt...).
- **W007 self-comparison that is always true/false** (`x == x`, `x != x`)
  for NON-float types only; NaN makes it meaningful for floats.

## Tier 3 -- opt-in (`--lint-strict`), measured

**DECISION (owner, 2026-11-02): opt-in, measure first.** LANDED as a
boolean `--lint-strict` flag (`crates/xiom/src/cli.rs` -> `main.rs` ->
`CompileConfig::lint_strict` -> `Checker::set_lint_strict`). Default builds
emit no tier-3 warnings; the flag is warning-only and never promotes a
finding to an error, including under `--strict` / `--strict-exhaustive`.
There is no suppression syntax in this batch -- the flag is the gate.

| Code | Lint | Rule |
|---|---|---|
| `W010` | unused function parameter | `_`-prefixed names, methods/impls and extern/FFI signatures are skipped |
| `W011` | unused import | only a `use` whose leaf/alias has zero references anywhere in the module body; glob and `pub use` are skipped |
| `W012` | constant condition | literal `if true/false`, literal `elif`, and `while false`; **not** `while true` |
| `W013` | self-assignment | the same syntactic place on both sides (`x = x`, `self.n = self.n`) |
| `W014` | vacuous integer arithmetic | `x + 0`, `x - 0`, `x * 1`, `x / 1`; the literal side must be an integer literal and the other side integer-typed; floats skipped; `0 + x`/`0 - x`/`1 * x` are not reported |
| `W015` | unused local binding | lexical per-scope usage analysis with shadowing; `_`-prefixed names and bindings used inside a nested closure are ignored; a binding whose scope ends before a later lexical use is still reported |

`W009` was already the reserved-system-symbol advisory
(REL-20261028-0052), so the unused-local lint uses `W015` -- the only
renumbering from the original plan. `W005` remains the codegen advisory.

**Correctness contract.** The usage walker resolves each identifier through
the live lexical scope stack (innermost binding wins), so a use before a
same-scope declaration resolves outward and can never mark a not-yet-visible
binding as used. Bindings are checked when their scope pops. Match/pattern
bindings and closure parameters are registered for shadowing but are not
warned; a binding referenced from a nested closure counts as used.

### Measured noise and triage (2026-11-03, `examples/*.xi`, 61 files)

Command: `xiom --check --lint-strict examples/*.xi` (the phase-1 set is
included by the glob; compile-only, no codegen). Counts are the clean
stderr stream (one line per warning). Tier-3 only; the two pre-existing
tier-1 `W004` warnings are listed separately and were not part of this
cleanup.

| Code | Before (2026-11-02) | After (2026-11-03) |
|---|---:|---:|
| `W010` | 1 | 0 |
| `W011` | 4 | 0 |
| `W012` | 3 | 0 |
| `W013` | 1 | 0 |
| `W014` | 8 | 0 |
| `W015` | 30 | 0 |
| **tier-3 total** | **47** | **0** |
| pre-existing `W004` (tier 1, not tier 3) | 2 | 2 |

Files with at least one tier-3 warning: 21 of 61 before, 0 of 61 after.

The 47 findings were all accidental corpus noise, not intentional lint
hits; the lint implementation was not changed and no suppression syntax
was added:

- `W010` -- `phase1_selfhost.xi` `compile(source)` did not use `source`;
  renamed the parameter to `_source`.
- `W011` -- removed genuinely unreferenced imports: `is_prime`
  (`benchmark_stress.xi`), `parse_ident` (`phase1_selfhost.xi`), and
  `Expr` plus the `PErr` alias (`phase1_hardening.xi`).
- `W012` -- dropped `if true` wrappers in `benchmark_safe.xi` and
  `benchmark_selfhost.xi` (the body statements remain).
- `W013` -- removed the `marker = marker;` typo in `concat_debug.xi` and
  the now-dead `marker` local.
- `W014` -- dropped `+ 0` / `* 1` in `Big25.new` and `deep_if_nested`
  (`benchmark_stress.xi`, `concat_debug.xi`) and in the `&Int` readers
  (`phase1_ownership.xi`, `stress_borrow_10level.xi`).
- `W015` -- deleted or `_`-prefixed intentional test scaffolding locals
  in `benchmark_stress.xi`, `concat_debug.xi`, `demo_float.xi`,
  `phase1_contracts.xi`, `phase1_derive.xi`, `phase1_full.xi`,
  `phase1_hardening.xi`, `stress_float_matrix.xi`, `test_address.xi`,
  `test_bst_exact.xi`, `test_counter_bool.xi`, `test_float.xi`,
  `test_node.xi`, `test_reverse.xi`, `test_user.xi`, and
  `test_variant_fields.xi`. The underscore form preserves the original
  construction/coverage intent.

**Files changed:** `examples/benchmark_safe.xi`,
`examples/benchmark_selfhost.xi`, `examples/benchmark_stress.xi`,
`examples/concat_debug.xi`, `examples/demo_float.xi`,
`examples/phase1_contracts.xi`, `examples/phase1_derive.xi`,
`examples/phase1_full.xi`, `examples/phase1_hardening.xi`,
`examples/phase1_ownership.xi`, `examples/phase1_selfhost.xi`,
`examples/stress_borrow_10level.xi`, `examples/stress_float_matrix.xi`,
`examples/test_address.xi`, `examples/test_bst_exact.xi`,
`examples/test_counter_bool.xi`, `examples/test_float.xi`,
`examples/test_node.xi`, `examples/test_reverse.xi`,
`examples/test_user.xi`, `examples/test_variant_fields.xi`, plus
`docs/JSON_DIAGNOSTICS_V1.md` and this document. No checker, codegen,
selfhost, or stdlib source was changed; `--lint-strict` remains opt-in.

**Remaining intentional sites:** none. The only corpus warnings left are
the two pre-existing tier-1 `W004` unreachable-arm findings
(`examples/benchmark_stress.xi:3144` and `examples/concat_debug.xi:8574`),
which are outside tier 3 and predate `--lint-strict`. Three files
(`benchmark_stress.xi`, `concat_dbg.xi`, `concat_debug.xi`) still exit
non-zero for pre-existing *type* errors, independent of the lint flag; the
lint adds no errors and no non-zero exits.

### Verification (2026-11-03)

- `xiom --check --lint-strict examples/*.xi` (61 files, one invocation per
  file): tier-3 total 0; the only output is the two pre-existing `W004`
  lines above.
- Default `--check` on the 21 touched files: 19 are zero-warning/exit 0;
  `benchmark_stress.xi` and `concat_debug.xi` retain only their pre-existing
  `W004` + type errors (unchanged counts).
- `cargo test -p xiom --test checker_locks`: 52 passed on the verified
  build.
- `cargo test -p xiom-check catalog_corpus_is_clean`: passed on the
  verified build.
- `cargo test -p xiom-codegen --test e2e_tests warnings`:
  `e2e_m16_no_warnings` and `e2e_m17_zero_warnings` passed on the verified
  build.
- Spot e2e for the touched examples (demo_float, ownership, derive,
  contracts, full, hardening, selfhost_sim, stress_borrow_10level,
  stress_float_matrix): 9 passed.

The checker/codegen/selfhost trees were not edited by this triage. A
separate in-flight strict-`let` init-once fix in `crates/xiom-check` can
transiently affect catalog-body and zero-warning e2e results; final re-runs
of `checker_locks`, `catalog_corpus_is_clean`, and the zero-warning e2e
gates are deferred to that owner.

### Default-on recommendation (post-triage; owner decides)

After the corpus cleanup the measured tier-3 noise is zero (47 -> 0 over
61 files, 21 -> 0 noisy files), and every finding was a real code defect
or intentional scaffold, not a lint false positive. Default-on for
`W010`-`W015` as warning-only lints is therefore defensible: on the
measured corpus it would add no output at all, and warning-only means it
cannot break builds.

Suggested rollout if the owner approves: flip all six to default-on in a
separate change (keep `--lint-strict` accepted as a compatibility no-op or
retain it as the gate for any future noisier tier), then re-run the
61-file corpus plus the full e2e and `catalog_corpus_is_clean` to confirm
zero user-code output. If a more conservative step is preferred, enable
`W011`/`W012`/`W013`/`W014` first and leave `W015` opt-in for one wave:
`W015` is the only family whose correctness depends on conservative
usage-walking, and it is the largest former noise source (30 of 47).

## Plumbing checklist (per lint)

1. Warning code + message; emit through the checker/codegen warning list.
2. Driver printer `warning[WNNN]` (see `crates/xiom/src/lib.rs` W000/W001
   printing) and `kind`/`code` in the JSON diagnostics envelope; update
   `docs/JSON_DIAGNOSTICS_V1.md` (add the kind, keep `schema_version: 1`).
3. Positive/negative fixtures under `tests/regression/` + assertions in
   `crates/xiom/tests/checker_locks.rs` (stderr contains the code; compile
   exit 0).
4. `catalog_corpus_is_clean`, stdlib-exec, and the full e2e stay green with
   zero new warnings (user-program scope makes this structural, the run
   proves it).
5. SESSION.md evidence entry with the fixture names.

## Companion diagnostics polish (same UX theme, not a lint)

The 2026-09-25 owner questions also exposed a teachability gap in `P001`.
Verified deterministic behavior:

| Shape | Result |
|---|---|
| `stmt;` + `stmt;` | compiles |
| `stmt` + `stmt` (none) | `error[P001]: 5:3: expected ';', found io` |
| `stmt;` + `stmt` (last omitted) | compiles (tail expression) |
| `stmt` + `stmt;` (first omitted) | `error[P001]: 5:3: expected ';', found io` |

`;` separates statements; it is optional only on a block's FINAL expression
(the block's value -- needed by expression-oriented returns). A one-statement
body is always tail position, so both one-liners compile; that is what makes
the rule feel inconsistent to a beginner.

Queued work (no grammar change):

1. `P001` missing-semicolon suggestion: when `expect(';')` fails and the next
   token begins a statement on a later line, emit
   `note: only the last expression in a block may omit ';'; add ';' after
   line N`. Both spans are already known to the parser. Keep the existing
   error text so message-matching tests are untouched (the note is additive).
2. Docs/starter-snippet pass: state the rule where beginners meet it ("every
   statement ends with `;`; the last expression in a block may omit it") and
   make the multi-statement starter example use `;` on every statement.
3. Explicitly DO NOT add newline-as-statement-separator (ASI): grammar
   ambiguity (method chains, multi-line operators, continuations), divergence
   from the frozen grammar, and it only relocates the confusion ("when does a
   newline end a statement?"). That is a versioned language-design decision,
   not a patch.

### Semicolon rule: why the tail form exists (design options)

The rule is NOT "the last statement is special"; it is "`;` ends a
statement, and a block's VALUE is its final expression, which is not a
statement". XIOM's own stdlib depends on it:

```
// stdlib/xiom/core/cmp.xi:96
pub fn max_int(a: Int, b: Int) -> Int {
  if a >= b { a } else { b }      // tail VALUE, no ';', no 'return'
}
// cmp.xi:88 (match arms as values)
```

and `fn add(a: Int, b: Int) -> Int { a + b }` compiles today (probe
`tmp/sprintc/tail_expr_probe.xi`). The single-statement hello-world case is
a CONSEQUENCE of the rule, not its purpose.

Three coherent options, for a future versioned decision (v0.62.0 keeps the
current semantics -- the artifact is frozen and the stdlib relies on tails):

- **(a) Current / Rust-like**: `;` separates statements; the tail expression
  is the block value. Pros: concise value blocks, `if`/`match` stay
  expressions, no `return` noise. Cons: a Unit-returning one-liner
  (`fn main() { io.println("hi") }`) teaches the wrong habit; needs the
  queued P001 note + docs.
- **(b) C-style**: `;` after every statement; value blocks require explicit
  `return`; `if`/`match` are not expressions in value position. Pros: one
  unconditional rule for beginners. Cons: breaking sweep (stdlib cmp.xi and
  friends, tests, docs), more verbose, loses expression orientation.
- **(c) Hybrid / Unit-tail rule**: tails stay allowed only when they produce
  a NON-Unit value; a Unit-typed tail expression (`io.println(...)`) must
  end with `;`. Pros: removes the hello-world trap while keeping `a + b` /
  `if..else` tails. Cons: still conditional (the rule depends on the type),
  and it breaks existing Unit-tail bodies -> also a breaking, versioned
  change.

Recommendation: keep (a) through v0.62.0; if the beginner trap is judged
worth removing later, take (c) as an explicit language-design change with a
migration note and a corpus sweep -- do not patch the grammar around a
release.

**DECISION (owner, 2026-09-25): (a) is the language's model and stays.**
Rationale: XIOM is already expression-oriented (`if`/`match` tails, block
values; stdlib `core/cmp.xi` depends on them), the modern/AI-generated idiom
is Rust-like tails, and the beginner friction is a diagnostics/docs problem
with a cheap fix. Mitigation shipped/queued as a package: the P001
missing-separator note (this wave), `AI_CONTEXT.md` + MCP language guide
updated, and the website/playground relay (SESSION cross-lane). Options
(b)/(c) stay recorded for reference only.

## Suggested order

W002 + W003 first (the probe class and the most common dead-code class), then
W004, then tier 2 (W006/W007/W008), then decide tier 3. Each landing is a
normal batch: implementation + locks + full e2e, no release coupling.
