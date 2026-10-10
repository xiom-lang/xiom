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

**REBASE RE-VERIFY (2026-10-09, v0.64.2 batch `a6ee8839`): lane rebased onto
local main (8 commits replayed, no conflicts). Two drifts fixed on the lane:
`SELFHOST_VERSION` 0.64.0 -> 0.64.2 (the release prep bumped Cargo; the
banner contract is the tree version) and m198 (Pulse C-PULSE-06)
missing-struct-field rejection ported to `ce_check_struct_lit` (+ updated
`unknown_struct_field_lit.expected` manifest). Full suite: `cargo test -p
xiom-codegen --test full_diff_tests` -> 6 passed / 0 failed (759.9 s),
`diff_ir_headers` counts identical (83 files / 8337 header lines / 258
primary define symbols / 4 tuple defs / 7909 declares). T2 tier red as
designed (module prologue before bodies); T3 not attempted. `--selfcheck`
OK; token/ast spot checks 0 diffs; meter stays 45%. Final certification:
re-rebased onto the v0.64.2 tip `afc63a9a` (m240/m241 landed mid-run) ->
full suite 6 passed / 0 failed (392.2 s).**

**REBASE RE-VERIFY (2026-10-05, v0.64.0 `fe346094`): lane rebased onto main
(no conflicts; v0.64.0 release-state text preserved; stdlib pin moved to
`6e60e958`, SELFHOST_VERSION 0.64.0). Full suite: `cargo test -p
xiom-codegen --test full_diff_tests` -> 6 passed / 0 failed (323.1 s).
`diff_ir_headers` standalone -> 1 passed with IDENTICAL counts (83 files /
8337 header lines / 258 primary define symbols / 4 tuple defs / 7909
declares, 198.3 s). m195/m196 caused no parity drift; nothing to fix.**

### H5 (2026-10-05): O1 cleanup -- PARTIAL (selfhost-attributed target met)

- v10 workaround audit: zero `1 == 0` / `0 == 1` "avoid the borrow checker"
  patterns anywhere in `selfhost/src` (lexer `done` flags are ordinary loop
  exits). Nothing to remove; Phases 1-3 never carried the v10 idiom over.
- Contracts: 13 trivially-true clauses added on the public API surface --
  `rt_str_len`, `rt_char_at`, `SymbolTable.intern`, `SymbolTable.count`,
  `FnTable.count`, `lex_count`, `check_count`, `dump_nodes`,
  `cg_cost_of_block` (`result >= 0`), `lx_tokenize`, `bc_run`,
  `decl_name`, `builtin_symbols` (`result.len() >= 0`). Full functional
  contracting of every ported module stays O1 follow-up.
- `--strict` on `selfhost/src/main.xi`: 0 warnings attributed to
  `selfhost_*` modules. The 15 printed warnings are all catalog stdlib
  functions missing `#[safety_audit]` (io/string/env/math/os/Pipe) -- the
  stdlib lane's remit (reproduced standalone: a `use xiom.io` probe prints
  the same class of warning).
- Gates: `cargo test -p xiom-codegen --test full_diff_tests` -> **6 passed**
  (diff_corpus T1, runtime_ffi_selfcheck, diff_tokens, diff_ast, diff_check,
  diff_ir_headers) in 784.5 s; selfhost driver rebuilt and `--selfcheck`
  SELFCHECK OK.
- Status: O1 row stays open (global zero warnings + contracts on all ported
  modules); this entry captures the selfhost-lane share.

**REBASE RE-VERIFY (2026-10-05, v0.63.1 `e4857847`): branch rebased onto main
(no conflicts; main's fetched Phase 4 status block auto-merged with this
evidence). `diff_ir_headers` green with the IDENTICAL contract: 83 files /
8337 header lines / 258 primary define symbols / 4 tuple defs / 7909 declare
lines (387.1 s). T1 `diff_corpus` green (83 files, 201.3 s). Selfhost driver
rebuilt in-worktree and `--selfcheck` -> SELFCHECK OK (exit 0). No output
delta from v0.63.0 -> v0.63.1.**

**PHASE 4 COMPLETE (2026-10-05): all stages H0-H4 green on the 83-file
corpus; the full gate is `diff_ir_headers` at final scope (banner, define
signatures with inline attributes, tuple definitions, declare order).**

### H4 (2026-10-05): declare order (`declare` scope) -- PHASE 4 GATE

- Port: `selfhost/src/codegen_declares.xi` carries the fixed 94-line builtin
  table (7 early + 87 main, verbatim from the Rust emitter) plus
  `builtin_symbols()`/`decl_name()`; `codegen.xi` emits early declares before
  the tuple block, the main table after it, then walks primary extern blocks
  (`emit_extern_declares` port: `extern_type_to_llvm`, already-declared skip
  against the builtin set) and the deferred `@xiom_thread_spawn` line
  (suppressed when a user extern declares it, matching lib.rs).
- Extractor: `allow.declares` = builtin symbols (reference compile of the
  no-import `examples/diff_test.xi`, cached) union the primary extern-block
  symbols; catalog-injected externs stay out of Phase 4 (documented).
- Gate (FULL PHASE 4): `cargo test -p xiom-codegen --test full_diff_tests
  diff_ir_headers` -> **1 passed**; 83 files / 8337 header lines / 258
  primary define symbols / 4 tuple defs / 7909 declare lines (218.0 s).
- T1: `cargo test -p xiom-codegen --test full_diff_tests diff_corpus` ->
  1 passed (83 files, 177.5 s) on the same tree.
- Meter: gate 4 flips to DONE; `docs/SELFHOST_PROGRESS.md` 45% (5 of 11).

### H3 (2026-10-05): inline policy (`approx_block_cost` + attribute scope)

- Port: `selfhost/src/codegen_cost.xi` (own module, mirroring the stable
  `check_borrow.xi` layout -- the statement-destructure AV class is
  context-sensitive; in-module it crashed and silently under-counted nested
  `if`s). Exact Rust rules: 1 per statement, nested if/elif/else, while/for,
  match arms (block = block cost, expr = 1), spawn; `<=10 -> alwaysinline`,
  `<=48 -> inlinehint`, else none.
- Extractor: `compare_inline_attrs = true` -- define lines now compare with
  the trailing inline attribute byte for byte.
- Gate: `cargo test -p xiom-codegen --test full_diff_tests diff_ir_headers`
  -> **1 passed**; 83 files / 428 header lines / 258 primary define symbols /
  4 tuple definitions (339.2 s). Differential sweep 83/83 with attributes.
- Finding updated: docs/COMPILER_BUGS.md 2026-10-04 H3 addendum (module
  isolation is the effective workaround).

### H2 (2026-10-05): tuple struct definitions (`Tuple__*` scope)

- Harness: `tuple_names_in()` seeds `allow.tuples` from the `%struct.Tuple__*`
  tokens of the already-filtered Rust define lines; both streams are then
  re-extracted so the selfhost must reproduce every referenced tuple
  definition byte for byte (BUG 1 element-key resolution included).
- Port: two-pass emission in `selfhost/src/codegen.xi` -- pass 0 walks the
  primary signatures registering `Tuple__` names + resolved element keys
  (`cg_register_tuple`/`cg_note_tuple_types`), the definition block is
  emitted sorted by full key (`cg_emit_tuple_defs`, `%struct.X = type { ... }`
  spelling, self-reference pointer rule), then pass 1 emits the define
  headers exactly as H1.
- Gate: `cargo test -p xiom-codegen --test full_diff_tests diff_ir_headers`
  -> **1 passed**; 83 files / 428 header lines / 258 primary define symbols /
  4 tuple definitions (276.6 s). Differential sweep
  `tmp/sprintc/phase4/h1_check.py` (defines + tuple-def text vs saved Rust
  dumps): 83/83 clean.

### H1 (2026-10-04): primary fn signatures (`define` scope)

- Harness: `header_allow_for()` builds the define allow-list from the source
  AST (`TopDecl::Fn`/`Impl`; `impl Trait[X]` target = `X`; skip generics,
  generic receivers, bodyless externs, shadowed empty main), intersected with
  the Rust define symbols; a source symbol Rust never defines fails the test.
- Port: `selfhost/src/codegen.xi` emits real headers for the primary unit --
  name/return/params, ptr/byval (`param_llvm_type`), concrete
  `Option__`/`Result__`/`Tuple__` names (`concrete_type_for` + BUG 1 naming),
  registry/builtin/suffix LLVM lowering, `main` argc/argv, stub bodies.
  Inline attributes are deliberately stripped by the extractor until H3.
- Gate: `cargo test -p xiom-codegen --test full_diff_tests diff_ir_headers`
  -> **1 passed**; 83 files / 424 header lines / 258 primary define symbols
  (312.7 s). Differential pre-check `tmp/sprintc/phase4/h1_check.py`: 83/83
  exact and order-clean against the saved Rust dumps.
- Findings: docs/COMPILER_BUGS.md (2026-10-04) statement-destructure AV with
  repro `tmp/sprintc/phase4/probe_c2.xi`; workaround = defer body-mutation
  ABI analysis (explicit `mut self` only) + tag-dispatch walkers.

### H0 (2026-10-04): harness extractor + banner gate

- `crates/xiom-codegen/tests/full_diff_tests.rs`: `ir_header_lines()` -- the
  staged `HeaderAllow` extractor (banner -> primary `define` signatures ->
  tuple struct defs -> `declare` order; inline attributes stripped until the
  H3 port) -- plus the `diff_ir_headers` test over the 83-file corpus.
  H0 scope = the module banner (`; XIOM v... LLVM IR` + `; Auto-generated by
  xiom`) asserted on BOTH drivers for every file.
- Gate: `cargo test -p xiom-codegen --test full_diff_tests diff_ir_headers`
  -> **1 passed**; 83 files / 166 banner lines (444.5 s, shared-box load).
- T1 `diff_corpus` re-ran green on the same tree in the same session
  (`1 passed`, 83 files, 309.0 s).
- Boundary (documented): H0's scope is the banner only -- signatures, tuple
  names, inline policy and declares widen per stage H1-H4, so the gate is
  green at every stage over its own committed scope.
