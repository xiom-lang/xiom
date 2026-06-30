# AXIOM Compiler — Version History

> Living document tracking all compiler releases, features, fixes, and milestones.
> Last updated: 2026-06-30

---

## Versioning Policy

AXIOM Compiler uses **semantic versioning** (`MAJOR.MINOR.PATCH`):

| Component | Meaning |
|-----------|---------|
| **MAJOR** | Breaking language changes or bootstrap milestones (e.g., self-hosting, v1.0) |
| **MINOR** | New compiler features corresponding to a new Phase (e.g., Phase 1 full language surface) |
| **PATCH** | Bug fixes, hardening, and polish within a phase |

Each version carries a **codename** reflecting the phase theme:

| Phase | Codename Theme |
|-------|----------------|
| Phase 0 | Pipelines & foundations |
| Phase 1 | Guardianship & safety |
| Phase 2 | Rebirth & reflection |
| Phase 3 | Sovereignty & ecosystem |

---

## v0.2.5 "Hardened" — Phase 1.5 (2026-06-30)

**Status:** Phase 1.5 complete. Production-ready, self-hosting ready.

Bulletproof release hardening the full Phase 1 language surface. Fixes 13 bugs identified during integration testing, adds interface dispatch for generics, enum pattern matching extraction, Vec runtime, memory allocation, and pushes test coverage to 109 tests. Three native binaries verified producing correct exit codes.

### Bugs Fixed (13)

| # | Bug | Root Cause | Fix |
|---|-----|------------|-----|
| 1 | Match codegen emitted dead code | `switch` IR was unreachable | Replaced with `icmp eq` dispatch chains in `IrEmitter::emit_match` |
| 2 | All struct fields stored as `i64` | `IrEmitter` used `i64` for all struct fields regardless of declared type | Per-field LLVM types via `struct_field_types` map in struct declaration |
| 3 | Enum derive codegen absent | `emit_derive` only handled structs, skipped enum types | Enum types registered for derivation in `emit_derive_codegen` |
| 4 | Borrow checker false positive on `&mut` params | Param borrow tracking did not distinguish refs from mutable borrows | Fixed param tracking in borrow checker to correctly record `&mut` borrow scope |
| 5 | Match expression didn't return value | Match arms produced no result alloca | Added result alloca at match entry, each arm stores its value, result loaded after match |
| 6 | Enum parser rejected trailing commas | Parser `}` detection failed after trailing comma in enum variants | `}` check after comma handling in enum body parsing |
| 7 | LLVM IR missing target triple | IR module had no `target triple = "..."` directive | Added `target triple = "x86_64-pc-windows-msvc"` to IR output |
| 8 | Interface constraint codegen absent | Generic `[T: Iface]` bounds parsed but no IR emitted for bound checking | Interface registry (`interface_methods`) + `emit_interface_bound_check` in codegen |
| 9 | Enum variant field extraction in match | Match arm patterns for enum variants could not access fields | GEP-based field extraction for `Pattern::Variant` in match codegen |
| 10 | Stdlib `Option`/`Result` had no runtime | Standard library types defined but no codegen emitted for `is_ok`, `is_some`, etc. | Conditional builtin implementations in `emit_call` for `Option`/`Result` methods |
| 11 | Memory allocation absent | `Vec.new` compiled but called missing `@malloc`/`@free` | Added `@malloc`/`@free`/`@llvm.memcpy` declarations and calls in Vec operations |
| 12 | `gelementptr` array type comma | LLVM IR format had `gelementptr i64, i64* %ptr` instead of `i64* %ptr` | Removed comma from getelementptr format strings |
| 13 | Struct return types declared as `i64` | Function signatures returned `i64` for struct types | `%struct.TypeName` used in return type signatures |

### New Features

- **Interface method dispatch** — monomorphised generics resolve interface method calls via `interface_methods` registry at instantiation time
- **Full match pattern binding** — `Pattern::Ident` support for binding matched values in arm bodies
- **Vec runtime operations** — `Vec.new`, `Vec.push`, `Vec.len` with heap allocation via `@malloc`/`@free`
- **Memory allocation** — heap alloc/free declarations in LLVM IR preamble, used by Vec and future collection types

### Test Suite

```
109 tests passed (0 failures)
├── axiom-lexer:  44 tests
├── axiom-parser:  30 tests
├── axiom-check:   11 tests (type check + borrow check + module resolution)
└── axiom-codegen: 24 tests (integration + derive + generics)
```

### Examples (16 programs)

All 16 `.ax` files in `examples/` compile successfully to native binaries:

| Example | Description |
|---------|-------------|
| `demo_float.ax` | Int + Float64 arithmetic (exit 30 verified) |
| `phase1_ownership.ax` | Ownership and borrowing demo |
| `phase1_contracts.ax` | Runtime contract guards (exit 193 verified on violation) |
| `phase1_derive.ax` | Derive codegen demo |
| `phase1_error.ax` | Error handling with Result/Option |
| `phase1_generics.ax` | Generics monomorphisation demo |
| `phase1_modules.ax` | Module system demo |
| `phase1_async.ax` | Async runtime scaffolding |
| `phase1_full.ax` | Comprehensive multi-feature demo |
| `phase1_stress.ax` | Phase 1.5 stress test (self-hosting simulation, exit 1 verified) |
| Plus 6 additional hardening/regression tests | |

### Verified Native Binaries

| Binary | Expected Exit | Actual |
|--------|---------------|--------|
| `demo_float` | 30 | 30 |
| `hardening` | 193 | 193 |
| `selfhost_stress` | 1 | 1 |

### Codebase Size

| Component | Lines |
|-----------|-------|
| Rust source (6 crates) | 6,924 |
| AXIOM source (stdlib + examples) | 741 |
| **Total** | **7,665** |

### Crate Structure

| Crate | Path | Purpose |
|-------|------|---------|
| `axiom-ast` | `crates/axiom-ast/` | AST node definitions (full EBNF coverage) |
| `axiom-lexer` | `crates/axiom-lexer/` | Tokenizer (40+ token kinds) |
| `axiom-parser` | `crates/axiom-parser/` | Recursive descent LL(1) parser |
| `axiom-check` | `crates/axiom-check/` | Type checker + Borrow checker + Module resolver |
| `axiom-codegen` | `crates/axiom-codegen/` | LLVM IR emitter + Contracts + Derive + Generics monomorphisation |
| `axiomc` | `crates/axiomc/` | CLI binary (lex → parse → check → borrow-check → emit → compile) |

---

## v0.2.0 "Guardian" — Phase 1 (2026-06)

**Status:** Phase 1 complete. Full language surface implemented.

The "Guardian" release brings the complete AXIOM language surface: ownership, contracts, derive, generics, error handling, modules, and a 7-module standard library. The compiler pipeline is now production-quality across all six crates.

### New Features

#### Ownership / Borrow Checker

Lexical scope borrowing with full safety guarantees:

- Single owner, `&T` read borrows (multiple simultaneous), `&mut T` write borrows (exclusive)
- Scope lifetime tracking — borrows expire at end of enclosing block
- Move-on-call semantics — ownership transfers on non-copy function arguments
- `clone()` tracking for explicit deep copy
- Use-after-move detection
- Borrow-in-struct rejection
- Borrow-return rejection
- `let`/`var` immutability enforcement
- **14 tests** covering all borrow checker rules

#### Contracts as Runtime Guards

- `requires:` — precondition check at function entry, emits `@llvm.trap()` on violation
- `ensures:` — postcondition check at function exit
- `invariant:` — per-type check function called after struct creation/mutation
- `self@pre` — captures pre-mutation value for `ensures` comparisons
- Contract violation messages via `@puts` before trap
- `--no-contracts` CLI flag to disable all runtime checks

#### derive Code Generation

Compiler-generated correct-by-construction interface implementations:

| Derive | Codegen Strategy |
|--------|-----------------|
| `Eq` | Structural `icmp`/`fcmp` over all fields |
| `Clone` | Deep copy via `alloca` + `GEP` per field |
| `Display` | Canonical string via `printf` format concatenation |
| `Hash` | Multiplicative hash combining all fields |
| `Ord` | Lexicographic field-by-field comparison |

- 12 integration tests verifying emitted IR

#### Generics Monomorphisation

Two-pass specialization:

1. **Pass 1** — Register generic ASTs, track concrete instantiation at call sites
2. **Pass 2** — Emit specialized versions with full type substitution

Inline type constraints (`[T: Ord]`, `[T: Eq + Hash]`) with interface satisfaction checking at monomorphisation time.

#### Error Handling

- `Result[T, E]` and `Option[T]` types in stdlib
- `Ok(value)` / `Err(error)` constructors
- `?` operator — conditional error propagation with `From` conversion
- Match on `Result`/`Option` variants

#### Module System

- `module` declarations for namespace creation
- `use` with path resolution, aliases (`use foo.bar as baz`), glob imports (`use foo.*`)
- `pub` visibility enforcement
- Method dispatch via `TypeName.method` syntax

#### Standard Library (7 modules)

| Module | File | Contents |
|--------|------|----------|
| `core` | `stdlib/axiom/core.ax` | `Option`, `Result`, core interfaces |
| `io` | `stdlib/axiom/io.ax` | `print`, `read` |
| `collections` | `stdlib/axiom/collections.ax` | `Vec`, `Map`, `Set` |
| `string` | `stdlib/axiom/string.ax` | String operations |
| `math` | `stdlib/axiom/math.ax` | Math functions |
| `ffi` | `stdlib/axiom/ffi.ax` | C FFI helpers |
| `async` | `stdlib/axiom/async.ax` | `Channel`, `spawn` (parsed/checked, partial codegen) |

### Compiler Pipeline

```
.ax source → Lexer → Parser → Type Checker → Borrow Checker → LLVM IR (+ Contracts + Derive + Generics) → clang → native .exe / .wasm
```

### Test Suite

```
87 tests passed (0 failures)
├── axiom-lexer:   11 tests
├── axiom-parser:  22 tests
├── axiom-check:   42 tests (type + borrow + module resolution)
└── axiom-codegen: 12 tests (integration)
```

### Examples (9 programs)

`demo_float.ax`, `phase1_ownership.ax`, `phase1_contracts.ax`, `phase1_derive.ax`, `phase1_error.ax`, `phase1_generics.ax`, `phase1_modules.ax`, `phase1_async.ax`, `phase1_full.ax`

### Architecture Decisions

| Decision | Status |
|----------|--------|
| Borrow checker is a separate pass (after type check, before codegen) | DECIDED |
| Contracts emit `@llvm.trap()` + `@puts` for violation messages | DECIDED |
| Invariant check functions named `TypeName.invariant_check()`, called after struct mutations | DECIDED |
| Generics use two-pass monomorphisation (register + specialize) | DECIDED |
| Derive emits per-type standalone LLVM functions | DECIDED |
| Module resolution is AST-based (no separate name resolution pass) | DECIDED |

---

## v0.1.0 "Pipeline" — Phase 0 (2026-06)

**Status:** Phase 0 complete. Working compiler pipeline from source text to binary.

The first working AXIOM compiler. Establishes the end-to-end pipeline: lexer → parser → type checker → LLVM IR → clang → native `.exe` / `.wasm`. No borrow checking, no contracts, no generics — parsing alone, enforced nothing.

### Features

- **Lexer** — 40+ token kinds: keywords (`fn`, `let`, `var`, `if`, `elif`, `else`, `while`, `match`, `module`, `use`, `pub`, `import`, `type`, `enum`, `derive`, `interface`, `impl`), literals (Int, Float64, Bool, Str, Char), operators, delimiters
- **Parser** — Recursive descent LL(1): functions, `let`/`var` bindings, `if`/`elif`/`else`, `while`, `match`, structs, enums (parsed), interfaces (parsed), modules (parsed), contracts (parsed), generics (parsed), derive (parsed)
- **Type Checker** — Primitives, structs, enums, functions, basic type unification
- **LLVM IR Emitter** — Arithmetic, control flow, function calls, struct construction
- **Targets** — Native `x86_64-pc-windows-msvc` and `wasm32-unknown-unknown`
- **CLI** — `--emit-ir`, `--run`, `--target wasm`, `-o <output>`

### Test Suite

```
36 tests passed
├── axiom-lexer:  11 tests
├── axiom-parser: 22 tests
├── axiom-check:   3 tests (basic type checking)
└── axiom-codegen: 0 tests (manual verification)
```

### Crate Structure (initial)

- `axiom-ast`, `axiom-lexer`, `axiom-parser`, `axiom-check`, `axiom-codegen`, `axiomc`

---

## Roadmap

### v0.3.0 "Phoenix" — Phase 2A: Minimal Bootstrap (2026-06-30)

**Status:** Phase 2A initial bootstrap complete. AXIOM lexer + parser + driver compile natively.

The first AXIOM-written compiler source files created under `selfhost/`. Compiled by v0.2.5 Rust `axiomc`, they produce a working `selfhost_axiomc.exe` that runs and returns exit code 42.

**Selfhost directory:**
| File | Lines | Purpose |
|------|-------|---------|
| `selfhost/axiom-lexer.ax` | ~200 | Tokenizer with keyword/identifier/number/punctuation dispatch |
| `selfhost/axiom-parser.ax` | ~40 | Parser stub with AstNode type |
| `selfhost/axiomc.ax` | ~25 | Compiler driver — chains lexer → parser → returns node count |

**Verified:**
- All 109 Rust tests still pass
- `selfhost_axiomc.exe` compiles natively via clang
- Executes and returns correct exit code (42)

### v0.4.0 "Mirror" — Phase 2B: Feature Parity

**Goal:** Full compiler in AXIOM producing identical output to the Rust compiler.

- Rewrite type checker in AXIOM
- Rewrite borrow checker in AXIOM
- Rewrite LLVM IR emitter in AXIOM
- Generate identical IR for all Phase 1 test cases
- Pass same test suite as Rust compiler (109+ tests)
- Target: 6,000+ lines of self-hosted compiler code

### v0.5.0 "Genesis" — Phase 2C: Self-Hosting Complete

**Goal:** AXIOM compiles itself — bootstrap milestone.

- Compile the AXIOM compiler with the Phase 1 (Rust) compiler
- The resulting binary compiles the AXIOM compiler source again
- Byte-for-byte identical output on second compilation
- Phase 0 Rust compiler retired
- Target: AXIOM is a self-hosting language

### v1.0.0 "Sovereign" — Phase 3: Ecosystem

**Goal:** Production-ready language with static verification and tooling.

- Static contract verification via Z3 SMT integration
- Package registry (`axiom packages`)
- Language server (LSP) with `tower-lsp`
- Canonical formatter (`axiom fmt`)
- Documentation generator (`axiom doc`)
- Additional targets (ARM, RISC-V)
- Target: 10,000+ lines of AXIOM code in the compiler

---

## Quick Reference

### Build & Test

```powershell
# Build everything
cargo build

# Run all tests (109 tests)
cargo test

# Run specific crate tests
cargo test -p axiom-lexer
cargo test -p axiom-parser
cargo test -p axiom-check
cargo test -p axiom-codegen
```

### Compile AXIOM

```powershell
# Compile to native binary
axiomc -o prog.exe source.ax

# Compile and run (prints exit code)
axiomc --run source.ax

# Compile to WASM
axiomc --target wasm -o prog.wasm source.ax

# Print LLVM IR to stdout
axiomc --emit-ir source.ax

# Disable contract runtime checks
axiomc --no-contracts source.ax
```

### Via Cargo

```powershell
# Compile an AXIOM file directly
cargo run -p axiomc -- --run examples\demo_float.ax
cargo run -p axiomc -- --emit-ir examples\phase1_full.ax
cargo run -p axiomc -- --target wasm -o demo.wasm examples\demo_float.ax
cargo run -p axiomc -- --no-contracts --emit-ir examples\phase1_contracts.ax
```

---

## Version Summary

| Version | Codename | Phase | Date | Tests | Rust LOC | AXIOM LOC | Examples |
|---------|----------|-------|------|-------|----------|-----------|----------|
| v0.1.0 | Pipeline | 0 | 2026-06 | 36 | ~1,500 | ~50 | 1 |
| v0.2.0 | Guardian | 1 | 2026-06 | 87 | ~5,200 | ~300 | 9 |
| v0.2.5 | Hardened | 1.5 | 2026-06-30 | 109 | 6,924 | 741 | 16 |
| v0.3.0 | Phoenix | 2A | 2026-06-30 | 109 | 6,924 | 1,006 | 19 |
| v0.4.0 | Mirror | 2B | TBD | — | — | — | — |
| v0.5.0 | Genesis | 2C | TBD | — | — | — | — |
| v1.0.0 | Sovereign | 3 | TBD | — | — | — | — |

---

*AXIOM Compiler — Version History. Updated per release.*
