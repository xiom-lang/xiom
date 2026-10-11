<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Rustc Study 07 -- Selfhost bootstrap + test infra (rustc vs XIOM)

**Date:** 2026-10-11 | **Scope:** src/bootstrap (stages, caching, thresholds),
src/tools/compiletest (UI harness, --bless, run-make), ICE discipline
(rustc_driver_impl, rustc_errors), determinism/reproducibility practices.
**rustc source:** `E:\repos\rust` @ `7211ea594cbf137ef6118b1752b959dea7cb9088`
(sparse: compiler/ + src/bootstrap + src/tools/compiletest; read-only).
**XIOM refs:** main @ `495d6672` (this worktree) and the selfhost lane
`selfhost-phase-4-codegen` @ `6417ca7b` (phase 6 P1b partial). Every path:line
below was read in these trees unless marked "(via `git show <ref>`)" for
objects that are not checked out in the study worktree.

ASCII only. Study only -- no code changes.

---

## 1. Planning input

Tagged against the XIOM plan stages: [S6] = Compiler Readiness Stage 6
(performance program), [S7] = Stage 7 selfhost gate, [S8-9] = the selfhost
stages (byte-identical IR, self1 == self2 by sha256, bootstrap chain).

- [S7] **A trustworthy selfhost chain starts with a pinned, hashed seed.**
  rustc commits `src/stage0`: a generated manifest that fixes the bootstrap
  compiler's exact date/version and the sha256 of every downloadable artifact;
  `bootstrap.py` verifies those hashes before extracting. XIOM has no
  equivalent pin: today the "seed" is whatever `target/debug/xiom.exe` the
  tree happens to build.
- [S7] **Gates must be frozen, machine-checked manifests, not counts in a
  doc.** rustc's compiletest compares against checked-in `.stderr` goldens;
  XIOM already does this for checker diagnostics (`selfhost/tests/
  check_negative/*.expected`) and IR (T3 byte equality) -- the pattern works;
  it needs to become the norm for every new gate (crash, differential,
  bootstrap-chain).
- [S7] **Crash triage is infrastructure, not debugging by hand.** rustc turns
  any panic into an ICE report file (version, platform, flags, query stack)
  plus a bug-report URL, and keeps a `tests/crashes` corpus of known crash
  reproducers. XIOM's REL-20261010-1854 (a raw ACCESS_VIOLATION with no
  report) was diagnosed only by manual binary archaeology; the missing piece
  is the pipeline.
- [S7]/[S8-9] **Determinism checks are relative comparisons between
  artifacts, not self-consistency.** rustc checks the parallel frontend by
  comparing its output against the serial one (line-order tolerant) and by
  re-running with `--iteration-count`. XIOM's existing
  `perf_determinism_ir_is_byte_identical` only proves "same binary, two runs";
  the selfhost gate needs "xiom.exe vs self1 vs self2, same corpus, zero byte
  diffs" as one scripted gate.
- [S8-9] **The bootstrap chain itself must be a scripted, hash-checked gate.**
  Phase 7 (`self1 = xiom.exe(selfhost/src/main.xi)`,
  `self2 = self1(selfhost/src/main.xi)`,
  `sha256(self1) == sha256(self2)` plus T3 of self1-vs-self2 IR) has no
  runner today; the only binary-hash bootstrap precedent
  (`test_selfhost_bootstrap_v050`, `crates/xiom-codegen/tests/diff_tests.rs:
  250-289`) is `#[ignore]`d and checked a structural count, not bytes.
- [S6] **Bootstrap-chain performance needs budgets and reuse.** rustc's
  bootstrap caches every expensive step behind content/smart stamps and skips
  unchanged compiletest cases; XIOM's T3 suite is currently a full,
  uncached sweep (6 tests, ~6-13 min per full run) and O2's targets (<60 s
  self-compile, within 2x of xiom.exe) are not measured anywhere yet.
- [S7] **A golden-diagnostic system needs a sanctioned regeneration path.**
  rustc's `--bless` rewrites `.stderr` files and review happens on the diff;
  XIOM's `.expected` files are hand-maintained today (the phase-4 rebase had
  to port m198 and hand-edit an `.expected`). Add a `--bless`-style tool so
  goldens cannot rot silently.
- [S7] **The -O0/-O2/-O3 x clang/gcc differential is a first-class suite in
  rustc terms.** compiletest `--compare-mode` exists precisely to run the
  entire suite under a second configuration; XIOM's Stage 7 item is the same
  shape and should reuse the corpus + a per-case manifest.
- [S7] **A >=1M-exec fuzz claim needs a checked-in corpus and a promotion
  policy.** XIOM's `fuzz/` workspace has 4 seed files and one artifact
  directory; rustc's answer to "crash found" is a minimized reproducer in
  `tests/crashes` with a `//@ known-bug: #N` annotation, moved to `tests/ui`
  when fixed.
- [S8-9] **Stage-scoped assertions prevent false progress.** rustc refuses
  `x test --stage 0` (it would test the prebuilt compiler), defaults test
  stage per config, and gate commits reference the stage. XIOM's T3/T6
  manifests (`T3_FILES == SCALAR_FILES`, self-arming `T6 == PHASE6`) already
  do this for the emitter; the same discipline should gate self1/self2 runs
  (run the suite on the *self-built* binary, not only on the Rust-hosted one).

## 2. How rustc does it

### 2.1 The stage model (`src/bootstrap`)

- `src/bootstrap/README.md:22-41`: entry script downloads the stage0
  compiler/Cargo, builds the bootstrap binary, then: "stage 0 ... used to
  build the stage 1 compiler, which links against the stage 0 standard
  library. The newly built stage 1 compiler is then used to build the stage 1
  standard library. After that, the stage 1 compiler is used once more to
  produce the stage 2 compiler, which links against the stage 1 standard
  library."
- `src/bootstrap/src/core/compiler.rs:11-45`: a `Compiler` is `(stage, host)`;
  `is_snapshot()` means `stage == 0 && host == host_target` -- the downloaded
  seed, not a build.
- `src/bootstrap/src/core/build_steps/compile.rs:986-1003`: the `Rustc` step
  takes the **previous** compiler as `build_compiler`; `:1077` computes
  `let stage = build_compiler.stage + 1`.
- Two reuse levers: `--keep-stage` (`compile.rs:1066-1074`) keeps an old
  compiler with a loud warning; "uplifting" (`compile.rs:1083-1090`) reuses
  the stage2 compiler for stage3+ when `full_bootstrap` is off, because
  "we cannot uplift stage 1, as it has a different ABI than stage 2+".
- `src/bootstrap/src/core/config/config.rs:1198-1204`: default doc/build/test
  stages are `stage 2` when `rust.download-rustc` (CI) is used, otherwise
  `stage 1`; `download-rustc` (`config.rs:615`, `:1737-1749`) fetches a
  CI-built rustc instead of rebuilding one.
- `src/bootstrap/src/core/build_steps/test.rs:2325-2503`: the `Compiletest`
  step passes `--stage`, `--stage-id` (`stageN-<target>`), rustc/rustdoc
  paths, sysroot lib dirs; `:2343-2350` refuses `--stage 0` with an
  explanatory error ("runs compiletest on the stage0 (precompiled) compiler,
  not your local changes").

### 2.2 The x.py flow and the stage0 pin

- `src/bootstrap/bootstrap.py:52-90,181-195`: every downloaded component is
  checked against a sha256 from the `src/stage0` manifest before use
  ("invalid checksum: found ... expected ...").
- `src/stage0` (via `git show HEAD:src/stage0`): `compiler_date=2026-09-29`,
  `compiler_version=beta`, per-artifact lines such as
  `dist/2026-09-29/rustc-beta-x86_64-pc-windows-msvc.tar.gz=
  fd2d7a0809aa431f3f2365f6a98901f359514d47967067c936ba68bc47437d5c`,
  plus cargo/rustdoc/rustfmt pins. The file is regenerated by
  `./x.py run src/tools/bump-stage0` (comment above the artifact list).
- `src/bootstrap/README.md:43-169`: directory contract -- `build/cache`
  (tarballs), `build/bootstrap` (the build system itself), then per-host
  `x86_64-unknown-linux-gnu/stage{0,1,2}`, `stageN-{std,rustc,tools}`, and
  `stage1/`/`stage2/` as complete sysroots linked with hard links.

### 2.3 Caching, stamps, fingerprints

- `src/bootstrap/src/utils/build_stamp.rs:20-107`: a `BuildStamp` is a file
  whose content is the stamp string; `is_up_to_date()` is byte equality of
  content. `libstd_stamp`/`librustc_stamp`/`codegen_backend_stamp`
  (`:136-164`) name the per-artifact stamps.
- `build_stamp.rs:181-212` (`generate_smart_stamp_hash`): sha256 over
  `git diff .` + `git status --porcelain -z --untracked-files=normal` +
  extra input -- the fingerprint for expensive external builds (LLVM: 
  `build_steps/llvm.rs:224,1326,1552,1806`; gcc: `build_steps/gcc.rs:239`).
- `src/bootstrap/src/core/build_steps/compile.rs:2650-2820` (`run_cargo`):
  runs cargo with JSON output, collects every produced artifact, and writes
  the *dependency list* into the stamp (NUL-separated, typed `h`/`t`/`s`).
  Because cargo artifact filenames embed a content hash, a changed input
  produces a new filename and the stamp mismatches -> rebuild.
- In-memory step graph: `src/bootstrap/src/core/builder/mod.rs:1573-1671`
  (`ensure`) caches step outputs, detects cycles, and supports
  `--print-step-timings` (`:1618-1655`).
- compiletest has its own up-to-date layer:
  `src/tools/compiletest/src/lib.rs:568-599` skips a test when the stamp file
  (a) matches `compute_stamp_hash(config, variant)` and (b) is newer than
  every input in `files_related_to_test` (`:501-561`, including auxiliary
  files and the expected `.stderr`); `--force-rerun` overrides.

### 2.4 compiletest: UI harness, --bless, crash tests, run-make

- Architecture: `src/tools/compiletest/src/lib.rs:403-493` -- one test file
  expands into one `CollectedTest` per revision (and per debugger for
  debuginfo); directives are parsed up front
  (`src/tools/compiletest/src/directives.rs`), and `ignore-*`/`only-*`/
  `needs-*` decisions are made per file/revision.
- UI runs (`src/tools/compiletest/src/runtest/ui.rs:16-143`): compile the
  test, extract/compare stderr/stdout snapshots, optionally run it, and on
  mismatch print "To update references, rerun the tests and pass the
  `--bless` flag" (`ui.rs:127-143`).
- Golden update: `src/tools/compiletest/src/runtest.rs:2797-2918`
  (`compare_output`) -- without `--bless` it writes the actual output to the
  build dir and shows a diff; with `--bless` it writes the actual output over
  the expected `.stderr`, prints "Blessing the {stream} ...", and returns
  `CompareOutcome::Blessed`; an empty actual output deletes a stale golden
  (`:2811-2813`). The comparison can be exact, line-set tolerant
  (runner subset, `:2828-2852`) or sorted-lines (`compare-output-by-lines`,
  `:2853-2862`) -- the latter is how non-deterministic orderings are tamed.
- Stable diagnostics: UI/incremental invocations add `-Zui-testing`
  (`runtest.rs:1786,1796`), which among other things anonymizes line numbers
  (`compiler/rustc_errors/src/annotate_snippet_emitter_writer.rs:528-543`);
  UI tests also get `-Ccodegen-units=1`, `-Zdeduplicate-diagnostics=no`,
  `-Zwrite-long-types-to-disk=no`, `-Cstrip=debuginfo`
  (`runtest.rs:1794-1800`) so snapshots do not depend on codegen-unit
  partitioning or debug info. `RUSTC_ICE=0` is set for test runs
  (`directives.rs:285,396`) so golden runs never litter ICE files.
- Crash tests: `src/tools/compiletest/src/runtest/crashes.rs:4-24` -- compile
  and require a *crash unless fixed*: if the compiler exits 0 or 1 the test
  fails with "crashtest no longer crashes/triggers ICE, hooray! Please ...
  move it to tests/ui". The rustc dev guide (tests/compiletest#crash-tests)
  states the contract: "A test will pass if rustc exits with an exit status
  other than 1 or 0", files are named after the issue number, and carry
  `//@ known-bug: #NNNN`. `tests/crashes/` holds hundreds of these
  (e.g. `100041.rs`, `105238-1.rs`, `111419.rs`).
- run-make: `test.rs:2422-2503` links `run_make_support` into
  `tests/run-make/<dir>/rmake.rs` recipes and hands compiletest
  `--run-make-support-rlib`; `runtest/run_make.rs:289-297` propagates
  `RUSTC_BLESS_TEST` so recipes can bless their own snapshots. Recipes are
  compiled with the stage0 compiler ("may not use any nightly/unstable
  features"), and `COMPILETEST_FORCE_STAGE0=1` exists for fast iteration.
- Suite taxonomy (dev guide, tests/compiletest): `ui`, `ui-fulldeps`,
  `pretty`, `incremental`, `debuginfo`, `codegen-*`, `assembly`, `mir-opt`,
  `coverage*`, `crashes`, `run-make`/`run-make-cargo`. Compare modes
  (`--compare-mode=next-solver|polonius|split-dwarf|...`) run the same suite
  under a second configuration; `--parallel-frontend-threads` plus
  `--iteration-count=M` verifies the parallel frontend against the serial
  one and implies `compare-output-by-lines` for all tests.

### 2.5 ICE discipline

- Entry point: `compiler/rustc_driver_impl/src/lib.rs` -- driver main installs
  `install_ice_hook` (`:1459`); a panic runs the default hook, then appends
  the panic message and a forced backtrace to the ICE file when one is
  configured (`:1501-1523`), then `report_ice`.
- ICE report: `report_ice` (`:1538-1626`) emits `error: internal compiler
  error: unexpected panic` / `...unexpected error` (`diagnostics.rs:55-109`),
  the bug-report URL (`DEFAULT_BUG_REPORT_URL`, `lib.rs:117`, linking to the
  `I-ICE` issue template), a nightly-upgrade note, then the version, host
  platform, and the full compiler flags (`IceVersion`/`IceFlags`); it writes
  the same info plus the query stack into
  `rustc-ice-<YYYY-MM-DDTHH_MM_SS>-<pid>.txt` (path set by `RUSTC_ICE`,
  default cwd, `RUSTC_ICE=0` opts out; `:1411-1443`), and prints
  "please attach the file at `...` to your bug report". For bootstrap
  debugging, `RUSTC_BREAK_ON_ICE` raises `DebugBreak()` on Windows
  (`:1621-1625`).
- Bug vs error classification: `dcx.bug`/`dcx.span_bug`
  (`compiler/rustc_errors/src/lib.rs:969,983`) are the unconditional-ICE
  paths; `delayed_bug` (`:1071`) records a latent bug; `ErrorGuaranteed`
  (`compiler/rustc_span/src/lib.rs:2858`, "cannot be constructed outside of
  this crate") is the token downstream passes require so recovery cannot
  cascade. `catch_fatal_errors`
  (`compiler/rustc_span/src/fatal_error.rs:33`) converts explicit fatal
  errors into a failure exit; ordinary ICE panics unwind to the runtime's
  default panic exit status, which is why crash tests assert "not 0 or 1".
- The ICE message is deliberately actionable: it names the URL, the file to
  attach, and the version -- everything the reporter needs, nothing they must
  remember.

### 2.6 Determinism and reproducibility

- Path remapping: `--remap-path-prefix` plus `-Zremap-path-scope`
  (`compiler/rustc_session/src/options.rs:414-417`) with explicit scopes
  `MACRO`, `DIAGNOSTICS`, `DEBUGINFO`, `COVERAGE`, `DOCUMENTATION` and the
  aggregate `OBJECT = MACRO|DEBUGINFO|COVERAGE`
  (`compiler/rustc_span/src/lib.rs:238-256`); `RealFileName` tracks which
  scopes were remapped so the same file can present different names to
  diagnostics vs debug info vs `file!()`.
- Dist builds: `rust.remap-debuginfo` (docs at
  `rustc_session/src/options.rs:423-434`; bootstrap config `config.rs:223`,
  default false `:1573`) maps the source root into the virtual
  `/rustc/$hash` path; LLVM's own debug info gets
  `-fdebug-prefix-map=$src=/rustc/llvm` (`build_steps/llvm.rs:814-833`); the
  sysroot carries a source symlink so `/rustc/$hash` resolves in tests
  (`compile.rs:2035-2060`), and std remapping is threaded to test commands
  (`test.rs:2845-2846 --with-std-remap-debuginfo`).
- Golden-output stability levers (2.4): `-Zui-testing` (anonymized line
  numbers), `-Zdeduplicate-diagnostics=no`, `-Ccodegen-units=1`,
  `compare-output-by-lines`, runner-subset comparison.
- Parallelism is verified, not assumed: `--parallel-frontend-threads=N`
  with `--iteration-count=M` compares parallel output against serial
  (dev guide, tests/compiletest#parallel-frontend).
- Note what is *not* in the compiler: `SOURCE_DATE_EPOCH` does not appear in
  compiler/ (grep found none); rustc's compiler-side determinism story is
  path/scoping hygiene, while tarball/timestamp determinism lives in
  packaging. XIOM already uses `SOURCE_DATE_EPOCH` in its package writer
  (`crates/xiom-pkg/src/tarball.rs:29-44`).

## 3. How XIOM does it today

Verified against `docs/SELFHOST_PLAN.md`, `docs/SELFHOST_PROGRESS.md`,
`crates/xiom-codegen/tests/full_diff_tests.rs` (main, 940 lines), the selfhost
lane harness (via `git show selfhost-phase-4-codegen:...`, 1591 lines),
`crates/xiom-codegen/tests/{diff_tests,e2e_tests,fuzz_tests,robustness_tests,
perf_budget_tests}.rs`, `fuzz/`, `.github/workflows/{ci,heavy}.yml`, and
`selfhost/`.

### 3.1 The parity gate: full_diff_tests.rs

- Tiers (main: `full_diff_tests.rs:5-50,628-634`): T1 = well-formed modules +
  min fn counts + fn-name overlap; T2 = normalized IR equality
  (`%tmpN`/`@.strN` stripped, `:526-556`); T3 = exact line-by-line IR
  equality. Tier is selected with `XIOM_SELFHOST_DIFF_TIER=1|2|3`
  (default 1); tiers stack.
- Corpus: hardcoded, sorted, 83 files (`:81-202`) across
  `tests/regression/m37_*`, `examples/catfix/*`, `examples/phase1_*`,
  extra examples, and one stdlib smoke file -- no runtime globbing.
- Drivers: `xiom.exe --emit-ir` vs the selfhost compiler built once per test
  process from `selfhost/src/main.xi` into `target/selfhost/` (`:327-354`);
  IR is compared via `first_diff` which reports the first mismatching line
  (`:558-577`). `diff_corpus` fans out over 2-4 worker threads.
- Canonical-dump gates each own a format definition mirrored by the selfhost:
  `--dump-tokens` (`diff_tokens`, `:742-790`), `--dump-ast`
  (`diff_ast`, `:792-845`), `--dump-check` (`diff_check`, `:847-991`).
  Format owners: `crates/xiom/src/main.rs::dump_tokens` (`main.rs:295`),
  `dump_ast` (`main.rs:1289`), `dump_check` (`main.rs:1338`).
- Golden diagnostics: `diff_check` asserts the 75 committed
  `selfhost/tests/check_negative/**/*.expected` files against BOTH drivers
  (`:945-975`), plus non-vacuity re-assertions of real corpus diagnostics
  (W003 on the smoke fixture, W008 on `m37_short_circuit`, E001 borrow
  warnings on `m37_f128`, `:886-943`).
- On the selfhost lane (not yet on main): the harness adds
  `SCALAR_FILES`/`T3_FILES` (`:228-252`), `PHASE6_FILES`/`T6_FILES`
  (`:277-347`), the `SCALAR xor PHASE6` partition assert (`:398`), the
  "T2/T3 run only over ported sets" scoping (`:877`), the self-arming
  `T6 == PHASE6` gate, and `diff_ir_headers` (`:1361`) with
  `ir_header_lines`/`strip_inline_attr` (`:1294-1310`) -- the Phase 4 header
  gate (banner, define signatures, tuple defs, inline attrs, declares).
- State (branch `6417ca7b`, per `docs/SELFHOST_PROGRESS.md`): Phase 5 T3
  19/19 scalar byte-exact; Phase 6 P1c landed, T6 2/64; meter 55% (6 of 11).

### 3.2 Bootstrap chain status

- `docs/SELFHOST_PLAN.md:88-99` defines the chain gate:
  `self1 = xiom.exe(selfhost/src/main.xi)`;
  `self2 = self1(selfhost/src/main.xi)`;
  `sha256(self1) == sha256(self2)`.
- `docs/SELFHOST_PROGRESS.md:180-182` (main tracker): gate 7 "Self-compile
  chain: self1 == self2 sha256 + T3 on self1-vs-self2 IR" is **NOT STARTED**
  (Phase 7); gate O2 (bootstrap perf) also NOT STARTED.
- Existing binary-hash precedent: `tests/diff_tests.rs:250-289`
  (`test_selfhost_bootstrap_v050`) is `#[ignore]`d ("exceeds the 300s compile
  timeout") and asserted a structural hash `431327` of the obsolete
  `selfhost/xiomc_v050.xi`, not a byte hash of a self-produced compiler.
- `XIOM_SELFHOST=1`-gated e2e tests (`e2e_tests.rs:621-679`, `:708-733`)
  only check that v10/v11 "self-compile" emits plausible IR (fn counts,
  `define i64 @main`), not equality between artifacts.
- `selfhost/batch_test.ps1` is a stale `.ax`-era v10 patch script
  (it string-replaces a target path in `xiomc_v10.ax`); it is not the
  current harness and does not gate anything.

### 3.3 Other suites that a selfhost gate would ride on

- e2e (`e2e_tests.rs`, 6986 lines): compile a `.xi` to a native binary and
  run it, asserting exit codes (with a documented race-retry for a
  concurrently rebuilt `xiom.exe`). The selfhost binary is expected to pass
  the same suite (plan section 7 item 3), but no harness parameterizes the
  compiler-under-test today.
- fuzz/robustness (`fuzz_tests.rs`, `robustness_tests.rs`): in-process
  pipeline inside `catch_unwind`, asserting "never panic" on adversarial
  input (fuzz_tests.rs:24-60). 24/24 fuzz-regression cases green.
- perf budgets (`perf_budget_tests.rs`): deterministic emitted-IR byte
  budgets and `perf_determinism_ir_is_byte_identical` (`:114-135`) -- same
  source compiled twice must be byte-identical (the HashMap-order canary).
- CI: `ci.yml` runs hygiene (MSRV check, cargo-deny/vet), unit tests,
  robustness+fuzz suites, perf budgets, stdlib API freeze, front-end locks,
  and the e2e subset; `heavy.yml` runs cargo-fuzz smoke (30 s/target, ASAN,
  nightly) and ASAN-compiled program smokes, weekly. `fuzz/` is a standalone
  cargo-fuzz workspace with four targets (`lexer`, `parser`, `ctfe`,
  `pipeline`) and four checked-in seed files; minimized findings are supposed
  to be added to `fuzz/corpus/<target>/` (`fuzz/README.md:48-53`).
- Determinism evidence today: `perf_determinism_ir_is_byte_identical`
  (same-binary repeat), T2/T3 (Rust-hosted vs selfhost emitter), and
  catalog-order fixes (`crates/xiom-check/src/catalog.rs` documents
  deterministic winner rules). There is no cross-stage (self1 vs self2)
  determinism run.

### 3.4 What is absent relative to rustc

- No stage0 seed pin/manifest and no x.py analogue: no `tools/bootstrap.*`
  that downloads or hashes a seed compiler, builds stages, records artifacts,
  or re-runs gates on the self-built binary.
- No ICE pipeline: grep found no panic hook / ICE file / bug-report URL in
  `crates/xiom/src` or `crates/xiom-codegen/src`; a compiler panic is a
  plain Rust panic, and a *generated program* crash (REL-20261010-1854) is a
  raw OS exception with no compiler-side report.
- No crash corpus and no promotion policy (only 4 fuzz seeds; no
  `tests/crashes`-style directory; no `known-bug` convention).
- No --bless workflow for the `.expected` goldens (hand-edited).
- No differential matrix suite (-O levels, clang/gcc) and no compare-mode
  analogue; Stage 7 lists it as a to-do.
- No stage-scoped test invocation: all suites run against the Rust-hosted
  binary; nothing refuses or labels a run that used a stale/self-built
  artifact (rustc's `--stage 0` refusal is the opposing discipline).
- No bootstrap-chain performance harness/budget (O2 target "self1 < 60 s,
  within 2x of xiom.exe").

## 4. Gap table

| # | Concern | rustc mechanism (paths) | XIOM today (verified) | Gap | Proposal |
|---|---------|-------------------------|-----------------------|-----|----------|
| 1 | stage0 seed equivalent | `src/stage0` pin + sha256 verify in `bootstrap/bootstrap.py:52-90`; snapshot compiler `compiler.rs:43` | seed = any locally built `xiom.exe`; no manifest | no pinned, verifiable bootstrap seed | P2: `selfhost/STAGE0` pin (release version + per-OS sha256) + a bootstrap script that verifies it |
| 2 | determinism locks (self1 == self2) | stage chain + parallel-frontend iteration checks (dev guide) + remap scopes | `perf_determinism_ir_is_byte_identical` (one binary, two runs); T2/T3 Rust-vs-selfhost | no self1-vs-self2 artifact or IR comparison | P2/P3: scripted gate `sha256(self1)==sha256(self2)` + T3 diff of self1/self2 IR over the corpus; run the e2e subset on self1 |
| 3 | golden diagnostic tests | compiletest UI `.stderr` + `--bless` (`runtest.rs:2797-2918`), `-Zui-testing` | `.expected` manifests for both drivers (`full_diff_tests.rs:945-975`); no bless tool | goldens are hand-edited; IR goldens are equality-only (no snapshot files) | P4: `tools/bless.ps1`/`--bless` for `.expected`; review diffs in the same commit |
| 4 | ICE-to-report pipeline | `install_ice_hook`/`report_ice` (`rustc_driver_impl/src/lib.rs:1459-1626`), `RUSTC_ICE`, `RUSTC_BREAK_ON_ICE`, crash suite (`runtest/crashes.rs`) | none (plain Rust panic; AV with no report: REL-20261010-1854) | crashes are untriaged, unreproducible-in-suite | P1: panic hook in the xiom driver writing `xiom-ice-<ts>-<pid>.txt` (version, flags, backtrace, input path); exit-code contract; `XIOM_ICE=0` opt-out |
| 5 | fuzz-corpus integration | `fuzz/` equivalents do not exist upstream; `tests/crashes` + `known-bug` is the crash-corpus model; fuzz smoke in rust-lang CI | 4 seed files + `fuzz/artifacts/` (ignored); `fuzz_tests.rs` case list | no accumulated corpus; no artifact promotion; Stage 7 ">=1M execs" unmeasured | P6: check in minimized corpus files; weekly heavy.yml budget with exec counts; promote crash artifacts to a crash corpus |
| 6 | differential harness (-O levels, clang/gcc) | `--compare-mode` (dev guide), codegen tests, `--parallel-frontend-threads` comparison | e2e runs one default codegen config; no -O matrix, no gcc path | Stage 7 differential item has no suite | P5: `tests/diff_matrix` (or an e2e feature): per case compile at -O0/-O2/-O3 and via clang/gcc, compare program stdout/exit + emitted-IR determinism rules |
| 7 | bootstrap perf | build stamps (`build_stamp.rs`), cargo fingerprints (`run_cargo`), `--keep-stage`, `download-rustc`, PGO/BOLT dist | T3 sweep is full each run; no budget harness | O2 unmeasured; long feedback loop (5-13 min) | P7/P8: stamp-based parity cache keyed on corpus hash + compiler hash; O2 budget test (wall time, peak alloc) on the release binary |
| 8 | stage-scoped trust | `--stage` plumbing; stage0 refusal (`test.rs:2343-2350`); suites on stage1/stage2 | suites always run the Rust-hosted binary; no stage label | a stale/self-built artifact can pass unnoticed | P3: emit the compiler identity (path+sha256+version) in every suite log; add a `--selfhost` runner mode that swaps the compiler-under-test |

## 5. Optimization & performance opportunities

The XIOM plan's O2 pass targets "self1 compile-time < 60 s; within 2x of
xiom.exe on the corpus" (`SELFHOST_PLAN.md:151-155`). What rustc's bootstrap
buys performance-wise, and the XIOM analogue:

- **Skip unchanged work with content stamps.** rustc writes a stamp whose
  content is the dependency list (`compile.rs:2808-2819`) and compiletest
  skips cases whose config hash and input mtimes are unchanged
  (`compiletest/src/lib.rs:568-599`). XIOM analogue: the T3 harness currently
  recompiles both drivers for every corpus file on every run; a stamp keyed
  on `(compiler hash, corpus file hash, tier)` would cut repeat-gate time to
  near zero for the unchanged subset, which matters exactly during the
  Phase 6/7 iterate-bless loop.
- **Fingerprint expensive toolchain inputs.** `generate_smart_stamp_hash`
  hashes `git diff` + `git status` + config for LLVM/gcc builds. XIOM's
  selfhost sources are pure XIOM; the analogue is hashing the selfhost
  source set + stdlib pin + driver version into the gate cache key.
- **Keep the fast path for the compiler being tested.** rustc's
  `--keep-stage`, `--download-rustc`, and default test-stage rules
  (`config.rs:1198-1204`) exist so day-to-day work never rebuilds what it
  does not test. XIOM analogue: cache `xiomc-self.exe` (already done per
  process) and add a persistent target/selfhost cache keyed on source hash;
  allow the parity gate to run against a prebuilt self1 during triage while
  the phase gate demands a fresh build.
- **Parallelism with deterministic comparison.** rustc verifies the
  parallel frontend instead of trusting it, and falls back to
  line-order-tolerant comparison (`compare-output-by-lines`). XIOM's
  `diff_corpus` already clamps workers to 2-4; if selfhost codegen ever goes
  parallel (plan: post-Phase-8), the IR comparison must stay exact (T3) and
  the program-output comparisons should adopt sorted/subset modes like
  compiletest.
- **Measure with budgets, not anecdotes.** rustc ships step timings
  (`--print-step-timings`) and volume metrics. XIOM analogue: wrap the O2
  measurement in a test with a generous ceiling (the perf_budget_tests.rs
  pattern already exists) so a regression is a red test, not a number in a
  doc; include both self1-vs-self2 wall time and peak memory.
- **Profile the chain, not just the binary.** O2's named hot paths (string
  concat in the emitter, hash maps, fn-table lookups) are the same class as
  the `IrBuffer`/symbol-table work in `runtime_ffi.xi`; the rustc lesson is
  that the *build system* (stamps, skip logic, caching) often dominates the
  loop, so fix the harness feedback time before micro-optimizing codegen.

## 6. Adoption proposals

Ordered by leverage on S7/S8-9. Each is concrete and testable.

1. **P1 -- Crash triage pipeline (ICE discipline).**
   - In the `xiom` driver, install a panic hook: write
     `xiom-ice-<UTC-timestamp>-<pid>.txt` (driver version, full argv, input
     path, panic message + backtrace, last diagnostic stream tail) unless
     `XIOM_ICE=0`; print the file path and a bug URL at the end.
   - Define the exit-code contract: 0 ok, 3 user diagnostics (existing),
     a distinct ICE status that is neither (document it in the CLI help and
     README).
   - Add `selfhost/tests/crashes/` (or `tests/crashes/`): one minimized file
     per known compiler crash with a header comment naming the tracking ID
     (use REL-20261010-1854 first), plus an `.expected` exit class. A fixed
     crash moves to the appropriate suite (rustc's promotion rule).
2. **P2 -- Bootstrap chain runner + stage0 pin.**
   - `tools/bootstrap.ps1` (ASCII, pure PowerShell + the built binary):
     1. verify a `selfhost/STAGE0` manifest (compiler version, per-OS
     sha256 of release archives) if it downloads a seed; 2. build
     `xiom.exe` from the tree; 3. `self1 = xiom.exe selfhost/src/main.xi`;
     4. `self2 = self1 selfhost/src/main.xi`; 5. compare sha256; 6. run the
     T2/T3 corpus with the compiler-under-test set to self1 and require the
     same results as the Rust binary; 7. write a stamp
     (`tools/.bootstrap-stamp`) with the corpus/compiler hashes.
   - Land the script in Phase 6/7; the gate command becomes one invocation.
3. **P3 -- Compiler-identity labels + stage-scoped runs.**
   - Every suite prints/asserts `compiler=<path> sha256=<hex>` at start
     (rustc's `--stage-id` analogue). Add an env switch
     (`XIOM_SELFHOST_COMPILER=<path>`) that swaps the compiler under test in
     e2e + full_diff_tests so "run the whole suite on self1" is one command.
4. **P4 -- Golden bless tool.**
   - `tools/bless.ps1` (or `xiom --bless`): regenerate the
     `selfhost/tests/check_negative/**/*.expected` files and any new snapshot
     files from the CURRENT good compiler, print a unified diff, and require
     the diff to be committed separately for review. This is the compiletest
     `--bless` discipline; without it, the m198-era hand edits will recur.
5. **P5 -- Differential matrix suite.**
   - New test target `tests/diff_matrix_tests.rs` driven by a manifest
     (source, argv variants): for each case compile with `-O0`, `-O2`,
     `-O3`, and (where available) `--cc=gcc`; run and compare stdout/exit;
     additionally require `--emit-ir` to be identical for two runs of the
     same configuration (extends the existing determinism test to the
     matrix). Fail loudly and skip-with-reason when a tool is missing
     (compiletest's `ignore-*` model).
6. **P6 -- Fuzz corpus accumulation + weekly budget.**
   - Move minimized findings from `fuzz/artifacts/` into
     `fuzz/corpus/<target>/` (the README already prescribes this; make it a
     checklist item), record the committed corpus hash, and extend
     `heavy.yml` with a total-executions budget (`-runs=N` or longer
     `-max_total_time`) whose artifact upload includes the corpus delta.
     The Stage 7 ">=1M execs crash-free" claim should quote a CI run ID.
7. **P7 -- Parity-gate caching.**
   - Add a stamp cache to `full_diff_tests` (per corpus file per driver, key
     = sha256(source) + sha256(compiler binary) + tier). Keep T1 exact and
     always-run; T2/T3 may skip unchanged pairs. This preserves the gate's
     semantics (a changed input invalidates) while cutting the 5-13 minute
     sweep to seconds on repeat runs.
8. **P8 -- O2 budget harness.**
   - `perf_bootstrap_tests.rs` (or a scripted budget): measure
     `self1(main.xi)` wall time and the ratio vs `xiom.exe(main.xi)`; assert
     < 60 s and <= 2x per the plan. Store measured values in the test log so
     the O2 pass has a trend.

## 7. Open-item relevance

**REL-20261010-1854** (`docs/COMPILER_BUGS.md`, commit `6cbcc53e` on main):
a program compiled by xiom dies at startup with AV 0xC0000005 before output;
the trigger is an UNUSED helper; the crashing binary is 341 KB -> 1.9 MB with
13,134 `%gs:` TLS accesses vs 307, and exactly two mis-lowered
`movq 0xf8, %rcx` (TLS offset used as an address); `-g` and `--release` alike;
no workaround.

- **Crash triage/ICE discipline:** this is the exact failure class the ICE
  pipeline is for. Today the compiler-side symptom is an OS exception in the
  *generated* program, so a compiler panic hook alone would not catch it; the
  pipeline needs a *program-crash triage* path as well: keep the emitted IR
  and binary alongside the exit class (compile with `--emit-ir` first, run,
  and on a nonzero "crash" class preserve `out.ll`, the exe hash, and the
  argv in the report). Section 4 row 4 and proposal P1 define the report
  artifact and the exit-code contract; REL-1854's classification (AV vs
  diagnostic) is what the crash corpus `.expected` files should encode.
- **Deterministic repro value:** the report's minimal repro (one 4-line helper
  in `tests/test_app.xi`; green in a 60-line probe; green with the site-mode
  src alone) is already a deterministic trigger. It should be frozen as the
  first crash-corpus entry (proposal P1) -- that converts "no workaround
  found" into a one-command regression lock, exactly the rustc
  `tests/crashes` move.
- **What the rustc patterns say about the root cause:** RUSTC-REFERENCE
  sections S6/S7 already map this to (a) TLS-ness set once per declaration
  (`set_thread_local_mode`; `rustc_codegen_llvm/src/consts.rs:413,542`) and
  (b) reachable-items-only emission (`rustc_monomorphize/src/collector.rs`).
  The 5.6x size growth from an unused function is the emission-cascade
  symptom; a "emit only reachable items" invariant plus the P7 stamp-hash
  idea (byte hash of green vs crashing IR) gives triage a bisectable signal:
  the two malformed loads must appear (or not) deterministically in
  `--emit-ir` output, so an IR-hash diff of the green and crashing builds is
  the first diagnostic artifact.
- Adjacent items: REL-20261010-1900 (W004 cascades -- ErrorGuaranteed
  discipline, rustc_span lib.rs:2858) and REL-20261010-1904 (fn-ptr return
  mode) are already mapped in `docs/RUSTC-REFERENCE.md`; the same triage
  pipeline should carry their repros.

## 8. Sources

rustc (all paths under `E:\repos\rust` @ `7211ea594`):

- `src/bootstrap/README.md:22-232` (stage model, directory layout, testing).
- `src/bootstrap/bootstrap.py:52-90,181-195` (download + sha256 verify).
- `src/stage0` (via `git show HEAD:src/stage0`; compiler pin 2026-09-29,
  per-artifact sha256 manifest).
- `src/bootstrap/src/core/compiler.rs:11-51` (Compiler, snapshot).
- `src/bootstrap/src/core/config/config.rs:143,223,615,890-915,1198-1204,
  1573,1737-1749` (incremental, remap-debuginfo, download-rustc, stages).
- `src/bootstrap/src/core/build_steps/compile.rs:47-131,972-1091,2035-2060,
  2650-2820` (Std/Rustc steps, keep-stage, uplift, sysroot symlink,
  run_cargo + stamps).
- `src/bootstrap/src/core/build_steps/{llvm.rs:814-833,gcc.rs:239,
  dist.rs:3112-3166,test.rs:2325-2509,2845-2846}`.
- `src/bootstrap/src/utils/build_stamp.rs:20-212` (BuildStamp,
  generate_smart_stamp_hash).
- `src/bootstrap/src/core/builder/mod.rs:1573-1671` (ensure, caching, timings).
- `src/tools/compiletest/src/lib.rs:403-599,600-729` (collections, up-to-date
  stamps, overlap/early checks).
- `src/tools/compiletest/src/runtest.rs:1775-1808,2797-2918` (UI flags,
  compare_output, bless).
- `src/tools/compiletest/src/runtest/{ui.rs:16-143,crashes.rs:1-24,
  run_make.rs:289-297}`.
- `src/tools/compiletest/src/directives.rs:234-330,905-999` (directive
  vocabulary, RUSTC_ICE=0 defaults).
- `compiler/rustc_driver_impl/src/lib.rs:117,1391-1396,1398-1443,1448-1530,
  1532-1626` (bug URL, exit codes, ICE path, hook, report).
- `compiler/rustc_driver_impl/src/diagnostics.rs:55-109` (ICE diagnostics).
- `compiler/rustc_errors/src/lib.rs:262,416,969,983,1071` (DelayedBugPanic,
  bug/span_bug/delayed_bug).
- `compiler/rustc_span/src/lib.rs:238-256,2858`, `fatal_error.rs:28-40`.
- `compiler/rustc_session/src/options.rs:405-434`.
- `compiler/rustc_errors/src/annotate_snippet_emitter_writer.rs:528-543`
  (anonymized line numbers).
- `tests/crashes/` listing (via `git -C E:\repos\rust ls-tree`), README.
- Rustc dev guide, "Compiletest" chapter (crash tests, compare modes,
  parallel frontend, run-make), fetched 2026-10-11.

XIOM (paths under this worktree; "(lane)" = `selfhost-phase-4-codegen`
@ `6417ca7b` via `git show`):

- `docs/SELFHOST_PLAN.md` (tiers, phases, O1/O2, section 7 definition of
  done).
- `docs/SELFHOST_PROGRESS.md` (main copy; gates 0-8, phase 4/5 evidence;
  lane copy adds phase 6 P0-P1b entries).
- `docs/COMPILER_READINESS_PLAN.md:201-210` (Stage 7 selfhost gate items).
- `docs/COMPILER_BUGS.md` (main): REL-20261010-1854 entry (commit
  `6cbcc53e`).
- `docs/RUSTC-REFERENCE.md` (main, commit `ecc587ee`): S1-S8 map.
- `crates/xiom-codegen/tests/full_diff_tests.rs` (main 940 lines; lane
  1591 lines with SCALAR/T3/PHASE6/T6 manifests and `diff_ir_headers`).
- `crates/xiom-codegen/tests/{diff_tests.rs:250-289,e2e_tests.rs:621-733,
  fuzz_tests.rs:24-80,perf_budget_tests.rs:114-135}`.
- `crates/xiom/src/main.rs:137,295,1289,1322-1361,1789-1791,2257-2283`
  (dump formats and flag wiring).
- `crates/xiom-pkg/src/tarball.rs:4-44` (SOURCE_DATE_EPOCH determinism).
- `fuzz/README.md`, `fuzz/fuzz_targets/*`, `fuzz/corpus/*`.
- `.github/workflows/{ci.yml,heavy.yml}`.
- `selfhost/src/` (18 modules), `selfhost/tests/check_negative/**`
  (goldens), `selfhost/batch_test.ps1` (stale), `selfhost/archive/`.
