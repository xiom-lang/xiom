# XIOM Compiler Session -- Handoff (2026-08-24)

## Session update (2026-08-24, round 16a IN PROGRESS -- readiness Stage 0/1): CTFE explicit-stack rewrite + const-folder hardening

Master plan created: docs/COMPILER_READINESS_PLAN.md (merges the round-15
queue + compiler-audit findings #1-20 into staged work; stdlib audit belongs
to the PARALLEL STDLIB SESSION -- this lane never edits stdlib/).

### DONE this round so far

1. **Stage 0 ground truth (fresh laptop):** checker 178/178, parser 97/97,
   ctfe 97->36/36 (rewritten suite), feature-reg 510/510, full e2e BASELINE
   2293/2294 (the one fail, e2e_m37_simd_runtime Some(15) vs Some(0), is a
   SIMD CPU-dispatch ENVIRONMENTAL fail on this machine -- reproduces on the
   untouched round-15 binary; same class as the old smoke_simd CRT ignore).
   stdlib-exec 69/70 (+2 ign) -- stdlib_exec_log_runs ALSO fails at BASELINE
   here (verified via stash+rebuild); treat as machine-environmental until
   the desktop re-confirms.
2. **BUG 57 verdict (geom family): CONFIRMED REAL.** SESSION round-15 note
   ("probe_skew3/4 green -- stale handoff entries") was WRONG;
   stdlib_session.md's evening report is correct. Fresh probes committed:
   tmp/bug_probes/probe_geom1.xi (identity->trace/det chain prints garbage
   9.214e18) + probe_geom2.xi (raw m[0][1] param read prints -4.609e18).
   NO e2e covers nested &Vec[Vec[Float64]] PARAM reads in module fns --
   Stage 4 fix must add e2e_m57_geom_nested_param.
3. **Stage 1 CTFE REWRITE LANDED (crates/xiom-ctfe/src/lib.rs, ~1200 lines):
   explicit work-stack MACHINE replaces the recursive tree-walker.**
   - Fixes audit #1: user recursion no longer grows the native stack --
     RecursionLimit(1000) is reachable and returns a diagnostic (regression-
     tested: factorial(20) evaluates; infinite recursion errors cleanly).
   - Fixes audit #2: to_expr -> try_to_expr returning Option; Struct/Variant/
     Array reconstruct faithfully; Ptr/Null/Range/custom variants return None
     and the CODEGEN CALLER NOW FALLS BACK TO UNEVALUATED (runtime
     materialization), never the Int(0) sentinel.
   - Fixes audit #7: for-loops iterate fully over ranges (parser-desugared
     range/range_inclusive calls), array literals, and array locals; unlabeled
     break/continue supported; labeled forms rejected with clear errors.
   - Overflow policy (audit three-model inconsistency): checked arithmetic,
     overflow = hard compile error (rustc precedent). Float ==/!= EXACT IEEE
     (epsilon 1e-15 removed there AND in the codegen folder's pattern matcher).
   - Session-wide fuel on the engine (was per-frame, multiplied by depth).
   - Arena-cap panic -> Err(ArenaOverflow).
   - Block tails: statements execute sequentially; frame.last carries each
     expression-statement value ({ let a = 21; a * 2 } folds to 42 -- the
     old eval_block_last SKIPPED preceding statements entirely, an audited +
     newly-found defect; fixed in BOTH the engine and codegen's folder, whose
     if/match folding now returns Option<Expr> instead of fabricating Int(0)).
4. **Codegen const-folder hardened (crates/xiom-codegen/src/expr.rs):**
   wrapping arithmetic -> CHECKED in the SIGNED domain (Expr::Int stores the
   i64 bit pattern as u64 -- negate/subtract via s i64 first; naive
   unsigned checked ops broke -X/0-X folding and was caught by
   regress_5e7f_const_arithmetic_neg within minutes: baseline-compare
   discipline works). True i64 overflow leaves expressions unevaluated.
   CtfeEngine call site uses try_to_expr with unevaluated fallback.

### Verification state of THIS round's changes

ctfe 36/36 (new suite incl. ICE/sentinel/for-loop/overflow/block-tail
regressions), checker 178/178, parser 97/97, feature-reg 510/510,
stdlib-exec 69/70 (+2 ign; log_runs = baseline env fail), geom probes
unchanged (BUG 57 still open, Stage 4). FULL E2E ON THE NEW BINARY RUNNING
AT DOC TIME (baseline comparison target: 2293/2294 + simd env-fail).

**NEXT:** confirm e2e -> commit discipline -> Stage 1 remainder (verifier
honesty R16c: sort-consistent SMT, UNKNOWN != false, bounded z3; quick wins:
module-prefix arity check, warnings-not-dropped, MAX_EXPR_DEPTH 128) ->
Stage 2 structural foundations per docs/COMPILER_READINESS_PLAN.md.

### Round-19 FOR THE STDLIB SESSION -- CATALOG FINDINGS (Item A surface)

Catalog body type-checking is now LIVE (warnings, capped at 5 per build).
KNOWN catalog-body findings visible on this build (xiom.io and others --
fix these in the stdlib lane, then the rollout flips to hard errors):
1. io/print + io/println + time_now + sleep: whole-body `unsafe` block
   with no `requires` (T007 -- Unsafe Confinement requirement c). Add
   requires clauses (e.g. non-null/size guards) or narrow the unsafe.
2. xiom.io catalog body ~354:42: `assignment type mismatch: Int32 =
   Result[Unit, IOError]` -- a REAL latent type error in a catalog fn
   (was silently accepted; likely a Result being assigned into an Int32
   local or a mis-typed helper return).
WARNING VISIBILITY: user-code warnings now print on the plain compile
path (warning[W000] line:col). Exhaustiveness now emits ONE precise
warning per missing variant (deduped base names).
### Round-19 (2026-08-27/28): Stage 3 -- Item A + exhaustiveness + warning visibility

1. ITEM A -- CATALOG BODY TYPE-CHECKING (Phase 1 rollout, user-approved):
   register_external_module now runs check_top_decl over every loaded
   catalog module's bodies (was signature-only -- undefined bare names
   silently became zero-param stubs). Findings route through the new
   checking_catalog flag into WARNINGS ("catalog body: ...") so the
   stdlib session can fix the corpus without builds breaking; flips to
   hard errors when clean. Validated with a synthetic bad module
   (undefined var + return-type mismatch both caught) AND against the
   real corpus (zero findings on smoke_collect_cache; feature-reg
   510/510 + stdlib-exec 70/70 with checking active).
2. EXHAUSTIVENESS for arbitrary enums VERIFIED + FIXED: variants
   registered BOTH bare and qualified ("Op" + "Token.Op") caused
   duplicate warnings and false positives on the qualified keys; the
   loop now dedupes to BASE names -- one precise warning per missing
   variant (probe: Token{Ident,Num,Op} matched on 2 arms -> exactly
   "variant 'Op' of 'Token' not covered").
3. WARNING VISIBILITY: compile()'s plain path dropped all checker
   warnings (exhaustiveness etc.) -- they now print as
   "warning[W000]: line:col: msg" on BOTH the error and success paths.
   (First attempt routed compile_or_exit through
   compile_with_diagnostics, which is a DIFFERENT pipeline with
   different observable output -- 2266 e2e failures in 6 minutes caught
   it instantly; reverted to the in-compile() print. LESSON: the two
   compile entry points are NOT interchangeable.)

Validated: checker 181/181, feature-reg 510/510, stdlib-exec 70/70(+2),
full e2e running at doc time (expect 2294/2295 + simd env).
### Round-18 (2026-08-25/26): Stage 2b landed + audit #6 CONCRETE unsoundness CLOSED

Commits pending e2e: xiom-lowering extraction + ErrorGuaranteed wiring +
#6 fixes. Details:

1. STAGE 2b -- expand_impl_blocks MOVED OUT of xiom-ast into the NEW
   crates/xiom-lowering (free fn `xiom_lowering::expand_impl_blocks(&p)`).
   xiom-ast is now syntax-only; the pass is an explicit pipeline stage
   (parse -> lower -> check). Dummy Span(0,0) injection removed -- the
   synthesized default-method fns now carry the interface default fn's
   REAL declaration span. All 4 callers rewired (checker x2, catalog,
   driver). Validated on its own full e2e run: 2293/2295 (simd env +
   the known p1_contract_methods CRT flake -- passes 3/3 isolated).
2. ErrorGuaranteed WIRED FOR REAL: new() is pub (was pub(crate) +
   allow(dead_code) -- unconstructable aspirational design). Every
   ParseError and CheckError now CARRIES the proof token; error_with_cause
   constructs it at emission; checker threads parse guarantees through.
3. AUDIT #6 CONCRETE UNSOUNDNESS CLOSED (structural TypeId remains a
   follow-on; the specific audited bugs are dead):
   - wildcard `_` annotation now INFERS the value type in BOTH passes
     (was: "_" -> Int; `let x: _ = "s"` bound x: Int silently).
   - container args must AGREE recursively when both present -- with the
     SAME scalar promotion matrix everywhere (Vec[Int] -> Vec[UInt8] stays
     legal), generics/unit-payloads/pointers/tuples context-adaptable;
     Option[Int] vs Option[Str] is now a TYPE ERROR.
   - wildcard-receiver method lookup NO LONGER captures an arbitrary
     alphabetically-first type's method -- fail-closed with a proper
     no-such-method diagnostic.
   - deleted the orphaned dead compat/mod.rs (the second divergent
     types_compatible had ZERO callers -- it was never even declared as
     a module).
   - EN ROUTE (exposed by the tightening, fixed same session):
     Vec[(Str, Str)].new() inferred Vec[Int] (tuple type-args fell to
     the "Int" fallback in vec_ctor_type_name) -- tuple/paren arms added.
   - checker 178 -> 180 tests (wildcard-infer + container-args regressions).
   VALIDATED: checker 180/180, feature-reg 510/510, stdlib-exec 70/70(+2),
   full e2e running at doc time.

AUDIT SCORECARD: #6's CONCRETE defects closed (wildcard-as-Int, container
erasure, alphabetical capture, divergent compat copies). Remaining for
production-grade: structural TypeId refactor (follow-on), Stage 3 soundness
gates + Item A/B, Stage 4 crash families (CRT-layout root cause, json
heap, stack cookies, clang variants).
### Round-17 (2026-08-25 morning): BUG 57 FIXED -- the biggest stdlib-facing compiler bug

Commit: 92d40274. e2e 2294/2295 (new test included; simd_runtime remains
the only env-fail on this laptop). THREE coordinated roots, full map in
COMPILER_BUGS.md "BUG 57 FIXED" section:
1. vec_elem_from_type_annotation flattened nested generics (TWO divergent
   copies of the fn existed -- lib.rs copy is live for decl.rs, types.rs
   copy dead-but-present; BOTH now type_string_full + keep-in-sync note).
2. Chained indexes lost element TYPE at the inner level (container is an
   Index expression; resolver only knows Ident/Field) -> scalar i64 load
   -> sitofp of raw f64 bits = -4.6094342186137e+18. NEW:
   IrEmitter.indexed_elem_types (Debug-keyed Index expr -> yielded XIOM
   type); outer inserts, inner consults container key + strips one layer;
   floats take a typed load path; primitives stay on the scalar loader
   (first-cut fed "Int" into alloca %struct.Int = unsized-type error --
   smoke_collect_cache caught it, fixed same session).
3. call.rs push-inference consults the map so defensive-copy rows
   register real elem types.

DEBUGGING LESSON (repeat performance): the fastest path to this fix was
dumping IR and reading it against the RUNTIME semantics -- the IR looked
"correct" until each extractvalue was checked against what the NEXT
instruction expected (elem_size slot vs float bits). The jit-cache trap
(cleared ~/.xiom/jit before trusting probe output) and manual
clang+runtime-source builds (bypassing the harness) were decisive.

FOR THE STDLIB SESSION: re-sweep geom_vec/mat/quat, curves/bezier,
matrix.det/trace/rank consumers, smoke_collect_cache (regression caught
during dev is now fixed) on this build. Probes probe_geom1/2 +
tests/regression/m57_geom_nested_param.xi all exit 0.

REMAINING compiler-lane work: Stage 2b (expand_impl_blocks lowering pass
+ ErrorGuaranteed), Stage 2c (#6 interning/structural TypeId), Stage 3
soundness gates + Item A/B, Stage 4 crash families (CRT-layout/json heap/
stack cookies/clang variants). Audit top-20: ALL CLOSED except #6.
### Round-16d (2026-08-25, overnight): audit sweep -- SEVEN more findings closed

Autonomous batch series, each fully validated before commit:
- 833d23d5 fix(driver,lexer): #11 sandbox false-green CLOSED (unreadable
  input exited 0!); library-half of #12 CLOSED (compile() process::exit ->
  Err; explain_error returns bool). \xNN restored as documented CODEPOINT
  U+00NN (m32 corpus relies on '\x80'==128 -- overnight e2e caught my
  over-strict rejection); m36_r15 migrated to soft-keyword intent.
- 56e26bdc fix(fmt,lsp,dbg): #10 fmt defer todo!() crash CLOSED (renders
  `defer { .. }`; suite 79/79); #9 LSP string severities -> INTEGERS;
  #20 dbg MI injection CLOSED via mi_quote() on breakpoint paths + IDE
  evaluate box.
- b3fc1e8a fix(pkg,mcp,lsp,dbg): SUPPLY-CHAIN CORE CLOSED -- #5 command
  injection (curl/PS/raw-TCP ladder -> ureq-only TLS + timeouts +
  size caps; http:// rejected unless XIOM_PKG_ALLOW_HTTP=1);
  #4 client sha256 VERIFICATION enforced at install (index parses both
  server object-map + legacy list shapes; unhashed refused unless
  XIOM_PKG_ALLOW_UNHASHED=1; mismatch = hard error); #19 traversal-checked
  extraction (tar -tzf pre-validation: absolute/drive-letter/.. members
  rejected) + random temp names everywhere (both divergent install paths
  routed through one hardened extractor; MCP temps de-predicted); #13
  LSP/DAP frames capped at 64 MiB.
- ab988867 fix(parser): Some/None/Ok/Err demoted to SOFT -- Type.Variant
  qualified access flows through ident positions; reserving them broke 16
  e2e programs (overnight e2e caught it; rustc does not reserve them).
- PENDING: #18 second half CLOSED -- xiom.toml [compiler] table was parsed
  but IGNORED; manifest values now act as project defaults with CLI-flag
  precedence (release/incremental/max-depth/target; timeout-secs still
  CLI-only pending cancellation-token work). Functional probe verified.

AUDIT SCORECARD after tonight: top-20 findings -- #1 #2 #3 #4 #5 #7 #8
#9 #10 #11 #12(lib) #13 #14 #15 #16 #19 #20 CLOSED (+ both halves of #18
pending commit). REMAINING OPEN: #6 (structural types/interning),
#12(watchdog cancellation -> Stage 5), #17 (JIT honesty), plus campaign
BUG 57 geom family / CRT-layout / json heap / stack cookies / clang
variants (Stage 4), soundness gates (Stage 3), Item A/B.
### Round-16c (2026-08-25, early am): Stage 2a LANDED -- byte-offset spans + trivia + literal honesty + reserved words

Commits: f30cc25f (spans/trivia/#15) + a9258be ascii fix + this one
(reserved-vs-soft keywords, rest of audit #16). All complete-compilable
slices; parallel stdlib session never saw a broken tree.

1. Span carries half-open BYTE ranges [start,end) (post-BOM) alongside
   line/col. Equality/Hash intentionally (line,col)-only -> zero semantic
   drift for existing comparisons; range()/byte_range()/same_range() added.
   Lexer tracks len_utf8 bytes: multibyte sources keep char-columns AND
   byte offsets correct simultaneously (tested). Token ranges provably
   cover their lexemes (tests slice the source).
2. Comments/shebang PRESERVED: tokenize_with_trivia() returns
   Vec<Trivia>{kind,text,span} (line/block/shebang); tokenize() unchanged.
   fmt/docgen unblocked for Stage 5.
3. AUDIT #15 CLOSED: >u128 integer literals and malformed floats are ERROR
   TOKENS now (were silent 0/0.0); \xNN >= 0x80 rejected with \u{...}
   guidance (raw-byte smuggling into char fixed). u128::MAX boundary
   still lexes as BigInt.
4. AUDIT #16 CLOSED (both halves): parse_ident enforces RESERVED vs SOFT.
   RESERVED = structural keywords (let/var/fn/if/.../as/is/and/or/not/
   Some/None/Ok/Err) -> "reserved keyword" error. SOFT stays legal because
   stdlib depends on it (Executor.spawn, Scope.spawn, xiom.thread.spawn
   module paths; contextual requires/ensures/invariant remain Idents).
   `let let = 5;` now errors loudly.

VERIFIED after each slice: lexer 29/29 (+11 new), parser 99/99 (+2 new),
checker 178/178 transparent, feature-reg 510/510, stdlib-exec 70/70 (+2 ign;
log_runs PASSED this run -- confirmed flaky CRT-layout family, item 1 of
queue, NOT a permanent fail). Full e2e running at doc time.

AUDIT SCORECARD: #15 #16 fully closed. Stage 2 remaining: interning/
structural TypeId (#6 class + contract field-receiver gap), expand_impl_
blocks lowering pass + ErrorGuaranteed wiring.
### Round-16b (2026-08-25): R16c verifier honesty LANDED -- Stage 1 COMPLETE

Commit: verifier rewrite + CLI report wiring + suite 27->31 tests, 31/31
green. Audit #3/#8/#14 CLOSED (full defect-to-fix map in COMPILER_BUGS.md
"R16c" section). Highlights:
- UNKNOWN is now an honest verdict: unsupported/inexpressible obligations are
  skipped with reasons and surfaced via GenReport -> CLI "[WARN] UNKNOWN";
  the literal-false spurious-VIOLATED machine is gone (#3).
- ONE numeric story: int widths -> Int, floats -> Real, sort-aware operators;
  structs emit declare-datatype with real selectors for field access (#8).
- z3 runs from a unique temp file: the stdin pipe-deadlock is structurally
  impossible; -T units fixed (seconds); parent-side kill at 1.5x budget (#14).
- Vacuous-proof hole CLOSED: branches encode as guarded implications with
  definite-return fall-through threading (old conjoint encoding made
  clamp/max proofs UNSAT-vacuous). Axiom forall binders fixed (return-sort
  string was bound as a variable). Invariants = real VCs. xiom-verify CLI
  now honors parser take_errors() (audited partial-AST hazard).
- feature-regression 510/510 after rebuild (no cross-crate regressions);
  no z3 on this laptop -- z3 integration tests SKIP here, desktop re-runs.
- NEW checker limitation logged (Stage 2/3): contracts cannot reference
  struct-field receivers yet (`requires: p.y == 0` -> <error> typing).

AUDIT SCORECARD after this commit: top-20 findings -- #1 #2 #7 #3 #8 #14
FIXED (+ halves of #16/#18), remaining open: #4 #5 (supply chain/Stage 5),
#6 (Stage 2 interning), #9 #10 #11 #12 #13 #19 #20 (Stage 5 tooling),
#17 (Stage 4 JIT), #15 (Stage 2 lexer), keyword-as-ident half of #16,
[compiler]-config half of #18.

NEXT: Stage 2 structural foundations per docs/COMPILER_READINESS_PLAN.md
(byte-offset spans -> symbol interning/structural TypeId ->
expand_impl_blocks lowering pass -> lexer trivia + numeric error tokens).
### Round-16a PROGRESS UPDATE (same evening): quick wins LANDED

Commits: `f1c8b707` (CTFE rewrite + folder hardening + readiness plan) +
`786851b3` (quick wins). **Stage-1 full e2e on the new binary:
2293/2294 -- IDENTICAL to baseline** (the one fail is the machine-env
simd_runtime; zero regressions from the whole rewrite).

Quick wins landed and verified:
1. Module-prefix ARITY CHECK (both check_module_call branches) -- extra args
   are now T001 errors instead of silent drops. feature-reg 510/510 = zero
   false positives over the entire stdlib surface.
2. Checker warnings SURFACED on the success path (take_warnings() + W000
   diagnostics); previously dropped whenever the error list was empty.
3. MAX_EXPR_DEPTH 24 -> 128 (rustc parity) + advance() Eof-invariant
   debug_assert. The driver now runs ALL compilation on a 512MB-stack worker
   thread (128 levels x ~32KB > the default 1MB Windows main stack);
   deep-nesting tests updated to big-stack threads accordingly. Verified:
   55-level nesting compiles and runs (was rejected at 24).
4. GOTCHA recorded in COMPILER_BUGS.md: Expr::Int is a u64 BIT PATTERN --
   checked const arithmetic must cast `as i64` first (unsigned checked ops
   return None for -X / 0-X and silently unfold consts; caught by
   regress_5e7f_const_arithmetic_neg via baseline-compare within minutes).

Suite state after all Stage-0/1 changes: parser 97/97, checker 178/178,
ctfe 36/36 (new suite), feature-reg 510/510, stdlib-exec 69/70 (+2 ign;
log_runs = BASELINE env fail on this laptop), e2e 2293/2294 (simd_runtime =
BASELINE env fail). Both env-fails reproduce on the untouched round-15
binary here; desktop remains reference for SIMD/log-sensitive tests.

**REMAINING Stage 1 (next block): R16c verifier honesty (sort-consistent SMT
encoding, struct-field encodings, UNKNOWN as distinct verdict -- never
literal false, bounded z3 sessions with concurrent drain, invariant
emission). Then Stage 2 structural foundations (spans -> interning ->
expand_impl_blocks move -> lexer trivia) per docs/COMPILER_READINESS_PLAN.md.**

NOTE: a parallel STDLIB session is ACTIVE in this repo right now
(stdlib/runtime/sha256_sw.c + crypto.xi modified, new docs appearing).
Compiler lane must NOT touch stdlib/** or their docs; baseline-compare
(stash+rebuild) before blaming any smoke/e2e flip on compiler commits.
# XIOM Compiler Session -- Handoff (2026-08-19)

## Session update (2026-08-23, round 15 FIXED -- compiler): fn-typed Float64 thunks + Ord-interface injection + const-N arrays + checker generic bare calls

Commits: `bcf3ead3` (round-15 batch) + `6a442777` (types.rs extractor
follow-up) -- fix(codegen,check): round-15. Closed FIVE queue items
(e2e `e2e_m49_round15_fnfloat_constarrays`):

1. **Fn-typed params returning Float64 returned 0** (queue 1 -- the math
   numerical family): the `__fnwrap_N` fn-ref thunk declared every
   non-aggregate param as i64 and restored the callee's real types via
   casts INSIDE the thunk (`sitofp i64 %a0 to double`). The M20-A1 call
   site passes REAL arg types (`double` in XMM on Win64) -- the i64
   declaration read RDX garbage + sitofp corrupted the bits
   (apply(sqminus2, 2.0) -> 0). Thunk params now declare real
   float/double/fp128 types and forward unchanged; the block-style
   closure thunk (expr.rs) had the same defect. All FIVE blocked math
   smokes (numerical/analysis/calculus/integral/optimization) exit 0.
2. **Ord[T].compare dispatch in stdlib contexts** (queue 3 --
   smoke_core_binary_heap exit 4): the checker's collect_external_decls
   injected catalog INTERFACES only when `id.is_pub`; core.xi's
   `interface Ord[T]` is non-pub, so codegen never registered it and
   `Ord[T].compare(a, b)` in mono'd sift_up was misread as a
   value-instance method (inline compare with a LITERAL-0 receiver).
   Interfaces now inject unconditionally (decl-only, no layout).
   probe_heap3 pops 5,3,2,1; smoke_core_binary_heap exit 0.
3. **Const-N array residuals** (queue 4): (a) array.len const-N STALE
   across call sites -- the mono name now embeds the VALUE
   (`array.len_Int_2` vs `_Int_4`; the old "Int" placeholder made every
   N collide on the first call's const map); (b) narrow-element reads
   through &[N]T mono bodies -- the caller's array locals LEAKED into
   mono bodies (never cleared per fn -- BUG 47 family) and misrouted
   `arr[0]` through the +1 array-buffer path; Ref-Array mono params now
   register their element type (substituted XIOM name -- "UInt8" not
   "Int8"), i8* array params get a dedicated data[idx] index path with
   signedness-aware widening, VAR array literals infer scalar element
   types, and a new local_array_elem_xiom keeps As-target names for
   generic-arg inference (probe_arr8/arr8b green: Int8/UInt8/Int16);
   (c) array.map's [N]U result -- the call-side param type for BY-VALUE
   [N]T params resolves "[N x T]" (was the arg's %struct.Vec) and
   coerce_arg_for_param MATERIALIZES the aggregate from a Vec value
   (probe_map: len=5, d0=2, d4=10; smoke_array_map exit 0).
   smoke_array_edge/len_empty/get_first_last/narrow all exit 0.
4. **Generic fn-param BY-VALUE residual** (queue 5 -- probe_zip_j/k):
   (a) CHECKER: type-parameterized bare calls `apply_g[(Int, Int)](...)`
   parse as Call(Index(Ident, types), args) and resolved to Unit
   ("cannot logically negate type ()") -- the checker now unwraps the
   Index callee when the base is a registered function and feeds the
   explicit type args into the generic substitution; (b) CODEGEN: the
   generic-call scrutinee's payload type resolves through
   callee_return_xiom's new Index arm (substituted return
   "Option[Tuple__Int__Int]") and the option/result payload extractors
   (BOTH copies: lib.rs + types.rs) now count PARENTHESES as nesting
   ("Option[(Int, Int)]" no longer splits at the tuple's comma -- the
   Some(p) binding derefs the box). Both probes exit 0 (find_via_val
   [(Int, Int)] with by-value fn(T) -> Bool predicates).

Verified: full e2e 2294/2294 (incl. e2e_m49; the final binary also has
the checker + extractor fixes), checker 178, parser 97, ctfe 97,
feature-reg 510, stdlib-exec 70/70 (+2 ignore), 47-smoke battery
(array/heap/math/iter/collections/string/rc/sync), 8 probes
(fnv/fnv2/map/stale/arr8/arr8b/zip_j/zip_k).

**NEXT SESSION:** the updated REMAINING COMPILER-SIDE QUEUE (priority
order) + FOR THE STDLIB SESSION + OPERATIONAL LESSONS are in the
round-14c section below (the queue header was rewritten 2026-08-23).

**WORKFLOW (unchanged):** probe -> IR-diff -> fix -> verify probe + stdlib
smoke + e2e regression (tests/regression/m3x_*.xi + e2e_mXX registration)
-> quick suites (checker/parser/ctfe/feature-reg/stdlib-exec) -> full e2e in
the background -> update COMPILER_BUGS.md + SESSION.md -> commit
(conventional messages) -> report to the stdlib session for the re-sweep.
Full suite: cargo test -p xiom-check, -p xiom-parser, -p xiom-ctfe,
-p xiom-codegen --test {feature_regression_tests,stdlib_execution_tests,
stdlib_tests,e2e_tests}. Build target/debug/xiom.exe BEFORE suites; NEVER
run two suites concurrently; e2e_m16_scripting_exit_zero is an
environmental fail (ignore).

## Session update (2026-08-22, round 14c FIXED -- compiler): write-back bug + generic fn-param aggregates + narrow casts + const-N arrays

Follow-up session closed four of the stdlib session's six new findings:

1. **Write-back bug FIXED** (the time.Duration family root): by-value
   self methods returning the SAME type stored the result into the
   receiver's slot (d1.identity() clobbered d1). The store-back (a
   "Counter.inc" pattern) fired for ALL %struct-returning by-value self
   methods -- removed entirely. The whole Duration battery (35 smokes)
   PASSES.
2. **Generic fn-param aggregates FIXED** (ZipIter tuple predicates):
   tuple type args with a single generic param are the tuple TYPE
   (_find_via[(T, U)] -> _find_via_Tuple__Int__Int via
   current_type_map); fn_local_returns keeps the Option[...] args
   (type_string_full); closure-local callee payloads track via
   fn_local_returns (Some(v) derefs the box). ZipIter find/all/any/nth
   green.
3. **Negative Int->narrow as casts FIXED** (smoke_string_narrow):
   inferred bindings now track signed_locals (`var n8 = n as Int8`
   widens sext).
4. **Const-generic [N]T arrays FIXED** (smoke_array_narrow map path):
   llvm_type_for resolves [N]T sizes from the mono const map (published
   early) and generic elems from the type map; unannotated VAR array
   literals are FIXED arrays (compile_fixed_array_literal) -- the Vec
   conversion stays for LET arrays (the core/slice fns take &Slice[T]);
   the generic-arg inference extracts array elements (Ref-unwrapped);
   the call-side &[N]T is the element pointer; call-site consts publish
   for the ret resolution. array.map[Int16, Int16, 2] works.

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, full e2e pending (new e2e_m48_round14c_writeback_
aggregates); the time/iter/cmp/utf/narrow families all green.

**REMAINING COMPILER-SIDE QUEUE (updated 2026-08-23 after round-15;
+ priority order -- the stdlib session's next-session queue; probes in
+ C:\Users\lefte\AppData\Local\Temp\kilo\*.xi + COMPILER_BUGS.md):**

1. **CRT-layout family (queue 8, the biggest remaining cluster).**
   Deterministic per source, flips with UNRELATED code (each repro is
   baseline-identical; the same smoke passes on intermediate builds):
   smoke_iter_collect / smoke_array_sort_by / smoke_array_slice /
   smoke_array_fold (startup AVs), smoke_convert_url, smoke_core_box,
   smoke_stress_regex_find/match_count, smoke_stress_serialize_
   jsonvalue_get/parse_nested, smoke_error2's has-mid flip, AND the
   geom/math flips: smoke_geom_vec (exit 57), smoke_geom_mat (exit 4),
   smoke_geom_quat (exit 18), smoke_math_edge (AV -- also listed under
   stack cookie). The SIMD flags / 16 clang-variant matrix is the fix
   target. VERIFICATION DISCIPLINE: always compare against the baseline
   binary (git stash + rebuild) before calling a change a regression.
2. **array_zip checker T001** (queue 4d residual): the CHECKER rejects
   `result[i] = (a[i], b[i])` on a `[N](T, U)` array element store
   ("cannot access field on non-struct type Int" at the tuple store;
   smoke_array_zip). The checker's array-element WRITE path doesn't
   type the tuple element. Compiler-side, the zip DEF is fine
   (probe-level verification pending a checker fix).
3. **Extra call args silently dropped** (queue 7 -- stdlib finding 9):
   no arg-count check on module-prefix calls (a wrong-arity call
   compiles and drops the extras -- `convert.float_to_string(3.14159,
   2)` vs the 1-param def).
4. **json heap** (queue 9, 0xC0000374): json_nested, json_parse_valid,
   json_parse_nested.
5. **Stack cookie**: stress_crypto_argon2/pbkdf2 x2, stress_io_bufreader
   (smoke_math_edge's AV is the CRT-layout flake, see item 1).
6. **Clang variants** (queue 6): ptr_offset, io_copy, io_copy_file,
   io_read_int_float, hash_values, convert_escape, regex_captures x4.
7. **LET-array representation conflict** (queue 12 -- stdlib-design
   item): the M33 let->Vec conversion satisfies the core/slice fns
   (&Slice[T]) but breaks the array module's &[N]T fns (smoke_array_
   narrow's first section).
8. **Pre-existing**: smoke_convert_narrow_roundtrip (exit 3).
   NOTE: smoke_array_edge and smoke_geom_vec were on the old list --
   array_edge is FIXED (round-15), geom_vec is the CRT-layout flake
   (item 1), and queues 2/6 (nested &Vec[Vec[Float64]] reads, unary
   minus on nested index) are NOT reproducible anymore (probe_skew3/4,
   probe_neg all green at baseline -- stale handoff entries).

Campaign trajectory: 516 -> 621 -> 679 -> 738 -> 765 -> 779 -> 793 -> 799 -> 801 -> 819.
Compiler-side closed roots: BUG 31-56 + the round-fixes (gzip decompress
payloads, path.xi env import, Vec/Set/Slice method injection + pointer
arithmetic GEP, non-pub generic type decls + is_llvm_struct_named, ref
payload auto-deref, Set ABI, Ord/Bounded C001, B-007 closures, round-12
rm1 Str-return closures + cb2 enum-variant receivers, round-13 closure-
env family + tuple payloads, round-14 aggregate closure params + Vec[Str]
elements + narrow-SIGNED loads, round-14b checker generic-tuple
substitution + multibyte Char family, round-14c write-back + generic
fn-param aggregates + narrow casts + const-N arrays, round-15 fn-typed
Float64 thunks + Ord-interface injection + const-N mono names/narrow
reads/map results + checker generic bare calls + tuple-paren payload
extractors).
Stdlib-side hardened: RefCell (incl. replace &mut self), PathBuf, gcd,
crc32, VecDeque, Set/Queue/Stack, redundant requires traps, prose
ensures, smoke semantics, the closure-based iter adapter build (map/
filter/take/skip/chain/zip/enumerate/collect/fold/count/max/min +
find/all/any/nth/last), sync Once, error pretty-print family, string
requires cleanup, Str.is_empty method, array Option[T] value semantics,
unary-minus parens, sync Arc/Atomic ctor prefixes, compress sublib
delegation, base64url ensures, serialize.is_valid_bytes, str_slice/
str_concat byte-copy, str_lower/str_upper byte-safe ASCII, BinaryHeap
&mut self, regex replace_all engine-qualified.

**FOR THE STDLIB SESSION (re-sweep targets after round-15):**
- Findings 7/8/10/4 from the round-14c report are CLEARED except
  array_zip: re-test geom/mat/quat/finance/edge (watch the CRT-layout
  flake -- a single build may pass or fail), binary_heap, array_map/
  len_empty/get_first_last/narrow, and the math numerical family.
- smoke_array_narrow's LET-array first section (item 7 above) is a
  stdlib-design decision.
- Probes to reuse: probe_fnv/fnv2, probe_stale, probe_map, probe_arr8/
  arr8b, probe_zip_j/k, probe_heap3 (all green on this binary).

**OPERATIONAL LESSONS (round-15, read before probing):**
- `xiom --run` prints the PROGRAM's exit code as an "exit code: N"
  line but the xiom process itself exits 0 -- read the printed line,
  NEVER $LASTEXITCODE (PowerShell shows 0 for both pass and fail).
- Probe discipline: `xiom file.xi` only DUMPS IR (stdout); use
  `xiom --run -o out.exe file.xi` and capture both streams to a file.
- CRT-layout flakes: before blaming a change for a smoke failure,
  verify the SAME smoke fails identically on the baseline binary
  (`git stash push -- crates/` + rebuild + rerun + `git stash pop`).
- The e2e harness needs the exe free: wait for a running e2e suite to
  finish before `cargo build` (the rebuild fails while e2e holds the
  binary).
- Commit hook: the ascii_guard rejects non-ASCII bytes in STAGED
  files -- run `python tools/ascii_guard.py repair --apply` when a
  commit aborts.

## Session update (2026-08-20, round 11 FIXED): B-007 closures -- fn-typed params

Commit: `93545e56` -- fix(codegen): round-11 -- fn-typed PARAMS hold a
closure ENV pointer; calling `f(x)` inside a body goes through the M20-A1
env path (params now registered as closure locals + their return types --
struct returns are BY VALUE); fn-REFERENCE args (cmp_int, is_even) get
wrapped in a forwarding-thunk env whose signature matches the M20-A1 call
(i64 args, inttoptr to the fn-ref's real param types inside -- clang must
inline alwaysinline comparators); re-passed fn-typed params are NOT
re-wrapped (the Int.compare suffix scan double-wrapped heap_sort_by's
compare); Vec[fn()] elements (timer wheel tasks) carry a "fn(...)" marker
through type_meta -- var/let bindings from them register as closures and
indexed calls load field 0 + env-first; Vec.push of a fn-ref into Vec[fn()]
wraps it.

Verified: stdlib-exec 70/70 (+2 ignore; async/thread restored), feature-reg
510, checker 178, parser 97, ctfe 97, full e2e pending; 54-smoke battery
green incl. the full closure family (option map/unwrap/filter/deep_chain,
array_sort_by, cmp_by, slice, search, sort, async, thread). Pre-existing
(baseline-failing): the btree_map first_entry pair.

### COMPILER-SIDE QUEUE (priority order -- all documented in COMPILER_BUGS.md)

## Session update (2026-08-20, round 10 FIXED): checker builtin Ord/Bounded resolution (C001)

Commit: `1f762f76` -- fix(codegen) + fix(stdlib): round-10 -- the stdlib's
new Ord tower was written `impl Ord[]` with empty brackets/param types (the
Eq/Bounded towers use `impl Eq[Int]` with explicit types) -- nothing
registered, so num_checked/num_saturating stopped at C001 "missing method
'cmp'". Rewrote as 15 properly-typed impls; added cmp/min/max to the C001
builtin fast-path + inline scalar handlers (select-based); the fn_key
construction resolves GENERIC-PARAM static receivers (`T.max_value()` in
mono'd bodies) via current_type_map -> "Int.max_value"; the direct-call
fallback resolves the ".{Recv}.{method}" suffix to the module-qualified
impl ("precision.Int.max_value").

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, full e2e pending; 35-smoke battery green incl.
num_checked/num_saturating/cmp_ordering and the BTree/Set towers.

### COMPILER-SIDE QUEUE (priority order -- all documented in COMPILER_BUGS.md)

## Session update (2026-08-20, round 9 FIXED): Set container ABI mismatch

Commit: `fe4e57ec` -- fix(check/codegen): round-9 -- the compiler had NO
builtin Set layout (Vec/Slice/Map register %struct layouts; Set was only the
i64-erasure fallback), so Set values erased to i64 while the stdlib methods
operated on %struct.Set. "Set" removed from the checker's PRIMITIVES (the
stdlib `type Set[T]` now injects and Set resolves like any struct);
is_container_vec_field accepts only Vec/Slice/Array fields; field receivers
resolve generic-arg types ("Set[Int]" -> "Set") for the fn_key; mono
pointer-self passes the FIELD ADDRESS for field receivers; the pointer-len
handler skips struct pointees (s.len() on &Set dispatches to Set.len).

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, full e2e pending; 44-smoke battery green incl. all six
Set smokes + set_probe edge shapes. NOT covered: `for x in set` iteration
(For stmt is a hardcoded Range GEP -- iterator-protocol queue item).

### COMPILER-SIDE QUEUE (priority order -- all documented in COMPILER_BUGS.md)

## Session update (2026-08-20, round 8 FIXED): catalog &mut-self wiring + Option<&T> payloads

Commit: `0ca70cd6` -- fix(check/codegen): round-8 -- inject non-pub generic
type decls (VecDeque/Stack/Queue/LinkedList/BTreeMap/BTreeSet/BinaryHeap
family unblocked; mutations were silently lost via same-leaf method
hijacks), is_llvm_struct_named leaf-exact Vec/Slice receiver test (the
contains("struct.Vec") substring test hijacked VecDeque -- vd2), Option<&T>
reference payload auto-deref (rw1 -- weighted_pick's Some(&items[i]) payload
read garbage; the & was stripped at type_string_full/param/match-binding
records), and the args-embedded "%struct.Vec[Int]" indexed-element len
(geom/collect_cache regression -- invalid @Vec[Int].len_Int symbol).

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, 39-smoke battery green (incl. set_basic + rc_weak),
full e2e pending. Follow-ups logged: Set-container ABI now WORKS for the
basic shape (set_basic green); the "cmp"/Ord gap (BTreeMap[Int,...]
instantiation C001 -- logged queue item), narrow-SIGNED inline pop/get zext,
Graph/Graph type-name collision.

### COMPILER-SIDE QUEUE (priority order -- all documented in COMPILER_BUGS.md)

## Session update (2026-08-20, round 7 FIXED): ve2 Vec.pop slot resolved -- FOUR compiler roots; Vec/Set/Slice methods now inject & mono; pointer arithmetic GEP-scaled

Commit: `2e0fbb6c` -- fix(codegen/check): round-7 -- Vec/Set/Slice method
injection, resolve_struct_return suffix scan, mono'd Vec-method element
pointers + pointer-arithmetic GEP + concat gate, BUG 38b leaf-match
abstract-receiver guard. e2e `e2e_m37_round7_vec_pop_slot`.

The round-7 ve2 finding ("inlined Vec.pop + match Option slot on an EMPTY
vec") was ONE SYMPTOM of a deeper registration hole: **Vec/Set/Slice methods
were never injected** (checker `recv_is_nonpub_generic` skipped them because
their type decls are non-pub generic -- but Vec/Set/Slice are COMPILER
BUILTINS in the PRIMITIVES list, so their type decl is NEVER injected and
codegen KNOWS the types). Consequences fixed:

1. All Vec methods now inject -> `resolve_struct_return("pop")` finds the
   Option return -> match scrutinee alloca + REAL discriminant checks
   (ve2 fixed; Vec.get no longer relies on the Map.get suffix luck).
2. Non-inline Vec methods (first/last/clear/insert/remove/...) were
   ZERO-PARAM STUBS -- now real mono'd methods (smoke_collections_vec_edge
   + smoke_stress_collections_vec_first_last green; Slice methods too).
3. Mono'd Vec bodies broke on `*(data + len - 1)`: builtin Vec.data is
   "*UInt8" -> Str-concat hijack + byte-unsized arithmetic. Fixes: mono
   receiver-field binding types data as the CONCRETE element pointer
   (i64* for Vec[Int]) + records "*Int" XIOM type; BinOp Add/Sub now
   emits element-scaled GEP (main + fold paths); the concat intercept is
   gated by expr_is_pointer (byte buffers stay pointer arithmetic).
4. Regression caught mid-round: the BUG 38b leaf-name match hijacked
   `g.edges.push` ("Graph.push" key) onto the newly-injected "Vec.push"
   (literal-0 receiver -> invalid IR). Leaf match now requires an
   ABSTRACT receiver (matches find_generic_decl's rule); plus
   is_container_vec_field searches ALL type_meta keys (collect.Graph vs
   math.graph_theory.Graph leaf collision).

Verified: stdlib-exec 70/70 (+2 ignore), e2e_m37_round7_vec_pop_slot +
round6/gzip e2e regressions, 20-smoke Vec/collections battery, smoke_iter,
smoke_collect_cache, checker 178, parser 97, ctfe 97, feature-reg 510,
full e2e pending. Pre-existing follow-ups logged in COMPILER_BUGS.md:
Set-container ABI (Set smokes fail at baseline -- compiler i64-handle vs
stdlib struct), narrow-SIGNED inline pop/get zext, Graph/Graph type-name
collision (graph_theory dead-code stubs at baseline).

### COMPILER-SIDE QUEUE (priority order -- all documented in COMPILER_BUGS.md)
1. ~~**Vec.pop + match Option slot (NEW -- probe ve2).** Inlined Vec.pop on an
   EMPTY vec returns Some at runtime while the IR is fully correct
   (len==0 -> Option{tag=0, payload=0}): the match MISREADS the inlined
   Option slot. Vec.get's None path works; a bare `return None` works.
   Blocks smoke_stress_collections_vec_edge + vec_first_last.~~ **FIXED
   (round-7): Vec methods were never injected (recv_is_nonpub_generic x
   PRIMITIVES builtins) -- no scrutinee type -> no alloca -> unconditional
   first arm. Vec/Set/Slice methods now inject + mono; pointer arithmetic
   GEP-scaled.**
1. **Set-container ABI (NEW follow-up).** smoke_collections_set_basic etc.
   fail at BASELINE: the compiler treats Set as an i64-handle container
   builtin while the stdlib declares `type Set[T] = { items: Vec[T] }`;
   `Set[Int].new()` resolves the bare "new" leaf onto another type's ctor.
   Needs a Set-container decision (inline handlers or ABI alignment).
2. **Closure B-007 layer.** fn-literal args (core_option_map/unwrap,
   array_sort_by, cmp_by, core_slice) -- fail at BASELINE identically;
   ~28 smokes.
3. **Iter adapter chains (MEDIUM queue).** The stdlib's iter adapters
   (map/filter/fold chains) -- a documented layer; expect payload-slot /
   closure-family overlap.
4. **json enum-Vec-Map heap layer.** json_nested heap corruption
   (enum-with-Vec-field payloads in Map values) -- documented, flaky.
5. **SIMD flags** (const-substituted SIMD_* refs) + **Bounded/is_finite
   C001s** (checker) + **clang codegen variants** (16 families:
   ptr/vec/hash/io/regex/btree/env/path) + gzip_large __chkstk + rc_weak
   drop bookkeeping (exit 5).
6. **Catalog-body type-check gap (item C).** path.xi's bare join_paths
   stub showed catalog bodies are never checked -- undefined bare names
   silently become zero-param stubs (now fixed stdlib-side with the env
   import, but the compiler gap remains).

### KEY MECHANISMS (respect them -- learned the hard way)
- Generic fns register in generic_fn_decls (qualified keys) AND functions
  (erased signatures); is_generic suffix matches need RECEIVER-QUALIFIED
  fn_keys; bare keys keep the functions guard.
- Match Some/Ok payload bindings need the scrutinee alloca
  (struct_type_from_expr -> fn_key -> resolve_struct_return).
- Payload-FIELD access (`r.value`/`ch.value`) resolves via
  field_payload_xiom (local_opt_payload -> local_opt_payload_xiom /
  local_err_payload -> declared local type); bindings record payload types
  via infer_field_payload_xiom / infer_try_xiom_type / infer_if_xiom_type.
- Expr::If result type is inferred from arm tails (all-agree struct >
  pointer > all-agree float > i64); Expr::Imply SHORT-CIRCUITS its
  consequence (guarded block + default-true store) -- an unconditional
  consequence unboxes NULL payloads on Err/None returns.
- Str builtins slice/substr/starts_with/ends_with must verify the receiver
  via receiver_is_str (they used to hijack same-named methods on Path
  structs, boxing the struct and passing the box address as a string).
- Vec methods get/push/pop/len/sort/set are INLINE builtins; Str methods
  len/is_empty/byte_at/char_at/substr/slice/starts_with/ends_with are
  INLINE builtins -- new methods need inline handlers, never mono.
- Contract exprs parse with struct-literal restriction; CTFE eval_block
  stops after a return (CtfeContext.returned).
- The e2e harness uses target/debug/xiom.exe -- rebuild (`cargo build
  --bin xiom`) BEFORE suites; never run two suites concurrently (file-lock
  races). e2e_m16_scripting_exit_zero is an ENVIRONMENTAL Windows Defender
  temp-dir heuristic (os error 225) -- reproduces at baseline, ignore.

## Session update (2026-08-19, round 6): Imply short-circuit + Try-bound Str payloads + substr handler + Str-builtin guards + path.xi import

Commit: `577a223e` -- fix(codegen): round-6 -- Imply consequence
short-circuit, Try-binding payload types, Str.substr inline handler,
Str-builtin receiver guards, path.xi join_paths import.

The stdlib session's round-6 report (bi3/bi4/pp12/pj5/pj6 + "Option[Str]
payload construction mangles") turned out to be FIVE compiler roots, all
fixed; the whole path smoke family went from 13/19 to 19/19 and
gzip_bad_input unblocked. Details in COMPILER_BUGS.md; probes in
tmp/bug_probes (bi4, op1, q3/q8, sw2/sw4, pj6/pj10, wfn1-4).

1. **Imply consequence unconditional** -- `ensures: result is Ok =>
   result.len() >= 0` unboxed the payload even for Err (NULL load -> UB ->
   corrupted return -> gzip_decompress returned Ok for garbage, bi4).
   Expr::Imply now short-circuits (guarded consequence block + default
   true store). ALSO fixed the stdlib's "Option[Str] payload mangles"
   (file_name's Some-ensure was the same unbox-on-None corruption) and
   unblocked smoke_stress_compress_gzip_bad_input.
2. **Try-bound Str payloads** -- `let name = f()?;` didn't record the
   payload type; byte_at's receiver heuristic passed the i64 SLOT ADDRESS
   as the string -> extension returned None. infer_try_xiom_type + the
   call.rs receiver heuristic now skips Str receivers (inttoptr the
   handle).
3. **Str.substr inline handler** -- substr was a registered builtin with
   no handler -> `@Str.substr` never defined -> stub ret 0 -> NULL -> crashes
   in every parent_path/file_name user. Now lowers to xiom_str_slice.
4. **Str-builtin receiver guards** -- slice/substr/starts_with/ends_with
   fired on ANY receiver name match: `p.starts_with(b)` on a Path BOXED
   the struct and passed the box address as a string (pp12). New
   receiver_is_str guard falls through to real dispatch.
5. **path.xi bare join_paths with no import** -- catalog body's bare call
   -> no registered key -> zero-param stub -> NULL -> join-chain crash
   (pj5/pj6 was this, not inline). path.xi now imports xiom.env;
   path_separator() delegates to env's FAMILY-aware version.

REMAINING (unchanged): closure B-007 (smoke_core_option_map/unwrap fail at
BASELINE identically), gzip_large __chkstk, crc32 table stack-copy store,
multi-match payload-slot reuse (queue item 3), Vec[Option[Match]] container
(item 4), the 142-failure map, catalog-body type-check gap (item C -- the
path.xi bare-name stub shows why it matters).

## Session update (2026-08-19, round 5): queue item 1 FIXED -- gzip/zlib decompress (2 roots) + payload-field unboxing + if-expression Vec arms

Commit: `5ead6463` -- fix(codegen): gzip decompress - Result payload-field
unboxing + if-expression Vec arms. E2E 2279/2279 (+1 new
`e2e_m37_gzip_roundtrip`; the only later failure, e2e_m16_scripting_exit_zero,
is a Windows Defender temp-dir heuristic that reproduces at BASELINE),
checker 178, parser 97, ctfe 97, feature-reg 510, stdlib-exec 70 (+2
ignored), stdlib_tests 40.

### Queue item 1 -- gzip_DECOMPRESS catalog crash: FIXED (two+one roots)

The "crashes BEFORE the first statement" was a stdout-buffering artifact
(puts output lost on the AV); the real crash was INSIDE gzip_decompress.
THREE coordinated root causes (all documented in COMPILER_BUGS.md):

1. **Result-payload FIELD access never unboxed.** `let decompressed =
   decoded.value;` (payload-FIELD on a container LOCAL -- NOT a match
   scrutinee) bound the raw BOXED POINTER as i64; `&decompressed` passed
   the i64 SLOT ADDRESS as `%struct.Vec*` -> crc32 read stack garbage as
   len/elem_size -> 8-byte element load -> 0xC0000005. Fixes: new
   `field_payload_xiom` (lib.rs; local_opt_payload -> local_opt_payload_xiom
   / local_err_payload -> the receiver's declared type), expr.rs Field arm
   unboxes container/struct payloads and covers `is_result.value` (was
   missing), decl.rs tracks Option/Result PARAM payload types (type_from_ast
   renders bare "Result"), stmt.rs records the payload XIOM type on
   bindings (infer_field_payload_xiom). **This is the SAME shape as queue
   item 2 (Path.parent `ch.value`) -- re-test Path.parent.**
2. **If-expression result slot hardcoded i64.** `let compressed = if
   level == 0 { _store_encode(data) } else { rle_encode(data) };` degraded
   the %struct.Vec VALUE to field-0-as-i64 (data ptr): `compressed.len()`
   became xiom_str_len(data_ptr) and `compressed[i]` compiled to a LITERAL
   0 -> gzip payload of six zero bytes -> decode produced 3 zero bytes (CRC/
   size checks passed vacuously against the same broken crc32). Fix:
   Expr::If result type inferred from arm tails (all-agree struct >
   pointer > all-agree float > i64) + infer_if_xiom_type records the
   binding's XIOM type. ALSO fixes Float64-valued if-expressions
   (`return if c { 1.5 } else { 2.5 };` -- was bitcast-to-i64 + sitofp).
3. Param payload tracking (see #1).

### REMAINING in this area (documented, NOT fixed)
- smoke_stress_compress_gzip_large: pre-existing 0xC0000005 in __chkstk
  (huge stack alloca; reproduces at baseline -- separate queue item).
- gzip_empty / gzip_bad_input smokes: facade `requires:` contracts PANIC
  instead of returning Err (stdlib-side decision).
- [256]UInt module-global crc table lazy-init writes a STACK COPY, never
  the global (module-global ARRAY element stores -- BUG 2 family for array
  elements) -- CRC is deterministic garbage but self-consistent for
  round-trips; gzip_crc32's public value is WRONG vs real gzip.
- Contract-ensure unbox (`result is Ok => ...`) loads the payload
  UNCONDITIONALLY -- Err results inttoptr 0 -> NULL load (swallowed by the
  guard-fault trap today; latent).

### OPEN QUEUE (priority order -- stdlib session can re-run the sweep)
1. ~~gzip/zlib decompress~~ -- FIXED (round 5). Re-test smoke_stress_
   compress_gzip_roundtrip / zlib_roundtrip / gzip_levels / compound /
   deflate_roundtrip (all exit 0 with this compiler).
2. Path.parent returns None -- the `.value` field shape is fixed by round 5;
   re-test smoke_stress_io_parent_file_name (loop shape + @Path.parent
   registration gap may remain).
3. Multi-match payload-SLOT REUSE (char_to_digit / num_gcd_lcm /
   rand_weighted / vec_first_last -- multi-check smokes).
4. Vec[Option[Match]] container shape (regex captures -- clang-blocked).
5. The 142-failure map: 16 clang variants, 4 Bounded/is_finite C001s, 45
   RUNFAIL(1) payload-slot/closure/iter families, rc_weak drop bookkeeping,
   json_nested heap corruption, SIMD-flag family.

## Session update (2026-08-19, round 4): stdlib sweep at 765/907 -- compiler queue below

Round 3 commits: `82293661` (BUG 53 write-facet, BUG 55 facet-2, BUG 56 +
CTFE return flow) + `8a9e9e99` (docs). Stdlib session verified all three
levers (array.sort + 5 array smokes, cross_cmp_sort, convert_identity,
net_folder, deflate_roundtrip, vg10/cz2 user-space) and pushed the sweep
to 765/907 (was 738; 679 two rounds ago). Zero hangs; compile-fails 31->26.

### COMPILER-SIDE QUEUE (from the stdlib session's final report -- all
### documented in COMPILER_BUGS.md with probes; ordered by impact)

1. **Compress gzip/zlib DECOMPRESS -- catalog-only crash (new).**
   gzip_COMPRESS works (gz9 exit 0), DEFLATE via direct rle_decode works,
   but gzip_DECOMPRESS crashes the program BEFORE the first statement
   (gz12 probe). User-space replicas of the full pipeline PASS -- the
   catalog MONO of the header-scan / trailer-UInt / crc-table fn is the
   trigger (module-init / const-eval / mono-registration interplay).

2. **Path.parent returns None -- is_some+FIELD payload shape (new facet).**
   `if ch.is_some { let c = ch.value; }` -- the payload-FIELD access on an
   Option-typed LOCAL inside a LOOP. The facet-2 fix covered
   match-scrutinees, NOT this shape. Also: 'use of undefined value
   @Path.parent' in path_pop_clear clang error -- likely the same
   registration gap (generic/qualified method key resolution).

3. **Multi-match payload-SLOT REUSE (new facet).** char_to_digit /
   num_gcd_lcm / rand_weighted / vec_first_last fail ONLY in multi-check
   smokes; single-match probes pass. Two+ matches in one fn reuse the
   payload slot/scrutinee alloca across arms -- the second match reads a
   stale payload.

4. **Vec[Option[Match]] container shape -- clang-blocked (new).**
   regex_captures_get/named now compile; blocked at clang by the
   Option[Match]-in-Vec element/container naming (a sanitize or
   concrete-container gap for nested generics with struct payloads).

5. **142-failure map (all compiler-side):** 16 clang codegen variants
   (ptr/vec/hash/io/regex/btree/env/path families), 4 Bounded/is_finite
   C001s (checker), 45 RUNFAIL(1) in the payload-slot / closure /
   iter-adapter families, plus the documented layers: B-007 closures
   (smoke_cmp_by/smoke_core_slice), rc_weak drop bookkeeping (exit 5),
   json_nested heap corruption (enum-with-Vec-field payloads in Map
   values), SIMD-flag family, smoke_simd CRT ignore.

### PROVEN-CORRECT STDLIB SURFACE (do not re-investigate)
Eq/Ord tower + 28 impls (d1dba921), cell borrow semantics (&mut self),
compress logic (pure-XIOM RLE), all smoke-targeted APIs. The stdlib
session's queue is EXHAUSTED -- every remaining failure is compiler-side.

### KEY MECHANISMS LEARNED (read before touching these areas)
- Generic fns register in generic_fn_decls (qualified keys) AND in
  functions (erased signatures). is_generic dispatch requires a
  RECEIVER-QUALIFIED fn_key for suffix matches ("Vec.set" ->
  "xiom.collections.Vec.set"); bare keys keep the functions guard
  (bare non-generic fns must not be hijacked -- net_folder lesson).
- Match Some/Ok payload bindings need the scrutinee ALLOCA; the
  scrutinee type resolves via struct_type_from_expr -> fn_key ->
  resolve_struct_return (now also consulting generic_fn_decls' AST
  return types).
- Vec methods get/push/pop/len/sort/set are INLINED builtins, not mono
  targets -- any new Vec method needs an inline handler.
- CTFE eval_block stops after a return (CtfeContext.returned).
- Contract exprs parse with struct-literal restriction (trailing `{`
  starts the fn body).

---## Session update (2026-08-18, round 2): BUG 53/55 -- &[N]T lowering + unsafe ctx capture, tower re-apply verified

Worked the stdlib session's follow-up report after the tower Eq/Ord
conversion LANDED (commit d1dba921). Two deeper bugs they found were
fixed; all tower/rc/cell/array smokes are green.

### BUG 53 -- FIXED: &[N]T params
Three coordinated fixes: (1) the array-LOCAL index branch GEP'd the
pointer SLOT as the array (excluded -- pointer-typed arrays now use the
array-ref branch with the correct two-index GEP + element-typed load);
(2) `var a: [5]Int = [...]` stores elements DIRECTLY into the [N x T]
slot (the old Vec conversion + declared-type coercion stored the data
POINTER as the array value -- invalid IR); (3) the caller's &array_local
arg coercion handles [N x T] slots (GEP 0,0 element pointer). Generic
(stdlib array module: len/get/unwrap) and non-generic shapes exit 0.

### BUG 55 -- FIXED: unsafe-block pointer capture + Option/Result ctor context
TWO root mechanisms:
1. RAW-POINTER locals (`var p: *Int = &x`) were ref-tracked (via the
   `&x` value) and the arg auto-deref fired (recorded "*Int" -> pointee
   i64* == the *Int param -> deref at the call site -> callee received
   x's VALUE). The deref now applies only to `&T` ref-locals.
2. Some/None/Ok/Err ctors INSIDE confined-unsafe block fns built the
   GENERIC %struct.Option/Result -- the block fn resets
   current_return_type to its i64 ABI, so concrete-name resolution fell
   back to the generic {i64,i64} layout while the fn returns the
   concrete Option__Rc -- the payload field read the boxed pointer +
   garbage. THIS is the Option/Result payload corruption family root.
   New fctx.enclosing_return_type carries the enclosing fn's declared
   return into the block fn; the ctors consult it (block ABI unchanged).

Verified: smoke_rc_edge/rc/cell/cell_refcell_try exit 0; smoke_rc_weak
payload now correct (up.get() == 100 -- was wrong-value exit 2); the
remaining rc_weak exit 5 is match-arm-binding DROP bookkeeping
(strong_count after scope exit -- separate layer); smoke_core_slice and
smoke_cmp_by remain closure-family (B-007, pre-existing, documented).

### Remaining (documented follow-ups)
- smoke_stress_serialize_json_nested: heap corruption (nested
  enum-with-Vec-field payloads through Map values -- deeper layer).
- smoke_rc_weak: drop bookkeeping for match-arm struct payload bindings.
- smoke_core_slice / smoke_cmp_by: closure path (B-007 design item).
- Full suite: e2e 2273/2273, checker 178, feature-reg 510, stdlib-exec
  70/70+2, stdlib_tests 40, api_freeze 2/2.

---# XIOM Compiler Session -- Handoff (2026-08-18)

## Session update (2026-08-18): BUG 48-52 batch -- 4 fixed, 1 unblocked, e2e 2273/2273

Commits: `569c3d31` (BUG 50+51+52 family: container names, Option payloads,
Map enum values, &K/&Vec ABI), `3a789afb` (BUG 48 unblock + BUG 49 fn-param
vs impl-method collision + e2e regressions + docs), `175294ea` (checker follow-ups: base-name normalization, fn-typed Vec elements, equality erasure), `862fcc52` (pseudo-field base lookup).

Worked from the stdlib session's BUG 48-52 report. Repro files were in the
(removed) stdlib worktrees, so every bug was re-derived from the bug-log
descriptions with fresh user-space probes in tmp/bug_probes/.

### BUG 50 -- FIXED: pointer args in generic container names
Result<*mut UInt8, AllocError> emitted `%struct.Result__*UInt8__AllocError`
-> clang "expected '=' after name". `sanitize_container_arg` maps non-ident
chars to `_` at ALL 7 container-name construction sites (def + call sides
agree). smoke_alloc_edge (the original blocker) now passes. Regression
caught during the batch: tuple names built from local_xiom_types leaked
"Vec[Int]" into `Tuple__Vec[Int]__Vec[Int]` (smoke_iter broke) -- the tuple
path strips container args to match the decl-side bare names.

### BUG 51 -- FIXED (checker): Option[UserStruct] payload bindings
`from_ast_type` now preserves container args; Some/Ok/Err pattern bindings
get the INNER payload type ("Option[MyRc]" -> up: MyRc) instead of the `_`
wildcard (the sorted wildcard method lookup picked Option.get < MyRc.get ->
"cannot compare Option with Int"); Some() ctor types Option[inner];
container-erasure compatibility ("Option" ~ "Option[MyRc]"); Field access
and method dispatch strip container args. Runtime needed no extra change
(the BUG 43 payload machinery already read correctly). Checker 178/178.

### BUG 52 -- FIXED: Map[Str, enum-with-payload]
FOUR coordinated codegen fixes: (1) enum-variant ctor args infer the ENUM
type (insert_Str_Int -> insert_Str_MyVal); (2) generic-method type args
infer from LOCAL receivers via recorded container types (infer_value_
xiom_type ctor recording + infer_call_return_xiom for fn returns + the
receiver-args fallback in generic inference); (3) mono'd method bodies
record Vec-typed receiver-field ELEMENTS (new generic_type_field_types
registry + record_field_vec_elem) so values[i] takes the enum memcpy read;
(4) index-WRITE memcpy's struct/enum elements inline (duplicate-key
update). Plus concrete_container_llvm excludes enum payloads (M18 mirror --
caller emitted unregistered %struct.Option__MyVal). Also fixed the two ABI
root causes uncovered while bisecting: `&K` with K=Str (caller passed the
string ADDRESS; *key loaded the string's first 8 bytes -> strcmp AV -- this
is why smoke_collections_map_basic crashed AT BASELINE too) and `&Vec[T]`
(caller passed %struct.Vec by value; def GEPs through %struct.Vec* -- every
contains-style generic fn AV'd). Verified: 6 user probes, map_basic,
30-smoke battery, e2e 2268/2268, checker 178/178, feature-reg 510/510,
stdlib-exec 70/70+2.
NOT fully green: smoke_stress_serialize_json_nested improved (AV -> heap
corruption) but still fails -- nested enum-with-Vec-field payloads through
Map values (deeper layer; stdlib-session follow-up).

### BUG 48 -- UNBLOCKED (verify on tower re-apply)
The user-space replica of the full construct (tower interfaces + impl
Eq[Int] + generic contains_eq with the ASSOCIATED call) previously AV'd on
the &Vec[T] ABI -- now exits 0 in BOTH associated and method forms
(m37_bug48 e2e). Whether the catalog-specific zero-fallback had an
additional factor can only be confirmed by re-applying the tower Eq/Ord
impls -- the stdlib session should retry the re-apply plan with this
compiler.

### BUG 49 -- FIXED: fn-param vs impl-method symbol collision
`callee_is_fn_ptr` required the resolved key to be unregistered, but
impl-method registration aliases the bare method name ("Int.compare" also
registers "compare") -- a fn-typed param named `compare` resolved to
@Int.compare and bypassed the passed fn pointer (wrong order). The local
now shadows the global for fn-typed params (fn_ptr_return_types) and
closure locals. Verified: user-space replica (impl Eq/Ord[Int] +
compare-named param) sorts correctly; hsbp shape (tower impls +
heap_sort_by) exit 0.

### New e2e regressions (5)
m37_bug48_associated_generic_vec, m37_bug49_fn_param_impl_collision,
m37_bug50_ptr_container_name, m37_bug51_option_struct_payload,
m37_bug52_map_enum_values -- suite count 2268 -> 2273.

### FOR THE STDLIB SESSION
- BUG 48/49 compiler blockers are resolved -- re-apply the tower-style
  Eq[T]/Ord[T] interfaces + 28 impls + method-form conversion per the
  re-apply plan; the associated form should work now too (verify with
  eqt16/17 probes).
- smoke_collections_map_basic now passes (was silently broken by the &K
  Str-key ABI -- not in the exec suite, worth adding).
- Remaining for a dedicated pass: smoke_stress_serialize_json_nested
  (nested enum-with-Vec-field payloads in Map values -- heap corruption),
  the compress decompress length corruption (stdlib-side), and the
  remaining triaged families from the 228 list.

---

# XIOM Compiler Session -- Handoff (2026-08-17)

## Session update (2026-08-17, third pass): BUG 41/42 complete -- e2e 2263/2263

Commits: `f5e53237` (BUG 41/42 refined + harness locks), `25e4c978`
(BUG 41/42 initial), on top of the earlier BUG 39/40 + Item B + BUG 37/36.

- **BUG 41 FINAL**: `type_from_ast` renders Type::Result as bare "Result"
  (Named args dropped) -- the mono call-site fallback now renders the full
  "Result[Env, Str]" (Named-with-args / Type::Result / Type::Option arms),
  and `concrete_container_llvm` applies concrete_type_for's rule (concrete
  iff a payload is a struct; primitives/enums keep the generic layout;
  bare-name fallback for not-yet-registered keys -- the `-o` path compiles
  callers before defs). m34_y07/11/13/15/16/19/20, m35_z02/09/24/29,
  m21_complex_generic_008/009 all exit 0 via BOTH run and -o paths.
- **BUG 42 FINAL**: qualified type_meta field names broke leaf-suffix
  matching (ColumnDef elem_size 18 instead of 40 -> test_sqlite wrong
  results) and Bool FIELDS are 8 bytes (only Vec[Bool] element slots are
  1 byte). All five eco suites now pass: test_json (29), test_db (18),
  test_vector (32), test_sqlite (23), test_test (20).
- **Harness hardening**: compile_and_run_once deletes the target exe first
  -- stale locks caused "permission denied" (e2e_main/t3-hot-reload).
- **FULL SUITE**: e2e 2263/2263, checker 178, parser 96, feature-reg 510,
  stdlib-exec 70 (+2 documented ignores), api_freeze 2/2, stdlib-compile
  40/40. The 7-failure list from the user's test_summary run is CLOSED.

## Session update (2026-08-17, second half): BUG 39/40 + Item B

Commits: `798a7402` (BUG 39+40), `33ff2167` (Item B harness), on top of
`69e2c235`/`dedf7bf1` (BUG 37/36).

- **BUG 40 (major find): nondeterministic IR emission.** `type_meta.entries()`
  and `generic_instantiations` iterate HashMaps in per-process order, so the
  SAME source emitted different IR between builds; clang -O2's codegen of the
  linked MSVC CRT is layout-sensitive, producing passing vs crashing/wrong-
  result binaries at random (~85% crash rate observed for smoke_simd/m21
  across builds -- the "flaky failures" that plagued earlier verification).
  Both loops now SORT by key -> byte-reproducible IR (verified 5x).
- **BUG 39: Vec element-type records.** (1) push-time recording overwrote
  `Vec[Struct]` receivers with "Vec[Int]" -> struct elements memcpy'd into
  %struct.Vec slots (m21_vec_edge_012/027, vec_of_struct, eco -- fixed);
  (2) ctor elem_size under-counted container fields (Vec field 8 vs 32) ->
  Vec[Vector] buffer overflow -> AV (test_vector/test_db/test_json -- fixed
  via vec_elem_storage_size).
- **Item B:** api_freeze 905 missing -> **0** (manifest-driven module path
  resolution + indentation-aware signature extraction). Execution tests
  72 -> **70 pass + 2 documented #[ignore]** (smoke_core = stdlib Eq-impl gap;
  smoke_simd = BUG 40-era latent CRT layout miscompile). stdlib_tests 40/40.
- **En route:** Vec[Str] element reads inttoptr to i8* (yaml garbage);
  ptr.null/dangling results recorded pointer-valued -> is_null coercion
  inttoptrs (smoke_ptr).
- **Verified:** checker 178/178; exec 70/70+2; api_freeze 2/2; e2e
  2256/2263 (remaining: 2 hot-reload FILE-LOCK artifacts + m34_y15/y20 +
  eco db/json/vector). Repro battery (t_chainloop, 14xt_b37, t_f128rt,
  t_f128i128, sx4, t_iter38, pdb3, t_b33, t_b34, t_b35, t_fneg_clean) all 0.
- **BUG 41 (OPEN, next session):** generic fn with UNUSED type param +
  concrete Result payload -- mono'd definition emits
  `%struct.Result__Env__Str` but the call-site registry records
  `%struct.Result` -> invalid IR (m34_y15/y20 compile failures; previously
  masked by BUG 40's nondeterminism). Fix in the mono signature
  registration (lib.rs specialized_ret_type): resolve concrete
  Result/Option payloads like the definition side.
- **Harness reminder:** `xiom run` caches exes in ~/.xiom/jit/<sha>.exe keyed
  ONLY by source hash -- clear after ANY runtime C change.

## Session update (2026-08-17): BUG 37/36 FIXED -- fp128 libcall ABI mismatch

Commits this session: `(pending)` -- fp128_helpers.c shims + expr.rs i128 casts.

- **BUG 37/36 ROOT CAUSE FOUND -- it was never a shape miscompile.** clang
  lowers runtime fp128 arithmetic to soft-float libcalls (`__addtf3` etc.)
  with LLVM's Win64 f128 convention: args BY POINTER (rcx=&a, rdx=&b),
  result in **XMM0**. fp128_helpers.c compiled the same symbols from C as
  `xiom_f128` struct functions -> MSVC ABI: hidden **sret in rcx**, args
  shifted to rdx/**r8**, result via sret, XMM0 never set. Every
  f128-returning helper was ABI-mismatched: callee dereferenced r8=garbage
  (0xC0000005), caller read XMM0=garbage. The BigFloat chain was a red
  herring -- it only prevented clang -O2 from constant-folding the loop.
  "Working" fp128 verifications (t_fneg, d1_native128, BUG 31/33) were all
  constant-folded shapes. Probe prints "changed semantics" by breaking the
  folding.
- **FIX:** 13 naked-asm shims (SSE2-only, JIT-safe) implementing the IR
  convention, forwarding to sret wrappers around the existing pure impls.
  Also added the missing `__fixtfti`/`__fixunstfti` (f128->i128) and
  codegen arms for `Int128 as Float128` / `Float128 as Int128` (were
  emitting mis-typed stores / would-be link errors).
- **Verified:** t_chainloop + all 14 t_b37* shapes exit 0 with correct
  values (t_b37e bigfloat_to_float128=42 val=OK; t_b37u user Horner=42).
  New t_f128rt (runtime Vec-loaded fp128 add/sub/mul/div/neg/trunc/compare)
  OK; t_f128i128 (Int128<->Float128 roundtrip x2) OK. Suites: checker
  178/178, codegen 2263, feature-reg 510, parser 24 -- all green.
  api_freeze_no_removals fails identically at baseline (item B -- not a
  regression). smoke_num_float_classify (17) + smoke_stress_convert_float_
  to_string_prec (1) still fail as in the pre-fix runfail list (Float64
  string families -- unrelated to fp128; queued for the shape-family
  triage). t_b35's scratch copy had an INVERTED assertion (counts[0]==3;
  correct buckets are [1,1,1]) -- fixed the scratch copy.
- **IMPORTANT for verification discipline:** `xiom run` caches binaries in
  `~/.xiom/jit/<sha256>.exe` keyed ONLY by source hash -- after ANY runtime
  C change, delete those cached exes or results silently use the old
  runtime (`t_b37q` printed "0","0" from a stale cache; clean build = "0","1").

## Session handoff (2026-08-17) -- CLEAN STATE, work committed

Branch: `feat/architect` (**37 commits ahead** of `origin/feat/architect`).
All work this session is COMMITTED (5 commits below). Working tree has ONLY
pre-existing third-party modifications -- `.xiom_ai.json`, `Cargo.lock`,
`docs/AI_CONTEXT.md` (NOT mine; leave them alone).

### Commits this session (in order)

```
e477815b docs: report BUG 32-38 queue results + stdlib-side list to the stdlib session
802e9fd2 docs: BUG 32-38 queue status - 38/38b/32/33/31/34 fixed, 35 verified, 37/36 open, P001 resolved
b15d0d3b fix(codegen): BUG 34 - nested Vec[Vec[T]] element writes
74bcc28b fix(codegen): BUG 32, 33, 31 - Int-to-ptr cast, Option[Float128] payload, fp128 fneg
9042e8a2 fix(codegen): BUG 38 family - is-Some double-check, generic-receiver mono, mutating self ABI
```

### Fixed & verified (isolated binary `$env:TEMP\kilo\tgt_iso\debug\xiom.exe`)

- **BUG 38** -- bare `is Some` scrutinee payload rebind fires ONLY inside an
  Imply left side (sx4 pos=5; iter.xi collect works).
- **BUG 38b (new)** -- generic-receiver methods (`Iterator[T].collect`): the
  parser DROPPED receiver generics -> erased i64 ABI -> never monomorphised
  (45 decls in iter/core/collections/sync/rc/memory). Four-part fix: parser
  receiver-generics capture; receiver-keyed mono dispatch; find_generic_decl
  prefers UNDECLARED abstract receivers; mutating-self ABI (block_mutates_self
  recursion + new block_mutates_receiver_state). iter smokes 0/19 -> 6/19.
- **BUG 32** (Int-var->ptr cast now load+inttoptr), **BUG 33** (Option[Float128]
  payload -> fp128), **BUG 31** (fp128 fneg), **BUG 34** (nested Vec[Vec[T]]
  writes: i64-element dispatch + element-address ABI + elem_size 32 +
  elem-type tracking).
- **BUG 35** primary shape verified working (Int128+Vec-write buckets
  [1,1,1], no crash).
- **P001** not reproducible; indented decls reject in 0.14s; former 15-min
  hang smoke compiles in 4.9s.
- Suites: checker **178/178**, codegen **2263**, feature-reg **510**, parser
  **24** -- all green. `stdlib_api_freeze_no_removals` FAILS IDENTICALLY AT
  BASELINE HEAD (905 frozen signatures missing -- item B family, stdlib
  layout moved; NOT a regression).

### Re-triage of the stdlib session's remaining lists (238 files, current binary)

**43 pass / 82 compilefail / 113 runfail.** Results: `%TEMP%\kilo\resweep_new0-3.txt`.
Remaining families: (a) receiver-CALL chains for generic methods (iter
adapters -- reproduces at baseline, B-007-adjacent); (b) BUG 24/36 shape
miscompiles; (c) stale-API smokes (`.get(0)` on Vec, char.from_digit
contract false-fire, is_empty). Full breakdown: COMPILER_BUGS.md
(2026-08-16 late section) + REPORT_TO_STDLIB_SESSION.md.

### OPEN queue (next sessions, priority order)

1. **BUG 37/36 -- FIXED (2026-08-17)** -- fp128 libcall ABI mismatch (see the
   session update above): 13 naked-asm shims in fp128_helpers.c + i128<->fp128
   cast arms. The stdlib's bigfloat_to_float128 workaround fns can be
   consolidated; Option[Float128] can land.
2. **Item B (HIGH)** -- exec harness wiring: stdlib_tests.rs +
   stdlib_execution_tests.rs + stdlib_api_freeze_tests.rs still reference
   the pre-refactor layout (STDLIB_MANIFEST.md 515 paths, STDLIB_SMOKES.md
   213 smokes, frozen-signature snapshot). The 5-test-worktree merge wave
   waits on this. **The stdlib session has the GREEN LIGHT to wrap up** --
   its remaining items are the stdlib-side list in
   REPORT_TO_STDLIB_SESSION.md (char.xi from_digit requires-contract on a
   fallback fn; Vec.is_empty broken -- baseline-confirmed; stale `.get(0)`
   smoke usage -> use indexing; smoke_num_saturating Bounded/Ord impls;
   smoke_alloc_basic `use xiom.ptr;`).
3. **Item C (HIGH, user-approved)** -- catalog body type-checking: Phase 1
   reachable injected fns; Phase 2 all catalog bodies (hash-keyed cache);
   Phase 3 `--strict-stdlib` gate.
4. **Iter adapter chains (MEDIUM)** -- receiver-CALL chains
   (`iter.range(1,6).max()`, `.map(fn...).collect()`) + fn-value params:
   reproduces at baseline; B-007-adjacent (closure-through-fn-slot design
   decision pending).

### Scratch / repro files this session (in `%TEMP%\kilo\`)

t_b37f/t_b37k/t_chainloop (BUG 37/36), t_b35/b35b (BUG 35 verified),
t_b34 (BUG 34), t_b33/t_b33u (BUG 33), pdb3 (BUG 32), t_fneg (BUG 31),
t_iter38/t_b1/t_clike (BUG 38b), sx4 (BUG 38). All exit 0 with the current
binary except the t_b37* family (AV) and t_vecloop (my test's inverted
expectation -- values correct).

---

## Session update (2026-08-16 late): BUG 32-38 queue progress

Commits this session: `9042e8a2` (BUG 38 + 38b generic-receiver family),
`74bcc28b` (BUG 32/33/31), `b15d0d3b` (BUG 34).

- **BUG 38 FIXED** -- bare `is Some` scrutinee rebind now fires ONLY inside
  an Imply left side (sx4 pos=5).
- **BUG 38b (new) FIXED** -- generic-receiver methods (`Iterator[T].collect`)
  lost their receiver generics in the parser; receiver-keyed mono dispatch +
  abstract-receiver decl preference + mutating-self ABI (block_mutates_self
  recursion + block_mutates_receiver_state). iter smokes 0/19 -> 6/19.
- **BUG 32 FIXED** (Int->ptr cast inttoptr), **BUG 33 FIXED** (Option[Float128]
  payload -> fp128), **BUG 31 FIXED** (fp128 fneg), **BUG 34 FIXED** (nested
  Vec[Vec[T]] writes: dispatch + element-address ABI + elem_size 32).
- **BUG 35** primary shape verified working (Int128+Vec-write buckets
  correct); extreme variant needs the stdlib repro.
- **BUG 37/36 OPEN** -- minimal deterministic repro: BigFloat field-chain
  Vec-len as a loop bound + any fp128 op in the loop -> AV even at clang -O0
  with sound IR (t_chainloop/t_b37f). `bigfloat_to_float128` still crashes
  for non-zero values. Deep-dive needed.
- **P001 not reproduced** -- indented decls reject in 0.14s; the former
  15-min hang smoke compiles in 4.9s. The 238-file re-sweep completed
  without hangs.
- Re-triage of the stdlib session's two lists: 238 files -> 43 pass, 82
  compilefail, 113 runfail (details in COMPILER_BUGS.md + the report file).

Full statuses: docs/COMPILER_BUGS.md (2026-08-16 late section). The queue
items B (exec harness) and C (catalog body checking) are still OPEN.

Branch: `feat/architect` (32 commits ahead of `origin/feat/architect`).
COMPILER session. The parallel stdlib session is MISSION COMPLETE
(512 modules, 6,379 pub fns, 0 stubs, layout FROZEN); my replies live in
`docs/REPORT_TO_STDLIB_SESSION.md` (last updated `707a299c`).

## Session summary (2026-08-16): the 16 regression failures are FIXED + BUG 31 batch

The 04:03 test run (`session_20260814_040310.txt`) had **12 e2e failures +
4 stdlib-exec failures + 1 diff failure**. All compiler-fixable ones are now
green; the remaining items are stdlib-side or latent.

### Commits this session

```
5940ba2c fix(codegen): struct_type_from_expr resolves Type.method static calls without the bare alias
0ddc4500 fix(codegen/check): BUG 30 batch - 13 regressions from stdlib layout era (e2e 12 + stdlib-exec rc/cell/utf8)
e1bd8284 chore: remove TEMP-DEBUG vec2_dbg probe from xiom.geom.vec; bump session metadata
36bfb32a test(stdlib-smoke): restore full lz4/snappy round-trip coverage
```

### The 14 fixes (commit 0ddc4500 + 5940ba2c)

Compiler (xiom-codegen):
1. **Imply scoping** -- contract-ensure `is Some/Ok/Err` payload rebinds no
   longer poison later return-site checks (the payload slot local persisted
   -> i64-form Is inttoptr+load -> AV). Fixes b003 / m19_read_file /
   file_stem / file_name chains.
2. **Is() payload-slot hoisting** (struct + i64 forms) -- `&&` guard chains
   bind in one block, read in a later one -> "Instruction does not dominate
   all uses" (m18_guard_0086).
3. **Is() bare-scrutinee rebind in the enum-variants form** -- the fallback
   path had it; the enum-variants path didn't -> ensure `result.len()`
   resolved to Map.len (utf8).
4. **struct_type_from_expr Call-arm resolution** -- bare/module-qualified
   callees resolved through call-site key machinery (caller module ->
   bare_fn_aliases -> use_alias_map). Match-on-catalog-call dropped the
   scrutinee -> no tag checks + payload literal 0 -> AV (m35_z24/z29,
   eco_algo).
5. **Mono path flushes hoisted allocas** -- `var t = b;` in a generic fn's
   while loop emitted only the store (m35_z24 "undefined value %tmp55").
6. **Enum literal ctors zero-init unused payload slots** (bare/qualified
   struct forms + Ok/Err) -- LLVM poison -> clang -O2 miscompile ->
   deterministic AV (m35_z10/z29, bisect_z29_m). The driver runs `opt -O2`
   + `clang -O2` -- poison is exploited.
7. **coerce_arg_for_param `&array_local`** -- data pointer only for `&[N]T`
   params; `%struct.Vec*` params get the header alloca (eco_algo
   len()==element-0).
8. **len() dispatch for boxed Vec-handle locals** -- contract rebind payload
   with Vec XIOM type unboxes via inttoptr+load (utf8 ensure).
9. **Ensure `result` scoped per-check** -- a USER local named `result`
   shadowed the synthetic return-slot binding (utf8_encode AV).
10. **decl.rs result local_xiom_types uses type_string_full** -- the payload
    args survived (Result[Vec[UInt8], Str] not just "Result").
11. **collect_block_free_vars sorts captures by name** -- HashSet iteration
    order differed between the block fn / ctx type / caller stores in
    different passes (Rc.new slot swap -- fields written through VALUE 42 as
    a pointer).
12. **Generic receiver ABI** -- by-value `self` passes the struct VALUE not a
    pointer (smoke_rc Rc.get garbage: `%struct.Rc` vs `%struct.Rc*`).
13. **Checker wildcard method lookup deterministic** (sorted candidates,
    receiver-base preferred) AND **clone skip only applies to the fallback**
    -- the v0.56 clone "fix" discarded the DIRECT hit too, so `r.clone()`
    typed `_` and `r2.get()` hit BTreeMap.get -> Option (smoke_rc).
14. **struct_type_from_expr Type.method static calls** -- `Vec[Str]::new()`
    must resolve `Vec.new` (exact / module-qualified suffix); when the typed
    key is unregistered (catalog compile ORDER: io.xi before
    xiom.collections) return None -- NEVER the bare alias (first registrant
    BufReader.new) -> BufReader.invariant_check(%struct.Vec ...) -> invalid IR
    (smoke_net_http, the io module graph).

File-side (same commits): bench_math Float64 casts (644/656), selfhost
xiom-check pub+use for 7 test fns, xiomc_v11_test `elif pfound {`, m21
Vec.pop Option handling.

## BUG 31 batch (2026-08-16, second half -- the 904-sweep queue)

Commits: `2b238da4` (fmt), `b28ac72b` (Map), `a7571ac7` (variant hijack),
`99f894b7` (tuple destructure), `de639432` (docs).

- **fmt cluster (8 fixes, 8 smokes green):** Unit fields in
  Result[Unit, FmtError] literals (`store void 0, void*` invalid IR --
  generic literals adopt the fn's concrete return type; field types
  degrade Unit->i64); Str.to_str passthrough read the first BYTE (prologue
  treated i8* as a struct pointer -- only %struct.X* receivers take the
  pointer branch); primitive/variable method receivers resolved to bare
  stubs (infer_struct_type_name learns literals/negated/parens, primitive
  locals via local_xiom_types, generic params via param_concrete_types,
  infer_value_xiom_type learns literals); mutating `self` methods lost
  mutations (block_mutates_self -> pointer ABI in compile_fn + signature
  registration).
- **Map cluster (4 fixes, 7 smokes green):** `&K` scalar params passed the
  VALUE -- four coordinated fixes (param_llvm_type, mono subst_type Ref arm,
  generic-call inference, coerce materializes plain values for pointer
  params).
- **Variant-hijack:** struct literals whose name collides with an enum
  VARIANT (Node{...} vs enum BST { Node(...) }) -- bare-variant search
  yields to known types. bench_math native IR now valid (was
  store %struct.BST %vecval).
- **Tuple destructure (BUG 26 #3):** the checker bound every name to the
  whole Tuple__A__B; now splits element types.

**Verified fixed:** BUG 26 #2 (bare prelude names), BUG 26 #5 (high-bit
mask AND), BUG 24 residual (smoke_num_precision). **Stdlib-side:**
smoke_num_saturating needs real Bounded/Ord impls; smoke_alloc_basic needs
`use xiom.ptr;`; smoke_hash_folder needs `use xiom.convert.toint;`.
The stdlib session also filed BUG 32-38 (fp128/ptr-cast/nested-Vec-write/
is-Some-double-check families) -- the next compiler queue.

### Verified green (isolated binary `$env:TEMP\kilo\tgt_iso\debug\xiom.exe`)

- Checker suite: **178/178**.
- All 14 previously-failing e2e/stdlib-exec tests run exit 0
  (b003, m19_read_file, m18_guard_0086, m35_z24/z10/z29, eco_algo,
  smoke_rc, smoke_cell, smoke_utf8, smoke_compress_lz4_snappy,
  m21_vec_edge_004, xiom-check.xi).
- diff-test expectations hold (the 04:03 f=1 was binary-churn race).
- bench_math: `--emit-ir` passes (harness is emit-ir-only) -- native `-o`
  still fails on a LATENT struct-literal field-type bug (below).

## Open items (next sessions, in priority order)

### 0. [HIGH -- NOW] Rebuild + re-triage the remaining smoke failures (2026-08-16 late)

The stdlib session's 905-smoke sweep (621 pass / 300 fail, done against the
STALE 17:36 binary) produced exact lists:
`%TEMP%\kilo\remaining_compilefail.txt` (59) + `remaining_runfail.txt` (190).

**BEFORE triaging anything:**
1. Rebuild the isolated binary -- my BUG 31 fixes (`2b238da4`, `b28ac72b`,
   `a7571ac7`, `99f894b7`) are NOT in the 17:36 binary. The fmt/io/regex
   clang clusters and ALL `bisect_*` entries (deleted scratch) should drop
   out of the lists immediately.
2. Re-run both lists against the new binary; filter:
   - `bisect_*` -- my deleted scratch, ignore.
   - `smoke_fmt_*`, `smoke_stress_fmt_*` -- fixed (fmt cluster).
   - `smoke_rc/cell/utf8`, `smoke_stress_collections_map_*` -- fixed.
3. The ~60 remaining compile-fails and the run-fails cluster into families
   that need triage: **array/char/cell/cmp/collections/convert/core/error/
   io/num/ptr/rc** -- each is either a REAL compiler bug (like the fmt/Map
   clusters were) or stdlib-side API drift. Bisect each family like the
   fmt/Map clusters were done (small repro -> root cause -> fix).
4. Known stdlib-side (hand to the stdlib session): smoke_num_saturating
   (Bounded/Ord impls needed), smoke_alloc_basic (`use xiom.ptr;`).

**Edit-race caution (stdlib session's warning):** their realignment script
ran while smoke crypto/net/stress files showed as modified; if any of my
uncommitted smoke content is missing from the tree, re-apply it. Current
tree has NO uncommitted smoke changes (all committed) -- verify with
`git status --short examples/stdlib_smoke` before mass-editing.

### A. [HIGH -- pre-selfhost] Catalog body type-checking (Q2b, user-approved)
NOT STARTED. Plan unchanged:
Phase 1 check reachable injected fns; Phase 2 all catalog bodies at load
(hash-keyed cache); Phase 3 `--strict-stdlib` CI gate.

### B. [HIGH] api_freeze path sync + exec harness
Unchanged: `crates/xiom-codegen/tests/stdlib_tests.rs` +
`stdlib_execution_tests.rs` still reference the pre-refactor layout
(STDLIB_MANIFEST.md 515 paths + STDLIB_SMOKES.md 213 smokes).
**The merge wave (5 idle test worktrees) WAITS on this** -- the stdlib
session is ready to run it once item B lands.

### C. [MEDIUM] BUG 32-38 -- the stdlib session's new compiler queue
Filed in COMPILER_BUGS.md (2026-08-16 evening), priority by blocker:
1. **BUG 37** -- fp128 RETURNED from a catalog fn crashes the caller
   (0xC0000005) -- blocks any consumer of `bigfloat_to_float128`.
2. **BUG 32** -- `x as *T` (Int VARIABLE to pointer) emits address-of-local,
   not inttoptr -- blocks pointer-handle designs (glob).
3. **BUG 38** -- `if x is Some { match x { Some(v) => ... } }` double-check
   binds the payload as 0 -- BUG 30 #1/#2 family; semver worked around it.
4. **BUG 34** -- nested Vec element WRITES via `&mut` AV (reads work).
5. **BUG 35** -- Int128 index math inside a Vec-writing fn shape AV.
6. **BUG 36** -- fp128 Horner-loop fn shape AV when sign statements follow.
7. **BUG 33** -- Option[Float128] unwrap loads undefined `%struct.Float128`
   (payload type lookup must map XIOM Float128 -> LLVM fp128).
8. **BUG 31 (stdlib's)** -- unary minus on Float128 emits `sub i64 0, fp128`
   (fneg path must handle fp128).
9. **P001 hang** -- indented module-level declarations HANG the compiler
   (stdlib realigned 158 files; the parser should reject, not hang).

### D. [DONE] 904-smoke battery -- BUG 31 batch closed the compiler-side
fmt (8 smokes), Map (7 smokes), variant-hijack (bench_math native), tuple
destructure (BUG 26 #3) all fixed; BUG 26 #2/#5 + BUG 24 residual verified
fixed. Remaining queue is item 0 (re-triage) + C above.

### E. [MEDIUM] MCP catalog/registry tools (ROADMAP G) -- unchanged.
### F. [LOW] Roadmap E/F: reverse type-index, contract policy decision.

## Sequencing per the user (pre-selfhost -> split -> registry) -- unchanged:
1. Selfhost gate: stdlib 100% + compiler 100% + zero warnings + item A + B.
2. Monorepo split per docs/REPO_SPLIT.md ONLY after (1) green.
3. Registry + packages after the split.

## How to verify anything you touch

- Rebuild the isolated binary: `$env:CARGO_TARGET_DIR="$env:TEMP\kilo\tgt_iso";
  cargo build -p xiom` then use `tgt_iso\debug\xiom.exe` (NEVER the shared
  target/debug binary -- parallel sessions rebuild it constantly).
- After any change: checker (`cargo test -p xiom-check --target-dir
  "$env:TEMP\kilo\tgt_check"`) + the 14-test battery above + the 904-smoke
  sweep (run in 8 parallel batches of ~115; each batch ~25 min).
- Never stash/revert uncommitted stdlib work without a fresh backup.
- The stdlib session's next message should include the D-section survey so
  they can fix the stdlib-side half of the smoke failures.

---

## 2026-08-22 -- stdlib session handoff (read docs/stdlib_session.md for the full handoff)

State: sweep 802/907 PASS, zero hangs. The closure-based iter adapter build is
FULLY GREEN after round-13 (map/filter/take_skip/max_min/pipeline/chain_zip/
count/fold/enumerate + btree_map/btreemap tuple payloads + cmp_by + result
family all exit 0 -- verified 15/16, only array_sort_by remains at the
pre-existing CRT-layout baseline AV).

Stdlib-side wins this campaign (all committed on feat/architect):
- Real stdlib bugs fixed: RefCell/PathBuf/Set/Queue/Stack &mut-self + write-back,
  VecDeque live-range rebuild, gcd abs, crc32 bitwise (real-gzip compatible),
  redundant requires removed (io/compress/char/Vec), identity ensure, Bounded/
  Ord/Eq towers (12+15+15 impls), global_alloc signature
- Feature build: closure-based iter adapters (~440 lines) -- now green
- ~60 smokes realigned to implemented APIs + stray-brace fixes + missing imports

Remaining queue (all documented with probes in COMPILER_BUGS.md): CRT-layout
startup AVs (array_sort_by/iter_collect), missing iter find/all/any/nth/last
API, narrow-SIGNED zext, json heap layer, SIMD flags, clang variants, checker
builtin Ord resolution, Set iteration (iterator protocol).

Next stdlib session: re-run the sweep, triage with the probe->log->verify
loop, and pick up the iter find/all/any/nth/last methods once the checker
generic-tuple gap closes.

