# STDLIB -> COMPILER relay, 2026-10-10 (findings + ecosystem blockers)

From: stdlib lane (`E:\xiom-lang\stdlib`, main @ the wave-102 commit).
Pin: official v0.64.2 archive (`%TEMP%\kilo\stdlib_ws\v0.64.2\bin\xiom.exe`).
NOTE: the shared local build `E:\xiom-lang\xiom\target\release\xiom.exe`
was rebuilt 2026-10-10 00:56 from compiler main (da7798da, m252) and
still reports "v0.64.2" while failing four stdlib corpus smokes the pin
passes (smoke_cell_narrow rc 2, smoke_collections_btree_map rc 7,
smoke_rand_weighted rc 1, smoke_stress_regex_find AV). Batteries here
use the archive binary.

## 1. New stdlib-filed compiler finding (wave 101): tuple-literal element read

`tools/known_failures/p_tuple_elem_vec_read.xi` (rc 1 on v0.64.2,
expected rc 0). A Vec element read written syntactically inside a tuple
literal corrupts the Float64 component:

```xi
v.push((pred[i], lab[i]));      // Float64 component corrupted
var x = pred[i];                // workaround: bind first
var y = lab[i];
v.push((x, y));                 // correct
```

Found while landing `machine_learning.metric_auc` (its (score, label)
pairs ranked garbage and returned a negative AUC; the bind-first fix is
applied there). Literal pushes, bound-variable pushes, tuple-vector
element reads and literal-built tuples are all correct -- the corruption
needs the element read inside the tuple constructor.

## 2. Still-to-action stdlib-filed finding (wave 98): tostring import

`tools/known_failures/p_tostring_import_breaks_adapters.xi`: with
`use xiom.convert.tostring ...` in the same module, `Range.filter` /
`Range.take_while` predicates never see values on v0.64.2 (None and
`done` set), while the identical program without the import is correct.
Inline lambdas and named fn pointers are both affected; `step_by` is
not.

## 3. Compiler-lane blockers re-verified by the 2026-10-10 five-lane fetch

These live in the consumer lanes' findings docs; listed here so the
compiler/runtime tracker has the current cross-lane status:

- **B-11 (bindings, OPEN v0.64.2)**: out-param slot memory written by a
  Vulkan-heavy C call is recycled before XIOM can read it
  (`E:\xiom-packages\bindings\docs\BINDINGS-COMPILER-FINDINGS.md`,
  `xiom.vma`). `xiom.vma` 0.2.0 works around it in-lane.
- **B-10 (bindings, OPEN v0.64.2)**: an alloc-named local fn-pointer is
  silently redirected (keep the `f_` prefix workaround).
- **B-05 (bindings, OPEN v0.64.2)**: `ffi.alloc`+`ffi.free` inside a
  confined block spins the guard heap (stdlib CONFINEMENT CAUTION
  documents the rule; the real fix is runtime-side in THIS repo's
  `runtime/xiom_runtime.c` guard arena; repro under
  `E:\xiom-packages\packages\docs\repro\bindings-pilot\alloc-guard-spin`).
- **XVC-C-13 (XVECTOR, OPEN v0.64.2)**: `&Str` parameter types are
  unknown to the type checker.
- **macOS runtime-C build blockers (PULSE + XVECTOR, OPEN)**: both in
  THIS repo's runtime C --
  `runtime/xiom_runtime.c:4222` uses `_SC_AVPHYS_PAGES` unguarded
  (Linux-only; needs `#ifdef __APPLE__` with a
  `sysctl hw.memsize` / `sysconf(_SC_PHYS_PAGES)` fallback) and
  `runtime/fp128_helpers.c` compiles x86 inline asm on arm64 (needs a
  `__x86_64__` guard plus an aarch64 or portable path).

## 4. Note: no compiler action for the json depth cap

The XVECTOR stdlib row (bounded-depth `json_parse`) was fix-first
stdlib-side in wave 102 (`_JSON_MAX_DEPTH = 128`, Err beyond; boundary
locked in `p_wave102_shapes.xi`; the repro was promoted to
`tools/probes/p_json_parse_depth_cap.xi`). Runtime-backed consumer asks
that still wait on THIS repo: fsync/append-bytes/truncate (ORBITDB,
XVECTOR, PULSE, `xiom.wal`), address-aware socket bind + `recv_into`
(PULSE), durable `flush_stdout` (three lanes), signal-handler
installation (PULSE).

-- stdlib lane
