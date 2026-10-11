# rustc study 02: parallelisation and partitioning

Study of how rustc bounds and schedules parallel codegen, why its
partitioning is deterministic, and what that implies for XIOM Stage 6
(parallel monomorphisation/emission, linker strategy) and stages 8-9
(selfhost byte-identical IR).

- rustc: `E:\repos\rust` @ `7211ea594cbf137ef6118b1752b959dea7cb9088`
  (sparse clone; never ran git writes).
- XIOM: worktree `study-rustc-parallel` @ `495d6672` (v0.64.3 tip).
- Companion index: `docs/RUSTC-REFERENCE.md` (main worktree copy at
  `E:\xiom-lang\xiom\docs\RUSTC-REFERENCE.md`; not in this branch tree).
- Scope: partitioning, parallel front-end/backend infrastructure,
  jobserver, determinism of partitioning and codegen.

## 1. Planning input

Highest-value adoptions for XIOM, tagged by stage:

- [S8-9] Establish canonical order BEFORE parallel work, never after:
  rustc's parallel mono-item walk is unordered on purpose and then the
  whole set is re-sorted by stable hash key (`collector.rs:1941-1944`).
  XIOM parallel emission must keep the same invariant: the worker only
  fills slots of a pre-ordered, pre-sized plan. Our declaration-order
  `sort_by_key(idx)` (`xiom-codegen/src/lib.rs:5699-5700`) is the right
  shape; it is simply not locked or invarianted.
- [S8-9] Serial-equivalence discipline: rayon helpers run the first task
  inline and spawn the rest in REVERSE order so single-worker execution
  matches serial rustc ("This ensures the execution order matches that
  of a single threaded rustc", `sync/parallel.rs:87-91`); `ParallelGuard`
  defers panics to preserve error order (`sync/parallel.rs:17-36`).
  Byte-identical IR needs the stronger version of this: worker naming
  and state must be functions of the plan only, so serial and parallel
  text coincide by construction, not by scheduling luck.
- [S6] The parallel unit is a bounded, cost-estimated work item, not a
  raw function: CGUs (default 16, `session.rs:1183-1248`), cost-sorted
  queue (`write.rs:1624-1637`), `queue_full_enough` scales queue depth
  to workers (`write.rs:1787-1810`). XIOM's per-function rayon tasks
  have no cap and produce a single monolithic artifact, so there is
  nothing to schedule.
- [S6] Separate frontend, backend and linker job limits
  (`Jobs { frontend, backend, linker }`, `config.rs:1733-1737`);
  frontend defaults to 1 thread, backend to all CPUs; `-Z threads` is a
  deprecated alias for `--jobs-frontend` (`options.rs:2936`). XIOM's
  single `--jobs` only feeds the parse pool.
- [S6] Jobserver as process-tree rate limiter: rustc inherits or creates
  one jobserver and every substantive LLVM/linker worker holds a token
  (`interface.rs:385-393`, `jobserver.rs:1-60`, `write.rs:1401-1412`).
  XIOM has no token accounting; a future clang pool must not
  oversubscribe a build that already runs under `cargo`/CI.
- [S7] Reachability is the gate to codegen: partitioning consumes the
  mono-item set produced by a reachability walk (Lazy strategy,
  `collector.rs:141-150`); unreachable items are never placed, never
  codegen'd. XIOM compiles every non-generic function with a body
  (`lib.rs:5517-5563`) -- this is the -1854 emission cascade.
- [S8-9] Per-symbol decisions (linkage, visibility, internalization)
  happen once, single-threaded, before any parallel codegen
  (`partitioning.rs:201-313`, `528-614`). Workers never re-decide
  identity. XIOM per-function emitters currently re-derive symbol,
  string, temp, label and unsafe-block identity independently.
- [S6] Linker strategy: rustc does NOT embed lld for host targets; it
  uses the platform linker via the cc driver (`linker_and_flavor`,
  `link.rs:1848`) and ships `rust-lld` for self-contained/cross targets.
  Linker parallelism is exposed only through jobserver tokens when the
  flavor is lld (`link.rs:3168-3210`). XIOM already uses the clang
  driver; the open "embed lld vs clang pool" decision should be closed
  with this evidence rather than by intuition.

## 2. How rustc does it

### 2.1 Partitioning pipeline

`collect_and_partition_mono_items` (`partitioning.rs:1169-1273`) runs,
inside one `par_join`, (a) `partition()` and (b) a symbol-distinctness
assert; a failure in either cancels both. `partition()`
(`partitioning.rs:143-199`) is: `place_mono_items` -> `merge_codegen_units`
-> `internalize_symbols` -> assert CGUs sorted by name.

A codegen unit is "a named set of (mono-item, linkage) pairs"
(`partitioning.rs:4-9`). Placement rules (`partitioning.rs:201-313`):

- The characteristic DefId is the owning module, so code from one source
  module lands together; methods are placed with their self-type
  (`characteristic_def_id_of_mono_item`, `partitioning.rs:638-702`).
- Two CGUs per module: one for non-generic ("stable") code, one for
  monomorphizations ("volatile", incremental-only, `partitioning.rs:230`).
- Linkage and visibility are assigned per mono item at placement
  (`mono_item_linkage_and_visibility`, `partitioning.rs:757-906`).
- Items eligible for inlining (`#[inline]` heuristic, module docs
  `partitioning.rs:84-93`) are copied into every CGU that reaches them
  with `Linkage::Internal` (`partitioning.rs:266-280`), so LLVM can
  inline without cross-module knowledge.
- After placement, per-CGU size estimates are computed
  (`partitioning.rs:294-296`); they drive merging and the final sort.

Merging and bounding (`partitioning.rs:317-508`):

- `max_codegen_units = sess.codegen_units()` (16 by default, 256 with
  incremental; `session.rs:1183-1248`). Why 16: power of two, "roughly a
  local optimum", and the jobserver prevents thread oversubscription;
  ThinLTO recovers the cross-CGU inlining lost to splitting
  (`session.rs:1200-1247`).
- Excess CGUs are merged into the smallest of the kept N by maximum
  inlined-item overlap, so duplicated inlined code is minimized
  (`partitioning.rs:330-382`).
- Non-incremental default builds additionally refuse CGUs smaller than
  `NON_INCR_MIN_CGU_SIZE = 1800` (`partitioning.rs:389-418`); the
  `CodegenUnits::Default` vs `User` distinction (`session.rs:578-593`)
  is what makes this legal only when the user did not ask for a count.
- Non-incremental CGUs are renamed `crate-<hash>-cgu.NN`, reverse-sorted
  by size with zero-padded suffixes, so name order encodes size order
  (`partitioning.rs:471-507`).
- `internalize_symbols` demotes every symbol with no user in another CGU
  to internal linkage, giving LLVM more optimization freedom
  (`partitioning.rs:528-614`).

### 2.2 Determinism machinery

- Insertion order of CGUs comes from a parallel-friendly unordered map;
  it is normalized with `tcx.with_stable_hashing_context(..)` and
  `collect_sorted(&mut hcx, true)` so the order is a stable hash of the
  CGU name, not a random seed (`partitioning.rs:290-292`). Unordered
  maps are iterated only through these sorted conversions.
- `merge_codegen_units` asserts its input is name-sorted before merging
  ("A sorted order here ensures merging is deterministic",
  `partitioning.rs:323-324`); sizes tie-break by the existing name
  order via stable `sort_by_key`.
- The final check in `partition`: `bug!` if CGUs are not sorted by name
  (`partitioning.rs:189-196`) -- determinism is an enforced invariant,
  not a comment.
- After all parallel backend work, the module list is re-sorted by name
  before any file is written: "Regardless of what order these modules
  completed in, report them to the backend in the same order every time
  to ensure that we're handing out deterministic results"
  (`write.rs:2158-2164`).
- The parallel mono-item walk (see 2.3) is explicitly re-sorted because
  "The set of MonoItems was created in an inherently indeterministic
  order because of parallelism" (`collector.rs:1941-1944`).

### 2.3 Parallel monomorphisation

`collect_crate_mono_items` (`collector.rs:1916-1954`): roots are found
from HIR (public non-generic items; eager mode adds everything for
`-Clink-dead-code`/incremental, `collector.rs:141-164`); the graph walk
runs `par_for_each_in(roots, ...)` (`collector.rs:1936`) over
`SharedState { visited, mentioned, usage_map }` guarded by locks
(`collector.rs:249`, `1928-1933`); `visited` insertion under the lock
gives exactly-once visitation (`collector.rs:361-364`). The unordered
result is then sorted via stable hashing (`collector.rs:1943-1945`).

So: parallelism is allowed only inside discovery, and the canonical
order is imposed at the boundary. Nothing downstream sees the race.

### 2.4 Front-end and backend infrastructure

- rustc vendors its thread pool as `compiler/rustc_thread_pool/` (a
  Rayon-core fork; README: "an unstable fork made for use in rustc").
- Front-end pool: `run_in_thread_pool_with_globals`
  (`rustc_interface/src/util.rs:214-348`) builds a pool with
  `num_threads(jobs_frontend)`, names threads "rustc", registers worker
  TLS session globals, and installs a deadlock handler for query-cycle
  reporting. `set_dyn_thread_safe_mode(jobs.frontend.is_some())` happens
  before any pool/Lock exists (`interface.rs:383`): when the frontend is
  single-threaded, the compiler runs in serial mode with no
  synchronization overhead.
- `sync/parallel.rs` exposes `par_join` (`106`), `par_fns` (`74`),
  `par_slice` (`126`), `par_for_each_in` (`172`), `par_map`, plus
  `ParallelGuard` (`17`). Every helper has a serial branch that runs the
  closure with the same guard, so behavior is identical with 1 thread.
  `par_fns`/`par_slice` deliberately reverse the spawned order so the
  single-worker schedule matches serial rustc (`74-100`, `145-166`).
- Backend pool: NOT the frontend pool. `--jobs-backend` (default all
  CPUs, 0 -> `available_parallelism`, `config.rs:1782-1807`,
  `parse_jobs_one` `1814-1843`) limits LLVM workers; `--jobs-linker`
  limits lld (`LinkerJobs`, `config.rs:1712-1728`). `--jobs` sets
  backend and linker; the frontend ignores it ("default to 1 thread
  irrespectively of `jobs` for now", `config.rs:1779`).

### 2.5 Backend work scheduling and the jobserver

The architecture comment at `write.rs:1293-1427` is the core reference:

- "LLVM work can be done on any thread. Codegen can only happen on the
  main thread. Each thread doing substantial work must be in possession
  of a Token from the Jobserver." (`write.rs:1300-1307`).
- A coordinator thread runs the message loop; the main thread is driven
  to codegen one CGU at a time (`CguMessage`) and hands the resulting
  LLVM work item back; worker threads are spawned per item under tokens;
  the compiler process's implicit token is "lent" to an LLVM worker when
  the queue is full enough so the main thread is never wasted
  (`write.rs:1365-1381`, `1487-1521`).
- Work items are queued cost-sorted (`module.cost()`, insertion at
  `write.rs:1633-1637`) and the queue depth heuristic scales with
  running workers to keep them fed without keeping too many LLVM modules
  alive (`queue_full_enough`, `write.rs:1787-1810`).
- Parallel backend only engages if `backend.supports_parallel()` and
  `--jobs-backend` is set (`write.rs:1229-1232`).
- The jobserver client is a per-process `OnceLock`, inherited from the
  environment (cargo/make) or created with `limit = max(frontend,
  backend, linker)`; the process always holds one implicit token so it
  cannot be starved (`interface.rs:385-393`, `jobserver.rs:1-60`).
- ThinLTO has its own coordinator and token loop (`do_thin_lto`,
  `write.rs:988-1146`); work is inserted by cost too (`1042-1044`).

### 2.6 The link step

`linker_and_flavor` (`link.rs:1848`) resolves the platform linker; for
host targets that is the system cc/ld/lld via the compiler driver.
`rust-lld` is used where target specs request it (wasm, arm-none,
uefi, etc. -- `rustc_target/src/spec/base/*.rs`) or with self-contained
linking. Linker parallelism exists only for LLD and only with
`--jobs-linker`: rustc acquires up to `limit - 1` extra jobserver tokens
and passes the count to the linker ("Only LLD supports controlling
parallelism at the moment", `link.rs:3168-3210`); the default is
`LinkerJobs::Default` for backwards compatibility (`config.rs:1804-1806`).

### 2.7 IR verification

`verify_llvm_ir` is a per-module config flag (`write.rs:102`, set from
`-Z verify-llvm-ir` at `write.rs:225`), passed to the LLVM writer so an
invalid module fails with a compiler diagnostic before objects exist.
This is the "verify before handing to an external tool" pattern already
recorded in RUSTC-REFERENCE.md S8.

## 3. How XIOM does it today

### 3.1 Flags and wiring

- `--parallel` = rayon lex+parse across source files, only when
  `file_count > 1` (`crates/xiom/src/lib.rs:407-488`). `--jobs` sets the
  process-global `RAYON_NUM_THREADS` only inside that branch
  (`lib.rs:409-417`) -- meaning `--jobs` alone does nothing for codegen.
- `--parallel-codegen` = per-function IR emission
  (`crates/xiom/src/main.rs:1844-1848`; config field
  `crates/xiom/src/lib.rs:124`, default false at `:177`; forwarded at
  `:729`/`:1230`; codegen flag `crates/xiom-codegen/src/context.rs:112-113`,
  default false at `:191`). It is opt-in; the driver never enables it
  for stdlib-graph builds (`STAGE6_PERF_PLAN.md:330-331` is still open).
- No frontend/backend/linker split, no jobserver, no dedicated pool:
  both phases share the global rayon pool.

### 3.2 Parallel emission (xiom-codegen)

`compile_program` (`crates/xiom-codegen/src/lib.rs:4875`) assembles ONE
module into ONE `String` (`IrEmitter.output`, `lib.rs:42`). Serial path:
`for item in &program.items { self.compile_top_decl(item)?; }`
(`lib.rs:5196-5198`). Parallel path (`lib.rs:5191-5193` ->
`compile_functions_parallel`, `lib.rs:5569-5725`):

1. Collect every non-generic fn with a body by AST walk; `idx` is
   declaration order (`collect_functions_to_compile`, `lib.rs:5517-5563`).
2. Pre-assign symbols on the main emitter with first-wins dedup
   (`preassign_fn_symbols`, `crates/xiom-codegen/src/decl.rs:324-...`);
   the parallel pre-pass repeats the same rule and drops duplicate
   definitions (R49, `lib.rs:5583-5618`).
3. `functions.par_iter()`: each task builds a fresh `IrEmitter`, seeded
   with cloned resolution state (`use_alias_map`, `fn_symbol_map`,
   `generic_fn_decls`, `bare_fn_aliases`, `fn_typed_params`,
   `prepass_call_types`, CTFE snapshot, module constants;
   `lib.rs:5640-5656`), `current_module` set from the walk prefix, and
   counter offsets keyed on `idx` (`str_counter = idx * 1000`,
   `tmp_counter`/`block_counter = idx * 10000`,
   `unsafe_block_counter = idx * 1000`; `lib.rs:5658-5670`).
   `mono.emitted_fns` is seeded with a `HashSet` snapshot of names seen
   before this fn (`lib.rs:5672-5676`).
4. Merge: `sort_by_key(idx)` then append output, generic
   instantiations, used builtins, strings and deferred types in idx
   order (`lib.rs:5695-5720`).

Determinism status of this path:

- Run-to-run the plan is stable (AST order is stable, `sort_by_key` is
  stable), and the offset counters are compile-time functions of `idx`,
  so a repeated parallel build should emit identical text. Nothing
  asserts this.
- Serial vs parallel are NOT byte-identical by construction: counter
  offsets change every `%tmpN` (emitter.rs:10-14), every label, and
  every `@.strN` (`push_string_global`, `lib.rs:8855-8863`). The
  serial path also accumulates cross-body state (types registered while
  compiling earlier bodies, emitted_fns growth), while parallel workers
  see a frozen snapshot plus the pre-assigned map. So the two modes are
  different artifacts, not two schedules of one artifact.
- XIOM has already fought order-dependence within one mode:
  `pick_deterministic` exists because HashMap iteration order produced
  `%struct.Message` vs `%struct.BST` across runs -- "the emitted IR was
  not byte-identical" (Round 76 note, `lib.rs:3422-3437`). The lesson
  recorded there -- sort candidates before picking -- is exactly the
  rustc `collect_sorted` pattern; it is applied ad hoc, not as an
  invariant.

### 3.3 Verification and the clang/link step

- The driver runs `opt -passes=verify -disable-output` (legacy `-verify`
  fallback); if neither spelling is supported it skips silently with a
  warning -- verification is best-effort (`crates/xiom/src/lib.rs:1339-1378`).
- ONE clang invocation compiles the merged module AND links: runtime C
  object cache paths or C sources, asm objects, staged IR, `-o`; the
  whole `cmd` is built in `crates/xiom/src/lib.rs:1441-1684` (staged IR
  copy at `1641-1653`). There is no per-module IR split and no parallel
  object compile.
- `rtcache` compiles the runtime C sources on a cache miss in a
  sequential loop, one clang invocation per file
  (`crates/xiom/src/rtcache.rs:127-148`); hits append cached objects to
  the single link command (`crates/xiom/src/lib.rs:1605-1628`).
- The JIT crate's header advertises a "Persistent clang process pool"
  (`crates/xiom-jit/src/lib.rs:5-9`) but the implementation spawns
  one-shot clang commands (`xiom-jit/src/lib.rs:199-222`, `529-548`);
  no pool exists for AOT either.

### 3.4 Locks and coverage

- `e2e_i2_parallel_codegen` (`crates/xiom-codegen/tests/e2e_tests.rs:1691-1708`)
  compiles 5 chaos tasks with `--parallel-codegen` and asserts exit 0;
  `e2e_parallel_multi_fn` (`:1744`) similar. Behavioral only: no
  serial-vs-parallel IR diff, no repeatability check, no unit-order
  invariant.

### 3.5 Roadmap vs reality

- M26.5 "Parallel codegen" (`docs/ROADMAP.md:931`): implemented opt-in,
  per-function granularity, one artifact.
- M26.6 "split .ll per function, compile in parallel pool"
  (`ROADMAP.md:932`): not implemented.
- M26.1/3/4 thread pool/dep graph/parallel check: not implemented
  (parse-only `--parallel`).
- Stage 6 line "parallel monomorphisation behind rayon; linker strategy
  decision" (`docs/COMPILER_READINESS_PLAN.md:193-199`): open; the
  monomorphisation worklist itself is sequential
  (`crates/xiom-codegen/src/context.rs:351-384`,
  `generic_instantiations`), and only body emission can run in rayon.

## 4. Gap table

| pattern | rustc ref | XIOM state | gap | stage | effort | priority |
|---|---|---|---|---|---|---|
| Reachability-pruned unit set before codegen | collector.rs:1916-1954, partitioning.rs:201-313 | every non-generic fn with a body is emitted (lib.rs:5517-5563) | dead code emitted; -1854 bloat; wasted parallel work | S7 | L | P0 |
| Canonical order imposed at the parallel boundary | collector.rs:1941-1944; partitioning.rs:290-292 | AST declaration order + stable sort_by_key; unasserted | works today; no invariant/check | S8-9 | S | P1 |
| Bounded emission units (CGU analog) with size estimates | partitioning.rs:143-199, 344-418; session.rs:1183-1248 | per-function rayon tasks; one output module | no unit concept, no cap, no cost model | S6 | L | P0 |
| Linkage/visibility/TLS/name decided once pre-parallel | partitioning.rs:757-906, 528-614 | each per-fn emitter decides symbols/strings/counters independently | serial != parallel artifacts; order-dependent identity risk | S8-9 | M | P0 |
| Serial-equivalence ordering (reverse spawn, guarded panics) | sync/parallel.rs:17-36, 74-100 | merge sorted by idx only | no serial==parallel equality guarantee or lock | S8-9 | M | P0 |
| Cost-ordered work queue + adaptive depth | write.rs:1428-1659, 1787-1810 | none (one clang invocation) | no scheduler; single work item | S6 | M | P1 |
| Jobserver tokens for nested tools | jobserver.rs:1-60; write.rs:1233-1240; interface.rs:385-393 | none | oversubscription risk once a clang pool exists | S6 | S | P1 |
| Frontend/backend/linker job split | config.rs:1733-1807; options.rs:2936 | single --jobs, parse path only; global env mutation | no phase control; embedder hazard (MCP/LSP share the pool) | S6 | S | P1 |
| Per-unit artifact cache (work products) | write.rs:2165-2173 | rtcache (runtime C) and script cache only | no IR/object unit cache for recompiles | S6/S7 | M | P1 |
| Verify IR before external tool | write.rs:102, 225 | `opt -passes=verify`, may silently skip | weaker guarantee than -Z verify-llvm-ir | S7 | S | P2 |
| Artifact order independent of completion order | write.rs:2158-2164 | single artifact today | needed once IR is split per unit | S6 | S | P2 |
| Linker strategy decision recorded | link.rs:1848, 3168-3210; target specs | clang driver; `-fuse-ld=lld` only under `--lto` (lib.rs:1500-1505) | decision doc still open | S6 | S | P2 |
| Parallel runtime-C build (cold path) | jobserver model | sequential loop (rtcache.rs:130-148) | first-build 5.5-6.3s serial | S6 | S | P2 |
| Byte-identical IR lock (serial vs parallel) | n/a (rustc does not promise this) | none | selfhost precondition unmet | S8-9 | M | P0 |

Effort: S = days, M = 1-2 weeks, L = multi-week slice. Priority is for
Stage 6/8-9 ordering in this worktree, not a release commitment.

## 5. Optimization & performance opportunities

Measured baselines to budget against (all from `STAGE6_PERF_PLAN.md`):

- lz4 smoke, release, true cold (`run --no-cache`): wall 2.94s = index
  0.576 + check 1.148 + borrow 1.235 + codegen 1.327 + rt-cache HIT +
  clang/link/run ~1.6 (`STAGE6_PERF_PLAN.md:478-483`).
- Arena samples are 7-8s cold (whole-stdlib re-check ~1.3s + codegen +
  `clang -c -O2` ~0.9s + link, on a loaded box; `:344-354`).
- t6 template, release: emitted IR 668 KB / 17k lines / 222 fns;
  `clang -c -O2` 0.86s; link ~0.6s (`:315-322`).

Opportunities, in dependency order:

- O1 (P0, S7): reachability prune before emission. Less IR text, less
  clang time, less parallel work, and it directly attacks the -1854
  bloat. Budget proposal: t6-template emitted IR bytes must not exceed
  the current 668 KB; hello stays at its current size; full e2e and
  diff suites green; a `--link-dead-code`-style escape keeps eager
  parity for tests.
- O2 (P0, S6): split the single .ll into per-unit files and compile
  them with a token-bounded clang `-c` pool, then link objects in unit
  order (M26.6). Budget proposal: t6 template codegen+clang+link wall
  <= 1.2s warm on the dev box (from ~1.5s serial); object bytes and
  program behavior unchanged; IR unit files stable across runs.
- O3 (P1, S6/S8-9): enable `--parallel-codegen` for stdlib-graph builds
  only after the byte-parity lock (gap table row 1/key 4-5). Budget:
  emission of 222 fns at N=#CPUs <= 50 percent of serial emission wall;
  parallel IR text delta vs serial = 0 (byte-identical).
- O4 (P1, S6): parallelize the cold runtime-C build (6 files;
  `rtcache.rs:130-148`): one-time 5.5-6.3s -> under 2s with a few
  concurrent `clang -c` jobs; no change to hit path.
- O5 (P2, S6/S7): per-unit object cache keyed by per-module content
  hash + compiler identity (the rtcache identity pattern
  `rtcache.rs:74-96` extended to program units). Target: edit one
  module -> one unit recompiles + link. This is the structural partner
  of the Stage 6 persistent check cache (`STAGE6_PERF_PLAN.md:533-566`).

Honesty note: all numbers above are dev-box measurements recorded in
`STAGE6_PERF_PLAN.md`; budgets are proposals to be re-measured in the
acceptance container before being treated as KPIs.

## 6. Adoption proposals

Concrete, each naming the crate/module to touch. Nothing here changes
behavior until its acceptance gate is added.

1. [S6] Emission units in `crates/xiom-codegen` (new module, e.g.
   `src/units.rs`): group functions and module-scoped globals into
   deterministic units keyed by module path (and a volatile flag for
   generic instantiations, mirroring rustc's stable/volatile split);
   cap the unit count from the backend job limit; merge tiny units when
   the count is not user-specified (rustc `partitioning.rs:389-418`
   semantics). `compile_functions_parallel` iterates units instead of
   raw functions.
2. [S8-9] Pre-pass decision table in `crates/xiom-codegen`: move ALL
   identity/allocation decisions for symbols, strings (`@.strN`),
   `%tmpN`, labels, unsafe-block ids and TLS mode into the main emitter
   before workers start; delete the `idx * K` counter offsets
   (`lib.rs:5658-5670`) and hand workers read-only tables. Serial and
   parallel naming then coincides by construction.
3. [S8-9] Byte-parity lock: extend `e2e_i2_parallel_codegen`
   (`crates/xiom-codegen/tests/e2e_tests.rs:1691-1708`) or add to
   `crates/xiom-codegen/tests/diff_tests.rs`: compile the corpus serial
   and parallel, assert `--emit-ir` stdout byte-identical; compile
   parallel twice, assert identical; add a sorted-unit-order debug
   assert mirroring `partitioning.rs:189-196`.
4. [S7, -1854] Reachability collector in `crates/xiom-codegen` (or a
   shared pass fed by `xiom-check`): roots = main + pub + extern +
   hot-reload entries + fn-references taken; walk the resolved-call
   tables already threaded to the emitter (`catalog_resolved_calls`,
   `program_bare_targets`, `crates/xiom/src/lib.rs:738-739`); feed
   `collect_functions_to_compile` only the reachable set. Keep an eager
   `--link-dead-code` switch for parity testing.
5. [S6] Driver jobs split in `crates/xiom/src/cli.rs` + `main.rs`:
   `--jobs-frontend` / `--jobs-backend` / `--jobs-linker`; defaults 1 /
   all CPUs / linker-default; stop mutating the process-global
   `RAYON_NUM_THREADS` (`crates/xiom/src/lib.rs:409-417`); build
   dedicated rayon pools per phase so embedding tools (xiom-mcp, LSP)
   are not affected.
6. [S6] Jobserver in `crates/xiom/src`: add the `jobserver` crate,
   initialize once with `max(frontend, backend, linker)` as early as
   possible (mirror `interface.rs:385-393`); every clang/lld child
   acquires a token; pass the token count to lld when the flavor is
   lld. For XIOM's scale a token semaphore suffices; no coordinator
   thread needed until O2 lands.
7. [S6] Per-unit object pipeline in `crates/xiom/src/lib.rs`: extend
   the codegen boundary to return `Vec<(unit_name, ir_text)>` alongside
   `compile_program`; stage per-unit `.ll`, run the token-bounded
   `clang -c` pool, collect objects in unit order, single link. Keep
   the monolithic path as fallback and as the byte-parity reference.
   This is M26.6 made concrete.
8. [S6] `crates/xiom/src/rtcache.rs:130-148`: compile the runtime C
   files through the same token pool; output names are already stable;
   cold-only win.
9. [S6] Record the linker-strategy decision in
   `docs/COMPILER_READINESS_PLAN.md` Stage 6 with this study's
   evidence: stay on the clang driver for host builds (rustc parity),
   use `-fuse-ld=lld` where available (existing `--lto` path,
   `crates/xiom/src/lib.rs:1500-1505`), ship/pin lld for cross targets,
   and defer "embed lld" until O2 has measurements.

## 7. Open-item relevance

- REL-20261010-1854 (unused fn grows a binary 341 KB -> 1.9 MB; mixed
  TLS accesses; per RUSTC-REFERENCE.md S6/S7). Proposal 4 is the
  structural fix direction: rustc's Lazy reachability means an unused
  fn is never a mono item and never reaches partitioning. Parallel
  angle: every unused fn is currently compiled by a rayon task, so the
  bloat also multiplies parallel work and its emission feeds the
  byte-layout where the malformed TLS loads appear. Unit/TLS identity
  must be decided in the pre-pass (proposal 2): rustc sets the TLS mode
  once per declaration and every access uses the same model
  (`codegen_llvm/src/consts.rs:413,542`), while XIOM's per-function
  emitters can each emit `define`/`declare` variants whose dedup only
  happens on the main emitter (`crates/xiom-codegen/src/lib.rs:5102-5110`).
- REL-20261010-1900 (W004 cascades): if emission units start reporting
  failures independently, adopt rustc's deterministic error-order
  discipline so N workers do not change which error surfaces
  (`sync/parallel.rs:17-36`, `74-100`) and keep cascades gated behind a
  proven-error token (RUSTC-REFERENCE.md S1, ErrorGuaranteed).
- REL-20261010-1904 (float cast fn-pointer returns): not
  parallel-specific, but unit boundaries must not re-derive fn-pointer
  ABI; the FnAbi/return-mode decision belongs in the shared pre-pass
  (proposal 2) so every unit and both modes agree (RUSTC-REFERENCE.md S4).
- M26 / Stage 6 mapping: proposal 5 = M26.1; proposal 1 = M26.3/M26.5;
  proposals 1-3 = M26.5; proposal 7 = M26.6; O5 = M26.8 + Stage 6 item 1
  continuation; proposal 4 = the "parallel monomorphisation" line in
  COMPILER_READINESS_PLAN Stage 6 (reachability then per-unit worklist).
- Stage 7 / 8-9: proposal 3 is the selfhost gate prerequisite. Stages
  8-9 require byte-identical IR, which is strictly stronger than rustc's
  own guarantee; that is why the parity lock precedes any default-on
  parallel emission.

## 8. Sources

rustc (`E:\repos\rust` @ 7211ea594cbf137ef6118b1752b959dea7cb9088):

- `compiler/rustc_monomorphize/src/partitioning.rs` (1-93 docs, 143-199
  partition, 201-313 place, 290-292 stable-hash sort, 317-508 merge,
  389 min-size, 471-507 renames, 528-614 internalize, 638-702
  characteristic def id, 757-906 linkage/visibility, 1169-1273
  collect_and_partition_mono_items)
- `compiler/rustc_monomorphize/src/collector.rs` (141-164 strategies,
  249 SharedState, 355-374 root dedup, 1916-1954 parallel walk + sort)
- `compiler/rustc_data_structures/src/sync/parallel.rs` (17-36
  ParallelGuard, 74-100 par_fns, 106-124 par_join, 126-170 par_slice,
  172-197 par_for_each_in)
- `compiler/rustc_data_structures/src/sync.rs` (85-123 dyn thread-safe
  mode)
- `compiler/rustc_data_structures/src/jobserver.rs` (1-60)
- `compiler/rustc_thread_pool/` (README, lib.rs)
- `compiler/rustc_interface/src/interface.rs` (379-395)
- `compiler/rustc_interface/src/util.rs` (214-348)
- `compiler/rustc_session/src/config.rs` (1712-1807, 1814-1843)
- `compiler/rustc_session/src/options.rs` (2936 `threads` deprecation)
- `compiler/rustc_session/src/session.rs` (578-593 CodegenUnits,
  1183-1248 defaults)
- `compiler/rustc_codegen_ssa/src/back/write.rs` (102/225
  verify_llvm_ir, 988-1146 thin LTO, 1203-1291 start_executing_work,
  1293-1427 architecture, 1428-1659 coordinator loop, 1633-1637
  cost-sorted queue, 1787-1810 queue_full_enough, 2158-2164 final sort,
  2199-2205 submit_codegened_module_to_llvm)
- `compiler/rustc_codegen_ssa/src/back/link.rs` (1232 link_natively,
  1848 linker_and_flavor, 3168-3210 lld parallelism via jobserver)

XIOM (this worktree @ 495d6672):

- `crates/xiom-codegen/src/lib.rs` (17 rayon import, 42 output String,
  4875-5303 compile_program, 5102-5110 global dedup, 5191-5198 mode
  gate, 5317-5393 hoist_static_allocas, 5517-5563 fn collection,
  5569-5725 compile_functions_parallel, 5658-5670 counter offsets,
  5695-5720 merge, 8855-8863 push_string_global)
- `crates/xiom-codegen/src/context.rs` (12-75 SyncRegistry, 112-113 +
  191 parallel_codegen flag, 351-384 generic_instantiations worklist)
- `crates/xiom-codegen/src/decl.rs` (324-... preassign_fn_symbols)
- `crates/xiom-codegen/src/emitter.rs` (10-19 fresh_tmp/fresh_block)
- `crates/xiom-codegen/tests/e2e_tests.rs` (1691-1708
  e2e_i2_parallel_codegen, 1744 e2e_parallel_multi_fn)
- `crates/xiom/src/lib.rs` (124/177 parallel_codegen config, 407-488
  parallel parse, 409-417 RAYON_NUM_THREADS, 714-748 emitter setup,
  729/1230 set_parallel_codegen, 1339-1378 opt verify, 1441-1684 single
  clang invocation, 1500-1505 -fuse-ld=lld under --lto, 1605-1628
  rtcache objects)
- `crates/xiom/src/main.rs` (1844-1848 flags, 2674-2677 help text)
- `crates/xiom/src/cli.rs` (100-101 flag set)
- `crates/xiom/src/rtcache.rs` (74-96 cache key identity, 127-148
  sequential runtime-C compile)
- `crates/xiom-jit/src/lib.rs` (5-9 pool claim in header, 199-222 and
  529-548 one-shot clang spawns)

XIOM docs consulted: `docs/COMPILER_READINESS_PLAN.md` (Stage 6/7),
`docs/ROADMAP.md` (M25/M26, v0.56 threading status),
`docs/STAGE6_PERF_PLAN.md` (item 1 history, measured baselines,
2026-10-09/10 intakes), `docs/SCALING_ARCHITECTURE.md` (per-file
compilation-unit design), `docs/THREADING_PLAN.md`,
`docs/COMPILER_ARCHITECTURE.md` (CTFE cache note for parallel codegen),
`docs/RUSTC-REFERENCE.md` (main worktree copy; REL map S1-S8).
