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
- [ ] `STDLIB_VERSION` = <TBD: stdlib-v0.64.5 tag sha>; repo-local
      `stdlib/` refreshed to that ref; stdlib candidate checks on the
      archive + tag cut (their reply).
- [ ] `release-notes/v0.64.5.json` committed; convert+verify green
      (merges the stdlib fragment; 6 highlights expected).
- [ ] VSIX: toolchain-only -- `editors/vscode/package.json` stays 0.12.2.

## Gate A -- suites at the pin state

- [ ] full e2e (whole ci.yml e2e list) __/__.
- [ ] feature-reg 549; checker 199 serial; parser; codegen 12; graph 36;
      verify 9+37; release-notes 6; pkg 79; stdlib_tests + execution +
      api_freeze; front-locks; ascii guard.
- [ ] nine-tool release build; every tool reports v0.64.5.
- [ ] thread acceptance AT THE PIN (vendored tag tree):
      dbg131s.xi + dbgworker.xi x5 -> `joined 42` / `val=42 done=1`.

## Blockers / notes

- 1423 (thread gateway) closed jointly: compiler half m312 `bc1d821b` +
  runtime half stdlib wave 139 `ae9c963`; pre-pin acceptance green 5/5 on
  both probes at stdlib main.
- 1548-bindings-5 (alloc-guard-spin) closed: m313 `56234edf` + the wave
  124 runtime; spincheck 3/3 `v=7`.
- 2119/1213 triaged probe-side UB (m321 `88d84f5b`); read-bounds contract
  locked; no engine-side action needed.
- (battery bounces, if any, go here)

## Gate D -- tag + publish

- [ ] pin commit (STDLIB_VERSION + vendored refresh + notes JSON + this
      record) pushed to origin main.
- [ ] tag `v0.64.5` pushed -> release run <TBD> GREEN; assets live (not
      draft) incl. `SHA256SUMS`.
- [ ] release recorded in the candidate record + handoff.
