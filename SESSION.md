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
- **Fixed earlier**: R14/R17/R19 with e2e locks (m71/m72/m73); R15 catalog
  delegation (checker-recorded call targets + full-path injected names);
  per-body alias isolation in `flush_catalog_bodies`.
- **Last gates (fresh canonical driver)**: checker 188/188 (corpus gate live
  and clean), stdlib-exec 85/85 (+2 ign), feature-reg 510/510, e2e
  **2321/2321**, xiom-ast 9/9, xiom 20/20, fmt 83/83, lsp 42/42, jit 5/5,
  `cargo check --workspace` clean. The stdlib lane's 1f4f0aad (encoding
  qualification) is part of that green state.

## Immediate task

R18 is the top queue item: contract false positive
`result.value.len() <= s.len()` (constant-bound payload form passes; negative
lock logged). R16 (`ptr + int` in a call argument) and R15b (same-leaf
same-name modules declared in the USER program) follow; then the catalog
collision hardening and Stage 5-7.

## Remaining queue (compiler lane)

1. **R18**: contract false positive `result.value.len() <= s.len()`
   (constant-bound payload form passes); negative lock logged.
2. **R16**: `ptr + int` in a call argument miscompiles (stdlib uses an
   Int-cast workaround); latent.
3. **R15b**: same-leaf + same-name modules declared in the USER program
   (catalog case is fixed; no catalog recording for user modules).
4. **Catalog collision hardening** (from R21d): two files declaring the same
   module path resolve last-insert-wins; make it deterministic and reported.
5. **Stage 5 remainder**: LSP incremental reparse + cross-file index; dbg
   async MI reader + `.xi` DWARF; cargo-fuzz targets over lexer/parser/CTFE +
   ASAN/UBSAN CI (`--sanitize=address` is wired and verified -- runtime at
   `C:\Program Files\LLVM\lib\clang\22\lib\windows`); full clap migration of
   the driver parser; supply chain (ed25519 + trust model, lockfile v2 with
   enforced `--locked`, git deps pinned to commits, authenticated publish);
   sandbox false-green + randomized temp names.
6. **Stage 6 performance**: incremental engine tiers, parallel monomorphization,
   linker strategy, benchmark CI budgets.
7. **Stage 7 selfhost**: zero-ICE self-build, >=1M fuzz execs, `-O`
   differential, release-binary suites, Rust-bootstrap equivalence.

Full ledger: `docs/COMPILER_BUGS.md` (RNN entries are appended at the end;
the R21 entry is newest). Round history: `docs/SESSION.md` (rounds 38-61).

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
(checker 188/188). R20 FIXED (owner-qualified catalog call targets +
deterministic module-call fallback; m75 lock). Last full gates: e2e 2322/2322,
stdlib-exec 85/85 (+2 ign), feature-reg 510/510, workspace check clean.

Your task, in order:
1. R18: contract false positive `result.value.len() <= s.len()` (negative
   lock logged in docs/COMPILER_BUGS.md R18); then R16 (ptr+int in a call
   argument) and R15b (user-program same-leaf modules).
2. Catalog collision hardening (R21d follow-up): two files declaring the same
   module path resolve last-insert-wins; make it deterministic and reported.
3. Then Stage 5 remainder (LSP index, dbg DWARF, fuzz+ASAN CI
   [--sanitize=address wired], clap migration, supply chain), Stage 6
   performance, Stage 7 selfhost.

Rules: the e2e/stdlib harnesses spawn target/debug/xiom.exe -- always
`cargo build -p xiom` after checker/codegen changes. Capture $LASTEXITCODE
right after each native command. Use --emit-ir / --sanitize=address for
miscompile work. Keep commits atomic (code + docs together) and never touch
stdlib/**.
```
