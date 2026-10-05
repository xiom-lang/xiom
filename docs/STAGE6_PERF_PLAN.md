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
- [x] PERF-1: stdlib lane annotated `stdlib/xiom/sync/sync.xi`
      (`#[unsafe_direct]`; shipped in the v0.62.3 pin `stdlib-perf3`,
      stdlib `2429ac3`)
- [x] PERF-1: benchmark lane re-run confirms t2-queue and no regression --
      **ACCEPTED 2026-10-03 on v0.62.3**: t2-queue 24 ms vs 11,968 ms on
      v0.62.2 (~500x), in family with c/cpp/zig/go; t1/t3/t4/t5 unchanged;
      t8 safety indices identical; 42 pairs, zero errors
- [ ] Stage 6 follow-up: general fast path for hot unsafe blocks that
      cannot be marked trusted (candidate fixes 1/2/3)
- [ ] Stage 6: define the confinement perf budget referenced by
      `UNSAFE_CONFINEMENT_PLAN` S7
- [ ] Stage 6: keep `xiom bench` as the local harness and record each
      benchmark relay in this doc
- [x] Stage 6 item 1 slice: runtime-object cache + index/check sub-phase
      timings (hello 8.7s -> 0.97s warm; lz4 program 13.3s -> 4.0s warm)
- [x] Stage 6 item 1 slice 2: persistent catalog index-header cache
      (warm index 0.70s -> 0.32s; hits=2397/2397)

## Benchmark-driven hardening/optimization backlog (2026-10-03, v0.62.3 arena runs)

Source: benchmark lane artifacts from the official v0.62.3 toolchain --
systems metrics `system-session-run_1791052819493` (42 trials, per-trial
compile/runtime/memory), scripting `scripting-session-run_1791053692625`,
contracts `contracts-session-run_1791053925431`, and the safety-probe
ladder. Gate P acceptance lives in `docs/RELEASE_GATE_v0.62.3.md`.

### What the data says

- RUNTIME: XIOM is at or near the front. t2-queue 24 ms (c 35, cpp 35),
  t4-packet 26 ms (c 83), t5-btree 13 ms (c 31), t8 safety probe 141 ms
  (c 712, rust 129). Exceptions: t1-allocator 71 ms (c 29, rust 14) and
  t3-hot-reload 256 ms (c 33) -- t3 task semantics need confirmation (if it
  times per-reload work, it is a compile-latency issue, not steady state).
- COMPILE -- the dominant measured gap: XIOM 6.4-13.6 s per task vs c
  0.3-0.6 s and rust 0.3-0.5 s (zig 7.0-12.5 s is the only peer). The
  arena compiles each program from scratch and XIOM re-parses, re-checks,
  and re-emits the whole reachable stdlib module graph on every
  invocation. This is the single largest compiler optimization
  opportunity measured so far.
- SCRIPTING: already fastest -- xiom-run 26-45 ms end-to-end vs python 43,
  lua 64, node 73 (t6) and 15-19 ms vs lua 19, python 88 (t7); cache / jit
  / aot variants are within noise at these sizes. Keep the cache path
  correct (C25 fixed 2026-10-03) and extend the win to larger scripts.
- MEMORY: t1-allocator 30 MB peak vs c 9 / rust 3; other tasks 2-9 MB.
- SAFETY (owner direction: harden after selfhost): 12 probes, compile
  prevention 0%, runtime detection 41.7%, process survival 50%, silent
  corruption 3 (use-after-free, double-free, type confusion -- canary
  damaged with no surfaced error), uninitialized read SILENT_UB, integer
  overflow DIV/0/null/stack correctly panic.

### Backlog (compiler lane, ordered)

1. COMPILE-TIME / stdlib module graph (top priority):
   - MEASURED (2026-10-03, debug driver, lz4 smoke via `XIOM_TIMINGS=1`):
     parse 0.003s; **check+borrow 10.61s** (check 10.16s = catalog +
     program, borrow 0.45s); codegen 0.18s; clang+link ~6.6s of 17.4s
     wall. The CHECKER -- overwhelmingly the injected catalog/stdlib
     bodies -- is the target, not parse/codegen.
   - MEASURED (2026-10-04, m190 tree, debug driver, ISOLATED project root;
     the 2026-10-03 "check 10.16s" was inflated by leftover `%TEMP%`
     stdlib copies being indexed -- W001 duplicates): hello wall 8.7s =
     index 0.75 + check ~0 + codegen ~0 + clang+link ~7.9; lz4 program
     wall 13.3s = index 0.84 + catalog-load 0.27 + **catalog-flush 2.56**
     + program bodies ~0 + borrow 0.76 + codegen 0.31 + clang+link ~7.7.
     The clang stage is dominated by the RUNTIME C RECOMPILE, not the
     program IR: 6 files / 364 KB take 5.5-6.3s at -O2 on EVERY build.
   - LANDED 2026-10-04 (`crates/xiom/src/rtcache.rs`): persistent runtime
     OBJECT cache -- runtime C compiled once per (clang path+version,
     compile flags, source content hashes) into `$HOME/.xiom/rtobj/<key>/`;
     later builds link the cached objects. hello 8.7s -> **0.97s** warm;
     lz4 program 13.3s -> **4.0s** warm; smoke_compress_lz4_snappy OK.
     Native non-static only; any cache failure falls back to the
     single-invocation C compile. New sub-phase marks: driver `index`;
     xiom-check `check-start/collect-sigs/catalog-load/catalog-flush/
     check-bodies` (XIOM_TIMINGS).
   - LANDED 2026-10-04 (second slice): persistent catalog index-header
     cache (`xiom-check/src/catalog.rs`; `$HOME/.xiom/catidx.txt`,
     identity-guarded by compiler version/OS/arch). Warm index phase
     0.70s -> **0.32s** (hits=2397 misses=0); cold fills the cache
     (hits=517 misses=1880). Edits invalidate via mtime/size. Driver
     opt-in; unit test + xiom-check 196/196 + catalog e2e 6/6 green.
   - PROFILE 2026-10-04 (lz4 warm): catalog-flush 1.9-2.3s over **31
     modules**; capture/restore of the per-module import context 0.26-0.32s;
     heavy-tailed -- `xiom.math` 0.68s alone, then io/string/num/collections
     module trees 0.07-0.16s each; the rest ~50ms avg. The module-level
     slow-item timing shows the whole `module xiom` tree as one item, so a
     function-level profile is the next step if we attack math directly.
   - LANDED 2026-10-04 (run-path fix): `xiom run` / script compiles no
     longer register temp-root ancestors as source dirs. A stray `.xi` in
     `%TEMP%` used to make the grandparent guard add the whole temp root:
     9 competing stdlib copies across `%TEMP%\kilo` trees, 85 W001
     collisions, ~140s cold index on every cache miss (benchmark scripting
     samples 7-9.6s). Cold `xiom run` now 3.95s, index 2565 visits, W001 0.
     See COMPILER_BUGS 2026-10-04.
   - LANDED 2026-10-05 (scripting baseline): `xiom run --jit --cache`
     (the benchmark's scored `xiom-run` lane) now serves the script cache
     on hit and AOT-compiles+caches on miss; plain `--jit` stays the
     cold-JIT measurement per C25. Local: warm replay **0.11s** vs 3.2s
     cold JIT; the lane should drop from 2.2-9.9s per sample to the
     `xiom-run-cache` band (10-20ms warm).
   - PROFILE 2026-10-05 (in-container release build, t6 script): wall
     2.22s = parse 0.03 + index 0.12 + catalog-load 0.14 + borrow 0.06 +
     codegen 0.02 + clang/link 0.81 + **catalog-flush 1.06** (22 modules).
     `xiom.math` alone 0.49s and it is NOT fn bodies (no fn >30ms): it is
     its ~25 submodule `use`s at 10-40ms each during catalog-body
     checking (per-use `register_fn_signature` over every item +
     export-map build). Next: skip catalog-body uses the body never
     references, dedupe registrations, and/or memoize per-module export
     maps. t3-hot-reload's 216ms system-arena sample needs the arena's
     exact sample command/solution (relay) before attributing it.
   - LANDED 2026-10-05 (catalog-flush): unreferenced catalog-body `use`
     declarations are SKIPPED. Each body computes a reference set once
     (`crates/xiom-check/src/type_qualify.rs::catalog_reference_names` --
     every identifier plus every use-path root, so a use another use needs
     as a prefix stays); a `use` runs only when the name it binds (alias or
     leaf) is in that set. Globs always run. Guard: when the body's bare
     TYPE surface is not self-contained -- a bare type that is neither
     declared in the body, a scalar, nor a declared generic parameter
     (`referenced_type_names` minus `declared_type_leaves` minus
     `declared_generic_params`) -- NO use in that body is skipped. Bare fn
     names leak through the global `functions` registry; types do not:
     the strict corpus gate caught serialize/json's `Map` falling back to a
     same-leaf foreign declaration once `use xiom.collections;` was skipped.
     Local (debug driver, t6 template, `run --no-cache`, XIOM_TIMINGS):
     catalog-bodies 1.114s -> 0.682s (uses-skipped=53; xiom.math's body
     leaves the slow list), catalog-flush 1.345s -> 0.931s, check 2.214s ->
     1.560s. Gates: strict `catalog_corpus_is_clean` green, e2e 2418/0/4,
     feature 519, verifier 34+4, driver 58, checker 196.
     Still open: in-container release re-measure; if the <1.5s scripting
     wall misses, dedupe per-module signature registration and memoize
     per-module export maps.
   - NEXT: per-module CHECKED cache (diagnostics keyed by source hash +
     checker identity) needs the generic-instantiation state-effects audit
     before skipping bodies; alternatively investigate `xiom.math` body
     check directly (671 lines / 61 fns for 0.68s). `-e` fast path;
     remaining index walk/stat cost (~0.3-0.5s).
   - Persistent CHECK cache for stdlib modules keyed by source hash +
     compiler identity + config (same identity scheme as `xiom::jit`
     script cache): skip re-checking the injected catalog graph; pair
     with a per-module IR/object cache for the codegen side.
   - Separate compilation: link precompiled stdlib objects instead of
     re-emitting the graph; ship a precompiled stdlib in the release
     archive, generated on install from the pinned source.
   - Parallel module codegen by default (`e2e_i2_parallel_codegen` already
     covers the emitter; wire the driver to it for stdlib-graph builds).
   - `xiom run` / `xiom run -e`: thin path over the precompiled stdlib;
     stop indexing unrelated `.xi` trees under the temp root (see
     COMPILER_BUGS 2026-10-03 `-e` entry).
   KPI: arena compile_ms under 2 s for the six system tasks (from 6-14 s);
   `xiom run -e` under 1 s warm / under 3 s cold.
2. t1-allocator runtime (71 ms vs c 29): profile the allocator
   implementation plus codegen hot loops (bounds checks, branch layout,
   inline policy); consider checker-proven unchecked iteration in release.
   KPI: within 1.5x of c.
3. t3-hot-reload (256 ms): confirm task semantics; if per-reload latency,
   covered by item 1.
4. SAFETY HARDENING (post-selfhost, per owner):
   - compile prevention (0% now): expand move/free diagnostics (existing
     mutation/borrow queue items); SAFE_SUBSET probes should be
     compile-rejected wherever the API makes the bug impossible.
   - silent corruption: allocator poison + guard words with double-free /
     UAF traps in debug; runtime tag checks for generic enum/container
     payloads (type confusion).
   - uninitialized reads: definite-assignment diagnostic (checker).
    - arena/handle overflow audit (COMPILER_BUGS 2026-10-04): guard_alloc
      alignment-wrap bound check (stdlib lane, 1 line) + generation tags
      for flat-arena handles (ABA hardening).
    - detector disclosure: canary all probes in the arena; add `--sanitize`
      builds as a detection row in the matrix.
   KPI: silent_corruption 0; SAFE_SUBSET detection >= 80%.
5. CONTRACTS ARENA (queued, tooling batch): the verifier now emits
   `(check-sat)` and proves 3 contracts, but invalid SMT
   (`unknown constant self`) plus X7007 loop/body limits keep it at
   3 proven / 9 unknown / 3 errors; the contracts run records the t1 trial
   as UNKNOWN.
