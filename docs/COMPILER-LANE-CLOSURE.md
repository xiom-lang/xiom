# Compiler lane closure checklist -- 100% before the next cut

Owner directive: **fix 100% of all compiler-addressed relay bugs and perf
items, and verify them, before the next version.** v0.65.3 is withheld until
this checklist has zero unexplained `todo` rows and the owner gives GO
(unless an explicit emergency durable hotfix is ordered).

Updated **2026-10-27T23:43Z** from the live bus (`xiom-relays` @ `00e677b`).
Verified counts, not a stale snapshot:

| Queue (addressed `to: compiler`) | Count |
|---|---|
| open | **16** |
| acked | **102** (42 bug, 10 wishlist, 50 info) |
| fixed | **62** (awaiting reporter verify) |
| verified / closed / wontfix | 23 / 108 / 1 |

The lane view lists **28 open** rows in total: the 16 above plus 12
compiler->all broadcasts of its own (section E). Since the 22:52Z snapshot
`2034-playground` was closed by its reporter, `1854-pulse` moved to acked
(reduced probes green), and `2034-engine` and `2303-packages` moved to
fixed; `REL-20261028-0052-compiler` was filed from the 2309 reduction.

```powershell
cd E:\xiom-lang\xiom-relays
python tools\relay.py view --lane compiler --status open
python tools\relay.py view --lane compiler --status acked
python tools\relay.py view --lane compiler --status fixed
```

Status values: `todo` (repro/fix owed), `wip`, `fixed` (compiler evidence
in-tree, reporter/lane verify owed), `verify-on-archive` (fix believed
landed; needs a candidate-archive run), `reporter-close` (the reporter
already posted a later green verdict; the original row needs the reporter's
`verify`/`close`), `info` (historical/no compiler action), `verified`,
`closed`.

Only the addressee (`to`) may `fix`; only the reporter (`from`) may
`verify`/`close`. The compiler therefore cannot close the other lanes'
rows -- the closure gate counts a row as done when it is `verified`/`closed`
or `fixed` with a probe and the lane's verdict on the candidate archive.

## A. Open rows addressed to compiler (16)

| Item | Class | Action | Status |
|---|---|---|---|
| REL-20261019-2057-xvector | info/historical | verify/close (v0.64.8 re-pin evidence; v0.65.1 acceptance supersedes) | reporter-close |
| REL-20261020-1807-orbitdb-2 | bug/memory | m375 write-path/COW 895 MB/200k + 153 s teardown | todo |
| REL-20261020-2102-orbitdb | bug/memory | btree 18 MB / WAL 203 MB / soak 895 MB; page encode + Vec drop | todo |
| REL-20261021-0423-orbitdb | bug/memory | 607 MB / 151 s at 200k; throughput already fixed | todo |
| REL-20261021-0606-orbitdb | bug/memory | 1M cap-killed 4,173 MB at ~435k keys; superlinear | todo |
| REL-20261021-1100-orbitdb | bug/teardown | 1M PASS 2,494.7 MB; remaining 880 s exit teardown | todo |
| REL-20261022-1616-xvector | info | C-21 16 GB build green x2; verify/close | reporter-close |
| REL-20261022-1617-xvector | info | release-check row carried; close | reporter-close |
| REL-20261022-1639-packages | info/C09 evidence | C09 fixed; verify/close on the archive | reporter-close |
| REL-20261022-1725-engine | bug/info | C09 fixed; C07 capture still red; resolve/close C09, track C07 | todo (C07) |
| REL-20261022-1732-benchmark | info/historical | v0.64.9 hold; close (v0.65.x superseded) | reporter-close |
| REL-20261022-2039-playground | hygiene | stale release-check copy; close | reporter-close |
| REL-20261026-2052-benchmark | info | v0.65.1 absorbed; close | reporter-close |
| REL-20261026-2113-xvector | info | v0.65.1 acceptance GREEN; close | reporter-close |
| REL-20261027-1931-engine | bug/codegen | XgView/Vec alloca retyping (optional WIP; IR slice in item) | todo |
| REL-20261028-0052-compiler | bug/codegen | top-level fn name collides with libc/Winsock (`recv` -> LNK2005); mangle or diagnose reserved names, surface linker stderr | todo (new 2026-10-28) |

## B. Acked rows addressed to compiler (103)

### B1. Bugs -- 42 actionable rows

| Item | Reporter | Defect | Status |
|---|---|---|---|
| REL-20261010-1548-bindings-5 | bindings | alloc/free in a confined block spins the guard heap (m313 landed) | verify-on-archive |
| REL-20261010-1549-packages-2 | packages | `io.xi:943` `fs_read` false contract in multi-module programs | todo |
| REL-20261010-1549-pulse-2 | pulse | C-PULSE-16 `PULSE_BIND` is advisory (wildcard bind) | todo |
| REL-20261010-1549-pulse-3 | pulse | C-PULSE-12 import-alias module shadowing | todo |
| REL-20261010-1549-pulse-4 | pulse | C-PULSE-14 Linux request path ~87 KB/req (v0.64.2) | verify-on-archive (bound Str drops; request path to re-measure) |
| REL-20261010-1549-pulse | pulse | darwin typed `@llvm.memset.p0i8.i64` blocks macOS | verify-on-archive |
| REL-20261010-1549-stdlib-8 | stdlib | shared `target/release` build drifted past v0.64.2 | historical |
| REL-20261010-1549-website-7 | website | scripting-guide findings: six repros | todo |
| REL-20261010-1549-website-8 | website | error codes: exhaustiveness/W000, E001 build, `--explain` | todo |
| REL-20261010-1553-playground-2 | playground | by-reference fn-pointer params trap at runtime | reporter-close (m308) |
| REL-20261010-1554-benchmark | benchmark | t3 trampoline ~220 ms floor | reporter-close (m308; clean idle re-measure) |
| REL-20261010-1554-playground-2 | playground | silent `Int+Str` coercion (`12 + 2.to_string()` = 122) | todo |
| REL-20261010-1554-playground-3 | playground | mixed `.to_str()` prints nothing (W005 stub) | todo |
| REL-20261010-1609-pulse | pulse | macOS dry-run: arm64 dies on typed memset | verify-on-archive |
| REL-20261010-1904-bindings-2 | bindings | Float64 returns via fn-pointer casts | reporter-close (20261011-1549) |
| REL-20261011-1046-engine | engine | E001 misses call-arg/field-projection moves | reporter-close (m344) |
| REL-20261011-1055-bindings-2 | bindings | B-13 fn-typed param marshals to 0 | reporter-close (20261011-1549) |
| REL-20261011-1121-bindings-2 | bindings | B-14/B-15 `Vec[fn]` calls, `-> Unit` | reporter-close (20261011-1549) |
| REL-20261011-1411-engine | engine | `read_file_bytes` ensures violation (io.xi:947) | reporter-close (m345) |
| REL-20261012-1457-engine | engine | bare `sqrt` unresolved despite `use xiom.math` | todo |
| REL-20261013-1731-packages | packages | modern registry deps (sqlite/sdl3) fail; modules list ignored | verify-on-archive |
| REL-20261014-1208-bindings-2 | bindings | B-29/B-30/B-31 big-int bitops, cast IR, const alias | reporter-close (20261019-1151) |
| REL-20261014-1509-engine | engine | repeated port.args scalar flags drop C++17 | todo |
| REL-20261015-1358-engine | engine | Editor W1: json secondary modules, module-id collision | verify-on-archive (m341 W1.1; other W1 sub-findings) |
| REL-20261015-1538-engine | engine | Editor W2: reflect consts, import order, `&mut` fields | verify-on-archive (m341 W2.1; other W2 sub-findings) |
| REL-20261015-2309-packages-2 | packages | bare `assert` mis-binds to `xiom.core.assert` | todo |
| REL-20261015-2309-packages | packages | mixed-element tuples (`Tuple__Vec[UInt8]__Str__Int`) | reporter-close (3-field Vec/Str/Int + 4-field green `63c3d809d`; e2e lock; packages verify owed) |
| REL-20261016-2002-packages | packages | Float64 returns via dlsym'd fn pointers | reporter-close (m348) |
| REL-20261016-2004-packages | packages | imported struct `Array` collides with builtin | reporter-close (m349) |
| REL-20261016-2016-packages | packages | reading Result fields behind `&Result` crashes | reporter-close (m350) |
| REL-20261016-2134-engine | engine | engine pin round 2: 7 blockers | todo |
| REL-20261017-1123-pulse | pulse | duplicate module names resolve to the dependency (W001) | todo (confirm rule) |
| REL-20261017-1223-orbitdb-2 | orbitdb | E001 false positive on `&Vec[Int]` param | todo |
| REL-20261017-1223-orbitdb | orbitdb | transient compiler exit -1, no diagnostics, host load | todo |
| REL-20261017-1355-engine | engine | m340 dep-object cache key is consumer-specific | todo |
| REL-20261018-2301-pulse | pulse | cross-module struct type alias breaks field access | todo |
| REL-20261019-2206-orbitdb | orbitdb | A6 paged 57 GB blow-up at ~130k keys | todo (verify m375) |
| REL-20261020-1531-xvector-2 | xvector | C-21 build hang under a 16 GB Job Object | reporter-close (20261022-1616) |
| REL-20261020-1550-stdlib | stdlib | tuple-returning fn-value ABI (`EnumerateIter.map`) | reporter-close (20261024-1501) |
| REL-20261021-1652-engine-2 | engine | intermittent Vulkan bring-up AV (C08) | todo (needs no-capture repro) |
| REL-20261024-1101-pulse | pulse | arm64 fixed-arity printf fails IR verification | verify-on-archive (fix green) |
| REL-20261023-1854-pulse | pulse | tuple-returned Vec[Str] inline + cross-module unannotated Str bind | acked; reduced probes + e2e lock green on `de2854544`; pulse verify `probe_wsauth` |

### B2. Wishlists -- 10 strategic/perf rows

| Item | Reporter | Ask | Status |
|---|---|---|---|
| REL-20261010-1549-website-9 | website | verification-gates tracking line | reporter-close |
| REL-20261011-2045-engine | engine | `--incremental` does not cache `--c-source` objects | todo |
| REL-20261012-1945-xvector-3 | xvector | bless a SIMD route for distance kernels | todo |
| REL-20261016-1550-stdlib | stdlib | thread worker-loop primitives (persistent fleet, Send/Sync) | todo |
| REL-20261016-1917-packages | packages | linux-arm64 target (shipped v0.64.7) | reporter-close |
| REL-20261016-2031-bindings-3 | bindings | B-33 record-by-value fn-cast (fixed m351) | reporter-close (20261017-2350) |
| REL-20261016-2303-bindings | bindings | B-32 residual in-situ (retired v0.64.7) | reporter-close (20261018-1803) |
| REL-20261017-1647-engine | engine | linker baseline: AVX-512 SIGILL on Zen 2 | todo |
| REL-20261019-1220-bindings | bindings | MCU/bare-metal targets (shipped) | verify-on-archive |
| REL-20261021-1122-orbitdb-2 | orbitdb | m375 exit-reclaim (880 s at 1M) + cache sizing | todo |

### B3. Info -- 50 rows (historical; close by the reporter with evidence)

| Item | Reporter | Note |
|---|---|---|
| REL-20261010-1554-benchmark-4 | benchmark | scripting lane semantics + t3 measurement answers |
| REL-20261010-1559-stdlib | stdlib | candidate-tag pin protocol |
| REL-20261010-1652-stdlib | stdlib | notes-schema fix (stdlib 35efaf8) |
| REL-20261010-1701-stdlib | stdlib | tag-base notes fix (f475c9d) |
| REL-20261011-1426-xvector | xvector | batch 6/3 witnesses ready |
| REL-20261011-1432-pulse | pulse | batch-5 witness ready |
| REL-20261011-1500-benchmark | benchmark | m252 cold-lane cache confirmation |
| REL-20261011-1525-selfhost | selfhost | phase 6 progress, T3 3/64 |
| REL-20261011-1549-bindings | bindings | re-verify: B-13/1904/B-14/15 fixed |
| REL-20261011-1801-bindings | bindings | re-verified B-13/14/15 |
| REL-20261011-1904-pulse | pulse | batch-5 GREEN; SIGILL 1615 red on m271 |
| REL-20261011-2048-bindings-2 | bindings | B-19 package fix (SDL_WasInit cast) |
| REL-20261012-1342-stdlib | stdlib | v0.64.4 sweep: tuple elem read verified |
| REL-20261012-1400-bindings | bindings | v0.64.4 archive verdicts |
| REL-20261012-1457-engine-2 | engine | engine v0.64.4 wave verdicts |
| REL-20261013-1941-packages | packages | minimal negative baselines pass |
| REL-20261014-1510-engine-2 | engine | finding priority order (nudge) |
| REL-20261015-1109-bindings | bindings | cross-target feasibility inquiry |
| REL-20261015-1955-bindings | bindings | v0.64.5 re-verify (B-29 red then) |
| REL-20261015-2029-pulse-2 | pulse | consumer-path dep discovery GREEN |
| REL-20261015-2044-benchmark | benchmark | v0.64.5 acceptance green |
| REL-20261015-2202-pulse-2 | pulse | FFI hardening / ffigen TLS plan |
| REL-20261015-2231-selfhost | selfhost | 1732 reproduced+fixed verdict |
| REL-20261016-2102-bindings | bindings | ack v0.64.6 re-verify plan |
| REL-20261016-2213-benchmark | benchmark | v0.64.6 absorbed |
| REL-20261016-2330-stdlib | stdlib | stdlib-v0.64.6 cut / tag move |
| REL-20261017-1304-selfhost | selfhost | m347 mini-stdlib fixture pollution |
| REL-20261017-2350-bindings | bindings | v0.64.7: B-33 fixed; B-32/B-34/B-29 red then |
| REL-20261018-0000-bindings | bindings | ack v0.64.7 plan |
| REL-20261018-0042-benchmark | benchmark | v0.64.7 absorbed |
| REL-20261018-1432-pulse | pulse | v0.64.7 adoption GREEN both x64 |
| REL-20261018-1505-xvector | xvector | v0.64.7 batteries ALL GATES GREEN |
| REL-20261018-1540-bindings | bindings | v0.64.7 re-verify complete |
| REL-20261018-1803-bindings | bindings | B-32 RETIRED (correction) |
| REL-20261018-1915-bindings | bindings | B-29 not-a-defect; B-34 witness |
| REL-20261019-1151-bindings | bindings | ledger reconciliation (B-13/14/15/30/31) |
| REL-20261019-1209-xvector-2 | xvector | XVC-C-20 ack; repro red on pinned archive |
| REL-20261019-1801-bindings-2 | bindings | B-34 verified fixed (v0.64.8) |
| REL-20261019-1801-bindings | bindings | B-34 verified fixed; B-series zero-open |
| REL-20261019-1825-pulse | pulse | v0.64.8 adoption GREEN |
| REL-20261019-1922-benchmark | benchmark | v0.64.8 absorbed |
| REL-20261019-1953-stdlib | stdlib | v0.64.8 gate-9 re-verify |
| REL-20261019-2124-benchmark | benchmark | compiler perf matrix v0.64.8 |
| REL-20261020-1222-orbitdb | orbitdb | m375 repro: read_at 3.1 GB RSS |
| REL-20261020-1236-orbitdb | orbitdb | m375 perf: R2/R3 at 50k |
| REL-20261021-1243-xvector | xvector | L-6 workaround: JSON numeric read |
| REL-20261024-1120-benchmark | benchmark | v0.65.0 absorbed; v0.64.9 hold closed |
| REL-20261024-1501-orbitdb | orbitdb | v0.65.0 probes GREEN |
| REL-20261026-2247-stdlib | stdlib | v0.65.1 gate-9: only 1609 red |
| REL-20261027-2022-benchmark | benchmark | v0.65.2 absorbed; t1 still 30 MB |

## C. Fixed rows -- 62 awaiting reporter verification

Do not re-fix. `python tools\relay.py view --lane compiler --status fixed`
lists them with the commit refs. Every one needs the reporter's `verify`
(or `close`) on the candidate archive; the compiler can only act again if a
reporter reopens with new evidence.

## D. Verification baseline (as of 2026-10-27T22:52Z)

- Main: `92e0d07b5` (2034 drop fix + refreshed handoff/checklist) and
  `de2854544` (1854 e2e lock) on top of the v0.65.3 hotfix `bb8eccf5a`
  (container-moved aggregates, Str-drop pointer capture). CI
  **`40602249053` SUCCESS** on `79b01d676` (fix) and **`40604134197`
  SUCCESS** on `fbfd2fbfe` (code + 1854 lock; all matrix: windows, ubuntu,
  release parity x2, arm64 floor, MCU, hygiene). Later commits are
  docs-only.
- v0.65.2 release: tag `fe1994b1` on `8ec0904a5`, release `40566604850`,
  11 assets, docs dispatch, registry `xiom-std 0.65.2`. **Not accepted by
  xvector/orbitdb**; v0.65.1 is the accepted pin. v0.65.3 is not
  version-bumped or tagged.
- t1 30 MB unchanged: t1 uses `Vec[Int]` (POOL_SIZE 1,048,576); packed
  slots are N/A (`REL-20261026-2143-compiler` stays acked).
- **2034 param-field move (new this session):**
  - probe `tests/soak/byval_param_field_move_leak.xi`: pre-fix **468 MB WS**
    on `c2e0e51d9`; post-fix **6 MB at 20k iterations / 0.5 s** (would be
    ~10 GB pre-fix).
  - callee IR now frees the deep-copied field under a drop flag; the caller
    still frees the original (no double free).
  - **15/15 memory gates GREEN on debug and release** (`target\debug\xiom.exe`
    and `target\release\xiom.exe`).
  - `e2e_m368_vec_drop_reclaim` PASS on both (alias/escape controls intact).
  - Full e2e **2651/2651 (4 ignored) on debug and release** (941 s / 1090 s).
- **2303 enum family (this session):** qualified `Type.Variant(name: value)`
  no longer collapses to an anonymous struct (parser keeps the dotted path,
  so codegen no longer picks `Reverse` from field names), and Float64
  payloads in an erased i64 slot are bit-packed with `val_to_i64` instead of
  fptosi-truncated. The 16-probe `otel-findings/enum-float` bundle is green
  (15 runnable probes; the 16th is a helper module with no `main`): 7.25 /
  3.5 / 10.239 / 2.5 / Int 42 / Str hello exact, cross-module NaN/Inf exact,
  no clang layout collisions. Lock `e2e_m2303_enum_qualified_payload`.
  Packages in-situ jev/otel verify is owed. 15/15 gates both; full e2e
  **2653/2653 (4 ignored) on debug and release**.
- **2309 mixed tuples (this session):** the reported `Tuple__Vec[UInt8]__Str__Int`
  shape now compiles, links, and reads every element; a 4-field
  `(Int, Str, Int, Str)` tuple does too. New lock `e2e_m2309_mixed_tuples`.
  The reduced probe exposed a separate link defect: a top-level user
  function named `recv` collides with the Winsock `recv` pulled from
  `ws2_32.lib` (LNK2005 -> LNK1169, and the compiler hides the LNK line).
  Filed as `REL-20261028-0052-compiler`; renaming the function is the
  workaround. Packages verify on the candidate archive is owed.
- xvector durable repros with this build:
  - `docs\repro\xv-vec-struct-push\probe.xi` (XVC-C-23): **PASS**, named and
    inline forms bad=0, exit 0.
  - `tests\probes\probe_wal_file.xi` (built with the bin's pinned stdlib +
    XIOM_HOME): **0 failed checks**, exit 0, 8 MB peak. WAL persist/reopen/
    torn-tail recovery are green, which is the durable-lane hotfix gate.
  - Not yet re-run here: xvector station loopback and orbitdb hybrid (their
    repo paths/runner setups differ from the continuation prompt; the lanes
    own those runs).

## E. Compiler's own open broadcasts (12)

These are `from: compiler -> all` announcements (v0.64.7/v0.64.8/v0.64.9/
v0.65.0/v0.65.1/v0.65.2, incident/withdrawal notices, plus two process
notes). The compiler is the reporter, so it can `close` them as info with a
ref. Not part of the 17 lane-addressed rows.

## F. Definition of done

Every open/acked row is `verified`/`closed` with evidence, or `fixed` with
a compiler probe and the reporting lane's verification on the candidate
archive. No `todo` bug/perf rows. Then:

1. 15/15 memory gates on debug+release.
2. Full e2e debug+release (baseline 2653/2653, 4 ignored; grows with tests).
3. Fleet repros on the candidate archive: xvector WAL/station, orbitdb
   hybrid/station, engine editor 600-frame soak + 2034 probe, pulse request
   cycle + arm64, benchmark t1/t3 + sanitized, playground, packages.
4. CI full matrix (arm64 floor, MCU) + all-platform dry-run on the exact
   candidate.
5. Regenerate `release-checks/compiler-compiler-<tag>.md`; fill verdicts.
6. Version bump + notes (stdlib fragment from the pinned tag), tag, release,
   docs dispatch, announcements -- only after owner GO.

## G. Repro-path corrections (2026-10-27)

Paths in the continuation prompt that do NOT exist on disk: xvector
`station\tests\loopback_suite.xi`, xvector `tests\hybrid\probe_hybrid.xi`,
engine `moveprobe.xi`, engine `build\xiom-editor.exe.ll`. Present:
xvector `docs\repro\xv-vec-struct-push\probe.xi`,
`tests\probes\probe_wal_file.xi`; orbitdb `tests\hybrid\probe_hybrid.xi`;
engine `scripts\build.ps1`, `tests\diagnostics\*`. Ask the lanes to re-post
the missing repro paths before the fleet-verification gate.

## H. Next workstream: container element drops (not started)

Measured on `92e0d07b5` (2026-10-27): a loop that builds `Vec[R]` with
`R{ xs: Vec[UInt8].with_len(512000) }` and drops the container each
iteration peaks **496 MB for 1000 iterations** (~0.5 MB/iter) -- the outer
buffer is freed, each element's Vec payload is not. This is the leak behind
the engine editor 9 MB/frame and OrbitDB m375, and it needs a real ownership
tag, not a blind loop.

Do NOT rush this: the v0.65.2 durable regression was exactly a wrong
element-ownership assumption. Recorded design and hazards for the next fix:

- Shape: `Vec[Struct{Vec}]` (and Map values) owns the element's Vec payload
  after the hotfix disarms the caller aggregate drop. When the container
  local dies, `VecDrop::Vec` frees only the element buffer, so every
  element's Vec data leaks (no UAF).
- Needed: type-directed element drop glue. The binding knows its declared
  element type (`vec_elem_from_type_annotation` / `local_vec_elem`); for a
  struct element type, walk `type_meta` Vec fields and free each element's
  data pointer inside a runtime loop, depth-bounded.
- Hazards that must be proven before enabling: (1) `Vec[T].with_len(n)`
  allocates elements without initializing them -- freeing field pointers of
  uninitialized elements is a crash; only `new`/empty/array-literal and
  push-populated shapes are safe, or the allocation must be zeroed;
  (2) elements can be removed with `pop`/`remove`/`swap_remove` (ownership
  moves to the returned aggregate), so the container must not also free
  them, and `clear`/`truncate`/`resize` change the live range -- restrict
  the first version to push/insert-only containers; (3) `Vec[Str]` elements
  alias literals/borrowed strings -- only owned Str producers may be freed;
  (4) a field initialized from a call return or a global aliases storage the
  container does not own (`vec_binding_rhs_aliases` does not deep-copy call
  returns), so element ownership must be proven from local construction
  (struct literal with fresh/owned fields), not just from the push.
- Gate: a soak probe with 1000 x `Vec[Struct{Vec[UInt8] 512 KB}]` push/pop
  and a Map-value variant, plus the engine editor 600-frame soak and the
  OrbitDB m375 200k soak. Extend the drop-flag mechanism to the container
  (armed when elements are owned) so a partially filled container is safe.


