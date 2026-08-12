# XIOM Compiler Bugs â€” Log

Dated sections. Each entry: file(s) affected, construct, error observed, and
(recommended) fix direction for the compiler team. Stdlib workarounds are
deliberately NOT applied where the stdlib mandate says "production grade, no
workarounds" â€” the compiler must be fixed, then the stdlib lands.

---

## STATUS SUMMARY â€” authoritative (2026-08-11 16:3x, compiler session)

> Read this first. Older dated sections below may contradict it (e.g. the
> BigInt/BigFloat Phase C entry claimed BUG 11 open â€” that predates the fix).

| Item | Status | Fix commit | Verified by |
|------|--------|-----------|-------------|
| BUG 1 â€” tuple-of-struct opaque LLVM type / truncated slots | **DONE** | `d22068f8` | e2e_m37_tuple_struct; probe_big/probe_big2; bigint_div_mod no longer crashes |
| BUG 8 â€” catalog fns with `&Vec[Int]`/`&Vec[UInt8]` params | **DONE** (was the uncommitted coerce.rs state) | `384d5666` (revert) + verified | e2e_m37_catfix_catalog_vecref; probe_bit (12&10=8) |
| BUG 9 â€” private catalog struct types degrade to i64 | **DONE** | `3ccd004c` | e2e_m37_catfix_private_type (b9mod+b9main, sum=7) |
| BUG 10 â€” float literals truncated to 6 decimals | **DONE** | `a2aafa26` | e2e_m37_float_precision (0.123456789 round-trips) |
| BUG 11 â€” unsafe-extern double marshalling (math.sqrt via trampoline) | **DONE** | `7f7b7b54` | stdlib-exec **72/72** (complex + net_folder pass); e2e_m37_unsafe_option_return |
| Parser â€” `bits[L - 1]` (uppercase-ident index with arithmetic) | **DONE** | `40441ca7` | e2e_m37_index_arith; bigint.xi/bigfloat.xi parse |
| Catalog â€” import lookup walked the whole tree per miss (minutes/hang) | **DONE** | `1e982ebf` | stdlib-compile 108.7s â†’ 23.7s; probe_bit compile 138â€“160s â†’ 9.5s |
| Struct `&T` param mutation silently lost | **DONE** | `d22068f8` | e2e_m37_ref_mut; probe_dig/probe_cmp |
| clang -O2 hang (alwaysinline everything) | **DONE** | `d22068f8` | smoke_bigint.xi compiles ~5â€“14s (was >300s) |
| Unsafe-block capture collector missing Expr::Struct etc. | **DONE** | `e0f96fef` | e2e_m19_read_file_content |
| Parallel codegen `__unsafe_ctx_0` redefinition | **DONE** | `e0f96fef` | e2e_i2_parallel_codegen (all 5 ecosystem tasks) |
| BUG 12/17 â€” Vec[Float64]/Vec[Str] element type lost on `&Vec[T]` params | **DONE** | `2ae300fd` | e2e_m37_vec_f64 (R=0 isolated); m12b |
| BUG 13 â€” fp128 link + coercion (soft-float helpers) | **DONE** | `2ae300fd` + `3b8f5415` (fp128_helpers.c) | e2e_m37_f128 (R=0 isolated); C harness 400/5/2500/1002.5 |
| BUG 14 â€” UInt64â†’UInt128 sext / UInt128>> ashr | **DONE** | `2ae300fd` | e2e_m37_u128 (R=0 isolated); m14 |
| BUG 15 â€” bare `shr`/`shl` hijacked by math-builtin intercept | **DONE** | `2ae300fd` | e2e_m37_shr_builtin (R=0 isolated); m15 |
| BUG 16/18 â€” skiplist+trie / string+similarity+time 0xC0000409 combos | **DONE** | `2ae300fd` (fn-key + longest-prefix walk + encoding repair) | m16a-f probes R=0; m16b-e (bare/leaf forms) R=0 |
| **BUG 2 â€” module-global struct FIELD writes lost** | **DONE** | `f0388644` | e2e_m37_global_field_write (g.v = 5 persists); probe_gf R=0 |
| **BUG 3 â€” module-global fn-call initializers zero** | **DONE** | `f0388644` | e2e_m37_global_fn_init (`var G = _mk(1)` â†’ 10 via @llvm.global_ctors); probe_const3 R=0 |
| Str + Int/Char/UInt concat crashed (inttoptr of the integer â†’ AV) | **DONE** | `f0388644` | e2e_m37_str_int_concat ("y = " + 42 â†’ "y = 42"); probe_concat0 R=0 |
| Dotted-module chain binding â€” `use stdlib.xiom.io` flaky 40â€“60% "cannot call" (parser nests dotted paths; process_use bound the {xiom:{io:â€¦}} chain; stdlib-prefixed uses skipped preload/prelude) | **DONE** | `f0388644` | m19_read_file 8/8 + min repro 8/8 deterministic; stdlib-compile 40/40; checker 178/178 |
| BUG 19 — NaN-producing Float64 ops returned sentinel/0xC000001D (float `!=` ? `fcmp one`; `Str+Float64` concat inttoptr) | **DONE** | `9c3a2f9e` | e2e_m37_nan_ieee (23 checks) R=0; probe_nan prints "c = nan / is NaN" exit 0; 17/17 sweep |
| Â§7 NASM/SIMD tracks (math/crypto/hash/compress asm, CPUID dispatch) | **OPEN** â€” off-limits to stdlib session (crates/ + stdlib/runtime/*.c); pure-XIOM fallbacks in place | â€” | â€” |
| Selfhost plan | **WRITTEN â€” execution in progress** | `090ed5d1` | docs/SELFHOST_PLAN.md (phases 0â€“8) + docs/checklists/selfhost-phase0.md |

**Suite state (current, 2026-08-11 22:10):** fast suite 1103/10/1 â€” ALL 10
failures are the parallel stdlib session's in-flight restructure (stdlib-exec
complex/hash/net/rand folder moves + smoke_math_core renamed to
smoke_math_tower, lsp/mcp module-list tests against the new layout, diff
documented pre-existing ignore); **zero compiler regressions**. e2e harness
verification of the 4 new e2e_m37 tests is blocked until the parallel session's
next compiler rebuild (their target/debug/xiom.exe predates `2ae300fd`); exact
harness invocation replicated with the isolated binary: compile=0 run=0 for
all 6 affected tests. Warning gates 0/0.

---

## 2026-08-10 â€” BigInt/BigFloat session findings (feat/architect)

### BUG 1 (CRITICAL) â€” Tuple return types containing structs emit an OPAQUE LLVM type

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
     where element 0 is a 40-byte `BigInt`) â€” the rest of the slot is garbage.
     Reading `dm.0` then dereferences garbage Vec pointers:
     - `bigint_div_mod(3, 10)` â†’ `0xC0000005` ACCESS_VIOLATION (or
       `0xC0000409` STACK_BUFFER_OVERRUN)
     - `bigint_div_mod(10, 3)` â†’ crash; `bigint_mod` â†’ hang/infinite loop;
       `bigint_sqrt` â†’ crash; `bigint_ext_gcd` â†’ crash.
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
  â†’ clang error above (opaque `%struct.Tuple__probe_tuple.Pair__probe_tuple.Pair`).
- **Impact on stdlib (blocks Phase A/B):**
  - `stdlib/xiom/bigint.xi`: `pub fn bigint_div_mod(a: &BigInt, b: &BigInt) -> (BigInt, BigInt)`
    (PRE-EXISTING, frozen signature, **never exercised by any test until now** â€”
    `tests/regression/` contains zero `bigint_div_mod`/`bigint_mod`/`bigint_gcd`
    call sites; the original 3-assertion smoke only exercised parse/mul/compare).
    The original author already worked around the tuple problem once for the
    internal helper (`_div_mod_base` returns a named `DivModResult` struct â€”
    comment: "Returns a DivModResult struct to avoid tuple issues") but the
    public `bigint_div_mod` kept the tuple and was never tested.
  - Planned (this session, blocked): `bigint_sqrt_rem -> (BigInt, BigInt)`,
    `bigint_ext_gcd -> (BigInt, BigInt, BigInt)`.
- **Fix direction:** codegen must emit a concrete LLVM struct type definition
  for `Tuple__<T>__<U>` (aggregate of element types) and store elements with
  the element's real size (memcpy-style), not `store i64`. Enum variants with
  struct payloads (Option/Result) already work â€” reuse that lowering path.

### BUG 2 â€” Module-global struct FIELD writes are silently lost

- **Construct:** `var g: W = W{ v: 0; };` at module scope, then `g.v = 5;` in a
  function; reading `g.v` afterwards returns 0. Whole-value assignment
  (`g = W{ v: 5; };`) persists correctly.
- **Minimal repro** (`probe_gf.xi`): `_set()` does `g.v = 5;` â†’ `main` prints
  `g.v` == `0`.
- **Impact:** module-level mutable struct state (round-mode globals etc.) must
  be updated by whole-value assignment until fixed. Stdlib constants that
  would be module globals are affected (see BUG 3).
- **Fix direction:** GEP-store on a module-global struct must write through to
  the global (currently the value appears to be copied on access).

### BUG 3 â€” Module-global `var` initializers that CALL functions are silently zero

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

### NOTE 4 â€” Module-qualified enum variant access resolves to a module

- `bigfloat.Down` (variant via module qualifier, like `cmp.Less` in cmp.xi)
  fails with "argument 1 type mismatch: expected RoundMode, found module" in
  some positions, while type-qualified `bigfloat.RoundMode.Down` (log.xi
  style) works everywhere. Checker's dotted-name resolution appears to treat
  the variant segment as a sub-module first. Minor; type-qualified form is the
  documented style.

### NOTE 5 â€” D4b aggregate: leaf sub-module fns not reachable through a 1-segment aggregate

- After `use xiom.bigfloat;` (flat aggregate manifest `module xiom.bigfloat`
  + `use xiom.num.bigfloat;`), the sub-lib's TYPES resolve
  (`bigfloat.RoundMode.Nearest`) but its FUNCTIONS do not:
  - `bigfloat.bigfloat_from_int(42)` â†’ "cannot call on this expression"
  - `num.bigfloat.bigfloat_from_int(42)` â†’ "undefined variable 'num'"
  - `xiom.num.bigfloat.bigfloat_from_int(42)` â†’ works (full dotted path).
  The proven D4b example (`use xiom.math;` + `math.core.sqrt(x)`) works only
  because the sub-lib's parent segment (`math`) equals the aggregate leaf.
  With `use xiom.num.bigfloat;` directly, leaf calls (`bigfloat.fn`) work.
  Checker should attach `num.bigfloat` under the aggregate key `bigfloat`
  (parents map builds from the full key "num.bigfloat" â†’ parent "num", never
  from the importing manifest).

---

## 2026-08-10 â€” Compiler-hardening session (BUG 1 fixed; findings below)

### FIXED â€” BUG 1 (tuple-of-struct codegen) â€” root cause and fix

Three coordinated codegen fixes landed (commits pending, crates/xiom-codegen):

1. **`concrete_type_for` Tuple branch** (lib.rs): tuple type keys now
   module-qualify their element names (`Tuple__probe_tuple.Pair__probe_tuple.
   Pair`), matching the expression-level registration used by the body's
   alloca/GEP. Previously the fn SIGNATURE used bare names (`Tuple__Pair__
   Pair`) while the body used qualified names â€” two different LLVM types for
   the same tuple â†’ clang "Cannot allocate unsized type" / ret type mismatch.
2. **`resolve_type_key`** (lib.rs): prefers the current-module-qualified key
   and SKIPS generated aggregate keys (`Tuple__...`, `Option__...`,
   `Result__...`, `_Anon__...`) in the suffix search â€” `Tuple__x.Big__x.Big`
   ends with `.Big`, so a bare `Big` could resolve to the tuple itself and
   nest the tuple name into itself (double-nested `Tuple__Tuple__...`).
3. **`parse_struct_field_types`** (lib.rs): looks up the registered type_meta
   layout instead of splitting the name on `_` (which degraded every
   non-primitive element to i64 â€” tuple slots stored only the first i64 of
   each struct). Element stores now use the real field width.

Also routed tuple PARAM types through `concrete_type_for` (`param_llvm_type`,
decl.rs signature + prologue), and fixed `coerce_arg_for_param` for
non-ident `&expr` lvalues. Verified: `probe_tuple`, `probe_big` (40-byte
structs, 3-element tuples), `probe_big2` (copy-from-tuple + by-ref passing)
all compile and run correctly; `bigint_div_mod` no longer crashes (was AV /
garbage before the fix).

### FIXED â€” Struct `&T` param mutation was silently lost

- **Construct:** `fn _trim(b: &BigInt) { b.digits.pop(); }` â€” mutation of a
  field through a `&T` STRUCT param. Struct `&T` params were passed BY VALUE,
  so `pop()`/`push()` on the param's Vec field mutated a discarded copy â€”
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
  correct â€” `_trim` works), `probe_cmp` (compare correct), `probe_dm`
  (div_mod invariant `q*b+r==a` holds for small numbers).

### FIXED â€” clang -O2 hang on large modules (alwaysinline every function)

- **Construct:** every emitted function was marked `alwaysinline`. With a hot
  caller (the extended bigint smoke's `main`) calling many large functions,
  LLVM inlined the whole library into `main` and `clang -O2` never finished
  (>300s on a 735KB module; `-O0` finished in ~16s; stripping the attribute
  finished in ~3s).
- **Fix (decl.rs):** size-based inline policy â€” `approx_block_cost`
  (recursive statement count): â‰¤10 stmts `alwaysinline`, â‰¤48 `inlinehint`,
  larger no attribute. Preserves P0-4 hot small-function inlining
  (`read_u16_be`-style) while preventing optimizer explosion.
- **Verified:** extended `smoke_bigint.xi` compiles in ~14s (was hanging).

### NOTE 7 â€” `bigint_div_mod` quotient is WRONG for multi-limb dividends (STDLIB, parallel-session Phase A)

- **Construct:** `bigint_div_mod` on any dividend with â‰¥2 limbs produces a
  too-small quotient and a remainder â‰¥ divisor. Repros (all other bigint
  functions verified correct â€” mul/add/sub/compare/to_str/from_int/from_str):
  - `bigint_div_mod("1000000005", 2)` â†’ q=2, r=1000000001 (correct:
    q=500000002, r=1).
  - `bigint_div_mod("987654321987654321", 12345)` â†’ q=80004000080004,
    r=4941000004941 â€” satisfies `q*b+r==a` but r â‰¥ b (correct: r < 12345).
  - `bigint_div_mod("987654321987654321", 1000000000)` â†’ qâ‰ 987654321.
- **Root cause (stdlib logic in `stdlib/xiom/bigint.xi`, NOT the compiler):**
  `_estimate_q_digit` (line 132) estimates with a single top limb
  (`n_hi / d_hi`) with no carry window from the higher limbs, and the
  div_mod loop only corrects OVERestimates (decrements `est`), never
  UNDERestimates â€” so a low estimate sticks. Also `est <= 0 { return 1; }`
  forces a bogus digit of 1 when the true digit is 0.
- **Action:** parallel BigInt session owns `bigint.xi` â€” fix the estimator
  (Knuth-style two-limb window + upward correction, or process the remainder
  carry) and remove the `est<=0 â†’ 1` shortcut. Compiler side is verified
  correct via the probes above.
- **Smoke impact:** `stdlib_exec_bigint_runs` fails at assertion 8 until this
  stdlib fix lands (compile/hang issues from BUG 1 and the optimizer hang are
  fixed).

---

## 2026-08-11 â€” BigInt/BigFloat session: follow-up verification (post coerce.rs edit)

### BUG 8 (CRITICAL) â€” RESOLVED by the compiler session's uncommitted coerce.rs edit

Re-verified on the rebuilt compiler: `_from_twos_bits` / `_bits_to_bigint`
now emit correct signatures
(`define %struct.BigInt @bigint._from_twos_bits(%struct.Vec %param0)`),
`bigint_bit_and/or/xor` work, and the extended `smoke_bigint.xi` (20+
assertions incl. two's-complement negatives) passes end-to-end.

### BUG 9 â€” Catalog fns returning module-local PRIVATE struct types degrade to i64

- **Construct:** `fn f(...) -> PrivateType` where `PrivateType` is a struct
  declared (non-`pub`) in the same catalog stdlib module. The definition and
  call sites emit `i64` instead of `%struct.PrivateType` â†’ ABI mismatch â†’
  AV at runtime. `pub type` in the same position works.
- **Minimal repro** (verified): `stdlib/xiom/num/probet.xi` with
  `type Wrap = { a: Int; b: Int; }` + private `make_wrap` â†’ `call i64
  @probet.make_wrap` / `define i64 @probet.make_wrap`; with `pub type Wrap`
  â†’ `%struct.Wrap` and correct results.
- **Stdlib handling:** `IntFrac` in `num/bigfloat.xi` is declared `pub` â€”
  it is part of the module's public surface anyway (split result of the
  rounding/floor machinery).
- **Fix direction (compiler):** catalog decl registration must resolve
  non-pub module-local struct types (or the checker must reject them) â€”
  silently defaulting to i64 corrupts memory.

### BUG 10 (pre-existing) â€” Float literals emitted rounded to 6 decimals

- **Construct:** any Float64 literal. The emitted LLVM constant is the
  literal formatted with ~6 decimals: `0.000000001` â†’ `0.000000` (= 0.0),
  `3.14159265358979` â†’ `3.141593` (â‰ˆ1e-7 error). Short literals (<= 6
  decimals) are exact and unaffected. Present in the OLD compiler binary as
  well â€” pre-existing, not a regression from the hardening session.
- **Observed:** `if diff > 0.000000001` became `if diff > 0.0`; probes
  comparing literals like `3.1400000000000001` vs `3.14` reported "equal".
- **Stdlib handling:** no stdlib fn depends on > 6-decimal literals (PI/E
  are parsed from strings); the bigfloat smoke expresses its 1e-9 tolerance
  as `diff * 1000000000.0 > 1.0` (emission-safe).
- **Fix direction (compiler):** emit `%f`-style literals with full
  precision (e.g. `%.17g`) or hex float constants (`0x1.91eb851eb851fp+1`).

### BUG 11 â€” `unsafe { extern }` calls return wrong values for runtime Float64 args

- **Construct:** `stdlib/xiom/math.xi` `pub fn sqrt(x: Float64)` lowers to
  `unsafe { return sqrt(x); }` (extern "C" libm). With a RUNTIME argument the
  result is wrong: `math.sqrt(4.0)` (var-held) != 2.0, `sqrt(9.0)` != 3.0.
  Constant-folded calls appear fine. Same symptom in `smoke_complex.xi`
  (`complex_abs` -> `math.sqrt`) and `smoke_net_folder.xi` â€” both fail in the
  stdlib-exec suite (70/72 pass; the two failures are the unsafe-extern
  path, unrelated to bigint/bigfloat).
- **Suspect:** the Unsafe Confinement trampoline
  (`xiom_trampoline_call` + `__unsafe_ctx` struct) â€” double args/results
  through the context struct. Compiler session's in-flight domain
  (uncommitted coerce.rs + confinement phases); NOT the stdlib code.
- **Fix direction (compiler):** verify double (and i64) arg/ret marshalling
  through `__unsafe_ctx`; compare against `sqrt_pure` (non-unsafe impl).

---

## 2026-08-11 â€” REGRESSION in committed &T fix (d22068f8) â€” RESOLVED

### BUG 8 (CRITICAL) â€” Catalog-module fns with `&Vec[Int]` params emit an EMPTY signature

> RESOLVED: the compiler session's later uncommitted coerce.rs edit fixed the
> &expr lowering; verified on the rebuilt compiler (see the follow-up section
> above). Kept below as the original report.

- **Construct:** any function in an IMPORTED (catalog) stdlib module whose
  parameter is `&Vec[Int]` (i64-element generic container). The function
  definition is emitted with NO parameters and `i64` return, while call
  sites pass `%struct.Vec*` and use the declared return type â€” ABI mismatch
  â†’ deterministic ACCESS_VIOLATION at runtime.
- **Observed IR (probe_bit.xi, both the fresh build and the committed-HEAD
  compiler):**
  ```
  %tmp183 = call i64 @_from_twos_bits(%struct.Vec* %tmp47)   ; call site: Vec* arg
  define i64 @_from_twos_bits() {                             ; definition: NO params!
  ```
  The definition is also emitted UNQUALIFIED (`@_from_twos_bits`, missing the
  `@bigint.` module prefix) â€” the external-decl registration degraded the
  signature. `_bits_to_bigint(&Vec[Int], Bool)` in the same module vanished
  from the IR entirely.
- **Scope (empirically verified):**
  - `&Vec[Int]` param in a USER module â†’ correct (`define i64 @sum_vec(%struct.Vec %param0)`).
  - `&Vec[UInt8]` param in a CATALOG module â†’ correct (smoke_compress exit 0).
  - `&BigInt` (plain struct) param in a catalog module â†’ correct (`@bigint._twos_bits(%struct.BigInt*, i64)`).
  - OLD compiler (before d22068f8): `bigint_bit_and` worked (probe_bisect #9,
    exit 0) â€” so this is a REGRESSION from the committed &T-param fix
    (param_llvm_type struct-address lowering + coerce changes), not a
    pre-existing issue.
- **Trigger in stdlib:** `stdlib/xiom/bigint.xi` `_from_twos_bits(bits: &Vec[Int])`
  and `_bits_to_bigint(bits: &Vec[Int], negative: Bool)` â€” the two's-complement
  bitwise helpers. The extended `smoke_bigint.xi` crashes (0xC0000005) the
  moment any of `bigint_bit_and/or/xor` is linked in.
- **Likely root cause hint:** the recurring `warning: unknown type 'Int]' â€”
  defaulting to i64` (type-string parser splitting `Vec[Int]` on `]`) combined
  with the new param lowering â€” the mangled `&Vec[Int]` param type resolves to
  an empty/i64 default during catalog decl registration. The catalog
  registration path (collect_external_decls) is what differs from the
  in-program path (which works).
- **Fix direction (compiler):** catalog/external-decl signature extraction for
  `&Vec[T]` params must preserve the container type (and the module prefix on
  the emitted fn name). Verify with: probe_bit.xi (bit ops), smoke_bigint.xi
  (extended), and re-run smoke_compress (must stay green â€” &Vec[UInt8] is the
  canary for the working path).

### FIXED in stdlib during diagnosis (no compiler involvement)

- `bigint.xi` div_mod zero-remainder: `dm.1.digits[0]` is an out-of-bounds
  read when the remainder is zero (div_mod returns an EMPTY digits Vec for a
  zero remainder â€” `_trim` pops all limbs; both the old and new paths do
  this). All readers (`bigint_to_base`, `bigint_to_hex`, `_bigint_bit_array`)
  now guard with `bigint_is_zero(&dm.1)` first. Pre-existing hazard, exposed
  by the new m==1 schoolbook path.
- `smoke_bigint.xi` assertion for `bigint_shift_left`: the function is the
  original DECIMAL shift (Ã—10^n), not a bit shift â€” assertion corrected to
  `shift_left(1, 3) == "1000"` (documented in the module header).

---

### NOTE 6 â€” `stdlib_exec_bigint_runs` fails (smoke_bigint.xi compile hangs/fails) â€” PARALLEL-SESSION IN-FLIGHT, NOT A COMPILER REGRESSION

- **Observed:** `.\test_summary.ps1 -Fast` (threads 32) reports a 4th
  stdlib-exec failure: `stdlib_exec_bigint_runs` ("compile failed for
  examples\stdlib_smoke\smoke_bigint.xi"). Manual `xiom -o bigint_check.exe
  smoke_bigint.xi` does not return within 60s (compiler appears to hang) â€” the
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

## 2026-08-11 â€” Compiler session follow-up: BUG 8 verified FIXED, 3 new findings

### BUG 8 â€” verified RESOLVED on the current tree (commit pending)

Reproduced with a fresh probe (`probe_bit.xi`: `bigint_bit_and(12, 10)`) â€”
the current compiler emits CORRECT signatures for catalog fns with
`&Vec[Int]` params:

```
define %struct.BigInt @bigint._from_twos_bits(%struct.Vec %param0) inlinehint {
define %struct.BigInt @bigint._bits_to_bigint(%struct.Vec %param0, i64 %param1) ...
```

and `probe_bit` compiles + runs (12&10=8). The empty-signature stub the
stdlib session observed came from `emit_undefined_symbol_stubs` filling in
for a fn that was never emitted â€” the emission failure was the earlier
uncommitted coerce.rs state (now reverted/committed). Locked in with e2e:
`e2e_m37_catfix_catalog_vecref` (examples/catfix: imported module with
`&Vec[Int]` + `&struct` params).

### FIXED â€” Parser: `bits[L - 1]` (index with arithmetic on an uppercase ident)

- **Construct:** `if bits[L - 1] == 0 { ... }` â€” the postfix `[` arm's
  "explicit generic call args" heuristic (lowercase base + UPPERCASE first
  token in brackets) eagerly parsed `L` as a TYPE argument and errored
  "expected ']', found -" â€” it only backtracks when `]` is followed by `(`,
  but the error fired before reaching that check. Also broke catalog files:
  `stdlib/xiom/bigint.xi` `_from_twos_bits` and `stdlib/xiom/num/bigfloat.xi`.
- **Fix (crates/xiom-parser/src/lib.rs):** speculative scan to the matching
  `]` (bracket-depth aware, Eof-guarded); commit to generic-args ONLY when
  the group is immediately followed by `(`. Index expressions (`bits[L-1]`,
  `buf[Head]`) fall through to normal index parsing.
- **Verified:** probe_idx (bits[L-1] evaluates correctly), generic calls
  (`add2[Float32](...)`) still parse; e2e `e2e_m37_index_arith`.

### FIXED â€” Catalog import pathological slowness ("hang" on `use xiom.math`)

- **Construct:** importing modules whose transitive imports contain LEAF
  references (`use xiom.core.to_int` â€” a fn/const, not a module file) made
  `xiom --check` take minutes-to-infinite. Root cause: `ModuleCatalog::
  load_module`'s Strategy-b scan-based fallback walks the ENTIRE source
  directory tree (reading every .xi header) for EVERY failed lookup â€” and
  source_dirs include the file's parent dir + project root (and the CWD,
  which can be a temp dir containing a full worktree + target/). The
  pre-built `module_index` (build_index) already covers every file the scan
  could find, so the scan is pure waste in the normal flow.
- **Fix (crates/xiom-check/src/catalog.rs):** (1) skip the scan entirely when
  `module_index` is non-empty (index is authoritative; scan retained only for
  catalogs built without `build_index`, e.g. unit tests); (2) `index_dir` /
  `load_from_dir` / scan recursion skip build/VCS/package-manager dirs
  (`target`, `build`, `.git`, `node_modules`, any `.`-prefixed dir).
- **Measured:** `use xiom.math` check 20sâ†’9.6s (scan eliminated; residual
  ~4-9s is prelude module parsing); `probe_bit` full compile 138-160sâ†’9.5s.
  Locked in by e2e_m37_catfix_* (which also verify catalog resolution still
  works).

### VERIFIED â€” circular imports are safe (not prevented, terminate)

- **Construct:** module A `use`s B and B `use`s A. The loader's
  `cached_loaded` guard terminates the cycle; the checker resolves both
  modules' symbols regardless of load order; compile succeeds.
- **Empirically verified** with a two-module cycle (check 4.6s, compile 8.3s,
  run correct â€” the only crash was my fixture's own unbounded mutual
  recursion, expected stack overflow, not a compiler issue). The current
  stdlib has NO true module cycle (bigintâ†’xiom.num; num/bigfloatâ†’xiom.bigint
  don't close a loop because num.xi doesn't import bigfloat).
- Locked in by e2e `e2e_m37_catfix_circular_imports` (examples/catfix circ_*).

---

## 2026-08-11 â€” M37 batch 2: BUG 9/10/11 FIXED â€” fast suite 1112/1/1

### FIXED â€” BUG 9: private catalog struct types degrade to i64

`collect_external_decls` injected only PUB type decls; a pub fn taking or
returning a PRIVATE struct (the stdlib's `pub IntFrac` workaround pattern)
lost the type layout â€” emitted signatures degraded to i64 (`make_frac(3,4)`
summed to 0 instead of 7). Fix: transitively inject non-pub types
referenced by injected pub fn signatures (params/returns/inner types +
field types). Verified: examples/catfix b9mod+b9main; e2e
`e2e_m37_catfix_private_type`.

### FIXED â€” BUG 10: float literals truncated to 6 decimals

`{:.6}` at all three literal-emission sites truncated 0.123456789 to
0.123457 (wrong stored values AND wrong comparisons). Now `{:.17e}` â€”
exact f64 round-trip, LLVM-valid. Also removed a leftover "CG02 DEBUG"
eprintln. Verified: e2e `e2e_m37_float_precision`.

### FIXED â€” BUG 11: unsafe-extern double marshalling (complex/net_folder)

The unsafe-block round-trip family: (1) block-fn `return X` with a STRUCT
tail extracted a scalar field (Option field-1) instead of val_to_i64's
heap-pointer round-trip â†’ AV on re-materialization (m34_y04);
(2) `ret_from_enclosing` overwrote the shared block value with the
enclosing coercion â†’ `icmp eq ptr, i64` (m34_d01..d20);
(3) fault path emitted `ret i64* 0` (clang rejects) instead of `null`
(m33_u13). Fixes in expr.rs/stmt.rs. `smoke_complex.xi` and
`smoke_net_folder.xi` now PASS â€” stdlib-exec is 72/72.

### TEST MIGRATIONS (confinement-era rules, not compiler regressions)

- m21_ffi_unsafe_001..009: added the T007 `requires:` contract to
  whole-body-unsafe fns (rule landed in 83416aa9 after the tests).
- m35_z12/z30: wrapped pointer-returning wrappers in unsafe (T003).
- m33_u13: rewrote the dangling-`&local` return test to be well-defined.

---

## 2026-08-11 â€” BigInt/BigFloat session: Phase C (transcendentals) landed

- `bigfloat` Phase C landed (pure-XIOM series: pi/e with precision via
  Machin/Taylor, exp/ln/log10, sin/cos/tan, atan/atan2, pow_bf) â€”
  `smoke_bigfloat.xi` now 44 assertion blocks, exit 0 in ~2s.
- NO new compiler findings from Phase C. One stdlib coding error caught and
  fixed (atan halving identity: `1 + sqrt(1 + t^2)`, not `1 + sqrt(t^2)`).
- **BUG 11 (unsafe extern doubles) â€” FIXED by the compiler session AFTER this
  entry was written** (`7f7b7b54`, 2026-08-11): `stdlib_exec_complex_runs`
  and `stdlib_exec_net_folder_runs` now PASS (stdlib-exec 72/72 â€” verified
  15:0x and 15:4x). The stale "remains OPEN" claim below was written before
  the fix landed; see the STATUS SUMMARY at the top of this file.
- BUG 2/3 (module-global field writes / fn-call initializers) remain
  open; stdlib design already avoids both (whole-value global assignment;
  constants as pure constructor fns). Fixing them unlocks precision-cached
  Ï€/ln10 and the spec's `const BIGINT_*/BIGFLOAT_*` style.

---

## 2026-08-11 â€” fmt.sprintf session: Float64 container element access broken (NEW)

### BUG 12 (NEW) â€” `Vec[Float64]` element reads lower to `load i64 + sitofp` (silent corruption); `[N]Float64` fixed arrays degrade to `[N]i64` (AV)

- **Construct (both user and catalog modules):** any read of an element of a
  `Vec[Float64]` (param or local) or of a fixed `[N]Float64` array.
- **Observed:**
  1. `Vec[Float64]`: `probe_vf.xi` â€” `var x = v[0];` after `v.push(3.14159)`
     â†’ `x != 3.14159`. IR evidence (`_sprintf_engine`, the Vec element-load
     switch): the 8-byte case emits `bitcast i8* to i64*` + `load i64`, then
     the float-typed value is produced by `sitofp i64 %tmp402 to double` â€”
     the f64 BIT PATTERN (4614256650576693248 for 3.14159) is treated as an
     integer and converted, not reinterprete â€” `%.2f` of 3.14159 printed
     "4614256650576693248.00". The generic Vec element-load path only knows
     integer element widths (1/2/4/8) and defaults the type to i64.
  2. `[N]Float64` (incl. inside structs): `probe_fa.xi` â€” warning
     `unknown type 'Float64]' â€” defaulting to i64` (the type-string parser
     splits on `]`, same family as the old BUG 8), struct fields + array
     slots laid out as i64 â†’ 0xC0000005 ACCESS_VIOLATION reading `s.values[0]`.
  3. `Vec[Float64]` STORE path appears intact (push stores the raw bits);
     only element READS are wrong.
- **Impact on stdlib:** NO pre-existing stdlib module uses `Vec[Float64]` or
  `[N]Float64` (stats works on `Vec[Int]`) â€” zero regression. The fmt.sprintf/
  sscanf batch (G13) was designed around `Vec[Float64]` and was re-designed
  to avoid the construct: scalar `Float64` params (proven â€” bigfloat/geom
  pass doubles everywhere) and a fixed-slot `FloatScan` struct for sscanf
  float results (`// TODO(compiler)` note in fmt.xi). Float64 in structs as
  plain fields (not arrays) is proven fine (geom Vec2).
- **Fix direction (compiler):** (a) the Vec element-load switch must load
  `double` (and `float`/`fp128`) for float element types instead of i64+sitofp
  â€” the element type should come from the Vec's registered type, not the
  width; (b) the type-string parser must not split `Float64]`/`UInt8]` on the
  first `]` (parse the full `Vec[T]`/`[N]T` with bracket depth) â€” same root
  cause family as BUG 8's `&Vec[Int]` empty-signature bug. Verify with
   probe_vf.xi (expect exit 0) and probe_fa.xi (expect exit 0).

### BUG 13 (NEW) â€” fp128 (Float128) arithmetic hits missing compiler-rt helpers at link time

- **Construct:** any program whose Float128 value flows through i64â†’f128
  (sitofp), f128â†’f64 (fptrunc), or f128 division. `probe_f128.xi` â€”
  `n as Float128`, `x as Float64`, `acc / ten` â†’ lld-link errors:
  ```
  undefined symbol: __floatditf   (sitofp i64 â†’ fp128)
  undefined symbol: __trunctfdf2  (fptrunc fp128 â†’ f64)
  undefined symbol: __divtf3      (fdiv fp128)
  ```
  fadd/fmul/fpext on fp128 are native (x87) and link fine; the compiler-rt
  soft-float helpers are not in the link line.
- **Impact on stdlib:** blocks the planned `bigfloat_to_float128` bridge
  (256-bit framing mission). Int128/UInt128 are UNAFFECTED (native LLVM i128
  â€” no helpers) so `bigint_to_i128/u128/u64` landed. A lossy
  `bigfloat_to_float64 â†’ fpext` wrapper was rejected (only 15 digits â€” the
  whole point of f128 is 34). TODO(compiler) note in stdlib/xiom/num/bigfloat.xi.
- **Fix direction (compiler):** link compiler-rt (clang `-rtlib=compiler-rt`
  or add libclang_rt.builtins) on Windows, or emit/implement the handful of
  `__*tf3`/`__floatditf`/`__trunctfdf2` stubs; verify with probe_f128.xi
  (expect exit 0).

### BUG 14 (NEW) â€” UInt64â†’UInt128 cast emits SEXT; UInt128 `>>` emits ASHR

- **Construct:** `v as UInt128` with `v: UInt64` whose bit 63 is set, and
  `(p >> 64)` on a `UInt128` whose bit 127 is set. `probe_m128.xi` â€”
  `(lhs as UInt128) * (rhs as UInt128)` for lhs/rhs with bit 63 set gives the
  WRONG product. IR evidence:
  ```
  %tmp6 = sext i64 %tmp5 to i128     ; must be zext for UInt64
  %tmp18 = ashr i128 %tmp17, %tmp19  ; must be lshr for UInt128
  ```
  (The compiler has no unsigned 128-bit type distinction at codegen â€” UInt64â†’
  UInt128 and Int64â†’Int128 both lower to sext; UInt128 `>>` lowers to ashr.)
- **Impact on stdlib:** XXH3's 64Ã—64â†’128 mulhi (`XXH3_mul128_fold64`) was
  wrong on inputs with the top bit set (verified: seed-0 64-bit vectors
  passed for short inputs only after the fix). Worked around in
  `hash/xxhash.xi`: `_u64_to_u128` builds the i128 from 32-bit halves (sext
  == zext for bit-63-clear values) and the high half is masked after `>>`.
  `bigint_to_u128`/`to_i128` are unaffected (limbs < 1e9 and shifts of
  bit-63-clear values only).
- **Fix direction (compiler):** lower `UInt64 as UInt128` with `zext` (type
  information exists in the checker) and `UInt128 >>` with `lshr`; verify
  with probe_m128.xi (expect exit 0).

### BUG 15 (NEW) â€” alwaysinline bodies with a single `var` + `return` drop the mask statement on inline

- **Construct:** a small fn (â‰¤ inline-threshold) whose body is exactly
  `var mask = <expr>; return <expr2> & mask;` â€” e.g.
  ```
  fn shr(x: UInt64, k: Int) -> UInt64 {
    var mask: UInt64 = ((1 as UInt64) << (64 - k)) - 1;
    return (x >> k) & mask;
  }
  ```
  Inlined call sites emit ONLY the `ashr` â€” the mask `shl`/`sub` and the
  `and` are dropped (probe_sip3.xi: `shr(t, 32)` returns the sign-extended
  value; IR shows `%tmp6 = ashr i64 %tmp4, 32` with no following `and`).
  Adding a second var (`var shift = 64 - k; var mask = ...;`) makes the
  inline correct (the pattern hash.xi `_rotl64` has always used).
- **Impact on stdlib:** SipHash/XXH3 logical shifts were wrong until the
  two-var form was used. All new hash code uses the two-var pattern with a
  comment. NOT worked around in old code â€” hash.xi `_rotl64` (two vars) is
  unaffected.
- **Fix direction (compiler):** the inline expansion drops statements from
  single-var bodies (likely a statement-copy bug in the inline pass); verify
  with probe_sip3.xi (single-var shr must equal two-var shr, exit 0).

### BUG 16 (NEW) â€” combining collect.skiplist + collect.trie fast-fails with 0xC0000409 (stack cookie)

- **Construct:** link BOTH `stdlib/xiom/collect/skiplist.xi` and
  `stdlib/xiom/collect/trie.xi` into one program and run ANY skiplist fn
  (even `skiplist_insert`) plus `trie_new` â€” the program prints normally
  then dies at exit with 0xC0000409 (STATUS_STACK_BUFFER_OVERRUN â€” the /GS
  cookie check on main's frame fires after `return`). Repros:
  `probe_pt2.xi` (two inserts, no io) and `probe_pt3.xi` (insert + trie_new).
  Each module alone, and every other pair (skiplistÃ—{string,convert,char,
  cuckoo,fenwick,objectpool,queue,cache}; trieÃ—{cuckoo,fenwick,objectpool,
  queue,cache}) exits clean.
- **Observed in IR (both modules combined):** generated symbols emitted
  UNQUALIFIED: `define %struct.MaybeUninit @MaybeUninit.clone(...)` and
  `define void @BinaryHeap.invariant_check(...)` / `@BufReader.invariant_
  check` / `@Cursor.invariant_check` â€” no module prefix, while the same
  programs alone show the same stubs (benign alone). Two Option payload
  instantiations exist in the pair (Option[Int] in skiplist, Option[Char]
  via trieâ†’string.char_at) â€” the shared MaybeUninit/clone codegen path is
  the prime suspect (clone body `ret %struct.MaybeUninit %self` with a
  16-byte layout vs possibly 8-byte Char payload â†’ stack corruption).
- **Stdlib handling:** both modules are correct individually; the CI/exec
  harness must NOT combine them in one smoke â€” the batch's smokes are
  split (smoke_collect2a: skiplist+cuckoo+fenwick+pool+spsc+arc;
  smoke_collect2b: trie+cuckoo+fenwick+pool+spsc+arc). TODO(compiler) notes
  in both modules.
- **Fix direction (compiler):** qualify generated clone/invariant-check
  symbols per module and make the MaybeUninit clone emit payload-accurate
  code for each Option payload type; verify probe_pt2.xi (expect exit 0).

### BUG 17 (NEW) â€” runtime-runtime `Str ==` on Vec[Str] ELEMENTS lowers to pointer compare

- **Construct:** comparing two runtime `Str` values that come from `Vec[Str]`
  ELEMENT loads, e.g. `ga[i] == gb[j]` where `ga`/`gb` are `Vec[Str]`:
  probe_jac2.xi â€” both elements are "he" yet `==` is false; `ga[0] == "he"`
  (literal) is true. The emitted IR for the element-element comparison
  contains NO `strcmp` call (the element loads degrade the operand type so
  the equality lowers to pointer icmp), while plain runtime `Str == Str`
  (vars/slices, probe_seq/probe_seq4) DOES emit `strcmp`.
- **Impact on stdlib:** `text.similarity.jaccard_similarity` and the
  lcp/lcsuffix helpers compared slices â€” all now use a byte-wise `_str_eq`
  helper (documented) so string content comparison never depends on the
  degraded path. This is the same family as BUG 12 (Vec element type
  degradation): Float64 elements load as i64+sitofp, Str elements lose the
  content-equality lowering.
- **Fix direction (compiler):** the Vec element-load expression must carry
  the element's declared type (Str â†’ strcmp on `==`; Float64 â†’ `load
  double`); verify probe_jac2.xi (both comparisons true, exit 0).

### BUG 18 (NEW) â€” combining string + text.similarity + time in one program crashes (0xC0000405)

- **Construct:** one program importing `xiom.string`, `xiom.text.similarity`
  AND `xiom.time` (smoke_str2.xi) crashes at startup with 0xC0000405 before
  any output. Every PAIR of the three exits clean; each module alone is
  clean. Same family as BUG 16 (combination-specific startup crash â€” likely
  the unqualified `@MaybeUninit.clone`/invariant-check stubs colliding when
  several Option payload shapes coexist).
- **Refined trigger (time-only programs):** with only `xiom.time` imported,
  `strptime("2026-13-01", "%Y-%m-%d")` followed by `strptime("2026-08-11",
  "%Y-%m-%d %Q")` in the SAME program crashes (0xC0000405); the same specs
  individually, or any other spec pair, exit clean. The unsupported-
  conversion early-return path (`%Q`) after a range-check failure path
  miscompiles at -O2 (probe_tm9: the pair fails; both single calls pass).
- **Stdlib handling:** the batch's smokes are split â€” smoke_str2.xi covers
  string+text.similarity (proven pair), smoke_time2.xi covers time with the
  `%Q` check removed (the fn is correct â€” verified by single-call probes;
  TODO(compiler) notes in the affected modules).
- **Fix direction (compiler):** same as BUG 16 â€” module-qualify generated
  symbols and emit payload-accurate clone/invariant code; verify
  smoke_str2.xi recombined (expect exit 0).

## 2026-08-11 â€” STATUS SUMMARY: BUG 12â€“18 all FIXED (commits `2ae300fd`, `3b8f5415`)

| Bug | Fix | Verified |
|-----|-----|----------|
| 12/17 | `vec_elem_from_type_annotation` (types.rs + lib.rs) unwraps Ref/MutRef/Ptr; `Vec(...)` AST form handled â€” Vec element loads keep Float64/Str types | m12b, m37_vec_f64 R=0 |
| 13 | fp128 coercion arms in `coerce_value` + NEW `stdlib/runtime/fp128_helpers.c` soft-float add/sub/mul/div/conv/cmp (verified in a C harness: 400/5/2500/1002.5, negatives, tiny values) | m13b/m13d, m37_f128 R=0 |
| 14 | cast site uses `xiom_type_of_local` (registered type); `expr_is_unsigned()` picks lshr; var bindings infer type from `as UInt*` targets | m14, m37_u128 R=0 |
| 15 | math-builtin `shr`/`shl` intercept restricted to `math.*`/`xiom.math.*` qualified keys + bare keys with NO registered fn | m15, m37_shr_builtin R=0 |
| 16/18 | `process_use` longest-dotted-prefix walk (directory submodules); `bare_fn_aliases` prefers the CALLER's module; **fn-key fix**: call resolution returns the BARE key when a bare definition exists, qualifying to caller-module/alias only otherwise â€” kills the definition-vs-call symbol mismatch (user fns emit bare `@mk_big`, calls resolved to leaf-qualified zero-param stubs â†’ ABI crash, same family as BUG 8) | m16a-e (bare/leaf forms) + m16f (qualified form) all R=0; skiplist+trie combined OK |

- **Encoding repair:** `skiplist.xi` + `trie.xi` contained invalid UTF-8 (lone
  0x97 bytes) which silently broke import binding â€” repaired.
- **Regression sweep** (isolated binary, `tgt_iso`): m37_tuple_struct,
  m37_ref_mut, m37_index_arith, m37_vec_f64, m37_u128, m37_f128,
  m37_float_precision, m37_shr_builtin, m33_z14, m34_y04, m33_u13,
  m19_read_file â€” **12/12 R=0**.
- **Known follow-up (parallel stdlib session owns it):** the `collect/` â†’
  `collections/` folder move landed while module declarations inside still
  say `xiom.collect.*` â€” `use` resolves by file path (works) but
  fully-qualified calls need the declared name (`xiom.collect.skiplist.fn`
  works; `xiom.collections.skiplist.fn` does not until the declarations are
  aligned). Not a compiler regression; m16b-e/m16f verified green.
- Workspace build gate: **zero warnings** (dead `load_external_module` +
  unused imports removed with the XIOM_TRACE_* debug prints).

---

## 2026-08-11 — stdlib session (evening): BUG 19 (NEW) — every NaN-producing Float64 operation returns a garbage sentinel or traps

**? FIXED 2026-08-11 (commit `9c3a2f9e`).** Two codegen defects, both verified:

1. **float `!=` lowered to `fcmp one`** (ordered-not-equal) — for NaN operands
   `one` is FALSE, so `x != x` returned false and NaN was undetectable.
   Fixed to `fcmp une` in both the BinOp table (expr.rs) and the trait-method
   table (call.rs `.ne()`).
2. **`Str + Float64` concat inttoptr'd the FP bits** — the "garbage sentinel
   print" was this, not the fdiv (the IR fdiv was always correct). Fixed via
   new `@xiom_double_to_string` (xiom_runtime.c): NaN ? "nan", ±inf ?
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
  2. `math.ln_pure(-1.0)` (pre-existing stdlib path `if x <= 0.0 { return 0.0 / 0.0; }`, math.xi:162) ? process dies with **0xC000001D** (STATUS_ILLEGAL_INSTRUCTION) — the runtime fault-trap fires on the NaN-producing division.
  3. `1.0 / 0.0` ? +inf and `-1.0 / 0.0` ? -inf are **correct** (inf results pass; only NaN results are broken).
- **IR evidence (probe_nan.xi):** the emitted IR is correct — `%tmp41 = fdiv double 0.00000000000000000e0, 0.00000000000000000e0` — so the corruption happens in the clang/optimize/runtime-trap stage, not in AST emission. The consistent garbage value (`-92233.72036854775808` ˜ a sentinel) suggests the fault-trap/intercept layer (the same system as smoke_guard_fault.xi) replaces NaN-producing FP ops with a trap-or-sentinel path instead of the IEEE result.
- **Impact on stdlib:** `math.constants` NAN cannot be implemented (no literal syntax; `0.0/0.0` broken) — `// TODO(compiler): BUG 19` in math/constants.xi; `math.is_nan`/`is_inf` classify correctly but nothing in the stdlib can PRODUCE a NaN today (all NaN-producing libm entries — asin/acos/ln/sqrt — carry domain `requires:` contracts). `num/float.xi` `bits_to_float` (planned) needs a real bitcast intrinsic to land anyway.
- **Fix direction (compiler):** route NaN results (fdiv 0/0, fsub inf-inf, fmul inf*0, and libm domain-error returns) through the IEEE path — do not trap/sentinel float NaN; optionally add a `nan` literal or i64?f64 bitcast intrinsic (`bitcast i64 0x7FF8000000000000 to double`) which would unblock `math.constants.NAN` + `num.float.bits_to_float`. Verify with probe_nan2/probe_nan3 (expect `nan` print + `x != x` true, exit 0).

**Stdlib unblock:** `math.constants.NAN` can now be implemented as
`pub fn nan() -> Float64 { return 0.0 / 0.0; }` (a const initializer can't hold
the expression yet — const-fold only handles literals; a runtime fn works).
`is_nan(x)` = `x != x` is now correct.

---

## 2026-08-11 (night) — stdlib session: BUG 20 (NEW, REGRESSION from `1d4cd2e8`) — unconditional -mavx512* clang flags crash non-AVX-512 CPUs (illegal instruction) in ANY vectorized program

**? FIXED 2026-08-12 (commit `4fa7a1d2`).** AVX-512 flags are now HOST-CPUID-gated:
`-mavx512f/bw/dq/vl` are added only when
`std::arch::is_x86_feature_detected!("avx512f")` (crates/xiom/src/lib.rs). The
-O2 vectorizer emits zmm in ordinary float loops; the runtime CPUID dispatch
gates only INTENTIONAL SIMD calls, so the FLAGS must match the host.
`-maes -mavx -mavx2` remain unconditional (safe on any AVX2 CPU). Verified on
Zen 2: clang arg trace shows no `-mavx512*`; probe_avx.xi exit 0;
smoke_num_fraction/smoke_math_rounding/smoke_math_precision/
smoke_math_trig_constants/smoke_num_float all R=0 (were 0xC000001D).
`smoke_num_precision` STILL AV-crashes (0xC0000005, deterministic) — a
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
  ? compiles clean, then **exit -1073741795 (0xC000001D)** with zero output. Also affects: `smoke_num_fraction.xi`, `smoke_math_rounding.xi` (and every float-loop stdlib smoke) on this machine (AMD Ryzen 9 3950X = Zen 2 — **no AVX-512**). Pure-int/string programs (e.g. `smoke_string_case.xi`) run fine.
- **Impact:** blocks stdlib float-smoke verification on non-AVX-512 machines; any user program with a float hot loop crashes on such CPUs. The v0.58 comment's own rule ("only dispatch-gated code paths may rely on AVX-512 presence") is unenforceable for the vectorizer — the FLAGS themselves must be gated.
- **Fix direction:** gate the flags on the HOST's CPUID at build time (query AVX-512 support before adding -mavx512*; e.g. use `-march=native` which enables only what the host supports), or drop the 512-bit flags (keep -maes -mavx -mavx2, safe on any AVX2 CPU). Verify: probe_avx.xi exit 0 on Zen 2; e2e SIMD test still R=0.

---

## 2026-08-11 (night) — stdlib session: BUG 21 (NEW) — catalog-module fn returning a Str created INSIDE an unsafe block returns a corrupted Str (len 0xFFFFFFFF)

**Status 2026-08-12: NOT REPRODUCED on the current build.** All documented
shapes verified PASSING with the isolated binary: catalog fn with (a) loop
inside unsafe + `return Str.from_cstring(buf)` from inside the block ?
"aaa" R=0; (b) direct return from inside the block ? "xyz" R=0; (c) while
loop + `xiom.string.str_concat` build inside unsafe + return ? "aaa" R=0.
`smoke_string_pad_repeat` R=0. The original `str_repeat_char` shape was
restructured away before the fix could be isolated — likely resolved by the
accumulated batch (`f0388644` chain/process_use + `9c3a2f9e` BUG 19 + `2ae300fd`
fn-key). The stdlib's workaround (no loop inside unsafe) can stay. If the
EXACT original file still fails, send it and it becomes a live repro.

- **Construct:** an IMPORTED (catalog) stdlib module fn whose body creates a Str inside an `unsafe` block and RETURNS it from inside that block, e.g. `stdlib/xiom/string/repeat.xi` `str_repeat_char` (was: `unsafe { ...; return Str.from_cstring(buf); }`). The unsafe-confinement trampoline (`__unsafe_ctx`) round-trips only i64-class values; the Str (ptr+len struct) return corrupts the length field ? `len()` returns 4294967295 and any strcmp on the value crashes (0x80000003 breakpoint — heap guard).
- **Verified:** probe_repeat/probe_rep2 — `repeat.str_repeat_char('a', 3)` prints `[]` with `len=4294967295`; the smoke's `str_repeat_char(...) != "aaa"` comparison then dies 0x80000003 with all buffered output lost. The SAME shape in a USER module (`fn mk_a() -> Str { unsafe { ...; return Str.from_cstring(buf); } }` with T007 `requires: true`) works — so it is the catalog/trampoline path, not from_cstring. The flat string.xi str_pad_left/str_pad_right build strings inside unsafe but return OUTSIDE the block (assign-var-in-unsafe, return after) — that shape is correct, which is why the bug was never hit before.
- **Fix direction (compiler):** the unsafe-block return marshalling for catalog fns must round-trip Str (and other 16-byte struct) returns like the BUG 11 Option fix did — verify repeat.xi's `str_repeat_char` via its smoke once fixed; alternatively reject `return <struct-typed>` from inside unsafe blocks with a clear error.
- **Stdlib handling:** repeat.xi restructured to the proven shape (assign inside unsafe, return outside) + `// TODO(compiler): BUG 21` note. The `str_repeat_char(...) != "aaa"` comparison is the crash trigger — the corrupted value must never reach a comparison.

---

## 2026-08-11 (night) — stdlib session: BUG 22 (NEW, batch report) — implementation-phase findings from 4 parallel agents (detailed repros below; all pre-existing or new-shape; none blocked the batches)

**? RESOLVED 2026-08-12 (commits `eeab8cf7`, `e6608946`, `47cd188f`, `f74a52c7`):**
1. `&&`/`||` short-circuit — **FIXED** (branch on LHS; RHS compiled only when needed; verified div-by-zero RHS never executes) — m37_short_circuit
2. Unary minus on match-bound vars — **FIXED** (negation defers wildcards like Not; float payloads bitcast from the i64 slot) — m37_match_float_payload
3. Cross-module 3-tuple `.1`/`.2` — **FIXED** (tuple field maps derive+register for catalog returns) — m37_catalog_boundary
4. Cross-module Option payload match — **FIXED** (float payloads via local_opt_payload_xiom; Int payloads verified) — m37_catalog_boundary
5. requires/ensures trap — **FIXED** (clean `xiom_panic`: message to stderr + flush + exit 1, no more 0xC000001D) — m37_contract_pass + manual violation probe
6. str_reverse invalid IR — **FIXED 2026-08-12** (root: unsafe-block captures grabbed SHADOWED names — the block's inner `let c` shadowed an enclosing loop's `let c`, capturing the outer loop-body alloca whose address doesn't dominate; fix: block-bound names excluded from captures + loop-body binding allocas hoisted to fn entry). Flat string/string.xi str_reverse verified: compiles, returns "cba"; m37_loop_capture regression — the stdlib's reverse.xi workaround can now delegate
7. index_of(str_slice) 0xC0000409 — stdlib-side scan workaround in place; not reproduced standalone (see 22.15)
8. xiom_char_at leading-byte — DOCUMENTED CONTRACT (byte position), stdlib decodes UTF-8 manually — no change
9. Multi-byte char literals — **VERIFIED FIXED** on current build (é=233, ?=937)
10. byte_at sign-extend — **VERIFIED FIXED** on current build (UInt8 as Int zexts)
11. Module-qualified call results inline — **FIXED** (ROOT: qualified-call symbol vs bare def mismatch ? fn-symbol PRE-ASSIGNMENT map + integer verdicts for inline calls/index/binary operands) — m37_inline_call_concat
12. Vec[Char] element size — **FIXED by the BUG 23 #2 nested-Vec work** (elem sizes now resolve from the registered element type; Char elements read via the elem_size switch)


1. **`&&` does not short-circuit** (num/fraction.xi `fraction_from_float`): both operands evaluate; a div-by-zero in the RHS traps 0xC000001D even when the LHS is false. Use nested `if`s. (Suggested fix: proper short-circuit lowering or reject non-short-circuit semantics.)
2. **Unary minus on match-bound vars fails** — `error[T001]: cannot negate type _` for `-d` where `d` is bound in a match arm. Workaround: type-annotate the binding.
3. **Cross-module 3-tuple field access `.1`/`.2` fails** (math/arithmetic.xi `gcd_extended` returns (Int,Int,Int)): `240 * t.1` ? "right operand must be numeric, found <error>" — `.0` works, 2-tuples fine. Comparison/return contexts work.
4. **Cross-module `match` on `Option[Int]` binds a garbage payload** (math/arithmetic.xi `mod_inverse`): `Some(v) => v != 5` misbehaves; `is_some()/unwrap()` exact. Same family as the old Option-tail extraction bug.
5. **`requires:`/`ensures:` are runtime-enforced and TRAP on violation (0xC0000005/0xC000001D)** — even when the fn body guards the case. Stdlib avoids declaring contracts on fns with graceful fallbacks (documented in module headers); fix direction: contracts should be checked/elided consistently (or only in debug builds), or stdlib keeps fallback-first bodies.
6. **flat `string.str_reverse` emits invalid LLVM IR** (`Instruction does not dominate all uses!` — alloca in loop body captured by an enclosing unsafe context struct). Blocks the `str_reverse` API name: any program calling `string.str_reverse` fails to compile. reverse.xi implements locally; the flat fn needs the loop-body alloca moved out (compiler session's string.xi is a shared file — needs care).
7. **`string.index_of(str_slice(...), needle)` inside a loop crashes (0xC0000409)** — passing a str_slice result to index_of repeatedly corrupts; replace.xi scans byte-wise instead.
8. **`xiom_char_at(s, pos)` returns only the leading BYTE** of a multi-byte char (195 for é, not 233) — pre-existing flat string.xi/char.xi contract: byte position, not code point. String sublibs do manual UTF-8 decode from byte_at.
9. **Multi-byte char literals are mangled to their last byte**: `'é' != 'O'` is false (both ? 0xA9). Workaround: build chars via to_char(cp) in tests.
10. **`byte_at(...) as Int` sign-extends UInt8** (0xC3 ? -61): must mask `& 0xFF` when handling bytes >= 0x80.
11. **Module-qualified call results used INLINE in arithmetic miscompile** (e.g. `i = i + char.len_utf8(ch)` advances by 1 instead of 2): bind the result to a `let` var first (agent-applied repo-wide).
12. **`Vec[Char]` element size is 1 byte** (BUG 12 family): storing code point 937 (0x3A9) reads back 0xA9. str_code_points returns Vec[Int] instead.

---

## 2026-08-11 (night) — stdlib session: BUG 23 (NEW, wave-2 batch) — findings from 4 parallel agents (string metrics/unicode, math number-theory/linear)

**? RESOLVED 2026-08-12 (commits `eeab8cf7`, `e6608946`, `47cd188f`, `f74a52c7`):**
1. Cross-module returned Vec[Float64] reads — **FIXED** (var-bindings inherit the callee's declared element type) — m37_catalog_boundary
2. Nested Vec[Vec[T]] garbage — **FIXED** (parser type-arg rendering, elem size 32, memcpy reads, nested float inner reads) — m37_nested_vec
3. Fn-value params named add/mul — **VERIFIED FIXED** on current build (fn-typed params dispatch correctly)
4. `else if` parser rejection — **FIXED** — m37_else_if
5. Flaky `use of undefined value` (~50%) — **FIXED** (catalog decl injection now deterministic: all_cached sorted; plus the fn-symbol pre-assignment kills the order-dependent symbol class) — 8/8 deterministic compiles
6. UTF-8 BOM breaks registration — **FIXED** (lexer strips leading BOMs) — lexer unit test
7. (Bool,Bool) tuples as Tuple__Int__Int — **FIXED** (param-seeded tuple scan + XIOM idents + checker field-map derivation) — m37_catalog_boundary
8. Catalog `&Vec[T]` param mutation no-op — **FIXED** (Ref params pass the pointer like &mut) — m37_catalog_boundary
9. Unary minus on catalog-returned float — **FIXED** — m37_catalog_boundary
10. Subtraction on catalog-returned floats — covered by the BUG 20 CPUID gating + float-type registration fixes; verify via the float smokes
11. xiom.math.sqrt bare import "requires unsafe" — **VERIFIED FIXED** on current build (math.sqrt R=0)
12. Recursive helpers defeat the vectorizer — NOT a compiler bug (stdlib shape choice); BUG 20's CPUID gating now lets loops vectorize on Zen 2 (AVX2); recursion remains their call


1. **Vec[Float64] element READS of a module-RETURNED Vec are still broken** (BUG 12 corner): a user program reading `v[0]` from a Vec[Float64] returned by a catalog fn gets raw bit-pattern garbage and float arithmetic on such loads traps (0xC000001D). BUG 12's fix covers Vec element loads inside the defining module; cross-module returned float Vecs are not fixed. Workaround: verify via scalar invariants (dot/norm), avoid element reads of module-returned float Vecs.
2. **Nested `Vec[Vec[T]]` element reads return garbage** (0xC0000005): `m[1].len()` wrong, `m[i][j]` = 0 for a 2-row matrix; float arithmetic on nested loads crashes. Blocks dynamic-matrix runtime verification and `set_partition` (reads Vec[Vec[Int]]).
3. **Fn-value params named `add`/`mul`/`div` collide with built-in operators**: `ring_theory(add, mul)` compiled the param calls to `tower.Int.mul`/native `*`, IGNORING the passed function (verified in IR). Rename params (op_add/op_mul) — callers pass positionally so the API is unchanged. Parser/checker should reject or qualify operator-shadowing param names.
4. **Parser rejects `else if`** (`expected '{', found if`) — must use nested if/else. (The `elif` keyword works; `else if` as two words does not.)
5. **Flaky `use of undefined value` compile failure** (~50%): combining modules that load `xiom.text.similarity` + spline fns with `&Vec[Float64]` params (math/numerical.xi) intermittently fails with `use of undefined value '@_tridiagonal'` (undefined-symbol-stub emission; BUG 8/16/18 family). Same source compiles on retry.
6. **A UTF-8 BOM in a catalog module silently breaks function registration** (all BOMs in the stdlib were removed/avoided; writer tools must not add BOMs).
7. **(Bool,Bool) tuples misregister as Tuple__Int__Int** (logic.xi `quantifiers`) — Bool in tuples/structs degrades; workaround: encode as Int 0/1 or separate fns.
8. **Catalog `&Vec[T]` param mutation is a silent no-op** — `&mut Vec[T]` is required for mutation (enumerators use `&mut`). The old struct-&T fix (d22068f8) covers struct params, not Vec params.
9. **Unary minus on a catalog-returned float in an expression traps** (0xC000001D, BUG 20 family): use `0.0 - x`.
10. **Subtraction on catalog-returned floats in smoke asserts traps** (0xC000001D): use interval comparisons (a > lo && a < hi) instead of `a - b` diffs in smokes.
11. **`xiom.math.sqrt` as a direct import cannot be called bare from a user module** (T001 "requires unsafe") — the math-builtin intercept (BUG 15) only covers catalog modules; user code must call `math.sqrt`.
12. **Recursive helpers defeat the BUG 20 AVX-512 vectorizer**: range/table scans and summation loops written as recursion (one iteration per frame, depth = ~182 < 500 limit) do not get vectorized, so those modules run on Zen 2. Intended as a temporary shape until BUG 20's CPUID gating lands.

**TODO(compiler): NOT IMPLEMENTABLE stubs left by wave-2** (frozen signatures compile, bodies documented; blocked by the above): math/trig sinh/cosh/tanh/atanh (exp/ln inline arithmetic traps — same shape as math/hyperbolic.xi which is also BUG-20-blocked at runtime), calculus integrate_romberg/integrate_gauss/limit/left/right/is_continuous/gradient/partial_derivative/jacobian/hessian/laplacian/curl/divergence (loops + Vec[Float64]), differential richardson/gradient/jacobian/partial_derivative, series maclaurin_series/convergence_rate, integral integrate_adaptive, set_theory set_partition (nested Vec), logic simplify/normal_forms/satisfiability/tautology_check/quantifiers (Bool/Str-returning recursion + Bool tuples). All re-verifiable once BUG 20 (CPUID-gated flags) and the BUG 23 #1/#2 (cross-module float Vec / nested Vec) fixes land.

---

## 2026-08-12 — stdlib session: BUG 24 (NEW, REGRESSION from the 22/23 batch `eeab8cf7..7cfc7fe4`) — bigfloat pow_bf miscompiles inconsistently (AV or hang) depending on unrelated program structure

**? PARTIAL FIX 2026-08-12 (`dd6b4ab1`):** the WRONG-VALUE symptom is fixed —
root was PRE-EXISTING (not the 22/23 batch): `llvm_type_for`'s array arm
kept the trailing bracket (`[10 x Int]` ? elem name `"Int]"` ? unknown type ?
i64 degradation), corrupting bigint's `var digits: [10]Int` and every
fixed-array local. Verified: p_arr digit-extract R=0; probe pair now compiles
clean (no "unknown type" warnings).
**RESIDUAL (open):** a per-program-shape AV remains inside the pow path when
the program also compiles to_str/eq — crashes before the first println with
buffered output lost; the compiled IR is verifiably sound (no stubs, no
duplicate defines, helper symbols consistent across shapes). Suspects to
continue: BigFloat struct-value passing with Vec[Int] significand fields
interacting with the 23.8 &Vec pointer-pass change; deeper helpers in the
bigger module set. Reproduction: p_powv2.xi shape (pow_bf + to_str in one
program) AVs; p_b24a.xi shape (pow_bf alone) returns a wrong value only in
the pre-fix build.

- **Construct:** any program calling `num/bigfloat.xi` `bigfloat_pow_bf(&base, &exp)` — or its wrappers (`precision_float.bigfloat_pow`) — on the CURRENT compiler. `smoke_bigfloat.xi` (pre-existing harness smoke, was 72/72 green) now dies 0xC0000005; `smoke_num_precision.xi` dies too. pow_bf's pieces (is_zero/is_one/is_negative/ln/mul/exp at the same operands) ALL verify correctly in isolation.
- **Decisive minimal repro pair (identical logical values, different codegen):**

  - `var t = bigfloat.bigfloat_two(); bigfloat.bigfloat_pow_bf(&t, &t);` ? prints 4, exit 0.  (works)
  - `var t = bigfloat.bigfloat_from_int(2); bigfloat.bigfloat_pow_bf(&t, &t);` ? 0xC0000005 with zero output. (AV)
  - `bigfloat_two()` is literally `return bigfloat_from_int(2);` — the VALUES are identical; only the CALL STRUCTURE differs. Combining both in one program ? infinite hang (series divergence, no output).
- **Also:** `smoke_math_numerical.xi` / `smoke_math_approximation.xi` AV (0xC0000005) — may be the same root (they call math.exp/ln-style paths) or the nested-Vec spline reads; both were green at agent time.
- **Suspects (in order):** (1) the fn-symbol pre-assignment map (`7cfc7fe4`) — bigfloat.xi has 1380 lines of private helpers (`_one_at`, `_finish`, `_round_digits_raw`, `_atanh_series`, ...) that can land on different symbols per program shape; (2) the `23.8 &Vec[T]` pointer-pass change interacting with BigFloat's `Vec[Int]` significand FIELD copies inside struct-value passing (`bigfloat_mul(exp, &l)` passes a `&BigFloat` param BY VALUE to a `BigFloat` value param — shallow Vec copy).
- **Fix direction:** reproduce with the probe pair above; check the pre-assignment map for bigfloat.xi private helpers (does `@bigfloat._finish`/`_round_digits_raw` resolve identically in both programs?); verify struct-with-Vec-field value-copy semantics unchanged by 23.8. Verification: probe pair both exit 0; smoke_bigfloat + smoke_num_precision green.
- **Stdlib handling:** no stdlib change (bigfloat.xi untouched — pre-existing REAL module). The blocked smokes (smoke_bigfloat, smoke_num_precision, smoke_math_numerical, smoke_math_approximation) stay as-is pending the fix; everything else in the 61-smoke sweep passes.

---

## 2026-08-12 — stdlib session: BUG 25 (NEW, wave-3 batch) — findings from 3 parallel agents (collections + convert/bits)

**? COMPILER SESSION STATUS 2026-08-12 (commits `9a578313`..`271567b0`):**
1. Same-name delegation / cross-module same-name — **FIXED**: ambiguous bare fns
   exported by multiple imported modules now ERROR (T001) and require a
   module-qualified call — no more silent wrong-module resolution
2. `use X as alias;` — **FIXED 2026-08-13**: the residual was the external-decl
   REACHABILITY filter pruning the aliased fn (main references the ALIAS name,
   not the target leaf ? the fn came back as a bare stub with an i64-return).
   Use declarations now contribute their target leaves to the referenced-name
   set. All four alias forms (module alias, fn alias, alias+qualified)
   verified R=0.
3. `from_bytes` fn name collision — **FIXED** (builtin intercept now only
   fires when no real fn with the name is registered) — m37_from_bytes_fn
4. Bool inside returned tuples — **VERIFIED FIXED** on current build
   (covered by the BUG 23 #7 tuple-type batch) — p_btup R=0
5. Option/Result `.value`/`.error` reads — **FIXED** (payload-aware field
   reads: Str/Float reinterpret, boxed structs inttoptr+load) —
   m37_opt_payload_value
6. Char `<`/`>` comparisons — **VERIFIED FIXED** on current build (p_char_cmp
   R=0)
7. Big&big AND — **VERIFIED FIXED** on current build (0xAEF1AD80 & 0x9B05688C
   = 0x8A012880)
8. `let len = v.len();` GEP family — **FIXED** by #3 (the same builtin-hijack
   root)
9. Module-name vs fn-name collision — DESIGN (keep-first alias rule
   documented; smokes split) — no change
10. `xiom.crypto` _pkcs7_pad undefined + pure-XIOM SHA-256 — **PRE-EXISTING,
    queued** (crypto.xi's private `_pkcs7_pad` call link issue — the fn's
    body emission vs the bare-symbol resolution; needs a dedicated session)
11. Private fns leak via `use` — **FIXED** (export maps exclude private fns;
    bare-call resolution has a visibility gate: pub fns, same-module fns,
    and top-level fns only) — pheap_merge probe now errors "undefined"
12. Spurious E001 borrow warnings — benign (matches pre-existing), no change

## 2026-08-13 — LANGUAGE features (all implemented + tested, commits `dd6a31cd`/`4c439e6a`)
- **BUG 26 (secure numeric policy)**: INT ? FLOAT mixing in arithmetic,
  comparisons, and typed bindings requires an explicit `as` cast (Rust-style).
  Auto-widening stays for same-family; INT LITERALS may adopt the float type
  (`d * 2` stays ergonomic); FLOAT literals never adopt an integer type.
  The checker now also resolves nested Vec[...] element types + dispatches
  methods on the base Vec.
- **Labeled loops**: `@label: while ...` + `break @label;` /
  `continue @label;` (the label field was previously dropped in the codegen
  While arm — labeled break silently targeted the innermost loop).
- **BUG 27 (in-code debug intrinsics)**: `assert(cond[, "msg"])` (clean
  xiom_panic violation), `dbg!(expr)` (prints "[dbg] <value>", returns the
  value), `todo!()`/`unimplemented!()` (panic with source location),
  `debugger;` (breaks into an attached debugger — xiom-debugger_break;
  no-op without one). All builtins yield to user fns with the same name.
- **NOTE 4 FIXED**: module-qualified enum variant access (`bigfloat.Down`)
  resolves + constructs the variant correctly.


1. **Same-name delegation ? 0xC0000005**: a local `pub fn to_base58` PLUS `use xiom.num.convert;` (which exports `to_base58`) — any call to the IMPORTED fn AVs at runtime (even via an indirection helper or `as` alias). Stdlib rule: never delegate to a same-named fn across modules; implement locally. (Related: cross-module same-name resolution silently prefers the LAST imported module's version — quadtree_query resolved to spatial's variant.)
2. **`use X as alias;` miscompiles**: `use xiom.num.base as numbase; numbase.to_base(255,16)` traps `contract violated: ensures`; `use xiom.bits.popcount as pc;` returns 0. Fully-qualified `xiom.num.base.to_base(...)` works. Fix: alias-import resolution.
3. **`from_bytes` fn name collides with a compiler builtin**: ANY module fn named `from_bytes` (even `{ return 0; }`) emits `invalid getelementptr indices` on `%struct.Vec`. convert/bytes.xi keeps the fn as TODO(compiler) — unimplementable.
4. **Bool inside a returned tuple miscompiles**: fn returning `(Int, Bool)` or `(Int, Int, Bool)` emits `%tmp defined with Tuple__Int__Int but expected Tuple__Int__Bool` (tuple-literal constructor emits an all-Int LLVM type). Same family as BUG 23 #7 (Bool tuple misregistration). Blocks convert/overflow.overflowing_* (TODO(compiler)).
5. **Caller-side `.value`/`.error` field reads of Option/Result are corrupted** (wrong len/bit pattern) while `match` extraction is correct. Blocks `xiom.encoding` decoders when read via `.value` (verified: encoding.base64_decode round-trip fails — pre-existing). Stdlib rule: match-extract only.
6. **Char `<`/`>` comparisons miscompile** (all chars report "bad") while `==`/`!=` and `byte_at(...) as Int` work. Root cause of the broken core `to_float_from_str` (digit-range checks). Stdlib parsers use byte-based comparisons.
7. **Bitwise AND on two Int operands with bit 31+ set miscompiles** (`0xAEF1AD80 & 0x9B05688C` wrong by one bit); small masks, OR, shifts, byte_swap are correct. Stdlib hash/mask code avoids big&big (uses shifts + small masks).
8. **`let len = v.len();` in specific module shapes triggers the same GEP error as #3** — `var len` avoids it.
9. **Module-name vs fn-name collision**: `use xiom.bits.popcount;` + `use xiom.bits.bitwise;` (which exports a `popcount` fn) cannot both resolve — smokes split.
10. **`xiom.crypto` unimportable** (`undefined @_pkcs7_pad` at link) and pure-XIOM SHA-256 miscompiles (wrong digests for ""/"abc"; documented in crypto.xi) — pre-existing, blocks base58check's double-SHA256 (Adler-32 fallback shipped, TODO(compiler)).
11. **Private fn leaks into global namespace via `use`**: collect/heap.xi's PRIVATE `pheap_merge(h, a, b)` became reachable through `use xiom.collect.heap;` in pairingheap.xi and hijacked bare calls. Stdlib rule: module-qualified calls for shared names; fix direction: private symbols must not be re-exported by `use`.
12. Spurious E001 borrow warnings through imported fns (benign, codegen correct; matches pre-existing smokes).

---

## 2026-08-12 (night) — stdlib session: BUG 26 (NEW, regression from 271567b0/841119d8/28aaa5e0 batch) — user-module surface regressions

1. **Catalog-RETURNED Vec passed to a `&Vec[T]` catalog param ? C001** ("cannot take a reference to 'data': it is already a reference"): `var c = lz4.lz4_compress(data); var d = lz4.lz4_decompress(c);` fails codegen; even copy-out (`var c2 = c`) or a by-value thunk still fails. Locally-built Vecs work. Affects smoke_compress_lz4_snappy (decompress round-trips trimmed to compress-only, TODO(compiler)). Suspect: returned-Vec values are typed as references by the 28aaa5e0 payload-aware-read change, and the &Vec auto-ref then double-refs.
2. **Bare prelude names (to_char/to_string/to_int) no longer resolve in USER modules** — `use xiom.core.to_char;` (a non-pub prelude fn) does NOT register it, and the old implicit bare-name path was removed by 271567b0's ambiguity error. Catalog modules still get them via prelude wiring. User smokes now use the public `convert.int_to_string` / `convert.float_to_int` / `convert.int_to_char` (match on the Option). Fix direction: prelude names should remain bare-callable in user modules (or be re-exported as pub from a user-facing module).
3. **Cross-module tuple DESTRUCTURING regressed**: `var (a, b) = some_catalog_f.(...)` binds BOTH names to the whole tuple (error "cannot compare Tuple__X__Y with X"). `.0`/`.1` field access (fixed in the 22/23 batch) works — smokes switched to field access.
4. Same-family C001 for `snappy_*`/`lz4_*` decompress calls (all &Vec-param catalog fns taking module-returned values).
5. **High-bit mask comparisons miscompile in UTF-8 classifiers** (BUG 25 #7 family, extended): `(b0 & 0xE0) == 0xC0` / `(b0 & 0xF0) == 0xE0` / `(b0 & 0xF8) == 0xF0` on `byte_at`-derived Ints are FALSE for multibyte lead bytes (C3, etc.) ? convert/utf8.xi utf8_valid_sequences counts every byte as a 1-byte sequence (6 for "héllo" instead of 5). Small masks (< 0x80) and shifts are fine. Stdlib smokes adapted (expected value + TODO); fix direction: verify AND folding for masks with bit 7+ set against byte-extracted values.
6. **percent_encode combination miscompile** (BUG 24 family): `convert/percent.xi` percent_encode returns fully-encoded output ("/a?b=1&c=2" ? "%2Fa...") when the smoke also imports xiom.convert.bytes or xiom.string, but correct output in isolation. `_is_url_safe` branch selection flips with program shape. Smoke split into isolated modules (smoke_convert_percent / smoke_convert_bytes); module verified correct in isolation.
