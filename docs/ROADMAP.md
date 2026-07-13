# XIOM Compiler — Production Roadmap

**Current:** v0.33.0 "Phase 4" — 37/37 stdlib, 84/84 e2e, all gates green  
**Branch:** `feat/guardian`  
**Target:** v1.0.0 self-hosting compiler

---

## 1. CURRENT STATE (2026-07-13)

| Gate | Count | Status |
|------|-------|--------|
| Stdlib execution | 37/37 strict + 4 ignored | ✅ |
| E2E tests | 84/84 | ✅ |
| Compiler gaps | 14/14 closed | ✅ |
| Regression: diff | 25 | ✅ |
| Regression: feat | 48 | ✅ |
| Regression: fulldiff | 23 | ✅ |
| Regression: fuzz | 23 + 1 ignored | ✅ |
| Regression: integration | 119 | ✅ |
| Regression: robustness | 29 | ✅ |
| Regression: check | 74 | ✅ |
| Regression: parser | 47 | ✅ |

### Delivered Compiler Fixes

| Category | Fixes |
|----------|-------|
| **Type System** | Map injection from PRIMITIVES, `generic_type_names` fallback |
| **Memory Safety** | GEP sizeof for heap alloc, array alloca reuse, 4-field Vec elem_size |
| **Match Pipeline** | Some/None/Ok/Err dispatch, inner literal checks, OR patterns, payload binding |
| **Method Dispatch** | i64→struct coercion, bare-call guard, zero_val_for null, receiver params |
| **Vec Infrastructure** | emit_elem_store/emit_elem_load with runtime width dispatch |
| **Stdlib** | Path self-contained, io pub exports, byte_at, SHA-256 C FFI |
| **Parser** | async contextual keyword (type paths work) |

### Resolved Bugs

| Bug | Status | Fix |
|-----|--------|-----|
| **SHA-256 wrong hash** | ✅ RESOLVED | C reference implementation via FFI (`sha256_sw.c`) |
| **IO functions not pub** | ✅ RESOLVED | `io.print/println/args` → `pub` |
| **Vec[UInt8] elem_size** | ✅ IMPLEMENTED | 4-field Vec with runtime elem dispatch |

### Known Issues

| Issue | Severity | Type |
|-------|----------|------|
| async expression paths (`read_module_header` + checker bridge) | MEDIUM | Checker bug |
| Interface/trait dispatch | MEDIUM | Feature gap |
| Derive macro codegen | MEDIUM | Feature gap |
| Const-generic N propagation | MEDIUM | Feature gap |

---

## 2. CANONICAL PHASE SYSTEM

| Phase | Codename | Version | Status |
|-------|----------|---------|--------|
| 0 | Pipeline | v0.1.0 | ✅ |
| 1 | Guardian | v0.2.0–v0.30.0 | ✅ |
| 2 | Hardened | v0.31.0 | ✅ |
| 3 | ARC-C | v0.32.0 | ✅ |
| 4 | Or-Patterns | v0.33.0 | ✅ |
| 5a | Codegen Hardening | current | In progress |
| 5b | Stdlib Completion | next | Planned |
| 5c | Architectural Features | later | Planned |
| 5d | Production Toolchain | later | Planned |
| 5e | Self-Hosting | v1.0.0 | Planned |

---

## 3. PHASE 5a — CODEGEN HARDENING

### 5a.1. ARC A: Real Pointer/Reference Types
**Design doc:** `docs/ARC_A_POINTERS.md`  
**Status:** Design complete (2026-07-09), implementation pending

`*T`/`&mut T` params are lowered as by-value `i64` instead of real pointers. Only scalar pointers are broken; `&Struct`/`&Vec` work by-value-carrying. Fix involves:
1. `type_from_ast` encoding: `*T` → `"*" + inner`
2. Deref read/write through real pointers
3. `from_ref`/`from_mut` address-of
4. `&x` at scalar-ref call sites
5. Param binding for pointer params
6. Gate check at each step

### 5a.2. Map Type Injection
**Status:** FIXED — Map removed from checker PRIMITIVES + generic_type_names search

### 5a.3. Fixed-Size Array Allocation
**Status:** FIXED — alloca reuse for Ident containers in loop bodies

### 5a.4. Option.unwrap() + Payload Binding
**Status:** FIXED — inline unwrap/unwrap_err, match dispatch, OR patterns

### 5a.5. Const-Generics: `[N]T` Arrays
**File:** `crates/xiom-check/src/lib.rs`, `crates/xiom-codegen/src/lib.rs`
**Status:** Partial — `type_from_ast` handles `Type::Array`, full `N` propagation needed

### 5a.6. Match Expression Type Unification
**File:** `crates/xiom-codegen/src/lib.rs:6275`
**Status:** TODO — `TODO(match-expr-typing)`. First arm only; heterogeneous arms mismatch.

### 5a.7. Expr::Array Indexing on Literal Arrays
**File:** `crates/xiom-codegen/src/lib.rs:5884`
**Status:** TODO — `TAIL-TODO`. Depends on 5a.5.

### 5a.8. Interface/Trait Object Dispatch
**File:** `crates/xiom-check/src/lib.rs`, `crates/xiom-codegen/src/lib.rs`
**Status:** TODO — needs monomorphization-time interface resolution or vtables

---

## 4. PHASE 5b — STDLIB COMPLETION

| Item | Status |
|------|--------|
| Debug/fix regex engine logic | Done (stdlib logic fine, was dispatch bug) |
| C runtime FFI for crypto | Done (SHA-256 via FFI) |
| Path module (extension, join, normalize) | Partial — `file_name`, `extension`, `file_stem`, `parent`, `is_absolute` done |
| `byte_at`/`substr` on Str | Done — `byte_at` added |
| Thread runtime support | Smoke passes |
| IO runtime support | Smoke passes (pub exports fixed) |
| Async module expression paths | OPEN — checker bug |
| Test module | Smoke passes |

---

## 5. PHASE 5c — ARCHITECTURAL FEATURES

| Item | Status |
|------|--------|
| Const-generic N value propagation | TODO |
| Interface dispatch (vtable or monomorphization) | TODO |
| Derive macro codegen | TODO |
| Borrow checker struct-field borrows | TODO |

---

## 6. PHASE 5d — PRODUCTION TOOLCHAIN

| Item | Status |
|------|--------|
| Package manager + registry | TODO |
| CLI toolchain (xiom build/run/test/bench) | Partial (`--run`, `--emit-ir` exist) |
| Cross-platform CI | TODO |
| External C FFI binding generator | TODO |

---

## 7. PHASE 5e — SELF-HOSTING (v1.0.0)

| Item | Status |
|------|--------|
| Z3 static verification | TODO |
| Write compiler in XIOM | TODO |
| Bootstrap with byte-for-byte identical output | TODO |

---

## 8. VERIFICATION PROTOCOL

```bash
cargo build -p xiomc
cargo test -p xiom-codegen --test stdlib_execution_tests -- --nocapture
cargo test -p xiom-codegen --test e2e_tests
cargo test -p xiom-codegen  # all regression gates
```

---

## 9. APPENDIX: Archival Documents

| Document | Status | Content |
|----------|--------|---------|
| `ARC_A_POINTERS.md` | **Active** — design reference for §5a.1 | Blast radius, 6-step implementation, e2e tests |
| `COMPILER_VERSIONS.md` | **Active** — canonical version log | Complete v0.1.0→v0.33.0 history |
| `PRODUCTION_HARDENING_BUGS.md` | **Active** — bug deep-dives | BUG-001 SHA-256 round-by-round, BUG-002 async investigation |
| `PRODUCTION_READINESS_PLAN.md` | **Superseded** by this document | Root cause chains absorbed into §3 |
| `CODEGEN_PRODUCTION_PLAN.md` | **Superseded** by this document | 4-phase plan absorbed into §5a-5e |
| `CODEGEN_TIER2.md` | **Superseded** by this document | T2 error taxonomy available for diagnostics |
