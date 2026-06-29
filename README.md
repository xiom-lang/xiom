# AXIOM

**A compiled, contract-first systems language targeting LLVM + WASM**

---

## Vision

AXIOM is a systems programming language designed around three properties no existing language provides simultaneously:

| SAFE | Memory safety without a garbage collector. Ownership with lexical scope borrowing — no lifetime annotations. |
| VERIFIED | Contracts are first-class language constructs enforced by the compiler, not documentation conventions. |
| PRECISE | One way to write each thing. No implicit coercions, no hidden allocations, no surprising control flow. |

AXIOM compiles to native code via LLVM and to WebAssembly as a co-equal target. The language is designed for a world where code is increasingly written by AI and maintained by people who did not write the original. Contracts are machine-readable intent — when AI generates code, the compiler verifies it against stated expectations before it ever runs.

## Design Principles

- **One Way** — Every construct has exactly one canonical form. `axiom fmt` enforces it.
- **Explicit Over Implicit** — No hidden constructors, no implicit type coercions, no silent allocations. What you read is what runs.
- **Contracts As Specification** — `requires`, `ensures`, and `invariant` are part of the function signature. The compiler enforces them at minimum via runtime guards with precise error messages.
- **No Null** — Absence is `Option[T]`. Exhaustive handling enforced at every use site.
- **Errors Are Values** — No exceptions. `Result[T, E]` with `?` propagation. Unhandled error paths are compile errors.
- **Structural Typing** — Types satisfy interfaces by shape. No `implements` keyword.
- **Derive** — Compiler generates `Eq`, `Clone`, `Display`, `Hash`, `Ord` implementations. Correct by construction, zero boilerplate.
- **Comptime** — A single mechanism for generics, reflection, and compile-time code generation. No macros, no templates.

## Quick Syntax Preview

```axiom
// Immutable and mutable bindings
let x: Int = 42
var y: Float64 = 3.14

// Types with derive — compiler generates Eq, Clone, Display
type Point = {
  x: Float64;
  y: Float64;
} derive[Eq, Clone, Display]

// Functions with contracts
fn divide(a: Float64, b: Float64) -> Float64
  requires: b != 0.0
  ensures:  result * b == a
{
  return a / b
}

// Generics with inline type constraints
fn max[T: Comparable](a: T, b: T) -> T {
  if a > b { return a }
  return b
}

// Algebraic types
enum AgentState {
  Idle
  Patrolling(route: Vec[Vector3])
  Attacking(target: EntityId)
  Dead(cause: DamageCause)
}

// Structural interfaces — no implements keyword
interface Damageable {
  health: Int
  fn takeDamage(amount: Int) -> Self
    requires: amount >= 0
    ensures:  result.health <= self.health
}

// Methods use implicit self
fn Vec3.dot(other: &Vec3) -> Float32 {
  return x * other.x + y * other.y + z * other.z
}

// Error handling with Result + ?
fn loadFile(path: Str) -> Result[File, IOError] {
  let f = open(path)?
  return Ok(f)
}
```

## Compiler Architecture

```
Source → Lexer → Parser → Name Resolution → Type Checker → Contract Verifier → AXIOM IR → Optimizer → LLVM Backend → Binary
```

- **LL(1) Grammar** — Single deterministic parse path. No backtracking.
- **AXIOM IR** — Explicitly typed, SSA-form intermediate representation. Human-readable. Platform-independent.
- **Contract Verifier** — Phase 1: runtime guards with precise error messages. Phase 3: Z3 SMT solver for static proof.
- **Dual Target** — Native machine code and WebAssembly from the same IR, available from Phase 0.

## Build Roadmap

### Phase 0 — Prototype Compiler (2–3 weeks)
**Status: starting** — Hello World on native + WASM.

- Lexer + Parser → AST (full grammar, including `derive`, inline constraints, method syntax)
- Basic type checker (primitives, structs, enums, functions)
- LLVM IR emission (arithmetic, control flow, function calls)
- Test suite: 200+ parser tests, 100+ type checker tests
- **No generics. No ownership. No contracts. Pipeline first.**

### Phase 1 — Full Language (3–6 months)
- Ownership model: lexical scope borrowing
- Generics via comptime with inline constraints
- Contracts as runtime guards
- `derive` code generation
- Standard library: core, io, collections, string, math, ffi
- Package manager, formatter (`axiom fmt`), LSP prototype

### Phase 2 — Self-Hosting (6–12 months)
- Rewrite the compiler in AXIOM itself
- Compiler compiles itself — bootstrap complete

### Phase 3 — Ecosystem (ongoing)
- Z3 SMT integration for static contract verification
- Package registry, documentation generator, additional targets

## AXIOM vs Existing Languages

| Feature | AXIOM | Rust | Go | Zig |
|---------|-------|------|----|-----|
| Memory model | Ownership, lexical borrows | Full borrow checker + lifetimes | GC | Manual |
| Contracts | First-class, compiler-enforced | Assertions only | None | None |
| Null safety | `Option[T]` | `Option<T>` | Null exists | Null exists |
| Error handling | `Result[T, E]` + `?` | `Result<T, E>` + `?` | Multi-return | Error unions |
| Generics | Comptime, inline constraints | Trait bounds | Limited | comptime |
| Interfaces | Structural | Nominal | Structural | — |
| Derive | Compiler-generated `Eq`, `Clone`, `Display`, `Hash`, `Ord` | `#[derive(...)]` | None (manual) | None |
| WASM target | First-class, co-equal | Supported | Limited | Supported |
| C FFI | Zero-cost | Zero-cost | Cgo (overhead) | First-class |
| Method receiver | Implicit `self`, inferred | Explicit `&self` | Explicit | Explicit |
| Grammar | No ambiguity, canonical form | Some | Minimal | Minimal |

## Repository Structure

```
AXIOM/
├── specs/             # Language specification and design decisions
├── src/               # Compiler source (Rust in Phase 0, AXIOM in Phase 2+)
├── tests/             # Language conformance test suite
├── stdlib/            # Standard library (axiom.core, axiom.io, etc.)
├── examples/          # Example AXIOM programs
└── docs/              # Developer documentation
```

## Getting Started

> **Status: Phase 0 — Language defined. Compiler implementation beginning.**

1. Read the [Language Specification](specs/AXIOM_Language_Spec.md) — normative definition
2. Read the [Build Strategy](specs/AXIOM_Build_Strategy.md) — decisions and timeline
3. Read the [Purpose Document](specs/AXIOM_Purpose.md) — why this language exists

---

**AXIOM — Language Specification v0.3**
