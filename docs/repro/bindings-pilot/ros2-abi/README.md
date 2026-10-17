# ros2-abi -- Win64 C ABI aggregate calls + typed confined faults (B-33)

> Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
> SPDX-License-Identifier: MIT OR Apache-2.0

m351 (REL-20261016-2031-bindings-3). Found 2026-10-16 while building
`xiom.ros2` 0.2.0 (rcl's C API takes `rcl_allocator_t` BY VALUE).
Confirmed on v0.64.5/v0.64.6 (compiler main 9fb10c40).

The relay item claimed this bundle was already in the compiler repo; it was
not. m351 landed it here and parameterized the DLL path.

## Build

```powershell
clang -shared -O1 -o probe.dll probe.c     # LLVM clang, lld-link
$env:XIOM_M351_PROBE_DLL = (Resolve-Path .\probe.dll)
```

The probe binaries are **not committed**; rebuild from `probe.c`.

## Cases

| Fixture | Shape | Expected | Pre-m351 (v0.64.6) |
|---|---|---|---|
| `probe_ladder.xi` c1 | pointer to a 40-byte XIOM slot (`fn(*UInt8) -> Int` -> `probe_sum5p`) | `165` | PASS |
| `probe_ladder.xi` c2 | **16B record BY VALUE** (`fn(Pair2, Int) -> Int` -> `probe_sum2`) | `17036` | FAULT, silent exit 0 |
| `probe_ladder.xi` c3 | **16B record RETURN** (`fn(Int) -> Pair2` -> `probe_make2`) | `507,509` | FAULT, silent exit 0 |
| `probe_ladder.xi` c4 | 40B record RETURN (`fn(Int) -> Alloc5` -> `probe_make5`) | `1001,1005` | PASS (both sret) |
| `probe_ladder.xi` c5 | **40B record BY VALUE** (`fn(Alloc5) -> Int` -> `probe_sum5`) | `165` | FAULT, silent exit 0 |
| `probe_ladder2.xi` | c3/c4/c5 without c2 | exact | FAULT |
| `probe_small.xi` c6 | 8B record BY VALUE (`{i32,i32}`) | `17036` | silent 0 / wrong (`17007`) |
| `probe_small.xi` c7 | 8B record RETURN (`{i32,i32}`) | `507,509` | garbage / silent 0 |
| `probe_c2_plain.xi` | c2 **without** `unsafe` | `17036` | hard AV `0xC0000005` |
| `probe_pretend.xi` p1/p2 | pointer-pretend workaround | `165` / `17036` | PASS |
| `probe_multi.xi` m1/m2/m3 | 9/7/3-arg rcl shapes via pointer-pretend | `8186` / `2166` / `969` | PASS |
| `probe_extern_direct.xi` | direct `extern "C"` 16B by value (`--c-source probe.c`) | `17036` | naked declare -> silent exit 0 / AV |
| `probe_extern_direct_unsafe.xi` | same inside `unsafe` | `17036`, exit 0, no fault | silent exit 0 |
| `probe_extern_make2.xi` | direct `extern "C"` 16B return | `507,509` | silent exit 0 |

Run:

```powershell
$env:XIOM_STDLIB = "E:\xiom-lang\xiom\stdlib"
$env:XIOM_M351_PROBE_DLL = "C:\path\to\probe.dll"
xiom --run probe_ladder.xi
xiom --run probe_ladder2.xi
xiom --run probe_small.xi
xiom --run probe_c2_plain.xi
xiom --run probe_pretend.xi
xiom --run probe_multi.xi
xiom --run probe_extern_direct.xi --c-source probe.c
xiom --run probe_extern_direct_unsafe.xi --c-source probe.c
xiom --run probe_extern_make2.xi --c-source probe.c
```

## Mechanism notes

- MS x64 passes aggregates of size 1/2/4/8 bytes as integers and every other
  aggregate **by reference to a caller copy**; returns larger than 8 bytes use
  a hidden first `sret` pointer. XIOM used to hand LLVM a naked `%struct.X`,
  whose lowering differs (16B in `rcx:rdx`, 16B return in `rax:rdx`, and
  8-byte aggregates split into field registers), so the C callee
  dereferenced a scalar as a pointer / read the wrong bytes.
- m351 adds a target-dispatched `CAbiPlan` applied at extern declares/calls
  and at raw `Int as fn(..)` cast calls: 1/2/4/8-byte aggregates are
  integer-coerced (raw bytes as `i8`/`i16`/`i32`/`i64`), larger by-value
  aggregates pass by pointer, returns follow the same classes (with `sret`
  above 8 bytes). XIOM-internal fn values keep the internal convention.
  System V / AArch64 are deliberately unchanged until validated per target.
- The confined fault path no longer returns a zero from the enclosing
  function: it manifests a typed TLS fault (always-on `[fault]` diagnostic)
  and continues with a defined fallback. An unacknowledged fault exits 70;
  `xiom.error.take_fault()` acknowledges it (the only exit-0 path).
- The pointer-pretend channel remains ABI-correct by construction where
  aggregate arguments are passed by reference to a caller copy (MS x64,
  AAPCS64). On x86-64 System V such aggregates are MEMORY-class and
  `xiom.ros2` refuses those hosts typed (`ROS2_ABI_HOST`).
