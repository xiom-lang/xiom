# XIOM Handoff -- 2026-09-14 (compiler lane; rounds 55-59 in docs/SESSION.md)

Branch `feat/architect`. Last compiler commit `0c1e3dff`. Working tree clean
except the generated `.xiom_ai.json`. The parallel stdlib session commits to
the same branch; never stage their `stdlib/**`,
`examples/stdlib_smoke/**`, `docs/stdlib_session.md`,
`docs/STDLIB_READINESS_PLAN.md`.

## State at handoff

- **Stage 3 Item A**: corpus gate `catalog_corpus_is_clean` runs UN-IGNORED
  and green (checker 188/188). The isolated corpus is clean.
- **Strict flip**: `strict_catalog_findings` is currently **false (HELD)**
  after two hold/release cycles. Both hold reasons are now believed fixed
  (R19 + alias isolation; stdlib's section-Q/bare-name burn-down: 509/509
  per-module probes clean, r40 937/937). **The first task is to re-test and
  finalize the flip.**
- **Fixed with locks**: R14 (`e2e_m71_concat_index_elem`), R17
  (`e2e_m72_nested_index_concat`), R19 (`e2e_m73_ptr_replace_str`), R15
  catalog delegation (temp-shim probes green; encoding-family dedup can
  re-land).
- **Alias isolation**: `flush_catalog_bodies` checks each catalog body under
  its own `use` bindings only (no user-alias leakage).
- **Last gates (with flip held)**: checker 188/188, feature-reg 510/510,
  stdlib-exec 85/85 (+2 ign), e2e **2320/2320**, xiom-ast 9/9, xiom 20/20,
  fmt 83/83, lsp 42/42, jit 5/5, `cargo check --workspace` clean.

## Immediate task: FLIP FINALIZATION

1. In `crates/xiom-check/src/lib.rs` set `strict_catalog_findings: true`
   (update its comment; it is around line 271 in `Checker::new`).
2. `cargo build -p xiom` then:
   - `cargo test -p xiom-check` (expect 188/188, gate live)
   - `cargo test -p xiom-codegen --test stdlib_execution_tests` (expect 85/85;
     this is the REAL-compile strict gate)
   - targeted: `e2e_m34_j08`, `e2e_m65_str_method_sugar`, `e2e_m71/m72/m73`
   - full `cargo test -p xiom-codegen --test e2e_tests` (expect 2320/2320)
3. If green, commit "feat(checker): FLIP -- catalog findings are hard
   errors" and mark Item A CLOSED in docs/COMPILER_BUGS.md + docs/SESSION.md.
   If a smoke/test fails, capture the site, revert the flag to false, and log
   it in COMPILER_BUGS with the module:line and the binding conflict.

## Remaining queue (compiler lane)

1. **R20** (top bug): Result-returning same-leaf delegation delivers an
   empty-payload `Err` and the shim smoke AVs (`p_b32_residual`). Blocks the
   encoding-family dedup. Evidence in docs/COMPILER_BUGS.md R20.
2. **R18**: contract false positive `result.value.len() <= s.len()`
   (constant-bound payload form passes); negative lock logged.
3. **R16**: `ptr + int` in a call argument miscompiles (stdlib uses an
   Int-cast workaround); latent.
4. **R15b**: same-leaf + same-name modules declared in the USER program
   (catalog case is fixed; no catalog recording for user modules).
5. **Order-independent resolution** (only if the flip re-test fails on
   bare/method load-order classes): make bare/method resolution scope-first
   (current module + explicit imports before the global first-wins table) so
   the corpus is a faithful superset gate.
6. **Stage 5 remainder**: LSP incremental reparse + cross-file index; dbg
   async MI reader + `.xi` DWARF; cargo-fuzz targets over lexer/parser/CTFE +
   ASAN/UBSAN CI (`--sanitize=address` is wired and verified -- runtime at
   `C:\Program Files\LLVM\lib\clang\22\lib\windows`); full clap migration of
   the driver parser; supply chain (ed25519 + trust model, lockfile v2 with
   enforced `--locked`, git deps pinned to commits, authenticated publish);
   sandbox false-green + randomized temp names.
7. **Stage 6 performance**: incremental engine tiers, parallel monomorphization,
   linker strategy, benchmark CI budgets.
8. **Stage 7 selfhost**: zero-ICE self-build, >=1M fuzz execs, `-O`
   differential, release-binary suites, Rust-bootstrap equivalence.

Full ledger: `docs/COMPILER_BUGS.md` (top entries are the newest).
Round history: `docs/SESSION.md` (rounds 38-59).

## Workflow rules

- e2e/stdlib harnesses spawn `target/debug/xiom.exe`: run
  `cargo build -p xiom` after ANY checker/codegen change before e2e runs,
  or the suite tests a stale driver (this has produced false failures).
- Corpus triage: `cargo test -p xiom-check catalog_corpus_is_clean
  -- --ignored --nocapture` (currently un-ignored, so plain `cargo test -p
  xiom-check` runs it); `$env:XIOM_CATALOG_DUMP='1'` prints every site.
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
38-59 + the remaining queue), docs/COMPILER_BUGS.md (newest entries first)
and docs/ITEM_A_STDLIB_FINDINGS.md before touching code. The stdlib session
works in parallel on stdlib/** only and commits to the same branch; never
stage their files.

State: rounds 55-59 landed. R14/R17/R19 fixed with e2e locks (m71/m72/m73);
R15 catalog delegation fixed; per-body alias isolation in
flush_catalog_bodies; corpus gate un-ignored and green (checker 188/188);
e2e 2320/2320, feature-reg 510/510, stdlib-exec 85/85 (+2 ign) with the
strict flip HELD (strict_catalog_findings=false). The stdlib lane's latest
report: section-Q/bare-name burn-down complete (509/509 per-module import
probes clean, r40 937/937, corpus green), so the flip's blockers are
believed resolved.

Your task, in order:
1. FLIP FINALIZATION: set strict_catalog_findings=true in Checker::new
   (crates/xiom-check/src/lib.rs, update the comment), rebuild the driver,
   then run: cargo test -p xiom-check; cargo test -p xiom-codegen --test
   stdlib_execution_tests; targeted e2e_m34_j08/e2e_m65_str_method_sugar/
   e2e_m71/e2e_m72/e2e_m73; then the full e2e suite (expect 2320/2320).
   If green, commit the flip and mark Stage 3 Item A CLOSED in
   docs/COMPILER_BUGS.md + docs/SESSION.md. If anything fails, capture the
   exact module:line + binding conflict, revert the flag, and log it.
2. R20: Result-returning same-leaf delegation empty payload + shim AV
   (docs/COMPILER_BUGS.md R20) -- blocks the encoding-family dedup.
3. R18 (contract false positive), R16 (ptr+int arg), R15b (user-program
   same-leaf modules).
4. Then Stage 5 remainder (LSP index, dbg DWARF, fuzz+ASAN CI [wired:
   --sanitize=address], clap migration, supply chain), Stage 6 performance,
   Stage 7 selfhost.

Rules: the e2e/stdlib harnesses spawn target/debug/xiom.exe -- always
`cargo build -p xiom` after checker/codegen changes. Capture $LASTEXITCODE
right after each native command. Use --emit-ir / --sanitize=address for
miscompile work. Keep commits atomic (code + docs together) and never touch
stdlib/**.
```
