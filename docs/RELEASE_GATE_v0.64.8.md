# Release gate -- v0.64.8

STATUS: **RELEASED 2026-10-19T17:49Z** -- owner GO received. Tag
`v0.64.8` (annotated `b0261db4d39f532a4f2cd697faed271c0228fe15`) on
`4d040604b`; release run **39296778685 SUCCESS** (guard,
windows/linux/macOS/VSIX packaging, publish, docs dispatch); GitHub release
live with 11 assets. The stdlib r2 gate run **39289746856 SUCCESS**
(`smoke_io_edge` fixed); `STDLIB_VERSION=stdlib-v0.64.8` at the tag.
v0.64.7 is not re-cut; v0.64.8 is the archive requested by the engine in
`REL-20261019-1312-engine`. The reporter rows below are post-release
verification: the archive is live and the lanes fill the v0.64.8 check.

## Scope

Runtime memory and type fixes since v0.64.7, from main `ac20559f9`:

- m360 i128 division/modulo lowering (`dc30929da`)
- m368 Vec-drop reclaim (`5a530cf7e`)
- escaped/per-frame Vecs -- documented scoped arena (`9c8842b5e`)
- m369 catalog index scan/cache (`330955136`)
- m370 generic-struct Vec element resolution (`f7fe288e3`)
- m367 fixture-tree catalog exclusion (`90244d42c`)
- C-ORBIT-12 alias position tuple (`c6b3362ef`)
- C-ORBIT-13 nested `&mut` field chain (`7cee95ac2`)
- XVC-C-20 module-global `&mut` aggregate (`56d362a31`)
- B-34 generic-argument alias unification (`344abbc23`)
- B-32 retired / B-29 confirmed not-a-compiler-defect (bindings correction)

Not in this cut (next cycle): m371 PULSE imported-struct layout, m366
capture-frames hang, m365 caller flag-split/per-key lock, m363 Send/Sync
enforcement, MCU/bare-metal targets, selfhost Phase 6 (separate track).
These are in `release-notes/v0.64.8.md` known issues and the v0.64.8 check.

## Preconditions (before the tag)

> Reporter verification rows (engine/xvector/orbitdb/bindings/PULSE) are
> post-release and are tracked in the v0.64.8 check; they were not tag
> blockers.

- [x] **stdlib-v0.64.8 r2 gate green**; `STDLIB_VERSION=stdlib-v0.64.8`.
      Run 39289746856 SUCCESS (Ubuntu + Windows full smoke + package);
      tag moved to `112c5b6` with
      `COMPILER_VERSION=candidate-v0.64.8-r2`. The pin/fragment stays
      `release-notes/v0.64.8.md`.
- [x] Workspace + selfhost version bump to 0.64.8 (`c45a247d2`), Cargo.lock,
      VSIX README pin.
- [x] Release notes `release-notes/v0.64.8.md` converted to JSON with the
      pinned stdlib fragment and verified (6 highlights, 0 breaking).
- [x] `STDLIB_VERSION=stdlib-v0.64.8` on main (the tag moves to r2).
- [x] r1 packaging dry-run 39283297787 SUCCESS on `39f1fddff`; the r1
      compiler is rejected by the stdlib gate.
- [x] r2 candidate dry-run archive on `4d040604b`: 39287949178 SUCCESS;
      all platform legs + VSIX + wasm + SHA256SUMS green.
- [x] Full battery on the release commit (battery2): e2e **2612/2613 + m17
      clean re-run** (the one failure was a local duplicate stdlib root; m17
      passes with the staged lib removed), feature_regression 549/549,
      integration 130/130, stdlib_execution 85/85, stdlib_tests 40/40,
      api_freeze 3/3, rtcache/JIT 72/72, checker 212/212, codegen lib 23/23.
      Log `%TEMP%\kilo\v0648-battery2.log`. The engine assimp build finished;
      the release binary is free.
- [ ] Engine 2032/2053/1343 + editor soak under the 2 GB cap (600+ frames,
      flat WS) on the candidate archive.
- [ ] Xvector 53-probe battery + XVC-C-20/G-2b on the candidate.
- [ ] Orbitdb 160/160 with the natural position tuple + C-ORBIT-13.
- [ ] Bindings B-34 verify; B-29 closed; B-32 correction closed.
- [ ] PULSE keeps the 1309/2301 workaround (m371 next cycle).
- [ ] Registry `xiom-std` pin decision recorded (stdlib/registry).
- [ ] v0.64.7 check rows closed or explicitly waived; v0.64.8 check verdicts
      filled by reporters.
- [ ] Tag `v0.64.8` on main; release workflow publishes; docs dispatch fires.

## Environment notes

- `RELEASE_BUILD_MACOS=true` (verified), so macOS x64/arm64 are expected in
  the matrix, matching v0.64.7.
- `linux-arm64` is in the matrix; the Pi/arm64 re-verify is still a packages
  item (`REL-20261018-1432-pulse`).
- The engine lane is currently building the editor with
  `E:\xiom-lang\xiom\target\release\xiom.exe` (0.64.7 label; the binary
  carries the m368/arena fixes from before the version bump). Do not rebuild
  that path until the build exits. CI builds the release archive fresh.
- Release checks: `release-checks/compiler-compiler-v0.64.7.md` (still open,
  114 unverdict rows) and `release-checks/compiler-compiler-v0.64.8.md`
  (opened for this cut).

## Tag + publish log

- [x] Push the pin/notes commits to main (`4d040604b` tagged).
- [x] Dry-run 39276425781 attempted on `6879d694a` (stdlib pin `9b88b46`)
      -- FAILED at "Checkout stdlib at the pin" because the stdlib commit is
      local-only; not a compiler failure.
- [x] r1 packaging dry-run 39283297787 SUCCESS (r1 rejected by the stdlib
      gate).
- [x] r2 dry-run 39287949178 SUCCESS on `4d040604b`.
- [x] Tag `v0.64.8` (annotated `b0261db4d39f532a4f2cd697faed271c0228fe15`)
      on `4d040604b`; guard green (tag == workspace 0.64.8, tag on main).
- [x] Release run 39296778685 SUCCESS; release published 2026-10-19T17:49Z
      with 11 assets; SHA256SUMS
      `f390844ff2f9b5eb524a1a41b97037862c1e1a7ffeedc127856f87715cbd11ad`;
      windows-x64 `c31d9c46...`; linux-x64 `a8c003c0...`; linux-arm64
      `e773260f...`; docs dispatch success.
- [x] Announce and unblock the lanes (engine archive, xvector/orbitdb/
      bindings/pulse re-verify) via the relay bus + v0.64.8 check.
