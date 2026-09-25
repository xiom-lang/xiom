# Stage 6: control-flow lint wave

Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
SPDX-License-Identifier: MIT OR Apache-2.0

**Status:** QUEUED for Stage 6 (starts after the combined v0.62.0 release).
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

- **W005 literal division/remainder by zero** (`1 / 0`, `x % 0`) for integer
  types. Float `/ 0.0` is inf and allowed. Runtime today: the trap fires
  (`lint_div0` probe -> exit `0xC000001D`).
- **W006 shift amount out of range** for integer literals (`1 << 64` on i64;
  the probe ran with a garbage result instead of trapping -- worse than a
  crash). Type-aware (Int8/16/32/64, UInt...).
- **W007 self-comparison that is always true/false** (`x == x`, `x != x`)
  for NON-float types only; NaN makes it meaningful for floats.

## Tier 3 -- deferred (needs a noise-budget decision)

- Unused locals / parameters / imports. Highest user value, noisiest; needs
  a usage analysis and a decision on default-on vs opt-in.
- Constant conditions (`if true`, `while false`), self-assignment (`x = x`),
  vacuous arithmetic (`x + 0`). Mostly typos; can ride the same plumbing
  once tier 1 proves the noise level.
- `while true` with no exit (W005-family): intentional server loops are
  common, so this needs a phrasing/opt-in decision before it ships.

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

## Suggested order

W002 + W003 first (the probe class and the most common dead-code class), then
W004, then tier 2 (W005-W007), then decide tier 3. Each landing is a normal
batch: implementation + locks + full e2e, no release coupling.
