# XIOM Compiler — Production Roadmap

**Current:** v0.45.2 "Phase 5b" — 37/37 smoke, 84/84 e2e, 47/47 parser, 74/74 checker, all gates green
**Branch:** `feat/guardian`
**Target:** v1.0.0 self-hosting compiler

---

## 1. CURRENT STATE (2026-07-14)

| Gate | Count | Status |
|------|-------|--------|
| Parser tests | 47/47 | ✅ |
| Checker tests | 74/74 | ✅ |
| Stdlib execution (smoke) | 37/37 strict + 4 ignored | ✅ |
| E2E tests | 84/84 | ✅ |
| Feature regression | 48/48 | ✅ |
| Integration regression | 119 | ✅ |
| All other regression gates | 25 diff, 23 fulldiff, 23 fuzz, 29 robustness | ✅ |

### Resolved Bugs

| Bug | Status | Fix |
|-----|--------|-----|
| **BUG-001** SHA-256 wrong hash | ✅ RESOLVED | C reference implementation via FFI (`sha256_sw.c`) |
| **BUG-002** async expression paths | ✅ RESOLVED | Parser: `fn()` as type arg in `[T]` brackets + contextual `async` routing |

### Active Bugs

| Bug | Severity | Symptom |
|-----|----------|---------|
| **BUG-003** sync AtomicBool bad IR | HIGH | `icmp eq i64* %tmp14, 2` — pointer-compared-to-int LLVM IR error |
| **BUG-004** path canonicalize crash | HIGH | Access violation 0xC0000005 at runtime |
| **BUG-005** mem swap/replace crash | ✅ RESOLVED | Monomorphisation naming collision — leaf-module key registration |
| **BUG-006** Option[Struct].unwrap() | MEDIUM | Heap-allocated struct payload round-trip; needs per-type Option layout |

---

## 2. CANONICAL PHASE SYSTEM

| Phase | Codename | Version | Status |
|-------|----------|---------|--------|
| 0 | Pipeline | v0.1.0 | ✅ |
| 1 | Guardian | v0.2.0–v0.30.0 | ✅ |
| 2 | Hardened | v0.31.0 | ✅ |
| 3 | ARC-C | v0.32.0 | ✅ |
| 4 | Or-Patterns | v0.33.0 | ✅ |
| 5a | Codegen Hardening | v0.45.x | ✅ Complete (2026-07-14) |
| 5b | Stdlib Completion | v0.45.x | ✅ Substantially Complete |
| 5c | Architectural Features | next | Planned |
| 5d | Production Toolchain | later | Planned |
| 5e | Self-Hosting | v1.0.0 | Planned |

---

## 3. PHASE 5a — CODEGEN HARDENING ✅

### 5a.1. ARC A: Real Pointer/Reference Types — ✅ DONE
**Design doc:** `docs/ARC_A_POINTERS.md`
All 6 implementation steps verified:
1. `type_from_ast` `*T` → `"*" + inner` encoding + `llvm_type_for` decode
2. Deref read/write through real pointers (`*p`, `*p = v`)
3. `ptr.from_ref`/`ptr.from_mut` address-of
4. `&x`/`&mut x` address-of at scalar-ref call sites (`coerce_arg_for_param`)
5. Param binding for pointer params (compile_fn + generic-mono)
6. e2e tests: `ptr_deref.xi`, `ref_mut_param.xi`, `mut_ref_swap.xi`

### 5a.2. Map Type Injection — ✅ DONE
Map removed from checker PRIMITIVES + generic_type_names search

### 5a.3. Fixed-Size Array Allocation — ✅ DONE
alloca reuse for Ident containers in loop bodies

### 5a.4. Option.unwrap() + Payload Binding — ✅ DONE
inline unwrap/unwrap_err, match dispatch, OR patterns, inner literal checks

### 5a.5. Const-Generics: `[N]T` Arrays — ⚠️ PARTIAL (80%)
| Complete | Remaining |
|----------|-----------|
| `type_from_ast` handles `Type::Array` with ident + element types | End-to-end monomorphisation of const-generic calls (e.g. `len[Int, 5](arr)`) untested |
| `llvm_type_for` resolves const-declared values from `self.constants` | N value extraction from type args not verified through all call-site patterns |
| Monomorphisation infra: `const_value_map`, `Type::Array` substitution in `subst_type` | |

### 5a.6. Match Expression Type Unification — ✅ DONE
`infer_match_llvm_type` collects types from ALL arms, picks widest (struct > ptr > i64).
TODO marker removed. `coerce_value` handles per-arm conversions.

### 5a.7. Expr::Array Indexing on Literal Arrays — ✅ DONE
`Expr::Index` recognizes `Expr::Array` i8* buffers, skips leading length slot,
reads element at position index+1. TAIL-TODO removed.

### 5a.8. Interface/Trait Object Dispatch — ⚠️ PARTIAL (85%)
| Complete | Remaining |
|----------|-----------|
| Checker: `interfaces` field, `register_interface_decl` pass | Vtable for multi-implementation polymorphic dispatch |
| Interface method dispatch on typed receivers (`self.description()`) | First-match-wins is deterministic but fragile for 1:N interfaces |
| Wildcard chains (`opt.value.description()`) | Pattern-bound generic types in match arms (from `Err(e)`) use cascade Error type |
| Generic param dispatch (`T: Foo` → `T.bar()`) | |
| Codegen fallback: searches concrete types for `*.method_name` | |
| `error.xi` compiles 0 errors (was 9) | |
| Str + Str concatenation type-checks in BinOp::Add | |
| Option/Result `.value`/`.error` pseudo-fields → `_` for interface chain support | |

---

## 4. PHASE 5b — STDLIB COMPLETION (In Progress)

| Item | Status | Notes |
|------|--------|-------|
| Debug/fix regex engine logic | ✅ DONE | Stdlib logic fine, was dispatch bug |
| C runtime FFI for crypto | ✅ DONE | SHA-256 via FFI, `aes_decrypt` software fallback fixed |
| Path module | ⚠️ PARTIAL | `file_name`, `extension`, `file_stem`, `parent`, `is_absolute`, `canonicalize`, `join` implemented. `canonicalize()` crashes at runtime (BUG-004) |
| `byte_at`/`substr` on Str | ✅ DONE | `byte_at` added |
| Thread runtime support | ⚠️ SMOKE ONLY | Smoke passes. Enhanced test needed |
| IO runtime support | ⚠️ SMOKE ONLY | Pub exports fixed (31 types/fns). Enhanced smoke test crashes on some paths |
| Async module expression paths | ✅ DONE | Parser + checker fix. `async.Executor.new()` compiles and runs |
| Test module | ⚠️ SMOKE ONLY | `#[ignore]` — basic assert runs |
| Crypto known-vector tests | ✅ DONE | SHA-256 empty string + "abc" known vectors verified |

### Phase 5b Active Bugs

| Bug | Module | Symptom | Root Cause |
|-----|--------|---------|------------|
| **BUG-003** | sync.xi | `icmp eq i64* %tmp14, 2` — invalid LLVM IR | AtomicBool lowered to pointer type instead of integer |
| **BUG-004** | path.xi | Access violation in `canonicalize()` | String-based implementation has memory bug in component manipulation |
| **BUG-005** | mem.xi | Access violation in `swap[Int]` / `replace[Int]` | `[T]` type args in expression context produce broken pointer IR |

---

## 5. PHASE 5c — ARCHITECTURAL FEATURES

| Item | Status |
|------|--------|
| Const-generic N value propagation (monomorphisation e2e) | ⚠️ PARTIAL — infra in place, untested e2e |
| Interface dispatch (vtable or exhaustive monomorphisation) | ⚠️ PARTIAL — static dispatch works, vtable needed |
| Derive macro codegen | TODO |
| Borrow checker struct-field borrows | TODO |
| Enhanced smoke tests (rating 3-5/5 for all modules) | TODO |

---

## 6. PHASE 5d — PRODUCTION TOOLCHAIN

| Item | Status |
|------|--------|
| Package manager + registry | TODO |
| CLI toolchain (xiom build/run/test/bench) | Partial (`--run`, `--emit-ir`, `-o` exist) |
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
cargo test -p xiom-parser --lib
cargo test -p xiom-check --lib
cargo test -p xiom-codegen --test stdlib_execution_tests -- --nocapture
cargo test -p xiom-codegen --test e2e_tests
cargo test -p xiom-codegen  # all regression gates
```

---

## 9. APPENDIX: Archival Documents

| Document | Status | Content |
|----------|--------|---------|
| `ARC_A_POINTERS.md` | **Active** — design reference for §5a.1 | Blast radius, 6-step implementation, e2e tests |
| `COMPILER_VERSIONS.md` | **Active** — canonical version log | Complete v0.1.0→v0.45.2 history |
| `PRODUCTION_HARDENING_BUGS.md` | **Active** — bug deep-dives | BUG-001 SHA-256, BUG-002 async, BUG-003/BUG-004/BUG-005 added |
| `PRODUCTION_READINESS_PLAN.md` | **Superseded** by this document | Root cause chains absorbed into §3 |
| `CODEGEN_PRODUCTION_PLAN.md` | **Superseded** by this document | 4-phase plan absorbed into §5a-5e |
| `CODEGEN_TIER2.md` | **Superseded** by this document | T2 error taxonomy available for diagnostics |
