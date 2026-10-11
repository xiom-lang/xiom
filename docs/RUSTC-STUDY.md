<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Rustc Study Program -- Consolidated Intake (2026-10-11)

Seven study agents (Agent Manager worktrees, branches `study/rustc-*`) compared
rustc (`E:\repos\rust` @ `7211ea594cbf137ef6118b1752b959dea7cb9088`) against
XIOM and produced per-area gap analyses. This file is the planning layer:
what is missing from XIOM, what to adopt, per roadmap stage, and where each
proposal lands. Per-area detail (all path-cited):
`docs/rustc-study/01..07-*.md`. Anchor index: `docs/RUSTC-REFERENCE.md`.

Tag legend: [S6] = Stage 6 performance program; [S7] = Stage 7 selfhost gate;
[S8-9] = stages 8-9 (selfhost chain: byte-identical IR, self1 == self2,
bootstrap). Accept/reject/defer happens in the planning docs
(`COMPILER_READINESS_PLAN.md`, `STAGE6_PERF_PLAN.md`); nothing here is
committed scope by itself.

## How to use

- Before implementing in an area, read that area's study doc; every adoption
  below carries the rustc anchor and the XIOM touch point.
- Open bus items have reference-backed fix directions (last section).
- The 7 study docs were landed into `docs/rustc-study/` by the consolidation
  commit; their worktree commits remain as provenance (hashes in section 6).

## 1. Stage 6 -- performance program: proposed additions

### 1.1 Incremental engine (study 01)
- Replace per-file mtime thinking with a **tracked read-graph**: record every
  dependency edge during compilation; the smallest viable unit is a dedup'd,
  ordered read list per task (inline buffer of 16, pooled beyond).
- **Node identity = (kind, stable hash of the key)** -- never dotted names.
  Highest-value slice 1: replace `signature_hash = hash(module_path + deps)`
  (`crates/xiom-graph/src/cache.rs:294-297`) with a structural interface hash
  (exported signatures in canonical order).
- **Recursive green marking** (a node is green iff every previous dependency
  is green; unknown dep -> recurse or force) -- today a one-level "entry
  exists" check invalidates whole files or nothing.
- **Value fingerprints** decide red/green after a recompute: re-executed node
  hashing to its previous value stays green; dependents not dirtied
  (comment/format-only edits become cheap).
- **Promoted roots**: on a hit, import the previous session's edges; re-
  serialize every green disk value into the new cache, or untouched entries
  are lost (rustc `graph.rs:837-854,1062-1091`; our save path only runs on a
  full compile today).
- Per-query **cache opt-ins** (`cache_on_disk`, `eval_always`, `no_hash`,
  `no_force`) instead of our bitmask with only `l4_ir` ever set; document an
  eligibility rule per tier.
- **Session-generation cache dirs** (`s-{ts}-{SVH}`, publish by rename,
  delete stale private dirs) replacing the flat 100 MiB mtime FIFO -- makes
  "cache consistent with source X" checkable.
- Gate cache load on a single **dep-tracking hash of the command line**.

### 1.2 Parallelism & scheduling (study 02)
- **Canonical order BEFORE parallel work**: workers fill slots of a
  pre-ordered, pre-sized plan; never sort after the fact (rustc re-sorts the
  mono set by stable hash key for parallel runs).
- **Serial-equivalence discipline**: rayon helpers run the first task inline
  and spawn the rest in reverse; panics are ordered. For byte-identical IR we
  need the stronger form: worker naming/state are functions of the plan only,
  so serial and parallel text coincide by construction.
- The parallel unit is a **bounded, cost-estimated work item** (CGU
  analogue, ~16 default, cost-sorted queue, queue depth scaled to workers).
  Our per-function rayon tasks are uncapped and produce one monolithic
  artifact -- nothing to schedule.
- **Separate frontend / backend / linker job limits**; our single `--jobs`
  feeds only the parse pool.
- **Jobserver tokens** as a process-tree rate limiter for the future clang
  pool (do not oversubscribe a build already running under cargo/CI).
- Linker strategy evidence: rustc uses the platform linker via the cc driver
  for host targets and `rust-lld` only for self-contained/cross targets;
  parallel linking is token-gated. Close the "embed lld vs clang pool"
  decision with this.

### 1.3 IR quality & optimization (study 03)
- **Make IR verification a hard gate**: `verify-llvm-ir` runs at pipeline
  start AND re-verifies every file write in rustc; ours is warning-only and
  skipped on `--emit-ir`. Add verifier-on-emit + verifier-before-clang (and
  an internal consistency pass: call/decl return+arg match, aggregate store
  field-type match -- would have caught m250/m254/m256 and 1904 at build
  time).
- **Type-derived optimization attributes** (`noalias`, `readonly`,
  `noundef`, `nonnull`, `dereferenceable`, `align`, `nofree`), emitted only
  when optimizing. Measured: XIOM emits ZERO today; only a heuristic
  alwaysinline/inlinehint exists.
- **Reachability-only emission + internal linkage**: rustc emits only
  reachable instantiated items and internalizes non-exported symbols; we
  emit every registered stdlib fn as externally visible (0 internal among
  147/287 defines), so GlobalDCE cannot drop anything -- the same class as
  REL-20261010-1854's unused-function cascade.
- **Cold panic paths**: branch to a `#[cold]` panic fn; mark our
  `@xiom_panic` cold/noreturn (declared bare today, though the runtime exits).
- Pipeline builder instead of bare `-O` (unlocks Os/Oz, merge-functions,
  per-function optnone later; parity at -O2 is plausible but these levers
  are unreachable today).

### 1.4 Measured baselines (recorded for Stage 6 budgets, study 03)
- 16-line program (core+io): **375,582 bytes IR / 5,313 lines / 147
  defines**; tests/ecosystem/t2-queue.xi: **591,812 bytes / 8,380 lines /
  287 defines**; 87% and 90% of defines are forced `alwaysinline`; a TLS
  recursion-counter load/store pair appears per fn (667 / 1,175 mentions).

### 1.5 Layout memoization (study 05)
- Interned type id + per-mono-instance layout cache; removes string
  round-trips (`parse_struct_field_types` split on `_`, three `join("__")`
  namers) without changing a single IR byte. rustc computes `layout_of` once
  per (type, env) and shares it.

### 1.6 Bootstrap-chain budgets (study 07)
- T3 suite is a full uncached sweep today (6 tests, ~6-13 min per run);
  O2 targets (<60 s self-compile; within 2x of xiom.exe) are measured
  nowhere. Adopt content/smart stamps for the chain.

## 2. Stage 7 -- selfhost gate: proposed additions

### 2.1 Error-token gating -- fixes the 1900 cascade class (study 04)
- Gate the Stage-6 lint wave (W002/W003/W004) on a **body-local error
  taint**, not on `checking_catalog`. Verified cascade mechanism: on a
  poisoned scrutinee `match_variant_names` returns empty
  (`xiom-check/src/lib.rs:9536-9560`) and `pattern_is_catch_all_v`
  (`:5897-5911`) then misclassifies a bare variant arm as a binding
  catch-all -> bogus W004. Design sketch with call-site conditions and
  minimal diff is in study 04 section 1.
- Single minting point for `ErrorGuaranteed` (ours is `pub`; rustc's field
  is private and only `DiagCtxtInner::emit_diagnostic` mints it).
- Phantom imports (feeds C-ORBIT-06): one batched error per use tree + a
  `Res::Err`-style placeholder binding, so later stages never see a hole;
  our `process_use` silently returns when the module fails to load.
- Warning dedup via a stable (code, parentless-span, message) hash set
  (replaces our per-warning linear scan); error-poisoned bodies must never
  enter the m252 body-check cache (add an explicit lock test).
- Optional XIOM analogue of `Level::DelayedBug` for silent recovery
  fallbacks.

### 2.2 Typed type layer -- fixes 1904 + the m-family structurally (study 05)
- Land a typed `LocalTypeTable`: binding sites (let/var, match bindings,
  params) record a STRUCTURED type (container base + element, tuple
  elements, fn-ptr signature), not a rendered name string; the three tuple
  namers and fn-ptr call paths read one record.
- 1904 root mechanism (verified in our code): `fn_ptr_return_types` is
  written only for `Type::Fn` params (`decl.rs:1764-1768`) and reset per
  function (`decl.rs:1295`); derive the call's return mode from the recorded
  fn-ptr signature at call emission (rustc `fn_abi_of_fn_ptr` analogue).
- Regression locks must assert IR shape (`call double %fnptr(`, tuple names),
  not only runtime exit codes; m254 needed three namer patches because the
  shared naming contract was unasserted.
- Sanity asserts before clang (`layout_sanity_check` / `fn_abi_sanity_check`
  analogues) for every call and aggregate store.

### 2.3 Check placement & profile semantics (study 06)
- Bounds checks at **place formation, once per index expression**, not
  per read/write site in codegen; provable ones disappear in mid-level
  passes. Expectation for the -O0/-O2/-O3 differential: overflow traps at
  O0 and wraps at O1+ (`-C overflow-checks` defaults to debug_assertions);
  bounds checks stay ON at every opt level; const bodies and div/rem are
  always checked.
- W004 usefulness rule to preserve: "a row is useful iff no (unguarded)
  rows above it" -- we already implement it; keep it in any W004 rework.
- If borrow checking over match results is strengthened: false edges from
  candidate leaves (rustc discipline) or the analysis is unsound.
- Borrowck shape: per-body `Result<_, ErrorGuaranteed>`, diagnostics
  suppressed on tainted bodies.

### 2.4 Selfhost infrastructure (study 07)
- **Pinned hashed seed** (stage0-manifest analogue): the bootstrap input
  must be a version+sha256-pinned artifact, not "whatever target/debug
  builds".
- **Frozen golden manifests**: compiletest `.stderr` discipline is already
  our pattern (`.expected`, T3 byte equality); make it the norm for every
  new gate; add a `--bless`-style regeneration tool so goldens cannot rot.
- **Crash triage pipeline**: any compiler panic -> ICE-style report
  (version/platform/flags/stack) + a `tests/crashes` corpus of minimized
  reproducers (1854 was manual binary archaeology; the pipeline is missing).
- **Differential suite as first-class** (`--compare-mode` analogue):
  -O0/-O2/-O3 x clang/gcc over the corpus, with a per-case manifest.
- **Fuzz**: >=1M-exec claims need a checked-in corpus + promotion policy
  (crash -> minimized reproducer -> `tests/crashes` -> fixed, promoted to
  the suite).
- **Stage-scoped assertions**: run gates on the SELF-built binary, not only
  the Rust-hosted one (rustc refuses `x test --stage 0` for the same
  reason); our T3/T6 self-arming manifests are the shape to extend.

## 3. Stages 8-9 -- selfhost chain: proposed additions

- **Bootstrap chain as a scripted, hash-checked gate**: self1 =
  xiom.exe(selfhost/src/main.xi); self2 = self1(selfhost/src/main.xi);
  sha256(self1) == sha256(self2) **plus** self1-vs-self2 IR T3. No runner
  exists today; the only precedent is `#[ignore]`d and checks a count.
- **Cross-artifact determinism gates**: "same binary, two runs" is not
  enough; compare xiom.exe vs self1 vs self2 over the same corpus, zero
  byte diffs; parallel output must equal serial output (rustc compares its
  parallel frontend against the serial one and uses `--iteration-count`).
- **Session-independent identity for package-level caching** (post-split):
  dep nodes must survive sessions without rebasing IDs; package identity
  needs the same property.
- **Mono-instance-keyed typed cache** (`Instance { def, args }` analogue) so
  an incremental cache can never leak one instantiation's types into
  another (the m250 fallout class).
- Keep later levers open by design: extra pass pipelines, PGO/coverage,
  per-function target features, Os/Oz.

## 4. Open bus items -- reference-backed fix directions

- **REL-20261010-1900** (bogus W004 before T001): study 04 section 1 --
  body taint + scrutinee-aware W004 + `match_variant_names_opt`; minimal
  diff sketched against current line numbers.
- **REL-20261010-1904** (Float64 through cast fn-ptrs): study 05 section 1 --
  record the fn-ptr signature at binding, derive `ret` at call emission;
  add the fp-return differential fixture.
- **REL-20261010-1854** (TLS mis-lower + 5.6x emission bloat): studies
  02/03 -- reachability-only emission + internal linkage + hard IR
  verification + crash triage pipeline; the emission-cascade is the same
  class as "emit every registered fn".

## 5. Cross-cutting rules adopted from the studies

1. No cascading diagnostics after an error (error tokens, taint).
2. Derive ABI/layout/typing from types; names are rendered last, never
   parsed back.
3. Canonical order before parallel work; serial equivalence by
   construction.
4. Emit only reachable items; verify IR before every hand-off.
5. Gates are frozen, machine-checked manifests; determinism is a
   cross-artifact comparison; crashes are triaged by pipeline, not by hand.

## 6. Provenance

| Study | Doc | Worktree commit |
|---|---|---|
| 01 incremental/query | `docs/rustc-study/01-incremental-query.md` | `ade40e1c` |
| 02 parallel/partitioning | `docs/rustc-study/02-parallel-partitioning.md` | `8e484f1d` |
| 03 opt pipeline/IR | `docs/rustc-study/03-optimization-ir.md` | `a501ef96` |
| 04 diagnostics/recovery | `docs/rustc-study/04-diagnostics-recovery.md` | `d5a4f162` |
| 05 ABI/layout/types | `docs/rustc-study/05-abi-layout-types.md` | `5148d362` |
| 06 MIR/safety | `docs/rustc-study/06-mir-safety-lowering.md` | `f0277d70` |
| 07 selfhost/infra | `docs/rustc-study/07-selfhost-bootstrap-infra.md` | `82a582ca` |

rustc reference: `E:\repos\rust` @
`7211ea594cbf137ef6118b1752b959dea7cb9088` (sparse: `compiler/`,
`src/bootstrap`, `src/tools/compiletest`; `tests/ui` readable lazily via
`git show`).
