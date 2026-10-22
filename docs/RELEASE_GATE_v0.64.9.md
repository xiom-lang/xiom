# Release gate -- v0.64.9

STATUS: **WITHDRAWN FROM ADOPTION 2026-10-22T18:30Z** -- GitHub release
marked pre-release; production remains v0.64.8; v0.65.0 supersedes. Known
blockers: nested enum-pattern payloads (Some(Number(n)) / Ok(Number(n)) /
Err(E.Bad(msg))), m372/1451 and true 1550 regression in `d04cd3265`,
non-transitive dependency port args (m1511), and C07. The tag and assets
remain for audit only. Do not pin or deploy v0.64.9. See
`docs/RELEASE_GATE_v0.65.0.md`.

Historical release data (for the audit): compiler tag `v0.64.9` (annotated
`ffde5aa0f88591b40db949239340cf66a36c3e41`) on `2bcf58db5`; release run
**39791522014 SUCCESS**; 11 assets; SHA256SUMS
`51c45a61dbfebb930acc99ad8cc383234482fc0ba52961628b8f7ffdf6a9ebac`.
Stdlib `stdlib-v0.64.9` at `aa8b656`; release run **39785430606 SUCCESS**;
`xiom-std 0.64.9` published (sha256
`6f0d3b1516c7c94fcc6fd7fc1fc34fa66025c9543456a4c2effa16142da73020`,
compiler v0.64.8, 2026-10-22T14:01:23Z; production publish run
39785430612 SUCCESS).

## Scope

Fixes since v0.64.8 (`4d040604b`), on main `67940f924` plus the release
prep commit:

- m1157 UTF-8 BOM catalog header discovery (`18415ede2`)
- 2045 part 2 boxed Option/Result Str payload binding (`9d31321df`)
- XVC-C-22 Vec element inference / PQ codebook and second-call AV
  (`d38cff2a8`)
- m1731 borrowed-enum match tag load (`ace095744`)
- m1703 stored function values, Vec alias env wrap, tuple returns
  (`c9b1f1009`)
- m1326 enum field equality discriminant (`d50d1436e`)
- m1520 adapter generic substitution (`_max_via[U]`) (`d04cd3265`)
- 1550 compiler groundwork + stdlib `EnumerateIter[Int].map`
  specialization (`d04cd3265`; stdlib `d53f93a`)
- m365 shared dependency-object key profile (`70b380c4e`)
- 1511 package.xi `deps:` alias so dependency port args apply
  (`e45059b76`)
- 1531 dead-builder cache-lock break (`1fb407377`)
- m375 opt-in `XIOM_FAST_EXIT` (`b51ce6ec2`)

## Preconditions (before the tag)

- [x] stdlib-v0.64.9 tag exists at `aa8b656` (annotated tag pushed;
      release run 39785430606 in progress), with `COMPILER_VERSION=v0.64.8`.
- [x] `STDLIB_VERSION=stdlib-v0.64.9` on the release commit.
- [x] Workspace version 0.64.9, Cargo.lock, selfhost `SELFHOST_VERSION`,
      VSIX README pin (`editors/vscode/README.md`).
- [x] `release-notes/v0.64.9.md` merged with the two stdlib highlights and
      converted to JSON with `xiom-release-notes`; verify passed (6
      highlights, 0 breaking).
- [x] Full battery on the release commit: feature_regression 549/549,
      integration 130/130, stdlib_tests 40/40, codegen lib 25/25, xiom lib
      77/77, checker 214/214, parser 109/109, plus the new 1550 e2e lock.
- [x] Packaged archive dry-run (Windows/Linux/macOS/VSIX/wasm) on the
      release commit: run 39785770942 SUCCESS (all package legs + VSIX;
      publish skipped on dispatch).
- [x] Registry `xiom-std` pin decision recorded by the stdlib lane
      (`xiom-std 0.64.9`, compiler v0.64.8, OIDC publish).
- [ ] v0.64.8 check rows closed or explicitly waived; v0.64.9 check opened
      (v0.64.9 check opened; reporter rows pending).

## Tag + publish

- [x] Annotated tag `v0.64.9` on `2bcf58db5`; pushed (tag object
      `ffde5aa0...`).
- [x] Release workflow 39791522014 SUCCESS; 11 assets + SHA256SUMS +
      docs dispatch.
- [x] stdlib registry publish: production run 39785430612 SUCCESS;
      `xiom-std 0.64.9` stable. The release's staging-canary dispatch
      (39791446653) is waiting for environment approval and is optional.
- [x] Relay announcement + lane unblocks (engine/xvector/orbitdb/packages/
      PULSE).

## Post-release verification

- [ ] Engine: m365 117-target assimp re-measure; archive under the 2 GB
      cap.
- [ ] Orbitdb: m375 1M A/B with `XIOM_FAST_EXIT=1`; A6 exact verify.
- [ ] Xvector: XVC-C-22 probe, 1531 build under the 16 GB Job Object,
      1243 shard JSON read.
- [ ] Packages: installed fontraster build with dependency port args;
      unhold publish.
- [ ] Stdlib: 1550 e2e lock on the synced pin.
