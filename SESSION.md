# AXIOM — Session Handoff: Phase 0 Complete → Phase 1 Ready

**Date:** 2026-06-30
**Branch:** `main`
**Version:** v0.1.0 "Pipeline"
**Status:** Phase 0 verified complete. 36/36 tests pass. Ready to begin Phase 1.

---

## What Phase 0 Delivered

A working AXIOM compiler pipeline in Rust (6 crates, ~3,800 lines):

```
.ax source → Lexer → Parser → Type Checker → LLVM IR → clang → native .exe
                                                         └→ clang → .wasm
```

### Crates

| Crate | Path | Purpose | Lines |
|-------|------|---------|-------|
| `axiom-ast` | `crates/axiom-ast/` | AST node definitions (full EBNF coverage) | 462 |
| `axiom-lexer` | `crates/axiom-lexer/` | Tokenizer (40+ token kinds) | 499 |
| `axiom-parser` | `crates/axiom-parser/` | Recursive descent LL(1) parser | ~1,400 |
| `axiom-check` | `crates/axiom-check/` | Basic type checker (primitives, structs, fns) | 680 |
| `axiom-codegen` | `crates/axiom-codegen/` | AST → text LLVM IR emitter | 650 |
| `axiomc` | `crates/axiomc/` | CLI binary (lex→parse→check→emit→compile) | 180 |

### Test Results
- **36/36 passing** across all crates
- 11 lexer, 15 parser, 10 checker

### Targets Verified
- **Native** (`add(10, 20)` = exit 30)
- **WASM** (723 bytes)
- **Float** (`fdiv double`, `fmul double` — correct LLVM IR)
- **IR output** (`--emit-ir`)

### What Works
- Functions, let/var, if/elif/else, while, match
- Struct types, field access, struct literals
- Integer + float arithmetic (correct LLVM types)
- Function calls (correct type signatures)
- Contract syntax, generics, derive, modules, interfaces — **parsed, not enforced**

---

## Design Decisions (Binding for Phase 1)

| Decision | Status | Details |
|----------|--------|---------|
| Ownership model | DECIDED | Lexical scope borrowing. No lifetimes. Implement borrow checker in Phase 1. |
| Contracts | DECIDED | Runtime guards only. `requires`/`ensures`/`invariant` emit panic-on-violation. Z3 is Phase 3. |
| Method receiver | DECIDED | `self` implicit. Fields accessed directly. Compiler infers `&Self` vs `&mut Self`. |
| Type constraints | DECIDED | Inline: `fn sort[T: Ord](...)`. NOT `requires: T satisfies Ord`. |
| Error type | DECIDED | `E` in `Result[T, E]` is any type. No forced `Error` interface. |
| `derive` codegen | DECIDED | Generate `Eq`, `Clone`, `Display`, `Hash`, `Ord` in Phase 1. |
| Derived fields | DEFERRED | Removed from grammar. Phase 2+. |
| Stdlib conformance | DECIDED | `test` package with contract-aware runner. Phase 1 addition. |

---

## Phase 1 — What Needs to Be Built

### 1. Ownership / Borrow Checker
- Single owner, `&T` read borrows (multiple), `&mut T` write borrows (exclusive)
- Scope lifetime — borrows expire at end of block/statement
- Move-on-call — passing values moves ownership
- Reject: use-after-move, double borrow, borrow-in-struct, borrow-return
- `clone()` tracking

### 2. Generics Codegen
- Monomorphisation: register + specialize per concrete instantiation
- Inline constraint checking: `T: Ord` verified at call sites

### 3. Contracts as Runtime Guards
- `requires:` check at fn entry → `@llvm.trap()` on violation
- `ensures:` check at fn exit
- `invariant:` check after every type mutation
- `self@pre` capture for ensures comparisons
- Contract collection methods: `is_sorted()`, `all()`, `none()`, `contains()`

### 4. `derive` Code Generation
- `Eq`: structural field comparison
- `Clone`: deep copy
- `Display`: canonical string format
- `Hash`: multiplicative hash
- `Ord`: lexicographic compare
- Restriction: types with invariants cannot derive `Eq`, `Hash`, `Ord`

### 5. Standard Library (7 modules)
- `core`, `io`, `collections`, `string`, `math`, `ffi`, `async`
- Stdlib conformance tests via `test` package

### 6. Error Handling — `Result`, `Option`, `?` operator

### 7. Module System — namespace resolution, `pub` enforcement

### 8. Async Runtime — `spawn`, channels (parsed/checked, partial codegen)

### 9. Tooling — `axiom fmt`, package manager prototype, LSP prototype

---

## Testing Strategy (Across All Phases)

| Phase | Tests |
|-------|-------|
| **Phase 1** | 36 → 100+ tests per new feature. Stdlib conformance. |
| **Phase 2A** | Differential correctness (Rust vs AXIOM compiler IR must be identical) |
| **Phase 2B** | Feature stress tests (async, floats, collections, deep generics, nested borrows) |
| **Phase 2C** | Compile-time benchmarks + regression suite |
| **Phase 3** | Ecosystem stress (AxiomDB compiles) |

See `specs/AXIOM_Build_Strategy.md` → Testing Strategy section.

---

## Ecosystem & Distribution Decisions

| Decision | Status |
|----------|--------|
| Single `main` branch, no per-OS forks | DECIDED |
| Three distribution paths: binaries, source, WASM playground | DECIDED |
| Showcase: AxiomDB first (KV → transactions), AxiomVDB second | DECIDED |
| `--diagnostics=json` + `--dump-contracts` | DECIDED — Phase 2 |
| Contract audit gates Phase 3 Z3 | DECIDED |
| Borrow-error tracking with ownership escape hatch | DECIDED |
| WASM compiler distribution | DECIDED — Phase 3 |
| Mechanical FFI binding generation | DECIDED — Phase 3 |

---

## Commands Quick Reference

```powershell
# Build everything
cargo build

# Run all tests (36 tests)
cargo test

# Compile an AXIOM program to IR
cargo run -p axiomc -- --emit-ir examples\demo_float.ax

# Compile and run natively
cargo run -p axiomc -- --run examples\demo_float.ax

# Compile to WASM
cargo run -p axiomc -- --target wasm -o demo.wasm examples\demo_float.ax

# Run specific test crate
cargo test -p axiom-check
cargo test -p axiom-parser
cargo test -p axiom-lexer
```

---

## Known Gotchas

1. **Rust 2024 edition** — No `ref` in match arms. Bind by reference automatically.
2. **Clang required** — Searches `C:\Program Files\LLVM\bin\clang.exe` and PATH.
3. **Float type tracking** — `locals` stores `(alloca_name, llvm_type)`. Always call `add_local(name, alloca, llvm_ty)` with correct LLVM type.
4. **Struct-literal heuristic** — Parser peeks ahead one token after `{` to decide struct-literal vs block. Fragile. Phase 1 should improve.
5. **`expect` method dead code** — unused parser method. Harmless warning.

---

## How to Continue (Fresh Session → Phase 1)

1. `cd E:\Projects\AXIOM`
2. `cargo test` — confirm 36/36
3. Read `specs/AXIOM_Build_Strategy.md` — Phase 1 section + Testing Strategy
4. Start with the ownership checker in `crates/axiom-check/` — add `BorrowChecker` struct
5. Add borrow tracking to `CheckedType` (Ref/MutRef variants with scope info)
6. Then generics codegen in `crates/axiom-codegen/`
7. Then contracts as runtime guards
8. Then `derive` codegen
9. Then stdlib + error handling + modules + async

---

**Phase 1 Entry Condition Met:** Phase 0 compiles and runs real code on native and WASM. All 36 tests pass. The pipeline architecture is validated.

**Next action:** Implement lexical scope borrow checker in `crates/axiom-check/`.
