# Release gate -- v0.64.8

STATUS: **IN PREP 2026-10-19** -- owner GO received. Not released. The
official v0.64.7 archive is not re-cut; v0.64.8 is the next cut and the
archive requested by the engine in `REL-20261019-1312-engine`.

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

- [ ] **stdlib-v0.64.8 tagged and published**; `STDLIB_VERSION` pinned to it.
      Hard gate: no compiler tag before the stdlib is on the same track.
      Request: `REL-20261019-1432-compiler` (stdlib lane). The pin must carry
      `release-notes/v0.64.8.md` for the combined notes.
- [x] Workspace + selfhost version bump to 0.64.8 (`c45a247d2`), Cargo.lock,
      VSIX README pin.
- [ ] Release notes `release-notes/v0.64.8.md` converted to JSON with the
      pinned stdlib fragment; `cargo run --quiet -p xiom-release-notes --
      verify --tag v0.64.8 --stdlib <pinned-checkout>` green.
- [ ] `STDLIB_VERSION=stdlib-v0.64.8` commit on main.
- [ ] Candidate dry-run archive on the pin commit: `gh workflow run
      release.yml -f version=0.64.8`; all platform legs + VSIX + wasm +
      SHA256SUMS green; every staged tool reports 0.64.8.
- [ ] Full battery on the release commit: e2e_tests (full), feature_regression,
      integration, stdlib_execution, stdlib_tests, stdlib_api_freeze, checker.
      Do not run concurrently with the engine's assimp editor build; the
      release binary may be locked by that build (use the debug binary for the
      harness, or build the release into a separate CARGO_TARGET_DIR).
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

- [ ] Push the pin/notes commits to main.
- [ ] Dry-run workflow green; record the run id.
- [ ] Tag `v0.64.8`; guard green (tag == workspace 0.64.8, tag on main).
- [ ] Release published; SHA256SUMS + assets verified; docs dispatch.
- [ ] Announce and unblock the lanes (engine archive, xvector/orbitdb/
      bindings/pulse re-verify).
