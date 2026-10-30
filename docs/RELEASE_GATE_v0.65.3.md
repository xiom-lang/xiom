# Release gate -- v0.65.3

STATUS: **RELEASED 2026-10-30T15:30:58Z** -- `v0.65.3` live (tag object
`8cf720c3`, commit `e2f238cb6`); release run `41077253955` SUCCESS with
**11 assets + SHA256SUMS**; bundled stdlib `stdlib-v0.65.3` (26ede85,
lib/package.xi 0.65.3). Lanes are doing post-release asset smoke.

## Scope

- Aggregate-local ownership across nested struct literals (REL-1324,
  orbitdb 0.2.1 carve): verified + closed by orbitdb on `ed2f38cff`.
- Aggregate Vec-field drop re-read + implicit tail-return ownership
  (REL-1641): verified + closed by stdlib on `605c6a879` (p_wave162 20/20,
  UAF witness 20/20, probes 418/418, smokes 963/963, modules 533/533,
  gate9 OK).
- Helper-wrapped aggregate returns, projected-field moves, by-value
  aggregate args to retaining callees (REL-2110 b4 + UDP u4): transport
  suite 36/36 x5 debug + x5 release; sim 5/5 ASan+release; carve 160/160;
  gates 20/20; e2e 2667/2667 (4 ignored) both; CI `40975146164`.
- stdlib non-owning Channel + Huffman decode; stdlib package ready for the
  owner's GO.

Not in this cut: Map/JSON/enum-tree retention (REL-1704), `Vec[Str]`
element strings, removal containers, callee-side by-value parameter drops,
the engine XgView/Vec rows.

## Preconditions

- [x] Owner GO recorded (2026-10-29T22:58Z).
- [x] Candidate commit `fd27be39a`; CI `40975146164` SUCCESS (all matrix).
- [x] 20/20 memory gates debug + release.
- [x] Full e2e 2667/2667 (4 ignored) debug + release.
- [x] 0.2.1 orbitdb carve 160/160; orbitdb modern/station/hybrid green
      on `ed2f38cff` (1324 verified/closed).
- [x] stdlib full matrix green on `605c6a879`; 1641/1910 verified/closed.
- [x] Transport suite 36/36 x5 debug + x5 release (b4 + u4 green).
- [ ] packages verifies `REL-20261029-2110-packages` on `fd27be39a`.
- [x] stdlib tag `stdlib-v0.65.3` live on `801d300` (created by the
      compiler lane after the stdlib verify; `STDLIB_VERSION` pinned).
- [x] Version bump 0.65.3 on `rc/v0.65.3` (`029d54948`); Cargo.lock.
- [ ] Release notes `v0.65.3.md`/`.json`; selfhost/VSIX pins.
- [x] All-platform dry-run `41065811987` SUCCESS on `c8b1fb1b1`
      (windows-x64, linux-x64+arm64, macos-x64+arm64, VSIX). The first
      dry-run `40980879241` was STALE: it bundled stdlib 0.65.2 because the
      tag still pointed at 801d300 before stdlib re-cut it to 26ede85. The
      rebuilt archive bundles lib/package.xi 0.65.3 (verified).
- [ ] Lanes verify the archive (requested `REL-20261030-0049-compiler`).
- [ ] Annotated tag `v0.65.3` on the rc commit merged to main; release
      workflow.
- [ ] Registry `xiom-std 0.65.3`, docs dispatch, lane announcements.

## Evidence

- Candidate `target\candidate\xiom-fd27be39a.exe`, sha256
  `C45E82A48AE16885DC4B75F5F4BC8E7F3415D4581743A7B64B724A935EEC846A`.
- 0.2.1 carve 160/160; sim `k0=1 k2=2`; transport u4 byte-exact 1500.
- e2e debug 970 s, release (final run) recorded below.
- Witnesses: `aggregate_nested_return`, `aggregate_vec_field_grow`,
  `tail_return_aggregate`, `aggregate_escapes_callee`,
  `return_helper_aggregate` (all pre-fix red, post-fix green).

## Release log

- [x] rc `c8b1fb1b1` (version 0.65.3); rebuilt dry-run `41065811987`
      bundling stdlib 0.65.3; archives: windows-x64 `fbb69538...`,
      linux-x64 `9435e745...`, linux-arm64 `40d30adc...`, macos-x64
      `aac5c305...`, macos-arm64 `ddf0d7b1...`, vsix `065e33df...`.
      The earlier `029d54948`/`40980879241` archive (`7ff7919d...`) is
      superseded (stale stdlib 0.65.2).
- [x] Fresh lane archive verdict: packages PASS/GO on the rebuilt RC
      (`fbb69538`, bundled stdlib 0.65.3). The final release asset
      (`8383aa5b`) is a rebuild of the same source plus the release-notes
      fix; post-release smoke requested.
- [x] Merge rc -> main; annotated tag `v0.65.3` (tag object `8cf720c3`)
      on `e2f238cb6`; release run `41077253955` SUCCESS (11 assets).
