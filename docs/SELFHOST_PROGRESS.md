<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM Selfhost -- bootstrap progress tracker

**Last updated:** 2026-10-03 | **Plan:** `docs/SELFHOST_PLAN.md` |
**Phase 0 checklist:** `docs/checklists/selfhost-phase0.md` |
**Phase 1 checklist:** `docs/checklists/selfhost-phase1.md` |
**Phase 2 checklist:** `docs/checklists/selfhost-phase2.md` |
**Phase 3 checklist:** `docs/checklists/selfhost-phase3.md` |
**Owner policy:** selfhost ships only at 100% bootstrap; every release stays
Rust-hosted until then.

## The 100% definition (bootstrap gate)

All six must hold (SELFHOST_PLAN section 7):

1. `self1.exe` and `self2.exe` byte-identical (sha256) -- the compiler
   reproduces itself.
2. T3 byte-identical IR on the full corpus (`full_diff_tests` T3 green).
3. The selfhost binary passes the full e2e suite with the same results as
   the Rust binary.
4. Zero C-codegen dependency (`runtime_ffi.xi` in pure XIOM).
5. Zero warnings on the selfhost sources under `--strict`.
6. Selfhost compile-time within 2x of the Rust compiler on the corpus (O2).

## Bootstrap meter

**27% -- 3 of 11 tracked gates complete.**

**Gates: e2e 2411/2411 (+4 ignored), checker 195/195, feature 518/518, robustness 63/63, fuzz 24/24, perf 3/3, formatter 86/86, lsp 45/45.**

e2e / checker / feature re-measured green at the v0.62.3 release state
(`3122cf9d`, pin `stdlib-perf3`, 2026-10-03); robustness / fuzz / perf /
formatter / lsp carry the 2026-10-02 snapshot on `f4734c07`. (Older e2e
snapshots: 2409 at `f4734c07`, 2408 at `592c64d4`, 2406 at `d1ab8ec4`,
2404 at `8abedb72` -- each grew with its regression locks.)

Release context: v0.62.3 shipped 2026-10-03 with `SELFHOST_VERSION` bumped
to 0.62.3; the selfhost source is otherwise unchanged by this release and
the Rust compiler remains the shipped bootstrap.

PHASE 3 STATUS (2026-10-03): **COMPLETE on branch
`selfhost-phase-3-checker`** -- full parity gate green (`diff_check`
line-exact on 83 corpus files + 75 manifest cases, 5/5 full_diff tests,
443.9 s), branch meter 36% (4/11), 15 commits ahead. Remaining: rebase
onto current main and merge (the merge flips this row and the meter;
until then main stays 27%). Next phase: **Phase 4 -- codegen fn-header T3
IR equality** (signatures, tuple names, inline policy `approx_block_cost`).

Weights are one gate each (equal weighting; phases differ in effort but a
gate is only "done" when its evidence is green). Update this line whenever a
row flips.

| # | Gate | Status | Evidence |
|---|------|--------|----------|
| 0 | T1 harness green on the corpus (foundations) | **DONE 2026-09-29** | `cargo test -p xiom-codegen --test full_diff_tests`: 2 passed; T1 over 84 files in 49.2 s; `runtime_ffi_selfcheck` ok; commit `e813449b` |
| 1 | Lexer: token-dump equality on the corpus (`--dump-tokens`) | **DONE 2026-10-02** | Phase 1; `selfhost/src/lexer.xi` ports `crates/xiom-lexer`; harness gate `full_diff_tests::diff_tokens` green over the 83-file corpus (3/3 tests, 72.7 s); torture parity (BOM/CRLF/NUL/bigints/suffix quirk) clean; checklist `docs/checklists/selfhost-phase1.md` |
| 2 | Parser: AST-dump equality on the corpus (`--dump-ast`) | **DONE 2026-10-02** | Phase 2; `selfhost/src/ast.xi`+`parser_state.xi`+`parser_expr.xi`+`parser_core.xi`+`ast_dump.xi` port `crates/xiom-parser`/`xiom-ast`; harness gate `full_diff_tests::diff_ast` green over the 83-file corpus (1 passed, 100.7 s); checklist `docs/checklists/selfhost-phase2.md` |
| 3 | Checker: diagnostics + type-annotation equality | **STAGED 2026-10-02** (not complete) | Phase 3 stage-1 subset; `full_diff_tests::diff_check` green over the 83-file corpus + 16-case manifest (5 corpus diagnostic lines, non-vacuous). Deferred sub-stages (catalog/imports, container method sets, unknown-method/struct field validation, W000/W004/W006/W007 lints, borrow pass) keep permissive fallbacks; the meter stays at 3 of 11 until full checker parity lands. Checklist `docs/checklists/selfhost-phase3.md` |
| 4 | Codegen: fn-header T3 IR equality | NOT STARTED | Phase 4; signatures, tuple names, inline policy (`approx_block_cost`) |
| O1 | Selfhost code quality: `--strict`, zero warnings, contracts on | NOT STARTED | after Phase 4; removes v10 borrow workarounds |
| 5 | Codegen: scalar bodies + control flow T3 (scalar corpus) | NOT STARTED | Phase 5 |
| 6 | Codegen: structs/tuples/generics/unsafe T3 (whole corpus) | NOT STARTED | Phase 6 |
| 7 | Self-compile chain: self1 == self2 sha256 + T3 on self1-vs-self2 IR | NOT STARTED | Phase 7 |
| O2 | Bootstrap perf: self1 within 2x of `xiom.exe` on the corpus | NOT STARTED | after Phase 7; target < 60 s self-compile on this box |
| 8 | Full green: T3 un-ignored, full e2e on both binaries, identical results | NOT STARTED | Phase 8 |

## Phase 0 evidence (landed 2026-09-29, commit `e813449b`)

- Harness: `crates/xiom-codegen/tests/full_diff_tests.rs` -- hardcoded
  deterministic 84-file corpus manifest, T1/T2/T3 tiers
  (`XIOM_SELFHOST_DIFF_TIER=1|2|3`, tiers stack), runner compiles
  `selfhost/src/main.xi` once per test process into
  `target/selfhost/xiomc-self.exe` and passes the source path as argv[1].
- Skeleton: `selfhost/src/{main,lexer,parser,checker,codegen,selfcheck,
  runtime_ffi}.xi`; driver runs read -> lex -> parse -> check -> codegen;
  `--selfcheck` mode.
- `runtime_ffi.xi` ports (pure XIOM, no C): `rt_str_len`, `rt_char_at`
  (UTF-8 codepoint decoder, exact C semantics), `rt_str_slice`,
  `SymbolTable` (1-based intern/lookup), `FnTable`, `IrBuffer`.
  Behavior asserted by `xiomc-self --selfcheck` against the C outputs in
  `stdlib/runtime/xiom_runtime.c`.
- Archived `selfhost/archive/xiomc_v050.xi`; deleted 13 stale `_diff_*`
  temp artifacts.
- Deferred (documented, not stubbed): float `{:.17e}` formatting -- lands
  with the emitter port (Phase 4/5), T3-gated.

## Phase 1 evidence (landed 2026-10-02)

- `selfhost/src/lexer.xi`: full port of `crates/xiom-lexer/src/lib.rs`
  (TokenKind/Token, char-indexed scan with an independent byte accumulator,
  BOM stripping, Unicode whitespace, comments/shebang, exact error text,
  big-int u128 classification, `\xNN`/`\u{...}` rules, the numeric-suffix
  span quirk).
- `crates/xiom/src/main.rs::dump_tokens` defines the canonical dump
  (`{line}:{col}:{byte_start}:{byte_end} {TAG}[ {PAYLOAD}]`); the selfhost
  side mirrors it. Float payloads dump the token LEXEME (value parity is
  deferred; see the Phase 1 checklist).
- Gate: `cargo test -p xiom-codegen --test full_diff_tests` -> 3 passed
  (`diff_tokens` line-exact over 83 corpus files + `diff_corpus` T1 +
  `runtime_ffi_selfcheck`).
- Findings filed in `docs/COMPILER_BUGS.md` (2026-10-02): nested self-method
  receiver mutation does not propagate (lexer uses free `&mut Lexer`
  helpers); enum payloads of 128-bit integer types lower as i64 (BigInt
  stored as hi/lo UInt); `1e999` codegen emits `double inf`; `Str + UInt`
  formats u64::MAX as -1 and UInt128 `/`/`%` are signed for high-bit values
  (the dump sidesteps all three by rendering hex via shifts/ands).

## Phase 2 evidence (landed 2026-10-02)

- Canonical `--dump-ast` on BOTH compilers, one byte-stable format:
  `crates/xiom/src/main.rs::dump_ast` (`AstDump`) owns the definition
  (`{indent}{Kind}[ key=value]... [span=l:c:bs:be]`, lowercase-hex payloads,
  `PARSE-ERROR` on a failed parse); `xiomc-self --dump-ast` mirrors it via
  `selfhost/src/parser.xi` + `selfhost/src/ast_dump.xi`. `--dump-ast` added
  to the clap surface (`crates/xiom/src/cli.rs`).
- Parser port (1:1 control flow, arena representation):
  `selfhost/src/ast.xi` (flat `Vec[Node]` + Int child indices, `-1` =
  absent; REQUIRED because recursive value enums mis-lower -- COMPILER_BUGS
  (e)), `parser_state.xi` (Parser + `p_*` helpers; `TokenKind` identity via
  Int tags, never `==`, because aggregate-payload enum equality emits
  invalid IR -- COMPILER_BUGS (g)), `parser_expr.xi` (types, params,
  generics, blocks, statements, patterns, all expression parsing),
  `parser_core.xi` (program/file-module wrapping, top-level decls,
  attributes, fn/where/contracts, consts, externs), `ast_dump.xi` (canonical
  walker).
- Float literal payloads dump the token LEXEME (value parity stays deferred;
  the lexeme is Phase 1-gated and equals the Rust byte-slice for every float
  token). Str literals dump decoded bytes (selfhost stores `Vec[UInt8]`).
- Gate: `cargo test -p xiom-codegen --test full_diff_tests` includes the
  un-ignored `diff_ast`; the Phase 2 run over the 83-file corpus passed
  line-exact (1 passed, 100.7 s; T1/T2 also green).
- Findings filed in `docs/COMPILER_BUGS.md` (2026-10-02): (e) recursive enum
  payloads pointer-boxed unsafely (crashes/mis-values); (f) qualified
  enum-variant patterns false-non-exhaustive; (g) `==` on enums with
  `Vec` payloads lowers to `icmp %struct.Vec`; (h) `NkExprGenericCall`
  destructure mis-maps payload fields in large functions (fixed by
  one-step construction + base/types side locals).

## Phase 3 evidence (staged 2026-10-02, branch `selfhost-phase-3-checker`)

Stage-1 checker port is GREEN as a gate; the phase itself is NOT complete
(the meter stays unchanged until full parity).

- Canonical `--dump-check` on BOTH compilers:
  `CompileConfig::dump_check` (Rust) stops `compile_with_diagnostics` right
  after the checker; `crates/xiom/src/main.rs::dump_check` owns
  `{kind} {code} {line}:{col} {escaped-message}` / `CHECK-OK` /
  `PARSE-ERROR`; `selfhost/src/checker.xi::dump_check` mirrors it.
- Checker port: `selfhost/src/check_types.xi` (canonical type names,
  `types_compatible` port), `check_state.xi` (scopes/symbol tables/
  diagnostics), `check_core.xi` (signature collection + program/fn walk +
  contracts), `check_expr.xi` (statements/expressions, W003 divergence,
  W008, calls/methods/generics). Permissive `_` fallbacks cover the
  catalog/container/borrow sub-stages listed in the checklist.
- Gate: `cargo test -p xiom-codegen --test full_diff_tests diff_check` --
  83 corpus files line-exact (81 `CHECK-OK`, 4x W003 on
  `stdlib/tests/smoke/smoke_guard_fault.xi`, 1x W008 on
  `tests/regression/m37_short_circuit.xi`) + 16 manifest cases
  (`selfhost/tests/check_negative/`, `.expected` is the source of truth for
  both drivers).
- Regression: `diff_tokens` green, `diff_ast` green, T1 `diff_corpus`
  green. T2/T3 stay unreachable (Phase 0 stub emitter; documented
  pre-existing).
- Finding: `docs/COMPILER_BUGS.md` 2026-10-02 (selfhost Phase 3) --
  `NkAssign(l, r)` destructure in one function reads `r` as pointer bits
  (crash `0xC0000005`) while one-arm accessor helpers read both fields
  correctly; workaround is side-helper field accessors (same class as
  Phase 2 (h)).
- Sub-stage 1 (2026-10-03): catalog/imports resolution.
  `selfhost/src/check_modules.xi` loads imported module sources (stdlib
  shapes, local files, static relocation table for the 19 modules whose
  declared name does not match their path), registers pub fns/types/consts
  and externs under dotted keys, binds `use` aliases on declared-module
  match and tracks in-program module names; qualified calls/fields and the
  `xiom.` namespace path resolve like Rust, and bare lowercase unknowns in
  `use` files error. `diff_check` remains green at 83 corpus files + 34
  manifest cases (18 new catalog cases). Finding filed: `io.list_dir`
  returns pointer bits instead of names (COMPILER_BUGS 2026-10-03). Meter
  unchanged (3 of 11).
- Sub-stage 2 (2026-10-03): container method sets. Rust's `register_builtins`
  table is ported (Vec/Slice/Map/Set constructors + Vec methods + free
  intrinsics); instance dispatch resolves builtin keys, catalog extension
  keys and the R8 UFCS scan with Rust's exact param-offset table; container
  constructor results keep their type arguments so nested indexing stays
  typed. `diff_check` green at 83 corpus files + 39 manifest cases.
  Meter unchanged (3 of 11).

## Open blockers and risks

| Item | Impact on 100% | State |
|------|----------------|-------|
| Float `{:.17e}` exactness | T3 on float-literal emission | deferred to Phase 4/5 (documented) |
| Recursion-depth trap (500) vs deep selfhost recursion | bootstrap crash risk | mitigation queued in O1/O2 per plan |

**Recently cleared:** m163 (method field `Vec[Str]` element miscompile) --
fixed 2026-09-29 with locks + full e2e 2390/2390; the compiler no longer
limits selfhost string-building code (the `IrBuffer` single-field shape is
kept for Phase 0 simplicity only). m162 (same-leaf user-module export
poisoned catalog-body resolution) -- fixed 2026-09-29 in checker + codegen;
the runtime_ffi `rt_` prefix rule is no longer required (kept; rename in
O1).

## Running the gates

```
# Phase 0 gate (T1 green); Phase 1 (diff_tokens) + Phase 2 (diff_ast) run in
# the same suite
cargo test -p xiom-codegen --test full_diff_tests
$env:XIOM_SELFHOST_DIFF_TIER=2; cargo test -p xiom-codegen --test full_diff_tests   # T2
$env:XIOM_SELFHOST_DIFF_TIER=3; cargo test -p xiom-codegen --test full_diff_tests   # T3 (phase completion)

# Skeleton build/run (mirrors what the harness does)
target/debug/xiom.exe -o target/selfhost/xiomc-self.exe selfhost/src/main.xi
target/selfhost/xiomc-self.exe --selfcheck
target/selfhost/xiomc-self.exe examples/diff_test.xi

# Phase 1 parity spot-check (canonical token dumps; CRLF on Windows pipes)
target/debug/xiom.exe --dump-tokens examples/diff_test.xi
target/selfhost/xiomc-self.exe --dump-tokens examples/diff_test.xi

# Phase 2 parity spot-check (canonical AST dumps)
target/debug/xiom.exe --dump-ast examples/diff_test.xi
target/selfhost/xiomc-self.exe --dump-ast examples/diff_test.xi
```

Long suites on this box: e2e `-- --test-threads 12`, stdlib_tests 8,
`scripting_tests` 4.
