<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Selfhost Phase 3 checklist -- checker parity (`--dump-check`)

Plan: `docs/SELFHOST_PLAN.md` (Phase 3). Branch: `selfhost-phase-3-checker`
(based on merged Phase 2, `60731523`). Method: repro-first under
`tmp/sprintc/phase3_checker/`; cargo commands sequential; `python
tools/ascii_guard.py check` before every commit; no push.

## Gate

```sh
cargo test -p xiom-codegen --test full_diff_tests diff_check
```

Green when, over the 83-file corpus AND the
`selfhost/tests/check_negative/*` manifest:

1. `xiom --dump-check <file>` and `xiomc-self --dump-check <file>` are
   line-for-line identical;
2. every manifest case matches its committed `.expected` on BOTH drivers;
3. the corpus is non-vacuous: the 4x W003 (`smoke_guard_fault.xi`) and the
   W008 (`m37_short_circuit.xi`) diagnostics are re-asserted exactly.

## Canonical format (owned by the Rust driver)

`crates/xiom/src/main.rs::dump_check`, mirrored by
`selfhost/src/checker.xi::dump_check`:

```
{kind} {code} {line}:{col} {escaped-message}   one line per diagnostic
CHECK-OK                                       no diagnostics
PARSE-ERROR                                    input/lex/parse failed
```

* `kind`/`code`: structured diagnostic fields (`type_error T001`;
  warnings carry their `WNNN` code, uncoded warnings `W000`).
* Order: exactly `CompileResult.diagnostics` -- catalog collisions (W001),
  then checker warnings, then type errors (Rust `compile_with_diagnostics`).
* `message` escapes `\`, LF, CR, TAB as `\\`, `\n`, `\r`, `\t`; all other
  bytes verbatim. The harness compares line-wise (CRLF tolerated).
* `CompileConfig::dump_check` stops right after the checker: no borrow pass,
  no codegen. The default selfhost compile path (`check_count`) fails only on
  hard errors, like the Rust driver.

## Staging

| Stage | Scope | State |
|-------|-------|-------|
| A | Rust canonical `--dump-check` + clap surface + unit test | DONE 2026-10-02 |
| 1 | Types/annotation equality (canonical type names, `types_compatible` port) | DONE |
| 2 | Statements/expressions (let/var/assign/return, if/while/for, blocks, W003) | DONE |
| 3 | Calls/generics (bare fns, methods, constructors, explicit type args) | DONE (subset; see deferred) |
| 4 | Contracts (requires/ensures strict clause check, `result` binding) | DONE |
| 5 | Diagnostics ordering (warnings before errors, push order) | DONE |
| B | Selfhost driver `--dump-check` + `diff_check` gate + manifest | DONE 2026-10-02 |
| C | Catalog/imports resolution (module map, aliases, qualified calls) | DONE 2026-10-03 (stage-1 scope) |
| D | Container method sets (builtin table, ctor typing, extension/R8 scan) | DONE 2026-10-03 |
| E | Unknown-method + struct-literal field validation | DONE 2026-10-03 |
| F | Lints W000 (non-exhaustive enums), W004 (unreachable arms), W006 (out-of-range shift), W007 (self-comparison) | DONE 2026-10-03 |

Deferred to later Phase 3 sub-stages (NOT ported yet; permissive `_`
fallbacks keep them from producing false positives):

* borrow-checker diagnostics (sub-stage 5; explicitly out of
  `compile_with_diagnostics`).
* module BODIES are not checked (the corpus has no catalog-body diagnostics);
  transitive `pub use` closure is unnecessary (0 re-exports in stdlib).
* uppercase bare names in `use` files stay permissive (module types the
  stage-1 closure cannot enumerate).
* dynamic stdlib header index blocked on `io.list_dir` returning pointer
  bits (COMPILER_BUGS 2026-10-03): a static 19-entry relocation table
  (`cm_static_module_path`) covers the relocated modules instead.

## Evidence

* Corpus ground truth (`tmp/sprintc/phase3_checker/dump_check_recon.txt`):
  81x `CHECK-OK`, `smoke_guard_fault.xi` 4x W003, `m37_short_circuit.xi`
  1x W008.
* `diff_check`: 83 corpus files + 58 manifest cases green
  (`83 files (5 diagnostic lines, non-vacuous) + 58 manifest cases`;
  sub-stage case sets live under `selfhost/tests/check_negative/`
  `catalog/` (18), `containers/` (5), `lints/` (8), `methods/` (11) plus
  the original 16).
* Regression gates after the port: `diff_tokens` green (83 files),
  `diff_ast` green (83 files), `diff_corpus` T1 green. T2/T3 remain
  unreachable (Phase 0 stub emitter; pre-existing).
* Findings: `docs/COMPILER_BUGS.md` 2026-10-02 Phase 3 section -- the
  `NkAssign` second-payload-field mis-read (same class as Phase 2 (h));
  workaround is side-helper field accessors + Int-tag statement dispatch.
  Also 2026-10-03: `io.list_dir` returns pointer bits (static relocation
  table workaround) and W000 multi-missing warning order is HashMap-random
  (port uses registration order; only single-missing cases gated).

## Manifest policy

`selfhost/tests/check_negative/<case>.xi` + `<case>.expected`. The
`.expected` file is the source of truth for BOTH drivers, so a Rust-side
message/span change must update the manifest (deliberate friction). Accept
cases (`CHECK-OK`) pin legacy/permissive behavior that the port must not
"improve" (e.g. assignment to `let` is not an error, unknown type
annotations pass, dotted variant patterns are catch-alls).
