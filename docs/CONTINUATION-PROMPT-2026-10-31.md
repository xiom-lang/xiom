# Continuation -- 2026-10-31 (after v0.65.4)

You are continuing the XIOM compiler lane in `E:\xiom-lang\xiom`. Read, in
order:

1. `docs/SESSION-HANDOFF.md` top block (2026-10-31T14:10Z).
2. `docs/COMPILER-LANE-CLOSURE.md` (counts + memory sections).
3. This file.

## 1. Released state

- **v0.65.4 is live**: annotated tag `v0.65.4` on `c0104e21e`, release run
  `41144793224` SUCCESS, 11 assets, published 2026-10-30T23:35:43Z.
- Bundled stdlib `stdlib-v0.65.4` at **407c92d** (package.xi 0.65.4,
  `net.xi` `xiom_socket_close -> Int`).
- Fixes in the cut: `34e42ad87` (aggregate drop GEPs in the entry block;
  epos), `13fc28352` (program-global drop-analysis caches; engine build),
  stdlib `407c92d` (int64 socket ABI + POSIX SIGPIPE).
- main `a5fb44f21`; only pre-existing `.gitignore` + untracked
  `m211_listdir_probe/`.
- Published archive re-verified here with its own exe + bundled lib:
  transport 42/42, epos 29/29. e2e 2668/2668 both; gates 20/20;
  CI 41139797014 (merge) and 41136384010 (rc) green.

## 2. Open engine items (both addressed `to: compiler`, status open)

### 2.1 `REL-20261030-2357-engine` -- 600-frame soak leak (2 GB gate RED)

- Build on `13fc28352` + stdlib 0.65.4 is GREEN
  (`scripts\build.ps1 -Release -NoTimeout`, `build\xiom-editor.exe`
  34,764,800 B, ~37 min parallel with showcase; no codegen stall).
- Soak `XIOM_DEMO_EDITOR_FRAMES=600`, `run_capped.ps1 -LimitMB 2048`:
  killed; peak WS 2051 / private 2540 MB. At 4096 MB: exit 0, peak WS
  3325 / private 3867. Linear ~100 MB/s WS, ~120 MB/s private.
- **Answer: `13fc28352` was not expected to change runtime reclaim**; it
  only caches compile-time ownership analysis. This is the still-open
  drop-family leak (2032 / REL-1704 class: Map/JSON/enum + per-frame
  container growth), not a build regression.
- Next: ack with that clarification, then scope the next memory slice
  (Map/Set values, JsonValue, enum payloads, removals, per-frame Vec
  growth). Do not claim the build fix addressed it.

### 2.2 `REL-20261030-2357-engine-2` -- showcase panel-hook AV (TOP)

- `build\ui_showcase.exe` (35,743,232 B) builds on v0.65.4 but every
  panel-hook P1 capture crashes `0xC0000005`; a plain SHOT frame works.
  `UI_SHOWCASE_ASSETS_SELFTEST=1` crashes; v0.65.1 showcase passes the
  same selftest.
- WER: fault module `ui_showcase.exe`, offset **0xE153BC**, same across
  runs. At that site:
  ```
  movabsq $0x4008000000000000, %rcx   ; 3.0
  movq %rcx, 0x1352289(%rip)          ; global
  cmpq $0x1f4, %rdx                   ; 500
  vmovsd (%rcx), %xmm1                ; deref 3.0 -> AV
  ```
  The Float64 `dist` (3.0) is used as the borrowed `AVec3*`. Shape:
  `proj(&AVec3, Int, Int, Float64, Float64) -> AProj`, as in
  `showcase_assets.proj` and `main.xi view_cam` (500.0).
- **Reduced probe does NOT reproduce:** `tests/probes/
  probe_aggregate_float.xi` (exact shape) passes on v0.65.4 (published
  exe + bundled lib) and v0.65.1. So the crash needs the real call
  context (inlining/release build, the actual AProj/AVec3 types, or more
  args/aggregates), not the minimal function.
- Next diagnostics (in order):
  1. Ack both items on the relay (they are open). Tell engine the probe
     passes so far and ask for the exact function/args/IR slice or the
     smallest showcase file that still crashes.
  2. Build the showcase with the released v0.65.4 exe and emit the IR of
     the `proj`/`view_cam` call (`--emit-ir` on the showcase root or its
     module) with the same `--release`/contract flags as
     `scripts/build.ps1`; compare v0.65.1 vs v0.65.4 IR for the argument
     order/registers of `proj(&AVec3, Int, Int, Float64, Float64)`.
  3. Look for an argument-evaluation/move change in the v0.65.2-v0.65.4
     line that could shift a borrowed lvalue with scalar Float64 args
     (`Expr::MutRef`/`Ref` coercion, by-value aggregate moves, aggregate
     literal field moves). The fault puts a constant double in the
     pointer slot, which suggests a mis-ordered call or a wrong arg class.
  4. Bisect the compiler at 92e0d07b5 / 63c3d809d / 6a58a60e1 /
     a13040ac0 / ab97014d9 / b135dfbe8 / e48213c66 / ed2f38cff /
     605c6a879 / aac15cc27 with the engine's showcase assets selftest
     (the engine can rebuild per commit) until it flips.
  5. Add a regression witness only when the real shape is known.

## 3. Verifying any fix

- Reproduce first, then minimal change. Witness must fail pre-fix.
- Engine: `scripts\build.ps1 -Release -NoTimeout` with
  `XIOM_ENGINE_TOOLCHAIN=<candidate exe>`; `UI_SHOWCASE_ASSETS_SELFTEST=1`
  and a P1 capture; optionally the 600-frame soak (separate leak).
- Compiler: `cargo build -p xiom` (debug), 20/20 gates, e2e debug+release,
  transport 36/36 (now 42/42 with the newer package tree), epos 29/29,
  carve 160/160 if the package store still has the 0.2.1 carve.
- Candidate release flow: build release in a private target, publish
  `target\candidate\xiom-<sha>.exe` + sha256, CI, relay INFO to all,
  lanes verify, then a **v0.65.5** hotfix tag. No bump/tag/release while
  a lane verdict is red.

## 4. Relay state (2026-10-31T14:10Z)

- Counts addressed to compiler: **32 open / 116 acked / 65 fixed / 25
  verified / 106 closed**; open broadcast `REL-20261030-2343-compiler`
  (v0.65.4 announcement).
- Ack within the work cycle; the engine items are open. The pulses/
  xvector/packages published-asset smokes are still follow-ups.
- Correction rule: mark a bug fixed only with compiler evidence; the
  reporter verifies. Close your own broadcasts as info with a run ref.

## 5. Hard stops

- Do not re-cut v0.65.4. A codegen/ABI fix is **v0.65.5** after lane
  verification.
- Do not tag with a red lane verdict, and do not change STDLIB_VERSION
  without the corresponding live stdlib tag.
- Keep docs (handoff + this file + closure) in the same commit as the
  code fix.
