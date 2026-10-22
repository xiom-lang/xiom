# Release gate -- v0.64.9

STATUS: **PREPARING** -- owner GO received 2026-10-22T13:01Z. No tag or
release yet. The stdlib tag lands first (docs/RELEASE_CHECKLIST.md): the
stdlib release pins the last released compiler (v0.64.8); this compiler cut
then pins `stdlib-v0.64.9`.

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

- [ ] stdlib-v0.64.9 tag exists, cut from `d53f93a` (or the stdlib release
      commit), with `COMPILER_VERSION=v0.64.8`; stdlib gate run green.
- [ ] `STDLIB_VERSION=stdlib-v0.64.9` on the release commit.
- [ ] Workspace version 0.64.9, Cargo.lock, selfhost `SELFHOST_VERSION`,
      VSIX README pin (`editors/vscode/README.md`).
- [ ] `release-notes/v0.64.9.md` merged with the two stdlib highlights and
      converted to JSON with `xiom-release-notes`; schema-valid (max 6
      highlights).
- [ ] Full battery on the release commit: feature_regression 549/549,
      integration 130/130, stdlib_tests 40/40, codegen lib, xiom lib,
      checker, parser, plus the new 1550 e2e lock.
- [ ] Packaged archive dry-run (Windows/Linux/macOS/VSIX/wasm) on the
      release commit.
- [ ] Registry `xiom-std` pin decision recorded by the stdlib lane.
- [ ] v0.64.8 check rows closed or explicitly waived; v0.64.9 check opened.

## Tag + publish

- [ ] Annotated tag `v0.64.9` on the green release commit; push the tag.
- [ ] Release workflow publishes 11 assets and dispatches docs.
- [ ] Relay announcement + lane unblocks (engine/xvector/orbitdb/bindings/
      packages/PULSE).

## Post-release verification

- [ ] Engine: m365 117-target assimp re-measure; archive under the 2 GB
      cap.
- [ ] Orbitdb: m375 1M A/B with `XIOM_FAST_EXIT=1`; A6 exact verify.
- [ ] Xvector: XVC-C-22 probe, 1531 build under the 16 GB Job Object,
      1243 shard JSON read.
- [ ] Packages: installed fontraster build with dependency port args;
      unhold publish.
- [ ] Stdlib: 1550 e2e lock on the synced pin.
