# AXIOM

**A compiled, contract-first, AI-native systems language targeting WASM + Vulkan + LLVM**

---

## Vision

AXIOM is a systems programming language designed around three core beliefs that no existing language fully satisfies simultaneously:

| SAFE | Memory safety without a garbage collector. Ownership is explicit and verified at compile time. |
| VERIFIED | Contracts are first-class language constructs, not documentation. The compiler enforces them. |
| AI-NATIVE | Unambiguous grammar, dense semantics, and self-describing contracts minimize token cost for AI-generated code. |

AXIOM compiles via LLVM to native code, WASM, and GPU-adjacent targets via Vulkan bindings. It is self-hostable — the compiler is written in AXIOM once the language is stable enough to bootstrap.

## Design Principles

- **Unambiguous Grammar** — Every construct has exactly one canonical form. No style debates. The grammar is deterministic and parseable without context.
- **Explicit Over Implicit** — No hidden constructors, no operator overloading surprises, no implicit type coercions. What you read is exactly what runs.
- **Contracts As First-Class Citizens** — Pre-conditions, post-conditions, and invariants are part of the function signature. The compiler verifies them statically where possible.
- **No Nulls** — Absence is expressed via `Option[T]`. The compiler enforces handling of the absent case everywhere.
- **Errors Are Values** — No exceptions. Functions that can fail return `Result[T, E]`. Error handling is explicit at the call site.
- **Structural Typing** — Types are compatible by shape, not by declared hierarchy. No `implements` keyword needed.
- **Comptime Metaprogramming** — Code that runs at compile time flows through a single `comptime` mechanism. No preprocessor, no macros, no templates.

## Quick Syntax Preview

```axiom
// Immutable and mutable bindings
let x: Int = 42
var y: Float64 = 3.14

// Functions with contracts
fn divide(a: Float64, b: Float64) -> Float64
  requires: b != 0.0
  ensures:  result * b == a
{
  return a / b
}

// Algebraic types
enum AgentState {
  Idle
  Patrolling(route: Vec[Vector3])
  Attacking(target: EntityId)
  Dead(cause: DamageCause)
}

// Structural interfaces (no implements keyword)
interface Damageable {
  health: Int
  fn takeDamage(amount: Int) -> Self
    requires: amount >= 0
    ensures:  result.health <= self.health
}

// Error handling with Result
fn loadFile(path: Str) -> Result[File, IOError] {
  let f = open(path)?   // ? propagates error up
  return Ok(f)
}

// Ownership model
fn process(data: Vec[Int]) { }          // takes ownership
fn inspect(data: &Vec[Int]) { }         // read borrow
fn mutate(data: &mut Vec[Int]) { }      // write borrow
```

## Target Platforms

| Platform | Path |
|----------|------|
| **x86-64** (Windows, Linux, macOS) | AXIOM → LLVM → native |
| **ARM64** (Apple Silicon, iOS, Android) | AXIOM → LLVM → native |
| **RISC-V** (Embedded) | AXIOM → LLVM → native |
| **WASM** (Browsers, Node.js, Edge, WASI) | AXIOM → LLVM → WASM32 |
| **GPU** (Vulkan, Metal via MoltenVK, DX12 via VKD3D) | AXIOM → Vulkan bindings |
| **Console** (PS5, Xbox Series) | AXIOM → LLVM + platform SDK |

## Build Roadmap

### Phase 0 — Foundation (Months 1–6)
Compiler written in Rust. Targets a minimal AXIOM subset.
- Lexer, Parser → AST
- Basic type checker (primitives, structs, functions)
- LLVM IR output → Hello World on native and WASM

### Phase 1 — Core Language (Months 6–14)
Full language minus advanced contracts. Compiled by Phase 0.
- Full type system: generics, enums, interfaces
- Ownership model (borrow checker lite)
- Basic contracts (requires/ensures, runtime checks)
- Error handling (Result, ? operator)
- Async/await and channels
- C FFI layer
- Basic stdlib

### Phase 2 — Self-Hosting (Months 14–24)
Rewrite the compiler in AXIOM. Bootstrap.
- Rewrite lexer, parser, type checker in AXIOM
- Compiler compiles itself
- Retire Rust prototype
- Static contract verification via Z3 SMT solver

### Phase 3 — Ecosystem (Months 24–48)
Production ready.
- Vulkan GPU layer
- Full WASM + WASI support
- Platform SDKs: iOS, Android, consoles
- Language server protocol (LSP)
- Package registry

## Compiler Architecture

```
Source → Lexer → Parser → Semantic Analysis → Contract Verifier → IR Generation → Optimizer → LLVM Backend → Binary
```

- **Contract Verifier** — Attempts static proof of contracts using Z3 SMT solver; falls back to runtime checks where static verification is impossible.
- **AXIOM IR** — Explicitly typed intermediate representation. Human-readable. AI can target AXIOM IR directly for performance-critical code generation.

## Repository Structure

```
AXIOM/
├── specs/             # Language specification and design documents
├── docs/              # Developer documentation
├── src/               # Compiler source (Rust in Phase 0, AXIOM in Phase 2+)
├── tests/             # Language conformance test suite
├── stdlib/            # Standard library (axiom.core, axiom.io, etc.)
├── examples/          # Example AXIOM programs
└── tools/             # Build tooling, formatter, LSP
```

## Getting Started

> **Status: Phase 0 — Pre-prototype specification.** The language is currently in design. The compiler implementation has not yet begun.

To contribute or follow development:
1. Read the full [Language Specification](specs/AXIOM_Language_Spec.md)
2. Study [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom
3. Review the [LLVM Kaleidoscope tutorial](https://llvm.org/docs/tutorial/)
4. Explore the [Z3 theorem prover](https://github.com/Z3Prover/z3)

## AXIOM vs Today's Languages

| Feature | AXIOM | Closest Alternative |
|---------|-------|-------------------|
| Memory Safety | Ownership-lite, no GC | Rust (borrow checker, more complex) |
| Contracts | First-class, compiler-verified | Eiffel (not mainstream) |
| WASM Target | Primary target | Rust (secondary), Go (limited) |
| GPU Bindings | Vulkan stdlib | Rust via ash crate |
| AI Code Gen | Unambiguous grammar, dense semantics | No language designed for this |
| C Interop | Zero-cost FFI, first-class | Zig (closest) |
| Async | Built-in, unified model | Rust (complex), Go (goroutines) |
| Bootstrapped | Yes, in AXIOM itself | Rust, Go, Zig |
| Learning Curve | Moderate | Easier than Rust, stricter than Go |

## License

*To be determined*

---

**AXIOM — Language Specification v0.1** — This is a living specification. All syntax and APIs subject to revision.
