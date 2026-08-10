# XIOM — Unsafe Confinement Architecture (Production Plan)

**Status:** PLAN ONLY — no implementation. This document is the design contract for the
Unsafe Confinement system (items a–i). Implementation begins after approval and follows the
phases in §6.
**Target:** v0.57.0 (PRE-SELFHOST — must land before the self-hosting bootstrap begins)
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

- `Checker.unsafe_depth: u32` (xiom-check/src/lib.rs:117) — incremented on `Expr::Unsafe`,
  decremented on exit.
- `gate_unsafe(op, span)` (xiom-check/src/lib.rs:658) — emits a hard T001 error for:
  - raw-pointer deref (`*p`) at depth 0,
  - `Int↔Ptr`, `Ptr→Ptr`, `Vec/Str→Ptr`, `fn↔Int/ptr` casts at depth 0,
  - `Stmt::Asm` at depth 0 (xiom-check/src/lib.rs:2986).
- `&x` → `*T` ref-coercion stays safe (FFI borrow pattern).
- `unsafe { }` compiles **transparently** in codegen (expr.rs ~3806): the block is an
  expression whose tail value flows out normally. No isolation, no trapping, no checks.

### 1.2 Gap analysis vs requirements

| Req | Status | Gap |
|-----|--------|-----|
| (a) Lexical Confinement | **PARTIAL** | deref/casts/asm gated. `extern "C"` CALLS are NOT gated — an `extern` callable can be invoked from safe code. `asm!` macro form doesn't exist (only `asm("...")`). |
| (b) Block-Scoped Only | **PARTIAL** | syntax is block-only already, but nothing *enforces* that a fn body consisting solely of `unsafe { ... }` is treated differently, and `unsafe fn` doesn't exist (good) — but nothing stops `unsafe` from being the whole body and effectively fn-wide. |
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
- **Stack guard pages on Windows:** `VirtualAlloc` with `PAGE_GUARD` — one-shot (auto-clears
  on first access), must be re-armed; or reserve + commit with `PAGE_NOACCESS` below the
  frame and catch the fault via the vector handler.
- **clang lowering:** `call void asm sideeffect` is opaque to the trap wrapper — the wrapper
  is per-`unsafe`-block, emitted by codegen around the block's IR.
- **The stdlib itself uses `unsafe` heavily** (37+ blocks in `alloc.xi`, `collections.xi`,
  `ptr.xi`, `encoding.xi`, `ffi.xi`, ...). Confinement must NOT slow the hot paths
  (Vec push/alloc) — a **perf budget** must be defined (see §7).
- **Recursion counter** (`@xiom_recursion_counter`, thread_local) interacts with longjmp:
  a fault unwinding past the block must still leave the counter balanced — the trap wrapper
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
  unsafe {                       // block boundary → transaction
    var p: *Int = ...;
    *p = x;                      // confined
    ...                          // may fault; may allocate on guard heap
    return *p;                   // safe value crosses out (i)
  }
}
```

Compiled (conceptually):

```
prologue:
  check requires(x)                          // (c) pre-entry contract — hard fail → Err(ContractViolation)
  tp = xiom_trap_enter(handler_state)        // (f) checkpoint (sigsetjmp / SEH __try)
  guard_heap = xiom_guard_heap_select()      // (d) isolate heap context
  push stack guard page                      // (e) red zone below frame
block_body: ...                               // the user's statements (allocations → guard heap)
  xiom_trap_leave()                          // (g) disarm
  ensure checks on the tail value            // (c) post-exit contract
  ret (safe value)                           // (i) only safe types cross
fault_handler:
  log fault (sig, address, pc)               // (f)
  restore recursion counter                  // §1.3
  retry_once: fresh memory slot (h) → re-run block_body
  else: return Err(HardwareFault{ sig, pc }) // (g)
```

### 2.2 (a) Lexical Confinement — hard errors

- Extend the D2 gate:
  - **`extern "C"` calls from safe code → hard error.** Only `unsafe { }` may call an
    `extern` function. The checker tracks `extern` fn names (already registered at
    TopDecl::Extern, xiom-check:1461) and gates `Expr::Call` targets that resolve to one.
  - **`asm` → hard error outside `unsafe`** (already done). Add the `asm!` macro alias.
  - **`*T`/`*mut T`/`*const T` type annotations in safe signatures → hard error** (a safe
    fn may not *name* a raw pointer type in its signature; pointer types only appear as
    locals inside the block, or in `unsafe`-internal helper signatures that are themselves
    unreachable from safe code).
- Error codes: extend T001 family — `T002` (extern call outside unsafe), `T003` (raw
  pointer type in safe signature), `T004` (asm outside unsafe — keep existing message).

### 2.3 (b) Block-Scoped Only — enforcement

- Parser/checker rule: `unsafe` applies strictly to the brace block `{ }`. Reject:
  - `unsafe fn`, `unsafe module`, `unsafe struct`, `unsafe impl` (parse errors).
- **Whole-body `unsafe`:** if a safe fn's body is *entirely* one `unsafe { }` block, the
  checker requires the fn to carry `requires`/`ensures` contracts (this is the (c) wrapper
  rule — see 2.4). This prevents "unsafe by delegation" without a contract.

### 2.4 (c) Pre-Entry Contracts

- **Checker rule:** every `unsafe { }` block must be lexically nested inside a fn that
  declares at least one `requires` contract. (Rationale: the wrapper validates *all inputs*
  before the block runs; a block with no wrapper has no validation story.)
- **Codegen injection:** before the block's first instruction, emit the `requires`
  checks (already supported: contracts compile to `br` + `@llvm.trap`-style guards today —
  change the trap target to the *handler* instead of process abort). If a check fails:
  return `Err(ContractViolation{ contract, value })` from the wrapper (not a crash).
- **Post-exit `ensures`:** the tail value is checked against the wrapper's `ensures`
  clauses inside the trap wrapper's success path; failure → `Err(ContractViolation)`.

### 2.5 (d) Heap Isolation — Guard Heap

- New runtime: **`xiom_guard_heap_*`** — a dedicated arena allocator:
  - `xiom_guard_heap_select()` — switch the block's `malloc`/`realloc`/`free` to the guard
    arena (per-thread pointer swap; the block's `malloc` calls route to arena-allocated
    slabs).
  - `xiom_guard_heap_reset(arena)` — free all slabs on block exit (or on fault retry).
  - Isolation: the arena's slabs come from dedicated `VirtualAlloc`/`mmap` regions that are
    **never handed to the main process heap**. Corruption inside the block cannot
    contaminate application memory; on block exit the arena is discarded wholesale.
  - FFI calls inside the block that allocate on the C heap (e.g. a lib that mallocs)
    cannot be intercepted — documented limitation; the (f) trap still bounds the damage to
    the block's execution window (see §7 risks).

### 2.6 (e) Stack Guard Pages

- Per-block frame setup: reserve a red zone below the block's stack frame using
  `VirtualAlloc(PAGE_GUARD)` (Windows) / `mmap(PROT_NONE)` (POSIX).
- On overflow, the hardware raises the fault *at the guard page* — before adjacent
  memory is written — and the (f) handler catches it.
- Windows `PAGE_GUARD` is one-shot: the handler re-arms it before the retry (h).
- Cost: one guard page per concurrently-active unsafe block (reuse via a per-thread pool of
  guard pages — `xiom_guard_page_pool`).

### 2.7 (f) Hardware Fault Trapping — portable trap abstraction

Runtime API (in `xiom_runtime.c`, platform-gated):

```c
typedef struct XiomTrapState { /* platform-specific */ } XiomTrapState;
int  xiom_trap_enter(XiomTrapState* st);        // 0 = normal, else fault code (sigsetjmp / SEH __try / VEH)
void xiom_trap_leave(XiomTrapState* st);
int  xiom_trap_retry(XiomTrapState* st);        // re-enter checkpoint for the (h) retry
const char* xiom_trap_signal_name(int sig);     // SIGSEGV → "SIGSEGV", etc.
uintptr_t   xiom_trap_fault_pc(XiomTrapState* st);
```

- **Windows:** `AddVectoredExceptionHandler` installed once per process; the block's
  `xiom_trap_enter` records the checkpoint (`RtlCaptureContext`-style or SEH `__try` via a
  small `.c` thunk — clang supports `__try/__except` on Windows). The vector handler checks
  whether the faulting PC is inside a confined block; if yes → longjmp-equivalent unwind to
  the checkpoint; if no → chain to the previous handler (so non-unsafe faults still crash
  loudly, as today).
- **POSIX:** `sigsetjmp` + `sigaction(SIGSEGV|SIGILL|SIGFPE, handler)` with `SA_SIGINFO`;
  handler calls `siglongjmp` to the checkpoint.
- **Codegen:** each `unsafe { }` block is wrapped: `call xiom_trap_enter`, the block body,
  `call xiom_trap_leave`; the handler path (separate LLVM blocks) returns the fault as an
  `Err` value. The wrapper fn's return type becomes `Result[T, HardwareFault]` when the
  block's tail type is not already a Result.

### 2.8 (g) Recoverable Errors

- New stdlib types (stdlib/xiom/error.xi — additive, freeze-gate safe):
  ```xiom
  pub type HardwareFault = { signal: Str; pc: UInt64; retried: Bool; }
  pub type ContractViolation = { contract: Str; }
  ```
- The wrapper's return: `Err(HardwareFault{...})` or `Err(ContractViolation{...})`. Callers
  `match` and continue — the process never terminates due to an unsafe-block fault.

### 2.9 (h) Transient Fault Retry

- On a caught fault, the handler:
  1. logs the fault (signal, pc, block id),
  2. resets the guard heap arena (`xiom_guard_heap_reset`),
  3. re-arms the stack guard page,
  4. restores the recursion counter delta,
  5. re-executes the block body **once** on a fresh memory slot,
  6. if it faults again → permanent `Err(HardwareFault{ retried: true })`.
- Retry is opt-out per block via an attribute: `#[unsafe_no_retry]` (some faults are not
  transient by nature — e.g. a guaranteed bad deref would just fault twice).

### 2.10 (i) Zero Escape

- Checker rule: the tail value of an `unsafe` block must have a **safe type** — reject
  `*T`, `Ptr`, `fn`-typed, or struct values containing raw pointers as the block's result.
  Raw pointers may be *used* inside but never *named* as the block's value type.
- Codegen: FFI handles returned from `extern` calls inside the block are wrapped at the
  boundary (e.g. an `extern` returning `*T` may only be consumed inside; if it must cross,
  the block returns `Some(handle as Int)` — an opaque safe integer, or `Err`).
- Register state (from `asm`) never crosses: asm outputs are confined to locals inside the
  block.

---

## 3. IMPLEMENTATION SURFACE (files touched, per phase)

| Area | Files |
|------|-------|
| Checker gates (a, b, c, i) | `crates/xiom-check/src/lib.rs` (gate_unsafe, check_call extern gate, tail-type check, contract-wrapper rule) |
| Parser (b, a) | `crates/xiom-parser/src/lib.rs` (reject `unsafe fn` etc.; `asm!` alias) |
| AST (b) | `crates/xiom-ast/src/lib.rs` (no new node needed — block already `Expr::Unsafe`) |
| Codegen wrap (f, d, e, g) | `crates/xiom-codegen/src/expr.rs` (Unsafe arm → transaction lowering), `decl.rs` (wrapper return type), `emitter.rs` (declare new runtime fns) |
| Runtime (d, e, f, g, h) | `stdlib/runtime/xiom_runtime.c` (guard heap, guard pages, trap enter/leave/retry, SEH/POSIX) |
| Stdlib types (g) | `stdlib/xiom/error.xi` (HardwareFault, ContractViolation), `stdlib/xiom/unsafe.xi` (block attributes, retry policy) |
| Tests | `crates/xiom-check` (gate unit tests), `crates/xiom-codegen/tests/stdlib_execution_tests.rs` (confinement smokes), `tests/regression/` (m34/m35 unsafe suites extended) |
| Docs | `docs/SAFETY_HARDENING.md`, `docs/STDLIB_EXTENSION.md` §13 D2, `docs/ROADMAP.md` §4 |

---

## 4. PHASES

Each phase is independently verifiable; the suite must stay green at every phase boundary.

### Phase 1 — Confinement Gates (a, b, i-checker)
- (a) `extern "C"` call gate: T002 hard error in safe code; `asm!` alias.
- (a) raw-pointer type names in safe signatures: T003.
- (b) parse rejects `unsafe fn/module/struct/impl`.
- (i) checker rejects raw-pointer/`fn`-typed tails from `unsafe` blocks (T005).
- **Verify:** checker unit tests (est. +12), full suite green. No runtime change.

### Phase 2 — Contracts & Wrapper Rule (c)
- Checker: every `unsafe` block is inside a fn with ≥1 `requires`; whole-body-unsafe fns
  must declare `requires`+`ensures`.
- Codegen: requires-checks fail → `Err(ContractViolation)` instead of process trap;
  ensures-check on tail.
- **Verify:** contract smokes (safe wrapper + violation path), suite green.

### Phase 3 — Guard Heap (d)
- Runtime `xiom_guard_heap_*` (arena slabs, per-thread context switch).
- Codegen: inside `unsafe` blocks, `malloc`/`realloc`/`free` calls route to the guard
  arena; reset on block exit.
- **Verify:** heap-isolation smoke (corrupt a pointer inside the block → main heap
  untouched, arena discarded); ASAN run green.

### Phase 4 — Stack Guard Pages (e)
- Runtime guard-page alloc/arm/disarm + per-thread pool.
- Codegen: page armed at block entry, disarmed at exit.
- **Verify:** deliberate stack-overflow-in-unsafe smoke → fault caught (Phase 5), process
  survives.

### Phase 5 — Hardware Fault Trapping + Recovery (f, g)
- Portable `xiom_trap_enter/leave/retry` (SEH/VEH on Windows; sigsetjmp on POSIX).
- Codegen: block wrapped; handler path returns `Err(HardwareFault)`; recursion counter
  restored.
- **Verify:** SIGSEGV/SIGILL/SIGFPE smokes on both platforms; the `test_diff_test_produces_correct_ir`
  and other suites stay green; faulting block returns Err, caller continues.

### Phase 6 — Transient Retry (h)
- Retry-once machinery + fresh arena slot + `#[unsafe_no_retry]`.
- **Verify:** transient-fault smoke (first run faults, retry succeeds, `retried: true`
  path); permanent-fault smoke (faults twice → Err).

### Phase 7 — Stdlib Adoption & Perf Budget
- Migrate the 37+ stdlib unsafe sites to the confined form (wrappers + contracts).
- Perf budget: guarded Vec push/alloc must stay within **1.5×** of unguarded; if exceeded,
  provide `#[unsafe_direct]` escape for ultra-hot stdlib internals (audited, counted).
- **Verify:** stdlib-exec full suite, e2e full suite, benchmark-chaos comparative run.

### Phase 8 — Self-Host Gate
- All unsafe confinement gates active when the compiler compiles itself.
- **Verify:** full `cargo test`, selfhost bootstrap smoke, docs updated.

---

## 5. ROADMAP INTEGRATION

Added to `docs/ROADMAP.md` §4 (PRE-SELFHOST ROADMAP) as **v0.57.0 "Unsafe Confinement"**,
after v0.56.0-pre and BEFORE the SELFHOST bootstrap:

| Milestone | Version | Scope | Depends on |
|---|---|---|---|
| Unsafe Confinement | **v0.57.0** | Phases 1–8 above | v0.56.0-pre (LTO, debug info, hardening) |
| Selfhost | SELFHOST | XIOM compiles itself | v0.57.0 (confinement gates must be live) |

Also referenced from: `docs/SAFETY_HARDENING.md` (new §"Unsafe Confinement"), and
`docs/STDLIB_EXTENSION.md` §13 (D2 extended → "D2.1 Confined Unsafe").

---

## 6. RISKS & OPEN QUESTIONS

1. **Windows SEH vs codegen IR.** The trap wrapper lives at the LLVM-IR level, but
   `__try/__except` is a C-language construct. Option A: emit the block as a separate
   function compiled from a generated C thunk (clang handles SEH). Option B: use
   `AddVectoredExceptionHandler` + `RtlRestoreContext`-style unwind (no __try needed) —
   preferred, keeps the IR pipeline intact. **Open question: validate Option B prototype
   early in Phase 5.**
2. **`longjmp` across LLVM-generated frames.** `siglongjmp` unwinds the C stack but skips
   C++/LLVM destructors (there are none — XIOM has no RAII yet, `defer` is manual). Must
   verify no `defer` inside unsafe blocks is skipped silently (Phase 5 note).
3. **Guard-heap interception coverage.** `extern` calls that allocate on the C heap
   bypass the guard arena. Mitigation: (f) trap bounds the window; document that FFI
   allocations are *not* isolated, only *fault-bounded*.
4. **Performance.** Guard-page arm/disarm + arena switch + checkpoint per block. Perf
   budget 1.5×; hot stdlib paths get the audited `#[unsafe_direct]` escape.
5. **Recursion counter balance across longjmp** (§1.3) — must be captured/restored in the
   handler; a leak would trip the 500-depth guard spuriously.
6. **Retry semantics.** Some faults (e.g. writing to a read-only page) are deterministic —
   retry is pointless. The `#[unsafe_no_retry]` attribute covers these; default = retry
   once (transient faults like first-touch guard pages benefit).
7. **Tail-value zero-escape vs existing stdlib.** Several stdlib fns currently *return*
   raw-pointer-backed values (e.g. `Str.from_cstring` returns i8*). The (i) rule must
   treat `Str`, `Vec[T]`, `&T` as **safe types** (they are handle-typed, not raw-pointer-
   named) — only *named raw pointer types* (`*T`, `Ptr`) are confined. This preserves the
   freeze gate.
8. **Freeze-gate impact.** New types (HardwareFault, ContractViolation) and new runtime
   fns are additive; existing signatures unchanged. Wrapper return types may change for
   unsafe-block fns — must be a **reviewed migration** (freeze snapshot update), not silent.

---

## 7. SUCCESS CRITERIA

- [ ] All nine requirements (a–i) have a passing test in the suite.
- [ ] A faulting `unsafe` block returns `Err(HardwareFault)` and the process continues
      (smoke: deliberate SIGSEGV in a confined block, then a println after).
- [ ] A contract-violating wrapper returns `Err(ContractViolation)` (no process trap).
- [ ] Guard-heap corruption smoke: main heap intact after a block corrupts its arena.
- [ ] Stack overflow in a confined block is caught at the guard page, not 0xC00000FD.
- [ ] Zero-escape checker test: returning `*T` from an `unsafe` block → T005.
- [ ] Stdlib migrates (37+ sites) with stdlib-exec + e2e green; perf budget met (≤1.5×).
- [ ] Self-host bootstrap runs with confinement gates active.

---

## 8. EFFORT ESTIMATE

| Phase | Est. effort | Risk |
|-------|-------------|------|
| 1 — Gates (a,b,i) | 6–8h | Low |
| 2 — Contracts (c) | 6–8h | Low |
| 3 — Guard heap (d) | 12–16h | Medium |
| 4 — Guard pages (e) | 8–12h | Medium |
| 5 — Trap + recovery (f,g) | 20–30h | **High** (SEH/VEH, longjmp) |
| 6 — Retry (h) | 6–8h | Medium |
| 7 — Stdlib adoption + perf | 16–24h | Medium |
| 8 — Self-host gate | 8–12h | Medium |
| **Total** | **~82–118h** | — |
