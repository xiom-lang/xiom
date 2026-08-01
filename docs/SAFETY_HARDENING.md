# XIOM — Honest Gaps & Safety Hardening

**Version:** v0.54 → v0.56 (Pre-Selfhost)
**Date:** 2026-08-02 (revised after full audit)
**Status:** Planning
**Principle:** _SAFEST and EASIEST with less tokens — designed for AI agents to write correct code._

---

## 0. AUDIT FINDINGS — What Already Exists

Full audit of 10 areas against the codebase revealed XIOM is much further along
than initially assessed. **7 of 10 are fully implemented** with production-grade quality:

| # | Area | Status | Reality |
|---|------|--------|---------|
| 1 | Stdlib | **EXISTS** | 56 modules. TCP/UDP/HTTP/DNS, filesystem, JSON, regex, random, datetime, HashMap, HashSet, BTreeMap, Deque, PriorityQueue, string formatting, crypto (SHA, AES, Ed25519, PBKDF), SIMD, async primitives, contracts |
| 2 | Build System | **EXISTS** | `xiom build`, `xiom-pkg` crate with registry, lockfile, `package.xi` manifest |
| 3 | Error Handling | **EXISTS** | `?` operator already parsed (`TokenKind::Question` → `Expr::Try`). Result/Option full pipeline. |
| 4 | Generics | **EXISTS** | Full monomorphisation. Interface bounds `[T: Ord]`. Default method expansion. |
| 5 | Sanitizers | **EXISTS** | `--sanitize=address|undefined|leak|thread`. Tests covering all 4 types. |
| 6 | Doc Generator | **EXISTS** | `xiom-doc` crate. Markdown + CSS-styled HTML. Contracts in output. |
| 7 | FFI | **EXISTS** | `extern "C"`, `xiom-ffigen` bindgen crate. C→XIOM type mapping. |
| 8 | Debug Info | **PARTIAL** | DAP debug server exists. But no DWARF/PDB emission in codegen. |
| 9 | **Inline Asm** | **MISSING** | No `asm!()` support. Standalone `.asm` files compiled separately. |
| 10 | **LTO** | **MISSING** | No link-time optimization. No `-flto` flag. |

**Takeaway:** The 3 genuine gaps are **Inline Asm**, **LTO**, and **Debug Info emission**.
Everything else from the original assessment already exists and works.

---

## 1. GENUINE GAP #1: INLINE ASSEMBLY (`asm!()`)

### 1.1 Why It Matters

For a systems language, inline assembly is non-negotiable. It's needed for:
- CPU-specific instructions (SIMD, AES-NI, SHA-NI, RDRAND)
- Context switching (green threads, fibers)
- Accessing control registers (CR0-CR4, MSRs)
- Performance-critical hot loops
- Interrupt handlers and syscall trampolines

Without `asm!()`, XIOM must rely on separately compiled `.asm` files and `extern "C"` FFI.
This works but is fragile, non-portable, and opaque to the compiler's optimizer.

### 1.2 Proposed Syntax

```xiom
// Basic: output-only (produces a value)
var timestamp: Int = asm!("rdtsc" : "={rax}"(result));

// With inputs and clobbers
asm!("cpuid"
    : "={rax}"(eax), "={rbx}"(ebx), "={rcx}"(ecx), "={rdx}"(edx)
    : "{rax}"(leaf)
    : "memory", "cc"
);

// Volatile (prevent optimization across the asm)
asm! volatile ("int $$0x80"
    :
    : "{rax}"(syscall_nr), "{rdi}"(arg1)
    : "rcx", "r11", "memory"
);

// Multiple instructions
asm!(
    "mov rax, rdi",
    "add rax, rsi",
    : "={rax}"(sum)
    : "{rdi}"(a), "{rsi}"(b)
    : "cc"
);
```

### 1.3 Implementation

**Parser:** Add `TokenKind::Asm` keyword, parse the format string, output/input/clobber
constraints, and optional `volatile` modifier.

**AST:**
```rust
pub struct AsmBlock {
    pub template: String,           // assembly template string
    pub outputs: Vec<AsmOperand>,   // output constraints
    pub inputs: Vec<AsmOperand>,    // input constraints
    pub clobbers: Vec<String>,      // clobbered registers
    pub volatile: bool,             // volatile flag
    pub span: Span,
}
pub struct AsmOperand {
    pub constraint: String,         // e.g. "={rax}", "{rdi}"
    pub expr: Expr,                 // the XIOM expression bound to this operand
}
```

**Codegen:** Forward to LLVM's `call i64 asm "..."` or use LLVM's `InlineAsm` API
through the C API.

```llvm
; Generated LLVM IR for: var x = asm!("mov rax, 42" : "={rax}"(result))
%result = call i64 asm "mov rax, 42", "={rax}"
```

**Safety:** All inline assembly is implicitly `unsafe`. The compiler cannot verify
constraints or clobbers — that's the programmer's responsibility.

---

## 2. GENUINE GAP #2: LINK-TIME OPTIMIZATION (LTO)

### 2.1 Why It Matters

LTO enables cross-module optimization that is impossible with separate compilation:
- **Inlining across crate boundaries** (stdlib functions inlined into user code)
- **Dead code elimination** (unused stdlib functions stripped from binary)
- **Constant propagation** across module boundaries
- **Devirtualization** of interface method calls

Without LTO, every function call across a module boundary is an indirect call through
a function pointer. With LTO, the optimizer sees the whole program and can specialize.

### 2.2 Implementation

Add a `--lto` flag that passes `-flto=thin` to the linker:

```
xiom build --release --lto app.xi
```

**ThinLTO vs Full LTO:** Use LLVM's ThinLTO — it scales to large programs without the
memory explosion of full LTO. ThinLTO compiles each module separately with summary
data, then does a lightweight cross-module optimization pass at link time.

```rust
// In CompileConfig:
pub struct CompileConfig {
    pub lto: bool,                    // --lto flag
    pub lto_type: Option<String>,     // "thin" or "full" (default: "thin")
    // ...
}

// In codegen, when LTO is enabled:
// 1. Emit LLVM bitcode (.bc) instead of object files (.o)
// 2. Pass -flto=thin to the linker
// 3. The linker invokes LLVM's LTO plugin automatically
```

**Expected impact:**
- Binary size: 20-40% smaller (DCE across modules)
- Runtime: 5-15% faster (cross-module inlining)
- Compile time: +10-20% (ThinLTO is parallel)

---

## 3. GENUINE GAP #3: EMBEDDED DEBUG INFO EMISSION

### 3.1 Current State

XIOM has a DAP debug server that connects to GDB/CDB and provides breakpoints,
step-through, stack traces, and variable inspection. But it relies entirely on
DWARF/PDB emitted by the system linker — not XIOM-embedded metadata.

This means:
- No source-level mapping for XIOM code (LLVM sees generated IR, not .xi source)
- Variable names may not match XIOM source names
- Line numbers may point to IR file, not .xi file
- No XIOM-specific debug info (type information, generics, contracts)

### 3.2 Implementation

Use LLVM's `DIBuilder` API to emit DWARF debug info directly from the codegen:

```rust
// In xiom-codegen, after generating IR for each function:
fn emit_debug_info(&mut self, fn_decl: &FnDecl) {
    // 1. Create compilation unit (points to .xi source file)
    let cu = self.di_builder.create_compile_unit(
        language::XIOM,
        &fn_decl.source_file,
        &fn_decl.source_dir,
        "xiom v0.54",
        false,  // not optimized
        "",     // no flags
        0,      // runtime version
    );

    // 2. Create function debug info
    let fn_di = self.di_builder.create_function(
        &cu,
        &fn_decl.name.name,
        &fn_decl.mangled_name,
        &fn_decl.source_file,
        fn_decl.line_number,
        fn_type_di,
        false,  // not local to unit
        true,   // definition
        fn_decl.line_number,
    );

    // 3. For each statement, emit location info
    self.di_builder.set_current_location(
        line_number,
        column_number,
        &fn_di.scope,
    );

    // ... emit instructions with source location attached ...
}
```

This requires:
- Tracking source file paths, line numbers, and column numbers through the pipeline
- Emitting LLVM debug metadata alongside IR instructions
- Preserving XIOM type names and variable names in debug info

**Expected impact:**
- GDB/LLDB: `break main.xi:42` works directly
- `info locals` shows XIOM variable names
- `print x` works with XIOM types
- Stack traces show `.xi` file locations

---

## 4. SAFETY HARDENING — 5 Improvements

These are the items from the original plan that are NOT yet implemented and
genuinely matter for XIOM's "safest language" mission:

| # | Improvement | Status | Effort |
|---|------------|--------|--------|
| S1 | **Debug overflow + bounds + null checks** | MISSING | 3 days |
| S2 | **Match exhaustiveness checking** | MISSING | 2 days |
| S3 | **Never type (`!`)** | MISSING | 3 days |
| S4 | **`defer` statement** | MISSING | 2 days |
| S5 | **`?` operator propagation** | **EXISTS** | — |

### S1: Debug Safety Checks

Insert LLVM overflow intrinsics, bounds checks, and null checks in debug mode.
Release mode removes them. Already detailed in the original plan — no changes needed.

### S2: Match Exhaustiveness

The checker doesn't verify that `match` on `Result`, `Option`, or `Bool` covers
all cases. Add exhaustiveness analysis. Already detailed — no changes needed.

### S3: Never Type (`!`)

Functions that diverge (`exit`, `panic`, infinite loops) should have type `!`,
which coerces to any type. This makes exhaustiveness sound and eliminates
"missing return" false positives.

### S4: `defer` Statement

`defer { cleanup() }` executes the block when the enclosing scope exits.
Guarantees resource cleanup even on early return or `?`. Multiple defers
execute LIFO. Already detailed — no changes needed.

---

## 5. AI AGENT DESIGN — "Safest and Easiest with Less Tokens"

XIOM's target audience includes AI agents that generate code. This imposes
additional design constraints beyond human ergonomics:

### 5.1 Design Principles for AI-Generated Code

| Principle | Implementation |
|-----------|---------------|
| **Minimal syntax surface** | Fewer ways to do the same thing. No 3 ways to declare a variable. |
| **Predictable semantics** | No hidden control flow. No operator overloading surprises. |
| **Compile-time errors over runtime panics** | Catch mistakes at generation time, not execution time. |
| **Clear error messages** | AI agents parse compiler output. Point to exact token + suggest fix. |
| **Deterministic formatting** | `xiom fmt` produces canonical output. AI can learn the pattern. |
| **No implicit conversions** | `Int` to `Float64` must be explicit. Prevents subtle bugs. |
| **Fail-fast by default** | Out-of-bounds → panic. Null pointer → panic. The AI gets clear feedback. |

### 5.2 Token Efficiency

XIOM is already more token-efficient than Rust for common patterns:

```xiom
// XIOM: 12 tokens
fn double(x: Int) -> Int { return x * 2; }

// Rust: 14 tokens (pub + semicolon)
pub fn double(x: i64) -> i64 { x * 2 }
```

```xiom
// XIOM: 8 tokens
var items: Vec[Int] = [1, 2, 3];

// Rust: 12 tokens
let mut items: Vec<i64> = vec![1, 2, 3];
```

The `?` operator, `defer`, and match exhaustiveness further reduce token count
for safe code — the AI writes less code to achieve the same correctness guarantee.

### 5.3 AI-Specific Compiler Hints

```xiom
// When the AI is uncertain about a type:
var x = complex_expression() as Int;  // explicit type assertion

// When the AI wants the compiler to verify an invariant:
debug_assert!(x > 0);  // checked in debug mode, stripped in release

// When the AI wants to express "this can't fail":
var file = open_file("config.json").unwrap();  // panic on error — clear feedback
```

---

## 6. FINAL GAP ASSESSMENT

| # | Gap | Priority | v0.54 | v0.55 | v0.56 |
|---|-----|----------|-------|-------|-------|
| G1 | Inline Assembly | HIGH | Design | Implement | Test |
| G2 | LTO (ThinLTO) | MEDIUM | — | Implement | Test |
| G3 | Debug Info Emission | MEDIUM | — | — | Implement |
| S1 | Debug overflow/bounds/null | HIGH | Implement | Test | — |
| S2 | Match exhaustiveness | HIGH | Implement | Test | — |
| S3 | Never type (`!`) | MEDIUM | — | Implement | Test |
| S4 | `defer` statement | MEDIUM | — | Implement | Test |
| S5 | `?` operator | **DONE** | ✓ | — | — |

---

## 7. ROADMAP UPDATE

```
v0.54 ──► v0.55 ──► v0.56 ──► SELFHOST

v0.54:  CTFE Phase A + Binary Cache + Parallel Parse + Thread-Safe Registry
        + Debug overflow/bounds/null checks (S1)
        + Match exhaustiveness (S2)
        + Inline assembly design (G1)

v0.55:  OrcJIT MVP + Spawn codegen + Move semantics + Parallel Check
        + Never type (!) (S3)
        + defer statement (S4)
        + Inline assembly implementation (G1)

v0.56:  Send/Sync + Channel + Deadlock detection + Hot reload
        + LTO (G2)
        + Debug info emission (G3)
```

---

**Status:** APPROVED. All known gaps documented. 7 of 10 original concerns already
implemented. 3 genuine gaps + 4 safety improvements remain.
