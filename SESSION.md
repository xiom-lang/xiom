# XIOM Handoff -- 2026-09-15 (compiler lane; round 61 in docs/SESSION.md)

Branch `feat/architect`. Last compiler commit: the round-61 flip commit.
Working tree should be clean except the generated `.xiom_ai.json`. The parallel
stdlib session commits to the same branch; never stage their `stdlib/**`,
`examples/stdlib_smoke/**`, `docs/stdlib_session.md`,
`docs/STDLIB_READINESS_PLAN.md`.

## State at handoff

- **Stage 3 Item A CLOSED**: `strict_catalog_findings = true` in
  `Checker::new` (crates/xiom-check/src/lib.rs). Catalog-body findings are
  hard errors; the un-ignored `catalog_corpus_is_clean` gate (checker suite)
  is the regression canary. See COMPILER_BUGS.md R21 for the hold/release
  history and the four fixes that made the flip stick.
- **Round-61 fixes**: R21a scope-first user aliases (pre-load worklist and
  single-segment `use X as Y;` binding), R21b container-receiver method
  wildcard guard, R21c generic-`!` defer, R21d `examples/test_mod` fixture
  namespace collision. New lock `m74_user_alias_shadows_catalog` +
  `e2e_m74_user_alias_shadows_catalog`.
- **R20 FIXED (round 62)**: catalog-body same-leaf alias delegation now binds
  the checker-recorded owner-qualified target (`"{owner}#{line}:{col}"`);
  `resolve_catalog_call` trusts it first and `resolve_module_call`'s suffix
  scan is deterministic (self-excluded, longest-first). Lock
  `tests/regression/m75_alias_delegation/` + `e2e_m75_alias_delegation`;
  shim probes green; encoding-family dedup unblocked.
- **R18 FIXED (round 63)**: contract implication payload reads on the bare
  `is` rebind (`result.value.len() <= s.len()`) emitted `inttoptr i64 0`
  (len(NULL) = -1) and spuriously violated; `.value`/`.error` on a marked
  rebind now resolves to the payload itself, and Err rebinds load Result
  field 2. Lock `m76_contract_payload_param_len` +
  `e2e_m76_contract_payload_param_len`.
- **R16 FIXED (round 64)**: `ptr + int` in a call argument compiled as Str
  concatenation because extern return types were never recorded, leaving an
  unannotated `var buf = malloc(n)` untyped. Extern registration now records
  `fn_return_xiom`; `buf + len` lowers to pointer arithmetic. Lock
  `m77_ptr_plus_int_arg` + `e2e_m77_ptr_plus_int_arg`.
- **R15b FIXED (round 65)**: same-leaf/same-name USER-program modules
  (command-line multi-source) self-recursed because free fns shared one bare
  key, module aliases lost their full path, and a suffix search counted
  registration aliases. Now free fns register module-qualified keys,
  preassign qualifies every definition of a cross-module key, aliases resolve
  full-path-first, and symbol-backed candidates win. Lock
  `tests/regression/m78_user_sameleaf/` + `e2e_m78_user_sameleaf_modules`.
- **Catalog collision hardening DONE (round 66, R21d follow-up)**: the module
  index resolves same-name declarations deterministically (source-dir
  priority -> structural path match -> smallest canonical path, canonical
  identity) and reports loaded-module ambiguities as `warning[W001]`;
  `release/` trees are skipped. Unit test + manual probe; checker 189/189.
- **LSP parse cache + cross-file definition DONE (round 67, Stage 5 start)**:
  `Backend::parse_cached` (content-hash invalidation) now backs hover,
  document/workspace symbols, and definition; `textDocument/definition`
  falls back to other open documents' cached ASTs. LSP 44/44.
- **R22 FIXED (round 68)**: plain `use xiom.convert.percent;` receiver calls
  now bind the used module (`module_receiver_paths`, catalog-isolated) --
  the explicit-alias items were already fixed by R15b. m78 extended with a
  plain leaf-import leg. Percent dedup unblocked; base58 needs INT_MIN
  translation.
- **Fuzz + sanitizer CI DONE (round 69)**: standalone `fuzz/` cargo-fuzz
  workspace (lexer/parser/ctfe/pipeline + seeds); CI `fuzz-smoke` (ASAN,
  30s/target) and `sanitizer-smoke` (`--sanitize=address` binaries);
  ci.yml e2e subset now runs the m74-m78 locks and the robustness suites.
- **dbg async MI reader DONE (round 70)**: reader thread + bounded-wait
  result/event queues; non-stopping continues report "running" instead of
  hanging the DAP.
- **`.xi` DWARF DONE (round 71)**: DISubroutineType node (the old
  `type: !{}` dropped all DWARF) + per-statement `!DILocation` attachments
  under `-g`; llvm-symbolizer maps body lines (m79 lock). Default builds
  stay metadata-free. NOTE: `diff_tests::test_selfhost_v092_compiles` is red
  from the stdlib lane's in-flight string/char WIP (verified not ours).
- **Lockfile v2 + pkg parser fixes DONE (round 72)**: `xiom pkg lock` writes
  v2 {version, source, integrity} and install enforces the locked digest;
  `deps:` blocks are actually parsed now (they never were) and unbraced
  manifests stop losing every field. MCP stdlib reference renders declared
  module names. CI gained the bin-only crate tests (pkg/dbg/lsp/mcp) that
  `--lib` skipped -- which immediately caught the rotted MCP test.
- **Fixed earlier**: R14/R17/R19 with e2e locks (m71/m72/m73); R15 catalog
  delegation (checker-recorded call targets + full-path injected names);
  per-body alias isolation in `flush_catalog_bodies`.
- **Last gates (fresh canonical driver)**: checker 188/188 (corpus gate live
  and clean), stdlib-exec 85/85 (+2 ign), feature-reg 510/510, e2e
  **2325/2325**, xiom-ast 9/9, xiom 20/20, fmt 83/83, lsp 42/42, jit 5/5,
  `cargo check --workspace` clean. The stdlib lane's 1f4f0aad (encoding
  qualification) is part of that green state.

## Immediate task

Stage 5 remainder, in suggested order:
1. Supply-chain signatures: ed25519 signing/verification + trust model,
   authenticated publish, transitive closure from registry metadata.
2. fmt: body-inline comment trivia attachment (stage-2 trivia dependency).
3. cargo-vet audits.
4. clap migration of the driver parser (large; keep the CLI surface
   byte-compatible and gate with the full e2e suite).
Then Stage 6 performance, Stage 7 selfhost.

Full ledger: `docs/COMPILER_BUGS.md` (RNN entries are appended at the end;
the R22 and R21d follow-up entries are newest). Round history:
`docs/SESSION.md` (rounds 38-68).

## Workflow rules

- e2e/stdlib harnesses spawn `target/debug/xiom.exe`: run
  `cargo build -p xiom` after ANY checker/codegen change before e2e runs,
  or the suite tests a stale driver (this has produced false failures).
- If the shared target dir acts up (`cargo check` disagreeing with an
  isolated build), the documented fix is
  `cargo clean -p xiom-codegen -p xiom` then rebuild the driver.
- Corpus triage: `cargo test -p xiom-check catalog_corpus_is_clean`
  (un-ignored; `$env:XIOM_CATALOG_DUMP='1'` prints every site).
- ASAN: `target\debug\xiom.exe --sanitize=address -o out.exe src.xi` then run
  with the LLVM ASAN runtime dir on PATH.
- PowerShell: capture `$LASTEXITCODE` immediately after each native call;
  it is unreliable across pipelines/loops.
- Do not edit `stdlib/**` (parallel session owns it); report stdlib findings
  in `docs/ITEM_A_STDLIB_FINDINGS.md` / COMPILER_BUGS and in chat.

## Paste-ready prompt for the next compiler session

```
Continue the AXIOM compiler-lane readiness campaign in E:\Projects\AXIOM on
branch feat/architect. Read SESSION.md (repo root), docs/SESSION.md (rounds
38-61; round-61 is the flip closure) and docs/COMPILER_BUGS.md (newest entries
at the end: R21 flip closure, R20, R18, R16) before touching code. The stdlib
session works in parallel on stdlib/** only and commits to the same branch;
never stage their files.

State: Stage 3 Item A CLOSED -- strict_catalog_findings=true, corpus gate live
(checker 189/189). R20/R18/R16/R15b all FIXED with locks (m75/m76/m77/m78);
catalog collisions are deterministic and reported (R21d follow-up, W001); LSP
parse cache + cross-file definition landed (lsp 44/44). Last full gates:
e2e 2325/2325, stdlib-exec 85/85 (+2 ign), feature-reg 510/510, workspace
check clean.

Your task, in order:
1. Stage 5 remainder: cargo-fuzz targets + ASAN/UBSAN CI (cargo-fuzz not
   installed locally); dbg async MI reader + .xi DWARF; clap migration of the
   driver parser; supply-chain hardening (locked/--locked, pinned git deps,
   signed publish); sandbox false-green + randomized temp names.
2. Then Stage 6 performance, Stage 7 selfhost.

Rules: the e2e/stdlib harnesses spawn target/debug/xiom.exe -- always
`cargo build -p xiom` after checker/codegen changes. Capture $LASTEXITCODE
right after each native command. Use --emit-ir / --sanitize=address for
miscompile work. Keep commits atomic (code + docs together) and never touch
stdlib/**.
```
