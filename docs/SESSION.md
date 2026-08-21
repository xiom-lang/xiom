# XIOM Compiler Session -- Handoff (2026-08-19)

## Session update (2026-08-21, round 12 FIXED -- compiler): rm1 Str-return closures + cb2 enum-variant receivers; stdlib sweep 801/907, tuple payloads remain

Compiler session (round 12, commit pending) closed TWO of the three
residual closure shapes:

1. **Str-returning closures through Err construction FIXED** (rm1):
   `err.map_err(fn(e: Str) -> Str { str_concat("ERR_", e) })` corrupted
   the payload -- the closure thunk registered params as i64 locals WITHOUT
   their declared XIOM types, so a Str param used in the body resolved to
   "Int" and the materialize path TRUNCATED the string handle to a byte
   (`alloca i8; trunc i64 to i8`). Fix: closure params now record
   local_xiom_types/ref_params/signed_locals (mirroring decl.rs); the mono
   fn-typed param's ret also substitutes through type_map (latent struct-
   return ABI hazard). smoke_core_result_map/deep_chain/chains exit 0.
2. **Ordering-returning &T-param closures FIXED** (cb2, deeper root):
   probing then_with/reverse exposed the real bug -- MODULE-QUALIFIED
   ENUM-VARIANT receivers (`cmp.Less.reverse()`) were DROPPED from method
   calls (`call @Ordering.reverse()` with no self; `then` lost its second
   arg) because infer_struct_type_name's Field arm had no enum-variant
   resolution -> receiver_is_instance=false. Passed by luck in script mode;
   garbage at -O2 (the e2e `-o` path). Fix: resolve the variant's parent
   enum via resolve_enum_for_variant. The full cmp battery
   (cmp_by/array_sort_by/reverse/then/then_with/zero-arg/capturing/
   struct-T) is green in BOTH run and -o modes.

**REMAINING COMPILER-SIDE QUEUE (~104 documented, each with minimal repros
+ user-space proofs in COMPILER_BUGS.md):**

1. **Option[(K, V)] TUPLE payloads** (BTreeMap.first_entry):
   `Some((keys[0], values[0]))` corrupts -- the index reads verify
   standalone, so the tuple-payload CONSTRUCTION through the Option is the
   suspect (the Tuple__K__V struct inside the Option payload slot -- the
   match-arm binding / payload field handling for 2-slot tuples).
   Blocks btree_map / btreemap (first_entry/last_entry -- pre-existing,
   exit 7 / exit 2 at baseline).
2. **Iter-adapter feature gap** (9 smokes): Range has only next/len/
   contains/sum/product -- the map/filter/collect ADAPTER CHAIN doesn't
   exist in the stdlib (targets the planned API). Needs the iterator
   PROTOCOL (the For stmt is a hardcoded Range GEP -- `for x in set` also
   blocked on this) + the closure ABI (now available). Stdlib-side work.
3. **narrow-SIGNED zext** (smoke_stress_collections_vec_narrow exit 5):
   the inline pop/get on SIGNED narrow elements (Int16 -30000) zext the
   bit pattern (35536) instead of sign-extending (emit_elem_payload_load).
4. **json enum-Vec-Map heap layer** (smoke_stress_serialize_json_nested --
   flaky; the json value enum with Vec/Map payloads through the heap).
5. **SIMD flags** (smoke_simd -- flaky across builds; the m34_y15/y20
   pattern -- clang -O2 codegen of the linked MSVC CRT is layout-sensitive;
   sorted emission fixed the flip, the SIMD flag set may still be off).
6. **The 16 clang codegen variants** (the documented C001-checker/
   clang-flags matrix).
7. **Pre-existing, unchanged**: smoke_collections_btree_map (exit 7) +
   smoke_stress_collections_btreemap (exit 2) -- the first_entry/last_entry
   traversal (the tuple-payload item above likely unblocks these);
   smoke_error_edge + smoke_iter_edge (exit 1 at baseline).

Campaign trajectory: 516 -> 621 -> 679 -> 738 -> 765 -> 779 -> 793 -> 799 -> 801.
Compiler-side closed roots: BUG 31-56 + the round-fixes (gzip decompress
payloads, path.xi env import, Vec/Set/Slice method injection + pointer
arithmetic GEP, non-pub generic type decls + is_llvm_struct_named, ref
payload auto-deref, Set ABI, Ord/Bounded C001, B-007 closures, round-12
rm1 Str-return closures + cb2 enum-variant receivers).
Stdlib-side hardened: RefCell, PathBuf, gcd, crc32, VecDeque, Set/Queue/
Stack, redundant requires traps, prose ensures, smoke semantics.

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
