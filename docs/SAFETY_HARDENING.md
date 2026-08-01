# XIOM — Honest Gaps & Safety Hardening

**Version:** v0.54 → v0.56 (Pre-Selfhost)
**Date:** 2026-08-02
**Status:** Planning
**Principle:** _A language that refuses to crash at runtime must be strict at compile time._

---

## 1. THE HONEST GAPS

After analyzing XIOM against Rust, Zig, and C++ for production systems programming,
these are the genuine gaps that matter — not features for features' sake, but
things that prevent real bugs and make safe code the path of least resistance:

| # | Gap | Impact | Rust Has? | Zig Has? |
|---|-----|--------|-----------|----------|
| 1 | **`?` operator** — error propagation sugar | Makes error handling 5x less verbose. Without it, programmers skip error checks. | ✓ | `try` |
| 2 | **Debug overflow + bounds checks** — UB prevention | Prevents integer overflow CVEs and buffer overruns. #1 source of security bugs. | ✓ | ✓ |
| 3 | **Match exhaustiveness** — compiler verifies all cases | Prevents "forgot to handle error" bugs. | ✓ | ✓ |
| 4 | **Never type (`!`)** — divergent functions | Makes exhaustiveness checking sound. `exit()` should be known to never return. | ✓ | ✓ |
| 5 | **`defer` / scope guard** | Prevents resource leaks. Cleanup code lives next to allocation. | `Drop` | `defer` |

---

## 2. IMPROVEMENT 1: `?` OPERATOR — Error Propagation Sugar

### 2.1 The Problem

```xiom
// CURRENT: every Result return requires explicit match — 8 lines of boilerplate
fn read_config() -> Result[Config, Error] {
    var file = match open_file("config.json") {
        Ok(f) => f,
        Err(e) => return Err(e),  // boilerplate
    };
    var data = match read_all(file) {
        Ok(d) => d,
        Err(e) => return Err(e),  // boilerplate
    };
    var config = match parse_json(data) {
        Ok(c) => c,
        Err(e) => return Err(e),  // boilerplate
    };
    return Ok(config);
}

// TARGET: ? operator — 3 lines, zero boilerplate
fn read_config() -> Result[Config, Error] {
    var file = open_file("config.json")?;
    var data = read_all(file)?;
    var config = parse_json(data)?;
    return Ok(config);
}
```

### 2.2 Semantics

```xiom
expr?
// Desugars to:
match expr {
    Ok(v) => v,
    Err(e) => return Err(e.into()),  // .into() converts error types if needed
}
```

`?` only works inside functions returning `Result[T, E]` or `Option[T]`.
The compiler rejects `?` in functions returning non-Result types — compile error.

### 2.3 Implementation

**Parser:** Add `TokenKind::Question` handling in expression parsing. The `?` is
a postfix operator with the same precedence as `.` field access.

**AST:** No new AST node needed — desugar during parsing or early in the checker.

```rust
// In parser, after parsing an expression:
if self.peek(TokenKind::Question) {
    self.advance();
    // Desugar expr? to match { Ok(v) => v, Err(e) => return Err(e) }
    let ok_ident = Ident::new("__ok", span);
    let err_ident = Ident::new("__err", span);
    let ret_stmt = Stmt::Return(
        Some(Expr::Err(Box::new(Expr::Ident(err_ident.clone())), span)),
        span,
    );
    let match_expr = Expr::Match(
        Box::new(expr),
        vec![
            MatchArm {
                pattern: Pattern::Enum("Ok", vec![Pattern::Bind(ok_ident)]),
                guard: None,
                body: MatchBody::Expr(Expr::Ident(ok_ident)),
            },
            MatchArm {
                pattern: Pattern::Enum("Err", vec![Pattern::Bind(err_ident)]),
                guard: None,
                body: MatchBody::Block(Block { stmts: vec![StmtOrExpr::Stmt(ret_stmt)] }),
            },
        ],
        span,
    );
    return self.parse_expr_continue(match_expr);
}
```

**Checker:** Verify that the enclosing function returns `Result[T, E]` or `Option[T]`.
If the function returns `Int`, `?` is a compile error: "`?` operator requires function to return Result or Option."

**Codegen:** The desugaring produces standard `match` + `return Err(...)` that goes
through existing match compilation. No new codegen needed.

### 2.4 Error Type Conversion

When `?` propagates an error from `fn a() -> Result[T, E1]` through `fn b() -> Result[U, E2]`,
the compiler needs to convert `E1` to `E2`. This follows Rust's `From` trait pattern:

```xiom
// Auto-implemented by the compiler for error conversion:
// If E1 can be converted to E2 (same type or explicit From impl), allow ?.
fn b() -> Result[U, E2] {
    var v = a()?;  // ok if E1 == E2 or From<E1, E2> exists
}
```

---

## 3. IMPROVEMENT 2: DEBUG SAFETY CHECKS

### 3.1 The Problem

C/C++ have **undefined behavior** on integer overflow, out-of-bounds access,
and null pointer dereference. These cause **millions of CVEs**. XIOM must
eliminate UB entirely.

```c
// C code — UNDEFINED BEHAVIOR, no error, silently corrupts memory:
int x = INT_MAX + 1;     // signed overflow = UB
int arr[10];
arr[20] = 42;            // out-of-bounds = UB
int* p = NULL;
*p = 42;                 // null deref = UB
```

```xiom
// XIOM — guaranteed behavior:
// Debug mode: panics with clear error message + stack trace
// Release mode: wraps (for Int), saturates (for Vec index), traps (for null)
var x: Int = 2147483647 + 1;  // Debug: PANIC "integer overflow at main.xi:42"
                               // Release: wraps to -2147483648
var arr = [1, 2, 3];
var y = arr[20];               // Debug: PANIC "index 20 out of bounds (len=3)"
                                // Release: traps
```

### 3.2 Check Generation

**Overflow checks:** Insert `llvm.sadd.with.overflow` / `llvm.uadd.with.overflow`
intrinsics in debug mode. In release mode, use plain `add` with wrapping semantics.

```llvm
; Debug mode: checked arithmetic
%tmp = call { i64, i1 } @llvm.sadd.with.overflow.i64(i64 %a, i64 %b)
%val = extractvalue { i64, i1 } %tmp, 0
%ovf = extractvalue { i64, i1 } %tmp, 1
br i1 %ovf, label %overflow_trap, label %ok

overflow_trap:
  call void @xiom_panic(i8* "integer overflow at main.xi:42")
  unreachable

ok:
  ; continue with %val
```

```llvm
; Release mode: plain arithmetic
%val = add i64 %a, %b
```

**Bounds checks:** For every `arr[idx]`, insert a bounds check in debug mode.

```llvm
; Debug mode:
%in_bounds = icmp ult i64 %idx, %arr_len
br i1 %in_bounds, label %ok, label %oob_trap

oob_trap:
  call void @xiom_panic(i8* "index %idx out of bounds (len=%arr_len)")
  unreachable
```

**Null checks:** For every raw pointer dereference, insert a null check.

```rust
// Codegen for *p where p: *T:
if debug_mode {
    self.emitln(format!("  %is_null = icmp eq {ptr_ty} %{p}, null"));
    self.emitln(format!("  br i1 %is_null, label %null_trap, label %ok"));
    // null_trap: xiom_panic("null pointer dereference")
}
```

### 3.3 CLI Integration

```
xiom --debug file.xi       # debug mode: all checks enabled
xiom --release file.xi     # release mode: checks removed, wrapping semantics
xiom --release-safe file.xi # release mode: bounds + null checks, no overflow checks
```

---

## 4. IMPROVEMENT 3: MATCH EXHAUSTIVENESS

### 4.1 The Problem

```xiom
// CURRENT: no exhaustiveness check — missing arm silently compiles:
fn describe(result: Result[Int, Str]) -> Str {
    match result {
        Ok(v) => "success",   // missing Err arm — no error!
    }
    // returns void, undefined behavior at call site
}

// TARGET: compile error
// Error: non-exhaustive match — missing pattern: Err(_)
```

### 4.2 Implementation

The checker already knows the scrutinee type and the list of patterns. Add an
exhaustiveness check after processing all match arms:

```rust
fn check_match_exhaustiveness(
    scrutinee_ty: &CheckedType,
    arms: &[MatchArm],
    span: Span,
) -> Result<(), CheckError> {
    match scrutinee_ty {
        // Result[T, E] requires Ok and Err
        CheckedType::Named(name) if name == "Result" => {
            let has_ok = arms.iter().any(|a| matches!(a.pattern, Pattern::Enum("Ok", _)));
            let has_err = arms.iter().any(|a| matches!(a.pattern, Pattern::Enum("Err", _)));
            if !has_ok || !has_err {
                let missing = if !has_ok { "Ok(_)" } else { "Err(_)" };
                return Err(CheckError::NonExhaustiveMatch {
                    missing: missing.to_string(),
                    span,
                });
            }
        }
        // Option[T] requires Some and None
        CheckedType::Named(name) if name == "Option" => {
            let has_some = arms.iter().any(|a| matches!(a.pattern, Pattern::Enum("Some", _)));
            let has_none = arms.iter().any(|a| matches!(a.pattern, Pattern::Enum("None", _)));
            if !has_some || !has_none {
                let missing = if !has_some { "Some(_)" } else { "None" };
                return Err(CheckError::NonExhaustiveMatch { missing: missing.to_string(), span });
            }
        }
        // Bool requires true and false
        CheckedType::Bool => {
            let has_true = arms.iter().any(|a| matches!(a.pattern, Pattern::Bool(true)));
            let has_false = arms.iter().any(|a| matches!(a.pattern, Pattern::Bool(false)));
            // A wildcard _ covers both
            let has_wildcard = arms.iter().any(|a| matches!(a.pattern, Pattern::Wildcard));
            if !has_wildcard && (!has_true || !has_false) {
                return Err(CheckError::NonExhaustiveMatch { ... });
            }
        }
        _ => {} // integers, strings, etc. — can't check exhaustiveness
    }
    Ok(())
}
```

---

## 5. IMPROVEMENT 4: NEVER TYPE (`!`)

### 5.1 The Problem

```xiom
fn exit_process(code: Int) -> ! {   // diverges — never returns
    unsafe { xiom_exit(code); }
    // without ! type, compiler complains about missing return
}

fn main() -> Int {
    var x = if config.is_valid() {
        0
    } else {
        exit_process(1);  // ! coerces to any type — no type error
    };
    return x;
}
```

The `!` type (called "never" or "bottom") represents computations that never
produce a value. It coerces to any type, making it useful for:
- `exit()`, `panic()`, `abort()` — functions that diverge
- Infinite loops: `loop { ... }` has type `!`
- Exhaustiveness: `match` on `!` has zero arms (unreachable)

### 5.2 Implementation

```rust
// In the type checker:
pub enum CheckedType {
    // ... existing variants ...
    Never,  // NEW: the ! type
}

// Coercion rule: ! coerces to any type
fn types_compatible(&self, from: &CheckedType, to: &CheckedType) -> bool {
    if matches!(from, CheckedType::Never) { return true; }
    // ... existing rules ...
}
```

---

## 6. IMPROVEMENT 5: `defer` STATEMENT

### 6.1 The Problem

```xiom
// CURRENT: cleanup code is far from allocation — easy to forget:
fn process_file(path: Str) -> Result[Data, Error] {
    var file = open_file(path)?;
    var data = parse_data(file)?;
    close_file(file);   // easy to forget if there's an early return
    return Ok(data);
}

// TARGET: defer — cleanup at allocation site, guaranteed to run:
fn process_file(path: Str) -> Result[Data, Error] {
    var file = open_file(path)?;
    defer { close_file(file); }  // runs on scope exit, even if error
    var data = parse_data(file)?;
    return Ok(data);
}
```

### 6.2 Semantics

`defer { ... }` executes the block when the enclosing scope exits — whether by
return, `?`, or falling off the end. Multiple `defer` statements execute in
LIFO order (last declared, first executed). This matches Go's `defer` and Zig's `defer`.

```xiom
fn example() {
    defer { io.println("third"); }
    defer { io.println("second"); }
    defer { io.println("first"); }
}
// Prints: first, second, third
```

### 6.3 Implementation

**AST:** Add `Stmt::Defer(Block, Span)`.

**Codegen:** At each return point in the scope, insert the deferred blocks in LIFO order:

```rust
fn compile_defer_scope(&mut self, body: &Block, defers: &[Block]) {
    // Save current defers
    let saved = self.local.deferred_blocks.clone();
    self.local.deferred_blocks.extend(defers.iter().cloned());

    // Compile body — any return/? will be intercepted
    self.compile_block(body)?;

    // Compile deferred blocks at scope exit (LIFO)
    for defer_block in self.local.deferred_blocks.iter().rev() {
        self.compile_block(defer_block)?;
    }

    // Restore
    self.local.deferred_blocks = saved;
}
```

For early returns, the codegen intercepts `ret` instructions and inserts defer
execution before them:

```llvm
; Before (simple ret):
;   ret i64 %val

; After (with defer):
;   ; execute defer blocks
;   call void @close_file(i8* %file)
;   ret i64 %val
```

---

## 7. ROADMAP INTEGRATION

These five improvements slot into the existing roadmap:

```
v0.54 ──► v0.55 ──► v0.56 ──► SELFHOST

v0.54:  CTFE Phase A + Binary Cache + Parallel Parse
        + ? operator + Debug overflow checks + Match exhaustiveness

v0.55:  OrcJIT MVP + Spawn codegen + Parallel Check
        + Never type (!) + defer statement

v0.56:  Send/Sync + Channel + Deadlock detection + Hot reload
        + Error type conversion (From trait) + Lifetime elision
```

---

## 8. IMPACT MATRIX

| Improvement | Prevents | Complexity | Rust Has? |
|-------------|----------|------------|-----------|
| `?` operator | Skipped error checks | Low — desugaring | ✓ |
| Debug overflow checks | Integer overflow CVEs | Low — LLVM intrinsics | ✓ |
| Match exhaustiveness | Missing arm bugs | Medium — checker | ✓ |
| Never type (`!`) | Wrong return types | Medium — type system | ✓ |
| `defer` statement | Resource leaks | Low — scope hook | Drop trait |

---

## 9. WHY THESE MATTER

XIOM's mission is **near-zero runtime errors**. The languages that achieve this
(Rust, Zig) do so through a combination of:

1. **Compile-time checks** (ownership, Send/Sync, exhaustiveness)
2. **Safe defaults** (overflow checks in debug, bounds checks always)
3. **Ergonomic error handling** (`?` — if it's painful, programmers skip it)
4. **Guaranteed cleanup** (`defer` / `Drop` — resources must be freed)

These five improvements close XIOM's gap with Rust/Zig on all four dimensions.
They don't add complexity for complexity's sake — each one eliminates a real
class of bugs that plague C/C++ codebases.

---

**Status:** APPROVED for v0.54 → v0.56 roadmap.
