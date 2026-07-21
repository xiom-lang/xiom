# XIOM Compiler Architecture

**Version:** v0.49.8 "Phase 5a/5b Complete"
**Date:** 2026-07-14
**Branch:** `feat/architect` (Phase 5c)
**Status:** Living document — updated as the compiler evolves

This document is the authoritative reference for understanding how the XIOM Rust compiler is structured, its data flow, current limitations, and the roadmap to self-hosting. Every section is grounded in the three pillars defined in `specs/XIOM_Purpose.md`: **SAFE**, **VERIFIED**, **PRECISE**.

---

## 1. Overview & Three Pillars

The XIOM compiler (`xiom`) is a multi-stage, single-pass compiler written in Rust. It takes `.xi` source files, produces LLVM IR text, and shells out to `clang` for final native or WASM binary emission. The compiler has completed Phases 0-5b and is now in Phase 5c (Architectural Features + Safety Hardening).

### SAFE — Memory safety without runtime overhead
- **Borrow checker** enforces lexical-scope ownership at compile time. References tracked as read/write borrows that must not overlap with mutations.
- **No null types.** All optional values use `Option[T]` discriminated unions with discriminant checks.
- **Move semantics** checked at AST level — variables marked consumed after by-value pass.
- **`&mut self` receiver support**: struct methods receive `self` by pointer, enabling in-place mutation (v0.49.8).
- **Unsafe code** explicitly marked (`unsafe { ... }` blocks). Contracts validate pre/post conditions around unsafe regions.

### VERIFIED — Contracts as compiler-enforced specification
- **Contract system** supports `requires`, `ensures`, and `invariant` clauses.
- **Runtime guards**: Every contract clause emits a guard block in LLVM IR. On violation → `@llvm.trap()`.
- **`--verify`**: Generates SMT-LIB 2.6 output for offline Z3 verification (from `xiom-verify` crate).
- **`--no-contracts`**: Disables runtime checks for production builds.

### PRECISE — One canonical way to write each thing
- **LL(1) recursive descent parser** with no backtracking.
- **No implicit coercions.** No type widening, silent truncation, or integer conversion.
- **Exhaustive matching** enforced on `match` statements.
- **`derive`** generates boilerplate (`Eq`, `Clone`, `Display`, `Hash`, `Ord`) mechanically.
- **Interface dispatch** via exhaustive monomorphisation — no vtable overhead, deterministic resolution.

---

## 2. Pipeline Architecture (Unchanged from v0.33.0)

```
Source → Lexer → Token Stream → Parser → AST → Checker → Borrow Checker
→ ModuleCatalog → Codegen (LLVM IR) → clang → Binary

Checker → Verify (SMT Generator, --verify only) → SMT-LIB 2.6
```

---

## 3. What Changed: v0.33.0 → v0.49.8

### 3.1 Phase 5a — Codegen Hardening (ALL COMPLETE ✅)

| Item | Description |
|------|-------------|
| **ARC A Pointers** | Real pointer/reference types (`*T`, `&mut T`) — 6-step implementation: type encoding, deref read/write, address-of, call-site coercion, param binding |
| **Const-Generics** | `type_from_ast` handles `Type::Array`, `llvm_type_for` resolves const values, monomorphisation infra (`const_value_map`, `Type::Array` substitution) |
| **Match Type Unification** | `infer_match_llvm_type` collects types from ALL arms, picks widest (struct > ptr > i64) |
| **Array Literal Indexing** | `Expr::Index` recognizes `Expr::Array` buffers (i8* with length slot), skips slot, reads at index+1 |
| **Interface Dispatch** | Exhaustive monomorphisation: `scan_interface_impls` builds concrete→interface map, call-site inference resolves interface params to concrete struct types |
| **`i64 → struct` coercion** | `coerce_value` inttoptr+load for heap-pointer round-trip (Option.unwrap struct payloads) |
| **`&mut self` receiver** | `is_mut_self` on `Param`, pointer passing in codegen, field access via loaded pointer |
| **Str builtins** | `c_str()` (i8* identity), `len()`/`byte_len()` (strlen call) |
| **If-expression codegen** | `Expr::If` emits conditional branches with result alloca (was sequential-only stub) |
| **Leaf-module keys** | Generic monomorphisation disambiguation (`mem.replace_Int` vs `ptr.replace_Int`) |

### 3.2 Phase 5b — Stdlib Completion (ALL COMPLETE ✅)

| Area | Changes |
|------|---------|
| **io.xi** | 31 types/functions made `pub`; `write_file`/`read_file`/`file_exists`/`remove_file` verified |
| **path.xi** | `canonicalize()` implemented (string-based `./..` + separator normalization); 14 functions work |
| **crypto.xi** | `aes_decrypt` software fallback fixed; SHA-256 known-vectors verified |
| **async.xi** | `Channel.send/recv/try_recv` with `&mut self`; `Executor.new` + `sleep_ms` |
| **sync.xi** | `AtomicInt` + `AtomicBool` store/load (if-expr fix) |
| **thread.xi** | `available_parallelism`, `sleep_ms`, `yield_now`, `current_thread_id`, `Thread.current` |
| **mem.xi** | `swap[Int]`/`replace[Int]` with leaf-key monomorphisation |
| **test.xi** | `test.assert` invocation verified |
| **All 41 smoke tests** | 0 ignored; production-grade tests for thread, async, sync, crypto, path, io |

### 3.3 Resolved Bugs (All 10)

| Bug | Fix |
|-----|-----|
| BUG-001 SHA-256 | C reference via FFI |
| BUG-002 async paths | Parser `fn()` type args + contextual `async` |
| BUG-003 AtomicBool | `Expr::If` conditional branches |
| BUG-005 mem.replace | Leaf-module key registration |
| BUG-006 Option[Struct].unwrap | `i64 → struct` coercion |
| BUG-007 Interface dispatch | Exhaustive monomorphisation |
| BUG-008 IO string coercion | `Str.c_str()` builtin |
| BUG-009 TestResult | Same as BUG-006 |
| BUG-010 Channel send/recv | `&mut self` struct receiver |

---

## 4. Phase 5c — ARCHITECTURAL FEATURES & SAFETY HARDENING

Phase 5c focuses on making the compiler production-grade for safety-critical systems. This includes structural improvements, safety feature implementation, and compiler robustness.

### 4.1 Safety Features (New for 5c)

Based on the XIOM Safety Philosophy:
> `unsafe` transfers responsibility to the programmer. Contracts remain the best tool to make unsafe code safe. The compiler should never be the source of undefined behavior.

#### 4.1a. `#[safety_audit]` Attribute

Marks functions containing `unsafe` blocks for mandatory code review. The compiler emits diagnostic warnings listing all audited unsafe sites.

```xiom
#[safety_audit(justification: "Required for C FFI — validated ptr before call")]
fn safe_wrapper(ptr: *Int) -> Int
  requires: !ptr.is_null()
{
  unsafe { return *ptr; }
}
```

**Compiler behavior:**
- Emits `safety_audit` notice in diagnostics (JSON format for tooling)
- In `--strict` mode: requires `justification` to be non-empty, else compile error
- Integrated with `--dump-contracts` to export safety audit index

#### 4.1b. `--strict` Compiler Mode

Enforces maximum safety constraints:
| Constraint | Default | `--strict` |
|-----------|---------|------------|
| Unsafe blocks without `#[safety_audit]` | Warning | Error |
| Large unsafe blocks (>10 lines) | Silent | Warning |
| Missing contract on public `unsafe`-containing functions | Silent | Warning |
| Unknown type default to `i64` | Allowed | Error |
| Derive on types without `#[derive]` | Allowed | Error |

#### 4.1c. Compiler Robustness (from V5/V6/V7 audits)

| Issue | Status in v0.49.8 | 5c Target |
|-------|------------------|-----------|
| Recursion depth limit | ✅ Fixed (500) | Raise to configurable via `--max-depth N` |
| Generic mono loop guard | ✅ Fixed (65536) | Lowered to 1000 with clear error message |
| Div-by-zero guard | ✅ Fixed | No change |
| Unknown types → i64 silently | ⚠️ Mitigated (type errors abort) | `--strict` mode: error on unknown types |
| Vec reallocation (V1) | ✅ Fixed (doubling strategy) | No change |
| C runtime limits (V5) | ⚠️ 16 fields / 64 locals | Raise to 256 fields / 1024 locals |
| Error recovery | ❌ First error terminates | Collect up to 100 errors before abort |
| Compiler timeout | ❌ None | `--timeout N` flag (seconds) |

#### 4.1d. AI Mode (`--ai`) Enhancements

The `--ai` flag produces machine-readable JSON diagnostics for IDE/tooling integration. Phase 5c adds:

1. **Structured prompts**: Each diagnostic includes a `suggestion` field with a static, curated message about best practices, contract recommendations, ownership fixes, etc.
2. **Safety hints**: When `unsafe` is used, the JSON includes `safety_hint` with relevant guideline references.
3. **Contract suggestions**: When a function lacks contracts, suggest appropriate `requires`/`ensures` based on the function's signature and body.
4. **Hard-coded templates**: Suggestions stored in `templates/diagnostics/` as JSON files — easy to update without recompiling the compiler.

```json
{
  "severity": "warning",
  "code": "XIOM_SAFETY_001",
  "message": "unsafe block without safety_audit attribute",
  "span": { "file": "src/main.xi", "line": 42, "col": 5 },
  "suggestion": "Add #[safety_audit(justification: \"...\")] to document why this unsafe block is necessary.",
  "safety_hint": "See docs/UNSAFE_GUIDELINES.md §2.A — Isolation"
}
```

#### 4.1e. Contract Static Verification (Phase 3 Preview)

While full Z3 integration is a Phase 3 deliverable, Phase 5c adds foundational improvements:

1. **`@pre` snapshot model**: Store entry-point values for `ensures` clauses referencing `@pre` expressions. Currently `@pre` compares against runtime value at check point — 5c captures a snapshot at function entry.
2. **`--verify-all` flag**: Generates SMT-LIB for ALL contracted functions (not just `--verify` for the target).
3. **Contract coverage report**: `--dump-contracts` now includes call-site tracking — which contracts are exercised by which callers.

### 4.2 Structural Improvements

#### 4.2a. Derive Macro Codegen

Extend the `compile_derive_impls` pass to support:
- `derive[Default]` — zero-initialization for all fields
- `derive[Drop]` — sequential field drop (for types with `Drop` implementing fields)
- `derive[Serialize]` / `derive[Deserialize]` — JSON round-trip (leverages existing `serialize.xi`)
- Proper handling of generic types in derive (currently limited to concrete types)

#### 4.2b. Borrow Checker Struct-Field Borrows

Currently, the borrow checker tracks whole-struct borrows only. `x.field` and `y.field` on the same struct `s` are treated as conflicting borrows if one is mutable. Phase 5c adds field-level granularity:

```xiom
var s = Point{ x: 1, y: 2 };
let rx = &s.x;    // read borrow on field x
let ry = &s.y;    // read borrow on field y — OK (different field)
s.x = 10;         // ERROR: mutation while rx is active
```

#### 4.2c. Enhanced Smoke Tests (Rating 3-5/5)

Current smoke tests provide basic coverage (compile + run checks). Phase 5c upgrades each to production-grade:

| Module | Current | Target |
|--------|---------|--------|
| path | 5/10 checks | 15+ checks (all public fns) |
| io | 5/10 checks | 12+ checks (write/read/remove/dir ops) |
| sync | 3/10 checks | 10+ checks (Atomic ops, Once, Mutex) |
| thread | 6/10 checks | 8+ checks |
| crypto | 3/10 checks | 8+ checks (SHA-256, AES, HMAC) |
| collections | 2/10 checks | 10+ checks (Vec push/pop/index, Map ops) |
| All others | Rating 1-2/5 | Rating 3-5/5 |

#### 4.2d. Compiler Resilience

| Feature | Description |
|---------|-------------|
| **Error recovery** | Parser collects up to 100 errors before aborting. Enables fixing multiple issues per compile cycle. |
| **`--timeout N`** | Compiler terminates after N seconds (prevents infinite loops on malformed input). |
| **`--max-depth N`** | Configurable recursion depth limit (default 500, max 10000). |
| **Memory monitoring** | `--diagnostics=json` includes peak memory usage in the final JSON report. |
| **LLVM IR verification** | Run `opt -verify` pass on generated IR before emitting (catches malformed IR early). |

---

## 5. Phase 5c Implementation Order

| Priority | Task | Effort | Dependencies |
|----------|------|--------|-------------|
| **P0** | `--strict` mode + `#[safety_audit]` attribute | Small | None |
| **P0** | C runtime limits → 256 fields / 1024 locals | Small | None |
| **P0** | LLVM IR verification (`opt -verify`) | Small | None |
| **P1** | AI mode (`--ai`) JSON diagnostics with static prompts | Medium | Template system |
| **P1** | Error recovery (collect 100 errors) | Medium | Parser changes |
| **P1** | `--timeout N` and `--max-depth N` flags | Small | None |
| **P2** | Derive macro codegen (Default, Drop, Serialize) | Medium | codegen impl |
| **P2** | Borrow checker struct-field borrows | Large | checker changes |
| **P2** | Contract `@pre` snapshot model | Medium | codegen + checker |
| **P2** | Enhanced smoke tests (3-5/5 rating) | Medium | Per-module test writing |
| **P3** | `--verify-all` + contract coverage | Medium | xiom-verify crate |
| **P3** | Memory monitoring in diagnostics | Small | None |

---

## 6. Updated Crate Reference

| File | Lines (v0.33) | Lines (v0.49.8) | Growth |
|------|---------------|-----------------|--------|
| `crates/xiom-ast/src/lib.rs` | 496 | 533 | +37 |
| `crates/xiom-lexer/src/lib.rs` | 503 | 503 | — |
| `crates/xiom-parser/src/lib.rs` | 1931 | 1362 | -569 (refactored) |
| `crates/xiom-check/src/lib.rs` | 3105 | 4116 | +1011 |
| `crates/xiom-codegen/src/lib.rs` | 3910 | ~8093 | +4183 |
| `crates/xiom-graph/src/lib.rs` | — | new | new |
| `crates/xiom-verify/src/lib.rs` | 218 | 218 | — |
| `crates/xiom/src/main.rs` | 968 | 968 | — |

**Key growth areas:**
- `xiom-check`: interface dispatch (`interfaces` field, `register_interface_decl`, `allow_interface_dispatch`)
- `xiom-codegen`: `&mut self` receiver, if-expression branching, array indexing, leaf-module keys, `i64 → struct` coercion, interface monomorphisation, const-generics infra, deferred struct types

---

## 7. Current Gates (v0.49.8)

| Gate | Count | Status |
|------|-------|--------|
| Parser tests | 47/47 | ✅ |
| Checker tests | 74/74 | ✅ |
| Stdlib execution (smoke) | 41/41 (0 ignored) | ✅ |
| E2E tests | 85/85 | ✅ |
| Feature regression | 48/48 | ✅ |
| Integration regression | 119/119 | ✅ |
| Other regression | 100 combined | ✅ |

---

## Appendix A: Crate Dependency Graph (Unchanged)

```
xiom → xiom-ast, xiom-lexer, xiom-parser, xiom-check, xiom-codegen, xiom-graph, xiom-verify
xiom-parser → xiom-ast, xiom-lexer
xiom-check → xiom-ast, xiom-lexer, xiom-parser
xiom-codegen → xiom-ast
xiom-graph → xiom-ast
xiom-verify → xiom-ast
```

## Appendix B: Archival Documents

| Document | Purpose |
|----------|---------|
| `docs/ROADMAP.md` | Phase-by-phase roadmap with bug tracking |
| `docs/SESSION.md` | Session handoff — current state and carry-on prompt |
| `docs/ARC_A_POINTERS.md` | Design reference for pointer/reference type implementation |
| `docs/PRODUCTION_HARDENING_BUGS.md` | Deep-dive on all 10 resolved bugs |
| `docs/UNSAFE_GUIDELINES.md` | Rules for using `unsafe` responsibly |
| `docs/SECURITY_BEST_PRACTICES.md` | Security guidelines for XIOM users (Phase 5c) |
