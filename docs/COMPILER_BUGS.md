# XIOM Compiler Bugs -- Log

Dated sections. Each entry: file(s) affected, construct, error observed, and
(recommended) fix direction for the compiler team. Stdlib workarounds are
deliberately NOT applied where the stdlib mandate says "production grade, no
workarounds" -- the compiler must be fixed, then the stdlib lands.

---

## STATUS SUMMARY -- authoritative (2026-08-11 16:3x, compiler session)

> Read this first. Older dated sections below may contradict it (e.g. the
> BigInt/BigFloat Phase C entry claimed BUG 11 open -- that predates the fix).

| Item | Status | Fix commit | Verified by |
|------|--------|-----------|-------------|
| BUG 1 -- tuple-of-struct opaque LLVM type / truncated slots | **DONE** | `d22068f8` | e2e_m37_tuple_struct; probe_big/probe_big2; bigint_div_mod no longer crashes |
| BUG 8 -- catalog fns with `&Vec[Int]`/`&Vec[UInt8]` params | **DONE** (was the uncommitted coerce.rs state) | `384d5666` (revert) + verified | e2e_m37_catfix_catalog_vecref; probe_bit (12&10=8) |
| BUG 9 -- private catalog struct types degrade to i64 | **DONE** | `3ccd004c` | e2e_m37_catfix_private_type (b9mod+b9main, sum=7) |
| BUG 10 -- float literals truncated to 6 decimals | **DONE** | `a2aafa26` | e2e_m37_float_precision (0.123456789 round-trips) |
| BUG 11 -- unsafe-extern double marshalling (math.sqrt via trampoline) | **DONE** | `7f7b7b54` | stdlib-exec **72/72** (complex + net_folder pass); e2e_m37_unsafe_option_return |
| Parser -- `bits[L - 1]` (uppercase-ident index with arithmetic) | **DONE** | `40441ca7` | e2e_m37_index_arith; bigint.xi/bigfloat.xi parse |
| Catalog -- import lookup walked the whole tree per miss (minutes/hang) | **DONE** | `1e982ebf` | stdlib-compile 108.7s -> 23.7s; probe_bit compile 138-160s -> 9.5s |
| Struct `&T` param mutation silently lost | **DONE** | `d22068f8` | e2e_m37_ref_mut; probe_dig/probe_cmp |
| clang -O2 hang (alwaysinline everything) | **DONE** | `d22068f8` | smoke_bigint.xi compiles ~5-14s (was >300s) |
| Unsafe-block capture collector missing Expr::Struct etc. | **DONE** | `e0f96fef` | e2e_m19_read_file_content |
| Parallel codegen `__unsafe_ctx_0` redefinition | **DONE** | `e0f96fef` | e2e_i2_parallel_codegen (all 5 ecosystem tasks) |
| BUG 12/17 -- Vec[Float64]/Vec[Str] element type lost on `&Vec[T]` params | **DONE** | `2ae300fd` | e2e_m37_vec_f64 (R=0 isolated); m12b |
| BUG 13 -- fp128 link + coercion (soft-float helpers) | **DONE** | `2ae300fd` + `3b8f5415` (fp128_helpers.c) | e2e_m37_f128 (R=0 isolated); C harness 400/5/2500/1002.5 |
| BUG 14 -- UInt64->UInt128 sext / UInt128>> ashr | **DONE** | `2ae300fd` | e2e_m37_u128 (R=0 isolated); m14 |
| BUG 15 -- bare `shr`/`shl` hijacked by math-builtin intercept | **DONE** | `2ae300fd` | e2e_m37_shr_builtin (R=0 isolated); m15 |
| BUG 16/18 -- skiplist+trie / string+similarity+time 0xC0000409 combos | **DONE** | `2ae300fd` (fn-key + longest-prefix walk + encoding repair) | m16a-f probes R=0; m16b-e (bare/leaf forms) R=0 |
| **BUG 2 -- module-global struct FIELD writes lost** | **DONE** | `f0388644` | e2e_m37_global_field_write (g.v = 5 persists); probe_gf R=0 |
| **BUG 3 -- module-global fn-call initializers zero** | **DONE** | `f0388644` | e2e_m37_global_fn_init (`var G = _mk(1)` -> 10 via @llvm.global_ctors); probe_const3 R=0 |
| Str + Int/Char/UInt concat crashed (inttoptr of the integer -> AV) | **DONE** | `f0388644` | e2e_m37_str_int_concat ("y = " + 42 -> "y = 42"); probe_concat0 R=0 |
| Dotted-module chain binding -- `use stdlib.xiom.io` flaky 40-60% "cannot call" (parser nests dotted paths; process_use bound the {xiom:{io:...}} chain; stdlib-prefixed uses skipped preload/prelude) | **DONE** | `f0388644` | m19_read_file 8/8 + min repro 8/8 deterministic; stdlib-compile 40/40; checker 178/178 |
| BUG 19 [U+FFFD] NaN-producing Float64 ops returned sentinel/0xC000001D (float `!=` ? `fcmp one`; `Str+Float64` concat inttoptr) | **DONE** | `9c3a2f9e` | e2e_m37_nan_ieee (23 checks) R=0; probe_nan prints "c = nan / is NaN" exit 0; 17/17 sweep |
| BUG 43 -- Result[Float64, Str] payload read via sitofp (direct-call scrutinee) | **DONE** | this session | e2e_m37_bug43_result_f64_payload; smoke_core_convert exit 0 |
| BUG 44 -- deref/coercion of `&Str` loads a byte (`i8`) instead of the pointer (`i8*`) | **DONE** | this session | e2e_m37_bug44_str_deref; probe_b44 exit 0 |
| BUG 45 -- method-form interface dispatch inside generic-bound fns -> stub | **DONE** (root cause = BUG 46's i64* param degradation) | this session | e2e_m37_bug45_iface_method_generic; eqt11/eqt18/eqt20 exit 0 |
| BUG 46 -- generic `&UserStruct[T]` param field reads return garbage (mono Ref arm degraded to i64*) | **DONE** | this session | e2e_m37_bug46_generic_struct_ref; eqt18/20 exit 0 |
| BUG 47 -- `ref_params`/`param_locals` leak across fns -> later value param mis-dereffed (AV) | **DONE** | this session | e2e_m37_bug47_ref_params_leak; smoke_sort exit 0 |
| BUG 41/42 follow-up -- mono Ref arm made `&Slice[T]`/`&[N]T` lowering UNREACHABLE (i64* params) | **DONE** | this session | e2e_m37_bug45/46/47 (restored `%struct.Vec` by-value + `elem_ty*`); warning gate zero |
| S7 NASM/SIMD tracks (math/crypto/hash/compress asm, CPUID dispatch) | **OPEN** -- off-limits to stdlib session (crates/ + stdlib/runtime/*.c); pure-XIOM fallbacks in place | -- | -- |
| Selfhost plan | **WRITTEN -- execution in progress** | `090ed5d1` | docs/SELFHOST_PLAN.md (phases 0-8) + docs/checklists/selfhost-phase0.md |

**Suite state (current, 2026-08-11 22:10):** fast suite 1103/10/1 -- ALL 10
failures are the parallel stdlib session's in-flight restructure (stdlib-exec
complex/hash/net/rand folder moves + smoke_math_core renamed to
smoke_math_tower, lsp/mcp module-list tests against the new layout, diff
documented pre-existing ignore); **zero compiler regressions**. e2e harness
verification of the 4 new e2e_m37 tests is blocked until the parallel session's
next compiler rebuild (their target/debug/xiom.exe predates `2ae300fd`); exact
harness invocation replicated with the isolated binary: compile=0 run=0 for
all 6 affected tests. Warning gates 0/0.

---

## 2026-08-18 -- Compiler session: BUG 43-47 FIXED + scripting test race + mono &Slice regression

### FIXED -- BUG 43: Result[Float64, Str] payload read via sitofp (direct-call scrutinee)

- **Root cause:** the Some/Ok/Err payload binding resolved the payload XIOM type
  ONLY for `Expr::Ident` scrutinees (`local_opt_payload*` tracking). A scrutinee
  that is a DIRECT CALL (`match core.to_float_from_str("3.14")`) had no declared
  payload -> the Float64 bound as raw i64 bits and float ops sitofp'd 3.14's
  pattern (~4.6e18) -- smoke_core_convert exit 11.
- **Fix (lib.rs + stmt.rs):** new `scrutinee_payload_xiom(expr, field_idx)` -- for
  Call/GenericCall scrutinees resolves the callee's declared return type via
  `callee_return_xiom` + the existing bracket-aware `option_result_payload` /
  `option_result_err_payload`; both payload-binding paths (guard pre-extract +
  arm binding) use it. Definition side (bitcast) was already correct.
- **Verified:** probe exit 0 (direct-call + ident-held + Err-payload Str);
  smoke_core_convert exit 0; e2e `e2e_m37_bug43_result_f64_payload`.

### FIXED -- BUG 44: deref/coercion of `&Str` loads a byte instead of the pointer

- **Construct:** `var p = &s; var d = *p;` (deref of &Str) and passing `&Str`
  where `Str` is expected (auto-coercion). Both AV'd or compared a byte.
- **Root cause (three defects):**
  1. `UnaryOp::Deref` used `trim_end_matches('*')` -- for a `&Str` param
     (`i8**` -- pointer to the Str slot) it stripped BOTH stars and emitted
     `load i8, i8**` instead of `load i8*, i8**`. One-star strip fixes it.
  2. LOCALS bound from `&expr`/`&T` were untracked (params had `ref_params`;
     locals had nothing) -- `*p` fell to the legacy byte-pointer path
     (`inttoptr i64 -> i8*; load i8`). New `ref_locals` set (context.rs) +
     `track_ref_local` (stmt.rs Let/Var) mirrors `ref_params`; the deref path
     resolves the pointee via `local_xiom_types`.
  3. `&T`->T auto-coercion: `expect_str(&s)` / `expect_str(p)` passed the slot
     ADDRESS (or truncated it to a byte) as the string. New
     `coerce_ref_arg_to_pointee` (coerce.rs) derefs the address when the param
     type exactly equals the pointee LLVM type (i8* for Str); `&T` params
     (i64/i8**) keep receiving the address.
- **Verified:** probe (deref local + param, coercion ident + literal-ref forms)
  exit 0; e2e `e2e_m37_bug44_str_deref`.

### FIXED -- BUG 46: generic `&UserStruct[T]` param field reads return garbage

- **Root cause:** the mono `subst_type` Ref arm's struct check only matched
  BARE registered keys (`struct_types.contains(&name)`); user generic structs
  register MODULE-QUALIFIED ("eqt20.Box2"), so `&Box2[T]` params degraded to
  `i64*` -- the callee's field GEPs indexed an i64, and every field read
  degraded to 0 (the "empty body" was `ret i64 0`).
- **Fix (lib.rs mono Ref arm):** the struct check now also matches the
  qualified suffix (5c.35 style, same as the Named arm) and emits
  `%struct.{qualified}*`.
- **Verified:** eqt18/eqt20 exit 0 (user-slice element reads + len reads),
  probe exit 0; e2e `e2e_m37_bug46_generic_struct_ref`.

### FIXED -- BUG 45: method-form interface dispatch inside generic-bound fns -> stub

- **Root cause:** the SAME mono Ref-arm i64* degradation as BUG 46 -- the
  slice element loads produced garbage, and the method-form dispatch
  (`el.eq(&value)`) resolved against the wrong shape. With the param typed
  correctly, the primitive fast-path inlines the correct comparison.
- **Verified:** eqt11 (user slice + generic contains), probe with a T-typed
  receiver (`f_eq[Int](3,3)`) -- both exit 0; e2e
  `e2e_m37_bug45_iface_method_generic`.

### FIXED -- BUG 47: `ref_params`/`param_locals` leak across functions (fn-param -> AV)

- **Root cause:** `param_locals` and `ref_params` were NEVER cleared between
  function compilations (the per-fn reset block missed them). A `&T` param
  named `b` in an EARLIER fn (`cmp_int(a: &Int, b: &Int)`) left a stale
  entry, so a LATER fn's plain value param named `b` was misidentified as a
  ref-param -- `x.compare(&b)` compiled the VALUE and the eq/compare fast-path
  dereferenced address 7 (`inttoptr i64 7 + load`) -> 0xC0000005. The fn-param
  `compare` in heap_sort_by/heap_sift_down_by was the same family (stale
  ref-param classification broke the fn-pointer arg flow).
- **Fix (decl.rs):** clear `param_locals` + `ref_params` (and the new
  `ref_locals`) in the per-fn reset block.
- **Verified:** full-collision probe (impl `Int.compare` used via a generic
  bound + `heap_sort_by(&mut hb, cmp_int)`) exit 0; smoke_sort exit 0; e2e
  `e2e_m37_bug47_ref_params_leak`.

### FIXED -- BUG 41/42 follow-up: mono Ref arm made `&Slice[T]`/`&[N]T` lowering UNREACHABLE

- The BUG 41/42 batch merged the Ref/Ptr/MutRef mono arm, making the
  `&Slice[T]` (by-value `%struct.Vec`) and `&[N]T` (`elem_ty*`) specialization
  UNREACHABLE (dead-code warning) -- generic fns then lowered `&Slice[T]`
  params to `i64*` and read the first element as the length. Restored the
  shape checks in the live arm; the dead arm is deleted (warning gate: zero).

### FIXED (test-side) -- scripting `test_standalone_simple` flake: shared `s_out.exe` race

- Both standalone tests derived the OUTPUT path from a shared "s_out" name;
  with `--test-threads=32` they raced on the same output file (Windows
  sharing violation -> intermittent "Access is denied" -> non-success status).
  The output path now derives from the UNIQUE script name. Verified 3/3 full
  scripting runs 34/34.

### NOTE -- checker gap (NOT this batch): `Eq5[T].eq(el, &value)` associated-form Self substitution

- eqt13's associated form still fails the CHECKER ("argument 1 type mismatch:
  expected Self, found Int") -- `Self` in an interface's method param doesn't
  substitute to the type arg in associated-form calls. The tower pattern
  (explicit `Eq5[Int].eq(...)` with fn-style params) avoids it; stdlib uses
  method form. Candidate for a future checker session.

---

## 2026-08-10 -- BigInt/BigFloat session findings (feat/architect)

### BUG 1 (CRITICAL) -- Tuple return types containing structs emit an OPAQUE LLVM type

- **Construct:** any function returning a tuple whose element is a non-primitive
  struct, e.g. `fn f() -> (BigInt, BigInt)`, `fn g() -> (BigInt, BigInt, BigInt)`.
  Tuple elements of i64 (e.g. `(Int, Int, Int)` in `time.civil_from_days`,
  `bits.unpack_u32_le`) work; tuple elements that are structs do not.
- **Error / observed behavior:**
  1. When clang rejects the IR:
     ```
     error: clang failed with exit code 1
     stderr: xiominput.ll:129:18: error: Cannot allocate unsized type
     129 |   %tmp5 = alloca %struct.Tuple__probe_tuple.Pair__probe_tuple.Pair
     ```
     The `%struct.Tuple__...` type is referenced but never defined (opaque).
  2. When clang tolerates it (bigint case), the constructor stores only the
     FIRST i64 of the struct into the tuple slot (`store i64 %tmp33, i64* %tmp34`
     where element 0 is a 40-byte `BigInt`) -- the rest of the slot is garbage.
     Reading `dm.0` then dereferences garbage Vec pointers:
     - `bigint_div_mod(3, 10)` -> `0xC0000005` ACCESS_VIOLATION (or
       `0xC0000409` STACK_BUFFER_OVERRUN)
     - `bigint_div_mod(10, 3)` -> crash; `bigint_mod` -> hang/infinite loop;
       `bigint_sqrt` -> crash; `bigint_ext_gcd` -> crash.
- **Minimal repro** (`probe_tuple.xi`):
  ```xiom
  module probe_tuple
  type Pair = { a: Int; b: Int; }
  fn make_pair(x: Int, y: Int) -> (Pair, Pair) {
    return (Pair{ a: x; b: y; }, Pair{ a: y; b: x; });
  }
  fn main() -> Int {
    var t = make_pair(3, 7);
    if t.0.a != 3 { return 1; }
    if t.1.b != 3 { return 2; }
    return 0;
  }
  ```
  -> clang error above (opaque `%struct.Tuple__probe_tuple.Pair__probe_tuple.Pair`).
- **Impact on stdlib (blocks Phase A/B):**
  - `stdlib/xiom/bigint.xi`: `pub fn bigint_div_mod(a: &BigInt, b: &BigInt) -> (BigInt, BigInt)`
    (PRE-EXISTING, frozen signature, **never exercised by any test until now** --
    `tests/regression/` contains zero `bigint_div_mod`/`bigint_mod`/`bigint_gcd`
    call sites; the original 3-assertion smoke only exercised parse/mul/compare).
    The original author already worked around the tuple problem once for the
    internal helper (`_div_mod_base` returns a named `DivModResult` struct --
    comment: "Returns a DivModResult struct to avoid tuple issues") but the
    public `bigint_div_mod` kept the tuple and was never tested.
  - Planned (this session, blocked): `bigint_sqrt_rem -> (BigInt, BigInt)`,
    `bigint_ext_gcd -> (BigInt, BigInt, BigInt)`.
- **Fix direction:** codegen must emit a concrete LLVM struct type definition
  for `Tuple__<T>__<U>` (aggregate of element types) and store elements with
  the element's real size (memcpy-style), not `store i64`. Enum variants with
  struct payloads (Option/Result) already work -- reuse that lowering path.

### BUG 2 -- Module-global struct FIELD writes are silently lost

- **Construct:** `var g: W = W{ v: 0; };` at module scope, then `g.v = 5;` in a
  function; reading `g.v` afterwards returns 0. Whole-value assignment
  (`g = W{ v: 5; };`) persists correctly.
- **Minimal repro** (`probe_gf.xi`): `_set()` does `g.v = 5;` -> `main` prints
  `g.v` == `0`.
- **Impact:** module-level mutable struct state (round-mode globals etc.) must
  be updated by whole-value assignment until fixed. Stdlib constants that
  would be module globals are affected (see BUG 3).
- **Fix direction:** GEP-store on a module-global struct must write through to
  the global (currently the value appears to be copied on access).

### BUG 3 -- Module-global `var` initializers that CALL functions are silently zero

- **Construct:** `var G: BigInt = _mk_one();` at module scope (fn call in the
  initializer). `G` reads back as zero at runtime, no diagnostic.
  Scalar literals (`var _BASE: Int = 1000000000;`) and struct literals
  (`var g: W = W{ v: 0; };`) initialize correctly.
- **Minimal repro** (`probe_const2.xi` / `probe_const3.xi`): module fn
  `_mk_one()` returns `bigint_from_int(1)`; global `G_ONE = _mk_one()` prints
  `G_ONE=0` while a local call prints `local=1`.
- **Impact:** the spec constants `BIGINT_ZERO/ONE/TEN` (bigint) and
  `BIGFLOAT_ZERO/ONE/TWO/TEN/HALF/PI/E` (bigfloat) cannot be module globals
  built by code; the stdlib exposes them as pure constructor functions
  (`bigint_one()`, `bigfloat_pi()`, ...) until fixed.
- **Fix direction:** run global initializer expressions (or lower them to
  `@llvm.global_ctors` / runtime startup init) instead of emitting a zero
  initializer.

### NOTE 4 -- Module-qualified enum variant access resolves to a module

- `bigfloat.Down` (variant via module qualifier, like `cmp.Less` in cmp.xi)
  fails with "argument 1 type mismatch: expected RoundMode, found module" in
  some positions, while type-qualified `bigfloat.RoundMode.Down` (log.xi
  style) works everywhere. Checker's dotted-name resolution appears to treat
  the variant segment as a sub-module first. Minor; type-qualified form is the
  documented style.

### NOTE 5 -- D4b aggregate: leaf sub-module fns not reachable through a 1-segment aggregate

- After `use xiom.bigfloat;` (flat aggregate manifest `module xiom.bigfloat`
  + `use xiom.num.bigfloat;`), the sub-lib's TYPES resolve
  (`bigfloat.RoundMode.Nearest`) but its FUNCTIONS do not:
  - `bigfloat.bigfloat_from_int(42)` -> "cannot call on this expression"
  - `num.bigfloat.bigfloat_from_int(42)` -> "undefined variable 'num'"
  - `xiom.num.bigfloat.bigfloat_from_int(42)` -> works (full dotted path).
  The proven D4b example (`use xiom.math;` + `math.core.sqrt(x)`) works only
  because the sub-lib's parent segment (`math`) equals the aggregate leaf.
  With `use xiom.num.bigfloat;` directly, leaf calls (`bigfloat.fn`) work.
  Checker should attach `num.bigfloat` under the aggregate key `bigfloat`
  (parents map builds from the full key "num.bigfloat" -> parent "num", never
  from the importing manifest).

---

## 2026-08-10 -- Compiler-hardening session (BUG 1 fixed; findings below)

### FIXED -- BUG 1 (tuple-of-struct codegen) -- root cause and fix

Three coordinated codegen fixes landed (commits pending, crates/xiom-codegen):

1. **`concrete_type_for` Tuple branch** (lib.rs): tuple type keys now
   module-qualify their element names (`Tuple__probe_tuple.Pair__probe_tuple.
   Pair`), matching the expression-level registration used by the body's
   alloca/GEP. Previously the fn SIGNATURE used bare names (`Tuple__Pair__
   Pair`) while the body used qualified names -- two different LLVM types for
   the same tuple -> clang "Cannot allocate unsized type" / ret type mismatch.
2. **`resolve_type_key`** (lib.rs): prefers the current-module-qualified key
   and SKIPS generated aggregate keys (`Tuple__...`, `Option__...`,
   `Result__...`, `_Anon__...`) in the suffix search -- `Tuple__x.Big__x.Big`
   ends with `.Big`, so a bare `Big` could resolve to the tuple itself and
   nest the tuple name into itself (double-nested `Tuple__Tuple__...`).
3. **`parse_struct_field_types`** (lib.rs): looks up the registered type_meta
   layout instead of splitting the name on `_` (which degraded every
   non-primitive element to i64 -- tuple slots stored only the first i64 of
   each struct). Element stores now use the real field width.

Also routed tuple PARAM types through `concrete_type_for` (`param_llvm_type`,
decl.rs signature + prologue), and fixed `coerce_arg_for_param` for
non-ident `&expr` lvalues. Verified: `probe_tuple`, `probe_big` (40-byte
structs, 3-element tuples), `probe_big2` (copy-from-tuple + by-ref passing)
all compile and run correctly; `bigint_div_mod` no longer crashes (was AV /
garbage before the fix).

### FIXED -- Struct `&T` param mutation was silently lost

- **Construct:** `fn _trim(b: &BigInt) { b.digits.pop(); }` -- mutation of a
  field through a `&T` STRUCT param. Struct `&T` params were passed BY VALUE,
  so `pop()`/`push()` on the param's Vec field mutated a discarded copy --
  trailing-zero limbs were never trimmed (`3*3` produced digits `[9, 0]` with
  len 2), breaking `bigint_eq`/`bigint_compare` on arithmetic results
  (`compare(3*3+1, 10)` returned 1 while both printed "10"). Scalar `&T`
  params already carried the address; structs now do too.
- **Fix (crates/xiom-codegen):** `param_llvm_type` lowers plain-struct `&T`
  params to `%struct.X*` (address), matching `&mut T`. Generic containers
  (`&Vec[T]`, `&Slice[T]`, `&Map`, `&Set`) keep the established by-value ABI.
  The existing auto-deref field-read path (expr.rs), Vec push/pop mutation
  path, `coerce_arg_for_param` (slot-address + fresh-alloca fallback for
  `&fn_call()` lvalues), and `infer_llvm_type`'s Field arm (pointer-base
  normalization + `Vec[...]` base-container resolution) complete the path.
- **Verified:** `probe_mut` (pop through `&W` propagates), `probe_ref`
  (scalar `&Int` unchanged), `probe_dig` (mul/add/sub limb lengths now
  correct -- `_trim` works), `probe_cmp` (compare correct), `probe_dm`
  (div_mod invariant `q*b+r==a` holds for small numbers).

### FIXED -- clang -O2 hang on large modules (alwaysinline every function)

- **Construct:** every emitted function was marked `alwaysinline`. With a hot
  caller (the extended bigint smoke's `main`) calling many large functions,
  LLVM inlined the whole library into `main` and `clang -O2` never finished
  (>300s on a 735KB module; `-O0` finished in ~16s; stripping the attribute
  finished in ~3s).
- **Fix (decl.rs):** size-based inline policy -- `approx_block_cost`
  (recursive statement count): <=10 stmts `alwaysinline`, <=48 `inlinehint`,
  larger no attribute. Preserves P0-4 hot small-function inlining
  (`read_u16_be`-style) while preventing optimizer explosion.
- **Verified:** extended `smoke_bigint.xi` compiles in ~14s (was hanging).

### NOTE 7 -- `bigint_div_mod` quotient is WRONG for multi-limb dividends (STDLIB, parallel-session Phase A)

- **Construct:** `bigint_div_mod` on any dividend with >=2 limbs produces a
  too-small quotient and a remainder >= divisor. Repros (all other bigint
  functions verified correct -- mul/add/sub/compare/to_str/from_int/from_str):
  - `bigint_div_mod("1000000005", 2)` -> q=2, r=1000000001 (correct:
    q=500000002, r=1).
  - `bigint_div_mod("987654321987654321", 12345)` -> q=80004000080004,
    r=4941000004941 -- satisfies `q*b+r==a` but r >= b (correct: r < 12345).
  - `bigint_div_mod("987654321987654321", 1000000000)` -> q=987654321.
- **Root cause (stdlib logic in `stdlib/xiom/bigint.xi`, NOT the compiler):**
  `_estimate_q_digit` (line 132) estimates with a single top limb
  (`n_hi / d_hi`) with no carry window from the higher limbs, and the
  div_mod loop only corrects OVERestimates (decrements `est`), never
  UNDERestimates -- so a low estimate sticks. Also `est <= 0 { return 1; }`
  forces a bogus digit of 1 when the true digit is 0.
- **Action:** parallel BigInt session owns `bigint.xi` -- fix the estimator
  (Knuth-style two-limb window + upward correction, or process the remainder
  carry) and remove the `est<=0 -> 1` shortcut. Compiler side is verified
  correct via the probes above.
- **Smoke impact:** `stdlib_exec_bigint_runs` fails at assertion 8 until this
  stdlib fix lands (compile/hang issues from BUG 1 and the optimizer hang are
  fixed).

---

## 2026-08-11 -- BigInt/BigFloat session: follow-up verification (post coerce.rs edit)

### BUG 8 (CRITICAL) -- RESOLVED by the compiler session's uncommitted coerce.rs edit

Re-verified on the rebuilt compiler: `_from_twos_bits` / `_bits_to_bigint`
now emit correct signatures
(`define %struct.BigInt @bigint._from_twos_bits(%struct.Vec %param0)`),
`bigint_bit_and/or/xor` work, and the extended `smoke_bigint.xi` (20+
assertions incl. two's-complement negatives) passes end-to-end.

### BUG 9 -- Catalog fns returning module-local PRIVATE struct types degrade to i64

- **Construct:** `fn f(...) -> PrivateType` where `PrivateType` is a struct
  declared (non-`pub`) in the same catalog stdlib module. The definition and
  call sites emit `i64` instead of `%struct.PrivateType` -> ABI mismatch ->
  AV at runtime. `pub type` in the same position works.
- **Minimal repro** (verified): `stdlib/xiom/num/probet.xi` with
  `type Wrap = { a: Int; b: Int; }` + private `make_wrap` -> `call i64
  @probet.make_wrap` / `define i64 @probet.make_wrap`; with `pub type Wrap`
  -> `%struct.Wrap` and correct results.
- **Stdlib handling:** `IntFrac` in `num/bigfloat.xi` is declared `pub` --
  it is part of the module's public surface anyway (split result of the
  rounding/floor machinery).
- **Fix direction (compiler):** catalog decl registration must resolve
  non-pub module-local struct types (or the checker must reject them) --
  silently defaulting to i64 corrupts memory.

### BUG 10 (pre-existing) -- Float literals emitted rounded to 6 decimals

- **Construct:** any Float64 literal. The emitted LLVM constant is the
  literal formatted with ~6 decimals: `0.000000001` -> `0.000000` (= 0.0),
  `3.14159265358979` -> `3.141593` (~=1e-7 error). Short literals (<= 6
  decimals) are exact and unaffected. Present in the OLD compiler binary as
  well -- pre-existing, not a regression from the hardening session.
- **Observed:** `if diff > 0.000000001` became `if diff > 0.0`; probes
  comparing literals like `3.1400000000000001` vs `3.14` reported "equal".
- **Stdlib handling:** no stdlib fn depends on > 6-decimal literals (PI/E
  are parsed from strings); the bigfloat smoke expresses its 1e-9 tolerance
  as `diff * 1000000000.0 > 1.0` (emission-safe).
- **Fix direction (compiler):** emit `%f`-style literals with full
  precision (e.g. `%.17g`) or hex float constants (`0x1.91eb851eb851fp+1`).

### BUG 11 -- `unsafe { extern }` calls return wrong values for runtime Float64 args

- **Construct:** `stdlib/xiom/math.xi` `pub fn sqrt(x: Float64)` lowers to
  `unsafe { return sqrt(x); }` (extern "C" libm). With a RUNTIME argument the
  result is wrong: `math.sqrt(4.0)` (var-held) != 2.0, `sqrt(9.0)` != 3.0.
  Constant-folded calls appear fine. Same symptom in `smoke_complex.xi`
  (`complex_abs` -> `math.sqrt`) and `smoke_net_folder.xi` -- both fail in the
  stdlib-exec suite (70/72 pass; the two failures are the unsafe-extern
  path, unrelated to bigint/bigfloat).
- **Suspect:** the Unsafe Confinement trampoline
  (`xiom_trampoline_call` + `__unsafe_ctx` struct) -- double args/results
  through the context struct. Compiler session's in-flight domain
  (uncommitted coerce.rs + confinement phases); NOT the stdlib code.
- **Fix direction (compiler):** verify double (and i64) arg/ret marshalling
  through `__unsafe_ctx`; compare against `sqrt_pure` (non-unsafe impl).

---

## 2026-08-11 -- REGRESSION in committed &T fix (d22068f8) -- RESOLVED

### BUG 8 (CRITICAL) -- Catalog-module fns with `&Vec[Int]` params emit an EMPTY signature

> RESOLVED: the compiler session's later uncommitted coerce.rs edit fixed the
> &expr lowering; verified on the rebuilt compiler (see the follow-up section
> above). Kept below as the original report.

- **Construct:** any function in an IMPORTED (catalog) stdlib module whose
  parameter is `&Vec[Int]` (i64-element generic container). The function
  definition is emitted with NO parameters and `i64` return, while call
  sites pass `%struct.Vec*` and use the declared return type -- ABI mismatch
  -> deterministic ACCESS_VIOLATION at runtime.
- **Observed IR (probe_bit.xi, both the fresh build and the committed-HEAD
  compiler):**
  ```
  %tmp183 = call i64 @_from_twos_bits(%struct.Vec* %tmp47)   ; call site: Vec* arg
  define i64 @_from_twos_bits() {                             ; definition: NO params!
  ```
  The definition is also emitted UNQUALIFIED (`@_from_twos_bits`, missing the
  `@bigint.` module prefix) -- the external-decl registration degraded the
  signature. `_bits_to_bigint(&Vec[Int], Bool)` in the same module vanished
  from the IR entirely.
- **Scope (empirically verified):**
  - `&Vec[Int]` param in a USER module -> correct (`define i64 @sum_vec(%struct.Vec %param0)`).
  - `&Vec[UInt8]` param in a CATALOG module -> correct (smoke_compress exit 0).
  - `&BigInt` (plain struct) param in a catalog module -> correct (`@bigint._twos_bits(%struct.BigInt*, i64)`).
  - OLD compiler (before d22068f8): `bigint_bit_and` worked (probe_bisect #9,
    exit 0) -- so this is a REGRESSION from the committed &T-param fix
    (param_llvm_type struct-address lowering + coerce changes), not a
    pre-existing issue.
- **Trigger in stdlib:** `stdlib/xiom/bigint.xi` `_from_twos_bits(bits: &Vec[Int])`
  and `_bits_to_bigint(bits: &Vec[Int], negative: Bool)` -- the two's-complement
  bitwise helpers. The extended `smoke_bigint.xi` crashes (0xC0000005) the
  moment any of `bigint_bit_and/or/xor` is linked in.
- **Likely root cause hint:** the recurring `warning: unknown type 'Int]' --
  defaulting to i64` (type-string parser splitting `Vec[Int]` on `]`) combined
  with the new param lowering -- the mangled `&Vec[Int]` param type resolves to
  an empty/i64 default during catalog decl registration. The catalog
  registration path (collect_external_decls) is what differs from the
  in-program path (which works).
- **Fix direction (compiler):** catalog/external-decl signature extraction for
  `&Vec[T]` params must preserve the container type (and the module prefix on
  the emitted fn name). Verify with: probe_bit.xi (bit ops), smoke_bigint.xi
  (extended), and re-run smoke_compress (must stay green -- &Vec[UInt8] is the
  canary for the working path).

### FIXED in stdlib during diagnosis (no compiler involvement)

- `bigint.xi` div_mod zero-remainder: `dm.1.digits[0]` is an out-of-bounds
  read when the remainder is zero (div_mod returns an EMPTY digits Vec for a
  zero remainder -- `_trim` pops all limbs; both the old and new paths do
  this). All readers (`bigint_to_base`, `bigint_to_hex`, `_bigint_bit_array`)
  now guard with `bigint_is_zero(&dm.1)` first. Pre-existing hazard, exposed
  by the new m==1 schoolbook path.
- `smoke_bigint.xi` assertion for `bigint_shift_left`: the function is the
  original DECIMAL shift (x10^n), not a bit shift -- assertion corrected to
  `shift_left(1, 3) == "1000"` (documented in the module header).

---

### NOTE 6 -- `stdlib_exec_bigint_runs` fails (smoke_bigint.xi compile hangs/fails) -- PARALLEL-SESSION IN-FLIGHT, NOT A COMPILER REGRESSION

- **Observed:** `.\test_summary.ps1 -Fast` (threads 32) reports a 4th
  stdlib-exec failure: `stdlib_exec_bigint_runs` ("compile failed for
  examples\stdlib_smoke\smoke_bigint.xi"). Manual `xiom -o bigint_check.exe
  smoke_bigint.xi` does not return within 60s (compiler appears to hang) -- the
  harness reported a compile failure rather than a hang, so behavior is racy.
- **Root cause (stdlib, not compiler):** `stdlib/xiom/bigint.xi` (+590 lines)
  and `examples/stdlib_smoke/smoke_bigint.xi` (+198 lines) carry LARGE
  uncommitted in-flight edits from the parallel BigInt/BigFloat stdlib session
  (adds `bigint_from_u64`, and `use xiom.num;` / `xiom.core.INT_MAX` /
  `xiom.core.to_int_from_char`). These helpers are mid-introduction and not yet
  self-consistent, which drives the compiler down a pathological/infinite
  path. Likely related to existing BUG 1 (opaque tuple-of-struct LLVM type).
- **Action:** NOT a compiler regression from this session's warning cleanup.
  Compiler-hardening session left it untouched (owned by the parallel stdlib
  session). Re-verify once the parallel session lands its BigInt refactor and
  the helpers exist; if it still hangs on a CONSISTENT stdlib, re-open as a
  compiler bug (likely BUG 1 area).

---

## 2026-08-11 -- Compiler session follow-up: BUG 8 verified FIXED, 3 new findings

### BUG 8 -- verified RESOLVED on the current tree (commit pending)

Reproduced with a fresh probe (`probe_bit.xi`: `bigint_bit_and(12, 10)`) --
the current compiler emits CORRECT signatures for catalog fns with
`&Vec[Int]` params:

```
define %struct.BigInt @bigint._from_twos_bits(%struct.Vec %param0) inlinehint {
define %struct.BigInt @bigint._bits_to_bigint(%struct.Vec %param0, i64 %param1) ...
```

and `probe_bit` compiles + runs (12&10=8). The empty-signature stub the
stdlib session observed came from `emit_undefined_symbol_stubs` filling in
for a fn that was never emitted -- the emission failure was the earlier
uncommitted coerce.rs state (now reverted/committed). Locked in with e2e:
`e2e_m37_catfix_catalog_vecref` (examples/catfix: imported module with
`&Vec[Int]` + `&struct` params).

### FIXED -- Parser: `bits[L - 1]` (index with arithmetic on an uppercase ident)

- **Construct:** `if bits[L - 1] == 0 { ... }` -- the postfix `[` arm's
  "explicit generic call args" heuristic (lowercase base + UPPERCASE first
  token in brackets) eagerly parsed `L` as a TYPE argument and errored
  "expected ']', found -" -- it only backtracks when `]` is followed by `(`,
  but the error fired before reaching that check. Also broke catalog files:
  `stdlib/xiom/bigint.xi` `_from_twos_bits` and `stdlib/xiom/num/bigfloat.xi`.
- **Fix (crates/xiom-parser/src/lib.rs):** speculative scan to the matching
  `]` (bracket-depth aware, Eof-guarded); commit to generic-args ONLY when
  the group is immediately followed by `(`. Index expressions (`bits[L-1]`,
  `buf[Head]`) fall through to normal index parsing.
- **Verified:** probe_idx (bits[L-1] evaluates correctly), generic calls
  (`add2[Float32](...)`) still parse; e2e `e2e_m37_index_arith`.

### FIXED -- Catalog import pathological slowness ("hang" on `use xiom.math`)

- **Construct:** importing modules whose transitive imports contain LEAF
  references (`use xiom.core.to_int` -- a fn/const, not a module file) made
  `xiom --check` take minutes-to-infinite. Root cause: `ModuleCatalog::
  load_module`'s Strategy-b scan-based fallback walks the ENTIRE source
  directory tree (reading every .xi header) for EVERY failed lookup -- and
  source_dirs include the file's parent dir + project root (and the CWD,
  which can be a temp dir containing a full worktree + target/). The
  pre-built `module_index` (build_index) already covers every file the scan
  could find, so the scan is pure waste in the normal flow.
- **Fix (crates/xiom-check/src/catalog.rs):** (1) skip the scan entirely when
  `module_index` is non-empty (index is authoritative; scan retained only for
  catalogs built without `build_index`, e.g. unit tests); (2) `index_dir` /
  `load_from_dir` / scan recursion skip build/VCS/package-manager dirs
  (`target`, `build`, `.git`, `node_modules`, any `.`-prefixed dir).
- **Measured:** `use xiom.math` check 20s->9.6s (scan eliminated; residual
  ~4-9s is prelude module parsing); `probe_bit` full compile 138-160s->9.5s.
  Locked in by e2e_m37_catfix_* (which also verify catalog resolution still
  works).

### VERIFIED -- circular imports are safe (not prevented, terminate)

- **Construct:** module A `use`s B and B `use`s A. The loader's
  `cached_loaded` guard terminates the cycle; the checker resolves both
  modules' symbols regardless of load order; compile succeeds.
- **Empirically verified** with a two-module cycle (check 4.6s, compile 8.3s,
  run correct -- the only crash was my fixture's own unbounded mutual
  recursion, expected stack overflow, not a compiler issue). The current
  stdlib has NO true module cycle (bigint->xiom.num; num/bigfloat->xiom.bigint
  don't close a loop because num.xi doesn't import bigfloat).
- Locked in by e2e `e2e_m37_catfix_circular_imports` (examples/catfix circ_*).

---

## 2026-08-11 -- M37 batch 2: BUG 9/10/11 FIXED -- fast suite 1112/1/1

### FIXED -- BUG 9: private catalog struct types degrade to i64

`collect_external_decls` injected only PUB type decls; a pub fn taking or
returning a PRIVATE struct (the stdlib's `pub IntFrac` workaround pattern)
lost the type layout -- emitted signatures degraded to i64 (`make_frac(3,4)`
summed to 0 instead of 7). Fix: transitively inject non-pub types
referenced by injected pub fn signatures (params/returns/inner types +
field types). Verified: examples/catfix b9mod+b9main; e2e
`e2e_m37_catfix_private_type`.

### FIXED -- BUG 10: float literals truncated to 6 decimals

`{:.6}` at all three literal-emission sites truncated 0.123456789 to
0.123457 (wrong stored values AND wrong comparisons). Now `{:.17e}` --
exact f64 round-trip, LLVM-valid. Also removed a leftover "CG02 DEBUG"
eprintln. Verified: e2e `e2e_m37_float_precision`.

### FIXED -- BUG 11: unsafe-extern double marshalling (complex/net_folder)

The unsafe-block round-trip family: (1) block-fn `return X` with a STRUCT
tail extracted a scalar field (Option field-1) instead of val_to_i64's
heap-pointer round-trip -> AV on re-materialization (m34_y04);
(2) `ret_from_enclosing` overwrote the shared block value with the
enclosing coercion -> `icmp eq ptr, i64` (m34_d01..d20);
(3) fault path emitted `ret i64* 0` (clang rejects) instead of `null`
(m33_u13). Fixes in expr.rs/stmt.rs. `smoke_complex.xi` and
`smoke_net_folder.xi` now PASS -- stdlib-exec is 72/72.

### TEST MIGRATIONS (confinement-era rules, not compiler regressions)

- m21_ffi_unsafe_001..009: added the T007 `requires:` contract to
  whole-body-unsafe fns (rule landed in 83416aa9 after the tests).
- m35_z12/z30: wrapped pointer-returning wrappers in unsafe (T003).
- m33_u13: rewrote the dangling-`&local` return test to be well-defined.

---

## 2026-08-11 -- BigInt/BigFloat session: Phase C (transcendentals) landed

- `bigfloat` Phase C landed (pure-XIOM series: pi/e with precision via
  Machin/Taylor, exp/ln/log10, sin/cos/tan, atan/atan2, pow_bf) --
  `smoke_bigfloat.xi` now 44 assertion blocks, exit 0 in ~2s.
- NO new compiler findings from Phase C. One stdlib coding error caught and
  fixed (atan halving identity: `1 + sqrt(1 + t^2)`, not `1 + sqrt(t^2)`).
- **BUG 11 (unsafe extern doubles) -- FIXED by the compiler session AFTER this
  entry was written** (`7f7b7b54`, 2026-08-11): `stdlib_exec_complex_runs`
  and `stdlib_exec_net_folder_runs` now PASS (stdlib-exec 72/72 -- verified
  15:0x and 15:4x). The stale "remains OPEN" claim below was written before
  the fix landed; see the STATUS SUMMARY at the top of this file.
- BUG 2/3 (module-global field writes / fn-call initializers) remain
  open; stdlib design already avoids both (whole-value global assignment;
  constants as pure constructor fns). Fixing them unlocks precision-cached
  pi/ln10 and the spec's `const BIGINT_*/BIGFLOAT_*` style.

---

## 2026-08-11 -- fmt.sprintf session: Float64 container element access broken (NEW)

### BUG 12 (NEW) -- `Vec[Float64]` element reads lower to `load i64 + sitofp` (silent corruption); `[N]Float64` fixed arrays degrade to `[N]i64` (AV)

- **Construct (both user and catalog modules):** any read of an element of a
  `Vec[Float64]` (param or local) or of a fixed `[N]Float64` array.
- **Observed:**
  1. `Vec[Float64]`: `probe_vf.xi` -- `var x = v[0];` after `v.push(3.14159)`
     -> `x != 3.14159`. IR evidence (`_sprintf_engine`, the Vec element-load
     switch): the 8-byte case emits `bitcast i8* to i64*` + `load i64`, then
     the float-typed value is produced by `sitofp i64 %tmp402 to double` --
     the f64 BIT PATTERN (4614256650576693248 for 3.14159) is treated as an
     integer and converted, not reinterprete -- `%.2f` of 3.14159 printed
     "4614256650576693248.00". The generic Vec element-load path only knows
     integer element widths (1/2/4/8) and defaults the type to i64.
  2. `[N]Float64` (incl. inside structs): `probe_fa.xi` -- warning
     `unknown type 'Float64]' -- defaulting to i64` (the type-string parser
     splits on `]`, same family as the old BUG 8), struct fields + array
     slots laid out as i64 -> 0xC0000005 ACCESS_VIOLATION reading `s.values[0]`.
  3. `Vec[Float64]` STORE path appears intact (push stores the raw bits);
     only element READS are wrong.
- **Impact on stdlib:** NO pre-existing stdlib module uses `Vec[Float64]` or
  `[N]Float64` (stats works on `Vec[Int]`) -- zero regression. The fmt.sprintf/
  sscanf batch (G13) was designed around `Vec[Float64]` and was re-designed
  to avoid the construct: scalar `Float64` params (proven -- bigfloat/geom
  pass doubles everywhere) and a fixed-slot `FloatScan` struct for sscanf
  float results (`// TODO(compiler)` note in fmt.xi). Float64 in structs as
  plain fields (not arrays) is proven fine (geom Vec2).
- **Fix direction (compiler):** (a) the Vec element-load switch must load
  `double` (and `float`/`fp128`) for float element types instead of i64+sitofp
  -- the element type should come from the Vec's registered type, not the
  width; (b) the type-string parser must not split `Float64]`/`UInt8]` on the
  first `]` (parse the full `Vec[T]`/`[N]T` with bracket depth) -- same root
  cause family as BUG 8's `&Vec[Int]` empty-signature bug. Verify with
   probe_vf.xi (expect exit 0) and probe_fa.xi (expect exit 0).

### BUG 13 (NEW) -- fp128 (Float128) arithmetic hits missing compiler-rt helpers at link time

- **Construct:** any program whose Float128 value flows through i64->f128
  (sitofp), f128->f64 (fptrunc), or f128 division. `probe_f128.xi` --
  `n as Float128`, `x as Float64`, `acc / ten` -> lld-link errors:
  ```
  undefined symbol: __floatditf   (sitofp i64 -> fp128)
  undefined symbol: __trunctfdf2  (fptrunc fp128 -> f64)
  undefined symbol: __divtf3      (fdiv fp128)
  ```
  fadd/fmul/fpext on fp128 are native (x87) and link fine; the compiler-rt
  soft-float helpers are not in the link line.
- **Impact on stdlib:** blocks the planned `bigfloat_to_float128` bridge
  (256-bit framing mission). Int128/UInt128 are UNAFFECTED (native LLVM i128
  -- no helpers) so `bigint_to_i128/u128/u64` landed. A lossy
  `bigfloat_to_float64 -> fpext` wrapper was rejected (only 15 digits -- the
  whole point of f128 is 34). TODO(compiler) note in stdlib/xiom/num/bigfloat.xi.
- **Fix direction (compiler):** link compiler-rt (clang `-rtlib=compiler-rt`
  or add libclang_rt.builtins) on Windows, or emit/implement the handful of
  `__*tf3`/`__floatditf`/`__trunctfdf2` stubs; verify with probe_f128.xi
  (expect exit 0).

### BUG 14 (NEW) -- UInt64->UInt128 cast emits SEXT; UInt128 `>>` emits ASHR

- **Construct:** `v as UInt128` with `v: UInt64` whose bit 63 is set, and
  `(p >> 64)` on a `UInt128` whose bit 127 is set. `probe_m128.xi` --
  `(lhs as UInt128) * (rhs as UInt128)` for lhs/rhs with bit 63 set gives the
  WRONG product. IR evidence:
  ```
  %tmp6 = sext i64 %tmp5 to i128     ; must be zext for UInt64
  %tmp18 = ashr i128 %tmp17, %tmp19  ; must be lshr for UInt128
  ```
  (The compiler has no unsigned 128-bit type distinction at codegen -- UInt64->
  UInt128 and Int64->Int128 both lower to sext; UInt128 `>>` lowers to ashr.)
- **Impact on stdlib:** XXH3's 64x64->128 mulhi (`XXH3_mul128_fold64`) was
  wrong on inputs with the top bit set (verified: seed-0 64-bit vectors
  passed for short inputs only after the fix). Worked around in
  `hash/xxhash.xi`: `_u64_to_u128` builds the i128 from 32-bit halves (sext
  == zext for bit-63-clear values) and the high half is masked after `>>`.
  `bigint_to_u128`/`to_i128` are unaffected (limbs < 1e9 and shifts of
  bit-63-clear values only).
- **Fix direction (compiler):** lower `UInt64 as UInt128` with `zext` (type
  information exists in the checker) and `UInt128 >>` with `lshr`; verify
  with probe_m128.xi (expect exit 0).

### BUG 15 (NEW) -- alwaysinline bodies with a single `var` + `return` drop the mask statement on inline

- **Construct:** a small fn (<= inline-threshold) whose body is exactly
  `var mask = <expr>; return <expr2> & mask;` -- e.g.
  ```
  fn shr(x: UInt64, k: Int) -> UInt64 {
    var mask: UInt64 = ((1 as UInt64) << (64 - k)) - 1;
    return (x >> k) & mask;
  }
  ```
  Inlined call sites emit ONLY the `ashr` -- the mask `shl`/`sub` and the
  `and` are dropped (probe_sip3.xi: `shr(t, 32)` returns the sign-extended
  value; IR shows `%tmp6 = ashr i64 %tmp4, 32` with no following `and`).
  Adding a second var (`var shift = 64 - k; var mask = ...;`) makes the
  inline correct (the pattern hash.xi `_rotl64` has always used).
- **Impact on stdlib:** SipHash/XXH3 logical shifts were wrong until the
  two-var form was used. All new hash code uses the two-var pattern with a
  comment. NOT worked around in old code -- hash.xi `_rotl64` (two vars) is
  unaffected.
- **Fix direction (compiler):** the inline expansion drops statements from
  single-var bodies (likely a statement-copy bug in the inline pass); verify
  with probe_sip3.xi (single-var shr must equal two-var shr, exit 0).

### BUG 16 (NEW) -- combining collect.skiplist + collect.trie fast-fails with 0xC0000409 (stack cookie)

- **Construct:** link BOTH `stdlib/xiom/collect/skiplist.xi` and
  `stdlib/xiom/collect/trie.xi` into one program and run ANY skiplist fn
  (even `skiplist_insert`) plus `trie_new` -- the program prints normally
  then dies at exit with 0xC0000409 (STATUS_STACK_BUFFER_OVERRUN -- the /GS
  cookie check on main's frame fires after `return`). Repros:
  `probe_pt2.xi` (two inserts, no io) and `probe_pt3.xi` (insert + trie_new).
  Each module alone, and every other pair (skiplistx{string,convert,char,
  cuckoo,fenwick,objectpool,queue,cache}; triex{cuckoo,fenwick,objectpool,
  queue,cache}) exits clean.
- **Observed in IR (both modules combined):** generated symbols emitted
  UNQUALIFIED: `define %struct.MaybeUninit @MaybeUninit.clone(...)` and
  `define void @BinaryHeap.invariant_check(...)` / `@BufReader.invariant_
  check` / `@Cursor.invariant_check` -- no module prefix, while the same
  programs alone show the same stubs (benign alone). Two Option payload
  instantiations exist in the pair (Option[Int] in skiplist, Option[Char]
  via trie->string.char_at) -- the shared MaybeUninit/clone codegen path is
  the prime suspect (clone body `ret %struct.MaybeUninit %self` with a
  16-byte layout vs possibly 8-byte Char payload -> stack corruption).
- **Stdlib handling:** both modules are correct individually; the CI/exec
  harness must NOT combine them in one smoke -- the batch's smokes are
  split (smoke_collect2a: skiplist+cuckoo+fenwick+pool+spsc+arc;
  smoke_collect2b: trie+cuckoo+fenwick+pool+spsc+arc). TODO(compiler) notes
  in both modules.
- **Fix direction (compiler):** qualify generated clone/invariant-check
  symbols per module and make the MaybeUninit clone emit payload-accurate
  code for each Option payload type; verify probe_pt2.xi (expect exit 0).

### BUG 17 (NEW) -- runtime-runtime `Str ==` on Vec[Str] ELEMENTS lowers to pointer compare

- **Construct:** comparing two runtime `Str` values that come from `Vec[Str]`
  ELEMENT loads, e.g. `ga[i] == gb[j]` where `ga`/`gb` are `Vec[Str]`:
  probe_jac2.xi -- both elements are "he" yet `==` is false; `ga[0] == "he"`
  (literal) is true. The emitted IR for the element-element comparison
  contains NO `strcmp` call (the element loads degrade the operand type so
  the equality lowers to pointer icmp), while plain runtime `Str == Str`
  (vars/slices, probe_seq/probe_seq4) DOES emit `strcmp`.
- **Impact on stdlib:** `text.similarity.jaccard_similarity` and the
  lcp/lcsuffix helpers compared slices -- all now use a byte-wise `_str_eq`
  helper (documented) so string content comparison never depends on the
  degraded path. This is the same family as BUG 12 (Vec element type
  degradation): Float64 elements load as i64+sitofp, Str elements lose the
  content-equality lowering.
- **Fix direction (compiler):** the Vec element-load expression must carry
  the element's declared type (Str -> strcmp on `==`; Float64 -> `load
  double`); verify probe_jac2.xi (both comparisons true, exit 0).

### BUG 18 (NEW) -- combining string + text.similarity + time in one program crashes (0xC0000405)

- **Construct:** one program importing `xiom.string`, `xiom.text.similarity`
  AND `xiom.time` (smoke_str2.xi) crashes at startup with 0xC0000405 before
  any output. Every PAIR of the three exits clean; each module alone is
  clean. Same family as BUG 16 (combination-specific startup crash -- likely
  the unqualified `@MaybeUninit.clone`/invariant-check stubs colliding when
  several Option payload shapes coexist).
- **Refined trigger (time-only programs):** with only `xiom.time` imported,
  `strptime("2026-13-01", "%Y-%m-%d")` followed by `strptime("2026-08-11",
  "%Y-%m-%d %Q")` in the SAME program crashes (0xC0000405); the same specs
  individually, or any other spec pair, exit clean. The unsupported-
  conversion early-return path (`%Q`) after a range-check failure path
  miscompiles at -O2 (probe_tm9: the pair fails; both single calls pass).
- **Stdlib handling:** the batch's smokes are split -- smoke_str2.xi covers
  string+text.similarity (proven pair), smoke_time2.xi covers time with the
  `%Q` check removed (the fn is correct -- verified by single-call probes;
  TODO(compiler) notes in the affected modules).
- **Fix direction (compiler):** same as BUG 16 -- module-qualify generated
  symbols and emit payload-accurate clone/invariant code; verify
  smoke_str2.xi recombined (expect exit 0).

## 2026-08-11 -- STATUS SUMMARY: BUG 12-18 all FIXED (commits `2ae300fd`, `3b8f5415`)

| Bug | Fix | Verified |
|-----|-----|----------|
| 12/17 | `vec_elem_from_type_annotation` (types.rs + lib.rs) unwraps Ref/MutRef/Ptr; `Vec(...)` AST form handled -- Vec element loads keep Float64/Str types | m12b, m37_vec_f64 R=0 |
| 13 | fp128 coercion arms in `coerce_value` + NEW `stdlib/runtime/fp128_helpers.c` soft-float add/sub/mul/div/conv/cmp (verified in a C harness: 400/5/2500/1002.5, negatives, tiny values) | m13b/m13d, m37_f128 R=0 |
| 14 | cast site uses `xiom_type_of_local` (registered type); `expr_is_unsigned()` picks lshr; var bindings infer type from `as UInt*` targets | m14, m37_u128 R=0 |
| 15 | math-builtin `shr`/`shl` intercept restricted to `math.*`/`xiom.math.*` qualified keys + bare keys with NO registered fn | m15, m37_shr_builtin R=0 |
| 16/18 | `process_use` longest-dotted-prefix walk (directory submodules); `bare_fn_aliases` prefers the CALLER's module; **fn-key fix**: call resolution returns the BARE key when a bare definition exists, qualifying to caller-module/alias only otherwise -- kills the definition-vs-call symbol mismatch (user fns emit bare `@mk_big`, calls resolved to leaf-qualified zero-param stubs -> ABI crash, same family as BUG 8) | m16a-e (bare/leaf forms) + m16f (qualified form) all R=0; skiplist+trie combined OK |

- **Encoding repair:** `skiplist.xi` + `trie.xi` contained invalid UTF-8 (lone
  0x97 bytes) which silently broke import binding -- repaired.
- **Regression sweep** (isolated binary, `tgt_iso`): m37_tuple_struct,
  m37_ref_mut, m37_index_arith, m37_vec_f64, m37_u128, m37_f128,
  m37_float_precision, m37_shr_builtin, m33_z14, m34_y04, m33_u13,
  m19_read_file -- **12/12 R=0**.
- **Known follow-up (parallel stdlib session owns it):** the `collect/` ->
  `collections/` folder move landed while module declarations inside still
  say `xiom.collect.*` -- `use` resolves by file path (works) but
  fully-qualified calls need the declared name (`xiom.collect.skiplist.fn`
  works; `xiom.collections.skiplist.fn` does not until the declarations are
  aligned). Not a compiler regression; m16b-e/m16f verified green.
- Workspace build gate: **zero warnings** (dead `load_external_module` +
  unused imports removed with the XIOM_TRACE_* debug prints).

---

## 2026-08-11 [U+FFFD] stdlib session (evening): BUG 19 (NEW) [U+FFFD] every NaN-producing Float64 operation returns a garbage sentinel or traps

**? FIXED 2026-08-11 (commit `9c3a2f9e`).** Two codegen defects, both verified:

1. **float `!=` lowered to `fcmp one`** (ordered-not-equal) [U+FFFD] for NaN operands
   `one` is FALSE, so `x != x` returned false and NaN was undetectable.
   Fixed to `fcmp une` in both the BinOp table (expr.rs) and the trait-method
   table (call.rs `.ne()`).
2. **`Str + Float64` concat inttoptr'd the FP bits** [U+FFFD] the "garbage sentinel
   print" was this, not the fdiv (the IR fdiv was always correct). Fixed via
   new `@xiom_double_to_string` (xiom_runtime.c): NaN ? "nan", [U+FFFD]inf ?
   "inf"/"-inf", finite values shortest-round-trip (%.15g else %.17g);
   declare added to emitter.rs so the undefined-symbol stub pass can't emit
   a conflicting zero-param definition (this WAS the 0xC000001D crash path).
   Float32 concat widens via fpext first.

Verified: `m37_nan_ieee.xi` (23 checks: 0/0, inf*0, inf?inf ? NaN; `x != x`
true; ordering-with-NaN all false; Float32 NaN; concat "nan"/"inf"/"-inf"/
"3.14"/"0"/"0.5") R=0; original probe prints "c = nan / is NaN" exit 0;
17/17 regression sweep + stdlib-compile 40/40 + checker 178/178 green.

- **Construct:** any IEEE-754 NaN-producing Float64 operation: `0.0 / 0.0`, `inf - inf`, `inf * 0.0` (with `inf` from `1.0 / 0.0` or `-1.0 / 0.0`). Verified on a fresh build from HEAD (incl. `2ae300fd` + `3b8f5415` + `f0388644`; `cargo rustc -p xiom --bin xiom -- -o <temp>\xiom.exe`).
- **Observed:**
  1. `var a = 0.0; var b = 0.0; var c = a / b;` ? `c` prints `-92233.-72036854775808` and `c != c` is **false** (not NaN). Same sentinel for `inf * 0.0` and `inf - inf`.
  2. `math.ln_pure(-1.0)` (pre-existing stdlib path `if x <= 0.0 { return 0.0 / 0.0; }`, math.xi:162) ? process dies with **0xC000001D** (STATUS_ILLEGAL_INSTRUCTION) [U+FFFD] the runtime fault-trap fires on the NaN-producing division.
  3. `1.0 / 0.0` ? +inf and `-1.0 / 0.0` ? -inf are **correct** (inf results pass; only NaN results are broken).
- **IR evidence (probe_nan.xi):** the emitted IR is correct [U+FFFD] `%tmp41 = fdiv double 0.00000000000000000e0, 0.00000000000000000e0` [U+FFFD] so the corruption happens in the clang/optimize/runtime-trap stage, not in AST emission. The consistent garbage value (`-92233.72036854775808` [U+FFFD] a sentinel) suggests the fault-trap/intercept layer (the same system as smoke_guard_fault.xi) replaces NaN-producing FP ops with a trap-or-sentinel path instead of the IEEE result.
- **Impact on stdlib:** `math.constants` NAN cannot be implemented (no literal syntax; `0.0/0.0` broken) [U+FFFD] `// TODO(compiler): BUG 19` in math/constants.xi; `math.is_nan`/`is_inf` classify correctly but nothing in the stdlib can PRODUCE a NaN today (all NaN-producing libm entries [U+FFFD] asin/acos/ln/sqrt [U+FFFD] carry domain `requires:` contracts). `num/float.xi` `bits_to_float` (planned) needs a real bitcast intrinsic to land anyway.
- **Fix direction (compiler):** route NaN results (fdiv 0/0, fsub inf-inf, fmul inf*0, and libm domain-error returns) through the IEEE path [U+FFFD] do not trap/sentinel float NaN; optionally add a `nan` literal or i64?f64 bitcast intrinsic (`bitcast i64 0x7FF8000000000000 to double`) which would unblock `math.constants.NAN` + `num.float.bits_to_float`. Verify with probe_nan2/probe_nan3 (expect `nan` print + `x != x` true, exit 0).

**Stdlib unblock:** `math.constants.NAN` can now be implemented as
`pub fn nan() -> Float64 { return 0.0 / 0.0; }` (a const initializer can't hold
the expression yet [U+FFFD] const-fold only handles literals; a runtime fn works).
`is_nan(x)` = `x != x` is now correct.

---

## 2026-08-11 (night) [U+FFFD] stdlib session: BUG 20 (NEW, REGRESSION from `1d4cd2e8`) [U+FFFD] unconditional -mavx512* clang flags crash non-AVX-512 CPUs (illegal instruction) in ANY vectorized program

**? FIXED 2026-08-12 (commit `4fa7a1d2`).** AVX-512 flags are now HOST-CPUID-gated:
`-mavx512f/bw/dq/vl` are added only when
`std::arch::is_x86_feature_detected!("avx512f")` (crates/xiom/src/lib.rs). The
-O2 vectorizer emits zmm in ordinary float loops; the runtime CPUID dispatch
gates only INTENTIONAL SIMD calls, so the FLAGS must match the host.
`-maes -mavx -mavx2` remain unconditional (safe on any AVX2 CPU). Verified on
Zen 2: clang arg trace shows no `-mavx512*`; probe_avx.xi exit 0;
smoke_num_fraction/smoke_math_rounding/smoke_math_precision/
smoke_math_trig_constants/smoke_num_float all R=0 (were 0xC000001D).
`smoke_num_precision` STILL AV-crashes (0xC0000005, deterministic) [U+FFFD] a
separate BUG 22/23 item (cross-module returned Vec[Float64] / nested Vec
family), queued with the wave batches.

- **Construct:** commit `1d4cd2e8` adds `-maes -mavx -mavx2 -mavx512f -mavx512bw -mavx512dq -mavx512vl` to every clang invocation on `Target::Native && x86_64` (crates/xiom/src/lib.rs:1049-1057). On a CPU WITHOUT AVX-512 the -O2 vectorizer can emit AVX-512 instructions in ordinary float loops ? 0xC000001D (STATUS_ILLEGAL_INSTRUCTION) at runtime. The CPUID dispatch in simd_runtime.c gates only the INTENTIONAL SIMD calls; it cannot gate the vectorizer.
- **Repro (minimal, probe_avx.xi):**
  ```xiom
  use xiom.io; use xiom.convert;
  fn main() -> Int {
    var acc = 0.0; var i = 0;
    while i < 64 { acc = acc + (i as Float64) * 0.5; i = i + 1; }
    io.println(convert.float_to_string(acc));
    return 0;
  }
  ```
  ? compiles clean, then **exit -1073741795 (0xC000001D)** with zero output. Also affects: `smoke_num_fraction.xi`, `smoke_math_rounding.xi` (and every float-loop stdlib smoke) on this machine (AMD Ryzen 9 3950X = Zen 2 [U+FFFD] **no AVX-512**). Pure-int/string programs (e.g. `smoke_string_case.xi`) run fine.
- **Impact:** blocks stdlib float-smoke verification on non-AVX-512 machines; any user program with a float hot loop crashes on such CPUs. The v0.58 comment's own rule ("only dispatch-gated code paths may rely on AVX-512 presence") is unenforceable for the vectorizer [U+FFFD] the FLAGS themselves must be gated.
- **Fix direction:** gate the flags on the HOST's CPUID at build time (query AVX-512 support before adding -mavx512*; e.g. use `-march=native` which enables only what the host supports), or drop the 512-bit flags (keep -maes -mavx -mavx2, safe on any AVX2 CPU). Verify: probe_avx.xi exit 0 on Zen 2; e2e SIMD test still R=0.

---

## 2026-08-11 (night) [U+FFFD] stdlib session: BUG 21 (NEW) [U+FFFD] catalog-module fn returning a Str created INSIDE an unsafe block returns a corrupted Str (len 0xFFFFFFFF)

**Status 2026-08-12: NOT REPRODUCED on the current build.** All documented
shapes verified PASSING with the isolated binary: catalog fn with (a) loop
inside unsafe + `return Str.from_cstring(buf)` from inside the block ?
"aaa" R=0; (b) direct return from inside the block ? "xyz" R=0; (c) while
loop + `xiom.string.str_concat` build inside unsafe + return ? "aaa" R=0.
`smoke_string_pad_repeat` R=0. The original `str_repeat_char` shape was
restructured away before the fix could be isolated [U+FFFD] likely resolved by the
accumulated batch (`f0388644` chain/process_use + `9c3a2f9e` BUG 19 + `2ae300fd`
fn-key). The stdlib's workaround (no loop inside unsafe) can stay. If the
EXACT original file still fails, send it and it becomes a live repro.

- **Construct:** an IMPORTED (catalog) stdlib module fn whose body creates a Str inside an `unsafe` block and RETURNS it from inside that block, e.g. `stdlib/xiom/string/repeat.xi` `str_repeat_char` (was: `unsafe { ...; return Str.from_cstring(buf); }`). The unsafe-confinement trampoline (`__unsafe_ctx`) round-trips only i64-class values; the Str (ptr+len struct) return corrupts the length field ? `len()` returns 4294967295 and any strcmp on the value crashes (0x80000003 breakpoint [U+FFFD] heap guard).
- **Verified:** probe_repeat/probe_rep2 [U+FFFD] `repeat.str_repeat_char('a', 3)` prints `[]` with `len=4294967295`; the smoke's `str_repeat_char(...) != "aaa"` comparison then dies 0x80000003 with all buffered output lost. The SAME shape in a USER module (`fn mk_a() -> Str { unsafe { ...; return Str.from_cstring(buf); } }` with T007 `requires: true`) works [U+FFFD] so it is the catalog/trampoline path, not from_cstring. The flat string.xi str_pad_left/str_pad_right build strings inside unsafe but return OUTSIDE the block (assign-var-in-unsafe, return after) [U+FFFD] that shape is correct, which is why the bug was never hit before.
- **Fix direction (compiler):** the unsafe-block return marshalling for catalog fns must round-trip Str (and other 16-byte struct) returns like the BUG 11 Option fix did [U+FFFD] verify repeat.xi's `str_repeat_char` via its smoke once fixed; alternatively reject `return <struct-typed>` from inside unsafe blocks with a clear error.
- **Stdlib handling:** repeat.xi restructured to the proven shape (assign inside unsafe, return outside) + `// TODO(compiler): BUG 21` note. The `str_repeat_char(...) != "aaa"` comparison is the crash trigger [U+FFFD] the corrupted value must never reach a comparison.

---

## 2026-08-11 (night) [U+FFFD] stdlib session: BUG 22 (NEW, batch report) [U+FFFD] implementation-phase findings from 4 parallel agents (detailed repros below; all pre-existing or new-shape; none blocked the batches)

**? RESOLVED 2026-08-12 (commits `eeab8cf7`, `e6608946`, `47cd188f`, `f74a52c7`):**
1. `&&`/`||` short-circuit [U+FFFD] **FIXED** (branch on LHS; RHS compiled only when needed; verified div-by-zero RHS never executes) [U+FFFD] m37_short_circuit
2. Unary minus on match-bound vars [U+FFFD] **FIXED** (negation defers wildcards like Not; float payloads bitcast from the i64 slot) [U+FFFD] m37_match_float_payload
3. Cross-module 3-tuple `.1`/`.2` [U+FFFD] **FIXED** (tuple field maps derive+register for catalog returns) [U+FFFD] m37_catalog_boundary
4. Cross-module Option payload match [U+FFFD] **FIXED** (float payloads via local_opt_payload_xiom; Int payloads verified) [U+FFFD] m37_catalog_boundary
5. requires/ensures trap [U+FFFD] **FIXED** (clean `xiom_panic`: message to stderr + flush + exit 1, no more 0xC000001D) [U+FFFD] m37_contract_pass + manual violation probe
6. str_reverse invalid IR [U+FFFD] **FIXED 2026-08-12** (root: unsafe-block captures grabbed SHADOWED names [U+FFFD] the block's inner `let c` shadowed an enclosing loop's `let c`, capturing the outer loop-body alloca whose address doesn't dominate; fix: block-bound names excluded from captures + loop-body binding allocas hoisted to fn entry). Flat string/string.xi str_reverse verified: compiles, returns "cba"; m37_loop_capture regression [U+FFFD] the stdlib's reverse.xi workaround can now delegate
7. index_of(str_slice) 0xC0000409 [U+FFFD] stdlib-side scan workaround in place; not reproduced standalone (see 22.15)
8. xiom_char_at leading-byte [U+FFFD] DOCUMENTED CONTRACT (byte position), stdlib decodes UTF-8 manually [U+FFFD] no change
9. Multi-byte char literals [U+FFFD] **VERIFIED FIXED** on current build ([U+FFFD]=233, ?=937)
10. byte_at sign-extend [U+FFFD] **VERIFIED FIXED** on current build (UInt8 as Int zexts)
11. Module-qualified call results inline [U+FFFD] **FIXED** (ROOT: qualified-call symbol vs bare def mismatch ? fn-symbol PRE-ASSIGNMENT map + integer verdicts for inline calls/index/binary operands) [U+FFFD] m37_inline_call_concat
12. Vec[Char] element size [U+FFFD] **FIXED by the BUG 23 #2 nested-Vec work** (elem sizes now resolve from the registered element type; Char elements read via the elem_size switch)


1. **`&&` does not short-circuit** (num/fraction.xi `fraction_from_float`): both operands evaluate; a div-by-zero in the RHS traps 0xC000001D even when the LHS is false. Use nested `if`s. (Suggested fix: proper short-circuit lowering or reject non-short-circuit semantics.)
2. **Unary minus on match-bound vars fails** [U+FFFD] `error[T001]: cannot negate type _` for `-d` where `d` is bound in a match arm. Workaround: type-annotate the binding.
3. **Cross-module 3-tuple field access `.1`/`.2` fails** (math/arithmetic.xi `gcd_extended` returns (Int,Int,Int)): `240 * t.1` ? "right operand must be numeric, found <error>" [U+FFFD] `.0` works, 2-tuples fine. Comparison/return contexts work.
4. **Cross-module `match` on `Option[Int]` binds a garbage payload** (math/arithmetic.xi `mod_inverse`): `Some(v) => v != 5` misbehaves; `is_some()/unwrap()` exact. Same family as the old Option-tail extraction bug.
5. **`requires:`/`ensures:` are runtime-enforced and TRAP on violation (0xC0000005/0xC000001D)** [U+FFFD] even when the fn body guards the case. Stdlib avoids declaring contracts on fns with graceful fallbacks (documented in module headers); fix direction: contracts should be checked/elided consistently (or only in debug builds), or stdlib keeps fallback-first bodies.
6. **flat `string.str_reverse` emits invalid LLVM IR** (`Instruction does not dominate all uses!` [U+FFFD] alloca in loop body captured by an enclosing unsafe context struct). Blocks the `str_reverse` API name: any program calling `string.str_reverse` fails to compile. reverse.xi implements locally; the flat fn needs the loop-body alloca moved out (compiler session's string.xi is a shared file [U+FFFD] needs care).
7. **`string.index_of(str_slice(...), needle)` inside a loop crashes (0xC0000409)** [U+FFFD] passing a str_slice result to index_of repeatedly corrupts; replace.xi scans byte-wise instead.
8. **`xiom_char_at(s, pos)` returns only the leading BYTE** of a multi-byte char (195 for [U+FFFD], not 233) [U+FFFD] pre-existing flat string.xi/char.xi contract: byte position, not code point. String sublibs do manual UTF-8 decode from byte_at.
9. **Multi-byte char literals are mangled to their last byte**: `'[U+FFFD]' != 'O'` is false (both ? 0xA9). Workaround: build chars via to_char(cp) in tests.
10. **`byte_at(...) as Int` sign-extends UInt8** (0xC3 ? -61): must mask `& 0xFF` when handling bytes >= 0x80.
11. **Module-qualified call results used INLINE in arithmetic miscompile** (e.g. `i = i + char.len_utf8(ch)` advances by 1 instead of 2): bind the result to a `let` var first (agent-applied repo-wide).
12. **`Vec[Char]` element size is 1 byte** (BUG 12 family): storing code point 937 (0x3A9) reads back 0xA9. str_code_points returns Vec[Int] instead.

---

## 2026-08-11 (night) [U+FFFD] stdlib session: BUG 23 (NEW, wave-2 batch) [U+FFFD] findings from 4 parallel agents (string metrics/unicode, math number-theory/linear)

**? RESOLVED 2026-08-12 (commits `eeab8cf7`, `e6608946`, `47cd188f`, `f74a52c7`):**
1. Cross-module returned Vec[Float64] reads [U+FFFD] **FIXED** (var-bindings inherit the callee's declared element type) [U+FFFD] m37_catalog_boundary
2. Nested Vec[Vec[T]] garbage [U+FFFD] **FIXED** (parser type-arg rendering, elem size 32, memcpy reads, nested float inner reads) [U+FFFD] m37_nested_vec
3. Fn-value params named add/mul [U+FFFD] **VERIFIED FIXED** on current build (fn-typed params dispatch correctly)
4. `else if` parser rejection [U+FFFD] **FIXED** [U+FFFD] m37_else_if
5. Flaky `use of undefined value` (~50%) [U+FFFD] **FIXED** (catalog decl injection now deterministic: all_cached sorted; plus the fn-symbol pre-assignment kills the order-dependent symbol class) [U+FFFD] 8/8 deterministic compiles
6. UTF-8 BOM breaks registration [U+FFFD] **FIXED** (lexer strips leading BOMs) [U+FFFD] lexer unit test
7. (Bool,Bool) tuples as Tuple__Int__Int [U+FFFD] **FIXED** (param-seeded tuple scan + XIOM idents + checker field-map derivation) [U+FFFD] m37_catalog_boundary
8. Catalog `&Vec[T]` param mutation no-op [U+FFFD] **FIXED** (Ref params pass the pointer like &mut) [U+FFFD] m37_catalog_boundary
9. Unary minus on catalog-returned float [U+FFFD] **FIXED** [U+FFFD] m37_catalog_boundary
10. Subtraction on catalog-returned floats [U+FFFD] covered by the BUG 20 CPUID gating + float-type registration fixes; verify via the float smokes
11. xiom.math.sqrt bare import "requires unsafe" [U+FFFD] **VERIFIED FIXED** on current build (math.sqrt R=0)
12. Recursive helpers defeat the vectorizer [U+FFFD] NOT a compiler bug (stdlib shape choice); BUG 20's CPUID gating now lets loops vectorize on Zen 2 (AVX2); recursion remains their call


1. **Vec[Float64] element READS of a module-RETURNED Vec are still broken** (BUG 12 corner): a user program reading `v[0]` from a Vec[Float64] returned by a catalog fn gets raw bit-pattern garbage and float arithmetic on such loads traps (0xC000001D). BUG 12's fix covers Vec element loads inside the defining module; cross-module returned float Vecs are not fixed. Workaround: verify via scalar invariants (dot/norm), avoid element reads of module-returned float Vecs.
2. **Nested `Vec[Vec[T]]` element reads return garbage** (0xC0000005): `m[1].len()` wrong, `m[i][j]` = 0 for a 2-row matrix; float arithmetic on nested loads crashes. Blocks dynamic-matrix runtime verification and `set_partition` (reads Vec[Vec[Int]]).
3. **Fn-value params named `add`/`mul`/`div` collide with built-in operators**: `ring_theory(add, mul)` compiled the param calls to `tower.Int.mul`/native `*`, IGNORING the passed function (verified in IR). Rename params (op_add/op_mul) [U+FFFD] callers pass positionally so the API is unchanged. Parser/checker should reject or qualify operator-shadowing param names.
4. **Parser rejects `else if`** (`expected '{', found if`) [U+FFFD] must use nested if/else. (The `elif` keyword works; `else if` as two words does not.)
5. **Flaky `use of undefined value` compile failure** (~50%): combining modules that load `xiom.text.similarity` + spline fns with `&Vec[Float64]` params (math/numerical.xi) intermittently fails with `use of undefined value '@_tridiagonal'` (undefined-symbol-stub emission; BUG 8/16/18 family). Same source compiles on retry.
6. **A UTF-8 BOM in a catalog module silently breaks function registration** (all BOMs in the stdlib were removed/avoided; writer tools must not add BOMs).
7. **(Bool,Bool) tuples misregister as Tuple__Int__Int** (logic.xi `quantifiers`) [U+FFFD] Bool in tuples/structs degrades; workaround: encode as Int 0/1 or separate fns.
8. **Catalog `&Vec[T]` param mutation is a silent no-op** [U+FFFD] `&mut Vec[T]` is required for mutation (enumerators use `&mut`). The old struct-&T fix (d22068f8) covers struct params, not Vec params.
9. **Unary minus on a catalog-returned float in an expression traps** (0xC000001D, BUG 20 family): use `0.0 - x`.
10. **Subtraction on catalog-returned floats in smoke asserts traps** (0xC000001D): use interval comparisons (a > lo && a < hi) instead of `a - b` diffs in smokes.
11. **`xiom.math.sqrt` as a direct import cannot be called bare from a user module** (T001 "requires unsafe") [U+FFFD] the math-builtin intercept (BUG 15) only covers catalog modules; user code must call `math.sqrt`.
12. **Recursive helpers defeat the BUG 20 AVX-512 vectorizer**: range/table scans and summation loops written as recursion (one iteration per frame, depth = ~182 < 500 limit) do not get vectorized, so those modules run on Zen 2. Intended as a temporary shape until BUG 20's CPUID gating lands.

**TODO(compiler): NOT IMPLEMENTABLE stubs left by wave-2** (frozen signatures compile, bodies documented; blocked by the above): math/trig sinh/cosh/tanh/atanh (exp/ln inline arithmetic traps [U+FFFD] same shape as math/hyperbolic.xi which is also BUG-20-blocked at runtime), calculus integrate_romberg/integrate_gauss/limit/left/right/is_continuous/gradient/partial_derivative/jacobian/hessian/laplacian/curl/divergence (loops + Vec[Float64]), differential richardson/gradient/jacobian/partial_derivative, series maclaurin_series/convergence_rate, integral integrate_adaptive, set_theory set_partition (nested Vec), logic simplify/normal_forms/satisfiability/tautology_check/quantifiers (Bool/Str-returning recursion + Bool tuples). All re-verifiable once BUG 20 (CPUID-gated flags) and the BUG 23 #1/#2 (cross-module float Vec / nested Vec) fixes land.

---

## 2026-08-12 [U+FFFD] stdlib session: BUG 24 (NEW, REGRESSION from the 22/23 batch `eeab8cf7..7cfc7fe4`) [U+FFFD] bigfloat pow_bf miscompiles inconsistently (AV or hang) depending on unrelated program structure

**? PARTIAL FIX 2026-08-12 (`dd6b4ab1`):** the WRONG-VALUE symptom is fixed [U+FFFD]
root was PRE-EXISTING (not the 22/23 batch): `llvm_type_for`'s array arm
kept the trailing bracket (`[10 x Int]` ? elem name `"Int]"` ? unknown type ?
i64 degradation), corrupting bigint's `var digits: [10]Int` and every
fixed-array local. Verified: p_arr digit-extract R=0; probe pair now compiles
clean (no "unknown type" warnings).
**RESIDUAL (open):** a per-program-shape AV remains inside the pow path when
the program also compiles to_str/eq [U+FFFD] crashes before the first println with
buffered output lost; the compiled IR is verifiably sound (no stubs, no
duplicate defines, helper symbols consistent across shapes). Suspects to
continue: BigFloat struct-value passing with Vec[Int] significand fields
interacting with the 23.8 &Vec pointer-pass change; deeper helpers in the
bigger module set. Reproduction: p_powv2.xi shape (pow_bf + to_str in one
program) AVs; p_b24a.xi shape (pow_bf alone) returns a wrong value only in
the pre-fix build.

- **Construct:** any program calling `num/bigfloat.xi` `bigfloat_pow_bf(&base, &exp)` [U+FFFD] or its wrappers (`precision_float.bigfloat_pow`) [U+FFFD] on the CURRENT compiler. `smoke_bigfloat.xi` (pre-existing harness smoke, was 72/72 green) now dies 0xC0000005; `smoke_num_precision.xi` dies too. pow_bf's pieces (is_zero/is_one/is_negative/ln/mul/exp at the same operands) ALL verify correctly in isolation.
- **Decisive minimal repro pair (identical logical values, different codegen):**

  - `var t = bigfloat.bigfloat_two(); bigfloat.bigfloat_pow_bf(&t, &t);` ? prints 4, exit 0.  (works)
  - `var t = bigfloat.bigfloat_from_int(2); bigfloat.bigfloat_pow_bf(&t, &t);` ? 0xC0000005 with zero output. (AV)
  - `bigfloat_two()` is literally `return bigfloat_from_int(2);` [U+FFFD] the VALUES are identical; only the CALL STRUCTURE differs. Combining both in one program ? infinite hang (series divergence, no output).
- **Also:** `smoke_math_numerical.xi` / `smoke_math_approximation.xi` AV (0xC0000005) [U+FFFD] may be the same root (they call math.exp/ln-style paths) or the nested-Vec spline reads; both were green at agent time.
- **Suspects (in order):** (1) the fn-symbol pre-assignment map (`7cfc7fe4`) [U+FFFD] bigfloat.xi has 1380 lines of private helpers (`_one_at`, `_finish`, `_round_digits_raw`, `_atanh_series`, ...) that can land on different symbols per program shape; (2) the `23.8 &Vec[T]` pointer-pass change interacting with BigFloat's `Vec[Int]` significand FIELD copies inside struct-value passing (`bigfloat_mul(exp, &l)` passes a `&BigFloat` param BY VALUE to a `BigFloat` value param [U+FFFD] shallow Vec copy).
- **Fix direction:** reproduce with the probe pair above; check the pre-assignment map for bigfloat.xi private helpers (does `@bigfloat._finish`/`_round_digits_raw` resolve identically in both programs?); verify struct-with-Vec-field value-copy semantics unchanged by 23.8. Verification: probe pair both exit 0; smoke_bigfloat + smoke_num_precision green.
- **Stdlib handling:** no stdlib change (bigfloat.xi untouched [U+FFFD] pre-existing REAL module). The blocked smokes (smoke_bigfloat, smoke_num_precision, smoke_math_numerical, smoke_math_approximation) stay as-is pending the fix; everything else in the 61-smoke sweep passes.

---

## 2026-08-12 [U+FFFD] stdlib session: BUG 25 (NEW, wave-3 batch) [U+FFFD] findings from 3 parallel agents (collections + convert/bits)

**? COMPILER SESSION STATUS 2026-08-12 (commits `9a578313`..`271567b0`):**
1. Same-name delegation / cross-module same-name [U+FFFD] **FIXED**: ambiguous bare fns
   exported by multiple imported modules now ERROR (T001) and require a
   module-qualified call [U+FFFD] no more silent wrong-module resolution
2. `use X as alias;` [U+FFFD] **FIXED 2026-08-13**: the residual was the external-decl
   REACHABILITY filter pruning the aliased fn (main references the ALIAS name,
   not the target leaf ? the fn came back as a bare stub with an i64-return).
   Use declarations now contribute their target leaves to the referenced-name
   set. All four alias forms (module alias, fn alias, alias+qualified)
   verified R=0.
3. `from_bytes` fn name collision [U+FFFD] **FIXED** (builtin intercept now only
   fires when no real fn with the name is registered) [U+FFFD] m37_from_bytes_fn
4. Bool inside returned tuples [U+FFFD] **VERIFIED FIXED** on current build
   (covered by the BUG 23 #7 tuple-type batch) [U+FFFD] p_btup R=0
5. Option/Result `.value`/`.error` reads [U+FFFD] **FIXED** (payload-aware field
   reads: Str/Float reinterpret, boxed structs inttoptr+load) [U+FFFD]
   m37_opt_payload_value
6. Char `<`/`>` comparisons [U+FFFD] **VERIFIED FIXED** on current build (p_char_cmp
   R=0)
7. Big&big AND [U+FFFD] **VERIFIED FIXED** on current build (0xAEF1AD80 & 0x9B05688C
   = 0x8A012880)
8. `let len = v.len();` GEP family [U+FFFD] **FIXED** by #3 (the same builtin-hijack
   root)
9. Module-name vs fn-name collision [U+FFFD] DESIGN (keep-first alias rule
   documented; smokes split) [U+FFFD] no change
10. `xiom.crypto` _pkcs7_pad undefined + pure-XIOM SHA-256 [U+FFFD] **FIXED 2026-08-13**
    (see the dedicated section below [U+FFFD] root cause was a chain of 6 defects,
    incl. an unsupported `[0; 16]` array literal and a transposed AES
    add-round-key mapping; crypto module now imports, all 25 crypto smokes
    pass, AES verified against FIPS-197 Appendix B + C.2)
11. Private fns leak via `use` [U+FFFD] **FIXED** (export maps exclude private fns;
    bare-call resolution has a visibility gate: pub fns, same-module fns,
    and top-level fns only) [U+FFFD] pheap_merge probe now errors "undefined"
12. Spurious E001 borrow warnings [U+FFFD] benign (matches pre-existing), no change

## 2026-08-13 [U+FFFD] BUG 25 #10 RESOLVED: xiom.crypto unimportable (`_pkcs7_pad` link failure)

**Symptom:** `use xiom.crypto;` + any call ? clang "use of undefined value
'@_pkcs7_pad'" with a degraded `call i64 @_pkcs7_pad(i64 0)` stub; the whole
module was unusable. Root cause was a CHAIN of independent defects (each
verified with a minimal repro before fixing):

1. **Unsupported `[0; 16]` array-repeat literal** (stdlib, crypto.xi:1461):
   the parser errors "expected ']', found ;" at `var ct_buf: [16]UInt8 =
   [0; 16];`, then error-recovers by treating the REST of `aes_encrypt`'s
   body as top-level items [U+FFFD] `var padded = _pkcs7_pad(plaintext)` became a
   module-global with a BUG 3 ctor that called the never-emitted private
   `_pkcs7_pad` (i64 0 stub). Not spec syntax (AI_CONTEXT.md documents only
   `[a, b, c]` and `[]`); fixed in the stdlib with the zero-init declaration
   form. **Fix: stdlib.**
2. **`&fixed_arr[i] as *UInt8` lowered to value-load + inttoptr** (codegen,
   expr.rs Expr::Ref arm): the Ref arm handled Ident and Vec-Index but not
   FIXED-ARRAY Index [U+FFFD] `&ct_buf[0] as *UInt8` loaded the BYTE (0) and
   inttoptr'd the VALUE ? ciphertext pointer NULL ? 0xC0000005 in the AES-NI
   FFI call. **Fix: GEP element-address case for `[N x T]` slot types.**
3. **Checker rejected `&x as *T` in user modules** ("unsupported type cast:
   UInt8 to *UInt8") while catalog bodies bypassed checking entirely [U+FFFD]
   user-side FFI with the same idiom failed to compile. **Fix: reference-to-
   pointer cast rule (unsafe-gated).**
4. **Match-bound Vec payloads bound as i64 heap HANDLES** (codegen, stmt.rs
   match arm): `Ok(ciphertext) => aes_decrypt(&key, &ciphertext)` passed the
   address of the HANDLE SLOT as `%struct.Vec*` ? callee read garbage
   (len=1 vs 2; contract violations; AV). **Fix: re-materialize a real
   `%struct.Vec` local (inttoptr + load volatile) at the binding site.**
5. **Private struct types used only as LOCALS in catalog fn bodies degrade
   to i64** (checker): BUG 9's injection covers fn signatures only; aes.xi's
   AesState/StateHolder/KeyExpState (locals of private fns) never reached
   codegen [U+FFFD] the struct var became one i64 slot and constructor zero-stores
   clobbered field reads mid-construction (wrong key schedule, rcon
   contract violations). **Fix: scan injected fn BODIES for struct-literal /
   annotated-var names and run the existing transitive type walk.**
6. **Const fixed-array element reads returned the LENGTH slot** (codegen,
   expr.rs Index arm): `const _AES_SBOX: [256]UInt8 = [...]` substituted to
   an array buffer but `_AES_SBOX[i]` read buf[0]=256 as the first element [U+FFFD]
   the whole AES S-box lookup was garbage. **Fix: const-substituted
   Expr::Array idents join the array-buffer path (index+1).**
7. **aes.xi `aes_add_round_key` transposed key mapping** (stdlib): sXY read
   `rk[4X+Y]` instead of the AES column-major `rk[4Y+X]` [U+FFFD] self-consistent
   encrypt?decrypt but NOT FIPS-197 (verified with Appendix B: expected
   69c4e0d8..., got 37f0d10c...). **Fix: correct index mapping.**
8. **AES-NI C decrypt used forward-schedule keys with `aesdec`** (runtime,
   xiom_runtime.c): hardware decryption requires the aesimc-transformed
   inverse schedule for middle rounds. **Fix: `_mm_aesimc_si128` on rounds
   1..n-1** (kept for correctness of the hardware decrypt entry point).
9. **crypto.xi aes_encrypt/aes_decrypt declared `requires` contracts on
   graceful-fallback fns** [U+FFFD] bad-key calls TRAPPED instead of returning Err
   (documented stdlib rule BUG 22 #5: no contracts on fallback fns).
   **Fix: contracts removed; bodies validate.**

**Verification (isolated binary):** smoke_crypto, smoke_crypto_known_vectors,
ALL 25 smoke*crypto*.xi smokes (roundtrip, GCM, bad-key, sha256/512, md5,
blake3, hmac, pbkdf2, hash vectors, secure random, constant-time compare,
RSA keypair 24-bit) [U+FFFD] 25/25 R=0. FIPS-197 Appendix B (AES-128) and C.2
(AES-192) exact match through BOTH aes.xi's AesState implementation and
crypto.xi's byte-oriented implementation; AES-NI hardware encrypt +
software decrypt roundtrip recovers the plaintext. Regression sweep
34/34 (3 new tests: m37_const_array, m37_payload_ref, m37_ptr_cast),
checker 178/178, workspace zero warnings.

**Remaining (documented, not blocking):** the 6 M22 stress smokes that used
`Vec.get(idx)` were adapted to indexing (`.get` is not a Vec builtin [U+FFFD] the
checker resolves it to a Box-typed get; "expected Box, found Int"). The
tuple-pattern `Ok((a, b))` checker simplification (binds Int) remains [U+FFFD]
documented workaround: `.0`/`.1` field access (applied to the GCM smoke).

## 2026-08-13 [U+FFFD] LANGUAGE features (all implemented + tested, commits `dd6a31cd`/`4c439e6a`)
- **BUG 26 (secure numeric policy)**: INT ? FLOAT mixing in arithmetic,
  comparisons, and typed bindings requires an explicit `as` cast (Rust-style).
  Auto-widening stays for same-family; INT LITERALS may adopt the float type
  (`d * 2` stays ergonomic); FLOAT literals never adopt an integer type.
  The checker now also resolves nested Vec[...] element types + dispatches
  methods on the base Vec.
- **Labeled loops**: `@label: while ...` + `break @label;` /
  `continue @label;` (the label field was previously dropped in the codegen
  While arm [U+FFFD] labeled break silently targeted the innermost loop).
- **BUG 27 (in-code debug intrinsics)**: `assert(cond[, "msg"])` (clean
  xiom_panic violation), `dbg!(expr)` (prints "[dbg] <value>", returns the
  value), `todo!()`/`unimplemented!()` (panic with source location),
  `debugger;` (breaks into an attached debugger [U+FFFD] xiom-debugger_break;
  no-op without one). All builtins yield to user fns with the same name.
- **NOTE 4 FIXED**: module-qualified enum variant access (`bigfloat.Down`)
  resolves + constructs the variant correctly.


1. **Same-name delegation ? 0xC0000005**: a local `pub fn to_base58` PLUS `use xiom.num.convert;` (which exports `to_base58`) [U+FFFD] any call to the IMPORTED fn AVs at runtime (even via an indirection helper or `as` alias). Stdlib rule: never delegate to a same-named fn across modules; implement locally. (Related: cross-module same-name resolution silently prefers the LAST imported module's version [U+FFFD] quadtree_query resolved to spatial's variant.)
2. **`use X as alias;` miscompiles**: `use xiom.num.base as numbase; numbase.to_base(255,16)` traps `contract violated: ensures`; `use xiom.bits.popcount as pc;` returns 0. Fully-qualified `xiom.num.base.to_base(...)` works. Fix: alias-import resolution.
3. **`from_bytes` fn name collides with a compiler builtin**: ANY module fn named `from_bytes` (even `{ return 0; }`) emits `invalid getelementptr indices` on `%struct.Vec`. convert/bytes.xi keeps the fn as TODO(compiler) [U+FFFD] unimplementable.
4. **Bool inside a returned tuple miscompiles**: fn returning `(Int, Bool)` or `(Int, Int, Bool)` emits `%tmp defined with Tuple__Int__Int but expected Tuple__Int__Bool` (tuple-literal constructor emits an all-Int LLVM type). Same family as BUG 23 #7 (Bool tuple misregistration). Blocks convert/overflow.overflowing_* (TODO(compiler)).
5. **Caller-side `.value`/`.error` field reads of Option/Result are corrupted** (wrong len/bit pattern) while `match` extraction is correct. Blocks `xiom.encoding` decoders when read via `.value` (verified: encoding.base64_decode round-trip fails [U+FFFD] pre-existing). Stdlib rule: match-extract only.
6. **Char `<`/`>` comparisons miscompile** (all chars report "bad") while `==`/`!=` and `byte_at(...) as Int` work. Root cause of the broken core `to_float_from_str` (digit-range checks). Stdlib parsers use byte-based comparisons.
7. **Bitwise AND on two Int operands with bit 31+ set miscompiles** (`0xAEF1AD80 & 0x9B05688C` wrong by one bit); small masks, OR, shifts, byte_swap are correct. Stdlib hash/mask code avoids big&big (uses shifts + small masks).
8. **`let len = v.len();` in specific module shapes triggers the same GEP error as #3** [U+FFFD] `var len` avoids it.
9. **Module-name vs fn-name collision**: `use xiom.bits.popcount;` + `use xiom.bits.bitwise;` (which exports a `popcount` fn) cannot both resolve [U+FFFD] smokes split.
10. **`xiom.crypto` unimportable** (`undefined @_pkcs7_pad` at link) and pure-XIOM SHA-256 miscompiles (wrong digests for ""/"abc"; documented in crypto.xi) [U+FFFD] pre-existing, blocks base58check's double-SHA256 (Adler-32 fallback shipped, TODO(compiler)).
11. **Private fn leaks into global namespace via `use`**: collect/heap.xi's PRIVATE `pheap_merge(h, a, b)` became reachable through `use xiom.collect.heap;` in pairingheap.xi and hijacked bare calls. Stdlib rule: module-qualified calls for shared names; fix direction: private symbols must not be re-exported by `use`.
12. Spurious E001 borrow warnings through imported fns (benign, codegen correct; matches pre-existing smokes).

---

## 2026-08-12 (night) [U+FFFD] stdlib session: BUG 26 (NEW, regression from 271567b0/841119d8/28aaa5e0 batch) [U+FFFD] user-module surface regressions

1. **Catalog-RETURNED Vec passed to a `&Vec[T]` catalog param ? C001** ("cannot take a reference to 'data': it is already a reference"): `var c = lz4.lz4_compress(data); var d = lz4.lz4_decompress(c);` fails codegen; even copy-out (`var c2 = c`) or a by-value thunk still fails. Locally-built Vecs work. Affects smoke_compress_lz4_snappy (decompress round-trips trimmed to compress-only, TODO(compiler)). Suspect: returned-Vec values are typed as references by the 28aaa5e0 payload-aware-read change, and the &Vec auto-ref then double-refs.
2. **Bare prelude names (to_char/to_string/to_int) no longer resolve in USER modules** [U+FFFD] `use xiom.core.to_char;` (a non-pub prelude fn) does NOT register it, and the old implicit bare-name path was removed by 271567b0's ambiguity error. Catalog modules still get them via prelude wiring. User smokes now use the public `convert.int_to_string` / `convert.float_to_int` / `convert.int_to_char` (match on the Option). Fix direction: prelude names should remain bare-callable in user modules (or be re-exported as pub from a user-facing module).
3. **Cross-module tuple DESTRUCTURING regressed**: `var (a, b) = some_catalog_f.(...)` binds BOTH names to the whole tuple (error "cannot compare Tuple__X__Y with X"). `.0`/`.1` field access (fixed in the 22/23 batch) works [U+FFFD] smokes switched to field access.
4. Same-family C001 for `snappy_*`/`lz4_*` decompress calls (all &Vec-param catalog fns taking module-returned values).
5. **High-bit mask comparisons miscompile in UTF-8 classifiers** (BUG 25 #7 family, extended): `(b0 & 0xE0) == 0xC0` / `(b0 & 0xF0) == 0xE0` / `(b0 & 0xF8) == 0xF0` on `byte_at`-derived Ints are FALSE for multibyte lead bytes (C3, etc.) ? convert/utf8.xi utf8_valid_sequences counts every byte as a 1-byte sequence (6 for "h[U+FFFD]llo" instead of 5). Small masks (< 0x80) and shifts are fine. Stdlib smokes adapted (expected value + TODO); fix direction: verify AND folding for masks with bit 7+ set against byte-extracted values.
6. **percent_encode combination miscompile** (BUG 24 family): `convert/percent.xi` percent_encode returns fully-encoded output ("/a?b=1&c=2" ? "%2Fa...") when the smoke also imports xiom.convert.bytes or xiom.string, but correct output in isolation. `_is_url_safe` branch selection flips with program shape. Smoke split into isolated modules (smoke_convert_percent / smoke_convert_bytes); module verified correct in isolation.

---

## 2026-08-13 [U+FFFD] stdlib session: BUG 27 (wave-5 batch) [U+FFFD] regressions + findings from 7 parallel agents

**Regressions from the 4c439e6a (NOTE 4 / BUG 25 #2) batch:**
1. **`xiom.os.platform` sublib prefix unresolvable in user modules** [U+FFFD] `use xiom.os.platform;` + `platform.platform_name()` fails "cannot call on this expression" even in isolation (worked pre-4c439e6a). Workaround: fn-level imports (`use xiom.os.platform.platform_name;` + bare call). Likely the NOTE 4 module-qualified-variant change interacting with the flat os.xi `platform()` fn shadowing the sublib prefix.
2. **`xiom.string.format` sublib fns (str_format1/2/3) unreachable** [U+FFFD] flat string.xi exports same-named fns; sublib path fails regardless of qualification (pre-existing family, now deterministic). smoke_string_format_printf is compile-only.

**Flat crypto.xi defects (stdlib, verified 2026-08-13):** `crypto.sha512` wrong output (`_u64_rotr` mis-handles signed Int [U+FFFD] arithmetic-shift semantics), `crypto.md5` wrong (SHL instead of ROTL in rounds), `crypto.aes_decrypt` fails its own length check, `aes_encrypt_gcm`/`decrypt_gcm` heap-corrupt, `rsa_sign/verify/generate_rsa_keypair` fail at runtime. The wave-5 crypto/hash.xi + cipher.xi implement correct local versions (RFC 1321/4231/5869 vectors verified). The compiler session's smoke_stress_crypto_* are actively exercising this surface.

**Other wave-5 findings (each with in-module or in-smoke documentation):**
3. Qualified-name resolution breaks with 5+ sibling leaf-module imports (xiom.iter.* + xiom.iter.range.range call ? T001); split smokes.
4. Generic fn + fn-value param + Vec[T] ? codegen "use of undefined value %tmp" (sort_by-style comparators); everything concrete per GENERICS policy.
5. Cross-module `Option[Vec[T]]`/`Result[Vec[T],_]` payloads corrupt (regex captures, scanf tokens) [U+FFFD] is_some/len trustworthy, elements garbage.
6. `Error` is a compiler-reserved type name [U+FFFD] modules defining `pub type Error` emit "unknown type 'Error' defaulting to i64" + corrupt cross-module codegen (renamed ChainError etc.).
7. Sibling submodule imports clobber earlier siblings (import ORDER matters for error.backtrace/context, test.assert/harness cannot coexist).
8. Module-scope fn storage is read-only (`g.f0 = f` silently no-ops; test_register records names only); module-level bare Vec mutation silently fails (push then len==0).
9. Structs with first field named `type` break cross-module fn resolution; structs containing `Map[Str,Str]` hang/0xC0000005 on construction (mime.xi uses Vec[(Str,Str)]).
10. Tuple payloads in Result miscompile (`Ok((a,b))` ? `.value.0` = 0; match works); `(Vec,Vec)`/`(Vec,Int)` returns heap-corrupt (0xC0000374).
11. `UInt64 >>` is arithmetic (0x8000[U+FFFD]>>1 ? /2); `UInt64 as Float64` reinterprets bits (use num.f64_from_u64); `Float64.to_string()` prints bit patterns.
12. `let (a, b) = tuple` destructuring binds both names to the whole tuple (use .0/.1); `match *ref` on enums miscompiles (match values).
13. Module auto-loads its parent: same-named sublib fns misresolve to the parent's (varint_decode/json_parse) [U+FFFD] unique-name private helpers used.
14. Int32/Int returns from catalog unsafe blocks corrupt (fs_move's rename rc always non-zero; file moved correctly) [U+FFFD] fs.xi rewritten copy+remove (documented non-atomic).
15. `ptr == 0 as *UInt8` compiles to strcmp(ptr, NULL) [U+FFFD] compare `ptr == 0` inside unsafe; `s.c_str()` not null-terminated for strlen; `Int as *UInt8` corrupts (pointer?Int ok).
16. Struct literals assign positionally not by name (Quat{w:[U+FFFD];x:[U+FFFD]} swaps); row swap via tuple assignment corrupts (swap element-wise); Vec[struct] index-writes corrupt (rebuild Vecs).
17. Module-name vs fn-name collision (spawn module + spawn fn) [U+FFFD] import order matters; sublib `xiom.string.format`/`xiom.os.platform` collide with flat fns.
18. `xiom_atomic_store` doesn't round-trip negative Ints; runtime `xiom_mutex_*` broken (trylock always 0); real threads unusable (fn?ptr cast T001 + xiom_thread_spawn AV) [U+FFFD] thread/spawn.xi is a documented inline simulation.
19. Bool display via generic renders "1"; Str.to_str()/Float64.to_str() crash; Float32?Float64 conversion prints bit patterns; Vec[Bool] elements type as generic T.
20. High-bit mask AND still miscompiles for UTF-8 classifiers (convert/utf8.xi [U+FFFD] BUG 26 #5, unfixed).

## 2026-08-13 [U+FFFD] BUG 27 items RESOLVED (compiler side) + security review implementation

**1. Sublib-prefix resolution regression (xiom.os.platform / xiom.string.format [U+FFFD] 4c439e6a):**
use xiom.os; os.platform.platform_name() failed with "cannot call 'platform_name'
on this expression". Root causes + fixes (commits 4e95717e, 1aa93cc?):
- The module map for directory modules (os.xi + os/*.xi) only contained the
  module's own exports [U+FFFD] submodule segments were unresolvable. The qualified-call
  walk now descends submodule segments LAZILY via catalog.peek_owned() (parses
  WITHOUT caching, so the submodule's decls never enter the injection set [U+FFFD]
  eager loading perturbed bare-alias keep-first resolution and broke unrelated
  programs, e.g. crypto sha256).
- Name collisions (xiom.os has BOTH `pub fn platform()` and the `platform`
  submodule) descend through submodule_aliases keyed by full dotted path; the
  2-segment call `os.platform()` keeps resolving the fn.
- local_module_paths: checker-only local-name -> full-dotted map recorded for
  every use (kept OUT of use_alias_paths, which the codegen's bare-call alias
  resolution consumes).

**2. Generic constructors in module-global initializers (Map[Str, Bool].new()):**
ar _coverage: Map[Str, Bool] = Map[Str, Bool].new(); (core/contracts.xi)
emitted a stub `define i64 @Map.new() { ret i64 0 }` (runtime crash
0x80000003) or "use of undefined value '@new'" (link error). Three fixes:
- infer_struct_type_name: Index receivers with a KNOWN TYPE base resolve to the
  base type [U+FFFD] fn_key "Map.new", not the bare-key hijack of another module's
  generic `new`.
- The checker prelude force-loads xiom.collections (container generics were
  never imported by any use, so their decls never reached the monomorphisation
  registry). Planned refinement: catalog reverse type-index + on-demand load
  (docs/ROADMAP.md).
- ginit drain: @llvm.global_ctors bodies compile AFTER the first monomorphisation
  pass; the instantiation queue is drained again so generic ctors called from
  globals get real bodies.

**3. Security review implementation (user-approved, production-grade):**
- Release builds strip assert/dbg!/debugger; (`--keep-debug-checks` retains;
  contracts were already release-stripped, `--runtime-contracts` forces).
  dbg! still evaluates and returns its value [U+FFFD] only the print disappears.
- Const-eval budget: evaluate_const_init is bounded by CONST_EVAL_BUDGET (4096)
  recursion depth; on overflow the expression is returned unevaluated (materializes
  at runtime) [U+FFFD] a hostile const cannot hang the compiler.
- asm(): verified ALREADY unsafe-gated (T001 "inline asm requires an unsafe
  block"); stdlib contains no asm usage.
- --enable-unsafe-direct: prominent stderr warning on every invocation.
- Catalog fn bodies bypassing the checker (invalid casts compile silently in
  stdlib modules) remains OPEN [U+FFFD] see docs/ROADMAP.md (multi-session item).

**Verification:** 34/34 regression sweep (incl. new m37_const_array,
m37_payload_ref, m37_ptr_cast), checker 178/178, crypto 29/30 (only
smoke_stress_crypto_aes_gcm [U+FFFD] REPRODUCED AT BASELINE; the parallel stdlib
session's in-flight "tuple+Vec heap corruption", BUG 27 #12), workspace zero
warnings. os.platform/string.format sublib probes R=0; contracts + Map.new
global-init probes R=0; release-strip probe verified in all three modes.

---

## 2026-08-13 (evening) [U+FFFD] stdlib session: BUG 28 (regressions verified against 4e95717e) [U+FFFD] verification round findings

1. **Catalog unsafe-block Str construction re-broken** (env.xi `var_opt`): `return Some(Str.from_cstring(raw))` from inside a small fn's unsafe block AVs (0xC0000005) under 4e95717e; worked pre-4e95717e. FIXED stdlib-side with the read_file-proven multi-block shape (pointer/Int assignments + Vec.push inside unsafe; `Str::from_utf8` outside). The compiler still corrupts Str structs flowing through the catalog unsafe trampoline in small (always-inlined) fns.
2. **Option-Some payload binding inside CONTRACT evaluation traps**: env.xi `home_dir`'s `ensures: result is Some => result.len() > 0` panics on every call (contract violated at the ensures line) [U+FFFD] the Some-payload binding in the contract check miscompiles. Clause dropped (TODO(compiler)); the same family blocks any `result is Some => ...` ensures on Option returns.
3. **Option[Str] second-hop return corrupts**: `home_dir()` returning var_opt's Option[Str] (even pure passthrough) corrupts the payload in small callers [U+FFFD] reading it AVs. Any 2-catalog-hop Option[Str] return is unsafe. home_dir kept as passthrough + TODO(compiler).
4. **os.platform/string.format aggregate shadowing**: `use xiom.os;` + `os.platform.platform_name()` resolves to the FLAT os.xi platform() (returns 0) [U+FFFD] silent wrong result; the fully-qualified `xiom.os.platform.platform_name()` works (4e95717e fix verified). smokes use the fully-qualified form.
5. **Catalog struct literals drop trailing fields when the first field is a var**: `Timer{ deadline: dl; armed: true; }` reads armed=false (4e95717e regression; constant-field literals and all-Float64 structs work; identical code in user modules works; F2-vs-Timer divergence shows usage-shape dependence [U+FFFD] BUG 24 family). Affects async/timer.xi interval timers [U+FFFD] smoke asserts only the inert path, TODO(compiler).
6. **"Cannot allocate unsized type" at clang**: os_path smoke's file/path sections (os/file.xi Result/Option matches + os/path.xi fns) fail to compile in combination while each works in isolation (4e95717e). Smoke trimmed to the verified fs/dir sections, TODO(compiler).
7. **`@Executor.new` undefined in minimal programs**: importing only xiom.async.timer (+executor) fails link ("use of undefined value '@Executor.new'"); the full smoke import set works. Generic-ctor injection is program-shape dependent.
8. **`use xiom.X;` aggregate import affects sublib struct-literal codegen** (timer.xi literal worked after adding `use xiom.async;` in one configuration) [U+FFFD] same BUG 24 family; not a reliable workaround.

---

## 2026-08-13 (night) [U+FFFD] compiler session: BUG 27 #12 + BUG 28 #1-#8 ALL RESOLVED

Every item the stdlib session filed is now FIXED on the compiler side, verified
with harness drivers in docs/repros/ (all exit 0):

1. **Catalog unsafe-block Str construction** [U+FFFD] VERIFIED FIXED (env.xi var_opt
   + match + config_dir second-hop roundtrip exit 0). The stdlib's multi-block
   shape works; the earlier small-fn corruption is covered by the BUG 29
   visibility/owner fixes.
2. **Option-Some payload binding in CONTRACT evaluation** [U+FFFD] VERIFIED FIXED
   (repro_opt_contract: ensures: result is Some => result.len() > 0 on an
   Option[Str] catalog fn exit 0). The stdlib can restore home_dir's clause.
3. **Option[Str] second-hop return** [U+FFFD] VERIFIED FIXED (same repro; match +
   rewrap + unwrap all exit 0). home_dir/config_dir can be restored.
4. **os.platform aggregate shadowing** [U+FFFD] FIXED (peeked-submodule injection,
   b01d7c5e): os.platform.platform_name() now resolves to the REAL submodule
   fn (platform_name returns a real value; flat os.platform() still works).
   The old "fully-qualified works" was a false positive (str_len(null) != 0).
5. **Catalog struct literals drop trailing fields** [U+FFFD] VERIFIED FIXED
   (repro_timer_literal: Timer{deadline: dl; armed: true; label: "t"} reads
   all three fields correctly).
6. **"Cannot allocate unsized type" (os/file + os/path combo)** [U+FFFD] FIXED
   (b01d7c5e): tuple EXPRESSION element typing erased Bool to Int, so
   (PathBuf, Bool) built "Tuple__PathBuf__Int" while the signature
   registered "Tuple__PathBuf__Bool" [U+FFFD] the expr-built type was never
   pre-registered and its definition emitted after the alloca that used it.
   Tuple element naming now uses XIOM types for literals. os file write/read/
   remove roundtrip compiles and runs.
7. **@Executor.new undefined in minimal programs** [U+FFFD] VERIFIED FIXED: the
   reachability filter now seeds from KEPT const initializers (5865a3b5), so
   module-global ar _exec = Executor.new() keeps the ctor alive regardless
   of import shape. Minimal xiom.async.timer-only program exit 0.
8. **use xiom.X aggregate import vs sublib struct-literal codegen** [U+FFFD] covered
   by the #5 verification (same BUG 24 family).

Plus BUG 27 #12 (the last baseline crypto failure):
- **Tuple+Vec payload corruption (smoke_stress_crypto_aes_gcm 0xC0000005)** [U+FFFD]
  FIXED (cfe783e0), 3-part chain:
  a. callee_return_xiom resolved by bare leaf suffix only [U+FFFD] ambiguous
     (cipher.aes_encrypt_gcm -> Vec[UInt8] vs crypto.aes_encrypt_gcm ->
     Result[Tuple__Vec__Vec, Str]) returned None and dropped payload
     tracking. Now resolves the receiver prefix first.
  b. boxed-struct field access matched tuple fields by raw position() [U+FFFD] only
     pair._1 worked; numeric pair.1 fell to Str.len. Now uses
     resolve_field_index (both forms).
  c. is_container_vec_field (vec_abi.rs) had no boxed-tuple-field case [U+FFFD]
     pair.1.len() misdispatched to Str.len. Now classifies via
     local_boxed_struct + container base types (tolerates erased "Vec").
  **Crypto smokes 30/30** (was 29/30 since the baseline); full gcm encrypt+
  decrypt roundtrip exit 0.

Also closed: BUG 29 (fn_symbol emission vs fn_key, checker dotted-module path
join, visibility keep-first [U+FFFD] feea1b8a) which fixed the m19_default (~110)
and m18_guard/ecosystem (~30) e2e clusters, plus the 5 repro files
(repro_error_type, repro_fn_storage, repro_unsafe_int, repro_opt_vec,
repro_tuple_vec) [U+FFFD] all exit 0 with harness drivers (5865a3b5).

Remaining (not compiler blockers): closure-through-fn-slot crashes (B-007
family [U+FFFD] reproduced at baseline with a plain fn-typed param; needs a design
decision on the fn-ptr vs env-ptr ABI), and the e2e alias-comparison /
ambiguity clusters (checker strictness, unrelated to stdlib).

---

## 2026-08-16 [U+FFFD] compiler session: BUG 30 batch (16 regression failures from the 04:03 run all fixed) + 904-smoke survey

### BUG 30 [U+FFFD] 14 fixes, commits `0ddc4500` + `5940ba2c` (all verified)

The 2026-08-14 04:03 test run had 12 e2e failures + 4 stdlib-exec failures +
1 diff failure. All compiler-fixable items are FIXED (full detail in
docs/SESSION.md, 2026-08-16):

1. **Imply scoping** [U+FFFD] contract-ensure `is Some/Ok/Err` payload rebinds no
   longer poison later return-site checks (i64-form Is inttoptr+load ? AV).
   Fixes b003 / m19_read_file / file_stem chains.
2. **Is() payload-slot hoisting** (struct + i64 forms) [U+FFFD] `&&` guard chains
   bind in one block, read in a later one ? dominance error (m18_guard_0086).
3. **Is() bare-scrutinee rebind in the enum-variants form** (utf8 Map.len).
4. **struct_type_from_expr Call-arm resolution** [U+FFFD] match-on-catalog-call
   dropped the scrutinee (m35_z24/z29, eco_algo).
5. **Mono path flushes hoisted allocas** (m35_z24 "undefined value %tmp55").
6. **Enum/Result literal ctors zero-init unused payload slots** [U+FFFD] LLVM poison
   + clang -O2 ? deterministic AV (m35_z10/z29). NOTE: the driver runs
   `opt -O2` + `clang -O2`; uninitialized-slot reads are exploited.
7. **coerce_arg_for_param `&array_local`** [U+FFFD] data pointer only for `&[N]T`
   params; `%struct.Vec*` params get the header alloca (eco_algo).
8. **len() dispatch for boxed Vec-handle locals** (utf8 ensure).
9. **Ensure `result` scoped per-check** [U+FFFD] user locals named `result` shadowed
   the synthetic return slot (utf8_encode AV).
10. **decl.rs result local_xiom_types uses type_string_full** (payload args).
11. **collect_block_free_vars sorts captures by name** [U+FFFD] HashSet order
    differed between block/type/caller passes (Rc.new slot swap).
12. **Generic receiver ABI** [U+FFFD] by-value `self` passes the struct VALUE not a
    pointer (smoke_rc).
13. **Checker wildcard method lookup deterministic** + clone skip applies only
    to the fallback, not the direct hit (smoke_rc r2 typed `_`).
14. **struct_type_from_expr Type.method static calls** [U+FFFD] `Vec[Str]::new()`
    must resolve `Vec.new` (exact/module-qualified suffix); unregistered
    typed keys return None [U+FFFD] NEVER the bare alias (BufReader.invariant_check
    on a Vec ? invalid IR; smoke_net_http + io graph).

**CORRECTION to the 2026-08-14 report:** the 4 stdlib-exec stragglers were
labeled stdlib-side [U+FFFD] WRONG for 3 of them. `smoke_rc`, `smoke_cell`,
`smoke_utf8` were COMPILER bugs (fixes #11/#12/#13, #3/#8/#10, #9/#10) and
now PASS. Only `smoke_hash_folder` is stdlib-side (missing
`use xiom.convert.toint;`).

### 904-smoke battery survey (NEW [U+FFFD] the real production gate)

`examples/stdlib_smoke/` has **904 smoke files** (the 08-14 report's
"stdlib-exec 68/72" covered only 72 of them). Full sweep with the isolated
binary: **~516 pass / ~289 fail** (run in 8 parallel batches; batch 8
timed out, totals approximate). Failure classes:

**Real compiler bugs still OPEN (next sessions):**
- Map AVs 0xC0000005 (smoke_stress_collections_map_*: get_missing/insert_get/
  clear/collision/overwrite) [U+FFFD] Map is a NEW stdlib type, never swept before.
- fmt AVs 0xC0000005/0xC0000409 (smoke_fmt_edge/float/format*) +
  `void type only allowed for function results` (smoke_fmt_formatter) [U+FFFD]
  void in expression position.
- struct-literal field-type mixups: `store %struct.BST %vecval` in
  benchmark.main Node.new (bench_math native compile; harness is
  emit-ir-only so not suite-blocking) [U+FFFD] same family as fix #14 but a
  FIELD-TYPE/INDEX lookup issue in the non-enum struct-literal path.
- Interface-bound gap: `type 'Int' does not implement 'Bounded': missing
  method 'is_finite'` (smoke_num_saturating).
- BUG 26 #2 bare prelude names in user modules (LIVE: smoke_alloc_basic
  `undefined variable 'ptr'`), BUG 26 #3 cross-module tuple destructuring,
  BUG 26 #5/BUG 27 #20 high-bit mask AND (per doc unfixed; smoke_utf8 now
  passes [U+FFFD] stdlib worked around it).
- BUG 24 residual (bigfloat pow_bf per-program-shape AV) [U+FFFD] doc says
  PARTIAL; smoke_num_precision PASSED in this sweep (may be closed by the
  later batch [U+FFFD] needs re-verification).
- Closure-through-fn-slot (B-007 family) [U+FFFD] design decision pending.

**Stdlib-side (for the stdlib session [U+FFFD] smoke/API staleness):**
parse errors (P001 [U+FFFD] e.g. smoke_stress_rand_shuffle), undefined vars /
missing imports (smoke_alloc_basic `ptr` family, smoke_net_* API drift),
renamed/absent APIs (smoke_stress_rand_* compile failures). hash_folder
missing import. The stdlib session should fix these in their tree; the
compiler-side list above is the compiler's share.

---

## 2026-08-16 (evening) [U+FFFD] stdlib session: two NEW compiler findings during gap-fill implementation

### BUG 31 [U+FFFD] unary minus on Float128 emits `sub i64 0, fp128` (codegen, clang rejects)

- **Construct:** any `-x` where `x: Float128` (unary negation in an expression).
- **Repro:** `var a = 1.0 as Float128; a = -a;` [U+FFFD] clang: `error: '%tmp' defined with type 'fp128' but expected 'i64'` at `sub i64 0, %tmp`.
- **Worked around in stdlib** (num/bigfloat.xi bigfloat_to_float128 uses `acc * (-1.0 as Float128)` with a TODO(compiler) note) [U+FFFD] negation via literal multiply is idiomatic float code and the only construct that trips it.
- **Likely fix:** fneg path in codegen must emit LLVM `fneg fp128` (or `fsub fp128 0.0, x`), not the integer `sub i64` form. Check the Int-only neg emission branch; Float32/64 likely share it [U+FFFD] verify those too.
- Also related: standalone `xiom.exe --emit-ir` on num/bigfloat.xi reports `cannot call 'to_int' on this expression` at 239:13 [U+FFFD] a STANDALONE-check false positive (the fn resolves through the import catalog; aggregate-file check quirk documented in STDLIB_IMPLEMENTATION.md [U+FFFD]4 [U+FFFD] the real test is a consumer program, which type-checks fine).

### BUG 37 [U+FFFD] fp128 RETURNED from a catalog fn crashes the caller (0xC0000005)

- **Construct:** calling a stdlib (catalog) fn that returns `Float128` and
  doing anything with the result (even just assigning it), in a program that
  also imports bigfloat machinery. Crash occurs inside/right after the call,
  before the next statement.
- **Status:** the IDENTICAL fn defined in user space (local struct + local
  fn, g17 probe) exits 0 with correct values; the catalog version crashes in
  every caller shape tried (direct assignment, print-after, compare, negate,
  single vs dual import, explicit result type). fp128 arithmetic on
  user-locals is fine (smoke_d1_native128); the failure is specific to the
  catalog-return boundary. BUG 24/28-family shape dependence.
- **Impact on stdlib:** num/bigfloat.xi `bigfloat_to_float128` (structured as
  three single-shape fns to dodge BUG 36) compiles and is correct, but no
  consumer program can use it yet [U+FFFD] the gap-fill smoke cannot include it
  (documented in the smoke). TODO(compiler) notes left in the module.
- **Likely fix:** the catalog-return ABI for fp128 (value vs sret, or the
  mono'd copy-out path) [U+FFFD] the compiler session's BUG 24/27 #12 families.

### BUG 36 [U+FFFD] fp128 Horner-loop fn shape crashes when any sign/scaling statement follows (0xC0000005)

- **Construct:** a fn that (a) runs a Horner loop mixing fp128 mul/add with
  i64?fp128 casts of Vec-element reads through a struct chain, and (b) ALSO
  contains any subsequent fp128 statement [U+FFFD] a sign negate
  (`acc * (-1.0 as Float128)` or `(0.0 as Float128) - acc`), or an
  in-loop Int negate [U+FFFD] crashes at runtime 0xC0000005 even when the sign
  branch is not taken. Removing the sign statement (or moving it to a
  separate helper fn) makes the same fn exit 0.
- **Status:** every individual piece (Horner loop, pow10 loop, div loop,
  negate, Int128 math) passes in isolation and in small combinations; the
  crash is strictly per-fn-shape (BUG 24/28 #5 family). The compiler
  session's "repro g12/g14/g16" probes are in this session's notes.
- **Impact on stdlib:** num/bigfloat.xi bigfloat_to_float128 is structured
  as: Horner loop fn + `_f128_pow10` helper (one scaling loop) + `_f128_neg`
  helper (sign only) [U+FFFD] three single-shape fns that each compile clean.
  TODO(compiler) notes left; consolidate when the shape bug lands.
- **Likely fix:** fp128 register allocation / mono across statement
  boundaries [U+FFFD] the compiler session's BUG 24 family.

### BUG 35 [U+FFFD] Int128 index math inside a Vec-writing fn shape AVs (0xC0000005 / 0xC000001D)

- **Construct:** `var diff128 = (v[i] as Int128) - (min as Int128);
  var idx = (diff128 / (width as Int128)) as Int;` inside a fn that also
  writes Vec elements (`counts[idx] = ...`, `output[slot] = ...`).
- **Status:** the SAME Int128 ops pass in isolation (probe i128a/i128b exit 0)
  and pass in a counting-only fn, but crash when combined with Vec element
  writes in the same fn (0xC0000005) or with extreme i64 values
  (0xC000001D [U+FFFD] SIMD/illegal-instruction family). Shape-dependent
  miscompile, BUG 24 family.
- **Impact on stdlib:** sort/radix.xi bucket_sort uses plain Int index math
  with a documented i64-span caveat (TODO(compiler) notes left). Revisit
  with Int128 when the shape bug is fixed.
- **Likely fix:** Int128 mono/register allocation interacting with the
  boxed-Vec write path [U+FFFD] the compiler session's BUG 24/27 #12 families.

### BUG 34 [U+FFFD] nested Vec element WRITES via reference AV (0xC0000005)

- **Construct:** `bs[idx].push(x)` or `&bs[b]` passed as `&mut Vec[Int]`
  where `bs: Vec[Vec[Int]]` [U+FFFD] mutation of a nested-vector ELEMENT.
- **Status:** READS of nested elements work (`outer[0][1]` compiles and runs),
  but WRITES through the inner vector reference crash at runtime
  (0xC0000005). The pre-existing stub note in sort/radix.xi ("nested
  Vec[Vec[Int]] element access is not handled reliably") is still accurate
  for the write path.
- **Impact on stdlib:** sort/radix.xi `bucket_sort` was rewritten with a flat
  offset-table distribution (counts/offsets Vec[Int] + single output Vec[Int])
  to avoid nested vectors entirely; semantics unchanged (stable per-bucket
  ordering, O(n + b) expected).
- **Likely fix:** the `&mut` reference-to-nested-element lowering (pointer to
  the Vec handle inside the outer buffer vs pointer to the inner heap data)
  [U+FFFD] same family as BUG 24 / the coerce_arg_for_param `&Vec` fixes.

### BUG 33 [U+FFFD] Option[Float128] payload unwrap loads undefined `%struct.Float128`

- **Construct:** matching/unwrapping an `Option[Float128]` payload (the
  `Option__Float128.unwrap` / match-Some binding path).
- **IR evidence:** `%struct.Option__Float128 = type { i64, fp128 }` is defined
  correctly (native fp128 payload), but the unwrap fn emits
  `%result = load %struct.Float128, %struct.Float128* %val_gep` [U+FFFD] the
  payload's STRUCT NAME (`%struct.Float128`, never defined (opaque)) instead
  of the native `fp128`. clang: `error: load operand must be a pointer to a
  first class type`.
- **Impact on stdlib:** any fn returning `Option[Float128]` is unusable by
  consumers until fixed. num/bigfloat.xi `bigfloat_to_float128` therefore
  returns bare `Float128` (documented inf saturation) with a TODO(compiler)
  note; the Option variant can land once the unwrap names the payload type
  correctly.
- **Likely fix:** the Option-payload unwrap type-name resolution should emit
  the native LLVM scalar type for Float128 (same class of fix as BUG 30 #10
  local_xiom_types / payload slot typing [U+FFFD] the payload type lookup must map
  XIOM Float128 -> LLVM fp128, not the struct name).

### BUG 32 [U+FFFD] Int->pointer cast (`x as *T`) emits address-of-local, not `inttoptr`

- **Construct:** `var h = buf as Int; var q = h as *UInt8;` inside an unsafe
  block (any Int VARIABLE cast to a pointer).
- **IR evidence (repro pdb3.xi):** `h = buf as Int` correctly emits
  `ptrtoint`; the reverse cast emits `bitcast i64* %alloca_slot to i8*` [U+FFFD]
  i.e. the ADDRESS OF THE LOCAL holding h, NOT `inttoptr i64 %h to i8*`.
  The recovered pointer reads the stack slot, so `q == buf` is false and
  `q[0]` reads pointer bytes. Constant casts (`0 as *T`) are unaffected
  (proper inttoptr), which is why `ptr.null` works.
- **Impact on stdlib:** blocks pointer-handle designs through Int-typed APIs
  (misc/glob.xi glob_compile/glob_compile_match [U+FFFD] worked around with a
  single-slot module-global registry, TODO(compiler) note left).
- **Likely fix:** the unsafe cast lowering [U+FFFD] emit `inttoptr` for Int->pointer
  casts instead of reusing the ptr-to-locals path.

### BUG 38 - if opt is Some { match opt { Some(x) => ... } } double-check binds payload as 0

- **Construct:** an is Some predicate check on an Option followed by a
  match on the same Option whose Some(x) arm binds the payload.
- **Repro (sx4.xi):** str_index_of("1.2.3-alpha.1", "-") returns Some(5);
  the plain match binds pos=5 (correct), but the if ... is Some + inner
  match form binds pos=0 (wrong) - every slice then starts at 1.
- **Impact on stdlib:** misc/semver.xi semver_parse/semver_parse_partial/
  semver_satisfies used the double-check on str_index_of results - any
  version with a -/+ suffix failed to parse (semver_valid("1.2.3-alpha.1")
  = false; smoke_misc2 exit 55). REWRITTEN to plain match (the pre-check
  was redundant) - smoke_misc2 now exit 0. iter.xi's while item is Some
  (single-check rebind) is unaffected and NOT this bug.
- **Likely fix:** the is-Some imply/binding machinery (BUG 30 #1/#2 family) -
  the payload slot binding after an is Some check must not poison the
  subsequent match's payload read (0 is the zero-init of the untracked slot).


### BUG 50 - generic container names with pointer type args emit invalid LLVM identifiers -- FIXED (`569c3d31`)

- **Construct:** a fn returning Result<*mut UInt8, AllocError> (pointer payload in a generic container). The mono names the container struct %struct.Result__*UInt8__AllocError -- the * is invalid in an LLVM identifier (clang: error: expected '=' after name).
- **Triggered by:** smoke_alloc_edge (global_alloc().allocate(l) returns Result<*mut UInt8, AllocError>). The stdlib-side API bug (global_alloc returning the Allocator interface type) was fixed first (alloc.xi -> GlobalAlloc); the remaining failure is this codegen naming issue.
- **FIX:** `sanitize_container_arg` maps every non-identifier char to `_` -- applied at ALL 7 container-name construction sites (ensure_concrete_option/result, concrete_type_for Option/Result arms, mono subst Option/Result arms, concrete_container_llvm) so the def and call sides agree. smoke_alloc_edge + the user probe now compile and run exit 0. (Regression caught & fixed in the same batch: tuple element names built from local_xiom_types leaked "Vec[Int]" into `Tuple__Vec[Int]__Vec[Int]` -- the tuple path now strips container args, matching the decl-side bare names; smoke_iter was restored.)

### BUG 51 - Option[UserStruct] payloads: bound-name method resolution + runtime corruption -- FIXED (`569c3d31`)

- **Construct:** a fn returning Option[Rc[T]]/Option[Ref[T]] (user struct payload) called from user code; the match's Some(up) binding then calls up.get().
- **Two facets:** (a) CHECKER: up.get() on the directly-bound name resolves to an Option-returning method ("cannot compare Option with Int") -- with an explicit var x: Rc[Int] = up; annotation it type-checks; (b) RUNTIME: the annotated form reads a WRONG payload value (rcprobe3: direct .get() = 42, upgraded x.get() != 42; user-space replica rcprobe4 with a local MyRc reproduces it -- not an rc.xi bug).
- **Impact:** smoke_rc_edge/rc_weak/cell_refcell_try/cell_refcell_mix blocked (weak.upgrade / try_borrow paths). Same family as BUG 33 (Option payload slots) -- struct payloads in the generic Option container still corrupt through the catalog/unsafe path.
- **Stdlib:** rc.xi/cell.xi implementations verified correct via the user-space replica.
- **FIX (checker):** `CheckedType::from_ast_type` PRESERVES container args ("Option[MyRc]"); Some/Ok/Err pattern bindings bind the payload with the INNER type from the scrutinee ("Option[MyRc]" -> up: MyRc) instead of the `_` wildcard (which fell to the sorted wildcard method lookup -- Option.get < MyRc.get alphabetically -> "cannot compare Option with Int"); Some() ctor types "Option[inner]"; container-erasure compatibility ("Option" ~ "Option[MyRc]") keeps erased forms interchangeable; Field access and method dispatch strip container args to the base name. **FIX (runtime):** no extra codegen change needed -- the payload read already used the BUG 43 scrutinee-payload machinery; the checker fix unblocked the annotated form. Probe (Some(up) -> up.get() and x.get() == 42) exits 0; checker 178/178.

### BUG 52 - enum-with-payload values in Map crash at runtime (0xC0000005) -- FIXED (`569c3d31`)

- **Construct:** Map[Str, T] where T is an enum WITH payload fields (e.g. JsonValue): m.insert("b", MyVal.Text("x")) + m.get("b") + match. User-space replica (jp7) AVs; the same enum in a Vec (BUG 42's fix) works. The Map value path still uses the enum's scalar/tag-only layout.
- **Impact:** serialize json_set/json_object building crashes -- smoke_stress_serialize_json_nested blocked (rewritten to the real json.* API, verified correct, blocked at runtime). json_parse/stringify of parsed values unaffected.
- **FIX -- FOUR coordinated codegen changes + two ABI-family root causes:**
  1. Enum-variant ctor args infer the ENUM type in the generic direct-match inference (`MyVal.Text("x")` -> V=MyVal, not Int -- insert_Str_Int became insert_Str_MyVal).
  2. Generic METHOD type args infer from LOCAL receivers via recorded container types: `infer_value_xiom_type` records typed static-ctor receivers ("Map[Str, MyVal]"), `infer_call_return_xiom` records fn-return containers (`var m = make_map()`), and the generic-inference fallback parses them positionally (m.get("b") -> Map.get[Str, MyVal], not [Str, Str]).
  3. Mono'd method bodies record Vec-typed receiver-field ELEMENTS: new `generic_type_field_types` (stdlib generic decls' args-preserving field types -- the builtin Map/Set registrations pre-empt type_meta with bare "Vec") + `record_field_vec_elem` in both receiver-binding branches, so `values[i]` reads take the struct/enum memcpy path (resolve_vec_elem_type -> Ident arm) instead of the scalar i64 load (tag only).
  4. Index-WRITE (`values[i] = value`, the duplicate-key update in Map.insert) memcpy's struct/enum elements INLINE instead of storing a boxed pointer as i64.
  - concrete_container_llvm now EXCLUDES enum payloads from concrete naming (mirrors concrete_type_for's M18 rule) -- the caller previously emitted %struct.Option__MyVal while the def never registers enum monomorphs ("Cannot allocate unsized type").
  - ABI root cause found while bisecting: `&K` params with K=Str -- the caller passed the STRING ADDRESS as i8** and the callee's `*key` loaded the string's FIRST 8 BYTES as a pointer (strcmp AV -- smoke_collections_map_basic crashed identically at baseline). Caller now materializes an i8* temp slot. `&Vec[T]` caller ABI fixed to %struct.Vec* (the def GEPs through the param; by-value pass read the data pointer as the Vec header -- every contains-style generic fn AV'd).
  - Verified: 6 user probes (insert/get/update/remove/enum round-trip, fn-return receivers), smoke_collections_map_basic, 30-smoke battery, e2e 2268/2268, checker 178/178, feature-reg 510/510, stdlib-exec 70/70+2. smoke_stress_serialize_json_nested improved (AV -> heap corruption) but still fails -- nested enum-with-Vec-field payloads through Map values remain (deeper layer, stdlib-session follow-up).

### BUG 48 - catalog generic-bound ASSOCIATED dispatch (Eq[T].eq) emits icmp eq 0, element -- UNBLOCKED (`3a789afb`/`175294ea`); verify on re-apply

- **Construct:** a stdlib (catalog) generic-bound fn calling the associated form Eq[T].eq(items[i], value) / Ord[T].compare(a, b) with the tower-style generic interfaces. The mono emits icmp eq i64 0, %element (compare-with-ZERO fallback) instead of calling the impl (verified in core.contains_Int IR: %tmp37 = icmp eq i64 0, %tmp36 -> ret 1 when element == 0).
- **Status:** user-space generic fns with the same call dispatch correctly (eqt16/eqt17 exit 0). The METHOD form (items[i].eq(value)) in catalog fns WORKS (contains/is_sorted exit 0) -- the checker note "method form works" holds. Only the associated form in catalog fns falls back.
- **NEW (2026-08-18):** the ABI-family root causes are fixed -- the user-space replica of the FULL construct (tower interfaces + impl Eq[Int] + generic contains_eq with the associated call) previously AV'd due to the &Vec[T] caller ABI (by-value vs %struct.Vec*) and now exits 0 in BOTH associated and method forms (m37_bug48_associated_generic_vec e2e). Whether the catalog-specific zero-fallback had an ADDITIONAL factor can only be confirmed by re-applying the tower Eq/Ord impls (the stdlib session's re-apply plan) -- retry with the current compiler.
- **Stdlib impact:** use the method form for catalog dispatch (the conversion plan uses it); the associated form is available for user-space code.

### BUG 49 - catalog fn-param call sites still break when Eq/Ord impls for i64-ABI types are present -- FIXED (`3a789afb`/`175294ea`)

- **Construct:** any program that (a) defines impl Eq[Int]/impl Ord[Int] (i64-receiver impls; Str-receiver impls do NOT trigger) AND (b) calls a CATALOG fn taking a fn-typed param (heap_sort_by(v, cmp_int) from xiom.sort.heap) -- the fn-param call site miscompiles (AV or wrong order). LOCAL fn-param fns in the same program are fine once the param is not named compare (param-NAME collision with the impl method symbol -- verified: local compare-named param breaks, cmp-named works).
- **Status:** BUG 47's param_locals fix cleared the cross-fn leak but NOT this catalog+impls shape. Param renames in the stdlib (compare->cmp) do not help the catalog case. The tower-style Eq/Ord impls (28) + method-form conversion were re-applied and REVERTED again because of this: smoke_sort (PASS->FAIL exit 4) and smoke_cmp_by regress.
- **Minimal repro:** hsbp.xi = use xiom.core; (brings the impls) + use xiom.sort.heap; + local cmp_int(a: &Int, b: &Int) -> Int + heap_sort_by(&mut hb, cmp_int) -- sorts wrong/AVs. Remove the use xiom.core; -> exit 0.
- **FIX (2026-08-18):** `callee_is_fn_ptr` (the bare-call dispatch) required the resolved fn key to be UNREGISTERED -- but impl-method registration aliases the bare method name ("Int.compare" also registers "compare"), so a fn-typed param named `compare` resolved to @Int.compare and the call bypassed the passed fn pointer. The local now SHADOWS the global for fn-typed params (fn_ptr_return_types) and closure locals: the call loads the param's i64 fn pointer and calls through it. Verified: user-space replica (impl Eq/Ord[Int] + local sort_by_cmp with compare-named param) sorts correctly (exit 0), hsbp shape (tower impls via xiom.math.tower + heap_sort_by) exit 0, m37_bug49_fn_param_impl_collision e2e.

- **Compress note (2026-08-18):** gzip/zlib decompress corruptions are this same family, NOT stdlib logic -- deflate (direct Result return of rle_decode) passes; local replicas of the full gzip pipeline (rle + intermediate decoded.value + re-wrap, probes cz1/cz2) pass in user space; the catalog intermediate Result[Vec[UInt8], Str] value read + re-return corrupts. The stdlib compress module is correct; the gzip/zlib smokes unblock when the catalog payload path lands.
- **Follow-up (2026-08-18):** READS through &[N]T params are fixed (array.len/array.get verified by the session), but WRITES are not -- n set_first(arr: &mut [5]Int, v: Int) { arr[0] = v; } still fails at clang (probe aw.xi). array.sort/sort_by (read+write insertion sorts) remain blocked. Also &[N]T slice-param passing (&mut arr from a ar arr = [1,2,3,4,5] literal) hits the same path.
### BUG 55 facet-2 - unsafe-block ctx capture of *T locals still corrupts cross-block reads

- The ctor-context facet of BUG 55 is fixed (rc/upgrade payloads OK), but the ctx-capture facet remains: a *T local written in one unsafe { } block and read via *(p) in ANOTHER block (or fn) still reads garbage (probe vg10: write p[0]=42 in block A, read *(p) in block B -- same fn, no calls -- exit 1; same-block reads exit 0).
- **Blocks:** Vec.get (Some(*(data + index)) -- payload corrupt -> cross_cmp_sort, smoke_core contains-style reads), json_set/stringify (enum-with-Vec-field payloads), and the compress gzip/zlib intermediate Result[Vec] path. The user-space replicas of all three (cz1/cz2/cb1) pass -- catalog-only.
- **Minimal repro:** vg10.xi (12 lines). The __unsafe_block_N(ctx) capture of the pointer local is misrouted; the fix likely belongs in the block-ctx builder (pointer-typed captures must store the pointer VALUE, not the slot address).
### BUG 56 - ensures clause before an EXPRESSION body drops the fn body (parser) -- FIXED (round-3 commit)

- **Construct:** fn identity_non(x: Int) -> Int ensures: result == x { x } -- the parser's CONTRACT expr parse consumed the fn's BODY block: `result == x { x }` parsed `x { x }` as a STRUCT LITERAL (name x, field x = x) -> fd.body = None -> the fn emitted `ret 0` / lost its param (convert.identity_Int -> store param; ret 0; smoke_stress_convert_identity exit 1). ALSO hit NON-generic fns (identity_non had zero params in the emitted def).
- **FIX (2026-08-18):** contract exprs parse with struct-literal restriction (the parse_cond pattern -- a trailing `{` starts the fn BODY, not a struct literal). Both generic and non-generic shapes verified.
- **Related CTFE discovery:** the decl.rs registration for CTFE bodies was structurally misplaced (nested inside the generic block, never running); the brace reconstruction exposed the engine's fall-through-after-if-return bug -- `if n <= 1 { return 1; }` then `n * factorial(n - 1)` kept evaluating -> unbounded recursion -> debug-build stack overflow on const CTFE. CtfeContext.returned now stops block/while evaluation after a return.### Compress gzip/zlib decompress -- catalog-only crash (2026-08-19 follow-up)

- After BUG 55 facet-2 + BUG 56 fixes: the intermediate Result[Vec] rewrap (cz2) and unsafe ctx capture (vg10) PASS in user space, and gzip_COMPRESS works in the catalog (gz9 exit 0), but gzip_DECOMPRESS crashes the program BEFORE the first statement (gz12 probe: println before the call never fires; exit 0xC0000005). deflate (rle_decode direct return) passes. The catalog mono of gzip_decompress (header scan + trailer UInt arithmetic + crc32 table lazy-init) is the trigger -- bisecting further needs the compiler session's IR-diff skills; user-space replicas of the same logic pass (cz1/cz2).
- Same family suspects: smoke_stress_io_parent_file_name (Path.parent returns None -- the ch.is_some { let c = ch.value; } payload-field pattern in a loop -- NOT the match-scrutinee shape the facet-2 fix covered) and smoke_array_sort_by (closure comparator -- B-007 documented layer).
### Compress gzip/zlib decompress -- catalog-only crash FIXED (2026-08-19, round-4 commit)

THREE coordinated root causes, all verified with fresh probes in tmp/bug_probes
(gz12/gz12d/gz12k now exit 0; gz12m2 payload bytes are real; e2e
`e2e_m37_gzip_roundtrip`):

1. **Result-payload FIELD access never unboxed (`decoded.value`).** The
   `let decompressed = decoded.value;` shape (payload-FIELD on an
   Option/Result LOCAL -- NOT a match scrutinee) bound the raw BOXED POINTER
   as an i64: `&decompressed` then passed the i64 SLOT ADDRESS as
   `%struct.Vec*`, so crc32 read stack garbage as len/elem_size -> 8-byte
   element loads -> 0xC0000005 inside gzip_decompress. (The "crashes before
   the first statement" claim was a stdout-buffering artifact: puts output
   was lost on the AV.) Fixes: new `field_payload_xiom` (lib.rs, mirrors
   scrutinee_payload_xiom: local_opt_payload -> local_opt_payload_xiom /
   local_err_payload -> the receiver's declared type), the Field arm in
   expr.rs now unboxes container/struct payloads (inttoptr + load
   %struct.Vec) and covers `is_result.value` (was missing -- only
   option.value/result.error reinterpretted), Option/Result PARAMS now
   track payload types in decl.rs (type_from_ast renders bare "Result"),
   and let/var bindings record the payload XIOM type via
   infer_field_payload_xiom (emitter.rs/stmt.rs) so `.len()`/indexing
   dispatch correctly.
2. **If-expression result slot hardcoded i64.** `let compressed = if
   level == 0 { _store_encode(data) } else { rle_encode(data) };` coerced
   the %struct.Vec VALUE to field-0-as-i64 (the data pointer): 
   `compressed.len()` became xiom_str_len(data_ptr) and `compressed[i]`
   compiled to a LITERAL 0 -> gzip payload of six zero bytes -> decode
   produced 3 zero bytes (and the CRC/size checks passed vacuously against
   the same broken crc32). Fix: the Expr::If result type is inferred from
   the arm tails (all-agree struct > pointer > all-agree float > i64), and
   infer_if_xiom_type records the arm-tail XIOM type on the binding.
   ALSO fixes Float64-valued if-expressions (`return if c { 1.5 } else {
   2.5 };` -- was bitcast-to-i64 + sitofp -> wrong value).
3. **Param payload tracking** (decl.rs) -- see #1; without it the
   `use_payload(r: Result[Vec[UInt8], Str])` param shape never resolved.

Verified: probe battery (gz9/12/12d/12e/12f/12h/12u/12m2, field_probe,
ifexpr_probe incl. Float64 arms) exit 0; stdlib smokes
smoke_compress / gzip_roundtrip / zlib_roundtrip / gzip_levels /
compound / deflate_roundtrip exit 0; checker 178, parser 97, ctfe 97,
feature-reg 510, stdlib-exec 70(+2 ignored), stdlib_tests 40.

REMAINING in this area (documented, NOT fixed here):
- smoke_stress_compress_gzip_large: pre-existing 0xC0000005 in __chkstk
  (huge stack alloca -- reproduces at baseline; separate queue item).
- smoke_stress_compress_gzip_empty / gzip_bad_input: the facade's
  `requires: data.len() > 0` / `>= 18` contracts PANIC instead of letting
  the fn return Err -- the smokes expect Err returns; stdlib-side decision
  (relax the requires or change the smokes).
- The [256]UInt module-global crc table lazy-init STILL writes the STACK
  COPY (module-global ARRAY element stores go to a copied alloca, never
  the global) -- the CRC is deterministic garbage but self-consistent for
  round-trips; gzip_crc32's public value is WRONG vs real gzip. Same
  family as BUG 2 (module-global field writes) but for array elements.
- The contract-ensure unbox (`result is Ok => result.len() >= 0`) loads
  the payload UNCONDITIONALLY -- for an Err result it inttoptrs 0 and
  loads from NULL (swallowed by the guard-fault trap today; latent).
### Round-7 finding FIXED (2026-08-20) -- inlined Vec.pop + match Option slot (probe ve2); Vec/Set/Slice method injection + pointer arithmetic

- **Construct:** inlined `Vec.pop` on an EMPTY vec returns Some at runtime
  while the emitted IR is fully correct (len==0 -> `Option{tag=0, payload=0}`)
  -- the CALLER's `match v.pop() { Some(x) => ..., None => ... }` misreads
  the inlined Option slot. Vec.get's None path works; a bare `return None`
  works; the pop builtin's own IR is right (verified: vec_pop_empty block
  stores tag 0). Blocks smoke_stress_collections_vec_edge and
  smoke_stress_collections_vec_first_last. The stdlib session's probe ve2
  has the IR evidence; user-space replica of the same pop+match shape
  passes -- catalog/inlined-context only (likely the inline-expansion or
  scrutinee-alloca resolution for the INLINE-builtin return, cf. the
  scrutinee machinery in lib.rs).
- **Next step:** build a minimal user probe with a fn returning
  `Option{tag=0}` that is `alwaysinline` and matched by the caller; diff
  the inlined vs non-inlined scrutinee alloca handling in expr.rs's
  match-check code.

**RESOLVED -- FOUR compiler roots (round-7 commits, e2e
`e2e_m37_round7_vec_pop_slot`):**

1. **Vec/Set/Slice methods were NEVER injected** (checker
   `collect_external_decls`): `recv_is_nonpub_generic` skipped methods on
   non-pub generic receivers -- correct for `BinaryHeap[T]`, but Vec/Set/
   Slice are COMPILER-BUILTIN generic types whose type decl is in
   PRIMITIVES (never injected), so ALL their methods vanished:
   - the six INLINE builtins (new/push/pop/get/len/sort/set) still worked
     at call time via call.rs inline handlers -- but `struct_type_from_expr`
     -> `resolve_struct_return` found NOTHING for bare "pop" (Vec methods
     register in NEITHER functions NOR generic_fn_decls) -> no scrutinee
     alloca -> the Some/None check blocks became unconditional branches ->
     the match always took the FIRST arm (Some on empty = ve2). Vec.get
     worked only by LUCK (the ".get" suffix happened to land on
     Map.get, also Option-returning).
   - every NON-inline Vec method (first/last/clear/reserve/extend/
     insert/remove/truncate/...) compiled to a ZERO-PARAM STUB
     (`define i64 @Vec.first() { ret 0 }`) -- smoke_collections_vec_
     first_last and every vec method beyond the six inline ones were
     broken. Fix: exempt PRIMITIVES receivers from recv_is_nonpub_generic
     (xiom-check/src/lib.rs) -- Vec/Set/Slice methods now inject and
     monomorphise like Map's.
2. **`resolve_struct_return` suffix search took the FIRST match and gave
   up** -- a ".pop" suffix could resolve to a non-Option-returning decl and
   return None even when a later match returned Option. Now scans all
   suffix matches for an Option/Result return.
3. **Mono'd Vec-method bodies broke on pointer arithmetic** (new exposure
   once Vec.first/last/insert/remove mono'd): the BUILTIN Vec type_meta
   registers `data` as "*UInt8" (i8*), so a mono'd body's `*(data + len -
   1)` (a) hit the Str+Int CONCAT intercept (xiom_str_concat of the buffer
   + len -- garbage) and (b) even past that, `*T + i64` lowered via
   ptrtoint/add/inttoptr -- element-UNSCALED + BYTE load. Three coordinated
   fixes: mono receiver-field binding types Vec.data as the CONCRETE
   element pointer (i64* for Vec[Int]; i8* kept for struct/enum/pointer
   elements) + records its XIOM type ("*Int") so the concat gate
   (`expr_is_pointer`) skips buffers; BinOp Add/Sub now emits
   `getelementptr` for pointer operands (element-scaled; Sub negates the
   index) in BOTH the main path and the iterative fold path; the concat
   intercept only fires when the i8* operand is a Str and the other side
   is an integer (Str+Str / Str+buffer keep concatenation).
4. **BUG 38b leaf-match hijack (regression caught by stdlib-exec
   smoke_collect_cache):** with Vec.push now registered, the is_generic
   leaf-name match made `g.edges.push(...)` (fn_key "Graph.push" -- base
   type of the FIELD receiver) resolve to the "Vec.push" decl -> mono'd
   @Graph.push_Int with a LITERAL-0 receiver -> clang "integer constant
   must have integer type". The leaf match now requires the fn_key's
   receiver part to be an ABSTRACT (unregistered) type -- exactly the rule
   find_generic_decl already uses for Iterator[T].collect. ALSO fixed
   `is_container_vec_field` to search ALL type_meta keys (two structs can
   share a leaf name -- collect.Graph vs math.graph_theory.Graph -- the old
   loop broke at the first suffix match and missed the other type's
   fields).

Verified: probes ve2/ve2b/ve2d/ve2e; smoke_collections_vec_edge,
smoke_stress_collections_vec_first_last, smoke_stress_collections_slice,
smoke_iter (leaf-match guard), smoke_collect_cache, 20-smoke Vec/collections
battery; stdlib-exec 70/70 (+2 ignore); e2e `e2e_m37_round7_vec_pop_slot`
+ round6/gzip regressions; checker 178, parser 97, ctfe 97, feature-reg 510.

FOLLOW-UPS (pre-existing, logged -- NOT regressions):
- smoke_collections_set_basic / set_ops: the compiler treats Set as an
  i64-handle CONTAINER builtin while the stdlib declares `type Set[T] =
  { items: Vec[T] }` (struct) -- `Set[Int].new()` resolves the bare "new"
  leaf to another type's ctor (Reverse.new/Vec.new by iteration order).
  Failing at baseline (different failure modes); needs a compiler-side
  decision on the Set container ABI or inline Set handlers.
- smoke_collections_vec_narrow exit 5: inline pop/get on SIGNED narrow
  elements (Int16 -30000) zext the bit pattern (35536) instead of
  sign-extending (emit_elem_payload_load) -- pre-existing, untouched.
  RE-CONFIRMED on the round-13 binary (2026-08-22): probe_narrow_zext.xi
  proves pop() returns 35536 for `-30000 as Int16` (pure zext, not sext);
  probe_narrow_get16.xi (Int16 get(1) negative) and probe_narrow_get8.xi
  (Int8 get(0) = -5) fail too -- the whole narrow-SIGNED inline get/pop
  read path is affected. Positives read back correctly.
- math/graph_theory.Graph vs collect.Graph bare-name type collision: the
  first registration (keep-first) wins the bare "Graph" layout; the other
  module's functions compile against the wrong field offsets (their
  `g.edges.push` stays a no-op stub -- graph_theory fns were dead code at
  baseline too).

### Round-6 findings FIXED (2026-08-19, round-6 commit) -- Imply short-circuit, Try-bound Str payloads, substr handler, Str-builtin receiver guards, path.xi import

FIVE coordinated root causes, all reproduced with fresh probes in
tmp/bug_probes (bi4/bi4u, op1, q3/q8, sw2/sw4, pj6/pj10 -- all exit 0; the
full path smoke family 19/19; e2e `e2e_m37_round6_path_gzip`):

1. **Imply consequence compiled UNCONDITIONALLY (bi4).** `ensures: result
   is Ok => result.len() >= 0` -- the old code compiled the consequence
   inline and masked it with `or (!l, r)`: for an Err result the
   consequence's payload UNBOX (inttoptr 0 + load %struct.Vec) executed
   anyway -> load from NULL -> UB -> the Err return value got corrupted ->
   gzip_decompress returned Ok for garbage input (the stdlib's bi4:
   "Err path never fires"). The Expr::Imply emission now short-circuits:
   the consequence compiles inside a guarded block (imply_conseq) with a
   DEFAULT TRUE store on the false path (imply_done reads the alloca).
   ALSO unblocked: smoke_stress_compress_gzip_bad_input (was blocked on
   bi4), and the stdlib's "Option[Str] payload construction mangles"
   report (file_name's `ensures: result is Some => result.len() > 0` was
   the same unbox-on-None corruption).
2. **Try-bound Str payloads (q3/q8).** `let name = io.file_name(p)?;` --
   the `?` binding stored the payload in an i64 slot WITHOUT recording
   the XIOM type; `name.byte_at(i)` then took the method-call receiver
   heuristic "p0 ends with * -> pass the SLOT ADDRESS" (i8* param of
   byte_at) -> the slot address was read as the string -> garbage -> the
   while loop never found the '.' -> io.extension/path.extension returned
   None. Fixes: infer_try_xiom_type records the payload type on `let x =
   f()?;` bindings (stmt.rs/emitter.rs), and the receiver-heuristic in
   call.rs skips Str receivers (xiom_type_of_local == "Str" -> coerce the
   i64 handle to i8* via inttoptr instead of passing the slot address).
3. **Str.substr had NO inline handler (op1/iop7/iop8).** The stdlib's
   substring form (`s.substr(0, i)` in io.parent_path, path.file_name/
   extension) was registered as a builtin Str method but the call fell
   through to normal dispatch -> `call i64 @Str.substr(...)` against a def
   that never exists -> zero-param stub `ret i64 0` -> NULL string ->
   Option[Str] payloads of 0 -> 0xC0000005 (the stdlib's "Option[Str]
   pointer-payload construction mangles"). substr now lowers to
   xiom_str_slice exactly like the existing slice handler.
4. **Str builtin handlers hijacked non-Str receivers (sw2/sw4/pp12).**
   `p.starts_with(b)` on a Path STRUCT matched the Str.starts_with builtin
   (name-only dispatch): the %struct.Path got BOXED (val_to_i8ptr struct
   path) and the BOX ADDRESS passed as the string -> always false (the
   stdlib's pp12 "alwaysinline returns wrong with correct IR"). All four
   handlers (slice/substr/starts_with/ends_with) now verify the receiver
   is really a Str (receiver_is_str: infer_llvm_type == i8*, or an
   Ident whose XIOM type is Str) and fall through to real method dispatch
   otherwise.
5. **path.xi called bare `join_paths` with NO import (pj6/pj10).** The
   catalog body's bare call resolved to no registered key -> the call-site
   fallback emitted the bare symbol -> zero-param stub `ret i64 0` -> NULL
   -> the join-chain smoke crashed (the stdlib's pj5/pj6 "join-chain AV"
   report was this stub, not an inline issue). path.xi now `use xiom.env;`
   (env.join_paths is FAMILY-aware) and path_separator() delegates to
   env.path_separator() (was hardcoded "/" while join produced "\" -- the
   smoke mismatch). The catalog-body type-check gap (item C) is what let
   the undefined bare name slip through silently.

Verified: probes bi4/bi4u/op1/q3/q4/q5/q8/iop4-9/sw1-4/pj1-10/wfn1-4 exit 0;
path smoke family 19/19 (incl. smoke_io_path, join, pop_clear,
starts_ends_with, with_extension, with_file_name -- the last three needed
stdlib smoke fixes: separator-aware comparisons and as_path().file_name());
compress smokes incl. gzip_bad_input green; checker 178, parser 97, ctfe 97,
feature-reg 510, stdlib-exec 70(+2), stdlib_tests 40; e2e pending.

STDLIB smoke fixes included in this round (examples/stdlib_smoke):
smoke_stress_path_extension (.hidden -> None -- Rust semantics, the impl was
right), smoke_stress_path_with_extension (separator-aware expectations),
smoke_stress_path_with_file_name (PathBuf has no file_name -- use
as_path()). smoke_core_option_map / smoke_core_option_unwrap fail at
BASELINE identically (B-007 closure layer -- unchanged).

### Round-6 findings (2026-08-19) -- path family

- Path.parent: the prose-style ensure (ensures: result is None => self.inner does not contain a parent directory) corrupted the fn (contract-eval Str FIELD read -- BUG 56 family) -- REMOVED (was documentation, not an expression) -- parent now returns correct values (smoke_stress_io_parent_file_name exit 0).
- PathBuf push/pop/clear took BY-VALUE self (stale note said &mut unsupported) -- pushes mutated a copy and were silent no-ops -- converted to &mut self (smoke_stress_pathbuf_push exit 0).
- Path.ends_with catalog alwaysinline returns wrong despite correct IR + working callee + working user-space replicas (probe pp12). The p.join("a").join("b").join("c") chain + Str comparisons in ONE fn AVs while each piece passes alone (probe pj5/pj6 exit 0, smoke AVs) -- a shape-dependent mono issue for the compiler session.
- &struct param field reads of Str-typed fields return garbage in user space (probe pp9/pp10 -- BUG 44 family extension) -- Path starts_with/ends_with switched to by-value Path as the stdlib-side dodge.
### Round-6 findings (2026-08-19) -- compress validation + Result field reads

- gzip_decompress([0,1,2]) returns Ok -- the validation reads (data.len() < 18, data[0] magic, trailer slices) miscompile for small inputs through the catalog (probe bi4: Err never fires; roundtrips of valid data pass). The bad-input smoke is correct (match-based) and blocked on this.
- Generic Result/Option first-field reads (esult.is_ok) return the wrong slot through the catalog while match works (bi3) -- a field-read-on-erased-container shape for the compiler session.
### Round-6 final finding (2026-08-19) -- Option[Str] payload construction

- Path.file_name returns an EMPTY name: the loop logic is verified correct (user-space replicas pass), parent (Option[Path] struct payload) works after its prose-ensure removal, but the Some(str_slice(...)) Option[Str] POINTER-payload construction through the catalog mangles the payload. The ensure is NOT the cause (removal doesn't change it). A remaining Option[Str]/pointer-payload shape for the compiler session -- file_name/file_stem/extension families blocked.
### Round-7 findings (2026-08-19) -- inlined Vec.pop + match Option slot

- Vec.pop on an EMPTY vec returns Some at runtime while the IR is fully correct (probe ve2: len==0 -> vec_pop_empty8 -> Option{tag=0,payload=0}; the match still takes the Some arm). pop on non-empty works; bare eturn None fns work; Vec.get's None works. The inlined-pop Option slot is misread by the subsequent match -- a slot-lifetime shape (single pop in the fn, probe ve2).
- Also logged: the char smoke failures were NOT multi-match slot reuse -- they were to_digit's redundant requires trapping (fixed stdlib-side, commit above).
### Round-11 findings FIXED (2026-08-20) -- B-007 closures (fn-typed params)

- core_option_map/unwrap/filter, array_sort_by, cmp_by, core_slice crashed
  (0xC0000005) or returned garbage: fn-typed PARAMS hold a closure ENV
  pointer (field 0 = the fn ptr) -- calling `f(x)` inside a body must go
  through the M20-A1 env path, and fn-REFERENCE args must be wrapped into
  envs with a forwarding thunk.

**RESOLVED (round-11 commit, e2e `e2e_m41_round11_b007_closures`):**

1. **Fn-typed params registered as closure locals** (decl.rs compile_fn +
   the mono body binding): the param was NOT in closure_locals, so `f(x)`
   inttoptr'd the ENV as a CODE pointer (0xC0000005 in Option.map). The
   param binding now registers Type::Fn params + their declared RETURN
   type (fn_local_returns) -- the M20-A1 call uses the REAL return type
   (struct returns are BY VALUE -- Option.and_then's closure crashed
   inttoptr'ing the by-value Option bits as a pointer).
2. **fn-REFERENCE args wrapped in a forwarding THUNK env**: passing
   `cmp_int` to a fn-typed param passed the RAW code address; the callee's
   M20-A1 loads field 0 = the first 8 bytes of CODE. The wrap emits
   `define {ret} @__fnwrap_N(i64 %__env, i64 %a0, ...) { inttoptr the
   params to the fn-ref's real types; ret call @fn_ref(...) }` and stores
   the thunk in the env. The thunk's signature MATCHES the M20-A1 call
   (all-i64 args) -- pointer params restored via inttoptr inside (the
   baseline sort's comparator call typed i64* matched the def; the first
   thunk version forwarded i64 to i64* params -- clang couldn't inline
   alwaysinline comparators and the heap sort corrupted the last pair).
3. **Re-pass double-wrap guard**: heap_sort_by -> heap_sift_down_by's
   `compare` arg ALSO matched the registered `Int.compare` suffix (the
   is_fn_ref suffix scan) -- the already-env param got wrapped AGAIN (env
   of env -> the inner M20-A1 read field 0 = the inner env ptr as code).
   is_fn_ref now excludes closure_locals.
4. **fn-typed container elements** (timer_wheel_tick's `var t = tw.tasks[i];
   t();` -- Vec[fn()]): the fn-marker arm of type_from_ast_with_args keeps
   "Vec[fn() -> Unit]" in the type_meta field strings; `var`/`let`
   bindings from fn-typed elements register as closure locals (M20-A1
   env-first calls); Vec.push of a bare fn-REF into a Vec[fn()] wraps it
   (uniform env convention -- the raw-address form crashed the indexed
   call); compile_index_fn_ptr_call (`tasks[i]()`) loads field 0 + env-first.
5. **Zero-arg thunks**: the trailing-comma `(i64, )` signature fixed.

Verified: stdlib-exec 70/70 (+2 ignore -- async/thread restored), e2e round
regressions, feature-reg 510, checker 178, parser 97, ctfe 97, 54-smoke
battery green incl. the full closure family + search/sort/async/thread.
Pre-existing (baseline-failing, unchanged): smoke_collections_btree_map
first_entry/last_entry (exit 7) + smoke_stress_collections_btreemap (exit 2).

### Round-10 findings FIXED (2026-08-20) -- checker builtin Ord/Bounded resolution (C001)

- num_checked/num_saturating stopped at `error[C001]: type 'Int' does not
  implement 'Ord': missing method 'cmp'` -- the stdlib's new Ord tower
  (compare+cmp on 15 types) did not register, and the math/interfaces.xi
  Ord interface adds lt/le/gt/ge/min/max on top of core.xi's compare+cmp.

**RESOLVED (round-10 commit, e2e `e2e_m40_round10_ord_bounded`):**

1. **Stdlib fix**: the round-10 Ord tower was written `impl Ord[] { fn
   compare(a: , b: ) ... }` -- EMPTY brackets and empty param types (the
   Eq/Bounded towers use `impl Eq[Int]` / `impl Bounded[Int]` with explicit
   types). Nothing registered; the C001 check had no "Int.cmp" to find.
   Rewrote the 14 malformed blocks as 15 properly-typed impls
   (`impl Ord[Int] { fn compare(a: Int, b: Int) -> Int ... }` etc.).
2. **C001 builtin fast-path**: "cmp" | "min" | "max" added to the
   is_builtin_method list -- min(a,b) = a < b ? a : b is derivable from the
   comparison ops for primitives (no tower impls exist for them).
3. **Inline scalar handlers**: cmp (the -1/0/1 select, same as compare) and
   min/max (select on lt/gt) in the is_scalar section -- the op match
   previously `unreachable!()`'d on these names.
4. **Generic-param static receivers**: `T.max_value()` inside a mono'd body
   -- "T" is a generic param, not a param name; receiver_is_instance(T) is
   FALSE (T is not a registered type), so the fn_key degraded to a bare
   "max_value" leaf -> zero-param stub -> 0 -> checked_add's overflow guards
   saw max_value = 0 and ALWAYS returned None. The fn_key construction now
   resolves Ident receivers through current_type_map (the T->concrete
   substitution recorded during mono body emission) FIRST -> "Int.max_value".
5. **Module-qualified impl suffix fallback**: the direct-call fallback only
   tried the bare leaf; the Bounded impls register as "precision.Int.
   max_value" -- the call resolved "Int.max_value" -> not found -> 0. The
   fallback now resolves the ".{Recv}.{method}" suffix (unique when it
   exists) to the module-qualified key.

Verified: e2e round-10/9/8/7 regressions, stdlib-exec 70/70 (+2 ignore),
feature-reg 510, checker 178, parser 97, ctfe 97; 35-smoke battery green
incl. num_checked/num_saturating/num/cmp_ordering and the BTree/Set towers.

### Round-9 findings FIXED (2026-08-20) -- Set container ABI mismatch

- The compiler had NO builtin Set layout (Vec/Slice/Map register %struct
  layouts in compile_program; Set was only the i64-erasure fallback in
  xiom_to_llvm_type), while the stdlib implements `type Set[T] = { items:
  Vec[T] }`. Every Set value erased to i64 while the injected methods
  operated on %struct.Set: `Set[Int].new()` hijacked Reverse.new (the
  type-receiver call degraded to a bare "new" leaf), Set-typed params/
  returns/fields compiled as i64 (make_set() -> i64, Holder.s -> i64), and
  `holder.s.insert(10)` mono'd Vec.insert with a %struct.Vec* receiver.

**RESOLVED (round-9 commit, e2e `e2e_m39_round9_set_abi`):**

1. **Inject the stdlib Set type**: "Set" removed from the checker's
   PRIMITIVES ("types that should never be injected") -- the stdlib
   `type Set[T]` now registers %struct.Set and Set resolves like any
   struct (methods were already injected since round-8).
2. **is_container_vec_field tightened**: the regular-field path matched
   ANY bracketed field type (`ftype.contains('[')`) -- a `Set[Int]` field
   fired the inline Vec.insert on the %struct.Set (invalid IR). Now only
   Vec/Slice/Array bases (Map/Set fields route through their own
   generic-mono methods).
3. **Field-receiver fn_key**: infer_struct_type_name's instance-field arm
   didn't split generic args -- `Set[Int]`/`Vec[Int]` fields degraded to
   None, so `holder.s.insert(10)` fell to a bare "insert" key (leaf-match
   -> Vec.insert mono). Split on '[' ("Set[Int]" -> "Set") so the key is
   "Set.insert".
4. **Mono pointer-self for FIELD receivers**: the pointer-self branch
   only handled Ident receivers (local slot / module global) -- field
   receivers passed the LOADED VALUE as the pointer. Now emits the field
   ADDRESS via the Ref arm (`&holder.s` -> GEP into the base struct).
5. **Pointer-len guard**: the "&Slice[Int] -> i64*: length at buf[0]"
   path fired for ANY pointer-typed receiver -- `s.len()` on a &Set param
   read the Vec's data pointer as the length. Skips when the pointee is a
   registered struct (falls through to the generic Set.len dispatch).

Verified: e2e round-9/8/7 regressions, stdlib-exec 70/70 (+2 ignore),
feature-reg 510, checker 178, parser 97, ctfe 97; 44-smoke battery green
incl. all six Set smokes + set_probe edge shapes (Set[Str], &Set params,
Set returns, struct fields, union/intersection). NOT covered: `for x in
set` iteration -- the For stmt is a hardcoded Range GEP (fields 0/1) and
needs the iterator-protocol work (iter adapter queue item).

### Round-8 findings FIXED (2026-08-20) -- catalog &mut-self receiver wiring + Option<&T> reference payloads

- **VecDeque/LinkedList/Stack/Queue/BTreeMap/BTreeSet mutations entirely LOST
  through the catalog** (probes vd2/vd6; identical code passes user-space in
  vd3-vd5): `dq.push_front(20)` left len at 0. Same family as the Set
  container gap.
- **Option<&T> reference payloads** (rw1): rand.weighted_pick returns the
  correct selection but the reference payload read gives garbage.

**RESOLVED (round-8 commits, e2e `e2e_m38_round8_catalog_mut_self` +
`e2e_m38_round8_ref_payload`):**

1. **Non-pub generic TYPE DECLS were never injected** -- the Type arm of
   `collect_pub_decls` only injected `pub` types, so VecDeque/Stack/
   LinkedList/Queue/BTreeMap/BTreeSet/BinaryHeap had no layout AND
   `generic_type_names` missed their receivers; their methods were skipped
   by `recv_is_nonpub_generic` (now dead -- removed). Calls hijacked
   same-leaf methods of OTHER types (`dq.push_front` -> Reverse.push_front;
   `VecDeque[Int].new()` -> Reverse.new -> dq typed %struct.Reverse). Fix:
   inject non-pub GENERIC type decls (`td.is_pub || !td.generics.is_empty()`)
   and drop the method skip -- the BUG 38b parser fix captures receiver
   generics, so every such method is mono-able; `recv_is_generic` blocks
   concrete direct-emission. Unblocked the whole collections family.
2. **Inline Vec/Slice handlers hijacked name-EMBEDDING types** (vd2):
   `recv_ty.contains("struct.Vec")` matched "%struct.VecDeque" -- `dq.len()`
   ran the inline Vec.len on the WHOLE VecDeque struct (extractvalue
   %struct.Vec on %struct.VecDeque -> invalid IR). Fix: `is_llvm_struct_named`
   -- leaf-name exact match (bare/qualified/args-embedded/pointer forms).
3. **`Option<&T>` payload auto-deref** (rw1): `Some(&items[i])` stores the T
   SLOT ADDRESS; consumers (strcmp content compare, icmp) treated it as the
   T VALUE -- the address bytes were strcmp'd -> garbage. The `&` was stripped
   from XIOM type records at THREE sites: `type_string_full` (fn_return_xiom
   "Option[&Str]"), param bindings (decl.rs `ref_preserving_name`), and the
   match payload bindings (guarded arm: scrutinee_payload fallback; unguarded
   arm: reference case + "&T" xiom record). `auto_deref_ref` now loads
   through &T-typed operands in Eq/Ne (bitcast for pointer forms, inttoptr
   for i64 bits) -- covers payloads, &Str/&Int params, and `&local`.
4. **Regression caught mid-round (geom + collect_cache):** with Vec.len now
   registered, indexed-element `.len()` (`ac[0].len()` on Vec[Vec[Float64]])
   typed the receiver as "%struct.Vec[Int]" -- `is_llvm_struct_named` rejected
   the args-embedded form -> inline handler skipped -> generic leaf-match mono'd
   a garbage "@Vec[Int].len_Int" symbol (brackets invalid in LLVM idents).
   Fix: `is_llvm_struct_named` splits container args ("%struct.Vec[Int]" ->
   "Vec"); VecDeque stays excluded ("VecDeque" = "Vec").

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, e2e round-8 fixtures + round-7 regressions, 39-smoke
collections/rand/geom battery green (incl. smoke_collections_set_basic and
smoke_rc_weak, previously failing).
### Round-12 residual closure/payload shapes (2026-08-21, post B-007)

The B-007 closure fix landed the Int-returning closure family (option map/filter/unwrap/deep_chain, and_then, core_slice, async, thread all exit 0). Three residual shapes remain, all with correct stdlib logic (probe-verified):
1. Str-returning closures through Result/Err construction: err.map_err(fn(e: Str) -> Str { str_concat("ERR_", e) }) produces a corrupted payload (rm1 probe prints garbage bytes). Int-returning map is fine.
2. Option[(K, V)] TUPLE payloads: BTreeMap.first_entry's Some((keys[0], values[0])) returns a corrupt tuple (the keys[0]/values[0] reads are verified fine standalone) -- smoke_collections_btree_map exit 7 / btreemap exit 2.
3. Ordering-returning closures with &T params: cmp.min_by's comparator (n(&T, &T) -> Ordering) returns the wrong Ordering (cb2 probe) -- the &-arg + enum-return closure shape.
### Round-12 findings FIXED (2026-08-21) -- rm1 Str-return closures + cb2 enum-variant receivers

- smoke_core_result_map/deep_chain/chains exited 5/4/4 -- `err.map_err(fn(e:
  Str) -> Str { str_concat("ERR_", e) })` produced a CORRUPTED payload
  (garbage bytes). The Int-returning map was fine.

**RESOLVED (round-12 commit, e2e `e2e_m42_round12_str_closures`):**

1. **Closure thunk params lost their XIOM types** (rm1): the Expr::Closure
   thunk stored every param as an i64 local (`add_local(name, alloca,
   "i64")`) WITHOUT recording the declared XIOM type -- a Str param used
   inside the body resolved to "Int" (via the i64 slot reverse lookup), so
   coerce_arg_for_param's materialize path fired: `alloca i8; trunc i64 to
   i8; store` -- the string HANDLE was truncated to a single byte and
   passed as i8* to str_concat (garbage payload). Fix (expr.rs Closure
   thunk): mirror decl.rs's param binding -- record local_xiom_types
   (ref_preserving_name keeps "&T"), ref_params for scalar-pointee &T
   params, signed_locals, param_locals. Verified in the IR: the body now
   emits `inttoptr i64 %e_0 to i8*` instead of `trunc i64 %e_0 to i8`.
2. **Mono fn_local_returns not substituted** (latent): the generic decl's
   `fn(E) -> F` ret was registered RAW ("F") -- llvm_type_for fails -- the
   M20-A1 call site degraded the fn-ptr signature to i64 (benign for
   8-byte returns, ABI-breaking for struct returns > 8 bytes). Fix
   (lib.rs mono body): substitute the Fn ret through type_map first.
3. **Module-qualified ENUM-VARIANT receivers dropped** (cb2, found while
   probing then_with/reverse): `cmp.Less.reverse()` emitted
   `call %struct.Ordering @Ordering.reverse()` with NO receiver arg, and
   `Ordering.then` lost its second arg -- infer_struct_type_name's Field
   arm had no enum-variant resolution, so receiver_is_instance returned
   FALSE and the receiver was skipped (undefined self at -O2 -- passed by
   luck in script mode, garbage in `-o`/e2e mode). Fix (lib.rs Field arm):
   resolve the variant's parent enum via resolve_enum_for_variant before
   the instance-field fallthrough. Verified: `-o` mode 0/0 on the
   reverse/then/then_with battery.

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, e2e round regressions incl. e2e_m42; the full closure
family green (result map/deep_chain/chains, option map/filter/unwrap/
deep_chain, cmp_by, array_sort_by, cb2 zero-arg + capturing + struct-T
probes). Pre-existing (baseline-failing, unchanged): smoke_collections_
btree_map first_entry/last_entry (exit 7) + smoke_stress_collections_
btreemap (exit 2) -- the tuple-payload queue item; smoke_error_edge +
smoke_iter_edge (exit 1 at baseline).

### Round-14 findings FIXED (2026-08-22) -- aggregate closure params + Vec[Str] elements + narrow-SIGNED loads

The stdlib session's round-13/14 sweep (819/907) surfaced three compiler
roots. All three are RESOLVED (e2e `e2e_m45_round14_aggregate_closure_
params` + `e2e_m46_round14_vec_str_elems_narrow`):

1. **AGGREGATE-typed closure params corrupt on call** (the ZipIter tuple
   predicates / EnumerateIter.map blocker): the closure thunk declared
   EVERY param as `i64` while the call site passed structs/tuples BY
   VALUE (a 16-byte %struct.Point splits across two registers; the i64
   thunk param read only the first -- p.x garbage, tuple elements bound
   literal 0, &Vec params garbage). Top-level fns were correct (decl.rs
   binds real param types). Fix (expr.rs Closure thunk + vec_abi.rs
   __fnwrap): params keep their REAL LLVM types when they cannot marshal
   through an i64 register -- struct/tuple values declare by value and
   bind typed (p.x GEPs the struct), struct-pointee refs declare
   %struct.X*; scalars/scalar-refs keep the uniform i64 convention.
   Verified: struct/tuple/&struct/&Vec params by value + match-
   destructure + fn-REFERENCE aggregate args (the __fnwrap path).
2. **Method call on a Vec[Str] ELEMENT emits invalid GEP** (context.xi
   pretty-print family): `v[0].len()`, `h.names[i].starts_with(..)` --
   the elem-load switch yields i64 so infer_llvm_type can't see the
   string; the Str builtin guards (len/slice/substr/starts_with/
   ends_with) EXCLUDED Index receivers (the 5c.30 Vec[Vec] handle rule)
   and the generic dispatch GEP'd the i8* receiver as a struct
   (getelementptr i8*, i8**, 0, 1 -- clang reject). Fix (call.rs +
   expr.rs): a shared `resolve_vec_elem_xiom(container)` (local_vec_elem
   / local_xiom_types "Vec[X]" / struct-FIELD type_meta) drives
   `is_vec_str_elem_receiver`; receiver_is_str now recognizes Vec[Str]
   element receivers so all Str builtins fire; the len handler only
   excludes i64-typed indexes (Vec handles) and casts the compiled value
   only when it is actually i64 (the elem path already returns i8*).
3. **narrow-SIGNED Vec loads zext** (queue item 3, re-confirmed by
   probe_narrow_zext): emit_elem_load hardcoded zext for 1/2/4-byte
   loads -- Int16 -30000 popped as 35536. Fix (vec_abi.rs): the loads
   take a `signed` flag from the container's element XIOM
   (vec_elem_signed via resolve_vec_elem_xiom -- Int8/Int16/Int32/Int64
   sext, UInt8../Char stay zext); emit_elem_payload_load (pop/get/remove)
   and the index-read paths pass it. smoke_collections_vec_narrow now
   PASSES (the old stress variant was renamed/removed by the stdlib
   session).

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, full e2e pending; the iter/cmp/closure family + the
new find/all/any/nth/last smokes (smoke_iter_find_all_any/nth_last/edge)
all green. PRE-EXISTING (unchanged, baseline-confirmed): smoke_iter_
collect + smoke_array_sort_by startup AV (clang -O2/MSVC-CRT layout
family -- queue 5/7), smoke_error2's has-mid layout flip (same family),
multibyte char len_utf8 (BUG 26 #7), the checker generic-tuple Str-
element typing gap.

### Round-13 findings FIXED (2026-08-22) -- closure-env family + tuple payloads

The closure-based iter adapter build (stdlib) was blocked on four
compiler roots. All four are RESOLVED (e2e `e2e_m43_round13_closure_
adapters` + `e2e_m44_round13_tuple_payloads`):

1. **Closure env STRUCT NAME collisions**: identical capture shapes in
   DIFFERENT mono fns redefined `%struct.__closure_env_N` (clang:
   redefinition of type -- smoke_iter_take_skip/chain/pipeline failed to
   compile). tmp_counter resets per fn (compile_generic_monomorphisations
   zeroes it), so every mono fn's first closure was `__closure_0`. Fix
   (lib.rs): a new GLOBAL `closure_counter` (never reset, mirroring
   unsafe_block_counter) names `__closure_N` / `__closure_env_N` /
   `__fnwrap_N` (expr.rs x3 + vec_abi.rs).
2. **Captured-state MUTATION did not persist**: the closure thunk copied
   captures into locals at entry -- `r.next()`'s implicit-self write only
   touched the copy, so the env's Range never advanced (Range.count/fold
   hung forever; take/skip countdowns never decremented). Fix (expr.rs):
   bind each capture DIRECTLY to its env-struct field GEP (reads load
   through, writes store through -- persistent across invocations).
3. **FN-TYPED FIELD calls compiled to zero-param stubs**: `self.next_fn()`
   resolved no fn for "{Struct}.{field}" and fell to @FilterIter.next_fn
   ret 0 -- the field holds a closure ENV (uniform convention), not a code
   pointer (0xC000001D in MapIter.next/FilterIter.next). Fix (call.rs):
   an instance-receiver fn-marker field ("fn(...)" in type_meta) loads
   the field (env bits), loads env field 0, and calls env-first with the
   field's declared return type (parsed from the fn-ptr field type --
   struct returns by value). Pointer-typed receivers GEP the pointee
   (ThreadLocal* tls_get shape -- the first cut emitted an invalid
   `getelementptr %struct.X*, %struct.X**`).
4. **TUPLE PAYLOADS through Option/Vec** (the first_entry family -- queue
   item 2): `Some((i, v))` bound the elements as literal 0 (the payload
   binding only handled Ident inner patterns -- the box was never
   deref'd); Vec[(Int, Int)] slots held 8 of 16 bytes (the push stored
   the box pointer; get() loaded a half-slot); "(Int, Int)" never
   resolved to the registered "Tuple__Int__Int" key; and the generic
   mono returns ("Vec[(Int, T)]") were rejected by the binding tracking.
   Fixes (stmt.rs/lib.rs/emitter.rs/decl.rs): (a) scrutinee_payload_xiom
   derives the payload from the RECEIVER's concrete Vec elem type for
   get/pop; (b) infer_call_return_xiom accepts mono-registered generic
   names (Vec[Tuple__Int__T] -- the registered layout is authoritative)
   and substitutes placeholders from the recorded mono instantiation;
   (c) substitute_type recurses into Named type ARGS + Type::Tuple;
   (d) resolve_vec_elem_type normalizes "(Int, Int)" ->
   "Tuple__Int__Int"; (e) the payload binding handles Tuple inner
   patterns (inttoptr the box, load the tuple, bind each element) and
   normalizes tuple names in the Ident-inner struct-deref arm.
   UNBLOCKED: smoke_collections_btree_map (exit 7 at baseline) now
   PASSES; enumerate/zip smokes pass.
5. **Enum-return scrutinees from closure calls**: `match compare(&a, &b)`
   (cmp.min_by) emitted NO discriminant checks (branched to the last
   arm) -- the scrutinee fallback only adopted enum_variants-registered
   names; Option/Result are structs. Broadened to any registered
   `%struct.X` (field 0 = discriminant/is_some/is_ok for all of them).

Stdlib fixes (iter.xi): the adapter TERMINAL helpers must call the
adapter's `.next()` METHOD (the raw `next_fn` closure bypasses map/filter/
take semantics); the chain delegates' next_fn consumed r2 AND second()
replayed its own copy of r2 (double-capture -- chain counted 9). Smoke
expectation fixes: smoke_iter_chain_zip (6 not 5 / 3 not 2),
smoke_iter_pipeline (225 not 729 -- take(5) of the squared multiples),
smoke_stress_collections_btreemap (`contains` -> `contains_key`).

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, full e2e pending; 19/20 iter smokes green +
btree_map/btreemap + the full closure family. PRE-EXISTING (unchanged,
baseline-confirmed via stash): smoke_iter_collect + smoke_array_sort_by
AV at STARTUP for the empty-range/combined shapes -- the documented
clang -O2 / MSVC-CRT layout miscompile family (m34_y15/y20, queue items
5/7) -- the combined m43-style module flips the CRT startup crash, so the
e2e fixtures are split to stay below the flip; smoke_iter_find_all_any /
nth_last / edge fail at the CHECKER (find/all/any/nth/last are not in
the stdlib iter API -- planned feature gap).

### Round-13 finding (2026-08-22, stdlib session) -- method call on a Vec[Str] ELEMENT emits invalid GEP

- **Construct:** any method call whose receiver is a Vec[Str] element
  expression -- `e.free[0].len()`, `items[i].len()` (both by-value Vec and
  `&Vec[Str]` params). IR: `getelementptr i8*, i8** %slot, i32 0, i32 1` --
  the element slot pointer (i8**) is indexed as if the STRING (i8*) were a
  struct -- clang: invalid getelementptr indices.
- **Probes (C:\Users\lefte\AppData\Local\Temp\kilo\):** probe_err_ctx_g.xi
  (minimal user-space repro, struct with Vec[Str] fields), probe_err_ctx_i.xi
  (&Vec[Str] param variant) -- both COMPILE FAIL. Workaround PROVEN:
  binding the element to a local first (`let s = v[i]; s.len()`) compiles
  and runs correctly (probe_err_ctx_h.xi).
- **Impact (stdlib):** context.xi's error_pretty_print/pretty_print_chain
  (FIXED stdlib-side with local binding -- smoke_error2 green), and latent
  in format/text.xi, textual.xi, fmt.xi text_columns/format_columns
  (FIXED, verified by probe_fmt_cols). Struct-element receivers
  (Vec[Vec[Float64]] `ac[0].len()`) compile fine -- Str (i8*) is the only
  affected element type (the BUG 44-era "Str receiver treated as struct
  pointer" family -- the round-6 fix #2 covered Str.to_str passthrough but
  not element-receiver method calls).
- **Fix direction:** when a method call's receiver is a Vec element of type
  Str, load the element (i8*) BEFORE lowering the self access -- the
  receiver currently resolves to the SLOT address (i8**).

### Round-13 finding (2026-08-22, stdlib session) -- smoke_error2 flips PASS/FAIL with unrelated stdlib code (layout family)

- **Observation:** smoke_error2 (chain + context + backtrace + io) fails
  at `chain.error_chain_has(e, "mid")` (exit 1) on the current tree, yet
  the IDENTICAL chain-section sequence passes as a chain-only program
  (probe_err_chain2.xi -- same chain.xi code, same ops). It also PASSED
  on the build right after the context.xi fix, and re-PASSES when
  error.xi's `wrap_error` is reverted to the pre-fix `[T, E: Error]`
  bound body (verified: "smoke_error2 OK", exit 0). The wrap_error fix
  itself is required (unblocks smoke_error_chain/edge) -- the chain
  module's has-mid behavior must not depend on it.
- **Construct:** error_chain_has loops `let m = e.messages[i]; if str_eq(m, message)`; str_eq compares via string.byte_at. Top/root/len checks in the same program pass; only the mid element comparison fails, and only in the full-program layout.
- **Conclusion:** same family as the documented clang -O2/MSVC-CRT layout miscompiles (m34_y15/y20, smoke_iter_collect/array_sort_by startup AVs, json flakiness): deterministic per source, flips with unrelated stdlib code in the program. Compiler-side (emitted IR for chain/str_eq/byte_at is layout-sensitive). stdlib-side code verified correct by the chain-only probe.
- **Probes:** probe_err_chain2.xi (chain-only, PASS), smoke_error2 with error.xi reverted (PASS), current tree (FAIL at has-mid).

### Round-13 finding (2026-08-22, stdlib session) -- multibyte char length via xiom_char_at reports wrong len_utf8

- **Construct:** `char.len_utf8(xiom_char_at("eOmega", 1))` returns != 2
  for the 2-byte UTF-8 char at byte 1 (probe_char_utf8.xi, exit 1). The
  runtime char reader / Char payload is byte-oriented (BUG 26 #7 family:
  chr() payload corruption). slice.str_chars (correct: advances by
  char.len_utf8) therefore returns the BYTE count for multibyte strings
  -- smoke_string_slice check 25 ("chars-5", str_chars("eOmega").len()
  != 2). stdlib code verified correct; the smoke comment documents this
  as BUG 26 #7.

### Round-14 finding (2026-08-22, stdlib session) -- typed [N]T let declarations lose the const N

- **Construct:** `let a: [3]Int = [1, 2, 3];` (annotated declaration with a
  literal) fails to compile: "unknown type ''", "unknown type '[N x T]'"
  warnings + clang reject `%tmp29 defined with type i64 but expected
  [3 x i64]` (probe_arr0c.xi). The UNINITIALIZED form `let a: [3]Int;`
  compiles but `array.len(&a)` returns garbage for ANY N (probe_arr0b:
  [3]Int and [1]Int both wrong; [0]Int wrong -- smoke_array_len_empty
  exit 3). Only pure-literal inference works (`let a = [1,2,3];` then
  len(&a) == 3).
- **Probes (C:\Users\lefte\AppData\Local\Temp\kilo\):** probe_arr0c.xi
  (annotated+literal -> COMPILE FAIL), probe_arr0b.xi (annotated only ->
  len garbage), probe_arr0.xi (smoke replica).
- **Impact (stdlib):** smoke_array_len_empty blocked; any user code
  declaring typed fixed arrays with const-N annotations is affected
  (the array/array.xi generic fns themselves are correct -- the const
  N param resolution is the gap).
- **Fix direction:** resolve `[N]T` type annotations in local
  declarations to the concrete const N (the checker/codegen appears to
  keep the unsubstituted '[N x T]' form); the literal-inference path
  already produces the correct type.

### Round-14 finding (2026-08-22, stdlib session) -- by-value self methods returning the SAME type write the result back into the receiver

- **Construct:** any method call `r = s.m(...)` where the method takes
  `self` BY VALUE and returns the SAME type as the receiver: the call site
  emits the result TWICE -- once into the lhs slot AND once into the
  RECEIVER's local slot (`store {res}, {recv_slot}`). The receiver
  variable is silently overwritten. Even a method that never reads self
  triggers it (`fn Dur.identity(self) -> Dur { Dur{secs:42,...} }`
  clobbers d1 -- probe_dur7 prints d1.secs=42 after
  `var r = d1.identity()`).
- **Probes (C:\Users\lefte\AppData\Local\Temp\kilo\):** probe_dur7.xi
  (identity clobbers d1; different-return-type methods are safe --
  to_other leaves d1 intact), probe_dur6.xi / probe_dur5.xi
  (`var r1 = d1.sum(d2); var r2 = d1.diff(d2)` -> r2 computes against the
  OVERWRITTEN d1: sum=8 then diff=5 instead of 2), probe_dur2.xi
  (stdlib-shaped replica: add=8, sub=5, div=2). IR evidence in
  probe_dur5.ir.ll: the spurious `store %tmp15, %tmp7` right after the
  call.
- **Impact (stdlib):** smoke_time_duration_ops (sub/mul/div wrong),
  smoke_time_duration_negative, smoke_stress_time_duration_add_sub,
  smoke_stress_time_duration_negative, smoke_time_normalize -- the whole
  time.Duration family (by-value pure methods). ANY module that calls a
  same-type-returning by-value self method and then reads the receiver
  again is corrupted. The error/chain "immutable push/pop" pattern
  (`e = error_chain_push(e, ...)`) is MASKED by the explicit lhs rebind
  (both stores hit the same slot) -- removal is safe for it.
- **Fix direction:** delete the unconditional result write-back into the
  receiver slot at the call site. The lhs assignment (`var r = s.m(...)`)
  and explicit rebinds (`s = s.m(...)`) are sufficient in every observed
  case; the write-back only corrupts.

### Round-14 finding (2026-08-22, stdlib session) -- generic fns with fn-typed params break on aggregate instantiations

- **Construct:** a GENERIC catalog fn whose parameter is a fn type
  (`fn _find_via[T](next_fn: fn() -> Option[T], predicate: fn(&T) -> Bool)`)
  called with an AGGREGATE instantiation (T = (Int, Int)): the mono'd
  fn-param forwarding corrupts -- the predicate always returns false /
  reads garbage from its argument (find returns None). The checker also
  DEGRADES the generic by-value form: `fn apply_g[T](f: fn(T) -> Bool,
  v: T)` instantiated with T = (Int, Int) infers `f` as `()` (error:
  "cannot logically negate type ()").
- **What WORKS (round-14 verified):** direct closure calls with
  struct/tuple/Vec params by value or &ref (probe_zip_f/g/e/c), and
  NON-GENERIC fns with fn-typed aggregate params (probe_zip_k Z1:
  `apply_v(f: fn((Int, Int)) -> Bool, v)` passes). The remaining gap is
  exactly the generic-mono path (probe_zip_k Z3 fails at runtime;
  probe_zip_j Z2 fails at the checker).
- **Probes (C:\Users\lefte\AppData\Local\Temp\kilo\):** probe_zip_k.xi
  (Z1 passes, Z3 fails), probe_zip_j.xi (Z2 checker degrade),
  probe_zip_h.xi (generic &T-predicate tuple case returns None).
- **Impact (stdlib):** ZipIter.find/all/any with `fn(&(T, U)) -> Bool`
  and the generic _find_via/_all_via/_any_via helpers stay blocked for
  tuple-yielding adapters (shipped in iter.xi; scalar adapters fine --
  Range/Map/Filter/Take/Skip/Chain predicate smokes green). The by-value
  predicate variant (`fn(T) -> Bool`) hits the same checker degrade, so
  no stdlib-side signature change can dodge it.
- **Fix direction:** the mono instantiation of fn-typed params must
  rebuild the fn type with the substituted concrete argument types
  (same substitution machinery as the round-13 mono-return fix), and the
  checker's inference must not collapse `fn(T) -> Bool` to unit when T is
  a registered tuple type.

### Round-13 finding (2026-08-22, stdlib session) -- aggregate-typed CLOSURE params corrupt on call

- **Construct:** any closure whose parameter is an AGGREGATE type -- user
  struct (`fn(p: Point) -> Bool { p.x == 3 }`), tuple (`fn(p: (Int, Int)) ->
  Bool { p.0 == 3 }`), or container (`fn(p: &Vec[Int]) -> Int { p.len() }`,
  by value or by `&` reference). The closure body reads garbage from the
  param (field access, deref, destructure, or len() all wrong). Top-level
  (non-closure) fns with identical shapes are CORRECT (probe_zip_d passes
  by-value tuple AND `&(Int, Int)` params), and scalar closure params work
  (the green iter smokes exercise `fn(&Int) -> Bool` predicates).
- **Probes (C:\Users\lefte\AppData\Local\Temp\kilo\):** probe_zip_f.xi
  (struct param by value + &ref -- W5/W6 fail), probe_zip_c2/c3.xi (tuple by
  value -- fail), probe_zip_b.xi (`&(Int, Int)` param -- fail), probe_zip_e.xi
  (match-destructure of tuple param -- fail), probe_zip_g.xi (Vec[Int] and
  &Vec[Int] params -- fail). Controls: probe_zip_c4.xi (local tuple field
  access -- PASS), probe_zip_d.xi (top-level fns, tuple and &tuple params --
  PASS).
- **Impact (stdlib):** the new iter find/all/any methods on tuple-yielding
  adapters (ZipIter.find/all/any with `fn(&(T, U)) -> Bool`, shipped in
  iter.xi this round) compile but cannot be exercised until this is fixed;
  the pre-existing `EnumerateIter[T].map[U]` (`f: fn((Int, T)) -> U`) has the
  same latent corruption (never called by any smoke). Any future stdlib
  closure taking a struct/Vec/tuple param is affected (e.g. fold accumulators
  of aggregate type).
- **Fix direction:** the closure thunk/`__fnwrap` call convention marshals
  aggregate-typed arguments incorrectly (probably the param's LLVM type in
  the thunk signature or the arg coercion at closure call sites); top-level
  fns demonstrate the correct lowering.
- **Not to be confused with:** BUG 46 (generic &UserStruct[T] PARAM field
  reads -- that's a non-closure generic fn shape, already fixed); this is
  the closure-call path only.

### Round-13 finding (2026-08-21) -- closure-env struct name collision

- **Construct:** multiple closures in one program with IDENTICAL capture shapes (e.g. Range.map's n() -> Option[Int] { r.next() } and Range.take's same-shape closure -- both capture one Range). The mono names both envs %struct.__closure_env_10 -> clang: error: redefinition of type. Reproduces with the new iter adapter build (smoke_iter_take_skip/smoke_iter_max_min/smoke_iter_pipeline fail at clang; map/filter run but crash 0xC000001D -- likely the same env-shape issue at runtime).
- **Impact:** the closure-based iter adapters (stdlib build) are blocked until the env naming dedups (reuse the first definition) or keys by site.
### Round-13 follow-ups (2026-08-21) -- closure-env mutation + cmp_by residual

- The closure-based iter adapters' Range.count/fold HANG (smoke_iter_count/smoke_iter_fold): the next-closure captures the Range by value; .next() inside the closure does NOT advance the env's copy (the implicit-self mutation doesn't wire through the closure env) -> infinite loop. Map/filter crash 0xC000001D + take/chain/pipeline fail at the env-struct redefinition (previous note) -- the whole env-in-catalog family.
- cmp_by STILL exits 1 here despite the round-12 battery claim (fn(&T,&T) -> Ordering closure returns the wrong Ordering; cb2 probe) -- the enum-variant receiver fix covered cmp.Less.reverse() but not the comparator closure's Ordering return through min_by.
### BUG 53 - &[N]T param element access emits invalid GEP -- read FIXED (9757e864) + WRITE facet FIXED (round-3 commit)

- **Construct:** n f(arr: &[5]Int) -> Int { return arr[0]; } -- the fixed-array reference param lowers to [5 x i64]** and element access emits getelementptr [5 x i64]*, [5 x i64]** %p, i64 0, i64 0 -- clang: invalid getelementptr indices. User-space probe (as2) reproduces; array.sort/sort_by and every &[N]T catalog fn is blocked.

### BUG 55 - unsafe-block context capture of pointer-typed variables corrupts reads (0xC0000005/wrong values) -- facet-2 FIXED (round-3 commit)

- **Construct:** write through a pointer inside one unsafe { } block, then read via *(p) inside a SEPARATE unsafe block (or fn): the read returns the wrong value. Same-block reads work (vg9 exit 0), cross-block reads fail (vg10 exit 1). The unsafe-block ctx-struct capture of pointer-typed locals is misrouted.
- **Root cause of:** Vec.get / Weak.upgrade / json_set / contains-style payload corruption -- the catalog fns construct Some(*(data + i)) inside unsafe and the CALLER's match reads the payload -- the caller-side read path (separate block/fn) corrupts. smoke_core_slice/cmp_by/cross_cmp_sort/rc/cell/json blocked.
- **Probes:** vg10.xi (minimal), vg4.xi (by-value struct + unsafe deref return), rcprobe3.xi (upgrade payload).
### BUG 43 - Float64 payload in generic Result/Option container read via sitofp (should itcast)

- **Construct:** a catalog fn returning Result[Float64, Str] (generic %struct.Result layout per BUG 41's primitive rule); the caller's match reads the payload as sitofp i64 - double while the definition stored it via itcast double - i64 - 3.14's bit pattern becomes ~4.6e18. IR: definition itcast double %x to i64 vs caller sitofp i64 %slot to double.
- **Impact:** smoke_core_convert exit 11 - 	o_float_from_str("3.14") reads garbage. ALL Float64 payloads in generic Result/Option containers are affected. The BUG 41 rule needs the READ side to bitcast for float payloads.
- **Stdlib:** core.xi's to_float_from_str is correct; no workaround possible from the stdlib side.

### BUG 44 - deref/coercion of &Str broken (loads a single byte)

- **Construct:** ar p = &s; var d = *p; (deref of &Str) - IR emits load i8 instead of load i8*; user-space probe AVs (0xC0000005). Passing &Str where Str is expected (auto-coercion) also miscompiles (compares a byte).
- **Impact:** any * on &Str, or implicit &Str-Str coercion, is unusable. Str is the only pointer-typed primitive; &Int/&Bool/struct derefs are fine. Blocks Eq-style impls taking &Self for Str.

### BUG 45 - method-form interface dispatch inside generic-bound fns resolves to a stub

- **Construct:** n f[T: Eq](x: T) -> Bool { x.eq(&y) } (method call on a T-typed or concrete-typed receiver inside a fn with an interface bound). Non-generic fns with the same call dispatch correctly (probe eqt12 exit 0); generic-bound fns return the stub's constant (eqt11/13 exit 1).
- **Impact:** core.contains/is_sorted (and every [T: Ord] user: cmp.min/max, sort.*) return wrong results. The tower pattern (EqT[T].eq(a, b) associated calls) works in user space - the stdlib conversion to that form was attempted and reverted pending BUG 47.

### BUG 46 - generic &UserStruct[T] param field reads return garbage

- **Construct:** n get_v[T](b: &Box2[T]) -> T { return b.v; } - reading a field of a USER-DEFINED generic struct through a & param returns garbage (42 != 42). By-value params and builtin &Vec[T]/&[N]T are fine; &Slice[T] (user struct with *T field) element reads break inside generic fns.
- **Impact:** blocks generic algorithms over &Slice[T] (contains/sort over slices). Distinct from BUG 45 (a plain == read fails too, probe eqt18).

### BUG 47 - impl method names collide with fn-typed params in mono (fn-param collapses to i64)

- **Construct:** impl Eq[Int] { fn eq ... } / impl Ord[Int] { fn compare ... } in the SAME program as a fn taking a fn-typed param (e.g. heap_sort_by(v, compare: fn(&Int,&Int)-Int)). The fn-param's mono collapses to a garbage i64 (IR: define ... heap_sift_down_by(%struct.Vec*, i64, i64, i64) - the fn param became an integer) - AV at the call. Impls named differently (ar) or for Str-typed receivers do NOT trigger it; renaming the fn-param to cmp does NOT fix it.
- **Impact:** the tower-style Eq/Ord impls (BUG 45 workaround) cannot land while fn-param fns (heap_sort_by, sort_by, min_by, merge_sort_stable...) exist - the stdlib conversion was reverted. Likely a symbol-key collision between impl method symbols and the fn-pointer trampoline naming for i64-ABI types.
### P001 indentation quirk (parser [U+FFFD] CAN HANG the compiler, priority for the compiler session)

- Indented module-level declarations (indented `use xiom.x;` + indented `fn main`) + a final column-0 `}` produce `error[P001]: expected declaration, found '}'` at EOF, while any one of those three properties removed compiles. Module-level `use`/`fn` at column 0 (repo convention) always works.
- **WORSE: the 2-space-indented variant HANGS the compiler indefinitely** (no error, no exit [U+FFFD] smoke_stress_crypto_hash_known_vector.xi hung the 904-sweep for 15+ min; the compiler session's own "batch 8 timed out" was the same family). The unindented copy of the same file compiles in ~14s.
- 158 smoke files in examples/stdlib_smoke were realigned to the col-0 convention on 2026-08-16 (stdlib session) [U+FFFD] zero indented `use` remain; re-check this note if a future sweep hangs.

---

## 2026-08-16 (late) -- compiler session: BUG 38 family + BUG 32/33/31/34/35 status

### BUG 38 -- FIXED (`9042e8a2`) -- is-Some double-check binds payload 0

The bare `is Some/Ok/Err` scrutinee payload rebind (the BUG 29 contract
convenience: `result is Some => result.len()`) fired in EVERY context. In
if/while conditions it poisoned the subsequent `match` on the same value --
the scrutinee read as an i64 payload, Some(v) arms bound 0 and skipped the
discriminant check (sx4: pos=0 instead of 5; iter.xi collect returned [0,0,0]
or len 0). The rebind now fires ONLY inside an Imply left side
(`in_imply_lhs` flag). The checker never rebinds, so codegen now matches it.

### BUG 38b (NEW, same commit) -- generic-receiver methods never monomorphised

`Iterator[T].collect(self)` (and 44 other decls in iter/core/collections/
sync/rc/memory) had their receiver generics DROPPED by the parser -- the decl
registered with an erased i64 receiver ABI and never entered the mono
machinery (iter smokes 0/19, all 16 in the stale runfail list). Four-part fix:
1. **Parser** (`xiom-parser`): receiver type-param names merge into
   fd.generics (receiver-first, deduped) -- `Iterator[T].collect` ==
   `Rc.get[T]` convention.
2. **Mono dispatch** (`call.rs`): receiver-keyed generic calls
   ("Range.collect" <-> decl "Iterator.collect") enter monomorphisation via a
   leaf-method match; `find_generic_decl` prefers UNDECLARED abstract
   receivers ("Iterator") over concrete types that merely share the leaf
   ("HashMap.count" no longer hijacks "Range.count" -- the wrong decl's
   [K,V] generics produced `Range.count_Int_Int` with a `%struct.HashMap*`
   receiver).
3. **Emission** (`lib.rs`): the concrete receiver LLVM type derives from the
   mono key (`mono_receiver_part`/`receiver_llvm_type`); container returns
   (Vec[T]) keep `%struct.Vec` instead of the erased i64.
4. **Mutating self ABI**: `block_mutates_self` now recurses into if/while/
   for/match bodies, and a new `block_mutates_receiver_state` catches BARE
   receiver-field assignments (`start = start + 1` in Range.next) -- both
   flip the ABI to BY POINTER (value copies silently dropped the mutation;
   iterators looped forever on the first element).

Verified: sx4 pos=5, `iter.range(1,4).collect()` == [1,2,3], user-space
collect/count shapes, checker 178/178. iter smokes 0/19 -> 6/19 (the
remaining adapter-chain failures -- receiver-CALL chains like
`iter.range(1,6).max()` and fn-value params -- reproduce identically at
baseline HEAD; B-007-adjacent).

### BUG 32 -- FIXED (`74bcc28b`) -- Int-var -> ptr cast emits address-of-local

`var h = buf as Int; var q = h as *UInt8;` emitted `bitcast i64* %slot to
i8*` -- the ADDRESS OF THE LOCAL. The As-cast arm now distinguishes the
`&x as *T` address-of form from the bare-Ident VALUE form (load + inttoptr).
Verified: pdb3 eq=OK/read=OK/from_cstring=OK.

### BUG 33 -- FIXED (`74bcc28b`) -- Option[Float128] unwrap loads opaque `%struct.Float128`

The concrete Option__T builtin impls hardcoded `%struct.{payload}` for
non-Int payloads. The payload LLVM type now resolves through llvm_type_for
(Float128 -> fp128). Verified: match-Some and unwrap() on Option[Float128]
both return 42.

### BUG 31 -- FIXED (`74bcc28b`) -- unary minus on Float128 emits `sub i64 0, fp128`

fneg now covers fp128 (was double/float only). Verified: -a + -2.0 == -3.0.

### BUG 34 -- FIXED (`b15d0d3b`) -- nested Vec[Vec[T]] element writes

`bs[i].push(x)` lost the mutation (and corrupted the buffer) through three
gaps: (1) dispatch -- `bs[0]` compiled as i64 so the inline push path missed
the Vec receiver (call fell to a bare `@push` stub); (2) receiver ABI --
resolve_vec_receiver_ptr inttoptr'd the LOADED element value for i64
receivers instead of GEP-ing the element address in the outer buffer;
(3) slot math -- Vec.new() defaulted elem_size to 8, so 32-byte Vec elements
overwrote 4 slots and reads used wrong offsets. Struct-element pushes now
use the struct's real byte size, persist it into field 3, and record the
nested element type so reads take the struct-element (memcpy) path.
Verified: bs[0].push(5) -> bs[0][0] == 5.

### BUG 35 -- primary shape VERIFIED WORKING (no crash, correct buckets)

The Int128 index-math + Vec-write shape (radix-sort distribution) compiles
and runs correctly -- counts[0..2] == [1,1,1] for (100,200,300)/min=100/
width=100. The documented 0xC0000005/0xC000001D variants did not reproduce;
likely resolved by the BUG 38/34 batches. The extreme-i64 variant needs the
stdlib session's exact repro to re-verify.

### BUG 37/36 -- FIXED (`feat/architect`, 2026-08-17) -- fp128 libcall ABI mismatch

**Root cause (NOT a shape miscompile):** the IR clang generates for fp128
arithmetic calls the soft-float helpers with LLVM's Win64 f128 convention
(f128 args by POINTER: `__addtf3`: rcx=&a, rdx=&b; f128 RESULT in XMM0),
but fp128_helpers.c compiled the helpers as `xiom_f128` STRUCT functions,
which get the MSVC ABI (hidden sret in rcx, args shifted to rdx/r8, result
written through sret, XMM0 never set). Every f128-returning helper was
therefore ABI-mismatched: the callee dereferenced r8 = garbage (0xC0000005
in every RUNTIME fp128 shape -- the chain was a red herring; it only
prevented clang -O2 from constant-folding the loop). The earlier
"working" fp128 verifications (t_fneg, smoke_d1_native128, BUG 31/33)
all had CONSTANT-FOLDABLE shapes -- clang -O2 eliminated the libcalls.

**Fix (stdlib/runtime/fp128_helpers.c):** the 13 f128-returning helper
symbols (`__addtf3 __subtf3 __multf3 __divtf3 __negtf2 __extenddftf2
__extendsftf2 __floatsitf __floatunsitf __floatditf __floatunditf
__floattitf __floatuntitf`) are now naked-asm shims implementing the IR
convention (arg pointers as the IR passes them, result in XMM0) that
forward to plain sret-ABI wrappers around the existing pure by-value
implementations. SSE2-only instructions (movdqu/movaps) so the JIT build
(no -mavx) works. Scalar-return helpers (__trunctfdf2/__fixtfdi/compares)
already matched. Also ADDED the previously-missing `__fixtfti` /
`__fixunstfti` (f128->i128) -- those were latent link errors.

**Verified:** all 14 t_b37* shapes + t_chainloop exit 0 with CORRECT
values (t_b37e `bigfloat_to_float128(&v)` non-zero -> 42 val=OK; t_b37u
user-space BigFloat Horner -> 42). New runtime test t_f128rt (Vec-loaded
values: add/sub/mul/div/neg/trunc/compare all exact) -> OK. Suites:
checker 178/178, codegen 2263, feature-reg 510, parser 24 -- all green.
`stdlib_api_freeze_no_removals` still fails IDENTICALLY at baseline (item
B family -- 905 stale paths, NOT a regression).

**Note for the stdlib session:** the bigfloat_to_float128
three-single-shape-fn workaround can be consolidated now; the Option
variant (BUG 33 follow-up) can land; the "TODO(compiler)" notes in
num/bigfloat.xi, sort/radix.xi, misc/glob.xi for the fp128/ptr-cast
families can be re-checked (BUG 32 is fixed too).

### BUG 37/36 -- PRE-FIX analysis record (shape AV era)

Minimal deterministic repro (user space, no catalog needed):
`var n = v.significand.digits.len();` (BigFloat field-chain Vec len) used as
a loop bound + ANY fp128 op in the loop body -> 0xC0000005, even at clang -O0,
with verifiably sound IR (t_chainloop/t_b37f/t_b37k in the compiler session's
scratch). `bigfloat_to_float128` (catalog) with non-zero values crashes in
every consumer shape -- the stdlib's three-single-shape-fn workaround did NOT
fully dodge it (the main fn still mixes chain + fp128). fp128 arithmetic
without the chain, and the chain without fp128, both work. The loop's
semantics even shift when a probe print is added -- because a probe print
breaks clang's constant folding, exposing the libcall ABI fault (the "shape
miscompile" was the folding/unfolding flip, not a miscompile).

### P001 -- NOT REPRODUCED (resolved by the stdlib realignment)

Indented module-level decls now reject in 0.14s (clean P001 error, no hang);
the former 15-min hang case (smoke_stress_crypto_hash_known_vector) compiles
in 4.9s and runs exit 0. The 238-file re-sweep completed with no hangs.

---

## 2026-08-17 -- compiler session: BUG 37/36 follow-up batch -- IR determinism, BUG 39, Item B

### BUG 39 -- Vec element-type records: push overwrite + ctor elem_size (FIXED)

Two coordinated bugs made `Vec[Struct]`/nested-Vec programs fail ~85% of
builds (layout-dependent):

1. **Push-time element-type overwrite** (call.rs): the BUG 34 nested-Vec
   recording (`local_vec_elem[recv] = "Vec[{inner}]"`) fired for EVERY
   struct-element push. For `Vec[Item]` receivers the arg-based inner lookup
   resolved the struct literal to None -> "Int" -> the record became
   `"Vec[Int]"` OVERWRITING the binding's correct `"Item"` -- so `v[0]` took
   the nested-Vec memcpy path and loaded a 16-byte Item into a %struct.Vec
   slot (garbage field reads; m21_vec_edge_012/027, vec_of_struct, eco
   suites). Fix: record only when the receiver's recorded elem is a nested
   Vec OR unrecorded (`Vec[Vec[Int]].new()` -- vec_ctor_elem_type cannot
   resolve nested type args, so the push is the first chance to record).
2. **Ctor elem_size under-counts container fields** (call.rs/stmt.rs):
   `Vec[Vector].new()` sized elements with struct_byte_size (XIOM-semantic
   field-count x 8) -- a `Vec[Float32]` FIELD counted as 8 instead of 32, so
   `Vector` got elem_size 16 instead of 40. The ctor's initial buffer
   (16 x 16 = 256B) overflowed once 40-byte elements were stored
   (test_vector/test_db/test_json 0xC0000005). Fix: new
   `vec_elem_storage_size` -- real byte size with container fields counted
   fully (Vec 32, Map/Set 64, Option 16, Result 24, arrays N x inner).

### BUG 40 -- nondeterministic IR emission (type_meta + mono order) (FIXED)

`type_meta.entries()` and `generic_instantiations` are HashMap-backed; their
iteration order varies per process, so the emitted IR (struct declaration
order, mono function order) differed BETWEEN BUILDS of the same source.
clang -O2's codegen of the linked MSVC CRT objects is layout-sensitive:
identical sources built passing vs crashing/wrong-result binaries. This
explained the session-long "flaky" failures (m21_vec_edge_012 exit-1,
smoke_simd 0xC0000005, eco suites, even a CRT-internal call reading an
uninitialized r9d). Fix: sort both emission loops by key. The output is now
byte-reproducible (verified 5x identical for m21/test_vector/m34).

### BUG 41 -- FIXED (`feat/architect`, 2026-08-17) -- mono call-site ret type vs concrete Result payload

`fn adjust[T](e: Env, target: Climate) -> Result[Env, Str]` (generic with an
UNUSED type param) mono'd to adjust_Env: the DEFINITION emitted
`%struct.Result__Env__Str` (the mono's subst_type resolves concrete
Result/Option keys), but the CALL-SITE fallback (call.rs generic_ret,
used when the specialized key isn't registered yet -- the `-o` compile
order compiles the caller first; `xiom run` happened to compile the def
first, masking the bug) produced the generic base `%struct.Result` ->
`call %struct.Result @adjust_Env` + `ret %struct.Result__Env__Str` ->
clang rejected the IR (m34_y15/y20 compile failures).

Two root causes, both fixed:
1. `type_from_ast` drops Named args and renders `Type::Result` as bare
   "Result" -- the call-site fallback now renders the FULL
   "Result[Env, Str]" (Named-with-args, Type::Result, Type::Option arms).
2. `concrete_container_llvm` (new) mirrors concrete_type_for's rule:
   concrete `%struct.Result__A__B` iff at least one payload is a STRUCT
   (primitives/enums keep the generic layout, matching the definition);
   falls back to the bare concrete name when the key isn't registered yet
   (a struct payload guarantees the def emits it). Verified via both the
   `run` and `-o` (harness) paths: m34_y07/11/13/15/16/19/20,
   m35_z02/09/24/29, m21_complex_generic_008/009 all exit 0.

### BUG 42 -- FIXED (`feat/architect`, 2026-08-17) -- enum values in Vec elements

`Vec[JsonValue]` / structs containing enum fields (JsonEntry
{ key: Str; value: JsonValue }) were stored/read as i64 SCALARS (tag
only, payload garbage):
1. **Push path**: is_struct_elem checked only `types` -- enums register in
   `enum_variants`, so a JsonValue push took the scalar store (tag, and
   the payload slot stayed zero) -> as_number read garbage bits ->
   float_to_string crashed in the CRT (shld on a garbage exponent).
2. **Read path**: resolve_vec_elem_type had no enum_variants fallback ->
   Vec[JsonValue][i] fell to the scalar i64 load instead of the struct
   memcpy.
3. **Storage size**: vec_elem_storage_size had no enum arm -- JsonEntry
   (with the JsonValue field) sized 16 instead of 24; the push's
   sizeof_struct also under-counted enum fields (8). The enum layout is
   `{ i64 tag, i64 x slots }` (payloads boxed to i64 handles; tuples one
   slot per element) -- 16 bytes for JsonValue, 24 for JsonEntry.
   ALSO: qualified field names broke the leaf-suffix lookup (ColumnDef's
   affinity -> 8 -> elem_size 18) and Bool FIELDS are 8 bytes (only
   Vec[Bool] ELEMENT slots are 1 byte) -- ColumnDef sized 18 instead of 40
   (test_sqlite wrong results).
Fix: is_struct_or_enum_type helper used by both push checks; the push's
struct esz now uses vec_elem_storage_size uniformly (structs, enums,
containers, arrays); resolve_vec_elem_type falls back to enum_variants
keys; leaf-segment matching for qualified names; Bool-field width 8.
Verified: test_json (29), test_db (18), test_vector (32), test_sqlite
(23), test_test (20) all exit 0.

### BUG 40-era harness hardening (2026-08-17)

The e2e harness's compile_and_run_once now deletes the target exe before
compiling -- a STALE exe from an interrupted run held the output path
open and clang failed with "permission denied" (e2e_main.exe,
e2e_t3-hot-reload.exe -- previously misread as real failures).

### Item B -- exec harness wiring (DONE)

- `stdlib_api_freeze_no_removals` 905 -> 0 missing: (1) module paths now
  resolve via STDLIB_MANIFEST.md (exact `xiom.X` then unique last-segment
  match, then filesystem scan) -- the 2026-08-16 folder refactor broke the
  flat `stdlib/xiom/{module}.xi` join; (2) the signature extractor's
  comment-stop is now indentation-aware (crypto.xi documents "no requires
  clauses" BETWEEN the sig and `{`).
- `stdlib_execution_tests` 72 -> 70 pass + 2 documented #[ignore]s:
  smoke_core (stdlib-side: `interface Eq` has ZERO impls anywhere -- add
  `impl Eq for Int/...` in core.xi), smoke_simd (BUG 40-era latent CRT
  layout miscompile; re-enable after a toolchain investigation).
- Fixed en route (verified): Vec[Str] element reads now inttoptr the loaded
  handle to i8* (yaml_emit_sequence garbage); null/dangling inline results
  are recorded as pointer-valued so is_null coercion inttoptrs instead of
  materializing a temp address; smoke_ptr/serialize/misc/net_folder pass.

### Verified after the batch

- checker 178/178; stdlib_tests 40/40; api_freeze 2/2; execution 70/70+2
  ignores; e2e 2256/2263 with the 7 remaining = 2 hot-reload file-lock
  artifacts + m34_y15/y20 (BUG 41) + eco db/json/vector (BUG 41-adjacent
  container/Result shapes; were flaky at baseline too).
- Repro battery: t_chainloop, t_b37* (14 shapes), t_f128rt, t_f128i128,
  sx4, t_iter38, pdb3, t_b33, t_b34, t_b35, t_fneg_clean -- all exit 0.
- Reminder: `xiom run` caches binaries in ~/.xiom/jit/<sha>.exe keyed ONLY
  by source hash -- delete them after ANY runtime C change or results use
  stale runtimes.

### Re-triage numbers (current isolated binary, 2026-08-16 late)

238 files from the stdlib session's two remaining lists -> 43 pass, 82
compilefail, 113 runfail. Known stdlib-side: smoke_num_saturating (Bounded/
Ord impls), smoke_alloc_basic (`use xiom.ptr;`), char.from_digit contract
false-fire (contract on a graceful-fallback fn -- stdlib rule BUG 22 #5),
smoke_collections_vec_push_pop (is_empty broken -- reproduces at baseline),
Vec.get usage in stale smokes (not a Vec method). Remaining compiler-side
families cluster in: receiver-CALL chains for generic methods (iter
adapters), the BUG 24/36 shape family, and the stale-API smokes.

---

## 2026-08-16 (late) -- compiler session: BUG 31 batch -- fmt, Map, variant-hijack, tuple destructure all fixed

The compiler session's queue from the 904-sweep. Commits:
`2b238da4` (fmt), `b28ac72b` (Map), `a7571ac7` (variant hijack), `99f894b7` (tuple destructure).

1. **Unit fields in `Result[Unit, FmtError]` literals -> `store void 0, void*`**
   (invalid IR): generic-name literals now resolve to the CONCRETE
   instantiation via the fn's return type; field types degrade Unit->i64
   (field_llvm_type + ctor sites + the 5c.39 value adoption never takes
   void). **Fixes smoke_fmt_align/formatter/format1-3/edge/float.**
2. **Str.to_str passthrough read the first BYTE of the string**: the
   prologue treated i8* receivers as struct pointers (self registered as
   the pointee "i8"); only `%struct.X*` receivers take that branch.
3. **`v.to_str()` on primitive locals -> bare `@to_str` stub**:
   infer_struct_type_name now resolves literal receivers (incl. negated +
   paren-wrapped), primitive-typed locals via local_xiom_types, and
   monomorphised generic params via param_concrete_types;
   infer_value_xiom_type learns literals.
4. **Mutating `self` methods (Formatter.write_int's `self.buf = ...`) lost
   the mutation**: block_mutates_self pushes the POINTER ABI in compile_fn
   AND the signature registration.
5. **Map[K,V] `&K` params (get/contains/remove) passed the VALUE**: four
   coordinated fixes -- param_llvm_type Type::Ref -> pointer for scalars;
   mono subst_type Ref -> pointer arm; generic-call inference &T scalar ->
   pointer; coerce_arg_for_param materializes plain VALUE args into temps
   for pointer params. **Fixes all 7 smoke_stress_collections_map_*.**
6. **Struct literal vs enum-variant name hijack**: `Node{...}` (a STRUCT)
   was hijacked by `enum BST[T] { Node(...) }`'s Node VARIANT -- the
   bare-variant search now only fires when the name is NOT a known type.
   **Fixes bench_math's native IR (was store %struct.BST %vecval).**
7. **Cross-module tuple destructuring** (`var (a, b) = pair()`): the
   checker bound every name to the WHOLE tuple -- now splits the
   Tuple__A__B element types. **Fixes BUG 26 #3.**

**Verified fixed this session:** BUG 26 #2 (bare prelude names -- repro
passes), BUG 26 #5 (high-bit mask AND -- repro passes), BUG 24 residual
(smoke_num_precision OK).

**Remaining compiler queue (documented):** BUG 32 (Int->ptr cast emits
address-of-local), BUG 33 (Option[Float128] unwrap opaque struct name),
BUG 34 (nested-Vec element WRITES), BUG 35 (Int128+Vec-write shape AV),
BUG 36 (fp128 Horner shape AV), BUG 37 (fp128 catalog-return AV),
BUG 38 (is-Some + match double-check binds 0 -- BUG 30 #1/#2 family).

**Stdlib-side (for the stdlib session):** smoke_num_saturating needs real
Bounded/Ord impls (generic bounded fns, no impls -- interface dispatch
itself verified working); smoke_alloc_basic needs `use xiom.ptr;`;
smoke_hash_folder needs `use xiom.convert.toint;`.
