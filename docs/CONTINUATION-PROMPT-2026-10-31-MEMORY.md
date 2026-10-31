# CONTINUATION PROMPT — 2026-10-31 memory-leak slice (m375 / 2357-engine)

Self-contained prompt for the next session. Read this first, then
`docs/SESSION-HANDOFF.md`, then the live relay.

## State

- v0.65.4 is released and must not be re-cut. The fix line is **v0.65.5**.
- Compiler repo: `E:\xiom-lang\xiom`, main at `dde4739af` (this prompt
  commits on top). Working tree must stay clean except the pre-existing
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

Do not revert these. The drop-glue work is conservative by design; do
not replace it with blind frees.

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

**Stdlib (open, blocked on stdlib lane):**
- `stdlib/xiom/string/string.xi:105-124` `str_concat` mallocs `buf`,
  then `Str.from_cstring(buf)` **copies** it and the malloc input is
  never freed.
- `stdlib/xiom/convert/cstring.xi:25-48` `from_cstring` builds a
  `Vec[UInt8]` and calls `Str::from_utf8(buf)`; the intermediate copy
  is part of the residual.
- `str_slice` (:127-148) and other `from_cstring(malloc(...))` sites
  share the shape.
- The v0.65.4 stdlib is **precompiled**; the new tail-return rebind and
  Vec-drop logic are not in `str_repeat`/`from_cstring` until stdlib is
  rebuilt with `683c77d5b` or later.

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
- `REL-20261031-2156-compiler` — **stdlib bug/rebuild ask** for the
  copy-leak; waiting on stdlib tag/sha. Check this first.
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
3. If stdlib has a new tag/sha:
   - build/retest the probes against the new stdlib;
   - require p_structstr/p_vecstr/p_mapstr/str_repeat flat or near
     baseline before the engine soak.
4. If stdlib has not replied, continue the compiler slices below.

## Remaining compiler slices (in order)

### A. Map/Set value payloads (`p_mapstr`)

Probe: `C:\Users\lefte\AppData\Local\Temp\kilo\leak-agent\proj\p_mapstr.xi`
(5k iterations, fresh 2 KB Str key+value into `StringStrMap`).
Baseline ~40 KB/iter.

`StringStrMap` is `{ buckets: Vec[Int]; keys: Vec[Str]; values: Vec[Str];
next: Vec[Int]; live: Vec[Bool]; count: Int }`
(`stdlib/xiom/collect/stringmap.xi:196+`). The aggregate drop frees the
Vec **buffers** but not `keys`/`values` Str elements. Do not free a
Vec[Str] field blindly: a map field can contain borrowed/literal Str.
Implement typed leaf ownership (per-field Str proof) or a Map-specific
drop helper with element tags. See the leak report §6.

### B. Removals (`p_remove`)

Probe: `...\proj\p_remove.xi`. `vec_drop.rs:879-880` marks
`remove/swap_remove/sort/retain/...` as `elem_unsafe`, so removed
payloads leak. Keep the element tag across `remove`/`swap_remove` and
drop the removed element at the removal site (or transfer it if used).
Do not drop an element whose returned value is consumed.

### C. By-value aggregate parameter ownership (`p_panel_move`)

Probe: `...\proj\p_panel_move.xi` (baseline ~190 KB/iter; currently
403 -> 277 MB peak). The callee frees its deep-copied locals, but the
caller's fresh P buffers / the callee's by-value aggregate param are
double-disarmed. Choose exactly one owner: either register the callee's
by-value aggregate param for aggregate drop, or keep the caller drop.
Do not do both. The existing `retaining_callees` logic is not sufficient
for by-value struct params.

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
  it until stdlib rebuilds. `REL-20261031-2156-compiler` is the ask.
- The temp patched lib is evidence only.
- One heavy build at a time (the PC crashed/OOM from concurrent builds).
- Relay evidence over prose; update the item with before/after slope.

## Definition of done

1. Stdlib tag/sha with `str_concat`/`from_cstring` ownership fixes and a
   rebuild on `683c77d5b`+.
2. p_structstr, p_vecstr, p_mapstr, p_remove, p_panel_move flat or near
   baseline (recorded numbers).
3. Editor 600 frames <2 GB, slope <=0.5 MB/frame.
4. Engine verifies 2357/1423; orbitdb re-measures m375; benchmark t1
   re-measures ~9.98 MB on v0.65.5.
5. Only then cut v0.65.5, update `SESSION-HANDOFF.md` and the closure
   doc, and mark the relay items fixed/verified.
