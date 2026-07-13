# Production Hardening — Deep Bugs & Open Items

## Session: 2026-07-13 — feat/guardian hardening
Branch: `feat/guardian`
State: 37/37 deterministic smoke, 84/84 e2e, all regression gates green

---

## BUG-001: SHA-256 produces wrong hashes (ALGORITHM)

**Severity:** HIGH (correctness — cryptographic hash wrong)
**Status:** OPEN — root cause not found after extensive debugging

### Symptoms
- `sha256_hex("")` produces hex string that does NOT match `e3b0c4...`
- `sha256_hex("abc")` does NOT match `ba7816bf...`
- Determinism test passes: `sha256(data) == sha256(data)` (consistent, just wrong)

### Verified Correct
- [x] `_SHA256_INIT` constants match RFC 6234
- [x] `_SHA256_K` constants match RFC 6234
- [x] `_u32_mask` tested in isolation — correct
- [x] `_u32_add` tested in isolation — wraps at 2^32 correctly
- [x] `_u32_rotr` tested in isolation — rotate right correct
- [x] `_u32_shr` tested — unsigned right shift via division correct
- [x] `_u32_shl` tested — left shift via multiplication correct
- [x] `_sha256_sigma0/1` formulas match RFC
- [x] `_sha256_eps0/1` formulas match RFC
- [x] `_sha256_ch` formula matches RFC
- [x] `_sha256_maj` formula matches RFC
- [x] Main round loop matches RFC step-by-step
- [x] Byte assembly (big-endian) matches RFC
- [x] Hex encoding (`sha256_hex`) produces correct hex for known byte values

### Hypothesis
The bug is likely in XIOM codegen behavior affecting 32-bit arithmetic across 64 composed rounds. Individual operations test correctly, but the full composition produces wrong state updates. Possible causes:

1. **Variable aliasing in while loop**: `a`, `b`, `c`, `d`, `e`, `f`, `g`, `h` are reassigned each iteration. If the codegen aliases the alloca incorrectly, later assignments might shadow earlier values.
2. **Array writes via `_u32_add`**: `state[0] = _u32_add(state[0], a)` — the `_u32_add` call evaluates `state[0]` as an argument. If `state[0]` is mutated by the function call itself (side effect from temp variable collision), the addition would use wrong values.
3. **i64 sign extension in ~ operator**: `~x` on 32-bit positive values produces 64-bit NOT. Masking with `_u32_mask` at end should handle this, but intermediate values might exceed i64 range.

### Attempted Fixes
1. **C runtime integration**: Replaced no-op `xiom_shani_sha256_compress` with software SHA-256 → crashed (ACCESS_VIOLATION 0xC0000005). Likely pointer/linking issue between XIOM malloc'd buffers and C function.
2. **Vec[elem_size] change**: Made Vec[UInt8] store bytes compactly (1 byte each) → correct behavior, no crash, but still wrong hashes (algorithm unaffected by storage layout).

### Next Steps
1. Add intermediate-value logging to C runtime SHA-256 (compiled separately)
2. Test with known intermediate values (after round 0, round 1, etc.)
3. Investigate variable aliasing in while-loop bodies (codegen temp allocation)
4. Try implementing SHA-256 in a separate XIOM module to isolate from crypto.xi

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

### Workaround
- `Executor.new()` works inside the `xiom.async` module itself
- External code cannot call methods on types from `xiom.async`

### Fix Required
Add raw-identifier support (`r#async`) to lexer/parser, or change `async` from reserved keyword to contextual keyword (like Rust's approach).

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
