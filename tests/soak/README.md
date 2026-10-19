# Escaped / per-frame Vec soak witnesses

Manual memory witnesses for `REL-20261019-1343-engine` (the m368 follow-up).
They are **not** part of the default e2e suite: the leak witness allocates
~4.8 GB if the arena is absent.

## Files

- `escaped_frame_vec_leak.xi` -- baseline: a frame-like struct with three
  1M-element Vecs is returned from `make_frame`, consumed by `consume`, and
  recreated 200 times. Without an arena this is the historical leak.
- `escaped_frame_arena_soak.xi` -- the same shape wrapped in
  `xiom_guard_heap_enter/exit` per frame; expected flat.

## Commands (Windows, main compiler)

```powershell
$env:XIOM_STDLIB = "E:\xiom-lang\xiom\stdlib"
$exe = "E:\xiom-lang\xiom\target\release\xiom.exe"

# Baseline (run CAPPED; 4.8 GB if the leak is present)
& $exe --no-cache --force -o escaped_frame_vec_leak.exe escaped_frame_vec_leak.xi
# or: out/retest/run_capped.ps1 -LimitMB 2048 escaped_frame_vec_leak.exe
& .\escaped_frame_vec_leak.exe

# Scoped arena (expected flat)
& $exe --no-cache --force -o escaped_frame_arena_soak.exe escaped_frame_arena_soak.xi
& .\escaped_frame_arena_soak.exe
```

## Measured on main (release compiler, 2026-10-19)

| Witness | Result |
|---|---|
| `escaped_frame_vec_leak` (no arena) | 4555 MB WS / 4765 MB private, 3.2 s |
| `escaped_frame_arena_soak` (arena) | 51.7-53.3 MB WS / 82.7 MB private, 8.5-9.9 s |
| m368 3000 x 1M Float64 loop | 15.0 MB WS / 47.5 MB private, flat |

The arena is documented in `docs/SCOPED-ARENA.md`. The engine may copy the
`extern "C" { fn xiom_guard_heap_enter(); fn xiom_guard_heap_exit(); }`
pattern into its frame loop; keep every arena-allocated value inside the
scope. The official v0.64.7 archive predates the allocation routing.
