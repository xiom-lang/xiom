# Release gate -- v0.65.1

STATUS: **RELEASED 2026-10-26T19:40Z** -- owner GO received; stdlib tag
`stdlib-v0.65.1` live. Tag `v0.65.1` (annotated
`c4329915f6a55e994ad7ddc05399059a85a0c796`) on `f2c01cf9c`; release run
**40388025015 SUCCESS** with 11 assets; docs dispatch success; registry
`xiom-std 0.65.1` signed (sha256 `3bb9b850...`). v0.65.0 is superseded;
v0.64.9 remains withdrawn.

## Scope

Memory ownership and UI-leak fixes on main since v0.65.0:

- m375 owned Vec reclamation at loop/match/aggregate/function exits
  (`778e01d57`), element access no longer escaping (`8d4968522`), and
  `#[owns_return]` transient drops behind unsafe FFI and fresh rebinds
  (`0eef57018`).
- `vecprobe` mode 0/1: 418/390 MB -> 5 MB at 800k.
- UI link: forced main-heap allocation uses libc `calloc`; no new runtime
  symbol (`8d4968522`).
- `xiom.fontraster` 0.1.1 `#[owns_return]` on `font_shape`,
  `font_glyph_raster`, `font_face_metrics` (packages `b196c10d`/`cb20373f`);
  engine `xiom.toml` pin `f394645` and script pins `ce8a292`.
- stdlib `stdlib-v0.65.1` (`9dc69b3`): `#[owns_return]` on
  `read_file_bytes`/`read_at`.
- New hard-RSS gate `tests/soak/ffi_owns_return_leak.xi`.

Not in this cut: the still-red correctness rows tracked in the v0.65.1
release check (by-ref fn pointer, W005 `.to_str`, Result payload index,
Int+Str policy, PULSE RSS, macOS/arm64 memset) and the t3 benchmark
perf/defer row. No v0.64.9.

## Preconditions (before the tag)

- [x] Owner GO (2026-10-26).
- [x] stdlib `stdlib-v0.65.1` tag live (`9dc69b3`); local checkout pinned.
- [x] Workspace version bump to 0.65.1 and `Cargo.lock` updated.
- [x] `STDLIB_VERSION=stdlib-v0.65.1`.
- [x] Release notes `release-notes/v0.65.1.md` converted with the pinned
      stdlib fragment (5 highlights, 0 breaking, 1 docs).
- [x] UI soak flat: `UI_SHOWCASE_ANIMATE=1`, 1200 frames, 2 GB cap, exit 0;
      peak private 1044 MB / WS 181 MB; sampled private 1037 -> 1011 MB.
- [x] Nine hard-RSS gates green on debug and release.
- [x] Full e2e + non-e2e battery re-run on the final pinned tree:
      2647/2647 (4 ignored) on both debug and release; battery green.
- [x] Release-notes `verify` passes against the pinned stdlib fragment.
- [x] Push the release commit to main; CI green (40385480182).
- [x] All-platform dry-run green on `f2c01cf9c` (40387243828): Windows/Linux
      x64+arm64, macOS x64+arm64, VSIX, wasm, SHA256SUMS.
- [x] Tag `v0.65.1` on `f2c01cf9c`; release workflow 40388025015 published
      11 assets + SHA256SUMS.
- [x] Docs dispatch (`compiler-release` to `xiom-lang/website`) success;
      registry `xiom-std 0.65.1` signed (sha256 `3bb9b850...`).
- [ ] Lane announcements and holds closed; release-check verdicts.

## Evidence

- UI soak trajectory: 1037 -> 1041 -> 1021 -> 1024 -> 1028 -> 1032 -> 1011
  MB private over 1200 frames; the old tree was killed at 2 GB / 2908 MB.
- IR: `font_shape` frees `buf`+`slots`; `ui2_draw_text`, `ui2_text_width`,
  `ui2_text_caret_x` free `run.glyphs`; `font_glyph_raster` frees `slots`;
  `ui2_atlas_entry` frees `gr.pixels`; `ui2_draw_text_wrapped` frees `lines`.
- Gates: with_len 4, fresh return 4, read_at 4, editor frame 26-27, arena
  loop 4, break/continue 4, ffi_owns_return 4, vecprobe 0/1 4 MB.
- Full e2e: 2651 tests, 2647 passed, 4 ignored, 0 failed (debug and release).
- Battery: feature 550, integration 130, stdlib 40 (execution 85/2 ignored),
  codegen 26, xiom 78, checker 214, parser 109, graph 43, API-freeze 3.
- `xiom.fontraster` 0.1.1 conformance 11/11.

## Known residual (not blocking)

- An actually-taken retry rebind leaks only the replaced buffer (one per
  retry); the final buffer transfers to the caller. Proper move/refcount
  semantics for replaced owning headers is future work.

## Tag + publish log

- [x] Push the pin/notes commit to main: `0c82babb9`, then the ASCII fix
      `f2c01cf9c` (encoding guard).
- [x] CI run **40385480182 SUCCESS** on `f2c01cf9c` (all matrix legs,
      including the encoding guard and release-parity).
- [x] Dry-run **40387243828 SUCCESS** on `f2c01cf9c`.
- [x] Tag `v0.65.1` (annotated
      `c4329915f6a55e994ad7ddc05399059a85a0c796`) on `f2c01cf9c`; guard
      green (tag == workspace 0.65.1, tag on main).
- [x] Release run **40388025015 SUCCESS**; 11 assets; SHA256SUMS file sha256
      `875650421fd231ff57e703749125d3f771dfe17244044bd69c61d5daede10c0d`;
      windows-x64 `69de1e27...` verified.
- [x] Docs dispatch success (tag `v0.65.1`, stdlib `stdlib-v0.65.1`).
- [x] Registry `xiom-std 0.65.1` signed, sha256
      `3bb9b85007d69f75a7379b90182f260e8d0bfba03f02431619481f9dae494d31`.
- [ ] Announces the archive to the engine/packages/stdlib lanes; closes the
      v0.65.1 holds; reporter verdicts in
      `release-checks/compiler-compiler-v0.65.1.md`.
