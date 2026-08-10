# XIOM Session Handoff — 2026-08-10 17:50 (Unsafe Confinement mid-implementation)

## ⚠️ CRITICAL CONSTRAINTS
1. **NEVER commit/modify `xiom-benchmark-chaos/`** — owner works in a PARALLEL session (it has uncommitted changes). It COPIES `stdlib/` directly into its build (stdlib-pin deleted). Stage only: `crates/`, `stdlib/`, `docs/`, `tests/`, `examples/`, `selfhost/` (except `_diff_*`), `packages/`.
2. **Parallel session rebuilds `target/debug/xiom.exe` frequently** — verify failures by recompiling the specific smoke manually before assuming regression. The 16-minute e2e suite is UNRELIABLE mid-parallel; use checker + stdlib-exec (~1-2 min) as fast gates, e2e only at phase boundaries.
3. **Selfhost tests are IGNORED**: `test_selfhost_bootstrap_v050` + all `full_diff_tests.rs` — `#[ignore]`. Do not touch.
4. **Models**: ALL agents/subagents run DeepSeek V4 FLASH only. NEVER pro. `subagent_variant_overrides`: flash → high.
5. **Monorepo split DEFERRED**. stdlib becomes its own repo later (backlog).
6. **`test_diff_test_produces_correct_ir` is a PRE-EXISTING failure** on the clean tree (verified repeatedly) — NOT caused by Unsafe Confinement. `g15_sret_abi` e2e: the test file is `examples/e2e/g15_sret.xi` (was failing from the T002 gate, now MIGRATED to unsafe-wrapped extern calls).
7. **`.xiom_ai.json` is a parallel-session artifact — never stage it.**

## CURRENT BRANCH
`feat/architect` — HEAD `db60082c` (tree clean except `.xiom_ai.json`)

## SESSION GOAL (in progress): Implement UNSAFE CONFINEMENT v0.57.0 100% (plan: `docs/UNSAFE_CONFINEMENT_PLAN.md`, revised to ~106-150h)

### ✅ COMMITTED PHASES
| Commit | Phase | What |
|--------|-------|------|
| `0c490a8d` | **P1 Gates** | T002 extern-call gate (confined to unsafe; contracts exemption), T003 safe-fn raw-ptr return gate (zero-escape at fn boundary, unsafe-internal-helper exemption via `block_contains_unsafe`), block-only `unsafe` parse diagnostic, `&T` borrow-confined rule (local ok, fn-return ok — only RAW ptrs gated); migrated `g15_sret.xi`; 7 gate tests (checker 176, exec 69) |
| `29fbf966` | **P2 Contracts** | T007 whole-body-unsafe `requires` gate (req c); T002 contract-exemption refinement; migrated m34_y04/m36_c10; 2 tests (checker 176) |
| `776ae148` | **P3 Guard heap + Copy-Out** | TLS slab arena (`xiom_guard_heap_enter/exit/alloc`, VirtualAlloc/mmap 64KB slabs), `emit_alloc` routes 7 malloc sites to arena inside unsafe; Copy-Out `xiom_guard_copy_str/out` promotes Str tails + return-exits to main heap BEFORE arena reset (UAF fix, req d+i); smoke_guard_heap (exec 70) |
| `fd1e9969` | **P4 Stack guard pages** | `xiom_guard_page_arm/disarm` per-thread PAGE_GUARD/PROT_NONE red-zone at unsafe entry/exit (req e); overflow now faults at guard page (0xC000001D vs 0xC00000FD) |
| `db60082c` | **P5 WIP — fault trap** | VEH `xiom_trap_enter/leave` (RtlCaptureContext/RtlRestoreContext, active-until-leave), `confined_fault`/`confined_normal` branches in Unsafe arm, `HardwareFault{signal,pc,retried}` + `ContractViolation` types in `stdlib/xiom/error.xi`, `xiom_trap_signal_name` |

### ⚠️ P5 CURRENT BLOCKER (the last thing worked on — context ran out)
**The trap test HANGS** (infinite loop, no output, no crash): `C:\Users\lefte\AppData\Local\Temp\kilo\p5_trap.xi` — an unsafe block doing `var p = 1 as *Int; var v = *p;` (deliberate AV). Expected: VEH catches → RtlRestoreContext resumes at capture → `%fault != 0` → confined_fault path → process survives. **Actual: process hangs.** Likely causes to investigate in order:
1. **`RtlRestoreContext` restores RIP to the `xiom_trap_enter` CALL SITE but the fault re-triggers** — the captured context's RIP points at the `icmp` after the call, but the AV re-fires because... check whether the handler's `xiom_trap_fault` is visible post-restore (it's thread-local — restore doesn't reset it, but verify the codegen's `%fault = call @xiom_trap_enter()` actually RECEIVES the fault code — RAX must be set in the restored context; `RtlRestoreContext` restores RAX from `xiom_trap_ctx.Rax` — verify `RtlCaptureContext` captured RAX with CONTEXT flags).
2. **The VEH may not fire at all** for the AV (then it should CRASH not hang — hang suggests the handler IS firing and the restore loops). If restoring to the capture point re-executes the faulting instruction (because RIP captured BEFORE the call completes?), add a **recovery flag + explicit branch**: instead of relying on the restored RAX, have `xiom_trap_enter` return the fault via a thread-local that the codegen reads: emit `call @xiom_trap_enter()` then `%f = load @xiom_trap_last_fault` (a new TLS global) — more deterministic than register-restore.
3. **Simpler alternative if VEH keeps misbehaving** (documented in plan §2.7 as the fallback): the **block-as-function + `__try/__except` trampoline** — `xiom_trampoline_call(xiom_block_fn fn, uint8_t* ctx)` ALREADY EXISTS in the runtime (compiles, SEH-based, returns fault codes 1-6). The codegen change: emit the unsafe block's statements into a standalone `__unsafe_block_N(ctx)` LLVM function (free-variable ctx capture — reuse closure machinery M20) and the call site emits `xiom_trampoline_call(block_fn, &ctx)` + branch on the fault code. This is the plan's canonical design; more work but deterministic.

### 📋 REMAINING WORK (after P5 unblocks)
- **P5 finish**: recoverable `Err(HardwareFault)` — the fault path currently `ret 0` for the fn's return type; the plan wants `Result[T, HardwareFault]` wrappers. At minimum: process survives + returns a fault indicator. Add fault-injection smoke (`smoke_guard_fault.xi`): deliberate AV in confined block → process continues (println after). Test SIGILL (asm ud2) + SIGFPE (div by zero) paths too.
- **P6 Transient retry (h)**: retry-once — re-arm guard page, fresh arena slot, re-run the block; `#[unsafe_no_retry]` attribute. Runtime: `xiom_trap_retry` (re-enter checkpoint). Fault path currently returns immediately — add ONE retry loop around the block execution.
- **P7 Stdlib adoption + batching + `#[unsafe_direct]`**: audit 37+ stdlib unsafe sites for transaction granularity (whole-loop unsafe, not per-op); `#[unsafe_direct]` escape hatch (restricted to stdlib/trusted, `--enable-unsafe-direct` gate for user code, counted cap).
- **P8 Self-host gate**: full suite + selfhost smoke with confinement active.
- **T006 FFI-ownership gate** (from review, still TODO — Phase 1 planned it): extern returning `*T` inside confined block must convert to owned XIOM type before tail (checker rule).

## TEST SUITE STATE (fast gates — verified after P4/P5-WIP)
| Suite | Result |
|-------|--------|
| checker | **176/176** (incl. P1/P2 gate tests) |
| stdlib-exec | **70/70** (incl. smoke_guard_heap) |
| stdlib-compile | 40/40 |
| parser | 96/96 |
| feature-reg / integration / freeze | 510 / 128 / 2 (last full pass at P3 commit) |
| e2e | 2231/2231 (at P3; P4/P5-WIP only fast-gated) |

## PRIOR SESSION COMPLETIONS (context before this session)
- D2 unsafe gating (`83416aa9`), D1 native Int128/UInt128/Float128 (`ba2dd6e9`), hardening 3a/3b/3e (`b34ebaa4`), 3c generic numeric tower (`f1cb3fef`+`cd48a8df`), docs (`1e59259f`+`be8309d6`), package README phase (`69f97f00`), folder-module refactor (`bbf3d21a`).

## NEXT SESSION — START HERE
1. Read `docs/UNSAFE_CONFINEMENT_PLAN.md` (the design contract, includes review refinements).
2. **Fix the P5 trap hang** (see blocker above — try the TLS-fault-flag readback first, then the trampoline fallback).
3. Verify with `C:\Users\lefte\AppData\Local\Temp\kilo\p5_trap.xi` (AV smoke) + `smoke_guard_heap.xi`.
4. Continue P6 → P7 → P8 → T006. Commit per phase. Run checker + stdlib-exec after every change; e2e at phase boundaries only.
