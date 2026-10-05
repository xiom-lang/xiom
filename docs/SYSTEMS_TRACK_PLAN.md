<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM Systems Track Plan -- `--freestanding` and bare-metal prerequisites

Status: QUEUED (owner request 2026-10-05; relayed from the stdlib lane's
`docs/PRODUCTION_READINESS_QUEUE.md` -> "Systems track"). Scope:
compiler/link/runtime work. Sequencing: after the current v0.64.0 bug batch,
before the selfhost Stage 6 push where possible; execution order follows the
five stdlib asks (1 freestanding, 2 repr(C), 3 volatile/atomics, 4 contract
disable, 5 SPIR-V later).

## Why

`xiom.ffi`'s header already names vulkan/imgui/glfw as consumers; `simd`
exposes 90 public functions; thread/sync + `AtomicInt`, the epoll/kqueue
async runtime, mmap/ioctl, and the guard-page/mprotect hot-reload machinery
already sit at OS/syscall level. Unlocking user-space drivers (and later
kernel/bare-metal) needs a no-CRT/no-libc link profile plus layout and
atomicity guarantees.

## What exists today (audited 2026-10-05)

- Ask (4) is effectively SATISFIED: `--no-contracts` already disables the
  runtime contract checks (main.rs), and the release profile defaults them
  off. Only a regression lock is missing.
- Targets: native, wasm, wasi, arm, riscv. No freestanding profile.
- No `#[repr(...)]` parsing today; struct layout comes from type_meta
  emission (declaration order in practice, no packing story or layout lock).
- The runtime C (`stdlib/runtime/xiom_runtime.c` + async/simd/sha/hot_reload)
  includes libc/OS headers across the board (stdio, stdlib, pthreads,
  sockets, dlfcn, mmap, signals, windows.h): freestanding is a runtime SPLIT,
  not just a linker flag.
- Direct externs of runtime symbols now work (m193), so a freestanding build
  can expose allocator/abort/clock hooks without duplicate-declare hazards.

## S1 -- `--target freestanding` (first unlock)

Compiler/link:
- `--target freestanding` (mutually exclusive with wasm/wasi/arm/riscv):
  clang gets `-ffreestanding -fno-builtin -nostdlib -nostartfiles`; optional
  `--linker-script <file>`; no CRT.
- Entry: the program supplies the entry symbol (default `_start`); do NOT
  emit `main(argc, argv)` or the `xiom_set_args` call; no argv seeding.
- Runtime selection: `find_runtime_c_files` gains a freestanding allow-list.
  Core `xiom_runtime.c` compiles with `-DXIOM_FREESTANDING`; every libc/OS
  helper (printf/puts/malloc/free/realloc, file, env, dl, TLS, signal, guard
  page, trampoline) compiles out. The program overrides weak hooks:
  `xiom_freestanding_alloc/realloc/free`, `xiom_freestanding_abort`,
  `xiom_freestanding_now_ms`.
- Semantic note: without the OS-backed guard page/trampoline, confined
  `unsafe` blocks cannot trap hardware faults in freestanding builds; the
  mode documents that unsafe = no confinement (or the checker warns).
- Linking a program that pulls an OS-backed runtime symbol fails with a
  clear diagnostic naming the symbol and the unsupported operation.

Acceptance:
- Build-only lock (all platforms): compile+link `--target freestanding` with
  `-nostdlib`; assert via `llvm-nm` that no libc/OS symbols remain undefined.
- Run lock (Linux CI): fixture with an asm `_start` that exits through a raw
  syscall runs and exits 0 (Windows cannot run without CRT/kernel32 imports;
  documented, build-only there).

## S2 -- `#[repr(C)]` / `#[repr(packed)]` + by-value ABI

- Parser: attributes on `type` declarations; checker validates
  `repr(C)`/`repr(packed)` (later `repr(transparent)`).
- Codegen: explicit C field order/alignment; `repr(packed)` removes interior
  padding; expose size/align/offset through reflect for tests.
- ABI: extern "C" by-value struct pass/return round-trip against a C helper.

Acceptance: e2e fixtures asserting `size_of`/`align_of`/field offsets for
`{i8, i64}`, packed `{u8, u32}`, and a nested struct, plus a C round-trip;
IR lock on the field GEP order.

## S3 -- volatile, fences, ordered atomics with CAS

- `volatile` loads/stores for mmio (pointer qualifier or
  `volatile_load`/`volatile_store` intrinsics lowering to
  `load volatile`/`store volatile`).
- `atomic.load/store/exchange/compare_exchange_weak|strong` on integer
  widths with orderings; `atomic.fence(...)` lowering to LLVM `fence`.
- Rebase the stdlib `AtomicInt` surface on these primitives.

Acceptance: IR locks (volatile/fence/`cmpxchg` present, orderings spelled)
plus a thread-stress e2e (CAS counter under N threads, no lost updates).

## S4 -- Contract-check disable

- `--no-contracts` and the release default already exist. Add an IR lock
  asserting no runtime contract call remains when checks are disabled.

## S5 -- SPIR-V device target (later)

- Memory-space/barrier types and `--target spirv` through clang; depends on
  S2/S3. Requires its own design doc (execution model, barriers, host/device
  ABI) before scheduling. Not before selfhost Stage 6.

## Documentation requirement (owner directive, 2026-10-05)

Every user-facing surface added by this track (`--target freestanding`,
`--linker-script`, `#[repr(C)]`/`#[repr(packed)]`, volatile/atomic
intrinsics, `--target spirv`) MUST ship its documentation in the SAME
commit, so the docs site can regenerate without a follow-up pass:
- compiler `AI_CONTEXT.md`: flags table + usage + semantics (mirror the
  existing `--no-contracts` / `--target` entries);
- website `docs/AI_CONTEXT.md` -- the source staged by
  `website/docs/build_mkdocs.py` into the MkDocs tree served at
  docs.xiom-lang.org -- plus the matching language-guide page under
  `website/docs/language/` for language-level features;
- `--help` text in the driver, and a `COMPILER_VERSIONS.md` note for the
  release that first carries the flag.
Acceptance for S1..S5 therefore adds: source docs updated + `xiom --help`
line + a website link/preview check when a new page is added.

## Sequencing and interactions

- S4's lock is trivial and can ride any batch.
- S1 + S2 are the stdlib "first unlock" pair (Vulkan-loader compute skeleton,
  mmio/volatile shims); S3 enables the deterministic SIMD/physics waves.
- Selfhost Phase 5+ does not touch target/link/runtime selection except
  through `find_runtime_c_files` (R65 owner): keep freestanding changes
  behind the profile so selfhost parity gates stay byte-stable.
- Stdlib delivers probe-first on each unlock; compiler landings ship with
  IR + e2e locks and CI lines per the standard protocol.
