# 03 -- Optimization pipeline and IR quality: rustc vs XIOM

Scope: compiler/rustc_codegen_ssa/src/back/write.rs (module config, opt
levels, LTO, verify-llvm-ir/lint-llvm-ir, parallel CGU execution), the
LLVM-wrapper pass pipeline (PassWrapper.cpp), attribute emission
(rustc_codegen_llvm/src/attributes.rs, abi.rs; rustc_ty_utils/src/abi.rs;
rustc_target/src/callconv/mod.rs), intrinsic-based checks
(builder.rs checked_binop; intrinsic.rs assume/expect), and how rustc keeps
IR lean (monomorphization reachability, linkage, opt-gated attributes).
XIOM side verified against this worktree of E:\xiom-lang\xiom (HEAD
495d6672), crates/xiom-codegen and crates/xiom.

Method note: rustc paths are relative to the shared clone at E:\repos\rust
(commit 7211ea594cbf137ef6118b1752b959dea7cb9088, read-only). IR volume
measurements used the v0.64.3 release xiom.exe built from the main worktree
(the worktree has no stdlib/ checkout, which is gitignored and fetched per
pin; XIOM_STDLIB was pointed at the main checkout). Attribute-relevant
emitter code is identical between the binary's tree and this worktree; the
only drift is the m256 tuple-namer family. Probe sources and emitted .ll
files are under tmp/rustc-study/ (gitignored, not committed).

--------------------------------------------------------------------------------
## 1. Planning input

- [S6] rustc drives LLVM with a named *pipeline builder*, not a bare -O
  flag: per-module, O0, thin-LTO, and fat-LTO pipelines are selected per
  codegen unit, with loop-unroll / loop-vectorize / SLP / merge-functions
  as independent PipelineTuningOptions (PassWrapper.cpp:699-704, 906-973).
  XIOM passes one -O level to one clang invocation (lib.rs:1388-1398).
  Parity at -O2 is plausible; the levers XIOM cannot touch today are Os/Oz,
  merge-functions, per-function optnone, and arbitrary pass pipelines.
- [S6] rustc makes IR verification a hard, first-class step: verify-llvm-ir
  installs a VerifierPass at pipeline start *and* re-verifies every file
  write, while lint-llvm-ir runs LLVM's lint pass with AbortOnError
  (write.rs:102-103,225-226; PassWrapper.cpp:798-811,523-527;
  rustc_session/src/session.rs:974-976; options.rs:2636-2637). XIOM's
  `opt -passes=verify` step exists but is warning-only and only on the AOT
  path (lib.rs:1339-1378); `--emit-ir` returns before it (lib.rs:1284-1295).
- [S6] rustc derives optimization attributes from the type/layout system
  and emits them only when optimizing: noalias, readonly, writable,
  noundef, nonnull, dereferenceable, align, nofree, captures
  (rustc_ty_utils/src/abi.rs:324-422; rustc_codegen_llvm/src/abi.rs:39-116).
  XIOM emits none of these (measured: 0 matches in two emitted modules);
  its only function attributes are a heuristic alwaysinline/inlinehint
  (decl.rs:1517-1532).
- [S6] rustc emits only reachable, instantiated items and gives
  non-exported symbols limited linkage (rustc_monomorphize/src/collector.rs:382;
  back/symbol_export.rs:55-72). XIOM emits every registered stdlib function
  as an externally visible define (measured: 0 internal/private among
  147 and 287 defines), so LLVM's GlobalDCE cannot drop anything. This is
  the same bloat class as REL-20261010-1854's unused-function cascade.
- [S6] IR-volume proxies measured on this box: a 16-line program (core+io)
  emits 375,582 bytes / 5,313 lines / 147 defines; tests/ecosystem/t2-queue.xi
  emits 591,812 bytes / 8,380 lines / 287 defines. 87% and 90% of those
  defines are forced `alwaysinline`. Every function body carries a TLS
  recursion-counter load/store pair (667 and 1,175 IR line mentions).
- [S6] rustc pipelines codegen and LLVM across CGUs: one coordinator
  thread, main-thread CGU codegen, LLVM worker threads, jobserver tokens
  (write.rs:1203-1342). XIOM has rayon parallel codegen (context.rs:113,
  e2e_i2 lock) but the driver is not wired to it for stdlib-graph builds
  (STAGE6_PERF_PLAN item 1 priority 2).
- [S7] rustc runs MIR inlining and MIR optimization even at -Copt-level=0
  (config.rs:121-126: mir_opt_level() == 1 at OptLevel::No; inliner
  thresholds 30/50/100 at inline.rs:369-373). The selfhost differential
  gate (-O0/-O2/-O3 equivalence) needs XIOM's IR to be verifier-clean and
  semantically stable at every level; opt-gating of attributes is how
  rustc avoids level-dependent IR surprises (abi.rs:80-105).
- [S7] rustc's panic/trap paths are cheap by construction: assertions
  branch to a call of a #[cold] lang-item panic fn (block.rs:753-858, esp.
  the comment at 782-784), and diverging compiler_builtins calls become
  `abort` + `unreachable` (block.rs:199-203). XIOM's `@xiom_panic` is
  declared with no attributes (emitter.rs:1225) even though the runtime
  exits (stdlib runtime xiom_runtime.c:990-994).
- [S8-9] Later-stage levers rustc keeps open (not needed for the current
  budgets, but design should not block them): extra pass pipelines
  (-Cpasses / -Zllvm-plugins), PGO/coverage wiring, per-function
  target-features, Os/Oz (attributes.rs:465-481; PassWrapper.cpp:983-990).

--------------------------------------------------------------------------------
## 2. How rustc does it

### 2.1 Pipeline wiring (back/write.rs and the LLVM wrapper)

- ModuleConfig is constructed per module kind; regular modules take the
  session opt level, the allocator module runs with no optimizations
  (write.rs:70-127). opt_level is an Option<config::OptLevel>.
- OptLevel is a six-valued enum: No/Less/More/Aggressive/Size/SizeMin
  (rustc_session/src/config.rs:105-118). It maps to LLVM's
  CodeGenOptLevel for the TargetMachine and to a PassBuilder level
  (rustc_codegen_llvm/src/back/write.rs:123-145).
- Two independent vectorization knobs are wired from the opt level:
  loop vectorization at O2+ and SLP at O3, copied from clang's behavior
  (write.rs:229-236); MergeFunctions is enabled at O2+ unless the target
  opts out (write.rs:238-256). These land in PipelineTuningOptions
  (PassWrapper.cpp:699-704).
- The actual pipeline is built by LLVMRustOptimize: a PassBuilder with
  PTO/PGO/instrumentation, then
  * buildO0DefaultPipeline for O0 non-LTO stages (PassWrapper.cpp:931-934),
  * buildPerModuleDefaultPipeline(OptLevel) for regular builds
    (PassWrapper.cpp:955),
  * buildThinLTOPreLinkDefaultPipeline / buildThinLTODefaultPipeline /
    buildLTODefaultPipeline for the LTO stages (PassWrapper.cpp:942,
    958-971).
  Verification and linting are registered as pipeline-start extension
  callbacks (PassWrapper.cpp:798-811), so they run first, before any pass.
- Extra passes: `-Cpasses` is joined and parsed into the module pipeline
  (write.rs:745; PassWrapper.cpp:983-990); LLVM plugins are loaded and
  register callbacks (PassWrapper.cpp:745-763). `-Zno-prepopulate-passes`
  skips the default pipeline but still runs verifier/instrumentation hooks
  (write.rs:227; PassWrapper.cpp:974-981).
- File emission re-verifies: every write_output_file passes verify_llvm_ir
  to LLVMRustWriteOutputFile, whose addPassesToEmitFile call takes
  `DisableVerify = !VerifyIR` (rustc_codegen_llvm/src/back/write.rs:74-82,
  1069-1077, 1102-1110; PassWrapper.cpp:468-533). Invalid IR is a hard
  error, not a warning. The switch is `-Zverify-llvm-ir` or a build-time
  RUSTC_VERIFY_LLVM_IR environment marker (session.rs:974-976).
- emit_lifetime_markers is opt-gated: lifetimes only when opt != No, plus
  sanitizer needs (session.rs:883-889). assume/expect are gated the same
  way at emission (intrinsic.rs:1030-1046), so O0 IR stays small.

### 2.2 Attributes are derived from types, not names

- Scalar and pointer argument attributes come from the layout system in
  rustc_ty_utils/src/abi.rs:324-422 (arg_attrs_for_rust_scalar):
  * NoUndef when the scalar has no uninitialized state (334-336);
  * NonNull when the validity range excludes zero (341-344);
  * pointee_align / pointee_size feed `align` / `dereferenceable`
    (348-354);
  * NoAlias for frozen shared refs, unpinned mutable refs, and Box under
    -Zbox-noalias (356-377; noalias is never set on returns);
  * ReadOnly + CapturesReadOnly for frozen shared refs, not on returns
    (414-417);
  * NoFree with per-pointer-kind reasoning (397-412), Writable for
    &mut where the callee may write (379-392).
- Indirect arguments/returns get NoAlias + CapturesAddress + NonNull +
  NoUndef + NoFree plus pointee size/align (rustc_target/src/callconv/mod.rs:426-446);
  the attribute bitset is callconv/mod.rs:136-147.
- The LLVM mapping (rustc_codegen_llvm/src/abi.rs:39-133) applies ABI
  attributes always, but all optimization attributes only when
  `sess.opts.optimize != OptLevel::No` (abi.rs:80-105); dereferenceable
  vs dereferenceable_or_null is chosen from NonNull (82-94). Attributes
  are applied both to definitions and to call sites
  (attributes.rs:29-39, abi.rs:118-133).
- Function-level attributes (rustc_codegen_llvm/src/attributes.rs:490-684):
  inline_attr from #[inline]/#[inline(always)]/#[inline(never)] plus
  requires_inline for cross-crate #[inline] (51-83); cold (543-545);
  FFI memory effects: rustc_ffi_pure -> readonly, rustc_ffi_const ->
  memory(none) (546-551); allocator family/align/allocsize/noalias return
  (592-649); alignment (650-652); the target-cpu attribute is ALWAYS
  emitted with the comment that without it ThinLTO will not inline Rust
  functions into Clang-generated functions (662-668); target-features and
  instruction sets (670-684). Os/Oz add optsize/minsize attrs (465-481).
- GEP flags: rustc has gep / inbounds_gep / inbounds_nuw_gep and picks the
  weakest valid form (builder.rs:959-1009); every alloca/load/store gets
  an explicit alignment (builder.rs:142-190, 668-676).

### 2.3 Intrinsic-based checks and canonical IR forms

- Checked arithmetic (rustc_codegen_llvm/src/builder.rs:609-650): signed
  add/sub/mul use llvm.s{add,sub,mul}.with.overflow; UNSIGNED add/sub are
  emitted as add/sub + icmp because LLVM considers that the canonical form
  (comment at 621-635). Rust's overflow failure is a panic call, not a
  trap; the panic block is cold by construction (block.rs:782-784).
- rustc-core intrinsics map to LLVM intrinsics in intrinsic.rs (sqrt,
  powi, fma, minnum/maxnum, fshl/fshr, saturating_*, prefetch,
  is_val_statically_known -> llvm.is.constant; black_box).
- llvm.expect and llvm.assume are emitted only when optimizing
  (intrinsic.rs:1030-1046); MIR opt and MIR inlining run before codegen
  (inline.rs:47-49,369-373).

### 2.4 Keeping IR lean

- Monomorphization collects reachable, instantiated items only; anything
  unreachable never reaches codegen (rustc_monomorphize/src/collector.rs:382;
  partitioning.rs:143). reachable_non_generics drives symbol export and
  linkage (back/symbol_export.rs:55-72).
- Attributes/expect/lifetime markers are suppressed at O0 (S2.1-S2.2), so
  debug IR is both smaller and faster for LLVM to process.
- Diverging runtime calls may be replaced by abort+unreachable for size
  (block.rs:199-203). Traps are terminal (`unreachable`), letting LLVM
  prune post-trap code.

### 2.5 Parallel codegen architecture

write.rs:1203-1342: the coordinator owns a work queue and spawns LLVM
worker threads as work items appear; the main thread codegens CGUs and
hands them over; a jobserver bounds total parallelism (1229-1240). This
overlaps codegen of CGU N+1 with LLVM optimization of CGU N -- the
compile-time half of the performance story, independent of IR quality.

--------------------------------------------------------------------------------
## 3. How XIOM does it today (verified)

### 3.1 Driver / opt levels (crates/xiom/src)

- Single-stage clang is the only optimizer by design: the old
  `opt -O<n>` + `clang -O<n>` double pipeline miscompiled on LLVM 18
  (comment lib.rs:1392-1398). Level selection: `--opt-level 0..=3`
  overrides; default -O2 debug, -O3 release (lib.rs:1388-1390);
  `xiom::jit::effective_opt_level` mirrors this for the script cache key
  (jit.rs:105-110); main.rs:1794-1801 decides release and contract
  stripping.
- clang flags today: -maes/-mavx/-mavx2 (unconditional x86_64 native),
  -mavx512* host-gated (lib.rs:1458-1485); -g (1491-1494); the opt level
  (1498-1499); -flto=thin + -fuse-ld=lld for --lto (1500-1505);
  sanitizer/stack-protector/-fPIC as configured (1520-1548).
- The IR is written to `<output>.ll` (lib.rs:1312-1323), copied to a temp
  dir as xiominput.ll and compiled together with the runtime C sources or
  their cached objects (`crates/xiom/src/rtcache.rs`; lib.rs:1605-1653).
- Verification today: best-effort `opt -passes=verify -disable-output`
  (legacy `-verify` fallback), warning-only, AOT path only
  (lib.rs:1339-1378). `--emit-ir` prints the module and returns before
  the verifier (lib.rs:1284-1295). clang's own parser verifies, which is
  why m256-class errors surfaced as clang type errors.

### 3.2 Emitter / IR quality (crates/xiom-codegen)

- Attributes: only inline hints. decl.rs:1517-1532 marks bodies with
  approx_block_cost <= 10 `alwaysinline`, <= 48 `inlinehint`, else
  nothing; the comment records a >300 s clang hang when everything was
  alwaysinline on a 735 KB module. No other function/parameter/return
  attributes exist anywhere in the emitter (grep: no noalias,
  dereferenceable, noundef, nonnull, readnone/readonly, nounwind,
  willreturn, cold, noreturn); declarations are bare, e.g.
  `declare void @xiom_panic(i8*)` (emitter.rs:1225-1251) and
  `declare i8* @malloc(i64)`. The runtime exits inside xiom_panic
  (stdlib xiom_runtime.c:990-994), so noreturn would be truthful.
- Linkage: function defines carry no linkage keywords. Globals are
  emitted `internal` for module-level `var`s (lib.rs:5106-5109), but
  measured emitted modules have 0 internal/private/hidden defines.
- Intrinsics: overflow checks use llvm.sadd/ssub/smul.with.overflow.i64 +
  `llvm.trap` + `unreachable` (expr.rs:283-307, 2028-2030), gated on
  --overflow-checks (context.rs:97-98). Bulk copies use
  llvm.memcpy/memmove. No llvm.expect, llvm.assume, or lifetime markers
  are emitted (measured 0).
- GEP flags: struct-field and linear GEPs are emitted without `inbounds`
  in the hot paths (measured 0 of 187 GEPs in the probe, 0 of 306 in
  t2-queue). `inbounds` appears only in a handful of compiler-generated
  tables (emitter.rs:1733,1848; lib.rs:320-369,6892-6902,8881).
- Alignment: only i128/fp128 allocas/loads/stores get `align 16`
  (lib.rs:778,812; expr.rs:72; 4 sites per module measured). Other
  load/store sites rely on LLVM's ABI-alignment defaults.
- Contracts: each boolean clause is compiled to `br ok/fail`; the fail
  block calls @puts then @xiom_panic then `unreachable`
  (contracts.rs:10-59). invariant_check functions are emitted per type
  that has invariants (contracts.rs:105-119, lib.rs:6197); in the probe
  10 were defined but only 2 call sites exist.
- Recursion guard: every function entry loads/adds/compares/stores
  @xiom_recursion_counter and every return path decrements it
  (decl.rs:1558-1571,1905-1927).

### 3.3 Measured module shape (v0.64.3 release driver, --emit-ir)

| metric                    | probe1.xi (16 loc) | t2-queue.xi |
|---------------------------|--------------------|-------------|
| bytes                     | 375,582            | 591,812     |
| lines                     | 5,313              | 8,380       |
| defines                   | 147                | 287         |
| declares                  | 179                | n/a         |
| alwaysinline              | 128                | 257         |
| inlinehint                | 3                  | n/a         |
| attribute groups (#N)     | 0                  | 0           |
| noalias/deref/noundef/... | 0                  | 0           |
| noreturn / cold           | 0                  | 0           |
| getelementptr / inbounds  | 187 / 0            | 306 / 0     |
| @llvm.trap sites          | 141                | 277         |
| @llvm.expect/assume/life  | 0                  | n/a         |
| recursion-counter mentions | 667                | 1,175       |
| user-defined functions    | 2                  | 4           |

Derived: 2.55 KB/define (probe), 2.06 KB/define (t2); 87-90% of defines
forced inline; ~12-14% of IR lines mention the recursion counter. The
recorded t6 template (STAGE6_PERF_PLAN.md:319-322) was 668 KB / 17k lines
/ 222 fns (3.0 KB/fn), so the shape is stable across programs.

--------------------------------------------------------------------------------
## 4. Gap table

| # | Dimension | rustc | XIOM today | Gap |
|---|-----------|-------|------------|-----|
| 1 | Opt pipeline | PassBuilder pipelines per stage + PTO knobs (PassWrapper.cpp:906-973) | single clang -O flag (lib.rs:1388-1398) | medium: works at O2, no Os/Oz/merge-functions/per-fn control |
| 2 | IR verification | hard-fail VerifierPass + pre-emit verify (write.rs:225; PassWrapper.cpp:798,523) | advisory opt verify, AOT only, `--emit-ir` unverified (lib.rs:1339-1378,1284) | high (m256-class) |
| 3 | IR lint | -Zlint-llvm-ir, AbortOnError (options.rs:2636; PassWrapper.cpp:805) | none | low |
| 4 | Param/ret attrs | type-derived noalias/readonly/noundef/nonnull/dereferenceable/align, opt-gated, def+callsite (abi.rs:39-116; ty_utils/abi.rs:324-422) | none (0 matches) | high |
| 5 | Fn attrs | target-cpu/features, cold, memory effects, alloc-family, alignment (attributes.rs:490-684) | alwaysinline/inlinehint only (decl.rs:1517) | high |
| 6 | Panic path | cold lang-item panic; abort+unreachable for divergent builtins (block.rs:199,753) | bare @xiom_panic declare, no cold/noreturn (emitter.rs:1225) | medium |
| 7 | Linkage/reachability | internal for non-exported; reachable-only mono (symbol_export.rs:55; collector.rs:382) | all defines externally visible, unused invariant_checks kept | high (binary size, REL-1854) |
| 8 | GEP flags | gep/inbounds/nuw chosen per site (builder.rs:959-1009) | 0 inbounds in probes | medium |
| 9 | align | explicit on every alloca/load/store (builder.rs:175-190) | only i128/fp128 align 16 | low/medium |
| 10 | Intrinsics | canonical checked ops; assume/expect/lifetimes opt-gated (builder.rs:609; intrinsic.rs:1030; session.rs:883) | with.overflow+trap; no expect/assume/lifetimes | low/medium |
| 11 | Parallel codegen | CGU partition + codegen/LLVM pipeline (write.rs:1203) | rayon path exists, driver not wired (context.rs:113) | medium (compile time) |
| 12 | Opt defaults | default O0; MIR opts run at O0 (config.rs:121); overflow checks in debug | -O2 debug / -O3 release; overflow only on flag (lib.rs:1388; context.rs:97) | policy: interacts with the safety-tier fork |

Not a gap: not having rustc's in-tree pipeline manager. Investing in
clang -O levels is a valid architecture; the transferable items are the
IR-level signals (attributes, linkage, verification, inline policy),
which clang/LLVM consumes identically for both compilers.

--------------------------------------------------------------------------------
## 5. Optimization & performance opportunities (quantified)

Tie-in to Stage 6 budgets: "release/O2 within 10% of Rust", "binary <= 2x
Rust stripped", "compile time <= 3x Rust debug". The recorded profile
says the compile budget is dominated by the checker/catalog/stdlib graph
(STAGE6_PERF_PLAN.md:210-336), so items 5.1/5.2 target RUNTIME and BINARY
SIZE first and clang-stage time second; they do not fix cold compile.

5.1 Inline policy (runtime + compile time + size). 87-90% of emitted
    functions are forced alwaysinline (128/147, 257/287). rustc reserves
    alwaysinline for #[inline(always)] and inlines small functions by
    cost/benefit instead (attributes.rs:69-82; MIR inliner thresholds
    30/50/100 at inline.rs:369-373). Consequence today: every call site
    of a small stdlib fn drags in its body plus its recursion-counter
    sequence; the recorded 300 s clang hang (decl.rs:1519-1524) is the
    tail risk of exactly this. Proposal: default small bodies to
    `inlinehint`, keep alwaysinline only for (a) explicit hot helpers
    proven by the benchmark lane, or (b) bodies <= 4 statements; validate
    against t2-queue 24 ms, t1/t5. Expected: lower IR bytes/fn (budget
    metric), smaller pre-opt module, faster clang -O2, no hot-path
    regression if the benchmark gate is used.

5.2 Linkage + reachability (binary size; REL-20261010-1854). All 147/287
    defines are externally visible, so LLVM cannot delete the ~8
    uncalled invariant_check bodies, contract helpers, spawn wrappers, or
    any unused stdlib fn. Marking non-`pub_functions` (config already
    carries that set, context.rs:99-100) `internal` lets GlobalDCE drop
    them at -O2; rustc does this via reachable_non_generics
    (symbol_export.rs:55-72). For 1854 (341 KB -> 1.9 MB from one unused
    helper) this is the structural fix direction: the extra emission
    would not exist. Expected: binary budget (<=2x) plus clang time
    proportional to kept IR.

5.3 Attributes on the pointer surface (runtime). XIOM methods already
    pass `&T`/`&mut T`/self as pointers (decl.rs:1449-1476), which is
    exactly where rustc attaches noalias/readonly/writable/nonnull/
    dereferenceable/align from layout (ty_utils/abi.rs:324-422). Start
    conservative: `noundef` on i64/i1/f64 immediates and the boxed
    handle types; `readonly` + `nonnull` + `dereferenceable(N)` + `align`
    on `&T` params of stdlib getters; never noalias on returns
    (ty_utils/abi.rs:373-375). Measurement path: emit attrs behind a
    flag, compare clang -O2 asm/vectorization on t1/t5 and the
    guard_cost2 probe; the attributes gated by `opt != 0` keep -O0
    output unchanged (abi.rs:80-105), which is also the selfhost
    differential-safe ordering.

5.4 Panic/trap path (runtime micro). `@xiom_panic` exits the process
    (xiom_runtime.c:990-994) but is declared bare; `noreturn` + `cold` on
    the declare is truthful and makes the contract-fail block terminal
    like rustc's panic blocks, which are cold by construction
    (block.rs:782-784). Do not add llvm.expect on contract checks; rustc
    deliberately does not (same comment).

5.5 inbounds GEPs (runtime). 0/187 and 0/306 GEPs carry inbounds. Field
    GEPs with constant indices on alloca/object pointers are always in
    bounds and can be `getelementptr inbounds` (rustc has a dedicated
    builder form, builder.rs:973-990). Linear index GEPs should stay
    without inbounds until a bounds proof exists (the safe-tier guards
    may prove them later).

5.6 Verification strength (correctness + compile time). The advisory,
    AOT-only verify should become: hard error when a supported verifier
    ran and failed (clang would fail anyway, with a worse message), and
    always verify `--emit-ir` output when `opt` is available (today the
    only artifact can be invalid IR with exit 0). Keep silent-skip when
    the host's opt lacks the pass syntax. rustc's model: verify at
    pipeline start and at every emission (PassWrapper.cpp:798,523).

5.7 IR-volume budgets (CI). Stage 6 explicitly wants "IR bytes/fn" as a
    budget. Current baselines from this study: 2.06-3.0 KB/define, and
    the share of defines with alwaysinline. Both are computable from
    `--emit-ir` alone; wire them into the xiom bench harness (main.rs
    bench path) with the same lock discipline as the IR diff gates.

5.8 Per-call recursion counter (runtime; needs safety-lane sign-off).
    ~12-14% of IR lines and a TLS load+store per call plus one per
    return path (decl.rs:1558-1571,1905-1927). rustc has no such per-call
    guard; stack overflow is caught by the guard page. Candidate: elide
    the counter update for leaf functions (cannot grow depth) -- note
    this shifts the trap threshold by one level for call chains whose
    last hop is a leaf; owner/safety decision, not a free change. Variant
    with the same caveat: keep the update only in functions that contain
    calls (every recursion cycle contains one), leaving leaf getters
    untouched.

5.9 ThinLTO cross-language inlining (runtime, verify first). rustc emits
    target-cpu on every function specifically so ThinLTO inlines Rust
    functions into clang-compiled ones (attributes.rs:662-665). XIOM
    compiles its module with `-maes -mavx -mavx2` (lib.rs:1458-1464)
    while its IR functions carry no target attributes; whether ThinLTO
    can inline XIOM fns into runtime-C functions today is unverified.
    Experiment: add `"target-cpu"` / `"target-features"` strings to
    emitted defines, derived from the runtime C compile (same clang
    flags), compare ThinLTO summaries / -Rpass remarks. Do not change
    the AVX-512 host gate (lib.rs:1465-1485) while experimenting.

--------------------------------------------------------------------------------
## 6. Adoption proposals (concrete, by crate/module)

P1 inline policy -- crates/xiom-codegen/src/decl.rs (approx_block_cost
   threshold site, 1517-1532). Swap the default for small bodies to
   inlinehint; keep an explicit allowlist constant for alwaysinline.
   Locks: extend the existing IR lock tests in crates/xiom-codegen/tests/
   with a no-unexpected-alwaysinline assertion; re-baseline expected IR
   only through the standard lock-refresh flow. Benchmark gate: t2-queue,
   t1, lz4 smoke.

P2 linkage/reachability -- crates/xiom-codegen/src/decl.rs (define line)
   plus lib.rs emit pass. Emit `internal` for every function not in
   config.pub_functions and not main/extern (context.rs:99-100 already
   carries the pub set); keep hot-reload builds fully exported
   (config.hot_reload). Optionally a post-pass that prunes defines with
   zero references (the m142 undefined-symbol scanner in emitter.rs
   already walks call sites for the inverse check).

P3 attribute emission -- new helper in crates/xiom-codegen/src/decl.rs
   (param/return rendering) used by crates/xiom-codegen/src/call.rs for
   call-site attrs. Phase 1: `noundef` on scalar params/returns, `cold
   noreturn` on the xiom_panic declare (emitter.rs:1225). Phase 2:
   readonly/nonnull/dereferenceable/align for reference params, derived
   from the checker's type surface (xiom-check resolved types are already
   handed to codegen). Gate all of it on an opt-attribute flag so IR
   baselines can be renewed incrementally.

P4 verification hardening -- crates/xiom/src/lib.rs (verifier block
   1339-1378) and the emit-ir early return (1284-1295). Run the verify
   step before the emit-ir return; promote verifier failure to an error
   when a supported spelling ran, with the IR path in the message; keep
   the warning path only for unsupported/spawn-failed opt. Add a
   XIOM_TIMINGS sub-phase mark (driver already has the timing marks).

P5 IR-volume metric -- crates/xiom/src/main.rs bench path + tools. Count
   bytes/defines/alwaysinline/internal-defs from the emitted module and
   enforce Stage 6 budgets in CI (guard: single source of truth for the
   numbers so docs do not drift).

P6 parallel stdlib codegen -- crates/xiom/src/lib.rs graph build path:
   set config.parallel_codegen (context.rs:113) for stdlib-graph builds;
   the emitter path is already covered by e2e_i2_parallel_codegen
   (STAGE6_PERF_PLAN item 1 priority 2).

P7 GEP inbounds -- crates/xiom-codegen/src/decl.rs field-access GEPs
   (e.g. 1612) and expr.rs struct-element GEPs: add `inbounds` for
   constant-index struct field GEPs only.

Ordering: P4 first (cheap, unblocks clean failure semantics), then P2
(binary-size and 1854-directional), then P1+P5 together (they need
re-baselined metrics), then P3, then P6/P7.

--------------------------------------------------------------------------------
## 7. Open-item relevance

m256-class invalid IR (fixed as m256 in docs/COMPILER_BUGS.md 2026-10-10,
"match-arm Vec binding loses its container type in a tuple return"):
the emitter produced `%tmp36` typed `%struct.Tuple__Int__Int` against a
declared `%struct.Tuple__Int__Vec` return; clang rejected it at parse.
rustc's two answers are both in this study: (a) verify before the
external tool (S8 in docs/RUSTC-REFERENCE.md; PassWrapper.cpp:798,523)
and (b) type from layout/variant identity, not name strings (S3/S5).
XIOM has (a) only as an advisory step that `--emit-ir` skips, so a
regression in the namer family can again ship unverified module text
(exit 0) on the emit-ir path, and a warning-only opt failure on the AOT
path. Proposal P4 closes the verification half; the naming half remains
tracked in the m250/m254/m256 family.

REL-20261010-1854 (unused helper => 341 KB -> 1.9 MB exe; 13,134 vs 307
%gs TLS accesses; two `movq 0xf8` TLS-offset-as-address loads):
optimization-pipeline relevance is threefold.
- Emission cascade: the module should never carry an unreferenced
  function. rustc's reachability discipline (collector.rs:382) plus
  internal linkage is the structural fix direction; proposal P2 makes
  LLVM able to drop such functions and shrinks the surface where the
  malformed accesses appear.
- Per-call TLS traffic: XIOM's recursion counter is a TLS load/store in
  every function (decl.rs:1558-1571); inlining then replicates it
  (667-1,175 line mentions here; 13,134 TLS accesses in the crashed
  exe). Rust has no per-call TLS counter. Proposal 5.8 is the candidate
  reduction; the crash itself (TLS offset used as an address) remains a
  codegen correctness bug whose declaration-consistency fix direction is
  RUSTC-REFERENCE S6 (set_thread_local_mode once per declaration;
  rustc_codegen_llvm/src/consts.rs:413,542) -- flagged here, not
  re-diagnosed.
- Verification: with a hard verifier, a module carrying a mis-typed or
  ill-formed access pattern from an unused fn fails at the earliest
  point instead of shipping to clang/link.

--------------------------------------------------------------------------------
## 8. Sources

rustc (E:\repos\rust @ 7211ea594cbf137ef6118b1752b959dea7cb9088):
- compiler/rustc_codegen_ssa/src/back/write.rs:70-127 (ModuleConfig),
  225-267 (flag wiring, vectorize/merge rules), 363-460, 1203-1342
  (CGU/worker pipeline), 745 (extra passes).
- compiler/rustc_codegen_llvm/src/back/write.rs:53-82 (write_output_file
  + verify), 123-145 (opt level mapping), 179-219 (target machine),
  745-795 (LLVMRustOptimize call), 1010-1111 (emit paths).
- compiler/rustc_llvm/llvm-wrapper/PassWrapper.cpp:468-533
  (write + DisableVerify), 676-1065 (LLVMRustOptimize: PTO 699-704,
  verifier/lint callbacks 798-811, O0 931-934, per-module 955, LTO
  942,958-971, extra passes 983-990, run 1063).
- compiler/rustc_session/src/config.rs:105-126 (OptLevel, mir_opt_level);
  session.rs:883-889 (lifetime markers), 974-976 (verify_llvm_ir);
  options.rs:2636-2637 (lint_llvm_ir).
- compiler/rustc_codegen_llvm/src/attributes.rs:29-83 (inline attr),
  465-481 (optsize/minsize), 490-684 (fn attrs incl. target-cpu comment
  at 662-668).
- compiler/rustc_codegen_llvm/src/abi.rs:39-133 (arg attrs -> LLVM,
  opt-gating at 80-105).
- compiler/rustc_ty_utils/src/abi.rs:324-422 (arg_attrs_for_rust_scalar),
  543-623 (fn_abi_new_uncached).
- compiler/rustc_target/src/callconv/mod.rs:136-147 (attribute bits),
  407-446 (ArgAbi::new, indirect_pass_mode).
- compiler/rustc_codegen_llvm/src/builder.rs:142-190 (alloca/align,
  load/store align), 609-650 (checked_binop), 959-1009 (gep forms).
- compiler/rustc_codegen_llvm/src/intrinsic.rs:1030-1046
  (assume/expect gating), 250-256, 404-424 (intrinsic mappings).
- compiler/rustc_codegen_ssa/src/mir/block.rs:199-203, 753-858 (assert
  lowering; cold panic block).
- compiler/rustc_codegen_ssa/src/mir/rvalue.rs:1056-1073
  (codegen_scalar_checked_binop).
- compiler/rustc_mir_transform/src/inline.rs:47-49, 369-373, 437-440
  (MIR inliner policy/thresholds).
- compiler/rustc_monomorphize/src/collector.rs:382;
  partitioning.rs:143 (reachability).
- compiler/rustc_codegen_ssa/src/back/symbol_export.rs:55-72
  (reachable non-generics).

XIOM (this worktree, HEAD 495d6672; docs cited from this repo):
- crates/xiom/src/lib.rs:1284-1295 (emit-ir early return), 1312-1323
  (ir file), 1339-1378 (advisory verify), 1380-1398 (opt levels,
  single-stage rationale), 1458-1485 (ISA flags), 1491-1550 (clang flags),
  1605-1653 (rtcache + staging).
- crates/xiom/src/jit.rs:105-110; main.rs:1794-1801.
- crates/xiom-codegen/src/decl.rs:1440-1476 (self/ref ABI), 1517-1532
  (inline policy), 1551-1571, 1900-1930 (recursion counter), 1612
  (field GEP sample).
- crates/xiom-codegen/src/emitter.rs:1225-1251 (declares), 1356-1391
  (debug module flags), 1406+ (undefined-symbol pass), 1733,1848.
- crates/xiom-codegen/src/expr.rs:283-307, 2028-2030 (overflow
  intrinsics); contracts.rs:10-59 (contract lowering), 105-119, 302
  (invariant_check).
- crates/xiom-codegen/src/context.rs:81-159 (CodegenConfig flags:
  check_contracts 86, overflow_checks 97-98, parallel_codegen 113,
  debug_symbols 115).
- crates/xiom-codegen/src/lib.rs:5042-5049 (module header), 5106-5109
  (internal globals), 6197 (invariant_check emission).
- crates/xiom-graph/src/paths.rs:28-134 (stdlib resolution used for the
  measurements).
- docs/STAGE6_PERF_PLAN.md (compile/runtime profile, t6 668 KB/17k/222,
  item-1 backlog); docs/COMPILER_READINESS_PLAN.md:193-210 (Stage 6);
  docs/RUSTC-REFERENCE.md (S6 TLS, S7 reachability, S8 verify);
  docs/COMPILER_BUGS.md (m256 fixed 2026-10-10; REL-20261010-1854-pulse
  open 2026-10-10, read from the main worktree).
- Pinned stdlib checkout (main worktree): stdlib/runtime/xiom_runtime.c:990-994
  (xiom_panic exits).
- Measured IR: tmp/rustc-study/probe1.ll, tmp/rustc-study/t2queue.ll
  (gitignored, produced by v0.64.3 release xiom.exe --emit-ir).
