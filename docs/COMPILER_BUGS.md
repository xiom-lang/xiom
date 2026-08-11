# XIOM Compiler Bugs — Log

Dated sections. Each entry: file(s) affected, construct, error observed, and
(recommended) fix direction for the compiler team. Stdlib workarounds are
deliberately NOT applied where the stdlib mandate says "production grade, no
workarounds" — the compiler must be fixed, then the stdlib lands.

---

## STATUS SUMMARY — authoritative (2026-08-11 16:3x, compiler session)

> Read this first. Older dated sections below may contradict it (e.g. the
> BigInt/BigFloat Phase C entry claimed BUG 11 open — that predates the fix).

| Item | Status | Fix commit | Verified by |
|------|--------|-----------|-------------|
| BUG 1 — tuple-of-struct opaque LLVM type / truncated slots | **DONE** | `d22068f8` | e2e_m37_tuple_struct; probe_big/probe_big2; bigint_div_mod no longer crashes |
| BUG 8 — catalog fns with `&Vec[Int]`/`&Vec[UInt8]` params | **DONE** (was the uncommitted coerce.rs state) | `384d5666` (revert) + verified | e2e_m37_catfix_catalog_vecref; probe_bit (12&10=8) |
| BUG 9 — private catalog struct types degrade to i64 | **DONE** | `3ccd004c` | e2e_m37_catfix_private_type (b9mod+b9main, sum=7) |
| BUG 10 — float literals truncated to 6 decimals | **DONE** | `a2aafa26` | e2e_m37_float_precision (0.123456789 round-trips) |
| BUG 11 — unsafe-extern double marshalling (math.sqrt via trampoline) | **DONE** | `7f7b7b54` | stdlib-exec **72/72** (complex + net_folder pass); e2e_m37_unsafe_option_return |
| Parser — `bits[L - 1]` (uppercase-ident index with arithmetic) | **DONE** | `40441ca7` | e2e_m37_index_arith; bigint.xi/bigfloat.xi parse |
| Catalog — import lookup walked the whole tree per miss (minutes/hang) | **DONE** | `1e982ebf` | stdlib-compile 108.7s → 23.7s; probe_bit compile 138–160s → 9.5s |
| Struct `&T` param mutation silently lost | **DONE** | `d22068f8` | e2e_m37_ref_mut; probe_dig/probe_cmp |
| clang -O2 hang (alwaysinline everything) | **DONE** | `d22068f8` | smoke_bigint.xi compiles ~5–14s (was >300s) |
| Unsafe-block capture collector missing Expr::Struct etc. | **DONE** | `e0f96fef` | e2e_m19_read_file_content |
| Parallel codegen `__unsafe_ctx_0` redefinition | **DONE** | `e0f96fef` | e2e_i2_parallel_codegen (all 5 ecosystem tasks) |
| **BUG 2 — module-global struct FIELD writes lost** | **OPEN** (advisory) | — | stdlib avoids (whole-value assignment); needed for precision-cached π/ln10 and `const BIGINT_*/BIGFLOAT_*` |
| **BUG 3 — module-global fn-call initializers zero** | **OPEN** (advisory) | — | stdlib avoids (constants as pure constructor fns) |
| §7 NASM/SIMD tracks (math/crypto/hash/compress asm, CPUID dispatch) | **OPEN** — off-limits to stdlib session (crates/ + stdlib/runtime/*.c); pure-XIOM fallbacks in place | — | — |
| Selfhost plan | **WRITTEN — execution in progress** | `090ed5d1` | docs/SELFHOST_PLAN.md (phases 0–8) + docs/checklists/selfhost-phase0.md |

**Suite state (current):** fast suite 1112/1/1 (only the documented pre-existing
`test_diff_test_produces_correct_ir`, handoff: IGNORE); e2e 2239/2240 at 15:57
(the single failure was a harness race — hardened in `a092cc35`, expect
2240/2240 on the next full run); stdlib-exec 72/72; warning gates 0/0.

---

## 2026-08-10 — BigInt/BigFloat session findings (feat/architect)

### BUG 1 (CRITICAL) — Tuple return types containing structs emit an OPAQUE LLVM type

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
     where element 0 is a 40-byte `BigInt`) — the rest of the slot is garbage.
     Reading `dm.0` then dereferences garbage Vec pointers:
     - `bigint_div_mod(3, 10)` → `0xC0000005` ACCESS_VIOLATION (or
       `0xC0000409` STACK_BUFFER_OVERRUN)
     - `bigint_div_mod(10, 3)` → crash; `bigint_mod` → hang/infinite loop;
       `bigint_sqrt` → crash; `bigint_ext_gcd` → crash.
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
  → clang error above (opaque `%struct.Tuple__probe_tuple.Pair__probe_tuple.Pair`).
- **Impact on stdlib (blocks Phase A/B):**
  - `stdlib/xiom/bigint.xi`: `pub fn bigint_div_mod(a: &BigInt, b: &BigInt) -> (BigInt, BigInt)`
    (PRE-EXISTING, frozen signature, **never exercised by any test until now** —
    `tests/regression/` contains zero `bigint_div_mod`/`bigint_mod`/`bigint_gcd`
    call sites; the original 3-assertion smoke only exercised parse/mul/compare).
    The original author already worked around the tuple problem once for the
    internal helper (`_div_mod_base` returns a named `DivModResult` struct —
    comment: "Returns a DivModResult struct to avoid tuple issues") but the
    public `bigint_div_mod` kept the tuple and was never tested.
  - Planned (this session, blocked): `bigint_sqrt_rem -> (BigInt, BigInt)`,
    `bigint_ext_gcd -> (BigInt, BigInt, BigInt)`.
- **Fix direction:** codegen must emit a concrete LLVM struct type definition
  for `Tuple__<T>__<U>` (aggregate of element types) and store elements with
  the element's real size (memcpy-style), not `store i64`. Enum variants with
  struct payloads (Option/Result) already work — reuse that lowering path.

### BUG 2 — Module-global struct FIELD writes are silently lost

- **Construct:** `var g: W = W{ v: 0; };` at module scope, then `g.v = 5;` in a
  function; reading `g.v` afterwards returns 0. Whole-value assignment
  (`g = W{ v: 5; };`) persists correctly.
- **Minimal repro** (`probe_gf.xi`): `_set()` does `g.v = 5;` → `main` prints
  `g.v` == `0`.
- **Impact:** module-level mutable struct state (round-mode globals etc.) must
  be updated by whole-value assignment until fixed. Stdlib constants that
  would be module globals are affected (see BUG 3).
- **Fix direction:** GEP-store on a module-global struct must write through to
  the global (currently the value appears to be copied on access).

### BUG 3 — Module-global `var` initializers that CALL functions are silently zero

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

### NOTE 4 — Module-qualified enum variant access resolves to a module

- `bigfloat.Down` (variant via module qualifier, like `cmp.Less` in cmp.xi)
  fails with "argument 1 type mismatch: expected RoundMode, found module" in
  some positions, while type-qualified `bigfloat.RoundMode.Down` (log.xi
  style) works everywhere. Checker's dotted-name resolution appears to treat
  the variant segment as a sub-module first. Minor; type-qualified form is the
  documented style.

### NOTE 5 — D4b aggregate: leaf sub-module fns not reachable through a 1-segment aggregate

- After `use xiom.bigfloat;` (flat aggregate manifest `module xiom.bigfloat`
  + `use xiom.num.bigfloat;`), the sub-lib's TYPES resolve
  (`bigfloat.RoundMode.Nearest`) but its FUNCTIONS do not:
  - `bigfloat.bigfloat_from_int(42)` → "cannot call on this expression"
  - `num.bigfloat.bigfloat_from_int(42)` → "undefined variable 'num'"
  - `xiom.num.bigfloat.bigfloat_from_int(42)` → works (full dotted path).
  The proven D4b example (`use xiom.math;` + `math.core.sqrt(x)`) works only
  because the sub-lib's parent segment (`math`) equals the aggregate leaf.
  With `use xiom.num.bigfloat;` directly, leaf calls (`bigfloat.fn`) work.
  Checker should attach `num.bigfloat` under the aggregate key `bigfloat`
  (parents map builds from the full key "num.bigfloat" → parent "num", never
  from the importing manifest).

---

## 2026-08-10 — Compiler-hardening session (BUG 1 fixed; findings below)

### FIXED — BUG 1 (tuple-of-struct codegen) — root cause and fix

Three coordinated codegen fixes landed (commits pending, crates/xiom-codegen):

1. **`concrete_type_for` Tuple branch** (lib.rs): tuple type keys now
   module-qualify their element names (`Tuple__probe_tuple.Pair__probe_tuple.
   Pair`), matching the expression-level registration used by the body's
   alloca/GEP. Previously the fn SIGNATURE used bare names (`Tuple__Pair__
   Pair`) while the body used qualified names — two different LLVM types for
   the same tuple → clang "Cannot allocate unsized type" / ret type mismatch.
2. **`resolve_type_key`** (lib.rs): prefers the current-module-qualified key
   and SKIPS generated aggregate keys (`Tuple__...`, `Option__...`,
   `Result__...`, `_Anon__...`) in the suffix search — `Tuple__x.Big__x.Big`
   ends with `.Big`, so a bare `Big` could resolve to the tuple itself and
   nest the tuple name into itself (double-nested `Tuple__Tuple__...`).
3. **`parse_struct_field_types`** (lib.rs): looks up the registered type_meta
   layout instead of splitting the name on `_` (which degraded every
   non-primitive element to i64 — tuple slots stored only the first i64 of
   each struct). Element stores now use the real field width.

Also routed tuple PARAM types through `concrete_type_for` (`param_llvm_type`,
decl.rs signature + prologue), and fixed `coerce_arg_for_param` for
non-ident `&expr` lvalues. Verified: `probe_tuple`, `probe_big` (40-byte
structs, 3-element tuples), `probe_big2` (copy-from-tuple + by-ref passing)
all compile and run correctly; `bigint_div_mod` no longer crashes (was AV /
garbage before the fix).

### FIXED — Struct `&T` param mutation was silently lost

- **Construct:** `fn _trim(b: &BigInt) { b.digits.pop(); }` — mutation of a
  field through a `&T` STRUCT param. Struct `&T` params were passed BY VALUE,
  so `pop()`/`push()` on the param's Vec field mutated a discarded copy —
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
  correct — `_trim` works), `probe_cmp` (compare correct), `probe_dm`
  (div_mod invariant `q*b+r==a` holds for small numbers).

### FIXED — clang -O2 hang on large modules (alwaysinline every function)

- **Construct:** every emitted function was marked `alwaysinline`. With a hot
  caller (the extended bigint smoke's `main`) calling many large functions,
  LLVM inlined the whole library into `main` and `clang -O2` never finished
  (>300s on a 735KB module; `-O0` finished in ~16s; stripping the attribute
  finished in ~3s).
- **Fix (decl.rs):** size-based inline policy — `approx_block_cost`
  (recursive statement count): ≤10 stmts `alwaysinline`, ≤48 `inlinehint`,
  larger no attribute. Preserves P0-4 hot small-function inlining
  (`read_u16_be`-style) while preventing optimizer explosion.
- **Verified:** extended `smoke_bigint.xi` compiles in ~14s (was hanging).

### NOTE 7 — `bigint_div_mod` quotient is WRONG for multi-limb dividends (STDLIB, parallel-session Phase A)

- **Construct:** `bigint_div_mod` on any dividend with ≥2 limbs produces a
  too-small quotient and a remainder ≥ divisor. Repros (all other bigint
  functions verified correct — mul/add/sub/compare/to_str/from_int/from_str):
  - `bigint_div_mod("1000000005", 2)` → q=2, r=1000000001 (correct:
    q=500000002, r=1).
  - `bigint_div_mod("987654321987654321", 12345)` → q=80004000080004,
    r=4941000004941 — satisfies `q*b+r==a` but r ≥ b (correct: r < 12345).
  - `bigint_div_mod("987654321987654321", 1000000000)` → q≠987654321.
- **Root cause (stdlib logic in `stdlib/xiom/bigint.xi`, NOT the compiler):**
  `_estimate_q_digit` (line 132) estimates with a single top limb
  (`n_hi / d_hi`) with no carry window from the higher limbs, and the
  div_mod loop only corrects OVERestimates (decrements `est`), never
  UNDERestimates — so a low estimate sticks. Also `est <= 0 { return 1; }`
  forces a bogus digit of 1 when the true digit is 0.
- **Action:** parallel BigInt session owns `bigint.xi` — fix the estimator
  (Knuth-style two-limb window + upward correction, or process the remainder
  carry) and remove the `est<=0 → 1` shortcut. Compiler side is verified
  correct via the probes above.
- **Smoke impact:** `stdlib_exec_bigint_runs` fails at assertion 8 until this
  stdlib fix lands (compile/hang issues from BUG 1 and the optimizer hang are
  fixed).

---

## 2026-08-11 — BigInt/BigFloat session: follow-up verification (post coerce.rs edit)

### BUG 8 (CRITICAL) — RESOLVED by the compiler session's uncommitted coerce.rs edit

Re-verified on the rebuilt compiler: `_from_twos_bits` / `_bits_to_bigint`
now emit correct signatures
(`define %struct.BigInt @bigint._from_twos_bits(%struct.Vec %param0)`),
`bigint_bit_and/or/xor` work, and the extended `smoke_bigint.xi` (20+
assertions incl. two's-complement negatives) passes end-to-end.

### BUG 9 — Catalog fns returning module-local PRIVATE struct types degrade to i64

- **Construct:** `fn f(...) -> PrivateType` where `PrivateType` is a struct
  declared (non-`pub`) in the same catalog stdlib module. The definition and
  call sites emit `i64` instead of `%struct.PrivateType` → ABI mismatch →
  AV at runtime. `pub type` in the same position works.
- **Minimal repro** (verified): `stdlib/xiom/num/probet.xi` with
  `type Wrap = { a: Int; b: Int; }` + private `make_wrap` → `call i64
  @probet.make_wrap` / `define i64 @probet.make_wrap`; with `pub type Wrap`
  → `%struct.Wrap` and correct results.
- **Stdlib handling:** `IntFrac` in `num/bigfloat.xi` is declared `pub` —
  it is part of the module's public surface anyway (split result of the
  rounding/floor machinery).
- **Fix direction (compiler):** catalog decl registration must resolve
  non-pub module-local struct types (or the checker must reject them) —
  silently defaulting to i64 corrupts memory.

### BUG 10 (pre-existing) — Float literals emitted rounded to 6 decimals

- **Construct:** any Float64 literal. The emitted LLVM constant is the
  literal formatted with ~6 decimals: `0.000000001` → `0.000000` (= 0.0),
  `3.14159265358979` → `3.141593` (≈1e-7 error). Short literals (<= 6
  decimals) are exact and unaffected. Present in the OLD compiler binary as
  well — pre-existing, not a regression from the hardening session.
- **Observed:** `if diff > 0.000000001` became `if diff > 0.0`; probes
  comparing literals like `3.1400000000000001` vs `3.14` reported "equal".
- **Stdlib handling:** no stdlib fn depends on > 6-decimal literals (PI/E
  are parsed from strings); the bigfloat smoke expresses its 1e-9 tolerance
  as `diff * 1000000000.0 > 1.0` (emission-safe).
- **Fix direction (compiler):** emit `%f`-style literals with full
  precision (e.g. `%.17g`) or hex float constants (`0x1.91eb851eb851fp+1`).

### BUG 11 — `unsafe { extern }` calls return wrong values for runtime Float64 args

- **Construct:** `stdlib/xiom/math.xi` `pub fn sqrt(x: Float64)` lowers to
  `unsafe { return sqrt(x); }` (extern "C" libm). With a RUNTIME argument the
  result is wrong: `math.sqrt(4.0)` (var-held) != 2.0, `sqrt(9.0)` != 3.0.
  Constant-folded calls appear fine. Same symptom in `smoke_complex.xi`
  (`complex_abs` -> `math.sqrt`) and `smoke_net_folder.xi` — both fail in the
  stdlib-exec suite (70/72 pass; the two failures are the unsafe-extern
  path, unrelated to bigint/bigfloat).
- **Suspect:** the Unsafe Confinement trampoline
  (`xiom_trampoline_call` + `__unsafe_ctx` struct) — double args/results
  through the context struct. Compiler session's in-flight domain
  (uncommitted coerce.rs + confinement phases); NOT the stdlib code.
- **Fix direction (compiler):** verify double (and i64) arg/ret marshalling
  through `__unsafe_ctx`; compare against `sqrt_pure` (non-unsafe impl).

---

## 2026-08-11 — REGRESSION in committed &T fix (d22068f8) — RESOLVED

### BUG 8 (CRITICAL) — Catalog-module fns with `&Vec[Int]` params emit an EMPTY signature

> RESOLVED: the compiler session's later uncommitted coerce.rs edit fixed the
> &expr lowering; verified on the rebuilt compiler (see the follow-up section
> above). Kept below as the original report.

- **Construct:** any function in an IMPORTED (catalog) stdlib module whose
  parameter is `&Vec[Int]` (i64-element generic container). The function
  definition is emitted with NO parameters and `i64` return, while call
  sites pass `%struct.Vec*` and use the declared return type — ABI mismatch
  → deterministic ACCESS_VIOLATION at runtime.
- **Observed IR (probe_bit.xi, both the fresh build and the committed-HEAD
  compiler):**
  ```
  %tmp183 = call i64 @_from_twos_bits(%struct.Vec* %tmp47)   ; call site: Vec* arg
  define i64 @_from_twos_bits() {                             ; definition: NO params!
  ```
  The definition is also emitted UNQUALIFIED (`@_from_twos_bits`, missing the
  `@bigint.` module prefix) — the external-decl registration degraded the
  signature. `_bits_to_bigint(&Vec[Int], Bool)` in the same module vanished
  from the IR entirely.
- **Scope (empirically verified):**
  - `&Vec[Int]` param in a USER module → correct (`define i64 @sum_vec(%struct.Vec %param0)`).
  - `&Vec[UInt8]` param in a CATALOG module → correct (smoke_compress exit 0).
  - `&BigInt` (plain struct) param in a catalog module → correct (`@bigint._twos_bits(%struct.BigInt*, i64)`).
  - OLD compiler (before d22068f8): `bigint_bit_and` worked (probe_bisect #9,
    exit 0) — so this is a REGRESSION from the committed &T-param fix
    (param_llvm_type struct-address lowering + coerce changes), not a
    pre-existing issue.
- **Trigger in stdlib:** `stdlib/xiom/bigint.xi` `_from_twos_bits(bits: &Vec[Int])`
  and `_bits_to_bigint(bits: &Vec[Int], negative: Bool)` — the two's-complement
  bitwise helpers. The extended `smoke_bigint.xi` crashes (0xC0000005) the
  moment any of `bigint_bit_and/or/xor` is linked in.
- **Likely root cause hint:** the recurring `warning: unknown type 'Int]' —
  defaulting to i64` (type-string parser splitting `Vec[Int]` on `]`) combined
  with the new param lowering — the mangled `&Vec[Int]` param type resolves to
  an empty/i64 default during catalog decl registration. The catalog
  registration path (collect_external_decls) is what differs from the
  in-program path (which works).
- **Fix direction (compiler):** catalog/external-decl signature extraction for
  `&Vec[T]` params must preserve the container type (and the module prefix on
  the emitted fn name). Verify with: probe_bit.xi (bit ops), smoke_bigint.xi
  (extended), and re-run smoke_compress (must stay green — &Vec[UInt8] is the
  canary for the working path).

### FIXED in stdlib during diagnosis (no compiler involvement)

- `bigint.xi` div_mod zero-remainder: `dm.1.digits[0]` is an out-of-bounds
  read when the remainder is zero (div_mod returns an EMPTY digits Vec for a
  zero remainder — `_trim` pops all limbs; both the old and new paths do
  this). All readers (`bigint_to_base`, `bigint_to_hex`, `_bigint_bit_array`)
  now guard with `bigint_is_zero(&dm.1)` first. Pre-existing hazard, exposed
  by the new m==1 schoolbook path.
- `smoke_bigint.xi` assertion for `bigint_shift_left`: the function is the
  original DECIMAL shift (×10^n), not a bit shift — assertion corrected to
  `shift_left(1, 3) == "1000"` (documented in the module header).

---

### NOTE 6 — `stdlib_exec_bigint_runs` fails (smoke_bigint.xi compile hangs/fails) — PARALLEL-SESSION IN-FLIGHT, NOT A COMPILER REGRESSION

- **Observed:** `.\test_summary.ps1 -Fast` (threads 32) reports a 4th
  stdlib-exec failure: `stdlib_exec_bigint_runs` ("compile failed for
  examples\stdlib_smoke\smoke_bigint.xi"). Manual `xiom -o bigint_check.exe
  smoke_bigint.xi` does not return within 60s (compiler appears to hang) — the
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

## 2026-08-11 — Compiler session follow-up: BUG 8 verified FIXED, 3 new findings

### BUG 8 — verified RESOLVED on the current tree (commit pending)

Reproduced with a fresh probe (`probe_bit.xi`: `bigint_bit_and(12, 10)`) —
the current compiler emits CORRECT signatures for catalog fns with
`&Vec[Int]` params:

```
define %struct.BigInt @bigint._from_twos_bits(%struct.Vec %param0) inlinehint {
define %struct.BigInt @bigint._bits_to_bigint(%struct.Vec %param0, i64 %param1) ...
```

and `probe_bit` compiles + runs (12&10=8). The empty-signature stub the
stdlib session observed came from `emit_undefined_symbol_stubs` filling in
for a fn that was never emitted — the emission failure was the earlier
uncommitted coerce.rs state (now reverted/committed). Locked in with e2e:
`e2e_m37_catfix_catalog_vecref` (examples/catfix: imported module with
`&Vec[Int]` + `&struct` params).

### FIXED — Parser: `bits[L - 1]` (index with arithmetic on an uppercase ident)

- **Construct:** `if bits[L - 1] == 0 { ... }` — the postfix `[` arm's
  "explicit generic call args" heuristic (lowercase base + UPPERCASE first
  token in brackets) eagerly parsed `L` as a TYPE argument and errored
  "expected ']', found -" — it only backtracks when `]` is followed by `(`,
  but the error fired before reaching that check. Also broke catalog files:
  `stdlib/xiom/bigint.xi` `_from_twos_bits` and `stdlib/xiom/num/bigfloat.xi`.
- **Fix (crates/xiom-parser/src/lib.rs):** speculative scan to the matching
  `]` (bracket-depth aware, Eof-guarded); commit to generic-args ONLY when
  the group is immediately followed by `(`. Index expressions (`bits[L-1]`,
  `buf[Head]`) fall through to normal index parsing.
- **Verified:** probe_idx (bits[L-1] evaluates correctly), generic calls
  (`add2[Float32](...)`) still parse; e2e `e2e_m37_index_arith`.

### FIXED — Catalog import pathological slowness ("hang" on `use xiom.math`)

- **Construct:** importing modules whose transitive imports contain LEAF
  references (`use xiom.core.to_int` — a fn/const, not a module file) made
  `xiom --check` take minutes-to-infinite. Root cause: `ModuleCatalog::
  load_module`'s Strategy-b scan-based fallback walks the ENTIRE source
  directory tree (reading every .xi header) for EVERY failed lookup — and
  source_dirs include the file's parent dir + project root (and the CWD,
  which can be a temp dir containing a full worktree + target/). The
  pre-built `module_index` (build_index) already covers every file the scan
  could find, so the scan is pure waste in the normal flow.
- **Fix (crates/xiom-check/src/catalog.rs):** (1) skip the scan entirely when
  `module_index` is non-empty (index is authoritative; scan retained only for
  catalogs built without `build_index`, e.g. unit tests); (2) `index_dir` /
  `load_from_dir` / scan recursion skip build/VCS/package-manager dirs
  (`target`, `build`, `.git`, `node_modules`, any `.`-prefixed dir).
- **Measured:** `use xiom.math` check 20s→9.6s (scan eliminated; residual
  ~4-9s is prelude module parsing); `probe_bit` full compile 138-160s→9.5s.
  Locked in by e2e_m37_catfix_* (which also verify catalog resolution still
  works).

### VERIFIED — circular imports are safe (not prevented, terminate)

- **Construct:** module A `use`s B and B `use`s A. The loader's
  `cached_loaded` guard terminates the cycle; the checker resolves both
  modules' symbols regardless of load order; compile succeeds.
- **Empirically verified** with a two-module cycle (check 4.6s, compile 8.3s,
  run correct — the only crash was my fixture's own unbounded mutual
  recursion, expected stack overflow, not a compiler issue). The current
  stdlib has NO true module cycle (bigint→xiom.num; num/bigfloat→xiom.bigint
  don't close a loop because num.xi doesn't import bigfloat).
- Locked in by e2e `e2e_m37_catfix_circular_imports` (examples/catfix circ_*).

---

## 2026-08-11 — M37 batch 2: BUG 9/10/11 FIXED — fast suite 1112/1/1

### FIXED — BUG 9: private catalog struct types degrade to i64

`collect_external_decls` injected only PUB type decls; a pub fn taking or
returning a PRIVATE struct (the stdlib's `pub IntFrac` workaround pattern)
lost the type layout — emitted signatures degraded to i64 (`make_frac(3,4)`
summed to 0 instead of 7). Fix: transitively inject non-pub types
referenced by injected pub fn signatures (params/returns/inner types +
field types). Verified: examples/catfix b9mod+b9main; e2e
`e2e_m37_catfix_private_type`.

### FIXED — BUG 10: float literals truncated to 6 decimals

`{:.6}` at all three literal-emission sites truncated 0.123456789 to
0.123457 (wrong stored values AND wrong comparisons). Now `{:.17e}` —
exact f64 round-trip, LLVM-valid. Also removed a leftover "CG02 DEBUG"
eprintln. Verified: e2e `e2e_m37_float_precision`.

### FIXED — BUG 11: unsafe-extern double marshalling (complex/net_folder)

The unsafe-block round-trip family: (1) block-fn `return X` with a STRUCT
tail extracted a scalar field (Option field-1) instead of val_to_i64's
heap-pointer round-trip → AV on re-materialization (m34_y04);
(2) `ret_from_enclosing` overwrote the shared block value with the
enclosing coercion → `icmp eq ptr, i64` (m34_d01..d20);
(3) fault path emitted `ret i64* 0` (clang rejects) instead of `null`
(m33_u13). Fixes in expr.rs/stmt.rs. `smoke_complex.xi` and
`smoke_net_folder.xi` now PASS — stdlib-exec is 72/72.

### TEST MIGRATIONS (confinement-era rules, not compiler regressions)

- m21_ffi_unsafe_001..009: added the T007 `requires:` contract to
  whole-body-unsafe fns (rule landed in 83416aa9 after the tests).
- m35_z12/z30: wrapped pointer-returning wrappers in unsafe (T003).
- m33_u13: rewrote the dangling-`&local` return test to be well-defined.

---

## 2026-08-11 — BigInt/BigFloat session: Phase C (transcendentals) landed

- `bigfloat` Phase C landed (pure-XIOM series: pi/e with precision via
  Machin/Taylor, exp/ln/log10, sin/cos/tan, atan/atan2, pow_bf) —
  `smoke_bigfloat.xi` now 44 assertion blocks, exit 0 in ~2s.
- NO new compiler findings from Phase C. One stdlib coding error caught and
  fixed (atan halving identity: `1 + sqrt(1 + t^2)`, not `1 + sqrt(t^2)`).
- **BUG 11 (unsafe extern doubles) — FIXED by the compiler session AFTER this
  entry was written** (`7f7b7b54`, 2026-08-11): `stdlib_exec_complex_runs`
  and `stdlib_exec_net_folder_runs` now PASS (stdlib-exec 72/72 — verified
  15:0x and 15:4x). The stale "remains OPEN" claim below was written before
  the fix landed; see the STATUS SUMMARY at the top of this file.
- BUG 2/3 (module-global field writes / fn-call initializers) remain
  open; stdlib design already avoids both (whole-value global assignment;
  constants as pure constructor fns). Fixing them unlocks precision-cached
  π/ln10 and the spec's `const BIGINT_*/BIGFLOAT_*` style.

---

## 2026-08-11 — fmt.sprintf session: Float64 container element access broken (NEW)

### BUG 12 (NEW) — `Vec[Float64]` element reads lower to `load i64 + sitofp` (silent corruption); `[N]Float64` fixed arrays degrade to `[N]i64` (AV)

- **Construct (both user and catalog modules):** any read of an element of a
  `Vec[Float64]` (param or local) or of a fixed `[N]Float64` array.
- **Observed:**
  1. `Vec[Float64]`: `probe_vf.xi` — `var x = v[0];` after `v.push(3.14159)`
     → `x != 3.14159`. IR evidence (`_sprintf_engine`, the Vec element-load
     switch): the 8-byte case emits `bitcast i8* to i64*` + `load i64`, then
     the float-typed value is produced by `sitofp i64 %tmp402 to double` —
     the f64 BIT PATTERN (4614256650576693248 for 3.14159) is treated as an
     integer and converted, not reinterprete — `%.2f` of 3.14159 printed
     "4614256650576693248.00". The generic Vec element-load path only knows
     integer element widths (1/2/4/8) and defaults the type to i64.
  2. `[N]Float64` (incl. inside structs): `probe_fa.xi` — warning
     `unknown type 'Float64]' — defaulting to i64` (the type-string parser
     splits on `]`, same family as the old BUG 8), struct fields + array
     slots laid out as i64 → 0xC0000005 ACCESS_VIOLATION reading `s.values[0]`.
  3. `Vec[Float64]` STORE path appears intact (push stores the raw bits);
     only element READS are wrong.
- **Impact on stdlib:** NO pre-existing stdlib module uses `Vec[Float64]` or
  `[N]Float64` (stats works on `Vec[Int]`) — zero regression. The fmt.sprintf/
  sscanf batch (G13) was designed around `Vec[Float64]` and was re-designed
  to avoid the construct: scalar `Float64` params (proven — bigfloat/geom
  pass doubles everywhere) and a fixed-slot `FloatScan` struct for sscanf
  float results (`// TODO(compiler)` note in fmt.xi). Float64 in structs as
  plain fields (not arrays) is proven fine (geom Vec2).
- **Fix direction (compiler):** (a) the Vec element-load switch must load
  `double` (and `float`/`fp128`) for float element types instead of i64+sitofp
  — the element type should come from the Vec's registered type, not the
  width; (b) the type-string parser must not split `Float64]`/`UInt8]` on the
  first `]` (parse the full `Vec[T]`/`[N]T` with bracket depth) — same root
  cause family as BUG 8's `&Vec[Int]` empty-signature bug. Verify with
   probe_vf.xi (expect exit 0) and probe_fa.xi (expect exit 0).

### BUG 13 (NEW) — fp128 (Float128) arithmetic hits missing compiler-rt helpers at link time

- **Construct:** any program whose Float128 value flows through i64→f128
  (sitofp), f128→f64 (fptrunc), or f128 division. `probe_f128.xi` —
  `n as Float128`, `x as Float64`, `acc / ten` → lld-link errors:
  ```
  undefined symbol: __floatditf   (sitofp i64 → fp128)
  undefined symbol: __trunctfdf2  (fptrunc fp128 → f64)
  undefined symbol: __divtf3      (fdiv fp128)
  ```
  fadd/fmul/fpext on fp128 are native (x87) and link fine; the compiler-rt
  soft-float helpers are not in the link line.
- **Impact on stdlib:** blocks the planned `bigfloat_to_float128` bridge
  (256-bit framing mission). Int128/UInt128 are UNAFFECTED (native LLVM i128
  — no helpers) so `bigint_to_i128/u128/u64` landed. A lossy
  `bigfloat_to_float64 → fpext` wrapper was rejected (only 15 digits — the
  whole point of f128 is 34). TODO(compiler) note in stdlib/xiom/num/bigfloat.xi.
- **Fix direction (compiler):** link compiler-rt (clang `-rtlib=compiler-rt`
  or add libclang_rt.builtins) on Windows, or emit/implement the handful of
  `__*tf3`/`__floatditf`/`__trunctfdf2` stubs; verify with probe_f128.xi
  (expect exit 0).

### BUG 14 (NEW) — UInt64→UInt128 cast emits SEXT; UInt128 `>>` emits ASHR

- **Construct:** `v as UInt128` with `v: UInt64` whose bit 63 is set, and
  `(p >> 64)` on a `UInt128` whose bit 127 is set. `probe_m128.xi` —
  `(lhs as UInt128) * (rhs as UInt128)` for lhs/rhs with bit 63 set gives the
  WRONG product. IR evidence:
  ```
  %tmp6 = sext i64 %tmp5 to i128     ; must be zext for UInt64
  %tmp18 = ashr i128 %tmp17, %tmp19  ; must be lshr for UInt128
  ```
  (The compiler has no unsigned 128-bit type distinction at codegen — UInt64→
  UInt128 and Int64→Int128 both lower to sext; UInt128 `>>` lowers to ashr.)
- **Impact on stdlib:** XXH3's 64×64→128 mulhi (`XXH3_mul128_fold64`) was
  wrong on inputs with the top bit set (verified: seed-0 64-bit vectors
  passed for short inputs only after the fix). Worked around in
  `hash/xxhash.xi`: `_u64_to_u128` builds the i128 from 32-bit halves (sext
  == zext for bit-63-clear values) and the high half is masked after `>>`.
  `bigint_to_u128`/`to_i128` are unaffected (limbs < 1e9 and shifts of
  bit-63-clear values only).
- **Fix direction (compiler):** lower `UInt64 as UInt128` with `zext` (type
  information exists in the checker) and `UInt128 >>` with `lshr`; verify
  with probe_m128.xi (expect exit 0).

### BUG 15 (NEW) — alwaysinline bodies with a single `var` + `return` drop the mask statement on inline

- **Construct:** a small fn (≤ inline-threshold) whose body is exactly
  `var mask = <expr>; return <expr2> & mask;` — e.g.
  ```
  fn shr(x: UInt64, k: Int) -> UInt64 {
    var mask: UInt64 = ((1 as UInt64) << (64 - k)) - 1;
    return (x >> k) & mask;
  }
  ```
  Inlined call sites emit ONLY the `ashr` — the mask `shl`/`sub` and the
  `and` are dropped (probe_sip3.xi: `shr(t, 32)` returns the sign-extended
  value; IR shows `%tmp6 = ashr i64 %tmp4, 32` with no following `and`).
  Adding a second var (`var shift = 64 - k; var mask = ...;`) makes the
  inline correct (the pattern hash.xi `_rotl64` has always used).
- **Impact on stdlib:** SipHash/XXH3 logical shifts were wrong until the
  two-var form was used. All new hash code uses the two-var pattern with a
  comment. NOT worked around in old code — hash.xi `_rotl64` (two vars) is
  unaffected.
- **Fix direction (compiler):** the inline expansion drops statements from
  single-var bodies (likely a statement-copy bug in the inline pass); verify
  with probe_sip3.xi (single-var shr must equal two-var shr, exit 0).

### BUG 16 (NEW) — combining collect.skiplist + collect.trie fast-fails with 0xC0000409 (stack cookie)

- **Construct:** link BOTH `stdlib/xiom/collect/skiplist.xi` and
  `stdlib/xiom/collect/trie.xi` into one program and run ANY skiplist fn
  (even `skiplist_insert`) plus `trie_new` — the program prints normally
  then dies at exit with 0xC0000409 (STATUS_STACK_BUFFER_OVERRUN — the /GS
  cookie check on main's frame fires after `return`). Repros:
  `probe_pt2.xi` (two inserts, no io) and `probe_pt3.xi` (insert + trie_new).
  Each module alone, and every other pair (skiplist×{string,convert,char,
  cuckoo,fenwick,objectpool,queue,cache}; trie×{cuckoo,fenwick,objectpool,
  queue,cache}) exits clean.
- **Observed in IR (both modules combined):** generated symbols emitted
  UNQUALIFIED: `define %struct.MaybeUninit @MaybeUninit.clone(...)` and
  `define void @BinaryHeap.invariant_check(...)` / `@BufReader.invariant_
  check` / `@Cursor.invariant_check` — no module prefix, while the same
  programs alone show the same stubs (benign alone). Two Option payload
  instantiations exist in the pair (Option[Int] in skiplist, Option[Char]
  via trie→string.char_at) — the shared MaybeUninit/clone codegen path is
  the prime suspect (clone body `ret %struct.MaybeUninit %self` with a
  16-byte layout vs possibly 8-byte Char payload → stack corruption).
- **Stdlib handling:** both modules are correct individually; the CI/exec
  harness must NOT combine them in one smoke — the batch's smokes are
  split (smoke_collect2a: skiplist+cuckoo+fenwick+pool+spsc+arc;
  smoke_collect2b: trie+cuckoo+fenwick+pool+spsc+arc). TODO(compiler) notes
  in both modules.
- **Fix direction (compiler):** qualify generated clone/invariant-check
  symbols per module and make the MaybeUninit clone emit payload-accurate
  code for each Option payload type; verify probe_pt2.xi (expect exit 0).

### BUG 17 (NEW) — runtime-runtime `Str ==` on Vec[Str] ELEMENTS lowers to pointer compare

- **Construct:** comparing two runtime `Str` values that come from `Vec[Str]`
  ELEMENT loads, e.g. `ga[i] == gb[j]` where `ga`/`gb` are `Vec[Str]`:
  probe_jac2.xi — both elements are "he" yet `==` is false; `ga[0] == "he"`
  (literal) is true. The emitted IR for the element-element comparison
  contains NO `strcmp` call (the element loads degrade the operand type so
  the equality lowers to pointer icmp), while plain runtime `Str == Str`
  (vars/slices, probe_seq/probe_seq4) DOES emit `strcmp`.
- **Impact on stdlib:** `text.similarity.jaccard_similarity` and the
  lcp/lcsuffix helpers compared slices — all now use a byte-wise `_str_eq`
  helper (documented) so string content comparison never depends on the
  degraded path. This is the same family as BUG 12 (Vec element type
  degradation): Float64 elements load as i64+sitofp, Str elements lose the
  content-equality lowering.
- **Fix direction (compiler):** the Vec element-load expression must carry
  the element's declared type (Str → strcmp on `==`; Float64 → `load
  double`); verify probe_jac2.xi (both comparisons true, exit 0).

### BUG 18 (NEW) — combining string + text.similarity + time in one program crashes (0xC0000405)

- **Construct:** one program importing `xiom.string`, `xiom.text.similarity`
  AND `xiom.time` (smoke_str2.xi) crashes at startup with 0xC0000405 before
  any output. Every PAIR of the three exits clean; each module alone is
  clean. Same family as BUG 16 (combination-specific startup crash — likely
  the unqualified `@MaybeUninit.clone`/invariant-check stubs colliding when
  several Option payload shapes coexist).
- **Refined trigger (time-only programs):** with only `xiom.time` imported,
  `strptime("2026-13-01", "%Y-%m-%d")` followed by `strptime("2026-08-11",
  "%Y-%m-%d %Q")` in the SAME program crashes (0xC0000405); the same specs
  individually, or any other spec pair, exit clean. The unsupported-
  conversion early-return path (`%Q`) after a range-check failure path
  miscompiles at -O2 (probe_tm9: the pair fails; both single calls pass).
- **Stdlib handling:** the batch's smokes are split — smoke_str2.xi covers
  string+text.similarity (proven pair), smoke_time2.xi covers time with the
  `%Q` check removed (the fn is correct — verified by single-call probes;
  TODO(compiler) notes in the affected modules).
- **Fix direction (compiler):** same as BUG 16 — module-qualify generated
  symbols and emit payload-accurate clone/invariant code; verify
  smoke_str2.xi recombined (expect exit 0).
