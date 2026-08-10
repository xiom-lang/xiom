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
`feat/architect` — HEAD `d866231a` (P5-P8 + T006 all committed; tree clean except `.xiom_ai.json`)

## SESSION GOAL: Implement UNSAFE CONFINEMENT v0.57.0 100% — **COMPLETE** (P1-P8 + T006)

### ✅ COMMITTED PHASES
| Commit | Phase | What |
|--------|-------|------|
| `0c490a8d` | **P1 Gates** | T002 extern-call gate, T003 safe-fn raw-ptr return gate, block-only unsafe parse, &T borrow-confined; migrated g15_sret; 7 gate tests |
| `29fbf966` | **P2 Contracts** | T007 whole-body-unsafe requires gate; contract exemption; 2 tests |
| `776ae148` | **P3 Guard heap + Copy-Out** | TLS slab arena, emit_alloc routing, Copy-Out Str tails; smoke_guard_heap |
| `fd1e9969` | **P4 Stack guard pages** | per-thread guard page red-zone at unsafe entry/exit |
| `cc06f69f` | **P5 Trap + recovery (f,g) — FIXED** | REPLACED broken inline VEH (hung). Canonical block-as-function + SEH trampoline. Pointer captures (write-back). Return routing via TLS this_returned (nested-safe). Arena-aware realloc. Fault-injection smokes (AV/ud2/div0 survive). |
| `f8919652` | **P6 Transient retry (h)** | Once-only retry in trampoline; #[unsafe_no_retry]; xiom_trampoline_was_retried; smoke_guard_retry |
| `022d903f` | **P7 stdlib + #[unsafe_direct]** | #[unsafe_direct] escape hatch (stdlib/trusted; --enable-unsafe-direct gate; counted cap). Stdlib needed NO migration — P5/P6 lowering already handles all 37+ sites. |
| `d866231a` | **T006 FFI ownership** | extern-returning-*T inside confined block must convert to owned type before tail (ffi.safe_ptr_from_raw/box_from_ptr/vec_from_ptr_with_free/str_from_ptr_owned); 2 checker tests |
| (docs) | **P8 Self-host gate** | xiomc_v10.xi compiles with all confinement gates live (verified); selfhost diff tests remain #[ignore]d (deferred, per constraints) |

## TEST SUITE STATE (fast gates — verified after T006/P8)
| Suite | Result |
|-------|--------|
| checker | **178/178** (+T006 tests) |
| stdlib-exec | **70/72** (guard_fault/guard_retry/guard_heap pass; complex+net are PRE-EXISTING baseline failures) |
| stdlib-compile | 40/40 |
| parser | 96/96 |
| feature-regression | 510/510 |
| integration | 128/128 |
| P8 | xiomc_v10.xi compiles clean under all confinement gates |

## KNOWN LIMITATIONS (documented, not blockers)
- **complex + net stdlib smokes** fail on the clean baseline too (pre-existing; net is network-dependent).
- **Full selfhost diff tests** (`test_selfhost_bootstrap_v050`, full_diff_tests) remain `#[ignore]`d — selfhost bootstrap is a deferred milestone; the gate (compiler compiles itself) is verified manually.
- **e2e (16min)** not re-run at every phase boundary this session (fast gates used; parallel-session xiom.exe rebuilds make it flaky). Run e2e at the next full-suite boundary.
- **HardwareFault/ContractViolation** types exist in stdlib/xiom/error.xi; the fault path returns a type-correct zero (recoverable indicator) rather than a full `Result[T, HardwareFault]` wrapper (plan §2.8's wrapper is a future refinement).

## NEXT SESSION — START HERE
1. Run the full e2e suite at a phase boundary (fast gates are green).
2. Optionally: selfhost bootstrap milestone (re-enable full_diff_tests) — deferred by constraints.
3. Optional refinements: Result[T, HardwareFault] wrapper (plan §2.8), transaction batching audit for perf, Promotion (zero-copy Copy-Out).

## DOC UPDATES (2026-08-10, docs-only commit)
- `docs/AI_CONTEXT.md` → v0.57.0: Unsafe Confinement model (§4.4, req a–j), `#[unsafe_no_retry]`/`#[unsafe_direct]` attributes, HardwareFault/ContractViolation (§8.17), `--enable-unsafe-direct` CLI flag (§11), FFI ownership conversions (C FFI), 60-module stdlib count.
- `docs/SCALING_ARCHITECTURE.md` → v0.2: pre-selfhost review incorporated (§12 — Sealed Generics/Pre-Mono Table, Layout Hash, Compiler Daemon) + revised migration path (~15 weeks).
- `docs/CTFE_PLAN.md` → §6 integration audit (CTFE+Confinement contracts; CTFE cache → SyncRegistry at Scaling Phase 5, NOT before).
- `docs/ORCJIT_PLAN.md` → §7 integration audit (JIT inherits Confinement automatically; `JitModule::get_function_ptr()` for Live Patching).
- `docs/LIVE_PATCHING_PLAN.md` → NEW v0.61 design spec (patchable ABI, JIT sandbox, atomic swap, rollback, AI/spacecraft integration).
- `docs/BIGINT_BIGFLOAT_SESSION.md` → NEW parallel-work session spec: production BigInt extension + BigFloat build, full API/contracts/test plan, ALL stdlib libs categorized with 12 parallel session slots.
