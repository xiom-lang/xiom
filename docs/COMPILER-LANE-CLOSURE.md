# Compiler lane closure checklist -- 100% before the next cut

Owner directive: **fix 100% of all compiler-addressed relay bugs and perf
items, and verify them, before the next version.** v0.65.3 is withheld until
this checklist has zero unexplained `todo` rows and the owner gives GO
(unless an explicit emergency durable hotfix is ordered).

Updated **2026-10-31T14:50Z** from the live bus after the v0.65.4 release.
Verified counts, not a stale snapshot:

| Queue (addressed `to: compiler`) | Count |
|---|---|
| open | **29** |
| acked | **111** |
| fixed | **57** (awaiting reporter verify) |
| verified / closed / wontfix | 24 / 130 / 1 |

The lane view lists **33 open** rows: the 32 above plus the one open
compiler->all broadcast (`REL-20261030-2343-compiler`, the v0.65.4
release announcement). **v0.65.4 is live** (tag `c0104e21e`, run
`41144793224`, stdlib 0.65.4 at 407c92d). The four release blockers
(1324/1641/1910/2110) remain verified/closed. New post-release engine
items are open: `2357-engine` (600-frame soak leak; build fixed, runtime
reclaim unchanged) and `2357-engine-2` (showcase panel-hook AV; the
reduced probe passes on v0.65.4, so the real call/IR is the next step).

### Triage 2026-10-31T14:50Z -- "all lane bugs" split into four buckets

The owner directive is 100% closure, but the backlog is not one kind of
work. Each open/acked bug belongs to exactly one bucket:

**A. v0.65.5 code-fix blockers (repro + fix + witness owed)**
- `REL-20261030-2357-engine-2` -- **FIXED in `2b2b51b54`, awaiting engine
  verify.** Root cause was not the proj/float ABI: `showcase_advanced.g_clip:
  Int` and `engine.editor.p1_data.g_clip: AnimClip` were both injected as
  bare `@g_clip`, codegen deduped the AnimClip definition, and
  `p1_notify_init` loaded the Int global (i64 1) and `inttoptr`'d
  duration 3.0 as `%struct.AnimClip*`. Fix qualifies mutable globals in
  the catalog injector, emits one symbol per module, keeps a bare leaf
  alias, and resolves globals with a module-scoped helper. Witness:
  `tests/regression/global_name_collision` (pre-fix `b=0.00`, post-fix
  `b=0.50`); 17/17 global e2e tests, 214/214 xiom-check tests; full
  pre-workaround showcase rebuild: `UI_SHOWCASE_ASSETS_SELFTEST=1` PASS,
  full selftest reaches source assertions without the AV.
- `REL-20261029-1415-packages` + `REL-20261029-1926-packages`:
  `Vec[Vec[UInt8]]` corruption. **Resolved on the published v0.65.4**:
  the minimal repro `sim_vecvec_corruption.xi` is green **10/10** with
  `r1[0]=k1`, `r2=[k2,k2,k0]`, `k0=1 k2=2` (the v0.65.1 expectation).
  Packages asked to verify b4 + UDP reassembly on the archive
  (`REL-20261031-1510-compiler`); re-open only with a red v0.65.4 run.
- `REL-20261027-1931-engine`: XgView/Vec alloca/slot type confusion; the
  optional editor WIP is not shipping, but the IR slice is a real codegen
  defect and may share a root with 2357-2.

**B. Drop-glue / reclaim slice (one memory project, not a one-line fix)**
- `REL-20261030-2357-engine` (600-frame soak) is the acceptance gate.
- orbitdb `1807/2102/0423/0606/1100` (m375 write-path/page/Vec-drop and
  teardown) are the same family at a different workload.
- compiler conservative leaks still open: Map/Set values, enum payloads,
  Str fields of elements, containers with removals/reorders, `Vec[Str]`
  (sections H/I below). `13fc28352` only caches compile-time analysis;
  no runtime-reclaim claim is attached to it.

**C. Verification-only (no new compiler code)**
- packages `1446/1523/1729`: superseded by the v0.65.4 publish; reporter
  verify/close requested in `REL-20261031-1448-compiler`.
- The 65 `fixed` rows: reporters verify on the published v0.65.4 archive.
- Older acked v0.64.x bugs: re-test on v0.65.4 before re-opening; do not
  re-fix from stale evidence.

**D. Not compiler-action**
- release/CI transients, lane-side workarounds (e.g. engine scalarization),
  and historical info rows. Record the disposition; do not count them as
  open code work.

Gate rule: `fixed` requires a witness that fails before and passes after,
plus the relevant gates/e2e. The reporter/lane verifies; no tag while a
lane verdict is red.

### 2026-10-31T18:55Z -- `2b2b51b54` fixes the 2357-2 showcase AV (unreleased)

The 1:26 `ui_showcase_0654.exe` crash is a **cross-module mutable global
collision**, reproduced and fixed on main:

- `showcase_advanced.xi:165` declares `var g_clip: Int = 1`;
  `p1_data.xi:2339` declares `var g_clip: AnimClip = anim_clip_new(...)`.
- The external-decl injector qualified only immutable consts
  (`if !cd.is_mut && ...`), so both `var g_clip`s injected as bare
  `g_clip`; codegen deduped the second definition and the aggregate
  module read the scalar's i64 storage. The 3.0 in the WER fault is
  `g_clip.duration`, not `proj`'s dist.
- Fix `2b2b51b54`: qualify mutable globals too, emit one LLVM symbol per
  module, keep a bare leaf alias for legacy lookups, and prefer the
  module-qualified key in codegen. New witness
  `tests/regression/global_name_collision` fails pre-fix (`b=0.00`) and
  passes post-fix (`b=0.50`).
- Verification: 17/17 global e2e tests (m227/m337/m361 included), 214/214
  `xiom-check` tests, and the full pre-workaround showcase rebuild:
  both `@showcase_advanced.g_clip = i64` and
  `@engine.editor.p1_data.g_clip = %struct.AnimClip` are emitted;
  `UI_SHOWCASE_ASSETS_SELFTEST=1` PASS; full selftest runs to the
  pre-workaround source assertions (P1 `s23`/`data`/`gizmo` FAIL) with no
  0xC0000005.

No tag yet. The next gate is engine verification on a v0.65.5 candidate.

### Cross-lane triage 2026-10-31T19:11Z (relay `745cf91`)

Read-only cross-lane pass (notes at
`C:\Users\lefte\AppData\Local\Temp\kilo\cross-agent\triage-2026-10-31.md`):

| Item | Status | Action |
|---|---|---|
| `2142-packages` ffmpeg demux | Resolved by `0.4.1` (`50f6d46b`, run 38577890832; 23/23 x2) | bindings marks fixed; packages yanks `0.4.0` (`910946e2`, eco-v0.1.177) and verifies |
| `2329-packages` imgui pack | Resolved by exception `fb55b372`; publish `a60d80a5` run 41111907416 | bindings marks fixed; packages verifies; `1740` is a duplicate |
| `0900-playground` relay BOM | Resolved (`d76bb39`; validate 1129 items) | playground/orbitdb close; no action |
| `1721-ops` pulse 0.2.0 SIGILL | Resolved by m297 + pulse 0.2.1 (run 38808850409) | pulse marks fixed; ops verifies |
| `2143-compiler` t1 30 MB | Misdiagnosed: fixture is `Vec[Int]`/`with_capacity(1M)`, no `Vec[UInt8]`; v0.65.4 probe ~34.3 MB | benchmark re-measures and re-scopes/closes; no compiler change |
| `1945-xvector-3` SIMD wishlist | No XIOM vector codegen in v0.65.x; owner/product route | deferred; xvector keeps acked or closes |
| `1415/1926-packages` Vec[Vec] | Verified/closed on v0.65.4 (42/42 x2, b4, UDP) | none |

The `2357-engine` soak and orbitdb m375 memory family are **not** in this
table: a dedicated leak/drop-glue agent is running them with a before/after
slope requirement, separate from the stdlib JSON item.

### 2026-10-31T19:24Z -- 1600/1619 fixed; 1552/1704 root-caused

- **1600 (Float128/Int128):** compiler half fixed in `61f9e3b6a`.
  Added `double/float -> i128` and `i128 -> double/float` cast arms plus
  the `coerce_value` width rule; the invalid `store i128 %double` is gone.
  Regression `m1600_float_to_int128_cast_emits_conversion` locks the IR.
  The e2e link still needs `__fixdfti`/`__fixunsdfti`/`__fixsfti`/
  `__fixunssfti` in `stdlib/runtime/fp128_helpers.c`; stdlib asked
  (`REL-20261031-1924-compiler`). Item stays acked until the archive
  link probe passes.
- **1619 (arm64 Int64 extern):** fixed in `61f9e3b6a`. Root cause was
  duplicate extern precedence (emitter first-wins vs `types.functions`
  last-wins), not an arm64 lowering rule. Externs now register
  first-wins; the real `xiom.http` shape emits
  `declare i32 @fseek(i8*, i64, i32)` and a matching `i64` call.
  Regression `m1619_duplicate_extern_first_declaration_wins`; pulse asked
  to re-run the arm gate.
- **1552 (by-value fn as `fn(&T)`):** reproduced; root cause is checker
  ref erasure in `CheckedType::from_ast_type` plus the wildcard
  `Named("fn")` compatibility. Proposed fix: preserve `&`/`&mut` in
  fn-pointer signatures, type bare fns with their real signature, and
  allow `&expr` only for `&T` fn params (reject the coercion).
  **Fixed in `216a70322`** (see the 19:42 section).
- **1704 (JSON retention):** confirmed compiler drop glue, not the
  stdlib API: recursive enum/aggregate payloads are never dropped and a
  call-returned fresh `Str` is not a drop candidate. The leak agent owns
  the before/after slope fix; optional `json_stringify_into` is only a
  mitigation.

### 2026-10-31T19:42Z -- t1 alloc fix; partial m375 leak fix

- **Benchmark t1 (`2143-compiler`):** fixed in `f644fd54b`. The +20,480 KB
  was not methodology: `value.alloc()`/`value.free()` on a user struct
  were hijacked by the allocation builtins, so the buddy allocator was
  bypassed. Both builtins now skip a value-instance receiver; regression
  `m2143_user_alloc_method_is_not_hijacked`. Benchmark must re-measure on
  v0.65.5; expected ~9.98 MB, not <5 MB. The earlier "no compiler fix"
  note is corrected by `REL-20261031-1933-compiler`.
- **Leak (`2357-engine`) partial:** `e875528ce` reclaims by-value
  `Vec[T]` parameter copies (`var b = bytes`). Measured `p_vecrebind`
  3000x64 KB: 403 MB -> **101 MB WS / 130 MB private**, same result.
  Targeted drop e2e (m368/m375/container/str/nested) pass; new lock
  `m375_by_value_vec_param_rebind_is_dropped`.
- **Leak still open:** `p_panel_move` (~190 KB/iter), `p_structstr`/
  `p_elemstr`/`p_vecstr` (~20 KB/iter), `p_mapstr` (~40 KB/iter),
  `p_remove` (~16 KB/iter). The 600-frame editor rebuild/soak has not
  been rerun; acceptance remains <2 GB and <=0.5 MB/frame. No tag.
- **1552 fixed in `216a70322`:** checker now preserves `&`/`&mut` in
  fn-pointer signatures, types bare fns with their real `Fn` value type,
  and compares Fn/Fn params strictly. `p1552.xi` now fails the let with
  `annotated fn(&Tuple) -> Bool, found fn(Tuple) -> Bool`; valid by-ref
  and by-value fn values still compile/run. 214/214 `xiom-check` tests
  and the catalog corpus pass; regressions
  `m1552_by_value_fn_is_not_accepted_as_by_ref` /
  `m1552_by_ref_fn_value_still_compiles`. Relay item marked fixed.

### 2026-10-31T21:15Z -- 1423 fixed; 1931 gated on the editor .ll

- **1423-engine fixed** (compiler half). Symptom B was the `g_clip`
  mutable-global collision (`p1_data.xi:2339` vs
  `showcase_advanced.xi:165`), fixed by `2b2b51b54`. On the unmodified
  `faa5a41` full link with the fixed compiler, `assets_selftest`
  (`bad_asset.err.code`), `world_selftest` (`bad_place.err.code`), and
  S76/S77 (`g_clip`/`g_montage` field writes) pass; the aggregate probe
  passes. The residual `FAIL p1 s23 index` is engine **source arithmetic**
  (`p1_layer_add` names by count; the worldsettings prelude leaves
  `"Layer 6"`), fixed by the uniqueness loop already in `1461c07`.
  `st p1 data`/`gizmo` are the same `ok` bool. Engine asked to verify and
  close; a large-link battery can remain separate.
- **1931-engine still open, distinct class:** `@g_ed_iv` is unique and
  the bad value comes through an SSA pointer, not a bare-name collision.
  The assignment uses the destination slot type (`stmt.rs:1514/1523`
  local, `:1532/1539` global) after `coerce_value`'s no-cast fallback
  (`coerce.rs:972-973`). The editor cannot be rebuilt on this host
  (allocation failure); engine asked for the original ~44 MB
  `xiom-editor.exe.ll` function body around line 65929 or a reduced
  two-module `load X; store as Y` repro (`REL-20261031-2115-compiler`).
  No fix claimed.

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

## A. Open rows addressed to compiler (15)

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

## B. Acked rows addressed to compiler (107)

### B1. Bugs -- 45 actionable rows

| Item | Reporter | Defect | Status |
|---|---|---|---|
| REL-20261010-1548-bindings-5 | bindings | alloc/free in a confined block spins the guard heap (m313 landed) | verify-on-archive |
| REL-20261010-1549-packages-2 | packages | `io.xi:943` `fs_read` false contract in multi-module programs | no-repro (m301: 73 runs/7 shapes); reporter close or provide repro |
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
| REL-20261012-1457-engine | engine | bare `sqrt` unresolved despite `use xiom.math` | verify-on-archive (reduced probe `sqrt(4.0)`=2 green 2026-10-28; engine repro to re-run) |
| REL-20261013-1731-packages | packages | modern registry deps (sqlite/sdl3) fail; modules list ignored | verify-on-archive |
| REL-20261014-1208-bindings-2 | bindings | B-29/B-30/B-31 big-int bitops, cast IR, const alias | reporter-close (20261019-1151) |
| REL-20261014-1509-engine | engine | repeated port.args scalar flags drop C++17 | todo |
| REL-20261015-1358-engine | engine | Editor W1: json secondary modules, module-id collision | verify-on-archive (m341 W1.1; other W1 sub-findings) |
| REL-20261015-1538-engine | engine | Editor W2: reflect consts, import order, `&mut` fields | verify-on-archive (m341 W2.1; other W2 sub-findings) |
| REL-20261015-2309-packages-2 | packages | bare `assert` mis-binds to `xiom.core.assert` | no-repro on `855766b73` (reduced `xiom.transport.foo` probe binds a panicking assert; both stdlib asserts are void now); packages re-verify with the harness |
| REL-20261015-2309-packages | packages | mixed-element tuples (`Tuple__Vec[UInt8]__Str__Int`) | acked; reduced probes + e2e lock green `63c3d809d`; packages verify owed |
| REL-20261016-2002-packages | packages | Float64 returns via dlsym'd fn pointers | reporter-close (m348) |
| REL-20261016-2004-packages | packages | imported struct `Array` collides with builtin | reporter-close (m349) |
| REL-20261016-2016-packages | packages | reading Result fields behind `&Result` crashes | reporter-close (m350) |
| REL-20261016-2134-engine | engine | engine pin round 2: 7 blockers | todo |
| REL-20261017-1123-pulse | pulse | duplicate module names resolve to the dependency (W001) | todo (confirm rule) |
| REL-20261017-1223-orbitdb-2 | orbitdb | E001 false positive on `&Vec[Int]` param | todo |
| REL-20261017-1223-orbitdb | orbitdb | transient compiler exit -1, no diagnostics, host load | todo |
| REL-20261017-1355-engine | engine | m340 dep-object cache key is consumer-specific | todo |
| REL-20261018-2301-pulse | pulse | cross-module struct type alias breaks field access | verify-on-archive (reduced cross-module alias probe green 2026-10-28; pulse repro to re-run) |
| REL-20261019-2206-orbitdb | orbitdb | A6 paged 57 GB blow-up at ~130k keys | todo (verify m375) |
| REL-20261020-1531-xvector-2 | xvector | C-21 build hang under a 16 GB Job Object | reporter-close (20261022-1616) |
| REL-20261020-1550-stdlib | stdlib | tuple-returning fn-value ABI (`EnumerateIter.map`) | reporter-close (20261024-1501) |
| REL-20261021-1652-engine-2 | engine | intermittent Vulkan bring-up AV (C08) | todo (needs no-capture repro) |
| REL-20261024-1101-pulse | pulse | arm64 fixed-arity printf fails IR verification | verify-on-archive (fix green) |
| REL-20261023-1854-pulse | pulse | tuple-returned Vec[Str] inline + cross-module unannotated Str bind | acked; reduced probes + e2e lock green on `de2854544`; pulse verify `probe_wsauth` |
| REL-20261028-1507-packages | packages | candidate orbitdb suite crash after 82 checks | **verified fixed + closed** by packages on e48213c66/ed2f38cff (0.2.1 160/160, 5/5 each) |
| REL-20261029-1324-orbitdb | orbitdb | published 0.2.1 carve crashed at 82 passes (0xC0000005); v0.65.1 160/160 | **verified + closed** by orbitdb on `ed2f38cff`; aggregate-local move fix (`e48213c66`); 0.2.1 160/160 release+ASan, modern/station/hybrid green |
| REL-20261028-1641-stdlib | stdlib | p_wave162 SSE loopback + spawned fixtures: 0xC0000374; v0.64.8 clean | **verified + closed by stdlib** on `605c6a879` (p_wave162 20/20, UAF witness 20/20, probes 418/418, smokes 963/963, modules 533/533, gate9 OK) |
| REL-20261029-1910-stdlib | stdlib | 1641 NOT fixed on ed2f38cff: UAF (p_wave162 row 12) + minimal early-return witness | **verified + closed by stdlib** on `605c6a879` (UAF witness 20/20; implicit tail-return ownership) |
| REL-20261029-1926-packages | packages | Vec[Vec]/UDP transport corruption still red (34-35/36); ed2f adds UDP reassembly failure | **acked**; superseded by `2110`; exact repro source requested in `2125-compiler` |
| REL-20261029-2110-packages | packages | 605c6a879 still red: transport 34-35/36 (b4 4/5, UDP 4/5); other 13 suites green | **VERIFIED + CLOSED by packages on `fd27be39a`** (transport 36/36 x5, sim 3/3, 13-suite matrix, orbitdb 160) |

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

### B3. Info -- 52 rows (historical; close by the reporter with evidence)

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
| REL-20261028-1532-packages | packages | inventory of colliding public symbols for 0052 |
| REL-20261029-1243-xvector | xvector | candidate ab97014d9 pre-verification GREEN; pin stays v0.65.1 |

## C. Fixed rows -- 66 awaiting reporter verification

Do not re-fix. `python tools\relay.py view --lane compiler --status fixed`
lists them with the commit refs. Every one needs the reporter's `verify`
(or `close`) on the candidate archive; the compiler can only act again if a
reporter reopens with new evidence.

## D. Verification baseline (as of 2026-10-27T22:52Z)

- Main: `0f86351cb` (docs) atop `b135dfbe8` (nested struct drops) and the
  rest of the session's fixes. Source candidate
  `target\candidate\xiom-b135dfbe8.exe` (sha256 `A7B48E5E...`). CI
  **`40602249053` SUCCESS** on `79b01d676` (2034 fix), **`40604134197`
  SUCCESS** on `fbfd2fbfe` (1854 lock), **`40610999107` SUCCESS** on
  `63c3d809d` (2303 fix), **`40612001738` SUCCESS** on `fea788908`
  (2309 lock), **`40692787953` SUCCESS** on `93d561107` (element drops),
  **`40701485338` SUCCESS** on `eac90fba6` (Bool payload patterns),
  **`40716560050` SUCCESS** on `855766b73` (W009 + linker stdout),
  **`40743760247` SUCCESS** on `a13040ac0` (Str rebind),
  **`40752513403` SUCCESS** on `46f7f50aa` (Vec[Vec] drops), and
  **`40763583267` SUCCESS** on `ab97014d9` (huffman borrow-retained return),
  and **`40884503130` SUCCESS** on `b135dfbe8` (nested struct drops; all
  matrix: windows, ubuntu, release parity x2, arm64 floor, MCU, hygiene).
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
- **Container element drops (ownership tag) landed this session:** a
  push-only `Vec[Struct{Vec}]` built from owned elements now frees each
  element's Vec payload at scope exit. Probe 496 MB -> **5 MB at 20k
  iterations**; disqualifying shapes (duplicate push, pop, escaped element
  Vec, with_len, borrow) stay leak-only and do not crash. **17/17 memory
  gates GREEN on debug+release; full e2e 2658/2658 (4 ignored) both.**
- **Str rebind reclamation (this session):** `s = s + chunk` leaked every
  replaced buffer (5000 small appends = **116 MB**). A per-local owned flag
  now frees the replaced owned buffer, never aliases/literals, and leaves
  self-assign leak-only; returned builders transfer the final value.
  Measured **116 MB -> 4 MB**; new gate `str_rebind_growth` + lock
  `str_rebind_shapes`. **18/18 memory gates; full e2e 2659/2659 both** (at
  that commit). Remaining measured app leaks after this and the Vec-of-Vec
  fix: nested struct 431 MB/1000, large `Vec[Str]`, Map/Set values, enum
  elements, removal-containing containers.
- **Vec[Vec[T]] element drops (this session):** pushing a fresh inner Vec
  now transfers its buffer to the container (the inner local's exit drop is
  disarmed); the tag frees each element's inner `data` pointer with stride
  32. 1000 x 512 KB leaked **505 MB**; now **5 MB at 20k iterations**.
  Alias-after-push, duplicate push, whole-element read, pop, element `push`,
  and non-header element methods disqualify (leak, no UAF). Gate
  `container_vec_of_vec`, lock `vec_of_vec_shapes`. **19/19 gates; full e2e
  2660/2660 both.**
- **Nested struct Vec fields (this session):** element field paths are
  recursive (`inner.data`); nested fresh Vec locals are marked MOVED, and
  codegen frees the leaf `Vec.data` through the GEP chain. 1000 x 512 KB
  leaked **431 MB**; now **5 MB at 20k iterations**. Nested reads/escapes,
  duplicate pushes disqualify; direct scalar reads allowed. Gate
  `container_nested_struct`, lock `nested_struct_drops`. **20/20 gates;
  full e2e 2662/2662 both.**
- **`REL-20261028-1159-stdlib` FIXED:** nested literal Bool payload patterns
  (`Ok(false)`, `Some(true)`) only checked the tag (Char/Int were handled),
  so both arms matched any Ok/Some. Bool now yields 0/1 and the payload slot
  is compared in plain and or-pattern arms. Repro runs rc 0; matrix covers
  Ok/Err/Some/None/or-patterns. Lock `e2e_result_bool_patterns`; 17/17
  gates; full e2e **2658/2658 (4 ignored) both**.
- **`REL-20261028-0052-compiler` FIXED (diagnose+surface):** top-level fns
  emitted as plain symbols that collide with libc/Winsock exports now warn
  (W009) for the known set (recv/send/connect/.../free/malloc/str*/...), and
  clang STDOUT is printed on failure, so LNK2005 + the defining object are
  visible instead of a bare exit code. Warning-only; renaming resolves the
  link. Symbol mangling remains a separate larger item. Locks
  `m0052_reserved_symbol_{warn,ok}` + checker_locks 39/39.
- **`REL-20261028-1801/1805-stdlib` FIXED (huffman SIGILL):** the v0.65.2
  blanket "borrowed arg is non-retaining" rule freed `lens` in
  `huffman_build` while `_tree_from_lengths` had stored the borrowed header
  in the returned `HuffmanTree` -> corrupted tree -> `llvm.trap`
  (0xC000001D). `borrow_safe` now requires a scalar/void callee return
  (struct/Vec/Str/pointer/unknown stays escaping). Huffman repro **10/10
  rc 0** debug+release; `borrow_vec_leak` still reclaimed; lock
  `e2e_borrow_tree_retained`. **`REL-20261028-1641` (p_wave162 SSE) is
  COMPLETE at `605c6a879`** (aggregate Vec-field exit drop re-reads the
  field; implicit tail return transfers ownership; p_wave162 20/20
  ASan+release).
- **Latest lane feedback (2026-10-29):**
  - stdlib (1533/1617): non-owning Channel on 5071f86; full matrix on
    ab97014d9 -- modules 533/533, probes 417/418 (only p_wave162), smokes
    963/963, barename 0/533, gate9 10/10+29; channel 5/5; 1801 verified.
  - xvector (1243): candidate ab97014d9 pre-verification GREEN (C-23/WAL
    x2, 8 previously-red probes, station/UI, C-21 16 GB); pin v0.65.1.
  - pulse: parse 8294 -> **1636 B/iter**, build 9092 -> **1446 B/iter**;
    /health 46.0 -> **21.9 KB/req**; residual Map/JSON (~2.0 KB obj, 287
    B/iter Map), parser ~1.6 KB, builder ~1.4 KB.
  - orbitdb: modern 166/166, station 169/169, hybrid 36/36, m375 PASS
    (200k **372.6 MB**, 1M **1,658.1 MB**); 0.2.1 carve still crashes.
- **`1324-orbitdb` VERIFIED + CLOSED by orbitdb on `ed2f38cff`
  (`e48213c66` fix):** 0.2.1 carve was 82 passes then 0xC0000005 on
  b135dfbe8. Root cause: `struct_literal_owns_vec` only moved fresh Vec
  locals into a literal; an aggregate local with owned Vec fields stayed in
  the drop set after the shallow header copy, so the returned Snapshot
  pointed at freed buffers (`snap_load_v2`). ASan UAF (2176 B
  `Vec[BTreeNode]`, element 8). Verdict: 0.2.1 160/160 release + ASan;
  modern 166/166 x2, station 169/169 x2, hybrid 36/36 + crash 5/5, 200k
  368.5 MB. Lock `aggregate_nested_return`.
- **`1641-stdlib` COMPLETE on our side (`605c6a879`):** two bugs. (a)
  `register_aggregate_drops` captured each Vec field's data pointer at the
  binding and freed it at exit; growing the field later reallocs and frees
  that buffer, so the exit drop double-freed (`ed2f38cff` re-reads the
  field instead). (b) `ed2f38cff` then exposed the real ownership miss: the
  body's trailing `Ok(conn)`/`Some(conn)` is an implicit return but was
  scanned as a plain escape, so the exit drop freed the buffer returned to
  the caller (stdlib `p_borrowed_mut_grown_vec_uaf`, p_wave162 row 12).
  `605c6a879` marks the body tail with `mark_returned_roots`. Evidence:
  stdlib witness 5/5 ASan+release; p_wave162 20/20 ASan + 20/20 release;
  lock `tail_return_aggregate`. **Verified + closed by stdlib.**
- **`1910-stdlib` FIXED at `605c6a879`:** the ed2f38cff UAF report; same
  implicit-tail-return fix. **Verified + closed by stdlib on 605c6a879.**
- **`2110-packages` VERIFIED + CLOSED by packages on `fd27be39a`:**
  transport 36/36 x5 (b4 + u4), sim 3/3, 13-suite matrix, orbitdb 160.
  Root causes: by-value aggregate into `_enqueue` through `&mut`; the
  `_ok_outcome(done)` helper return not treated as a return (payload
  dangling at `rl_on_packet`); a projected Vec field shallow-copied while
  its source aggregate still dropped. Fixes: retaining-callee by-value
  transfer, helper-wrapped return recognition, projected-field move.
- xvector durable repros with this build:  - `docs\repro\xv-vec-struct-push\probe.xi` (XVC-C-23): **PASS**, named and
    inline forms bad=0, exit 0.
  - `tests\probes\probe_wal_file.xi` (built with the bin's pinned stdlib +
    XIOM_HOME): **0 failed checks**, exit 0, 8 MB peak. WAL persist/reopen/
    torn-tail recovery are green, which is the durable-lane hotfix gate.
  - Not yet re-run here: xvector station loopback and orbitdb hybrid (their
    repo paths/runner setups differ from the continuation prompt; the lanes
    own those runs).

## E. Compiler's own open broadcasts (14)

These are `from: compiler -> all` announcements (v0.64.7/v0.64.8/v0.64.9/
v0.65.0/v0.65.1/v0.65.2, incident/withdrawal notices, candidate notices).
The compiler is the reporter, so it can `close` them as info with a ref.
Not part of the 15 lane-addressed rows.

## F. Definition of done

Every open/acked row is `verified`/`closed` with evidence, or `fixed` with
a compiler probe and the reporting lane's verification on the candidate
archive. No `todo` bug/perf rows. Then:

1. 20/20 memory gates on debug+release.
2. Full e2e debug+release (baseline 2662/2662, 4 ignored; grows with tests).
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

## H. Container element drops -- ownership tag (IMPLEMENTED, v1 conservative)

Pre-fix measurement on `92e0d07b5`: 1000 iterations of
`Vec[R].new(); rs.push(R{ xs: Vec[UInt8].with_len(512000) })` peaked
**496 MB** (~0.5 MB/iter): the outer buffer was freed, each element's Vec
payload was not. The engine editor 9 MB/frame and OrbitDB m375 sit in this
family.

Implemented as a prepass OWNERSHIP TAG, not a blind loop:
- Candidates: `Vec[T].new()`/`with_capacity` (EMPTY) only; `with_len`
  elements are uninitialized. T must be a plain struct with >=1 Vec field
  (aliases followed; enums/containers skipped; ambiguous leaves skipped).
- A container stays tagged only while every push/insert element is owned:
  a same-block named aggregate in the prepass aggregate set (pushed once),
  or an inline literal of the same element type whose every Vec field is
  fresh / an owned deep-copy projection (ident fields rejected).
- Disqualifiers: pop/remove/swap_remove/clear/truncate/resize/set/dedup/
  retain/sort/fill/swap/extend/add/append/put; borrows of the container;
  method calls on an element; an element projection in an escape context;
  a Vec-field read of an element in a binding/argument/return context.
  Scalar element reads and header-only safe-method uses of an element's Vec
  field (`rs[i].xs.len()`) are allowed.
- One declaration only (no shadowed tag reuse); must already be in the Vec
  drop-safe set. Codegen frees only the validated field NAMES whose LLVM
  type is `%struct.Vec`, reads len/data at scope exit, frees payloads before
  the element buffer, and uses the existing idempotent flag.

Evidence: `container_elem_owned_drop` (496 MB -> **5 MB at 20k iters /
0.1 s**) and `container_elem_unsafe_shapes` (duplicate push, pop, escaped
element Vec, with_len, borrow: leak-only, no crash), plus
`container_elem_tag_edges` (shadowing, blocks, with_capacity, scalar write,
two Vec fields). **20/20 memory gates GREEN on debug+release**; full e2e
**2662/2662 (4 ignored) on both**.

### H1b. Vec[Vec[T]] element drops (IMPLEMENTED)

`Vec[Vec[UInt8]]` chunk buffers leaked the inner buffer per element
(505 MB/1000 x 512 KB). A fresh inner Vec pushed into a container store is
now MOVED (its own drop disarmed); the element tag accepts same-block fresh
locals or inline fresh inner Vecs, and frees each element's `data` pointer
(stride 32) before the outer buffer. Alias-after-push, duplicate push,
whole-element reads, pop, element `push`, and non-header element methods
disqualify; header-only `outer[0].len()` stays allowed. Measured **505 MB ->
5 MB at 20k iterations**. Gate `container_vec_of_vec`, lock
`vec_of_vec_shapes`.

### H1c. Nested struct Vec fields (IMPLEMENTED)

`Vec[Outer{ inner: Inner{ data: Vec } }]` leaked the nested payload
(431 MB/1000 x 512 KB). Element field paths are now RECURSIVE (`inner.data`,
depth 8, cycle-safe, aliases followed); the aggregate prepass marks nested
fresh Vec locals MOVED, so the source and the element never double-own.
Direct scalar element reads stay allowed; nested struct/Vec reads, escapes
and duplicate pushes disqualify. Codegen emits the full GEP chain and frees
the leaf `Vec.data`. Measured **431 MB -> 5 MB at 20k iterations**. Gate
`container_nested_struct`, lock `nested_struct_drops`.

### H2. Str rebind -- replaced-buffer reclamation (IMPLEMENTED)

`s = s + chunk` / `s = int_to_string(..)` leaked every replaced buffer:
5000 small appends peaked **116 MB** on 855766b73. A per-local owned flag
now frees the previous owned buffer on each fresh replacement, disarms on a
non-fresh alias (never freeing aliases/literals), leaves self-assign
leak-only, and skips the exit drop for the rebind locals (returned builders
transfer the final value). Measured **116 MB -> 4 MB**; residual final-value
leak for a reassigned non-returned local only. New gate `str_rebind_growth`
+ lock `str_rebind_shapes`.

Still conservative leaks (never UAF): Str fields of elements, Map/Set
values, enum element types, containers with removals/reorders, and
large-string `Vec[Str]`. These are the next memory targets. The engine
editor 600-frame soak and OrbitDB m375 200k soak must re-measure on the
candidate archive.

## I. Candidate verification, then release acceptance (workflow)

Do NOT use ad-hoc `cargo build` binaries as lane acceptance. A cargo build
is a useful per-fix smoke, but it does not prove the shipped form: flags,
asset layout, arm64 cross-builds and pinned stdlib differ, and the durable
regression was found in a packaged lane build. Lanes also pin by artifact
hash, so the acceptance object must be an artifact.

1. **Candidate (pre-release, no tag):** create a candidate ref from main
   (e.g. `release/v0.65.3-rc1`) with ONLY `Cargo.toml` + `Cargo.lock`
   workspace version `0.65.3-rc.1` (the release dry run labels artifacts
   with the Cargo.toml version; leaving 0.65.2 would make the zip
   indistinguishable from the shipped v0.65.2). Never tag and never publish
   a release from it.
2. `gh workflow run release.yml --ref release/v0.65.3-rc1` builds the same
   packages as a release, uploads workflow artifacts, and creates nothing.
   Record the run id, commit SHA, and each asset's SHA256.
3. Nudge the owning lanes with one relay item each (engine, orbitdb,
   stdlib, packages, pulse, xvector, benchmark) containing the run URL, the
   candidate label, the commit SHA, and their exact repro/soak. Ask them to
   run against the EXTRACTED candidate and post the verdict; a `fixed` row
   becomes `verified` only on that run.
4. Keep fixing on main in parallel; cut `rc2` when fixes land. A candidate
   does not change any lane pin -- adoption waits.
5. When every `fixed` row is verified, the checklist has no `todo`
   bug/perf row, and the owner says GO: bump main to `0.65.3`, tag, publish,
   run release-check, then ask lanes for **release acceptance** on the
   official assets (hash pin) and, separately, adoption of the new pin.
   Acceptance and adoption are two different decisions; the release is not
   the first test of a fix.



