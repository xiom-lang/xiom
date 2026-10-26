# CONTINUATION PROMPT -- XIOM m375 memory ownership / next candidate (2026-10-26)

You are the XIOM compiler lane. The owner's standing rule: **do not cut/tag the next candidate until the UI memory leak is fixed and the 1200-frame soak is flat.** Production is v0.65.0; v0.64.9 stays withdrawn. Do not edit the stdlib except via relay; do not pin/deploy v0.64.9. The owner wants a permanent ownership fix, not a package patch.

Read first:
- `docs/COMPILER_BUGS.md` -- m375 entries + the 2026-10-26 follow-up.
- `docs/SESSION-HANDOFF.md` -- top memory block.
- `docs/RELEASE_GATE_v0.65.0.md` -- post-release/next-candidate section.
- `docs/SCOPED-ARENA.md` -- "Owned Vecs and `#[owns_return]`".
- `E:\xiom-projects\xiom-engine\docs\COMPILER-LEAK-REPORT.md`.
- Relay items: `REL-20261026-1158-compiler` (engine), `REL-20261026-1158-compiler-2` (packages), `REL-20261026-0040-compiler` (engine), `REL-20261026-0040-compiler-2` (stdlib), `REL-20261016-1128-engine` (fixed), `release-checks/compiler-compiler-v0.65.1.md`.

## Environment / constraints
- Compiler repo: `E:\xiom-lang\xiom`; engine repo: `E:\xiom-projects\xiom-engine`; package source: `E:\xiom-packages\packages\packages\xiom-fontraster`; relay bus: `E:\xiom-lang\xiom-relays`.
- Cargo: `C:\Users\lefte\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin`.
- Engine LLVM: `E:\xiom-projects\xiom-engine\.toolchain\llvm-21.1.8\bin\clang.exe`.
- Pinned stdlib for local tests: `E:\xiom-lang\xiom\stdlib` (the tag `stdlib-v0.65.0` plus the uncommitted `#[owns_return]` annotations on `read_at`/`read_file_bytes`; the stdlib tag `v0.65.1` is not cut).
- `STDLIB_VERSION` is still `stdlib-v0.65.0`.

## Current commits (verify with git)
- Compiler main: `2e9c503a5` docs over `8d4968522` (index-safe escapes, `#[owns_return]` transient trust, `calloc`) over `778e01d57` (m375 ownership model) over `0b70dee22`, `c03828336`, `38a199cc9`, `1fa161e12`.
- v0.65.0: tag `f527d5a` -> `a0dc452dc`; release 40041016248; 11 assets; registry `xiom-std 0.65.0`; stdlib `3eb9c05`.
- Engine: main `511cfb8` + 18, clean; `tests/diagnostics/vecprobe.xi`, `tests/probes/probe_ui2_text.xi`, `apps/ui_showcase/main.xi`, `engine/ui2/text.xi`, `scripts/run_capped.ps1`.
- Fontraster package: `0.1.0`; `src/fontraster.xi`; `font_shape` (line ~131), `font_glyph_raster` (~190), `font_face_metrics` (~86); engine `xiom.toml` pins `xiom.fontraster = "0.1.0"`.

## Verified fixed
- `vecprobe` mode 0: **418 MB -> 5 MB**; mode 1: **390 MB -> 5 MB** at 800k. Cause: `v[i]` element read/write marked the Vec local escaping. Only `&v[i]` escapes now.
- UI link: `probe_ui2_text` and full `ui_showcase` build and run with the new compiler, explicit `XIOM_STDLIB`, and engine LLVM 21. The 1120 came from the new `xiom_alloc` runtime symbol against a stale runtime path; forced main-heap now uses libc `calloc` (no new runtime symbol). Rebuild the compiler and clear the engine package/runtime cache before retesting.
- `#[owns_return]` now trusts that unsafe/FFI does not retain the function's owned transient Vecs, so `font_shape`-style `buf`/`slots` can drop.
- Eight hard-RSS gates (`tests/soak/run_memory_gates.ps1`) pass on debug and release before the latest trust commit: with_len 4-5, fresh-return 4-5, read_at 4-5, editor frame 28, arena loop 5, break/continue 4, vecprobe mode 0 4-5, mode 1 4-5 MB.
- Earlier full e2e green on `778e01d57`: **2647/2647 (4 ignored) debug and release**; non-e2e battery green (550/130/40/26/78/214/109/43/3). The later `8d4968522` index/calloc/trust changes are AFTER that full run; rerun.

## Still broken -- the release blocker
- `UI_SHOWCASE_ANIMATE=1`, 1200 frames, 2 GB cap: killed at 2 GB WS / 2908 MB private in my run (engine saw 1437 MB at 82 s).
- Remaining source: the `xiom.fontraster` package. `font_shape` returns `TextRun` with `glyphs: Vec[ShapedGlyph]`; `font_glyph_raster` returns `GlyphRaster` with `pixels: Vec[UInt8]`; `font_face_metrics` has transient `slots`. Their bodies are unsafe/FFI. They need `#[owns_return]`, package `0.1.1`, and the engine `xiom.toml` dependency `0.1.1`.
- Callers: `ui2_draw_text` does `let run = font_shape(...)`; `ui2_glyph_get` does `let r = font_glyph_raster(...)`; both need the aggregate drop. `ui2_text_wrap` returns a fresh Vec; the `lines` caller in `ui2_draw_text_wrapped` should drop.
- If the soak still grows after those annotations, debug deeper in this order:
  1. `font_glyph_raster`'s `pix` retry/rebind (`pix = Vec.with_len(need)`): the `Stmt::Assign` on a bare local marks it escaping; the old `pix` buffer can leak; the final `pix` is returned as `GlyphRaster.pixels`. Check the IR for the transient `slots` and the old `pix`.
  2. `ui2_text_wrap` (`out`/`lines`) and `ui2_text_wrap`'s callers; the `lines`/`Ui2TextLine` Vec.
  3. `ui2_draw_text`'s `run` aggregate (`run.glyphs`) and `ui2_text_width`/`ui2_text_caret_x`/`ui2_text_x` calls to `font_shape`.
  4. `ui2_glyph_get`'s `r` aggregate (`r.pixels`) and the atlas copy/drop.
  5. The live caches: `Ui2GlyphCache` (`keys`, `refs`, `atlas`), `Ui2Atlas` pages, the glyph cache/atlas growth; distinguish live cache from per-frame leak.
  6. `g_hits` (`Vec[Ui2Hit]`, clear/push capacity) and the UI text/Str paths (`ui2_text_ellipsis`), the per-frame list/labels.
  7. `font_face_metrics` `slots`, `font_shape` `buf`/`slots`, `font_glyph_raster` `slots`/`pix`.
- The permanent investigation: complete drop/ownership for struct returns and unsafe/FFI transient Vecs; `Vec[Str]` element drops and nested aggregate fields; move/refcount semantics for owning headers; region/lifetime escape analysis for arenas (the engine's whole-frame-arena crash); diagnostics for long-loop arena allocations; the `#[owns_return]` trusted contract (a wrong annotation is UAF). Lexical scope + arenas is not the flaw; missing drop/region analysis is. Rust lifetimes are reference-validity, not freeing; the leak is ownership/drop, the arena escape is the region gap.

## Exact repro commands
Engine env:
```powershell
$env:PATH='E:\xiom-projects\xiom-engine\.toolchain\llvm-21.1.8\bin;'+$env:PATH
$env:XIOM_CLANG='E:\xiom-projects\xiom-engine\.toolchain\llvm-21.1.8\bin\clang.exe'
$env:XIOM_CLANGC=$env:XIOM_CLANG
$env:XIOM_STDLIB='E:\xiom-lang\xiom\stdlib'
Set-Location E:\xiom-projects\xiom-engine
```

Vecprobe:
```powershell
& E:\xiom-lang\xiom\target\release\xiom.exe tests\diagnostics\vecprobe.xi -o $env:TEMP\vecprobe_cargo.exe --release
$env:PROBE_VEC_MODE='0'; $env:PROBE_VEC_N='800000'; & .\scripts\run_capped.ps1 $env:TEMP\vecprobe_cargo.exe 1024
$env:PROBE_VEC_MODE='1'; & .\scripts\run_capped.ps1 $env:TEMP\vecprobe_cargo.exe 1024
```

UI link + soak:
```powershell
& E:\xiom-lang\xiom\target\release\xiom.exe apps\ui_showcase\main.xi -o $env:TEMP\ui_showcase_fixed.exe --release
$env:UI_SHOWCASE_ANIMATE='1'
& .\scripts\run_capped.ps1 $env:TEMP\ui_showcase_fixed.exe 2048
```

Compiler gates:
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tests\soak\run_memory_gates.ps1 -Compiler target\debug\xiom.exe
powershell -NoProfile -ExecutionPolicy Bypass -File tests\soak\run_memory_gates.ps1 -Compiler target\release\xiom.exe
```
The gate suite includes `vecprobe.xi` modes 0/1 (`PROBE_VEC_*`) and the six earlier witnesses.

Full e2e (debug then release):
```powershell
powershell -File C:\Users\lefte\AppData\Local\Temp\kilo\run_e2e_both_fix.ps1
```
It reads `XIOM_STDLIB=E:\xiom-lang\xiom\stdlib`, runs debug e2e, builds release, runs release e2e; logs in `C:\Users\lefte\AppData\Local\Temp\kilo\e2e_*_fix.log` and `e2e_fix_summary.txt`.

## Immediate plan
1. Patch the fontraster package source `E:\xiom-packages\packages\packages\xiom-fontraster\src\fontraster.xi`: `#[owns_return]` on `font_shape`, `font_glyph_raster`, `font_face_metrics`; bump `package.xi` to `0.1.1`; update the engine `xiom.toml` to `0.1.1`; install/publish the package; rebuild the engine UI.
2. Run the 1200-frame soak. If it still grows, follow the deeper order above and record the per-frame allocation source.
3. Run the eight gates and full e2e debug+release on the final compiler; run the non-e2e battery; verify the IR drops for `font_shape`/raster (`run`/`r` aggregate fields, transient `buf`/`slots`/`pix`).
4. Only then: stdlib `v0.65.1` with `#[owns_return]` on `read_at`/`read_file_bytes` (relay `REL-20261026-0040-compiler-2`), pin `STDLIB_VERSION=stdlib-v0.65.1`, bump the compiler to the next candidate, regenerate notes, and run CI + all-platform dry-run.
5. Release mechanics after owner GO: tag, 11 assets + SHA256SUMS, docs dispatch, registry `xiom-std 0.65.1`, lane announcements, holds closed. No v0.64.9.

## Relay / still-red context
- Fixed on main and awaiting next-archive verification: arm64 `printf` (`REL-20261024-1101-pulse`), `.xiomignore` (`REL-20261014-1504-registry` + 2104), warm cache (`REL-20261022-2032-playground`).
- v0.65.0 still-red correctness/memory rows: `REL-20261010-1553-playground-2` by-ref fn pointer, `REL-20261010-1554-playground-3` W005 `.to_str`, `REL-20261010-1609-stdlib` Result payload index, `REL-20261010-1554-playground-2` Int+Str policy, `REL-20261010-1549-pulse-4` RSS, `REL-20261010-1549-pulse`/`1609-pulse` macOS/arm64 memset; t3 `REL-20261010-1554-benchmark` is perf/defer.
- The new `compiler-compiler-v0.65.1.md` release-check is generated but uncommitted; it lists 117 historical open/acked rows. The v0.65.0 filled verdicts are the actionable release input.

## Hard stop
Do not tag, publish, or deploy the next candidate until the UI 1200-frame soak is flat, the stdlib v0.65.1 tag exists and is pinned, the engine frame/text builders are annotated, the full e2e after the final tree is green, CI + the all-platform dry-run are green, and the owner gives GO. v0.65.0 remains production.
