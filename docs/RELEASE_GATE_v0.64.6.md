# Release gate -- v0.64.6

STATUS: pin READY -- battery GREEN on the frozen wave (m334-m346, rerun after the XVC-C-18 fold-in); cut commit +
notes ready; tag + publish section fills in after the owner go and the dry-run
archive. STDLIB_VERSION target = `b81430baa5723795ad855d460f72aba84d0e18b5`
(stdlib-v0.64.6, annotated tag object; carries e72288f + waves 151-154).

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
- [ ] Workspace + selfhost version bump to 0.64.6 (bump commit staged).
- [ ] `STDLIB_VERSION` = `b81430baa5723795ad855d460f72aba84d0e18b5`
      (pin commit staged; stdlib-v0.64.6 tag verified present).
- [ ] Dry-run archive on the pin sha (release.yml workflow_dispatch) -- runs
      after push; fills in below.
- [ ] Tag `v0.64.6` (must equal the workspace version; must be an ancestor of
      main per the release guard) + push -> release run builds the archives.

## Wave context

- Darwin stream COMPLETE (v0.64.6 wraps it; stdlib pin carries 2bc064e/9d6a4d5 +
  e72288f; compiler side ships the m331 memset declare line).
- Relay coverage (owner directive): every pulse/engine/orbitdb/xvector intake
  item carries a wave assignment in release-checks/compiler-compiler-v0.64.6.md;
  re-verify requests filed on 2142/2216/2022/1046/1358/1538/1411/1128 items.
- Held-out (post-cut): m343 aggregate-copy + inline Vec field, XVC-C-18
  (JSON keys via match binding -- RECON launched at the 18:15 sweep), the m339
  queued second non-SFB layout corruption (XVC-C-08/09/14 family), engine
  classes 2/4, packages 2303/2309/2328/2343/B-32 queue.

## Tag + publish (fills in after the owner go)

- [ ] Push local main to origin (all wave commits; owner go required).
- [ ] Dry-run archive on the pin sha -- run id TBD.
- [ ] Tag `v0.64.6` + push; release run id TBD; assets verified on the
      release page; upstream lanes pinged (engine pin, xvector WSL re-pin,
      orbitdb re-verify, packages windows).
