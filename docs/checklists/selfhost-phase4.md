<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Selfhost Phase 4 checklist -- codegen fn-header T3 IR equality

Plan: `docs/SELFHOST_PLAN.md` (Phase 4). Branch: `selfhost-phase-4-codegen`
(seed off the Phase 3 merge). Method: repro-first under
`tmp/sprintc/phase4/`; cargo commands sequential; `python
tools/ascii_guard.py check` before every commit; no push.

## Gate

```sh
cargo test -p xiom-codegen --test full_diff_tests diff_ir_headers
```

Header-region IR equality over the 83-file corpus: for every file, the
ordered sequence of module-level declarations (`declare`/`define` signature
lines, parameter types + attributes, tuple struct type names, inline policy
attributes) emitted by the selfhost must equal the Rust emitter's byte for
byte. Function BODIES are Phase 5/6 and stay out of this extractor, so the
Phase 4 gate can go green while `selfhost/src/codegen.xi` still emits stub
bodies. `diff_corpus` T1 must stay green; T2/T3 remain unreachable until
bodies land.

## Staging

| Stage | Scope | State |
|-------|-------|-------|
| H0 | Harness: `ir_header_lines()` extractor + `diff_ir_headers` test (83 files) | TODO |
| H1 | Port fn signatures: name, return type, param list, `ptr`/byval shapes | TODO |
| H2 | Tuple struct names (`Tuple__*` / BUG 1 naming parity) + param attrs | TODO |
| H3 | Inline policy: port `approx_block_cost`, `alwaysinline`/`inlinehint` assignment | TODO |
| H4 | `declare` order for externals + module banner/version line | TODO |
| H5 | O1 cleanup pass: remove v10 "avoid the borrow checker" workarounds, `--strict` clean | after H4 |

## Rules of engagement

* Read the Rust emitter first (`crates/xiom-codegen/src/`): the exact
  spelling of every header line is the contract; do not invent formatting.
* Keep `selfhost/src/codegen.xi` emitting valid IR at every stage (the T1
  gate runs it over the whole corpus); stage H1..H4 incrementally replace
  the stub.
* One commit per stage with SESSION.md + checklist evidence; a stage is
  DONE only when `diff_ir_headers` is green over all 83 files.
* Findings (emitter/IR surprises) go to `docs/COMPILER_BUGS.md` with a
  minimal repro under `tmp/sprintc/phase4/`.
* The Bootstrap meter flips only when the Phase 4 gate is green; Phase 3
  is already DONE (4 of 11).

## Evidence

(none yet -- fill after H0 lands: gate command, counts, commit)
