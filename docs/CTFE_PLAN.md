<!-- Copyright (c) 2026 Eleftherios Notas and XIOM Foundation -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM CTFE -- Compile-Time Function Evaluation

**Version:** v0.55 (Implementation Complete -- Phase A + B)
**Date:** 2026-08-03 (post-implementation audit)
**Status:** **Phase A + B COMPLETE.** Phase C (type-level CTFE) deferred to post-selfhost.

---

## 1. IMPLEMENTATION STATUS

| Milestone | Status | Reality |
|-----------|--------|---------|
| **Phase A: CEE** | [OK] COMPLETE | Full const expression evaluator. Arithmetic, comparison, boolean, unary, builtins, `if`/`match` folding, const refs, `const { }` blocks. |
| **Phase B: VM Interpreter** | [OK] COMPLETE | Tree-walking interpreter. Pure function evaluation, recursion (depth 1000), while loops, struct/array, pattern matching with bindings. |
| **Phase C: Type-Level CTFE** | [FAIL] DEFERRED | `type T = [const { expr }]Int`, constant generic inference. Post-selfhost. |

---

## 2. PHASE A -- Const Expression Evaluator [OK]

### 2.1 What's Implemented

| Feature | Implementation | Tests |
|---------|---------------|-------|
| Integer arithmetic (+ - * / %) | `evaluate_const_init` in `expr.rs` | ctfe_phase_a.xi (S1) |
| Float arithmetic (+ - * /) | Same | ctfe_phase_a.xi (S2) |
| Comparison (== != < > <= >=) Int | Same | ctfe_phase_a.xi (S3) |
| Comparison (== != < > <= >=) Float | Same | ctfe_phase_a.xi (S4) |
| Boolean ops (and, or, not) | Same | ctfe_phase_a.xi (S5) |
| Unary ops (-X, !X, ~X) | Same | ctfe_phase_a.xi (S6) |
| Const variable references | `self.local.constants` resolution | ctfe_phase_a.xi (S7) |
| `sizeof::<T>()` | Turbofish + method form | ctfe_phase_a.xi (S8) |
| `align_of::<T>()` | Natural alignment calculation | ctfe_phase_a.xi (S8) |
| `type_id::<T>()` | FNV-1a hash of type name | ctfe_phase_a.xi (S8) |
| `field_offset::<T>(name)` | Byte offset of named field | ctfe_builtins_struct.xi |
| `is_signed::<T>()` | Bool: is type signed? | ctfe_phase_a.xi (S9) |
| `const { expr }` block | `Expr::ConstBlock` -> CTFE eval | ctfe_phase_a.xi (S10) |
| `if`/`else` folding | Evaluate condition, pick branch | ctfe_phase_a.xi (S11) |
| `match` folding | Pattern matching with bindings, Some/None/Ok/Err | ctfe_phase_a.xi (S12) |
| String comparison | `==`, `!=` on compile-time strings | code |
| Bitwise ops (Shl, Shr, BitAnd, BitOr, BitXor) | Integer bitwise folding | code |
| Enum constructors | `Some(expr)`, `None`, `Ok(expr)`, `Err(expr)` recursively evaluated | code |

### 2.2 Architecture

```
Source -> Parser -> AST -> Type Checker -> Evaluate Const Inits -> Codegen
                                              |
                                       +------+------+
                                       |  CTFE Phase A |
                                       |  (evaluate_   |
                                       |   const_init)  |
                                       |               |
                                       |  - arithmetic |
                                       |  - comparison |
                                       |  - boolean    |
                                       |  - builtins   |
                                       |  - if/match   |
                                       |  - const refs |
                                       `--------------+
                                               |
                                       Const values inserted
                                       into AST as literals
```

### 2.3 Code Location

- **Evaluator:** `crates/xiom-codegen/src/expr.rs` -- `evaluate_const_init()`
- **Checker registration:** `crates/xiom-check/src/lib.rs` -- `register_builtins()`
- **Builtin helpers:** `crates/xiom-codegen/src/lib.rs` -- `align_of_type()`, `type_id_of()`, `field_offset_of()`, `is_signed_xiom_type()`
- **Formatter:** `crates/xiom-fmt/src/expr.rs` -- `ConstBlock` formatting

---

## 3. PHASE B -- Full CTFE Interpreter [OK]

### 3.1 What's Implemented

| Feature | Implementation | Test |
|---------|---------------|------|
| Tree-walking interpreter | `CtfeEngine` in `crates/xiom-ctfe/src/lib.rs` | ctfe_phase_b.xi |
| Pure function calls | `eval_function()` with depth 1000 limit | factorial(5) = 120 |
| Recursion | Depth tracking, step limit (100K) | factorial(10) = 3628800 |
| While loops | Loop body re-evaluated with step counting | sum_to(10) = 55 |
| if/elif/else in functions | Branch evaluation | max_of_three(3,5,9) = 9 |
| Pattern matching with bindings | `pattern_matches_with_bindings` | Some(v) => v |
| Enum constructors | Some/None/Ok/Err handling | match Ok(42) { ... } |
| Builtins | str_len, str_concat, int_to_string | ctfe_phase_b.xi |
| Arena allocator | 256MB bounded heap for CTFE allocations | -- |
| Integration | `RefCell<CtfeEngine>` on `IrEmitter` | -- |

### 3.2 Architecture

```
crates/xiom-ctfe/
|-- Cargo.toml
`-- src/
    `-- lib.rs  (632 lines)
        |-- CtfeEngine      -- top-level evaluator
        |-- CtfeContext     -- evaluation state (depth, steps, locals)
        |-- CtfeValue       -- runtime values (Int, Float, Bool, Str, Struct, Variant)
        |-- CtfeArena       -- bounded memory allocator
        |-- CtfeError       -- error types (RecursionLimit, Timeout, TypeError, etc.)
        |-- eval_expr()     -- expression evaluator
        |-- eval_stmt()     -- statement evaluator
        |-- eval_block()    -- block evaluator
        |-- eval_function() -- function call evaluator
        |-- eval_binary()   -- binary op evaluator
        |-- eval_unary()    -- unary op evaluator
        |-- eval_builtin()  -- builtin function handler
        `-- pattern_matches() -- pattern matching
```

### 3.3 Purity Analysis (Simplified)

Currently, all non-method, non-generic functions are registered for CTFE. Full purity analysis (detecting I/O, FFI, mutable globals) is deferred to a future hardening pass.

### 3.4 Safety Limits

| Limit | Value | Purpose |
|-------|-------|---------|
| Recursion depth | 1000 | Prevent stack overflow |
| Step count | 100,000 | Prevent infinite loops |
| Arena size | 256 MB | Prevent memory exhaustion |
| Timeout | None (implicit via step count) | -- |

---

## 4. PHASE C -- Type-Level CTFE (Deferred to Post-Selfhost)

### 4.1 Remaining Scope

| Feature | Priority | Effort |
|---------|----------|--------|
| `type Buf = [const { N * 2 }]i8` | MEDIUM | 1 week |
| `fn f<const N: Int>()` -- const generics | MEDIUM | 2 weeks |
| Memoization cache for CTFE results | LOW | 3 days |
| `@comptime` annotation on functions | LOW | 2 days |
| CTFE trace mode (`--ctfe-trace`) | LOW | 2 days |

---

## 5. TEST RESULTS

| Test | Assertions | Status |
|------|-----------|--------|
| `ctfe_phase_a.xi` | 54 assertions | [OK] All pass |
| `ctfe_builtins_struct.xi` | 4 assertions | [OK] All pass |
| `ctfe_phase_b.xi` | 8 assertions | [OK] All pass |
| **Total** | **66** | **100% pass** |

E2E: 17/17 pass (including CTFE + eco + ASM + Never + spawn).

---

**Status:** CTFE Phase A + B COMPLETE. Phase C deferred. Production-grade with 66 assertions and safety limits. Ready for selfhost bootstrap usage (table generation, constant computation).

---

## 6. INTEGRATION AUDIT (2026-08-10 -- pre-selfhost review)

### Integration A: CTFE + Unsafe Confinement (v0.57)

**Interaction.** Confinement contracts (`requires: x >= sqrt(y) + 5`) may contain
complex arithmetic. When the checker evaluates a `requires` expression against
constant arguments, the CTFE evaluator (`CtfeEngine::eval_expr`) reduces it; if
it cannot reduce to a constant (runtime variable), it falls back to the existing
runtime guard generation.

**What to do (Implementation Note -- post-selfhost, optional):** in
`xiom-check/src/lib.rs`, when evaluating a `requires` expression with constant
arguments, call `CtfeEngine::eval_expr`. Architecture is already designed for
this (`eval_expr` handles binary ops, builtins, recursion). No change needed
before or during selfhost.

### Integration C: CTFE + Scaling Architecture (v0.58-v0.60)

**Interaction (the "Gotcha").** The CTFE engine is a `RefCell<CtfeEngine>` on the
`IrEmitter`. In parallel compilation (128 threads), a shared emitter panics on
concurrent RefCell access; per-file clones recompute `sizeof::<Int>()` thousands
of times.

**Production Fix (Zero-Cost, at Scaling Phase 5 -- NOT before).**
1. Move the CTFE result cache OUT of the IrEmitter into a thread-safe global:
   `CtfeCache = RwLock<HashMap<u64, CtfeValue>>` stored in the `SyncRegistry`.
2. File workers lock once to check the cache; miss -> compute (< 1 ms) -> store.
3. Result: no duplicate computation across 10K files; minimal lock contention.

**Effort:** ~2 days of refactoring. **Do this at Scaling Phase 5, not earlier.**
Selfhost is single-threaded and < 200 files -- the current RefCell design works
flawlessly at that scale.
