<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM -- Unsafe Confinement Architecture (Production Plan)

**Status:** PLAN ONLY -- no implementation. This document is the design contract for the
Unsafe Confinement system (items a-i). Implementation begins after approval and follows the
phases in S6.
**Target:** v0.57.0 (PRE-SELFHOST -- must land before the self-hosting bootstrap begins)
**Branch:** `feat/architect`
**Owner:** Language team (Architect + Compiler hardening + Runtime)

---

## 0. EXECUTIVE SUMMARY

XIOM's `unsafe { }` blocks are currently a **lexical gate only** (D2, committed 2026-08-08):
raw-pointer deref/casts and `asm` are rejected outside `unsafe`, but once inside a block the
program executes with **no isolation, no fault trapping, and no recovery**. A bug inside an
`unsafe` block can corrupt the process heap, smash the stack, or crash the whole service.

This plan upgrades `unsafe` from a *permission gate* to a **confinement container**:

| Requirement | What it delivers |
|---|---|
| (a) Lexical Confinement | `*T` / `asm!` / `extern "C"` hard-error outside `unsafe { }` |
| (b) Block-Scoped Only | `unsafe` is a lexical block; never fn/module/struct-wide |
| (c) Pre-Entry Contracts | every `unsafe` block is wrapped by a safe fn enforcing `requires`/`ensures` |
| (d) Heap Isolation | allocations inside the block use a dedicated guard heap |
| (e) Stack Guard Pages | red-zone pages below the block's stack frame (hardware overflow catch) |
| (f) Hardware Fault Trapping | sigsetjmp/longjmp checkpoint; SIGSEGV/SIGILL/SIGFPE intercepted |
| (g) Recoverable Errors | traps become `Err(HardwareFault)` / `Err(ContractViolation)` |
| (h) Transient Fault Retry | one retry on a fresh memory slot before permanent error |
| (i) Zero Escape | raw pointers/registers/FFI handles never cross the block boundary |

**Design principle:** `unsafe { }` becomes a *sandboxed transaction*: validate inputs
(c), run in isolated memory (d, e), trap faults (f, g), retry once (h), and return only safe
values (i). Safe code is never exposed to the failure modes of unsafe code.

---

## 1. CURRENT STATE (grounded audit, 2026-08-10)

### 1.1 What exists today (D2, commit `83416aa9`)

- `Checker.unsafe_depth: u32` (xiom-check/src/lib.rs:117) -- incremented on `Expr::Unsafe`,
  decremented on exit.
- `gate_unsafe(op, span)` (xiom-check/src/lib.rs:658) -- emits a hard T001 error for:
  - raw-pointer deref (`*p`) at depth 0,
  - `Int<->Ptr`, `Ptr->Ptr`, `Vec/Str->Ptr`, `fn<->Int/ptr` casts at depth 0,
  - `Stmt::Asm` at depth 0 (xiom-check/src/lib.rs:2986).
- `&x` -> `*T` ref-coercion stays safe (FFI borrow pattern).
- `unsafe { }` compiles **transparently** in codegen (expr.rs ~3806): the block is an
  expression whose tail value flows out normally. No isolation, no trapping, no checks.

### 1.2 Gap analysis vs requirements

| Req | Status | Gap |
|-----|--------|-----|
| (a) Lexical Confinement | **PARTIAL** | deref/casts/asm gated. `extern "C"` CALLS are NOT gated -- an `extern` callable can be invoked from safe code. `asm!` macro form doesn't exist (only `asm("...")`). |
| (b) Block-Scoped Only | **PARTIAL** | syntax is block-only already, but nothing *enforces* that a fn body consisting solely of `unsafe { ... }` is treated differently, and `unsafe fn` doesn't exist (good) -- but nothing stops `unsafe` from being the whole body and effectively fn-wide. |
| (c) Pre-Entry Contracts | **MISSING** | no rule requires a safe wrapper with `requires`/`ensures` around each `unsafe` block; no checker enforcement; no contract injection into the block prologue. |
| (d) Heap Isolation | **MISSING** | `malloc`/`realloc` inside unsafe blocks use the process heap (the same `xiom_runtime.c` allocator). |
| (e) Stack Guard Pages | **MISSING** | no guard/red-zone pages; stack overflow is a hard process crash (0xC00000FD). |
| (f) Hardware Fault Trapping | **MISSING** | no sigsetjmp/longjmp or SEH wrapping; SIGSEGV/SIGILL/SIGFPE terminate the process. |
| (g) Recoverable Errors | **MISSING** | no `Err(HardwareFault)`/`Err(ContractViolation)` surface. |
| (h) Transient Fault Retry | **MISSING** | no retry machinery, no fresh-slot reallocation. |
| (i) Zero Escape | **MISSING** | raw pointers can be returned from `unsafe { }` as the tail value (e.g. `unsafe { buf }` returns the i8* as a Str-typed value). |

### 1.3 Key constraints discovered

- **Windows first, POSIX second.** The project's primary target is `x86_64-pc-windows-msvc`
  (clang + lld-link). Windows has **no `sigsetjmp`/`longjmp`**; fault trapping must use
  **Structured Exception Handling (SEH)** via `__try/__except` or a `VectorHandler`-based
  `AddVectoredExceptionHandler`. POSIX (Linux/macOS) uses `sigsetjmp/siglongjmp` +
  `sigaction`. The plan must define a **portable trap abstraction** (`xiom_trap_enter/exit`).
- **Stack guard pages on Windows:** `VirtualAlloc` with `PAGE_GUARD` -- one-shot (auto-clears
  on first access), must be re-armed; or reserve + commit with `PAGE_NOACCESS` below the
  frame and catch the fault via the vector handler.
- **clang lowering:** `call void asm sideeffect` is opaque to the trap wrapper -- the wrapper
  is per-`unsafe`-block, emitted by codegen around the block's IR.
- **The stdlib itself uses `unsafe` heavily** (37+ blocks in `alloc.xi`, `collections.xi`,
  `ptr.xi`, `encoding.xi`, `ffi.xi`, ...). Confinement must NOT slow the hot paths
  (Vec push/alloc) -- a **perf budget** must be defined (see S7).
- **Recursion counter** (`@xiom_recursion_counter`, thread_local) interacts with longjmp:
  a fault unwinding past the block must still leave the counter balanced -- the trap wrapper
  must restore it (or the block's counter delta must be captured and restored on fault).

---

## 2. DESIGN

### 2.1 The `unsafe` block as a confined transaction

Every `unsafe { ... }` block compiles to a **transaction** with this shape:

```xiom
// source
fn f(x: Int) -> Result[Int, HardwareFault] {
  requires: x >= 0
  ensures: result is Ok => result.value >= 0
  unsafe {                       // block boundary -> transaction
    var p: *Int = ...;
    *p = x;                      // confined
    ...                          // may fault; may allocate on guard heap
    return *p;                   // safe value crosses out (i)
  }
}
```

Compiled (conceptually):

```
safe wrapper fn:
  check requires(x)                          // (c) pre-entry contract -- hard fail -> Err(ContractViolation)
  pack ctx { x, ...captured locals }          // free vars of the unsafe block
  result = xiom_trampoline_call(&__unsafe_block_N, &ctx)   // (f) pre-compiled CRT trampoline
  match result {
    Ok(v)  => { copy-out v to main heap if heap-backed (S2.11); ensure checks; return v }
    Err(f) => return Err(HardwareFault{...}) // (g) recoverable
  }

__unsafe_block_N(ctx):                        // standalone fn, NOT alwaysinline
  guard_heap = xiom_guard_heap_select(ctx)    // (d) isolate heap context (TLS arena)
  arm stack guard page                        // (e) red zone
  ... user statements (allocations -> guard heap) ...
  disarm guard page
  return tail value (safe type only, S2.10/2.11)

trampoline (pre-compiled, re-entrant, in CRT):
  checkpoint (SEH __try / sigsetjmp) -> call block fn -> on fault: restore recursion
  counter, unwind only trampoline frame, return fault code; retry once (h) with fresh
  arena + re-armed guard page.
```

### 2.2 (a) Lexical Confinement -- hard errors

- Extend the D2 gate:
  - **`extern "C"` calls from safe code -> hard error.** Only `unsafe { }` may call an
    `extern` function. The checker tracks `extern` fn names (already registered at
    TopDecl::Extern, xiom-check:1461) and gates `Expr::Call` targets that resolve to one.
  - **`asm` -> hard error outside `unsafe`** (already done). Add the `asm!` macro alias.
  - **`*T`/`*mut T`/`*const T` type annotations in safe signatures -> hard error** (a safe
    fn may not *name* a raw pointer type in its signature; pointer types only appear as
    locals inside the block, or in `unsafe`-internal helper signatures that are themselves
    unreachable from safe code).
- Error codes: extend T001 family -- `T002` (extern call outside unsafe), `T003` (raw
  pointer type in safe signature), `T004` (asm outside unsafe -- keep existing message).

### 2.3 (b) Block-Scoped Only -- enforcement

- Parser/checker rule: `unsafe` applies strictly to the brace block `{ }`. Reject:
  - `unsafe fn`, `unsafe module`, `unsafe struct`, `unsafe impl` (parse errors).
- **Whole-body `unsafe`:** if a safe fn's body is *entirely* one `unsafe { }` block, the
  checker requires the fn to carry `requires`/`ensures` contracts (this is the (c) wrapper
  rule -- see 2.4). This prevents "unsafe by delegation" without a contract.

### 2.4 (c) Pre-Entry Contracts

- **Checker rule:** every `unsafe { }` block must be lexically nested inside a fn that
  declares at least one `requires` contract. (Rationale: the wrapper validates *all inputs*
  before the block runs; a block with no wrapper has no validation story.)
- **Codegen injection:** before the block's first instruction, emit the `requires`
  checks (already supported: contracts compile to `br` + `@llvm.trap`-style guards today --
  change the trap target to the *handler* instead of process abort). If a check fails:
  return `Err(ContractViolation{ contract, value })` from the wrapper (not a crash).
- **Post-exit `ensures`:** the tail value is checked against the wrapper's `ensures`
  clauses inside the trap wrapper's success path; failure -> `Err(ContractViolation)`.

### 2.5 (d) Heap Isolation -- Guard Heap

- New runtime: **`xiom_guard_heap_*`** -- a dedicated arena allocator:
  - `xiom_guard_heap_select()` -- switch the block's `malloc`/`realloc`/`free` to the guard
    arena (per-thread pointer swap; the block's `malloc` calls route to arena-allocated
    slabs).
  - `xiom_guard_heap_reset(arena)` -- free all slabs on block exit (or on fault retry).
  - Isolation: the arena's slabs come from dedicated `VirtualAlloc`/`mmap` regions that are
    **never handed to the main process heap**. Corruption inside the block cannot
    contaminate application memory; on block exit the arena is discarded wholesale.
  - FFI calls inside the block that allocate on the C heap (e.g. a lib that mallocs)
    cannot be intercepted -- documented limitation; the (f) trap still bounds the damage to
    the block's execution window (see S7 risks).

### 2.6 (e) Stack Guard Pages

- Per-block frame setup: reserve a red zone below the block's stack frame using
  `VirtualAlloc(PAGE_GUARD)` (Windows) / `mmap(PROT_NONE)` (POSIX).
- On overflow, the hardware raises the fault *at the guard page* -- before adjacent
  memory is written -- and the (f) handler catches it.
- Windows `PAGE_GUARD` is one-shot: the handler re-arms it before the retry (h).
- Cost: one guard page per concurrently-active unsafe block (reuse via a per-thread pool of
  guard pages -- `xiom_guard_page_pool`).

### 2.7 (f) Hardware Fault Trapping -- the CRT Trampoline (REVISED 2026-08-10)

**Design decision (adopted from review):** do NOT wrap the unsafe block inline in the
LLVM IR with sigsetjmp/longjmp. `longjmp` unwinds C stack frames, not LLVM IR frames --
unwinding out of an IR block with live PHI nodes / pending state corrupts the register
allocator. Instead:

1. **Codegen lowers every unsafe block to a standalone callable function** (not
   alwaysinline): `int64_t __unsafe_block_N(uint8_t* ctx)` -- a fixed ABI, with the
   block's FREE VARIABLES packed into a context struct by the codegen (the same
   capture machinery XIOM's closures (M20) already provide). The call site becomes:
   `xiom_trampoline_call(block_fn, ctx) -> Result[T, HardwareFault]`.
2. **The runtime ships ONE pre-compiled, re-entrant, thread-safe Trampoline** (written
   in C/ASM, platform-gated, compiled into `xiom_runtime.c` / a small `.asm` object --
   never generated at runtime, immune to LLVM upgrades):
   ```c
   // Windows: SEH __try/__except (clang supports it); POSIX: sigsetjmp/siglongjmp.
   int64_t xiom_trampoline_call(
       int64_t (*block_fn)(uint8_t*),   // the unsafe block's function
       uint8_t* ctx)                    // captured-variable context
   {
       int64_t result;
       int fault = xiom_trap_enter();           // checkpoint (SEH __try / sigsetjmp)
       if (!fault) {
           result = block_fn(ctx);              // run the confined block
           xiom_trap_leave();                   // disarm
           return result;
       }
       return XIOM_FAULT_BASE + fault;          // unwound here -- only the trampoline's
   }                                            // own frame + block frame are touched
   ```
   The faulting unwinding touches ONLY the trampoline's frame and the block fn's frame --
   the caller's IR stack stays balanced and intact.
3. **Platform fault registration** (once per process):
   - **Windows:** `AddVectoredExceptionHandler`; the handler checks whether the faulting
     PC is inside a confined block (a registered block-range table). If yes -> unwind to
     the trampoline checkpoint; if no -> chain to the previous handler (non-unsafe faults
     still crash loudly, as today).
   - **POSIX:** `sigaction(SIGSEGV|SIGILL|SIGFPE, handler, SA_SIGINFO)`; handler calls
     `siglongjmp` to the trampoline checkpoint.
4. **The recursion counter** (`@xiom_recursion_counter`, thread-local) is saved/restored
   by the trampoline around the block call -- on fault, the saved value is restored
   before returning the error (S1.3).
5. **Retry (h)** re-enters the trampoline: re-arm the guard page, re-select a fresh
   arena, and call `block_fn(ctx)` again.

**Why this scales:** one function, no global state, per-thread TLS arenas -> 128 parallel
threads call the same trampoline without blocking each other; faults in thread A unwind
only thread A's stack.

**Codegen consequence (the core Phase 5 work item):** the unsafe-block-as-function
lowering is the largest single piece -- block tail -> `Result[T, HardwareFault]` wrapper,
free-variable capture into the ctx struct, and a trampoline call at the block site.
Reuses closure capture (M20) but must handle: nested unsafe blocks (one trampoline per
OUTERMOST block -- S2.13), `defer` inside the block (must not be skipped silently by an
unwind -- run deferred cleanups in the block fn's normal exit; on fault, run them in the
trampoline's error path before returning), and non-inlined emission (block fns must NOT
be alwaysinline).

### 2.8 (g) Recoverable Errors

- New stdlib types (stdlib/xiom/error.xi -- additive, freeze-gate safe):
  ```xiom
  pub type HardwareFault = { signal: Str; pc: UInt64; retried: Bool; }
  pub type ContractViolation = { contract: Str; }
  ```
- The wrapper's return: `Err(HardwareFault{...})` or `Err(ContractViolation{...})`. Callers
  `match` and continue -- the process never terminates due to an unsafe-block fault.

### 2.9 (h) Transient Fault Retry

- On a caught fault, the handler:
  1. logs the fault (signal, pc, block id),
  2. resets the guard heap arena (`xiom_guard_heap_reset`),
  3. re-arms the stack guard page,
  4. restores the recursion counter delta,
  5. re-executes the block body **once** on a fresh memory slot,
  6. if it faults again -> permanent `Err(HardwareFault{ retried: true })`.
- Retry is opt-out per block via an attribute: `#[unsafe_no_retry]` (some faults are not
  transient by nature -- e.g. a guaranteed bad deref would just fault twice).

### 2.10 (i) Zero Escape -- REVISED 2026-08-10 (UAF fix)

- Checker rule: the tail value of an `unsafe` block must have a **safe type** -- reject
  `*T`, `Ptr`, `fn`-typed, or struct values containing raw pointers as the block's result.
  Raw pointers may be *used* inside but never *named* as the block's value type.
- **Strengthened (review): `&T` references are ALSO rejected as tail types.** A borrow
  into arena memory cannot be copy-out'd safely, and the caller could hold it past the
  arena reset. Allowed tail types: value types (`Int/Float/Bool/Char/Int128/...`) and
  heap-backed owned types (`Vec[T]`, `Str`, `Box[T]`, `Option`/`Result` of those).
- Codegen: FFI handles returned from `extern` calls inside the block are wrapped at the
  boundary -- see S2.12.
- Register state (from `asm`) never crosses: asm outputs are confined to locals inside the
  block.

### 2.11 Copy-Out / Promotion Semantics (i) -- THE UAF FIX (REVISED 2026-08-10)

**The conflict the review caught:** a `Vec[Int]` allocated on the GUARD arena and returned
as a "safe type" would dangle when (d) resets the arena -- a use-after-free that bypasses
zero-escape because `Vec` is a safe type.

**Rule: the tail value is PROMOTED to the main process heap before the arena is discarded.**

- **v1 (Phase 3): Copy-Out -- simple and correct.** The codegen emits a deep copy of the
  tail value's heap payload from the guard arena to the main heap before the arena resets:
  - `Str` -> `memcpy` the bytes into a main-heap allocation.
  - `Vec[T]` -> allocate a new main-heap buffer, `memcpy` the elements, swap the data
    pointer into the returned Vec.
  - `Option/Result` wrapping either -> copy the payload.
  - **Cost is bounded: exactly ONE copy per unsafe block** (zero-escape means only the
    tail crosses; intermediate allocations inside the block are discarded with the
    arena, never copied). A 1M-item `Vec.push` loop inside the block copies nothing --
    only the final returned buffer is copied once at exit.
- **Phase 7+ optimization: Promotion (zero-copy).** Instead of copying, the arena page(s)
  backing the tail value are ADOPTED into the main heap (O(1) ownership transfer): the
  Vec/Str's allocator tag is switched to the main heap, and when the value is later
  dropped it frees the page normally. Only enabled once per-value; requires the arena
  allocator and the main allocator to share a page-tracking layer. Copy-Out remains the
  correctness baseline; Promotion is an optimization on top.
- **Large surviving allocations:** `#[heap = "main"]` on an allocation inside the block
  routes that specific allocation to the main heap directly (bypassing the guard arena) --
  the escape hatch for values too large to copy (e.g. a 10MB buffer that must survive).
  Documented limitation: main-heap allocations inside the block are fault-bounded but NOT
  arena-isolated.

### 2.12 FFI Heap Bypass -- pointer ownership rule (REVISED 2026-08-10)

The plan already documented that `extern` calls allocating on the C heap bypass the guard
arena (fault-bounded, not isolated). The review's addition -- a STRICT checker rule -- is
adopted:

- **Checker rule (T006):** inside a confined block, an `extern` call whose return type is
  a raw pointer (`*T`) must have its result converted to an OWNED XIOM type **before the
  block's tail is evaluated**. Allowed conversions (stdlib `ffi` helpers):
  - `ffi.box_from_ptr[T](p, drop_fn)` -- take ownership with a registered destructor,
  - `ffi.vec_from_ptr_with_free[T](p, len, cap, free_fn)` -- adopt a C-allocated buffer,
  - `ffi.str_from_ptr_owned(p)` -- take ownership of a C string.
- Rationale: a `*T` returned by libc memory (malloc'd on the C heap) must be freed by
  `free`, not the guard arena -- without the conversion rule the pointer would be leaked
  (the arena doesn't own it) or double-freed (if the caller tries to drop it as arena
  memory).
- Enforcement: the checker tracks the result type of every `extern` call inside the block;
  a raw-pointer-typed value that reaches the tail (directly, or nested in a struct/vec)
  without passing through a registered conversion fn -> hard error T006.
- Conversions are themselves confined: they run inside the block and may fault (bounded).

### 2.13 Transaction Boundary & `#[unsafe_direct]` (REVISED 2026-08-10)

**Transaction boundary = the OUTERMOST unsafe block.** Nested `unsafe { ... unsafe { ... } }`
inside an already-active transaction do NOT create a second trampoline/arena/guard page --
the outer transaction covers them. This is both a semantic rule (one checkpoint per
transaction) and the performance batching rule:

- **Perf (review):** a 1M-iteration loop with a per-iteration unsafe block would pay
  ~200-300ns x 1M ~= 200ms. Rule: hot stdlib paths (e.g. `Vec.push` resize logic) wrap the
  WHOLE loop/resize in one unsafe block, not per-operation. Phase 7 audits all 37 stdlib
  sites for transaction granularity.
- **Measured overhead budget per transaction:** arm/disarm guard page ~100ns + trampoline
  checkpoint ~50ns + arena select ~20ns ~= **200-300ns per transaction** (not per
  operation). With batching, <=1.5x on hot paths is achievable.

**`#[unsafe_direct]` -- the trusted-escape hatch (governance):**
- Marks a confined block as TRUSTED: no trampoline, no guard page, no arena -- runs as
  today's plain unsafe block. Intended for: self-host compiler internals (FFI bindings,
  JIT memory mapper), stdlib hot paths, and other audited trusted code.
- **Governance (my addition):** `#[unsafe_direct]` is RESTRICTED to stdlib/trusted
  packages by default. User code cannot tag blocks `#[unsafe_direct]` unless the compiler
  is invoked with `--enable-unsafe-direct` (and a counted, audited cap -- the compiler
  reports the number of direct blocks). Without this gate, user code could bypass the
  entire confinement story.
- The self-host compiler uses `#[unsafe_direct]` ONLY in its FFI/JIT blocks; its lexer,
  parser, and checker are 100% safe (zero unsafe) and run at native speed with zero
  confinement overhead.

---

## 3. IMPLEMENTATION SURFACE (files touched, per phase)

| Area | Files |
|------|-------|
| Checker gates (a, b, c, i) | `crates/xiom-check/src/lib.rs` (gate_unsafe, check_call extern gate, tail-type check, contract-wrapper rule) |
| Parser (b, a) | `crates/xiom-parser/src/lib.rs` (reject `unsafe fn` etc.; `asm!` alias) |
| AST (b) | `crates/xiom-ast/src/lib.rs` (no new node needed -- block already `Expr::Unsafe`) |
| Codegen wrap (f, d, e, g) | `crates/xiom-codegen/src/expr.rs` (Unsafe arm -> transaction lowering), `decl.rs` (wrapper return type), `emitter.rs` (declare new runtime fns) |
| Runtime (d, e, f, g, h) | `stdlib/runtime/xiom_runtime.c` (guard heap, guard pages, trap enter/leave/retry, SEH/POSIX) |
| Stdlib types (g) | `stdlib/xiom/error.xi` (HardwareFault, ContractViolation), `stdlib/xiom/unsafe.xi` (block attributes, retry policy) |
| Tests | `crates/xiom-check` (gate unit tests), `crates/xiom-codegen/tests/stdlib_execution_tests.rs` (confinement smokes), `tests/regression/` (m34/m35 unsafe suites extended) |
| Docs | `docs/SAFETY_HARDENING.md`, `docs/STDLIB_EXTENSION.md` S13 D2, `docs/ROADMAP.md` S4 |

---

## 4. PHASES

Each phase is independently verifiable; the suite must stay green at every phase boundary.

### Phase 1 -- Confinement Gates (a, b, i-checker)
- (a) `extern "C"` call gate: T002 hard error in safe code; `asm!` alias.
- (a) raw-pointer type names in safe signatures: T003.
- (b) parse rejects `unsafe fn/module/struct/impl`.
- (i) checker rejects raw-pointer/`fn`-typed tails from `unsafe` blocks (T005);
  **`&T` tails also rejected (review)**.
- (i) **FFI pointer ownership rule: T006 (review)** -- `extern` returning `*T` inside a
  confined block must convert to an owned XIOM type before the tail (S2.12).
- **Verify:** checker unit tests (est. +16), full suite green. No runtime change.

### Phase 2 -- Contracts & Wrapper Rule (c)
- Checker: every `unsafe` block is inside a fn with >=1 `requires`; whole-body-unsafe fns
  must declare `requires`+`ensures`.
- Codegen: requires-checks fail -> `Err(ContractViolation)` instead of process trap;
  ensures-check on the promoted tail (S2.11) after copy-out.
- **Verify:** contract smokes (safe wrapper + violation path), suite green.

### Phase 3 -- Guard Heap (d) + Copy-Out (i)
- Runtime `xiom_guard_heap_*` (arena slabs, per-thread TLS context switch).
- Codegen: inside `unsafe` blocks, `malloc`/`realloc`/`free` calls route to the guard
  arena; reset on block exit.
- **Copy-Out (review/UAF fix):** codegen emits a deep copy of the tail value's heap
  payload to the main heap before the arena resets (S2.11) -- one copy per block, bounded.
  `#[heap = "main"]` attribute routes specific allocations to the main heap.
- **Verify:** heap-isolation smoke (corrupt a pointer inside the block -> main heap
  untouched, arena discarded); **UAF smoke: return a Vec/Str from a confined block,
  use it after the block, assert intact (copy-out worked); mutate the arena after and
  assert the returned value is unaffected**; ASAN run green.

### Phase 4 -- Stack Guard Pages (e)
- Runtime guard-page alloc/arm/disarm + per-thread pool.
- Codegen: page armed at block entry, disarmed at exit.
- **Verify:** deliberate stack-overflow-in-unsafe smoke -> fault caught (Phase 5), process
  survives.

### Phase 5 -- Hardware Fault Trapping + Recovery (f, g) -- Trampoline (REVISED)
- **Codegen (core work item): unsafe-block-as-function lowering** -- standalone
  `int64_t __unsafe_block_N(uint8_t* ctx)` per OUTERMOST block, free-variable capture into
  a ctx struct (reuse closure capture M20), block fns NOT alwaysinline, `defer` inside the
  block run on both normal exit and the trampoline's fault path (S2.7).
- Runtime: pre-compiled re-entrant **Trampoline** in the CRT (`xiom_trampoline_call`) --
  SEH `__try/__except` on Windows, `sigsetjmp/siglongjmp` on POSIX; one VEH/`sigaction`
  registration per process with a block-range table; recursion counter saved/restored
  around the block call.
- Codegen: block site -> `xiom_trampoline_call(block_fn, &ctx)` -> `Result[T, HardwareFault]`
  wrapper; handler returns `Err(HardwareFault{ signal, pc, retried })`.
- **Fault-injection test strategy (my addition):** add a test-only `extern` helper that
  writes to a known-bad address / executes `ud2` / divides by zero, called INSIDE a
  confined block; verify the block returns Err and the process continues (print after).
- **Verify:** SIGSEGV/SIGILL/SIGFPE smokes on both platforms; suite green; faulting block
  returns Err, caller continues.

### Phase 6 -- Transient Retry (h)
- Retry-once machinery via the trampoline (re-arm guard page, fresh arena, re-call
  `block_fn(ctx)`) + `#[unsafe_no_retry]`.
- **Verify:** transient-fault smoke (first run faults, retry succeeds, `retried: true`
  path); permanent-fault smoke (faults twice -> Err).

### Phase 7 -- Stdlib Adoption & Perf Budget
- Migrate the 37+ stdlib unsafe sites to the confined form (wrappers + contracts).
- **Transaction batching (review):** audit every site for transaction granularity -- hot
  paths (Vec.push resize, alloc) wrap the WHOLE loop/resize in one unsafe block, not
  per-operation. Budget: ~200-300ns per TRANSACTION; <=1.5x on hot paths.
- **`#[unsafe_direct]` (review + governance):** trusted sites (stdlib hot paths) use the
  escape hatch with a counted, audited cap; user code requires `--enable-unsafe-direct`.
- **Verify:** stdlib-exec full suite, e2e full suite, benchmark-chaos comparative run.

### Phase 8 -- Self-Host Gate
- All unsafe confinement gates active when the compiler compiles itself.
- **Verify:** full `cargo test`, selfhost bootstrap smoke, docs updated.

---

## 5. ROADMAP INTEGRATION

Added to `docs/ROADMAP.md` S4 (PRE-SELFHOST ROADMAP) as **v0.57.0 "Unsafe Confinement"**,
after v0.56.0-pre and BEFORE the SELFHOST bootstrap:

| Milestone | Version | Scope | Depends on |
|---|---|---|---|
| Unsafe Confinement | **v0.57.0** | Phases 1-8 above | v0.56.0-pre (LTO, debug info, hardening) |
| Selfhost | SELFHOST | XIOM compiles itself | v0.57.0 (confinement gates must be live) |

Also referenced from: `docs/SAFETY_HARDENING.md` (new S"Unsafe Confinement"), and
`docs/STDLIB_EXTENSION.md` S13 (D2 extended -> "D2.1 Confined Unsafe").

---

## 6. RISKS & OPEN QUESTIONS

1. **Windows SEH vs LLVM IR -- RESOLVED by the Trampoline (review).** The trap wrapper no
   longer lives in LLVM IR: the unsafe block is a standalone function called through a
   pre-compiled C trampoline where `__try/__except` (Windows) / `sigsetjmp` (POSIX) is
   valid C. The remaining validation is the trampoline's VEH/`sigaction` handler table
   (block-range check) -- prototype early in Phase 5.
2. **`longjmp` across LLVM frames -- RESOLVED by the Trampoline (review).** Unwinding
   touches only the trampoline's frame and the block fn's frame; the caller's IR stack is
   never unwound. **Remaining risk:** `defer` inside the block must run on both the normal
   exit AND the fault path (the trampoline's error return runs the block's deferred
   cleanups before returning the fault code) -- Phase 5 must verify no `defer` is skipped.
3. **Guard-heap interception coverage.** `extern` calls that allocate on the C heap
   bypass the guard arena. Mitigation: (f) trap bounds the window; **T006 checker rule
   (S2.12)** forces pointer ownership conversion before the tail; document that FFI
   allocations are *not* isolated, only *fault-bounded* and *ownership-checked*.
4. **Performance.** Guard-page arm/disarm + trampoline checkpoint + arena select ~=
   200-300ns per TRANSACTION (outermost block). Perf budget 1.5x requires **transaction
   batching** (whole-loop unsafe blocks, not per-operation); trusted hot paths use the
   audited, capped `#[unsafe_direct]`.
5. **Recursion counter balance across the trampoline fault path** (S1.3) -- the trampoline
   saves/restores the thread-local counter around the block call; a leak would trip the
   500-depth guard spuriously.
6. **Retry semantics.** Some faults (e.g. writing to a read-only page) are deterministic --
   retry is pointless. The `#[unsafe_no_retry]` attribute covers these; default = retry
   once (transient faults like first-touch guard pages benefit).
7. **Tail-value zero-escape vs existing stdlib.** Several stdlib fns currently *return*
   raw-pointer-backed values (e.g. `Str.from_cstring` returns i8*). The (i) rule treats
   `Str`, `Vec[T]` as safe types (handle-typed) -- but with **Copy-Out (S2.11)** semantics,
   so their heap payload is promoted to the main heap before the arena resets. `&T` is now
   REJECTED as a tail type (cannot copy-out a borrow). This preserves the freeze gate
   (signatures unchanged) while fixing the UAF the review identified.
8. **Freeze-gate impact.** New types (HardwareFault, ContractViolation) and new runtime
   fns are additive; existing signatures unchanged. Wrapper return types may change for
   unsafe-block fns -- must be a **reviewed migration** (freeze snapshot update), not silent.
9. **`#[unsafe_direct]` governance (new).** The escape hatch must be restricted to
   stdlib/trusted packages; user code requires `--enable-unsafe-direct` + a counted cap.
   Without the gate, user code could bypass confinement entirely.
10. **Copy-Out cost on large tails (new).** One memcpy of the tail per block is bounded,
    but a 10MB returned buffer copies 10MB. Mitigation: `#[heap = "main"]` for large
    surviving allocations (fault-bounded, not isolated); Phase 7+ Promotion (O(1) page
    adoption) as the long-term optimization.

---

## 7. SUCCESS CRITERIA

- [ ] All nine requirements (a-i) have a passing test in the suite.
- [ ] A faulting `unsafe` block returns `Err(HardwareFault)` and the process continues
      (smoke: deliberate SIGSEGV in a confined block, then a println after).
- [ ] A contract-violating wrapper returns `Err(ContractViolation)` (no process trap).
- [ ] Guard-heap corruption smoke: main heap intact after a block corrupts its arena.
- [ ] **UAF smoke (review): a Vec/Str returned from a confined block stays intact after
      the arena resets (Copy-Out); a later mutation of the arena doesn't affect it.**
- [ ] Stack overflow in a confined block is caught at the guard page, not 0xC00000FD.
- [ ] Zero-escape checker test: returning `*T` from an `unsafe` block -> T005;
      returning `&T` from an `unsafe` block -> T005; un-converted extern `*T` tail -> T006.
- [ ] **FFI ownership test: an extern returning `*T` must be converted before the tail
      (T006) -- and a converted pointer is freed by the registered destructor, not leaked.**
- [ ] **Trampoline test: the same confined block faults twice -> `Err(HardwareFault{
      retried: true })`; a transient fault retries once and succeeds.**
- [ ] Stdlib migrates (37+ sites) with transaction batching; stdlib-exec + e2e green;
      perf budget met (<=1.5x); `#[unsafe_direct]` count reported and capped.
- [ ] Self-host bootstrap runs with confinement gates active.

---

## 8. EFFORT ESTIMATE (REVISED 2026-08-10 -- review refinements)

| Phase | Est. effort | Risk |
|-------|-------------|------|
| 1 -- Gates (a,b,i,T006) | 8-10h | Low |
| 2 -- Contracts (c) | 6-8h | Low |
| 3 -- Guard heap (d) + Copy-Out (i) | 14-20h | Medium |
| 4 -- Guard pages (e) | 8-12h | Medium |
| 5 -- Trap + recovery (f,g) -- **Trampoline + block-as-function lowering** | 36-52h | **High** (block-as-function, SEH/sigsetjmp, defer-on-fault) |
| 6 -- Retry (h) | 6-8h | Medium |
| 7 -- Stdlib adoption + batching + unsafe_direct audit | 20-28h | Medium |
| 8 -- Self-host gate | 8-12h | Medium |
| **Total** | **~106-150h** | -- |

Delta vs original (~82-118h): +24-32h for the review refinements -- the trampoline +
unsafe-block-as-function lowering (+16-22h on Phase 5), Copy-Out codegen (+2-4h on
Phase 3), T006 FFI-ownership gate (+2h on Phase 1), and the transaction-batching +
`#[unsafe_direct]` governance audit (+4h on Phase 7).
