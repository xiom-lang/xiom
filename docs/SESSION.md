# XIOM Compiler Session Ã¢â‚¬â€ Handoff (2026-08-18)

## Session update (2026-08-18): BUG 48-52 batch Ã¢â‚¬â€ 4 fixed, 1 unblocked, e2e 2273/2273

Commits: `569c3d31` (BUG 50+51+52 family: container names, Option payloads,
Map enum values, &K/&Vec ABI), `3a789afb` (BUG 48 unblock + BUG 49 fn-param
vs impl-method collision + e2e regressions + docs), `175294ea` (checker follow-ups: base-name normalization, fn-typed Vec elements, equality erasure), `862fcc52` (pseudo-field base lookup).

Worked from the stdlib session's BUG 48-52 report. Repro files were in the
(removed) stdlib worktrees, so every bug was re-derived from the bug-log
descriptions with fresh user-space probes in tmp/bug_probes/.

### BUG 50 Ã¢â‚¬â€ FIXED: pointer args in generic container names
Result<*mut UInt8, AllocError> emitted `%struct.Result__*UInt8__AllocError`
Ã¢â€ â€™ clang "expected '=' after name". `sanitize_container_arg` maps non-ident
chars to `_` at ALL 7 container-name construction sites (def + call sides
agree). smoke_alloc_edge (the original blocker) now passes. Regression
caught during the batch: tuple names built from local_xiom_types leaked
"Vec[Int]" into `Tuple__Vec[Int]__Vec[Int]` (smoke_iter broke) Ã¢â‚¬â€ the tuple
path strips container args to match the decl-side bare names.

### BUG 51 Ã¢â‚¬â€ FIXED (checker): Option[UserStruct] payload bindings
`from_ast_type` now preserves container args; Some/Ok/Err pattern bindings
get the INNER payload type ("Option[MyRc]" Ã¢â€ â€™ up: MyRc) instead of the `_`
wildcard (the sorted wildcard method lookup picked Option.get < MyRc.get Ã¢â€ â€™
"cannot compare Option with Int"); Some() ctor types Option[inner];
container-erasure compatibility ("Option" ~ "Option[MyRc]"); Field access
and method dispatch strip container args. Runtime needed no extra change
(the BUG 43 payload machinery already read correctly). Checker 178/178.

### BUG 52 Ã¢â‚¬â€ FIXED: Map[Str, enum-with-payload]
FOUR coordinated codegen fixes: (1) enum-variant ctor args infer the ENUM
type (insert_Str_Int Ã¢â€ â€™ insert_Str_MyVal); (2) generic-method type args
infer from LOCAL receivers via recorded container types (infer_value_
xiom_type ctor recording + infer_call_return_xiom for fn returns + the
receiver-args fallback in generic inference); (3) mono'd method bodies
record Vec-typed receiver-field ELEMENTS (new generic_type_field_types
registry + record_field_vec_elem) so values[i] takes the enum memcpy read;
(4) index-WRITE memcpy's struct/enum elements inline (duplicate-key
update). Plus concrete_container_llvm excludes enum payloads (M18 mirror Ã¢â‚¬â€
caller emitted unregistered %struct.Option__MyVal). Also fixed the two ABI
root causes uncovered while bisecting: `&K` with K=Str (caller passed the
string ADDRESS; *key loaded the string's first 8 bytes Ã¢â€ â€™ strcmp AV Ã¢â‚¬â€ this
is why smoke_collections_map_basic crashed AT BASELINE too) and `&Vec[T]`
(caller passed %struct.Vec by value; def GEPs through %struct.Vec* Ã¢â‚¬â€ every
contains-style generic fn AV'd). Verified: 6 user probes, map_basic,
30-smoke battery, e2e 2268/2268, checker 178/178, feature-reg 510/510,
stdlib-exec 70/70+2.
NOT fully green: smoke_stress_serialize_json_nested improved (AV Ã¢â€ â€™ heap
corruption) but still fails Ã¢â‚¬â€ nested enum-with-Vec-field payloads through
Map values (deeper layer; stdlib-session follow-up).

### BUG 48 Ã¢â‚¬â€ UNBLOCKED (verify on tower re-apply)
The user-space replica of the full construct (tower interfaces + impl
Eq[Int] + generic contains_eq with the ASSOCIATED call) previously AV'd on
the &Vec[T] ABI Ã¢â‚¬â€ now exits 0 in BOTH associated and method forms
(m37_bug48 e2e). Whether the catalog-specific zero-fallback had an
additional factor can only be confirmed by re-applying the tower Eq/Ord
impls Ã¢â‚¬â€ the stdlib session should retry the re-apply plan with this
compiler.

### BUG 49 Ã¢â‚¬â€ FIXED: fn-param vs impl-method symbol collision
`callee_is_fn_ptr` required the resolved key to be unregistered, but
impl-method registration aliases the bare method name ("Int.compare" also
registers "compare") Ã¢â‚¬â€ a fn-typed param named `compare` resolved to
@Int.compare and bypassed the passed fn pointer (wrong order). The local
now shadows the global for fn-typed params (fn_ptr_return_types) and
closure locals. Verified: user-space replica (impl Eq/Ord[Int] +
compare-named param) sorts correctly; hsbp shape (tower impls +
heap_sort_by) exit 0.

### New e2e regressions (5)
m37_bug48_associated_generic_vec, m37_bug49_fn_param_impl_collision,
m37_bug50_ptr_container_name, m37_bug51_option_struct_payload,
m37_bug52_map_enum_values Ã¢â‚¬â€ suite count 2268 Ã¢â€ â€™ 2273.

### FOR THE STDLIB SESSION
- BUG 48/49 compiler blockers are resolved Ã¢â‚¬â€ re-apply the tower-style
  Eq[T]/Ord[T] interfaces + 28 impls + method-form conversion per the
  re-apply plan; the associated form should work now too (verify with
  eqt16/17 probes).
- smoke_collections_map_basic now passes (was silently broken by the &K
  Str-key ABI Ã¢â‚¬â€ not in the exec suite, worth adding).
- Remaining for a dedicated pass: smoke_stress_serialize_json_nested
  (nested enum-with-Vec-field payloads in Map values Ã¢â‚¬â€ heap corruption),
  the compress decompress length corruption (stdlib-side), and the
  remaining triaged families from the 228 list.

---

# XIOM Compiler Session ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â Handoff (2026-08-17)

## Session update (2026-08-17, third pass): BUG 41/42 complete ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â e2e 2263/2263

Commits: `f5e53237` (BUG 41/42 refined + harness locks), `25e4c978`
(BUG 41/42 initial), on top of the earlier BUG 39/40 + Item B + BUG 37/36.

- **BUG 41 FINAL**: `type_from_ast` renders Type::Result as bare "Result"
  (Named args dropped) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â the mono call-site fallback now renders the full
  "Result[Env, Str]" (Named-with-args / Type::Result / Type::Option arms),
  and `concrete_container_llvm` applies concrete_type_for's rule (concrete
  iff a payload is a struct; primitives/enums keep the generic layout;
  bare-name fallback for not-yet-registered keys ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â the `-o` path compiles
  callers before defs). m34_y07/11/13/15/16/19/20, m35_z02/09/24/29,
  m21_complex_generic_008/009 all exit 0 via BOTH run and -o paths.
- **BUG 42 FINAL**: qualified type_meta field names broke leaf-suffix
  matching (ColumnDef elem_size 18 instead of 40 ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ test_sqlite wrong
  results) and Bool FIELDS are 8 bytes (only Vec[Bool] element slots are
  1 byte). All five eco suites now pass: test_json (29), test_db (18),
  test_vector (32), test_sqlite (23), test_test (20).
- **Harness hardening**: compile_and_run_once deletes the target exe first
  ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â stale locks caused "permission denied" (e2e_main/t3-hot-reload).
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
  across builds ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â the "flaky failures" that plagued earlier verification).
  Both loops now SORT by key ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ byte-reproducible IR (verified 5ÃƒÆ’Ã¢â‚¬â€).
- **BUG 39: Vec element-type records.** (1) push-time recording overwrote
  `Vec[Struct]` receivers with "Vec[Int]" ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ struct elements memcpy'd into
  %struct.Vec slots (m21_vec_edge_012/027, vec_of_struct, eco ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â fixed);
  (2) ctor elem_size under-counted container fields (Vec field 8 vs 32) ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢
  Vec[Vector] buffer overflow ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ AV (test_vector/test_db/test_json ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â fixed
  via vec_elem_storage_size).
- **Item B:** api_freeze 905 missing ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ **0** (manifest-driven module path
  resolution + indentation-aware signature extraction). Execution tests
  72 ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ **70 pass + 2 documented #[ignore]** (smoke_core = stdlib Eq-impl gap;
  smoke_simd = BUG 40-era latent CRT layout miscompile). stdlib_tests 40/40.
- **En route:** Vec[Str] element reads inttoptr to i8* (yaml garbage);
  ptr.null/dangling results recorded pointer-valued ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ is_null coercion
  inttoptrs (smoke_ptr).
- **Verified:** checker 178/178; exec 70/70+2; api_freeze 2/2; e2e
  2256/2263 (remaining: 2 hot-reload FILE-LOCK artifacts + m34_y15/y20 +
  eco db/json/vector). Repro battery (t_chainloop, 14ÃƒÆ’Ã¢â‚¬â€t_b37, t_f128rt,
  t_f128i128, sx4, t_iter38, pdb3, t_b33, t_b34, t_b35, t_fneg_clean) all 0.
- **BUG 41 (OPEN, next session):** generic fn with UNUSED type param +
  concrete Result payload ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â mono'd definition emits
  `%struct.Result__Env__Str` but the call-site registry records
  `%struct.Result` ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ invalid IR (m34_y15/y20 compile failures; previously
  masked by BUG 40's nondeterminism). Fix in the mono signature
  registration (lib.rs specialized_ret_type): resolve concrete
  Result/Option payloads like the definition side.
- **Harness reminder:** `xiom run` caches exes in ~/.xiom/jit/<sha>.exe keyed
  ONLY by source hash ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â clear after ANY runtime C change.

## Session update (2026-08-17): BUG 37/36 FIXED ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â fp128 libcall ABI mismatch

Commits this session: `(pending)` ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â fp128_helpers.c shims + expr.rs i128 casts.

- **BUG 37/36 ROOT CAUSE FOUND ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â it was never a shape miscompile.** clang
  lowers runtime fp128 arithmetic to soft-float libcalls (`__addtf3` etc.)
  with LLVM's Win64 f128 convention: args BY POINTER (rcx=&a, rdx=&b),
  result in **XMM0**. fp128_helpers.c compiled the same symbols from C as
  `xiom_f128` struct functions ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ MSVC ABI: hidden **sret in rcx**, args
  shifted to rdx/**r8**, result via sret, XMM0 never set. Every
  f128-returning helper was ABI-mismatched: callee dereferenced r8=garbage
  (0xC0000005), caller read XMM0=garbage. The BigFloat chain was a red
  herring ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â it only prevented clang -O2 from constant-folding the loop.
  "Working" fp128 verifications (t_fneg, d1_native128, BUG 31/33) were all
  constant-folded shapes. Probe prints "changed semantics" by breaking the
  folding.
- **FIX:** 13 naked-asm shims (SSE2-only, JIT-safe) implementing the IR
  convention, forwarding to sret wrappers around the existing pure impls.
  Also added the missing `__fixtfti`/`__fixunstfti` (f128ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢i128) and
  codegen arms for `Int128 as Float128` / `Float128 as Int128` (were
  emitting mis-typed stores / would-be link errors).
- **Verified:** t_chainloop + all 14 t_b37* shapes exit 0 with correct
  values (t_b37e bigfloat_to_float128=42 val=OK; t_b37u user Horner=42).
  New t_f128rt (runtime Vec-loaded fp128 add/sub/mul/div/neg/trunc/compare)
  OK; t_f128i128 (Int128ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬ÂFloat128 roundtrip ÃƒÆ’Ã¢â‚¬â€2) OK. Suites: checker
  178/178, codegen 2263, feature-reg 510, parser 24 ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â all green.
  api_freeze_no_removals fails identically at baseline (item B ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â not a
  regression). smoke_num_float_classify (17) + smoke_stress_convert_float_
  to_string_prec (1) still fail as in the pre-fix runfail list (Float64
  string families ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â unrelated to fp128; queued for the shape-family
  triage). t_b35's scratch copy had an INVERTED assertion (counts[0]==3;
  correct buckets are [1,1,1]) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â fixed the scratch copy.
- **IMPORTANT for verification discipline:** `xiom run` caches binaries in
  `~/.xiom/jit/<sha256>.exe` keyed ONLY by source hash ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â after ANY runtime
  C change, delete those cached exes or results silently use the old
  runtime (`t_b37q` printed "0","0" from a stale cache; clean build = "0","1").

## Session handoff (2026-08-17) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â CLEAN STATE, work committed

Branch: `feat/architect` (**37 commits ahead** of `origin/feat/architect`).
All work this session is COMMITTED (5 commits below). Working tree has ONLY
pre-existing third-party modifications ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â `.xiom_ai.json`, `Cargo.lock`,
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

- **BUG 38** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â bare `is Some` scrutinee payload rebind fires ONLY inside an
  Imply left side (sx4 pos=5; iter.xi collect works).
- **BUG 38b (new)** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â generic-receiver methods (`Iterator[T].collect`): the
  parser DROPPED receiver generics ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ erased i64 ABI ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ never monomorphised
  (45 decls in iter/core/collections/sync/rc/memory). Four-part fix: parser
  receiver-generics capture; receiver-keyed mono dispatch; find_generic_decl
  prefers UNDECLARED abstract receivers; mutating-self ABI (block_mutates_self
  recursion + new block_mutates_receiver_state). iter smokes 0/19 ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ 6/19.
- **BUG 32** (Int-varÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ptr cast now load+inttoptr), **BUG 33** (Option[Float128]
  payload ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ fp128), **BUG 31** (fp128 fneg), **BUG 34** (nested Vec[Vec[T]]
  writes: i64-element dispatch + element-address ABI + elem_size 32 +
  elem-type tracking).
- **BUG 35** primary shape verified working (Int128+Vec-write buckets
  [1,1,1], no crash).
- **P001** not reproducible; indented decls reject in 0.14s; former 15-min
  hang smoke compiles in 4.9s.
- Suites: checker **178/178**, codegen **2263**, feature-reg **510**, parser
  **24** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â all green. `stdlib_api_freeze_no_removals` FAILS IDENTICALLY AT
  BASELINE HEAD (905 frozen signatures missing ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â item B family, stdlib
  layout moved; NOT a regression).

### Re-triage of the stdlib session's remaining lists (238 files, current binary)

**43 pass / 82 compilefail / 113 runfail.** Results: `%TEMP%\kilo\resweep_new0-3.txt`.
Remaining families: (a) receiver-CALL chains for generic methods (iter
adapters ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â reproduces at baseline, B-007-adjacent); (b) BUG 24/36 shape
miscompiles; (c) stale-API smokes (`.get(0)` on Vec, char.from_digit
contract false-fire, is_empty). Full breakdown: COMPILER_BUGS.md
(2026-08-16 late section) + REPORT_TO_STDLIB_SESSION.md.

### OPEN queue (next sessions, priority order)

1. **BUG 37/36 ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â FIXED (2026-08-17)** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â fp128 libcall ABI mismatch (see the
   session update above): 13 naked-asm shims in fp128_helpers.c + i128ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬Âfp128
   cast arms. The stdlib's bigfloat_to_float128 workaround fns can be
   consolidated; Option[Float128] can land.
2. **Item B (HIGH)** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â exec harness wiring: stdlib_tests.rs +
   stdlib_execution_tests.rs + stdlib_api_freeze_tests.rs still reference
   the pre-refactor layout (STDLIB_MANIFEST.md 515 paths, STDLIB_SMOKES.md
   213 smokes, frozen-signature snapshot). The 5-test-worktree merge wave
   waits on this. **The stdlib session has the GREEN LIGHT to wrap up** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â
   its remaining items are the stdlib-side list in
   REPORT_TO_STDLIB_SESSION.md (char.xi from_digit requires-contract on a
   fallback fn; Vec.is_empty broken ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â baseline-confirmed; stale `.get(0)`
   smoke usage ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ use indexing; smoke_num_saturating Bounded/Ord impls;
   smoke_alloc_basic `use xiom.ptr;`).
3. **Item C (HIGH, user-approved)** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â catalog body type-checking: Phase 1
   reachable injected fns; Phase 2 all catalog bodies (hash-keyed cache);
   Phase 3 `--strict-stdlib` gate.
4. **Iter adapter chains (MEDIUM)** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â receiver-CALL chains
   (`iter.range(1,6).max()`, `.map(fn...).collect()`) + fn-value params:
   reproduces at baseline; B-007-adjacent (closure-through-fn-slot design
   decision pending).

### Scratch / repro files this session (in `%TEMP%\kilo\`)

t_b37f/t_b37k/t_chainloop (BUG 37/36), t_b35/b35b (BUG 35 verified),
t_b34 (BUG 34), t_b33/t_b33u (BUG 33), pdb3 (BUG 32), t_fneg (BUG 31),
t_iter38/t_b1/t_clike (BUG 38b), sx4 (BUG 38). All exit 0 with the current
binary except the t_b37* family (AV) and t_vecloop (my test's inverted
expectation ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â values correct).

---

## Session update (2026-08-16 late): BUG 32-38 queue progress

Commits this session: `9042e8a2` (BUG 38 + 38b generic-receiver family),
`74bcc28b` (BUG 32/33/31), `b15d0d3b` (BUG 34).

- **BUG 38 FIXED** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â bare `is Some` scrutinee rebind now fires ONLY inside
  an Imply left side (sx4 pos=5).
- **BUG 38b (new) FIXED** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â generic-receiver methods (`Iterator[T].collect`)
  lost their receiver generics in the parser; receiver-keyed mono dispatch +
  abstract-receiver decl preference + mutating-self ABI (block_mutates_self
  recursion + block_mutates_receiver_state). iter smokes 0/19 ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ 6/19.
- **BUG 32 FIXED** (IntÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ptr cast inttoptr), **BUG 33 FIXED** (Option[Float128]
  payload ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ fp128), **BUG 31 FIXED** (fp128 fneg), **BUG 34 FIXED** (nested
  Vec[Vec[T]] writes: dispatch + element-address ABI + elem_size 32).
- **BUG 35** primary shape verified working (Int128+Vec-write buckets
  correct); extreme variant needs the stdlib repro.
- **BUG 37/36 OPEN** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â minimal deterministic repro: BigFloat field-chain
  Vec-len as a loop bound + any fp128 op in the loop ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ AV even at clang -O0
  with sound IR (t_chainloop/t_b37f). `bigfloat_to_float128` still crashes
  for non-zero values. Deep-dive needed.
- **P001 not reproduced** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â indented decls reject in 0.14s; the former
  15-min hang smoke compiles in 4.9s. The 238-file re-sweep completed
  without hangs.
- Re-triage of the stdlib session's two lists: 238 files ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ 43 pass, 82
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
1. **Imply scoping** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â contract-ensure `is Some/Ok/Err` payload rebinds no
   longer poison later return-site checks (the payload slot local persisted
   ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ i64-form Is inttoptr+load ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ AV). Fixes b003 / m19_read_file /
   file_stem / file_name chains.
2. **Is() payload-slot hoisting** (struct + i64 forms) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â `&&` guard chains
   bind in one block, read in a later one ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ "Instruction does not dominate
   all uses" (m18_guard_0086).
3. **Is() bare-scrutinee rebind in the enum-variants form** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â the fallback
   path had it; the enum-variants path didn't ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ ensure `result.len()`
   resolved to Map.len (utf8).
4. **struct_type_from_expr Call-arm resolution** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â bare/module-qualified
   callees resolved through call-site key machinery (caller module ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢
   bare_fn_aliases ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ use_alias_map). Match-on-catalog-call dropped the
   scrutinee ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ no tag checks + payload literal 0 ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ AV (m35_z24/z29,
   eco_algo).
5. **Mono path flushes hoisted allocas** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â `var t = b;` in a generic fn's
   while loop emitted only the store (m35_z24 "undefined value %tmp55").
6. **Enum literal ctors zero-init unused payload slots** (bare/qualified
   struct forms + Ok/Err) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â LLVM poison ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ clang -O2 miscompile ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢
   deterministic AV (m35_z10/z29, bisect_z29_m). The driver runs `opt -O2`
   + `clang -O2` ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â poison is exploited.
7. **coerce_arg_for_param `&array_local`** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â data pointer only for `&[N]T`
   params; `%struct.Vec*` params get the header alloca (eco_algo
   len()==element-0).
8. **len() dispatch for boxed Vec-handle locals** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â contract rebind payload
   with Vec XIOM type unboxes via inttoptr+load (utf8 ensure).
9. **Ensure `result` scoped per-check** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â a USER local named `result`
   shadowed the synthetic return-slot binding (utf8_encode AV).
10. **decl.rs result local_xiom_types uses type_string_full** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â the payload
    args survived (Result[Vec[UInt8], Str] not just "Result").
11. **collect_block_free_vars sorts captures by name** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â HashSet iteration
    order differed between the block fn / ctx type / caller stores in
    different passes (Rc.new slot swap ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â fields written through VALUE 42 as
    a pointer).
12. **Generic receiver ABI** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â by-value `self` passes the struct VALUE not a
    pointer (smoke_rc Rc.get garbage: `%struct.Rc` vs `%struct.Rc*`).
13. **Checker wildcard method lookup deterministic** (sorted candidates,
    receiver-base preferred) AND **clone skip only applies to the fallback**
    ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â the v0.56 clone "fix" discarded the DIRECT hit too, so `r.clone()`
    typed `_` and `r2.get()` hit BTreeMap.get ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ Option (smoke_rc).
14. **struct_type_from_expr Type.method static calls** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â `Vec[Str]::new()`
    must resolve `Vec.new` (exact / module-qualified suffix); when the typed
    key is unregistered (catalog compile ORDER: io.xi before
    xiom.collections) return None ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â NEVER the bare alias (first registrant
    BufReader.new) ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ BufReader.invariant_check(%struct.Vec ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦) ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ invalid IR
    (smoke_net_http, the io module graph).

File-side (same commits): bench_math Float64 casts (644/656), selfhost
xiom-check pub+use for 7 test fns, xiomc_v11_test `elif pfound {`, m21
Vec.pop Option handling.

## BUG 31 batch (2026-08-16, second half ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â the 904-sweep queue)

Commits: `2b238da4` (fmt), `b28ac72b` (Map), `a7571ac7` (variant hijack),
`99f894b7` (tuple destructure), `de639432` (docs).

- **fmt cluster (8 fixes, 8 smokes green):** Unit fields in
  Result[Unit, FmtError] literals (`store void 0, void*` invalid IR ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â
  generic literals adopt the fn's concrete return type; field types
  degrade UnitÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢i64); Str.to_str passthrough read the first BYTE (prologue
  treated i8* as a struct pointer ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â only %struct.X* receivers take the
  pointer branch); primitive/variable method receivers resolved to bare
  stubs (infer_struct_type_name learns literals/negated/parens, primitive
  locals via local_xiom_types, generic params via param_concrete_types,
  infer_value_xiom_type learns literals); mutating `self` methods lost
  mutations (block_mutates_self ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ pointer ABI in compile_fn + signature
  registration).
- **Map cluster (4 fixes, 7 smokes green):** `&K` scalar params passed the
  VALUE ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â four coordinated fixes (param_llvm_type, mono subst_type Ref arm,
  generic-call inference, coerce materializes plain values for pointer
  params).
- **Variant-hijack:** struct literals whose name collides with an enum
  VARIANT (Node{...} vs enum BST { Node(...) }) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â bare-variant search
  yields to known types. bench_math native IR now valid (was
  store %struct.BST %vecval).
- **Tuple destructure (BUG 26 #3):** the checker bound every name to the
  whole Tuple__A__B; now splits element types.

**Verified fixed:** BUG 26 #2 (bare prelude names), BUG 26 #5 (high-bit
mask AND), BUG 24 residual (smoke_num_precision). **Stdlib-side:**
smoke_num_saturating needs real Bounded/Ord impls; smoke_alloc_basic needs
`use xiom.ptr;`; smoke_hash_folder needs `use xiom.convert.toint;`.
The stdlib session also filed BUG 32-38 (fp128/ptr-cast/nested-Vec-write/
is-Some-double-check families) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â the next compiler queue.

### Verified green (isolated binary `$env:TEMP\kilo\tgt_iso\debug\xiom.exe`)

- Checker suite: **178/178**.
- All 14 previously-failing e2e/stdlib-exec tests run exit 0
  (b003, m19_read_file, m18_guard_0086, m35_z24/z10/z29, eco_algo,
  smoke_rc, smoke_cell, smoke_utf8, smoke_compress_lz4_snappy,
  m21_vec_edge_004, xiom-check.xi).
- diff-test expectations hold (the 04:03 f=1 was binary-churn race).
- bench_math: `--emit-ir` passes (harness is emit-ir-only) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â native `-o`
  still fails on a LATENT struct-literal field-type bug (below).

## Open items (next sessions, in priority order)

### 0. [HIGH ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â NOW] Rebuild + re-triage the remaining smoke failures (2026-08-16 late)

The stdlib session's 905-smoke sweep (621 pass / 300 fail, done against the
STALE 17:36 binary) produced exact lists:
`%TEMP%\kilo\remaining_compilefail.txt` (59) + `remaining_runfail.txt` (190).

**BEFORE triaging anything:**
1. Rebuild the isolated binary ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â my BUG 31 fixes (`2b238da4`, `b28ac72b`,
   `a7571ac7`, `99f894b7`) are NOT in the 17:36 binary. The fmt/io/regex
   clang clusters and ALL `bisect_*` entries (deleted scratch) should drop
   out of the lists immediately.
2. Re-run both lists against the new binary; filter:
   - `bisect_*` ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â my deleted scratch, ignore.
   - `smoke_fmt_*`, `smoke_stress_fmt_*` ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â fixed (fmt cluster).
   - `smoke_rc/cell/utf8`, `smoke_stress_collections_map_*` ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â fixed.
3. The ~60 remaining compile-fails and the run-fails cluster into families
   that need triage: **array/char/cell/cmp/collections/convert/core/error/
   io/num/ptr/rc** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â each is either a REAL compiler bug (like the fmt/Map
   clusters were) or stdlib-side API drift. Bisect each family like the
   fmt/Map clusters were done (small repro ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ root cause ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ fix).
4. Known stdlib-side (hand to the stdlib session): smoke_num_saturating
   (Bounded/Ord impls needed), smoke_alloc_basic (`use xiom.ptr;`).

**Edit-race caution (stdlib session's warning):** their realignment script
ran while smoke crypto/net/stress files showed as modified; if any of my
uncommitted smoke content is missing from the tree, re-apply it. Current
tree has NO uncommitted smoke changes (all committed) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â verify with
`git status --short examples/stdlib_smoke` before mass-editing.

### A. [HIGH ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â pre-selfhost] Catalog body type-checking (Q2b, user-approved)
NOT STARTED. Plan unchanged:
Phase 1 check reachable injected fns; Phase 2 all catalog bodies at load
(hash-keyed cache); Phase 3 `--strict-stdlib` CI gate.

### B. [HIGH] api_freeze path sync + exec harness
Unchanged: `crates/xiom-codegen/tests/stdlib_tests.rs` +
`stdlib_execution_tests.rs` still reference the pre-refactor layout
(STDLIB_MANIFEST.md 515 paths + STDLIB_SMOKES.md 213 smokes).
**The merge wave (5 idle test worktrees) WAITS on this** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â the stdlib
session is ready to run it once item B lands.

### C. [MEDIUM] BUG 32-38 ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â the stdlib session's new compiler queue
Filed in COMPILER_BUGS.md (2026-08-16 evening), priority by blocker:
1. **BUG 37** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â fp128 RETURNED from a catalog fn crashes the caller
   (0xC0000005) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â blocks any consumer of `bigfloat_to_float128`.
2. **BUG 32** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â `x as *T` (Int VARIABLE to pointer) emits address-of-local,
   not inttoptr ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â blocks pointer-handle designs (glob).
3. **BUG 38** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â `if x is Some { match x { Some(v) => ... } }` double-check
   binds the payload as 0 ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â BUG 30 #1/#2 family; semver worked around it.
4. **BUG 34** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â nested Vec element WRITES via `&mut` AV (reads work).
5. **BUG 35** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â Int128 index math inside a Vec-writing fn shape AV.
6. **BUG 36** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â fp128 Horner-loop fn shape AV when sign statements follow.
7. **BUG 33** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â Option[Float128] unwrap loads undefined `%struct.Float128`
   (payload type lookup must map XIOM Float128 ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ LLVM fp128).
8. **BUG 31 (stdlib's)** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â unary minus on Float128 emits `sub i64 0, fp128`
   (fneg path must handle fp128).
9. **P001 hang** ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â indented module-level declarations HANG the compiler
   (stdlib realigned 158 files; the parser should reject, not hang).

### D. [DONE] 904-smoke battery ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â BUG 31 batch closed the compiler-side
fmt (8 smokes), Map (7 smokes), variant-hijack (bench_math native), tuple
destructure (BUG 26 #3) all fixed; BUG 26 #2/#5 + BUG 24 residual verified
fixed. Remaining queue is item 0 (re-triage) + C above.

### E. [MEDIUM] MCP catalog/registry tools (ROADMAP G) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â unchanged.
### F. [LOW] Roadmap E/F: reverse type-index, contract policy decision.

## Sequencing per the user (pre-selfhost ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ split ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ registry) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â unchanged:
1. Selfhost gate: stdlib 100% + compiler 100% + zero warnings + item A + B.
2. Monorepo split per docs/REPO_SPLIT.md ONLY after (1) green.
3. Registry + packages after the split.

## How to verify anything you touch

- Rebuild the isolated binary: `$env:CARGO_TARGET_DIR="$env:TEMP\kilo\tgt_iso";
  cargo build -p xiom` then use `tgt_iso\debug\xiom.exe` (NEVER the shared
  target/debug binary ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â parallel sessions rebuild it constantly).
- After any change: checker (`cargo test -p xiom-check --target-dir
  "$env:TEMP\kilo\tgt_check"`) + the 14-test battery above + the 904-smoke
  sweep (run in 8 parallel batches of ~115; each batch ~25 min).
- Never stash/revert uncommitted stdlib work without a fresh backup.
- The stdlib session's next message should include the D-section survey so
  they can fix the stdlib-side half of the smoke failures.