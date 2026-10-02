<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM Selfhost -- bootstrap progress tracker

**Last updated:** 2026-10-02 | **Plan:** `docs/SELFHOST_PLAN.md` |
**Phase 0 checklist:** `docs/checklists/selfhost-phase0.md` |
**Phase 1 checklist:** `docs/checklists/selfhost-phase1.md` |
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

**18% -- 2 of 11 tracked gates complete.**

**Gates: e2e 2401/2401 (+4 ignored), checker 195/195, feature 517/517, robustness 63/63, fuzz 24/24, perf 3/3, formatter 86/86, lsp 45/45.**

All eight suites re-measured green on `565b9924` (2026-10-02, Windows box):
e2e at 8 threads with the debug driver rebuilt from HEAD; the rest via their
cargo suites. (The earlier 2395 e2e snapshot was `aca6aaee`, before the
m169/m170/m168b locks.)

Weights are one gate each (equal weighting; phases differ in effort but a
gate is only "done" when its evidence is green). Update this line whenever a
row flips.

| # | Gate | Status | Evidence |
|---|------|--------|----------|
| 0 | T1 harness green on the corpus (foundations) | **DONE 2026-09-29** | `cargo test -p xiom-codegen --test full_diff_tests`: 2 passed; T1 over 84 files in 49.2 s; `runtime_ffi_selfcheck` ok; commit `e813449b` |
| 1 | Lexer: token-dump equality on the corpus (`--dump-tokens`) | **DONE 2026-10-02** | Phase 1; `selfhost/src/lexer.xi` ports `crates/xiom-lexer`; harness gate `full_diff_tests::diff_tokens` green over the 83-file corpus (3/3 tests, 72.7 s); torture parity (BOM/CRLF/NUL/bigints/suffix quirk) clean; checklist `docs/checklists/selfhost-phase1.md` |
| 2 | Parser: AST-dump equality on the corpus (`--dump-ast`) | NOT STARTED | Phase 2; largest single phase (statements/exprs -> types -> patterns -> modules -> contracts -> generics) |
| 3 | Checker: diagnostics + type-annotation equality | NOT STARTED | Phase 3; same accepted/rejected set + same message order/text |
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
# Phase 0 gate (T1 green); Phase 1 gate (diff_tokens) runs in the same suite
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
```

Long suites on this box: e2e `-- --test-threads 12`, stdlib_tests 8,
`scripting_tests` 4.
