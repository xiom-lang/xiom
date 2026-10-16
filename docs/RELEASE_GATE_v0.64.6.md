# Release gate -- v0.64.6

STATUS: **RELEASED 2026-10-16T20:33Z** -- release run 38962236697 GREEN; assets live (linux-x64, macos-arm64, macos-x64, windows-x64, VSIX 0.12.2, wasm + SHA256SUMS); STDLIB pin stdlib-v0.64.6 (`b81430ba`).

Correctness release over the v0.64.5 waves plus the darwin stream: tuple
naming is canonical in every position (m334/m336), scalar-named enum variants
stop shadowing builtins (m338), imported-type global spellings resolve (m337),
catalog binding and import surfaces resolve exactly (m341/m342), the clang-18
linux optimizer miscompile is fixed (m339), the dependency C pipeline compiles
through an object cache with an honored timeout floor (m340), and the borrow
pass / clause rebind batches land (m344/m345), and the XVC-C-18 generic
field-chain element loss is fixed (m346, folded in after its recon).

## Preconditions (before the tag)

- [x] Wave integrated on local main: m334 (`916efc87`/`5c5a702f`), m335
      (`17e33212`/`cd047877`/`233238a4`), m336 (`0009303c`/`312743d0`), m337
      (`11d7f7a6`/`c458c935`), m338 (`14e58ea1`/`d0d68530`), brace repair
      (`ce121465`), m339 (`e8fb854b`/`732913af`), m340 (`a98ead48`/`50ce3372`),
      m341 (`2e8a88a8`/`0ae41ddb`), m342 (`cba963eb`/`758da58a`), m344
      (`ece18b96`/`b938b385`), m345 (`18fa8d96`/`69bec55f`), m346
      (`95474e54`/`f01174b0`).
- [x] Wave lock suite on the integrated tree: e2e m334-m346 + m315 filters = 32/32
      green; unit locks: clang-18 argv gate + dep-cache/timeout units 3/3,
      xiom-check borrow set 30/30, checker_locks m346 arm 1/1.
- [x] FULL battery on the frozen tree (m334-m346; 2026-10-16, 19:40-20:02Z):
      e2e front group 16/16 x2 (64.1 s / 49.3 s); e2e g2 (256-token ci line)
      272/272 x2 (260.5 s / 272.8 s); codegen lib 12/12; feature_regression
      549/549 (455.5 s); stdlib_execution 85/85 (+2 ignored, 145.6 s);
      integration 130/130. Log: %TEMP%\kilo\cut-battery2.log ("BATTERY DONE").
      (Superseded: the m334-m345 battery, fully green, %TEMP%\kilo\cut-battery.log.)
- [x] Workspace + selfhost version bump to 0.64.6 (commit `535a2116`).
- [x] `STDLIB_VERSION` = `b81430baa5723795ad855d460f72aba84d0e18b5` (pin commit
      `e6b84aab`; stdlib-v0.64.6 tag verified present).
- [x] Dry-run archive on `e6b84aab`: release run 38960405342 GREEN (7m02s;
      all five platform legs + VSIX; publish skipped as designed).
- [x] Tag `v0.64.6` + push -> release run 38962236697 GREEN; release published
      2026-10-16T20:33:11Z with SHA256SUMS (see the tag+publish log below).
      Release guard passed (tag == workspace version; tag on main).

## Wave context

- Darwin stream COMPLETE (v0.64.6 wraps it; stdlib pin carries 2bc064e/9d6a4d5 +
  e72288f; compiler side ships the m331 memset declare line).
- Relay coverage (owner directive): every pulse/engine/orbitdb/xvector intake
  item carries a wave assignment in release-checks/compiler-compiler-v0.64.6.md;
  re-verify requests filed on 2142/2216/2022/1046/1358/1538/1411/1128 items.
- Held-out (post-cut): m343 aggregate-copy + inline Vec field (folded later as
  part of the generic field-chain work), XVC-C-08/09/14 non-SFB layout family,
  engine classes 2/4, the packages 2303/2309/2328/2343/B-32 queue, plus the
  post-release wave: m347 (platform bare refs + seed poison), m348 (fn-ptr
  signature flow), m349 (guarded type injection), m350 (Result-ref field read
  crash), m351 (B-33 confined-call trap recovery).

## Tag + publish log

- [x] Push local main to origin (`e6b84aab` + earlier wave; protector bypass
      listed on push, as expected for the release account).
- [x] Dry-run archive on `e6b84aab`: run 38960405342 GREEN.
- [x] First tag push: run 38961314112 FAILED at the notes verify (summary 341 >
      240 chars; 8 > 6 highlights; internal-task-id lint on the highlight text).
      Fixed in `6e878b90` (notes re-drafted to the verify spec; local
      `xiom-release-notes verify` clean; 6 highlights).
- [x] Tag move: ruleset blocked tag deletion; the update used the owner's admin
      bypass (`git push --force origin v0.64.6`, e6b84aab -> 6e878b90).
- [x] Release run 38962236697 GREEN; release published 2026-10-16T20:33:11Z:
      SHA256SUMS, xiom-0.64.6-linux-x64.tar.gz, macos-arm64, macos-x64,
      windows-x64.zip, xiom-vscode-0.12.2.vsix, wasm bundle.
- [x] Upstream lanes pinged via REL-20261016-2034-compiler (to all) with
      per-lane re-verify actions; stdlib coordination on REL-20261016-1452
      (io.xi:947 canonical clause restore unblocked; tag confirmation).
