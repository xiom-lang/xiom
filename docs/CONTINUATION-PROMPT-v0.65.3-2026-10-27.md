# CONTINUATION PROMPT -- XIOM compiler lane: close 100% of lane relays before the next cut (2026-10-27)

You are the XIOM compiler lane. Owner directive: **fix 100% of all
compiler-addressed relay bugs and perf items, and verify them, before the
next version is cut.** No v0.65.3 tag until the closure checklist is green
and the owner gives GO -- unless the owner explicitly orders an emergency
durable hotfix.

Read first (in this order):
- `docs/SESSION-HANDOFF.md` top block (2026-10-27T21:50Z).
- `docs/COMPILER_BUGS.md` -- v0.65.2 entries + the 2026-10-27 hotfix entry.
- `docs/RELEASE_GATE_v0.65.2.md` (post-release section).
- `docs/NEXT-CANDIDATE-v0.65.2.md` (retention/ownership plan).
- This prompt, then the relay queue (commands at the end).

## Current state (verify with git/gh)

- Compiler repo `E:\xiom-lang\xiom`; main **`bb8eccf5a`** = v0.65.3 hotfix
  (container-moved aggregates + Str-drop pointer capture). CI
  **`40587096152` SUCCESS** (Windows, Ubuntu, release parity x2, arm64 floor,
  MCU, hygiene).
- **v0.65.2 is live**: annotated tag `fe1994b10305471d2cf4378508393980410af9a5`
  on `8ec0904a5`; release **`40566604850`** with 11 assets; docs dispatch to
  `xiom-lang/website` success; registry `xiom-std 0.65.2` sha256
  `9638b576...`. `STDLIB_VERSION=stdlib-v0.65.2`.
- **v0.65.2 is NOT accepted by xvector/orbitdb** (named `Struct{Vec}` push
  freed the Vec payload; WAL recovery corruption + station double-free).
  The hotfix on main fixes it; **v0.65.1 remains the accepted pin** for
  xvector/orbitdb. Treat v0.65.2 as upstream-only until a fixed archive is
  accepted.
- **v0.65.3 is not version-bumped/tagged.** The hotfix code is on main; the
  version is still 0.65.2. Do not tag a new version until the closure
  checklist is green.
- Local proof for the hotfix: xvector XVC-C-23 repro PASS
  (`tests/regression/xv_vec_struct_push`), `str_if_arm_drop` PASS,
  **14/14** memory gates on debug+release, full e2e **2651/2651 (4 ignored)
  on both**.
- t1 correction: the benchmark confirms t1 uses `Vec[Int]`
  (POOL_SIZE 1,048,576; MIN_BLOCK 64), so the packed-slot fix is N/A. **t1
  remains 30 MB** and `REL-20261026-2143-compiler` stays acked. Packed slots
  are a byte-buffer fix only.
- Selfhost parallel worktree `E:\xiom-lang\xiom\.kilo\worktrees\selfhost-phase-4-codegen`
  (branch `selfhost-phase-4-codegen`): T6 **19/64**, diff_ast 83/83, T3
  green; commits `c035efc8e`, `98fc4494d`, `7b816803f`. Next blocker:
  call-scrutinee payload typing (`phase1_error.xi`), then
  `Vec.first`/`last`/`remove`, concrete `Option__T`/`Result__T`. Never edit
  main from that worktree; never push it.

## Environment / repro

```powershell
$env:PATH='C:\Users\lefte\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin;'+$env:PATH
$env:PATH='E:\xiom-projects\xiom-engine\.toolchain\llvm-21.1.8\bin;'+$env:PATH
$env:XIOM_CLANG='E:\xiom-projects\xiom-engine\.toolchain\llvm-21.1.8\bin\clang.exe'
$env:XIOM_CLANGC=$env:XIOM_CLANG
$env:XIOM_STDLIB='E:\xiom-lang\xiom\stdlib'   # pinned stdlib-v0.65.2
```

- Build: `cargo build --release -p xiom` and `cargo build -p xiom`.
- Gates: `powershell -NoProfile -ExecutionPolicy Bypass -File tests\soak\run_memory_gates.ps1 -Compiler target\release\xiom.exe` (and `target\debug\xiom.exe`). Current 14 gates.
- Full e2e: `C:\Users\lefte\AppData\Local\Temp\kilo\run_e2e_both_fix.ps1` (debug+release; if the temp script is gone, run `cargo test -p xiom-codegen --test e2e_tests` twice with `XIOM_TEST_COMPILER` set to the built debug/release `xiom.exe`; `XIOM_REQUIRE_STDLIB=1`).
- CI: `gh workflow run ci.yml --ref main`; dry-run: `gh workflow run release.yml --ref main -f version=<version>`; tag builds run `release.yml` on the tag.
- Relay: `E:\xiom-lang\xiom-relays`, `python tools\relay.py ...`.

Lane repros:
- xvector: `E:\xiom-projects\xiom-xvector\docs\repro\xv-vec-struct-push\probe.xi`;
  `tests\probes\probe_wal_file.xi`; `station\tests\loopback_suite.xi`;
  `tests\hybrid\probe_hybrid.xi`.
- orbitdb: `E:\xiom-projects\xiom-orbitdb` station/hybrid probes;
  `probe_hybrid` with `ORBITDB_HYBRID_ROWS`/`ORBITDB_HYBRID_VEC`.
- engine: `E:\xiom-projects\xiom-engine`: `moveprobe.xi` (2034),
  `tests/diagnostics/probe_ui2_font_vk.xi`, `tests/diagnostics/probe_ui2_gpu_text_live.xi`,
  `build/xiom-editor.exe.ll` (1931), `scripts\build.ps1`.
- pulse: `E:\xiom-projects\xiom-pulse`: `tests\probes\probe_req_cycle.xi`,
  CI arm64; `REL-20261027-2041-pulse` is a packages/FFI-width blocker.
- benchmark: `REL-20261027-2022-benchmark` (t1/t3; `--sanitize` present).
- New compiler regressions already in-tree:
  `tests\regression\xv_vec_struct_push\main.xi`,
  `tests\regression\str_if_arm_drop\main.xi`,
  `tests\soak\borrow_vec_leak.xi`, `aggregate_vec_leak.xi`,
  `aggregate_return_leak.xi`, `str_loop_leak.xi`,
  `vec_uint8_packed_slots.xi`.

## The closure checklist (first action)

1. `cd E:\xiom-lang\xiom-relays`; pull; list the lane:
   ```powershell
   python tools\relay.py view --lane compiler --status open
   python tools\relay.py view --lane compiler --status acked
   python tools\relay.py view --lane compiler --status fixed
   ```
   Current snapshot: **17 open / 103 acked / 60 fixed**. Most acked/fixed
   are historical verification; classify every row.
2. Create `docs/COMPILER-LANE-CLOSURE.md` with one row per open/acked item:
   id, title, class (`bug`, `perf`, `info/verify`, `historical`), owner,
   repro, fix commit, verification evidence, status (`todo`, `fixed`,
   `verified`, `closed`). This file is the release gate for "100%".
3. For each `bug`/`perf` row: reproduce, fix, add a probe/regression, run the
   gates + e2e, and get the reporting lane to verify on the candidate
   archive. Do not mark fixed without both compiler evidence and a lane
   result where the lane owns the repro.
4. For `info/verify` and historical rows: verify/close with evidence (or
   record why they are already fixed); the goal is zero unexplained
   open/acked rows.

## Known actionable set (verify against the live queue)

### P0 -- durable-lane hotfix verification
- `REL-20261027-1956-xvector` (fixed on `bb8eccf5a`): named `Struct{Vec}`
  push; xvector XVC-C-23. The hotfix disarms the caller aggregate drop for
  `push`/`insert`/`set`/`add`/`append`/`put`. The **container element drop**
  for `Vec[Struct]`/Map is still missing (leak, no UAF) -- fix it next.
- `REL-20261027-2033-orbitdb` / `2042-orbitdb` (fixed on main): station
  `wire_header_decode` double-free and XVECTOR WAL reopen empty; re-run the
  station loopback and hybrid probes.
- `REL-20261027-2000-engine` (fixed): Str-drop if-arm dominance.

### P1 -- editor/UI ownership
- `REL-20261027-2034-engine` (acked): callee moving a Vec field out of a
  by-value aggregate param leaks 0.37 MB/iter; the editor 600-frame soak is
  still ~9 MB/frame (killed at 2 GB). Implement callee-side parameter/field
  ownership or make such params borrow-only, then run the engine editor soak
  and the moveprobe.
- `REL-20261027-1931-engine` (open, optional): XgView/Vec alloca type
  confusion in the reverted buffer-reuse WIP. The IR slice is in the item;
  fix the alloca retyping/reuse, but it no longer gates the editor.
- Container element drops: `Vec[Struct]`, Map values, nested aggregate
  fields; required for the durable WAL memory and JSON/Map retention.

### P1 -- retention
- OrbitDB m375 write-path/COW/teardown: `REL-20261020-1807-orbitdb-2`,
  `2102-orbitdb`, `0423-orbitdb`, `0606-orbitdb`, `1100-orbitdb`
  (895 MB/200k, 607 MB/151s, 1M 2,494.7 MB + 880 s teardown).
- PULSE: `REL-20261010-1549-pulse-4` (request path); stdlib
  `REL-20261027-1955-pulse-2` (`server_parse_request` ~8 KB/req) is the
  remaining stdlib site.
- JSON enum-tree drop (1704: ~2.3 KB/iter), Map/Set, HTTP builders, unbound
  Str temporaries.
- **t1 30 MB**: `Vec[Int]` (not the packed slot). Separate allocator/runtime
  investigation; item `2143` stays acked.

### P1 -- correctness carry
- macOS/arm64 typed memset: `REL-20261010-1549-pulse`,
  `REL-20261010-1609-pulse` (the arm64 floor fix may cover; verify on the
  candidate archive).
- Playground: `REL-20261010-1553-playground-2` (by-ref fn pointer),
  `1554-playground-2` (Int+Str), `1554-playground-3` (W005 `.to_str()`).
- `REL-20261023-1854-pulse` (catalog-body tuple `Vec[Str]`).
- Packages: `2303` (enum variant), `2309` (mixed tuples), `2002`
  (Float64 dlsym), `2016` (Result behind `&Result`);
  `REL-20261014-1208-bindings-2`; `REL-20261016-2031-bindings-3` (B-33).
- Engine: `1046` (E001 moves), `1509` (port.args), `2045`
  (`--incremental` + `--c-source`), `2134` (7 blockers).

### P2 -- perf / strategic
- t3 trampoline `REL-20261010-1554-benchmark` (~5.3-5.8x C; the host-load
  timing caveat; clean idle re-measure owed).
- SIMD route `REL-20261012-1945-xvector-3`; FFI hardening/TLS
  `REL-20261015-2202-pulse-2`.
- `REL-20261016-1917-packages` (linux-arm64) shipped since v0.64.7; verify
  and close.
- C08 Vulkan bring-up (`REL-20261021-1652-engine-2`) remains acked pending
  a no-capture repro. C09 is fixed (`1652-engine-3`/`1725` should be
  verified/closed by the engine).
- `REL-20261027-2041-pulse`: `xiom.http 0.1.8` libc FFI widths block arm64
  -- a packages fix, not compiler. The arm64 compiler defects are clear.

### Ops / hygiene
- v0.65.2 is upstream-only; xvector/orbitdb stay on v0.65.1 until a fixed
  archive is accepted (`REL-20261027-2010-xvector`, all-item
  `REL-20261027-2128-compiler`).
- Close/verify historical info rows (`1616/1617/2113/2052/2034-playground`,
  `1732-benchmark`, `2039-playground`, old stdlib notes, bindings
  corrections) with evidence.
- 60 `fixed` rows await the reporters' verification; 103 `acked` include
  historical notes. Triage, do not mass-close unexplained rows.

## Verification protocol (before any tag)

1. Repro-first for each fix; add a regression probe under `tests/soak` or
   `tests/regression`.
2. 14/14 memory gates on debug and release.
3. Full e2e debug+release (baseline 2651/2651, 4 ignored; it grows with new
   tests).
4. Fleet repros on the candidate archive: xvector `probe_wal_file` and
   station loopback; orbitdb hybrid/station; engine editor build + 600-frame
   soak + moveprobe; pulse request cycle + arm64 CI; benchmark t1/t3 and
   sanitized path; playground; packages.
5. CI full matrix (arm64 floor, MCU) and all-platform dry-run on the exact
   candidate commit.
6. Regenerate `release-checks/compiler-compiler-<tag>.md` at the cut and
   fill reporter verdicts.
7. Only then bump the version, regenerate notes (stdlib fragment from the
   pinned tag; for a compiler-only hotfix use a temporary stdlib fragment if
   needed), tag, release, docs dispatch, announcements.
8. **Owner GO is required for the tag.** No v0.64.9. No publish/deploy of a
   red archive.

## Relay rules

- Roles: `fix` is for the addressee (`to`), `verify`/`close` for the
  reporter (`from`). The compiler is the addressee for lane bugs; it can
  `ack`/`fix`, then the reporter verifies/closes.
- `relay.py fix` takes `--note` and `--ref` (singular); `verify` takes
  `--note` (put refs in the note). Include commit/run/tag evidence.
- Every mutation is append-only; commit/push with
  `relay.py commit --lane compiler --message ... <ids>`.
- Never edit another lane's item body; add a new item or a history note on
  your own item. Do not mark another lane's item fixed.

## Hard stops

- **Do not cut v0.65.3 until the closure checklist is green and the owner
  gives GO** (unless the owner explicitly orders an emergency durable
  hotfix; then cut from `bb8eccf5a` after CI/dry-run and owner GO).
- v0.65.2 is live but not accepted for durable/vector lanes; v0.65.1 is the
  accepted pin. Do not advertise v0.65.2 for those lanes.
- Do not edit the stdlib directly; changes go through the stdlib lane/relay.
  Do not make the `vec_slot_size`/stdlib packed-slot change one-sided.
- Do not push the engine/xvector/orbitdb/pulse repos; they are other lanes.
- Circuit breaker: max 3 attempts per issue; log to
  `docs/failed_attempts.md` and escalate.
- Documentation coupling: update `COMPILER_BUGS.md`,
  `SESSION-HANDOFF.md`, and the closure checklist with code changes in the
  same commit.
- ASCII guard before every commit; one logical commit per fix.
- Selfhost worktree is parallel and separate; do not let it edit main.

## Immediate first actions

1. Pull the relay and build `docs/COMPILER-LANE-CLOSURE.md`; classify all
   17 open + 103 acked rows.
2. Verify the hotfix against the **actual** lane probes (xvector WAL/station,
   orbitdb hybrid/station) with the current main compiler; record the
   results; ask the lanes for archive-level verification once the candidate
   exists.
3. Fix engine 2034 (by-value aggregate parameter field move) and the
   container element drops; run the editor 600-frame soak and moveprobe.
4. Fix the OrbitDB m375 memory/teardown rows and the request/JSON retention.
5. Work the correctness carry in priority order, then perf.
6. Only after the closure checklist is fully green: version bump, notes,
   gates, CI, dry-run, fleet verification, owner GO, tag.

Start by reading the handoff and the relay queue; do not trust this prompt's
counts blindly -- verify them with the live relay.
