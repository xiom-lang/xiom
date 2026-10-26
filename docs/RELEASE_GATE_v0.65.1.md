# Release gate -- v0.65.1

STATUS: **CANDIDATE CUT IN PROGRESS 2026-10-26** -- owner GO received;
stdlib tag `stdlib-v0.65.1` is live. Tag `v0.65.1` is **not cut yet**; CI,
the all-platform dry-run, and the final-tree e2e must be green first.

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
- [ ] Push the release commit to main; CI green.
- [ ] All-platform dry-run green (Windows/Linux x64+arm64, macOS, VSIX,
      wasm, SHA256SUMS).
- [ ] Tag `v0.65.1` on main; release workflow publishes 11 assets.
- [ ] `release-checks/compiler-compiler-v0.65.1.md` verdicts; docs dispatch;
      registry `xiom-std 0.65.1`; lane announcements; holds closed.

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

- [ ] Push the pin/notes commit to main (`<sha>`).
- [ ] CI run `<id>` SUCCESS.
- [ ] Dry-run `<id>` SUCCESS.
- [ ] Tag `v0.65.1` (annotated `<tag-object>`) on `<commit>`; guard green.
- [ ] Release run `<id>` SUCCESS; 11 assets + SHA256SUMS.
- [ ] Docs dispatch, registry `xiom-std 0.65.1`, announcements.
