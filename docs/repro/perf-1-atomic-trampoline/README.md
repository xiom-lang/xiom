<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# PERF-1 repro: unsafe-block trampoline overhead (AtomicInt load/store)

Isolation of the benchmark lane's t2-queue finding (see
`docs/STAGE6_PERF_PLAN.md` PERF-1 and `docs/COMPILER-RELAY-2026-09-29.md`
in the benchmark workspace).

## Run

```
target/debug/xiom.exe -o perf_atomic.exe perf_atomic_trampoline.xi
./perf_atomic.exe
```

## Local results (2026-09-29, v0.62.1, this box)

```
plain:  0        (ms; sub-millisecond for 4M increments)
atomic: 8000     (ms; 4M atomic store+load pairs)
x:      4000000
```

- Plain 4M-iteration loop: below the 1 ms clock resolution.
- 4M atomic pairs (8M calls): **8000 ms -> ~1.0 us per call**.
- Benchmark container measured the same shape at 18.5 s / ~2.4 us per call,
  so the effect reproduces at the same order of magnitude.

## IR evidence (per CALL, not per block)

Every stdlib atomic wrapper wraps its intrinsic in its own unsafe block, and
the D2.1 confinement pays arm/trampoline/disarm for EACH call. From
`xiom --emit-ir perf_atomic_trampoline.xi`:

```
define i64 @sync.atomic_load(%struct.AtomicInt* %param0) alwaysinline {
  ...
  %tmp11 = ptrtoint ptr @__unsafe_block_16 to i64
  call void @xiom_trampoline_set_allow_retry(i64 1)
  %tmp12 = call i64 @xiom_trampoline_call(i64 %tmp11, i8* %tmp10)

define i64 @__unsafe_block_16(i8* %ctx_raw) {
  ...
  call void @xiom_guard_heap_enter()
  call void @xiom_guard_page_arm()
  %tmp16003 = call i64 @xiom_atomic_load(i64* %tmp16002)
  call void @xiom_trampoline_set_returned()
  call void @xiom_guard_heap_exit()
  call void @xiom_guard_page_disarm()
  call void @xiom_trap_leave()
  ...
}
```

The compiled closure contains 25 `xiom_trampoline_call` sites; the atomic
load and store wrappers alone account for the hot loop (they are
`alwaysinline` into the caller), each with its own `__unsafe_block_N`
function carrying the guard-page arm/disarm pair.

## Interpretation

`xiom.sync.atomics` wrappers (`atomic_load` / `atomic_store`) each contain
`unsafe { xiom_atomic_load/store(...) }`; the confinement cost is charged per
atomic operation. A t2-queue implemented with atomics therefore pays ~1-2.4 us
per operation instead of a few nanoseconds.

Candidate fixes (design + safety-lane sign-off) are listed in
`docs/STAGE6_PERF_PLAN.md` PERF-1: block-level arming, callee classification
(trampoline only true FFI boundaries), a no-alloc/no-fault fast path, and
inlining the atomic intrinsics.
