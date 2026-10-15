# Release gate -- v0.64.5

STATUS: pin pending -- waiting for the stdlib-v0.64.5 tag reply
(REL-20261014-1739-compiler). Tag + publish section fills in after the pin
battery.

Correctness release over the m276-m321 program: the thread gateway closes
(compiler read path m312 + runtime trampoline wave 139), 128-bit and
unsigned integer semantics land (m314), confined-aggregate copy-out and
the alloc-guard-spin close (m313/m318), and a broad resolution sweep
(constants, imports, name collisions, soft keywords, pointer spellings)
lands m315-m320. The last two engine observationals (2119/1213) are
triaged to probe-side UB with the read-bounds contract locked (m321).

## Preconditions (before the tag)

- [x] Release-wave push: origin main `1e4a444e` (90 commits from
      `91860f11`); workspace + `SELFHOST_VERSION` 0.64.5; VSIX README pin.
      `xiom --version` reports v0.64.5.
- [x] Dry-run archive: release run `38626737822` on `1e4a444e` GREEN
      (19:48-19:52Z).
- [x] FRESH release wave: origin main `13081238` (m322 release-gate
      full-suite fixes + m323-m326 all four 2123 classes + m327 the m326
      follow-up regression fix); full e2e on that sha 2550 passed / 0
      failed / 4 ignored.
- [x] Fresh dry-run archive: release run `38731274663` on `13081238`
      GREEN -- all legs success (linux-x64, windows-x64, macos-x64,
      macos-arm64, VSIX, guard).
- [x] `STDLIB_VERSION` = `e08262316d3c53dc2559d951362d9740adb2143f`
      (stdlib-v0.64.5, annotated tag object ecc3549...); repo-local
      `stdlib/` refreshed to the tag; stdlib candidate battery GREEN on
      archive `38731274663` (corpus 955/955, probes 315/315, modules
      510/510, barename 0/510, floors/ratchets OK, 27-repro sweep
      matches; their thread check 5/5 dbg131s + 5/5 dbg139w).
- [x] `release-notes/v0.64.5.json` committed; converter + verify green
      (merges the stdlib fragment; 6 highlights, 0 breaking, 2 docs).
- [ ] VSIX: toolchain-only -- `editors/vscode/package.json` stays 0.12.2.

## Gate A -- suites at the pin state

- [x] full e2e (whole list): 2550 passed / 0 failed / 4 ignored (1109.94s).
- [x] feature-reg 549; workspace libs incl. checker 199, parser, codegen
      12, graph, verify, pkg, release-notes 6; fuzz 24; robustness 63;
      perf 3/3 (fmt_peek 107,728 vs budget 112,000); stdlib api_freeze
      3/3 (m329 repair); stdlib_tests 40; stdlib_execution 85 (+2
      ignored); front-locks green; ascii guard OK.
- [~] nine-tool release build: the pin-state PR CI + the v0.64.5 tag
      release run are the authoritative build (dry-run `38731274663`
      green pre-pin-fixes).
- [x] thread acceptance AT THE PIN (vendored tag tree): dbg131s x5
      `joined 42`; dbgworker x5 `val=42 done=1`.

Pin-gate fix round (all landed on main before the PR): m328 (perf
budget: leaner guards + documented 95k->112k), m329 (freeze: wave-105
canonicalization + wave-98 snapshot repair), m330 (POSIX test dir +
rustls 0.23.46 RUSTSEC-2026-0346..0349), m331 (memset declare, quick
Darwin fix).

## Blockers / notes

- Release-gate round m322-m327: the full-suite gate found and fixed three
  latent regressions (m322: signed cast compares + scalar-pointee field
  indexing) and the fresh-wave full suite found + fixed the m326
  follow-up store-back regression (m327); plus all four stdlib candidate
  classes (m323-m326). Full e2e re-gate on the frozen sha `13081238`:
  2550 passed / 0 failed / 4 ignored.
- 1423 (thread gateway) closed jointly: compiler half m312 `bc1d821b` +
  runtime half stdlib wave 139 `ae9c963`; acceptance green pre-pin AND at
  the pin (vendored tag tree: dbg131s 5/5 `joined 42`, dbgworker 5/5
  `val=42 done=1`).
- 1548-bindings-5 (alloc-guard-spin) closed: m313 `56234edf` + the wave
  124 runtime; spincheck 3/3 `v=7`.
- 2119/1213 triaged probe-side UB (m321 `88d84f5b`); read-bounds contract
  locked.
- stdlib bonus finding: `p_polyhedra_nested_hull` is green on the
  candidate (5/5) after being red on v0.64.4 -- item updated stdlib-side.
- (battery bounces, if any, go here)

## Gate D -- tag + publish

- [ ] pin commit (STDLIB_VERSION + vendored refresh + notes JSON + this
      record) pushed to origin main.
- [ ] tag `v0.64.5` pushed -> release run <TBD> GREEN; assets live (not
      draft) incl. `SHA256SUMS`.
- [ ] release recorded in the candidate record + handoff.
