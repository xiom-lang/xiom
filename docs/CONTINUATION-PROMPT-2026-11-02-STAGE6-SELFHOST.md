# CONTINUATION PROMPT - 2026-11-02 (v0.65.5 shipped; orbitdb hotfix; Stage 6 + selfhost)

Read this first, then `docs/SESSION-HANDOFF.md`, then the live relay
(`E:\xiom-lang\xiom-relays`, `python tools/relay.py ...`). This file is
self-contained for the next session. Use ASCII only in repo docs
(`python tools/ascii_guard.py check` must stay green).

## 1. Current state

- Compiler repo `E:\xiom-lang\xiom`, main **1df9bd5af** (pushed):
  - v0.65.5 public compiler release, and
  - the orbitdb v0.65.5 regression fix (Vec rebind double-free).
- **Compiler v0.65.5 is published:**
  - https://github.com/xiom-lang/xiom/releases/tag/v0.65.5
  - tag v0.65.5 at 576965617; published 2026-11-02T14:38:13Z; workflow
    run 41461717955 success.
  - assets: windows-x64 zip 86a253a5..., linux-x64 cae74441...,
    linux-arm64 01652f04..., macos-x64 b6042810..., macos-arm64
    fe4cc054..., plus wasm/vsix/SHA256SUMS.
- **Stdlib v0.65.5 is published and pinned:**
  - https://github.com/xiom-lang/stdlib/releases/tag/stdlib-v0.65.5
  - tag d1e4b36 at 88f3c12, code 7b7583c, registry xiom-std 0.65.5 LIVE.
  - compiler `STDLIB_VERSION=stdlib-v0.65.5`; pin PR #12 merged at
    b2538ae5a. Compiler binary version is 0.65.5; stdlib
    COMPILER_VERSION is v0.65.4.
- **Orbitdb v0.65.5 is NOT accepted** (critical):
  - REL-20261102-1605-orbitdb: hybrid crash/recovery verify is
    0xC0000374 (STATUS_HEAP_CORRUPTION); ASan double-free in
    db.engine.db_file_open_cfg WAL replay; v0.65.4 is green on the same
    files.
  - ROOT CAUSE FOUND AND FIXED in 1df9bd5af: the v0.65.5 Vec
    overwrite-drop captured a data pointer at binding/assignment; a
    `push` (realloc) after that left it stale, and the next rebind freed
    the realloc-freed pointer. The fix frees the data pointer read from
    the CURRENT `%struct.Vec` header at the assignment, and the normal
    Vec drop frees the final header at exit.
  - Evidence: `p_vec_rebind_grow` (Vec.new, 1000 pushes, rebind) exited
    -1073740940 pre-fix; exits 0 post-fix. New soak gate
    `vec_rebind_growth` PASS; MEMORY GATES GREEN 21/21; feature
    575/575; check 214/214; e2e drop/container/param/aggregate/str/
    guard/return/unsafe/confined green.
  - **Not yet released:** v0.65.6 must be cut and orbitdb must re-run
    `scripts/hybrid_crash_test.ps1` before orbitdb accepts.

## 2. Acceptance map (relay)

Accepted on v0.65.5:
- bindings: REL-20261102-1540-bindings (archive/slot/pin 579/0,
  sanity 21/85/67, epos+MCU x2, nested-payload 0).
- packages: REL-20261102-1544-packages (archive 86a253a5 verified 3
  ways; repin 585/0; guard 579/571/8/0; epos 29/29, transport 42/42,
  orbitdb split 166/166 x2).
- xvector: REL-20261102-1716-xvector / 1718-xvector (full Win+WSL
  batteries green; pins/roadmap moved).
- pulse: REL-20261102-1717-pulse (all five platforms; arm64 closed).
- engine: official 0.65.5 gate green with saved evidence; 600 frames
  419/826 MB, slope 0.037.
- stdlib: v0.65.5 released/pinned.

NOT accepted:
- orbitdb: REL-20261102-1716-orbitdb (verdict) + REL-20261102-1605
  (critical blocker); v0.65.4 remains orbitdb's last accepted release.

Benchmark:
- REL-20261010-1554-benchmark is still acked/open: t3-hot-reload
  trampoline floor ~220 ms (2.2 us x 100k), ~10x C/Rust/Zig on the same
  loop. `-2`/`-3` were stdlib and are verified; `-4` is answers.
- No v0.65.5 benchmark acceptance has been posted.

## 3. Work list in priority order

### P0 - v0.65.6 hotfix for orbitdb (do first)
1. Cut v0.65.6 from main (1df9bd5af or later) using the release process
   in section 4. Include the orbitdb fix.
2. Ask/orbitdb re-run `scripts/hybrid_crash_test.ps1` on the published
   v0.65.6 assets; verify hybrid crash/recovery, ASan double-free gone,
   A/B same db/WAL files green; then request the orbitdb acceptance item.
3. Close/update REL-20261102-1605 and 1716 with the release asset SHAs.

### P1 - Benchmark / Stage 6 performance (t3 trampoline)
- Read `docs/STAGE6_PERF_PLAN.md` and `docs/COMPILER_IMPROVEMENT_PLAN.md`.
- PERF-1: confined-unsafe trampoline overhead. `#[unsafe_direct]` fixed
  trusted stdlib/users; hot unsafe blocks that cannot be marked trusted
  still pay the trampoline. Candidate routes: block-level arming, callee
  classification, trampoline fast path.
- Re-measure t3 on v0.65.6, with the benchmark harness semantics from
  REL-20261010-1554-benchmark-4. Record the before/after medians and a
  lock. Target: within 10% of Rust on the benchmark categories.
- Reply on REL-20261010-1554-benchmark with the measurement and fix.

### P2 - memory follow-ups (next release after the orbitdb hotfix)
- JSON 1704 (REL-20261031-2308-stdlib): `p_json_1704_obj` at 100k =
  406/444 MB, main has 0 drops. Distinct slice: JsonValue/enum payload
  drops, `Map[Str, JsonValue]` ownership, `Result[JsonValue]` locals,
  and a fresh-Str return contract (`#[owns_str]`). Do not
  `#[consumes_str]` literal keys.
- 2117 field/whole-struct overwrite: needs an opt-in
  `#[consumes_agg]`/`#[borrows_agg]` (or uniqueness proof) plus
  recursive nested-field drop registration. The trace ring
  (`var out = r; out.events = fresh`) shallow-copies, so a blind field
  free is a UAF. Local Vec reassign is fixed in a28cb352a + 1df9bd5af.

### P3 - Selfhost Phase 6 (current, from the repo commits)
- Latest: **T6 19/64** (`7b816803f`, Option/Result ctors + variant
  patterns), T3 83/83, diff_ast 83/83, T3 suite 6/0
  (`XIOM_SELFHOST_DIFF_TIER=3`).
- Parser/AST note: the selfhost parser in
  `.kilo\worktrees\selfhost-phase-4-codegen` must mirror the qualified
  enum/struct literal dotted AST name; diff_ast/T6 was not re-run after
  the note. Apply that before the next diff_ast run.
- Remaining: 45 of 64 Phase-6 files ported by the selfhost codegen
  (continue `selfhost/src/codegen.xi`); re-run the T3/diff_ast suite.
- Then the 100% bootstrap gates (`docs/SELFHOST_PROGRESS.md` section
  "The 100% definition"):
  1. self1.exe == self2.exe by sha256.
  2. T3 byte-identical IR over the full corpus.
  3. selfhost binary passes the full e2e suite like the Rust binary.
  4. zero C-codegen dependency (`runtime_ffi.xi` in pure XIOM).
  5. zero warnings on selfhost sources under `--strict`.
  6. selfhost compile time <= 2x Rust on the corpus (O2).
  Plus O1/O2 and a rebase onto v0.65.6.
- `docs/SELFHOST_PROGRESS.md` top meter is stale (55% / 6 of 11,
  2026-10-10); the tail has the Oct 27 updates. Refresh the meter/top
  when touching the lane.

### P4 - Stage 6 lint wave
- `docs/STAGE6_LINT_WAVE.md`: tier 1 (W002/W003/W004) and tier 2
  (W006/W007/W008) landed. **Tier 3 landed OPT-IN (`--lint-strict`)** per
  the 2026-11-02 owner decision. Codes: W010 unused param, W011 unused
  import, W012 constant condition, W013 self-assignment, W014 vacuous
  integer arithmetic, W015 unused local. Default builds stay
  warning-clean. Measured over 61 examples at `--lint-strict`: 47 tier-3
  warnings (W015 30, W014 8, W011 4, W012 3, W010 1, W013 1), 21/61 files
  noisy. Keep opt-in until the 30 W015 + 8 W014 sites are triaged
  (cleanup vs suppression) and a default-on proposal is accepted.

## 4. Release process (exact, used for v0.65.5)

1. Bump `Cargo.toml [workspace.package] version` (one version for all
   crates), then `cargo check -p xiom-codegen` to update `Cargo.lock`
   workspace versions. Commit `chore: bump workspace version to X`.
2. Author `release-notes/vX.md`:
   - ASCII; `## Summary` must be ONE sentence and contain NO version
     number; highlight text must be PLAIN (no `#[...]`, no brackets/`#`).
   - Run `cargo run -q -p xiom-release-notes -- convert --tag vX
     --stdlib <stdlib checkout at the pinned tag>` to write
     `release-notes/vX.json`, then `verify` the same way.
   - The stdlib fragment at `<stdlib>/release-notes/vX.md` is merged
     automatically; use a checkout at the pinned stdlib tag and keep the
     fragment raw (no BOM).
3. Push main. The release guard requires: tag == workspace version and
   tag is an ancestor of main.
4. Create an annotated tag `vX` on the release-notes commit, push it.
   `.github/workflows/release.yml` builds linux-x64/arm64, windows,
   macos (if RELEASE_BUILD_MACOS=true), wasm, vsix and uploads the
   release; no local build.
5. Monitor `gh run list --workflow release.yml`, then verify
   `gh release view vX -R xiom-lang/xiom --json assets,url`.
6. Pin stdlib: update `STDLIB_VERSION`, commit with `-s`, and merge a
   signed pin PR (the v0.65.5 flow: PR #12, squash; `--admin` only if
   the owner policy requires it; merge commits are disabled).

Known pitfalls already hit:
- Missing `release-notes/vX.md` fails "Verify release notes".
- A BOM in the stdlib fragment fails the notes parser.
- Summary with a version number, multiple sentences, or markdown in
  highlight text fails the notes validator.
- `target/release/lib` can be a stale stdlib (v0.64.8). For probes,
  create a toolchain dir with `bin/xiom.exe` plus `lib` pointing at an
  extracted stdlib tag (e.g. 7b7583c), not the live repo junction.

## 5. Commands and probes

- Memory gates:
  `powershell -NoProfile -ExecutionPolicy Bypass -File
  tests/soak/run_memory_gates.ps1 -Compiler <toolchain>\bin\xiom.exe
  -Stdlib <stdlib-extracted> -TimeoutSec 120`
  Current: 21/21 GREEN (incl. `vec_rebind_growth`).
- Orbitdb repro:
  `E:\xiom-projects\xiom-orbitdb`; `scripts/hybrid_crash_test.ps1`;
  `src/engine.xi` lines 281-355 (`db_file_open_cfg`, `pending`
  `Vec[TxOp]` replay). Set `XIOM_COMPILER` to the published
  `xiom-0.65.6\bin\xiom.exe` and `XIOM_HOME` to the package home; ASan
  via `--sanitize=address` (0xC0000374 pre-fix).
- Compiler tests:
  `cargo test -p xiom-codegen --test feature_regression_tests`
  `cargo test -p xiom-check`
  `cargo test -p xiom-codegen --test e2e_tests <filter>`
- ASCII gate: `python tools/ascii_guard.py check` (repair with
  `python tools/ascii_guard.py repair --apply`). Relay notes must also be
  pure ASCII; `relay.py validate` must stay green.
- Relay: `cd E:\xiom-lang\xiom-relays; python tools/relay.py ...`
  Commit relay item changes with `relay.py commit --lane compiler
  --path items`.

## 6. Hard stops

- No v0.65.6 tag without the owner GO; no rewriting the published
  v0.65.5 release.
- Orbitdb must re-verify the published v0.65.6 before calling it accepted.
- No blind Str/Map/aggregate frees; a UAF is worse than a leak.
- One memory-heavy build at a time; do not swap a toolchain another lane
  is using mid-build.
- Keep docs and relay notes ASCII; no em dashes/arrows in relay URLs.

## 7. Definition of done for this cycle

1. v0.65.6 published; orbitdb hybrid crash/recovery GREEN and accepted.
2. Benchmark t3 trampoline measured on v0.65.6, fix landed or a concrete
   plan with numbers.
3. JSON 1704 and 2117 fixed (or explicitly deferred by the owner) with
   probes/regression gates.
4. Selfhost Phase 6 T6 advanced from 19/64 with the T3/diff_ast suite
   green; the 100% bootstrap gates tracked.
5. `docs/SELFHOST_PROGRESS.md` meter refreshed; relay/acceptance records
   current.

## 8. Update 2026-11-02 (later session)

- **P1 t3 CLOSED in code (not yet released).** Root cause was the POSIX
  confinement trampoline calling `sigsetjmp(..., 1)` per confined-block entry
  (a `sigprocmask` syscall, ~21 ms / 100k). Fixed in
  `E:\xiom-lang\stdlib\runtime\xiom_runtime.c`: savemask=0 on
  `xiom_trampoline_call` and `xiom_trap_enter`, with the fault handler
  unblocking the delivered signal before `siglongjmp`. Linux/WSL scored t3:
  27 ms internal / 31 ms wall -> **5 ms / 8.5 ms** (peer Rust 6 ms). Locks:
  `e2e_t3_trampoline_floor_budget`, `e2e_confined_repeat_fault_contained`
  (wired into ci.yml), fixtures under `tests/regression/`. A new stdlib tag
  and `STDLIB_VERSION` bump are required to ship it.
- **P3 AST note resolved.** The qualified-literal names already matched; the
  real diff was `NkPatField` dumping its sub-pattern one level too deep
  (`struct_depth+3` vs Rust's `+2`). Fixed in the selfhost worktree
  `selfhost/src/ast_dump.xi`; `diff_ast` is green (83 files, 12877 nodes,
  206.76 s). **T6 37/64 - non-phase-8 work complete.** `phase1_hardening.xi`
  is byte-identical (2013/2013) via enum payload derives (`Eq`/`Clone`),
  module-generic symbol spelling, invariant calls, `&mut` scalar params, and
  Option/Result pseudo-fields. Gates: diff_corpus tier 3 T3 19/19, phase-6
  37/64, T1 83/83; diff_ast 83/83; full tier-3 suite 6/6; ported-set spot
  check 56/56.
  Next: phase-8 `xiom.*` import-closure catalog/preamble emission, starting
  with `examples/catfix/b9main.xi` (`xiom.io`) and
  `tests/regression/m37_bug53_array_ref_param.xi`. All 27 remaining unported
  PHASE6 files are stdlib-catalog/preamble; no non-catalog delta remains.
- **P2 JSON 1704 fixed (R1+R3+R4).** The parse tree and the builder half are
  reclaimed. `p_json_1704_obj` peak: 7.3/25.5/45.9 MB at 1k/10k/20k (R3) ->
  **4.7/6.6/8.6 MB** (R4), i.e. a plateau. Provenance stays strict: only a
  direct `let p = json_parse(s)` frees Strings; builder/stringify trees free
  containers only (literal keys and borrowed String leaves are never freed).
  `#[consumes_agg]` on `build(self)`/`json_stringify`. Verification: 577/577
  regression, 21/21 memory gates, ASan clean, m375 22/22. **R2 also landed**
  (`#[owns_str]` + `#[borrows_str]` + `#[borrows_vec]` on `_json_buf_str`):
  `p_json_1704_obj` is now 4.6/5.0/5.5 MB at 1k/10k/20k (~0.05 KB/iter), with
  literal/borrowed strings leak-only and ASan clean. 2117 field /
  whole-struct overwrite is fixed too (unique-aggregate proof; 5 MB at 20k;
  trace-ring/alias negatives leak-only; gates 22/22).
- **Release:** no `v0.65.6` tag, no version bump. The orbitdb Vec fix remains
  on main; the t3 fix lives in stdlib.
