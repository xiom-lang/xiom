# AXIOM Compiler — Version History & Roadmap

> Living document tracking all compiler releases and planned milestones.
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

## Current Release

---

## v0.1.0 "Pipeline" — Phase 0 (2026-06-30)

**Status: Released.** Working compiler pipeline from source text to binary.

The first working AXIOM compiler. Establishes the end-to-end pipeline: lexer → parser → type checker → LLVM IR → clang → native `.exe` / `.wasm`. No borrow checking, no contracts, no generics — parses everything, enforces nothing.

### Features

- **Lexer** — 40+ token kinds: keywords (`fn`, `let`, `var`, `if`, `elif`, `else`, `while`, `match`, `module`, `use`, `pub`, `type`, `enum`, `derive`, `interface`), literals (Int, Float64, Bool, Str, Char), operators, delimiters
- **Parser** — Recursive descent LL(1): functions, `let`/`var` bindings, `if`/`elif`/`else`, `while`, `match`, structs, enums (parsed), interfaces (parsed), modules (parsed), contracts `requires`/`ensures`/`invariant` (parsed), generics `[T: Comparable]` (parsed), `derive[Eq, Clone, ...]` (parsed), methods `fn Type.method()` (parsed)
- **Type Checker** — Primitives, structs, enums, functions, basic type unification
- **LLVM IR Emitter** — Arithmetic (int + float), control flow (`if`/`elif`/`else`, `while`, `match`), function calls with correct type signatures, struct construction
- **Targets** — Native `x86_64-pc-windows-msvc` and `wasm32-unknown-unknown` (both verified)
- **CLI** — `--emit-ir`, `--run`, `--target wasm`, `-o <output>`, `--run`

### Verified Targets

| Target | Exit Code | Status |
|--------|-----------|--------|
| Native (`add(10, 20)` via clang) | 30 | Verified |
| WASM (`final.wasm`) | 723 bytes | Verified |
| Native (`sq(3.0)` + `add(10, 20)`) | 30 | Verified |

### Test Suite

```
36 tests passed (0 failures)
├── axiom-lexer:   11 tests
├── axiom-parser:  15 tests
├── axiom-check:   10 tests
└── axiom-codegen:  0 tests (manual IR verification)
```

### Codebase Size

| Component | Lines |
|-----------|-------|
| Rust source (6 crates) | ~3,800 |
| AXIOM source (example) | 1 file |
| Spec docs | 3 files |

### Crate Structure

| Crate | Path | Purpose |
|-------|------|---------|
| `axiom-ast` | `crates/axiom-ast/` | AST node definitions (full EBNF coverage) |
| `axiom-lexer` | `crates/axiom-lexer/` | Tokenizer (40+ token kinds) |
| `axiom-parser` | `crates/axiom-parser/` | Recursive descent LL(1) parser |
| `axiom-check` | `crates/axiom-check/` | Basic type checker (primitives, structs, functions) |
| `axiom-codegen` | `crates/axiom-codegen/` | Text LLVM IR emitter |
| `axiomc` | `crates/axiomc/` | CLI binary (lex → parse → check → emit → compile) |

### Architecture Decisions

| Decision | Status |
|----------|--------|
| Phase 0 compiler language | **Rust** |
| LLVM backend | **Text IR emission** (no LLVM library dependency) |
| WASM target | **LLVM wasm32-unknown-unknown** via clang |
| Ownership model | **DECIDED** — lexical scope borrowing (implementation deferred to Phase 1) |
| Contracts | **DECIDED** — runtime guards in Phase 1, Z3 in Phase 3 |
| Type constraints | **DECIDED** — inline `[T: Ord]`, not `requires: T satisfies Ord` |
| Method receiver | **DECIDED** — `self` implicit, inferred from body |
| `derive` codegen | **DECIDED** — Phase 1 (parsed, stored in AST, no codegen yet) |
| Derived fields | **DEFERRED** — removed from grammar, planned Phase 2+ |
| Timeline | **DECIDED** — AI-assisted 12-24 months to self-hosting |

---

## Roadmap (Planned — Not Yet Built)

Versions below are **planned**. Feature lists, test counts, and dates are targets — not commitments. They will be filled in as actual releases occur.

---

### v0.2.0 "Guardian" — Phase 1 (Target: 2026)

**Goal:** Full language surface. Ownership, contracts, derive, generics, error handling, modules, stdlib.

| Feature | Scope |
|---------|-------|
| Ownership / borrow checker | Lexical scope: `&T`, `&mut T`, move semantics, use-after-move detection |
| Contracts as runtime guards | `requires`/`ensures`/`invariant` → runtime checks with `@llvm.trap()` |
| `derive` code generation | `Eq`, `Clone`, `Display`, `Hash`, `Ord` — compiler-generated |
| Generics monomorphisation | Two-pass: register + specialize with inline constraint checking |
| Error handling | `Result[T, E]`, `Option[T]`, `?` operator |
| Module system | `module`/`use`/`pub` resolution |
| Standard library | `core`, `io`, `collections`, `string`, `math`, `ffi`, `async` (7 modules) |
| Stdlib conformance testing | `test` package with contract-aware runner |

**Architecture decisions for Phase 1:**

| Decision | Status |
|----------|--------|
| Borrow checker pass (after type check, before codegen) | DECIDED |
| Contracts emit `@llvm.trap()` + message | DECIDED |
| Generics: two-pass monomorphisation | DECIDED |
| Derive: per-type standalone LLVM functions | DECIDED |
| Module resolution: AST-based | DECIDED |
| Stdlib conformance: `test` package | DECIDED |

---

### v0.3.0 "Phoenix" — Phase 2 (Target: 2026–2027)

**Goal:** Self-hosting. Rewrite the AXIOM compiler in AXIOM. Bootstrap.

| Milestone | Scope |
|-----------|-------|
| **Phase 2A** | Rewrite lexer + parser in AXIOM. Compiled by Phase 1. |
| **Phase 2B** | Rewrite type checker + borrow checker + IR emitter in AXIOM. Feature parity with Rust compiler — generate identical IR. |
| **Phase 2C** | Compiler compiles itself. Byte-for-byte identical output. Phase 0 Rust compiler retired as primary but kept as permanent bootstrap fallback. |

**Phase 2 tooling (built during self-hosting):**

| Item | Scope | Why Phase 2 |
|------|-------|-------------|
| `--diagnostics=json` | Structured compiler output for AI tooling | Data already exists — new output format. 2-day addition. |
| `--dump-contracts` | Queryable contract index across a package | AST traversal + output flag. AI tooling needs this during self-hosting. |
| Contract semantics audit | Track contract quality in Phase 2 corpus | Gate on Phase 3 Z3 work. Audit before building SMT integration. |
| Borrow-error AI friction tracking | Measure agent failure rate on borrow errors | Data collection only. Informs ownership model revision decision. |

---

### v1.0.0 "Sovereign" — Phase 3 (Target: 2027+)

**Goal:** Production-ready language with static verification and ecosystem.

- Static contract verification via Z3 SMT integration
- Package registry (`axiom packages`)
- Language server (LSP) with `tower-lsp`
- Canonical formatter (`axiom fmt`)
- Documentation generator (`axiom doc`)
- Additional targets (ARM, RISC-V)
- WASM compiler distribution (playground)
- Mechanical FFI binding generation with contract inference
- Showcase projects: AxiomDB (KV store → transactions), AxiomVDB (vector store)

---

## Quick Reference

### Build & Test

```powershell
# Build everything
cargo build

# Run all tests (36 tests)
cargo test

# Run specific crate tests
cargo test -p axiom-lexer
cargo test -p axiom-parser
cargo test -p axiom-check
```

### Compile AXIOM

```powershell
# Compile and run (prints exit code)
cargo run -p axiomc -- --run examples\demo_float.ax

# Compile to WASM
cargo run -p axiomc -- --target wasm -o demo.wasm examples\demo_float.ax

# Print LLVM IR to stdout
cargo run -p axiomc -- --emit-ir examples\demo_float.ax
```

---

## Version Summary

| Version | Codename | Phase | Date | Status | Tests | Rust LOC | AXIOM LOC |
|---------|----------|-------|------|--------|-------|----------|-----------|
| **v0.1.0** | Pipeline | 0 | 2026-06-30 | **Released** | 36 | ~3,800 | 0 |
| v0.2.0 | Guardian | 1 | TBD | Planned | — | — | — |
| v0.3.0 | Phoenix | 2 | TBD | Planned | — | — | — |
| v1.0.0 | Sovereign | 3 | TBD | Planned | — | — | — |

---

*AXIOM Compiler — Version History. Updated per release.*
