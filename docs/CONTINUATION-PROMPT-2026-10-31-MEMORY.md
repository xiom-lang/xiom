# CONTINUATION PROMPT — 2026-10-31 memory-leak slice (m375 / 2357-engine)

Self-contained prompt for the next session. Read this first, then
`docs/SESSION-HANDOFF.md`, then the live relay.

## State

- v0.65.4 is released and must not be re-cut. The fix line is **v0.65.5**.
- Compiler repo: `E:\xiom-lang\xiom`, main at `f3e18120a` (slice C +
  Str copy-out + rebind exit `f537b1b14`, typed Str element leaves
  `e86989f46`, removal payloads `2e8193e2b`, Map insert-path contract
  `f3e18120a`). Working tree must stay clean except the pre-existing
  `.gitignore` change and untracked `m211_listdir_probe/`.
- No v0.65.5 tag yet. Do not tag until the rebuilt stdlib + engine
  600-frame soak + lane sign-off are green.
- PC crash/OOM warning: do NOT run multiple memory-heavy editor/IR
  builds concurrently. One full `apps/ui_showcase/main.xi` build is
  ~37 min and the host OOMs under parallel builds. Use the targeted
  probes and the full build only when necessary, one at a time.

## Commits already landed on main (unreleased v0.65.5 candidates)

| commit | fix | tests/evidence |
|---|---|---|
| `2b2b51b54` | 2357-2 global mutable-name collision (`g_clip` Int vs AnimClip) | full showcase assets selftest PASS |
| `61f9e3b6a` | Float128/Int128 casts + first-wins extern signatures | `m1600_float_to_int128_cast_emits_conversion`, `m1619_duplicate_extern_first_declaration_wins` |
| `f644fd54b` | user `value.alloc/free` no longer hijacked by builtins | `m2143_user_alloc_method_is_not_hijacked` |
| `216a70322` | reject by-value fn -> `fn(&T)` (1552) | `m1552_by_value_fn_is_not_accepted_as_by_ref`, `m1552_by_ref_fn_value_still_compiles` |
| `e875528ce` | reclaim by-value `Vec[T]` parameter copies | `m375_by_value_vec_param_rebind_is_dropped`; p_vecrebind 403 MB -> flat |
| `364d19efd` | fresh Str fields of aggregates (deferred scope-exit free), tail-return Str rebind, safe str builders | `m375_struct_fresh_str_field_is_dropped`, `m375_tail_return_str_rebind_is_reclaimed`; p_structstr 364 -> 278 MB |
| `683c77d5b` | `Vec[Str]` element payload drops | `m375_vec_str_elements_are_dropped`; p_vecstr 402 -> 355 MB; p_panel_move 403 -> 277 MB |
| `f537b1b14` | reader by-value Vec args keep caller drop; arena-aware Str copy-out; non-returned Str rebind exit free | `m375_by_value_vec_arg_to_reader_keeps_caller_drop`, `..._writer_stays_leak_only`, `m375_str_rebind_local_exit_is_freed`, `m375_returned_str_rebind_is_not_freed_at_exit`; p_panel_move 337/368 -> 5/34 MB; p_structstr 255/259 -> 5/33; p_vecstr 329/364 -> 9/35; p_strrepeat 210/242 -> 5/33; p_vecrebind 5/33 |
| `e86989f46` | typed `Str` leaves of container elements (`Vec[Struct{Str}]`) | `m375_elem_str_leaf_is_dropped`, `m375_elem_borrowed_str_stays_leak_only`; p_elemstr 15/44 -> 5/33 MB |
| `2e8193e2b` | removal/reorder tag + dead boxed removed-element free | `m375_dead_removed_vec_element_is_freed`, `m375_used_removed_vec_element_stays_leak_only`; p_remove 297/313 -> flat (50k x 16 KB: 4/34 MB) |
| `f3e18120a` | `#[consumes_str]` + `#[owns_return]` Map/Set Vec[Str] element drops | `m375_consumes_str_map_payloads_are_freed`, `m375_consumes_str_literal_arg_stays_leak_only`; p_mapstr insert 27/55 -> 5/34 MB; the one-map update witness stays 8/36 MB (stdlib callee must free discarded key/overwritten value) |

Do not revert these. The drop-glue work is conservative by design; do
not replace it with blind frees.

### Slice C detail (landed this session)

A by-value `Vec[T]` parameter is a shallow header: the caller's data
buffer is shared. The argument escape used to disarm the caller's drop
and the callee never freed a borrowed param, so every `read(v)` call
leaked the caller's buffer (p_panel_move 190 KB/iter). New whole-program
analysis `fresh_return::collect_byval_vec_readers` marks a function as a
reader only when EVERY by-value Vec param is used solely as the receiver
of `len`/`is_empty`/`capacity`/`count`, or as the RHS of a local bind
(which deep-copies). A reader returning a scalar / carrying nothing keeps
the caller's ownership; any bare use (return, call arg, assignment,
element projection) or mutating method (`push`/`reserve`/`remove`/...)
leaves the historical leak-only behavior. `retaining_callees` (&mut or
carrying return) still wins. Do not widen the read-only method set to
element-returning methods (`get`/`first`/`last`) without owning the
element-alias proof.

## The memory leak: current model

Two acceptance gates, one compiler slice:

1. `REL-20261030-2357-engine` — editor 600-frame soak; ~100 MB/s WS /
   ~120 MB/s private; 2 GB gate RED.
2. orbitdb m375 (`1807/2102/0423/0606/1100`) — current v0.65.4 numbers
   are ~607 MB/200k, 2.49 GB/1M; 57.8 GB/129k was the old v0.64.8
   transient Vec retention, not current.
3. Compiler drop glue: Map/Set values, enum payloads, removals,
   by-value aggregate params, recursive aggregate leaves.

### Confirmed root causes

**Compiler (fixed slices above):**
- by-value `Vec[T]` params were not drop sites (`vec_drop.rs` bare-ident
  exclusion);
- fresh Str fields of local aggregates were never freed;
- function-body TAIL expressions were scanned as alias escapes, which
  disabled Str rebind for every builder whose final expression is the
  accumulator;
- `Vec[Str]` element payloads were never freed.

**Stdlib (FIXED on `11fb9b2` / candidate `7d47924`):**
- `stdlib/xiom/string/string.xi` `str_concat` mallocs `buf`, then
  `Str.from_cstring(buf)` copies it; all 10 local sites now
  `free(buf)` after the copy.
- `stdlib/xiom/convert/cstring.xi` `from_cstring` no longer builds a
  `Vec[UInt8]` + `from_utf8`; it calls `xiom_str_len` /
  `xiom_str_from_vec` directly.
- The compiler half of the same leak remained after the stdlib fix:
  confined-block Str copy-out duplicated main-heap results. Fixed by
  `emit_str_copy_out_if_arena` (C07 promote); see the slice list above.
- Rebuild the paired toolchain on `7d47924` for any further probe
  numbers; the v0.65.4 precompiled lib is superseded for v0.65.5.

**Proof of the stdlib part:** a temp copy of the v0.65.4 lib with

```xiom
let out = Str.from_cstring(buf);
free(buf);
return out;
```

in `str_concat` drops `p_structstr` **278 MB -> 159 MB**. The temp lib is
`C:\Users\lefte\AppData\Local\Temp\kilo\v0654-patched-lib` (experiment
only; do not ship/rely on it).

### Relay state

- `REL-20261030-2357-engine-2` — fixed (`2b2b51b54`), engine verify.
- `REL-20261031-1423-engine` — compiler half fixed; residual P1 s23 is
  engine source arithmetic (`p1_layer_add` names by count; uniqueness
  loop already in `1461c07`). Engine verify.
- `REL-20261031-1552-stdlib` — fixed (`216a70322`), stdlib verify.
- `REL-20261031-1600-stdlib` — compiler half fixed (`61f9e3b6a`); link
  needs `__fixdfti` family in `stdlib/runtime/fp128_helpers.c`.
- `REL-20261031-1619-pulse` — fixed (`61f9e3b6a`), pulse re-run arm gate.
- `REL-20261031-2156-compiler` — **FIXED + VERIFIED on our side.**
  Stdlib main `11fb9b2`, candidate `release/v0.65.5-prep` `7d47924`
  (package 0.65.5, COMPILER_VERSION v0.65.4). The 10
  `Str.from_cstring(buf)` sites free the input; `from_cstring` uses
  `xiom_str_len`/`xiom_str_from_vec` directly. Paired-toolchain
  measurements are in the State/commits sections. Note: the stdlib's
  quoted compiler `d4d52e0c9` predates `683c77d5b`; our numbers are on
  the current main + `7d47924`.
- `REL-20261031-2243-compiler` — engine checkpoint soak ask: rebuild the
  editor on compiler `f537b1b14` + stdlib `7d47924` (paired toolchain
  path in the item) and run 300/600 frames under 2 GB. Interim, not the
  final v0.65.5 cut.
- `REL-20261027-1931-engine` — still open, distinct class; needs the
  original ~44 MB editor `.ll` function body around line 65929 or a
  reduced two-module `load X; store as Y` repro.
- `2357-engine` — partial compiler fixes; full soak not rerun.

## First actions next session

1. Read the live relay (stdlib reply first):
   ```powershell
   cd E:\xiom-lang\xiom-relays
   python tools\relay.py show REL-20261031-2156-compiler
   python tools\relay.py view --lane compiler --status open
   python tools\relay.py view --lane compiler --status acked --kind bug
   ```
2. Re-check the compiler tree:
   ```powershell
   git -C E:\xiom-lang\xiom status --short
   git -C E:\xiom-lang\xiom log --oneline -10
   ```
3. Build the paired toolchain from `C:\Users\lefte\AppData\Local\Temp\kilo\toolchain-7d47924`
   (bin = this compiler, lib = `E:\xiom-lang\stdlib` at `7d47924`) and
   re-measure p_structstr/p_vecstr/p_mapstr/str_repeat/p_panel_move.
   Current numbers (WS/private MB): p_structstr 5/33, p_structstr_small
   5/33, p_vecstr 9/35, p_strrepeat_iso 5/33, p_panel_move 5/34,
   p_vecrebind 5/33, p_elemstr 5/33, p_remove flat (50k long: 4/34),
   p_mapstr insert 5/34 (update witness 8/36; stdlib callee fix open).
4. Engine rebuild + 600-frame soak on the paired toolchain is the
   remaining acceptance gate. If engine cannot take the temp toolchain,
   wait for the v0.65.5 candidate pin.
5. Slice A (Map/Set payloads) is the last measured compiler residual and
   needs the stdlib contract below; then 1931 (needs the editor `.ll`).

## Remaining compiler slices (in order)

### A. Map/Set value payloads (`p_mapstr`) — INSERT PATH LANDED, UPDATE PATH OPEN

Probe: `C:\Users\lefte\AppData\Local\Temp\kilo\leak-agent\proj\p_mapstr.xi`
(5k iterations, fresh 2KB Str key+value into a fresh `StringStrMap`, one
insert per map). **27/55 -> 5/34 MB flat** in `f3e18120a` (compiler half
of the stdlib `9c26a88` contract).

Contract now implemented:
1. `#[owns_return]` on the map/set constructors makes the aggregate's
   Vec buffers drop.
2. `#[consumes_str]` on `*_map_put` marks fresh Str args moved; the
   prepass proves every `&`/`&mut` use is a contracted call with fresh Str
   args (or a safe borrow read) and frees each owned `Vec[Str]` field's
   elements at exit. Literal/borrowed args or unknown uses stay leak-only.

**Open (stdlib callee fix):** the shared-map witness
`stdlib/tools/probes/p_mapstr_shapes.xi` (one map, the SAME 2KB key every
iteration) still peaks **8/36 MB at 5k** (~4KB/iter). The update branch
`stdlib/xiom/collect/stringmap.xi:270-272` discards the fresh key and
overwrites the old value without freeing either; the compiler cannot know
insert-vs-update, so the callee must `free` them. Same for
`string_map_put` in stringmap/intmap (key not stored on update). Radix
needs the same treatment or disqualification. Tracked in the stdlib
relay item filed after `f3e18120a`.

### B. Removals (`p_remove`) — LANDED (`2e8193e2b`)

Probe: `...\proj\p_remove.xi`. Removal/reorder-only methods keep the
element tag; the exit loop walks the current len, so remaining elements
are owned and removed slots are never freed twice. A provably-dead
`let gone = v.remove(i)` frees the boxed element's Vec data + box at the
binding; a consumed result stays leak-only (transfer path). Measured
297/313 -> flat (50k x 16 KB: 4/34 MB). The `container_elem_unsafe_shapes`
negative control stays green. A future slice can transfer the removed
payload into a local drop instead of only handling the dead case.

### C. By-value aggregate parameter ownership (`p_panel_move`) — LANDED

Probe: `...\proj\p_panel_move.xi`. Fixed in `f537b1b14`: a
by-value Vec local passed to a callee proven to only READ its by-value
Vec params (`len`/`is_empty`/`capacity`/`count`, or a local bind that
deep-copies) keeps the caller's drop. A callee that mutates (`push`,
`reserve`, `remove`, ...) or can carry the value out (`&mut` param,
aggregate-carrying return) still disarms the caller and stays leak-only.
Measured: p_panel_move 337/368 MB -> **5/34 MB**; p_vecrebind flat
(30k x 64 KB churn, **5/33 MB**). Reader analysis:
`fresh_return::collect_byval_vec_readers`; tests
`m375_by_value_vec_arg_to_reader_keeps_caller_drop` and
`..._writer_stays_leak_only`. Do not widen the read-only method set to
element-returning methods (`get`/`first`/`last`) without an element-alias
proof.

### D. 1931 alloca/slot type confusion

Distinct from the collision. Needs engine evidence:
`@g_ed_iv` is unique; bad value comes through an SSA pointer. Code path
`stmt.rs:1514/1523`/`:1532/1539` + `coerce.rs:972-973`. Ask engine for
the `.ll` slice/reduced repro; do not guess a fix.

### E. 600-frame soak

Only after the stdlib rebuild + A/B/C:
- rebuild editor (one at a time, ~37 min);
- run 600 frames under 2 GB, slope <=0.5 MB/frame;
- update `REL-20261030-2357-engine` with before/after and lane numbers.

## Test inventory (already written)

- `cargo test -p xiom-codegen --test feature_regression_tests m375` (4)
- `cargo test -p xiom-codegen --test feature_regression_tests m16`
  (1600/1619)
- `cargo test -p xiom-codegen --test feature_regression_tests m1552`
- `cargo test -p xiom-codegen --test feature_regression_tests m2143`
- `cargo test -p xiom-codegen --test e2e_tests drop`
- `cargo test -p xiom-codegen --test e2e_tests str`
- `cargo test -p xiom-codegen --test e2e_tests global`
- `cargo test -p xiom-check` (214/214 expected)
- release build: `cargo build -p xiom --release`; run the probes with
  `C:\Users\lefte\AppData\Local\Temp\kilo\fixed-toolchain\bin\xiom.exe`
  (lib = v0.65.4) or `target\release\xiom.exe` with an appropriate lib.

## Probe/measure commands

Leak report: `C:\Users\lefte\AppData\Local\Temp\kilo\leak-agent\REPORT.md`.
Probes: `...\leak-agent\proj\*.xi`. Sampler:
`...\leak-agent\sample_mem.ps1`, e.g.

```powershell
$d="C:\Users\lefte\AppData\Local\Temp\kilo\leak-agent\proj"
$x="C:\Users\lefte\AppData\Local\Temp\kilo\debug-toolchain\bin\xiom.exe"
& $x "$d\p_structstr.xi" -o "$d\p_structstr_new.exe"
& "C:\Users\lefte\AppData\Local\Temp\kilo\leak-agent\sample_mem.ps1" `
  -Exe "$d\p_structstr_new.exe" -Tag "p_structstr_new" -TimeoutSec 120
```

Editor soak (from the leak report):

```powershell
$env:XIOM_DEMO_EDITOR_FRAMES=300
E:\xiom-projects\xiom-engine\build\xiom-editor.exe   # cwd = engine root
```

Full showcase reproduction: `C:\Users\lefte\AppData\Local\Temp\kilo\p1-pre`
(faa5a41) built with the published v0.65.4; the fixed compiler build is
`build\ui_showcase.exe` there. The engine worktree is
`E:\xiom-projects\xiom-engine\.kilo\worktrees\feat-p1-data-wiring`
(1461c07, green P1).

## Coordination / hard rules

- The leak is **not closed**. Do not claim it is; do not tag v0.65.5
  until stdlib rebuild + engine soak + lane sign-off.
- A UAF is worse than a leak: do not free Str/Map/removed payloads
  without ownership proof. Conservative leak-only is acceptable until
  the typed pass proves ownership.
- `13fc28352` is compile-time only; never claim it as runtime reclaim.
- The v0.65.4 stdlib is precompiled; compiler drop fixes cannot change
  it. The rebuilt stdlib candidate is `7d47924`; use the paired
  toolchain. `REL-20261031-2156-compiler` is fixed/verified.
- The temp patched lib is evidence only.
- One heavy build at a time (the PC crashed/OOM from concurrent builds).
- Relay evidence over prose; update the item with before/after slope.

## Definition of done

1. Stdlib rebuilt on `7d47924` with `str_concat`/`from_cstring`
   ownership fixes (done; pin it in the v0.65.5 candidate).
2. p_structstr, p_vecstr, p_mapstr, p_remove, p_panel_move flat or near
   baseline (recorded numbers). Current: p_structstr 5/33,
   p_vecstr 9/35, p_panel_move 5/34, p_vecrebind 5/33, p_elemstr 5/33,
   p_remove flat (50k long: 4/34); **p_mapstr insert 27/55 -> 5/34**
   (`f3e18120a`), update path 8/36 pending a stdlib callee free for the
   discarded key / overwritten value.
3. Editor 600 frames <2 GB, slope <=0.5 MB/frame.
4. Engine verifies 2357/1423; orbitdb re-measures m375; benchmark t1
   re-measures ~9.98 MB on v0.65.5.
5. Only then cut v0.65.5, update `SESSION-HANDOFF.md` and the closure
   doc, and mark the relay items fixed/verified.
