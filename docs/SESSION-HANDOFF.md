<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Session Handoff -- 2026-10-13 (compiler lane; v0.64.4 SHIPPED; v0.64.5 batches m276-m284 landed)

This file captures state for continuing the compiler lane in a fresh
session. Read this FIRST, then `docs/RUSTC-STUDY.md` + the relevant
`docs/rustc-study/*` area doc. rustc reference source: `E:\repos\rust`
(sparse blob-less clone @ `7211ea594`; READ-ONLY -- never run git writes
there; non-materialized files via `git show HEAD:<path>`).

## 1. Status: v0.64.4 RELEASED; stack caught up; v0.64.5 plan in section 1c

v0.64.4 shipped 2026-10-12T13:37Z (tag on `df493530`; release run
`38256358845` GREEN; assets live). The commit map that produced it
(m254-m275; all pushed) is kept below for reference:

*** SESSION UPDATE 2026-10-17T13:20Z (compiler lane; MIDDAY+ -- pin merged, m352/m359/cold-lane landed, cleanup done) -- READ FIRST ***
- PIN MERGED: **xiom-lang/xiom#7 REBASE-MERGED** 2026-10-17T12:48Z (`8fdc3292`
  on main: STDLIB_VERSION -> `stdlib-v0.64.6`). CI fully green (ubuntu rerun
  7m19s after the pre-existing all-modules order flake; the flake reproduced at
  the OLD pin too -- filed as a residual). Pin battery v2 (clean stdlib
  worktree `E:\xiom-lang\stdlib-verify-8d95815`) was INVALID -- pointing
  XIOM_STDLIB at an external stdlib while the repo-local `stdlib/` also exists
  makes the checker index DUPLICATE stdlib roots (W001 module collisions) and
  reddens warning-sensitive e2e locks (m17 etc.). Correct setup: an isolated
  compiler worktree with its own pinned `stdlib/` and NO XIOM_STDLIB:
  `E:\xiom-lang\xiom-pin-verify` (detached at `007f352dd`, nested stdlib
  worktree at 8d95815). Battery v3 running there (3 threads): e2e / feature /
  integration / stdlib suites / api-freeze / checker; log
  `%TEMP%\kilo\pin-verify-battery3.log`. Record the result on
  REL-20261016-2330-stdlib when green.
- LANDED ON MAIN (`eff579c9`, pushed): **m252 follow-up for the benchmark cold
  lanes** (REL-20261011-2100-benchmark, owner option (a)) -- `--no-cache` now
  disables the persistent catalog index cache (`catidx.txt`) and the
  catalog-body check cache (`checkbodies.txt`) READ+WRITE in the compile, AOT
  run and in-process JIT paths (`CompileConfig::no_cache`,
  `jit_execute_with`). Lock `bench_no_cache_catalog_caches` wired into ci.yml;
  m252 parity 1/1, run_script_cli 8/8, JIT lib 4/4. Benchmark notified
  (REL-20261017-1304-compiler) to re-score xiom-run-jit/aot.
- M351 HARVESTED LOCALLY, HELD FOR THE STDLIB RUNTIME: 3 commits cherry-picked
  on local main (`fcb38d7ef` fix: new c_abi.rs + call/expr/decl/emitter/lib,
  Win64 C-ABI aggregates + typed confined-fault exit-70; `a279d8679` tests:
  in-repo ros2-abi ladder/direct-extern/IR twin/no-silent-exit + e2e locks;
  `d745fab38` docs). Locks GREEN with the m351 binary: ffi_c_abi_aggregate 2/2,
  e2e m351 2/2, m244 3/3, m308_guard 3/3 (the earlier isolated-target run
  failed only because the e2e harness resolves `target/debug/xiom.exe`; do not
  use CARGO_TARGET_DIR for e2e). **NOT PUSHED**: the runtime half lives in the
  stdlib repo -- branch `fix/m351-typed-fault` (pushed; HEAD `c21ad9f`, parent
  8d95815: runtime TLS fault record + always-on diagnostic + exit 70 + typed
  `xiom.error.take_fault` + retry only for guard-page transients). The released
  pin 8d95815 lacks `xiom_fault_*`, so a compiler pinned there would fail to
  LINK confined-block programs. Coordination item
  **REL-20261017-1444-compiler** (to stdlib, HIGH): rebase onto stdlib main
  (waves 172-177), include in the stdlib v0.64.7 release, reply with tag/sha;
  then pin STDLIB_VERSION, re-run the pin battery, push m351.
- RELAY INTAKE TRIAGED (all acked + pushed): 1223-orbitdb (transient exit -1
  = host memory pressure evidence: rustc E0786 paging-file + rc -1 spawn
  failures on this box; capture-a-dump ask), 1223-orbitdb-2 (E001 over-report
  in the combined suite only; standalone shapes + block.xi are clean ->
  **m362** recon slot), 1227-packages (HIGH fn-value trio ->
  **m361 LAUNCHED**, branch fix/m361-fn-value-trio). **m353 LAUNCHED**
  (branch fix/m353-builtin-shadow-sweep: stdlib 1053 .hash + 1456 Mutex).
  Heavy slots: m351 (finishing) + selfhost Phase 6 + m361 + m353.
- SELFHOST PHASE 6 advanced: P2b-iii-a tuple field reads (.0) + tuple-typed
  locals, probe byte-exact (`f7e2585d4`), after the RD4 rebase onto the
  morning main; T6 meter continues from 13/64.
- WORKTREE/BRANCH CLEANUP: 65 -> 14 git worktrees; 54 completed fixer
  branches deleted; stale pin branches (v0.64.5/v0.61.3) deleted; selfhost
  phases 1-3 removed (0 unique commits); disk free on E: 190 GB. Kept: main,
  m351 (live), selfhost-phase-4 (live), 7 rustc-study worktrees, xiom-bench.
  Six deregistered dirs (m347-m350, m352, m359) are still file-locked by the
  VS Code extension; delete after an extension reload (their branches/registry
  are already gone).
- STUDY PROTOCOL (owner directive 2026-10-17, now enforced): `E:\repos\rust`
  (READ-ONLY sparse clone @ 7211ea594) is the battle-tested reference. Every
  design/perf/scaling/selfhost-gap/bug fix must (1) read the matching
  `docs/rustc-study/0N-*.md` area doc, (2) crawl the rustc source for the
  mechanism that already solves it, (3) record the rustc anchor in the
  COMPILER_BUGS/decision entry, (4) use benchmark relays as study questions.
  Written into `docs/RUSTC-STUDY.md` ("Protocol -- rustc-first"); both live
  fixer sessions (m361/m353) received the directive and must cite anchors.
- THREAD POOL OWNERSHIP (owner question): 1834-engine-6 is stdlib's (pool flip
  rides 1550-stdlib); the compiler-owned spawn gateway 2053 is FIXED (m282 +
  runtime wave 146 -- engine to re-verify); the remaining compiler-owned piece
  is Send/Sync ENFORCEMENT (THREADING_PLAN Phase B) -> queued **m363**.
  Recorded on REL-20261017-1438-compiler.
- The 11:10Z block below retains the morning context; superseded where it
  disagrees.

*** SESSION UPDATE 2026-10-17T12:45Z (compiler lane; MIDDAY -- pin verify + m352/m359 harvested) -- superseded by 13:20Z ***
- PIN (REL-20261016-2330-stdlib): stdlib-v0.64.6 moved to 8d95815 and PUBLISHED
  (xiom-std 0.64.6 live; publish 38979638142 SUCCESS 23:57Z). Pin PR
  **xiom-lang/xiom#7** open; the auto-bot commit lacked DCO sign-off (amended
  `d65654bd` with the bot sign-off; DCO green). First PR CI run fully GREEN; the
  re-run hit a **pre-existing all-modules-compile ORDER flake** (the freeze
  test builds `use xiom.X;` from a HashSet -> random import order; the same
  test also flaked at the OLD pin b81430ba; reproduced 1/3 old vs 3/3 pass new
  on clean checkouts). ubuntu job re-running. Pin battery at 8d95815 running on
  a CLEAN stdlib worktree (E:\xiom-lang\stdlib-verify-8d95815; the shared
  repo-local `stdlib/` is being edited by the m351 session and must not be
  used for verification): e2e / feature / integration / stdlib suites /
  checker, log %TEMP%\kilo\pin-verify-battery.log. Merge PR #7 per the 1559
  protocol when the rerun is green; record the battery result on 2330.
- HARVESTED ON MAIN (pushed `883f69bf`): m352 `001570e9` + `7d1a9cd2`
  (host-arch Target::Native table + toolchain_cmd linux-arm64 arms; unit lock
  green) and m359 `736f876f` + `883f69bf` (duplicate value flags after the
  positional -> rc 2, tolerant clap accessors; 2-test lock wired into ci.yml;
  bin unit 8/8, lock 2/2). m351 (B-33) still in flight (3 commits + uncommitted
  docs/tests in its worktree): monitor, do not harvest yet.
- SELFHOST PHASE 6 (parallel, ses_ebeb8f547ffeSvMCBgei6xvota): T6 meter 13/64;
  P2a/P2b-i tuple-name + tuple-literal parity landed (`aa62a2d1`); lane rebased
  onto v0.64.6 release main 19eb7218 (36 ahead); continuing the struct/tuple/
  generic corpus. Status relay: REL-20261017-1245-compiler.
- RELAY INTAKE (new, triage next): REL-20261017-1223-orbitdb (transient
  compiler exit -1, no diagnostics, under host load; rerun green -- exit-path/
  OOM audit + dump recipe), REL-20261017-1223-orbitdb-2 (E001 false positive on
  borrowed &Vec[Int] reads; pairs the m344 E001 coverage), REL-20261017-1227-
  packages (HIGH: module-global fn var codegen, adjacent fn struct fields
  layout, fn-alias callable param AV). Queued slots m353-m360 remain; m352/
  m359 landing frees two heavy slots.
- STAGE 6 (perf/lint): benchmark t3 row (~230 ms vs C 25 ms) still awaits a
  profile; slotted behind the v0.64.7 wave. Worktree cleanup of the integrated
  m29x-m350 fixer worktrees/branches is in progress (live m351/m352/m359 +
  selfhost/rustc-study worktrees preserved).
- Relay status item: REL-20261017-1245-compiler (to: all) -- the website lane
  reads the bus.

*** SESSION UPDATE 2026-10-17T11:10Z (compiler lane; MORNING RESUME -- context handover) -- READ FIRST ***
- CONTEXT WINDOW FULL in the previous session (this block supersedes the
  2026-10-16T23:22Z PAUSED block; keep it for reference).
- MORNING STATE: v0.64.6 released 2026-10-16T20:33Z (run 38962236697) and is
  VERIFIED by upstream: orbitdb 2022 CLOSED (verified on v0.64.6); xvector
  Windows legs green (C-17 7/7, C-18 green; C-16 WSL deferred by env + owner
  WSL pause); stdlib 2100 W2-W4 fixed (W1 residual -> m357); benchmark absorbed
  (green); benchmark + packages closures recorded.
- WAVES INTEGRATED ON MAIN (compiler repo, all pushed): m334-m346 (the v0.64.6
  wave) + m347 (platform consts target-value + seed poison) + m348 (fn-ptr
  return ABI flow) + m349 (Array injection + reserved-name diagnostic) +
  **m350 (Result-ref field reads; just harvested: fa475ddd/1a84767e picked as
  e4557ce0/9fb10c40; wave locks 8/8 on the integrated tree; full locks 39/39
  before m350)**. Relay records: f15c01c + a7d349d.
- LIVE SESSIONS (Agent Manager, may finish while this session ends -- HARVEST
  THEM FIRST in the next session):
  * m359 CLI duplicate-flag clap panic -- session ses_eb92e43cbffeiQbgJ9ZYhbCp4e,
    worktree fix-m359-cli-clap-duplicate-flags-*. Was EMPTY at resume; nudged to
    begin (11:12Z). Closes REL-20261016-1128-engine class 4. Definition in
    %TEMP%\kilo\engine-round2-recon-report.md finding 3.
  * m352 arm64 native triple -- THIRD spawn requested (am-1792235123971-5w1aom)
    after two silent failures. If no worktree appears: IMPLEMENT DIRECTLY in a
    manual worktree (git worktree add .kilo/worktrees/m352-manual -b
    fix/m352-arch-native-triple): arch-aware Target::Native (crates/xiom/src/
    lib.rs:1400-1411 + ixom-codegen/src/context.rs:186-194; aarch64 linux ->
    aarch64-unknown-linux-gnu, aarch64 macos -> aarch64-apple-darwin) with a
    pure (os,arch) table test + toolchain_cmd.rs linux-arm64 asset/sums arms
    (+unit tests ~286-298) + optional nasm x86_64 gating. Then add the
    functional on-runner native-compile gate to the release.yml linux-arm64 leg
    and dispatch a workflow_dispatch dry-run.
  * B-33 recon COMPLETE (report C:\Users\lefte\AppData\Local\Temp\kilo\recon-b33\REPORT.md,
    543 lines; artifacts + register proofs + naked-vs-clang IR/asm under
    %TEMP%\kilo\recon-b33\). Verdict: TWO composed defects -- (1) Win64 C-ABI
    aggregate lowering is wrong at extern + raw-Int-as-fn boundaries (16B
    by-value rcx:rdx vs C pointer-in-rcx; 40B 5 words vs pointer; 16B return
    rax:rdx vs sret), defect matrix + file:line map in the report; (2) the
    confined fault path turns any AV into a SILENT exit-0 (SEH catches; the
    language fault is just not represented; P1 CI masking; secondary:
    uninitialized alloca on struct-return fault paths, attempt trace always 0,
    deterministic faults retried -> side effects twice). The item's in-repo
    bundle claim was WRONG -- the bundle must be landed by the fixer.
    **m351 LAUNCHED** (fix/m351-ffi-aggregate-abi; brief = the REPORT.md
    sections 5-7; 4 commits: codegen CAbiPlan + runtime typed fault +
    docs + tests; locks: in-repo probe ladder e2e, direct-extern e2e, IR twin
    no-naked-%struct/sret, no-silent-exit exit-70 + take_fault exit-0; update
    the m244/m308 exit-0 assertions; target-dispatch must not regress
    SysV/AArch64).
  * selfhost Phase 6 (ses_ebeb8f547ffeSvMCBgei6xvota): nudged 11:12Z to rebase
    onto current main (was 8 behind) and advance; verify parity against m347-m350.
- STDLIB SYNC (owner rule, HARD): NO v0.64.7 candidate until the stdlib v0.64.7
  release is prepared with markers bumped AND approved/published, and
  STDLIB_VERSION references that released rev. Current status: stdlib v0.64.6
  tag moved to 8d95815 (waves 155-171); stdlib block 224 = PAUSED by the owner;
  registry publish 38979638142 STILL AWAITING OWNER APPROVAL; the release.yml
  auto pin PR (STDLIB_VERSION -> 8d95815) has NOT appeared yet (gh pr list
  empty at 11:05Z). When it appears: merge per the 1559 protocol + run the
  archive-level verify on the resulting revision; record on REL-20261016-2330.
- QUEUED FIXER SLOTS (v0.64.7, launch as machine slots free; max 3-4 heavy):
  m351 (B-33), m352 (arm64 triple), m353 (builtin-shadow sweep: stdlib 1053
  .hash + 1456 Mutex barrier), m354 (1626 trailing-match T001), m355 (2343 bare
  inline i32-extern sign), m356 (2328 catalog re-add recon), m357 (2100 W1
  non-bare field spellings/method dispatch), m358 (B-32 in-situ sqlite; m346
  extension), m359 (CLI, in flight), m360 (i128 division AV recon), plus the
  older acked backlog wave (packages 1504 re-queue, playground rows, engine
  classes 2/4, XVC-C-08/09/14 family).
- OWNER-DECISION QUEUE: engine-2134 finding 5 (consume-vs-copy rule after the
  m318 pin), m252 cold-lane, the 5 compiler->owner items (Oct 10), B-33
  severity flow, WSL pause (C-16/C-17-WSL close-gates).
- PROCESS CHEATSHEET (hard-won this session):
  * Cherry-pick conflict pattern: ci.yml -> `git checkout --ours` + append the
    fixer's e2e tokens at the line end; COMPILER_BUGS.md -> union (remove the 3
    markers; keep both sides) and insert `    );\n}` + a blank line at joins
    where a side ends mid-function; e2e_tests.rs -> union with the SAME brace
    repair (a side ending at an assert message line needs `    );\n}` inserted
    before the next block). Always run the marker grep + `git diff --cached
    --check` before `--continue`.
  * Relay commits: a concurrent lane push can reject ours -> manual `git fetch
    origin` + `git push origin main` (never stage other lanes' untracked items
    in the shared clone; never `git add .`).
  * WSL: default distro lacks bash -- use `wsl -d Ubuntu -e bash -c "..."`; a
    raw aarch64 binary cannot run on this x64 box (qemu absent) -- `file` is
    the check.
  * release.yml: tag-guarded (tag == workspace version, ancestor of main);
    workflow_dispatch = dry-run artifacts; notes verify limits: 6 highlights,
    summary 240, title 60, text 320 chars, no internal task ids (m###) in text;
    required MD sections Summary/Highlights/Breaking changes/Known issues/Docs.
  * arm64 leg: matrix on ubuntu-22.04-arm + z3 linux-arm64 pin
    (4.13.4-arm64-glibc-2.34, sha 37533e7a...); wasm gated to linux-x64;
    per-platform archive naming + SHA256SUMS (commit 19eb7218).
- WATCH: the */15 cron is DELETED (pause + context handover); re-arm in the
  next session by re-scheduling the "Post-cut wave + relay watch v2" prompt
  (in session history) or sweep manually. Zero open compiler items at 11:05Z
  besides the acked queue; relay head a7d349d.

*** SESSION PAUSED 2026-10-16T23:22Z (owner away; resumes 2026-10-17) -- historical, superseded by the morning block above ***
- Watch DISARMED (the */15 cron deleted). In-flight Agent Manager fixer
  sessions continue on their own in their worktrees: m350
  (fix/m350-result-ref-field-read; recon %TEMP%\kilo\m350-recon-report.md) and
  m359 (fix/m359-cli-clap-duplicate-flags; closes 1128 class 4) were mid-battery
  at pause -- their commits wait on their branches for harvest. m352
  (fix/m352-arch-native-triple) had NO worktree at pause -- RE-REQUEST on
  resume (its first two spawn requests silently failed; check the AM list).
  m351 recon (B-33) was running in-session; its report lands under
  %TEMP%\kilo\recon-b33 when done.
- RESUME CHECKLIST (next session):
  1) Harvest m350/m359/m352 (+ m351 recon verdict) with the standard pattern:
     verify, cherry-pick (ci.yml / COMPILER_BUGS / e2e_tests union + brace care
     at joins), lock suite, relay notes + plan rows, commit.
  2) Stdlib pin PR watch: release.yml auto-files STDLIB_VERSION -> stdlib-v0.64.6
     (8d95815) -- merge per the 1559 protocol + run the archive-level verify
     (item REL-20261016-2330-stdlib).
  3) arm64: after m352 lands, add the on-runner native-compile gate to the
     release.yml linux-arm64 leg + dispatch a dry-run; record on
     REL-20261016-1917.
  4) Queued fixer slots: m358 (B-32 in-situ residual), m360 (i128 division AV),
     m353 (builtin-shadow sweep 1053+1456), m354 (1626), m355 (2343),
     m356 (2328 recon), m357 (2100 W1).
  5) RELEASE RULE (owner): NO v0.64.7 candidate until the stdlib v0.64.7
     release is prepared with markers bumped AND approved/published, with
     STDLIB_VERSION referencing that released rev.
  6) Relay watch: re-arm a cron (the v2 prompt is in the session history) or
     sweep manually; zero open compiler items at pause.
  7) Owner-decision queue: engine-2134 finding 5 (consume-vs-copy), m252
     cold-lane, the 5 compiler->owner items (Oct 10), B-33 severity flow,
     1504 re-queue, playground rows, WSL pause (C-16/WSL close-gates).
- State at pause: v0.64.6 released + verified; wave m347/m348/m349 integrated
  (locks 39/39 + checker 1/1 + ffi 2/2); compiler main pushed (ee62e817-era);
  relay pushed (9ebc3a5-era); stdlib-sync gate recorded in three places.

*** SESSION UPDATE 2026-10-16T23:25Z (compiler lane; POST-v0.64.6, release-protocol correction) -- READ FIRST ***
- OWNER PROTOCOL CORRECTION (23:12Z): do NOT cut/release a compiler candidate
  unless the stdlib is on the SAME TRACK -- the stdlib release for the same
  version must be prepared AND approved/published first (version markers bumped,
  tag on the released rev). v0.64.6's archive pinned stdlib b81430ba whose
  package.xi still said version "0.64.5", so installs packed a 0.64.5-labeled
  stdlib; the stdlib lane is now releasing 0.64.6 to match (block 223: prep
  8d95815, tag MOVED to 8d95815, release run 38979638226, publish 38979638142
  awaiting owner approval). HARD RULE for v0.64.7: cut only after (a) the
  stdlib v0.64.7 release is prepared to the same version with markers bumped,
  (b) its publish is approved, (c) STDLIB_VERSION references that released rev.
- STATE: v0.64.6 RELEASED 20:33Z (release run 38962236697 green; assets
  verified). Upstream closures: orbitdb 2022 CLOSED (verified on v0.64.6);
  xvector Windows legs green (C-17 7/7, C-18 green; C-16 WSL leg deferred by
  environment + owner WSL pause); stdlib 2100 W2-W4 fixed on v0.64.6 (W1
  residual -> m357, medium); benchmark absorbed (green, pin 6d2c142);
  packages 1941/1304 closed. Wave 2 integrated: m347 (platform consts
  register the TARGET value + seed poison removed), m348 (fn-ptr return ABI
  flow for call-valued/Ident-copy bindings), m349 (Array catalog injection +
  reserved-name diagnostic) -- commits d266a080..5cb7515c; wave e2e locks
  39/39 + checker 1/1 + ffi 2/2. Relay record f15c01c.
- IN FLIGHT: m350 (Result-ref field read crash; recon %TEMP%\kilo\m350-recon-report.md),
  m359 (CLI duplicate --timeout/--jobs panic -> closes 1128 class 4),
  m352 (arch-aware native triple + toolchain_cmd linux-arm64; RETRY spawn --
  first request silently failed), m351 recon (B-33 confined-call silent
  exit-0; report %TEMP%\kilo\recon-b33), selfhost Phase 6 (rebased onto
  release main; advancing; T6 meter report pending).
- ARM64 (REL-20261016-1917): release leg MERGED (19eb7218) + arm64 z3 pin
  (4.13.4 glibc-2.34, sha verified); dry-run 38967096669 GREEN incl.
  package linux-arm64; artifact aarch64-verified; functional on-runner
  native-compile gate + m352 remain; ships in v0.64.7 (owner call);
  packages Pi smoke x2 = acceptance.
- OWNER-DECISION QUEUE: engine-2134 finding 5 (consume-vs-copy rule; runtime
  ABI copies -- only the diagnostic is live), m252 cold-lane (2100-benchmark),
  the 5 compiler->owner items from Oct 10, B-33 severity flow (m351),
  registry 1504 re-queue (slotted for v0.64.7), playground rows still red
  (backlog wave), WSL pause (C-16/C-17-WSL close-gates).
- WATCH: a */15 cron "Post-cut wave + relay watch" is ACTIVE
  (wku_1467905dc001BkHXug14AsOM2W) -- relay sweeps + fixer harvest + arm64
  follow-ups; re-arm in a fresh session if this one ends.
- PROCESS unchanged otherwise: fixers commit on their branches, the main lane
  cherry-picks + owns relay rows; main pushes are normal now (release path).

*** SESSION UPDATE 2026-10-16T11:55Z (same compiler session as the wrap below; owner directive + wave A started) ***
- OWNER DIRECTIVE (11:44Z): ALL incoming relay intake from pulse, engine,
  orbitdb and xvector is to be addressed to 100% coverage in the next waves;
  agents allowed; items added to the next candidate list.
- DONE THIS SESSION:
  * Engine intake REL-20261016-1128-engine acked with a four-class triage
    (relay commit e80e382, pushed).
  * Candidate list created: relay release-checks/compiler-compiler-v0.64.6.md
    (check-new compiler-v0.64.6) with a Coverage-plan section assigning every
    four-lane item to wave A/B (relay commit d4da0ea, pushed). 118 rows total.
  * m334 re-test round COMPLETE (verdicts committed on the items):
    1941 GREEN on the rebuilt m334 exe (5784c7c3; probe_tuple + control +
    module-boundary all green); C-ORBIT-12 (2022) STILL RED (same clang
    two-name split); 2303 all 16 probe shapes unchanged vs v0.64.5; 2309
    STILL RED with three clean repros on record: probe_c.xi (socket path),
    NEW user-level probe_d3_result_value.xi (single-file Result.value
    destructure) and probe_d4_main.xi (module-boundary) all warn
    unknown type Tuple__Vec[UInt8]__Str__Int + misdecode (len=6, pointer-like
    peer/port); the match-arm and plain field-access variants are green, so
    the trigger is Result.value payload access for tuples with Vec elements.
    Repro scratch: %TEMP%\kilo\m334-retest\transport\.
  * Wave A recon IN FLIGHT (4 read-only Task agents): orbitdb-2022 namer root
    cause; xvector-2216 imported-type ptr-field stores; xvector-2142 linux
    /search localization; engine-1128 classes 1-4. Round 2 queued: engine
    1358/1538/1046/1411, pulse 1549-pulse-2/3/4.
- NEXT: AM fixer sessions per triaged item (m336+ numbering; wave process owns
  merges); m334/m335 integration into local main still pending -- do NOT
  integrate while recon builds are running.

*** SESSION WRAP 2026-10-16T11:30Z (compiler lane; context moving to the next session) -- READ THIS FIRST ***
- Last session's scope: the v0.64.6 general-Darwin stream (COMPLETE, CI-green; the
  23:40Z block below carries the fix/evidence detail) + ad-hoc compiler-lane duty:
  relay intakes, the m334/m335 fixer wave, and the selfhost Phase 6 kick-off.
- STATE AT WRAP (all verified):
  * stdlib: 2bc064e + 9d6a4d5 on main; e72288f (win64 fp128 convention split:
    clang>=21 pointer shims / clang<=20 XMM shims) is on the witness branch
    witness/darwin-runtime-v0.64.6 and NOT merged -- TOP OPEN LOOP: the stdlib
    lane should merge e72288f at their next wave (relay 2011 says so); the
    branch is disposable afterwards. Waves 145/146 (real OS threads) landed on
    stdlib main overnight.
  * Compiler repo: local main = 9096c711 + this wrap commit (docs only,
    UNPUSHED per constraints). Fixer branches done, not merged (the wave
    process merges): m334 fix/m334-tuple-name-collision (b5b97427 + 5784c7c3;
    tuple-name collision via the from_utf8 Call-arm type inference), m335
    m335-bigint-bitops (d234d792 + e50d79dd; B-29 triaged to the bindings
    probe's own literal mis-conversion -- big-int bitops are EXACT; correction
    filed on REL-20261014-1208-bindings-2 with a reporter re-run requested).
  * Relay: zero open+unacked compiler items. Fixer queue (all acked, to
    schedule): C-ORBIT-12 (REL-20261015-2022; still red on the official
    artifacts per the orbitdb check), XVC-C-16 (2142; linux /search
    regression), XVC-C-17 (2216), 1941 (fixed by m334 -- re-test C-ORBIT-12 +
    2303/2309 against the m334 branch FIRST), 2303 enum-variant defects, 2309
    mixed-element tuples, 2309-2 bare-assert mis-bind, 2328 stdlib-6 residuals,
    2343 bare-inline i32 misread, B-32 (2130), plus the older acked set
    (1504/1509/1358/1538/1109/2032...). Open finding: __fixdfti (Float128 as
    Int128 links the missing double->i128 helper on win64).
  * Selfhost: Phase 6 advanced overnight (rebase-drift port, P5b, tuple-name
    P2a byte-exact; REL-20261013-1732 FIXED + standalone repro filed). Lane
    home: AM worktree selfhost-phase-4-codegen; check the AM list for its
    current state.
  * Watch: the hourly relay-watch cron is session-scoped and dies with this
    session's goal -- a next session should re-arm a watch if desired.
- NEXT SESSION: start from this block + the 23:40Z block below; the stdlib lane
  merge of e72288f and the fixer queue are the two live threads; everything
  else is recorded on the relay (2011/2013 and the intake items).

*** v0.64.6 DARWIN STREAM COMPLETE 2026-10-15T23:40Z -- READ THIS FIRST ***
- Scope (owner decision per relay 1549/1609-pulse; this session's goal): the
  general-Darwin workstream. ALL THREE NAMED RUNTIME GAPS FIXED; the engine
  ask is relayed and acked. Nothing else in flight from this stream.
- stdlib fixes (on stdlib main unless noted):
  * 2bc064e -- fp128 darwin ABI: the whole f128 libcall family defined for
    __APPLE__ x86_64+arm64 with 16-byte-vector C signatures (Apple clang has
    no f128 C type; the asm shims never materialized -> macos-x64 linked the
    tf3 family UNDEFINED; REL-20261010-1609-pulse-2). mach host_statistics64
    free-memory fallback in xiom_runtime.c (the 1549-pulse-7 follow-up; the
    -1 sentinel is gone). PRE-EXISTING fp128 opposite-sign addsub + zero
    fast-path corrections (all platforms; shift-UB + exponent-term sign;
    0.0-a returned +a).
  * 9d6a4d5 -- ELF x64 takes the same vector wrappers (the old Win64 shims
    were being selected on linux and were mis-ABI: first runtime-valued
    exercise SIGSEGV'd).
  * e72288f (WITNESS BRANCH ONLY -- stdlib lane to merge at the next wave) --
    win64 fp128 convention split: clang >= 21 pointer shims; clang <= 20
    (GH runner images ship LLVM 20.1.8) XMM-convention shims. Root cause of
    the windows AV: the win64 f128 libcall lowering changed at clang 21
    (probed with real 20.1.8/21.1.8/22.1.8 toolchains; the dev box
    reproduced the runner AV exactly under 20.1.8).
- Locks: tests/smoke/smoke_darwin_runtime.xi + smoke_diag_f128_l0..l5
  (corpus 955 -> 962; runtime-valued fp128 libcalls; macOS free-memory
  contract). All green on every leg.
- CI EVIDENCE (heavy run 38819521229 on witness/darwin-runtime-v0.64.6,
  compiler_ref=v0.64.5): windows-latest 962/962 GREEN; ubuntu-latest
  962/962 GREEN; macos-14 957/962 -- the remaining five are OUT of this
  stream (guard_fault = the POSIX fault-trace item, section 2 below;
  platform / ffi2 / simd compilefail / stress_crypto_aes_roundtrip =
  stdlib-lane items). The darwin lock passes on macos arm64 end-to-end.
- Records: relay REL-20261015-2011-compiler (fixes + verdicts; includes the
  lane action to merge e72288f); engine ask REL-20261015-2013-compiler
  (engine lane ACKED -- upstream gate cleared + macOS assessment recipe;
  their macOS run is the engine-side step, not ours).
- OPEN FINDING (recorded, not fixed): runtime `Float128 as Int128` links
  __fixdfti (double -> i128) on win64 -- the runtime does not ship it.
- Local fp128 toolchains: LLVM 20.1.8 + 21.1.8 extracted under
  %TEMP%\kilo\llvm20\ / llvm21\ (PATH-prepend + XIOM_RUNTIME_DIR overlay
  reproduces the full runner clang matrix locally).
- Companion lanes: selfhost Phase 6 session running (AM worktree
  selfhost-phase-4-codegen); fixers m334 (DONE: b5b97427 + 5784c7c3) and
  m335 (B-29 big-int bitops, running). The stdlib lane owns stdlib main.

CURRENT STATE (2026-10-13T20:30Z; read this before anything below):

*** v0.64.5 RELEASED 2026-10-15T19:20Z -- READ THIS FIRST ***
- RELEASE: tag `v0.64.5` (annotated) on merged main `84f761c5` (pin
  commit: STDLIB_VERSION = stdlib-v0.64.5 sha `092dfa0` (tag obj
  1667b3a8) + release-notes/v0.64.5.json (6 highlights) + gate record).
  Pin PR xiom#6 rebase-merged after PR CI round 4 all green (ubuntu +
  windows + workspace hygiene); tag release run `38792051847` FULLY
  GREEN incl. the publish job; GitHub Release v0.64.5 LIVE + non-draft
  (SHA256SUMS + xiom-0.64.5-{linux-x64,windows-x64,macos-x64,macos-arm64}
  + VSIX 0.12.2 + wasm bundle). stdlib side: stdlib-v0.64.5 (092dfa0) +
  xiom-std 0.64.5 published. Pin-gate fix round m328-m333 (perf budget,
  api-freeze snapshot, rustls/vet, C++ runtime link, trap-lock
  platform-awareness) shipped in the tag. Records: candidate record
  (bus `4b73d3f` + `fdaee6c`) + this file.
- LOCAL STATE: main fully pushed (local == origin; only untracked
  `m211_listdir_probe/` scratch). All Agent Manager fixer sessions idle
  (m311-m333 worktrees kept for reference). No pending wakeups needed;
  the goal completed with the release.
- NEXT SESSION WORK (v0.64.6):
  1. HEADLINE: general Darwin/macOS -- stdlib runtime items (tf x64
     symbols, `_SC_AVPHYS_PAGES`, arm64 fp128 asm) + ENGINE macOS
     build/run evidence (games); the `@llvm.memset` declare already
     shipped (m331). PULSE 1549/1609 items acked with the archive-verify
     pending on v0.64.5.
  2. POSIX fault-trace parity (m244/m308 markers are Windows-VEH-only;
     stdlib runtime) + any Linux semantics gaps the platform-aware m333
     locks now separate out.
  3. Queued acked items for triage: `REL-20261014-1208-bindings-2`
     (wide-int/confine witnesses re-run on the v0.64.5 archive),
     `1504-registry` (pkg guard / .xiomignore), `1509-engine` (repeated
     scalar flags in port.args), `1510-engine-2` (ordering adopted),
     `1109-bindings` (android cross-target scoping; owner priority),
     `1358/1538-engine` (editor W1/W2 toolchain).
  4. POST-RELEASE WAVE: archive reporter re-verification of the v0.64.5
     artifacts across lanes (bindings/orbitdb/playground/engine/stdlib
     rows per the candidate record; the 27-repro sweep expectations).
  5. Older queue: 2032 owner decision, macOS pair archive re-verify,
     1549-packages-2 faithful repro, release-gate record wrap.
- OPS: battery recipe = section 5; ops lessons = section 6; the whole
  m311-m333 fight log is in the RELEASE LEG STATUS block + section 8.
  ALWAYS: fetch `origin` AND the relays before trusting anything.
- 2026-10-16 m335 (B-29 leg of `REL-20261014-1208-bindings-2`) DONE --
  TRIAGED, NO compiler defect. The bindings bundle's bigint-bitops probe
  mis-converts its own literal: 3132620800 = 0xBAB80000, NOT 0xBAD00000
  (the real NVSDK_NGX_Result base 0xBAD00000 = 3134193664), so the values
  it reports as wrong (2987 / 3132096512 / 2987) are the arithmetically
  correct ones for the literal as written and its "want" values are
  impossible. Re-verified on the v0.64.5 archive AND on main across
  const-folded/hex/runtime-global/fn-param shapes, UInt32->Int zext +
  Int32->Int sext provenance, a runtime shift sweep, and the NGX
  `(rc & 0xFFF00000) == 0xBAD00000` loop over `0xBAD00000|N` (N 0..18);
  `--emit-ir` shows `ashr/and/sdiv/or i64` only -- no 32-bit path. Branch
  `m335-bigint-bitops` (UNPUSHED; stacked on the m334 tip 5784c7c3):
  d234d792 (green-first triage lock tests/regression/m335_bigint_bitops +
  `e2e_m335_bigint_bitops` + `_ir` twin + ci.yml g2 token + COMPILER_BUGS
  entry) and 63d3dd48 (doc: lock sha). Battery all green: locks 2/2, e2e
  g1 16 + g2 245 (x2), codegen lib 12, feature_regression 549, integration
  130; the lock reports 0 failing checks on the archive too (there is no
  red arm -- the finding was never a defect). REAL defect is package-side:
  `xiom.dlss` `NVSDK_NGX_RESULT_FAIL = 3132620800` (comment "0xBAD00000"),
  so `dlss_failed(rc) = (rc - 3132620800) in [0, 18]` reads every real
  NGX failure as success (its suite's equivalence pin shares the wrong
  constant and cannot catch it); the bundle probe.xi + the B-29 ledger row
  need the 3134193664 correction before the workaround can be retired.
  The requester's Agent Manager session closed before the reply landed --
  this block + the COMPILER_BUGS entry + the commit messages carry the
  full report.


- origin/main = `91860f11` (fetch before trusting -- the doorbell may push
  staged work). Local main = the latest commit (see git log), AHEAD by ~68
  unpushed commits (exact: `git rev-list --count origin/main..HEAD`):
  `633ee65b`+`dc3a0102` (v0.64.5 plan/refresh docs) + m276..m292 + m293 +
  m294 + m295 + m296 + m297 + m298 + m299 + m301..m309 + m310 + handoff follow-ups + 2 attempt logs (stdlib-6 leg b m291 reverted, see
  docs/failed_attempts.md; the bindings-4 m288 attempt is superseded by
  m292, and the stdlib-6 leg-b m295 attempt supersedes its failed entry).
  Do NOT push without the owner's authorization.
- v0.64.5 batches m276-m292 + m293..m299 + m301..m309 + m310 (commit -> scope; all relayed):
  - `0268c7d9` m276 env-unwrap veto on concretely-typed names (T1 kv AV).
  - `29e35ef8` m277 cast-call C handoff keeps the RAW fn code (two-word
    env: 0=thunk, 1=raw) + nested fn-type markers parse (T2 B-22 + 1403).
  - `484d9f21` m278 same-named types keep module identity (T3 2137).
  - `1a8141fb` m279 as_ptr/as_mut_ptr inference -- `*T + Int` was Str
    concat (B-17 + B-16).
  - `e1207485` m280 unsafe Copy-Out pointer gate + temp Vec[Str] concat
    (2230).
  - `77827766` m281 `p[i]` raw-pointer byte reads were the UTF-8
    `xiom_char_at` decoder (B-24 stale-zero rounds; B-20/B-11 readback);
    bare `null` at a pointer arg was a stack byte (B-23); `base + off`
    confined tails Str-copied (B-20 helper). Locks `ffi_ptr_slot_readback`
    + `null_literal_marshal`.
  - `d6690868` m282 fn-typed value `f as *UInt8` handed the closure ENV
    handle instead of raw code (thread/spawn worker-entry AV). Lock
    `ffi_fn_env_cast` + `m282_fn_env_raw_ptr`.
  - `3ef4d6a9` m283 (a) postcondition call-cycles -> per-function contract
    re-entrancy counters (1549-packages-8); (b) nested enum payloads
    resolve the concrete monomorph (B-26 1524-bindings-2). Locks
    `e2e_m283_contract_cycle` + `e2e_m283_nested_payload_read`.
  - `186edb3a` m284 `collect_ident_names` had no `Expr::MutRef` arm ->
    `&mut q` in a confined block was never captured (B-21 2140-bindings).
    Lock `e2e_m284_mut_of_struct_return`.
  - `77dfc786` m285 `type_arg_compatible` requires a BASE match when exactly
    one side is a parameterized container -- a concrete scalar is no longer
    treated as an erased container, so `&Vec[Float64]` for `&Vec[Vec[Float64]]`
    is now a call-site T001 (1549-stdlib-5 HIGH; witness compiled rc 0 then
    AVed 0xC0000005). Placeholder carve-outs (wildcard `_`, generic `T`/`*T`,
    empty/unit payloads) keep the legacy bare-vs-parameterized flows. Locks
    `m285_vec_shape_arg_mismatch` + `m285_vec_shape_arg_matched`
    (checker_locks.rs).
  - `7e07d144` m286 cross-type generic callbacks kept their substituted
    types: (1) bare/bracket generic-call bindings substitute the declared
    return via a per-CALL-SITE record (`bare_generic_call_return_xiom`;
    placeholder elems upgrade once the concrete xiom type is recorded);
    (2) match scrutinee payloads fall back to the substituted call return
    ("Option[Str]"); (3) method arg inference skips a leading `Self` param
    so `f: fn(T) -> U` resolves U from the argument's own signature (was
    recorded as the raw "U"); (4) Str `<`/`>`/`<=`/`>=` compare CONTENT via
    strcmp (was first-byte, so `Ord[Str].compare` said "1" == "10"). All
    three 1549-stdlib-7 repros green (map 41->0, core_map 41->0, sortbykey
    1->0; fnptr stays green). Locks `e2e_m286_cross_type_callback_bind` /
    `_core_map_match_payload` / `_sort_by_key_cross_type` /
    `_str_lexicographic_compare` (ci group 2).
  - `1f01e812` m287 `use xiom.convert.tostring` pulled
    `math.mathematical_logic` in; the adapter closure-FIELD call
    `self.predicate(&v)` (FilterIter/TakeWhileIter.next) was hijacked by the
    same-leaf free fn via the UFCS/suffix scans (a DIRECT call bypassing the
    closure env; predicates never saw values). Instance fn-typed fields now
    skip those scans so the env-first closure call wins (1548-stdlib-3 HIGH).
    Lock `e2e_m287_tostring_import_adapters` (ci group 2).
  - `554489f5` m288 LOCK for 1548-stdlib-4 (`x.a@pre` on a `&mut` param
    field): the filed witness is RED on the archived v0.64.4 binary and
    GREEN at HEAD -- fixed within the m276-m287 range (clause/entry-snapshot
    machinery); enforcement verified LIVE at HEAD (+5 mutated body still
    violates). Lock `e2e_m288_mut_param_field_pre` (ci group 2).
  - NOTE: 1548-bindings-4 has NO m288 fix (attempt reverted; see
    docs/failed_attempts.md 2026-10-13 and the bus) -- next free batch is
    m290.
  - `b33a8439` m289 annotated `Slice[Int]` locals allocated an i64 slot
    holding field 0 (the data pointer): `concrete_type_for` fell through to
    `type_from_ast`, which strips Slice to its element ("Int" -> i64).
    `.len()` then strlen'd the data buffer (1 instead of 3) and the slice
    helpers AV'd. Now maps `Type::Slice(_)` to "Slice" -> canonical
    `%struct.Slice` (1548-stdlib-5; its two filed witness FILES were stale
    green on v0.64.4 -- the extended list was the live part). Lock
    `e2e_m289_slice_annotated_local` (ci group 2).
  - `bc716bca` m290 PARTIAL for 1548-stdlib-6: leg a FIXED -- `get_type`
    expands a dotted name's first segment via
    use_alias_paths/local_module_paths, so `ser.SerializeError{..}` resolves
    (was T001). Leg b OPEN -- `v.json_get_path(&path)` on a by-value
    JsonValue still C001 `'JsonValue.json_get_path'`; the UFCS candidate
    helper requires a pointer-like receiver (see COMPILER_BUGS m290).
    Lock `e2e_m290_alias_type_path` (ci group 2).
  - `a6bfe1e1` m291 **ENFORCEMENT GATE FIX + two items**: (1) m283's
    contract re-entrancy guard had made runtime requires/ensures
    enforcement SILENTLY DEAD (skip test `d > 0` with the depth counter
    incremented at function entry, d==1 at the first clause site); test is
    now `d > 1` (only re-entrant evaluations skip). (2) the guard form
    `(result.is_ok == true) => ...` now rebinds the payload like
    `result is Ok =>` (1548-stdlib-7). (3) 1548-stdlib-4 CORRECTION: its
    m288 green was MASKED by (1); the real fix is the Paren arm in
    `collect_atpre_vars` (`(x.a@pre < 100) => ...` collected no snapshot
    vars). Witness rcs 1->0 with enforcement LIVE. Locks
    `e2e_m291_ensures_isok_guard` + checker_locks
    `m291_contract_enforcement_live` (negative twin); `e2e_m288_mut_param_
    field_pre` now passes for real. LESSON: any item closed by a green
    contract BEFORE m291 must be re-trusted only with enforcement live.
  - `c5fe4d80` m292 1548-bindings-4 FIXED: the catalog pre-load worklist
    skipped `[xiom,ffi]` whenever the BARE leaf "ffi" was already in
    `self.modules`; `flatten_submodules` registers nested program modules
    under their leaf, so `module probe.ffi` + `use xiom.ffi;` bound the
    program's own module and the stdlib submodule fns (xiom.ffi.c) never
    registered (T001 undefined variable; this is why m288's direct-load
    registration patches had nothing persistent to attach to). Guard is
    now single-segment-only (`prefix.len() == 1 && ...`). Locks
    `e2e_m292_ffi_alias_shadow` + fixture (negative A/B via a rebuilt
    pre-fix binary).
  - `5fbd54be` m295 1548-stdlib-6 leg b FIXED (item fully done; leg a
    m290): `v.json_get_path(&path)` on a by-value JsonValue was C001
    'JsonValue.json_get_path' -- `ufcs_registered_fn_for_receiver` required
    a pointer-like receiver, so the registered fn (param 0
    `%struct.JsonValue` BY VALUE) never matched. Codegen now mirrors the
    checker's param-0 UFCS search for bound `Ident` receivers
    (xiom-check lib.rs ~8786). IR evidence:
    `call %struct.Option__JsonValue @serialize.json.json_get_path(...)`.
    Locks `e2e_m295_foreign_method_call` (runtime payload check) + m290
    leg-a stays green. NOTE: the m291 "no .json_get_path key" trace claim
    was wrong (both same-leaf keys are registered) -- corrected in
    COMPILER_BUGS m295.
  - `c18e550e` m298 same-named types keep module identity in two more
    paths (1956-orbitdb HIGH + 1709-xvector): (a) the R44 all-catalog
    facade exemption compared shape STRINGS, so the engine's BTree stayed
    on the bare key while its `store` refs had been rewritten to the
    package's NodeStore -> `payloads.push/pop` C001; the triage now marks a
    group conflicting when any shape references ANOTHER multi-owner leaf
    (`shape_mentions_leaves`). (b) `build_renames` rewrote only
    dotted/aliased spellings, so a bare same-leaf field ref stayed bare and
    the suffix pick took the shortest key; a bare ref now binds the UNIQUE
    imported owner (ambiguous/local keeps legacy). Locks
    `e2e_m298_orbitdb_compose` + `e2e_m298_xvector_counter_shadow` (A/B
    negative twins on a rebuilt pre-fix binary). NOTE: the orbitdb probe
    needs the stdlib LANE tree for its byte-IO primitives (bundle dev-env);
    the vendored pin dies earlier on unrelated T001s.
  - `2914cb15` m293 Map/Set deep `==` for by-value fields (packages-11):
    `is_deep_eq_container` excluded Map/Set, so `m1 == m2` hit the
    struct-equality cascade and emitted `icmp eq %struct.Vec` (clang
    reject). deep_eq.rs now gates Map/Set/BTreeMap/BTreeSet (concrete
    K/V/T args, strcmp for Str keys) + legacy non-generic HashSet;
    expr.rs recursion for by-value container fields without `.eq`.
    Locks `e2e_m293_mapset_eq`. CAVEATS recorded (COMPILER_BUGS m293):
    same-order layout semantics (permuted lookup follow-up), HashMap[K,V]
    Str keys unverified, and a latent pre-existing m239 gap: `Vec[Bool]`
    compares with an i64 stride (legacy mapch.HashMap == can
    false-negative).
  - `f6690a77` m294 bare `&mut`/`&` reads load the pointee in EVERY value
    sink (packages-9 HIGH): m197 only covered `return p` + arithmetic;
    let/var bindings returned the stack ADDRESS, `v[p]` AV'd, casts/negation
    read pointer bits, and Eq/Neq double-derefed (`inttoptr` of the loaded
    value -> AV). autoderef now runs at let/var + assign/deref-store + Unary
    + scalar `As`; new `compile_index_i64` at all 12 index sites; Eq/Neq
    excluded from the m197 binop deref. Locks `e2e_m294_mut_int_bare`
    (+ m197/m284 stay green). Caveat: i64-address `&T`/`&Str` params stay
    outside the pointer-typed gate (unchanged).
  - `ee4fc29a` m296 2320-bindings part 1 FIXED: variants of `Vec` sized
    buffers under the documented 8-bytes-per-slot contract -- the stdlib
    `with_len` zero-fills n * 8 but construction sized `cap * elem_size`
    (Vec[UInt8] 16 bytes for 16 slots), so `with_len(16)` wrote 128 bytes
    past a 16-byte region (STATUS_HEAP_CORRUPTION class in xiom.shaderc).
    All construction sites now floor slots at 8 bytes via `vec_slot_size`;
    elements > 8 bytes keep their size; push growth element-sized. IR
    twin: pre `malloc(i64 16)` -> post `malloc(i64 128)`. Lock
    `e2e_m296_vec_with_len`. Parts 2/3 remain open.
  - `6797197d` m297 1615-stdlib FIXED (HIGH): `Path.is_dir`'s body
    bare-call of the same-leaf free fn `is_dir(self.inner)` self-bound
    (legacy in_caller_module fallback + G-10 implicit-self override) ->
    `@Path.is_dir` recursion -> depth-500 llvm.trap -> 0xC000001D for any
    program touching it. Fix mirrors the m261 self-exclusion and makes
    G-10 skip only when the standard resolution bound a DIFFERENT real
    fn. Lock `e2e_m297_path_println_context` + i2/hot-reload regressions
    stay green.
  - `50361abf` m299 1549-packages-12 FIXED: `let (k, v) = &vec[i]` bound
    component ADDRESSES (m209 `&T`-annotated i64 refs); value uses leaked
    addresses and the Str concat path AV'd. The arm now loads VALUE
    copies (m209 `k == &key` compare still works); the
    `regress_m209_tuple_ref_destructure` test was updated to the
    value-copy shape. Lock `e2e_m299_tuple_ref_destructure`.
  - `1b78d20f` m301 1549-stdlib-4 FIXED: clause-position `result[i]` never
    registered the synthetic `result` in `local_vec_elem`, so it fell to
    the scalar `emit_elem_load` fallback -- `Vec[Float64]` f64 bits were
    sitofp'd (1.0 -> 4.6e18 -> false `contract violated: ensures`) and
    `Vec[Vec[T]]` read the inner data pointer as a Vec header. The
    witness's [1,2] Int control passed only by COINCIDENCE (the [7,9] row
    violates pre-fix). contracts.rs now registers the return Vec element
    per clause + snapshots the vec maps. Lock
    `e2e_m301_clause_float_vec_index` (FAILS Some(1) on a pre-fix binary).
    ALSO m301: 1549-packages-2 (io fs_read ensures) = NO-REPRO on HEAD --
    73 runs over 7 multi-module shapes (sibling/chain/permuted imports,
    --parallel-codegen, package-catalog kv shape, heavy append loops,
    stdlib smoke_io2) all green with enforcement LIVE; item stays open
    pending a faithful repro (original nondeterminism was v0.64.0/2-era
    clause machinery that m291 rewrote).
  - `f67adcea` m303 stdlib-2 + stdlib-3 FIXED (one batch for the m300
    fixer's two items): (a) `geometry_3d.Box{..}` T001 unknown-type and
    the spelling-compare param check made aabb_contains/
    aabb_intersection uncallable -- xiom-check now resolves
    module-qualified catalog type spellings in signatures+calls, codegen
    carries them through construction/param matching; (b)
    `var l2 = lu.0;` on an un-annotated nested Vec[Vec[Float64]] result
    lost a level (`type_string_full` erased tuple element containers to
    "Vec" -> scalar elem path, row len 0) -- container args kept in tuple
    types + tuple-field vec reads. Evidence re-run here:
    p_geom_box_unnameable v0.64.4 compile rc 1 (T001 at 31:11) -> rc 0 +
    run rc 0; p_geom_matrix_result_infer v0.64.4 run rc 4 -> rc 0. Locks
    e2e_m300_geom_box_unnameable + e2e_m300_geom_matrix_result_infer.
  - `8c298c91` m302 1659-orbitdb FIXED: a Vec bound/assigned from an
    ALIASING RHS (`var b = a;`, `var w = arena[i];`, `var k = s.keys;`,
    `b = a;`) now deep-copies the buffer at the binding
    (`emit_vec_value_deep_copy`, cap=max(len,1) to dodge the
    realloc(p,0) NULL trap in the stdlib extract-push-storeback idiom);
    fresh values stay uncopied; push/pop unchanged. Witness
    `src[0]=99 copy[0]=99` -> `src[0]=1 copy[0]=99`; fixture pre-fix exit
    4 -> 0. CAVEATS (recorded): by-value call args, `push()` args,
    for-loop element bindings, and whole-struct copies with Vec FIELDS
    still share buffers; generic-boxed container-field handles outside
    the predicate.
  - `d96e9a2b` m304 1550-orbitdb-2 FIXED: standalone `--check` of a
    self-referential module failed T001 `expected Engine, found module`
    while the same file compiled clean AS AN IMPORTED module -- the
    declared chain's segments are flattened into the module map and
    `check_module_field_access` tried module-qualified resolution before
    local-scope lookup, so `db.engine` on a `db` parameter resolved to the
    submodule. Locals now shadow module aliases (locals-first precedence),
    mirroring the Ident arm. Lock `e2e_m304_module_local_shadow`.
  - `81e4743c` m305 1219-orbitdb PART A FIXED (PART B was m260 -- item
    fully done): a tuple from an enum CONSTRUCTOR call
    (`Ok((Color.Custom(n), n + 3))`) fell back to the i64-erased element
    type and built a 16-byte `Tuple__Int__Int` (tag in slot 0, payload
    dropped) while the read side used the 24-byte `Tuple__Color__Int` --
    silent wrong values + 0xC000001D on the ub-grade read. Enum ctor-call
    elements are now named by the parent enum key (expr.rs literal path +
    `infer_llvm_type` tuple arm). Lock `e2e_m305_enum_ctor_tuple`.
  - `06860b27` m306 registry/pkg cluster FIXED (four tooling items):
    (a) 1549-registry-2 trust hint now pins the registry INDEX key
    (--index-key, C5 signed index digest) instead of the ephemeral OIDC
    publisher key; (b) 1549-registry `xiom pkg yank <pkg>@<ver>` against
    the live contract (Bearer token, optional reason, ok=true required,
    error JSON surfaced); (c) 1549-registry-3 `xiom pkg resolve
    <name>@<spec>` outside a workspace (index.json, no auth, closure +
    --json); (d) 1548-registry-2 publish pack guard (build/artifact
    refusal list + gitignore-syntax .xiomignore + declared-modules keep
    set; refuse writes NOTHING; --force reports). pkg suite 92/92.
  - `7822f4b6` m307 1324-selfhost FIXED: a variant-pattern binding NAME
    won the direct field lookup before the canonical field at position fi
    (NkExprField(recv, name_node) read slot 0 = 461 for a parsed 23;
    same class as the 2026-10-02 large-dispatch finding). Canonical field
    at fi now wins; name match is fallback only. Red on v0.64.4 too.
    Locks `e2e_m307_variant_binding_shadow` (+ `_ir` twin: read_renamed
    must GEP `, i32 0, i32 2`, never `, i32 0, i32 1`).
  - `d52a813f` m308 playground/benchmark cluster FIXED (three items):
    (a) 1553-playground-2 by-ref Str comparator params (`&Str`/`&mut Str`)
    fell to first-byte pointer compare (the m286 strcmp branch only matched
    plain "Str"); ref spellings now recognized, find_min returns 2 not 0;
    (b) 1554-playground stdout `fflush(NULL)` before blocking C text-stdio
    reads (pipe block-buffering delayed prompts to exit; playwright probe
    3002/3004 ms); (c) 1554-benchmark guard-page arm/disarm elided for
    arena-free confined blocks (ONE VirtualProtect/mprotect per block entry
    was the ~220 ms t3 floor; ablation 130 -> 2 ms; fault containment
    lock kept). 7 locks incl. IR twins.
  - `776f8c9d` m310 packages cluster FIXED (two items): (a)
    1549-packages-7 local-fallback installs place files + `[dependencies]`
    RANGE specs resolve (old resolver used literal starts_with; new
    xiom-graph version.rs constraint matching); (b) 1549-packages-5 a
    dependency's shipped port.args.json (${PACKAGE_DIR}) applies at
    CONSUMER build time (no manual --c-source; was `undefined symbol`
    on v0.64.4). Battery: pkg 95 + 1, graph 40, e2e g2 209.
  - `d1eab753` m309 2320-bindings parts 2 + 3 FIXED (item fully fixed with
    part 1 m296): (a) confined-block Vec Copy-Out v1 -- a Vec built inside
    a confined block routed its buffer to the guard arena and faulted the
    caller after `xiom_guard_heap_exit` (0xC0000005; the xiom.shaderc
    trampolined crash class); `%struct.Vec` tails on BOTH lowering paths
    (stmt.rs return-inside-block + expr.rs block tail) now malloc+memcpy
    to the main heap before the arena reset; shaderc passes 17/17 x10 with
    all 50 `#[unsafe_direct]` markers stripped; (b) `register_external_
    module` merged export maps one level deep so 4-segment sibling modules
    dropped each other's subtree (LIFO last-use won, import-order flip);
    SubModule collisions now recurse via
    `merge_module_exports_first_wins`. Locks
    `e2e_m309_confined_vec_copy_out` (+IR twin) +
    `e2e_m309_four_segment_const_import` (both orders).
- CLOSED / awaiting reporter re-verify at the next archive (compiler work
  done; items are `fixed` on the bus): T1-T3, B-17+B-16, B-23+B-24
  (2051-bindings-2), B-11+B-20 readback family (0304-bindings +
  1548-bindings-2), 1549-packages-3, 1549-stdlib-5 (shape-mismatched &Vec
  arg is now a call-site T001), 1549-stdlib-7 (cross-type generic
  callbacks), 1548-stdlib-3 (tostring import no longer hijacks adapter
  closure fields), 1548-bindings-4 (leaf-colliding nested module + stdlib
  import -- m292), 1549-packages-11 (Map/Set == -- m293), 1549-packages-9
  (bare &mut/& value sinks -- m294), 1615-stdlib (Path.is_dir self-bind
  SIGILL -- m297), 1549-packages-12 (tuple-ref destructure value copies --
  m299), 1549-stdlib-4 (clause result[i] element binding -- m301),
  1549-stdlib-2 + 1549-stdlib-3 (module-qualified type spellings + tuple
  container args -- m303), 1659-orbitdb (Vec copy-alias deep copy -- m302),
  1550-orbitdb-2 (locals shadow module chain segments -- m304),
  1219-orbitdb (enum ctor-call tuple shape -- m305 PART A + m260 PART B),
  1548-registry-2 + 1549-registry/2/3 (registry cluster -- m306),
  1324-selfhost (variant-pattern canonical field -- m307),
  1553-playground-2 + 1554-playground + 1554-benchmark (playground cluster
  -- m308), 1549-packages-5 + 1549-packages-7 (install/build cluster --
  m310),
  1548-stdlib-6 (leg a m290 alias type paths + leg b m295
  by-value UFCS), 1956-orbitdb + 1709-xvector (same-named types keep
  module identity -- m298), 1548-stdlib-4 (@pre on &mut param fields snapshots the
  entry value -- REAL fix m291 Paren arm; the m288 green was masked), 1548-stdlib-7
  (guard-form ensures + enforcement restored), 1548-stdlib-5 (annotated
  Slice locals), B-25 (1309-bindings, re-probe GREEN),
  B-26 (1524-bindings-2), B-21 (2140-bindings), 1549-packages-8 (contract
  cycles), 1549-stdlib-6 (VERIFIED FIXED cross-platform: clause re-added in
  a scratch tree, stdlib repro + family probe + smoke suite green on
  Windows AND WSL/Ubuntu), 1358-orbitdb (NOT a compiler bug: the documented
  Linux artifact exits 1 for both probes in 20+ variants; rc=0 reproduces
  with the sibling `xiom-dbg` DAP adapter binary -- the lane harness ran
  the wrong binary; asked them to pin/assert XIOM_COMPILER).
- BLOCKED ON THE STDLIB LANE (compiler-lane items filed with ready diffs):
  - `REL-20261013-1423-compiler` -- `xiom.thread.spawn` result buffer is
    guard-arena memory freed at block exit before join (UAF). Thread gate
    (2053/1627/1402) stays open until the buffer is hoisted to main-heap
    allocation outside the confined block.
  - `REL-20261013-1445-compiler` -- arena-aware `xiom_guard_free` +
    `ffi.free` route (alloc-guard-spin 1548-bindings-5). Verified in a
    scratch tree; applies at the next stdlib pin.
- OPEN FINDINGS recorded (no fix yet; in COMPILER_BUGS):
  - (-O2 constant-fold pruning of contract checks: FIXED m311 `3237aaa1`
    -- volatile i8 round-trip barrier in `compile_contract_check`; lock
    `o2_contract_barrier_cli` 4/4.)
  - Duplicate stdlib trees + cross-tree function-body mixing in ad-hoc
    compiles (dedupe module registration by canonical root).
  - Vec has no drop glue (2032-engine) -- language memory-model decision,
    owner-level.
- REMAINING CANDIDATE QUEUE (reds; the bus + the check file are the
  record). Priority order for the next session:
  1. borrow/codegen: 1548-bindings-4 (m292), 1548-stdlib-6 (m295),
     1549-packages-11 (m293), 1549-packages-12 (m299), 1549-stdlib-2 +
     1549-stdlib-3 (m303), 1549-stdlib-4 (m301) DONE. OPEN:
     1549-packages-2 -- NO-REPRO m301 (73 runs / 7 multi-module shapes
     green with enforcement live; needs a faithful repro).
     (1549-packages-10 is a wave stale-green close.)
   2. resolver/checker: 1549-packages-9 DONE (m294), 1550-orbitdb-2 DONE
      (m304); 1956-orbitdb + 1709-xvector DONE (m298).
  3. allocator/runtime: 2320-bindings DONE (part 1 m296 + parts 2/3 m309);
     2032 (owner).
  4. contracts/clauses: 1549-stdlib-4 DONE (m301); 1549-packages-2
     NO-REPRO on HEAD (m301, stays open); 1548-stdlib-7.
   5. runtime/misc: 1615-stdlib DONE (m297); 1659-orbitdb DONE (m302;
      call-arg/struct-copy shapes remain shared -- see COMPILER_BUGS
      m302), 1219-orbitdb DONE (PART A m305 + PART B m260), 1324-selfhost
      DONE (m307), 1553-playground-2 + 1554-playground + 1554-benchmark
      DONE (m308).
  6. tooling/registry: 1548-registry-2 + 1549-registry/2/3 DONE (m306);
     1549-packages-5/7 DONE (m310).
   7. macOS: 1549-pulse + 1609-pulse (typed `@llvm.memset`) -- needs a NEW
      archive to verify.
   8. ENGINE RELAY BATCH -- 17 items queued into the candidate record on
      2026-10-14 (rows added; no triage yet): 1819 (Int ptr arg, B-16
      class), 2035 (&Str param spelling), 2042 + 2042-2 (Int128/UInt128
      miscompile HIGH; UInt64 bit-63), 2119 (large-main startup AV, low),
      2139 (enum-variant/geom name collision), 2215 + 2215-2 (Vec push in
      match arm; move-semantics pin), 2227 (UInt64 div/mod sign), 2238 x3
      (CLI lane: vecvecstr / fnvalue_records / meshencoder), 2256 (const
      shadowing hang + pub use drops consts), 2324-2 (var fn export drop),
      2324-3 (diamond pub const), 1158 (ident named spawn drops fn),
      1213 (layout-sensitive probe AV, low). These are the next fixer-wave
      candidates after m311-m313 land.
   IN FLIGHT / NEXT SESSION'S FIRST ACTIONS (Agent Manager fixer worktrees;
   details in section 8 item 0): m313 fix/m313-ffi-free-guard LANDED on main
   `56234edf` (ffi.free call-site lowering; acceptance GREEN on stdlib main
   -- spincheck 3/3 `v=7` exit 0 <=263 ms; compiler note filed on
   REL-20261013-1445-compiler); m312 fix/m312-thread-spawn-readback LANDED
   on main `bc1d821b` (ptr-typed struct-field READ PATH; locks + IR twin
   green) -- acceptance vs stdlib main NOT green: residual root-caused to
   the RUNTIME (xiom_task_trampoline stores the i64 result as C `long` = 4
   bytes on Windows; low32 == 42 in every garbage readback; fix =
   long long/int64_t in the trampoline ~5962-5977, filed on
   REL-20261013-1423; thread gate 2053/1627/1402 stays OPEN, do NOT flip
   the spawn e2e without it); m311 fix/m311-o2-no-fold-barrier LANDED on
   main `066bc253` (fixer commit 3237aaa1 + doc/ci tokens; contract no-fold
   barrier via volatile round-trip; lock 4/4 + battery green). ALL THREE
   INTEGRATED + relayed (bus 071f5aa/8dbc424/d7c26ba). ENGINE WAVE m314-m317
   ALL LANDED on main + relayed (bus c036f0a/6f88343): m314 `b7ed15b1`
   (2042 + -2042-engine-2 + 2227: Int128/UInt128 16-byte slots/align-16,
   16-byte memcpy stores/typed loads, zext widening, icmp u*/udiv/urem),
   m315 `32923053` (2139: current-module variant precedence + struct-fit
   literal), m316 `f4583fb1` (1158: parser soft-keyword struct-literal
   lookahead -- NOT a catalog shadow), m317 `af3d6460` (2256 + 2324-2 +
   2324-3: const-substitution cycle guard/hang, cross-module const + export
   surfaces, pub-use consts, pub var). WAVE 2: m318 (2215 + 2215-2) and
   m319 (2238 x3 CLI lane) still in flight; m320 `b672a19f` LANDED
   (1819 `ptr as Int` at named-call boundaries + 2035 `&Str` param
   spelling; locks x4; relay 0817396). m321 (2119 + 1213) IN FLIGHT:
   2119's bisect_88 crash is a WRONG-EXPECTATION reduction (chord cannot
   fire -> unguarded events[0] on an empty Vec; identical on the pinned
   v0.64.4 toolchain; the real engine probe is 22/22) PLUS a real gap --
   no Vec read-path bounds lock (default silent UB read, --overflow-checks
   traps) -- being pinned; 1213 REPRODUCES (compile rc 0, run 0xC0000005,
   crash at line 381, strcmp-on-garbage class) and is being localized.
   THREAD GATE GREEN pre-pin: stdlib main `ae9c963` (wave 139,
   long->long-long trampoline) -- dbg131s/dbgworker x5/5 `joined 42`
   (independent re-run 2026-10-14T17:33Z). PIN REQUEST (prep) filed to
   stdlib: REL-20261014-1739-compiler (bus af92693) -- freeze a sha
   carrying wave 139, tag stdlib-v0.64.5 on my signal. OWNER GO for
   v0.64.5 release received (no supervision; pushes authorized for the
   release leg). Next free m322.
   RELEASE v0.64.5 PUNCH LIST (prepared 2026-10-14T18:35Z):
   - freeze = m321 resolved + stdlib tag reply on REL-20261014-1739-compiler
     (tag on my signal after the release-wave push; also ask them to keep
     their release-notes/v0.64.5.md fragment in the tag for the converter).
   - version: Cargo.toml `[workspace.package]` 0.64.4 -> 0.64.5;
     selfhost/src/codegen.xi `SELFHOST_VERSION` -> 0.64.5; README pin text.
   - pin: `STDLIB_VERSION` (repo root, read by crates/xiom/build.rs) -> tag
     sha; vendored `stdlib/` worktree refreshed to the tag
     (scripts/fetch-stdlib.ps1).
   - notes: release-notes/v0.64.5.md (Summary + Highlights `kind:` lines;
     max 6 merged with the stdlib fragment) then
     `xiom-release-notes convert --tag v0.64.5` (converts + validates sync).
   - record: docs/RELEASE_GATE_v0.64.5.md modeled on v0.64.4 (preconditions
     / Gate A suites / blockers / Gate D tag+publish).
   - battery: full e2e (whole ci.yml list), feature-reg 549, check 199 /
     parser / codegen 12 / graph 36 / verify / pkg / release-notes 6 /
     stdlib_tests, front-locks, nine-tool release build (all report
     v0.64.5), ascii guard, doctor; thread acceptance at the PIN.
   - publish: push main -> tag `v0.64.5` (guard: tag == workspace version,
     tag on main) -> release.yml; verify assets + SHA256SUMS; VSIX is
     toolchain-only (editors/vscode/package.json stays 0.12.2).
   RELEASE LEG STATUS (2026-10-14T19:50Z): ALL queue items resolved
   (wave 2 + m321 landed; engine batch fully closed). Release-wave PUSHED
   to origin at `1e4a444e` (90 commits from 91860f11; workspace + selfhost
   bumped to v0.64.5; VSIX README pin). Dry-run archive run 38626737822 on
   that sha (in progress). Tag signal sent to stdlib (bus 06d0a7c) --
   waiting for the annotated stdlib-v0.64.5 tag ref + sha reply on
   REL-20261014-1739-compiler (frozen candidate db2fbe8; asked to drop the
   now-stale alloc/free known-issue line). NEXT: pin commit (STDLIB_VERSION
   + vendored refresh + notes JSON convert + gate record) + full battery
   incl. thread acceptance at the pin -> push -> tag v0.64.5 -> release
   run -> verify assets -> record. Origin CI (ci.yml) also running on the
   push.
   RELEASE LEG STATUS (2026-10-15T09:20Z): NOT released yet; stdlib tag
   HELD pending the candidate battery. (a) m322 `eb256cf3` LANDED: the
   gate's full-suite run found + fixed three latent regressions (signed
   cast targets decide compares -- m32_int_0194/0195; scalar-pointee
   field indexing -- m37_bug45; full e2e 2544 passed / 0 failed / 4
   ignored; the three tests now in the ci.yml e2e list). (b) stdlib's
   candidate checks bounced FOUR classes on the dry-run archive
   (REL-20261014-2123-stdlib, acked): Cell[Int8] set/get readback (m323),
   Result Ok-pair Vec lengths 0xFFFFFFFF + p_wave107/108 (m324),
   zext-float cast p_sweep_single_param (m325), duplicate
   LinkedList.invariant_check p_wave7/8 (m326) -- all four REPRODUCED on
   current main; fixers m323-m326 in flight. Stdlib tag target is now
   `c8d713a` (their notes-cleanup commit; no other waves ride).
   NEXT: integrate m323-m326 (standard flow), push, dispatch a FRESH
   dry-run archive, stdlib re-runs the candidate battery -> on green:
   tag signal -> pin commit (STDLIB_VERSION -> tag sha) + notes JSON +
   gate record + full battery incl. thread acceptance at the pin ->
   push -> tag v0.64.5 -> release run -> verify assets -> record.
   UPDATE 2026-10-15T10:05Z: THREE OF FOUR classes landed on main --
   m323 `0cde8579` (Cell[Int8] set/get: skip the self slot pairing generic
   method decl params; witness rc 2->0), m324 `9914c2ed` (Ok-pair Vec
   lengths: tuple struct keys canonicalized at the first bracket;
   witnesses rc 3->0, p_wave107 20->0, p_wave108 32->0, aes-gcm AV->0),
   m325 `6caf7a16` (float operands never take integer extensions; witness
   p_sweep_single_param compilefail->green); all with locks + ci tokens;
   battery g1 16 + g2 239 + codegen 12 green over the stack; relay note
   filed. m326 (duplicate LinkedList.invariant_check) IN FLIGHT: checker
   worklist dedupe + codegen emit-once guard implemented, p_wave7 green,
   p_wave8 compiles but traps at run (being localized; files touched
   include xiom-check/lib.rs, codegen context/contracts/decl/expr/lib,
   xiom/src/lib.rs). ON m326 LANDING: full e2e (2544+ baseline) -> push ->
   fresh dry-run -> stdlib candidate re-run -> tag signal.
   UPDATE 2026-10-15T10:50Z: m326 LANDED `1f7c1c22` -- ALL FOUR classes
   integrated (m323 `0cde8579`, m324 `9914c2ed`, m325 `6caf7a16`, m326
   `1f7c1c22`; the wave8 trap was the @pre snapshot deep-copy reading the
   6-field list as a Vec header -> malloc(huge) -> trap). p_wave7/8 probes
   compile+run 0/0; stack battery green (g1 16, g2 241, codegen 12, check
   serial 199, front-locks). 2123 relayed fixed. FULL E2E RUNNING (bgp
   _13f333cf6001X2XAgtJuKKWNsd); ON GREEN: push -> fresh dry-run ->
   archive signal to stdlib (REL-20261014-1739-compiler) -> candidate
   re-run -> tag -> pin -> publish.
   NEXT-CANDIDATE QUEUE (owner decision 2026-10-15T11:26Z): GENERAL
   Darwin/macOS support -- emit the matching declare for the typed
   `@llvm.memset.p0i8.i64` (sites emitter.rs:1153, expr.rs:4229,
   stmt.rs:737/1190; or switch to the untyped form) + the darwin runtime
   items (tf x64 symbols, `_SC_AVPHYS_PAGES`, fp128 arm64 asm; stdlib
   runtime, cross-filed) -- so the ENGINE lane can build and run macOS
   games. v0.64.5 DOES NOT wait for it (owner: hurry the release; lots of
   relays depend on it); pulse items REL-20261010-1549-pulse +
   REL-20261010-1609-pulse stay acked with the decision noted (bus
   39bfb97); v0.64.5 still produces + job-verifies both macOS legs.
   UPDATE 2026-10-15T11:30Z: FULL E2E = 2549 passed / 1 FAILED / 4
   ignored: e2e_m37_bug55_unsafe_ptr_capture. Bisect: green at m322/m323/
   m324/m325, RED at m326 (`1f7c1c22`) -- your m326 llvm_type_for
   qualification reroutes an xiom.rc field write into the struct-value
   alloca path and DROPS the store-back (IR A/B on @__unsafe_block_3:
   m325 stores `+1` to field1; m326 computes it, discards it + a dead
   load) -> the strong-count bump never lands -> upgraded Rc reads a
   freed box -> `up.get() != 100` (rc 2). Follow-up fix requested from
   the m326 session (ses_ec12801c2ffepB5lk7uSZ6UxEv); fix lands as an
   m327 batch (second commit on fix/m326...; cherry-pick the follow-up
   only), then FULL e2e again, then the push -> fresh dry-run -> stdlib
   archive signal. NOTHING pushes until bug55 is green.
   UPDATE 2026-10-15T11:40Z: m326 follow-up LANDED + integrated as m327
   `7a574f9c` (struct_field_names_for: types exact/leaf + type_meta
   exact/suffix; direct gep+store kept on the qualified struct path).
   bug55 0/0; m326 locks + p_wave7/8 probes 0/0; g1 16 + g2 241, codegen
   12, check serial 199. FULL e2e re-gate RUNNING (bgp
   _13f5c8479001l23yrQp4W3MQxd); ON GREEN: fetch + push main (~13
   commits) -> fresh dry-run -> signal stdlib (REL-20261014-1739-compiler)
   -> candidate battery -> tag -> pin -> publish. m327 note filed (relay).
   UPDATE 2026-10-15T12:00Z: FULL E2E GREEN 2550 passed / 0 failed / 4
   ignored -- stack FROZEN and PUSHED (`1e4a444e..13081238`). Fresh
   dry-run `38731274663` on `13081238` (in progress). Archive signal sent
   to stdlib (bus c836588): candidate battery re-run on the new archive;
   cut stdlib-v0.64.5 on c8d713a (or newer) + reply ref+sha on green.
   NEXT: tag reply -> pin commit (STDLIB_VERSION -> tag sha + vendored
   refresh + notes JSON convert + gate record finalize) + full battery
   incl. thread acceptance AT THE PIN -> push -> tag `v0.64.5` + push tag
   -> release run -> verify assets + SHA256SUMS -> record in the
   candidate record + handoff.
   UPDATE 2026-10-15T17:15Z (recovery after the loop stalled post-12:44Z,
   plus the pin-PR CI findings):
   - stdlib RE-CUT the tag: stdlib-v0.64.5 = tag obj 1667b3a8 -> commit
     `092dfa0` (old e082623 WITHDRAWN; their smoke_os_sync tripped the
     m296 pin-era Vec-ctor class; stdlib test fix only). xiom-std 0.64.5
     registry publish RESOLVED. Local STDLIB_VERSION now `092dfa0` +
     vendored worktree at the re-cut tag.
   - Pin PR xiom#6 (chore/pin-stdlib-v0.64.5, owner-opened; branch =
     main@13081238 + STDLIB_VERSION) CI found FOUR blockers:
     1) perf_budget_fmt_peek (windows leg): IR 113323 > 95000 --
        compiler-side (repros with BOTH pins); fixer m328
        (`ses_ebf5a8d7cffeqaPfwUpkg1JKnz`) running.
     2) stdlib_api_freeze_no_removals (ubuntu): 30 signatures missing --
        extractor regression, NOT stdlib content (repros with old pin);
        fixer m329 (`ses_ebf5a2ab9ffenEU0bfTVy7OH9c`) running; do NOT
        update the snapshot.
     3) ubuntu `dep_port_args` test: `C:/` is not absolute on POSIX ->
        double-prefixed dir; FIXED m330 `7628fbc0`.
     4) cargo-deny advisories RUSTSEC-2026-0346..0349 (rustls 0.23.45 ->
        `>=0.23.46`); FIXED in m330 (rustls 0.23.46).
     m330 pushed to main (`13081238..7628fbc0`).
   - Local uncommitted pin set (rides the PR branch): STDLIB_VERSION +
     notes JSON (6 highlights; convert+verify green) + gate record;
     thread acceptance at the pin 5/5+5/5; pin e2e 2550/0/4.
   - m331 `b6cf620d` LANDED + pushed (owner-approved quick Darwin fix):
     `@llvm.memset.p0i8.i64` declared in the builtin preamble (4 sites)
     + `hardcoded_declare_names`; PULSE 1549/1609 items acked-updated
     (bus 4152229); proper darwin work queued v0.64.6. NOTE: the declare
     adds ~49 bytes to every module's preamble -- when m328's budget
     lands, re-run perf_budget_tests after BOTH are on main (m328 was
     calibrated without the declare).
   - NEXT: m328+m329 land -> push main -> rebuild the PR branch
     (main + pin set) -> PR CI green -> merge xiom#6 -> tag v0.64.5 ->
     release run -> verify assets -> record -> goal_report complete.
   UPDATE 2026-10-15T18:35Z: ALL pin-CI blockers cleared on main --
   m328 `738e7158` (perf: leaner guards + 95k->112k + rationale),
   m329 `989c1bc9` (freeze: wave-105 canonicalization + wave-98 M7
   snapshot repair), m330 `7628fbc0` (POSIX test + rustls 0.23.46),
   m331 `b6cf620d` (memset declare). Pin-gate suites at the pin state:
   e2e 2550/0/4, feature-reg 549, workspace libs, fuzz 24, robustness 63,
   perf 3/3, freeze 3/3, stdlib 40 + execution 85, front-locks, ascii,
   thread acceptance 5/5+5/5. Pin branch `chore/pin-stdlib-v0.64.5`
   FORCE-PUSHED with the full pin set (`858d2e8d` = main + STDLIB_VERSION
   sha `092dfa0` + notes JSON (6) + gate record); PR CI RUN `38786292404`
   in progress. ON GREEN: merge xiom#6 -> tag v0.64.5 on the merged main
   -> release run -> verify assets + SHA256SUMS -> record + goal_report.
   UPDATE 2026-10-15T18:45Z: PR CI round 2 (`38786292404`) -- windows
   GREEN; two leftovers FIXED on main: m332 `6aa7dc4f` (ubuntu
   `cxx_standard_applies_to_c_source_cpp`: C++ --c-source link now adds
   `-lstdc++`/`-lc++` on non-Windows; + rustls vet exemption 0.23.46 for
   cargo-vet). Pin branch REBASED (`0ff49434`) + force-pushed; PR CI
   ROUND 3 = run `38787764451` (queued). Same next-steps on green.
   UPDATE 2026-10-15T19:00Z: round 3 -- windows + hygiene GREEN (vet +
   advisories fixed); ubuntu red on FIVE g2 fault locks (first ubuntu
   exposure: llvm.trap signal-terminates on unix -> `code()` None; the
   m244/m308 "[trampoline] fault" marker is Windows-VEH-only). FIXED m333
   `81e98099` (platform-aware trap locks: unix None accepted, Windows
   strict; marker asserts cfg(windows), containment exit-0 asserted
   everywhere). Pin branch rebased (`247b0f14`) + force-pushed; PR CI
   ROUND 4 = run `38790060894`. v0.64.6 QUEUE ADDED: POSIX fault-trace
   marker + cross-OS fault-containment observables (stdlib runtime).
   ON GREEN (round 4): merge xiom#6 -> tag v0.64.5 -> release run ->
   verify assets -> record -> goal_report complete.
   UPDATE 2026-10-15T19:13Z: PR CI ROUND 4 `38790060894` ALL GREEN
   (ubuntu + windows + hygiene). PR #6 MERGED via rebase-merge (repo
   disallows merge commits) -- origin main = `84f761c5` (the pin commit).
   TAG `v0.64.5` (annotated) PUSHED on `84f761c5`; RELEASE RUN
   `38792051847` in progress.
   *** RELEASED 2026-10-15T19:20Z ***: release run FULLY GREEN (guard +
   linux-x64 + windows-x64 + macos-x64 + macos-arm64 + VSIX + publish).
   GitHub Release `v0.64.5` live + non-draft: SHA256SUMS, the four
   platform archives, VSIX 0.12.2, wasm bundle + typings. Pin:
   STDLIB_VERSION = stdlib-v0.64.5 sha `092dfa0` (tag obj 1667b3a8) +
   notes JSON (6 highlights) + gate record; pin-gate fix round m328-m333
   shipped in the tag. stdlib side: xiom-std 0.64.5 published. v0.64.6
   headline queue: general Darwin work (declare landed m331; runtime tf
   x64 / _SC_AVPHYS_PAGES / arm64 fp128 asm + engine macOS build/run
   evidence) + POSIX fault-trace parity (m244/m308 markers). Post-release
   wave: archive reporter re-verification (bindings/orbitdb/playground/
   engine/stdlib rows) per the candidate record.
  Row-sweep note: several check-file rows still show `acked`/red for items
  fixed in m276-m284 (e.g. 1437/1512 -> m279; 2230/2053/2137/1402/1403/
  1418 -> m276-m282) -- sweep rows when convenient.
- OPS LESSONS (all cost time this session; obey):
  - LINE ENDINGS: `core.autocrlf=true` and repo docs stored CRLF (e.g.
    COMPILER_BUGS.md). PowerShell `Set-Content`/`-replace`/append silently
    converts whole files (whole-file diff). Edit markdown with the `edit`
    tool or Python `io.open(newline='')`; verify `git show --numstat` is
    small before committing; fix with `git update-index --cacheinfo` of
    the raw blob if it happens.
  - STDLIB RESOLUTION: multiple trees on this box (pinned
    `E:\xiom-lang\xiom\stdlib`; live `E:\xiom-lang\stdlib`). For
    `target/debug/xiom.exe`, exe-ancestor order picks the LIVE tree, and
    29 scanned duplicates can mix bodies across trees. ALWAYS set
    `XIOM_STDLIB=E:\xiom-lang\xiom\stdlib` for manual probes. Prefer
    `--manifest-path` + `cargo test` harness for locks.
  - WSL CROSS-PLATFORM CHECKS (works): run the Windows compiler FROM WSL
    via interop with WINDOWS-style paths (inputs `E:\...`, not `/mnt/e/...`)
    and `XIOM_STDLIB=E:\\xiom-lang\\xiom\\stdlib`; it links with the Linux
    clang and produces Linux binaries (`E:\xiom-lang\xiom\target\tmp-*` as
    the scratch dir). WSL user: `lefteris`; repo at `/mnt/e/...`; released
    toolchain in `~/.local/share/xiom/bin/`.
  - BINARY CONFUSION: `xiom-dbg` (in the toolchain bin/) is the DAP debug
    adapter -- ignores `--check`, exits 0 always. Never gate on it; assert
    the compiler sha256 in harnesses.
  - HOST: `dbg125p.exe` runaway (94 GB) kills cargo/CLR; `taskkill /F /IM
    dbg125p.exe`. `xiom-check --lib` needs `--test-threads=1` under memory
    pressure (serial 199/199).
- Relay bus SHAs: ... `438b714` (m281), `40e3c40` (m282), `9b4ec0d`
  (allocator triage), `fbda2a7` + `7aa7bfa` (m283/m284 closers + B-21),
  `da3160b` (stdlib-6 cross-platform verify), `61dfda8` (orbitdb-1358
  diagnosis), `5a2f5d2` (m285 fix + row), `deb173d` (m286 fix + row),
  `b8caee2` (m287 fix + row), `ce5f2b4` (bindings-4 attempt triage),
  `76869eb` (stdlib-4 fix + lock row), `bb30a68` (stdlib-5 fix + row),
  `d5e84f8` (stdlib-6 part-a fix + leg-b triage), `170cdd9` (leg-b
  attempt-2 triage), `9712942` (m291 enforcement + stdlib-7/4 correction).
  NEXT-BATCH FIX REFS rows + history bullets current.
- Selfhost: phase-4 worktree (ses_eded8c4d8ffe, sole writer) idle after its
  13:00Z prompt; sibling ses_ef8e3a27cffe idle. Rebases/tests will pick up
  m276-m284 at the next rebase.
- Verification battery (current): e2e g1/g2 filters at ci.yml lines
  152/153 (g1 = 16, g2 now 174 incl. m283/m284 locks); front-locks = the
  ci.yml CLI list (now incl. `ffi_ptr_arith`, `ffi_ptr_slot_readback`,
  `null_literal_marshal`, `ffi_fn_env_cast`); `xiom-check --lib` (SERIAL),
  `xiom-codegen --lib`, `xiom-graph --lib` (36), `xiom-pkg` (79). Capture
  compiler output to files. Last full green: e2e 16+174, front-locks,
  libs 199/36/79.

Landed in the m254-m275 release wave (historical, already pushed):

- `677d65a7` m261/m262 module-local call precedence + module-private const
  identity. Closes REL-20261011-1124-orbitdb, REL-20261011-1052-stdlib.
- `fde5548e` m263-m265 fn-value ABI: C-boundary marshaling (B-13/1904),
  Vec[fn] element-call arity + unit returns (B-14/B-15), module-scope
  fn-ptr storage (0317-pulse-7). CLI lock `crates/xiom/tests/ffi_fnptr_abi.rs`
  (real C host) + e2e m264/m265.
- `0b1c6374` m266/m267 unresolved imports are hard single errors (2124 +
  1802 gate) + bare pattern variants prefer the scrutinee's enum (twin).
  Locks: `checker_locks::m266_unresolved_import_one_clear_error`, e2e m267.
- `c86e2f2d` m268/m269 install-target validation + deterministic tar (2100),
  packages-bearing home preference (2102).
- `0219c5e3` m270 pulse site-mode AV root-caused to the m261-class bare-call
  hijack of a module-local struct-returning helper; lock e2e m270 + fixture;
  the 1854 COMPILER_BUGS section flipped to FIXED. Closes 1854 + 2122-pulse.
- `f69b542e` m271 float Vec slot stores narrow numeric sources (XVC-C-15):
  closes REL-20261011-1134-xvector. Lock e2e m271 + fixture; IR witness
  (fptrunc/sitofp); relay item filed `fixed`, release-check row appended.
- `b1ee2f4f` chore(release): v0.64.4 prep -- workspace + SELFHOST_VERSION
  bump, `release-notes/v0.64.4.md` draft (4 compiler highlights; dry-run
  convert 6 total), VSIX README pin.
- `745ff47e` m272 fn-value ABI regressions found by the gate's full e2e:
  C-boundary-only `callee_is_extern` unwrap gate + module-qualified
  `fn_typed_params` keys; locks = the six tests added to ci.yml.
- `c44d9fae` m273: m219 IR-shape lock follows the m263 `i64 ()**` spelling.
- `b873c6d6` m274: B-18 fix (bare fn NAME to an extern fn-typed param
  passes the raw code address; the four call-site wrap paths are gated by
  `callee_is_extern`); m263 CLI lock gained the bare-name arm. B-19
  resolved `wontfix` -- package-side `UInt8` vs `Uint32` declaration for
  SDL_WasInit (see COMPILER_BUGS m274; probe/package evidence).
- `4c42fc0a` m275: stdlib wave-regression fix (REL-20261011-2045-stdlib):
  the As-cast env unwrap now requires the compiled value to be the i64 env
  slot, so a fn-typed param name leaked into closure_locals can no longer
  hijack a float's cast (`Float32.to_float(v) { return v as Float64; }`);
  collision lock added to the m263 CLI fixture.
- `df493530` chore(release): v0.64.4 pin + notes JSON + gate record (the
  release commit); `91860f11` released-record docs. Gate record:
  `docs/RELEASE_GATE_v0.64.4.md`.

## 1b. Release wave state (next actions)

- PIN DONE (2026-10-11T22:38Z): stdlib tagged `stdlib-v0.64.4` = `3df0843`
  and verified (witness green on the v0.64.4 archive from the `87c6ca3c`
  push; metro revert `4e8b696`; refresh run 38177738188: corpus 954/954,
  probes 288/288, modules 509/509, barename 0/509;
  REL-20261011-1812-compiler verified + closed).
  - `STDLIB_VERSION` = `3df0843b2c384f9768867d96cfd4a611bed3034b`; vendored
    `E:\xiom-lang\xiom\stdlib` is a worktree pinned at the tag;
  - `release-notes/v0.64.4.json` converted + verified from the clean tag
    checkout (6 highlights = 4 compiler + 2 stdlib, 0 breaking, 2 docs);
  - origin/main = `df493530` (the release commit; do NOT push again until
    the tag run settles).
- RELEASED (2026-10-12T13:37Z): tag `v0.64.4` on `df493530`; release run
  `38256358845` GREEN; assets live (linux-x64, macos-x64/arm64,
  windows-x64, VSIX 0.12.2, wasm bundle, SHA256SUMS).
- WAVE LANDED (2026-10-12 evening): `release-checks/compiler-compiler-v0.64.4.md`
  (created 15:01Z) carries the reporter verdicts -- bindings, packages,
  orbitdb, playground, stdlib filed; successes and the ~35 still-red rows
  are distilled in section 1c. The to-all item `REL-20261011-1731-compiler`
  stays open until every row has a verdict (no compiler action needed for
  that bookkeeping).

## 1c. Next program (v0.64.5) -- plan from the v0.64.4 release-check

The v0.64.4 verdict wave is landing (`release-checks/compiler-compiler-v0.64.4.md`,
created 15:01Z). Wave closed: B-13/1904/B-14/B-15/B-18/B-19; m260 casts
PART B; C-ORBIT-06/07; 2100-pkg WSL paths; 1548-stdlib-2/5; packages -4/-6;
1900-engine + 1803-selfhost (fixed in the wave); 2045 (m275). Still red /
open: ~35 rows. Plan:

TRIAGE FIRST (P0):
- T1 `REL-20261012-1418-pulse`: DONE (m276, this commit). Not a v0.64.4
  wave regression but the m263 env-unwrap class on the EMPTY-store path:
  kv_open -> _create_segment -> stdlib write_u32_le's `var x = v as Int`
  unwrapped a leaked closure name (`v`); m275's i64 gate missed the
  zext'ed UInt32. Fix = type veto in `unwrap_fn_env_for_c`. Witness:
  tests/regression/m276_env_name_leak_cast (pre-fix 0xC0000005, post-fix
  exit 0) + PULSE empty-store serves + creates seg-1. (The reporter's
  "exit 0" was a %ERRORLEVEL% parse-time artifact; it AVs at 0xC0000005.)
- T2 `REL-20261012-1630-bindings` B-22: DONE (m277, this commit). Root
  causes: (a) nested fn-type markers split at `find(") -> ")` +
  `split(',')` (the `unknown type 'Int32, Int32) -> Int32'` family,
  1403-packages) -- now depth-aware `split_fn_marker_body`/
  `split_top_level_type_params`; (b) cast-call fn-typed args kept the
  closure env, and the env field 0 is the env-first THUNK (handing it to
  C shifts every arg) -- `wrap_fn_ref_env` envs are now two words
  (0 = thunk, 1 = RAW code) and every C boundary (`f as Int`, extern
  fn-typed params, cast-call args) loads field 1. Witness:
  tests/regression/m263_fn_value_c_abi gained via_cast_param /
  via_cast2_param; probe9 cb-abi green (42/42/42); pre-fix exit 9 + the
  warning, post-fix exit 0, no warnings. Closure-to-C stays on the
  workaround APIs (closures were never C-callable).
- T3 `REL-20261011-2137-engine`: DONE (m278, this commit). Not symbol
  mangling but TYPE identity: the catalog injection deduped same-named
  types by BARE name, so `xiom.<root>.render.mesh.Mesh` vs stdlib
  `xiom.geom.geometry_3d.Mesh` were decided by ASCII module order
  (`Runtime` < `geom` < `runtime`/`rt` < `zz`) -- the loser compiled
  against the winner's layout (field 2 vs 0) -> AV. Fix: collision
  pre-scan + module-qualified injection of colliding types,
  `pick_deterministic` proximity (current module, then entry `use`
  imports via the new `primary_imports`), and injected flat fns derive
  module context from their leaf-qualified names. Witness: engine
  scratch `engine` and `zz` roots both exit 0 (zz was 0xC0000005);
  lock `e2e_m278_type_name_collision` + fixture
  `tests/regression/m278_type_name_collision/`.

THEMED BATCHES (m-numbering assigned as they land; one commit per batch):
- FFI pointer: B-16/B-17 DONE (m279) and 2230 DONE (m280: unsafe Copy-Out
  Str-copied raw-pointer returns -> gated on !expr_is_pointer at all three
  sites; temporary Vec[Str] element concat -> vec_value_xiom_type Call arm;
  lock e2e_m280_unsafe_ptr_temp_str). Remaining in this batch:
  B-11/0304/1548-bindings-2 (out-param recycling / foreign buffer -- not yet
  triaged) and B-23/B-24 (REL-20261012-2051-bindings-2: null literal to C;
  ffi out-slot reads after a C write).
- Thread/spawn runtime: 1627/1402-xvector + 2053-engine (fn-value runtime
  entrypoint AV; marshal half fixed at v0.64.4).
- Allocator/runtime: 2032-engine (Vec drop leak), 2320#1 (with_len HBO,
  ASAN), 1548-bindings-5 (confined alloc/free spin).
- Borrow/codegen: B-21 (undefined IR temp on &mut of struct-returning
  call), 1659-orbitdb (copy elem-assign aliasing), 1549-packages-9 (bare
  &mut Int reads pointer).
- Resolver/checker: 1548-bindings-4, 1457-engine (bare sqrt), 2320#3
  (4-segment const), 1548-stdlib-6, 1550-orbitdb-2 + 1358-orbitdb
  (--check T001 + Linux rc), 1709-xvector, XVC-C-13, 1053/1456 (hash /
  Mutex shadow), 1626.
- Contracts/clauses: 1549-packages-8 (recursion AV), 1549-packages-3
  (unsafe return cast null), 1549-packages-2, 1411-engine, 1548-stdlib-4/7,
  1549-stdlib-2..7 (payload reads, polyhedra, cross-type callbacks,
  shape-mismatch silent).
- Tooling/registry: registry B1/B2/trust/resolve, 1549-packages-5/7,
  1500-benchmark cache ask, website-7/8/9, benchmark trampoline row, pulse
  RSS, playground rows (coercion/buffering/drvfs).
- macOS leg: 1549-pulse/1609-pulse typed `@llvm.memset.p0i8.i64` emission.

## 2. Batch 6 (LAST) -- FLOAT32 SLOT STORES -- DONE (m271)

- Fix (`f69b542e`): `stmt.rs` Vec indexed/field store converts numeric
  sources to the element float width before the bit-level elem store
  (float sources via coerce_value -- fptrunc/fpext; Int via val_to_i64 +
  sitofp); the same branch widens Int literal stores into Vec[Float64].
- Witnesses: pre-fix exit 3 (A `a[1]=0`, B `b[1]=9.80908925027372e-45`,
  E `e.data[1]=0`; C/F/G green) -> post-fix exit 0; IR `fptrunc double 7.0`
  at the A site, `sitofp i64 7` at the B site. The xvector probe needs the
  lane-local `Vector` type; the local-struct matrix reproduces the same
  values (scratch recipe: `%TEMP%\kilo\m271\repro.xi`).
- Lock: `e2e_m271_f32_slot_store` + `tests/regression/m271_f32_slot_store/`
  (fails Some(4) pre-fix), ci.yml group 2.
- Relay: `REL-20261011-1134-xvector` acked -> fixed with evidence; to-all
  status item updated; release-check row `m271 | f69b542e` appended
  (relay commits `a3e9508` + `6386301`). Remaining gate: reporters re-run
  their probes at the archive and set `verified` or bounce.

## 3. Selfhost track (parallel)

- Worktree: `.kilo/worktrees/selfhost-phase-4-codegen` (branch
  `selfhost-phase-4-codegen`); session `ses_eded8c4d8ffeMNzBsYzBYIgAO8`;
  sibling session `ses_ef8e3a27cffeKY1Sz63tGGas2t` must stay IDLE.
- State (last confirmed 2026-10-11 ~19:40Z; re-check with agent_manager
  before relying on it): phase 6 at **T6 4/64** (P0-P1e + P5a landed; all
  five gates re-run green; the relay item `REL-20261011-1525-selfhost`
  records P1a/P1b/P1c/P5a + T3 meter 3/64). Remaining: P1 remainder
  (method receivers: self alloca + field GEPs + `x.method()` calls) ->
  P2 tuples / P5 stack arrays -> phase-6 meter flip (4 of 11).
- Cadence: the session works a chunk then idles. Check with agent_manager
  `list`; if idle, prompt: continue the next unchecked stage per
  `docs/checklists/selfhost-phase6.md`, rebase onto the newest LOCAL main
  before full parity runs, sole writer, same commit/doc/relay discipline.

## 4. Release coordination -- DONE (historical)

v0.64.4 coordination complete: `REL-20261011-1731-compiler` (to all;
status + updates 1-6), `1731-compiler-2` (to stdlib), `1812` (pin
decision; verified + closed), `2014` (push nudge; fixed). The only
follow-up is bookkeeping: close `1731` once every release-check row has a
verdict (the check file is the source of truth). Nothing else pending.

## 5. Verification command set (copy/paste)

- `cargo build -p xiom`
- e2e groups from ci.yml (PowerShell; the `-split` matters):
  `$l1=(Get-Content ".github\workflows\ci.yml")[152]; $fa1=($l1.Substring($l1.IndexOf('-- ')+3)) -split '\s+'; cargo test -p xiom-codegen --test e2e_tests -- $fa1`
  `$l2=(Get-Content ".github\workflows\ci.yml")[153]; $fa2=($l2.Substring($l2.IndexOf('-- ')+3)) -split '\s+'; cargo test -p xiom-codegen --test e2e_tests -- $fa2`
  (currently 16 + 167 green; the m272 six joined group 2)
- front-end locks: `cargo test -p xiom --test doctor_cli --test borrow_e001 --test checker_locks --test run_script_cli --test cxx_standard_cli --test ffi_fnptr_abi`
- units: `cargo test -p xiom-check --lib` (199, incl. full catalog corpus
  gate), `-p xiom-codegen --lib` (12), `-p xiom-graph --lib` (36),
  `cargo test -p xiom-pkg` (79).
- A/B baseline switch: `git checkout <sha> -- crates/` then build; restore
  with `git checkout HEAD -- crates/`. Attribution trick that worked
  repeatedly: disable ONE fix's effect (early `return None`) + rebuild +
  retest.

## 6. Ops notes for the fresh session (learned this session)

- NEVER pipe compiler output into `Select-Object -First N` directly -- the
  broken pipe kills the compiler (exit -1). Capture with `*> file` then
  grep the file.
- `%TEMP%\kilo` holds many stale toolchain checkouts -> W001 flood noise
  (harmless; the chosen stdlib is the pin). Suppress with `2>$null` when
  only the exit code matters.
- A/B runs: use the PINNED stdlib `E:\xiom-lang\xiom\stdlib` (unset
  XIOM_STDLIB); `E:\xiom-lang\stdlib` is the stdlib lane's moving working
  repo (commits while you work).
- Pulse 1854 repro: `git -C E:\xiom-projects\xiom-pulse archive 01dcd1b`
  into a fresh temp dir + append the unused `out_text` helper to
  `tests/test_app.xi`. A stale `a.exe` in the tree breaks 8 resource tests
  on ANY compiler (delete it, or compile with `-o` to a path outside the
  tree); fresh extracts are clean.
- bash tool: `workdir` must EXIST before the call (create the dir from its
  parent first).
- `m211_listdir_probe/` at repo root is another lane's untracked scratch --
  leave it. `.kilo/worktrees/*` belong to their sessions -- do not touch.
- relay.py: `new --body-file` for item bodies; `fix --note` inline; `commit
  --lane compiler [-m] <ids> --path <repo-file>` when a non-item file (e.g.
  a release-check) changed too.
- Diagnostics for unresolved imports: env `XIOM_TRACE_IMPORT` prints
  `[imp]` traces (kept in-tree, env-gated).
- `gh` CLI IS available: `gh run list -L 8`, `gh run view <id>`, `gh run
  watch <id> --exit-status`, `gh release view v0.64.4 --json assets`.
- The vendored stdlib is now a proper git WORKTREE pinned at `3df0843`
  (not the old stale checkout). To re-pin later: `git -C
  E:\xiom-lang\stdlib worktree prune`; rename the old dir aside; `git -C
  E:\xiom-lang\stdlib worktree add --detach E:\xiom-lang\xiom\stdlib
  <sha>`; delete the old dir after verifying.
- Push regime: NEVER push without the owner's authorization. The v0.64.4
  pushes (20:21Z, 21:10Z, tag, release-record) were owner-authorized; the
  doorbell cron may also push staged work -- ALWAYS `git fetch` and check
  `origin/main` first.
- The scheduled "XIOM Heavy Suites" workflow is PRE-EXISTING broken
  (rustup/cargo-fuzz setup; ~45s failure on every run since 2026-09-21) --
  ignore it for release purposes.
- Lesson from m275/2045: `local.closure_locals` (and the other
  name-keyed maps) are NOT function-scoped -- when adding a new consumer,
  gate on the compiled LLVM type where possible (an env is always the
  `i64` slot) instead of trusting the name set.

NEW (2026-10-13, v0.64.5 triage + FFI batches):

- Reporter "exit 0" from `cmd /c "... & echo EXIT=%ERRORLEVEL%"` is a
  PARSE-TIME artifact (expands before the app runs); capture exit codes
  with `cmd /v /c "... & echo EXITCODE=!ERRORLEVEL!"` or PowerShell
  `$LASTEXITCODE`. T1's "silent exit 0" was really 0xC0000005; WER can
  hold the process so a short watchdog misses it -- check the Application
  Error event log.
- Control the PRECONDITION state in repros (T1: the kv AV only fired on
  an EMPTY store dir; once one run created seg-1 every later build
  served, silently invalidating a whole bisect round).
- The loader has TWO module paths: EAGER nested Module decls (small
  single-file/dir fixtures -> qualified type keys) vs the LAZY catalog
  injection (`collect_external_decls` -> flat, bare-deduped). A
  single-file fixture often CANNOT reproduce injection-path bugs; use a
  real project tree A/B (the engine scratch copies) as the witness.
- xiom-check's R39/R44 `colliding` triage (shape rules; identical-layout
  facade duplicates like the two stdlib AtomicInt decls stay bare) is THE
  collision authority -- derive new injection-side logic from that map; a
  hand-rolled bare-name pre-scan broke t2-queue (AtomicInt).
- Builtins (as_ptr/as_mut_ptr, null, ...) need an INFERENCE counterpart:
  call.rs lowers them, but bindings stay untyped unless
  infer_call_return_xiom mirrors them -- and `i8*` is ambiguous (Str vs
  *UInt8), so gate operator lowering on AST-level pointer-ness
  (`expr_is_pointer`), never on the LLVM type alone (Copy-Out Str-copied
  a *UInt8 return; `base + N` became str_concat).
- relay.py notes: PowerShell passes argv to python; EMBEDDED DOUBLE
  QUOTES break the argument (the note split at `->`). Use quote-free
  notes.
- cargo from the wrong cwd fails: use
  `cargo build --manifest-path E:\xiom-lang\xiom\Cargo.toml -p xiom`
  when the probe shell sits in another repo.
- Fixture conventions: pure-XIOM checks -> e2e fixture + ci e2e group 2
  filter; anything needing `--c-source` -> CLI test under
  crates/xiom/tests/ (ffi_fnptr_abi.rs pattern) + the front-locks list.

## 7. m-numbering

- m261-m292 + m293-m299 + m301-m309 + m310 LANDED; m311/m312/m313 are
  reserved by the in-flight fixers (O2 contract barrier, spawn readback,
  ffi.free guard lowering); **next free is m314**.

## 8. CONTINUATION PROMPT (paste into a fresh session)

```
Session: XIOM compiler lane (E:\xiom-lang\xiom), continuing the v0.64.5
program after the candidate-closing batches m276-m284. Read
docs/SESSION-HANDOFF.md FIRST (CURRENT STATE block at the top, the ops
lessons, and the remaining-queue list); consult docs/RUSTC-STUDY.md +
docs/rustc-study/ for reference patterns; rustc source is E:\repos\rust
(READ-ONLY; no git writes there).

STATE: v0.64.4 released (df493530) and verified. Git: origin/main =
91860f11 (fetch first -- the doorbell may push staged work). Local main = the latest commit (see git log), AHEAD by ~68 unpushed
commits (m276-m292 + m293..m299 + m301..m309 + m310 + docs + attempt logs); do NOT push
without the owner's authorization. Relay bus: latest compiler commits
97e524e/3554774/ec470bb (+ the m292..m310 fix notes); the check
file release-checks/compiler-compiler-v0.64.4.md is the candidate record
(the 17-item engine batch was queued into it on 2026-10-14).

CLOSED this program (reporter re-verify at the next archive): T1-T3,
B-17+B-16, B-23+B-24, B-11+B-20 readback family, 1549-packages-3,
1549-stdlib-5, 1549-stdlib-7 (cross-type generic callbacks), 1548-stdlib-3
(tostring import no longer hijacks adapter closure fields), 1548-bindings-4
(m292 worklist guard), 1548-stdlib-4
(@pre on &mut param fields -- real fix m291), 1548-stdlib-7 (guard-form
ensures + enforcement restored), 1548-stdlib-5 (annotated Slice locals),
B-25, B-26,
B-21, 1549-packages-8 (contract cycles), 1549-stdlib-6 (cross-platform
verified), 1358-orbitdb (not a compiler bug -- xiom-dbg binary confusion
diagnosed).

NEXT -- first INTEGRATE the pending fixer results, then start the engine wave
(witness-first triage; see the check file + the bus for details):
0. PENDING INTEGRATION (Agent Manager fixer worktrees; check `agent_manager
   list` + `git log --oneline main..fix/<branch>`):
   a. m313 fix/m313-ffi-free-guard (ses_ec537c634ffej3MZekLVoymbeR): COMMITTED
      32626879 -- ffi.free call sites lower through the arena-aware wrapper
      (the compiler half of REL-20261013-1445). Integrate first: cherry-pick -n,
      ci.yml token(s), COMPILER_BUGS section, build, battery, ONE batch commit;
      then ACCEPTANCE: compile %TEMP%\kilo\threadcheck\spincheck.xi with
      XIOM_STDLIB='E:\xiom-lang\stdlib' --force --no-cache, run with a 15s
      watchdog; must print v=7 fast. If green, append a compiler note to
      REL-20261013-1445-compiler (compiler half landed + acceptance green;
      alloc/free workarounds retirable at the next pin) + check-file bullet +
      relay.py commit --path.
   b. m312 fix/m312-thread-spawn-readback (ses_ec5377b3effeDu3aCdH56UcotP):
      no commits yet at 14:55Z. On idle+commit: integrate; then re-run
      dbg131s.xi/dbgworker.xi vs stdlib main (expect `joined 42` stable x5;
      probes in %TEMP%\kilo\threadcheck\; rawspawn/rawgeneric/structret are
      the isolation set). If green: phase-2 note on REL-20261013-1423-compiler
      (compiler half green; thread gate 2053/1627/1402 re-runnable at the next
      pin) + bullet + relay commit.
   c. m311 fix/m311-o2-no-fold-barrier (ses_ec54222e7ffeO1IOB0h3cLS72X): no
      commits yet. The -O2 contract constant-fold pruning finding
      (COMPILER_BUGS ~15886); no bus item -- fold into COMPILER_BUGS +
      handoff + a check-file history bullet.
1. ENGINE RELAY BATCH (17 items queued into the check file 2026-10-14,
   untriaged -- the next fixer-wave candidates, natural clusters):
   2042 + 2042-2 + 2227 (UInt64 bit-63 + Int128/UInt128; 2042 is HIGH),
   1158 (ident named spawn drops fn -- m292 catalog leaf-shadow class),
   2139 (enum-variant/geom name collision -- m298 class), 2256 + 2324-2 +
   2324-3 (constant surface: shadow hang, var fn export drop, diamond const),
   2215 + 2215-2 (Vec push in match arm; move-semantics pin), 2238 x3
   (CLI lane: vecvecstr / fnvalue_records / meshencoder), 1819 (Int ptr arg
   B-16 class), 2035 (&Str param spelling), 2119 + 1213 (observational/low).
   Keep 3-4 fixers in flight.
2. Everything else (m276-m310) is DONE and relayed; remaining non-code
   gates: owner push authorization (~68 unpushed commits), a pin carrying
   stdlib main + this tree, the archive reporter re-verification wave
   (incl. thread gate + spin acceptance), 2032 owner decision, macOS pair
   (needs a new archive), 1549-packages-2 faithful repro, release-gate record.
   The stdlib-lane verification notes for 1423/1445 are ALREADY on the bus
   (97e524e: 1423 hang gone/readback localized; 1445 spin + call-site
   finding) -- only the phase-2 green notes remain.

Number from m314 (m311/m312/m313 reserved); ONE commit per batch (fix +
fixtures + docs + COMPILER_BUGS section) + a relay fix-note with numeric/IR
witnesses + a check-file row.

RULES: reproduce from the item's witness before touching code; battery =
e2e g1/g2 filters at ci.yml lines 152/153 + the front-locks CLI list
(incl. ffi_ptr_arith/ffi_ptr_slot_readback/null_literal_marshal/
ffi_fn_env_cast) + xiom-check --lib -- --test-threads=1 + xiom-codegen +
xiom-graph + xiom-pkg; A/B with the PINNED stdlib by setting
XIOM_STDLIB=E:\xiom-lang\xiom\stdlib explicitly (exe-ancestor order
otherwise picks the live tree and can mix module bodies across trees);
capture compiler output to files; relay notes must be quote-free
(PS->python argv); use cargo --manifest-path when the shell sits outside
the repo; fixture convention = pure-XIOM -> e2e fixture + ci group 2,
--c-source shapes -> crates/xiom/tests CLI test; edit markdown with the
edit tool or Python (PowerShell text appends convert line endings and
produce whole-file diffs -- verify git show --numstat before committing);
WSL checks run the Windows compiler with Windows-style paths and link
Linux binaries; do not gate on xiom-dbg (DAP adapter, exits 0); house
voice; ASCII; never push without owner authorization.

HYBRID RECON (2026-10-13, owner-directed): fan out READ-ONLY triage to
subagents to compress the queue. Pattern that works: 3-4 parallel `general`
subagents (Task tool), one per item -- each reads its relay item + witness,
reproduces with the PINNED stdlib (`XIOM_STDLIB=E:\xiom-lang\xiom\stdlib`)
using the already-built `target/debug/xiom.exe`, and returns a concise
report: repro rc, root cause with file:line, minimal fix proposal, blast
radius, and a suggested lock. HARD LIMITS for subagents: NO file edits, NO
cargo builds (the main lane owns the target dir), scratch only under
%TEMP%\kilo\; the main session applies and verifies every fix serially (ONE
commit per batch, full battery). Integrate the topmost tractable diagnosis
first; the two decision items (1548-stdlib-6 leg b, 1548-bindings-4) each
have a precise attempt log in docs/failed_attempts.md.
FIX WAVE (2026-10-13, owner-directed): fixers now run in isolated Agent
Manager WORKTREES (m293-m295 branch seeds: packages-11, packages-9,
stdlib-6 leg b) -- each fixes + tests in its own worktree and commits
source+tests only; the main lane cherry-picks serially and owns
docs/relay/CI rows + the final full battery.

PARALLEL SELFHOST: phase-4 worktree (ses_eded8c4d8ffe) is idle and the
sole writer there; sibling ses_ef8e3a27cffe stays idle. Prompt it only if
it needs to continue docs/checklists/selfhost-phase6.md on the current
main (post-m284).
```
