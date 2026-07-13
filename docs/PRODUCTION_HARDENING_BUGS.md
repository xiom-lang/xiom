# Production Hardening — Deep Bugs & Open Items

## Session: 2026-07-13 — feat/guardian hardening
Branch: `feat/guardian`
State: 37/37 deterministic smoke, 84/84 e2e, all regression gates green

---

## BUG-001: SHA-256 produces wrong hashes (CODEGEN — Long While-Loop)

**Severity:** HIGH (correctness — cryptographic hash wrong)
**Status:** OPEN — root cause narrowed to codegen issue with long while-loops

### Symptoms
- `sha256_hex("")` produces hex string that does NOT match `e3b0c4...`
- `sha256_hex("abc")` does NOT match `ba7816bf...`
- Determinism test passes: `sha256(data) == sha256(data)` (consistent, just wrong)

### Critical Finding (2026-07-13 round 2)
**Round 0-9 verified PERFECT against Python reference.** After round ~10-19, the state diverges:
- Round 9 (10 iterations): `a=0xefe8bf51, e=0x53072289` ✓
- Round 19 (20 iterations): should be `a=0x0a480908, e=0x900d27ac` but XIOM produces DIFFERENT values ✗

**This is a CODEGEN BUG in how XIOM compiles long-running while-loops.** The exact same arithmetic in a 10-iteration loop produces correct results; the same arithmetic in a 64-iteration loop does not. The loop body is identical — only the iteration count differs.

### Verified Correct
- [x] `_SHA256_INIT` constants match RFC 6234
- [x] `_SHA256_K` constants match RFC 6234
- [x] `_u32_mask` tested in isolation — correct
- [x] `_u32_add` tested in isolation — wraps at 2^32 correctly
- [x] `_u32_rotr` tested in isolation — rotate right correct
- [x] `_u32_shr` tested — unsigned right shift via division correct
- [x] `_u32_shl` tested — left shift via multiplication correct
- [x] All round operations verified for rounds 0-9 against Python reference
- [x] Byte assembly (big-endian) matches RFC
- [x] Hex encoding (`sha256_hex`) produces correct hex for known byte values
- [x] Large loop body (same as SHA-256 round) with 64 iterations using simple values — correct
- [x] All function calls inlined — no change (rules out function-call aliasing)
- [x] Loop split into 4×16 chunks — no change
- [x] Array-based `s[8]` vs separate `a-h` variables — no change
- [x] Word expansion w[0..63] verified correct
- [x] `i64` multiplication never overflows (max shift verified)

### Root Cause Hypothesis
The 64-round while loop triggers a codegen optimization that incorrectly aliases temporary alloca slots. With 64 iterations, the compiler reuses stack slots for temporaries, and somewhere around iteration 10-19, two temporaries that should be DISTINCT end up sharing the same alloca, causing one value to overwrite another.

### Attempted Fixes (all failed)
1. **C runtime integration**: Replaced no-op `xiom_shani_sha256_compress` with software SHA-256 → crashed (ACCESS_VIOLATION). Pointer/linking issue.
2. **Function call inlining**: All helper functions inlined in loop body → no change
3. **Loop chunking**: 64→4×16 → no change
4. **Separate temporaries**: `s0old..s7old` read before writes → no change
5. **Vec[Int] for w**: Replaced fixed-size array → no change

### Next Steps (in priority order)
1. **Disable LLVM optimizations** for `_sha256_block` function — test with `-O0`
2. **Check IR for alloca reuse**: Compare IR of 10-round vs 64-round version
3. **Add `volatile`-like barriers**: Force all variables to stack after each round
4. **Manual unrolling**: Write 64 rounds as 64 sequential blocks (no loop)
5. **C FFI bridge**: Write SHA-256 compression in C, link as separate object file

### Test File
`examples/stdlib_smoke/smoke_crypto_known_vectors.xi` — known-vector tests, expected to fail

---

## BUG-002: async module inaccessible from bare modules (LANGUAGE)

**Severity:** MEDIUM (env-dependent module)
**Status:** OPEN — language feature needed

### Root Cause
`async` is a reserved keyword (`TokenKind::Async` in lexer, line 20 of `crates/xiom-lexer/src/lib.rs`). The parser uses `TokenKind::Async` in function declaration parsing (lines 167, 385 of `crates/xiom-parser/src/lib.rs`). Module paths containing `async` (e.g., `xiom.async.Executor`) cannot be lexed as identifiers.

### Impact
- `use xiom.async` compiles but `async.Executor.new()` fails with "undefined variable"
- `xiom.async.Executor` works as a type annotation (type checker resolves it)
- Call expressions fail because the parser lexes `async` as a keyword token

### Workaround (after partial fix)
- `async.Executor` works as a **type annotation** after the contextual keyword fix
- Expression paths (`async.Executor.new()`) still fail — checker resolves module paths differently in expression vs type context
- `Executor.new()` works inside the `xiom.async` module itself
- External code cannot call methods on types from `xiom.async` via expression paths

### Fix Applied (commit `03bacd7`)
- Removed `TokenKind::Async` from lexer
- `async` is now lexed as `Ident("async")` in all contexts
- Parser checks for `Ident("async")` followed by `TokenKind::Fn` to trigger async-fn
- Result: `use xiom.async` imports work; `async.Executor` as type path works

### Remaining (BUG-002a)
- Expression resolution in checker treats module paths differently from type paths
- `async.Executor.new()` fails with "undefined variable 'async'"
- Needs checker fix to resolve module-qualified expression paths

---

## BUG-003: IO module functions not pub (STDLIB)

**Severity:** LOW (already fixed)
**Status:** RESOLVED (commit `eb64620`)

### Fix
Made `io.print`, `io.println`, `io.args` pub. Smoke test now passes with `--ignored`.

---

## Vec[UInt8] elem_size — Architecture Decision

**Severity:** N/A
**Status:** IMPLEMENTED (commit `618ad0d`)

### Decision
Vec now has 4 fields: `{data: i8*, len: i64, cap: i64, elem_size: i64}`. Narrow types (UInt8, Int16, etc.) get elem_size=1/2/4 instead of default 8. All codegen sites (new, push, pop, get, index read, index write) updated with `emit_elem_store`/`emit_elem_load` helpers that use runtime elem_size dispatch.

### Trade-off
- Pro: Correct byte-level storage for Vec[UInt8], enabling compact hex buffers
- Con: Adds icmp+branch per push/pop/get/index operation (minor overhead)
- Con: Old code assuming 3-field Vec would break (all updated)
