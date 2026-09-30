<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Stage 6: performance plan (benchmark-driven)

**Status:** PLANNING. This is the intake doc for benchmark-lane relays; the
first finding (unsafe-block trampoline overhead) is queued as PERF-1.
**Siblings:** `docs/COMPILER_IMPROVEMENT_PLAN.md` (production targets),
`docs/STAGE6_LINT_WAVE.md` (the other Stage 6 workstream),
`docs/UNSAFE_CONFINEMENT_PLAN.md` (D2.1 confinement; its S7 already flags
that a **perf budget must be defined** for guard-page arming on hot paths).

## Target (from COMPILER_IMPROVEMENT_PLAN.md)

- Release/O2 within **10% of Rust** on the benchmark categories.
- Binary size <= 2x Rust (stripped); compile time <= 3x Rust (debug).
- `xiom bench` (COMPILER_IMPROVEMENT_PLAN 1.x) is the intended local harness.

## Intake format (what a benchmark relay must provide)

Session id + dashboard/API paths, host export JSON, per-trial evidence dir,
manifest + task-set hashes, compiler/contract versions, the per-task numbers,
isolation probes, disassembly notes, and repro source paths. Each finding
becomes a numbered PERF item with: evidence, root-cause hypothesis, candidate
fixes, a lock proposal, and a re-run confirmation step.

## PERF-1 (priority 1, RELEASE-BLOCKING for v0.62.2): unsafe-block call trampoline overhead -- t2-queue

**Fix route (2026-09-29, resolved):** the root cause chain is now fully
identified and the SANCTIONED `#[unsafe_direct]` route fixes it with no
retry-semantics change and no new safety policy:

1. The parser silently dropped `#[unsafe_direct]` written above `pub fn`
   (`P001` absorbed by error recovery) -- fixed as **m166** with locks
   (parser unit test + IR trust test).
2. `#[unsafe_direct]` trust only looked at the PRIMARY source path, so
   stdlib fns compiled inside a user program were rejected -- fixed as
   m166 by trusting the injected catalog fn keys (driver hands
   `catalog_fn_keys` to codegen).
3. Local verification with the two atomic wrappers annotated: 4M pairs
   8000 ms -> **0 ms**. IR: no `xiom_trampoline_call` / guard arm-disarm
   in `@sync.atomic_load` / `@sync.atomic_store`.

Remaining release steps: the stdlib lane annotates
`stdlib/xiom/sync/atomics.xi` (relay in COMPILER_BUGS m166 + the release
gate), the pin moves for v0.62.2, and the benchmark lane re-runs t2-queue.

**v0.62.2 outcome (2026-09-30): t2 NOT cleared.** The atomics.xi
annotation shipped, but t2 stayed ~34 s (full arena) / ~12 s (probe):
consumers of the LEGACY `xiom.sync` API bind `sync.xi`'s own AtomicInt
methods/helpers (unannotated), and the m166 provenance lookup missed
METHOD decls (short `fd.name.name` vs the receiver-qualified injected key).
The compiler gap is FIXED (receiver-qualified trust key in `compile_fn` +
the mono path; locks + full e2e 2393/2393; local proof 7000 ms -> 0 ms
with the legacy methods annotated). Remaining: stdlib annotates
`sync.xi`'s unsafe-block fns (relay in COMPILER_BUGS 2026-09-30), tags
`stdlib-perf2`, pin moves, benchmark re-runs -- Gate P then accepted for
the FOLLOW-UP release (v0.62.3), not v0.62.2.

Candidate fixes (1)/(2)/(3) from the list below (block-level arming,
callee classification, trampoline fast path) remain Stage 6 follow-ups for
hot unsafe blocks that CANNOT be marked trusted (user code without
`--enable-unsafe-direct`, third-party packages).

**Provenance (benchmark lane, 2026-09-29):**

- Session `run_1790700620309` (profile "System", 42/42 complete);
  API `/api/sessions/run_1790700620309`, `/full`,
  `/api/export/json/run_1790700620309`.
- Host export `E:\tmp_benchmark_results\session-run_1790700620309.json`
  (890 KB), plus the failure-rich `session-run_1790636666348.json`
  (7P/35F, 1.3 MB).
- Evidence `E:\xiom-projects\xiom-benchmark-chaos\data\evidence\run_1790700620309\`
  (per-trial source / compiler log / stdout); manifest in `data\manifests\`.
- XIOM Compiler **v0.62.1**, contract **v1.13**, task-set hash
  `e9c838eb...`, manifest `0ee1a841f836...`.

**Numbers:** XIOM is normal on all tasks except t2-queue.

| language | t2-queue |
|----------|----------|
| rust | 35 ms |
| ada | 116 ms |
| go | 123 ms |
| zig | 154 ms |
| c | 189 ms |
| cpp | 191 ms |
| **xiom** | **50,787 ms** (~1450x Rust) |

**Isolation (benchmark lane):**

- 4M `AtomicInt` load/store pairs: **18.5 s** vs **4 ms** for a plain loop.
- The same queue with plain `Int`s: **24 ms**.
- Disassembly: every compiled unsafe-block call pays
  `xiom_trampoline_call` + guard-page arm/disarm at **~2.4 us/call**;
  4M pairs x 2 calls x 2.4 us ~= 19 s -- matches the atomic isolation.
- Repro staged: `data/probes/atomic_trampoline.xi` +
  `atomic_overhead.sh` (container `/app/data/probes/`), plus
  `tcp_stream_read.xi`.

**Root-cause hypothesis:** the D2.1 confinement wraps EACH potentially-faulting
call inside an unsafe block in its own trampoline invocation + guard-page
arm/disarm, while `UNSAFE_CONFINEMENT_PLAN` sections 2.4-2.6 describe the
arming at BLOCK scope. A hot loop of intrinsic/FFI calls inside one block
therefore pays the full confinement cost per call.

**Candidate fixes (need safety-lane sign-off; D2.1 guarantees must not
silently weaken):**

1. **Hoist arming to the block**: arm the guard page + select the guard heap
   once per unsafe block, run all statements, disarm once. Per-call recovery
   remains only for calls that can actually fault. Retry semantics: a fault
   may re-run the block -- allowed only for replayable blocks or with a
   documented "fault aborts the block" policy.
2. **Classify callees**: trampoline only true FFI/extern boundaries; pure
   XIOM calls and compiler intrinsics (AtomicInt load/store) run direct.
3. **Fast-path the trampoline**: skip TLS round-trips and guard-page arming
   when the block provably performs no allocation/FFI (block-level flag set
   at construction).
4. **Inline the intrinsics**: compile AtomicInt load/store inline instead of
   calling runtime helpers -- likely a standalone win independent of the
   trampoline design.

**Local repro (landed 2026-09-29):** `docs/repro/perf-1-atomic-trampoline/`
(probe + README with measurements and IR excerpts). On this box, v0.62.1:
plain 4M-iteration loop below clock resolution vs **8000 ms** for 4M atomic
store+load pairs (~1.0 us/call; the benchmark container saw 18.5 s /
~2.4 us/call -- same order). IR confirms the cost is per CALL: each
`sync.atomic_load` / `atomic_store` wrapper builds a context and calls
`xiom_trampoline_call(@__unsafe_block_N, ctx)`; each block body runs
`xiom_guard_heap_enter` + `xiom_guard_page_arm` ... `_exit` + `_disarm` +
`xiom_trap_leave` around the single `xiom_atomic_*` intrinsic.

**Plan:** reproduce locally (`atomic_trampoline.xi` + `atomic_overhead.sh`,
`tcp_stream_read.xi`), decide between (1)+(2) as the design, land with a
perf-budget lock (dedicated micro-benchmark fixture wired into
`perf_budget_tests` or an e2e stress fixture), then ask the benchmark lane
to re-run t2-queue and confirm the target band.

**Reference sources (benchmark lane):** all 35 systems-arena files
(C/C++/Rust/XIOM/Zig/Go/Ada x t1-allocator, t2-queue, t3-hot-reload,
t4-packet, t5-btree, t8-safety-probe) under
`E:\xiom-projects\xiom-benchmark-chaos\reference\systems-arena\<task>.<ext>`;
companion conformance solutions in `tests/toolchain/solutions/` and LLM
templates in `tasks/systems-arena/`.

## Checklist

- [x] PERF-1: reproduce the atomic trampoline overhead locally
      (`docs/repro/perf-1-atomic-trampoline/`; plain ~0 ms vs atomic
      8000 ms for 4M pairs, ~1.0 us/call; IR: per-call trampoline +
      guard-page arm/disarm; benchmark container 18.5 s / ~2.4 us)
- [x] PERF-1: root-cause chain + fix route (m166 parser attribute-order +
      injected-catalog trust); landed with parser + IR locks + full e2e
- [x] PERF-1: design decision -- use the sanctioned `#[unsafe_direct]`
      route (no retry-semantics change, no new safety policy); block-level
      arming / callee classification stay as fallbacks for untrusted blocks
- [ ] PERF-1: stdlib lane annotates `stdlib/xiom/sync/atomics.xi`
      (`#[unsafe_direct]` per wrapper; relayed via COMPILER_BUGS m166 +
      RELEASE_GATE_v0.62.2) and tags for the v0.62.2 pin
- [ ] PERF-1: benchmark lane re-run confirms t2-queue (and no regression
      on t1/t3/t4/t5/t8)
- [ ] Stage 6 follow-up: general fast path for hot unsafe blocks that
      cannot be marked trusted (candidate fixes 1/2/3)
- [ ] Stage 6: define the confinement perf budget referenced by
      `UNSAFE_CONFINEMENT_PLAN` S7
- [ ] Stage 6: keep `xiom bench` as the local harness and record each
      benchmark relay in this doc
