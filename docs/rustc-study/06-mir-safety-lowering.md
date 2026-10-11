<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Rustc Study 06 -- MIR / match lowering + runtime safety checks (rustc vs XIOM)

Scope: how rustc inserts bounds/overflow/assert checks ONCE in a mid-level IR
(not the backend) and then optimizes provable ones away; how match lowering
binds arm variables from typed places; how exhaustiveness/reachability analysis
is gated on resolved types; and how MIR borrowck is structured. Findings feed
XIOM Stage 7 (selfhost gate: bounds/coherence enforcement, borrow check default
in strict profiles, -O0/-O2/-O3 differential) and the Stage 6 performance
program.

Rustc clone: `E:\repos\rust` @ `7211ea594cbf137ef6118b1752b959dea7cb9088`
(sparse `compiler/` + `src/bootstrap` + `src/tools/compiletest`). Paths below
are relative to that clone unless prefixed with `crates/` (XIOM). XIOM sources
verified against this worktree at main `ecc587ee`.

Tags in section 1: [S6] = COMPILER_READINESS_PLAN Stage 6 (performance
program), [S7] = Stage 7 (selfhost gate), [S8-9] = post-gate follow-ons
(split/registry hardening; the same patterns carry forward).

## 1. Planning input

- [S7] Exhaustiveness/reachability diagnostics are gated on resolved types by
  construction in rustc: `check_match` returns `Result<(), ErrorGuaranteed>`,
  starts with `tcx.thir_body(def_id)?`, and every caller short-circuits on any
  pattern error (`check_match.rs:30-32, 265-285, 508-520`). XIOM's W004 has no
  such gate today -- only `!checking_catalog` at the two call sites
  (`xiom-check/src/lib.rs:6676-6679, 9505-9508`) -- which is exactly the
  REL-20261010-1900 failure class. The fix shape is a checker "error state /
  Error-typed scrutinee" early return, mirroring rustc.
- [S7] Bounds checks belong in the mid-level IR, inserted once per index
  expression at place formation time (`as_place.rs:647, 727-756`), lowered by
  the backend to a branch + `panic_bounds_check` call
  (`codegen_ssa/src/mir/block.rs:753-808`). XIOM currently emits a fresh
  cmp/and/br/trap per read and per write site inside codegen (`expr.rs:3252-3268`,
  `stmt.rs:1196-1220`) and only under `--overflow-checks`.
- [S7] Overflow semantics in rustc come from one session flag with an
  opt-level-coupled default: `-C overflow-checks` (Option<bool>,
  `rustc_session/src/options.rs:2314-2315`) defaults to `debug_assertions`
  (`session.rs:1080-1081`), which defaults to `opt_level == No`
  (`config.rs:2961`). So -O0 traps, -O1+ wraps, and bounds checks stay on at
  every opt level (only provably-true ones disappear). Stage 7's -O0/-O2/-O3
  differential test must encode this as expected behavior for +,-,* and as an
  invariant for bounds/null/div-by-zero.
- [S6] Checks are cheap because they are plain MIR asserts plus a cold
  branch, not runtime transactions: MIR passes (constant condition
  simplification, const-prop, GVN, SSA range propagation) delete provable
  checks before codegen (`mir_transform/src/simplify_branches.rs:78-96`,
  `lib.rs:694-782`, `ssa_range_prop.rs:158`). Contrast XIOM's PERF-1
  confinement trampoline at ~2.4 us/call (`docs/STAGE6_PERF_PLAN.md:27-128`);
  check placement, not check existence, is the budget lever.
- [S6] Guarded arms do not cover later arms in rustc's usefulness engine:
  "A row is useful iff it has no (unguarded) rows above it" and
  `useful &= row.is_under_guard` (`rustc_pattern_analysis/src/usefulness.rs:1714-1726`).
  XIOM's W004 already implements the same rule (`xiom-check/src/lib.rs:5941-5945`)
  -- preserve it in any W004 rework.
- [S6] Match lowering emits false edges from every candidate leaf to the next
  candidate's start (and guard-false to the next candidate) purely so the
  borrow checker cannot exploit the lowered CFG
  (`matches/mod.rs:1581-1620`). Any XIOM borrow-check strengthening over match
  results needs the same discipline or it will be unsound (or over-eager).
- [S7] Borrow check is one query per body returning
  `Result<_, ErrorGuaranteed>` (`rustc_borrowck/src/lib.rs:115-118`), composed
  of three dataflow analyses over the same MIR as the checks
  (`dataflow.rs:15-23`), with diagnostics suppressed on tainted bodies
  (`root_cx.rs:74-80`). XIOM's checker is lexical (scope stack + loan set) and
  runs after typecheck, warning-only unless `--strict`
  (`crates/xiom/src/lib.rs:1136-1154`).
- [S7] Constant bodies always get overflow checks regardless of
  `-C overflow-checks` (`rustc_mir_build/src/builder/mod.rs:788-792`); div/rem
  are always checked in every profile (`as_rvalue.rs:530+`). XIOM needs the
  same carve-outs when profile semantics are defined.
- [S8-9] Arm bindings derive from typed places: `MatchPair`s carry a
  `PlaceBuilder` plus the pattern, and variant/field identity is
  variant-relative index, never a global name lookup (`match_pair.rs:70-90`,
  `prefix_slice_suffix` 16-68). The m250/m254/m256 regression family
  (name-string typing) is the XIOM evidence for moving match codegen the same
  way.

## 2. How rustc does it

### 2.1 Match lowering: THIR pattern -> MIR decision tree

- Entry `Builder::match_expr` (`compiler/rustc_mir_build/src/builder/matches/mod.rs:335`);
  the scrutinee is lowered to a place (`lower_scrutinee`, `:376-388`) and a
  `push_place_mention` records it.
- `lower_match_tree` (`:1547-1657`) assembles one top-level `Candidate` per
  arm (`:1560-1565`; sub-candidates appear later for or-patterns), then runs
  `match_candidates` (`:1721-1803`) which repeatedly picks a test
  (`pick_test`, `:2145`) and partitions candidates into outcome buckets
  (`buckets.rs:58-93`; unbucketed candidates stay in source order to preserve
  the "arms are tried in order" illusion).
- Tests are typed: `test()` maps `TestableCase` to `TestKind`
  (`test.rs:33-62`) -- `Switch` for enum variants, `SwitchInt` for
  int/char constants, `StringEq`, `ScalarEq`, `Range`, `SliceLen`, `Deref`,
  `Never`; `apply_test_cases` emits the actual `SwitchTargets`/branch CFG.
- Bindings and guards: `lower_match_arms` (`:396-483`) per arm declares
  bindings (`declare_bindings`, `:687`), then `bind_pattern` (`:499-551`).
  With or-patterns each sub-branch gets its own bindings and its own copy of
  the guard; `bind_and_guard_matched_candidate` (`:2388+`) evaluates the guard
  after binding and branches to the next candidate's otherwise-block on guard
  failure (`:2472`). Guard temporaries and binding drops are unscheduled
  between sub-branches (`clear_match_arm_and_guard_scopes`,
  `builder/scope.rs:1770-1783`).
- Borrow-checker hardening baked into lowering: false edges
  (`mod.rs:1581-1620`) and, for exhaustive matches, the failure block is
  `Unreachable` preceded by a fake read of the scrutinee (`:1622-1650`).
- Index/slice patterns project with `ConstantIndex`/`Subslice`
  (`match_pair.rs:16-68`) -- the place carries the layout-derived element
  identity; nothing is resolved by field name.

### 2.2 Exhaustiveness/reachability (the analysis W004 mirrors) + error gating

- `check_match(tcx, def_id) -> Result<(), ErrorGuaranteed>`
  (`compiler/rustc_mir_build/src/thir/pattern/check_match.rs:30-64`):
  `tcx.typeck(def_id)`, `tcx.thir_body(def_id)?` (`:32`), then a THIR visitor
  whose `error: Result<(), ErrorGuaranteed>` field (`:97`) accumulates the
  first error.
- `lower_pattern` (`:265-285`) checks `pat.pat_error_reported()` FIRST; a
  pattern whose type failed is never analyzed.
- `analyze_patterns` (`:396-421`) delegates to the shared
  `rustc_pattern_analysis` crate (`analyze_match`) and emits unreachable
  subpattern lints for `Usefulness::Useful(redundant_subpats)` arms; any
  `Err` propagates and stores the error.
- The match-expression driver (`:495-609`): per arm, `lower_pattern` failure
  aborts the whole check (`:508-517`); `analyze_patterns` failure aborts
  (`:520`); `report_arm_reachability` (`:532`, definition `:1190-1213`)
  warns on `Usefulness::Redundant` arms; missing witnesses produce the
  non-exhaustive diagnostic (`:598-607`). Guards are modeled by
  `has_guard` (`:511`) and, for the diagnostic note, by re-running the
  analysis with all guards removed (`:587-597`).
- Guard semantics live in the analysis engine: row usefulness is cut by
  guards (`usefulness.rs:1714-1726`, `is_under_guard` `:1105-1106, 1141`), so
  a guarded arm is treated as possibly-failing (later arms stay reachable)
  and is itself never flagged redundant.
- Net effect for REL-20261010-1900: rustc literally cannot emit
  reachability/exhaustiveness diagnostics from a body whose patterns or types
  failed -- every path is a `?` on `ErrorGuaranteed` or an early `return`.
  The gate is structural (Result/ErrorGuaranteed), not a string check for
  "unresolved".

### 2.3 Bounds check insertion: once, in MIR

- `lower_index_expression` (`builder/expr/as_place.rs:621-659`): the index is
  materialized into a FRESH temporary (`:638-645`, so nothing can mutate it
  after the check), then `bounds_check` runs before the index projection is
  formed (`:647`), followed by fake borrows that keep base/len stable
  (comment `:617-620`, `add_fake_borrows_of_base` `:758-815`).
- `bounds_check` (`:727-756`): `len` comes from `Rvalue::UnaryOp(UnOp::PtrMetadata, ...)`
  (`:711-719`), then `lt = index < len` and `msg = BoundsCheck { len, index }`,
  emitted through `self.assert(lt, true, msg, span)`.
- `assert` (`builder/scope.rs:1738-1763`) creates
  `TerminatorKind::Assert { cond, expected, msg, target: success, unwind:
  Continue }` and diverges; the assert is a normal MIR terminator, so every
  backend (LLVM, cranelift) lowers the same IR.
- Codegen (`rustc_codegen_ssa/src/mir/block.rs:753-834`): constant-true
  condition -> branch straight to the target, no panic block at all
  (`:767-780`); otherwise `cond_br` to a cold `panic` sibling block calling
  the `panic_bounds_check(index, len, location)` lang item
  (`:803-808`). LLVM then eliminates what its own passes can prove.
- `get_unchecked` bypass is not a backend trick: the check simply was never
  inserted because the index expression was lowered as a pointer op.

### 2.4 Overflow checks: one flag, three insertion points, profile-coupled default

- Flag plumbing: `-C overflow-checks` is `Option<bool>` marked TRACKED
  (`rustc_session/src/options.rs:2314-2315`); `Session::overflow_checks()` =
  `unwrap_or(debug_assertions)` (`session.rs:1080-1081`); `debug_assertions`
  defaults to `opt_level == OptLevel::No` (`config.rs:2961`). Explicit
  `-C overflow-checks=yes|no` overrides; at -O0 the default is checked, at
  -O1+ it is wrap.
- Where "check" is decided: `Builder::new` (`rustc_mir_build/src/builder/mod.rs:770-792`)
  computes `check_overflow = has #[rustc_inherit_overflow_checks] |
  sess.overflow_checks() | body is const/static`. The inherit attribute plus
  a codegen-side re-check exist because of cross-crate inlining of core ops
  (comment `:782-785`; codegen gate `block.rs:769-775`).
- Insertion points in `as_rvalue.rs`:
  - unary neg of MIN -> `AssertKind::OverflowNeg` (`:102-123`);
  - add/sub/mul on integral types -> `wrapping_to_overflowing` Rvalue
    producing `(val, of)`, then `assert(of == false, Overflow(op, lhs, rhs))`
    (`:457-480`);
  - shl/shr -> `inbounds = rhs < bits` then assert (`:481-529`);
  - div/rem -> ALWAYS inserted, not gated by overflow-checks: divide-by-zero
    and the signed `MIN / -1` overflow (`:530+`).
- `AssertKind` definition: `compiler/rustc_middle/src/mir/syntax.rs:1041`.
  `is_optional_overflow_check` (`mir/terminator.rs:164-170`) is exactly the
  `-C overflow-checks`-controlled subset: `OverflowNeg` and `Overflow(Add |
  Sub | Mul | Shl | Shr)`.

### 2.5 How asserts are optimized out (MIR passes, then LLVM)

- The optimized-MIR pipeline (`rustc_mir_transform/src/lib.rs:694-782`)
  runs after drop elaboration; relevant passes in order:
  `InstSimplify` (`:715, :737`), `SimplifyConstCondition`
  (`simplify_branches.rs:78-96`: an `Assert` whose condition const-folds to
  `expected` becomes a `goto`), `GVN` (`:751`),
  `SsaRangePropagation` (`ssa_range_prop.rs:158` consumes `Assert` edges as
  range facts), `DataflowConstProp` (`dataflow_const_prop.rs:215` visits
  `Assert` terminators), `JumpThreading` (`:760`), and more.
- Statically-false asserts become diagnostics instead of code:
  `known_panics_lint.rs:468-527` const-propagates the condition and reports
  `unconditional_panic` with the actual operands (bounds check included at
  `:515-519`; "remaining overflow errors are already covered by checks on the
  binary operators", `:520-521`).
- Bodies tainted by errors never reach the optimizer at all
  (`inner_optimized_mir`, `mir_transform/src/lib.rs:810-812`).
- Codegen sees the survivors as `cond_br` + call to a noreturn cold panic
  function; `codegen_assert_terminator` also suppresses optional overflow
  checks entirely when the crate being codegen'd has overflow checks off
  (`block.rs:769-775`). Everything provable that MIR left behind is left to
  LLVM's own pipelines.
- Constants: `rustc_const_eval/src/interpret/machine.rs:191-193`
  (`ignore_optional_overflow_checks`) documents that const-eval honors the
  same optional-overflow-check switch.

### 2.6 MIR borrowck architecture

- One query per body: `mir_borrowck` provider (`rustc_borrowck/src/lib.rs:108-130`)
  returns `Result<..., ErrorGuaranteed>`; `BorrowCheckRootCtxt`
  (`root_cx.rs:30-49`) is shared between the root body and nested items
  (closures), collects per-body region-constraint results
  (`:42-43`), and routes diagnostics through a taint-aware `DiagCtxt`
  (`:74-80`) so a body with errors emits nothing.
- Per body, three dataflow analyses are composed (`dataflow.rs:15-23`):
  `Borrows` (which loans are valid at each point), `MaybeUninitializedPlaces`
  (moves), `EverInitializedPlaces` -- run individually to fixpoint for speed
  (`:15-18`).
- Loans: `BorrowSet`/`BorrowData` (`borrow_set.rs`, imported `lib.rs:54`);
  conflicts: `places_conflict.rs:1-75` -- a lockstep walk over place
  projections treated as an `Overlap` monoid, with `PlaceConflictBias`
  (conservative for unknown indices) and access depths (Deep/Shallow)
  deciding prefix cases; `Deref` truncates the walk.
- Regions/liveness: `region_infer/` (constraint solving), `type_check/liveness/`
  (loan liveness), `constraints/`; the Polonius engine is an alternative
  consumer (`polonius/` directory).
- Diagnostics are separated (`diagnostics/`: borrow conflicts, move errors,
  mutability errors, ...), all downstream of the tainted-DiagCtxt gate.

## 3. How XIOM does it today

Verified against `crates/xiom-check`, `crates/xiom-codegen`, the driver, and
the safety docs in this worktree (main `ecc587ee`).

### 3.1 Checker (xiom-check)

- W004: `lint_w004_unreachable_arms` (`xiom-check/src/lib.rs:5913-5955`)
  scans arms in order; an UNGUARDED catch-all shadows later arms (`:5943-5945`,
  matching rustc's guard rule); duplicate exact keys are flagged; pattern keys
  come from `pattern_shadow_key` (`:5957-6001`) -- literals by value, variants
  by SPELLING (`leaf` name + payload arity, `:5974-5988`), not by resolved
  variant identity.
- Gating today: both call sites only test `!self.checking_catalog`
  (`:6676-6679` statement match, `:9505-9508` match expression). There is no
  error/taint gate: `match_variant_names` (`:9533-9564`) returns empty for
  non-`Named` types, but literal/variant keys still compare, so an unresolved
  scrutinee type can still trigger W004 (REL-20261010-1900).
- Exhaustiveness (S2): `check_match_exhaustiveness` (`:9566-9593`):
  `resolve_alias` -> only `CheckedType::Named`; variants from
  `match_variant_names` (Option/Result/Bool builtins + `enum_variants` map);
  missing variant -> warning, or hard error under `--strict-exhaustive`
  (`:9580-9590`).
- Error poison exists but is not consulted: `Expr::Error(_guarantee, _span)`
  short-circuits expression checking to `CheckedType::Error`
  (`:9484-9486`), but W004/S2 do not check for it.
- No bounds checking and no coherence/impl-overlap checking exist in the
  checker (grep: none; only a shift-amount range lint at `:7672`).
  COMPILER_READINESS_PLAN Stage 3 lists "coherence-lite impl-overlap checks"
  and Stage 7 requires "Checker enforces bounds/coherence" -- both still open.
- Borrow checker: lexical, not dataflow. `BorrowChecker`
  (`xiom-check/src/lib.rs:9886-9895`) = scope stack of `OwnershipInfo` +
  `ScopeBorrow` + 5c-R `active_loans: LoanSet`. `pop_scope` releases every
  borrow and clears all loans (`:9908-9923`). `borrow/place.rs` walks places
  with `Overlap/Disjoint/Ambiguous` (Deref -> conservative Overlap);
  `borrow/loans.rs` `LoanSet::grant` (`:31-58`) checks read/write conflicts;
  `mark`/`release_since` release statement temporaries (`:71-82`).
- Driver integration: `crates/xiom/src/lib.rs:1136-1154` runs the checker
  after typecheck ("Stage 4"); errors are E001 warnings unless `--strict`
  (`strict_mode` makes them hard errors). M21 status per docs/ROADMAP.md:
  "already active, 170+ tests" (`:876`, `:1204`), still partial (`:790-800`).

### 3.2 Codegen checks (LLVM IR emission)

- Overflow: only `i64` Add/Sub/Mul and only when `config.overflow_checks`:
  `expr.rs:284-307` emits `@llvm.sadd/ssub/smul.with.overflow.i64`,
  `extractvalue`, branch, `call void @llvm.trap()`, `unreachable`. The same
  guard is duplicated in a second binop path (`expr.rs:2080-2099`). Other
  widths (i8/i16/i32/i128, UInt widths) and other ops fall through to
  unchecked arithmetic. Division/remainder get an unconditional zero trap in
  the second path (`expr.rs:2061-2078`).
- Bounds: Vec read guard (`expr.rs:3252-3268`: `idx >= 0 && idx < len` else
  trap), Vec write guard (`stmt.rs:1196-1220`, m241), fixed-array read guard
  (`expr.rs:3419, 3488`) and write guard (`stmt.rs:1273-1277`, m243) through
  `emit_fixed_array_bounds_guard` (`emitter.rs:44-65`, compile-time length).
  Null-deref guard for raw pointers: `emitter.rs:28-42` (m244).
- All of these are gated on `config.overflow_checks` and are emitted directly
  as LLVM IR text branches with `llvm.trap` + `unreachable`; there is no
  XIOM middle IR, no check-once representation, and no const-folded
  diagnostic for statically-false checks.
- Traps are `llvm.trap` (ud2 class); contracts use a runtime trap path
  (`contracts.rs:52+` comment).
- Defaults/plumbing: driver `CompileConfig.overflow_checks` defaults false
  (`crates/xiom/src/lib.rs:100,169`); CLI flag `--overflow-checks`
  (`cli.rs:101`, read at `main.rs:1825`); the emitter's own default is true
  (`xiom-codegen/src/context.rs:187`) but the driver overrides it via
  `set_overflow_checks` (`crates/xiom/src/lib.rs:744, 1252`). Net effect:
  checks are opt-in at the CLI, independent of `--release`.
- Docs vs code: SAFETY_HARDENING.md S1 ("Debug overflow + bounds + null
  checks", `:109`) and the comparison matrix (`:187-195`) describe these as
  DONE; they are done as FLAG-GATED runtime traps, not as profile-driven
  semantics.

### 3.3 Profiles/flags today

- `--strict-exhaustive` promotes S2 warnings to errors; `--strict` escalates
  borrow E001 warnings to errors. No profile defaults toggle checks.
- COMPILER_READINESS_PLAN standing default ("TRAP in debug/profile builds,
  WRAP in release", `:216-218`) is documented intent, not implemented.
- Stage 6 PERF-1: unsafe-block confinement trampolines cost ~2.4 us/call
  (each potentially-faulting call inside an unsafe block) and are being
  addressed via `#[unsafe_direct]` plus batching follow-ups
  (`docs/STAGE6_PERF_PLAN.md:27-128`).

## 4. Gap table

| # | Area | rustc | XIOM today | Gap / impact | Stage |
|---|------|-------|------------|--------------|-------|
| 1 | W004/exhaustiveness gating | `Result<(), ErrorGuaranteed>` end-to-end; tainted bodies emit nothing (`check_match.rs:30-32, 265-285, 508-520`) | Only `!checking_catalog`; pattern keys are spelled, not typed (`lib.rs:6676-6679, 9505-9508, 5957-6001`) | 1900-class cascades remain; fix = error-state/Error scrutinee early return | S7 |
| 2 | Exhaustiveness engine | Usefulness matrix over deconstructed patterns; guards via `is_under_guard`; or-patterns native | Per-variant coverage scan; variant names from strings; guards only via W004 shadowing rule | False positives/negatives possible; no witness machinery | S7 |
| 3 | Bounds checks | Inserted once per index in MIR; backend lowers; fake borrows make hoisting legal | Per-site codegen branches under `--overflow-checks`; default OFF | Default-off safety; duplicated read/write emission; no const-known diagnostic | S7 |
| 4 | Check placement/cost | Assert in shared mid-level IR; provable checks deleted by MIR passes then LLVM | LLVM-IR only; no middle layer; confinement trampoline is the real hotspot (PERF-1) | Checks vs Stage 6 budgets; see proposals 5.7/6.7 | S6 |
| 5 | Overflow semantics | `-C overflow-checks` default = debug_assertions = (opt == 0); consts always checked; div/rem always checked | One CLI flag, default OFF; not coupled to `--release`; i64 add/sub/mul only | -O0/-O2/-O3 expectations undefined; narrow-int/const carve-outs missing | S7 |
| 6 | Borrow check | Per-body dataflow + regions; loans/places_conflict; conservative Deref; taint gate | Lexical scope stack + place loans; no paths/liveness; no move dataflow; warning-only unless `--strict` | Branch-sensitive unsoundness; no two-phase borrows | S7 |
| 7 | Match lowering | Decision tree with typed `MatchPair` places; false edges for borrowck; guards per sub-branch | Direct per-arm lowering in codegen from name/string-typed positions | m250/m254/m256 regression family; borrow analysis cannot consume match CFG | S7/S8-9 |
| 8 | Error-gating precedent | `ErrorGuaranteed` mint + `tainted_by_errors` suppress all dependent diagnostics | `Expr::Error` poison exists but lints do not consult it | Cheap, high-value 1900 fix; sets the house style for W002-W004 | S7 |

## 5. Optimization & performance opportunities

- One insertion point, not N call sites. XIOM emits the Vec bounds guard
  textually in two places (read `expr.rs:3252-3268`, write `stmt.rs:1196-1220`)
  and the fixed-array guard from two more (`expr.rs:3419, 3488`,
  `stmt.rs:1273-1277`). The rustc pattern (`bounds_check` calls `assert`
  once, `as_place.rs:727-756`) suggests a single "guarded index" helper so
  future policies (strength reduction, hoisting, diagnostics) change in one
  place. This also kills the read/write drift that m241/m243 each had to
  patch separately.
- Const-fold the check first: rustc diagnoses statically-false asserts
  (`known_panics_lint.rs:468-527`) and eliminates statically-true ones; the
  XIOM equivalent is a checker/codegen fold for constant index vs constant
  length -- no IR needed, and it removes the m243-family check entirely for
  unrolled loops with literal indices.
- Loop cost: a guard per element access is the naive cost model; rustc's
  fake borrows (`as_place.rs:617-620, 758-815`) exist precisely so the check
  can be hoisted out of loops/derefs legally. XIOM's Stage 6 target is within
  10% of Rust (`STAGE6_PERF_PLAN.md:13-17`); a sane rule is: in checked
  profiles, allowed to hoist only when the container cannot be mutated in the
  region (no aliasing writes visible in the statement), else keep per-access.
- Interaction with confinement (PERF-1): the 2.4 us/call trampoline shows
  what happens when a safety mechanism is placed around CALLS instead of
  around the OPERATION. Bounds/overflow checks are dataflow-visible facts in
  rustc (SSA range propagation consumes assert edges, `ssa_range_prop.rs:158`);
  XIOM should not add per-check runtime calls ever -- keep every check inline
  until a middle IR exists.
- IR size: every guard today creates two fresh blocks (`*_ok`, `*_trap`,
  each with trap call + unreachable). A per-function shared cold trap block
  (all guards branch to it) would shrink IR for check-heavy code and matches
  the "cold panic block" shape rustc hands to LLVM (`block.rs:782-808`).
  Watch the IR-bytes/fn Stage 6 budget.
- Profile split as the primary lever: the standing default (trap in
  debug/profile, wrap in release) is exactly rustc's semantics
  (`session.rs:1080-1081` + `config.rs:2961`); implementing it gives Stage 6
  a clean cost story: no check cost in the release-wrap build, checks in the
  checked build.

## 6. Adoption proposals

For strict profiles (Stage 7 gate):

1. Add an error-state gate before W004 and S2: return early when the match
   scrutinee's `CheckedType` is `Error`/non-resolved, or when the checker has
   recorded errors for the enclosing item (an XIOM analogue of
   `ErrorGuaranteed`). Primary fix for REL-20261010-1900; mirrors
   `check_match.rs:30-32, 265-285, 520`.
2. Make "strict" imply the full checked profile: borrow E001 = hard error
   (already), overflow checks ON (explicit), bounds checks ON,
   `--strict-exhaustive` ON. Document it as THE profile Stage 7's "borrow
   check on by default in strict profiles" refers to.
3. Strengthen W004 keys toward typed identity: for `Pattern::Ident`/`Variant`
   resolve against the scrutinee's variant set (already partially done via
   `match_variant_names`); when the scrutinee type is unresolved, do not
   compare at all (proposal 1). This is the cheap half of the rustc
   "type from layout/variant index, not names" rule.
4. Longer term (S8-9): a small typed CFG between checker and codegen (even a
   statement-level, not full MIR) would give one assert insertion point, one
   const-prop/fold pass, and a place model for match bindings that m256-class
   bugs cannot drift away from.

For debug/release semantics:

5. Debug/profile builds: `overflow-checks` default ON; release: default OFF
   (wrap), overridable with `--overflow-checks` (mirrors `-C overflow-checks`
   defaulting to `debug_assertions` = `opt-level == 0`).
6. Bounds and null checks stay ON in BOTH profiles (as rustc does for safe
   indexing); div/rem always checked; constants/CTFE always overflow-checked
   regardless of profile (`builder/mod.rs:788-792` is the precedent).
7. Differential harness (-O0/-O2/-O3, Stage 7): encode expected differences
   (overflow: trap at -O0, wrap at -O2/-O3) vs invariants (bounds, null,
   div-by-zero behave identically at all levels). A differential failure log
   that does not know this will misclassify by-design overflow differences.
8. Emit shared per-function cold trap blocks and single-source the guard
   emission (section 5, first two bullets) before Stage 6 re-baselines.

## 7. Open-item relevance

- REL-20261010-1900 (W004 gating / REL-... orphan): PRIMARY. rustc's
  mechanism is section 2.2 (structural `ErrorGuaranteed` gating of
  `check_match`); XIOM's current state is section 3.1 (only
  `!checking_catalog`; spelling-based keys; `Expr::Error` poison unconsulted).
  The adoption is Proposal 6.1/6.3: early-return when the scrutinee type is
  Error/unresolved, and stop comparing variant keys before resolution. This
  is independently small and testable with the documented phantom-import
  probes.
- REL-20261010-1854 (pulse site-mode startup crash): NOT a match-lowering
  item. Per COMPILER_BUGS.md:14979-15015 and RUSTC-REFERENCE S6/S7, the
  crash is mixed TLS-ness (`movq 0xf8` offset-as-address) plus an
  unused-fn emission/bloat cascade -- a monomorphization-reachability +
  TLS-model fix, not lowering. The genuine match-lowering member of that
  bug batch is m256 (match-arm Vec binding loses its container type in a
  tuple return, COMPILER_BUGS.md:14901, FIXED): its fix follows the
  "typed place over name table" rule of section 2.1, which is the direction
  Proposal 6.4 keeps.
- REL-20261010-1904 (float cast fn-pointer returns): out of scope, as
  expected. It is an ABI/`FnAbi` issue (RUSTC-REFERENCE S4) with no match
  lowering, exhaustiveness, borrowck, or runtime-check content; skipped
  per instructions.

## 8. Sources

rustc (`E:\repos\rust` @ `7211ea594cbf137ef6118b1752b959dea7cb9088`):

- `compiler/rustc_mir_build/src/builder/matches/mod.rs` (335, 376, 396, 499,
  1547, 1560-1620, 1721-1803, 1805, 2145, 2388-2472)
- `compiler/rustc_mir_build/src/builder/matches/match_pair.rs` (16-68, 70-90)
- `compiler/rustc_mir_build/src/builder/matches/buckets.rs` (58-93)
- `compiler/rustc_mir_build/src/builder/matches/test.rs` (33-62, 73+)
- `compiler/rustc_mir_build/src/builder/expr/as_place.rs` (613-659, 711-719,
  727-756, 758-815)
- `compiler/rustc_mir_build/src/builder/expr/as_rvalue.rs` (102-123, 445-480,
  481-529, 530+)
- `compiler/rustc_mir_build/src/builder/mod.rs` (770-792)
- `compiler/rustc_mir_build/src/builder/scope.rs` (1738-1763, 1770-1783)
- `compiler/rustc_mir_build/src/thir/pattern/check_match.rs` (30-64, 97,
  265-285, 396-421, 495-609, 963, 1190-1213)
- `compiler/rustc_pattern_analysis/src/lib.rs` (131-137);
  `.../src/usefulness.rs` (1105-1106, 1136-1147, 1714-1726)
- `compiler/rustc_middle/src/mir/syntax.rs` (1041);
  `.../mir/terminator.rs` (164-170)
- `compiler/rustc_session/src/options.rs` (2314-2315);
  `.../src/session.rs` (1080-1081); `.../src/config.rs` (2961)
- `compiler/rustc_mir_transform/src/lib.rs` (694-782, 810-812);
  `.../simplify_branches.rs` (78-96); `.../known_panics_lint.rs` (468-527,
  754-759); `.../ssa_range_prop.rs` (158); `.../dataflow_const_prop.rs`
  (215); `.../pass_manager.rs` (280+)
- `compiler/rustc_codegen_ssa/src/mir/block.rs` (753-834, 1719-1720)
- `compiler/rustc_borrowck/src/lib.rs` (1, 108-130); `.../root_cx.rs`
  (30-49, 74-80); `.../dataflow.rs` (15-23); `.../places_conflict.rs`
  (1-75); `.../borrow_set.rs`; `.../polonius/`; `.../region_infer/`;
  `.../type_check/liveness/`; `.../diagnostics/`
- `compiler/rustc_const_eval/src/interpret/machine.rs` (191-193)

XIOM (this worktree @ main `ecc587ee`):

- `crates/xiom-check/src/lib.rs` (5913-6010, 6673-6697, 9502-9593,
  9880-9923, 7672)
- `crates/xiom-check/src/borrow/mod.rs`, `.../borrow/place.rs`,
  `.../borrow/loans.rs`
- `crates/xiom-codegen/src/expr.rs` (284-307, 2061-2099, 3252-3268, 3419,
  3488); `.../stmt.rs` (1196-1220, 1273-1277); `.../emitter.rs` (28-65);
  `.../context.rs` (97-98, 187); `.../lib.rs` (181-182)
- `crates/xiom/src/lib.rs` (100, 169, 744, 1136-1154, 1252);
  `crates/xiom/src/main.rs` (1825); `crates/xiom/src/cli.rs` (101)
- `docs/RUSTC-REFERENCE.md`; `docs/SAFETY_HARDENING.md` (109-113, 187-195);
  `docs/UNSAFE_CONFINEMENT_PLAN.md` (351-353, 424-439, 493-496);
  `docs/COMPILER_READINESS_PLAN.md` (71-79, 193-210, 216-219);
  `docs/STAGE6_PERF_PLAN.md` (13-17, 27-128); `docs/ROADMAP.md` (790-800,
  876, 1204); `docs/COMPILER_BUGS.md` (14864-14897, 14901, 14979-15015)
