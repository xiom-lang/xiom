# XIOM CTFE — Compile-Time Function Evaluation

**Version:** v0.54.0 (Roadmap Phase)
**Date:** 2026-08-02
**Status:** Planning — Pre-Selfhost

---

## 1. MOTIVATION

XIOM currently evaluates only trivial constant expressions at compile time (literal
folding, `5 + 5 → 10`). A full CTFE system would:

| Benefit | Current | With CTFE |
|---------|---------|-----------|
| Generic array sizes | `type Buf = [N*8]i8` — N must be literal | `type Buf = [compute_size()]i8` |
| Compile-time validation | Runtime panics only | `@comptime assert(align_of(T) == 8)` |
| Dead code elimination | None | `if is_debug_build() { ... }` folded away |
| Embedded constants | Hardcoded in source | `const TABLE = generate_lookup()` at build time |
| Type-level computation | None | `type SmallVec = Vec[T, min(16, N)]` — generic over N |
| Self-host bootstrap | Must hand-write constants | Compiler can compute table sizes, offsets, hashes |

---

## 2. ARCHITECTURE — Two-Phase Design

### Phase A: Const Expression Evaluator (CEE) — v0.54-target
A lightweight const folder that handles the most common CTFE use cases
**without a full interpreter**. Works on the typed AST before codegen.

**Scope:**
- Arithmetic on `Int`/`Float64`/`Bool`
- `if`/`match` on compile-time-known conditions
- Builtin functions: `sizeof()`, `align_of()`, `type_id()`, `field_offset()`
- `const` block: `const { ... }` evaluates at compile time
- No loops, no user-function calls (yet)

### Phase B: Full CTFE Interpreter — v0.55-target
A bytecode interpreter that can evaluate arbitrary pure XIOM functions
at compile time. Integrated into the checker/codegen pipeline.

**Scope:**
- Evaluate any pure function (no I/O, no mutable globals)
- Recursion with depth limit (configurable, default 1000)
- Heap allocation within compile-time evaluation (arena-based, discarded after)
- CTFE of generic/monomorphised functions
- `@comptime` attribute on function declarations

---

## 3. PHASE A — Const Expression Evaluator

### 3.1 AST Changes

```rust
// xiom-ast/src/lib.rs
pub enum TopDecl {
    // NEW: Compile-time constant declaration
    Const(ConstDecl),
    // ... existing variants
}

pub struct ConstDecl {
    pub name: Ident,
    pub ty: Option<Type>,       // optional type annotation
    pub value: Expr,            // must be const-evaluable
    pub span: Span,
}

pub enum Expr {
    // NEW: Compile-time block expression
    ConstBlock(Vec<StmtOrExpr>, Span),
    // ... existing variants
}
```

### 3.2 Grammar Additions

```xiom
// Constant declaration — evaluated at compile time
const MAX_SIZE: Int = 64 * 1024;
const TABLE_SIZE: Int = size_of::<Int>() * MAX_SIZE;

// Compile-time block — entire block evaluated at compile time
// All bindings within are comptime-only
var x = const { MAX_SIZE / 2 + 1 };

// Type-level constants
type Buf = [const { MAX_SIZE * 2 }]i8;
```

### 3.3 Builtin Compile-Time Functions

| Function | Signature | Returns |
|----------|-----------|---------|
| `sizeof::<T>()` | `() -> Int` | Byte size of type T |
| `align_of::<T>()` | `() -> Int` | Alignment of type T |
| `type_id::<T>()` | `() -> Int` | Unique type identifier |
| `field_offset::<T>(name: Str)` | `(Str) -> Int` | Byte offset of named field in T |
| `is_signed::<T>()` | `() -> Bool` | Whether T is a signed integer type |

### 3.4 Evaluator Architecture

```
Source → Parser → AST → Type Checker → CEE (NEW) → Codegen
                                          │
                                   ┌──────┴──────┐
                                   │  ConstFolder │
                                   │  - arithmetic│
                                   │  - conditionals│
                                   │  - builtins  │
                                   │  - const refs│
                                   └──────────────┘
```

The CEE runs AFTER type checking (so types are known) and BEFORE codegen.
It walks the AST and replaces const-evaluable subtrees with their computed
values (literal expressions).

### 3.5 Evaluation Rules

1. **Literals** evaluate to themselves: `42 → 42`
2. **Binary ops** on const operands: `5 + 3 * 2 → 11`
3. **`if`/`match`** with const condition: evaluate only the taken branch
4. **`const` block**: evaluate all statements, last expression is the value
5. **`const` variable references**: substitute the computed value
6. **Builtins**: evaluate according to known semantics
7. **Non-evaluable** expressions are left as-is (no error — they become runtime code)

### 3.6 Example: AST Before/After

```xiom
// Source
const SIZE: Int = 64 * 1024;
var buf: [SIZE]i8 = [0; SIZE];
```

**Before CEE:**
```
Stmt::Const("SIZE", Expr::Binary(Int(64), Mul, Int(1024)))
Stmt::Var("buf", Array([0; Ident("SIZE")]))
```

**After CEE:**
```
// SIZE is stored in const table
Stmt::Var("buf", Array([0; Int(65536)]))  // SIZE → 65536
```

---

## 4. PHASE B — Full CTFE Interpreter

### 4.1 Design Principles

**Safety-first:** CTFE sandboxes all evaluation. No file I/O, no network, no
foreign function calls. Memory allocated during CTFE is arena-based and freed
when evaluation completes.

**Deterministic:** Same input always produces same output. No dependence on
system time, random seeds, or external state.

**Bounded:** Recursion depth limit (1000). Evaluation time limit (5s default).
Memory limit (256MB default). Prevents infinite loops from hanging compilation.

### 4.2 CTFE Interpreter Pipeline

```
  ┌──────────────┐
  │ CTFE Request │  Function call with all args known at compile time
  └──────┬───────┘
         │
  ┌──────▼───────┐
  │ Purity Check │  Must be pure: no I/O, no mut globals, no FFI
  └──────┬───────┘
         │
  ┌──────▼───────┐
  │ AST→Bytecode │  Compile function body to CTFE bytecode
  └──────┬───────┘
         │
  ┌──────▼───────┐
  │  Interpreter │  Stack-based VM with arena allocator
  └──────┬───────┘
         │
  ┌──────▼───────┐
  │  Result      │  Replace call site with computed literal value
  └──────────────┘
```

### 4.3 Bytecode VM

A minimal stack-based VM for CTFE:

| Opcode | Operands | Description |
|--------|----------|-------------|
| `PushInt` | i64 | Push integer constant |
| `PushFloat` | f64 | Push float constant |
| `PushBool` | bool | Push boolean constant |
| `PushStr` | string_id | Push interned string |
| `LoadLocal` | slot | Push local variable |
| `StoreLocal` | slot | Pop and store local |
| `LoadField` | field_idx | Pop struct, push field |
| `Add/Sub/Mul/Div` | — | Binary arithmetic |
| `CmpEq/Ne/Lt/Gt/Le/Ge` | — | Comparisons |
| `Jump` | offset | Unconditional branch |
| `JumpIf` | offset | Conditional branch (pop bool) |
| `Call` | fn_key | Push frame, jump to function |
| `Return` | — | Pop frame, push result |
| `NewStruct` | type_key, n_fields | Allocate struct on arena |
| `NewArray` | type_key, n_elems | Allocate array on arena |
| `Alloc` | byte_size | Allocate raw bytes on arena |

### 4.4 Purity Analysis

A function is CTFE-eligible if:
1. **No I/O** — no calls to `print`, `read_file`, `puts`, `printf`, etc.
2. **No mutable globals** — no writes to module-level `var`
3. **No FFI** — no `extern` function calls
4. **No thread operations** — no `spawn`, `channel` operations
5. **All callees are pure** — transitive closure of called functions
6. **No raw pointer arithmetic** — `*T` operations only within arena
7. **No `unsafe` blocks** (initially; may relax later)

Purity is inferred automatically. No annotation needed for Phase B.

### 4.5 `@comptime` Annotation

```xiom
// Explicit: force evaluation at compile time (error if not possible)
@comptime
fn compute_table() -> [256]Int {
    var table: [256]Int = [0; 256];
    var i = 0;
    while i < 256 {
        table[i] = i * i;
        i = i + 1;
    }
    return table;
}

// Implicit: compiler may CTFE when all args are const
fn factorial(n: Int) -> Int {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
}

// Usage: CTFE'd because arg is const
const FACT10: Int = factorial(10);  // → 3628800 at compile time
```

### 4.6 Error Handling

CTFE errors produce clear diagnostics, not runtime panics:

```
error[CTFE]: recursion depth limit (1000) exceeded in 'factorial'
  ┌─ test.xi:5:10
  │
5 │     return n * factorial(n - 1);
  │            ^^^^^^^^^^^^^^^^^^^^ infinite recursion?
  │
  = note: CTFE recursion limit is 1000. Increase with --ctfe-depth=N.

error[CTFE]: 'read_file' is not CTFE-pure
  ┌─ test.xi:a:18
  │
a │     var data = read_file("config.json");
  │                 ^^^^^^^^^^^^^^^^^^^^^^^^ I/O not allowed in CTFE

error[CTFE]: evaluation timed out after 5.0s
  = note: Increase timeout with --ctfe-timeout=N.
```

---

## 5. IMPLEMENTATION PLAN

### Milestone 1: CEE Foundation (v0.54 — 2 weeks)
- [ ] `crates/xiom-ctfe/` — new crate
- [ ] `Const` top-level declaration (`const X: T = expr;`)
- [ ] `ConstBlock` expression (`const { ... }`)
- [ ] `ConstFolder` struct walking typed AST
- [ ] Arithmetic, comparisons, boolean folding
- [ ] `if`/`match` with const conditions
- [ ] Builtins: `sizeof`, `align_of`, `type_id`
- [ ] Integration: CEE runs between checker and codegen
- [ ] Tests: 50+ test cases covering edge cases

### Milestone 2: CTFE VM (v0.54 — 3 weeks)
- [ ] Bytecode instruction set and codec
- [ ] Stack-based interpreter with arena allocator
- [ ] AST→bytecode compiler for CTFE-eligible functions
- [ ] Purity analysis pass
- [ ] Recursion depth limit and cycle detection
- [ ] `@comptime` annotation
- [ ] CTFE result caching (memoization)
- [ ] Error diagnostics with source spans
- [ ] Tests: 100+ test cases

### Milestone 3: Integration & Hardening (v0.55 — 2 weeks)
- [ ] Type-level CTFE (`type T = [const { expr }]Int`)
- [ ] Generic parameter CTFE (`fn f<const N: Int>()`)
- [ ] Constant generic inference from CTFE results
- [ ] Self-host bootstrap using CTFE for table generation
- [ ] Performance: CTFE memoization cache
- [ ] Documentation: CTFE chapter in language guide
- [ ] Full test suite: 200+ CTFE-specific tests

---

## 6. RISKS & MITIGATIONS

| Risk | Mitigation |
|------|------------|
| CTFE infinite loop hangs compiler | Hard timeout (5s default), depth limit (1000) |
| CTFE allocates unbounded memory | Arena with 256MB cap, freed after evaluation |
| Non-deterministic CTFE results | Purity analysis rejects I/O, time, RNG |
| CTFE slows down compilation | Memoization cache; only evaluate when needed |
| Complex CTFE bugs hard to debug | CTFE trace mode (`--ctfe-trace`) prints step-by-step |

---

## 7. RELATIONSHIP TO SELFHOST

The self-host compiler (XIOM written in XIOM) will benefit from CTFE in
several ways:

1. **Token tables and keyword maps** generated at compile time via CTFE
2. **Parser tables** (LL(1) or Pratt) computed via CTFE instead of hand-coded
3. **Type IDs and vtable layouts** computed at compile time
4. **Constant IR patterns** (common codegen sequences) pre-computed

Without CTFE, the self-host compiler would need to compute these at startup
or hard-code them. With CTFE, the compiler binary contains pre-computed data.

---

## 8. SUMMARY

CTFE is a foundational feature that enables:
- **Type-level computation** (sized arrays, generic constants)
- **Zero-cost abstractions** (compile-time code generation)
- **Self-host efficiency** (pre-computed tables)
- **Better error detection** (compile-time assertions)

**Recommendation:** Implement Phase A (CEE) before selfhost begins. Phase B
(CTFE VM) can be developed in parallel with selfhost work.

**Status:** APPROVED for v0.54 roadmap.
