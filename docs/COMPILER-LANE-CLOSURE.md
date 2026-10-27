# Compiler lane closure checklist -- 100% before the next cut (2026-10-27)

Owner directive: **fix 100% of all compiler-addressed relay bugs and perf
items, and verify them, before the next version.** v0.65.3 is withheld until
this checklist has zero unexplained `todo` rows and the owner gives GO
(unless an explicit emergency durable hotfix is ordered).

Live queue snapshot: **17 open / 103 acked / 60 fixed** to `compiler`.
This file is the working gate; verify the counts with
`python tools\relay.py view --lane compiler --status ...` before editing.

Status values: `todo`, `wip`, `fixed`, `verified`, `closed`, `info`.

## A. Open rows (17)

| Item | Class | Action | Status |
|---|---|---|---|
| REL-20261019-2057-xvector | info/historical | verify/close (v0.64.8 re-pin evidence) | todo |
| REL-20261020-1807-orbitdb-2 | bug/memory | m375 write-path/COW 895 MB/200k + 153 s teardown | todo |
| REL-20261020-2102-orbitdb | bug/memory | btree 18 MB / WAL 203 MB / soak 895 MB; page encode + Vec drop | todo |
| REL-20261021-0423-orbitdb | bug/memory | 607 MB / 151 s at 200k; throughput already fixed | todo |
| REL-20261021-0606-orbitdb | bug/memory | 1M cap-killed 4,173 MB at ~435k keys; superlinear | todo |
| REL-20261021-1100-orbitdb | bug/teardown | 1M PASS 2,494.7 MB; remaining 880 s exit teardown | todo |
| REL-20261022-1616-xvector | info | C-21 16 GB build green x2; verify/close | todo |
| REL-20261022-1617-xvector | info | release-check row carried; close | todo |
| REL-20261022-1639-packages | info/C09 evidence | C09 fixed; verify/close on the archive | todo |
| REL-20261022-1725-engine | bug/info | C09 fixed; C07 capture still red; resolve/close C09, track C07 | todo |
| REL-20261022-1732-benchmark | info/historical | v0.64.9 hold; close (v0.65.x superseded) | todo |
| REL-20261022-2039-playground | hygiene | stale release-check copy; close | todo |
| REL-20261023-1854-pulse | bug/correctness | catalog-body tuple Vec[Str] inference | todo |
| REL-20261026-2052-benchmark | info | v0.65.1 absorbed; close | todo |
| REL-20261026-2113-xvector | info | v0.65.1 acceptance GREEN; close | todo |
| REL-20261027-1931-engine | bug/codegen | XgView/Vec alloca retyping (optional WIP; IR slice in item) | todo |
| REL-20261027-2034-playground | info | v0.65.2 absorbed; close | todo |

## B. Acked high-severity rows (actionable)

| Item | Class | Action | Status |
|---|---|---|---|
| REL-20261027-2034-engine | bug/ownership | by-value aggregate param field move leaks 0.37 MB/iter; editor 9 MB/frame (remaining UI gate) | todo |
| REL-20261010-1549-pulse | bug/arm64 | macOS typed `@llvm.memset.p0i8.i64`; verify on archive | todo |
| REL-20261010-1609-pulse | bug/arm64 | macOS dry-run memset evidence; same | todo |
| REL-20261010-1549-pulse-2 | bug/ffi | C-PULSE-16 PULSE_BIND advisory wildcard | todo |
| REL-20261012-1945-xvector-3 | perf/design | SIMD route for distance kernels | todo |
| REL-20261013-1731-packages | bug/package | modern registry deps (sqlite/sdl3) | todo |
| REL-20261014-1208-bindings-2 | bug/codegen | big-int bitops >=2^31; confined-block cast IR undef; const-alias overflow | todo |
| REL-20261015-2202-pulse-2 | perf/strategic | FFI hardening + ffigen TLS/h2 | todo |
| REL-20261015-2303-packages | bug/codegen | enum variant construct/read (Float64 denormals, qualified match, layout) | todo |
| REL-20261015-2309-packages | bug/codegen | mixed-element tuples; `Tuple__Vec[UInt8]__Str__Int`; 4-field link clash | todo |
| REL-20261016-1917-packages | info/target | linux-arm64 shipped since v0.64.7; verify/close | todo |
| REL-20261016-2002-packages | bug/codegen | Float64 returns via dlsym'd fn ptrs | todo |
| REL-20261016-2016-packages | bug/codegen | reading Result fields behind `&Result` crashes | todo |
| REL-20261016-2031-bindings-3 | bug/codegen | B-33 record-by-value fn-cast / confinement trap | todo |
| REL-20261016-2134-engine | bug/engine | 7 blockers with shipped workarounds | todo |
| REL-20261016-2303-bindings | bug/codegen | B-32 residual (check 1803 retirement) | todo |
| REL-20261016-2330-stdlib | info/historical | v0.64.6 tag move; close | todo |
| REL-20261018-1803-bindings | info | B-32 retired; close | todo |
| REL-20261018-1915-bindings | info | B-29 corrected; B-34 witness; close | todo |
| REL-20261019-2206-orbitdb | bug/memory | A6 paged 57 GB blow-up; verify if m375 fixed | todo |
| REL-20261020-1222-orbitdb | bug/memory | read_at 3.1 GB; likely fixed by m375; verify | todo |
| REL-20261020-1236-orbitdb | perf | R2/R3 update/read; verify current numbers | todo |
| REL-20261021-1652-engine-2 | bug/ffi | intermittent Vulkan bring-up AV (C08); needs no-capture repro | todo |
| REL-20261024-1101-pulse | bug/arm64 | arm64 printf; compiler fix green; fleet verify | todo |
| REL-20261024-1501-orbitdb | info | v0.65.0 probes green; close | todo |
| REL-20261026-2247-stdlib | info | gate-9; 1609 fixed; close after archive re-verify | todo |
| REL-20261010-1652-stdlib / 1701-stdlib | info/historical | notes-schema fixes; verify/close | todo |
| REL-20261011-1904-pulse | bug/codegen | SIGILL 1615; verify on current archive | todo |

Also triage the remaining acked medium/low rows (query the relay); most are
historical info/verification and should close with evidence.

## C. Already fixed, awaiting reporter verification (60)

Run `python tools\relay.py view --lane compiler --status fixed` and list
them. Do not re-fix; ask the reporters to verify/close, and close only where
the compiler is the reporter.

## D. Current verification baseline

- Hotfix on main `bb8eccf5a`: xvector XVC-C-23 PASS, `str_if_arm_drop`
  PASS, 14/14 gates, e2e 2651/2651 (4 ignored) both, CI `40587096152` green.
- v0.65.2 release: tag `fe1994b1` on `8ec0904a5`, release 40566604850,
  11 assets, docs dispatch, registry `xiom-std 0.65.2`.
- v0.65.2 is **not accepted** by xvector/orbitdb; v0.65.1 is the accepted
  pin. Do not advertise v0.65.2 for durable lanes.
- t1 30 MB is unchanged: t1 uses `Vec[Int]` (POOL_SIZE 1,048,576); packed
  slots are N/A (`REL-20261026-2143-compiler` acked).

## E. Definition of done

Every open/acked row is `verified`/`closed` with evidence, or `fixed` with
a compiler probe and the reporting lane's verification on the candidate
archive. No `todo` bug/perf rows. Then:

1. 14/14 gates on debug+release.
2. Full e2e debug+release (baseline 2651/2651, 4 ignored; grows with tests).
3. Fleet repros on the candidate archive: xvector WAL/station, orbitdb
   hybrid/station, engine editor + soak + moveprobe, pulse request/arm64,
   benchmark t1/t3 + sanitized, playground, packages.
4. CI full matrix (arm64 floor, MCU) + all-platform dry-run on the exact
   candidate.
5. Regenerate `release-checks/compiler-compiler-<tag>.md`; fill verdicts.
6. Version bump + notes (stdlib fragment from the pinned tag), tag, release,
   docs dispatch, announcements -- only after owner GO.
